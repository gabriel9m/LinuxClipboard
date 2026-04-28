use crate::app::ClipboardHistoryApp;
use crate::clipboard::{ClipboardController, ClipboardPort, ClipboardSnapshot};
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
        let history_app = ClipboardHistoryApp::load_with_paths(
            &desktop_paths.history_path,
            &desktop_paths.images_dir,
        );
        let popup = GtkPopup::new(app, history_app.history().items().to_vec());

        popup.show();
    });

    app.run();
}

#[derive(Debug)]
pub struct GtkPopup {
    window: gtk4::ApplicationWindow,
}

impl GtkPopup {
    pub fn new(app: &gtk4::Application, items: Vec<HistoryItem>) -> Self {
        let state = Rc::new(RefCell::new(PopupState::from_items(&items)));
        let items = Rc::new(items);
        let selection_runtime = GtkSelectionRuntime::new()
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

        render_popup(&list_box, &items, &state.borrow());

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
                handle_popup_action(&window, action, &items, selection_runtime.as_ref());
                render_popup(&list_box, &items, &state.borrow());

                gtk4::glib::Propagation::Stop
            });
        }
        window.add_controller(key_controller);

        Self { window }
    }

    pub fn show(&self) {
        self.window.present();
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
    clipboard_controller: ClipboardController,
    selection_controller: SelectionController,
    clipboard: GdkClipboardPort,
    paste: GtkPastePort,
}

impl GtkSelectionRuntime {
    fn new() -> io::Result<Self> {
        Ok(Self {
            clipboard_controller: ClipboardController::new(),
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
            &mut self.clipboard_controller,
            &mut self.clipboard,
            &mut self.paste,
        )
    }
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
