// Editor toolbar component with Unity-style design

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Separator, Widget};
use std::rc::Rc;
use std::cell::RefCell;
use crate::{EditorTheme, Themeable, EditorButton};

/// Tool definition for toolbar
#[derive(Debug, Clone)]
pub struct Tool {
    pub id: String,
    pub icon_name: String,
    pub tooltip: String,
    pub enabled: bool,
    pub active: bool,
}

impl Tool {
    /// Create new tool
    pub fn new(id: &str, icon_name: &str, tooltip: &str) -> Self {
        Self {
            id: id.to_string(),
            icon_name: icon_name.to_string(),
            tooltip: tooltip.to_string(),
            enabled: true,
            active: false,
        }
    }
    
    /// Create tool in disabled state
    pub fn disabled(id: &str, icon_name: &str, tooltip: &str) -> Self {
        Self {
            id: id.to_string(),
            icon_name: icon_name.to_string(),
            tooltip: tooltip.to_string(),
            enabled: false,
            active: false,
        }
    }
}

/// Group of related tools
#[derive(Debug, Clone)]
pub struct ToolGroup {
    pub name: String,
    pub tools: Vec<Tool>,
}

impl ToolGroup {
    /// Create new tool group
    pub fn new(name: &str, tools: Vec<Tool>) -> Self {
        Self {
            name: name.to_string(),
            tools,
        }
    }
}

/// Editor toolbar with consistent theming and tool management
pub struct EditorToolbar {
    container: Box,
    tool_buttons: Vec<EditorButton>,
    tool_groups: Vec<ToolGroup>,
    theme: Rc<RefCell<EditorTheme>>,
    on_tool_clicked: Option<std::boxed::Box<dyn Fn(&str)>>,
}

impl EditorToolbar {
    /// Create new editor toolbar
    pub fn new(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();
            
        let toolbar = Self {
            container,
            tool_buttons: Vec::new(),
            tool_groups: Vec::new(),
            theme,
            on_tool_clicked: None,
        };
        
        toolbar.setup_styling();
        toolbar
    }
    
    /// Add tool group to toolbar
    pub fn add_tool_group(&mut self, group: ToolGroup) {
        if !self.tool_groups.is_empty() {
            // Add separator between groups
            let separator = Separator::builder()
                .orientation(Orientation::Vertical)
                .build();
            self.container.append(&separator);
        }
        
        for tool in &group.tools {
            let button = EditorButton::toolbar_icon(
                &tool.icon_name,
                &tool.tooltip,
                self.theme.clone(),
            );
            
            button.set_enabled(tool.enabled);
            
            // Connect click handler
            let tool_id = tool.id.clone();
            let on_click = self.on_tool_clicked.as_ref().map(|f| f.as_ref());
            if let Some(handler) = on_click {
                let handler_clone = unsafe {
                    std::mem::transmute::<&dyn Fn(&str), &'static dyn Fn(&str)>(handler)
                };
                button.connect_clicked(move || {
                    handler_clone(&tool_id);
                });
            }
            
            self.container.append(&button.as_widget());
            self.tool_buttons.push(button);
        }
        
        self.tool_groups.push(group);
    }
    
    /// Add single tool
    pub fn add_tool(&mut self, tool: Tool) {
        self.add_tool_group(ToolGroup::new("", vec![tool]));
    }
    
    /// Set tool click handler
    pub fn set_tool_handler<F>(&mut self, handler: F)
    where
        F: Fn(&str) + 'static,
    {
        self.on_tool_clicked = Some(std::boxed::Box::new(handler));
    }
    
    /// Enable/disable tool by ID
    pub fn set_tool_enabled(&mut self, tool_id: &str, enabled: bool) {
        for group in &mut self.tool_groups {
            for tool in &mut group.tools {
                if tool.id == tool_id {
                    tool.enabled = enabled;
                    break;
                }
            }
        }
        
        // Update button state
        // TODO: Map tool IDs to button indices for efficient lookup
        self.rebuild_toolbar();
    }
    
    /// Set tool active state (for toggle tools)
    pub fn set_tool_active(&mut self, tool_id: &str, active: bool) {
        for group in &mut self.tool_groups {
            for tool in &mut group.tools {
                if tool.id == tool_id {
                    tool.active = active;
                    break;
                }
            }
        }
        
        // Update button appearance
        self.rebuild_toolbar();
    }
    
