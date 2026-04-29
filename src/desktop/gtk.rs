use crate::app::ClipboardHistoryApp;
use crate::clipboard::{CaptureOutcome, ClipboardController, ClipboardPort, ClipboardSnapshot};
use crate::desktop::paths;
use crate::domain::{
    ClipboardContent, ClipboardImage, ClipboardKind, HistoryItem, ImageFileExtension,
};
use crate::paste::{PastePort, SelectionOutcome, SelectionSource};
use crate::ui::{PopupAction, PopupCommand, PopupState};
use gtk4::gdk;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::thread;
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

fn install_popup_css() {
    let Some(display) = gdk::Display::default() else {
        log_error("popup css: no default display available");
        return;
    };
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        r#"
        window.clipboard-window {
            background: transparent;
        }

        .clipboard-popup {
            background: @theme_bg_color;
            border: 1px solid alpha(@theme_fg_color, 0.10);
            border-radius: 18px;
            box-shadow: 0 12px 36px alpha(black, 0.24);
            padding: 0;
        }

        .clipboard-header {
            padding: 16px 18px 12px;
            border-bottom: 1px solid alpha(@theme_fg_color, 0.08);
            background: linear-gradient(
                180deg,
                alpha(@theme_fg_color, 0.045),
                alpha(@theme_fg_color, 0.015)
            );
            border-top-left-radius: 18px;
            border-top-right-radius: 18px;
        }

        .clipboard-title {
            color: @theme_fg_color;
            font-size: 17px;
            font-weight: 700;
        }

        .clipboard-subtitle {
            color: alpha(@theme_fg_color, 0.62);
            font-size: 12px;
        }

        .clipboard-scroll {
            background: transparent;
            padding: 8px;
        }

        .clipboard-list {
            background: transparent;
        }

        button.clipboard-row {
            min-height: 42px;
            padding: 0;
            margin: 2px 4px;
            border: 1px solid transparent;
            border-radius: 14px;
            background: transparent;
            color: @theme_fg_color;
            box-shadow: none;
        }

        button.clipboard-row:hover {
            background: alpha(@theme_fg_color, 0.055);
            border-color: alpha(@theme_fg_color, 0.08);
        }

        button.clipboard-row:focus {
            outline: none;
            box-shadow: 0 0 0 2px alpha(@theme_selected_bg_color, 0.35);
        }

        button.clipboard-row.clipboard-row-selected {
            background: alpha(@theme_selected_bg_color, 0.16);
            border-color: alpha(@theme_selected_bg_color, 0.40);
        }

        .clipboard-text-preview {
            padding: 11px 12px;
            font-size: 14px;
        }

        .clipboard-image-row {
            padding: 8px;
        }

        .clipboard-thumbnail {
            background: alpha(@theme_fg_color, 0.04);
            border-radius: 12px;
        }
        "#,
    );
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
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
        install_popup_css();
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
            .spacing(6)
            .build();
        list_box.add_css_class("clipboard-list");

        let header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .build();
        header.add_css_class("clipboard-header");
        let title = gtk4::Label::builder()
            .label("Área de transferência")
            .xalign(0.0)
            .build();
        title.add_css_class("clipboard-title");
        header.append(&title);
        let subtitle = gtk4::Label::builder()
            .label("Use ↑ ↓, Enter ou clique para colar")
            .xalign(0.0)
            .build();
        subtitle.add_css_class("clipboard-subtitle");
        header.append(&subtitle);

        let root = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .build();
        root.add_css_class("clipboard-popup");
        root.append(&header);

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("LinuxClipboard")
            .default_width(460)
            .default_height(420)
            .resizable(false)
            .build();
        window.add_css_class("clipboard-window");
        let scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .min_content_height(300)
            .vexpand(true)
            .child(&list_box)
            .build();
        scroll.add_css_class("clipboard-scroll");
        root.append(&scroll);
        window.set_child(Some(&root));

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
        key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        {
            let state = Rc::clone(&state);
            let items = Rc::clone(&items);
            let list_box = list_box.clone();
            let scroll = scroll.clone();
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
                    debug_log("keyboard: ignored key");
                    return gtk4::glib::Propagation::Proceed;
                };

                let is_navigation =
                    matches!(command, PopupCommand::MoveUp | PopupCommand::MoveDown);
                if !is_navigation {
                    debug_log(&format!("popup: handling command {command:?}"));
                }
                let action = state.borrow_mut().handle_command(command);
                handle_popup_action(&window, action, &items.borrow(), selection_runtime.as_ref());
                if is_navigation {
                    update_selected_row(&list_box, &scroll, state.borrow().selected_index());
                }

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

    let mut selected_row = None;

    for (index, item) in items.borrow().iter().enumerate() {
        let content = row_content_for_item(item);
        let row = gtk4::Button::builder()
            .child(&content)
            .hexpand(true)
            .build();
        row.add_css_class("flat");
        row.add_css_class("clipboard-row");

        let is_selected = state.borrow().selected_index() == Some(index);
        if is_selected {
            row.add_css_class("clipboard-row-selected");
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
            selected_row = Some(row.clone().upcast::<gtk4::Widget>());
        }
    }

    scroll_selected_row_into_view(
        scroll,
        container,
        selected_row,
        state.borrow().selected_index(),
    );
}

