use crate::domain::{ClipboardContent, ClipboardKind, History, HistoryItem};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn load_history(path: &Path) -> History {
    let Ok(contents) = fs::read_to_string(path) else {
        return History::new();
    };

    serde_json::from_str(&contents).unwrap_or_default()
}

pub fn save_history(path: &Path, history: &History) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = temporary_path(path);
    let contents = serde_json::to_vec_pretty(history).map_err(io::Error::other)?;

    fs::write(&temp_path, contents)?;
    fs::rename(temp_path, path)?;

    Ok(())
}

pub fn cleanup_removed_image_files(items: &[HistoryItem]) -> io::Result<()> {
    for item in items {
        if item.kind != ClipboardKind::Image {
            continue;
        }

        if let ClipboardContent::Image { path } = &item.content {
            remove_file_if_exists(path)?;

            let preview_path = Path::new(&item.preview);
            if preview_path != path {
                remove_file_if_exists(preview_path)?;
            }
        }
    }

    Ok(())
}

fn remove_file_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temp_path = path.to_path_buf();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map_or_else(|| "tmp".to_string(), |extension| format!("{extension}.tmp"));
    temp_path.set_extension(extension);
    temp_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_history_from_json() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("history.json");
        let mut history = History::new();
        history.push(HistoryItem::text("persisted").expect("valid text item"));

        save_history(&path, &history).expect("save history");
        let loaded = load_history(&path);

        assert_eq!(loaded.items().len(), 1);
        assert_eq!(loaded.items()[0].preview, "persisted");
    }

    #[test]
    fn returns_empty_history_when_json_is_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("missing.json");

        let loaded = load_history(&path);

        assert!(loaded.is_empty());
    }

    #[test]
    fn returns_empty_history_when_json_is_corrupted() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("history.json");
        fs::write(&path, "{not-json").expect("write corrupted json");

        let loaded = load_history(&path);

        assert!(loaded.is_empty());
    }

    #[test]
    fn saves_using_a_temporary_file_then_renames_to_final_path() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("nested").join("history.json");
        let mut history = History::new();
        history.push(HistoryItem::text("safe write").expect("valid text item"));

        save_history(&path, &history).expect("save history");

        assert!(path.exists());
        assert!(!temporary_path(&path).exists());
    }

    #[test]
    fn removes_image_files_for_removed_image_items() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let image_path = temp_dir.path().join("image.png");
        let preview_path = temp_dir.path().join("image-thumb.png");
        fs::write(&image_path, "image").expect("write image");
        fs::write(&preview_path, "preview").expect("write preview");
        let removed = vec![HistoryItem::image(&image_path, &preview_path)];

        cleanup_removed_image_files(&removed).expect("cleanup image files");

        assert!(!image_path.exists());
        assert!(!preview_path.exists());
    }

    #[test]
    fn ignores_text_items_when_cleaning_removed_image_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let text_file_path = temp_dir.path().join("text-owned-by-something-else.txt");
        fs::write(&text_file_path, "keep").expect("write text file");
        let removed = vec![HistoryItem::text("plain text").expect("valid text item")];

        cleanup_removed_image_files(&removed).expect("cleanup image files");

        assert!(text_file_path.exists());
    }

    #[test]
    fn treats_missing_image_files_as_already_cleaned() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let image_path = temp_dir.path().join("missing-image.png");
        let preview_path = temp_dir.path().join("missing-preview.png");
        let removed = vec![HistoryItem::image(&image_path, &preview_path)];

        cleanup_removed_image_files(&removed).expect("cleanup missing image files");
    }
}
