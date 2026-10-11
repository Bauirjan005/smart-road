use crate::types::Point2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyStatus {
    Safe,
    CloseCall,
    Violation,
}

pub struct PhysicsEngine;

impl PhysicsEngine {
    /// Velocity = distance / time
    pub fn velocity(distance: f32, time: f32) -> f32 {
        if time.abs() > 0.00001 {
            distance / time
        } else {
            0.0
        }
    }

    /// Distance = velocity * time
    pub fn distance(velocity: f32, time: f32) -> f32 {
        velocity * time
    }

    /// Time = distance / velocity
    pub fn time(distance: f32, velocity: f32) -> f32 {
        if velocity.abs() > 0.00001 {
            distance / velocity
        } else {
            f32::INFINITY
        }
    }

    /// Smooth velocity update taking acceleration and deceleration into account
    pub fn step_velocity(
        current_v: f32,
        target_v: f32,
        accel: f32,
        decel: f32,
        dt: f32,
    ) -> f32 {
        if current_v < target_v {
            (current_v + accel * dt).min(target_v)
        } else if current_v > target_v {
            (current_v - decel * dt).max(target_v)
        } else {
            current_v
        }
    }

    /// Theoretical stopping distance under constant deceleration: d = v^2 / (2 * a)
    pub fn stopping_distance(speed: f32, decel: f32) -> f32 {
        if decel > 0.0 {
            (speed * speed) / (2.0 * decel)
        } else {
            f32::INFINITY
        }
    }

    /// Evaluates proximity between two points with respect to the safety distance
    pub fn evaluate_safety_distance(
        p1: Point2D,
        p2: Point2D,
        safety_distance: f32,
    ) -> SafetyStatus {
        let dist = p1.distance_to(&p2);
        if dist < safety_distance * 0.4 {
            SafetyStatus::Violation
        } else if dist <= safety_distance {
            SafetyStatus::CloseCall
        } else {
            SafetyStatus::Safe
        }
    }
}
