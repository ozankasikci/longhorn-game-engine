//! Entity-related integration tests.
//!
//! Tests for entity CRUD operations.

use longhorn_test_client::EditorClient;

/// Test getting all entities.
#[test]
#[ignore]
fn test_get_entities() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let entities = client.get_entities().expect("Failed to get entities");

    println!("Found {} entities", entities.len());
    for entity in &entities {
        println!("  - {} (id: {})", entity.name, entity.id);
    }
}

/// Test creating and deleting an entity.
#[test]
#[ignore]
fn test_create_and_delete_entity() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let initial_count = client.get_entities().expect("Failed to get entities").len();

    // Create entity
    let id = client.create_entity("TestEntity").expect("Failed to create entity");
    println!("Created entity with id: {}", id);

    // Verify count increased
    let new_count = client.get_entities().expect("Failed to get entities").len();
    assert_eq!(new_count, initial_count + 1, "Entity count should increase by 1");

    // Get the entity details
    let entity = client.get_entity(id).expect("Failed to get entity");
    assert_eq!(entity.name, "TestEntity");

    // Delete entity
    client.delete_entity(id).expect("Failed to delete entity");

    // Verify count back to initial
    let final_count = client.get_entities().expect("Failed to get entities").len();
    assert_eq!(final_count, initial_count, "Entity count should return to initial");
}

/// Test selecting an entity.
#[test]
#[ignore]
fn test_select_entity() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create an entity to select
    let id = client.create_entity("SelectableEntity").expect("Failed to create entity");

    // Select it
    client.select_entity(id).expect("Failed to select entity");

    // Verify it's selected in state
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.selected_entity, Some(id), "Entity should be selected");

    // Clean up
    client.delete_entity(id).expect("Failed to delete entity");
}

/// Test modifying entity properties.
#[test]
#[ignore]
fn test_set_entity_property() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create an entity
    let id = client.create_entity("ModifiableEntity").expect("Failed to create entity");

    // Set transform position
    client.set_property(id, "Transform", "position.x", 100.0).expect("Failed to set position.x");
    client.set_property(id, "Transform", "position.y", 200.0).expect("Failed to set position.y");

    // Verify the changes
    let entity = client.get_entity(id).expect("Failed to get entity");
    if let Some(transform) = entity.transform {
        assert_eq!(transform.position_x, 100.0, "position.x should be 100.0");
        assert_eq!(transform.position_y, 200.0, "position.y should be 200.0");
    } else {
        panic!("Entity should have a transform");
    }

    // Clean up
    client.delete_entity(id).expect("Failed to delete entity");
}

/// Test dumping entity state.
#[test]
#[ignore]
fn test_dump_entity() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Create an entity
    let id = client.create_entity("DumpableEntity").expect("Failed to create entity");

    // Dump its state
    let dump = client.dump_entity(id).expect("Failed to dump entity");

    println!("Entity dump:");
    println!("  id: {}", dump.id);
    println!("  name: {:?}", dump.name);
    println!("  transform: {:?}", dump.transform);
    println!("  sprite: {:?}", dump.sprite);
    println!("  has_script: {}", dump.has_script);
    println!("  components: {:?}", dump.component_names);

    assert_eq!(dump.id, id);
    assert_eq!(dump.name, Some("DumpableEntity".to_string()));

    // Clean up
    client.delete_entity(id).expect("Failed to delete entity");
}
