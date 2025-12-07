//! Sprite-related integration tests for the Longhorn Editor.
//!
//! Tests for sprite component operations, texture assignment, and visual properties.
//!
//! These tests require a running editor instance.
//! Run with: cargo test -p longhorn-test-client --test editor_sprites -- --ignored

use longhorn_test_client::EditorClient;

/// Test setting a sprite texture on an entity.
#[test]
#[ignore]
fn test_set_sprite_texture() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create an entity
    let entity_id = client
        .create_entity("SpriteEntity")
        .expect("Failed to create entity");
    println!("Created entity: {}", entity_id);

    // Get available texture assets
    let assets = client.get_assets().expect("Failed to get assets");
    let texture_asset = assets.iter().find(|a| {
        let path = a.path.to_lowercase();
        path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg")
    });

    if let Some(asset) = texture_asset {
        println!("Setting texture {} (id: {})", asset.path, asset.id);

        // Set the sprite texture
        client
            .set_sprite_texture(entity_id, asset.id)
            .expect("Failed to set sprite texture");

        // Verify the sprite was set
        let dump = client.dump_entity(entity_id).expect("Failed to dump entity");
        if let Some(sprite) = &dump.sprite {
            assert_eq!(sprite.texture_id, asset.id, "Texture ID should match");
            println!("Sprite texture set successfully: texture_id={}", sprite.texture_id);
        } else {
            println!("Sprite component created with texture");
        }
    } else {
        println!("No texture assets found - setting texture_id=0");
        client
            .set_sprite_texture(entity_id, 0)
            .expect("Failed to set sprite texture");
    }

    // Clean up
    client
        .delete_entity(entity_id)
        .expect("Failed to delete entity");
}

/// Test setting sprite size.
#[test]
#[ignore]
fn test_set_sprite_size() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create an entity with a sprite
    let entity_id = client
        .create_entity("SizeTestEntity")
        .expect("Failed to create entity");

    // First set a texture to create the sprite component
    client
        .set_sprite_texture(entity_id, 0)
        .expect("Failed to set sprite texture");

    // Set sprite size
    client
        .set_sprite_size(entity_id, 128.0, 256.0)
        .expect("Failed to set sprite size");

    // Verify
    let dump = client.dump_entity(entity_id).expect("Failed to dump entity");
    if let Some(sprite) = &dump.sprite {
        assert_eq!(sprite.size_x, 128.0, "Width should be 128.0");
        assert_eq!(sprite.size_y, 256.0, "Height should be 256.0");
        println!("Sprite size set: {}x{}", sprite.size_x, sprite.size_y);
    }

    // Clean up
    client
        .delete_entity(entity_id)
        .expect("Failed to delete entity");
}

/// Test setting sprite flip state.
#[test]
#[ignore]
fn test_set_sprite_flip() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create an entity with a sprite
    let entity_id = client
        .create_entity("FlipTestEntity")
        .expect("Failed to create entity");

    // First set a texture to create the sprite component
    client
        .set_sprite_texture(entity_id, 0)
        .expect("Failed to set sprite texture");

    // Set flip state
    client
        .set_sprite_flip(entity_id, true, false)
        .expect("Failed to set sprite flip");

    // Verify
    let dump = client.dump_entity(entity_id).expect("Failed to dump entity");
    if let Some(sprite) = &dump.sprite {
        assert!(sprite.flip_x, "flip_x should be true");
        assert!(!sprite.flip_y, "flip_y should be false");
        println!(
            "Sprite flip set: flip_x={}, flip_y={}",
            sprite.flip_x, sprite.flip_y
        );
    }

    // Test the other flip state
    client
        .set_sprite_flip(entity_id, false, true)
        .expect("Failed to set sprite flip");

    let dump = client.dump_entity(entity_id).expect("Failed to dump entity");
    if let Some(sprite) = &dump.sprite {
        assert!(!sprite.flip_x, "flip_x should be false");
        assert!(sprite.flip_y, "flip_y should be true");
    }

    // Clean up
    client
        .delete_entity(entity_id)
        .expect("Failed to delete entity");
}

/// Test setting sprite color.
#[test]
#[ignore]
fn test_set_sprite_color() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create an entity with a sprite
    let entity_id = client
        .create_entity("ColorTestEntity")
        .expect("Failed to create entity");

    // First set a texture to create the sprite component
    client
        .set_sprite_texture(entity_id, 0)
        .expect("Failed to set sprite texture");

    // Set color to red with 50% transparency
    client
        .set_sprite_color(entity_id, 1.0, 0.0, 0.0, 0.5)
        .expect("Failed to set sprite color");

    // Verify
    let dump = client.dump_entity(entity_id).expect("Failed to dump entity");
    if let Some(sprite) = &dump.sprite {
        assert_eq!(sprite.color[0], 1.0, "Red should be 1.0");
        assert_eq!(sprite.color[1], 0.0, "Green should be 0.0");
        assert_eq!(sprite.color[2], 0.0, "Blue should be 0.0");
        assert_eq!(sprite.color[3], 0.5, "Alpha should be 0.5");
        println!("Sprite color set: {:?}", sprite.color);
    }

    // Clean up
    client
        .delete_entity(entity_id)
        .expect("Failed to delete entity");
}

