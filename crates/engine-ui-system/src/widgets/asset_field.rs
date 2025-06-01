// AssetField widget for Unity-style asset reference selection

use gtk4::prelude::*;
use gtk4::{Box, Label, Button, Entry, Image, Orientation, Widget};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use crate::{EditorTheme, Themeable};

/// Asset type information
#[derive(Debug, Clone, PartialEq)]
pub struct AssetType {
    pub name: String,
    pub extensions: Vec<String>,
    pub icon_name: String,
}

impl AssetType {
    /// Create new asset type
    pub fn new(name: &str, extensions: Vec<&str>, icon_name: &str) -> Self {
        Self {
            name: name.to_string(),
            extensions: extensions.into_iter().map(|s| s.to_string()).collect(),
            icon_name: icon_name.to_string(),
        }
    }
    
    /// Check if file matches this asset type
    pub fn matches_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            self.extensions.iter().any(|e| e.to_lowercase() == ext_str)
        } else {
            false
        }
    }
}

/// Common asset types
impl AssetType {
    /// Texture/Image asset type
    pub fn texture() -> Self {
        Self::new("Texture", vec!["png", "jpg", "jpeg", "tga", "bmp", "hdr"], "image-x-generic")
    }
    
    /// Material asset type
    pub fn material() -> Self {
        Self::new("Material", vec!["mat"], "applications-graphics")
    }
    
    /// Mesh/Model asset type
    pub fn mesh() -> Self {
        Self::new("Mesh", vec!["obj", "fbx", "gltf", "glb", "dae"], "applications-engineering")
    }
    
    /// Audio asset type
    pub fn audio() -> Self {
        Self::new("Audio", vec!["wav", "mp3", "ogg", "flac"], "audio-x-generic")
    }
    
    /// Script asset type
    pub fn script() -> Self {
        Self::new("Script", vec!["rs", "cs", "js", "lua", "py"], "text-x-script")
    }
    
    /// Prefab asset type
    pub fn prefab() -> Self {
        Self::new("Prefab", vec!["prefab"], "package-x-generic")
    }
    
    /// Scene asset type
    pub fn scene() -> Self {
        Self::new("Scene", vec!["scene"], "folder-documents")
    }
    
    /// Shader asset type
    pub fn shader() -> Self {
        Self::new("Shader", vec!["wgsl", "glsl", "hlsl", "shader"], "text-x-generic")
    }
    
    /// Any asset type (accepts all files)
    pub fn any() -> Self {
        Self::new("Any", vec![], "text-x-generic")
    }
}

/// Asset reference
#[derive(Debug, Clone, PartialEq)]
pub struct AssetRef {
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub display_name: String,
}

impl AssetRef {
    /// Create new asset reference
    pub fn new(path: PathBuf, asset_type: AssetType) -> Self {
        let display_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
            
        Self {
            path,
            asset_type,
            display_name,
        }
    }
    
    /// Get file name
    pub fn file_name(&self) -> String {
        self.display_name.clone()
    }
    
    /// Get file extension
    pub fn extension(&self) -> Option<String> {
        self.path.extension()
            .map(|ext| ext.to_string_lossy().to_string())
    }
    
