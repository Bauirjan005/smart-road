use crate::types::{
    Origin, Point2D, Turn, CANVAS_HEIGHT, CANVAS_WIDTH, CENTER_X, CENTER_Y,
    DETECTION_DISTANCE_AHEAD, HALF_ROAD_WIDTH, LANE_WIDTH,
};

#[derive(Debug, Clone)]
pub struct Route {
    pub origin: Origin,
    pub turn: Turn,
    pub waypoints: Vec<Point2D>,
    pub cumulative_dist: Vec<f32>,
    pub total_distance: f32,
    pub detection_distance: f32,
    pub stop_line_distance: f32,
    pub exit_distance: f32,
}

impl Route {
    pub fn new(origin: Origin, turn: Turn) -> Self {
        let (raw_points, stop_idx, exit_idx) = generate_route_points(origin, turn);
        
        let mut cumulative_dist = Vec::with_capacity(raw_points.len());
        cumulative_dist.push(0.0);
        let mut total = 0.0;
        
        for i in 1..raw_points.len() {
            let seg_len = raw_points[i - 1].distance_to(&raw_points[i]);
            total += seg_len;
            cumulative_dist.push(total);
        }

        let stop_line_distance = cumulative_dist[stop_idx.min(raw_points.len() - 1)];
        let exit_distance = cumulative_dist[exit_idx.min(raw_points.len() - 1)];
        let detection_distance = (stop_line_distance - DETECTION_DISTANCE_AHEAD).max(0.0);

        Self {
            origin,
            turn,
            waypoints: raw_points,
            cumulative_dist,
            total_distance: total,
            detection_distance,
            stop_line_distance,
            exit_distance,
        }
    }

    pub fn position_at(&self, d: f32) -> Point2D {
        if self.waypoints.is_empty() {
            return Point2D::ZERO;
        }
        if d <= 0.0 {
            return self.waypoints[0];
        }
        if d >= self.total_distance {
            return *self.waypoints.last().unwrap();
        }

        // Binary search for the segment
        match self.cumulative_dist.binary_search_by(|dist| dist.partial_cmp(&d).unwrap()) {
            Ok(idx) => self.waypoints[idx],
            Err(idx) => {
                if idx == 0 {
                    self.waypoints[0]
                } else if idx >= self.waypoints.len() {
                    *self.waypoints.last().unwrap()
                } else {
                    let d0 = self.cumulative_dist[idx - 1];
                    let d1 = self.cumulative_dist[idx];
                    let t = if (d1 - d0).abs() > 0.0001 {
                        (d - d0) / (d1 - d0)
                    } else {
                        0.0
                    };
                    self.waypoints[idx - 1].lerp(&self.waypoints[idx], t)
                }
            }
        }
    }

    pub fn heading_at_smooth(&self, d: f32, previous_heading: f32) -> f32 {
        let sample_offset = 3.0;
        let d_prev = (d - sample_offset).max(0.0);
        let d_next = (d + sample_offset).min(self.total_distance);

        let p1 = self.position_at(d_prev);
        let p2 = self.position_at(d_next);

        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;

        // Плавный поворот: игнорируем микро-смещения меньше 0.1 пикселя, сохраняя угол
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < 0.01 { // 0.1^2 = 0.01
            previous_heading
        } else {
            dy.atan2(dx).to_degrees()
        }
    }

    pub fn heading_at(&self, d: f32) -> f32 {
        self.heading_at_smooth(d, self.origin.incoming_heading())
    }

    /// Check if distance is in the physical intersection box
    pub fn is_in_intersection(&self, d: f32) -> bool {
        d >= self.stop_line_distance && d <= self.exit_distance
    }

    /// Check if distance is in detection zone
    pub fn is_detected(&self, d: f32) -> bool {
        d >= self.detection_distance
    }

