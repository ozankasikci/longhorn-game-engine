use glam::Vec2;
use crate::events::TouchEvent;
use crate::touch::Touch;

/// Manages input state for the application.
#[derive(Debug, Clone)]
pub struct InputState {
    /// The current touch state.
    touch: Touch,
    /// Whether a touch was just pressed this frame.
    just_pressed: bool,
    /// Whether a touch was just released this frame.
    just_released: bool,
    /// Events received this frame.
    events_this_frame: Vec<TouchEvent>,
}

impl InputState {
    /// Creates a new InputState with default values.
    pub fn new() -> Self {
        Self {
            touch: Touch::new(),
            just_pressed: false,
            just_released: false,
            events_this_frame: Vec::new(),
        }
    }

    /// Resets per-frame state. Call this at the beginning of each frame.
    pub fn begin_frame(&mut self) {
        self.just_pressed = false;
        self.just_released = false;
        self.events_this_frame.clear();
    }

    /// Handles a touch event and updates the input state.
    pub fn handle_event(&mut self, event: TouchEvent) {
        // Check for just_pressed and just_released
        if event.is_start() {
            self.just_pressed = true;
        } else if event.is_end() {
            self.just_released = true;
        }

        // Store the event
        self.events_this_frame.push(event);

        // Update touch state
        self.touch.handle_event(event);
    }

    /// Returns whether a touch is currently active.
    pub fn is_touching(&self) -> bool {
        self.touch.is_down()
    }

    /// Returns whether a touch was just pressed this frame.
    pub fn just_pressed(&self) -> bool {
        self.just_pressed
    }

    /// Returns whether a touch was just released this frame.
    pub fn just_released(&self) -> bool {
        self.just_released
    }

    /// Returns the current touch position.
    pub fn position(&self) -> Vec2 {
        self.touch.position()
    }

    /// Returns the drag delta (distance from start position to current position).
    pub fn drag_delta(&self) -> Vec2 {
        self.touch.drag_delta()
    }

