use crate::app::ClipboardHistoryApp;
use crate::clipboard::{CaptureOutcome, ClipboardController, ClipboardPort, ClipboardSnapshot};
use crate::desktop::paths;
use crate::domain::{ClipboardContent, HistoryItem};
use crate::paste::{PastePort, SelectionOutcome, SelectionSource};
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
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_shutdown(|_| {
        debug_log("application: shutdown");
        GTK_APP_STATE.with(|state| {
            *state.borrow_mut() = None;
        });
    });

    app.connect_activate(|app| {
        debug_log("application: activate");
        ensure_gtk_app_state(app);
        show_popup();
    });

    app.connect_command_line(|app, command_line| {
        let command = DesktopCommand::from_args(command_line.arguments());
        debug_log(&format!("application: command-line {command:?}"));
        ensure_gtk_app_state(app);
        match command {
            DesktopCommand::Daemon => {
                debug_log("application: daemon command; keeping popup hidden");
            }
            DesktopCommand::ShowPopup => show_popup(),
            DesktopCommand::TogglePopup => toggle_popup(),
            DesktopCommand::Quit => {
                debug_log("application: quit command");
                app.quit();
            }
        }

        gtk4::glib::ExitCode::SUCCESS
    });

    let exit_code = app.run();
    debug_log(&format!("run_application: exited with {exit_code:?}"));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesktopCommand {
    Daemon,
    ShowPopup,
    TogglePopup,
    Quit,
}

impl DesktopCommand {
    fn from_args(args: Vec<std::ffi::OsString>) -> Self {
        if args.iter().skip(1).any(|arg| arg == "--quit") {
            Self::Quit
        } else if args.iter().skip(1).any(|arg| arg == "--toggle-popup") {
            Self::TogglePopup
        } else if args.iter().skip(1).any(|arg| arg == "--show-popup") {
            Self::ShowPopup
        } else {
            Self::Daemon
        }
    }
}

fn ensure_gtk_app_state(app: &gtk4::Application) {
    let already_initialized = GTK_APP_STATE.with(|state| state.borrow().is_some());
    if already_initialized {
        debug_log("application: state already initialized");
        return;
    }

    debug_log("application: initializing resident state");
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
}

fn show_popup() {
    GTK_APP_STATE.with(|state| {
        if let Some(state) = state.borrow().as_ref() {
            state._popup.show();
        } else {
            log_error("popup: missing from thread-local state before show");
        }
    });
}

fn toggle_popup() {
    GTK_APP_STATE.with(|state| {
        if let Some(state) = state.borrow().as_ref() {
            state._popup.toggle();
        } else {
            log_error("popup: missing from thread-local state before toggle");
        }
    });
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

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("LinuxClipboard")
            .default_width(420)
            .default_height(260)
            .resizable(false)
            .build();
        let scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&list_box)
            .build();
        window.set_child(Some(&scroll));

        window.connect_show(|_| {
            debug_log("window: show signal");
        });
        window.connect_hide(|_| {
            debug_log("window: hide signal");
        });
        window.connect_close_request(move |window| {
            debug_log("window: close-request signal");
            debug_log("popup: hiding after close-request");
            window.hide();
            gtk4::glib::Propagation::Stop
        });
        schedule_window_diagnostics(&window);

        let key_controller = gtk4::EventControllerKey::new();
        {
            let state = Rc::clone(&state);
            let items = Rc::clone(&items);
            let list_box = list_box.clone();
            let scroll = scroll.clone();
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
                render_popup(
                    &list_box,
                    &scroll,
                    Rc::clone(&items),
                    Rc::clone(&state),
                    &window,
                    selection_runtime.as_ref(),
                );

                gtk4::glib::Propagation::Stop
            });
        }
        window.add_controller(key_controller);
        render_popup(
            &list_box,
            &scroll,
            Rc::clone(&items),
            Rc::clone(&state),
            &window,
            selection_runtime.as_ref(),
        );

        let view = GtkPopupView {
            items,
            state,
            list_box,
            scroll,
            window: window.clone(),
            selection_runtime,
        };

        Self { window, view }
    }

    pub fn show(&self) {
        debug_log("popup: present requested");
        self.window.present();
    }

    pub fn hide(&self) {
        debug_log("popup: hide requested");
        self.window.hide();
    }

    pub fn toggle(&self) {
        if self.window.is_visible() {
            self.hide();
        } else {
            self.show();
        }
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
    scroll: gtk4::ScrolledWindow,
    window: gtk4::ApplicationWindow,
    selection_runtime: Option<Rc<RefCell<GtkSelectionRuntime>>>,
}

