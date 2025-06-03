//! Physics-specific mathematical calculations

use glam::{Vec2, Vec3};

/// Calculate the moment of inertia for a solid sphere
pub fn sphere_moment_of_inertia(mass: f32, radius: f32) -> f32 {
    (2.0 / 5.0) * mass * radius * radius
}

/// Calculate the moment of inertia for a solid cylinder around its central axis
pub fn cylinder_moment_of_inertia(mass: f32, radius: f32) -> f32 {
    0.5 * mass * radius * radius
}

/// Calculate the moment of inertia for a solid box around its center
pub fn box_moment_of_inertia(mass: f32, width: f32, height: f32, depth: f32) -> Vec3 {
    let factor = mass / 12.0;
    Vec3::new(
        factor * (height * height + depth * depth),
        factor * (width * width + depth * depth),
        factor * (width * width + height * height),
    )
}

/// Calculate the moment of inertia for a thin rod around its center
pub fn rod_moment_of_inertia(mass: f32, length: f32) -> f32 {
    (mass * length * length) / 12.0
}

/// Calculate gravitational force between two objects
pub fn gravitational_force(mass1: f32, mass2: f32, distance: f32, g_constant: f32) -> f32 {
    if distance <= f32::EPSILON {
        0.0
    } else {
        g_constant * mass1 * mass2 / (distance * distance)
    }
}

/// Calculate spring force using Hooke's law
pub fn spring_force(displacement: f32, spring_constant: f32) -> f32 {
    -spring_constant * displacement
}

/// Calculate damping force
pub fn damping_force(velocity: f32, damping_coefficient: f32) -> f32 {
    -damping_coefficient * velocity
}

/// Calculate terminal velocity for an object in a fluid
pub fn terminal_velocity(mass: f32, gravity: f32, drag_coefficient: f32, fluid_density: f32, cross_sectional_area: f32) -> f32 {
    ((2.0 * mass * gravity) / (drag_coefficient * fluid_density * cross_sectional_area)).sqrt()
}

/// Calculate drag force
pub fn drag_force(velocity: f32, drag_coefficient: f32, fluid_density: f32, cross_sectional_area: f32) -> f32 {
    0.5 * drag_coefficient * fluid_density * cross_sectional_area * velocity * velocity
}

/// Calculate centripetal force
pub fn centripetal_force(mass: f32, velocity: f32, radius: f32) -> f32 {
    if radius <= f32::EPSILON {
        0.0
    } else {
        mass * velocity * velocity / radius
    }
}

/// Calculate escape velocity from a gravitational body
pub fn escape_velocity(mass: f32, radius: f32, g_constant: f32) -> f32 {
    if radius <= f32::EPSILON {
        0.0
    } else {
        (2.0 * g_constant * mass / radius).sqrt()
    }
}

/// Calculate orbital velocity for a circular orbit
pub fn orbital_velocity(central_mass: f32, orbital_radius: f32, g_constant: f32) -> f32 {
    if orbital_radius <= f32::EPSILON {
        0.0
    } else {
        (g_constant * central_mass / orbital_radius).sqrt()
    }
}

/// Calculate kinetic energy
pub fn kinetic_energy(mass: f32, velocity: f32) -> f32 {
    0.5 * mass * velocity * velocity
}

/// Calculate rotational kinetic energy
pub fn rotational_kinetic_energy(moment_of_inertia: f32, angular_velocity: f32) -> f32 {
    0.5 * moment_of_inertia * angular_velocity * angular_velocity
}

/// Calculate potential energy in a gravitational field
pub fn gravitational_potential_energy(mass: f32, height: f32, gravity: f32) -> f32 {
    mass * gravity * height
}

/// Calculate elastic potential energy in a spring
pub fn elastic_potential_energy(displacement: f32, spring_constant: f32) -> f32 {
    0.5 * spring_constant * displacement * displacement
}

/// Calculate momentum
pub fn momentum(mass: f32, velocity: Vec3) -> Vec3 {
    mass * velocity
}

/// Calculate angular momentum for a point mass
pub fn angular_momentum(position: Vec3, momentum: Vec3) -> Vec3 {
    position.cross(momentum)
}

/// Calculate impulse from force and time
pub fn impulse(force: Vec3, time: f32) -> Vec3 {
    force * time
}

/// Calculate torque from force and lever arm
pub fn torque(force: Vec3, lever_arm: Vec3) -> Vec3 {
    lever_arm.cross(force)
}

/// Calculate the coefficient of restitution from velocities before and after collision
pub fn coefficient_of_restitution(
    velocity1_before: f32,
    velocity2_before: f32,
    velocity1_after: f32,
    velocity2_after: f32,
) -> f32 {
    let relative_velocity_before = velocity1_before - velocity2_before;
    let relative_velocity_after = velocity1_after - velocity2_after;
    
    if relative_velocity_before.abs() < f32::EPSILON {
        0.0
    } else {
        -relative_velocity_after / relative_velocity_before
    }
}

/// Calculate final velocities after 1D elastic collision
pub fn elastic_collision_1d(
    mass1: f32,
    mass2: f32,
    velocity1_initial: f32,
    velocity2_initial: f32,
) -> (f32, f32) {
    let total_mass = mass1 + mass2;
    let momentum_conservation = mass1 * velocity1_initial + mass2 * velocity2_initial;
    let relative_velocity = velocity1_initial - velocity2_initial;
    
    let velocity1_final = (momentum_conservation - mass2 * relative_velocity) / total_mass;
    let velocity2_final = (momentum_conservation + mass1 * relative_velocity) / total_mass;
    
    (velocity1_final, velocity2_final)
}

