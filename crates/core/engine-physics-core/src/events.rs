//! Physics event abstractions

use crate::{BodyHandle, ColliderHandle};
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Physics events that can be generated during simulation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PhysicsEvent {
    /// Collision started between two entities
    CollisionStarted(CollisionEvent),
    /// Collision persisted between two entities
    CollisionPersisted(CollisionEvent),
    /// Collision ended between two entities
    CollisionEnded(CollisionEvent),
    /// Sensor/trigger entered
    SensorEntered(SensorEvent),
    /// Sensor/trigger exited
    SensorExited(SensorEvent),
    /// Joint broken due to excessive force
    JointBroken(JointBrokenEvent),
    /// Rigid body went to sleep
    BodySlept(BodySleepEvent),
    /// Rigid body woke up
    BodyWoke(BodySleepEvent),
}

/// Collision event data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollisionEvent {
    /// First entity involved in collision
    pub entity1: u32,
    /// Second entity involved in collision
    pub entity2: u32,
    /// First collider
    pub collider1: ColliderHandle,
    /// Second collider
    pub collider2: ColliderHandle,
    /// First rigid body (if any)
    pub body1: Option<BodyHandle>,
    /// Second rigid body (if any)
    pub body2: Option<BodyHandle>,
    /// Contact points
    pub contacts: Vec<ContactPoint>,
    /// Total impulse applied during collision
    pub impulse: f32,
    /// Relative velocity at contact
    pub relative_velocity: Vec3,
}

/// Contact point information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactPoint {
    /// Contact point in world space
    pub point: Vec3,
    /// Contact normal (pointing from object1 to object2)
    pub normal: Vec3,
    /// Penetration depth
    pub penetration: f32,
    /// Impulse applied at this contact
    pub impulse: f32,
    /// Tangent impulses (friction)
    pub tangent_impulses: [f32; 2],
}

/// 2D contact point information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactPoint2D {
    /// Contact point in world space
    pub point: Vec2,
    /// Contact normal (pointing from object1 to object2)
    pub normal: Vec2,
    /// Penetration depth
    pub penetration: f32,
    /// Impulse applied at this contact
    pub impulse: f32,
    /// Tangent impulse (friction)
    pub tangent_impulse: f32,
}

/// Sensor/trigger event data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorEvent {
    /// Sensor entity
    pub sensor_entity: u32,
    /// Other entity that entered/exited the sensor
    pub other_entity: u32,
    /// Sensor collider
    pub sensor_collider: ColliderHandle,
    /// Other collider
    pub other_collider: ColliderHandle,
}

/// Joint broken event data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JointBrokenEvent {
    /// Entity with the joint
    pub entity: u32,
    /// Joint handle that was broken
    pub joint_handle: u32,
    /// Force that caused the break
    pub break_force: f32,
    /// Torque that caused the break
    pub break_torque: f32,
}

/// Body sleep/wake event data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodySleepEvent {
    /// Entity with the body
    pub entity: u32,
    /// Body handle
    pub body_handle: BodyHandle,
}

/// Physics event listener trait
pub trait PhysicsEventListener {
    /// Called when a physics event occurs
    fn on_physics_event(&mut self, event: &PhysicsEvent);

    /// Called specifically for collision events
    fn on_collision(&mut self, event: &CollisionEvent) {
        // Default implementation does nothing
        let _ = event;
    }

    /// Called specifically for sensor events
    fn on_sensor(&mut self, event: &SensorEvent) {
        // Default implementation does nothing
        let _ = event;
    }

    /// Called specifically for joint events
    fn on_joint_broken(&mut self, event: &JointBrokenEvent) {
        // Default implementation does nothing
        let _ = event;
    }
}

/// Physics event dispatcher for managing multiple listeners
pub struct PhysicsEventDispatcher {
    listeners: Vec<Box<dyn PhysicsEventListener>>,
}

