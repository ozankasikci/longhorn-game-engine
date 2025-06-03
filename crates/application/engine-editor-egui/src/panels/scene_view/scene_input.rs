// Scene input handling for gizmos and object manipulation

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::Transform;
use engine_components_ui::Name;
use crate::types::{SceneNavigation, SceneTool, GizmoSystem, GizmoComponent, GizmoInteractionState};
use crate::editor_state::ConsoleMessage;

/// Handles mouse input for scene view including navigation and gizmo interactions
pub fn handle_scene_input(
    world: &mut World,
    ui: &egui::Ui,
    response: &egui::Response,
    rect: egui::Rect,
    scene_navigation: &mut SceneNavigation,
    gizmo_system: &mut GizmoSystem,
    selected_entity: Option<Entity>,
) {
    // Check for hover position
    let hover_pos = response.hover_pos();
    
    // Handle scene navigation (right mouse button + WASD)
    super::navigation::SceneNavigator::handle_scene_navigation(
        scene_navigation,
        ui,
        response,
        rect,
    );
    
    // Skip gizmo interaction if we're in navigation mode
    if scene_navigation.is_navigating {
        return;
    }
    
    // Only handle input if we have a selected entity and move tool is active
    if gizmo_system.get_active_tool() != SceneTool::Move {
        return;
    }
    
    let Some(selected_entity) = selected_entity else {
        return;
    };
    
    let Some(transform) = world.get_component::<Transform>(selected_entity).cloned() else {
        return;
    };
    
    
    // Calculate scene center and gizmo position
    let scene_center = rect.center();
    let scale = 50.0;
    let gizmo_screen_x = scene_center.x + transform.position[0] * scale;
    let gizmo_screen_y = scene_center.y - transform.position[2] * scale;
    let gizmo_center = egui::pos2(gizmo_screen_x, gizmo_screen_y);
    
    // Handle mouse interaction
    if let Some(mouse_pos) = response.hover_pos() {
        // Test for gizmo component hits
        let hit_component = gizmo_system.test_gizmo_hit(mouse_pos, gizmo_center);
        
        // Check for mouse press (start of drag) or click
        if response.clicked() || (response.drag_started() && hit_component.is_some()) {
            if let Some(component) = hit_component {
                // Start dragging
                if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
                    gizmo.set_interaction_state(GizmoInteractionState::Dragging {
                        component,
                        start_mouse_pos: mouse_pos,
                        start_object_pos: transform.position,
                    });
                }
            }
        } else if response.drag_stopped() {
            // Stop dragging
            if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
                if gizmo.is_interacting() {
                    gizmo.set_interaction_state(GizmoInteractionState::Idle);
                }
            }
        } else if response.dragged() {
            handle_gizmo_dragging(
                world,
                gizmo_system,
                mouse_pos,
                selected_entity,
                transform,
                scale,
            );
        } else {
            // Handle hovering
            if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
                if !gizmo.is_interacting() {
                    gizmo.set_interaction_state(if let Some(component) = hit_component {
                        GizmoInteractionState::Hovering(component)
                    } else {
                        GizmoInteractionState::Idle
                    });
                }
            }
        }
    }
    // Input handled
}

fn handle_gizmo_dragging(
    world: &mut World,
    gizmo_system: &mut GizmoSystem,
    mouse_pos: egui::Pos2,
    selected_entity: Entity,
    transform: Transform,
    scale: f32,
) {
    // Handle dragging - extract values to avoid borrowing conflicts
    let mut new_position = transform.position;
    let mut should_update = false;
    
    if let Some(gizmo) = gizmo_system.get_move_gizmo() {
        if let GizmoInteractionState::Dragging { component, start_mouse_pos, start_object_pos } = gizmo.get_interaction_state() {
            let mouse_delta = mouse_pos - *start_mouse_pos;
            let delta_vec2 = egui::Vec2::new(mouse_delta.x, mouse_delta.y);
            new_position = gizmo_system.calculate_new_position(*start_object_pos, delta_vec2, *component, scale);
            should_update = true;
        }
    }
    
    if should_update {
        // Apply snapping if enabled
        let snap_enabled = gizmo_system.is_snap_enabled();
        let snap_increment = gizmo_system.get_snap_increment();
        
        if snap_enabled {
            new_position[0] = (new_position[0] / snap_increment).round() * snap_increment;
            new_position[1] = (new_position[1] / snap_increment).round() * snap_increment;
            new_position[2] = (new_position[2] / snap_increment).round() * snap_increment;
        }
        
        // Update transform in ECS
        if let Some(transform_mut) = world.get_component_mut::<Transform>(selected_entity) {
            transform_mut.position = new_position;
        }
        
        // Update gizmo position
        if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
            gizmo.set_position(new_position);
        }
    }
}