use crate::domain::{ClipboardContent, ClipboardImage, ClipboardKind, History, HistoryItem};
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const PRIVATE_DIR_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;

pub fn load_history(path: &Path) -> History {
    let Ok(contents) = fs::read_to_string(path) else {
        return History::new();
    };

    match serde_json::from_str(&contents) {
        Ok(history) => history,
        Err(error) => {
            eprintln!("failed to parse clipboard history; preserving corrupt file: {error}");
            if let Err(preserve_error) = preserve_corrupted_history(path) {
                eprintln!("failed to preserve corrupt clipboard history: {preserve_error}");
            }
            History::new()
        }
    }
}

pub fn save_history(path: &Path, history: &History) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        create_private_dir_all(parent)?;
    }

    let temp_path = temporary_path(path);
    let contents = serde_json::to_vec_pretty(history).map_err(io::Error::other)?;

    write_private_file(&temp_path, &contents)?;
    fs::rename(temp_path, path)?;
    set_private_file_permissions(path)?;

    Ok(())
}

pub fn harden_storage_permissions(
    data_dir: &Path,
    history_path: &Path,
    images_dir: &Path,
) -> io::Result<()> {
    create_private_dir_all(data_dir)?;
    create_private_dir_all(images_dir)?;
    set_private_file_permissions_if_exists(history_path)?;

    if let Ok(entries) = fs::read_dir(images_dir) {
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                set_private_file_permissions(&path)?;
            }
        }
    }

    Ok(())
}

pub fn cleanup_removed_image_files(items: &[HistoryItem], images_dir: &Path) -> io::Result<()> {
    for item in items {
        if item.kind != ClipboardKind::Image {
            continue;
        }

        if let ClipboardContent::Image { path } = &item.content {
            remove_image_file_if_safe(path, images_dir)?;

            let preview_path = Path::new(&item.preview);
            if preview_path != path {
                remove_image_file_if_safe(preview_path, images_dir)?;
            }
        }
    }

    Ok(())
}

pub fn save_clipboard_image(images_dir: &Path, image: &ClipboardImage) -> io::Result<PathBuf> {
    create_private_dir_all(images_dir)?;

    let file_name = format!("{}.{}", Uuid::new_v4(), image.extension().as_str());
    let final_path = images_dir.join(file_name);
    let temp_path = final_path.with_extension(format!("{}.tmp", image.extension().as_str()));

    write_private_file(&temp_path, image.bytes())?;
    fs::rename(temp_path, &final_path)?;
    set_private_file_permissions(&final_path)?;

    Ok(final_path)
}

fn remove_file_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_image_file_if_safe(path: &Path, images_dir: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let canonical_images_dir = fs::canonicalize(images_dir)?;
    let canonical_path = fs::canonicalize(path)?;
    if !canonical_path.starts_with(canonical_images_dir) {
        return Ok(());
    }

    remove_file_if_exists(path)
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

fn preserve_corrupted_history(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_secs();
    let corrupt_path = path.with_file_name(format!("history.json.corrupt.{timestamp}"));
    fs::rename(path, corrupt_path)
}

fn create_private_dir_all(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    set_private_dir_permissions(path)
}

fn write_private_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut file = private_file_options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    set_private_file_permissions(path)
}

