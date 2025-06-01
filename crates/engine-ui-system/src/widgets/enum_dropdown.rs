// EnumDropdown widget for Unity-style enum selection

use gtk4::prelude::*;
use gtk4::{ComboBoxText, Widget, Label, Box, Orientation};
use std::rc::Rc;
use std::cell::RefCell;
use crate::{EditorTheme, Themeable};

/// Trait for types that can be displayed in enum dropdown
pub trait EnumOption: Clone + PartialEq {
    /// Get display name for this option
    fn display_name(&self) -> String;
    
    /// Get all possible options
    fn all_options() -> Vec<Self>;
    
    /// Get option by index
    fn from_index(index: usize) -> Option<Self> {
        Self::all_options().get(index).cloned()
    }
    
    /// Get index of this option
    fn to_index(&self) -> Option<usize> {
        Self::all_options().iter().position(|opt| opt == self)
    }
}

/// Dropdown widget for enum selection
pub struct EnumDropdown<T: EnumOption> {
    container: Box,
    label: Option<Label>,
    dropdown: ComboBoxText,
    options: Vec<T>,
    current_value: Option<T>,
    theme: Rc<RefCell<EditorTheme>>,
    on_changed: Option<std::boxed::Box<dyn Fn(Option<&T>)>>,
}

impl<T: EnumOption + 'static> EnumDropdown<T> {
    /// Create new enum dropdown
    pub fn new(
        label: Option<&str>,
        initial_value: Option<T>,
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
        
        // Dropdown
        let dropdown = ComboBoxText::builder()
            .build();
        
        container.append(&dropdown);
        
        let options = T::all_options();
        let mut enum_dropdown = Self {
            container,
            label: field_label,
            dropdown,
            options: options.clone(),
            current_value: initial_value.clone(),
            theme,
            on_changed: None,
        };
        
        enum_dropdown.populate_options(&options);
        
        if let Some(value) = initial_value {
            enum_dropdown.set_selected_value(Some(value));
        }
        
        enum_dropdown.setup_styling();
        enum_dropdown.setup_handlers();
        enum_dropdown
    }
    
    /// Create dropdown without label
    pub fn without_label(initial_value: Option<T>, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(None, initial_value, theme)
    }
    
    /// Create dropdown with label
    pub fn with_label(label: &str, initial_value: Option<T>, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(Some(label), initial_value, theme)
    }
    
    /// Get currently selected value
    pub fn selected_value(&self) -> Option<&T> {
        self.current_value.as_ref()
    }
    
    /// Set selected value
    pub fn set_selected_value(&mut self, value: Option<T>) {
        self.current_value = value.clone();
        
        if let Some(val) = value {
            if let Some(index) = val.to_index() {
                self.dropdown.set_active(Some(index as u32));
            }
        } else {
            self.dropdown.set_active(None);
        }
    }
    
    /// Set change handler
    pub fn set_on_changed<F>(&mut self, callback: F)
    where
        F: Fn(Option<&T>) + 'static,
    {
        self.on_changed = Some(std::boxed::Box::new(callback));
    }
    
    /// Add custom option (if T supports it)
    pub fn add_custom_option(&mut self, option: T) {
        self.options.push(option.clone());
        self.dropdown.append_text(&option.display_name());
    }
    
    /// Set dropdown label
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
    
    /// Enable/disable the dropdown
    pub fn set_enabled(&self, enabled: bool) {
        self.dropdown.set_sensitive(enabled);
    }
    
    /// Get underlying GTK widget
    pub fn widget(&self) -> &Box {
        &self.container
    }
    
    /// Get widget as generic Widget
    pub fn as_widget(&self) -> Widget {
        self.container.clone().upcast()
    }
    
    /// Get dropdown widget
    pub fn dropdown(&self) -> &ComboBoxText {
        &self.dropdown
    }
    
    /// Populate dropdown with options
    fn populate_options(&self, options: &[T]) {
        self.dropdown.remove_all();
        for option in options {
            self.dropdown.append_text(&option.display_name());
        }
    }
    
    /// Setup initial styling
    fn setup_styling(&mut self) {
        let style_context = self.container.style_context();
        style_context.add_class("enum-dropdown");
        
        let dropdown_context = self.dropdown.style_context();
        dropdown_context.add_class("enum-dropdown-widget");
        
        // Style label if present
        if let Some(label) = &self.label {
            let label_context = label.style_context();
            label_context.add_class("field-label");
        }
        
        self.apply_current_theme();
    }
    
    /// Setup change handlers
    fn setup_handlers(&mut self) {
        // Note: In a real implementation, we'd need to handle the callback
        // differently to avoid borrowing issues
        self.dropdown.connect_changed(move |dropdown| {
            if let Some(_active) = dropdown.active() {
                // Update current value and trigger callback
                // This requires additional state management
            }
        });
    }
    
    /// Apply current theme
    fn apply_current_theme(&mut self) {
        let theme = self.theme.borrow().clone();
        self.apply_theme(&theme);
    }
}

