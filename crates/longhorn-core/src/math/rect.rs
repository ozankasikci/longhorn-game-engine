use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Axis-aligned bounding rectangle
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    /// Create a new rectangle from min and max corners
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Create a rectangle from position and size
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self {
            min: pos,
            max: pos + size,
        }
    }

    /// Create a rectangle centered at a position
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Get the width of the rectangle
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Get the height of the rectangle
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Get the size of the rectangle
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Get the center of the rectangle
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// Check if a point is inside the rectangle
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if this rectangle intersects another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Get the intersection of two rectangles
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        Some(Rect {
            min: Vec2::new(
                self.min.x.max(other.min.x),
                self.min.y.max(other.min.y),
            ),
            max: Vec2::new(
                self.max.x.min(other.max.x),
                self.max.y.min(other.max.y),
            ),
        })
    }

    /// Get the union of two rectangles
    pub fn union(&self, other: &Rect) -> Rect {
        Rect {
            min: Vec2::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
            ),
            max: Vec2::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
            ),
        }
    }

    /// Expand the rectangle by a margin
    pub fn expand(&self, margin: f32) -> Rect {
        Rect {
            min: self.min - Vec2::splat(margin),
            max: self.max + Vec2::splat(margin),
        }
    }

    /// Translate the rectangle by a vector
    pub fn translate(&self, offset: Vec2) -> Rect {
        Rect {
            min: self.min + offset,
            max: self.max + offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_new() {
        let rect = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert_eq!(rect.min, Vec2::new(0.0, 0.0));
        assert_eq!(rect.max, Vec2::new(10.0, 10.0));
    }

    #[test]
    fn test_rect_from_pos_size() {
        let rect = Rect::from_pos_size(Vec2::new(5.0, 5.0), Vec2::new(10.0, 10.0));
        assert_eq!(rect.min, Vec2::new(5.0, 5.0));
        assert_eq!(rect.max, Vec2::new(15.0, 15.0));
    }

    #[test]
    fn test_rect_from_center_size() {
        let rect = Rect::from_center_size(Vec2::new(10.0, 10.0), Vec2::new(4.0, 4.0));
        assert_eq!(rect.min, Vec2::new(8.0, 8.0));
        assert_eq!(rect.max, Vec2::new(12.0, 12.0));
    }

    #[test]
    fn test_rect_size() {
        let rect = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 20.0));
        assert_eq!(rect.width(), 10.0);
        assert_eq!(rect.height(), 20.0);
        assert_eq!(rect.size(), Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_rect_center() {
        let rect = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert_eq!(rect.center(), Vec2::new(5.0, 5.0));
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert!(rect.contains(Vec2::new(5.0, 5.0)));
        assert!(rect.contains(Vec2::new(0.0, 0.0)));
        assert!(rect.contains(Vec2::new(10.0, 10.0)));
        assert!(!rect.contains(Vec2::new(-1.0, 5.0)));
        assert!(!rect.contains(Vec2::new(11.0, 5.0)));
    }

    #[test]
    fn test_rect_intersects() {
        let rect1 = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let rect2 = Rect::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        let rect3 = Rect::new(Vec2::new(20.0, 20.0), Vec2::new(30.0, 30.0));

        assert!(rect1.intersects(&rect2));
        assert!(rect2.intersects(&rect1));
        assert!(!rect1.intersects(&rect3));
        assert!(!rect3.intersects(&rect1));
    }

    #[test]
    fn test_rect_intersection() {
        let rect1 = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let rect2 = Rect::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));

        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.min, Vec2::new(5.0, 5.0));
        assert_eq!(intersection.max, Vec2::new(10.0, 10.0));

        let rect3 = Rect::new(Vec2::new(20.0, 20.0), Vec2::new(30.0, 30.0));
        assert!(rect1.intersection(&rect3).is_none());
    }

    #[test]
    fn test_rect_union() {
        let rect1 = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let rect2 = Rect::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));

        let union = rect1.union(&rect2);
        assert_eq!(union.min, Vec2::new(0.0, 0.0));
        assert_eq!(union.max, Vec2::new(15.0, 15.0));
    }

    #[test]
    fn test_rect_expand() {
        let rect = Rect::new(Vec2::new(5.0, 5.0), Vec2::new(10.0, 10.0));
        let expanded = rect.expand(2.0);
        assert_eq!(expanded.min, Vec2::new(3.0, 3.0));
        assert_eq!(expanded.max, Vec2::new(12.0, 12.0));
    }

    #[test]
    fn test_rect_translate() {
        let rect = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let translated = rect.translate(Vec2::new(5.0, 5.0));
        assert_eq!(translated.min, Vec2::new(5.0, 5.0));
        assert_eq!(translated.max, Vec2::new(15.0, 15.0));
    }
}
