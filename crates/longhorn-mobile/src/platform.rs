use longhorn_input::TouchEvent;

/// Platform events.
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    Touch(TouchEvent),
    Resize { width: u32, height: u32 },
    Suspend,
    Resume,
    Quit,
}

/// Platform trait for abstracting OS-specific behavior.
pub trait Platform {
    fn get_display_size(&self) -> (u32, u32);
    fn poll_events(&mut self) -> Vec<PlatformEvent>;
}