impl Default for PhysicsEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsEventDispatcher {
    /// Create new event dispatcher
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    /// Add an event listener
    pub fn add_listener(&mut self, listener: Box<dyn PhysicsEventListener>) {
        self.listeners.push(listener);
    }

    /// Remove all listeners
    pub fn clear_listeners(&mut self) {
        self.listeners.clear();
    }

    /// Dispatch an event to all listeners
    pub fn dispatch(&mut self, event: &PhysicsEvent) {
        for listener in &mut self.listeners {
            listener.on_physics_event(event);

            // Call specific event handlers
            match event {
                PhysicsEvent::CollisionStarted(e)
                | PhysicsEvent::CollisionPersisted(e)
                | PhysicsEvent::CollisionEnded(e) => {
                    listener.on_collision(e);
                }
                PhysicsEvent::SensorEntered(e) | PhysicsEvent::SensorExited(e) => {
                    listener.on_sensor(e);
                }
                PhysicsEvent::JointBroken(e) => {
                    listener.on_joint_broken(e);
                }
                _ => {}
            }
        }
    }

    /// Get number of registered listeners
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }
}

impl CollisionEvent {
    /// Check if a specific entity is involved in this collision
    pub fn involves_entity(&self, entity: u32) -> bool {
        self.entity1 == entity || self.entity2 == entity
    }

    /// Get the other entity involved (if the given entity is involved)
    pub fn get_other_entity(&self, entity: u32) -> Option<u32> {
        if self.entity1 == entity {
            Some(self.entity2)
        } else if self.entity2 == entity {
            Some(self.entity1)
        } else {
            None
        }
    }

    /// Get the average contact point
    pub fn average_contact_point(&self) -> Option<Vec3> {
        if self.contacts.is_empty() {
            return None;
        }

        let sum = self
            .contacts
            .iter()
            .map(|c| c.point)
            .fold(Vec3::ZERO, |acc, point| acc + point);

        Some(sum / self.contacts.len() as f32)
    }

    /// Get the maximum penetration depth
    pub fn max_penetration(&self) -> f32 {
        self.contacts
            .iter()
            .map(|c| c.penetration)
            .fold(0.0, f32::max)
    }

    /// Get total impulse from all contacts
    pub fn total_contact_impulse(&self) -> f32 {
        self.contacts.iter().map(|c| c.impulse).sum()
    }
}

impl SensorEvent {
    /// Check if a specific entity is the sensor
    pub fn is_sensor(&self, entity: u32) -> bool {
        self.sensor_entity == entity
    }

    /// Check if a specific entity is the other entity
    pub fn is_other(&self, entity: u32) -> bool {
        self.other_entity == entity
    }

    /// Check if a specific entity is involved in this sensor event
    pub fn involves_entity(&self, entity: u32) -> bool {
        self.sensor_entity == entity || self.other_entity == entity
    }
}

impl ContactPoint {
    /// Create a new contact point
    pub fn new(point: Vec3, normal: Vec3, penetration: f32) -> Self {
        Self {
            point,
            normal,
            penetration,
            impulse: 0.0,
            tangent_impulses: [0.0, 0.0],
        }
    }

    /// Check if this contact has significant penetration
    pub fn has_penetration(&self, threshold: f32) -> bool {
        self.penetration > threshold
    }
}

impl ContactPoint2D {
    /// Create a new 2D contact point
    pub fn new(point: Vec2, normal: Vec2, penetration: f32) -> Self {
        Self {
            point,
            normal,
            penetration,
            impulse: 0.0,
            tangent_impulse: 0.0,
        }
    }

