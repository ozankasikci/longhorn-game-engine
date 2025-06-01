// Editor input field component with Unity-style design

use gtk4::prelude::*;
use gtk4::{Entry, Widget};
use std::rc::Rc;
use std::cell::RefCell;
use crate::{EditorTheme, Themeable};

/// Input field variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputVariant {
    Default,    // Standard input
    Search,     // Search input with icon
    Number,     // Numeric input
    Password,   // Password input
}

/// Input field states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputState {
    Normal,
    Focus,
    Error,
    Disabled,
    ReadOnly,
}

/// Input field sizes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputSize {
    Small,
    Medium,
    Large,
}

/// Editor input field with consistent theming
pub struct EditorInput {
    widget: Entry,
    variant: InputVariant,
    state: InputState,
    size: InputSize,
    theme: Rc<RefCell<EditorTheme>>,
}

impl EditorInput {
    /// Create new editor input
    pub fn new(
        variant: InputVariant,
        size: InputSize,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        let entry = Entry::builder()
            .build();
            
        let mut editor_input = Self {
            widget: entry,
            variant,
            state: InputState::Normal,
            size,
            theme,
        };
        
        editor_input.setup_styling();
        editor_input.setup_behavior();
        editor_input
    }
    
    /// Create input with placeholder text
    pub fn with_placeholder(
        placeholder: &str,
        variant: InputVariant,
        size: InputSize,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        let input = Self::new(variant, size, theme);
        input.set_placeholder(placeholder);
        input
    }
    
    /// Set placeholder text
    pub fn set_placeholder(&self, text: &str) {
        self.widget.set_placeholder_text(Some(text));
    }
    
    /// Set input text
    pub fn set_text(&self, text: &str) {
        self.widget.set_text(text);
    }
    
    /// Get input text
    pub fn text(&self) -> String {
        self.widget.text().to_string()
    }
    
    /// Set input state
    pub fn set_state(&mut self, state: InputState) {
        self.state = state;
        
        match state {
            InputState::Disabled => {
                self.widget.set_sensitive(false);
            },
            InputState::ReadOnly => {
                self.widget.set_editable(false);
            },
            _ => {
                self.widget.set_sensitive(true);
                self.widget.set_editable(true);
            }
        }
        
        self.update_styling();
    }
    
    /// Set input variant
    pub fn set_variant(&mut self, variant: InputVariant) {
        self.variant = variant;
        self.update_styling();
    }
    
    /// Set validation error
    pub fn set_error(&mut self, error: Option<&str>) {
        if error.is_some() {
            self.set_state(InputState::Error);
            // TODO: Show error message tooltip or below field
        } else {
            self.set_state(InputState::Normal);
        }
    }
    
    /// Connect text changed handler
    pub fn connect_changed<F>(&self, callback: F)
    where
        F: Fn(&str) + 'static,
    {
        self.widget.connect_changed(move |entry| {
            callback(&entry.text());
        });
    }
    
    /// Connect focus handlers
    pub fn connect_focus_handlers<F1, F2>(&mut self, on_focus: F1, on_blur: F2)
    where
        F1: Fn() + 'static,
        F2: Fn() + 'static,
    {
        // Focus in
        let controller = gtk4::EventControllerFocus::new();
        controller.connect_enter(move |_| {
            on_focus();
        });
        self.widget.add_controller(controller);
        
        // Focus out
        let controller = gtk4::EventControllerFocus::new();
        controller.connect_leave(move |_| {
            on_blur();
        });
        self.widget.add_controller(controller);
    }
    
    /// Get underlying GTK widget
    pub fn widget(&self) -> &Entry {
        &self.widget
    }
    
    /// Get widget as generic Widget
    pub fn as_widget(&self) -> Widget {
        self.widget.clone().upcast()
    }
    
    /// Setup initial styling and classes
    fn setup_styling(&mut self) {
        let style_context = self.widget.style_context();
        
        // Add base class
        style_context.add_class("editor-input");
        
        // Add variant class
        let variant_class = match self.variant {
            InputVariant::Default => "default",
            InputVariant::Search => "search",
            InputVariant::Number => "number",
            InputVariant::Password => "password",
        };
        style_context.add_class(variant_class);
        
        // Add size class
        let size_class = match self.size {
            InputSize::Small => "small",
            InputSize::Medium => "medium",
            InputSize::Large => "large",
        };
        style_context.add_class(size_class);
        
        self.apply_current_theme();
    }
    
    /// Setup input behavior based on variant
    fn setup_behavior(&self) {
        match self.variant {
            InputVariant::Password => {
                self.widget.set_visibility(false);
            },
            InputVariant::Number => {
                self.widget.set_input_purpose(gtk4::InputPurpose::Number);
            },
            InputVariant::Search => {
                self.widget.set_input_purpose(gtk4::InputPurpose::FreeForm);
                // Could add search icon here
            },
            _ => {}
        }
    }
    
