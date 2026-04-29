use crate::app::ClipboardHistoryApp;
use crate::clipboard::{CaptureOutcome, ClipboardController, ClipboardPort, ClipboardSnapshot};
use crate::desktop::paths;
use crate::domain::{ClipboardContent, HistoryItem};
use crate::paste::{PastePort, SelectionController, SelectionOutcome, SelectionSource};
use crate::ui::{PopupAction, PopupCommand, PopupState};
use gtk4::gdk;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APP_ID: &str = "io.github.gabriel9m.LinuxClipboard";
const DEBUG_LOG_PATH: &str = "clipboard-history-debug.log";

thread_local! {
    static GTK_APP_STATE: RefCell<Option<GtkAppState>> = const { RefCell::new(None) };
}

pub fn run_application() {
    debug_log("run_application: starting GTK application");
    let app = gtk4::Application::builder().application_id(APP_ID).build();

    app.connect_shutdown(|_| {
        debug_log("application: shutdown");
        GTK_APP_STATE.with(|state| {
            *state.borrow_mut() = None;
        });
    });

    app.connect_activate(|app| {
        debug_log("application: activate");
        let hold_guard = app.hold();
        debug_log("application: hold acquired");
        let desktop_paths = paths::default_paths();
        debug_log(&format!(
            "paths: history_path={} images_dir={}",
            desktop_paths.history_path.display(),
            desktop_paths.images_dir.display()
        ));
        let history_app = Rc::new(RefCell::new(ClipboardHistoryApp::load_with_paths(
            &desktop_paths.history_path,
            &desktop_paths.images_dir,
        )));
        debug_log(&format!(
            "history: loaded {} item(s)",
            history_app.borrow().history().items().len()
        ));
        let clipboard_controller = Rc::new(RefCell::new(ClipboardController::new()));

        let popup = GtkPopup::new(
            app,
            history_app.borrow().history().items().to_vec(),
            Rc::clone(&clipboard_controller),
        );
        let popup_view = popup.view_handle();

        if let Err(error) = start_text_clipboard_monitor(
            Rc::clone(&history_app),
            Rc::clone(&clipboard_controller),
            popup_view,
        ) {
            log_error(&format!("clipboard monitor unavailable: {error}"));
        }

        GTK_APP_STATE.with(|state| {
            *state.borrow_mut() = Some(GtkAppState {
                _popup: popup,
                _hold_guard: hold_guard,
            });
        });
        debug_log("popup: stored in thread-local state");
        GTK_APP_STATE.with(|state| {
            if let Some(state) = state.borrow().as_ref() {
                state._popup.show();
            } else {
                log_error("popup: missing from thread-local state before show");
            }
        });
    });

    let exit_code = app.run();
    debug_log(&format!("run_application: exited with {exit_code:?}"));
}

#[derive(Debug)]
struct GtkAppState {
    _popup: GtkPopup,
    _hold_guard: gtk4::gio::ApplicationHoldGuard,
}

#[derive(Debug)]
pub struct GtkPopup {
    window: gtk4::ApplicationWindow,
    view: GtkPopupView,
}

impl GtkPopup {
    pub fn new(
        app: &gtk4::Application,
        items: Vec<HistoryItem>,
        clipboard_controller: Rc<RefCell<ClipboardController>>,
    ) -> Self {
        debug_log(&format!("popup: constructing with {} item(s)", items.len()));
        let state = Rc::new(RefCell::new(PopupState::from_items(&items)));
        let items = Rc::new(RefCell::new(items));
        let selection_runtime = GtkSelectionRuntime::new(clipboard_controller)
            .map(|runtime| Rc::new(RefCell::new(runtime)))
            .map_err(|error| {
                eprintln!("selection runtime unavailable: {error}");
                error
            })
            .ok();
        let list_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(8)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        render_popup(&list_box, &items.borrow(), &state.borrow());

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("LinuxClipboard")
            .default_width(420)
            .default_height(260)
            .resizable(false)
            .child(&list_box)
            .build();

        window.connect_show(|_| {
            debug_log("window: show signal");
        });
        window.connect_hide(|_| {
            debug_log("window: hide signal");
        });
        let app_for_close = app.clone();
        window.connect_close_request(move |_| {
            debug_log("window: close-request signal");
            debug_log("application: quit after close-request");
            app_for_close.quit();
            gtk4::glib::Propagation::Proceed
        });
        schedule_window_diagnostics(&window);

        let key_controller = gtk4::EventControllerKey::new();
        {
            let state = Rc::clone(&state);
            let items = Rc::clone(&items);
            let list_box = list_box.clone();
            let window = window.clone();
            let selection_runtime = selection_runtime.clone();

            key_controller.connect_key_pressed(move |_, key, _, _| {
                debug_log(&format!("keyboard: key pressed {key:?}"));
                let command = match key {
                    gdk::Key::Up => Some(PopupCommand::MoveUp),
                    gdk::Key::Down => Some(PopupCommand::MoveDown),
                    gdk::Key::Return | gdk::Key::KP_Enter => Some(PopupCommand::Enter),
                    gdk::Key::Escape => Some(PopupCommand::Escape),
                    _ => None,
                };

                let Some(command) = command else {
                    debug_log("keyboard: ignored key");
                    return gtk4::glib::Propagation::Proceed;
                };

                debug_log(&format!("popup: handling command {command:?}"));
                let action = state.borrow_mut().handle_command(command);
                handle_popup_action(&window, action, &items.borrow(), selection_runtime.as_ref());
                render_popup(&list_box, &items.borrow(), &state.borrow());

                gtk4::glib::Propagation::Stop
            });
        }
        window.add_controller(key_controller);

        let view = GtkPopupView {
            items,
            state,
            list_box,
        };

        Self { window, view }
    }

