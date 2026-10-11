use crate::vehicle::Vehicle;

#[derive(Debug, Clone)]
pub struct StatsTracker {
    /// Max number of vehicles that passed the intersection
    pub max_vehicles_passed: usize,

    /// Max velocity of all vehicles (Display the fastest speed achieved)
    pub max_velocity: f32,

    /// Min velocity of all vehicles (Display the slowest speed reached)
    pub min_velocity: f32,

    /// Max time that the vehicles took to pass the intersection
    pub max_time: f32,

    /// Min time that the vehicles took to pass the intersection
    pub min_time: f32,

    /// Close calls (when vehicles pass each other violating safety distance)
    pub close_calls: usize,

    // Additional bonus statistics
    pub total_spawned: usize,
    pub total_transit_time: f64,
    pub total_transit_velocity_sum: f64,
    pub simulation_duration: f32,
    pub collisions_count: usize,
}

impl StatsTracker {
    pub fn new() -> Self {
        Self {
            max_vehicles_passed: 0,
            max_velocity: 0.0,
            min_velocity: f32::MAX,
            max_time: 0.0,
            min_time: f32::MAX,
            close_calls: 0,
            total_spawned: 0,
            total_transit_time: 0.0,
            total_transit_velocity_sum: 0.0,
            simulation_duration: 0.0,
            collisions_count: 0,
        }
    }

    pub fn record_spawn(&mut self) {
        self.total_spawned += 1;
    }

    pub fn record_close_call(&mut self) {
        self.close_calls += 1;
    }

    pub fn record_collision(&mut self) {
        self.collisions_count += 1;
    }

    pub fn record_vehicle_passed(&mut self, vehicle: &Vehicle) {
        self.max_vehicles_passed += 1;

        // Max velocity achieved
        if vehicle.max_velocity_reached > self.max_velocity {
            self.max_velocity = vehicle.max_velocity_reached;
        }

        // Min velocity reached (non-zero)
        if vehicle.min_velocity_reached > 0.0 && vehicle.min_velocity_reached < self.min_velocity {
            self.min_velocity = vehicle.min_velocity_reached;
        }

        // Max time to pass intersection
        if vehicle.transit_time > self.max_time {
            self.max_time = vehicle.transit_time;
        }

        // Min time to pass intersection
        if vehicle.transit_time > 0.0 && vehicle.transit_time < self.min_time {
            self.min_time = vehicle.transit_time;
        }

        self.total_transit_time += vehicle.transit_time as f64;
        self.total_transit_velocity_sum += vehicle.velocity as f64;
    }

    pub fn update_duration(&mut self, dt: f32) {
        self.simulation_duration += dt;
    }

    pub fn display_min_velocity(&self) -> f32 {
        if self.min_velocity == f32::MAX {
            0.0
        } else {
            self.min_velocity
        }
    }

    pub fn display_min_time(&self) -> f32 {
        if self.min_time == f32::MAX {
            0.0
        } else {
            self.min_time
        }
    }

    pub fn avg_time(&self) -> f32 {
        if self.max_vehicles_passed > 0 {
            (self.total_transit_time / self.max_vehicles_passed as f64) as f32
        } else {
            0.0
        }
    }

    pub fn avg_velocity(&self) -> f32 {
        if self.max_vehicles_passed > 0 {
            (self.total_transit_velocity_sum / self.max_vehicles_passed as f64) as f32
        } else {
            0.0
        }
    }
}
