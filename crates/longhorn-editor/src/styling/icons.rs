/// Unicode icons for consistent UI across the editor
/// Using simple Unicode characters that render well in most fonts
pub struct Icons;

impl Icons {
    // Navigation/Structure
    pub const FOLDER_CLOSED: &'static str = "\u{25B6}"; // ▶
    pub const FOLDER_OPEN: &'static str = "\u{25BC}";   // ▼
    pub const FOLDER: &'static str = "\u{1F4C1}";       // 📁

    // File types
    pub const FILE_SCRIPT: &'static str = "\u{1F4DC}";  // 📜
    pub const FILE_IMAGE: &'static str = "\u{1F5BC}";   // 🖼
    pub const FILE_AUDIO: &'static str = "\u{1F3B5}";   // 🎵
    pub const FILE_SCENE: &'static str = "\u{1F3AC}";   // 🎬
    pub const FILE_GENERIC: &'static str = "\u{1F4C4}"; // 📄

    // Actions
    pub const REFRESH: &'static str = "\u{21BB}";       // ↻
    pub const PLUS: &'static str = "+";
    pub const MINUS: &'static str = "-";

    // Breadcrumb
    pub const CHEVRON_RIGHT: &'static str = "\u{203A}"; // ›
    pub const HOME: &'static str = "\u{1F3E0}";         // 🏠
}