    /// Clear all tools
    pub fn clear(&mut self) {
        self.tool_groups.clear();
        self.tool_buttons.clear();
        
        // Remove all children
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
    }
    
    /// Get underlying GTK widget
    pub fn widget(&self) -> &Box {
        &self.container
    }
    
    /// Get widget as generic Widget
    pub fn as_widget(&self) -> Widget {
        self.container.clone().upcast()
    }
    
    /// Setup initial styling and classes
    fn setup_styling(&self) {
        let style_context = self.container.style_context();
        style_context.add_class("editor-toolbar");
        
        let theme = self.theme.borrow();
        self.container.set_size_request(-1, theme.sizes.toolbar_height as i32);
        
        // Add padding
        let padding = theme.spacing.xs as i32;
        self.container.set_margin_top(padding);
        self.container.set_margin_bottom(padding);
        self.container.set_margin_start(theme.spacing.sm as i32);
        self.container.set_margin_end(theme.spacing.sm as i32);
    }
    
    /// Rebuild toolbar after state changes
    fn rebuild_toolbar(&mut self) {
        // Clear current buttons
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
        self.tool_buttons.clear();
        
        // Rebuild from tool groups
        let groups = self.tool_groups.clone();
        self.tool_groups.clear();
        
        for group in groups {
            self.add_tool_group(group);
        }
    }
}

impl Themeable for EditorToolbar {
    fn apply_theme(&mut self, theme: &EditorTheme) {
        self.container.set_size_request(-1, theme.sizes.toolbar_height as i32);
        
        // Update spacing
        self.container.set_spacing(theme.spacing.xs as i32);
        
        // Update margins
        let padding = theme.spacing.xs as i32;
        self.container.set_margin_top(padding);
        self.container.set_margin_bottom(padding);
        self.container.set_margin_start(theme.spacing.sm as i32);
        self.container.set_margin_end(theme.spacing.sm as i32);
    }
    
    fn get_css(&self, theme: &EditorTheme) -> String {
        format!(
            r#"
            .editor-toolbar {{
                background-color: {bg_color};
                border-bottom: {border_width}px solid {border_color};
                min-height: {height}px;
            }}
            "#,
            bg_color = theme.colors.surface.to_hex(),
            border_color = theme.colors.border.to_hex(),
            border_width = theme.sizes.border_width,
            height = theme.sizes.toolbar_height,
        )
    }
}

/// Helper functions for creating common toolbars
impl EditorToolbar {
    /// Create a standard editor toolbar with common tools
    pub fn standard(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let mut toolbar = Self::new(theme);
        
        // File operations
        toolbar.add_tool_group(ToolGroup::new("File", vec![
            Tool::new("new", "document-new", "New Scene"),
            Tool::new("open", "document-open", "Open Scene"),
            Tool::new("save", "document-save", "Save Scene"),
        ]));
        
        // Edit operations
        toolbar.add_tool_group(ToolGroup::new("Edit", vec![
            Tool::new("undo", "edit-undo", "Undo"),
            Tool::new("redo", "edit-redo", "Redo"),
        ]));
        
        // Transform tools
        toolbar.add_tool_group(ToolGroup::new("Transform", vec![
            Tool::new("select", "edit-select", "Select Tool"),
            Tool::new("move", "transform-move", "Move Tool"),
            Tool::new("rotate", "transform-rotate", "Rotate Tool"),
            Tool::new("scale", "transform-scale", "Scale Tool"),
        ]));
        
        // View tools
        toolbar.add_tool_group(ToolGroup::new("View", vec![
            Tool::new("hand", "edit-hand", "Hand Tool"),
            Tool::new("zoom", "zoom-in", "Zoom Tool"),
        ]));
        
        toolbar
    }
    
    /// Create a play controls toolbar
    pub fn play_controls(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let mut toolbar = Self::new(theme);
        
        toolbar.add_tool_group(ToolGroup::new("Playback", vec![
            Tool::new("play", "media-playback-start", "Play"),
            Tool::new("pause", "media-playback-pause", "Pause"),
            Tool::new("stop", "media-playback-stop", "Stop"),
            Tool::new("step", "media-skip-forward", "Step Frame"),
        ]));
        
        toolbar
    }
    
