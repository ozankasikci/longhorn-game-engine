//! UI state tracking for remote control and automated testing.
//!
//! This module is intentionally decoupled from core editor logic to keep
//! the remote control functionality isolated and maintainable.

use serde::Serialize;

/// Type of UI action to simulate
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerAction {
    /// Single left click
    Click,
    /// Double left click
    DoubleClick,
    /// Right click (context menu)
    RightClick,
}

/// A pending trigger request
#[derive(Debug, Clone)]
pub struct PendingTrigger {
    /// Element ID to trigger
    pub element_id: String,
    /// Type of action to perform
    pub action: TriggerAction,
}

/// Tracks UI state for remote queries and control.
///
/// This struct is updated each frame by panels and queried by remote commands.
/// It acts as a communication bridge between the UI and external automation.
#[derive(Default)]
pub struct UiStateTracker {
    /// Currently focused panel (if any)
    pub focused_panel: Option<String>,
    /// Panels visible this frame
    visible_panels: Vec<PanelState>,
    /// Clickable elements in the current panel
    clickable_elements: Vec<ClickableElement>,
    /// Pending focus request from remote command
    pending_focus: Option<String>,
    /// Pending tree expansion by path
    pending_tree_expand: Option<String>,
    /// Pending tree collapse by path
    pending_tree_collapse: Option<String>,
    /// Pending tree selection by path
    pending_tree_select: Option<String>,
    /// Pending element trigger by ID (legacy, for simple clicks)
    pending_trigger: Option<String>,
    /// Pending element trigger with action type
    pending_trigger_action: Option<PendingTrigger>,
}

/// State of a single panel
#[derive(Clone, Serialize)]
pub struct PanelState {
    /// Panel identifier (e.g., "hierarchy", "inspector")
    pub id: String,
    /// Display title
    pub title: String,
    /// Whether this panel currently has focus
    pub is_focused: bool,
}

/// A clickable UI element that can be triggered remotely
#[derive(Clone, Serialize)]
pub struct ClickableElement {
    /// Unique ID for triggering (e.g., "add_entity", "tree_node_Player")
    pub id: String,
    /// Display text/label
    pub label: String,
    /// Element type: "button", "menu_item", "tree_node", "selectable"
    pub element_type: String,
}

/// Snapshot of the current UI state for remote queries
#[derive(Clone, Serialize)]
pub struct UiSnapshot {
    /// Currently focused panel
    pub focused_panel: Option<String>,
    /// All visible panels
    pub panels: Vec<PanelState>,
    /// Clickable elements (from focused panel)
    pub clickable_elements: Vec<ClickableElement>,
}

impl UiStateTracker {
    pub fn new() -> Self {
        Self::default()
    }

    // ========== Frame Lifecycle ==========

    /// Called at the start of each frame to reset transient state
    pub fn begin_frame(&mut self) {
        self.visible_panels.clear();
        self.clickable_elements.clear();
        self.focused_panel = None;
    }

    // ========== Panel Registration (called by panels) ==========

    /// Register a panel as visible this frame
    pub fn register_panel(&mut self, id: &str, title: &str, is_focused: bool) {
        self.visible_panels.push(PanelState {
            id: id.to_string(),
            title: title.to_string(),
            is_focused,
        });
        if is_focused {
            self.focused_panel = Some(id.to_string());
        }
    }

    /// Register a clickable element in the current panel
    pub fn register_clickable(&mut self, id: &str, label: &str, element_type: &str) {
        self.clickable_elements.push(ClickableElement {
            id: id.to_string(),
            label: label.to_string(),
            element_type: element_type.to_string(),
        });
    }

    // ========== Remote Command Requests ==========

    /// Request focus on a specific panel
    pub fn request_focus(&mut self, panel: String) {
        self.pending_focus = Some(panel);
    }

    /// Check and consume pending focus request
    pub fn take_pending_focus(&mut self) -> Option<String> {
        self.pending_focus.take()
    }

