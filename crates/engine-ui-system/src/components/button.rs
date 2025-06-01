// Editor button component with Unity-style design

use gtk4::prelude::*;
use gtk4::{Button, Widget};
use std::rc::Rc;
use std::cell::RefCell;
use crate::{EditorTheme, Themeable};

/// Button style variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Primary,    // Main action button
    Secondary,  // Secondary action
    Outline,    // Outlined button
    Ghost,      // Text-only button
    Danger,     // Destructive action
}

/// Button sizes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

/// Editor button with consistent theming
pub struct EditorButton {
    widget: Button,
    variant: ButtonVariant,
    size: ButtonSize,
    icon_name: Option<String>,
    theme: Rc<RefCell<EditorTheme>>,
}

impl EditorButton {
    /// Create new editor button
    pub fn new(
        text: &str,
        variant: ButtonVariant,
        size: ButtonSize,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        let button = Button::builder()
            .label(text)
            .build();
            
        let mut editor_button = Self {
            widget: button,
            variant,
            size,
            icon_name: None,
            theme,
        };
        
        editor_button.setup_styling();
        editor_button
    }
    
    /// Create button with icon
    pub fn with_icon(
        text: &str,
        icon_name: &str,
        variant: ButtonVariant,
        size: ButtonSize,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        let mut button = Self::new(text, variant, size, theme);
        button.set_icon(Some(icon_name.to_string()));
        button
    }
    
    /// Create icon-only button
    pub fn icon_only(
        icon_name: &str,
        variant: ButtonVariant,
        size: ButtonSize,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        let button = Button::builder()
            .icon_name(icon_name)
            .build();
            
        let mut editor_button = Self {
            widget: button,
            variant,
            size,
            icon_name: Some(icon_name.to_string()),
            theme,
        };
        
        editor_button.setup_styling();
        editor_button
    }
    
    /// Set button icon
    pub fn set_icon(&mut self, icon_name: Option<String>) {
        self.icon_name = icon_name.clone();
        if let Some(name) = icon_name {
            self.widget.set_icon_name(&name);
        } else {
            self.widget.set_icon_name("");
        }
    }
    
    /// Set button text
    pub fn set_text(&self, text: &str) {
        self.widget.set_label(text);
    }
    
    /// Set button variant
    pub fn set_variant(&mut self, variant: ButtonVariant) {
        self.variant = variant;
        self.apply_current_theme();
    }
    
    /// Set button size
    pub fn set_size(&mut self, size: ButtonSize) {
        self.size = size;
        self.apply_current_theme();
    }
    
    /// Enable/disable button
    pub fn set_enabled(&self, enabled: bool) {
        self.widget.set_sensitive(enabled);
    }
    
    /// Connect click handler
    pub fn connect_clicked<F>(&self, callback: F) 
    where
        F: Fn() + 'static,
    {
        self.widget.connect_clicked(move |_| callback());
    }
    
    /// Get underlying GTK widget
    pub fn widget(&self) -> &Button {
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
        style_context.add_class("editor-button");
        
        // Add variant class
        let variant_class = match self.variant {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Outline => "outline",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::Danger => "danger",
        };
        style_context.add_class(variant_class);
        
        // Add size class
        let size_class = match self.size {
            ButtonSize::Small => "small",
            ButtonSize::Medium => "medium",
            ButtonSize::Large => "large",
        };
        style_context.add_class(size_class);
        
        self.apply_current_theme();
    }
    
    /// Apply current theme
    fn apply_current_theme(&mut self) {
        let theme = self.theme.borrow().clone();
        self.apply_theme(&theme);
    }
}

impl Themeable for EditorButton {
    fn apply_theme(&mut self, theme: &EditorTheme) {
        // CSS will be applied via theme manager's global CSS
        // This ensures consistent styling across all buttons
        
        // Set button dimensions based on size
        let (height, min_width) = match self.size {
            ButtonSize::Small => (theme.sizes.button_height_sm as i32, theme.sizes.button_min_width as i32),
            ButtonSize::Medium => (theme.sizes.button_height_md as i32, theme.sizes.button_min_width as i32),
            ButtonSize::Large => (theme.sizes.button_height_lg as i32, (theme.sizes.button_min_width * 1.5) as i32),
        };
        
        self.widget.set_size_request(min_width, height);
    }
    
    fn get_css(&self, theme: &EditorTheme) -> String {
        let variant_name = match self.variant {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary", 
            ButtonVariant::Outline => "outline",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::Danger => "danger",
        };
        
        let size_name = match self.size {
            ButtonSize::Small => "small",
            ButtonSize::Medium => "medium",
            ButtonSize::Large => "large",
        };
        
        theme.button_css(variant_name, size_name)
    }
}

/// Helper functions for common button types
impl EditorButton {
    /// Create a primary action button
    pub fn primary(text: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(text, ButtonVariant::Primary, ButtonSize::Medium, theme)
    }
    
    /// Create a secondary action button
    pub fn secondary(text: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(text, ButtonVariant::Secondary, ButtonSize::Medium, theme)
    }
    
    /// Create a danger/destructive action button
    pub fn danger(text: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(text, ButtonVariant::Danger, ButtonSize::Medium, theme)
    }
    
    /// Create an outline button
    pub fn outline(text: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(text, ButtonVariant::Outline, ButtonSize::Medium, theme)
    }
    
    /// Create a ghost/text-only button
    pub fn ghost(text: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(text, ButtonVariant::Ghost, ButtonSize::Medium, theme)
    }
    
    /// Create a toolbar button with icon
    pub fn toolbar_icon(icon_name: &str, tooltip: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        let button = Self::icon_only(icon_name, ButtonVariant::Ghost, ButtonSize::Small, theme);
        button.widget().set_tooltip_text(Some(tooltip));
        button
    }
    
    /// Create a toggle button
    pub fn toggle(text: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        let button = Self::new(text, ButtonVariant::Outline, ButtonSize::Medium, theme);
        // GTK4 ToggleButton would be used here for real toggle functionality
        button
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
    fn test_button_creation() {
        let theme = create_test_theme();
        let button = EditorButton::new("Test", ButtonVariant::Primary, ButtonSize::Medium, theme);
        assert_eq!(button.variant, ButtonVariant::Primary);
        assert_eq!(button.size, ButtonSize::Medium);
    }

    #[test]
    fn test_button_variants() {
        let theme = create_test_theme();
        
        let primary = EditorButton::primary("Primary", theme.clone());
        assert_eq!(primary.variant, ButtonVariant::Primary);
        
        let secondary = EditorButton::secondary("Secondary", theme.clone());
        assert_eq!(secondary.variant, ButtonVariant::Secondary);
        
        let danger = EditorButton::danger("Delete", theme);
        assert_eq!(danger.variant, ButtonVariant::Danger);
    }

    #[test]
    fn test_css_generation() {
        let theme = create_test_theme();
        let mut button = EditorButton::primary("Test", theme.clone());
        
        let css = button.get_css(&theme.borrow());
        assert!(css.contains("editor-button"));
        assert!(css.contains("primary"));
        assert!(css.contains("medium"));
    }
}