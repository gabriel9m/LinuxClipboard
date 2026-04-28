use crate::app::ClipboardHistoryApp;
use crate::clipboard::{CaptureOutcome, ClipboardController, ClipboardPort, ClipboardSnapshot};
use crate::desktop::paths;
use crate::domain::{ClipboardContent, HistoryItem};
use crate::paste::{PastePort, SelectionController, SelectionOutcome, SelectionSource};
use crate::ui::{PopupAction, PopupCommand, PopupState};
use gtk4::gdk;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

const APP_ID: &str = "io.github.gabriel9m.LinuxClipboard";

pub fn run_application() {
    let app = gtk4::Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        let desktop_paths = paths::default_paths();
        let history_app = Rc::new(RefCell::new(ClipboardHistoryApp::load_with_paths(
            &desktop_paths.history_path,
            &desktop_paths.images_dir,
        )));
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
            eprintln!("clipboard monitor unavailable: {error}");
        }

        popup.show();
    });

    app.run();
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

        let key_controller = gtk4::EventControllerKey::new();
        {
            let state = Rc::clone(&state);
            let items = Rc::clone(&items);
            let list_box = list_box.clone();
            let window = window.clone();
            let selection_runtime = selection_runtime.clone();

            key_controller.connect_key_pressed(move |_, key, _, _| {
                let command = match key {
                    gdk::Key::Up => Some(PopupCommand::MoveUp),
                    gdk::Key::Down => Some(PopupCommand::MoveDown),
                    gdk::Key::Return | gdk::Key::KP_Enter => Some(PopupCommand::Enter),
                    gdk::Key::Escape => Some(PopupCommand::Escape),
                    _ => None,
                };

                let Some(command) = command else {
                    return gtk4::glib::Propagation::Proceed;
                };

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
        self.window.present();
    }

    pub fn view_handle(&self) -> GtkPopupView {
        self.view.clone()
    }
}

#[derive(Debug, Clone)]
pub struct GtkPopupView {
    items: Rc<RefCell<Vec<HistoryItem>>>,
    state: Rc<RefCell<PopupState>>,
    list_box: gtk4::Box,
}

impl GtkPopupView {
    pub fn refresh_from_history(&self, items: Vec<HistoryItem>) {
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
            let Some(item) = items.get(index) else {
                eprintln!("popup activation ignored: index out of range: {index}");
                return;
            };

            let Some(selection_runtime) = selection_runtime else {
                eprintln!("popup activation ignored: selection runtime unavailable");
                return;
            };

            match selection_runtime.borrow_mut().activate(source, item) {
                Ok(outcome) => eprintln!("popup activation outcome: {outcome:?}"),
                Err(error) => eprintln!("popup activation failed: {error}"),
            }
            window.hide();
        }
        PopupAction::Cancel => {
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
    let display = gdk::Display::default().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "no default GDK display available")
    })?;
    let clipboard = display.clipboard();

    clipboard.connect_changed(move |clipboard| {
        if clipboard_controller.borrow_mut().consume_self_update() {
            eprintln!("clipboard change ignored: self update");
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
                    eprintln!("clipboard text read failed: {error}");
                    ClipboardSnapshot::Unsupported
                }
            };

            match clipboard_controller
                .borrow_mut()
                .capture_external_snapshot(snapshot, &mut app.borrow_mut())
            {
                Ok(CaptureOutcome::Captured) => {
                    eprintln!("clipboard text captured");
                    popup_view.refresh_from_history(app.borrow().history().items().to_vec());
                }
                Ok(outcome) => eprintln!("clipboard capture ignored: {outcome:?}"),
                Err(error) => eprintln!("clipboard capture failed: {error}"),
            }
        });
    });

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
