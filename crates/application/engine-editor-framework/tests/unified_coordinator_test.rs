//! Tests for the UnifiedEditorCoordinator

use engine_editor_framework::{UnifiedEditorCoordinator, PlayState};

#[test]
fn test_unified_coordinator_creation() {
    let coordinator = UnifiedEditorCoordinator::new();
    
    // Should start in editing mode
    assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Editing);
    
    // Should have valid game context
    let game_context = coordinator.game_context();
    assert!((game_context.time.target_fps() - 60.0).abs() < 0.001);
}

#[test]
fn test_play_mode_transitions() {
    let mut coordinator = UnifiedEditorCoordinator::new();
    
    // Start in editing mode
    assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Editing);
    
    // Transition to play mode
    coordinator.play_state_manager_mut().start();
    assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Playing);
    
    // Update should handle the transition
    coordinator.update(0.016); // 60 FPS frame time
    
    // Pause
    coordinator.play_state_manager_mut().pause();
    assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Paused);
    
    // Resume
    coordinator.play_state_manager_mut().resume();
    assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Playing);
    
    // Stop
    coordinator.play_state_manager_mut().stop();
    assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Editing);
}

#[test]
fn test_frame_updates() {
    let mut coordinator = UnifiedEditorCoordinator::new();
    
    // Start play mode
    coordinator.play_state_manager_mut().start();
    
    // Multiple updates should work
    for _ in 0..10 {
        coordinator.update(0.016); // 60 FPS
    }
    
    // Should still be playing
    assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Playing);
}

#[test]
fn test_interpolation_factor() {
    let coordinator = UnifiedEditorCoordinator::new();
    
    // Should provide a valid interpolation factor
    let interpolation = coordinator.get_interpolation();
    assert!(interpolation >= 0.0 && interpolation <= 1.0);
}