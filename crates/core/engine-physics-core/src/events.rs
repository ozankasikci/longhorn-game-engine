//! Physics event abstractions

use glam::{Vec2, Vec3};
use serde::{Serialize, Deserialize};
use crate::{BodyHandle, ColliderHandle};

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
                PhysicsEvent::CollisionStarted(e) | 
                PhysicsEvent::CollisionPersisted(e) | 
                PhysicsEvent::CollisionEnded(e) => {
                    listener.on_collision(e);
                }
                PhysicsEvent::SensorEntered(e) | 
                PhysicsEvent::SensorExited(e) => {
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
        
        let sum = self.contacts.iter()
            .map(|c| c.point)
            .fold(Vec3::ZERO, |acc, point| acc + point);
        
        Some(sum / self.contacts.len() as f32)
    }
    
    /// Get the maximum penetration depth
    pub fn max_penetration(&self) -> f32 {
        self.contacts.iter()
            .map(|c| c.penetration)
            .fold(0.0, f32::max)
    }
}