    /// Точки траектории внутри перекрестка от current_d до exit_distance
    pub fn remaining_intersection_path(&self, current_d: f32) -> Vec<Point2D> {
        let start_d = current_d.max(self.stop_line_distance);
        let end_d = self.exit_distance;
        if start_d >= end_d {
            return Vec::new();
        }
        let mut pts = Vec::new();
        let step = 12.0;
        let mut d = start_d;
        while d <= end_d {
            pts.push(self.position_at(d));
            d += step;
        }
        pts.push(self.position_at(end_d));
        pts
    }
}

/// Точные данные о пересечении двух маршрутов
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteConflict {
    pub dist_a: f32,
    pub dist_b: f32,
    pub min_dist: f32,
}

/// Индекс маршрута от 0 до 11 (4 стороны * 3 поворота)
pub fn route_index(origin: Origin, turn: Turn) -> usize {
    let o = match origin {
        Origin::North => 0,
        Origin::South => 1,
        Origin::East => 2,
        Origin::West => 3,
    };
    let t = match turn {
        Turn::Right => 0,
        Turn::Straight => 1,
        Turn::Left => 2,
    };
    o * 3 + t
}

/// Вычисляет точку минимального расстояния и возможное пересечение маршрутов внутри перекрестка
pub fn compute_route_conflict(route_a: &Route, route_b: &Route) -> Option<RouteConflict> {
    if route_a.origin == route_b.origin && route_a.turn == route_b.turn {
        return None;
    }

    // ТРЕБОВАНИЕ 1: Траектории правого поворота огибают перекресток и не пересекают центр
    if route_a.turn == Turn::Right || route_b.turn == Turn::Right {
        return None;
    }

    // ТРЕБОВАНИЕ 3: Встречные потоки, идущие прямо, не пересекаются
    let is_opposite_straight = (route_a.origin == Origin::North && route_b.origin == Origin::South
        || route_a.origin == Origin::South && route_b.origin == Origin::North
        || route_a.origin == Origin::East && route_b.origin == Origin::West
        || route_a.origin == Origin::West && route_b.origin == Origin::East)
        && route_a.turn == Turn::Straight
        && route_b.turn == Turn::Straight;

    if is_opposite_straight {
        return None;
    }

    // ТРЕБОВАНИЕ 3: Встречные левые повороты расходятся правыми бортами без пересечения (starboard-to-starboard)
    let is_opposite_left = (route_a.origin == Origin::North && route_b.origin == Origin::South
        || route_a.origin == Origin::South && route_b.origin == Origin::North
        || route_a.origin == Origin::East && route_b.origin == Origin::West
        || route_a.origin == Origin::West && route_b.origin == Origin::East)
        && route_a.turn == Turn::Left
        && route_b.turn == Turn::Left;

    if is_opposite_left {
        return None;
    }

    let start_a = route_a.stop_line_distance;
    let end_a = route_a.exit_distance;
    let start_b = route_b.stop_line_distance;
    let end_b = route_b.exit_distance;

    let step = 3.0;
    let mut min_dist = f32::MAX;
    let mut best_da = start_a;
    let mut best_db = start_b;

    let mut da = start_a;
    while da <= end_a {
        let pa = route_a.position_at(da);
        let mut db = start_b;
        while db <= end_b {
            let pb = route_b.position_at(db);
            let d = pa.distance_to(&pb);
            if d < min_dist {
                min_dist = d;
                best_da = da;
                best_db = db;
            }
            db += step;
        }
        da += step;
    }

    // Ширина полосы 36.0 px, ширина машины 20.0 px.
    // Если минимальное расстояние между траекториями меньше 34.0 px,
    // машины на этих маршрутах пересекаются и могут столкнуться.
    if min_dist < 34.0 {
        Some(RouteConflict {
            dist_a: best_da,
            dist_b: best_db,
            min_dist,
        })
    } else {
        None
    }
}