impl GtkPopupView {
    pub fn refresh_from_history(&self, items: Vec<HistoryItem>) {
        debug_log(&format!("popup: refreshing with {} item(s)", items.len()));
        *self.items.borrow_mut() = items;
        *self.state.borrow_mut() = PopupState::from_items(&self.items.borrow());
        render_popup(
            &self.list_box,
            &self.scroll,
            Rc::clone(&self.items),
            Rc::clone(&self.state),
            &self.window,
            self.selection_runtime.as_ref(),
        );
    }
}

fn render_popup(
    container: &gtk4::Box,
    scroll: &gtk4::ScrolledWindow,
    items: Rc<RefCell<Vec<HistoryItem>>>,
    state: Rc<RefCell<PopupState>>,
    window: &gtk4::ApplicationWindow,
    selection_runtime: Option<&Rc<RefCell<GtkSelectionRuntime>>>,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    debug_log(&format!(
        "popup: rendering {} item(s), selected={:?}",
        items.borrow().len(),
        state.borrow().selected_index()
    ));

    if let Some(message) = state.borrow().empty_message() {
        container.append(
            &gtk4::Label::builder()
                .label(message)
                .xalign(0.0)
                .wrap(true)
                .build(),
        );
        return;
    }

    for (index, item) in items.borrow().iter().enumerate() {
        let label = gtk4::Label::builder()
            .label(&item.preview)
            .xalign(0.0)
            .wrap(false)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(8)
            .margin_end(8)
            .build();
        let row = gtk4::Button::builder().child(&label).hexpand(true).build();
        row.add_css_class("flat");

        let is_selected = state.borrow().selected_index() == Some(index);
        if is_selected {
            row.add_css_class("suggested-action");
        }

        let window = window.clone();
        let items_for_click = Rc::clone(&items);
        let state_for_click = Rc::clone(&state);
        let selection_runtime = selection_runtime.cloned();
        row.connect_clicked(move |_| {
            debug_log(&format!("mouse: clicked row index={index}"));
            let action = state_for_click.borrow_mut().click_item(index);
            handle_popup_action(
                &window,
                action,
                &items_for_click.borrow(),
                selection_runtime.as_ref(),
            );
        });

        container.append(&row);
        if is_selected {
            row.grab_focus();
        }
    }

    scroll_selected_row_into_view(
        scroll,
        state.borrow().selected_index(),
        items.borrow().len(),
    );
}

fn scroll_selected_row_into_view(
    scroll: &gtk4::ScrolledWindow,
    selected_index: Option<usize>,
    item_count: usize,
) {
    let Some(selected_index) = selected_index else {
        return;
    };
    let adjustment = scroll.vadjustment();

    gtk4::glib::idle_add_local_once(move || {
        let upper = adjustment.upper();
        let page_size = adjustment.page_size();
        if item_count <= 1 || upper <= page_size {
            return;
        }

        let max_value = upper - page_size;
        let last_index = item_count.saturating_sub(1).max(1) as f64;
        let target = (selected_index as f64 / last_index) * max_value;
        debug_log(&format!(
            "popup: scroll selected index={selected_index} target={target:.2}"
        ));
        adjustment.set_value(target);
    });
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
    clipboard: GdkClipboardPort,
    paste: GtkPastePort,
}

impl GtkSelectionRuntime {
    fn new(clipboard_controller: Rc<RefCell<ClipboardController>>) -> io::Result<Self> {
        debug_log("selection runtime: initializing");
        Ok(Self {
            clipboard_controller,
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
        self.clipboard_controller
            .borrow_mut()
            .mark_next_update_as_self();
        if let Err(error) = self.clipboard.write(&item.content) {
            self.clipboard_controller
                .borrow_mut()
                .clear_next_update_as_self();
            return Err(error);
        }

        if self.paste.try_paste()? {
            Ok(SelectionOutcome::AutoPasted { source })
        } else {
            Ok(SelectionOutcome::ClipboardOnly { source })
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_no_arguments_as_daemon_command() {
        assert_eq!(
            DesktopCommand::from_args(args(&["clipboard-history"])),
            DesktopCommand::Daemon
        );
    }

    #[test]
    fn parses_show_popup_command() {
        assert_eq!(
            DesktopCommand::from_args(args(&["clipboard-history", "--show-popup"])),
            DesktopCommand::ShowPopup
        );
    }

    #[test]
    fn parses_toggle_popup_command() {
        assert_eq!(
            DesktopCommand::from_args(args(&["clipboard-history", "--toggle-popup"])),
            DesktopCommand::TogglePopup
        );
    }

    #[test]
    fn parses_quit_command_with_priority() {
        assert_eq!(
            DesktopCommand::from_args(args(&["clipboard-history", "--toggle-popup", "--quit"])),
            DesktopCommand::Quit
        );
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
