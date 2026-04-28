use std::path::PathBuf;

const APP_DIR_NAME: &str = "clipboard-history";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopPaths {
    pub data_dir: PathBuf,
    pub history_path: PathBuf,
    pub images_dir: PathBuf,
}

impl DesktopPaths {
    pub fn from_data_dir(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();

        Self {
            history_path: data_dir.join("history.json"),
            images_dir: data_dir.join("images"),
            data_dir,
        }
    }
}

pub fn default_data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME").map_or_else(
        || {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local")
                .join("share")
                .join(APP_DIR_NAME)
        },
        |xdg_data_home| PathBuf::from(xdg_data_home).join(APP_DIR_NAME),
    )
}

pub fn default_paths() -> DesktopPaths {
    DesktopPaths::from_data_dir(default_data_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_paths_from_data_directory() {
        let paths = DesktopPaths::from_data_dir("/tmp/app-data");

        assert_eq!(paths.data_dir, PathBuf::from("/tmp/app-data"));
        assert_eq!(
            paths.history_path,
            PathBuf::from("/tmp/app-data/history.json")
        );
        assert_eq!(paths.images_dir, PathBuf::from("/tmp/app-data/images"));
    }
}
