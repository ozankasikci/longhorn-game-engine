// Vector3Field widget for Unity-style 3D vector editing

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Widget};
use std::rc::Rc;
use std::cell::RefCell;
use crate::{EditorTheme, Themeable, EditorInput, InputSize, InputVariant};

/// 3D Vector value
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    /// Create new vector
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    /// Zero vector
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
    
    /// One vector
    pub fn one() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
    
    /// Forward vector
    pub fn forward() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }
    
    /// Up vector
    pub fn up() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }
    
    /// Right vector
    pub fn right() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }
}

impl Default for Vector3 {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::fmt::Display for Vector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.3}, {:.3}, {:.3})", self.x, self.y, self.z)
    }
}

/// Unity-style Vector3 input field with X, Y, Z components
pub struct Vector3Field {
    container: Box,
    label: Option<Label>,
    x_input: EditorInput,
    y_input: EditorInput,
    z_input: EditorInput,
    x_label: Label,
    y_label: Label,
    z_label: Label,
    value: Vector3,
    theme: Rc<RefCell<EditorTheme>>,
    on_changed: Option<std::boxed::Box<dyn Fn(Vector3)>>,
}

impl Vector3Field {
    /// Create new Vector3 field
    pub fn new(
        label: Option<&str>,
        initial_value: Vector3,
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
        
        // Input container
        let input_container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .homogeneous(true)
            .build();
        
        // X component
        let x_label = Label::builder()
            .label("X")
            .width_chars(1)
            .halign(gtk4::Align::Center)
            .build();
        let x_input = EditorInput::new(InputVariant::Number, InputSize::Small, theme.clone());
        x_input.set_text(&initial_value.x.to_string());
        
        // Y component
        let y_label = Label::builder()
            .label("Y")
            .width_chars(1)
            .halign(gtk4::Align::Center)
            .build();
        let y_input = EditorInput::new(InputVariant::Number, InputSize::Small, theme.clone());
        y_input.set_text(&initial_value.y.to_string());
        
        // Z component
        let z_label = Label::builder()
            .label("Z")
            .width_chars(1)
            .halign(gtk4::Align::Center)
            .build();
        let z_input = EditorInput::new(InputVariant::Number, InputSize::Small, theme.clone());
        z_input.set_text(&initial_value.z.to_string());
        
        // Component containers with labels above inputs
        let x_container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .build();
        x_container.append(&x_label);
        x_container.append(&x_input.as_widget());
        
        let y_container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .build();
        y_container.append(&y_label);
        y_container.append(&y_input.as_widget());
        
        let z_container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .build();
        z_container.append(&z_label);
        z_container.append(&z_input.as_widget());
        
        input_container.append(&x_container);
        input_container.append(&y_container);
        input_container.append(&z_container);
        
        container.append(&input_container);
        
        let mut field = Self {
            container,
            label: field_label,
            x_input,
            y_input,
            z_input,
            x_label,
            y_label,
            z_label,
            value: initial_value,
            theme,
            on_changed: None,
        };
        
        field.setup_styling();
        field.setup_input_handlers();
        field
    }
    
    /// Create Vector3 field without label
    pub fn without_label(initial_value: Vector3, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(None, initial_value, theme)
    }
    
    /// Create Vector3 field with label
    pub fn with_label(label: &str, initial_value: Vector3, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(Some(label), initial_value, theme)
    }
    
    /// Get current vector value
    pub fn value(&self) -> Vector3 {
        self.value
    }
    
    /// Set vector value
    pub fn set_value(&mut self, value: Vector3) {
        self.value = value;
        self.x_input.set_text(&value.x.to_string());
        self.y_input.set_text(&value.y.to_string());
        self.z_input.set_text(&value.z.to_string());
    }
    
    /// Set change handler
    pub fn set_on_changed<F>(&mut self, callback: F)
    where
        F: Fn(Vector3) + 'static,
    {
        self.on_changed = Some(std::boxed::Box::new(callback));
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
    
    /// Set component labels (X, Y, Z by default)
    pub fn set_component_labels(&self, x_label: &str, y_label: &str, z_label: &str) {
        self.x_label.set_text(x_label);
        self.y_label.set_text(y_label);
        self.z_label.set_text(z_label);
    }
    
    /// Enable/disable the field
    pub fn set_enabled(&self, enabled: bool) {
        self.x_input.widget().set_sensitive(enabled);
        self.y_input.widget().set_sensitive(enabled);
        self.z_input.widget().set_sensitive(enabled);
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
        style_context.add_class("vector3-field");
        
        // Style component labels
        for label in [&self.x_label, &self.y_label, &self.z_label] {
            let label_context = label.style_context();
            label_context.add_class("component-label");
        }
        
        // Style main label if present
        if let Some(label) = &self.label {
            let label_context = label.style_context();
            label_context.add_class("field-label");
        }
        
        self.apply_current_theme();
    }
    
    /// Setup input change handlers
    fn setup_input_handlers(&mut self) {
        // X input handler
        // Note: Callback setup is complex with Rust's ownership model
        // In a real implementation, we'd use weak references or channels
        
        // Note: In a real implementation, we'd need to manage state differently
        // to avoid borrowing issues with the callback closures
        // For now, this shows the intended structure
    }
    
    /// Update value from inputs
    fn update_value_from_inputs(&mut self) {
        let x = self.x_input.text().parse().unwrap_or(self.value.x);
        let y = self.y_input.text().parse().unwrap_or(self.value.y);
        let z = self.z_input.text().parse().unwrap_or(self.value.z);
        
        let new_value = Vector3::new(x, y, z);
        if new_value != self.value {
            self.value = new_value;
            if let Some(callback) = &self.on_changed {
                callback(new_value);
            }
        }
    }
    
    /// Apply current theme
    fn apply_current_theme(&mut self) {
        let theme = self.theme.borrow().clone();
        self.apply_theme(&theme);
    }
}

impl Themeable for Vector3Field {
    fn apply_theme(&mut self, theme: &EditorTheme) {
        // Apply spacing
        let spacing = theme.spacing.xs as i32;
        self.container.set_spacing(spacing);
        
        // Style component labels
        for label in [&self.x_label, &self.y_label, &self.z_label] {
            label.set_size_request(-1, theme.sizes.input_height_sm as i32 / 2);
        }
    }
    
