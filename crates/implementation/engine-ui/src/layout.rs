//! UI layout system

/// Layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

/// Layout alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Layout container
pub struct Layout {
    pub direction: LayoutDirection,
    pub alignment: Alignment,
    pub spacing: f32,
}

impl Layout {
    /// Create a new layout
    pub fn new(direction: LayoutDirection) -> Self {
        Self {
            direction,
            alignment: Alignment::Start,
            spacing: 0.0,
        }
    }

    /// Set alignment
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set spacing
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}
