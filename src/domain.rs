use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

pub const HISTORY_LIMIT: usize = 25;
pub const MAX_TEXT_BYTES: usize = 1_000_000;
pub const MAX_IMAGE_BYTES: usize = 10_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct History {
    items: Vec<HistoryItem>,
}

impl History {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_items(items: Vec<HistoryItem>) -> Self {
        let mut history = Self { items };
        history.enforce_limit();
        history
    }

    pub fn push(&mut self, item: HistoryItem) -> Vec<HistoryItem> {
        self.items.insert(0, item);
        self.enforce_limit()
    }

    pub fn items(&self) -> &[HistoryItem] {
        &self.items
    }

    pub fn toggle_pin(&mut self, id: Uuid) -> bool {
        let Some(item) = self.items.iter_mut().find(|item| item.id == id) else {
            return false;
        };

        item.pinned = !item.pinned;
        true
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) -> Vec<HistoryItem> {
        std::mem::take(&mut self.items)
    }

    fn enforce_limit(&mut self) -> Vec<HistoryItem> {
        let mut removed = Vec::new();

        while self.items.len() > HISTORY_LIMIT {
            let Some(index) = self.items.iter().rposition(|item| !item.pinned) else {
                break;
            };
            removed.push(self.items.remove(index));
        }

        removed
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: Uuid,
    pub kind: ClipboardKind,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub pinned: bool,
    pub preview: String,
    pub content: ClipboardContent,
}

impl HistoryItem {
    pub fn text(text: impl Into<String>) -> Option<Self> {
        let text = normalize_text(text.into())?;
        let preview = text.lines().next().unwrap_or("").trim().to_string();

        Some(Self {
            id: Uuid::new_v4(),
            kind: ClipboardKind::Text,
            created_at: Utc::now(),
            pinned: false,
            preview,
            content: ClipboardContent::Text { text },
        })
    }

