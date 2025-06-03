// Object rendering logic for the scene view

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Mesh, MeshType, Material};
use engine_components_2d::SpriteRenderer;
use engine_components_ui::Name;
use engine_camera::Camera;
use crate::editor_state::ConsoleMessage;

/// Renders a camera entity in the scene view
pub fn render_camera(
    painter: &egui::Painter,
    screen_pos: egui::Pos2,
    name: &str,
    is_selected: bool,
) {
    let size = 12.0;
    let color = if is_selected { egui::Color32::YELLOW } else { egui::Color32::BLUE };
    painter.circle_filled(screen_pos, size, color);
    painter.text(
        screen_pos + egui::vec2(size + 5.0, -size),
        egui::Align2::LEFT_CENTER,
        format!("📷 {}", name),
        egui::FontId::proportional(12.0),
        color
    );
}

/// Renders a cube mesh in the scene view
pub fn render_cube(
    painter: &egui::Painter,
    screen_pos: egui::Pos2,
    size: f32,
    rotation: [f32; 3],
    base_color: egui::Color32,
    camera_rot: [f32; 3],
    name: &str,
) {
    // Convert rotations to radians
    let obj_rot_x = rotation[0].to_radians();
    let obj_rot_y = rotation[1].to_radians();
    let obj_rot_z = rotation[2].to_radians();
    
    // Define cube vertices in local space
    let half_size = 0.5;
    let vertices = [
        [-half_size, -half_size, -half_size], // 0: Left-Bottom-Back
        [ half_size, -half_size, -half_size], // 1: Right-Bottom-Back
        [ half_size,  half_size, -half_size], // 2: Right-Top-Back
        [-half_size,  half_size, -half_size], // 3: Left-Top-Back
        [-half_size, -half_size,  half_size], // 4: Left-Bottom-Front
        [ half_size, -half_size,  half_size], // 5: Right-Bottom-Front
        [ half_size,  half_size,  half_size], // 6: Right-Top-Front
        [-half_size,  half_size,  half_size], // 7: Left-Top-Front
    ];
    
    // Apply object rotation to vertices
    let mut rotated_vertices = [[0.0; 3]; 8];
    for (i, vertex) in vertices.iter().enumerate() {
        // Apply rotations in order: Y -> X -> Z (Longhorn-style)
        // Y rotation (yaw)
        let cos_y = obj_rot_y.cos();
        let sin_y = obj_rot_y.sin();
        let x1 = vertex[0] * cos_y - vertex[2] * sin_y;
        let z1 = vertex[0] * sin_y + vertex[2] * cos_y;
        
        // X rotation (pitch)
        let cos_x = obj_rot_x.cos();
        let sin_x = obj_rot_x.sin();
        let y2 = vertex[1] * cos_x - z1 * sin_x;
        let z2 = vertex[1] * sin_x + z1 * cos_x;
        
        // Z rotation (roll)
        let cos_z = obj_rot_z.cos();
        let sin_z = obj_rot_z.sin();
        let x3 = x1 * cos_z - y2 * sin_z;
        let y3 = x1 * sin_z + y2 * cos_z;
        
        rotated_vertices[i] = [x3, y3, z2];
    }
    
    // Transform vertices to camera view space for proper depth sorting and culling
    let yaw = camera_rot[1];
    let pitch = camera_rot[0];
    
    let mut view_vertices = [[0.0; 3]; 8];
    for (i, vertex) in rotated_vertices.iter().enumerate() {
        // Camera yaw (rotate around Y axis)
        let cam_cos_yaw = (-yaw).cos();
        let cam_sin_yaw = (-yaw).sin();
        let view_x = vertex[0] * cam_cos_yaw - vertex[2] * cam_sin_yaw;
        let view_z = vertex[0] * cam_sin_yaw + vertex[2] * cam_cos_yaw;
        
        // Camera pitch (rotate around X axis)
        let cam_cos_pitch = (-pitch).cos();
        let cam_sin_pitch = (-pitch).sin();
        let view_y = vertex[1] * cam_cos_pitch - view_z * cam_sin_pitch;
        let final_view_z = vertex[1] * cam_sin_pitch + view_z * cam_cos_pitch;
        
        view_vertices[i] = [view_x, view_y, final_view_z];
    }
    
    // Project vertices to screen
    let mut screen_vertices = [egui::Pos2::ZERO; 8];
    for (i, vertex) in rotated_vertices.iter().enumerate() {
        let proj_x = vertex[0] * size;
        let proj_y = -vertex[1] * size; // Flip Y for screen coordinates
        screen_vertices[i] = screen_pos + egui::vec2(proj_x, proj_y);
    }
    
    // Define faces with correct winding order
    let faces = [
        ([0, 1, 2, 3], base_color), // Front
        ([4, 7, 6, 5], base_color.gamma_multiply(0.6)), // Back
        ([3, 2, 6, 7], base_color.gamma_multiply(1.2)), // Top
        ([4, 5, 1, 0], base_color.gamma_multiply(0.5)), // Bottom
        ([1, 5, 6, 2], base_color.gamma_multiply(0.8)), // Right
        ([0, 3, 7, 4], base_color.gamma_multiply(0.7)), // Left
    ];
    
    // Sort faces by depth and draw
    let mut face_depths: Vec<(usize, f32)> = faces.iter().enumerate().map(|(i, (indices, _))| {
        let avg_z = indices.iter()
            .map(|&idx| view_vertices[idx][2])
            .sum::<f32>() / 4.0;
        (i, avg_z)
    }).collect();
    
    face_depths.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    // Draw faces
    for (face_idx, _) in face_depths {
        let (indices, face_color) = &faces[face_idx];
        
        // Backface culling
        let v0 = screen_vertices[indices[0]];
        let v1 = screen_vertices[indices[1]];
        let v2 = screen_vertices[indices[2]];
        
        let area = (v1.x - v0.x) * (v2.y - v0.y) - (v2.x - v0.x) * (v1.y - v0.y);
        
        if area > 0.0 {
            let face_points: Vec<egui::Pos2> = indices.iter()
                .map(|&idx| screen_vertices[idx])
                .collect();
            
            painter.add(egui::Shape::convex_polygon(
                face_points,
                *face_color,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60))
            ));
        }
    }
    
    // Draw label
    painter.text(
        screen_pos + egui::vec2(size * 0.7 + 10.0, -size * 0.7),
        egui::Align2::LEFT_CENTER,
        format!("⬜ {}", name),
        egui::FontId::proportional(12.0),
        base_color
    );
}