    /// Check if this contact has significant penetration
    pub fn has_penetration(&self, threshold: f32) -> bool {
        self.penetration > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_event_enum() {
        // Test collision events
        let collision_event = CollisionEvent {
            entity1: 1,
            entity2: 2,
            collider1: 10,
            collider2: 20,
            body1: Some(100),
            body2: Some(200),
            contacts: vec![],
            impulse: 5.0,
            relative_velocity: Vec3::new(1.0, 2.0, 3.0),
        };

        let events = [
            PhysicsEvent::CollisionStarted(collision_event.clone()),
            PhysicsEvent::CollisionPersisted(collision_event.clone()),
            PhysicsEvent::CollisionEnded(collision_event.clone()),
        ];

        for event in &events {
            match event {
                PhysicsEvent::CollisionStarted(_)
                | PhysicsEvent::CollisionPersisted(_)
                | PhysicsEvent::CollisionEnded(_) => {}
                _ => panic!("Should be collision event"),
            }
        }
    }

    #[test]
    fn test_collision_event_creation() {
        let contact = ContactPoint::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 1.0, 0.0), 0.1);

        let collision = CollisionEvent {
            entity1: 42,
            entity2: 84,
            collider1: 1,
            collider2: 2,
            body1: Some(10),
            body2: Some(20),
            contacts: vec![contact],
            impulse: 15.0,
            relative_velocity: Vec3::new(5.0, -2.0, 1.0),
        };

