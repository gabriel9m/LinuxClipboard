use crate::app::ClipboardHistoryApp;
use crate::domain::{
    ClipboardContent, ClipboardImage, HistoryItem, looks_like_sensitive_text, normalize_text,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;

pub trait ClipboardPort {
    fn read(&self) -> io::Result<ClipboardSnapshot>;
    fn write(&mut self, content: &ClipboardContent) -> io::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardSnapshot {
    Text(String),
    Image(ClipboardImage),
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureOutcome {
    Captured,
    IgnoredDuplicate,
    IgnoredSelfUpdate,
    IgnoredUnsupported,
    IgnoredInvalidText,
    IgnoredSensitiveText,
}

#[derive(Debug, Default)]
pub struct ClipboardController {
    ignore_next_update: bool,
    last_external_capture: Option<ClipboardFingerprint>,
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
        if self.consume_self_update() {
            return Ok(CaptureOutcome::IgnoredSelfUpdate);
        }

        self.capture_external_snapshot(clipboard.read()?, app)
    }

    pub fn consume_self_update(&mut self) -> bool {
        if self.ignore_next_update {
            self.ignore_next_update = false;
            true
        } else {
            false
        }
    }

    pub fn capture_external_snapshot(
        &mut self,
        snapshot: ClipboardSnapshot,
        app: &mut ClipboardHistoryApp,
    ) -> io::Result<CaptureOutcome> {
        let fingerprint = ClipboardFingerprint::from_snapshot(&snapshot);
        if fingerprint.is_some() && fingerprint == self.last_external_capture {
            return Ok(CaptureOutcome::IgnoredDuplicate);
        }

        match snapshot {
            ClipboardSnapshot::Text(text) => {
                if looks_like_sensitive_text(&text) {
                    self.last_external_capture = fingerprint;
                    return Ok(CaptureOutcome::IgnoredSensitiveText);
                }

                let Some(item) = HistoryItem::text(text) else {
                    return Ok(CaptureOutcome::IgnoredInvalidText);
                };

                app.add_item(item)?;
                self.last_external_capture = fingerprint;
                Ok(CaptureOutcome::Captured)
            }
            ClipboardSnapshot::Image(image) => {
                app.add_image(&image)?;
                self.last_external_capture = fingerprint;
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
        self.ignore_next_update = true;
        if let Err(error) = clipboard.write(&item.content) {
            self.ignore_next_update = false;
            return Err(error);
        }

        Ok(())
    }

    pub fn mark_next_update_as_self(&mut self) {
        self.ignore_next_update = true;
    }

    pub fn clear_next_update_as_self(&mut self) {
        self.ignore_next_update = false;
    }

    pub fn will_ignore_next_update(&self) -> bool {
        self.ignore_next_update
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClipboardFingerprint {
    Text(String),
    Image {
        hash: u64,
        len: usize,
        extension: &'static str,
    },
}

impl ClipboardFingerprint {
    fn from_snapshot(snapshot: &ClipboardSnapshot) -> Option<Self> {
        match snapshot {
            ClipboardSnapshot::Text(text) => normalize_text(text.clone()).map(Self::Text),
            ClipboardSnapshot::Image(image) => {
                let mut hasher = DefaultHasher::new();
                image.bytes().hash(&mut hasher);
                Some(Self::Image {
                    hash: hasher.finish(),
                    len: image.bytes().len(),
                    extension: image.extension().as_str(),
                })
            }
            ClipboardSnapshot::Unsupported => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ClipboardContent, ImageFileExtension};
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

    #[derive(Debug)]
    struct FailingClipboard;

    impl ClipboardPort for FailingClipboard {
        fn read(&self) -> io::Result<ClipboardSnapshot> {
            Ok(ClipboardSnapshot::Unsupported)
        }

        fn write(&mut self, _content: &ClipboardContent) -> io::Result<()> {
            Err(io::Error::other("write failed"))
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
    fn captures_image_from_clipboard_into_history() {
        let (temp_dir, mut app) = app_in_temp_dir();
        let image = ClipboardImage::new([1, 2, 3], ImageFileExtension::Png).expect("valid image");
        let clipboard = FakeClipboard::new(ClipboardSnapshot::Image(image.clone()));
        let mut controller = ClipboardController::new();

        let outcome = controller
            .capture_current(&clipboard, &mut app)
            .expect("capture clipboard");

        assert_eq!(outcome, CaptureOutcome::Captured);
        assert_eq!(app.history().items().len(), 1);
        let ClipboardContent::Image { path } = &app.history().items()[0].content else {
            panic!("expected image item");
        };
        assert!(path.starts_with(temp_dir.path().join("images")));
        assert_eq!(path.extension().unwrap(), "png");
        assert_eq!(std::fs::read(path).expect("read image"), image.bytes());
    }

    #[test]
    fn ignores_duplicate_consecutive_text_snapshots() {
        let (_temp_dir, mut app) = app_in_temp_dir();
        let mut controller = ClipboardController::new();

        let first = controller
            .capture_external_snapshot(ClipboardSnapshot::Text("D2C4B4".to_string()), &mut app)
            .expect("capture first text");
        let duplicate = controller
            .capture_external_snapshot(ClipboardSnapshot::Text("D2C4B4".to_string()), &mut app)
            .expect("capture duplicate text");

        assert_eq!(first, CaptureOutcome::Captured);
        assert_eq!(duplicate, CaptureOutcome::IgnoredDuplicate);
        assert_eq!(app.history().items().len(), 1);
        assert_eq!(app.history().items()[0].preview, "D2C4B4");
    }

    #[test]
    fn captures_different_consecutive_text_snapshots() {
        let (_temp_dir, mut app) = app_in_temp_dir();
        let mut controller = ClipboardController::new();

        controller
            .capture_external_snapshot(ClipboardSnapshot::Text("D2C4B4".to_string()), &mut app)
            .expect("capture first text");
        let outcome = controller
            .capture_external_snapshot(ClipboardSnapshot::Text("F3E3D0".to_string()), &mut app)
            .expect("capture second text");

        assert_eq!(outcome, CaptureOutcome::Captured);
        assert_eq!(app.history().items().len(), 2);
        assert_eq!(app.history().items()[0].preview, "F3E3D0");
        assert_eq!(app.history().items()[1].preview, "D2C4B4");
    }

    #[test]
    fn ignores_duplicate_consecutive_image_snapshots() {
        let (_temp_dir, mut app) = app_in_temp_dir();
        let image = ClipboardImage::new([1, 2, 3], ImageFileExtension::Png).expect("valid image");
        let mut controller = ClipboardController::new();

        let first = controller
            .capture_external_snapshot(ClipboardSnapshot::Image(image.clone()), &mut app)
            .expect("capture first image");
        let duplicate = controller
            .capture_external_snapshot(ClipboardSnapshot::Image(image), &mut app)
            .expect("capture duplicate image");

        assert_eq!(first, CaptureOutcome::Captured);
        assert_eq!(duplicate, CaptureOutcome::IgnoredDuplicate);
        assert_eq!(app.history().items().len(), 1);
    }

    #[test]
    fn persists_captured_image_reference() {
        let (temp_dir, mut app) = app_in_temp_dir();
        let history_path = temp_dir.path().join("history.json");
        let image =
            ClipboardImage::new([255, 216, 255], ImageFileExtension::Jpeg).expect("valid image");
        let clipboard = FakeClipboard::new(ClipboardSnapshot::Image(image));
        let mut controller = ClipboardController::new();

        controller
            .capture_current(&clipboard, &mut app)
            .expect("capture clipboard");
        let loaded = storage::load_history(&history_path);

        assert_eq!(loaded.items().len(), 1);
        let ClipboardContent::Image { path } = &loaded.items()[0].content else {
            panic!("expected image item");
        };
        assert!(path.exists());
        assert_eq!(path.extension().unwrap(), "jpg");
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
    fn ignores_sensitive_text_without_persisting() {
        let (temp_dir, mut app) = app_in_temp_dir();
        let history_path = temp_dir.path().join("history.json");
        let clipboard = FakeClipboard::new(ClipboardSnapshot::Text(
            "SECRET_TOKEN_TEST_123 password=super-secret".to_string(),
        ));
        let mut controller = ClipboardController::new();

        let outcome = controller
            .capture_current(&clipboard, &mut app)
            .expect("capture clipboard");

        assert_eq!(outcome, CaptureOutcome::IgnoredSensitiveText);
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
    fn clears_self_update_marker_when_selection_write_fails() {
        let selected_item = HistoryItem::text("selected").expect("valid text item");
        let mut clipboard = FailingClipboard;
        let mut controller = ClipboardController::new();

        let result = controller.write_from_selection(&mut clipboard, &selected_item);

        assert!(result.is_err());
        assert!(!controller.will_ignore_next_update());
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
    fn consumes_self_update_before_async_clipboard_read() {
        let mut controller = ClipboardController::new();
        let selected_item = HistoryItem::text("selected").expect("valid text item");
        let mut clipboard = FakeClipboard::new(ClipboardSnapshot::Text("selected".to_string()));

        controller
            .write_from_selection(&mut clipboard, &selected_item)
            .expect("write selection");

        assert!(controller.consume_self_update());
        assert!(!controller.consume_self_update());
        assert_eq!(clipboard.read_count.get(), 0);
    }

    #[test]
    fn captures_external_snapshot_that_was_read_asynchronously() {
        let (_temp_dir, mut app) = app_in_temp_dir();
        let mut controller = ClipboardController::new();

        let outcome = controller
            .capture_external_snapshot(ClipboardSnapshot::Text("async text".to_string()), &mut app)
            .expect("capture async snapshot");

        assert_eq!(outcome, CaptureOutcome::Captured);
        assert_eq!(app.history().items()[0].preview, "async text");
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
