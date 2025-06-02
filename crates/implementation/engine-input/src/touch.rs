//! Touch input handling

/// Touch phase
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

/// Touch input data
#[derive(Debug, Clone)]
pub struct TouchInput {
    pub id: u64,
    pub phase: TouchPhase,
    pub position: (f32, f32),
    pub force: Option<f32>,
}

impl TouchInput {
    /// Create a new touch input
    pub fn new(id: u64, phase: TouchPhase, position: (f32, f32)) -> Self {
        Self {
            id,
            phase,
            position,
            force: None,
        }
    }
}