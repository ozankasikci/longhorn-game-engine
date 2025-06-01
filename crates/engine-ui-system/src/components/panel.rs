// Editor panel component with Unity-style design

use gtk4::prelude::*;
use gtk4::{Box, Frame, Label, Widget, Orientation, HeaderBar};
use std::rc::Rc;
use std::cell::RefCell;
use crate::{EditorTheme, Themeable};

/// Panel style variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelStyle {
    Default,    // Standard panel
    Elevated,   // Elevated panel with shadow
    Flat,       // Flat panel without border
    Card,       // Card-style panel
}

/// Editor panel with consistent theming and layout
pub struct EditorPanel {
    container: Frame,
    content_box: Box,
    header: Option<HeaderBar>,
    title: Option<String>,
    style: PanelStyle,
    theme: Rc<RefCell<EditorTheme>>,
}

impl EditorPanel {
    /// Create new editor panel
    pub fn new(
        title: Option<&str>,
        style: PanelStyle,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        let frame = Frame::builder()
            .build();
            
        let content_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();
            
        let header = if let Some(title_text) = title {
            let header_bar = HeaderBar::builder()
                .title_widget(&Label::new(Some(title_text)))
                .show_title_buttons(false)
                .build();
            content_box.append(&header_bar);
            Some(header_bar)
        } else {
            None
        };
        
        frame.set_child(Some(&content_box));
        
        let mut panel = Self {
            container: frame,
            content_box,
            header,
            title: title.map(|s| s.to_string()),
            style,
            theme,
        };
        
        panel.setup_styling();
        panel
    }
    
    /// Create panel without header
    pub fn without_header(
        style: PanelStyle,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        Self::new(None, style, theme)
    }
    
    /// Create panel with title
    pub fn with_title(
        title: &str,
        style: PanelStyle,
        theme: Rc<RefCell<EditorTheme>>,
    ) -> Self {
        Self::new(Some(title), style, theme)
    }
    
    /// Add widget to panel content
    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.content_box.append(widget);
    }
    
    /// Add widget with spacing
    pub fn add_child_with_spacing(&self, widget: &impl IsA<Widget>) {
        let theme = self.theme.borrow();
        let spacing_box = Box::builder()
            .orientation(Orientation::Vertical)
            .margin_top(theme.spacing.sm as i32)
            .margin_bottom(theme.spacing.sm as i32)
            .margin_start(theme.spacing.md as i32)
            .margin_end(theme.spacing.md as i32)
            .build();
            
        spacing_box.append(widget);
        self.content_box.append(&spacing_box);
    }
    
    /// Set panel title
    pub fn set_title(&mut self, title: Option<&str>) {
        self.title = title.map(|s| s.to_string());
        
        if let Some(title_text) = title {
            if let Some(header) = &self.header {
                header.set_title_widget(Some(&Label::new(Some(title_text))));
            } else {
                // Create header if it doesn't exist
                let header_bar = HeaderBar::builder()
                    .title_widget(&Label::new(Some(title_text)))
                    .show_title_buttons(false)
                    .build();
                    
                // Insert at beginning
                self.content_box.prepend(&header_bar);
                self.header = Some(header_bar);
            }
        } else if let Some(header) = &self.header {
            self.content_box.remove(header);
            self.header = None;
        }
    }
    
    /// Set panel style
    pub fn set_style(&mut self, style: PanelStyle) {
        self.style = style;
        self.update_styling();
    }
    
    /// Add action button to header
    pub fn add_header_action(&self, button: &impl IsA<Widget>) {
        if let Some(header) = &self.header {
            header.pack_end(button);
        }
    }
    
    /// Set minimum size
    pub fn set_min_size(&self, width: i32, height: i32) {
        self.container.set_size_request(width, height);
    }
    
    /// Get content box for direct manipulation
    pub fn content_box(&self) -> &Box {
        &self.content_box
    }
    
    /// Get header bar
    pub fn header(&self) -> Option<&HeaderBar> {
        self.header.as_ref()
    }
    
    /// Get underlying GTK widget
    pub fn widget(&self) -> &Frame {
        &self.container
    }
    
    /// Get widget as generic Widget
    pub fn as_widget(&self) -> Widget {
        self.container.clone().upcast()
    }
    
    /// Setup initial styling and classes
    fn setup_styling(&mut self) {
        let style_context = self.container.style_context();
        
        // Add base class
        style_context.add_class("editor-panel");
        
        // Add style class
        let style_class = match self.style {
            PanelStyle::Default => "default",
            PanelStyle::Elevated => "elevated",
            PanelStyle::Flat => "flat",
            PanelStyle::Card => "card",
        };
        style_context.add_class(style_class);
        
        // Style header if present
        if let Some(header) = &self.header {
            let header_context = header.style_context();
            header_context.add_class("editor-panel-header");
        }
        
        self.apply_current_theme();
    }
    
    /// Update styling after style change
    fn update_styling(&mut self) {
        let style_context = self.container.style_context();
        
        // Remove old style classes
        style_context.remove_class("default");
        style_context.remove_class("elevated");
        style_context.remove_class("flat");
        style_context.remove_class("card");
        
        // Add current style class
        let style_class = match self.style {
            PanelStyle::Default => "default",
            PanelStyle::Elevated => "elevated",
            PanelStyle::Flat => "flat",
            PanelStyle::Card => "card",
        };
        style_context.add_class(style_class);
        
        self.apply_current_theme();
    }
    
    /// Apply current theme
    fn apply_current_theme(&mut self) {
        let theme = self.theme.borrow().clone();
        self.apply_theme(&theme);
    }
}