/// Renders a sphere mesh in the scene view
pub fn render_sphere(
    painter: &egui::Painter,
    screen_pos: egui::Pos2,
    radius: f32,
    rotation: [f32; 3],
    base_color: egui::Color32,
    name: &str,
) {
    let obj_rot_x = rotation[0].to_radians();
    let obj_rot_y = rotation[1].to_radians();
    let obj_rot_z = rotation[2].to_radians();
    
    // Main sphere
    painter.circle_filled(screen_pos, radius, base_color);
    
    // Draw rotation indicator
    if obj_rot_x.abs() > 0.1 || obj_rot_y.abs() > 0.1 || obj_rot_z.abs() > 0.1 {
        let band_y_offset = (obj_rot_x.sin() * radius * 0.3).abs();
        let band_width = radius * 2.0 * obj_rot_y.cos().abs().max(0.2);
        let band_height = radius * 0.2;
        
        let band_rect = egui::Rect::from_center_size(
            screen_pos + egui::vec2(0.0, band_y_offset),
            egui::vec2(band_width, band_height)
        );
        painter.rect_filled(
            band_rect,
            egui::Rounding::same(band_height / 2.0),
            base_color.gamma_multiply(0.7)
        );
    }
    
    // Highlight
    let highlight_angle = obj_rot_y - std::f32::consts::PI / 4.0;
    let highlight_pos = screen_pos + egui::vec2(
        highlight_angle.cos() * radius * 0.3,
        -radius * 0.3
    );
    painter.circle_filled(
        highlight_pos,
        radius * 0.3,
        egui::Color32::from_rgba_unmultiplied(
            ((base_color.r() as f32 * 1.5).min(255.0)) as u8,
            ((base_color.g() as f32 * 1.5).min(255.0)) as u8,
            ((base_color.b() as f32 * 1.5).min(255.0)) as u8,
            base_color.a(),
        )
    );
    
    // Shadow
    painter.circle_stroke(
        screen_pos,
        radius,
        egui::Stroke::new(2.0, base_color.gamma_multiply(0.6))
    );
    
    // Label
    painter.text(
        screen_pos + egui::vec2(radius + 5.0, -radius),
        egui::Align2::LEFT_CENTER,
        format!("⚫ {}", name),
        egui::FontId::proportional(12.0),
        base_color
    );
}

