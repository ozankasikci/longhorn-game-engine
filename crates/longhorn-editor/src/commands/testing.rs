//! Testing, gizmo, and scene tree command handlers.
//!
//! Handles screenshot capture, log tailing, frame waiting, gizmo simulation,
//! and scene tree drag-drop simulation for E2E testing.

use longhorn_core::{EntityHandle, EntityId, Name, Transform, Vec2};
use longhorn_engine::Engine;
use longhorn_remote::{
    GizmoDragResult, GizmoStateData, LogEntry, LogTailResult, RemoteResponse, ResponseData,
    ScreenshotResult, WaitFramesResult,
};

use crate::gizmo::{update_transform_from_drag, GizmoHandle, GizmoMode};
use crate::Editor;

// --- Testing Command Handlers ---

pub fn handle_take_screenshot(editor: &mut Editor, path: &str) -> RemoteResponse {
    editor.request_screenshot(path.to_string());

    RemoteResponse::with_data(ResponseData::Screenshot(ScreenshotResult {
        path: path.to_string(),
        width: 0,
        height: 0,
    }))
}

pub fn handle_get_log_tail(lines: usize) -> RemoteResponse {
    let log_path = std::path::Path::new("logs/editor.log");

    let entries = if log_path.exists() {
        match std::fs::read_to_string(log_path) {
            Ok(content) => {
                let all_lines: Vec<&str> = content.lines().collect();
                let start = all_lines.len().saturating_sub(lines);
                all_lines[start..]
                    .iter()
                    .filter_map(|line| parse_log_line(line))
                    .collect()
            }
            Err(e) => {
                log::warn!("Failed to read log file: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    RemoteResponse::with_data(ResponseData::LogTail(LogTailResult { entries }))
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    if line.starts_with('[') {
        if let Some(bracket_end) = line.find(']') {
            let header = &line[1..bracket_end];
            let message = line[bracket_end + 1..].trim();

            let level = if header.contains("ERROR") {
                "ERROR"
            } else if header.contains("WARN") {
                "WARN"
            } else if header.contains("INFO") {
                "INFO"
            } else if header.contains("DEBUG") {
                "DEBUG"
            } else if header.contains("TRACE") {
                "TRACE"
            } else {
                "INFO"
            };

            let timestamp = header.split_whitespace().next().unwrap_or("").to_string();

            return Some(LogEntry {
                timestamp,
                level: level.to_string(),
                message: message.to_string(),
            });
        }
    }

    Some(LogEntry {
        timestamp: String::new(),
        level: "INFO".to_string(),
        message: line.to_string(),
    })
}

pub fn handle_wait_frames(editor: &Editor, count: u32) -> RemoteResponse {
    editor.request_wait_frames(count);

    RemoteResponse::with_data(ResponseData::FramesWaited(WaitFramesResult {
        frames_waited: count,
    }))
}

// --- Gizmo Handlers ---

pub fn handle_get_gizmo_state(editor: &Editor) -> RemoteResponse {
    let gizmo_state = editor.gizmo_state();

    let mode_str = match gizmo_state.mode {
        GizmoMode::None => "none",
        GizmoMode::Move => "move",
        GizmoMode::Rotate => "rotate",
        GizmoMode::Scale => "scale",
    };

    let handle_to_string = |handle: GizmoHandle| -> String {
        match handle {
            GizmoHandle::MoveX => "move_x".to_string(),
            GizmoHandle::MoveY => "move_y".to_string(),
            GizmoHandle::MoveXY => "move_xy".to_string(),
            GizmoHandle::RotateCircle => "rotate_circle".to_string(),
            GizmoHandle::ScaleX => "scale_x".to_string(),
            GizmoHandle::ScaleY => "scale_y".to_string(),
            GizmoHandle::ScaleXY => "scale_xy".to_string(),
        }
    };

    let active_handle = gizmo_state.active_handle.map(handle_to_string);
    let hover_handle = gizmo_state.hover_handle.map(handle_to_string);
    let is_dragging = gizmo_state.is_dragging();

    RemoteResponse::with_data(ResponseData::GizmoState(GizmoStateData {
        mode: mode_str.to_string(),
        active_handle,
        hover_handle,
        is_dragging,
    }))
}

pub fn handle_simulate_gizmo_drag(
    _editor: &mut Editor,
    engine: &mut Engine,
    entity_id: u64,
    handle: &str,
    delta_x: f32,
    delta_y: f32,
) -> RemoteResponse {
    let Some(entity) = EntityId::from_bits(entity_id) else {
        return RemoteResponse::error(format!("Invalid entity ID: {}", entity_id));
    };
    let handle_entity = EntityHandle::new(entity);

    let old_transform = {
        let Ok(transform) = engine.world().get::<Transform>(handle_entity) else {
            return RemoteResponse::error("Entity not found or has no Transform component");
        };
        *transform
    };
    let old_position = [old_transform.position.x, old_transform.position.y];

    let gizmo_handle = match handle {
        "move_x" => GizmoHandle::MoveX,
        "move_y" => GizmoHandle::MoveY,
        "move_xy" => GizmoHandle::MoveXY,
        _ => return RemoteResponse::error(format!("Invalid gizmo handle: {}", handle)),
    };

    let world_delta = Vec2::new(delta_x, delta_y);
    let new_transform = update_transform_from_drag(gizmo_handle, old_transform, world_delta);

    if let Ok(mut transform) = engine.world_mut().get_mut::<Transform>(handle_entity) {
        *transform = new_transform;
    } else {
        return RemoteResponse::error("Failed to update transform");
    }

    let new_position = [new_transform.position.x, new_transform.position.y];

    RemoteResponse::with_data(ResponseData::GizmoDrag(GizmoDragResult {
        entity_id,
        old_position,
        new_position,
    }))
}

// --- Scene Tree Drag-Drop Handlers ---

pub fn handle_simulate_scene_tree_drag(
    _editor: &mut Editor,
    engine: &mut Engine,
    dragged_entity_id: u64,
    target_entity_id: u64,
) -> RemoteResponse {
    use longhorn_core::ecs::hierarchy::set_parent;

    log::info!(
        "Remote: Simulating scene tree drag: {} -> {}",
        dragged_entity_id,
        target_entity_id
    );

    let dragged_entity = match EntityId::from_bits(dragged_entity_id) {
        Some(id) => EntityHandle::new(id),
        None => {
            return RemoteResponse::error(format!(
                "Invalid dragged entity id: {}",
                dragged_entity_id
            ))
        }
    };

    let target_entity = match EntityId::from_bits(target_entity_id) {
        Some(id) => EntityHandle::new(id),
        None => {
            return RemoteResponse::error(format!(
                "Invalid target entity id: {}",
                target_entity_id
            ))
        }
    };

    if engine.world().get::<Name>(dragged_entity).is_err() {
        return RemoteResponse::error(format!("Dragged entity {} not found", dragged_entity_id));
    }
    if engine.world().get::<Name>(target_entity).is_err() {
        return RemoteResponse::error(format!("Target entity {} not found", target_entity_id));
    }

    match set_parent(engine.world_mut(), dragged_entity, target_entity) {
        Ok(()) => {
            log::info!(
                "Successfully reparented entity {} to {}",
                dragged_entity_id,
                target_entity_id
            );
            RemoteResponse::ok()
        }
        Err(e) => {
            log::error!("Failed to reparent: {:?}", e);
            RemoteResponse::error(format!("Failed to reparent: {:?}", e))
        }
    }
}

pub fn handle_simulate_scene_tree_drag_to_root(
    _editor: &mut Editor,
    engine: &mut Engine,
    entity_id: u64,
) -> RemoteResponse {
    use longhorn_core::ecs::hierarchy::clear_parent;

    log::info!(
        "Remote: Simulating scene tree drag to root: {}",
        entity_id
    );

    let entity = match EntityId::from_bits(entity_id) {
        Some(id) => EntityHandle::new(id),
        None => return RemoteResponse::error(format!("Invalid entity id: {}", entity_id)),
    };

    if engine.world().get::<Name>(entity).is_err() {
        return RemoteResponse::error(format!("Entity {} not found", entity_id));
    }

    match clear_parent(engine.world_mut(), entity) {
        Ok(()) => {
            log::info!("Successfully cleared parent for entity {}", entity_id);
            RemoteResponse::ok()
        }
        Err(e) => {
            log::error!("Failed to clear parent: {:?}", e);
            RemoteResponse::error(format!("Failed to clear parent: {:?}", e))
        }
    }
}