    pub fn show(&self) {
        debug_log("popup: present requested");
        self.window.present();
    }

    pub fn view_handle(&self) -> GtkPopupView {
        self.view.clone()
    }
}

fn schedule_window_diagnostics(window: &gtk4::ApplicationWindow) {
    let window = window.clone();
    let mut tick = 0;

    gtk4::glib::timeout_add_local(Duration::from_millis(250), move || {
        tick += 1;
        debug_log(&format!(
            "window diagnostic #{tick}: visible={} mapped={} active={} default_width={} default_height={}",
            window.is_visible(),
            window.is_mapped(),
            window.is_active(),
            window.default_width(),
            window.default_height()
        ));

        if tick >= 16 {
            gtk4::glib::ControlFlow::Break
        } else {
            gtk4::glib::ControlFlow::Continue
        }
    });
}

#[derive(Debug, Clone)]
pub struct GtkPopupView {
    items: Rc<RefCell<Vec<HistoryItem>>>,
    state: Rc<RefCell<PopupState>>,
    list_box: gtk4::Box,
}

impl GtkPopupView {
    pub fn refresh_from_history(&self, items: Vec<HistoryItem>) {
        debug_log(&format!("popup: refreshing with {} item(s)", items.len()));
        *self.items.borrow_mut() = items;
        *self.state.borrow_mut() = PopupState::from_items(&self.items.borrow());
        render_popup(&self.list_box, &self.items.borrow(), &self.state.borrow());
    }
}

fn render_popup(container: &gtk4::Box, items: &[HistoryItem], state: &PopupState) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    if let Some(message) = state.empty_message() {
        container.append(
            &gtk4::Label::builder()
                .label(message)
                .xalign(0.0)
                .wrap(true)
                .build(),
        );
        return;
    }

    for (index, item) in items.iter().enumerate() {
        let row = gtk4::Label::builder()
            .label(&item.preview)
            .xalign(0.0)
            .wrap(false)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(8)
            .margin_end(8)
            .build();

        if state.selected_index() == Some(index) {
            row.add_css_class("accent");
        }

        container.append(&row);
    }
}

fn handle_popup_action(
    window: &gtk4::ApplicationWindow,
    action: PopupAction,
    items: &[HistoryItem],
    selection_runtime: Option<&Rc<RefCell<GtkSelectionRuntime>>>,
) {
    match action {
        PopupAction::None => {}
        PopupAction::Activate { index, source } => {
            debug_log(&format!(
                "popup: activate requested index={index} source={source:?}"
            ));
            let Some(item) = items.get(index) else {
                log_error(&format!(
                    "popup activation ignored: index out of range: {index}"
                ));
                return;
            };

            let Some(selection_runtime) = selection_runtime else {
                log_error("popup activation ignored: selection runtime unavailable");
                return;
            };

            match selection_runtime.borrow_mut().activate(source, item) {
                Ok(outcome) => debug_log(&format!("popup activation outcome: {outcome:?}")),
                Err(error) => log_error(&format!("popup activation failed: {error}")),
            }
            debug_log("popup: hiding after activation");
            window.hide();
        }
        PopupAction::Cancel => {
            debug_log("popup: cancel requested; hiding");
            window.hide();
        }
    }
}

