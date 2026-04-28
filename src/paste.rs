use crate::clipboard::{ClipboardController, ClipboardPort};
use crate::domain::HistoryItem;
use std::io;

pub trait PastePort {
    fn try_paste(&mut self) -> io::Result<bool>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionSource {
    Click,
    Enter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionOutcome {
    AutoPasted { source: SelectionSource },
    ClipboardOnly { source: SelectionSource },
}

#[derive(Debug, Default)]
pub struct SelectionController;

impl SelectionController {
    pub fn new() -> Self {
        Self
    }

    pub fn activate(
        &mut self,
        source: SelectionSource,
        item: &HistoryItem,
        clipboard_controller: &mut ClipboardController,
        clipboard: &mut impl ClipboardPort,
        paste: &mut impl PastePort,
    ) -> io::Result<SelectionOutcome> {
        clipboard_controller.write_from_selection(clipboard, item)?;

        if paste.try_paste()? {
            Ok(SelectionOutcome::AutoPasted { source })
        } else {
            Ok(SelectionOutcome::ClipboardOnly { source })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clipboard::{CaptureOutcome, ClipboardSnapshot};
    use crate::domain::{ClipboardContent, HistoryItem};
    use std::cell::Cell;

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
    struct FakePaste {
        result: bool,
        calls: Vec<&'static str>,
    }

    impl FakePaste {
        fn new(result: bool) -> Self {
            Self {
                result,
                calls: Vec::new(),
            }
        }
    }

    impl PastePort for FakePaste {
        fn try_paste(&mut self) -> io::Result<bool> {
            self.calls.push("try_paste");
            Ok(self.result)
        }
    }

    fn selected_text_item() -> HistoryItem {
        HistoryItem::text("selected").expect("valid text item")
    }

    #[test]
    fn click_selection_writes_clipboard_before_trying_auto_paste() {
        let item = selected_text_item();
        let mut clipboard_controller = ClipboardController::new();
        let mut clipboard = FakeClipboard::new(ClipboardSnapshot::Unsupported);
        let mut paste = FakePaste::new(true);
        let mut selection = SelectionController::new();

        let outcome = selection
            .activate(
                SelectionSource::Click,
                &item,
                &mut clipboard_controller,
                &mut clipboard,
                &mut paste,
            )
            .expect("activate selection");

        assert_eq!(
            clipboard.written,
            vec![ClipboardContent::Text {
                text: "selected".to_string()
            }]
        );
        assert_eq!(
            outcome,
            SelectionOutcome::AutoPasted {
                source: SelectionSource::Click
            }
        );
        assert_eq!(paste.calls, vec!["try_paste"]);
        assert!(clipboard_controller.will_ignore_next_update());
    }

    #[test]
    fn enter_selection_has_same_behavior_as_click() {
        let item = selected_text_item();
        let mut clipboard_controller = ClipboardController::new();
        let mut clipboard = FakeClipboard::new(ClipboardSnapshot::Unsupported);
        let mut paste = FakePaste::new(true);
        let mut selection = SelectionController::new();

        let outcome = selection
            .activate(
                SelectionSource::Enter,
                &item,
                &mut clipboard_controller,
                &mut clipboard,
                &mut paste,
            )
            .expect("activate selection");

        assert_eq!(
            clipboard.written,
            vec![ClipboardContent::Text {
                text: "selected".to_string()
            }]
        );
        assert_eq!(
            outcome,
            SelectionOutcome::AutoPasted {
                source: SelectionSource::Enter
            }
        );
        assert!(clipboard_controller.will_ignore_next_update());
    }

    #[test]
    fn failed_auto_paste_keeps_content_available_in_clipboard() {
        let item = selected_text_item();
        let mut clipboard_controller = ClipboardController::new();
        let mut clipboard = FakeClipboard::new(ClipboardSnapshot::Unsupported);
        let mut paste = FakePaste::new(false);
        let mut selection = SelectionController::new();

        let outcome = selection
            .activate(
                SelectionSource::Click,
                &item,
                &mut clipboard_controller,
                &mut clipboard,
                &mut paste,
            )
            .expect("activate selection");

        assert_eq!(
            outcome,
            SelectionOutcome::ClipboardOnly {
                source: SelectionSource::Click
            }
        );
        assert_eq!(
            clipboard.written,
            vec![ClipboardContent::Text {
                text: "selected".to_string()
            }]
        );
        assert!(clipboard_controller.will_ignore_next_update());
    }

    #[test]
    fn paste_is_not_attempted_when_clipboard_write_fails() {
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

        let item = selected_text_item();
        let mut clipboard_controller = ClipboardController::new();
        let mut clipboard = FailingClipboard;
        let mut paste = FakePaste::new(true);
        let mut selection = SelectionController::new();

        let result = selection.activate(
            SelectionSource::Click,
            &item,
            &mut clipboard_controller,
            &mut clipboard,
            &mut paste,
        );

        assert!(result.is_err());
        assert!(paste.calls.is_empty());
        assert!(!clipboard_controller.will_ignore_next_update());
    }

    #[test]
    fn self_update_after_selection_is_still_ignored_by_clipboard_controller() {
        let item = selected_text_item();
        let mut clipboard_controller = ClipboardController::new();
        let mut clipboard = FakeClipboard::new(ClipboardSnapshot::Text("selected".to_string()));
        let mut paste = FakePaste::new(false);
        let mut selection = SelectionController::new();

        selection
            .activate(
                SelectionSource::Enter,
                &item,
                &mut clipboard_controller,
                &mut clipboard,
                &mut paste,
            )
            .expect("activate selection");
        let mut app = crate::app::ClipboardHistoryApp::new_empty(
            tempfile::tempdir()
                .expect("temp dir")
                .path()
                .join("history.json"),
        );
        let outcome = clipboard_controller
            .capture_current(&clipboard, &mut app)
            .expect("capture clipboard");

        assert_eq!(outcome, CaptureOutcome::IgnoredSelfUpdate);
        assert_eq!(clipboard.read_count.get(), 0);
    }
}