    /// Check if asset exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

/// Unity-style asset field for selecting asset references
pub struct AssetField {
    container: Box,
    label: Option<Label>,
    asset_container: Box,
    asset_icon: Image,
    asset_name: Entry,
    browse_button: Button,
    clear_button: Button,
    asset_type: AssetType,
    current_asset: Option<AssetRef>,
    theme: Rc<RefCell<EditorTheme>>,
    on_changed: Option<std::boxed::Box<dyn Fn(Option<&AssetRef>)>>,
    on_browse: Option<std::boxed::Box<dyn Fn(&AssetType) -> Option<PathBuf>>>,
}

impl AssetField {
    /// Create new asset field
    pub fn new(
        label: Option<&str>,
        asset_type: AssetType,
        initial_asset: Option<AssetRef>,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();
        
        // Optional label
        let field_label = if let Some(label_text) = label {
            let label = Label::builder()
                .label(label_text)
                .halign(gtk4::Align::Start)
                .build();
            container.append(&label);
            Some(label)
        } else {
            None
        };
        
        // Asset container (horizontal layout)
        let asset_container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();
        
        // Asset icon
        let asset_icon = Image::builder()
            .icon_name(&asset_type.icon_name)
            .pixel_size(20)
            .build();
        
        // Asset name entry (read-only display)
        let asset_name = Entry::builder()
            .editable(false)
            .placeholder_text("None")
            .build();
        
        // Browse button
        let browse_button = Button::builder()
            .icon_name("document-open")
            .tooltip_text("Browse for asset")
            .build();
        
        // Clear button
        let clear_button = Button::builder()
            .icon_name("edit-clear")
            .tooltip_text("Clear asset reference")
            .build();
        
        asset_container.append(&asset_icon);
        asset_container.append(&asset_name);
        asset_container.append(&browse_button);
        asset_container.append(&clear_button);
        
        container.append(&asset_container);
        
        let mut field = Self {
            container,
            label: field_label,
            asset_container,
            asset_icon,
            asset_name,
            browse_button,
            clear_button,
            asset_type,
            current_asset: initial_asset.clone(),
            theme,
            on_changed: None,
            on_browse: None,
        };
        
        if let Some(asset) = initial_asset {
            field.set_asset(Some(asset));
        }
        
        field.setup_styling();
        field.setup_handlers();
        field
    }
    
    /// Create asset field without label
    pub fn without_label(
        asset_type: AssetType,
        initial_asset: Option<AssetRef>,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        Self::new(None, asset_type, initial_asset, theme)
    }
    
    /// Create asset field with label
    pub fn with_label(
        label: &str,
        asset_type: AssetType,
        initial_asset: Option<AssetRef>,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        Self::new(Some(label), asset_type, initial_asset, theme)
    }
    
    /// Get current asset
    pub fn asset(&self) -> Option<&AssetRef> {
        self.current_asset.as_ref()
    }
    
    /// Set asset
    pub fn set_asset(&mut self, asset: Option<AssetRef>) {
        self.current_asset = asset.clone();
        
        if let Some(asset_ref) = &asset {
            self.asset_name.set_text(&asset_ref.display_name);
            self.asset_icon.set_icon_name(Some(&asset_ref.asset_type.icon_name));
            self.clear_button.set_visible(true);
        } else {
            self.asset_name.set_text("");
            self.asset_icon.set_icon_name(Some(&self.asset_type.icon_name));
            self.clear_button.set_visible(false);
        }
        
        if let Some(callback) = &self.on_changed {
            callback(self.current_asset.as_ref());
        }
    }
    
    /// Set change handler
    pub fn set_on_changed<F>(&mut self, callback: F)
    where
        F: Fn(Option<&AssetRef>) + 'static,
    {
        self.on_changed = Some(std::boxed::Box::new(callback));
    }
    
    /// Set browse handler (for opening file dialogs)
    pub fn set_on_browse<F>(&mut self, callback: F)
    where
        F: Fn(&AssetType) -> Option<PathBuf> + 'static,
    {
        self.on_browse = Some(std::boxed::Box::new(callback));
    }
    
    /// Set field label
    pub fn set_label(&mut self, text: Option<&str>) {
        if let Some(label_text) = text {
            if let Some(label) = &self.label {
                label.set_text(label_text);
            } else {
                // Create new label and insert at top
                let label = Label::builder()
                    .label(label_text)
                    .halign(gtk4::Align::Start)
                    .build();
                self.container.prepend(&label);
                self.label = Some(label);
            }
        } else if let Some(label) = &self.label {
            self.container.remove(label);
            self.label = None;
        }
    }
    
    /// Set accepted asset type
    pub fn set_asset_type(&mut self, asset_type: AssetType) {
        self.asset_type = asset_type;
        
        // Update icon if no asset is selected
        if self.current_asset.is_none() {
            self.asset_icon.set_icon_name(Some(&self.asset_type.icon_name));
        }
    }
    
    /// Enable/disable the field
    pub fn set_enabled(&self, enabled: bool) {
        self.browse_button.set_sensitive(enabled);
        self.clear_button.set_sensitive(enabled);
    }
    
    /// Get underlying GTK widget
    pub fn widget(&self) -> &Box {
        &self.container
    }
    