    fn get_css(&self, theme: &EditorTheme) -> String {
        format!(
            r#"
            .vector3-field {{
                margin: {margin}px;
            }}
            
            .vector3-field .component-label {{
                color: {label_color};
                font-family: "{font_family}";
                font-size: {font_size}px;
                font-weight: {font_weight};
                text-align: center;
                min-height: {label_height}px;
            }}
            
            .vector3-field .field-label {{
                color: {primary_text};
                font-family: "{font_family}";
                font-size: {field_font_size}px;
                font-weight: {field_font_weight};
                margin-bottom: {label_margin}px;
            }}
            "#,
            margin = theme.spacing.xs,
            label_color = theme.colors.text_secondary.to_hex(),
            font_family = theme.typography.font_family_primary,
            font_size = theme.typography.sizes.xs,
            font_weight = theme.typography.weights.medium,
            label_height = theme.sizes.input_height_sm / 2.0,
            primary_text = theme.colors.text_primary.to_hex(),
            field_font_size = theme.typography.sizes.sm,
            field_font_weight = theme.typography.weights.medium,
            label_margin = theme.spacing.xs,
        )
    }
}

/// Helper functions for common Vector3Field types
impl Vector3Field {
    /// Create position field
    pub fn position(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label("Position", Vector3::zero(), theme)
    }
    
    /// Create rotation field (in degrees)
    pub fn rotation(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label("Rotation", Vector3::zero(), theme)
    }
    
    /// Create scale field
    pub fn scale(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label("Scale", Vector3::one(), theme)
    }
    
    /// Create color RGB field
    pub fn color_rgb(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let field = Self::with_label("Color", Vector3::one(), theme);
        field.set_component_labels("R", "G", "B");
        field
    }
    
    /// Create direction field
    pub fn direction(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label("Direction", Vector3::forward(), theme)
    }
    
    /// Create velocity field
    pub fn velocity(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_label("Velocity", Vector3::zero(), theme)
    }
}

/// Vector2 field for 2D vectors
pub struct Vector2Field {
    vector3_field: Vector3Field,
}

impl Vector2Field {
    /// Create new Vector2 field
    pub fn new(
        label: Option<&str>,
        initial_x: f32,
        initial_y: f32,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        let field = Vector3Field::new(
            label,
            Vector3::new(initial_x, initial_y, 0.0),
            theme,
        );
        
        // Hide Z component
        field.z_label.set_visible(false);
        field.z_input.widget().set_visible(false);
        
        Self {
            vector3_field: field,
        }
    }
    
    /// Get X, Y values
    pub fn value(&self) -> (f32, f32) {
        let vec3 = self.vector3_field.value();
        (vec3.x, vec3.y)
    }
    
    /// Set X, Y values
    pub fn set_value(&mut self, x: f32, y: f32) {
        self.vector3_field.set_value(Vector3::new(x, y, 0.0));
    }
    
    /// Get underlying Vector3Field
    pub fn field(&self) -> &Vector3Field {
        &self.vector3_field
    }
    
    /// Get widget
    pub fn as_widget(&self) -> Widget {
        self.vector3_field.as_widget()
    }
}

/// Helper functions for Vector2Field
impl Vector2Field {
    /// Create size field
    pub fn size(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let field = Self::new(Some("Size"), 1.0, 1.0, theme);
        field.vector3_field.set_component_labels("W", "H", "");
        field
    }
    
    /// Create position 2D field
    pub fn position_2d(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(Some("Position"), 0.0, 0.0, theme)
    }
    
    /// Create UV coordinate field
    pub fn uv(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let field = Self::new(Some("UV"), 0.0, 0.0, theme);
        field.vector3_field.set_component_labels("U", "V", "");
        field
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
    fn test_vector3_creation() {
        let vec = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);
    }

    #[test]
    fn test_vector3_presets() {
        assert_eq!(Vector3::zero(), Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(Vector3::one(), Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(Vector3::forward(), Vector3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_vector3_field_creation() {
        let theme = create_test_theme();
        let field = Vector3Field::new(
            Some("Test"),
            Vector3::new(1.0, 2.0, 3.0),
            theme,
        );
        
        assert_eq!(field.value(), Vector3::new(1.0, 2.0, 3.0));
        assert!(field.label.is_some());
    }

    #[test]
    fn test_vector3_field_presets() {
        let theme = create_test_theme();
        
        let position = Vector3Field::position(theme.clone());
        assert_eq!(position.value(), Vector3::zero());
        
        let scale = Vector3Field::scale(theme);
        assert_eq!(scale.value(), Vector3::one());
    }

    #[test]
    fn test_vector2_field() {
        let theme = create_test_theme();
        let field = Vector2Field::new(Some("Test2D"), 1.0, 2.0, theme);
        
        let (x, y) = field.value();
        assert_eq!(x, 1.0);
        assert_eq!(y, 2.0);
    }

    #[test]
    fn test_css_generation() {
        let theme = create_test_theme();
        let mut field = Vector3Field::position(theme.clone());
        
        let css = field.get_css(&theme.borrow());
        assert!(css.contains("vector3-field"));
        assert!(css.contains("component-label"));
    }
}