    /// Update styling after state/variant change
    fn update_styling(&self) {
        let style_context = self.widget.style_context();
        
        // Remove old state classes
        style_context.remove_class("normal");
        style_context.remove_class("focus");
        style_context.remove_class("error");
        style_context.remove_class("disabled");
        style_context.remove_class("readonly");
        
        // Add current state class
        let state_class = match self.state {
            InputState::Normal => "normal",
            InputState::Focus => "focus",
            InputState::Error => "error",
            InputState::Disabled => "disabled",
            InputState::ReadOnly => "readonly",
        };
        style_context.add_class(state_class);
    }
    
    /// Apply current theme
    fn apply_current_theme(&mut self) {
        let theme = self.theme.borrow().clone();
        self.apply_theme(&theme);
    }
}

impl Themeable for EditorInput {
    fn apply_theme(&mut self, theme: &EditorTheme) {
        // Set input dimensions based on size
        let height = match self.size {
            InputSize::Small => theme.sizes.input_height_sm as i32,
            InputSize::Medium => theme.sizes.input_height_md as i32,
            InputSize::Large => theme.sizes.input_height_lg as i32,
        };
        
        self.widget.set_size_request(-1, height);
    }
    
    fn get_css(&self, theme: &EditorTheme) -> String {
        let variant_name = match self.variant {
            InputVariant::Default => "default",
            InputVariant::Search => "search",
            InputVariant::Number => "number",
            InputVariant::Password => "password",
        };
        
        let size_name = match self.size {
            InputSize::Small => "small",
            InputSize::Medium => "medium",
            InputSize::Large => "large",
        };
        
        theme.input_css(variant_name, size_name)
    }
}

/// Helper functions for common input types
impl EditorInput {
    /// Create a standard text input
    pub fn text_input(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(InputVariant::Default, InputSize::Medium, theme)
    }
    
    /// Create a search input
    pub fn search(placeholder: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_placeholder(placeholder, InputVariant::Search, InputSize::Medium, theme)
    }
    
    /// Create a number input
    pub fn number(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(InputVariant::Number, InputSize::Medium, theme)
    }
    
    /// Create a password input
    pub fn password(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(InputVariant::Password, InputSize::Medium, theme)
    }
    
    /// Create a small input for compact layouts
    pub fn small(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(InputVariant::Default, InputSize::Small, theme)
    }
    
    /// Create a large input for forms
    pub fn large(placeholder: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_placeholder(placeholder, InputVariant::Default, InputSize::Large, theme)
    }
}

/// Specialized numeric input with validation
pub struct NumericInput {
    input: EditorInput,
    min_value: Option<f64>,
    max_value: Option<f64>,
    step: f64,
}

impl NumericInput {
    /// Create numeric input with range
    pub fn new(
        min: Option<f64>,
        max: Option<f64>,
        step: f64,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        let input = EditorInput::number(theme.clone());
        
        let mut numeric = Self {
            input,
            min_value: min,
            max_value: max,
            step,
        };
        
        numeric.setup_validation();
        numeric
    }
    
    /// Get numeric value
    pub fn value(&self) -> Option<f64> {
        self.input.text().parse().ok()
    }
    
    /// Set numeric value
    pub fn set_value(&self, value: f64) {
        let clamped = self.clamp_value(value);
        self.input.set_text(&clamped.to_string());
    }
    
    /// Get underlying input
    pub fn input(&self) -> &EditorInput {
        &self.input
    }
    
    /// Setup numeric validation
    fn setup_validation(&mut self) {
        // TODO: Add input filtering for numeric characters only
        // TODO: Add validation on text change
        // TODO: Add increment/decrement on arrow keys
    }
    
    /// Clamp value to range
    fn clamp_value(&self, value: f64) -> f64 {
        let mut clamped = value;
        
        if let Some(min) = self.min_value {
            clamped = clamped.max(min);
        }
        
        if let Some(max) = self.max_value {
            clamped = clamped.min(max);
        }
        
        clamped
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
    fn test_input_creation() {
        let theme = create_test_theme();
        let input = EditorInput::new(InputVariant::Default, InputSize::Medium, theme);
        assert_eq!(input.variant, InputVariant::Default);
        assert_eq!(input.size, InputSize::Medium);
        assert_eq!(input.state, InputState::Normal);
    }

    #[test]
    fn test_input_variants() {
        let theme = create_test_theme();
        
        let text = EditorInput::text_input(theme.clone());
        assert_eq!(text.variant, InputVariant::Default);
        
        let search = EditorInput::search("Search...", theme.clone());
        assert_eq!(search.variant, InputVariant::Search);
        
        let number = EditorInput::number(theme);
        assert_eq!(number.variant, InputVariant::Number);
    }

    #[test]
    fn test_numeric_input() {
        let theme = create_test_theme();
        let numeric = NumericInput::new(Some(0.0), Some(100.0), 1.0, theme);
        
        numeric.set_value(50.0);
        assert_eq!(numeric.value(), Some(50.0));
        
        // Test clamping
        assert_eq!(numeric.clamp_value(-10.0), 0.0);
        assert_eq!(numeric.clamp_value(150.0), 100.0);
    }

    #[test]
    fn test_css_generation() {
        let theme = create_test_theme();
        let mut input = EditorInput::text_input(theme.clone());
        
        let css = input.get_css(&theme.borrow());
        assert!(css.contains("editor-input"));
        assert!(css.contains("default"));
        assert!(css.contains("medium"));
    }
}