    /// Get widget as generic Widget
    pub fn as_widget(&self) -> Widget {
        self.container.clone().upcast()
    }
    
    /// Setup initial styling
    fn setup_styling(&mut self) {
        let style_context = self.container.style_context();
        style_context.add_class("asset-field");
        
        let asset_context = self.asset_container.style_context();
        asset_context.add_class("asset-container");
        
        let name_context = self.asset_name.style_context();
        name_context.add_class("asset-name");
        
        // Style label if present
        if let Some(label) = &self.label {
            let label_context = label.style_context();
            label_context.add_class("field-label");
        }
        
        // Initially hide clear button if no asset
        if self.current_asset.is_none() {
            self.clear_button.set_visible(false);
        }
        
        self.apply_current_theme();
    }
    
    /// Setup button handlers
    fn setup_handlers(&mut self) {
        // Browse button handler
        self.browse_button.connect_clicked(move |_| {
            // In a real implementation, this would trigger the browse callback
            // and handle file selection
        });
        
        // Clear button handler
        self.clear_button.connect_clicked(move |_| {
            // In a real implementation, this would clear the asset
        });
    }
    
    /// Apply current theme
    fn apply_current_theme(&mut self) {
        let theme = self.theme.borrow().clone();
        self.apply_theme(&theme);
    }
}

impl Themeable for AssetField {
    fn apply_theme(&mut self, theme: &EditorTheme) {
        // Set field dimensions
        let height = theme.sizes.input_height_md as i32;
        self.asset_container.set_size_request(-1, height);
        
        // Set icon size
        let icon_size = theme.sizes.icon_md as i32;
        self.asset_icon.set_pixel_size(icon_size);
        
        // Apply spacing
        let spacing = theme.spacing.xs as i32;
        self.container.set_spacing(spacing);
        self.asset_container.set_spacing(spacing);
    }
    
    fn get_css(&self, theme: &EditorTheme) -> String {
        format!(
            r#"
            .asset-field {{
                margin: {margin}px;
            }}
            
            .asset-field .field-label {{
                color: {label_color};
                font-family: "{font_family}";
                font-size: {font_size}px;
                font-weight: {font_weight};
                margin-bottom: {label_margin}px;
            }}
            
            .asset-container {{
                background-color: {bg_color};
                border: {border_width}px solid {border_color};
                border-radius: {border_radius}px;
                padding: {padding}px;
                min-height: {height}px;
            }}
            
            .asset-container:focus-within {{
                border-color: {focus_color};
                box-shadow: 0 0 0 2px {focus_color}33;
            }}
            
            .asset-name {{
                background-color: transparent;
                border: none;
                color: {text_color};
                font-family: "{font_family}";
                font-size: {input_font_size}px;
                flex: 1;
            }}
            
            .asset-name:focus {{
                outline: none;
                box-shadow: none;
            }}
            
            .asset-field button {{
                background-color: {button_bg};
                border: {border_width}px solid {button_border};
                border-radius: {button_radius}px;
                padding: {button_padding}px;
                min-width: {button_size}px;
                min-height: {button_size}px;
            }}
            
            .asset-field button:hover {{
                background-color: {button_hover};
            }}
            
            .asset-field button:disabled {{
                background-color: {button_disabled};
                color: {disabled_text};
            }}
            "#,
            margin = theme.spacing.xs,
            label_color = theme.colors.text_primary.to_hex(),
            font_family = theme.typography.font_family_primary,
            font_size = theme.typography.sizes.sm,
            font_weight = theme.typography.weights.medium,
            label_margin = theme.spacing.xs,
            bg_color = theme.colors.surface.to_hex(),
            border_color = theme.colors.border.to_hex(),
            border_width = theme.sizes.border_width,
            border_radius = theme.sizes.border_radius_sm,
            padding = theme.spacing.xs,
            height = theme.sizes.input_height_md,
            focus_color = theme.colors.border_focus.to_hex(),
            text_color = theme.colors.text_primary.to_hex(),
            input_font_size = theme.typography.sizes.base,
            button_bg = theme.colors.surface_elevated.to_hex(),
            button_border = theme.colors.border.to_hex(),
            button_radius = theme.sizes.border_radius_sm,
            button_padding = theme.spacing.xs,
            button_size = theme.sizes.button_height_sm,
            button_hover = theme.colors.surface_hover.to_hex(),
            button_disabled = theme.colors.surface.darken(0.1).to_hex(),
            disabled_text = theme.colors.text_disabled.to_hex(),
        )
    }
}

/// Helper functions for common asset field types
impl AssetField {
    /// Create texture asset field
    pub fn texture(label: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label(label, AssetType::texture(), None, theme)
    }
    
