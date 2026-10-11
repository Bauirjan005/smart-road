use crate::route::get_route_conflict;
use crate::stats::StatsTracker;
use crate::types::{
    Turn, VelocityTier, SAFETY_DISTANCE, VEHICLE_LENGTH,
};
use crate::vehicle::Vehicle;

#[derive(Debug)]
pub struct SmartController {
    pub safety_distance: f32,
    pub close_call_pairs: std::collections::HashSet<(usize, usize)>,
}

pub type SmartIntersectionManager = SmartController;

impl SmartController {
    pub fn new() -> Self {
        Self {
            safety_distance: SAFETY_DISTANCE,
            close_call_pairs: std::collections::HashSet::new(),
        }
    }

    /// Алиас для обратной совместимости с существующим кодом симуляции
    pub fn manage_traffic(
        &mut self,
        vehicles: &mut [Vehicle],
        stats: &mut StatsTracker,
        dt: f32,
    ) {
        self.update(vehicles, stats, dt);
    }

    /// Главная логика умного контроллера перекрестка (SmartController).
    /// Полностью исключает циклические взаимные ожидания (Deadlock Free),
    /// обеспечивает абсолютный tie-breaker на стоп-линии, параллельный проезд
    /// встречных потоков и непрерывность движения внутри перекрестка.
    pub fn update(
        &mut self,
        vehicles: &mut [Vehicle],
        stats: &mut StatsTracker,
        _dt: f32,
    ) {
        let n = vehicles.len();
        if n == 0 {
            return;
        }

        let fast_speed = VelocityTier::Fast.speed();     // 165.0 px/s
        let medium_speed = VelocityTier::Medium.speed(); // 105.0 px/s
        let slow_speed = VelocityTier::Slow.speed();     // 55.0 px/s
        let same_lane_safety = self.safety_distance.max(45.0); // 45.0 px безопасная дистанция в одной полосе

        // 1. По умолчанию все автомобили стремятся двигаться с высокой скоростью (Fast = 165.0)
        for v in vehicles.iter_mut() {
            v.target_velocity = fast_speed;
        }

        // 2. ДИНАМИЧЕСКАЯ ОЧЕРЕДЬ И РАЗРЕШЕНИЕ КОНФЛИКТОВ (AIM + DEADLOCK-FREE TIE-BREAKER)
        // ТРЕБОВАНИЕ 3: Автомобиль внутри периметра перекрестка (is_in_intersection) получает
        // абсолютный приоритет (Fast) и беспрепятственно освобождает перекресток.
        // ТРЕБОВАНИЕ 1: На стоп-линии при взаимной блокировке право первого проезда строго
        // отдается машине с наименьшим ID / наибольшим временем ожидания.
        // ТРЕБОВАНИЕ 2: Встречные параллельные потоки (North <-> South, East <-> West) едут парами одновременно.
        // ТРЕБОВАНИЕ 4: Пересечения проверяются только в пределах границ перекрестка (box intersection).

        // Шаг 2.1: Определение начального статуса уступки дороги (Pass 1: Конфликты приоритетов)
        let mut must_yield = vec![false; n];

        for i in 0..n {
            let in_intersection_i = vehicles[i].route.is_in_intersection(vehicles[i].distance)
                || vehicles[i].distance >= vehicles[i].route.stop_line_distance;

            // ТРЕБОВАНИЕ 3: Машины внутри перекрестка никогда не уступают
            if in_intersection_i {
                continue;
            }

            // ТРЕБОВАНИЕ 1 & 2: Правые повороты идут по внешней траектории без пересечения центра
            if vehicles[i].turn == Turn::Right {
                continue;
            }

            let dist_to_stop_i = vehicles[i].route.stop_line_distance - vehicles[i].distance;
            let is_at_gate_i = dist_to_stop_i <= 65.0 && dist_to_stop_i >= -5.0;

            for j in 0..n {
                if i == j {
                    continue;
                }

                // Машины в одной полосе регулируются car-following (шаг 3 ниже)
                if vehicles[i].origin == vehicles[j].origin && vehicles[i].turn == vehicles[j].turn {
                    continue;
                }

                // Правые повороты не конфликтуют
                if vehicles[j].turn == Turn::Right {
                    continue;
                }

                // ТРЕБОВАНИЕ 4: Проверяем пересечение в границах перекрестка (box intersection)
                let conflict = match get_route_conflict(&vehicles[i].route, &vehicles[j].route) {
                    Some(c) => c,
                    None => continue, // Встречные потоки прямо (North-South, East-West) и встречные левые возвращают None!
                };

                let d_cross_i = conflict.dist_a;
                let d_cross_j = conflict.dist_b;

                // Если машина j уже проехала точку пересечения — она освободила дорогу для i
                if vehicles[j].distance > d_cross_j + VEHICLE_LENGTH * 0.75 {
                    continue;
                }

                // Если машина i сама уже миновала точку пересечения
                if vehicles[i].distance > d_cross_i + VEHICLE_LENGTH * 0.75 {
                    continue;
                }

                let in_intersection_j = vehicles[j].route.is_in_intersection(vehicles[j].distance)
                    || vehicles[j].distance >= vehicles[j].route.stop_line_distance;

                let dist_to_stop_j = vehicles[j].route.stop_line_distance - vehicles[j].distance;
                let is_at_gate_j = dist_to_stop_j <= 65.0 && dist_to_stop_j >= -5.0;

                let delta_d_i = (d_cross_i - vehicles[i].distance).max(0.0);
                let delta_d_j = (d_cross_j - vehicles[j].distance).max(0.0);

                let v_i_est = vehicles[i].velocity.max(slow_speed);
                let v_j_est = vehicles[j].velocity.max(slow_speed);

                let t_i = delta_d_i / v_i_est;
                let t_j = delta_d_j / v_j_est;

                // Определение приоритета между i и j:
                let j_has_priority = if in_intersection_j {
                    // ТРЕБОВАНИЕ 3: Машина j уже на перекрестке -> безоговорочный приоритет
                    true
                } else if is_at_gate_i && is_at_gate_j {
                    // ТРЕБОВАНИЕ 1: АБСОЛЮТНЫЙ TIE-BREAKER НА СТОП-ЛИНИИ
                    // Если обе машины у ворот: приоритет той, которая ждет дольше, затем по наименьшему ID (FIFO)
                    if (vehicles[j].transit_time - vehicles[i].transit_time).abs() > 0.4 {
                        vehicles[j].transit_time > vehicles[i].transit_time
                    } else {
                        vehicles[j].id < vehicles[i].id
                    }
                } else if is_at_gate_j && !is_at_gate_i {
                    // j уже у стоп-линии, а i еще на дальнем подъезде
                    true
                } else if is_at_gate_i && !is_at_gate_j {
                    // i уже у стоп-линии, а j еще на дальнем подъезде
                    false
                } else {
                    // Обе машины на дальнем подъезде: сравниваем время прибытия к точке конфликта
                    if (t_i - t_j).abs() > 0.4 {
                        t_j < t_i
                    } else {
                        if (dist_to_stop_j - dist_to_stop_i).abs() > 6.0 {
                            dist_to_stop_j < dist_to_stop_i
                        } else {
                            vehicles[j].id < vehicles[i].id
                        }
                    }
                };

                if j_has_priority {
                    let time_conflict = in_intersection_j
                        || (is_at_gate_i && is_at_gate_j)
                        || is_at_gate_j
                        || (t_i - t_j).abs() < 1.35;

                    if time_conflict {
                        must_yield[i] = true;
                        break;
                    }
                }
            }
        }

        // Шаг 2.2: Параллельное движение встречных потоков (Pass 2: Parallel Cohort Activation)
        // ТРЕБОВАНИЕ 2: Если машина i помечена как уступающая из-за машины j, но машина j сама уступает дорогу
        // и не может двигаться, а с машиной k (которая сейчас едет) машина i не конфликтует,
        // то машина i имеет право ехать параллельно с машиной k!
        for i in 0..n {
            if !must_yield[i] {
                continue;
            }
            let in_intersection_i = vehicles[i].route.is_in_intersection(vehicles[i].distance)
                || vehicles[i].distance >= vehicles[i].route.stop_line_distance;
            if in_intersection_i || vehicles[i].turn == Turn::Right {
                must_yield[i] = false;
                continue;
            }

            // Проверяем: конфликтует ли машина i ХОТЯ БЫ С ОДНОЙ машиной, которая РЕАЛЬНО сейчас едет (in_intersection или !must_yield)
            let conflicts_with_active = (0..n).any(|k| {
                if i == k {
                    return false;
                }
                if vehicles[i].origin == vehicles[k].origin && vehicles[i].turn == vehicles[k].turn {
                    return false;
                }
                if vehicles[k].turn == Turn::Right {
                    return false;
                }

                let in_intersection_k = vehicles[k].route.is_in_intersection(vehicles[k].distance)
                    || vehicles[k].distance >= vehicles[k].route.stop_line_distance;
                let k_is_active = in_intersection_k || !must_yield[k];

                if !k_is_active {
                    return false;
                }

                // Проверяем, пересекаются ли пути i и активной машины k
                if let Some(c) = get_route_conflict(&vehicles[i].route, &vehicles[k].route) {
                    // Конфликтует, если k еще не миновала точку пересечения
                    vehicles[k].distance <= c.dist_b + VEHICLE_LENGTH * 0.75
                } else {
                    false
                }
            });

            // Если машина i НЕ конфликтует ни с одной активно двигающейся машиной — она едет параллельно!
            if !conflicts_with_active {
                must_yield[i] = false;
            }
        }

        // Шаг 2.3: Применение скоростных профилей на основе статуса уступки
        for i in 0..n {
            let in_intersection_i = vehicles[i].route.is_in_intersection(vehicles[i].distance)
                || vehicles[i].distance >= vehicles[i].route.stop_line_distance;

            // ТРЕБОВАНИЕ 3: Машины внутри перекрестка идут на Fast скорости
            if in_intersection_i || vehicles[i].turn == Turn::Right {
                vehicles[i].target_velocity = fast_speed;
                continue;
            }

            if must_yield[i] {
                let dist_to_stop_i = vehicles[i].route.stop_line_distance - vehicles[i].distance;
                if dist_to_stop_i <= 14.0 {
                    // Полная остановка перед стоп-линией
                    vehicles[i].target_velocity = 0.0;
                } else if dist_to_stop_i <= 70.0 {
                    // Плавное торможение к стоп-линии
                    let ratio = (dist_to_stop_i / 70.0).clamp(0.0, 1.0);
                    let target = (slow_speed * ratio).max(12.0);
                    vehicles[i].target_velocity = vehicles[i].target_velocity.min(target);
                } else {
                    // Заблаговременное замедление до Slow
                    vehicles[i].target_velocity = vehicles[i].target_velocity.min(slow_speed);
                }
            } else {
                // ТРЕБОВАНИЕ 1: Машина с правом проезда обязана начать движение (Fast)
                vehicles[i].target_velocity = fast_speed;
            }
        }

        // 3. Движение в одной полосе (Car-Following Leader-Follower Queue)
        // Строго соблюдаем безопасную дистанцию между идущими друг за другом машинами в одной полосе
        for i in 0..n {
            let mut min_gap = f32::MAX;
            let mut leader_idx = None;

            for j in 0..n {
                if i == j {
                    continue;
                }
                if vehicles[i].origin == vehicles[j].origin && vehicles[i].turn == vehicles[j].turn {
                    if vehicles[j].distance > vehicles[i].distance {
                        let gap = vehicles[j].distance - vehicles[i].distance - VEHICLE_LENGTH;
                        if gap < min_gap {
                            min_gap = gap;
                            leader_idx = Some(j);
                        }
                    }
                }
            }

            if let Some(leader) = leader_idx {
                if min_gap <= same_lane_safety {
                    // Опасное сближение бампер к бамперу в полосе -> остановка
                    vehicles[i].target_velocity = 0.0;
                } else if min_gap <= same_lane_safety + 35.0 {
                    // Следование за лидером на скорости лидера / Slow
                    let leader_speed = vehicles[leader].velocity;
                    vehicles[i].target_velocity = vehicles[i]
                        .target_velocity
                        .min(leader_speed)
                        .min(slow_speed);
                } else if min_gap <= same_lane_safety + 80.0 {
                    vehicles[i].target_velocity = vehicles[i]
                        .target_velocity
                        .min(medium_speed);
                }
            }
        }

        // 4. Мониторинг опасных сближений (Close Calls & Collision Prevention)
        // ТРЕБОВАНИЕ 4: Проверять пересечение путей только в пределах границ перекрестка (box intersection)
        for i in 0..n {
            for j in (i + 1)..n {
                let p1 = vehicles[i].position();
                let p2 = vehicles[j].position();
                let dist = p1.distance_to(&p2);

                let min_id = vehicles[i].id.min(vehicles[j].id);
                let max_id = vehicles[i].id.max(vehicles[j].id);
                let pair_key = (min_id, max_id);

                let in_box_i = vehicles[i].route.is_in_intersection(vehicles[i].distance);
                let in_box_j = vehicles[j].route.is_in_intersection(vehicles[j].distance);

                if dist <= self.safety_distance {
                    // Фиксируем close call, если машины на конфликтных маршрутах
                    let is_conflicting = get_route_conflict(&vehicles[i].route, &vehicles[j].route).is_some();
                    if is_conflicting || (vehicles[i].origin == vehicles[j].origin && vehicles[i].turn == vehicles[j].turn) {
                        if !self.close_call_pairs.contains(&pair_key) {
                            self.close_call_pairs.insert(pair_key);
                            stats.record_close_call();
                        }
                    }

                    // Экстренная остановка только для конфликтующих маршрутов, если одна машина пытается заехать на уже занятый перекресток
                    if is_conflicting {
                        if in_box_i && !in_box_j {
                            vehicles[j].target_velocity = 0.0;
                        } else if in_box_j && !in_box_i {
                            vehicles[i].target_velocity = 0.0;
                        }
                    }
                } else if dist > self.safety_distance * 1.5 {
                    self.close_call_pairs.remove(&pair_key);
                }
            }
        }
    }
}
