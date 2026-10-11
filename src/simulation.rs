use crate::intersection::SmartIntersectionManager;
use crate::stats::StatsTracker;
use crate::types::{Origin, Turn, SAFETY_DISTANCE, VEHICLE_LENGTH, VehicleType, VelocityTier};
use crate::vehicle::Vehicle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleetPaintMode {
    SummitWhiteOnly,
    SparkPalette,
}

#[derive(Debug)]
pub struct Simulation {
    pub vehicles: Vec<Vehicle>,
    pub manager: SmartIntersectionManager,
    pub stats: StatsTracker,
    pub next_id: usize,
    pub auto_spawn: bool,
    pub auto_spawn_timer: f32,
    pub auto_spawn_interval: f32,
    pub is_finished: bool,
    pub color_counter: usize,
    pub fleet_paint_mode: FleetPaintMode,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            vehicles: Vec::new(),
            manager: SmartIntersectionManager::new(),
            stats: StatsTracker::new(),
            next_id: 1,
            auto_spawn: false,
            auto_spawn_timer: 0.0,
            auto_spawn_interval: 0.85,
            is_finished: false,
            color_counter: 0,
            fleet_paint_mode: FleetPaintMode::SummitWhiteOnly,
        }
    }

    /// Spawns a vehicle from the given origin if the spawn zone is clear (anti-overlap check)
    pub fn spawn_vehicle(&mut self, origin: Origin) -> bool {
        let candidate_turns = [Turn::Straight, Turn::Right, Turn::Left];
        
        // Pseudo-random selection based on counter
        let mut preferred_turns = candidate_turns;
        let shift = (self.next_id + self.color_counter) % 3;
        preferred_turns.rotate_left(shift);

        for turn in preferred_turns {
            // Anti-overlap check: verify no car is at the spawn location for this origin & turn
            let spawn_blocked = self.vehicles.iter().any(|v| {
                v.origin == origin
                    && v.turn == turn
                    && v.distance < (VEHICLE_LENGTH + SAFETY_DISTANCE + 15.0)
            });

            if !spawn_blocked {
                // Select speed tier
                let speed_tiers = VelocityTier::all_speeds();
                let initial_speed = speed_tiers[self.next_id % speed_tiers.len()];

                let color_idx = match self.fleet_paint_mode {
                    FleetPaintMode::SummitWhiteOnly => 0,
                    FleetPaintMode::SparkPalette => self.color_counter % 8,
                };

                let vehicle_type = match (self.next_id + self.color_counter) % 4 {
                    0 => VehicleType::Hatchback,
                    1 => VehicleType::Sedan,
                    2 => VehicleType::Suv,
                    _ => VehicleType::Van,
                };

                let vehicle = Vehicle::new_with_type(
                    self.next_id,
                    origin,
                    turn,
                    initial_speed,
                    color_idx,
                    vehicle_type,
                );

                self.next_id += 1;
                self.color_counter += 1;
                self.stats.record_spawn();
                self.vehicles.push(vehicle);
                return true;
            }
        }

        // All 3 lanes at this entrance are currently occupied at spawn point
        false
    }

    /// Toggles the 'R' auto random vehicle generation loop
    pub fn toggle_auto_spawn(&mut self) -> bool {
        self.auto_spawn = !self.auto_spawn;
        self.auto_spawn
    }

    /// Toggles Chevrolet Spark paint mode (Summit White vs Spark Factory Palette)
    pub fn toggle_paint_mode(&mut self) -> &'static str {
        match self.fleet_paint_mode {
            FleetPaintMode::SummitWhiteOnly => {
                self.fleet_paint_mode = FleetPaintMode::SparkPalette;
                for (idx, v) in self.vehicles.iter_mut().enumerate() {
                    v.color_idx = idx % 8;
                }
                "Chevrolet Spark Factory Palette"
            }
            FleetPaintMode::SparkPalette => {
                self.fleet_paint_mode = FleetPaintMode::SummitWhiteOnly;
                for v in &mut self.vehicles {
                    v.color_idx = 0;
                }
                "Summit White (Photo Match)"
            }
        }
    }

    /// Triggers when Escape key is pressed to finish simulation and display statistics
    pub fn finish_simulation(&mut self) {
        self.is_finished = true;
    }

    /// Restart simulation for another run
    pub fn restart(&mut self) {
        self.vehicles.clear();
        self.stats = StatsTracker::new();
        self.is_finished = false;
        self.auto_spawn_timer = 0.0;
        self.manager.close_call_pairs.clear();
    }

    /// Main physics and AI simulation step
    pub fn update(&mut self, dt: f32) {
        if self.is_finished {
            return;
        }

        self.stats.update_duration(dt);

        // Auto spawn timer logic
        if self.auto_spawn {
            self.auto_spawn_timer += dt;
            if self.auto_spawn_timer >= self.auto_spawn_interval {
                self.auto_spawn_timer = 0.0;
                let origins = [Origin::South, Origin::North, Origin::West, Origin::East];
                let chosen_origin = origins[(self.next_id * 3 + self.color_counter) % 4];
                self.spawn_vehicle(chosen_origin);
            }
        }

        // 1. Run smart intersection traffic management
        self.manager.manage_traffic(&mut self.vehicles, &mut self.stats, dt);

        // 2. Update each vehicle kinematics
        for v in self.vehicles.iter_mut() {
            v.update(dt);
        }

        // 3. Remove completed vehicles and record final statistics
        let mut i = 0;
        while i < self.vehicles.len() {
            if self.vehicles[i].is_finished() {
                let finished_v = self.vehicles.remove(i);
                self.stats.record_vehicle_passed(&finished_v);
            } else {
                i += 1;
            }
        }
    }
}