    /// Returns all events received this frame.
    pub fn events(&self) -> &[TouchEvent] {
        &self.events_this_frame
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_lifecycle() {
        let mut state = InputState::new();

        // Initial state
        assert!(!state.is_touching());
        assert!(!state.just_pressed());
        assert!(!state.just_released());
        assert_eq!(state.position(), Vec2::ZERO);
        assert_eq!(state.drag_delta(), Vec2::ZERO);
        assert_eq!(state.events().len(), 0);

        // Touch start
        state.handle_event(TouchEvent::Start { x: 10.0, y: 20.0 });
        assert!(state.is_touching());
        assert!(state.just_pressed());
        assert!(!state.just_released());
        assert_eq!(state.position(), Vec2::new(10.0, 20.0));
        assert_eq!(state.drag_delta(), Vec2::ZERO);
        assert_eq!(state.events().len(), 1);

        // New frame - just_pressed should be cleared
        state.begin_frame();
        assert!(state.is_touching());
        assert!(!state.just_pressed());
        assert!(!state.just_released());
        assert_eq!(state.events().len(), 0);

        // Touch move
        state.handle_event(TouchEvent::Move { x: 15.0, y: 25.0 });
        assert!(state.is_touching());
        assert!(!state.just_pressed());
        assert!(!state.just_released());
        assert_eq!(state.position(), Vec2::new(15.0, 25.0));
        assert_eq!(state.drag_delta(), Vec2::new(5.0, 5.0));
        assert_eq!(state.events().len(), 1);

        // New frame
        state.begin_frame();
        assert!(state.is_touching());
        assert!(!state.just_pressed());
        assert!(!state.just_released());
        assert_eq!(state.events().len(), 0);

        // Touch end
        state.handle_event(TouchEvent::End { x: 20.0, y: 30.0 });
        assert!(!state.is_touching());
        assert!(!state.just_pressed());
        assert!(state.just_released());
        assert_eq!(state.position(), Vec2::new(20.0, 30.0));
        assert_eq!(state.drag_delta(), Vec2::new(10.0, 10.0));
        assert_eq!(state.events().len(), 1);

        // New frame - just_released should be cleared
        state.begin_frame();
        assert!(!state.is_touching());
        assert!(!state.just_pressed());
        assert!(!state.just_released());
        assert_eq!(state.events().len(), 0);
    }

    #[test]
    fn test_drag_delta_tracking() {
        let mut state = InputState::new();

        // Start touch
        state.handle_event(TouchEvent::Start { x: 100.0, y: 200.0 });
        assert_eq!(state.drag_delta(), Vec2::ZERO);

        // Move touch multiple times
        state.begin_frame();
        state.handle_event(TouchEvent::Move { x: 110.0, y: 210.0 });
        assert_eq!(state.drag_delta(), Vec2::new(10.0, 10.0));

        state.begin_frame();
        state.handle_event(TouchEvent::Move { x: 150.0, y: 250.0 });
        assert_eq!(state.drag_delta(), Vec2::new(50.0, 50.0));

        state.begin_frame();
        state.handle_event(TouchEvent::Move { x: 90.0, y: 190.0 });
        assert_eq!(state.drag_delta(), Vec2::new(-10.0, -10.0));

        // End touch
        state.begin_frame();
        state.handle_event(TouchEvent::End { x: 120.0, y: 220.0 });
        assert_eq!(state.drag_delta(), Vec2::new(20.0, 20.0));
    }

    #[test]
    fn test_multiple_events_per_frame() {
        let mut state = InputState::new();

        // Simulate multiple events in one frame
        state.handle_event(TouchEvent::Start { x: 10.0, y: 20.0 });
        state.handle_event(TouchEvent::Move { x: 15.0, y: 25.0 });
        state.handle_event(TouchEvent::Move { x: 20.0, y: 30.0 });

        assert_eq!(state.events().len(), 3);
        assert!(state.just_pressed());
        assert!(!state.just_released());
        assert_eq!(state.position(), Vec2::new(20.0, 30.0));

        // New frame
        state.begin_frame();
        assert_eq!(state.events().len(), 0);
    }

    #[test]
    fn test_quick_tap() {
        let mut state = InputState::new();

        // Quick tap: start and end in same frame
        state.handle_event(TouchEvent::Start { x: 10.0, y: 20.0 });
        state.handle_event(TouchEvent::End { x: 10.0, y: 20.0 });

        assert!(!state.is_touching());
        assert!(state.just_pressed());
        assert!(state.just_released());
        assert_eq!(state.events().len(), 2);

        // New frame
        state.begin_frame();
        assert!(!state.just_pressed());
        assert!(!state.just_released());
    }

    #[test]
    fn test_event_storage() {
        let mut state = InputState::new();

        // Add multiple events
        state.handle_event(TouchEvent::Start { x: 1.0, y: 2.0 });
        state.handle_event(TouchEvent::Move { x: 3.0, y: 4.0 });
        state.handle_event(TouchEvent::Move { x: 5.0, y: 6.0 });

        let events = state.events();
        assert_eq!(events.len(), 3);
        assert!(events[0].is_start());
        assert!(events[1].is_move());
        assert!(events[2].is_move());
        assert_eq!(events[0].position(), Vec2::new(1.0, 2.0));
        assert_eq!(events[1].position(), Vec2::new(3.0, 4.0));
        assert_eq!(events[2].position(), Vec2::new(5.0, 6.0));
    }

    #[test]
    fn test_multiple_touch_cycles() {
        let mut state = InputState::new();

        // First touch cycle
        state.handle_event(TouchEvent::Start { x: 10.0, y: 10.0 });
        assert!(state.just_pressed());

        state.begin_frame();
        state.handle_event(TouchEvent::End { x: 20.0, y: 20.0 });
        assert!(state.just_released());

        // Second touch cycle
        state.begin_frame();
        state.handle_event(TouchEvent::Start { x: 30.0, y: 30.0 });
        assert!(state.just_pressed());
        assert!(!state.just_released());
        assert_eq!(state.position(), Vec2::new(30.0, 30.0));
        assert_eq!(state.drag_delta(), Vec2::ZERO); // New touch, delta resets

        state.begin_frame();
        state.handle_event(TouchEvent::Move { x: 40.0, y: 40.0 });
        assert_eq!(state.drag_delta(), Vec2::new(10.0, 10.0));
    }
}
