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
    camera_view_matrix: Option<glam::Mat4>,
    camera_projection_matrix: Option<glam::Mat4>,
) -> Vec<ConsoleMessage> {
    // Check for hover position
    let hover_pos = response.hover_pos();
    
    // Handle scene navigation (right mouse button + WASD)
    let mut console_messages = super::navigation::SceneNavigator::handle_scene_navigation(
        scene_navigation,
        ui,
        response,
        rect,
    );
    
    // Skip gizmo interaction if we're in navigation mode
    if scene_navigation.is_navigating {
        return console_messages;
    }
    
    // Only handle input if we have a selected entity and move tool is active
    if gizmo_system.get_active_tool() != SceneTool::Move {
        eprintln!("Gizmo: Not in move mode, active tool: {:?}", gizmo_system.get_active_tool());
        return console_messages;
    }
    
    let Some(selected_entity) = selected_entity else {
        eprintln!("Gizmo: No entity selected");
        return console_messages;
    };
    
    let Some(transform) = world.get_component::<Transform>(selected_entity).cloned() else {
        return console_messages;
    };
    
    
    // TEMPORARY: Force use simple 2D projection to bypass 3D camera issues
    eprintln!("Gizmo: FORCING 2D FALLBACK FOR DEBUGGING");
    let gizmo_center = {
        let scene_center = rect.center();
        let scale = 50.0;
        egui::pos2(
            scene_center.x + transform.position[0] * scale,
            scene_center.y - transform.position[2] * scale
        )
    };
    
    // Handle mouse interaction
    if let Some(mouse_pos) = response.hover_pos() {
        // Test for gizmo component hits
        let hit_component = gizmo_system.test_gizmo_hit(mouse_pos, gizmo_center);
        if hit_component.is_some() {
            eprintln!("Gizmo: Hit detected: {:?} at mouse: {:?}, gizmo center: {:?}", hit_component, mouse_pos, gizmo_center);
        }
        
        // Handle drag states in priority order
        if response.drag_stopped() {
            // Stop dragging
            eprintln!("Gizmo: Drag stopped");
            if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
                if gizmo.is_interacting() {
                    gizmo.set_interaction_state(GizmoInteractionState::Idle);
                }
            }
        } else if response.dragged() {
            // Continue existing drag or handle drag movement
            eprintln!("Gizmo: Dragging detected");
            
            // Check if we're already dragging, if not try to start
            if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
                if !gizmo.is_interacting() && hit_component.is_some() {
                    // Start new drag
                    if let Some(component) = hit_component {
                        eprintln!("Gizmo: Starting drag for component: {:?}", component);
                        gizmo.set_interaction_state(GizmoInteractionState::Dragging {
                            component,
                            start_mouse_pos: mouse_pos,
                            start_object_pos: transform.position,
                        });
                    }
                }
            }
            
            handle_gizmo_dragging(
                world,
                gizmo_system,
                mouse_pos,
                selected_entity,
                transform,
                camera_view_matrix,
                camera_projection_matrix,
                rect,
            );
        } else if response.clicked() || response.drag_started() {
            // Handle initial click/drag start
            eprintln!("Gizmo: Click/Drag start - clicked: {}, drag_started: {}, hit: {:?}", 
                response.clicked(), response.drag_started(), hit_component);
            if let Some(component) = hit_component {
                // Start dragging
                if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
                    eprintln!("Gizmo: Setting dragging state for component: {:?}", component);
                    gizmo.set_interaction_state(GizmoInteractionState::Dragging {
                        component,
                        start_mouse_pos: mouse_pos,
                        start_object_pos: transform.position,
                    });
                }
            }
        } else {
            // Handle hovering only when not dragging
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
    
    console_messages
}

fn handle_gizmo_dragging(
    world: &mut World,
    gizmo_system: &mut GizmoSystem,
    mouse_pos: egui::Pos2,
    selected_entity: Entity,
    transform: Transform,
    camera_view_matrix: Option<glam::Mat4>,
    camera_projection_matrix: Option<glam::Mat4>,
    viewport_rect: egui::Rect,
) {
    // Handle dragging - extract values to avoid borrowing conflicts
    let mut new_position = transform.position;
    let mut should_update = false;
    
    if let Some(gizmo) = gizmo_system.get_move_gizmo() {
        eprintln!("Gizmo: Checking drag state: {:?}", gizmo.get_interaction_state());
        if let GizmoInteractionState::Dragging { component, start_mouse_pos, start_object_pos } = gizmo.get_interaction_state() {
            let mouse_delta = mouse_pos - *start_mouse_pos;
            eprintln!("Gizmo: Dragging - mouse delta: {:?}", mouse_delta);
            
            // TEMPORARY: Force simple calculation to test basic gizmo functionality
            eprintln!("Gizmo: FORCING SIMPLE CALCULATION FOR DEBUGGING");
            let delta_vec2 = egui::Vec2::new(mouse_delta.x, mouse_delta.y);
            new_position = gizmo_system.calculate_new_position(*start_object_pos, delta_vec2, *component, 50.0);
            should_update = true;
        }
    }
    
    if should_update {
        eprintln!("Gizmo: Updating position from {:?} to {:?}", transform.position, new_position);
        
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
            eprintln!("Gizmo: Setting transform position to {:?}", new_position);
            transform_mut.position = new_position;
        } else {
            eprintln!("Gizmo: ERROR - Could not get mutable transform for entity {:?}", selected_entity);
        }
        
        // Update gizmo position
        if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
            gizmo.set_position(new_position);
        }
    } else {
        eprintln!("Gizmo: No position update - should_update is false");
    }
}