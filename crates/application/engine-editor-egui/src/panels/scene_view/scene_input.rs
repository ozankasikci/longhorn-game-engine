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
) -> Vec<ConsoleMessage> {
    let mut console_messages = Vec::new();
    
    // DEBUG: Log basic input state
    let hover_pos = response.hover_pos();
    let hovered = response.hovered();
    let has_focus = response.has_focus();
    
    // Check for any pointer activity
    let (primary_down, secondary_down, secondary_pressed) = ui.input(|i| {
        (i.pointer.primary_down(), i.pointer.secondary_down(), i.pointer.secondary_pressed())
    });
    
    // Log response state
    if secondary_pressed || secondary_down || response.clicked() {
        console_messages.push(ConsoleMessage::info(&format!(
            "🎯 Response state: hovered={}, has_focus={}, hover_pos={:?}, rect={:?}",
            hovered, has_focus, hover_pos, response.rect
        )));
    }
    
    // Simple test: log any click and check if we're using ui.interact response
    if response.clicked() {
        console_messages.push(ConsoleMessage::info("✅ LEFT CLICK DETECTED via ui.interact!"));
    }
    if response.secondary_clicked() {
        console_messages.push(ConsoleMessage::info("✅ RIGHT CLICK DETECTED via ui.interact!"));
    }
    
    // Also check if we're detecting drags
    if response.dragged() {
        console_messages.push(ConsoleMessage::info(&format!(
            "🖱️ DRAG DETECTED: by={:?}, delta={:?}", 
            response.drag_started_by(egui::PointerButton::Primary) || response.drag_started_by(egui::PointerButton::Secondary),
            response.drag_delta()
        )));
    }
    
    if let Some(mouse_pos) = hover_pos {
        // Only log occasionally to avoid spam
        if response.clicked() || response.dragged() || response.drag_stopped() || secondary_down || secondary_pressed {
            console_messages.push(ConsoleMessage::info(&format!(
                "🖱️ Mouse input: pos=({:.1}, {:.1}), clicked={}, dragged={}, drag_stopped={}, secondary_down={}, secondary_pressed={}",
                mouse_pos.x, mouse_pos.y, response.clicked(), response.dragged(), response.drag_stopped(), secondary_down, secondary_pressed
            )));
        }
    }
    
    // Handle scene navigation (right mouse button + WASD)
    let nav_messages = super::navigation::SceneNavigator::handle_scene_navigation(
        scene_navigation,
        ui,
        response,
        rect,
    );
    console_messages.extend(nav_messages);
    
    // Skip gizmo interaction if we're in navigation mode
    if scene_navigation.is_navigating {
        return console_messages;
    }
    
    // Only handle input if we have a selected entity and move tool is active
    if gizmo_system.get_active_tool() != SceneTool::Move {
        // Only log if there's actual input
        if response.clicked() || response.dragged() {
            console_messages.push(ConsoleMessage::info(&format!(
                "🔧 Not handling input: tool={:?}", gizmo_system.get_active_tool()
            )));
        }
        return console_messages;
    }
    
    let Some(selected_entity) = selected_entity else {
        if response.clicked() || response.dragged() {
            console_messages.push(ConsoleMessage::info("🔧 No selected entity"));
        }
        return console_messages;
    };
    
    let Some(transform) = world.get_component::<Transform>(selected_entity).cloned() else {
        if response.clicked() || response.dragged() {
            console_messages.push(ConsoleMessage::info("🔧 No transform component on selected entity"));
        }
        return console_messages;
    };
    
    // Only log when we're actually processing input
    if response.clicked() || response.dragged() || response.drag_stopped() {
        console_messages.push(ConsoleMessage::info(&format!(
            "🔧 Processing input for entity {} at position [{:.2}, {:.2}, {:.2}]",
            selected_entity.id(), transform.position[0], transform.position[1], transform.position[2]
        )));
    }
    
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
        
        // DEBUG: Log gizmo hit testing (only when relevant)
        if response.clicked() || hit_component.is_some() {
            console_messages.push(ConsoleMessage::info(&format!(
                "🎯 Gizmo hit test: mouse=({:.1}, {:.1}), gizmo_center=({:.1}, {:.1}), hit={:?}",
                mouse_pos.x, mouse_pos.y, gizmo_center.x, gizmo_center.y, hit_component
            )));
        }
        
        // Check for mouse press (start of drag) or click
        if response.clicked() || (response.drag_started() && hit_component.is_some()) {
            if response.clicked() {
                console_messages.push(ConsoleMessage::info("🖱️ Mouse CLICKED"));
            } else {
                console_messages.push(ConsoleMessage::info("🖱️ Mouse DRAG_STARTED"));
            }
            
            if let Some(component) = hit_component {
                // Start dragging
                if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
                    gizmo.set_interaction_state(GizmoInteractionState::Dragging {
                        component,
                        start_mouse_pos: mouse_pos,
                        start_object_pos: transform.position,
                    });
                    console_messages.push(ConsoleMessage::info(&format!(
                        "🔗 Started dragging {:?} at mouse=({:.1}, {:.1}), object_pos=[{:.2}, {:.2}, {:.2}]", 
                        component, mouse_pos.x, mouse_pos.y,
                        transform.position[0], transform.position[1], transform.position[2]
                    )));
                } else {
                    console_messages.push(ConsoleMessage::info("❌ Failed to get mutable gizmo"));
                }
            } else {
                console_messages.push(ConsoleMessage::info("🔗 Input detected but no gizmo component hit"));
            }
        } else if response.drag_stopped() {
            // Stop dragging
            if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
                if gizmo.is_interacting() {
                    gizmo.set_interaction_state(GizmoInteractionState::Idle);
                    console_messages.push(ConsoleMessage::info("🔗 Finished dragging"));
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
                &mut console_messages,
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
    
    console_messages
}

fn handle_gizmo_dragging(
    world: &mut World,
    gizmo_system: &mut GizmoSystem,
    mouse_pos: egui::Pos2,
    selected_entity: Entity,
    transform: Transform,
    scale: f32,
    console_messages: &mut Vec<ConsoleMessage>,
) {
    console_messages.push(ConsoleMessage::info("🖱️ Mouse DRAGGED"));
    
    // Handle dragging - extract values to avoid borrowing conflicts
    let mut new_position = transform.position;
    let mut should_update = false;
    
    if let Some(gizmo) = gizmo_system.get_move_gizmo() {
        console_messages.push(ConsoleMessage::info(&format!(
            "🔧 Gizmo state: {:?}", gizmo.get_interaction_state()
        )));
        
        if let GizmoInteractionState::Dragging { component, start_mouse_pos, start_object_pos } = gizmo.get_interaction_state() {
            let mouse_delta = mouse_pos - *start_mouse_pos;
            let delta_vec2 = egui::Vec2::new(mouse_delta.x, mouse_delta.y);
            new_position = gizmo_system.calculate_new_position(*start_object_pos, delta_vec2, *component, scale);
            should_update = true;
            
            // Debug output
            console_messages.push(ConsoleMessage::info(&format!(
                "🖱️ Dragging {:?}: delta=({:.1}, {:.1}), start=[{:.2}, {:.2}, {:.2}], new=[{:.2}, {:.2}, {:.2}]",
                component, mouse_delta.x, mouse_delta.y,
                start_object_pos[0], start_object_pos[1], start_object_pos[2],
                new_position[0], new_position[1], new_position[2]
            )));
        } else {
            console_messages.push(ConsoleMessage::info("🔧 Not in dragging state"));
        }
    } else {
        console_messages.push(ConsoleMessage::info("❌ No gizmo available"));
    }
    
    if should_update {
        console_messages.push(ConsoleMessage::info(&format!(
            "🔄 About to update transform from [{:.2}, {:.2}, {:.2}] to [{:.2}, {:.2}, {:.2}]",
            transform.position[0], transform.position[1], transform.position[2],
            new_position[0], new_position[1], new_position[2]
        )));
        
        // Apply snapping if enabled
        let snap_enabled = gizmo_system.is_snap_enabled();
        let snap_increment = gizmo_system.get_snap_increment();
        
        if snap_enabled {
            let old_new_pos = new_position;
            new_position[0] = (new_position[0] / snap_increment).round() * snap_increment;
            new_position[1] = (new_position[1] / snap_increment).round() * snap_increment;
            new_position[2] = (new_position[2] / snap_increment).round() * snap_increment;
            console_messages.push(ConsoleMessage::info(&format!(
                "🔄 Snapping applied: [{:.2}, {:.2}, {:.2}] → [{:.2}, {:.2}, {:.2}]",
                old_new_pos[0], old_new_pos[1], old_new_pos[2],
                new_position[0], new_position[1], new_position[2]
            )));
        }
        
        // Update transform in ECS
        console_messages.push(ConsoleMessage::info(&format!(
            "🔄 Attempting ECS mutation for entity {}", selected_entity.id()
        )));
        
        match world.get_component_mut::<Transform>(selected_entity) {
            Some(transform_mut) => {
                let old_position = transform_mut.position;
                transform_mut.position = new_position;
                console_messages.push(ConsoleMessage::info(&format!(
                    "✅ Transform updated: [{:.2}, {:.2}, {:.2}] → [{:.2}, {:.2}, {:.2}]", 
                    old_position[0], old_position[1], old_position[2],
                    transform_mut.position[0], transform_mut.position[1], transform_mut.position[2]
                )));
            }
            None => {
                console_messages.push(ConsoleMessage::info("❌ Failed to get mutable transform - entity doesn't exist or no transform component"));
            }
        }
        
        // Update gizmo position
        if let Some(gizmo) = gizmo_system.get_move_gizmo_mut() {
            gizmo.set_position(new_position);
        }
    }
}