/// Предварительный расчет матрицы пересечений всех 12 маршрутов (144 комбинации)
pub fn compute_conflict_matrix() -> [[Option<RouteConflict>; 12]; 12] {
    let origins = [Origin::North, Origin::South, Origin::East, Origin::West];
    let turns = [Turn::Right, Turn::Straight, Turn::Left];

    let mut routes = Vec::with_capacity(12);
    for &o in &origins {
        for &t in &turns {
            routes.push(Route::new(o, t));
        }
    }

    let mut matrix = [[None; 12]; 12];
    for i in 0..12 {
        for j in 0..12 {
            if i == j {
                continue;
            }
            matrix[i][j] = compute_route_conflict(&routes[i], &routes[j]);
        }
    }
    matrix
}

/// Быстрый O(1) поиск информации о пересечении двух маршрутов через OnceLock матрицу
pub fn get_route_conflict(route_a: &Route, route_b: &Route) -> Option<RouteConflict> {
    static CONFLICT_MATRIX: std::sync::OnceLock<[[Option<RouteConflict>; 12]; 12]> =
        std::sync::OnceLock::new();
    let matrix = CONFLICT_MATRIX.get_or_init(compute_conflict_matrix);
    let idx_a = route_index(route_a.origin, route_a.turn);
    let idx_b = route_index(route_b.origin, route_b.turn);
    matrix[idx_a][idx_b]
}

/// Проверка пересечения траекторий внутри зоны перекрестка (обратная совместимость)
pub fn trajectories_intersect(
    route_a: &Route,
    dist_a: f32,
    route_b: &Route,
    dist_b: f32,
    safety_threshold: f32,
) -> bool {
    if route_a.origin == route_b.origin && route_a.turn == route_b.turn {
        return false;
    }

    // Правые повороты не пересекают центр
    if route_a.turn == Turn::Right || route_b.turn == Turn::Right {
        return false;
    }

    // Встречные потоки прямо
    let is_opposite_straight = (route_a.origin == Origin::North && route_b.origin == Origin::South
        || route_a.origin == Origin::South && route_b.origin == Origin::North
        || route_a.origin == Origin::East && route_b.origin == Origin::West
        || route_a.origin == Origin::West && route_b.origin == Origin::East)
        && route_a.turn == Turn::Straight
        && route_b.turn == Turn::Straight;

    if is_opposite_straight {
        return false;
    }

    // Встречные левые повороты расходятся правыми бортами
    let is_opposite_left = (route_a.origin == Origin::North && route_b.origin == Origin::South
        || route_a.origin == Origin::South && route_b.origin == Origin::North
        || route_a.origin == Origin::East && route_b.origin == Origin::West
        || route_a.origin == Origin::West && route_b.origin == Origin::East)
        && route_a.turn == Turn::Left
        && route_b.turn == Turn::Left;

    if is_opposite_left {
        return false;
    }

    let path_a = route_a.remaining_intersection_path(dist_a);
    let path_b = route_b.remaining_intersection_path(dist_b);

    if path_a.is_empty() || path_b.is_empty() {
        return false;
    }

    // Безопасный габарит ширины полосы (LANE_WIDTH = 36.0, порог 34.0)
    let threshold = safety_threshold.min(34.0);

    for p_a in &path_a {
        for p_b in &path_b {
            if p_a.distance_to(p_b) < threshold {
                return true;
            }
        }
    }
    false
}