/// Renders a plane mesh in the scene view
pub fn render_plane(
    painter: &egui::Painter,
    screen_pos: egui::Pos2,
    size: f32,
    rotation: [f32; 3],
    color: egui::Color32,
    name: &str,
) {
    let obj_rot_x = rotation[0].to_radians();
    let obj_rot_y = rotation[1].to_radians();
    let obj_rot_z = rotation[2].to_radians();
    
    // Define plane corners
    let half_size = 0.5;
    let corners = [
        [-half_size, 0.0, -half_size],
        [ half_size, 0.0, -half_size],
        [ half_size, 0.0,  half_size],
        [-half_size, 0.0,  half_size],
    ];
    
    // Apply rotation
    let mut rotated_corners = [[0.0; 3]; 4];
    for (i, corner) in corners.iter().enumerate() {
        // Y rotation
        let cos_y = obj_rot_y.cos();
        let sin_y = obj_rot_y.sin();
        let x1 = corner[0] * cos_y - corner[2] * sin_y;
        let z1 = corner[0] * sin_y + corner[2] * cos_y;
        
        // X rotation
        let cos_x = obj_rot_x.cos();
        let sin_x = obj_rot_x.sin();
        let y2 = corner[1] * cos_x - z1 * sin_x;
        let z2 = corner[1] * sin_x + z1 * cos_x;
        
        // Z rotation
        let cos_z = obj_rot_z.cos();
        let sin_z = obj_rot_z.sin();
        let x3 = x1 * cos_z - y2 * sin_z;
        let y3 = x1 * sin_z + y2 * cos_z;
        
        rotated_corners[i] = [x3, y3, z2];
    }
    
    // Project to screen
    let screen_corners: Vec<egui::Pos2> = rotated_corners.iter()
        .map(|corner| {
            let proj_x = corner[0] * size;
            let proj_y = -corner[1] * size;
            screen_pos + egui::vec2(proj_x, proj_y)
        })
        .collect();
    
    // Draw plane
    painter.add(egui::Shape::convex_polygon(
        screen_corners.clone(),
        color,
        egui::Stroke::new(1.0, color.gamma_multiply(0.6))
    ));
    
    // Draw grid lines if not too edge-on
    if rotated_corners[0][1].abs() < 0.9 {
        painter.line_segment(
            [(screen_corners[0] + screen_corners[2].to_vec2()) / 2.0,
             (screen_corners[1] + screen_corners[3].to_vec2()) / 2.0],
            egui::Stroke::new(1.0, color.gamma_multiply(0.4))
        );
        painter.line_segment(
            [(screen_corners[0] + screen_corners[1].to_vec2()) / 2.0,
             (screen_corners[2] + screen_corners[3].to_vec2()) / 2.0],
            egui::Stroke::new(1.0, color.gamma_multiply(0.4))
        );
    }
    
    // Label
    painter.text(
        screen_pos + egui::vec2(size * 0.7 + 5.0, 0.0),
        egui::Align2::LEFT_CENTER,
        format!("▭ {}", name),
        egui::FontId::proportional(12.0),
        color
    );
}

/// Renders a sprite in the scene view
pub fn render_sprite(
    painter: &egui::Painter,
    screen_pos: egui::Pos2,
    size: egui::Vec2,
    color: egui::Color32,
    is_selected: bool,
    name: &str,
) {
    let final_color = if is_selected {
        egui::Color32::YELLOW
    } else {
        color
    };
    
    // Draw sprite rectangle
    let sprite_rect = egui::Rect::from_center_size(screen_pos, size);
    painter.rect_filled(sprite_rect, egui::Rounding::same(2.0), final_color);
    
    // Draw border
    painter.rect_stroke(
        sprite_rect,
        egui::Rounding::same(2.0),
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100))
    );
    
    // Draw selection outline
    if is_selected {
        painter.rect_stroke(
            sprite_rect,
            egui::Rounding::same(2.0),
            egui::Stroke::new(3.0, egui::Color32::WHITE)
        );
    }
    
    // Label
    painter.text(
        screen_pos + egui::vec2(size.x * 0.5 + 5.0, -size.y * 0.5),
        egui::Align2::LEFT_CENTER,
        format!("🖼️ {}", name),
        egui::FontId::proportional(12.0),
        final_color
    );
}