    /// Create material asset field
    pub fn material(label: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label(label, AssetType::material(), None, theme)
    }
    
    /// Create mesh asset field
    pub fn mesh(label: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label(label, AssetType::mesh(), None, theme)
    }
    
    /// Create audio asset field
    pub fn audio(label: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label(label, AssetType::audio(), None, theme)
    }
    
    /// Create script asset field
    pub fn script(label: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label(label, AssetType::script(), None, theme)
    }
    
    /// Create prefab asset field
    pub fn prefab(label: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label(label, AssetType::prefab(), None, theme)
    }
    
    /// Create scene asset field
    pub fn scene(label: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label(label, AssetType::scene(), None, theme)
    }
    
    /// Create any-type asset field
    pub fn any(label: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label(label, AssetType::any(), None, theme)
    }
}

/// Specialized asset picker dialog (would integrate with file system)
pub struct AssetPicker {
    asset_type: AssetType,
    current_directory: PathBuf,
}

impl AssetPicker {
    /// Create new asset picker
    pub fn new(asset_type: AssetType, starting_directory: PathBuf) -> Self {
        Self {
            asset_type,
            current_directory: starting_directory,
        }
    }
    
    /// Show picker dialog and return selected asset
    pub fn pick_asset(&self) -> Option<AssetRef> {
        // In a real implementation, this would show a file dialog
        // filtered by the asset type's extensions
        None
    }
    
    /// Validate that a path is acceptable for this asset type
    pub fn validate_path(&self, path: &Path) -> bool {
        if self.asset_type.extensions.is_empty() {
            // Any type accepts all files
            return true;
        }
        
        self.asset_type.matches_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;

    fn create_test_theme() -> Rc<RefCell<EditorTheme>> {
        Rc::new(RefCell::new(EditorTheme::unity_dark()))
    }

    #[test]
    fn test_asset_type_creation() {
        let texture_type = AssetType::texture();
        assert_eq!(texture_type.name, "Texture");
        assert!(texture_type.extensions.contains(&"png".to_string()));
        
        let path = Path::new("test.png");
        assert!(texture_type.matches_file(path));
        
        let wrong_path = Path::new("test.txt");
        assert!(!texture_type.matches_file(wrong_path));
    }

    #[test]
    fn test_asset_ref_creation() {
        let path = PathBuf::from("assets/texture.png");
        let asset_type = AssetType::texture();
        let asset_ref = AssetRef::new(path.clone(), asset_type);
        
        assert_eq!(asset_ref.path, path);
        assert_eq!(asset_ref.file_name(), "texture.png");
        assert_eq!(asset_ref.extension(), Some("png".to_string()));
    }

    #[test]
    fn test_asset_field_creation() {
        let theme = create_test_theme();
        let field = AssetField::new(
            Some("Texture"),
            AssetType::texture(),
            None,
            theme,
        );
        
        assert!(field.asset().is_none());
    }

    #[test]
    fn test_specialized_asset_fields() {
        let theme = create_test_theme();
        
        let texture_field = AssetField::texture("Albedo", theme.clone());
        assert_eq!(texture_field.asset_type.name, "Texture");
        
        let material_field = AssetField::material("Material", theme);
        assert_eq!(material_field.asset_type.name, "Material");
    }

    #[test]
    fn test_asset_picker() {
        let picker = AssetPicker::new(
            AssetType::texture(),
            PathBuf::from("assets"),
        );
        
        assert!(picker.validate_path(Path::new("test.png")));
        assert!(!picker.validate_path(Path::new("test.txt")));
    }

    #[test]
    fn test_css_generation() {
        let theme = create_test_theme();
        let mut field = AssetField::texture("Test", theme.clone());
        
        let css = field.get_css(&theme.borrow());
        assert!(css.contains("asset-field"));
        assert!(css.contains("asset-container"));
    }
}