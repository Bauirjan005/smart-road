use crate::route::Route;
use crate::types::{
    Origin, Point2D, Turn, CAR_MAKE, DEFAULT_ACCEL, DEFAULT_DECEL,
    VehicleType,
};

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: usize,
    pub origin: Origin,
    pub turn: Turn,
    pub route: Route,

    // Core physics state
    pub distance: f32,
    pub velocity: f32,
    pub target_velocity: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub is_braking: bool,

    // Intersection transit state
    pub is_detected: bool,
    pub transit_time: f32,
    pub transit_distance: f32,
    pub has_cleared: bool,

    // Individual records
    pub max_velocity_reached: f32,
    pub min_velocity_reached: f32,

    // Make & Model & Type specification
    pub make: &'static str,
    pub model: &'static str,
    pub vehicle_type: VehicleType,

    // Dimensions & visuals
    pub length: f32,
    pub width: f32,
    pub color_idx: usize,
    pub current_heading: f32,
}

impl Vehicle {
    pub fn new(id: usize, origin: Origin, turn: Turn, initial_velocity: f32, color_idx: usize) -> Self {
        let vehicle_type = VehicleType::from_id(id);
        Self::new_with_type(id, origin, turn, initial_velocity, color_idx, vehicle_type)
    }

    pub fn new_with_type(
        id: usize,
        origin: Origin,
        turn: Turn,
        initial_velocity: f32,
        color_idx: usize,
        vehicle_type: VehicleType,
    ) -> Self {
        let (width, length) = vehicle_type.dimensions();
        let route = Route::new(origin, turn);
        let current_heading = route.heading_at(0.0);
        Self {
            id,
            origin,
            turn,
            route,
            distance: 0.0,
            velocity: initial_velocity,
            target_velocity: initial_velocity,
            acceleration: DEFAULT_ACCEL,
            deceleration: DEFAULT_DECEL,
            is_braking: false,
            is_detected: false,
            transit_time: 0.0,
            transit_distance: 0.0,
            has_cleared: false,
            max_velocity_reached: initial_velocity,
            min_velocity_reached: initial_velocity,
            make: CAR_MAKE,
            model: vehicle_type.as_str(),
            vehicle_type,
            length,
            width,
            color_idx,
            current_heading,
        }
    }

    pub fn position(&self) -> Point2D {
        self.route.position_at(self.distance)
    }

    pub fn heading(&self) -> f32 {
        self.current_heading
    }

    pub fn angle(&self) -> f32 {
        self.current_heading
    }

    pub fn is_braking(&self) -> bool {
        self.is_braking
            || (self.velocity > self.target_velocity + 0.1)
            || (self.target_velocity == 0.0 && self.velocity < 1.0)
    }

    pub fn is_finished(&self) -> bool {
        self.distance >= self.route.total_distance
    }

    /// Update physics using acceleration, deceleration, and velocity = distance / time
    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 || self.is_finished() {
            return;
        }

        // Apply smooth acceleration or deceleration towards target velocity
        if self.velocity < self.target_velocity {
            self.velocity = (self.velocity + self.acceleration * dt).min(self.target_velocity);
            self.is_braking = false;
        } else if self.velocity > self.target_velocity {
            self.velocity = (self.velocity - self.deceleration * dt).max(self.target_velocity);
            self.is_braking = true;
        } else {
            self.is_braking = self.target_velocity == 0.0 && self.velocity < 1.0;
        }

        // Velocity = distance / time  =>  delta_distance = velocity * delta_time
        let delta_d = self.velocity * dt;
        self.distance += delta_d;

        // Smooth heading update ignoring micro-deltas < 0.1px
        self.current_heading = self.route.heading_at_smooth(self.distance, self.current_heading);

        // Check detection by smart intersection algorithm
        if !self.is_detected && self.route.is_detected(self.distance) {
            self.is_detected = true;
        }

        // Accumulate transit metrics if detected
        if self.is_detected {
            self.transit_time += dt;
            self.transit_distance += delta_d;
        }

        // Track max and min velocities reached
        if self.velocity > self.max_velocity_reached {
            self.max_velocity_reached = self.velocity;
        }
        if self.velocity > 0.1 && self.velocity < self.min_velocity_reached {
            self.min_velocity_reached = self.velocity;
        }

        if self.route.is_in_intersection(self.distance) {
            self.has_cleared = false;
        } else if self.distance > self.route.exit_distance {
            self.has_cleared = true;
        }
    }

    /// Front bumper position
    pub fn front_bumper(&self) -> Point2D {
        let front_dist = (self.distance + self.length / 2.0).min(self.route.total_distance);
        self.route.position_at(front_dist)
    }

    /// Rear bumper position
    pub fn rear_bumper(&self) -> Point2D {
        let rear_dist = (self.distance - self.length / 2.0).max(0.0);
        self.route.position_at(rear_dist)
    }
}