fn set_private_file_permissions_if_exists(path: &Path) -> io::Result<()> {
    match set_private_file_permissions(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn private_file_options() -> OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options.mode(PRIVATE_FILE_MODE);
    options
}

#[cfg(not(unix))]
fn private_file_options() -> OpenOptions {
    OpenOptions::new()
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_DIR_MODE))
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_FILE_MODE))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ImageFileExtension;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

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
        assert!(!path.exists());
        assert!(
            fs::read_dir(temp_dir.path())
                .expect("read temp dir")
                .any(|entry| entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .starts_with("history.json.corrupt."))
        );
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

    #[cfg(unix)]
    #[test]
    fn saves_history_with_private_permissions() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("nested").join("history.json");
        let mut history = History::new();
        history.push(HistoryItem::text("private").expect("valid text item"));

        save_history(&path, &history).expect("save history");

        assert_eq!(mode(path.parent().expect("parent")), PRIVATE_DIR_MODE);
        assert_eq!(mode(&path), PRIVATE_FILE_MODE);
    }

    #[test]
    fn removes_image_files_for_removed_image_items() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let image_path = temp_dir.path().join("image.png");
        let preview_path = temp_dir.path().join("image-thumb.png");
        fs::write(&image_path, "image").expect("write image");
        fs::write(&preview_path, "preview").expect("write preview");
        let removed = vec![HistoryItem::image(&image_path, &preview_path)];

        cleanup_removed_image_files(&removed, temp_dir.path()).expect("cleanup image files");

        assert!(!image_path.exists());
        assert!(!preview_path.exists());
    }

    #[test]
    fn ignores_text_items_when_cleaning_removed_image_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let text_file_path = temp_dir.path().join("text-owned-by-something-else.txt");
        fs::write(&text_file_path, "keep").expect("write text file");
        let removed = vec![HistoryItem::text("plain text").expect("valid text item")];

        cleanup_removed_image_files(&removed, temp_dir.path()).expect("cleanup image files");

        assert!(text_file_path.exists());
    }

    #[test]
    fn treats_missing_image_files_as_already_cleaned() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let image_path = temp_dir.path().join("missing-image.png");
        let preview_path = temp_dir.path().join("missing-preview.png");
        let removed = vec![HistoryItem::image(&image_path, &preview_path)];

        cleanup_removed_image_files(&removed, temp_dir.path())
            .expect("cleanup missing image files");
    }

    #[test]
    fn ignores_image_cleanup_paths_outside_controlled_directory() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let images_dir = temp_dir.path().join("images");
        fs::create_dir_all(&images_dir).expect("create images dir");
        let outside_file = temp_dir.path().join("outside.png");
        fs::write(&outside_file, "outside").expect("write outside");
        let removed = vec![HistoryItem::image(&outside_file, &outside_file)];

        cleanup_removed_image_files(&removed, &images_dir).expect("cleanup image files");

        assert!(outside_file.exists());
    }

    #[cfg(unix)]
    #[test]
    fn ignores_image_cleanup_symlink_to_outside_controlled_directory() {
        use std::os::unix::fs::symlink;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let images_dir = temp_dir.path().join("images");
        fs::create_dir_all(&images_dir).expect("create images dir");
        let outside_file = temp_dir.path().join("outside.png");
        fs::write(&outside_file, "outside").expect("write outside");
        let symlink_path = images_dir.join("linked.png");
        symlink(&outside_file, &symlink_path).expect("create symlink");
        let removed = vec![HistoryItem::image(&symlink_path, &symlink_path)];

        cleanup_removed_image_files(&removed, &images_dir).expect("cleanup image files");

        assert!(outside_file.exists());
        assert!(symlink_path.exists());
    }

    #[test]
    fn saves_clipboard_image_inside_controlled_directory() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let images_dir = temp_dir.path().join("images");
        let image =
            ClipboardImage::new([137, 80, 78, 71], ImageFileExtension::Png).expect("valid image");

        let image_path = save_clipboard_image(&images_dir, &image).expect("save image");

        assert!(image_path.starts_with(&images_dir));
        assert_eq!(image_path.extension().unwrap(), "png");
        assert_eq!(fs::read(&image_path).expect("read image"), image.bytes());
    }

    #[test]
    fn saves_clipboard_image_atomically_without_leaving_temp_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let images_dir = temp_dir.path().join("images");
        let image =
            ClipboardImage::new([255, 216, 255], ImageFileExtension::Jpeg).expect("valid image");

        let image_path = save_clipboard_image(&images_dir, &image).expect("save image");
        let temp_path = image_path.with_extension("jpg.tmp");

        assert!(image_path.exists());
        assert!(!temp_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn saves_clipboard_image_with_private_permissions() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let images_dir = temp_dir.path().join("images");
        let image =
            ClipboardImage::new([137, 80, 78, 71], ImageFileExtension::Png).expect("valid image");

        let image_path = save_clipboard_image(&images_dir, &image).expect("save image");

        assert_eq!(mode(&images_dir), PRIVATE_DIR_MODE);
        assert_eq!(mode(&image_path), PRIVATE_FILE_MODE);
    }

    #[cfg(unix)]
    #[test]
    fn hardens_existing_storage_permissions() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let data_dir = temp_dir.path().join("clipboard-history");
        let images_dir = data_dir.join("images");
        let history_path = data_dir.join("history.json");
        fs::create_dir_all(&images_dir).expect("create images dir");
        fs::write(&history_path, "[]").expect("write history");
        let image_path = images_dir.join("image.png");
        fs::write(&image_path, "image").expect("write image");
        fs::set_permissions(&data_dir, fs::Permissions::from_mode(0o775)).expect("chmod data");
        fs::set_permissions(&images_dir, fs::Permissions::from_mode(0o775)).expect("chmod images");
        fs::set_permissions(&history_path, fs::Permissions::from_mode(0o664))
            .expect("chmod history");
        fs::set_permissions(&image_path, fs::Permissions::from_mode(0o664)).expect("chmod image");

        harden_storage_permissions(&data_dir, &history_path, &images_dir)
            .expect("harden permissions");

        assert_eq!(mode(&data_dir), PRIVATE_DIR_MODE);
        assert_eq!(mode(&images_dir), PRIVATE_DIR_MODE);
        assert_eq!(mode(&history_path), PRIVATE_FILE_MODE);
        assert_eq!(mode(&image_path), PRIVATE_FILE_MODE);
    }

    #[cfg(unix)]
    fn mode(path: &Path) -> u32 {
        fs::metadata(path).expect("metadata").permissions().mode() & 0o777
    }
}