impl Themeable for EditorPanel {
    fn apply_theme(&mut self, theme: &EditorTheme) {
        // Set minimum dimensions
        self.container.set_size_request(
            theme.sizes.panel_min_width as i32,
            theme.sizes.panel_min_height as i32
        );
        
        // Apply margins to content
        let margin = theme.spacing.sm as i32;
        self.content_box.set_margin_top(margin);
        self.content_box.set_margin_bottom(margin);
        self.content_box.set_margin_start(margin);
        self.content_box.set_margin_end(margin);
    }
    
    fn get_css(&self, theme: &EditorTheme) -> String {
        theme.panel_css()
    }
}

/// Helper functions for common panel types
impl EditorPanel {
    /// Create a standard content panel
    pub fn content(title: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_title(title, PanelStyle::Default, theme)
    }
    
    /// Create an elevated dialog-style panel
    pub fn dialog(title: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::with_title(title, PanelStyle::Elevated, theme)
    }
    
    /// Create a flat panel without borders
    pub fn flat(theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::without_header(PanelStyle::Flat, theme)
    }
    
    /// Create a card-style panel
    pub fn card(title: Option<&str>, theme: Rc<RefCell<EditorTheme>>) -> Self {
        Self::new(title, PanelStyle::Card, theme)
    }
    
    /// Create a sidebar panel
    pub fn sidebar(title: &str, theme: Rc<RefCell<EditorTheme>>) -> Self {
        let panel = Self::with_title(title, PanelStyle::Default, theme.clone());
        let theme_ref = theme.borrow();
        panel.set_min_size(theme_ref.sizes.sidebar_width as i32, -1);
        panel
    }
    
    /// Create a toolbar panel
    pub fn toolbar(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let panel = Self::without_header(PanelStyle::Flat, theme.clone());
        let theme_ref = theme.borrow();
        panel.set_min_size(-1, theme_ref.sizes.toolbar_height as i32);
        panel
    }
}

/// Specialized inspector panel for Unity-style property editing
pub struct InspectorPanel {
    panel: EditorPanel,
    sections: Vec<Box>,
}

impl InspectorPanel {
    /// Create new inspector panel
    pub fn new(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let panel = EditorPanel::with_title("Inspector", PanelStyle::Default, theme);
        
        Self {
            panel,
            sections: Vec::new(),
        }
    }
    
    /// Add a property section
    pub fn add_section(&mut self, title: &str) -> &Box {
        let theme = self.panel.theme.borrow();
        
        // Section header
        let header = Label::builder()
            .label(title)
            .halign(gtk4::Align::Start)
            .margin_top(theme.spacing.md as i32)
            .margin_bottom(theme.spacing.sm as i32)
            .build();
        header.style_context().add_class("section-header");
        
        // Section content box
        let section_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(theme.spacing.xs as i32)
            .margin_start(theme.spacing.sm as i32)
            .build();
        
        self.panel.add_child(&header);
        self.panel.add_child(&section_box);
        
        self.sections.push(section_box.clone());
        self.sections.last().unwrap()
    }
    
    /// Get underlying panel
    pub fn panel(&self) -> &EditorPanel {
        &self.panel
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
    fn test_panel_creation() {
        let theme = create_test_theme();
        let panel = EditorPanel::new(Some("Test Panel"), PanelStyle::Default, theme);
        assert_eq!(panel.title, Some("Test Panel".to_string()));
        assert_eq!(panel.style, PanelStyle::Default);
        assert!(panel.header.is_some());
    }

    #[test]
    fn test_panel_without_header() {
        let theme = create_test_theme();
        let panel = EditorPanel::without_header(PanelStyle::Flat, theme);
        assert!(panel.title.is_none());
        assert!(panel.header.is_none());
    }

    #[test]
    fn test_panel_variants() {
        let theme = create_test_theme();
        
        let content = EditorPanel::content("Content", theme.clone());
        assert_eq!(content.style, PanelStyle::Default);
        
        let dialog = EditorPanel::dialog("Dialog", theme.clone());
        assert_eq!(dialog.style, PanelStyle::Elevated);
        
        let flat = EditorPanel::flat(theme);
        assert_eq!(flat.style, PanelStyle::Flat);
    }

    #[test]
    fn test_inspector_panel() {
        let theme = create_test_theme();
        let mut inspector = InspectorPanel::new(theme);
        
        let _section = inspector.add_section("Transform");
        assert_eq!(inspector.sections.len(), 1);
    }

    #[test]
    fn test_css_generation() {
        let theme = create_test_theme();
        let mut panel = EditorPanel::content("Test", theme.clone());
        
        let css = panel.get_css(&theme.borrow());
        assert!(css.contains("editor-panel"));
        assert!(css.contains("background-color"));
    }
}