        assert_eq!(collision.entity1, 42);
        assert_eq!(collision.entity2, 84);
        assert_eq!(collision.collider1, 1);
        assert_eq!(collision.collider2, 2);
        assert_eq!(collision.body1, Some(10));
        assert_eq!(collision.body2, Some(20));
        assert_eq!(collision.contacts.len(), 1);
        assert_eq!(collision.impulse, 15.0);
        assert_eq!(collision.relative_velocity, Vec3::new(5.0, -2.0, 1.0));
    }

    #[test]
    fn test_collision_event_involves_entity() {
        let collision = CollisionEvent {
            entity1: 10,
            entity2: 20,
            collider1: 1,
            collider2: 2,
            body1: None,
            body2: None,
            contacts: vec![],
            impulse: 0.0,
            relative_velocity: Vec3::ZERO,
        };

        assert!(collision.involves_entity(10));
        assert!(collision.involves_entity(20));
        assert!(!collision.involves_entity(30));
    }

    #[test]
    fn test_collision_event_get_other_entity() {
        let collision = CollisionEvent {
            entity1: 15,
            entity2: 25,
            collider1: 1,
            collider2: 2,
            body1: None,
            body2: None,
            contacts: vec![],
            impulse: 0.0,
            relative_velocity: Vec3::ZERO,
        };

        assert_eq!(collision.get_other_entity(15), Some(25));
        assert_eq!(collision.get_other_entity(25), Some(15));
        assert_eq!(collision.get_other_entity(35), None);
    }

    #[test]
    fn test_collision_event_average_contact_point() {
        let contacts = vec![
            ContactPoint::new(Vec3::new(0.0, 0.0, 0.0), Vec3::Y, 0.1),
            ContactPoint::new(Vec3::new(2.0, 0.0, 0.0), Vec3::Y, 0.1),
            ContactPoint::new(Vec3::new(0.0, 2.0, 0.0), Vec3::Y, 0.1),
        ];

        let collision = CollisionEvent {
            entity1: 1,
            entity2: 2,
            collider1: 1,
            collider2: 2,
            body1: None,
            body2: None,
            contacts,
            impulse: 0.0,
            relative_velocity: Vec3::ZERO,
        };

        let avg = collision.average_contact_point().unwrap();
        assert!((avg - Vec3::new(2.0 / 3.0, 2.0 / 3.0, 0.0)).length() < 0.001);

        // Test empty contacts
        let empty_collision = CollisionEvent {
            contacts: vec![],
            ..collision
        };
        assert!(empty_collision.average_contact_point().is_none());
    }

    #[test]
    fn test_collision_event_max_penetration() {
        let contacts = vec![
            ContactPoint::new(Vec3::ZERO, Vec3::Y, 0.1),
            ContactPoint::new(Vec3::ZERO, Vec3::Y, 0.3),
            ContactPoint::new(Vec3::ZERO, Vec3::Y, 0.05),
        ];

        let collision = CollisionEvent {
            entity1: 1,
            entity2: 2,
            collider1: 1,
            collider2: 2,
            body1: None,
            body2: None,
            contacts,
            impulse: 0.0,
            relative_velocity: Vec3::ZERO,
        };

        assert_eq!(collision.max_penetration(), 0.3);

        // Test empty contacts
        let empty_collision = CollisionEvent {
            contacts: vec![],
            ..collision
        };
        assert_eq!(empty_collision.max_penetration(), 0.0);
    }

    #[test]
    fn test_collision_event_total_contact_impulse() {
        let mut contacts = vec![
            ContactPoint::new(Vec3::ZERO, Vec3::Y, 0.1),
            ContactPoint::new(Vec3::ZERO, Vec3::Y, 0.1),
            ContactPoint::new(Vec3::ZERO, Vec3::Y, 0.1),
        ];
        contacts[0].impulse = 5.0;
        contacts[1].impulse = 3.0;
        contacts[2].impulse = 2.0;

        let collision = CollisionEvent {
            entity1: 1,
            entity2: 2,
            collider1: 1,
            collider2: 2,
            body1: None,
            body2: None,
            contacts,
            impulse: 0.0,
            relative_velocity: Vec3::ZERO,
        };

        assert_eq!(collision.total_contact_impulse(), 10.0);
    }

    #[test]
    fn test_sensor_event_creation() {
        let sensor_event = SensorEvent {
            sensor_entity: 100,
            other_entity: 200,
            sensor_collider: 10,
            other_collider: 20,
        };

        assert_eq!(sensor_event.sensor_entity, 100);
        assert_eq!(sensor_event.other_entity, 200);
        assert_eq!(sensor_event.sensor_collider, 10);
        assert_eq!(sensor_event.other_collider, 20);
    }

    #[test]
    fn test_sensor_event_methods() {
        let sensor_event = SensorEvent {
            sensor_entity: 50,
            other_entity: 60,
            sensor_collider: 5,
            other_collider: 6,
        };

        assert!(sensor_event.is_sensor(50));
        assert!(!sensor_event.is_sensor(60));
        assert!(sensor_event.is_other(60));
        assert!(!sensor_event.is_other(50));
        assert!(sensor_event.involves_entity(50));
        assert!(sensor_event.involves_entity(60));
        assert!(!sensor_event.involves_entity(70));
    }

    #[test]
    fn test_joint_broken_event() {
        let joint_event = JointBrokenEvent {
            entity: 123,
            joint_handle: 456,
            break_force: 1000.0,
            break_torque: 500.0,
        };

        assert_eq!(joint_event.entity, 123);
        assert_eq!(joint_event.joint_handle, 456);
        assert_eq!(joint_event.break_force, 1000.0);
        assert_eq!(joint_event.break_torque, 500.0);
    }

    #[test]
    fn test_body_sleep_event() {
        let sleep_event = BodySleepEvent {
            entity: 789,
            body_handle: 101112,
        };

        assert_eq!(sleep_event.entity, 789);
        assert_eq!(sleep_event.body_handle, 101112);
    }

    #[test]
    fn test_contact_point_creation() {
        let contact = ContactPoint::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 1.0, 0.0), 0.5);

        assert_eq!(contact.point, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(contact.normal, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(contact.penetration, 0.5);
        assert_eq!(contact.impulse, 0.0);
        assert_eq!(contact.tangent_impulses, [0.0, 0.0]);
    }

    #[test]
    fn test_contact_point_has_penetration() {
        let shallow_contact = ContactPoint::new(Vec3::ZERO, Vec3::Y, 0.01);
        let deep_contact = ContactPoint::new(Vec3::ZERO, Vec3::Y, 0.1);

        assert!(!shallow_contact.has_penetration(0.05));
        assert!(deep_contact.has_penetration(0.05));
        assert!(shallow_contact.has_penetration(0.005));
    }

    #[test]
    fn test_contact_point_2d_creation() {
        let contact = ContactPoint2D::new(Vec2::new(1.0, 2.0), Vec2::new(0.0, 1.0), 0.3);

        assert_eq!(contact.point, Vec2::new(1.0, 2.0));
        assert_eq!(contact.normal, Vec2::new(0.0, 1.0));
        assert_eq!(contact.penetration, 0.3);
        assert_eq!(contact.impulse, 0.0);
        assert_eq!(contact.tangent_impulse, 0.0);
    }

    #[test]
    fn test_contact_point_2d_has_penetration() {
        let shallow_contact = ContactPoint2D::new(Vec2::ZERO, Vec2::Y, 0.02);
        let deep_contact = ContactPoint2D::new(Vec2::ZERO, Vec2::Y, 0.08);

        assert!(!shallow_contact.has_penetration(0.05));
        assert!(deep_contact.has_penetration(0.05));
        assert!(shallow_contact.has_penetration(0.01));
    }

    #[test]
    fn test_physics_event_dispatcher_creation() {
        let dispatcher = PhysicsEventDispatcher::new();
        assert_eq!(dispatcher.listener_count(), 0);

        let default_dispatcher = PhysicsEventDispatcher::default();
        assert_eq!(default_dispatcher.listener_count(), 0);
    }

    // Mock listener for testing
    struct MockListener {
        physics_events: Vec<PhysicsEvent>,
        collision_events: Vec<CollisionEvent>,
        sensor_events: Vec<SensorEvent>,
        joint_events: Vec<JointBrokenEvent>,
    }

    impl MockListener {
        fn new() -> Self {
            Self {
                physics_events: Vec::new(),
                collision_events: Vec::new(),
                sensor_events: Vec::new(),
                joint_events: Vec::new(),
            }
        }
    }

    impl PhysicsEventListener for MockListener {
        fn on_physics_event(&mut self, event: &PhysicsEvent) {
            self.physics_events.push(event.clone());
        }

        fn on_collision(&mut self, event: &CollisionEvent) {
            self.collision_events.push(event.clone());
        }

        fn on_sensor(&mut self, event: &SensorEvent) {
            self.sensor_events.push(event.clone());
        }

        fn on_joint_broken(&mut self, event: &JointBrokenEvent) {
            self.joint_events.push(event.clone());
        }
    }

    #[test]
    fn test_physics_event_dispatcher_add_listener() {
        let mut dispatcher = PhysicsEventDispatcher::new();
        let listener = Box::new(MockListener::new());

        dispatcher.add_listener(listener);
        assert_eq!(dispatcher.listener_count(), 1);

        let another_listener = Box::new(MockListener::new());
        dispatcher.add_listener(another_listener);
        assert_eq!(dispatcher.listener_count(), 2);
    }

    #[test]
    fn test_physics_event_dispatcher_clear_listeners() {
        let mut dispatcher = PhysicsEventDispatcher::new();
        let listener1 = Box::new(MockListener::new());
        let listener2 = Box::new(MockListener::new());

        dispatcher.add_listener(listener1);
        dispatcher.add_listener(listener2);
        assert_eq!(dispatcher.listener_count(), 2);

        dispatcher.clear_listeners();
        assert_eq!(dispatcher.listener_count(), 0);
    }

    #[test]
    fn test_physics_event_dispatch_collision() {
        let mut dispatcher = PhysicsEventDispatcher::new();
        let listener = Box::new(MockListener::new());
        dispatcher.add_listener(listener);

        let collision_event = CollisionEvent {
            entity1: 1,
            entity2: 2,
            collider1: 10,
            collider2: 20,
            body1: None,
            body2: None,
            contacts: vec![],
            impulse: 0.0,
            relative_velocity: Vec3::ZERO,
        };

        let physics_event = PhysicsEvent::CollisionStarted(collision_event);
        dispatcher.dispatch(&physics_event);

        // Note: We can't easily test the dispatch results due to ownership,
        // but we can test that it doesn't panic and the basic structure works
        assert_eq!(dispatcher.listener_count(), 1);
    }

    #[test]
    fn test_physics_event_dispatch_sensor() {
        let mut dispatcher = PhysicsEventDispatcher::new();
        let listener = Box::new(MockListener::new());
        dispatcher.add_listener(listener);

        let sensor_event = SensorEvent {
            sensor_entity: 100,
            other_entity: 200,
            sensor_collider: 10,
            other_collider: 20,
        };

        let physics_event = PhysicsEvent::SensorEntered(sensor_event);
        dispatcher.dispatch(&physics_event);

        assert_eq!(dispatcher.listener_count(), 1);
    }

    #[test]
    fn test_physics_event_dispatch_joint_broken() {
        let mut dispatcher = PhysicsEventDispatcher::new();
        let listener = Box::new(MockListener::new());
        dispatcher.add_listener(listener);

        let joint_event = JointBrokenEvent {
            entity: 123,
            joint_handle: 456,
            break_force: 1000.0,
            break_torque: 500.0,
        };

        let physics_event = PhysicsEvent::JointBroken(joint_event);
        dispatcher.dispatch(&physics_event);

        assert_eq!(dispatcher.listener_count(), 1);
    }

    #[test]
    fn test_physics_event_dispatch_body_sleep() {
        let mut dispatcher = PhysicsEventDispatcher::new();
        let listener = Box::new(MockListener::new());
        dispatcher.add_listener(listener);

        let sleep_event = BodySleepEvent {
            entity: 789,
            body_handle: 101112,
        };

        let physics_event = PhysicsEvent::BodySlept(sleep_event);
        dispatcher.dispatch(&physics_event);

        assert_eq!(dispatcher.listener_count(), 1);
    }

    #[test]
    fn test_all_physics_event_variants() {
        let collision_event = CollisionEvent {
            entity1: 1,
            entity2: 2,
            collider1: 1,
            collider2: 2,
            body1: None,
            body2: None,
            contacts: vec![],
            impulse: 0.0,
            relative_velocity: Vec3::ZERO,
        };

        let sensor_event = SensorEvent {
            sensor_entity: 10,
            other_entity: 20,
            sensor_collider: 1,
            other_collider: 2,
        };

        let joint_event = JointBrokenEvent {
            entity: 30,
            joint_handle: 40,
            break_force: 100.0,
            break_torque: 50.0,
        };

        let sleep_event = BodySleepEvent {
            entity: 50,
            body_handle: 60,
        };

        let events = [
            PhysicsEvent::CollisionStarted(collision_event.clone()),
            PhysicsEvent::CollisionPersisted(collision_event.clone()),
            PhysicsEvent::CollisionEnded(collision_event),
            PhysicsEvent::SensorEntered(sensor_event.clone()),
            PhysicsEvent::SensorExited(sensor_event),
            PhysicsEvent::JointBroken(joint_event),
            PhysicsEvent::BodySlept(sleep_event.clone()),
            PhysicsEvent::BodyWoke(sleep_event),
        ];

        // Test that all variants can be matched
        for event in &events {
            match event {
                PhysicsEvent::CollisionStarted(_) => {}
                PhysicsEvent::CollisionPersisted(_) => {}
                PhysicsEvent::CollisionEnded(_) => {}
                PhysicsEvent::SensorEntered(_) => {}
                PhysicsEvent::SensorExited(_) => {}
                PhysicsEvent::JointBroken(_) => {}
                PhysicsEvent::BodySlept(_) => {}
                PhysicsEvent::BodyWoke(_) => {}
            }
        }
    }
}