/// Helper to generate detailed waypoints for each origin & turn combination
fn generate_route_points(origin: Origin, turn: Turn) -> (Vec<Point2D>, usize, usize) {
    let cx = CENTER_X;
    let cy = CENTER_Y;
    let half_road = HALF_ROAD_WIDTH; // 108.0

    // Coordinates for lane centers
    // Incoming lane offsets from road center line (driving on the right):
    // Lane 0 (Left turn 'l'): offset = 0.5 * LANE_WIDTH = 18.0
    // Lane 1 (Straight 's'): offset = 1.5 * LANE_WIDTH = 54.0
    // Lane 2 (Right turn 'r'): offset = 2.5 * LANE_WIDTH = 90.0
    let l_in = 0.5 * LANE_WIDTH;  // 18.0
    let s_in = 1.5 * LANE_WIDTH;  // 54.0
    let r_in = 2.5 * LANE_WIDTH;  // 90.0

    // Outgoing lane offsets from centerline (leaving intersection on the right):
    let l_out = 0.5 * LANE_WIDTH; // 18.0
    let _s_out = 1.5 * LANE_WIDTH; // 54.0
    let r_out = 2.5 * LANE_WIDTH; // 90.0

    let mut pts = Vec::new();
    let num_arc_steps = 24;

    match (origin, turn) {
        // --- SOUTH APPROACH: Moving North (y from 900 to 0) ---
        (Origin::South, Turn::Straight) => {
            let x = cx + s_in;
            pts.push(Point2D::new(x, CANVAS_HEIGHT));
            let stop_idx = pts.len();
            pts.push(Point2D::new(x, cy + half_road));
            pts.push(Point2D::new(x, cy - half_road));
            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(x, 0.0));
            (pts, stop_idx, exit_idx)
        }
        (Origin::South, Turn::Right) => {
            // Turns 90 deg right into Eastbound road (leaving towards x = 900)
            let in_x = cx + r_in;
            let out_y = cy + r_out;
            pts.push(Point2D::new(in_x, CANVAS_HEIGHT));
            let stop_idx = pts.len();
            pts.push(Point2D::new(in_x, cy + half_road));

            // Arc center is (cx + half_road, cy + half_road)
            let arc_cx = cx + half_road;
            let arc_cy = cy + half_road;
            let radius = half_road - r_in; // 108 - 90 = 18.0

            for i in 1..=num_arc_steps {
                let frac = i as f32 / num_arc_steps as f32;
                let angle = std::f32::consts::PI + frac * (std::f32::consts::FRAC_PI_2); // PI to 1.5 PI
                let px = arc_cx + radius * angle.cos();
                let py = arc_cy + radius * angle.sin();
                pts.push(Point2D::new(px, py));
            }

            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(CANVAS_WIDTH, out_y));
            (pts, stop_idx, exit_idx)
        }
        (Origin::South, Turn::Left) => {
            // Turns 90 deg left into Westbound road (leaving towards x = 0)
            let in_x = cx + l_in;
            let out_y = cy - l_out;
            pts.push(Point2D::new(in_x, CANVAS_HEIGHT));
            let stop_idx = pts.len();
            pts.push(Point2D::new(in_x, cy + half_road));

            // Smooth cubic Bezier or circular arc for turning left
            let p_start = Point2D::new(in_x, cy + half_road);
            let p_ctrl1 = Point2D::new(in_x, cy);
            let p_ctrl2 = Point2D::new(cx, out_y);
            let p_end = Point2D::new(cx - half_road, out_y);

            for i in 1..=num_arc_steps {
                let t = i as f32 / num_arc_steps as f32;
                let px = (1.0 - t).powi(3) * p_start.x
                    + 3.0 * (1.0 - t).powi(2) * t * p_ctrl1.x
                    + 3.0 * (1.0 - t) * t.powi(2) * p_ctrl2.x
                    + t.powi(3) * p_end.x;
                let py = (1.0 - t).powi(3) * p_start.y
                    + 3.0 * (1.0 - t).powi(2) * t * p_ctrl1.y
                    + 3.0 * (1.0 - t) * t.powi(2) * p_ctrl2.y
                    + t.powi(3) * p_end.y;
                pts.push(Point2D::new(px, py));
            }

            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(0.0, out_y));
            (pts, stop_idx, exit_idx)
        }

        // --- NORTH APPROACH: Moving South (y from 0 to 900) ---
        (Origin::North, Turn::Straight) => {
            let x = cx - s_in;
            pts.push(Point2D::new(x, 0.0));
            let stop_idx = pts.len();
            pts.push(Point2D::new(x, cy - half_road));
            pts.push(Point2D::new(x, cy + half_road));
            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(x, CANVAS_HEIGHT));
            (pts, stop_idx, exit_idx)
        }
        (Origin::North, Turn::Right) => {
            // Turns 90 deg right into Westbound road (leaving towards x = 0)
            let in_x = cx - r_in;
            let out_y = cy - r_out;
            pts.push(Point2D::new(in_x, 0.0));
            let stop_idx = pts.len();
            pts.push(Point2D::new(in_x, cy - half_road));

            let arc_cx = cx - half_road;
            let arc_cy = cy - half_road;
            let radius = half_road - r_in;

            for i in 1..=num_arc_steps {
                let frac = i as f32 / num_arc_steps as f32;
                let angle = 0.0 + frac * (std::f32::consts::FRAC_PI_2); // 0 to 0.5 PI
                let px = arc_cx + radius * angle.cos();
                let py = arc_cy + radius * angle.sin();
                pts.push(Point2D::new(px, py));
            }

            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(0.0, out_y));
            (pts, stop_idx, exit_idx)
        }
        (Origin::North, Turn::Left) => {
            // Turns 90 deg left into Eastbound road (leaving towards x = 900)
            let in_x = cx - l_in;
            let out_y = cy + l_out;
            pts.push(Point2D::new(in_x, 0.0));
            let stop_idx = pts.len();
            pts.push(Point2D::new(in_x, cy - half_road));

            let p_start = Point2D::new(in_x, cy - half_road);
            let p_ctrl1 = Point2D::new(in_x, cy);
            let p_ctrl2 = Point2D::new(cx, out_y);
            let p_end = Point2D::new(cx + half_road, out_y);

            for i in 1..=num_arc_steps {
                let t = i as f32 / num_arc_steps as f32;
                let px = (1.0 - t).powi(3) * p_start.x
                    + 3.0 * (1.0 - t).powi(2) * t * p_ctrl1.x
                    + 3.0 * (1.0 - t) * t.powi(2) * p_ctrl2.x
                    + t.powi(3) * p_end.x;
                let py = (1.0 - t).powi(3) * p_start.y
                    + 3.0 * (1.0 - t).powi(2) * t * p_ctrl1.y
                    + 3.0 * (1.0 - t) * t.powi(2) * p_ctrl2.y
                    + t.powi(3) * p_end.y;
                pts.push(Point2D::new(px, py));
            }

            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(CANVAS_WIDTH, out_y));
            (pts, stop_idx, exit_idx)
        }

        // --- WEST APPROACH: Moving East (x from 0 to 900) ---
        (Origin::West, Turn::Straight) => {
            let y = cy + s_in;
            pts.push(Point2D::new(0.0, y));
            let stop_idx = pts.len();
            pts.push(Point2D::new(cx - half_road, y));
            pts.push(Point2D::new(cx + half_road, y));
            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(CANVAS_WIDTH, y));
            (pts, stop_idx, exit_idx)
        }
        (Origin::West, Turn::Right) => {
            // Turns 90 deg right into Southbound road (leaving towards y = 900)
            let in_y = cy + r_in;
            let out_x = cx - r_out;
            pts.push(Point2D::new(0.0, in_y));
            let stop_idx = pts.len();
            pts.push(Point2D::new(cx - half_road, in_y));

            let arc_cx = cx - half_road;
            let arc_cy = cy + half_road;
            let radius = half_road - r_in;

            for i in 1..=num_arc_steps {
                let frac = i as f32 / num_arc_steps as f32;
                let angle = 1.5 * std::f32::consts::PI + frac * (std::f32::consts::FRAC_PI_2);
                let px = arc_cx + radius * angle.cos();
                let py = arc_cy + radius * angle.sin();
                pts.push(Point2D::new(px, py));
            }

            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(out_x, CANVAS_HEIGHT));
            (pts, stop_idx, exit_idx)
        }
        (Origin::West, Turn::Left) => {
            // Turns 90 deg left into Northbound road (leaving towards y = 0)
            let in_y = cy + l_in;
            let out_x = cx + l_out;
            pts.push(Point2D::new(0.0, in_y));
            let stop_idx = pts.len();
            pts.push(Point2D::new(cx - half_road, in_y));

            let p_start = Point2D::new(cx - half_road, in_y);
            let p_ctrl1 = Point2D::new(cx, in_y);
            let p_ctrl2 = Point2D::new(out_x, cy);
            let p_end = Point2D::new(out_x, cy - half_road);

            for i in 1..=num_arc_steps {
                let t = i as f32 / num_arc_steps as f32;
                let px = (1.0 - t).powi(3) * p_start.x
                    + 3.0 * (1.0 - t).powi(2) * t * p_ctrl1.x
                    + 3.0 * (1.0 - t) * t.powi(2) * p_ctrl2.x
                    + t.powi(3) * p_end.x;
                let py = (1.0 - t).powi(3) * p_start.y
                    + 3.0 * (1.0 - t).powi(2) * t * p_ctrl1.y
                    + 3.0 * (1.0 - t) * t.powi(2) * p_ctrl2.y
                    + t.powi(3) * p_end.y;
                pts.push(Point2D::new(px, py));
            }

            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(out_x, 0.0));
            (pts, stop_idx, exit_idx)
        }

        // --- EAST APPROACH: Moving West (x from 900 to 0) ---
        (Origin::East, Turn::Straight) => {
            let y = cy - s_in;
            pts.push(Point2D::new(CANVAS_WIDTH, y));
            let stop_idx = pts.len();
            pts.push(Point2D::new(cx + half_road, y));
            pts.push(Point2D::new(cx - half_road, y));
            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(0.0, y));
            (pts, stop_idx, exit_idx)
        }
        (Origin::East, Turn::Right) => {
            // Turns 90 deg right into Northbound road (leaving towards y = 0)
            let in_y = cy - r_in;
            let out_x = cx + r_out;
            pts.push(Point2D::new(CANVAS_WIDTH, in_y));
            let stop_idx = pts.len();
            pts.push(Point2D::new(cx + half_road, in_y));

            let arc_cx = cx + half_road;
            let arc_cy = cy - half_road;
            let radius = half_road - r_in;

            for i in 1..=num_arc_steps {
                let frac = i as f32 / num_arc_steps as f32;
                let angle = 0.5 * std::f32::consts::PI + frac * (std::f32::consts::FRAC_PI_2);
                let px = arc_cx + radius * angle.cos();
                let py = arc_cy + radius * angle.sin();
                pts.push(Point2D::new(px, py));
            }

            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(out_x, 0.0));
            (pts, stop_idx, exit_idx)
        }
        (Origin::East, Turn::Left) => {
            // Turns 90 deg left into Southbound road (leaving towards y = 900)
            let in_y = cy - l_in;
            let out_x = cx - l_out;
            pts.push(Point2D::new(CANVAS_WIDTH, in_y));
            let stop_idx = pts.len();
            pts.push(Point2D::new(cx + half_road, in_y));

            let p_start = Point2D::new(cx + half_road, in_y);
            let p_ctrl1 = Point2D::new(cx, in_y);
            let p_ctrl2 = Point2D::new(out_x, cy);
            let p_end = Point2D::new(out_x, cy + half_road);

            for i in 1..=num_arc_steps {
                let t = i as f32 / num_arc_steps as f32;
                let px = (1.0 - t).powi(3) * p_start.x
                    + 3.0 * (1.0 - t).powi(2) * t * p_ctrl1.x
                    + 3.0 * (1.0 - t) * t.powi(2) * p_ctrl2.x
                    + t.powi(3) * p_end.x;
                let py = (1.0 - t).powi(3) * p_start.y
                    + 3.0 * (1.0 - t).powi(2) * t * p_ctrl1.y
                    + 3.0 * (1.0 - t) * t.powi(2) * p_ctrl2.y
                    + t.powi(3) * p_end.y;
                pts.push(Point2D::new(px, py));
            }

            let exit_idx = pts.len() - 1;
            pts.push(Point2D::new(out_x, CANVAS_HEIGHT));
            (pts, stop_idx, exit_idx)
        }
    }
}