    /// Request expansion of a tree node by path
    pub fn request_tree_expand(&mut self, path: String) {
        self.pending_tree_expand = Some(path);
    }

    /// Check and consume pending tree expand request
    pub fn take_pending_tree_expand(&mut self) -> Option<String> {
        self.pending_tree_expand.take()
    }

    /// Request collapse of a tree node by path
    pub fn request_tree_collapse(&mut self, path: String) {
        self.pending_tree_collapse = Some(path);
    }

    /// Check and consume pending tree collapse request
    pub fn take_pending_tree_collapse(&mut self) -> Option<String> {
        self.pending_tree_collapse.take()
    }

    /// Request selection by entity path in the scene tree
    pub fn request_tree_select(&mut self, path: String) {
        self.pending_tree_select = Some(path);
    }

    /// Check and consume pending tree select request
    pub fn take_pending_tree_select(&mut self) -> Option<String> {
        self.pending_tree_select.take()
    }

    /// Request triggering of an element by ID (simple click)
    pub fn request_trigger(&mut self, element_id: String) {
        self.pending_trigger = Some(element_id);
    }

    /// Check and consume pending trigger request (simple click)
    pub fn take_pending_trigger(&mut self) -> Option<String> {
        self.pending_trigger.take()
    }

    /// Request triggering of an element with a specific action type
    pub fn request_trigger_action(&mut self, element_id: String, action: TriggerAction) {
        self.pending_trigger_action = Some(PendingTrigger { element_id, action });
    }

    /// Check and consume pending trigger action request
    pub fn take_pending_trigger_action(&mut self) -> Option<PendingTrigger> {
        self.pending_trigger_action.take()
    }

    /// Check if there's a pending trigger for a specific element
    pub fn has_pending_trigger_for(&self, element_id: &str) -> Option<&TriggerAction> {
        self.pending_trigger_action.as_ref()
            .filter(|t| t.element_id == element_id)
            .map(|t| &t.action)
    }

    // ========== Queries (for remote responses) ==========

    /// Get a snapshot of the current UI state
    pub fn snapshot(&self) -> UiSnapshot {
        UiSnapshot {
            focused_panel: self.focused_panel.clone(),
            panels: self.visible_panels.clone(),
            clickable_elements: self.clickable_elements.clone(),
        }
    }

    /// Get list of visible panels
    pub fn panels(&self) -> &[PanelState] {
        &self.visible_panels
    }

    /// Get list of clickable elements
    pub fn clickable_elements(&self) -> &[ClickableElement] {
        &self.clickable_elements
    }

    /// Check if a specific panel should receive focus this frame
    pub fn should_focus(&self, panel_id: &str) -> bool {
        self.pending_focus.as_deref() == Some(panel_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_registration() {
        let mut tracker = UiStateTracker::new();
        tracker.begin_frame();

        tracker.register_panel("hierarchy", "Hierarchy", false);
        tracker.register_panel("inspector", "Inspector", true);

        assert_eq!(tracker.focused_panel, Some("inspector".to_string()));
        assert_eq!(tracker.panels().len(), 2);
    }

    #[test]
    fn test_pending_focus() {
        let mut tracker = UiStateTracker::new();

        tracker.request_focus("inspector".to_string());
        assert!(tracker.should_focus("inspector"));

        let focus = tracker.take_pending_focus();
        assert_eq!(focus, Some("inspector".to_string()));
        assert!(!tracker.should_focus("inspector"));
    }

    #[test]
    fn test_clickable_registration() {
        let mut tracker = UiStateTracker::new();
        tracker.begin_frame();

        tracker.register_clickable("add_entity", "Add Entity", "button");
        tracker.register_clickable("tree_Player", "Player", "tree_node");

        assert_eq!(tracker.clickable_elements().len(), 2);
        assert_eq!(tracker.clickable_elements()[0].id, "add_entity");
    }
}