#[derive(Debug)]
struct GtkSelectionRuntime {
    clipboard_controller: Rc<RefCell<ClipboardController>>,
    selection_controller: SelectionController,
    clipboard: GdkClipboardPort,
    paste: GtkPastePort,
}

impl GtkSelectionRuntime {
    fn new(clipboard_controller: Rc<RefCell<ClipboardController>>) -> io::Result<Self> {
        debug_log("selection runtime: initializing");
        Ok(Self {
            clipboard_controller,
            selection_controller: SelectionController::new(),
            clipboard: GdkClipboardPort::from_default_display()?,
            paste: GtkPastePort,
        })
    }

    fn activate(
        &mut self,
        source: SelectionSource,
        item: &HistoryItem,
    ) -> io::Result<SelectionOutcome> {
        debug_log(&format!("selection runtime: activating source={source:?}"));
        self.selection_controller.activate(
            source,
            item,
            &mut *self.clipboard_controller.borrow_mut(),
            &mut self.clipboard,
            &mut self.paste,
        )
    }
}

fn start_text_clipboard_monitor(
    app: Rc<RefCell<ClipboardHistoryApp>>,
    clipboard_controller: Rc<RefCell<ClipboardController>>,
    popup_view: GtkPopupView,
) -> io::Result<()> {
    debug_log("clipboard monitor: starting");
    let display = gdk::Display::default().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "no default GDK display available")
    })?;
    let clipboard = display.clipboard();

    clipboard.connect_changed(move |clipboard| {
        debug_log("clipboard monitor: changed signal");
        if clipboard_controller.borrow_mut().consume_self_update() {
            debug_log("clipboard monitor: ignored self update");
            return;
        }

        let app = Rc::clone(&app);
        let clipboard_controller = Rc::clone(&clipboard_controller);
        let popup_view = popup_view.clone();
        clipboard.read_text_async(None::<&gtk4::gio::Cancellable>, move |result| {
            let snapshot = match result {
                Ok(Some(text)) => ClipboardSnapshot::Text(text.to_string()),
                Ok(None) => ClipboardSnapshot::Unsupported,
                Err(error) => {
                    log_error(&format!("clipboard text read failed: {error}"));
                    ClipboardSnapshot::Unsupported
                }
            };

            let capture_result = {
                let mut app = app.borrow_mut();
                let mut clipboard_controller = clipboard_controller.borrow_mut();
                clipboard_controller.capture_external_snapshot(snapshot, &mut app)
            };

            match capture_result {
                Ok(CaptureOutcome::Captured) => {
                    debug_log("clipboard monitor: text captured");
                    popup_view.refresh_from_history(app.borrow().history().items().to_vec());
                }
                Ok(outcome) => debug_log(&format!("clipboard capture ignored: {outcome:?}")),
                Err(error) => log_error(&format!("clipboard capture failed: {error}")),
            }
        });
    });

    debug_log("clipboard monitor: connected");
    Ok(())
}

#[derive(Debug)]
struct GtkPastePort;

impl PastePort for GtkPastePort {
    fn try_paste(&mut self) -> io::Result<bool> {
        // Automatic paste depends on compositor/session support and will be
        // wired separately. Returning false preserves the manual paste fallback.
        Ok(false)
    }
}

fn log_error(message: &str) {
    eprintln!("{message}");
    debug_log(&format!("ERROR: {message}"));
}

fn debug_log(message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or_default();

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(DEBUG_LOG_PATH)
    {
        let _ = writeln!(file, "[{timestamp:.3}] {message}");
    }
}

#[derive(Debug)]
pub struct GdkClipboardPort {
    clipboard: gdk::Clipboard,
}

impl GdkClipboardPort {
    pub fn from_display(display: &gdk::Display) -> Self {
        Self {
            clipboard: display.clipboard(),
        }
    }

    pub fn from_default_display() -> io::Result<Self> {
        let display = gdk::Display::default().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "no default GDK display available")
        })?;

        Ok(Self::from_display(&display))
    }
}

impl ClipboardPort for GdkClipboardPort {
    fn read(&self) -> io::Result<ClipboardSnapshot> {
        // GTK exposes clipboard reads asynchronously. The production event loop
        // adapter will call the async API and feed snapshots into the core
        // controller; this sync trait implementation is intentionally limited.
        Ok(ClipboardSnapshot::Unsupported)
    }

    fn write(&mut self, content: &ClipboardContent) -> io::Result<()> {
        match content {
            ClipboardContent::Text { text } => {
                self.clipboard.set_text(text);
                Ok(())
            }
            ClipboardContent::Image { .. } => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "GDK image clipboard writing is not wired yet",
            )),
        }
    }
}