fn update_selected_row(
    container: &gtk4::Box,
    scroll: &gtk4::ScrolledWindow,
    selected_index: Option<usize>,
) {
    let mut selected_row = None;
    let mut index = 0;
    let mut child = container.first_child();

    while let Some(widget) = child {
        if Some(index) == selected_index {
            widget.add_css_class("clipboard-row-selected");
            selected_row = Some(widget.clone());
        } else {
            widget.remove_css_class("clipboard-row-selected");
        }

        child = widget.next_sibling();
        index += 1;
    }

    scroll_selected_row_into_view(scroll, container, selected_row, selected_index);
}

fn scroll_selected_row_into_view(
    scroll: &gtk4::ScrolledWindow,
    container: &gtk4::Box,
    selected_row: Option<gtk4::Widget>,
    selected_index: Option<usize>,
) {
    let Some(selected_row) = selected_row else {
        return;
    };
    let adjustment = scroll.vadjustment();
    let container = container.clone();

    gtk4::glib::idle_add_local_once(move || {
        let upper = adjustment.upper();
        let page_size = adjustment.page_size();
        if upper <= page_size {
            return;
        }

        if selected_index == Some(0) {
            adjustment.set_value(0.0);
            return;
        }

        let Some(bounds) = selected_row.compute_bounds(&container) else {
            log_error("popup scroll: selected row bounds unavailable");
            return;
        };

        let row_top = bounds.y() as f64;
        let row_bottom = row_top + bounds.height() as f64;
        let visible_top = adjustment.value();
        let visible_bottom = visible_top + page_size;
        let max_value = upper - page_size;

        let target = if row_top < visible_top {
            row_top
        } else if row_bottom > visible_bottom {
            row_bottom - page_size
        } else {
            visible_top
        }
        .clamp(0.0, max_value);

        adjustment.set_value(target);
    });
}

fn row_content_for_item(item: &HistoryItem) -> gtk4::Widget {
    match item.kind {
        ClipboardKind::Text => {
            let label = gtk4::Label::builder()
                .label(&item.preview)
                .xalign(0.0)
                .wrap(false)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            label.add_css_class("clipboard-text-preview");
            label.upcast()
        }
        ClipboardKind::Image => image_row_content(item).upcast(),
    }
}

fn image_row_content(item: &HistoryItem) -> gtk4::Box {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(0)
        .build();
    row.add_css_class("clipboard-image-row");

    if let ClipboardContent::Image { path } = &item.content {
        let picture = gtk4::Picture::for_filename(path);
        picture.set_size_request(136, 90);
        picture.set_keep_aspect_ratio(true);
        picture.set_can_shrink(true);
        picture.add_css_class("clipboard-thumbnail");
        row.append(&picture);
    }

    row
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
        let clipboard_for_texture = clipboard.clone();
        clipboard.read_text_async(None::<&gtk4::gio::Cancellable>, move |result| {
            match result {
                Ok(Some(text)) => {
                    capture_clipboard_snapshot(
                        ClipboardSnapshot::Text(text.to_string()),
                        &app,
                        &clipboard_controller,
                        &popup_view,
                    );
                }
                Ok(None) => {
                    debug_log("clipboard monitor: no text; trying texture");
                    read_clipboard_texture(
                        clipboard_for_texture,
                        app,
                        clipboard_controller,
                        popup_view,
                    );
                }
                Err(error) => {
                    debug_log(&format!(
                        "clipboard monitor: text unavailable ({error}); trying texture"
                    ));
                    read_clipboard_texture(
                        clipboard_for_texture,
                        app,
                        clipboard_controller,
                        popup_view,
                    );
                }
            };
        });
    });

    debug_log("clipboard monitor: connected");
    Ok(())
}

fn read_clipboard_texture(
    clipboard: gdk::Clipboard,
    app: Rc<RefCell<ClipboardHistoryApp>>,
    clipboard_controller: Rc<RefCell<ClipboardController>>,
    popup_view: GtkPopupView,
) {
    clipboard.read_texture_async(None::<&gtk4::gio::Cancellable>, move |result| {
        let snapshot = match result {
            Ok(Some(texture)) => texture_to_png_image(&texture).map_or_else(
                |error| {
                    log_error(&format!("clipboard texture conversion failed: {error}"));
                    ClipboardSnapshot::Unsupported
                },
                ClipboardSnapshot::Image,
            ),
            Ok(None) => ClipboardSnapshot::Unsupported,
            Err(error) => {
                debug_log(&format!("clipboard monitor: texture unavailable ({error})"));
                ClipboardSnapshot::Unsupported
            }
        };

        capture_clipboard_snapshot(snapshot, &app, &clipboard_controller, &popup_view);
    });
}