impl<T: EnumOption> Themeable for EnumDropdown<T> {
    fn apply_theme(&mut self, theme: &EditorTheme) {
        // Set dropdown dimensions
        self.dropdown.set_size_request(-1, theme.sizes.input_height_md as i32);
        
        // Apply spacing
        let spacing = theme.spacing.xs as i32;
        self.container.set_spacing(spacing);
    }
    
    fn get_css(&self, theme: &EditorTheme) -> String {
        format!(
            r#"
            .enum-dropdown {{
                margin: {margin}px;
            }}
            
            .enum-dropdown .field-label {{
                color: {label_color};
                font-family: "{font_family}";
                font-size: {font_size}px;
                font-weight: {font_weight};
                margin-bottom: {label_margin}px;
            }}
            
            .enum-dropdown-widget {{
                background-color: {bg_color};
                color: {text_color};
                border: {border_width}px solid {border_color};
                border-radius: {border_radius}px;
                padding: 0 {padding}px;
                font-family: "{font_family}";
                font-size: {input_font_size}px;
                min-height: {height}px;
            }}
            
            .enum-dropdown-widget:focus {{
                border-color: {focus_color};
                outline: none;
                box-shadow: 0 0 0 2px {focus_color}33;
            }}
            
            .enum-dropdown-widget:disabled {{
                background-color: {disabled_bg};
                color: {disabled_text};
                border-color: {disabled_border};
            }}
            "#,
            margin = theme.spacing.xs,
            label_color = theme.colors.text_primary.to_hex(),
            font_family = theme.typography.font_family_primary,
            font_size = theme.typography.sizes.sm,
            font_weight = theme.typography.weights.medium,
            label_margin = theme.spacing.xs,
            bg_color = theme.colors.surface.to_hex(),
            text_color = theme.colors.text_primary.to_hex(),
            border_color = theme.colors.border.to_hex(),
            border_width = theme.sizes.border_width,
            border_radius = theme.sizes.border_radius_sm,
            padding = theme.spacing.sm,
            input_font_size = theme.typography.sizes.base,
            height = theme.sizes.input_height_md,
            focus_color = theme.colors.border_focus.to_hex(),
            disabled_bg = theme.colors.surface.darken(0.1).to_hex(),
            disabled_text = theme.colors.text_disabled.to_hex(),
            disabled_border = theme.colors.border.darken(0.1).to_hex(),
        )
    }
}

// Example enum implementations

/// Render mode enum for materials
#[derive(Debug, Clone, PartialEq)]
pub enum RenderMode {
    Opaque,
    Cutout,
    Fade,
    Transparent,
}

impl EnumOption for RenderMode {
    fn display_name(&self) -> String {
        match self {
            RenderMode::Opaque => "Opaque".to_string(),
            RenderMode::Cutout => "Cutout".to_string(),
            RenderMode::Fade => "Fade".to_string(),
            RenderMode::Transparent => "Transparent".to_string(),
        }
    }
    
    fn all_options() -> Vec<Self> {
        vec![
            RenderMode::Opaque,
            RenderMode::Cutout,
            RenderMode::Fade,
            RenderMode::Transparent,
        ]
    }
}

/// Light type enum
#[derive(Debug, Clone, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Area,
}

impl EnumOption for LightType {
    fn display_name(&self) -> String {
        match self {
            LightType::Directional => "Directional".to_string(),
            LightType::Point => "Point".to_string(),
            LightType::Spot => "Spot".to_string(),
            LightType::Area => "Area".to_string(),
        }
    }
    
