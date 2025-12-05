use crate::types::AssetId;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Name component for entities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Enabled component for toggling entity behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enabled(pub bool);

impl Enabled {
    pub fn new(enabled: bool) -> Self {
        Self(enabled)
    }

    pub fn is_enabled(&self) -> bool {
        self.0
    }

    pub fn enable(&mut self) {
        self.0 = true;
    }

    pub fn disable(&mut self) {
        self.0 = false;
    }

    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

impl Default for Enabled {
    fn default() -> Self {
        Self(true)
    }
}

/// Sprite component for rendering
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: AssetId,
    pub size: Vec2,
    pub color: [f32; 4], // RGBA
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Sprite {
    /// Create a new sprite with default white color
    pub fn new(texture: AssetId, size: Vec2) -> Self {
        Self {
            texture,
            size,
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
        }
    }

    /// Create a sprite with a specific color tint
    pub fn with_color(texture: AssetId, size: Vec2, color: [f32; 4]) -> Self {
        Self {
            texture,
            size,
            color,
            flip_x: false,
            flip_y: false,
        }
    }

    /// Set the color tint
    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    /// Set the alpha value
    pub fn set_alpha(&mut self, alpha: f32) {
        self.color[3] = alpha;
    }

    /// Flip the sprite horizontally
    pub fn flip_horizontal(&mut self, flip: bool) {
        self.flip_x = flip;
    }

    /// Flip the sprite vertically
    pub fn flip_vertical(&mut self, flip: bool) {
        self.flip_y = flip;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_component() {
        let name = Name::new("Player");
        assert_eq!(name.as_str(), "Player");

        let name: Name = "Enemy".into();
        assert_eq!(name.as_str(), "Enemy");
    }

    #[test]
    fn test_enabled_component() {
        let mut enabled = Enabled::default();
        assert!(enabled.is_enabled());

        enabled.disable();
        assert!(!enabled.is_enabled());

        enabled.toggle();
        assert!(enabled.is_enabled());
    }

    #[test]
    fn test_sprite_component() {
        let texture = AssetId::new(1);
        let mut sprite = Sprite::new(texture, Vec2::new(32.0, 32.0));

        assert_eq!(sprite.texture, texture);
        assert_eq!(sprite.size, Vec2::new(32.0, 32.0));
        assert_eq!(sprite.color, [1.0, 1.0, 1.0, 1.0]);

        sprite.set_alpha(0.5);
        assert_eq!(sprite.color[3], 0.5);

        sprite.flip_horizontal(true);
        assert!(sprite.flip_x);
    }

    #[test]
    fn test_sprite_with_color() {
        let texture = AssetId::new(1);
        let color = [1.0, 0.0, 0.0, 0.5];
        let sprite = Sprite::with_color(texture, Vec2::new(32.0, 32.0), color);

        assert_eq!(sprite.color, color);
    }
}
