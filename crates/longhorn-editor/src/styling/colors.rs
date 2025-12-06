use egui::Color32;

/// Longhorn Game Engine color palette with grayscale hierarchy
/// Darker values to match v1 appearance
pub struct Colors;

impl Colors {
    // ============================================
    // TEXT COLORS - Hierarchical text styling
    // ============================================

    /// Primary text - main content, headings
    pub const TEXT_PRIMARY: Color32 = Color32::from_gray(200);
    /// Secondary text - labels, descriptions
    pub const TEXT_SECONDARY: Color32 = Color32::from_gray(160);
    /// Tertiary text - hints, placeholders
    pub const TEXT_TERTIARY: Color32 = Color32::from_gray(120);
    /// Disabled text - inactive elements
    pub const TEXT_DISABLED: Color32 = Color32::from_gray(80);
    /// Muted text - empty states, subtle info
    pub const TEXT_MUTED: Color32 = Color32::from_gray(100);
    /// Bright text - on accent backgrounds
    pub const TEXT_ON_ACCENT: Color32 = Color32::WHITE;

    // ============================================
    // BACKGROUND COLORS - From darkest to lightest
    // ============================================

    /// Extreme dark - deepest backgrounds, tree panels
    pub const BG_EXTREME: Color32 = Color32::from_gray(20);
    /// Panel background - standard panel fill
    pub const BG_PANEL: Color32 = Color32::from_gray(30);
    /// Window background - floating windows, dialogs
    pub const BG_WINDOW: Color32 = Color32::from_gray(35);
    /// Widget inactive - unfocused inputs
    pub const BG_WIDGET_INACTIVE: Color32 = Color32::from_gray(38);
    /// Widget default - standard interactive elements
    pub const BG_WIDGET_DEFAULT: Color32 = Color32::from_gray(40);
    /// Widget hovered - hover state for interactive elements
    pub const BG_WIDGET_HOVERED: Color32 = Color32::from_gray(50);
    /// Viewport placeholder - empty viewport background
    pub const BG_VIEWPORT: Color32 = Color32::from_gray(30);

    // ============================================
    // BORDER/STROKE COLORS
    // ============================================

    /// Dark stroke - subtle separators
    pub const STROKE_DARK: Color32 = Color32::from_gray(15);
    /// Default stroke - standard borders
    pub const STROKE_DEFAULT: Color32 = Color32::from_gray(25);
    /// Hovered stroke - hover state borders
    pub const STROKE_HOVERED: Color32 = Color32::from_gray(70);
    /// Active stroke - focused/active borders
    pub const STROKE_ACTIVE: Color32 = Color32::from_gray(100);

    // ============================================
    // ACCENT COLORS - Primary brand colors
    // ============================================

    /// Primary accent - buttons, active tabs, selection
    pub const ACCENT: Color32 = Color32::from_rgb(50, 110, 180);
    /// Muted accent - secondary actions
    pub const ACCENT_MUTED: Color32 = Color32::from_rgb(44, 93, 135);
    /// Selection background - selected items
    pub const SELECTION_BG: Color32 = Color32::from_rgb(50, 110, 180);
    /// Active state background
    pub const ACTIVE_BG: Color32 = Color32::from_gray(35);

    // ============================================
    // SEMANTIC COLORS - Status and feedback
    // ============================================

    // Error colors (red spectrum)
    /// Error primary - error headings, icons
    pub const ERROR: Color32 = Color32::from_rgb(255, 100, 100);
    /// Error bright - error indicators, dots
    pub const ERROR_BRIGHT: Color32 = Color32::from_rgb(255, 80, 80);
    /// Error text - error labels
    pub const ERROR_TEXT: Color32 = Color32::from_rgb(255, 150, 150);
    /// Error muted - error descriptions
    pub const ERROR_MUTED: Color32 = Color32::from_rgb(255, 200, 200);
    /// Error background - error panels
    pub const ERROR_BG: Color32 = Color32::from_rgb(60, 30, 30);

    // Warning colors (yellow/orange spectrum)
    /// Warning primary - warning headings, icons
    pub const WARNING: Color32 = Color32::from_rgb(255, 200, 80);
    /// Warning text - warning messages
    pub const WARNING_TEXT: Color32 = Color32::from_rgb(255, 220, 120);
    /// Warning background - warning panels
    pub const WARNING_BG: Color32 = Color32::from_rgb(60, 50, 30);

    // Success colors (green spectrum)
    /// Success primary - success headings, icons
    pub const SUCCESS: Color32 = Color32::from_rgb(80, 200, 120);
    /// Success text - success messages
    pub const SUCCESS_TEXT: Color32 = Color32::from_rgb(120, 220, 150);
    /// Success background - success panels
    pub const SUCCESS_BG: Color32 = Color32::from_rgb(30, 50, 35);

    // Info colors (blue spectrum, distinct from accent)
    /// Info primary - info headings, icons
    pub const INFO: Color32 = Color32::from_rgb(100, 180, 255);
    /// Info text - info messages
    pub const INFO_TEXT: Color32 = Color32::from_rgb(150, 200, 255);
    /// Info background - info panels
    pub const INFO_BG: Color32 = Color32::from_rgb(30, 40, 55);
}

/// Stroke helper methods
impl Colors {
    pub fn stroke_default() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_DEFAULT)
    }

    pub fn stroke_dark() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_DARK)
    }

    pub fn stroke_hovered() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_HOVERED)
    }

    pub fn stroke_active() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_ACTIVE)
    }
}

/// Border radius constants for consistent rounding
pub struct Radius;

impl Radius {
    /// No rounding - sharp corners
    pub const NONE: f32 = 0.0;
    /// Small rounding - buttons, inputs
    pub const SMALL: f32 = 2.0;
    /// Medium rounding - cards, tabs
    pub const MEDIUM: f32 = 4.0;
    /// Large rounding - dialogs, panels
    pub const LARGE: f32 = 8.0;
    /// Full rounding - pills, badges
    pub const FULL: f32 = 999.0;
}

impl Radius {
    /// Create egui::Rounding with same radius on all corners
    pub fn all(radius: f32) -> egui::Rounding {
        egui::Rounding::same(radius)
    }
}
