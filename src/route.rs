use crate::types::{
    Origin, Point2D, Turn, CENTER_X, CENTER_Y,
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
            total += raw_points[i - 1].distance_to(&raw_points[i]);
            cumulative_dist.push(total);
        }
        let stop_line_distance = cumulative_dist[stop_idx.min(raw_points.len() - 1)];
        let exit_distance = cumulative_dist[exit_idx.min(raw_points.len() - 1)];
        let detection_distance = (stop_line_distance - DETECTION_DISTANCE_AHEAD).max(0.0);

        Self {
            origin, turn, waypoints: raw_points, cumulative_dist, total_distance: total,
            detection_distance, stop_line_distance, exit_distance,
        }
    }

    pub fn position_at(&self, d: f32) -> Point2D {
        if self.waypoints.is_empty() || d <= 0.0 {
            return self.waypoints.first().copied().unwrap_or(Point2D::ZERO);
        }
        if d >= self.total_distance {
            return *self.waypoints.last().unwrap();
        }
        match self.cumulative_dist.binary_search_by(|dist| dist.partial_cmp(&d).unwrap()) {
            Ok(idx) => self.waypoints[idx],
            Err(idx) => {
                let (d0, d1) = (self.cumulative_dist[idx - 1], self.cumulative_dist[idx]);
                let t = if (d1 - d0).abs() > 0.0001 { (d - d0) / (d1 - d0) } else { 0.0 };
                self.waypoints[idx - 1].lerp(&self.waypoints[idx], t)
            }
        }
    }

    pub fn heading_at_smooth(&self, d: f32, previous_heading: f32) -> f32 {
        let (p1, p2) = (self.position_at((d - 3.0).max(0.0)), self.position_at((d + 3.0).min(self.total_distance)));
        let (dx, dy) = (p2.x - p1.x, p2.y - p1.y);
        if dx * dx + dy * dy < 0.01 { previous_heading } else { dy.atan2(dx).to_degrees() }
    }

    pub fn heading_at(&self, d: f32) -> f32 {
        self.heading_at_smooth(d, self.origin.incoming_heading())
    }

    pub fn is_in_intersection(&self, d: f32) -> bool {
        d >= self.stop_line_distance && d <= self.exit_distance
    }

    pub fn is_detected(&self, d: f32) -> bool {
        d >= self.detection_distance
    }

    pub fn remaining_intersection_path(&self, current_d: f32) -> Vec<Point2D> {
        let (start, end) = (current_d.max(self.stop_line_distance), self.exit_distance);
        if start >= end { return Vec::new(); }
        let mut pts = Vec::new();
        let mut d = start;
        while d <= end {
            pts.push(self.position_at(d));
            d += 12.0;
        }
        pts.push(self.position_at(end));
        pts
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteConflict {
    pub dist_a: f32,
    pub dist_b: f32,
    pub min_dist: f32,
}

pub fn route_index(origin: Origin, turn: Turn) -> usize {
    let o = match origin { Origin::North => 0, Origin::South => 1, Origin::East => 2, Origin::West => 3 };
    let t = match turn { Turn::Right => 0, Turn::Straight => 1, Turn::Left => 2 };
    o * 3 + t
}

pub fn compute_route_conflict(route_a: &Route, route_b: &Route) -> Option<RouteConflict> {
    let opp = matches!((route_a.origin, route_b.origin),
        (Origin::North, Origin::South) | (Origin::South, Origin::North) |
        (Origin::East, Origin::West) | (Origin::West, Origin::East));
    if (route_a.origin == route_b.origin && route_a.turn == route_b.turn)
        || route_a.turn == Turn::Right || route_b.turn == Turn::Right
        || (opp && (route_a.turn == Turn::Straight && route_b.turn == Turn::Straight
            || route_a.turn == Turn::Left && route_b.turn == Turn::Left)) {
        return None;
    }

    let (mut min_dist, mut best_da, mut best_db, step) = (f32::MAX, route_a.stop_line_distance, route_b.stop_line_distance, 3.0);
    let mut da = route_a.stop_line_distance;
    while da <= route_a.exit_distance {
        let pa = route_a.position_at(da);
        let mut db = route_b.stop_line_distance;
        while db <= route_b.exit_distance {
            let d = pa.distance_to(&route_b.position_at(db));
            if d < min_dist { min_dist = d; best_da = da; best_db = db; }
            db += step;
        }
        da += step;
    }

    (min_dist < 34.0).then_some(RouteConflict { dist_a: best_da, dist_b: best_db, min_dist })
}

pub fn compute_conflict_matrix() -> [[Option<RouteConflict>; 12]; 12] {
    let routes: Vec<_> = [Origin::North, Origin::South, Origin::East, Origin::West]
        .into_iter()
        .flat_map(|o| [Turn::Right, Turn::Straight, Turn::Left].map(|t| Route::new(o, t)))
        .collect();
    let mut matrix = [[None; 12]; 12];
    for (i, ra) in routes.iter().enumerate() {
        for (j, rb) in routes.iter().enumerate() {
            if i != j { matrix[i][j] = compute_route_conflict(ra, rb); }
        }
    }
    matrix
}

pub fn get_route_conflict(route_a: &Route, route_b: &Route) -> Option<RouteConflict> {
    static CONFLICT_MATRIX: std::sync::OnceLock<[[Option<RouteConflict>; 12]; 12]> = std::sync::OnceLock::new();
    let matrix = CONFLICT_MATRIX.get_or_init(compute_conflict_matrix);
    matrix[route_index(route_a.origin, route_a.turn)][route_index(route_b.origin, route_b.turn)]
}

pub fn trajectories_intersect(route_a: &Route, dist_a: f32, route_b: &Route, dist_b: f32, safety_threshold: f32) -> bool {
    let (path_a, path_b) = (route_a.remaining_intersection_path(dist_a), route_b.remaining_intersection_path(dist_b));
    let threshold = safety_threshold.min(34.0);
    !path_a.is_empty() && !path_b.is_empty() && compute_route_conflict(route_a, route_b).is_some()
        && path_a.iter().any(|pa| path_b.iter().any(|pb| pa.distance_to(pb) < threshold))
}

fn generate_route_points(origin: Origin, turn: Turn) -> (Vec<Point2D>, usize, usize) {
    let (cx, cy, hr) = (CENTER_X, CENTER_Y, HALF_ROAD_WIDTH);
    let (l, s, r) = (0.5 * LANE_WIDTH, 1.5 * LANE_WIDTH, 2.5 * LANE_WIDTH);
    let mut pts = Vec::with_capacity(30);
    let n = 24;

    match turn {
        Turn::Straight => {
            pts.extend([(s, cy), (s, hr), (s, -hr), (s, -cy)].map(|(x, y)| Point2D::new(x, y)));
            finalize_route(pts, 1, 2, origin, cx, cy)
        }
        Turn::Right => {
            pts.extend([(r, cy), (r, hr)].map(|(x, y)| Point2D::new(x, y)));
            for i in 1..=n {
                let angle = std::f32::consts::PI + (i as f32 / n as f32) * std::f32::consts::FRAC_PI_2;
                pts.push(Point2D::new(hr + (hr - r) * angle.cos(), hr + (hr - r) * angle.sin()));
            }
            pts.push(Point2D::new(cx, r));
            let exit_idx = pts.len() - 1;
            finalize_route(pts, 1, exit_idx, origin, cx, cy)
        }
        Turn::Left => {
            pts.extend([(l, cy), (l, hr)].map(|(x, y)| Point2D::new(x, y)));
            let (p0, p1, p2, p3) = ((l, hr), (l, 0.0), (0.0, -l), (-hr, -l));
            for i in 1..=n {
                let t = i as f32 / n as f32;
                let (u, u2, u3, t2, t3) = (1.0 - t, (1.0 - t).powi(2), (1.0 - t).powi(3), t * t, t.powi(3));
                let bez = |a, b, c, d| u3 * a + 3.0 * u2 * t * b + 3.0 * u * t2 * c + t3 * d;
                pts.push(Point2D::new(bez(p0.0, p1.0, p2.0, p3.0), bez(p0.1, p1.1, p2.1, p3.1)));
            }
            pts.push(Point2D::new(-cx, -l));
            let exit_idx = pts.len() - 1;
            finalize_route(pts, 1, exit_idx, origin, cx, cy)
        }
    }
}

fn finalize_route(mut pts: Vec<Point2D>, stop_idx: usize, exit_idx: usize, origin: Origin, cx: f32, cy: f32) -> (Vec<Point2D>, usize, usize) {
    let k = match origin { Origin::South => 0, Origin::West => 1, Origin::North => 2, Origin::East => 3 };
    for p in &mut pts {
        let (mut x, mut y) = (p.x, p.y);
        for _ in 0..k {
            let tmp = x;
            x = -y;
            y = tmp;
        }
        *p = Point2D::new(cx + x, cy + y);
    }
    (pts, stop_idx, exit_idx)
}
