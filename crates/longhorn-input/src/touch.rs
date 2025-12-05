use glam::Vec2;
use crate::events::TouchEvent;

/// Represents the state of a touch input.
#[derive(Debug, Clone)]
pub struct Touch {
    /// Current touch position.
    position: Vec2,
    /// Position where the touch started.
    start_position: Vec2,
    /// Whether the touch is currently active.
    is_down: bool,
}

impl Touch {
    /// Creates a new Touch with default values.
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            start_position: Vec2::ZERO,
            is_down: false,
        }
    }

    /// Handles a touch event and updates the touch state.
    pub fn handle_event(&mut self, event: TouchEvent) {
        match event {
            TouchEvent::Start { x, y } => {
                let pos = Vec2::new(x, y);
                self.position = pos;
                self.start_position = pos;
                self.is_down = true;
            }
            TouchEvent::Move { x, y } => {
                self.position = Vec2::new(x, y);
            }
            TouchEvent::End { x, y } => {
                self.position = Vec2::new(x, y);
                self.is_down = false;
            }
        }
    }

    /// Returns the current touch position.
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Returns the position where the touch started.
    pub fn start_position(&self) -> Vec2 {
        self.start_position
    }

    /// Returns whether the touch is currently active.
    pub fn is_down(&self) -> bool {
        self.is_down
    }

    /// Returns the drag delta (distance from start position to current position).
    pub fn drag_delta(&self) -> Vec2 {
        self.position - self.start_position
    }
}

impl Default for Touch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_lifecycle() {
        let mut touch = Touch::new();
        assert!(!touch.is_down());
        assert_eq!(touch.position(), Vec2::ZERO);

        // Start touch
        touch.handle_event(TouchEvent::Start { x: 10.0, y: 20.0 });
        assert!(touch.is_down());
        assert_eq!(touch.position(), Vec2::new(10.0, 20.0));
        assert_eq!(touch.start_position(), Vec2::new(10.0, 20.0));
        assert_eq!(touch.drag_delta(), Vec2::ZERO);

        // Move touch
        touch.handle_event(TouchEvent::Move { x: 15.0, y: 25.0 });
        assert!(touch.is_down());
        assert_eq!(touch.position(), Vec2::new(15.0, 25.0));
        assert_eq!(touch.start_position(), Vec2::new(10.0, 20.0));
        assert_eq!(touch.drag_delta(), Vec2::new(5.0, 5.0));

        // End touch
        touch.handle_event(TouchEvent::End { x: 20.0, y: 30.0 });
        assert!(!touch.is_down());
        assert_eq!(touch.position(), Vec2::new(20.0, 30.0));
        assert_eq!(touch.start_position(), Vec2::new(10.0, 20.0));
        assert_eq!(touch.drag_delta(), Vec2::new(10.0, 10.0));
    }

    #[test]
    fn test_drag_delta() {
        let mut touch = Touch::new();

        touch.handle_event(TouchEvent::Start { x: 100.0, y: 200.0 });
        assert_eq!(touch.drag_delta(), Vec2::ZERO);

        touch.handle_event(TouchEvent::Move { x: 150.0, y: 250.0 });
        assert_eq!(touch.drag_delta(), Vec2::new(50.0, 50.0));

        touch.handle_event(TouchEvent::Move { x: 80.0, y: 180.0 });
        assert_eq!(touch.drag_delta(), Vec2::new(-20.0, -20.0));
    }
}