fn texture_to_png_image(texture: &gdk::Texture) -> io::Result<ClipboardImage> {
    let temp_path =
        std::env::temp_dir().join(format!("clipboard-history-{}.png", uuid::Uuid::new_v4()));
    texture.save_to_png(&temp_path).map_err(io::Error::other)?;
    let bytes = std::fs::read(&temp_path);
    let cleanup = std::fs::remove_file(&temp_path);

    if let Err(error) = cleanup {
        debug_log(&format!(
            "clipboard texture temp cleanup failed: {} ({error})",
            temp_path.display()
        ));
    }

    let bytes = bytes?;
    ClipboardImage::new(bytes, ImageFileExtension::Png)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "empty png texture"))
}

fn capture_clipboard_snapshot(
    snapshot: ClipboardSnapshot,
    app: &Rc<RefCell<ClipboardHistoryApp>>,
    clipboard_controller: &Rc<RefCell<ClipboardController>>,
    popup_view: &GtkPopupView,
) {
    let capture_result = {
        let mut app = app.borrow_mut();
        let mut clipboard_controller = clipboard_controller.borrow_mut();
        clipboard_controller.capture_external_snapshot(snapshot, &mut app)
    };

    match capture_result {
        Ok(CaptureOutcome::Captured) => {
            debug_log("clipboard monitor: item captured");
            popup_view.refresh_from_history(app.borrow().history().items().to_vec());
        }
        Ok(outcome) => debug_log(&format!("clipboard capture ignored: {outcome:?}")),
        Err(error) => log_error(&format!("clipboard capture failed: {error}")),
    }
}

#[derive(Debug)]
struct GtkPastePort;

impl PastePort for GtkPastePort {
    fn try_paste(&mut self) -> io::Result<bool> {
        let Some(backend) = AutoPasteBackend::detect() else {
            debug_log("auto-paste: no supported keyboard automation command found");
            return Ok(false);
        };

        debug_log(&format!("auto-paste: scheduling {backend:?}"));
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(180));
            if let Err(error) = backend.run() {
                log_error(&format!("auto-paste failed: {error}"));
            }
        });

        Ok(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutoPasteBackend {
    Ydotool,
    Wtype,
    Xdotool,
}

impl AutoPasteBackend {
    fn detect() -> Option<Self> {
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        Self::detect_with(&session_type, command_exists)
    }

    fn detect_with(session_type: &str, command_exists: impl Fn(&str) -> bool) -> Option<Self> {
        let is_wayland = session_type.eq_ignore_ascii_case("wayland");
        let is_x11 = session_type.eq_ignore_ascii_case("x11");

        if is_wayland {
            if command_exists("ydotool") {
                return Some(Self::Ydotool);
            }
            if command_exists("wtype") {
                return Some(Self::Wtype);
            }
        }

        if is_x11 && command_exists("xdotool") {
            return Some(Self::Xdotool);
        }

        if command_exists("ydotool") {
            Some(Self::Ydotool)
        } else if command_exists("wtype") {
            Some(Self::Wtype)
        } else if command_exists("xdotool") {
            Some(Self::Xdotool)
        } else {
            None
        }
    }

    fn run(self) -> io::Result<()> {
        let mut command = match self {
            Self::Ydotool => {
                let mut command = Command::new("ydotool");
                command.args(["key", "ctrl+v"]);
                command
            }
            Self::Wtype => {
                let mut command = Command::new("wtype");
                command.args(["-M", "ctrl", "-k", "v", "-m", "ctrl"]);
                command
            }
            Self::Xdotool => {
                let mut command = Command::new("xdotool");
                command.args(["key", "--clearmodifiers", "ctrl+v"]);
                command
            }
        };

        let status = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if status.success() {
            debug_log(&format!("auto-paste: {self:?} completed"));
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "{self:?} exited with status {status}"
            )))
        }
    }
}

fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {command}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
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

    #[test]
    fn detects_ydotool_first_on_wayland() {
        let backend = AutoPasteBackend::detect_with("wayland", |command| {
            matches!(command, "ydotool" | "xdotool")
        });

        assert_eq!(backend, Some(AutoPasteBackend::Ydotool));
    }

    #[test]
    fn detects_wtype_on_wayland_when_ydotool_is_missing() {
        let backend =
            AutoPasteBackend::detect_with("wayland", |command| matches!(command, "wtype"));

        assert_eq!(backend, Some(AutoPasteBackend::Wtype));
    }

    #[test]
    fn detects_xdotool_first_on_x11() {
        let backend = AutoPasteBackend::detect_with("x11", |command| command == "xdotool");

        assert_eq!(backend, Some(AutoPasteBackend::Xdotool));
    }

    #[test]
    fn returns_no_auto_paste_backend_when_no_command_exists() {
        let backend = AutoPasteBackend::detect_with("wayland", |_| false);

        assert_eq!(backend, None);
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
            ClipboardContent::Image { path } => {
                let file = gtk4::gio::File::for_path(path);
                let texture = gdk::Texture::from_file(&file).map_err(io::Error::other)?;
                self.clipboard.set_texture(&texture);
                Ok(())
            }
        }
    }
}