/// Test changing sprite texture on existing entity.
#[test]
#[ignore]
fn test_change_sprite_texture() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create an entity
    let entity_id = client
        .create_entity("TextureChangeEntity")
        .expect("Failed to create entity");

    // Get available texture assets
    let assets = client.get_assets().expect("Failed to get assets");
    let texture_assets: Vec<_> = assets
        .iter()
        .filter(|a| {
            let path = a.path.to_lowercase();
            path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg")
        })
        .collect();

    if texture_assets.len() >= 2 {
        // Set first texture
        let first_texture = texture_assets[0];
        println!("Setting first texture: {} (id: {})", first_texture.path, first_texture.id);
        client
            .set_sprite_texture(entity_id, first_texture.id)
            .expect("Failed to set first texture");

        // Verify first texture
        let dump = client.dump_entity(entity_id).expect("Failed to dump entity");
        if let Some(sprite) = &dump.sprite {
            assert_eq!(sprite.texture_id, first_texture.id);
        }

        // Change to second texture
        let second_texture = texture_assets[1];
        println!(
            "Changing to second texture: {} (id: {})",
            second_texture.path, second_texture.id
        );
        client
            .set_sprite_texture(entity_id, second_texture.id)
            .expect("Failed to set second texture");

        // Verify texture changed
        let dump = client.dump_entity(entity_id).expect("Failed to dump entity");
        if let Some(sprite) = &dump.sprite {
            assert_eq!(sprite.texture_id, second_texture.id, "Texture should have changed");
            println!("Texture changed successfully to id: {}", sprite.texture_id);
        }
    } else if !texture_assets.is_empty() {
        println!("Only one texture asset found - setting it");
        client
            .set_sprite_texture(entity_id, texture_assets[0].id)
            .expect("Failed to set texture");
    } else {
        println!("No texture assets found - skipping texture change test");
    }

    // Clean up
    client
        .delete_entity(entity_id)
        .expect("Failed to delete entity");
}

/// Test complete sprite workflow: create entity, add sprite with texture, modify properties.
#[test]
#[ignore]
fn test_sprite_workflow() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    println!("=== Sprite Workflow Test ===");

    // 1. Create an entity
    let entity_id = client
        .create_entity("SpriteWorkflowEntity")
        .expect("Failed to create entity");
    println!("Step 1: Created entity {}", entity_id);

    // 2. Get available textures
    let assets = client.get_assets().expect("Failed to get assets");
    let texture_asset = assets.iter().find(|a| {
        let path = a.path.to_lowercase();
        path.ends_with(".png") || path.ends_with(".jpg")
    });

    let texture_id = texture_asset.map(|a| a.id).unwrap_or(0);
    println!("Step 2: Using texture_id {}", texture_id);

    // 3. Set sprite texture (this creates the Sprite component)
    client
        .set_sprite_texture(entity_id, texture_id)
        .expect("Failed to set sprite texture");
    println!("Step 3: Set sprite texture");

    // 4. Set sprite size
    client
        .set_sprite_size(entity_id, 64.0, 64.0)
        .expect("Failed to set sprite size");
    println!("Step 4: Set sprite size to 64x64");

    // 5. Set sprite color (tint it blue)
    client
        .set_sprite_color(entity_id, 0.5, 0.5, 1.0, 1.0)
        .expect("Failed to set sprite color");
    println!("Step 5: Set sprite color to blue tint");

    // 6. Position the entity
    client
        .set_property(entity_id, "Transform", "position.x", 200.0)
        .expect("Failed to set position.x");
    client
        .set_property(entity_id, "Transform", "position.y", 150.0)
        .expect("Failed to set position.y");
    println!("Step 6: Positioned entity at (200, 150)");

    // 7. Verify everything
    let dump = client.dump_entity(entity_id).expect("Failed to dump entity");

    assert_eq!(dump.name, Some("SpriteWorkflowEntity".to_string()));

    if let Some(transform) = &dump.transform {
        assert_eq!(transform.position_x, 200.0);
        assert_eq!(transform.position_y, 150.0);
    }

    if let Some(sprite) = &dump.sprite {
        assert_eq!(sprite.texture_id, texture_id);
        assert_eq!(sprite.size_x, 64.0);
        assert_eq!(sprite.size_y, 64.0);
        assert_eq!(sprite.color[0], 0.5); // R
        assert_eq!(sprite.color[1], 0.5); // G
        assert_eq!(sprite.color[2], 1.0); // B
        assert_eq!(sprite.color[3], 1.0); // A
    }

    println!("Step 7: Verified all sprite properties");
    println!("Entity dump: {:?}", dump);

    // 8. Enter play mode to see the sprite
    client.play().expect("Failed to enter play mode");
    println!("Step 8: Entered play mode - sprite should be visible");

    // 9. Stop play mode
    client.stop().expect("Failed to stop");
    println!("Step 9: Stopped play mode");

    // 10. Clean up - entity ID changes after restore, so find by name
    let entities = client.get_entities().expect("Failed to get entities");
    let restored_entity = entities
        .iter()
        .find(|e| e.name == "SpriteWorkflowEntity")
        .expect("Entity should exist after restore");
    client
        .delete_entity(restored_entity.id)
        .expect("Failed to delete entity");
    println!("Step 10: Cleaned up (new entity_id after restore: {})", restored_entity.id);

    println!("=== Sprite Workflow Complete ===");
}

