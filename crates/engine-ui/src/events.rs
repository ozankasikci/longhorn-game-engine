//! UI event handling

/// UI event types
#[derive(Debug, Clone)]
pub enum UiEvent {
    ButtonClicked { id: String },
    TextChanged { id: String, text: String },
    ValueChanged { id: String, value: f64 },
    WindowClosed { id: String },
}

/// UI event handler trait
pub trait UiEventHandler {
    /// Handle a UI event
    fn handle_event(&mut self, event: UiEvent);
}

/// UI event manager
pub struct UiEventManager {
    handlers: Vec<Box<dyn UiEventHandler>>,
}

impl UiEventManager {
    /// Create a new UI event manager
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    
    /// Add an event handler
    pub fn add_handler(&mut self, handler: Box<dyn UiEventHandler>) {
        self.handlers.push(handler);
    }
    
    /// Dispatch an event to all handlers
    pub fn dispatch(&mut self, event: UiEvent) {
        for handler in &mut self.handlers {
            handler.handle_event(event.clone());
        }
    }
}