/// Calculate final velocities after 1D inelastic collision
pub fn inelastic_collision_1d(
    mass1: f32,
    mass2: f32,
    velocity1_initial: f32,
    velocity2_initial: f32,
    coefficient_of_restitution: f32,
) -> (f32, f32) {
    let total_mass = mass1 + mass2;
    let momentum_conservation = mass1 * velocity1_initial + mass2 * velocity2_initial;
    let relative_velocity = velocity1_initial - velocity2_initial;
    
    let velocity1_final = (momentum_conservation - mass2 * coefficient_of_restitution * relative_velocity) / total_mass;
    let velocity2_final = (momentum_conservation + mass1 * coefficient_of_restitution * relative_velocity) / total_mass;
    
    (velocity1_final, velocity2_final)
}

/// Convert linear velocity to angular velocity for rolling motion
pub fn linear_to_angular_velocity(linear_velocity: f32, radius: f32) -> f32 {
    if radius <= f32::EPSILON {
        0.0
    } else {
        linear_velocity / radius
    }
}

/// Convert angular velocity to linear velocity for rolling motion
pub fn angular_to_linear_velocity(angular_velocity: f32, radius: f32) -> f32 {
    angular_velocity * radius
}

/// Calculate the period of a simple pendulum
pub fn pendulum_period(length: f32, gravity: f32) -> f32 {
    if gravity <= f32::EPSILON {
        f32::INFINITY
    } else {
        2.0 * std::f32::consts::PI * (length / gravity).sqrt()
    }
}

/// Calculate the frequency of a mass-spring system
pub fn spring_frequency(mass: f32, spring_constant: f32) -> f32 {
    if mass <= f32::EPSILON {
        f32::INFINITY
    } else {
        (1.0 / (2.0 * std::f32::consts::PI)) * (spring_constant / mass).sqrt()
    }
}

/// Calculate projectile motion trajectory
pub fn projectile_trajectory(
    initial_velocity: Vec2,
    gravity: f32,
    time: f32,
) -> Vec2 {
    Vec2::new(
        initial_velocity.x * time,
        initial_velocity.y * time - 0.5 * gravity * time * time,
    )
}

/// Calculate projectile range for given launch angle and speed
pub fn projectile_range(initial_speed: f32, launch_angle: f32, gravity: f32) -> f32 {
    if gravity <= f32::EPSILON {
        f32::INFINITY
    } else {
        (initial_speed * initial_speed * (2.0 * launch_angle).sin()) / gravity
    }
}

/// Calculate the time of flight for a projectile
pub fn projectile_time_of_flight(initial_velocity_y: f32, gravity: f32) -> f32 {
    if gravity <= f32::EPSILON {
        f32::INFINITY
    } else {
        (2.0 * initial_velocity_y) / gravity
    }
}

/// Calculate work done by a force
pub fn work(force: Vec3, displacement: Vec3) -> f32 {
    force.dot(displacement)
}

/// Calculate power from work and time
pub fn power_from_work(work: f32, time: f32) -> f32 {
    if time <= f32::EPSILON {
        0.0
    } else {
        work / time
    }
}

/// Calculate power from force and velocity
pub fn power_from_force(force: Vec3, velocity: Vec3) -> f32 {
    force.dot(velocity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moment_of_inertia() {
        let mass = 10.0;
        let radius = 2.0;
        
        let sphere_moi = sphere_moment_of_inertia(mass, radius);
        assert!((sphere_moi - 16.0).abs() < f32::EPSILON); // (2/5) * 10 * 4 = 16
        
        let cylinder_moi = cylinder_moment_of_inertia(mass, radius);
        assert!((cylinder_moi - 20.0).abs() < f32::EPSILON); // 0.5 * 10 * 4 = 20
    }

    #[test]
    fn test_spring_force() {
        let displacement = 0.1;
        let spring_constant = 100.0;
        
        let force = spring_force(displacement, spring_constant);
        assert!((force - (-10.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_kinetic_energy() {
        let mass = 5.0;
        let velocity = 10.0;
        
        let ke = kinetic_energy(mass, velocity);
        assert!((ke - 250.0).abs() < f32::EPSILON); // 0.5 * 5 * 100 = 250
    }

    #[test]
    fn test_elastic_collision() {
        let mass1 = 1.0;
        let mass2 = 1.0;
        let v1_initial = 5.0;
        let v2_initial = 0.0;
        
        let (v1_final, v2_final) = elastic_collision_1d(mass1, mass2, v1_initial, v2_initial);
        
        // For equal masses, velocities should exchange
        assert!((v1_final - 0.0).abs() < f32::EPSILON);
        assert!((v2_final - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_projectile_motion() {
        let initial_velocity = Vec2::new(10.0, 10.0);
        let gravity = 9.8;
        let time = 1.0;
        
        let position = projectile_trajectory(initial_velocity, gravity, time);
        
        assert!((position.x - 10.0).abs() < f32::EPSILON);
        assert!((position.y - 5.1).abs() < 0.1); // 10 - 0.5 * 9.8 * 1 = 5.1
    }

    #[test]
    fn test_work_and_power() {
        let force = Vec3::new(10.0, 0.0, 0.0);
        let displacement = Vec3::new(5.0, 0.0, 0.0);
        let time = 2.0;
        
        let work_done = work(force, displacement);
        assert!((work_done - 50.0).abs() < f32::EPSILON);
        
        let power = power_from_work(work_done, time);
        assert!((power - 25.0).abs() < f32::EPSILON);
    }
}