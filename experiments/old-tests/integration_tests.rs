use mobile_game_engine::*;

#[test]
fn test_editor_state_integration() {
    let mut state = EditorState::new();
    
    // Test initial state
    assert_eq!(state.object_count(), 2); // Camera and light
    assert!(state.message_count() > 0);
    
    // Create several objects
    let cube_id = state.create_object("Cube".to_string());
    let sphere_id = state.create_object("Sphere".to_string());
    let plane_id = state.create_object("Plane".to_string());
    
    assert_eq!(state.object_count(), 5);
    
    // Test object selection workflow
    assert!(state.select_object(cube_id));
    assert_eq!(state.selected_object, Some(cube_id));
    
    // Test object modification workflow
    {
        let cube = state.get_object_mut(cube_id).unwrap();
        cube.transform.position = [5.0, 0.0, 0.0];
        cube.name = "Modified Cube".to_string();
    }
    
    let cube = state.get_object(cube_id).unwrap();
    assert_eq!(cube.name, "Modified Cube");
    assert_eq!(cube.transform.position, [5.0, 0.0, 0.0]);
    
    // Test deletion workflow
    assert!(state.delete_object(sphere_id));
    assert_eq!(state.object_count(), 4);
    assert!(state.get_object(sphere_id).is_none());
    
    // Test console logging throughout
    let initial_message_count = state.message_count();
    state.log_warning("Test warning in integration");
    assert_eq!(state.message_count(), initial_message_count + 1);
}

#[test]
fn test_gameobject_hierarchy() {
    let mut state = EditorState::new();
    
    let parent_id = state.create_object("Parent".to_string());
    let child1_id = state.create_object("Child1".to_string());
    let child2_id = state.create_object("Child2".to_string());
    
    // Set up hierarchy relationships
    {
        let parent = state.get_object_mut(parent_id).unwrap();
        parent.children.push(child1_id);
        parent.children.push(child2_id);
    }
    
    {
        let child1 = state.get_object_mut(child1_id).unwrap();
        child1.parent = Some(parent_id);
    }
    
    {
        let child2 = state.get_object_mut(child2_id).unwrap();
        child2.parent = Some(parent_id);
    }
    
    // Verify hierarchy
    let parent = state.get_object(parent_id).unwrap();
    assert_eq!(parent.children.len(), 2);
    assert!(parent.children.contains(&child1_id));
    assert!(parent.children.contains(&child2_id));
    
    let child1 = state.get_object(child1_id).unwrap();
    assert_eq!(child1.parent, Some(parent_id));
    
    let child2 = state.get_object(child2_id).unwrap();
    assert_eq!(child2.parent, Some(parent_id));
}

#[test]
fn test_scene_management() {
    let mut state = EditorState::new();
    
    // Add some objects to the scene
    let cube_id = state.create_object("Cube".to_string());
    let sphere_id = state.create_object("Sphere".to_string());
    
    assert_eq!(state.object_count(), 4); // 2 default + 2 created
    
    // Simulate scene clear (keeping default objects)
    let default_objects: Vec<_> = state.scene_objects
        .iter()
        .filter(|(_, obj)| obj.name == "Main Camera" || obj.name == "Directional Light")
        .map(|(id, _)| *id)
        .collect();
    
    // Clear all non-default objects
    state.scene_objects.retain(|id, obj| {
        obj.name == "Main Camera" || obj.name == "Directional Light"
    });
    
    state.selected_object = None;
    
    assert_eq!(state.object_count(), 2); // Only default objects remain
    assert_eq!(state.selected_object, None);
    
    // Verify default objects still exist
    assert!(default_objects.iter().all(|id| state.scene_objects.contains_key(id)));
}

#[test]
fn test_editor_state_consistency() {
    let mut state = EditorState::new();
    
    // Create objects and track IDs
    let mut created_ids = Vec::new();
    for i in 0..10 {
        let id = state.create_object(format!("Object{}", i));
        created_ids.push(id);
    }
    
    // Verify all objects were created with unique IDs
    assert_eq!(created_ids.len(), 10);
    let mut sorted_ids = created_ids.clone();
    sorted_ids.sort();
    sorted_ids.dedup();
    assert_eq!(sorted_ids.len(), 10); // All IDs should be unique
    
    // Verify all objects exist in the state
    for id in &created_ids {
        assert!(state.get_object(*id).is_some());
    }
    
    // Test deleting every other object
    for (i, id) in created_ids.iter().enumerate() {
        if i % 2 == 0 {
            assert!(state.delete_object(*id));
        }
    }
    
    // Verify correct number of objects remain
    let remaining_objects = created_ids.iter()
        .filter(|id| state.get_object(**id).is_some())
        .count();
    assert_eq!(remaining_objects, 5);
}

#[test]
fn test_transform_operations() {
    let mut state = EditorState::new();
    let cube_id = state.create_object("Cube".to_string());
    
    // Test position modification
    {
        let cube = state.get_object_mut(cube_id).unwrap();
        cube.transform.position[0] += 5.0;
        cube.transform.position[1] -= 2.0;
        cube.transform.position[2] *= 3.0;
    }
    
    let cube = state.get_object(cube_id).unwrap();
    assert_eq!(cube.transform.position, [5.0, -2.0, 0.0]);
    
    // Test rotation modification
    {
        let cube = state.get_object_mut(cube_id).unwrap();
        cube.transform.rotation = [90.0, 45.0, 30.0];
    }
    
    let cube = state.get_object(cube_id).unwrap();
    assert_eq!(cube.transform.rotation, [90.0, 45.0, 30.0]);
    
    // Test scale modification
    {
        let cube = state.get_object_mut(cube_id).unwrap();
        cube.transform.scale = [2.0, 0.5, 1.5];
    }
    
    let cube = state.get_object(cube_id).unwrap();
    assert_eq!(cube.transform.scale, [2.0, 0.5, 1.5]);
}