    fn all_options() -> Vec<Self> {
        vec![
            LightType::Directional,
            LightType::Point,
            LightType::Spot,
            LightType::Area,
        ]
    }
}

/// Camera projection enum
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionType {
    Perspective,
    Orthographic,
}

impl EnumOption for ProjectionType {
    fn display_name(&self) -> String {
        match self {
            ProjectionType::Perspective => "Perspective".to_string(),
            ProjectionType::Orthographic => "Orthographic".to_string(),
        }
    }
    
    fn all_options() -> Vec<Self> {
        vec![
            ProjectionType::Perspective,
            ProjectionType::Orthographic,
        ]
    }
}

/// Texture filter mode enum
#[derive(Debug, Clone, PartialEq)]
pub enum FilterMode {
    Point,
    Bilinear,
    Trilinear,
}

impl EnumOption for FilterMode {
    fn display_name(&self) -> String {
        match self {
            FilterMode::Point => "Point (no filter)".to_string(),
            FilterMode::Bilinear => "Bilinear".to_string(),
            FilterMode::Trilinear => "Trilinear".to_string(),
        }
    }
    
    fn all_options() -> Vec<Self> {
        vec![
            FilterMode::Point,
            FilterMode::Bilinear,
            FilterMode::Trilinear,
        ]
    }
}

/// Helper functions for common enum dropdowns
impl<T: EnumOption + 'static> EnumDropdown<T> {
    /// Create a simple enum dropdown with first option selected
    pub fn with_default(label: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        let default_value = T::all_options().into_iter().next();
        Self::with_label(label, default_value, theme)
    }
}

// Specialized dropdown constructors
impl EnumDropdown<RenderMode> {
    /// Create render mode dropdown
    pub fn render_mode(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label("Rendering Mode", Some(RenderMode::Opaque), theme)
    }
}

impl EnumDropdown<LightType> {
    /// Create light type dropdown
    pub fn light_type(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label("Type", Some(LightType::Directional), theme)
    }
}

impl EnumDropdown<ProjectionType> {
    /// Create projection type dropdown
    pub fn projection_type(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label("Projection", Some(ProjectionType::Perspective), theme)
    }
}

impl EnumDropdown<FilterMode> {
    /// Create filter mode dropdown
    pub fn filter_mode(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label("Filter Mode", Some(FilterMode::Bilinear), theme)
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
    fn test_enum_option_trait() {
        let options = RenderMode::all_options();
        assert_eq!(options.len(), 4);
        
        let opaque = RenderMode::Opaque;
        assert_eq!(opaque.display_name(), "Opaque");
        assert_eq!(opaque.to_index(), Some(0));
        
        let from_index = RenderMode::from_index(1).unwrap();
        assert_eq!(from_index, RenderMode::Cutout);
    }

    #[test]
    fn test_enum_dropdown_creation() {
        let theme = create_test_theme();
        let dropdown = EnumDropdown::new(
            Some("Test"),
            Some(RenderMode::Opaque),
            theme,
        );
        
        assert_eq!(dropdown.selected_value(), Some(&RenderMode::Opaque));
    }

    #[test]
    fn test_specialized_dropdowns() {
        let theme = create_test_theme();
        
        let render_mode = EnumDropdown::render_mode(theme.clone());
        assert_eq!(render_mode.selected_value(), Some(&RenderMode::Opaque));
        
        let light_type = EnumDropdown::light_type(theme);
        assert_eq!(light_type.selected_value(), Some(&LightType::Directional));
    }

    #[test]
    fn test_css_generation() {
        let theme = create_test_theme();
        let mut dropdown = EnumDropdown::render_mode(theme.clone());
        
        let css = dropdown.get_css(&theme.borrow());
        assert!(css.contains("enum-dropdown"));
        assert!(css.contains("enum-dropdown-widget"));
    }

    #[test]
    fn test_value_setting() {
        let theme = create_test_theme();
        let mut dropdown = EnumDropdown::render_mode(theme);
        
        dropdown.set_selected_value(Some(RenderMode::Transparent));
        assert_eq!(dropdown.selected_value(), Some(&RenderMode::Transparent));
        
        dropdown.set_selected_value(None);
        assert_eq!(dropdown.selected_value(), None);
    }
}