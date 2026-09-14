//! Persisted, provider-neutral font identity. Paths are managed relative paths.
use serde::{Deserialize, Serialize};

pub const TEXT_LAYOUT_PROFILE: &str = "opencut-text-v2";
pub const MAX_FONT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_FONT_FILES: usize = 256;
pub const MAX_TOTAL_FONT_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_FONT_CANDIDATES: usize = 4096;
pub const MAX_FONT_DIRECTORY_DEPTH: usize = 8;
pub const MAX_SHAPED_GLYPHS: usize = 16384;
pub const MAX_TEXT_LINES: usize = 4096;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FontRecord {
    pub sha256: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub face_index: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FontBinding {
    pub profile: String,
    pub regular: String,
    pub bold: String,
    pub italic: String,
    pub bold_italic: String,
    pub warnings: Vec<String>,
}

impl FontBinding {
    pub fn face_hash(&self, bold: bool, italic: bool) -> &str {
        match (bold, italic) {
            (false, false) => &self.regular,
            (true, false) => &self.bold,
            (false, true) => &self.italic,
            (true, true) => &self.bold_italic,
        }
    }

    pub fn hashes(&self) -> [&str; 4] {
        [&self.regular, &self.bold, &self.italic, &self.bold_italic]
    }
}
