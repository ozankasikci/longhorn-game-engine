# Grid Rendering Improvement Plan

## Overview
Based on investigation and research, we'll implement a robust grid rendering system that addresses current issues and matches professional 3D editor standards.

## Implementation Strategy

### Phase 1: Fix Immediate Issues (Quick Wins)
1. **Fix Line Clipping**
   - Implement proper near-plane clipping algorithm
   - Clip lines at camera near plane instead of culling entire lines
   - Ensure partial line visibility

2. **Improve Culling Logic**
   - Replace aggressive culling with smart visibility checks
   - Allow lines that cross the near plane to render partially
   - Use screen-space bounds checking

3. **Dynamic Grid Sizing**
   - Scale grid extent based on camera height
   - Implement multiple grid levels (1m, 10m, 100m)
   - Smooth transitions between levels

### Phase 2: Enhanced Rendering (Professional Quality)
1. **Distance-Based Fading**
   - Implement linear fade based on distance from camera
   - Fade minor lines first, then major lines
   - Maintain axis lines visibility

2. **Adaptive LOD System**
   - Show fewer lines when zoomed out
   - Increase line density when zoomed in
   - Smooth transitions between LOD levels

3. **Improved Visual Quality**
   - Anti-aliased lines using proper stroke weights
   - Consistent line thickness regardless of distance
   - Better color/opacity management

### Phase 3: Advanced Features (Future)
1. **Shader-Based Rendering**
   - Move to GPU-based infinite grid
   - Better performance for large grids
   - Pixel-perfect lines

2. **Configurable Grid**
   - User-adjustable grid size
   - Toggle between metric/imperial
   - Customizable colors

## Immediate Implementation Plan

### 1. Line Clipping Algorithm
```rust
fn clip_line_to_near_plane(
    start: [f32; 3], 
    end: [f32; 3], 
    start_depth: f32, 
    end_depth: f32,
    near_plane: f32
) -> Option<([f32; 3], [f32; 3])> {
    // If both points in front, return as-is
    if start_depth > near_plane && end_depth > near_plane {
        return Some((start, end));
    }
    
    // If both behind, cull
    if start_depth <= near_plane && end_depth <= near_plane {
        return None;
    }
    
    // Clip at near plane
    let t = (near_plane - start_depth) / (end_depth - start_depth);
    let clip_point = [
        start[0] + t * (end[0] - start[0]),
        start[1] + t * (end[1] - start[1]),
        start[2] + t * (end[2] - start[2]),
    ];
    
    if start_depth > near_plane {
        Some((start, clip_point))
    } else {
        Some((clip_point, end))
    }
}
```

### 2. Dynamic Grid Levels
```rust
struct GridLevel {
    spacing: f32,
    extent: f32,
    minor_alpha: f32,
    major_alpha: f32,
}

fn get_grid_level(camera_height: f32) -> GridLevel {
    if camera_height < 10.0 {
        GridLevel { 
            spacing: 1.0, 
            extent: 50.0,
            minor_alpha: 0.3,
            major_alpha: 0.6,
        }
    } else if camera_height < 50.0 {
        GridLevel { 
            spacing: 10.0, 
            extent: 200.0,
            minor_alpha: 0.2,
            major_alpha: 0.5,
        }
    } else {
        GridLevel { 
            spacing: 100.0, 
            extent: 1000.0,
            minor_alpha: 0.1,
            major_alpha: 0.4,
        }
    }
}
```

### 3. Distance-Based Fading
```rust
fn calculate_line_opacity(distance: f32, base_alpha: f32) -> f32 {
    let fade_start = 20.0;
    let fade_end = 100.0;
    
    if distance < fade_start {
        base_alpha
    } else if distance > fade_end {
        0.0
    } else {
        let t = (distance - fade_start) / (fade_end - fade_start);
        base_alpha * (1.0 - t)
    }
}
```

## Benefits
1. **Consistent Rendering**: Grid always visible and properly rendered
2. **Professional Appearance**: Matches Unity/Unreal quality
3. **Better Performance**: Smart culling and LOD
4. **Improved Usability**: Clear spatial reference at all times
5. **Scalability**: Works at any viewing distance/angle

## Testing Criteria
1. Grid lines don't disappear when camera moves
2. Smooth transitions between grid levels
3. Proper clipping at camera near plane
4. Consistent visual density
5. Good performance with large grids

## Next Steps
1. Implement Phase 1 fixes
2. Test and iterate
3. Move to Phase 2 enhancements
4. Consider Phase 3 for future releases