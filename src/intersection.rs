use crate::route::get_route_conflict;
use crate::stats::StatsTracker;
use crate::types::{Turn, VelocityTier, SAFETY_DISTANCE, VEHICLE_LENGTH};
use crate::vehicle::Vehicle;
use std::collections::HashSet;

#[derive(Debug)]
pub struct SmartController {
    pub safety_distance: f32,
    pub close_call_pairs: HashSet<(usize, usize)>,
}

pub type SmartIntersectionManager = SmartController;

impl SmartController {
    pub fn new() -> Self {
        Self { safety_distance: SAFETY_DISTANCE, close_call_pairs: HashSet::new() }
    }

    pub fn manage_traffic(&mut self, vehicles: &mut [Vehicle], stats: &mut StatsTracker, dt: f32) {
        self.update(vehicles, stats, dt);
    }

    pub fn update(&mut self, vehicles: &mut [Vehicle], stats: &mut StatsTracker, _dt: f32) {
        let n = vehicles.len();
        if n == 0 { return; }

        let (fast, med, slow) = (VelocityTier::Fast.speed(), VelocityTier::Medium.speed(), VelocityTier::Slow.speed());
        let same_lane_safety = self.safety_distance.max(45.0);
        for v in vehicles.iter_mut() { v.target_velocity = fast; }

        let in_box: Vec<bool> = vehicles.iter().map(|v| v.route.is_in_intersection(v.distance) || v.distance >= v.route.stop_line_distance).collect();
        let dist_to_stop: Vec<f32> = vehicles.iter().map(|v| v.route.stop_line_distance - v.distance).collect();
        let at_gate: Vec<bool> = dist_to_stop.iter().map(|&d| d <= 65.0 && d >= -5.0).collect();

        // 1. Pass 1: Выявление конфликтов и приоритетов (FIFO / tie-breaker)
        let mut must_yield = vec![false; n];
        for i in 0..n {
            if in_box[i] || vehicles[i].turn == Turn::Right { continue; }
            for j in 0..n {
                if i == j || (vehicles[i].origin == vehicles[j].origin && vehicles[i].turn == vehicles[j].turn) || vehicles[j].turn == Turn::Right { continue; }
                let conflict = match get_route_conflict(&vehicles[i].route, &vehicles[j].route) {
                    Some(c) => c,
                    None => continue,
                };
                let clen = VEHICLE_LENGTH * 0.75;
                if vehicles[j].distance > conflict.dist_b + clen || vehicles[i].distance > conflict.dist_a + clen { continue; }

                let t_i = (conflict.dist_a - vehicles[i].distance).max(0.0) / vehicles[i].velocity.max(slow);
                let t_j = (conflict.dist_b - vehicles[j].distance).max(0.0) / vehicles[j].velocity.max(slow);

                let j_prio = if in_box[j] {
                    true
                } else if at_gate[i] && at_gate[j] {
                    if (vehicles[j].transit_time - vehicles[i].transit_time).abs() > 0.4 {
                        vehicles[j].transit_time > vehicles[i].transit_time
                    } else {
                        vehicles[j].id < vehicles[i].id
                    }
                } else if at_gate[j] != at_gate[i] {
                    at_gate[j]
                } else if (t_i - t_j).abs() > 0.4 {
                    t_j < t_i
                } else if (dist_to_stop[j] - dist_to_stop[i]).abs() > 6.0 {
                    dist_to_stop[j] < dist_to_stop[i]
                } else {
                    vehicles[j].id < vehicles[i].id
                };

                if j_prio && (in_box[j] || (at_gate[i] && at_gate[j]) || at_gate[j] || (t_i - t_j).abs() < 1.35) {
                    must_yield[i] = true;
                    break;
                }
            }
        }

        // 2. Pass 2: Параллельное движение встречных потоков (Parallel cohort activation)
        for i in 0..n {
            if !must_yield[i] || in_box[i] || vehicles[i].turn == Turn::Right {
                must_yield[i] = false;
                continue;
            }
            let blocked = (0..n).any(|k| {
                if i == k || vehicles[k].turn == Turn::Right || (vehicles[i].origin == vehicles[k].origin && vehicles[i].turn == vehicles[k].turn) { return false; }
                if !in_box[k] && must_yield[k] { return false; }
                get_route_conflict(&vehicles[i].route, &vehicles[k].route)
                    .map_or(false, |c| vehicles[k].distance <= c.dist_b + VEHICLE_LENGTH * 0.75)
            });
            if !blocked { must_yield[i] = false; }
        }

        // 3. Скоростные профили для машин
        for i in 0..n {
            if in_box[i] || vehicles[i].turn == Turn::Right || !must_yield[i] {
                vehicles[i].target_velocity = fast;
            } else {
                let d = dist_to_stop[i];
                vehicles[i].target_velocity = if d <= 14.0 { 0.0 }
                else if d <= 70.0 { vehicles[i].target_velocity.min((slow * (d / 70.0).clamp(0.0, 1.0)).max(12.0)) }
                else { vehicles[i].target_velocity.min(slow) };
            }
        }

        // 4. Car-Following в одной полосе
        for i in 0..n {
            let lead = vehicles.iter().enumerate().filter(|&(j, v)| {
                i != j && vehicles[i].origin == v.origin && vehicles[i].turn == v.turn && v.distance > vehicles[i].distance
            }).min_by(|(_, a), (_, b)| a.distance.partial_cmp(&b.distance).unwrap());

            if let Some((_, l)) = lead {
                let gap = l.distance - vehicles[i].distance - VEHICLE_LENGTH;
                if gap <= same_lane_safety {
                    vehicles[i].target_velocity = 0.0;
                } else if gap <= same_lane_safety + 35.0 {
                    vehicles[i].target_velocity = vehicles[i].target_velocity.min(l.velocity).min(slow);
                } else if gap <= same_lane_safety + 80.0 {
                    vehicles[i].target_velocity = vehicles[i].target_velocity.min(med);
                }
            }
        }

        // 5. Мониторинг опасных сближений (Close Calls)
        for i in 0..n {
            for j in (i + 1)..n {
                let dist = vehicles[i].position().distance_to(&vehicles[j].position());
                let pair_key = (vehicles[i].id.min(vehicles[j].id), vehicles[i].id.max(vehicles[j].id));
                let same_lane = vehicles[i].origin == vehicles[j].origin && vehicles[i].turn == vehicles[j].turn;
                let is_conflicting = get_route_conflict(&vehicles[i].route, &vehicles[j].route).is_some();

                if dist <= self.safety_distance {
                    if (is_conflicting || same_lane) && self.close_call_pairs.insert(pair_key) {
                        stats.record_close_call();
                    }
                    if is_conflicting {
                        if in_box[i] && !in_box[j] { vehicles[j].target_velocity = 0.0; }
                        else if in_box[j] && !in_box[i] { vehicles[i].target_velocity = 0.0; }
                    }
                } else if dist > self.safety_distance * 1.5 {
                    self.close_call_pairs.remove(&pair_key);
                }
            }
        }
    }
}
