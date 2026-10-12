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
    println!("=====================================================\n      SMART-ROAD: AUTONOMOUS INTERSECTION SIMULATION \n         Fleet Vehicle: Chevrolet Spark (Hatchback)\n=====================================================\nControls:\n  [Up Arrow]    : Spawn vehicle from South (moving North)\n  [Down Arrow]  : Spawn vehicle from North (moving South)\n  [Left Arrow]  : Spawn vehicle from East  (moving West)\n  [Right Arrow] : Spawn vehicle from West  (moving East)\n  [R]           : Toggle continuous random auto-spawning\n  [C]           : Toggle Chevrolet Spark Paint Mode\n  [Escape]      : Finish simulation & display Statistics Window\n=====================================================");

    let Ok(sdl) = sdl2::init() else { eprintln!("SDL2 initialization failed. Run tests via `cargo test`."); return Ok(()); };
    let Ok(video) = sdl.video() else { return Ok(()); };
    let Ok(window) = video.window("Smart Road - Autonomous Intersection (Chevrolet Spark)", CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32).position_centered().build() else { return Ok(()); };
    let Ok(mut canvas) = window.into_canvas().accelerated().present_vsync().build() else { return Ok(()); };
    let Ok(mut event_pump) = sdl.event_pump() else { return Ok(()); };

    let mut sim = Simulation::new();
    let target_frame = Duration::from_nanos(1_000_000_000 / 60);
    let mut last_tick = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(key), .. } => match key {
                    Keycode::Up => { sim.spawn_vehicle(Origin::South); }
                    Keycode::Down => { sim.spawn_vehicle(Origin::North); }
                    Keycode::Left => { sim.spawn_vehicle(Origin::East); }
                    Keycode::Right => { sim.spawn_vehicle(Origin::West); }
                    Keycode::R => {
                        let on = sim.toggle_auto_spawn();
                        println!("Auto-spawn continuous mode: {}", if on { "ENABLED" } else { "DISABLED" });
                    }
                    Keycode::C => println!("Chevrolet Spark Paint Mode: {}", sim.toggle_paint_mode()),
                    Keycode::Escape => if !sim.is_finished {
                        println!("Simulation finished by user. Generating Statistics Window...");
                        sim.finish_simulation();
                    } else {
                        break 'running;
                    },
                    Keycode::Return | Keycode::Return2 if sim.is_finished => {
                        println!("Restarting simulation...");
                        sim.restart();
                    }
                    Keycode::Q if sim.is_finished => break 'running,
                    _ => {}
                },
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f32().min(0.05);
        last_tick = now;

        sim.update(dt);
        if let Err(e) = Renderer::render(&mut canvas, &sim) {
            eprintln!("Render error: {}", e);
            break 'running;
        }

        let frame_time = Instant::now().duration_since(now);
        if frame_time < target_frame {
            std::thread::sleep(target_frame - frame_time);
        }
    }

    println!("\nSimulation ended.\nFinal Summary:\n  Vehicle Make:        Chevrolet\n  Vehicle Model:       Spark (Subcompact Hatchback)\n  Max Vehicles Passed: {}\n  Max Velocity:        {:.1} px/s\n  Min Velocity:        {:.1} px/s\n  Max Transit Time:    {:.2} s\n  Min Transit Time:    {:.2} s\n  Close Calls:         {}\n=====================================================",
        sim.stats.max_vehicles_passed, sim.stats.max_velocity, sim.stats.display_min_velocity(),
        sim.stats.max_time, sim.stats.display_min_time(), sim.stats.close_calls);
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

    #[test]
    fn test_physics_engine_velocity_distance_time() {
        for &speed in &VelocityTier::all_speeds() {
            let (time, dist) = (3.5, PhysicsEngine::distance(speed, 3.5));
            assert!((dist - speed * time).abs() < 0.001);
            assert!((PhysicsEngine::velocity(dist, time) - speed).abs() < 0.001);
            assert!((PhysicsEngine::time(dist, speed) - time).abs() < 0.001);
        }
        assert_eq!(PhysicsEngine::velocity(100.0, 0.0), 0.0);
        assert!(PhysicsEngine::time(100.0, 0.0).is_infinite());
    }

    #[test]
    fn test_physics_engine_acceleration_and_deceleration() {
        let v1 = PhysicsEngine::step_velocity(50.0, 150.0, 100.0, 200.0, 0.5);
        assert_eq!(v1, 100.0);
        assert_eq!(PhysicsEngine::step_velocity(v1, 150.0, 100.0, 200.0, 1.0), 150.0);
        assert_eq!(PhysicsEngine::step_velocity(150.0, 50.0, 100.0, 200.0, 0.5), 50.0);
    }

    #[test]
    fn test_safety_distance_detection() {
        let p1 = Point2D::new(100.0, 100.0);
        let check = |dy, expected| assert_eq!(PhysicsEngine::evaluate_safety_distance(p1, Point2D::new(100.0, 100.0 + dy), SAFETY_DISTANCE), expected);
        check(SAFETY_DISTANCE - 5.0, SafetyStatus::CloseCall);
        check(SAFETY_DISTANCE, SafetyStatus::CloseCall);
        check(SAFETY_DISTANCE * 0.2, SafetyStatus::Violation);
        check(SAFETY_DISTANCE + 25.0, SafetyStatus::Safe);
    }

    #[test]
    fn test_car_following_safety_stop() {
        let mut sim = Simulation::new();
        let (mut v1, mut v2) = (Vehicle::new(1, Origin::South, Turn::Straight, 100.0, 0), Vehicle::new(2, Origin::South, Turn::Straight, 100.0, 1));
        v1.distance = 200.0;
        v2.distance = 200.0 - (VEHICLE_LENGTH + SAFETY_DISTANCE - 10.0);
        sim.vehicles.extend([v1, v2]);
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);
        assert_eq!(sim.vehicles[1].target_velocity, 0.0);
    }

    #[test]
    fn test_smart_intersection_algorithm_conflicting_routes() {
        let mut sim = Simulation::new();
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Straight, 120.0, 0);
        v1.distance = v1.route.stop_line_distance + 162.0 - 180.0;
        let mut v2 = Vehicle::new(2, Origin::East, Turn::Straight, 120.0, 1);
        v2.distance = v2.route.stop_line_distance + 54.0 - 190.0;
        sim.vehicles.extend([v1, v2]);
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);
        let (t1, t2) = (sim.vehicles[0].target_velocity, sim.vehicles[1].target_velocity);
        assert!(t1 < 120.0 || t2 < 120.0);
        assert!(t2 < t1);
    }

    #[test]
    fn test_conflicting_left_turn_and_straight() {
        let mut sim = Simulation::new();
        let mut v1 = Vehicle::new(1, Origin::North, Turn::Left, 110.0, 0);
        v1.distance = v1.route.stop_line_distance - 25.0;
        let mut v2 = Vehicle::new(2, Origin::South, Turn::Straight, 110.0, 1);
        v2.distance = v2.route.stop_line_distance - 30.0;
        sim.vehicles.extend([v1, v2]);
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);
        assert!(sim.vehicles[0].target_velocity != sim.vehicles[1].target_velocity || sim.vehicles[0].target_velocity < 110.0);
    }

    #[test]
    fn test_statistics_accumulation() {
        let mut tr = StatsTracker::new();
        assert_eq!((tr.max_vehicles_passed, tr.max_velocity, tr.close_calls), (0, 0.0, 0));
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Straight, 120.0, 0);
        (v1.max_velocity_reached, v1.min_velocity_reached, v1.transit_time) = (150.0, 60.0, 4.2);
        tr.record_spawn(); tr.record_vehicle_passed(&v1);
        assert_eq!((tr.max_vehicles_passed, tr.max_velocity, tr.min_velocity, tr.max_time, tr.min_time), (1, 150.0, 60.0, 4.2, 4.2));
        let mut v2 = Vehicle::new(2, Origin::North, Turn::Right, 90.0, 1);
        (v2.max_velocity_reached, v2.min_velocity_reached, v2.transit_time) = (180.0, 40.0, 2.8);
        tr.record_spawn(); tr.record_vehicle_passed(&v2);
        assert_eq!((tr.max_vehicles_passed, tr.max_velocity, tr.min_velocity, tr.max_time, tr.min_time), (2, 180.0, 40.0, 4.2, 2.8));
        tr.record_close_call(); tr.record_close_call();
        assert_eq!(tr.close_calls, 2);
    }

    #[test]
    fn test_anti_overlap_spawn_protection() {
        let mut sim = Simulation::new();
        assert!(sim.spawn_vehicle(Origin::South) && sim.spawn_vehicle(Origin::South) && sim.spawn_vehicle(Origin::South));
        assert!(!sim.spawn_vehicle(Origin::South));
        (0..100).for_each(|_| sim.update(0.016));
        assert!(sim.spawn_vehicle(Origin::South));
    }

    #[test]
    fn test_route_turning_headings() {
        let r_right = Route::new(Origin::South, Turn::Right);
        let (h_start, h_end) = (r_right.heading_at(5.0), r_right.heading_at(r_right.total_distance - 5.0));
        assert!((h_start - (-90.0)).abs() < 5.0 || (h_start - 270.0).abs() < 5.0);
        assert!(h_end.abs() < 5.0 || (h_end - 360.0).abs() < 5.0);
        let r_str = Route::new(Origin::South, Turn::Straight);
        assert!((r_str.heading_at(50.0) - r_str.heading_at(r_str.total_distance - 50.0)).abs() < 1.0);
    }

    #[test]
    fn test_parallel_non_conflicting_routes_flow_simultaneously() {
        let mut sim = Simulation::new();
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Straight, 120.0, 0);
        v1.distance = v1.route.stop_line_distance - 40.0;
        let mut v2 = Vehicle::new(2, Origin::North, Turn::Straight, 120.0, 1);
        v2.distance = v2.route.stop_line_distance - 40.0;
        sim.vehicles.extend([v1, v2]);
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);
        let fast = VelocityTier::Fast.speed();
        assert_eq!((sim.vehicles[0].target_velocity, sim.vehicles[1].target_velocity), (fast, fast));
    }

    #[test]
    fn test_right_turn_free_flow_unhindered() {
        let mut sim = Simulation::new();
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Right, 100.0, 0);
        v1.distance = v1.route.stop_line_distance - 20.0;
        let mut v2 = Vehicle::new(2, Origin::East, Turn::Straight, 100.0, 1);
        v2.distance = v2.route.stop_line_distance - 20.0;
        sim.vehicles.extend([v1, v2]);
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);
        assert_eq!(sim.vehicles[0].target_velocity, VelocityTier::Fast.speed());
    }

    #[test]
    fn test_time_to_conflict_separation_no_false_stops() {
        let mut sim = Simulation::new();
        let mut v1 = Vehicle::new(1, Origin::South, Turn::Straight, 165.0, 0);
        v1.distance = v1.route.stop_line_distance + 162.0 - 50.0;
        let mut v2 = Vehicle::new(2, Origin::East, Turn::Straight, 105.0, 1);
        v2.distance = v2.route.stop_line_distance + 54.0 - 230.0;
        sim.vehicles.extend([v1, v2]);
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);
        assert_eq!(sim.vehicles[0].target_velocity, VelocityTier::Fast.speed());
        assert!(sim.vehicles[1].target_velocity > 0.0);
    }

    #[test]
    fn test_four_way_simultaneous_gate_deadlock_breaker() {
        let mut sim = Simulation::new();
        for (id, (orig, lane)) in [(10, (Origin::North, 0)), (11, (Origin::East, 1)), (12, (Origin::South, 2)), (13, (Origin::West, 3))] {
            let mut v = Vehicle::new(id, orig, Turn::Straight, 0.0, lane);
            v.distance = v.route.stop_line_distance - 5.0;
            sim.vehicles.push(v);
        }
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);
        assert_eq!(sim.vehicles[0].target_velocity, VelocityTier::Fast.speed());
    }

    #[test]
    fn test_opposite_left_turns_diverge_without_blocking() {
        let mut sim = Simulation::new();
        let mut v1 = Vehicle::new(1, Origin::North, Turn::Left, 100.0, 0);
        v1.distance = v1.route.stop_line_distance - 25.0;
        let mut v2 = Vehicle::new(2, Origin::South, Turn::Left, 100.0, 1);
        v2.distance = v2.route.stop_line_distance - 25.0;
        sim.vehicles.extend([v1, v2]);
        sim.manager.manage_traffic(&mut sim.vehicles, &mut sim.stats, 0.016);
        let fast = VelocityTier::Fast.speed();
        assert_eq!((sim.vehicles[0].target_velocity, sim.vehicles[1].target_velocity), (fast, fast));
    }

    #[test]
    fn test_eight_way_straight_and_left_turns_no_deadlock() {
        let mut sim = Simulation::new();
        let mut id = 1;
        for orig in [Origin::North, Origin::East, Origin::South, Origin::West] {
            for turn in [Turn::Straight, Turn::Left] {
                let mut v = Vehicle::new(id, orig, turn, 0.0, (id - 1) % 4);
                v.distance = v.route.stop_line_distance - 8.0;
                sim.vehicles.push(v);
                id += 1;
            }
        }
        assert_eq!(sim.vehicles.len(), 8);
        (0..750).for_each(|_| sim.update(0.016));
        assert_eq!(sim.stats.max_vehicles_passed, 8);
    }
}
