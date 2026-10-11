pub mod intersection;
pub mod physics;
pub mod renderer;
pub mod route;
pub mod simulation;
pub mod stats;
pub mod types;
pub mod vehicle;

use std::time::{Duration, Instant};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::renderer::Renderer;
use crate::simulation::Simulation;
use crate::types::{Origin, CANVAS_HEIGHT, CANVAS_WIDTH};

fn main() -> Result<(), String> {
    println!("=====================================================");
    println!("      SMART-ROAD: AUTONOMOUS INTERSECTION SIMULATION ");
    println!("         Fleet Vehicle: Chevrolet Spark (Hatchback)");
    println!("=====================================================");
    println!("Controls:");
    println!("  [Up Arrow]    : Spawn vehicle from South (moving North)");
    println!("  [Down Arrow]  : Spawn vehicle from North (moving South)");
    println!("  [Left Arrow]  : Spawn vehicle from East  (moving West)");
    println!("  [Right Arrow] : Spawn vehicle from West  (moving East)");
    println!("  [R]           : Toggle continuous random auto-spawning");
    println!("  [C]           : Toggle Chevrolet Spark Paint Mode");
    println!("  [Escape]      : Finish simulation & display Statistics Window");
    println!("=====================================================");

    // Initialize SDL2 subsystem
    let sdl_context = match sdl2::init() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Note: SDL2 display initialization failed: {}", e);
            eprintln!("You can run the full verification test suite via `cargo test`.");
            return Ok(());
        }
    };

    let video_subsystem = match sdl_context.video() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Note: Video subsystem initialization failed: {}", e);
            eprintln!("You can run the full verification test suite via `cargo test`.");
            return Ok(());
        }
    };

    let window = match video_subsystem
        .window("Smart Road - Autonomous Intersection (Chevrolet Spark)", CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32)
        .position_centered()
        .build()
    {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Note: Window creation failed: {}", e);
            return Ok(());
        }
    };

    let mut canvas = match window.into_canvas().accelerated().present_vsync().build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Note: Canvas creation failed: {}", e);
            return Ok(());
        }
    };

    let mut event_pump = match sdl_context.event_pump() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Note: Event pump creation failed: {}", e);
            return Ok(());
        }
    };

    let mut sim = Simulation::new();
    let target_frame_duration = Duration::from_nanos(1_000_000_000 / 60); // 60 FPS
    let mut last_tick = Instant::now();

    'running: loop {
        // Event Handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Up => {
                        sim.spawn_vehicle(Origin::South);
                    }
                    Keycode::Down => {
                        sim.spawn_vehicle(Origin::North);
                    }
                    Keycode::Left => {
                        sim.spawn_vehicle(Origin::East);
                    }
                    Keycode::Right => {
                        sim.spawn_vehicle(Origin::West);
                    }
                    Keycode::R => {
                        let active = sim.toggle_auto_spawn();
                        println!("Auto-spawn continuous mode: {}", if active { "ENABLED" } else { "DISABLED" });
                    }
                    Keycode::C => {
                        let mode = sim.toggle_paint_mode();
                        println!("Chevrolet Spark Paint Mode: {}", mode);
                    }
                    Keycode::Escape => {
                        if !sim.is_finished {
                            println!("Simulation finished by user. Generating Statistics Window...");
                            sim.finish_simulation();
                        } else {
                            // Already on statistics screen, second Esc quits
                            break 'running;
                        }
                    }
                    Keycode::Return | Keycode::Return2 => {
                        if sim.is_finished {
                            println!("Restarting simulation...");
                            sim.restart();
                        }
                    }
                    Keycode::Q => {
                        if sim.is_finished {
                            break 'running;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Fixed / clamped delta time for stable physics
        let now = Instant::now();
        let elapsed = now.duration_since(last_tick);
        last_tick = now;
        let dt = (elapsed.as_secs_f32()).min(0.05);

        // Update physics and smart intersection manager
        sim.update(dt);

        // Render current state (simulation or statistics screen)
        if let Err(e) = Renderer::render(&mut canvas, &sim) {
            eprintln!("Render error: {}", e);
            break 'running;
        }

        // Frame rate limiter
        let frame_time = Instant::now().duration_since(now);
        if frame_time < target_frame_duration {
            std::thread::sleep(target_frame_duration - frame_time);
        }
    }

    println!("\nSimulation ended.");
    println!("Final Summary:");
    println!("  Vehicle Make:        Chevrolet");
    println!("  Vehicle Model:       Spark (Subcompact Hatchback)");
    println!("  Max Vehicles Passed: {}", sim.stats.max_vehicles_passed);
    println!("  Max Velocity:        {:.1} px/s", sim.stats.max_velocity);
    println!("  Min Velocity:        {:.1} px/s", sim.stats.display_min_velocity());
    println!("  Max Transit Time:    {:.2} s", sim.stats.max_time);
    println!("  Min Transit Time:    {:.2} s", sim.stats.display_min_time());
    println!("  Close Calls:         {}", sim.stats.close_calls);
    println!("=====================================================");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::{PhysicsEngine, SafetyStatus};
    use crate::route::Route;
    use crate::stats::StatsTracker;
    use crate::types::{Origin, Point2D, Turn, SAFETY_DISTANCE, VEHICLE_LENGTH, VelocityTier};
    use crate::vehicle::Vehicle;

    /// 1. Verify the Physics Engine by ensuring the velocity = distance / time calculations
    /// are accurate for various vehicle speeds.
    #[test]
    fn test_physics_engine_velocity_distance_time() {
        // Standard speeds: 55.0, 105.0, 165.0
        let test_speeds = VelocityTier::all_speeds();
        for &speed in &test_speeds {
            let time = 3.5;
            let distance = PhysicsEngine::distance(speed, time);
            assert!((distance - (speed * time)).abs() < 0.001);

            let computed_v = PhysicsEngine::velocity(distance, time);
            assert!(
                (computed_v - speed).abs() < 0.001,
                "Expected speed {}, got {}",
                speed,
                computed_v
            );

            let computed_t = PhysicsEngine::time(distance, speed);
            assert!(
                (computed_t - time).abs() < 0.001,
                "Expected time {}, got {}",
                time,
                computed_t
            );
        }

        // Zero time check (avoid division by zero)
        assert_eq!(PhysicsEngine::velocity(100.0, 0.0), 0.0);
        assert!(PhysicsEngine::time(100.0, 0.0).is_infinite());
    }

    /// Bonus: Verify smooth acceleration and deceleration kinematics.
    #[test]
    fn test_physics_engine_acceleration_and_deceleration() {
        let v0 = 50.0;
        let target_v = 150.0;
        let accel = 100.0;
        let decel = 200.0;
        let dt = 0.5;

        // Accelerate
        let v1 = PhysicsEngine::step_velocity(v0, target_v, accel, decel, dt);
        assert_eq!(v1, 50.0 + 100.0 * 0.5); // 100.0

        // Clamping at target
        let v2 = PhysicsEngine::step_velocity(v1, target_v, accel, decel, 1.0);
        assert_eq!(v2, 150.0);

        // Decelerate
        let v3 = PhysicsEngine::step_velocity(150.0, 50.0, accel, decel, dt);
        assert_eq!(v3, 150.0 - 200.0 * 0.5); // 50.0
    }

    /// 2. Verify Safety Distance Detection by testing that a vehicle correctly identifies
    /// a "close call" or a "stop" condition when the gap between it and another AV is equal
    /// to or less than your defined safety distance.
    #[test]
    fn test_safety_distance_detection() {
        let p1 = Point2D::new(100.0, 100.0);

        // Within safety distance (close call)
        let p_close = Point2D::new(100.0, 100.0 + SAFETY_DISTANCE - 5.0);
        let status_close = PhysicsEngine::evaluate_safety_distance(p1, p_close, SAFETY_DISTANCE);
        assert_eq!(status_close, SafetyStatus::CloseCall);

        // Exactly at safety distance (close call)
        let p_exact = Point2D::new(100.0, 100.0 + SAFETY_DISTANCE);
        let status_exact = PhysicsEngine::evaluate_safety_distance(p1, p_exact, SAFETY_DISTANCE);
        assert_eq!(status_exact, SafetyStatus::CloseCall);

        // Violation / near collision
        let p_violation = Point2D::new(100.0, 100.0 + SAFETY_DISTANCE * 0.2);
        let status_viol = PhysicsEngine::evaluate_safety_distance(p1, p_violation, SAFETY_DISTANCE);
        assert_eq!(status_viol, SafetyStatus::Violation);

        // Completely safe distance
        let p_safe = Point2D::new(100.0, 100.0 + SAFETY_DISTANCE + 25.0);
        let status_safe = PhysicsEngine::evaluate_safety_distance(p1, p_safe, SAFETY_DISTANCE);
        assert_eq!(status_safe, SafetyStatus::Safe);
    }

    /// Same-lane leader-follower car-following safety test:
    /// Follower stops or reduces speed when gap <= SAFETY_DISTANCE.
    #[test]
    fn test_car_following_safety_stop() {
        let mut sim = Simulation::new();
        // Spawn leader on South straight
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Straight, 100.0, 0);
        v1.distance = 200.0;

        // Spawn follower closely behind on same route
        let mut v2 = Vehicle::new(2, Origin::South, Turn::Straight, 100.0, 1);
        v2.distance = 200.0 - (VEHICLE_LENGTH + SAFETY_DISTANCE - 10.0);

        sim.vehicles.push(v1);
        sim.vehicles.push(v2);

        // Run smart intersection manager
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);

        // Follower must be commanded to stop (target_velocity = 0.0)
        assert_eq!(
            sim.vehicles[1].target_velocity, 0.0,
            "Follower within safety distance must stop"
        );
    }

    /// 3. Test the Smart Intersection Algorithm by simulating conflicting routes
    /// (e.g., a left turn crossing a straight path) and verifying that at least
    /// one vehicle adjusts its velocity to prevent a collision.
    #[test]
    fn test_smart_intersection_algorithm_conflicting_routes() {
        let mut sim = Simulation::new();

        // Vehicle 1: Coming from South going North (straight route)
        // Set position right at detection zone approaching stop line
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Straight, 120.0, 0);
        // Crossing conflict point between South Straight (x=504) and East Straight (y=396)
        // Vehicle 1 arrives at (504, 396) at dist = stop_line + 162.0
        let stop_dist_1 = v1.route.stop_line_distance;
        let cross_dist_1 = stop_dist_1 + 162.0;
        v1.distance = cross_dist_1 - 180.0; // 180px before crossing point

        // Vehicle 2: Coming from East going West (crossing straight path)
        // Arrives at (504, 396) at dist = stop_line + 54.0
        let mut v2 = Vehicle::new(2, Origin::East, Turn::Straight, 120.0, 1);
        let stop_dist_2 = v2.route.stop_line_distance;
        let cross_dist_2 = stop_dist_2 + 54.0;
        v2.distance = cross_dist_2 - 190.0; // 190px before crossing point (synchronized collision course)

        sim.vehicles.push(v1);
        sim.vehicles.push(v2);

        // Run traffic management
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);

        let v1_target = sim.vehicles[0].target_velocity;
        let v2_target = sim.vehicles[1].target_velocity;

        // At least one vehicle MUST adjust its velocity to prevent collision
        let speed_adjusted = v1_target < 120.0 || v2_target < 120.0;
        assert!(
            speed_adjusted,
            "Smart intersection must adjust velocity of at least one conflicting vehicle (v1: {}, v2: {})",
            v1_target, v2_target
        );

        // Vehicle 1 (closer) keeps high speed, Vehicle 2 (further) yields
        assert!(
            v2_target < v1_target,
            "Yielding vehicle target speed ({}) must be less than priority vehicle ({})",
            v2_target, v1_target
        );
    }

    /// Conflicting Left Turn crossing opposing Straight path
    #[test]
    fn test_conflicting_left_turn_and_straight() {
        let mut sim = Simulation::new();

        // Vehicle 1: North turning Left (turns East)
        let mut v1 = Vehicle::new(1, Origin::North, Turn::Left, 110.0, 0);
        v1.distance = v1.route.stop_line_distance - 25.0;

        // Vehicle 2: South going Straight (North)
        let mut v2 = Vehicle::new(2, Origin::South, Turn::Straight, 110.0, 1);
        v2.distance = v2.route.stop_line_distance - 30.0;

        sim.vehicles.push(v1);
        sim.vehicles.push(v2);

        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);

        let v1_target = sim.vehicles[0].target_velocity;
        let v2_target = sim.vehicles[1].target_velocity;

        assert!(
            v1_target != v2_target || v1_target < 110.0,
            "One vehicle must yield or decelerate at crossing left turn conflict"
        );
    }

    /// 4. Verify Statistics Accumulation by ensuring that variables like max/min velocity
    /// and vehicle count are correctly updated in the internal state after each simulated pass.
    #[test]
    fn test_statistics_accumulation() {
        let mut tracker = StatsTracker::new();

        assert_eq!(tracker.max_vehicles_passed, 0);
        assert_eq!(tracker.max_velocity, 0.0);
        assert_eq!(tracker.display_min_velocity(), 0.0);
        assert_eq!(tracker.max_time, 0.0);
        assert_eq!(tracker.display_min_time(), 0.0);
        assert_eq!(tracker.close_calls, 0);

        // Simulate Vehicle 1 passing
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Straight, 120.0, 0);
        v1.max_velocity_reached = 150.0;
        v1.min_velocity_reached = 60.0;
        v1.transit_time = 4.2;
        tracker.record_spawn();
        tracker.record_vehicle_passed(&v1);

        assert_eq!(tracker.max_vehicles_passed, 1);
        assert_eq!(tracker.max_velocity, 150.0);
        assert_eq!(tracker.min_velocity, 60.0);
        assert_eq!(tracker.max_time, 4.2);
        assert_eq!(tracker.min_time, 4.2);

        // Simulate Vehicle 2 passing with different stats
        let mut v2 = Vehicle::new(2, Origin::North, Turn::Right, 90.0, 1);
        v2.max_velocity_reached = 180.0; // Higher max
        v2.min_velocity_reached = 40.0;  // Lower min
        v2.transit_time = 2.8;          // Faster transit
        tracker.record_spawn();
        tracker.record_vehicle_passed(&v2);

        assert_eq!(tracker.max_vehicles_passed, 2);
        assert_eq!(tracker.max_velocity, 180.0);
        assert_eq!(tracker.min_velocity, 40.0);
        assert_eq!(tracker.max_time, 4.2);
        assert_eq!(tracker.min_time, 2.8);

        // Record a close call
        tracker.record_close_call();
        assert_eq!(tracker.close_calls, 1);
        tracker.record_close_call();
        assert_eq!(tracker.close_calls, 2);
    }

    /// 5. Test Anti-Overlap Spawning (spamming the key should not spawn vehicles on top of each other)
    #[test]
    fn test_anti_overlap_spawn_protection() {
        let mut sim = Simulation::new();

        // First 3 spawns on South should succeed (occupying all 3 lanes: s, r, l)
        let s1 = sim.spawn_vehicle(Origin::South);
        let s2 = sim.spawn_vehicle(Origin::South);
        let s3 = sim.spawn_vehicle(Origin::South);
        assert!(s1);
        assert!(s2);
        assert!(s3);

        // A 4th immediate spam on South without moving must be rejected
        // because all 3 entrance lanes have cars at distance 0
        let s4 = sim.spawn_vehicle(Origin::South);
        assert!(!s4, "Spamming key must NOT spawn vehicle on top of existing vehicle");

        // Advance simulation so cars move forward past spawn threshold
        for _ in 0..100 {
            sim.update(0.016);
        }

        // Now spawn zone is clear, a new spawn must succeed
        let s5 = sim.spawn_vehicle(Origin::South);
        assert!(s5, "Spawn must succeed once spawn zone has cleared");
    }

    /// 6. Test Route Waypoint Heading & Turning Animation
    #[test]
    fn test_route_turning_headings() {
        // South Right Turn (starts heading North 270 deg, finishes heading East 0 deg)
        let route_r = Route::new(Origin::South, Turn::Right);
        let start_heading = route_r.heading_at(5.0);
        let end_heading = route_r.heading_at(route_r.total_distance - 5.0);

        // Tangent angles: 270 (-90) -> 0 deg (or 360)
        assert!((start_heading - (-90.0)).abs() < 5.0 || (start_heading - 270.0).abs() < 5.0);
        assert!(end_heading.abs() < 5.0 || (end_heading - 360.0).abs() < 5.0);

        // South Straight (constant heading North)
        let route_s = Route::new(Origin::South, Turn::Straight);
        let s_heading_1 = route_s.heading_at(50.0);
        let s_heading_2 = route_s.heading_at(route_s.total_distance - 50.0);
        assert!((s_heading_1 - s_heading_2).abs() < 1.0);
    }

    /// 7. Test Parallel Non-Conflicting Routes Flow Simultaneously:
    /// North->South and South->North straight routes must proceed concurrently at Fast speed.
    #[test]
    fn test_parallel_non_conflicting_routes_flow_simultaneously() {
        let mut sim = Simulation::new();

        // Vehicle 1: Coming from South going North (straight route)
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Straight, 120.0, 0);
        v1.distance = v1.route.stop_line_distance - 40.0;

        // Vehicle 2: Coming from North going South (parallel opposite straight route)
        let mut v2 = Vehicle::new(2, Origin::North, Turn::Straight, 120.0, 1);
        v2.distance = v2.route.stop_line_distance - 40.0;

        sim.vehicles.push(v1);
        sim.vehicles.push(v2);

        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);

        let v1_target = sim.vehicles[0].target_velocity;
        let v2_target = sim.vehicles[1].target_velocity;
        let fast_speed = VelocityTier::Fast.speed();

        assert_eq!(
            v1_target, fast_speed,
            "South straight must proceed at Fast speed without waiting"
        );
        assert_eq!(
            v2_target, fast_speed,
            "North straight must proceed at Fast speed without waiting"
        );
    }

    /// 8. Test Right Turn Free Flow:
    /// Right turns (Turn::Right) must proceed without waiting for left turns or cross traffic.
    #[test]
    fn test_right_turn_free_flow_unhindered() {
        let mut sim = Simulation::new();

        // Vehicle 1: South turning Right
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Right, 100.0, 0);
        v1.distance = v1.route.stop_line_distance - 20.0;

        // Vehicle 2: East going Straight
        let mut v2 = Vehicle::new(2, Origin::East, Turn::Straight, 100.0, 1);
        v2.distance = v2.route.stop_line_distance - 20.0;

        sim.vehicles.push(v1);
        sim.vehicles.push(v2);

        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);

        let v1_target = sim.vehicles[0].target_velocity;
        let fast_speed = VelocityTier::Fast.speed();

        assert_eq!(
            v1_target, fast_speed,
            "Right-turn vehicle must proceed at Fast speed and never get blocked by cross traffic"
        );
    }

    /// 9. Test Time-to-Conflict Separation:
    /// If two vehicles have crossing paths in space but will arrive at different times (> 1.2s apart),
    /// neither vehicle should be stopped.
    #[test]
    fn test_time_to_conflict_separation_no_false_stops() {
        let mut sim = Simulation::new();

        // Vehicle 1: Arrives at crossing point in ~0.3s
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Straight, 165.0, 0);
        let cross_1 = v1.route.stop_line_distance + 162.0;
        v1.distance = cross_1 - 50.0; // 50px away at 165 px/s => ~0.3s

        // Vehicle 2: Arrives at crossing point in ~2.2s
        let mut v2 = Vehicle::new(2, Origin::East, Turn::Straight, 105.0, 1);
        let cross_2 = v2.route.stop_line_distance + 54.0;
        v2.distance = cross_2 - 230.0; // 230px away at 105 px/s => ~2.2s

        sim.vehicles.push(v1);
        sim.vehicles.push(v2);

        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);

        let v1_target = sim.vehicles[0].target_velocity;
        let v2_target = sim.vehicles[1].target_velocity;

        // Vehicle 1 has priority and keeps fast speed
        assert_eq!(v1_target, VelocityTier::Fast.speed());
        // Vehicle 2 is far enough away in time that it should NOT be brought to a halt (target_velocity > 0)
        assert!(
            v2_target > 0.0,
            "Vehicle 2 must not be brought to a full stop when arrival times are separated (v2: {})",
            v2_target
        );
    }

    /// 10. Test Deadlock Breaker: 4 Vehicles arrive simultaneously at the gate from all 4 sides.
    /// The algorithm must strictly grant the right of way to the lowest ID vehicle (FIFO),
    /// completely preventing circular wait deadlock.
    #[test]
    fn test_four_way_simultaneous_gate_deadlock_breaker() {
        let mut sim = Simulation::new();

        // 4 vehicles all at the stop line (is_at_gate = true)
        let mut v_north = Vehicle::new(10, Origin::North, Turn::Straight, 0.0, 0);
        v_north.distance = v_north.route.stop_line_distance - 5.0;

        let mut v_east = Vehicle::new(11, Origin::East, Turn::Straight, 0.0, 1);
        v_east.distance = v_east.route.stop_line_distance - 5.0;

        let mut v_south = Vehicle::new(12, Origin::South, Turn::Straight, 0.0, 2);
        v_south.distance = v_south.route.stop_line_distance - 5.0;

        let mut v_west = Vehicle::new(13, Origin::West, Turn::Straight, 0.0, 3);
        v_west.distance = v_west.route.stop_line_distance - 5.0;

        sim.vehicles.push(v_north);
        sim.vehicles.push(v_east);
        sim.vehicles.push(v_south);
        sim.vehicles.push(v_west);

        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);

        // Vehicle with the lowest ID (North, id 10) MUST receive Fast speed to break the circular wait!
        let north_speed = sim.vehicles[0].target_velocity;
        assert_eq!(
            north_speed,
            VelocityTier::Fast.speed(),
            "Deadlock Breaker must grant green light to lowest ID vehicle (North: {})",
            north_speed
        );
    }

    /// 11. Test Opposite Left Turns Diverge Starboard-to-Starboard:
    /// North turning Left and South turning Left must proceed simultaneously without blocking each other.
    #[test]
    fn test_opposite_left_turns_diverge_without_blocking() {
        let mut sim = Simulation::new();

        let mut v1 = Vehicle::new(1, Origin::North, Turn::Left, 100.0, 0);
        v1.distance = v1.route.stop_line_distance - 25.0;

        let mut v2 = Vehicle::new(2, Origin::South, Turn::Left, 100.0, 1);
        v2.distance = v2.route.stop_line_distance - 25.0;

        sim.vehicles.push(v1);
        sim.vehicles.push(v2);

        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);

        let v1_speed = sim.vehicles[0].target_velocity;
        let v2_speed = sim.vehicles[1].target_velocity;

        assert_eq!(v1_speed, VelocityTier::Fast.speed());
        assert_eq!(v2_speed, VelocityTier::Fast.speed());
    }

    /// 12. Test 8-way simultaneous Straight and Left turns:
    /// All 4 approaches have both a Straight and a Left turn vehicle stopped at the stop line.
    /// The smart intersection algorithm must continuously resolve conflicts and clear all vehicles
    /// without any circular wait deadlock!
    #[test]
    fn test_eight_way_straight_and_left_turns_no_deadlock() {
        let mut sim = Simulation::new();

        let origins = [Origin::North, Origin::East, Origin::South, Origin::West];
        let mut id = 1;
        for &origin in &origins {
            // Straight vehicle
            let mut v_s = Vehicle::new(id, origin, Turn::Straight, 0.0, 0);
            v_s.distance = v_s.route.stop_line_distance - 8.0;
            sim.vehicles.push(v_s);
            id += 1;

            // Left turn vehicle
            let mut v_l = Vehicle::new(id, origin, Turn::Left, 0.0, 1);
            v_l.distance = v_l.route.stop_line_distance - 8.0;
            sim.vehicles.push(v_l);
            id += 1;
        }

        assert_eq!(sim.vehicles.len(), 8);

        // Run simulation for 12 seconds
        for _ in 0..750 {
            sim.update(0.016);
        }

        // All 8 vehicles must successfully move through, clear, and complete their journey without deadlock!
        assert_eq!(
            sim.stats.max_vehicles_passed, 8,
            "All 8 vehicles (4 Straight + 4 Left) must completely clear the intersection without any deadlock!"
        );
    }
}