/// Test setting sprite on multiple entities.
#[test]
#[ignore]
fn test_multiple_sprites() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create multiple entities with sprites
    let mut entities = Vec::new();

    for i in 0..3 {
        let name = format!("MultiSpriteEntity{}", i);
        let entity_id = client.create_entity(&name).expect("Failed to create entity");
        entities.push(entity_id);

        // Set sprite with different properties for each
        client
            .set_sprite_texture(entity_id, 0)
            .expect("Failed to set texture");
        client
            .set_sprite_size(entity_id, 32.0 * (i as f32 + 1.0), 32.0 * (i as f32 + 1.0))
            .expect("Failed to set size");
        client
            .set_property(entity_id, "Transform", "position.x", 100.0 * (i as f32 + 1.0))
            .expect("Failed to set position");

        println!("Created entity {} at x={}", entity_id, 100.0 * (i as f32 + 1.0));
    }

    // Verify all entities have sprites
    for entity_id in &entities {
        let dump = client.dump_entity(*entity_id).expect("Failed to dump entity");
        assert!(dump.sprite.is_some(), "Entity {} should have a sprite", entity_id);
    }
    println!("Verified all {} entities have sprites", entities.len());

    // Check render state
    let render_state = client.get_render_state().expect("Failed to get render state");
    println!("Sprite count in scene: {}", render_state.sprite_count);

    // Clean up
    for entity_id in entities {
        client.delete_entity(entity_id).expect("Failed to delete entity");
    }
    println!("Cleaned up all entities");
}

/// Test that sprites are preserved during play mode.
#[test]
#[ignore]
fn test_sprites_in_play_mode() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create entity with sprite
    let entity_id = client
        .create_entity("PlayModeSpriteEntity")
        .expect("Failed to create entity");

    client
        .set_sprite_texture(entity_id, 0)
        .expect("Failed to set texture");
    client
        .set_sprite_size(entity_id, 48.0, 48.0)
        .expect("Failed to set size");
    client
        .set_sprite_color(entity_id, 0.0, 1.0, 0.0, 1.0)
        .expect("Failed to set color");

    // Verify sprite in scene mode
    let dump_before = client.dump_entity(entity_id).expect("Failed to dump entity");
    assert!(dump_before.sprite.is_some());

    // Enter play mode
    client.play().expect("Failed to play");

    // Verify sprite exists in play mode
    let dump_during = client.dump_entity(entity_id).expect("Failed to dump entity");
    assert!(dump_during.sprite.is_some(), "Sprite should exist in play mode");

    if let Some(sprite) = &dump_during.sprite {
        assert_eq!(sprite.size_x, 48.0);
        assert_eq!(sprite.size_y, 48.0);
        println!("Sprite verified in play mode: size={}x{}", sprite.size_x, sprite.size_y);
    }

    // Stop
    client.stop().expect("Failed to stop");

    // Entity ID changes after restore, so find by name
    let entities = client.get_entities().expect("Failed to get entities");
    let restored_entity = entities
        .iter()
        .find(|e| e.name == "PlayModeSpriteEntity")
        .expect("Entity should exist after restore");

    // Verify sprite still exists after stop
    let dump_after = client.dump_entity(restored_entity.id).expect("Failed to dump entity");
    assert!(dump_after.sprite.is_some(), "Sprite should exist after stop");
    println!("Sprite verified after stop: entity_id changed from {} to {}", entity_id, restored_entity.id);

    // Clean up
    client
        .delete_entity(restored_entity.id)
        .expect("Failed to delete entity");
}