    /// Create a build toolbar
    pub fn build_controls(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let mut toolbar = Self::new(theme);
        
        toolbar.add_tool_group(ToolGroup::new("Build", vec![
            Tool::new("build", "application-x-executable", "Build"),
            Tool::new("build_run", "media-playback-start", "Build and Run"),
            Tool::new("clean", "edit-clear", "Clean Build"),
        ]));
        
        toolbar
    }
}

/// Specialized toolbar for scene view with transform gizmos
pub struct SceneToolbar {
    toolbar: EditorToolbar,
    current_tool: String,
}

impl SceneToolbar {
    /// Create new scene toolbar
    pub fn new(theme: Rc<RefCell<EditorTheme>>) -> Self {
        let mut toolbar = EditorToolbar::new(theme);
        
        // Scene manipulation tools
        toolbar.add_tool_group(ToolGroup::new("Scene", vec![
            Tool::new("select", "edit-select", "Select (Q)"),
            Tool::new("move", "transform-move", "Move (W)"),
            Tool::new("rotate", "transform-rotate", "Rotate (E)"),
            Tool::new("scale", "transform-scale", "Scale (R)"),
            Tool::new("rect", "edit-select-rectangle", "Rect Tool (T)"),
        ]));
        
        // View controls
        toolbar.add_tool_group(ToolGroup::new("View", vec![
            Tool::new("hand", "edit-hand", "Hand Tool"),
            Tool::new("zoom", "zoom-in", "Zoom"),
        ]));
        
        // Gizmo options
        toolbar.add_tool_group(ToolGroup::new("Gizmos", vec![
            Tool::new("center", "transform-center", "Center"),
            Tool::new("pivot", "transform-pivot", "Pivot"),
            Tool::new("local", "transform-local", "Local"),
            Tool::new("global", "transform-global", "Global"),
        ]));
        
        Self {
            toolbar,
            current_tool: "select".to_string(),
        }
    }
    
    /// Set active transform tool
    pub fn set_transform_tool(&mut self, tool: &str) {
        // Deactivate current tool
        self.toolbar.set_tool_active(&self.current_tool, false);
        
        // Activate new tool
        self.current_tool = tool.to_string();
        self.toolbar.set_tool_active(&self.current_tool, true);
    }
    
    /// Get underlying toolbar
    pub fn toolbar(&self) -> &EditorToolbar {
        &self.toolbar
    }
    
    /// Get mutable toolbar
    pub fn toolbar_mut(&mut self) -> &mut EditorToolbar {
        &mut self.toolbar
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
    fn test_tool_creation() {
        let tool = Tool::new("test", "edit-select", "Test Tool");
        assert_eq!(tool.id, "test");
        assert_eq!(tool.icon_name, "edit-select");
        assert!(tool.enabled);
        assert!(!tool.active);
    }

    #[test]
    fn test_toolbar_creation() {
        let theme = create_test_theme();
        let toolbar = EditorToolbar::new(theme);
        assert_eq!(toolbar.tool_groups.len(), 0);
        assert_eq!(toolbar.tool_buttons.len(), 0);
    }

    #[test]
    fn test_tool_group_addition() {
        let theme = create_test_theme();
        let mut toolbar = EditorToolbar::new(theme);
        
        let group = ToolGroup::new("Test", vec![
            Tool::new("tool1", "icon1", "Tool 1"),
            Tool::new("tool2", "icon2", "Tool 2"),
        ]);
        
        toolbar.add_tool_group(group);
        assert_eq!(toolbar.tool_groups.len(), 1);
        assert_eq!(toolbar.tool_buttons.len(), 2);
    }

    #[test]
    fn test_standard_toolbar() {
        let theme = create_test_theme();
        let toolbar = EditorToolbar::standard(theme);
        assert!(toolbar.tool_groups.len() > 0);
    }

    #[test]
    fn test_scene_toolbar() {
        let theme = create_test_theme();
        let mut scene_toolbar = SceneToolbar::new(theme);
        assert_eq!(scene_toolbar.current_tool, "select");
        
        scene_toolbar.set_transform_tool("move");
        assert_eq!(scene_toolbar.current_tool, "move");
    }

    #[test]
    fn test_css_generation() {
        let theme = create_test_theme();
        let mut toolbar = EditorToolbar::new(theme.clone());
        
        let css = toolbar.get_css(&theme.borrow());
        assert!(css.contains("editor-toolbar"));
        assert!(css.contains("background-color"));
    }
}