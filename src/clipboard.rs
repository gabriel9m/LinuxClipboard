use crate::app::ClipboardHistoryApp;
use crate::domain::{ClipboardContent, HistoryItem};
use std::io;

pub trait ClipboardPort {
    fn read(&self) -> io::Result<ClipboardSnapshot>;
    fn write(&mut self, content: &ClipboardContent) -> io::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardSnapshot {
    Text(String),
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureOutcome {
    Captured,
    IgnoredSelfUpdate,
    IgnoredUnsupported,
    IgnoredInvalidText,
}

#[derive(Debug, Default)]
pub struct ClipboardController {
    ignore_next_update: bool,
}

impl ClipboardController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture_current(
        &mut self,
        clipboard: &impl ClipboardPort,
        app: &mut ClipboardHistoryApp,
    ) -> io::Result<CaptureOutcome> {
        if self.ignore_next_update {
            self.ignore_next_update = false;
            return Ok(CaptureOutcome::IgnoredSelfUpdate);
        }

        match clipboard.read()? {
            ClipboardSnapshot::Text(text) => {
                let Some(item) = HistoryItem::text(text) else {
                    return Ok(CaptureOutcome::IgnoredInvalidText);
                };

                app.add_item(item)?;
                Ok(CaptureOutcome::Captured)
            }
            ClipboardSnapshot::Unsupported => Ok(CaptureOutcome::IgnoredUnsupported),
        }
    }

    pub fn write_from_selection(
        &mut self,
        clipboard: &mut impl ClipboardPort,
        item: &HistoryItem,
    ) -> io::Result<()> {
        clipboard.write(&item.content)?;
        self.ignore_next_update = true;

        Ok(())
    }

    pub fn will_ignore_next_update(&self) -> bool {
        self.ignore_next_update
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ClipboardContent;
    use crate::storage;
    use std::cell::Cell;
    use std::path::PathBuf;

    #[derive(Debug)]
    struct FakeClipboard {
        snapshot: ClipboardSnapshot,
        written: Vec<ClipboardContent>,
        read_count: Cell<usize>,
    }

    impl FakeClipboard {
        fn new(snapshot: ClipboardSnapshot) -> Self {
            Self {
                snapshot,
                written: Vec::new(),
                read_count: Cell::new(0),
            }
        }
    }

    impl ClipboardPort for FakeClipboard {
        fn read(&self) -> io::Result<ClipboardSnapshot> {
            self.read_count.set(self.read_count.get() + 1);
            Ok(self.snapshot.clone())
        }

        fn write(&mut self, content: &ClipboardContent) -> io::Result<()> {
            self.written.push(content.clone());
            Ok(())
        }
    }

    fn app_in_temp_dir() -> (tempfile::TempDir, ClipboardHistoryApp) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("history.json");
        let app = ClipboardHistoryApp::new_empty(history_path);

        (temp_dir, app)
    }

    #[test]
    fn captures_text_from_clipboard_into_history() {
        let (_temp_dir, mut app) = app_in_temp_dir();
        let clipboard = FakeClipboard::new(ClipboardSnapshot::Text(" captured ".to_string()));
        let mut controller = ClipboardController::new();

        let outcome = controller
            .capture_current(&clipboard, &mut app)
            .expect("capture clipboard");

        assert_eq!(outcome, CaptureOutcome::Captured);
        assert_eq!(app.history().items().len(), 1);
        assert_eq!(app.history().items()[0].preview, "captured");
    }

    #[test]
    fn persists_captured_text() {
        let (temp_dir, mut app) = app_in_temp_dir();
        let history_path = temp_dir.path().join("history.json");
        let clipboard = FakeClipboard::new(ClipboardSnapshot::Text("persisted".to_string()));
        let mut controller = ClipboardController::new();

        controller
            .capture_current(&clipboard, &mut app)
            .expect("capture clipboard");
        let loaded = storage::load_history(&history_path);

        assert_eq!(loaded.items().len(), 1);
        assert_eq!(loaded.items()[0].preview, "persisted");
    }

    #[test]
    fn ignores_unsupported_clipboard_content() {
        let (_temp_dir, mut app) = app_in_temp_dir();
        let clipboard = FakeClipboard::new(ClipboardSnapshot::Unsupported);
        let mut controller = ClipboardController::new();

        let outcome = controller
            .capture_current(&clipboard, &mut app)
            .expect("capture clipboard");

        assert_eq!(outcome, CaptureOutcome::IgnoredUnsupported);
        assert!(app.history().is_empty());
    }

    #[test]
    fn ignores_invalid_text_without_persisting() {
        let (temp_dir, mut app) = app_in_temp_dir();
        let history_path = temp_dir.path().join("history.json");
        let clipboard = FakeClipboard::new(ClipboardSnapshot::Text(" \n\t ".to_string()));
        let mut controller = ClipboardController::new();

        let outcome = controller
            .capture_current(&clipboard, &mut app)
            .expect("capture clipboard");

        assert_eq!(outcome, CaptureOutcome::IgnoredInvalidText);
        assert!(app.history().is_empty());
        assert!(!history_path.exists());
    }

    #[test]
    fn marks_next_clipboard_update_as_self_update_after_writing_selection() {
        let (_temp_dir, mut app) = app_in_temp_dir();
        let selected_item = HistoryItem::text("selected").expect("valid text item");
        app.add_item(selected_item.clone())
            .expect("add selected item");
        let mut clipboard = FakeClipboard::new(ClipboardSnapshot::Text("selected".to_string()));
        let mut controller = ClipboardController::new();

        controller
            .write_from_selection(&mut clipboard, &selected_item)
            .expect("write selection");

        assert!(controller.will_ignore_next_update());
        assert_eq!(
            clipboard.written,
            vec![ClipboardContent::Text {
                text: "selected".to_string()
            }]
        );
    }

    #[test]
    fn skips_recapturing_the_next_update_after_app_writes_to_clipboard() {
        let (_temp_dir, mut app) = app_in_temp_dir();
        let selected_item = HistoryItem::text("selected").expect("valid text item");
        app.add_item(selected_item.clone())
            .expect("add selected item");
        let mut clipboard = FakeClipboard::new(ClipboardSnapshot::Text("selected".to_string()));
        let mut controller = ClipboardController::new();

        controller
            .write_from_selection(&mut clipboard, &selected_item)
            .expect("write selection");
        let outcome = controller
            .capture_current(&clipboard, &mut app)
            .expect("capture clipboard");

        assert_eq!(outcome, CaptureOutcome::IgnoredSelfUpdate);
        assert_eq!(app.history().items().len(), 1);
        assert_eq!(clipboard.read_count.get(), 0);
        assert!(!controller.will_ignore_next_update());
    }

    #[test]
    fn can_capture_external_update_after_self_update_was_skipped() {
        let (_temp_dir, mut app) = app_in_temp_dir();
        let selected_item = HistoryItem::text("selected").expect("valid text item");
        app.add_item(selected_item.clone())
            .expect("add selected item");
        let mut clipboard = FakeClipboard::new(ClipboardSnapshot::Text("external".to_string()));
        let mut controller = ClipboardController::new();

        controller
            .write_from_selection(&mut clipboard, &selected_item)
            .expect("write selection");
        controller
            .capture_current(&clipboard, &mut app)
            .expect("skip self update");
        let outcome = controller
            .capture_current(&clipboard, &mut app)
            .expect("capture external update");

        assert_eq!(outcome, CaptureOutcome::Captured);
        assert_eq!(app.history().items()[0].preview, "external");
        assert_eq!(app.history().items().len(), 2);
    }

    #[test]
    fn writes_image_selection_through_clipboard_port() {
        let image_path = PathBuf::from("/tmp/image.png");
        let preview_path = PathBuf::from("/tmp/image-thumb.png");
        let selected_item = HistoryItem::image(&image_path, &preview_path);
        let mut clipboard = FakeClipboard::new(ClipboardSnapshot::Unsupported);
        let mut controller = ClipboardController::new();

        controller
            .write_from_selection(&mut clipboard, &selected_item)
            .expect("write image selection");

        assert_eq!(
            clipboard.written,
            vec![ClipboardContent::Image { path: image_path }]
        );
        assert!(controller.will_ignore_next_update());
    }
}
