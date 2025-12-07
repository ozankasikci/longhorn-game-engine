//! UI automation integration tests for the Longhorn Editor.
//!
//! Tests for panel management, UI element interaction, and scene tree navigation.
//!
//! These tests require a running editor instance.
//! Run with: cargo test --test editor_ui -- --ignored

use longhorn_test_client::EditorClient;

/// Test getting the full UI state.
#[test]
#[ignore]
fn test_get_ui_state() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let ui_state = client.get_ui_state().expect("Failed to get UI state");

    println!("UI State:");
    println!("  Focused panel: {:?}", ui_state.focused_panel);
    println!("  Panels ({}):", ui_state.panels.len());
    for panel in &ui_state.panels {
        let focused = if panel.is_focused { " [FOCUSED]" } else { "" };
        println!("    - {} ({}){}", panel.title, panel.id, focused);
    }
    println!(
        "  Clickable elements: {}",
        ui_state.clickable_elements.len()
    );

    // Should have at least some panels
    assert!(
        !ui_state.panels.is_empty(),
        "Expected at least one panel to be registered"
    );
}

/// Test listing all panels.
#[test]
#[ignore]
fn test_list_panels() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let panels = client.list_panels().expect("Failed to list panels");

    println!("Panels ({}):", panels.len());
    for panel in &panels {
        let focused = if panel.is_focused { " [FOCUSED]" } else { "" };
        println!("  - {} (id: {}){}", panel.title, panel.id, focused);
    }

    assert!(!panels.is_empty(), "Expected at least one panel");
}

/// Test focusing a panel.
#[test]
#[ignore]
fn test_focus_panel() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Get the list of panels
    let panels = client.list_panels().expect("Failed to list panels");

    if panels.len() < 2 {
        println!("Not enough panels to test focusing (need at least 2)");
        return;
    }

    // Find a panel that is not currently focused
    let unfocused_panel = panels.iter().find(|p| !p.is_focused);

    if let Some(panel) = unfocused_panel {
        println!("Focusing panel: {} ({})", panel.title, panel.id);

        client
            .focus_panel(&panel.id)
            .expect("Failed to focus panel");

        // Verify the panel is now focused
        let updated_panels = client.list_panels().expect("Failed to list panels");
        let focused = updated_panels
            .iter()
            .find(|p| p.id == panel.id)
            .expect("Panel should still exist");

        assert!(focused.is_focused, "Panel should now be focused");
        println!("Panel is now focused: {}", focused.is_focused);
    } else {
        println!("All panels are focused (unusual state)");
    }
}

/// Test getting all clickable elements.
#[test]
#[ignore]
fn test_get_clickable_elements() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let elements = client
        .get_clickable_elements()
        .expect("Failed to get clickable elements");

    println!("Clickable elements ({}):", elements.len());
    for element in &elements {
        println!(
            "  - {} (id: {}, type: {})",
            element.label, element.id, element.element_type
        );
    }
}

/// Test clicking a UI element.
#[test]
#[ignore]
fn test_click_element() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let elements = client
        .get_clickable_elements()
        .expect("Failed to get clickable elements");

    // Find a button element to click
    let button = elements.iter().find(|e| e.element_type == "button");

    if let Some(element) = button {
        println!("Clicking element: {} ({})", element.label, element.id);
        client
            .click_element(&element.id)
            .expect("Failed to click element");
        println!("Click completed");
    } else {
        println!("No button elements found to click");
    }
}

/// Test triggering a UI element.
#[test]
#[ignore]
fn test_trigger_element() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let elements = client
        .get_clickable_elements()
        .expect("Failed to get clickable elements");

    if !elements.is_empty() {
        let element = &elements[0];
        println!("Triggering element: {} ({})", element.label, element.id);
        client
            .trigger_element(&element.id)
            .expect("Failed to trigger element");
        println!("Trigger completed");
    } else {
        println!("No elements found to trigger");
    }
}

/// Test toggling the console.
#[test]
#[ignore]
fn test_toggle_console() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Toggle console on
    client.toggle_console().expect("Failed to toggle console");
    println!("Console toggled (first toggle)");

    // Toggle console off
    client.toggle_console().expect("Failed to toggle console");
    println!("Console toggled (second toggle)");
}

/// Test scene tree expansion.
#[test]
#[ignore]
fn test_scene_tree_expansion() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create a parent entity
    let parent_id = client
        .create_entity("ParentEntity")
        .expect("Failed to create parent entity");
    println!("Created parent entity: {}", parent_id);

    // Expand the tree node (using entity name as path)
    client
        .expand_tree_node("ParentEntity")
        .expect("Failed to expand tree node");
    println!("Expanded tree node");

    // Collapse the tree node
    client
        .collapse_tree_node("ParentEntity")
        .expect("Failed to collapse tree node");
    println!("Collapsed tree node");

    // Clean up
    client
        .delete_entity(parent_id)
        .expect("Failed to delete entity");
}

/// Test selecting entity by path.
#[test]
#[ignore]
fn test_select_by_path() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create an entity
    let id = client
        .create_entity("PathSelectableEntity")
        .expect("Failed to create entity");
    println!("Created entity: {}", id);

    // Select by path
    client
        .select_by_path("PathSelectableEntity")
        .expect("Failed to select by path");

    // Verify selection
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(
        state.selected_entity,
        Some(id),
        "Entity should be selected"
    );
    println!("Entity selected via path");

    // Clean up
    client.delete_entity(id).expect("Failed to delete entity");
}

/// Test right-click on UI element.
#[test]
#[ignore]
fn test_right_click_element() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let elements = client
        .get_clickable_elements()
        .expect("Failed to get clickable elements");

    if !elements.is_empty() {
        let element = &elements[0];
        println!(
            "Right-clicking element: {} ({})",
            element.label, element.id
        );

        client
            .right_click_element(&element.id)
            .expect("Failed to right-click element");

        println!("Right-click completed");
    } else {
        println!("No elements found to right-click");
    }
}

/// Test double-click on UI element.
#[test]
#[ignore]
fn test_double_click_element() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let elements = client
        .get_clickable_elements()
        .expect("Failed to get clickable elements");

    if !elements.is_empty() {
        let element = &elements[0];
        println!(
            "Double-clicking element: {} ({})",
            element.label, element.id
        );

        client
            .double_click_element(&element.id)
            .expect("Failed to double-click element");

        println!("Double-click completed");
    } else {
        println!("No elements found to double-click");
    }
}

/// Test panel focus cycle - ensure we can focus each panel.
#[test]
#[ignore]
fn test_panel_focus_cycle() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let panels = client.list_panels().expect("Failed to list panels");
    let panel_ids: Vec<String> = panels.iter().map(|p| p.id.clone()).collect();

    println!("Cycling through {} panels:", panel_ids.len());

    for panel_id in &panel_ids {
        client.focus_panel(panel_id).expect("Failed to focus panel");

        let updated = client.list_panels().expect("Failed to list panels");
        let panel = updated.iter().find(|p| &p.id == panel_id);

        if let Some(p) = panel {
            println!("  - Focused: {} ({})", p.title, p.id);
            assert!(p.is_focused, "Panel should be focused after focus_panel");
        }
    }

    println!("Panel focus cycle complete");
}
