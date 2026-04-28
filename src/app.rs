use crate::domain::{ClipboardImage, History, HistoryItem};
use crate::storage;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ClipboardHistoryApp {
    history: History,
    history_path: PathBuf,
    images_dir: PathBuf,
}

impl ClipboardHistoryApp {
    pub fn load(history_path: impl Into<PathBuf>) -> Self {
        let history_path = history_path.into();
        let images_dir = default_images_dir(&history_path);

        Self::load_with_paths(history_path, images_dir)
    }

    pub fn load_with_paths(
        history_path: impl Into<PathBuf>,
        images_dir: impl Into<PathBuf>,
    ) -> Self {
        let history_path = history_path.into();
        let history = storage::load_history(&history_path);

        Self {
            history,
            history_path,
            images_dir: images_dir.into(),
        }
    }

    pub fn new_empty(history_path: impl Into<PathBuf>) -> Self {
        let history_path = history_path.into();
        let images_dir = default_images_dir(&history_path);

        Self::new_empty_with_paths(history_path, images_dir)
    }

    pub fn new_empty_with_paths(
        history_path: impl Into<PathBuf>,
        images_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            history: History::new(),
            history_path: history_path.into(),
            images_dir: images_dir.into(),
        }
    }

    pub fn add_item(&mut self, item: HistoryItem) -> io::Result<()> {
        let removed = self.history.push(item);

        storage::cleanup_removed_image_files(&removed)?;
        storage::save_history(&self.history_path, &self.history)?;

        Ok(())
    }

    pub fn add_image(&mut self, image: &ClipboardImage) -> io::Result<()> {
        let image_path = storage::save_clipboard_image(&self.images_dir, image)?;
        let item = HistoryItem::image(&image_path, &image_path);

        self.add_item(item)
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn history_path(&self) -> &Path {
        &self.history_path
    }

    pub fn images_dir(&self) -> &Path {
        &self.images_dir
    }
}

fn default_images_dir(history_path: &Path) -> PathBuf {
    history_path
        .parent()
        .map_or_else(|| PathBuf::from("images"), |parent| parent.join("images"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ClipboardContent, HISTORY_LIMIT, ImageFileExtension};
    use std::fs;

    fn text_item(value: &str) -> HistoryItem {
        HistoryItem::text(value).expect("valid text item")
    }

    #[test]
    fn loads_existing_history_from_storage() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("history.json");
        let mut existing = History::new();
        existing.push(text_item("restored"));
        storage::save_history(&history_path, &existing).expect("save existing history");

        let app = ClipboardHistoryApp::load(&history_path);

        assert_eq!(app.history().items().len(), 1);
        assert_eq!(app.history().items()[0].preview, "restored");
        assert_eq!(app.history_path(), history_path.as_path());
        assert_eq!(app.images_dir(), temp_dir.path().join("images").as_path());
    }

    #[test]
    fn loads_existing_history_with_explicit_images_directory() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("history.json");
        let images_dir = temp_dir.path().join("custom-images");
        let mut existing = History::new();
        existing.push(text_item("restored"));
        storage::save_history(&history_path, &existing).expect("save existing history");

        let app = ClipboardHistoryApp::load_with_paths(&history_path, &images_dir);

        assert_eq!(app.history().items().len(), 1);
        assert_eq!(app.history().items()[0].preview, "restored");
        assert_eq!(app.history_path(), history_path.as_path());
        assert_eq!(app.images_dir(), images_dir.as_path());
    }

    #[test]
    fn adding_item_persists_history_to_json() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("history.json");
        let mut app = ClipboardHistoryApp::new_empty(&history_path);

        app.add_item(text_item("persist me")).expect("add item");
        let loaded = storage::load_history(&history_path);

        assert_eq!(loaded.items().len(), 1);
        assert_eq!(loaded.items()[0].preview, "persist me");
    }

    #[test]
    fn adding_item_applies_retention_before_persisting() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("history.json");
        let mut app = ClipboardHistoryApp::new_empty(&history_path);

        for index in 0..=HISTORY_LIMIT {
            app.add_item(text_item(&format!("item {index}")))
                .expect("add item");
        }

        let loaded = storage::load_history(&history_path);
        assert_eq!(loaded.items().len(), HISTORY_LIMIT);
        assert_eq!(loaded.items()[0].preview, "item 25");
        assert_eq!(loaded.items().last().unwrap().preview, "item 1");
    }

    #[test]
    fn adding_item_cleans_removed_image_files_when_retention_discards_them() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("history.json");
        let old_image_path = temp_dir.path().join("old-image.png");
        let old_preview_path = temp_dir.path().join("old-image-thumb.png");
        fs::write(&old_image_path, "old image").expect("write old image");
        fs::write(&old_preview_path, "old preview").expect("write old preview");
        let mut app = ClipboardHistoryApp::new_empty(&history_path);

        app.add_item(HistoryItem::image(&old_image_path, &old_preview_path))
            .expect("add old image");
        for index in 0..HISTORY_LIMIT {
            app.add_item(text_item(&format!("new item {index}")))
                .expect("add text item");
        }

        let loaded = storage::load_history(&history_path);
        assert_eq!(loaded.items().len(), HISTORY_LIMIT);
        assert!(!old_image_path.exists());
        assert!(!old_preview_path.exists());
        assert!(
            loaded
                .items()
                .iter()
                .all(|item| item.preview != old_preview_path.to_string_lossy())
        );
    }

    #[test]
    fn adding_image_saves_file_and_persists_image_reference() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("history.json");
        let images_dir = temp_dir.path().join("controlled-images");
        let image =
            ClipboardImage::new([1, 2, 3, 4], ImageFileExtension::Png).expect("valid image");
        let mut app = ClipboardHistoryApp::new_empty_with_paths(&history_path, &images_dir);

        app.add_image(&image).expect("add image");

        let loaded = storage::load_history(&history_path);
        assert_eq!(loaded.items().len(), 1);
        assert!(
            loaded.items()[0]
                .preview
                .starts_with(images_dir.to_str().unwrap())
        );
        let ClipboardContent::Image { path } = &loaded.items()[0].content else {
            panic!("expected image item");
        };
        assert!(path.starts_with(&images_dir));
        assert_eq!(path.extension().unwrap(), "png");
        assert_eq!(fs::read(path).expect("read image"), image.bytes());
    }
}