    pub fn image(path: impl Into<PathBuf>, preview: impl Into<PathBuf>) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: ClipboardKind::Image,
            created_at: Utc::now(),
            pinned: false,
            preview: preview.into().to_string_lossy().into_owned(),
            content: ClipboardContent::Image { path: path.into() },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardKind {
    Text,
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClipboardContent {
    Text { text: String },
    Image { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardImage {
    bytes: Vec<u8>,
    extension: ImageFileExtension,
}

impl ClipboardImage {
    pub fn new(bytes: impl Into<Vec<u8>>, extension: ImageFileExtension) -> Option<Self> {
        let bytes = bytes.into();

        if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
            return None;
        }

        Some(Self { bytes, extension })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn extension(&self) -> ImageFileExtension {
        self.extension
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFileExtension {
    Png,
    Jpeg,
}

impl ImageFileExtension {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
}

pub fn normalize_text(text: String) -> Option<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim().to_string();

    if trimmed.is_empty() || trimmed.len() > MAX_TEXT_BYTES {
        None
    } else {
        Some(trimmed)
    }
}

pub fn looks_like_sensitive_text(text: &str) -> bool {
    let Some(normalized) = normalize_text(text.to_string()) else {
        return false;
    };
    let lowercase = normalized.to_ascii_lowercase();

    normalized.contains("BEGIN PRIVATE KEY")
        || normalized.contains("BEGIN OPENSSH PRIVATE KEY")
        || normalized.starts_with("ghp_")
        || normalized.starts_with("github_pat_")
        || normalized.starts_with("sk-")
        || lowercase.contains("password=")
        || lowercase.contains("passwd=")
        || lowercase.contains("secret=")
        || lowercase.contains("api_key=")
        || looks_like_jwt(&normalized)
        || looks_like_totp_code(&normalized)
}

fn looks_like_jwt(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| part.len() >= 8 && part.chars().all(is_base64_url_char))
}

fn is_base64_url_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '-' || character == '_'
}

fn looks_like_totp_code(text: &str) -> bool {
    let code = text.trim();
    (code.len() == 6 || code.len() == 8) && code.chars().all(|character| character.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_item(value: &str) -> HistoryItem {
        HistoryItem::text(value).expect("valid text item")
    }

    #[test]
    fn keeps_newest_item_at_the_top() {
        let mut history = History::new();

        history.push(text_item("first"));
        history.push(text_item("second"));

        assert_eq!(history.items()[0].preview, "second");
        assert_eq!(history.items()[1].preview, "first");
    }

    #[test]
    fn keeps_at_most_twenty_five_items() {
        let mut history = History::new();

        for index in 0..26 {
            history.push(text_item(&format!("item {index}")));
        }

        assert_eq!(history.items().len(), HISTORY_LIMIT);
        assert_eq!(history.items()[0].preview, "item 25");
        assert_eq!(history.items().last().unwrap().preview, "item 1");
    }

    #[test]
    fn returns_removed_items_when_limit_is_exceeded() {
        let mut history = History::new();

        for index in 0..HISTORY_LIMIT {
            let removed = history.push(text_item(&format!("item {index}")));
            assert!(removed.is_empty());
        }

        let removed = history.push(HistoryItem::image(
            "/tmp/latest.png",
            "/tmp/latest-thumb.png",
        ));

        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].preview, "item 0");
    }

    #[test]
    fn pinned_items_are_not_removed_when_limit_is_exceeded() {
        let mut history = History::new();

        history.push(text_item("pinned"));
        let pinned_id = history.items()[0].id;
        assert!(history.toggle_pin(pinned_id));

        for index in 0..HISTORY_LIMIT {
            history.push(text_item(&format!("item {index}")));
        }

        assert_eq!(history.items().len(), HISTORY_LIMIT);
        assert!(history.items().iter().any(|item| item.preview == "pinned"));
        assert!(history.items().iter().all(|item| item.preview != "item 0"));
        assert!(
            history
                .items()
                .iter()
                .find(|item| item.preview == "pinned")
                .expect("pinned item")
                .pinned
        );
    }

    #[test]
    fn toggle_pin_returns_false_for_unknown_item() {
        let mut history = History::new();

        assert!(!history.toggle_pin(Uuid::new_v4()));
    }

    #[test]
    fn clear_returns_removed_items_and_empties_history() {
        let mut history = History::new();
        history.push(text_item("first"));
        history.push(text_item("second"));

        let removed = history.clear();

        assert_eq!(removed.len(), 2);
        assert!(history.is_empty());
    }

    #[test]
    fn rejects_empty_or_whitespace_text() {
        assert!(HistoryItem::text("   \n\t  ").is_none());
    }

    #[test]
    fn rejects_text_payload_above_limit() {
        assert!(HistoryItem::text("A".repeat(MAX_TEXT_BYTES + 1)).is_none());
    }

    #[test]
    fn normalizes_plain_text_line_endings_and_outer_whitespace() {
        let item = HistoryItem::text("  first\r\nsecond\r  ").expect("valid text item");

        assert_eq!(
            item.content,
            ClipboardContent::Text {
                text: "first\nsecond".to_string()
            }
        );
        assert_eq!(item.preview, "first");
    }

    #[test]
    fn detects_sensitive_text_patterns() {
        assert!(looks_like_sensitive_text("ghp_1234567890abcdef"));
        assert!(looks_like_sensitive_text("sk-1234567890abcdef"));
        assert!(looks_like_sensitive_text("-----BEGIN PRIVATE KEY-----"));
        assert!(looks_like_sensitive_text("password=super-secret"));
        assert!(looks_like_sensitive_text("aaaaaaaa.bbbbbbbb.cccccccc"));
        assert!(looks_like_sensitive_text("123456"));
    }

    #[test]
    fn does_not_flag_regular_text_as_sensitive() {
        assert!(!looks_like_sensitive_text("normal clipboard text"));
        assert!(!looks_like_sensitive_text("#AACDDC"));
        assert!(!looks_like_sensitive_text("item 123"));
    }

    #[test]
    fn rejects_empty_image_payload() {
        assert!(ClipboardImage::new(Vec::new(), ImageFileExtension::Png).is_none());
    }

    #[test]
    fn rejects_image_payload_above_limit() {
        assert!(
            ClipboardImage::new(vec![1; MAX_IMAGE_BYTES + 1], ImageFileExtension::Png).is_none()
        );
    }

    #[test]
    fn accepts_non_empty_image_payload_with_safe_extension() {
        let image = ClipboardImage::new([1, 2, 3], ImageFileExtension::Jpeg).expect("valid image");

        assert_eq!(image.bytes(), &[1, 2, 3]);
        assert_eq!(image.extension().as_str(), "jpg");
    }
}
