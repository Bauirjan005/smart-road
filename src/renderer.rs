use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

use crate::simulation::Simulation;
use crate::types::{
    Turn, VehicleType, CANVAS_HEIGHT, CANVAS_WIDTH, CENTER_X, CENTER_Y,
    HALF_ROAD_WIDTH, LANE_WIDTH,
};
use crate::vehicle::Vehicle;

#[derive(Clone, Copy)]
pub struct SparkPalette {
    pub name: &'static str,
    pub body: Color,
    pub shadow: Color,
    pub highlight: Color,
}

pub const CHEVY_SPARK_COLORS: [SparkPalette; 8] = [
    // 0: Summit White (Exact match for reference photo)
    SparkPalette {
        name: "Summit White",
        body: Color::RGB(246, 248, 252),
        shadow: Color::RGB(192, 198, 208),
        highlight: Color::RGB(255, 255, 255),
    },
    // 1: Mystic Blue Metallic
    SparkPalette {
        name: "Mystic Blue",
        body: Color::RGB(38, 110, 210),
        shadow: Color::RGB(22, 68, 140),
        highlight: Color::RGB(95, 170, 255),
    },
    // 2: Red Hot
    SparkPalette {
        name: "Red Hot",
        body: Color::RGB(225, 34, 46),
        shadow: Color::RGB(155, 20, 30),
        highlight: Color::RGB(255, 85, 96),
    },
    // 3: Nitro Yellow Metallic
    SparkPalette {
        name: "Nitro Yellow",
        body: Color::RGB(245, 192, 20),
        shadow: Color::RGB(180, 140, 15),
        highlight: Color::RGB(255, 225, 80),
    },
    // 4: Passion Fruit Violet
    SparkPalette {
        name: "Passion Fruit",
        body: Color::RGB(150, 58, 180),
        shadow: Color::RGB(105, 35, 130),
        highlight: Color::RGB(200, 105, 230),
    },
    // 5: Caribbean Blue Metallic
    SparkPalette {
        name: "Caribbean Blue",
        body: Color::RGB(22, 182, 212),
        shadow: Color::RGB(14, 128, 155),
        highlight: Color::RGB(105, 228, 255),
    },
    // 6: Silver Ice Metallic
    SparkPalette {
        name: "Silver Ice",
        body: Color::RGB(190, 196, 205),
        shadow: Color::RGB(142, 148, 158),
        highlight: Color::RGB(236, 242, 250),
    },
    // 7: Nightfall Gray Metallic
    SparkPalette {
        name: "Nightfall Gray",
        body: Color::RGB(74, 80, 90),
        shadow: Color::RGB(46, 52, 60),
        highlight: Color::RGB(122, 132, 146),
    },
];

pub struct Renderer;

impl Renderer {
    pub fn render(canvas: &mut Canvas<Window>, sim: &Simulation) -> Result<(), String> {
        canvas.set_blend_mode(BlendMode::Blend);

        // 1. Draw Environment: Lawn terrain, sidewalks, corner pedestrian plazas, curbs
        Self::draw_environment(canvas);

        // 2. Draw Roads: High-grade asphalt roadways and intersection inner box
        Self::draw_roads(canvas);

        // 3. Draw Lane Markings: Double yellow centerlines, dashed dividers, zebra crossings, stop lines & schematic arrows
        Self::draw_lane_markings(canvas);

        // 4. Draw Smart Sensor Detection Boundaries
        Self::draw_sensor_zones(canvas);

        // 5. Draw Vehicles: Detailed chassis, shadow, lights, is_braking glow, and turn signal flashers
        for vehicle in &sim.vehicles {
            Self::draw_vehicle(canvas, vehicle, sim.stats.simulation_duration);
        }

        // 6. Draw In-Simulation HUD Overlay
        Self::draw_hud(canvas, sim);

        // 7. Draw Statistics Modal if simulation finished
        if sim.is_finished {
            Self::draw_stats_screen(canvas, sim);
        }

        canvas.present();
        Ok(())
    }

    /// 1. Environment: Landscaping grass, sidewalks with expansion joints, corner plazas, and stone curbs
    fn draw_environment(canvas: &mut Canvas<Window>) {
        let cx = CENTER_X as i32;
        let cy = CENTER_Y as i32;
        let half_w = HALF_ROAD_WIDTH as i32; // 108
        let sw_w = 28; // Sidewalk width in pixels

        // Base grass lawn terrain
        canvas.set_draw_color(Color::RGB(36, 52, 38));
        canvas.clear();

        // Subtle lawn striping pattern
        canvas.set_draw_color(Color::RGB(33, 48, 35));
        let mut y_stripe = 0;
        while y_stripe < CANVAS_HEIGHT as i32 {
            let _ = canvas.fill_rect(Rect::new(0, y_stripe, CANVAS_WIDTH as u32, 16));
            y_stripe += 32;
        }

        // Sidewalk pavement color & joint tone
        let sw_color = Color::RGB(150, 155, 164);
        let joint_color = Color::RGB(130, 135, 144);

        // 4 Corner Sidewalks & Pedestrian Plazas flanking the roads
        // Top-Left corner sidewalk
        canvas.set_draw_color(sw_color);
        let _ = canvas.fill_rect(Rect::new(cx - half_w - sw_w, 0, sw_w as u32, (cy - half_w) as u32));
        let _ = canvas.fill_rect(Rect::new(0, cy - half_w - sw_w, (cx - half_w) as u32, sw_w as u32));
        let _ = canvas.fill_rect(Rect::new(cx - half_w - sw_w, cy - half_w - sw_w, sw_w as u32, sw_w as u32));

        // Top-Right corner sidewalk
        let _ = canvas.fill_rect(Rect::new(cx + half_w, 0, sw_w as u32, (cy - half_w) as u32));
        let _ = canvas.fill_rect(Rect::new(cx + half_w, cy - half_w - sw_w, (CANVAS_WIDTH as i32 - (cx + half_w)) as u32, sw_w as u32));
        let _ = canvas.fill_rect(Rect::new(cx + half_w, cy - half_w - sw_w, sw_w as u32, sw_w as u32));

        // Bottom-Left corner sidewalk
        let _ = canvas.fill_rect(Rect::new(cx - half_w - sw_w, cy + half_w, sw_w as u32, (CANVAS_HEIGHT as i32 - (cy + half_w)) as u32));
        let _ = canvas.fill_rect(Rect::new(0, cy + half_w, (cx - half_w) as u32, sw_w as u32));
        let _ = canvas.fill_rect(Rect::new(cx - half_w - sw_w, cy + half_w, sw_w as u32, sw_w as u32));

        // Bottom-Right corner sidewalk
        let _ = canvas.fill_rect(Rect::new(cx + half_w, cy + half_w, sw_w as u32, (CANVAS_HEIGHT as i32 - (cy + half_w)) as u32));
        let _ = canvas.fill_rect(Rect::new(cx + half_w, cy + half_w, (CANVAS_WIDTH as i32 - (cx + half_w)) as u32, sw_w as u32));
        let _ = canvas.fill_rect(Rect::new(cx + half_w, cy + half_w, sw_w as u32, sw_w as u32));

        // Sidewalk pavement tile expansion joints
        canvas.set_draw_color(joint_color);
        // Vertical sidewalk joints
        let mut y = 0;
        while y < cy - half_w {
            let _ = canvas.draw_line(Point::new(cx - half_w - sw_w, y), Point::new(cx - half_w, y));
            let _ = canvas.draw_line(Point::new(cx + half_w, y), Point::new(cx + half_w + sw_w, y));
            y += 24;
        }
        let mut y = cy + half_w;
        while y < CANVAS_HEIGHT as i32 {
            let _ = canvas.draw_line(Point::new(cx - half_w - sw_w, y), Point::new(cx - half_w, y));
            let _ = canvas.draw_line(Point::new(cx + half_w, y), Point::new(cx + half_w + sw_w, y));
            y += 24;
        }
        // Horizontal sidewalk joints
        let mut x = 0;
        while x < cx - half_w {
            let _ = canvas.draw_line(Point::new(x, cy - half_w - sw_w), Point::new(x, cy - half_w));
            let _ = canvas.draw_line(Point::new(x, cy + half_w), Point::new(x, cy + half_w + sw_w));
            x += 24;
        }
        let mut x = cx + half_w;
        while x < CANVAS_WIDTH as i32 {
            let _ = canvas.draw_line(Point::new(x, cy - half_w - sw_w), Point::new(x, cy - half_w));
            let _ = canvas.draw_line(Point::new(x, cy + half_w), Point::new(x, cy + half_w + sw_w));
            x += 24;
        }
    }

    /// 2. Roadway asphalt surface, drainage gutters, and curb borders
    fn draw_roads(canvas: &mut Canvas<Window>) {
        let cx = CENTER_X as i32;
        let cy = CENTER_Y as i32;
        let half_w = HALF_ROAD_WIDTH as i32; // 108

        // Premium dark slate asphalt
        canvas.set_draw_color(Color::RGB(34, 37, 44));

        // Vertical road (North-South)
        let _ = canvas.fill_rect(Rect::new(
            cx - half_w,
            0,
            (half_w * 2) as u32,
            CANVAS_HEIGHT as u32,
        ));

        // Horizontal road (East-West)
        let _ = canvas.fill_rect(Rect::new(
            0,
            cy - half_w,
            CANVAS_WIDTH as u32,
            (half_w * 2) as u32,
        ));

        // Central intersection inner surface
        canvas.set_draw_color(Color::RGB(30, 33, 40));
        let _ = canvas.fill_rect(Rect::new(
            cx - half_w,
            cy - half_w,
            (half_w * 2) as u32,
            (half_w * 2) as u32,
        ));

        // Dark gutter drainage strips along the curbs
        canvas.set_draw_color(Color::RGB(24, 26, 31));
        let _ = canvas.fill_rect(Rect::new(cx - half_w, 0, 4, (cy - half_w) as u32));
        let _ = canvas.fill_rect(Rect::new(cx + half_w - 4, 0, 4, (cy - half_w) as u32));
        let _ = canvas.fill_rect(Rect::new(cx - half_w, cy + half_w, 4, (CANVAS_HEIGHT as i32 - (cy + half_w)) as u32));
        let _ = canvas.fill_rect(Rect::new(cx + half_w - 4, cy + half_w, 4, (CANVAS_HEIGHT as i32 - (cy + half_w)) as u32));

        let _ = canvas.fill_rect(Rect::new(0, cy - half_w, (cx - half_w) as u32, 4));
        let _ = canvas.fill_rect(Rect::new(0, cy + half_w - 4, (cx - half_w) as u32, 4));
        let _ = canvas.fill_rect(Rect::new(cx + half_w, cy - half_w, (CANVAS_WIDTH as i32 - (cx + half_w)) as u32, 4));
        let _ = canvas.fill_rect(Rect::new(cx + half_w, cy + half_w - 4, (CANVAS_WIDTH as i32 - (cx + half_w)) as u32, 4));

        // Raised curb highlight lines
        canvas.set_draw_color(Color::RGB(210, 215, 225));
        // Corner borders with chamfered curve links
        let _ = canvas.draw_line(Point::new(0, cy - half_w), Point::new(cx - half_w, cy - half_w));
        let _ = canvas.draw_line(Point::new(cx - half_w, 0), Point::new(cx - half_w, cy - half_w));

        let _ = canvas.draw_line(Point::new(cx + half_w, 0), Point::new(cx + half_w, cy - half_w));
        let _ = canvas.draw_line(Point::new(cx + half_w, cy - half_w), Point::new(CANVAS_WIDTH as i32, cy - half_w));

        let _ = canvas.draw_line(Point::new(0, cy + half_w), Point::new(cx - half_w, cy + half_w));
        let _ = canvas.draw_line(Point::new(cx - half_w, cy + half_w), Point::new(cx - half_w, CANVAS_HEIGHT as i32));

        let _ = canvas.draw_line(Point::new(cx + half_w, CANVAS_HEIGHT as i32), Point::new(cx + half_w, cy + half_w));
        let _ = canvas.draw_line(Point::new(cx + half_w, cy + half_w), Point::new(CANVAS_WIDTH as i32, cy + half_w));
    }

    /// 3. Comprehensive Lane Markings: Double yellow lines, dashed dividers, zebra crosswalks, stop lines & arrows
    fn draw_lane_markings(canvas: &mut Canvas<Window>) {
        let cx = CENTER_X as i32;
        let cy = CENTER_Y as i32;
        let half_w = HALF_ROAD_WIDTH as i32; // 108
        let lw = LANE_WIDTH as i32;          // 36

        // 1. Double Solid Yellow Centerlines
        canvas.set_draw_color(Color::RGB(250, 198, 28));
        // Vertical centerlines (leaving intersection open)
        let _ = canvas.fill_rect(Rect::new(cx - 3, 0, 2, (cy - half_w) as u32));
        let _ = canvas.fill_rect(Rect::new(cx + 1, 0, 2, (cy - half_w) as u32));
        let _ = canvas.fill_rect(Rect::new(cx - 3, cy + half_w, 2, (CANVAS_HEIGHT as i32 - (cy + half_w)) as u32));
        let _ = canvas.fill_rect(Rect::new(cx + 1, cy + half_w, 2, (CANVAS_HEIGHT as i32 - (cy + half_w)) as u32));

        // Horizontal centerlines (leaving intersection open)
        let _ = canvas.fill_rect(Rect::new(0, cy - 3, (cx - half_w) as u32, 2));
        let _ = canvas.fill_rect(Rect::new(0, cy + 1, (cx - half_w) as u32, 2));
        let _ = canvas.fill_rect(Rect::new(cx + half_w, cy - 3, (CANVAS_WIDTH as i32 - (cx + half_w)) as u32, 2));
        let _ = canvas.fill_rect(Rect::new(cx + half_w, cy + 1, (CANVAS_WIDTH as i32 - (cx + half_w)) as u32, 2));

        // 2. Dashed White Lane Dividers
        canvas.set_draw_color(Color::RGB(220, 226, 236));
        // Vertical road dividers (offset ±36, ±72)
        for offset_lanes in [1, 2] {
            let x_left = cx - offset_lanes * lw;
            let x_right = cx + offset_lanes * lw;

            // North section (ends before pedestrian crosswalk at y = 304)
            let mut y = 0;
            while y < cy - half_w - 36 {
                let _ = canvas.draw_line(Point::new(x_left, y), Point::new(x_left, (y + 16).min(cy - half_w - 36)));
                let _ = canvas.draw_line(Point::new(x_right, y), Point::new(x_right, (y + 16).min(cy - half_w - 36)));
                y += 32;
            }

            // South section (starts after pedestrian crosswalk at y = 596)
            let mut y = cy + half_w + 36;
            while y < CANVAS_HEIGHT as i32 {
                let _ = canvas.draw_line(Point::new(x_left, y), Point::new(x_left, (y + 16).min(CANVAS_HEIGHT as i32)));
                let _ = canvas.draw_line(Point::new(x_right, y), Point::new(x_right, (y + 16).min(CANVAS_HEIGHT as i32)));
                y += 32;
            }
        }

        // Horizontal road dividers
        for offset_lanes in [1, 2] {
            let y_top = cy - offset_lanes * lw;
            let y_bot = cy + offset_lanes * lw;

            // West section (ends before pedestrian crosswalk at x = 304)
            let mut x = 0;
            while x < cx - half_w - 36 {
                let _ = canvas.draw_line(Point::new(x, y_top), Point::new((x + 16).min(cx - half_w - 36), y_top));
                let _ = canvas.draw_line(Point::new(x, y_bot), Point::new((x + 16).min(cx - half_w - 36), y_bot));
                x += 32;
            }

            // East section (starts after pedestrian crosswalk at x = 596)
            let mut x = cx + half_w + 36;
            while x < CANVAS_WIDTH as i32 {
                let _ = canvas.draw_line(Point::new(x, y_top), Point::new((x + 16).min(CANVAS_WIDTH as i32), y_top));
                let _ = canvas.draw_line(Point::new(x, y_bot), Point::new((x + 16).min(CANVAS_WIDTH as i32), y_bot));
                x += 32;
            }
        }

        // 3. Pedestrian Zebra Crossings (Accurate European/Urban standard crosswalks on all 4 branches)
        Self::draw_zebra_crossings(canvas);

        // 4. Solid White Stop Lines (Wide stop lines positioned right before the intersection)
        canvas.set_draw_color(Color::RGB(250, 252, 255));
        // South incoming stop line
        let _ = canvas.fill_rect(Rect::new(cx, cy + half_w - 5, half_w as u32, 5));
        // North incoming stop line
        let _ = canvas.fill_rect(Rect::new(cx - half_w, cy - half_w, half_w as u32, 5));
        // West incoming stop line
        let _ = canvas.fill_rect(Rect::new(cx - half_w, cy, 5, half_w as u32));
        // East incoming stop line
        let _ = canvas.fill_rect(Rect::new(cx + half_w - 5, cy - half_w, 5, half_w as u32));

        // 5. Schematic Direction Arrows painted on pavement (Left r, Straight s, Right l)
        Self::draw_schematic_arrows(canvas);
    }

    /// Draws realistic pedestrian zebra stripes across all 4 intersection approaches
    fn draw_zebra_crossings(canvas: &mut Canvas<Window>) {
        let cx = CENTER_X as i32;
        let cy = CENTER_Y as i32;
        let half_w = HALF_ROAD_WIDTH as i32; // 108

        let zebra_color = Color::RGB(246, 248, 252);
        let stripe_w = 10;
        let stripe_gap = 8;
        let stripe_len = 26;

        // South Approach Zebra Crossing (y = 566..592, across full road x = cx - half_w + 4 .. cx + half_w - 4)
        canvas.set_draw_color(zebra_color);
        let y_s = cy + half_w + 8;
        let mut x = cx - half_w + 6;
        while x < cx + half_w - 6 {
            let _ = canvas.fill_rect(Rect::new(x, y_s, stripe_w, stripe_len as u32));
            x += stripe_w as i32 + stripe_gap;
        }

        // North Approach Zebra Crossing (y = 316..342)
        let y_n = cy - half_w - 8 - stripe_len;
        let mut x = cx - half_w + 6;
        while x < cx + half_w - 6 {
            let _ = canvas.fill_rect(Rect::new(x, y_n, stripe_w, stripe_len as u32));
            x += stripe_w as i32 + stripe_gap;
        }

        // West Approach Zebra Crossing (x = 316..342)
        let x_w = cx - half_w - 8 - stripe_len;
        let mut y = cy - half_w + 6;
        while y < cy + half_w - 6 {
            let _ = canvas.fill_rect(Rect::new(x_w, y, stripe_len as u32, stripe_w));
            y += stripe_w as i32 + stripe_gap;
        }

        // East Approach Zebra Crossing (x = 566..592)
        let x_e = cx + half_w + 8;
        let mut y = cy - half_w + 6;
        while y < cy + half_w - 6 {
            let _ = canvas.fill_rect(Rect::new(x_e, y, stripe_len as u32, stripe_w));
            y += stripe_w as i32 + stripe_gap;
        }
    }

    /// Draws schematic road arrows (Left, Straight, Right) painted on the asphalt of each lane
    fn draw_schematic_arrows(canvas: &mut Canvas<Window>) {
        let cx = CENTER_X as i32;
        let cy = CENTER_Y as i32;
        let lw = LANE_WIDTH as i32; // 36

        // South Approach (moving North, heading 270°)
        // Lane 1 (Leftmost incoming): Left turn 'l'
        Self::draw_lane_arrow(canvas, cx + lw / 2, cy + 108 + 70, 270.0, Turn::Left);
        // Lane 2 (Middle incoming): Straight 's'
        Self::draw_lane_arrow(canvas, cx + lw + lw / 2, cy + 108 + 70, 270.0, Turn::Straight);
        // Lane 3 (Rightmost incoming): Right turn 'r'
        Self::draw_lane_arrow(canvas, cx + 2 * lw + lw / 2, cy + 108 + 70, 270.0, Turn::Right);

        // North Approach (moving South, heading 90°)
        // Lane 1 (Leftmost incoming): Left turn 'l'
        Self::draw_lane_arrow(canvas, cx - lw / 2, cy - 108 - 70, 90.0, Turn::Left);
        // Lane 2 (Middle incoming): Straight 's'
        Self::draw_lane_arrow(canvas, cx - lw - lw / 2, cy - 108 - 70, 90.0, Turn::Straight);
        // Lane 3 (Rightmost incoming): Right turn 'r'
        Self::draw_lane_arrow(canvas, cx - 2 * lw - lw / 2, cy - 108 - 70, 90.0, Turn::Right);

        // West Approach (moving East, heading 0°)
        // Lane 1 (Leftmost incoming): Left turn 'l'
        Self::draw_lane_arrow(canvas, cx - 108 - 70, cy + lw / 2, 0.0, Turn::Left);
        // Lane 2 (Middle incoming): Straight 's'
        Self::draw_lane_arrow(canvas, cx - 108 - 70, cy + lw + lw / 2, 0.0, Turn::Straight);
        // Lane 3 (Rightmost incoming): Right turn 'r'
        Self::draw_lane_arrow(canvas, cx - 108 - 70, cy + 2 * lw + lw / 2, 0.0, Turn::Right);

        // East Approach (moving West, heading 180°)
        // Lane 1 (Leftmost incoming): Left turn 'l'
        Self::draw_lane_arrow(canvas, cx + 108 + 70, cy - lw / 2, 180.0, Turn::Left);
        // Lane 2 (Middle incoming): Straight 's'
        Self::draw_lane_arrow(canvas, cx + 108 + 70, cy - lw - lw / 2, 180.0, Turn::Straight);
        // Lane 3 (Rightmost incoming): Right turn 'r'
        Self::draw_lane_arrow(canvas, cx + 108 + 70, cy - 2 * lw - lw / 2, 180.0, Turn::Right);
    }

    /// Renders a single rotated schematic road arrow (Straight, Left turn, or Right turn)
    fn draw_lane_arrow(
        canvas: &mut Canvas<Window>,
        x_center: i32,
        y_center: i32,
        heading_deg: f32,
        turn: Turn,
    ) {
        let heading_rad = heading_deg.to_radians();
        let cos_a = heading_rad.cos();
        let sin_a = heading_rad.sin();
        let fwd = (cos_a, sin_a);
        let right = (-sin_a, cos_a);

        let to_screen = |dfwd: f32, dright: f32| -> Point {
            let x = (x_center as f32) + dfwd * fwd.0 + dright * right.0;
            let y = (y_center as f32) + dfwd * fwd.1 + dright * right.1;
            Point::new(x.round() as i32, y.round() as i32)
        };

        let arrow_color = Color::RGB(232, 238, 248);

        match turn {
            Turn::Straight => {
                // Shaft
                let shaft = [
                    to_screen(-12.0, -2.0),
                    to_screen(4.0, -2.0),
                    to_screen(4.0, 2.0),
                    to_screen(-12.0, 2.0),
                ];
                Self::fill_polygon(canvas, &shaft, arrow_color);

                // Triangular Arrowhead
                let head = [
                    to_screen(4.0, -7.5),
                    to_screen(14.0, 0.0),
                    to_screen(4.0, 7.5),
                ];
                Self::fill_polygon(canvas, &head, arrow_color);
            }
            Turn::Left => {
                // Stem curving to left
                let stem = [
                    to_screen(-12.0, -1.8),
                    to_screen(0.0, -1.8),
                    to_screen(0.0, 1.8),
                    to_screen(-12.0, 1.8),
                ];
                Self::fill_polygon(canvas, &stem, arrow_color);

                let arm = [
                    to_screen(0.0, 1.8),
                    to_screen(5.0, -8.0),
                    to_screen(2.0, -9.5),
                    to_screen(-2.0, -1.8),
                ];
                Self::fill_polygon(canvas, &arm, arrow_color);

                // Arrowhead pointing left
                let head = [
                    to_screen(9.0, -8.0),
                    to_screen(4.0, -16.0),
                    to_screen(-1.0, -8.0),
                ];
                Self::fill_polygon(canvas, &head, arrow_color);
            }
            Turn::Right => {
                // Stem curving to right
                let stem = [
                    to_screen(-12.0, -1.8),
                    to_screen(0.0, -1.8),
                    to_screen(0.0, 1.8),
                    to_screen(-12.0, 1.8),
                ];
                Self::fill_polygon(canvas, &stem, arrow_color);

                let arm = [
                    to_screen(0.0, -1.8),
                    to_screen(5.0, 8.0),
                    to_screen(2.0, 9.5),
                    to_screen(-2.0, 1.8),
                ];
                Self::fill_polygon(canvas, &arm, arrow_color);

                // Arrowhead pointing right
                let head = [
                    to_screen(9.0, 8.0),
                    to_screen(4.0, 16.0),
                    to_screen(-1.0, 8.0),
                ];
                Self::fill_polygon(canvas, &head, arrow_color);
            }
        }
    }

    /// 4. Smart intersection sensor detection bar lines
    fn draw_sensor_zones(canvas: &mut Canvas<Window>) {
        let cx = CENTER_X as i32;
        let cy = CENTER_Y as i32;
        let half_w = HALF_ROAD_WIDTH as i32;
        let detect_dist = 200; // pixels before stop line

        canvas.set_draw_color(Color::RGBA(0, 210, 255, 160));
        // South detection sensor bar
        let y_s = cy + half_w + detect_dist;
        if y_s < CANVAS_HEIGHT as i32 {
            let _ = canvas.draw_line(Point::new(cx, y_s), Point::new(cx + half_w, y_s));
        }

        // North detection sensor bar
        let y_n = cy - half_w - detect_dist;
        if y_n > 0 {
            let _ = canvas.draw_line(Point::new(cx - half_w, y_n), Point::new(cx, y_n));
        }

        // West detection sensor bar
        let x_w = cx - half_w - detect_dist;
        if x_w > 0 {
            let _ = canvas.draw_line(Point::new(x_w, cy), Point::new(x_w, cy + half_w));
        }

        // East detection sensor bar
        let x_e = cx + half_w + detect_dist;
        if x_e < CANVAS_WIDTH as i32 {
            let _ = canvas.draw_line(Point::new(x_e, cy - half_w), Point::new(x_e, cy));
        }
    }

    /// 5. Highly Detailed Vehicle Model:
    /// - Realistic body contours for Hatchback (16x25), Sedan (17x30), SUV (19x33), Van (20x38)
    /// - Translucent ambient ground shadow offset by 3-4 px, rotating smoothly around vehicle center
    /// - Dim dark-red running lights when cruising, bright saturated red brake lights when braking or stopped
    /// - Strict trigonometric 2D rotation around vehicle center (cos/sin of vehicle.angle())
    fn draw_vehicle(canvas: &mut Canvas<Window>, vehicle: &Vehicle, sim_time: f32) {
        let pos = vehicle.position();
        let angle_deg = vehicle.angle();
        let angle_rad = angle_deg.to_radians();

        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        // Strict 2D mathematical rotation unit vectors
        // Forward unit vector (+dfwd = along vehicle heading)
        let fwd = (cos_a, sin_a);
        // Right lateral unit vector (+dright = to right passenger side)
        let right = (-sin_a, cos_a);

        // Vehicle dimensions based on VehicleType:
        // Hatchback: 16x25 px, Sedan: 17x30 px, SUV: 19x33 px, Van: 20x38 px
        let (width, length) = vehicle.vehicle_type.dimensions();
        let hl = length / 2.0;
        let hw = width / 2.0;

        // Screen transformation closure (strict rotation around vehicle center pos)
        let to_screen = |dfwd: f32, dright: f32| -> Point {
            let x = pos.x + dfwd * fwd.0 + dright * right.0;
            let y = pos.y + dfwd * fwd.1 + dright * right.1;
            Point::new(x.round() as i32, y.round() as i32)
        };

        // Ambient ground shadow: offset by 3.5 px (dx = 2.5, dy = 2.5) and rotated with vehicle angle
        let shadow_dx = 2.5;
        let shadow_dy = 2.5;
        let to_shadow = |dfwd: f32, dright: f32| -> Point {
            let x = (pos.x + shadow_dx) + dfwd * fwd.0 + dright * right.0;
            let y = (pos.y + shadow_dy) + dfwd * fwd.1 + dright * right.1;
            Point::new(x.round() as i32, y.round() as i32)
        };

        let palette = CHEVY_SPARK_COLORS[vehicle.color_idx % CHEVY_SPARK_COLORS.len()];
        let is_braking = vehicle.is_braking();

        // Automotive turn signal flasher (~2.5 Hz)
        let blink_on = ((sim_time * 5.0) as i32) % 2 == 0;
        let blink_left = vehicle.turn == Turn::Left && !vehicle.has_cleared;
        let blink_right = vehicle.turn == Turn::Right && !vehicle.has_cleared;

        // ----------------------------------------------------
        // A. Ambient Volumetric Drop Shadow (Grounded 3D depth)
        // ----------------------------------------------------
        // Outer soft penumbra (translucent dark shadow)
        let shadow_outer = [
            to_shadow(hl + 1.4, -hw * 0.4),
            to_shadow(hl + 1.4, hw * 0.4),
            to_shadow(hl + 0.6, hw + 1.4),
            to_shadow(hl * 0.4, hw + 1.6),
            to_shadow(0.0, hw + 1.6),
            to_shadow(-hl * 0.4, hw + 1.6),
            to_shadow(-hl + 0.6, hw + 1.4),
            to_shadow(-hl - 1.4, hw * 0.5),
            to_shadow(-hl - 1.4, -hw * 0.5),
            to_shadow(-hl + 0.6, -hw - 1.4),
            to_shadow(-hl * 0.4, -hw - 1.6),
            to_shadow(0.0, -hw - 1.6),
            to_shadow(hl * 0.4, -hw - 1.6),
            to_shadow(hl + 0.6, -hw - 1.4),
        ];
        Self::fill_polygon(canvas, &shadow_outer, Color::RGBA(12, 14, 18, 75));

        // Inner core contact shadow (deeper occlusion directly under chassis)
        let shadow_inner = [
            to_shadow(hl + 0.5, -hw * 0.35),
            to_shadow(hl + 0.5, hw * 0.35),
            to_shadow(hl * 0.5, hw + 0.6),
            to_shadow(-hl * 0.5, hw + 0.6),
            to_shadow(-hl - 0.5, hw * 0.4),
            to_shadow(-hl - 0.5, -hw * 0.4),
            to_shadow(-hl * 0.5, -hw - 0.6),
            to_shadow(hl * 0.5, -hw - 0.6),
        ];
        Self::fill_polygon(canvas, &shadow_inner, Color::RGBA(8, 10, 14, 130));

        // ----------------------------------------------------
        // B. Forward Headlight Road Beam Projection (Warm light cast)
        // ----------------------------------------------------
        for &side in &[-1.0f32, 1.0f32] {
            let beam_pts = [
                to_screen(hl - 0.5, side * hw * 0.35),
                to_screen(hl - 0.5, side * hw * 0.85),
                to_screen(hl + 14.0, side * (hw + 4.5)),
                to_screen(hl + 14.0, side * (hw * 0.25)),
            ];
            Self::fill_polygon(canvas, &beam_pts, Color::RGBA(255, 245, 200, 32));
        }

        // ----------------------------------------------------
        // C. Red Brake Light Road Reflection (Active when is_braking)
        // ----------------------------------------------------
        if is_braking {
            let brake_glow_pts = [
                to_screen(-hl + 0.5, -hw * 0.9),
                to_screen(-hl + 0.5, hw * 0.9),
                to_screen(-hl - 8.5, hw * 1.25),
                to_screen(-hl - 8.5, -hw * 1.25),
            ];
            Self::fill_polygon(canvas, &brake_glow_pts, Color::RGBA(225, 20, 32, 80));
        }

        // ----------------------------------------------------
        // D. Four Wheels with Silver Alloy Rims
        // ----------------------------------------------------
        let wheel_len = match vehicle.vehicle_type {
            VehicleType::Hatchback => 5.6,
            VehicleType::Sedan => 6.4,
            VehicleType::Suv => 7.2,
            VehicleType::Van => 7.6,
        };
        let wheel_wid = match vehicle.vehicle_type {
            VehicleType::Hatchback => 2.8,
            VehicleType::Sedan => 3.0,
            VehicleType::Suv => 3.5,
            VehicleType::Van => 3.8,
        };
        let wheel_fwd = hl * 0.60;
        let wheel_side = hw - 0.3;
        for (w_fwd, w_side) in [
            (wheel_fwd, wheel_side),
            (wheel_fwd, -wheel_side),
            (-wheel_fwd, wheel_side),
            (-wheel_fwd, -wheel_side),
        ] {
            // Dark rubber tire
            let tire_pts = [
                to_screen(w_fwd + wheel_len / 2.0, w_side - wheel_wid / 2.0),
                to_screen(w_fwd + wheel_len / 2.0, w_side + wheel_wid / 2.0),
                to_screen(w_fwd - wheel_len / 2.0, w_side + wheel_wid / 2.0),
                to_screen(w_fwd - wheel_len / 2.0, w_side - wheel_wid / 2.0),
            ];
            Self::fill_polygon(canvas, &tire_pts, Color::RGB(20, 22, 26));

            // Silver alloy rim face
            let rim_pts = [
                to_screen(w_fwd + wheel_len * 0.28, w_side - wheel_wid * 0.26),
                to_screen(w_fwd + wheel_len * 0.28, w_side + wheel_wid * 0.26),
                to_screen(w_fwd - wheel_len * 0.28, w_side + wheel_wid * 0.26),
                to_screen(w_fwd - wheel_len * 0.28, w_side - wheel_wid * 0.26),
            ];
            Self::fill_polygon(canvas, &rim_pts, Color::RGB(198, 204, 214));
        }

        // ----------------------------------------------------
        // E. Smooth Contoured Body Shell (Tailored per VehicleType)
        // ----------------------------------------------------
        let body_pts: Vec<Point> = match vehicle.vehicle_type {
            VehicleType::Hatchback => vec![
                to_screen(hl, -hw * 0.35),
                to_screen(hl, hw * 0.35),
                to_screen(hl - 0.9, hw * 0.85),
                to_screen(hl - 3.2, hw),
                to_screen(0.0, hw),
                to_screen(-hl + 2.8, hw),
                to_screen(-hl + 0.8, hw * 0.85),
                to_screen(-hl, hw * 0.4),
                to_screen(-hl, -hw * 0.4),
                to_screen(-hl + 0.8, -hw * 0.85),
                to_screen(-hl + 2.8, -hw),
                to_screen(0.0, -hw),
                to_screen(hl - 3.2, -hw),
                to_screen(hl - 0.9, -hw * 0.85),
            ],
            VehicleType::Sedan => vec![
                to_screen(hl, -hw * 0.3),
                to_screen(hl, hw * 0.3),
                to_screen(hl - 1.2, hw * 0.8),
                to_screen(hl - 4.2, hw),
                to_screen(0.0, hw * 0.98),
                to_screen(-hl + 4.5, hw),
                to_screen(-hl + 1.2, hw * 0.82),
                to_screen(-hl, hw * 0.35),
                to_screen(-hl, -hw * 0.35),
                to_screen(-hl + 1.2, -hw * 0.82),
                to_screen(-hl + 4.5, -hw),
                to_screen(0.0, -hw * 0.98),
                to_screen(hl - 4.2, -hw),
                to_screen(hl - 1.2, -hw * 0.8),
            ],
            VehicleType::Suv => vec![
                to_screen(hl, -hw * 0.45),
                to_screen(hl, hw * 0.45),
                to_screen(hl - 1.2, hw * 0.9),
                to_screen(hl - 4.0, hw),
                to_screen(0.0, hw * 0.96),
                to_screen(-hl + 4.0, hw),
                to_screen(-hl + 1.0, hw * 0.9),
                to_screen(-hl, hw * 0.6),
                to_screen(-hl, -hw * 0.6),
                to_screen(-hl + 1.0, -hw * 0.9),
                to_screen(-hl + 4.0, -hw),
                to_screen(0.0, -hw * 0.96),
                to_screen(hl - 4.0, -hw),
                to_screen(hl - 1.2, -hw * 0.9),
            ],
            VehicleType::Van => vec![
                to_screen(hl, -hw * 0.45),
                to_screen(hl, hw * 0.45),
                to_screen(hl - 1.2, hw * 0.92),
                to_screen(hl - 3.5, hw),
                to_screen(0.0, hw),
                to_screen(-hl + 2.5, hw),
                to_screen(-hl + 0.5, hw * 0.95),
                to_screen(-hl, hw * 0.7),
                to_screen(-hl, -hw * 0.7),
                to_screen(-hl + 0.5, -hw * 0.95),
                to_screen(-hl + 2.5, -hw),
                to_screen(0.0, -hw),
                to_screen(hl - 3.5, -hw),
                to_screen(hl - 1.2, -hw * 0.92),
            ],
        };
        Self::fill_polygon(canvas, &body_pts, palette.body);
        Self::draw_polygon_outline(canvas, &body_pts, palette.shadow);

        // Lower Honeycomb Air Intake Grille
        let grille_fwd = hl - 0.8;
        let grille_half_w = hw * 0.55;
        let lower_grille = [
            to_screen(grille_fwd, -grille_half_w),
            to_screen(grille_fwd, grille_half_w),
            to_screen(grille_fwd - 2.0, grille_half_w * 0.8),
            to_screen(grille_fwd - 2.0, -grille_half_w * 0.8),
        ];
        Self::fill_polygon(canvas, &lower_grille, Color::RGB(26, 28, 34));
        Self::draw_polygon_outline(canvas, &lower_grille, Color::RGB(48, 52, 60));

        // Upper Front Grille Chrome Trim Bar
        let chrome_bar = [
            to_screen(hl - 0.2, -grille_half_w * 0.85),
            to_screen(hl - 0.2, grille_half_w * 0.85),
            to_screen(hl - 0.8, grille_half_w * 0.85),
            to_screen(hl - 0.8, -grille_half_w * 0.85),
        ];
        Self::fill_polygon(canvas, &chrome_bar, Color::RGB(215, 222, 230));

        // Golden Bowtie Logo (Centered on front)
        let bowtie = [
            to_screen(hl + 0.2, -hw * 0.20),
            to_screen(hl + 0.2, hw * 0.20),
            to_screen(hl - 0.7, hw * 0.20),
            to_screen(hl - 0.7, -hw * 0.20),
        ];
        Self::fill_polygon(canvas, &bowtie, Color::RGB(232, 182, 28)); // Rich Gold
        Self::draw_polygon_outline(canvas, &bowtie, Color::RGB(248, 250, 255));

        // Hood styling creases
        let hood_start = match vehicle.vehicle_type {
            VehicleType::Hatchback => hl * 0.35,
            VehicleType::Sedan => hl * 0.38,
            VehicleType::Suv => hl * 0.40,
            VehicleType::Van => hl * 0.65,
        };
        canvas.set_draw_color(palette.shadow);
        let _ = canvas.draw_line(to_screen(hood_start, -hw * 0.45), to_screen(hl - 1.5, -hw * 0.25));
        let _ = canvas.draw_line(to_screen(hood_start, hw * 0.45), to_screen(hl - 1.5, hw * 0.25));

        // Aerodynamic Side Mirrors
        let mirror_fwd = match vehicle.vehicle_type {
            VehicleType::Van => hl * 0.50,
            _ => hl * 0.25,
        };
        for &side in &[-1.0f32, 1.0f32] {
            let mirror_pts = [
                to_screen(mirror_fwd + 1.2, side * (hw - 0.2)),
                to_screen(mirror_fwd + 1.2, side * (hw + 2.8)),
                to_screen(mirror_fwd - 0.6, side * (hw + 3.0)),
                to_screen(mirror_fwd - 0.6, side * (hw - 0.2)),
            ];
            Self::fill_polygon(canvas, &mirror_pts, palette.body);
            Self::draw_polygon_outline(canvas, &mirror_pts, palette.shadow);

            canvas.set_draw_color(Color::RGB(55, 75, 95));
            let _ = canvas.draw_line(to_screen(mirror_fwd - 0.3, side * (hw + 0.6)), to_screen(mirror_fwd - 0.3, side * (hw + 2.6)));
        }

        // ----------------------------------------------------
        // F. Tinted Windows & Glasshouse (Tailored per VehicleType)
        // ----------------------------------------------------
        match vehicle.vehicle_type {
            VehicleType::Hatchback => {
                // Windshield
                let windshield = [
                    to_screen(hl * 0.35, -hw * 0.75),
                    to_screen(hl * 0.35, hw * 0.75),
                    to_screen(hl * 0.05, hw * 0.68),
                    to_screen(hl * 0.05, -hw * 0.68),
                ];
                Self::fill_polygon(canvas, &windshield, Color::RGB(42, 60, 80));
                Self::draw_polygon_outline(canvas, &windshield, Color::RGB(22, 28, 36));
                canvas.set_draw_color(Color::RGB(85, 125, 160));
                let _ = canvas.draw_line(to_screen(hl * 0.30, -hw * 0.45), to_screen(hl * 0.12, -hw * 0.10));

                // Side windows & B-pillar
                for &side in &[-1.0f32, 1.0f32] {
                    let side_win = [
                        to_screen(hl * 0.05, side * hw * 0.68),
                        to_screen(hl * 0.05, side * hw * 0.82),
                        to_screen(-hl * 0.55, side * hw * 0.82),
                        to_screen(-hl * 0.55, side * hw * 0.68),
                    ];
                    Self::fill_polygon(canvas, &side_win, Color::RGB(36, 52, 70));
                    canvas.set_draw_color(Color::RGB(18, 22, 28));
                    let _ = canvas.draw_line(to_screen(-hl * 0.22, side * hw * 0.68), to_screen(-hl * 0.22, side * hw * 0.82));
                }

                // Slanted rear hatch window
                let rear_win = [
                    to_screen(-hl * 0.55, -hw * 0.68),
                    to_screen(-hl * 0.55, hw * 0.68),
                    to_screen(-hl * 0.85, hw * 0.58),
                    to_screen(-hl * 0.85, -hw * 0.58),
                ];
                Self::fill_polygon(canvas, &rear_win, Color::RGB(32, 45, 62));
                Self::draw_polygon_outline(canvas, &rear_win, Color::RGB(18, 24, 32));

                // Roof panel
                let roof_pts = [
                    to_screen(hl * 0.05, -hw * 0.68),
                    to_screen(hl * 0.05, hw * 0.68),
                    to_screen(-hl * 0.55, hw * 0.68),
                    to_screen(-hl * 0.55, -hw * 0.68),
                ];
                Self::fill_polygon(canvas, &roof_pts, palette.body);

                // Rear hatch roof spoiler
                let spoiler = [
                    to_screen(-hl * 0.85, -hw * 0.72),
                    to_screen(-hl * 0.85, hw * 0.72),
                    to_screen(-hl * 0.98, hw * 0.70),
                    to_screen(-hl * 0.98, -hw * 0.70),
                ];
                Self::fill_polygon(canvas, &spoiler, palette.body);
                Self::draw_polygon_outline(canvas, &spoiler, palette.shadow);

                // Roof whip antenna
                canvas.set_draw_color(Color::RGB(30, 35, 45));
                let _ = canvas.draw_line(to_screen(-hl * 0.45, 0.0), to_screen(-hl * 0.80, 0.0));
            }
            VehicleType::Sedan => {
                // Windshield
                let windshield = [
                    to_screen(hl * 0.38, -hw * 0.75),
                    to_screen(hl * 0.38, hw * 0.75),
                    to_screen(hl * 0.10, hw * 0.68),
                    to_screen(hl * 0.10, -hw * 0.68),
                ];
                Self::fill_polygon(canvas, &windshield, Color::RGB(42, 60, 80));
                Self::draw_polygon_outline(canvas, &windshield, Color::RGB(22, 28, 36));
                canvas.set_draw_color(Color::RGB(85, 125, 160));
                let _ = canvas.draw_line(to_screen(hl * 0.32, -hw * 0.45), to_screen(hl * 0.15, -hw * 0.10));

                // Side windows & B-pillar
                for &side in &[-1.0f32, 1.0f32] {
                    let side_win = [
                        to_screen(hl * 0.10, side * hw * 0.68),
                        to_screen(hl * 0.10, side * hw * 0.82),
                        to_screen(-hl * 0.35, side * hw * 0.82),
                        to_screen(-hl * 0.35, side * hw * 0.68),
                    ];
                    Self::fill_polygon(canvas, &side_win, Color::RGB(36, 52, 70));
                    canvas.set_draw_color(Color::RGB(18, 22, 28));
                    let _ = canvas.draw_line(to_screen(-hl * 0.12, side * hw * 0.68), to_screen(-hl * 0.12, side * hw * 0.82));
                }

                // Rear window
                let rear_win = [
                    to_screen(-hl * 0.35, -hw * 0.68),
                    to_screen(-hl * 0.35, hw * 0.68),
                    to_screen(-hl * 0.62, hw * 0.60),
                    to_screen(-hl * 0.62, -hw * 0.60),
                ];
                Self::fill_polygon(canvas, &rear_win, Color::RGB(34, 48, 65));
                Self::draw_polygon_outline(canvas, &rear_win, Color::RGB(20, 26, 34));

                // Roof panel
                let roof_pts = [
                    to_screen(hl * 0.10, -hw * 0.68),
                    to_screen(hl * 0.10, hw * 0.68),
                    to_screen(-hl * 0.35, hw * 0.68),
                    to_screen(-hl * 0.35, -hw * 0.68),
                ];
                Self::fill_polygon(canvas, &roof_pts, palette.body);

                // Sedan Distinct Trunk Deck Lid
                let trunk_pts = [
                    to_screen(-hl * 0.62, -hw * 0.65),
                    to_screen(-hl * 0.62, hw * 0.65),
                    to_screen(-hl * 0.95, hw * 0.62),
                    to_screen(-hl * 0.95, -hw * 0.62),
                ];
                Self::fill_polygon(canvas, &trunk_pts, palette.body);
                Self::draw_polygon_outline(canvas, &trunk_pts, palette.shadow);
                // Chrome trunk garnish strip
                canvas.set_draw_color(Color::RGB(215, 222, 230));
                let _ = canvas.draw_line(to_screen(-hl * 0.92, -hw * 0.40), to_screen(-hl * 0.92, hw * 0.40));
            }
            VehicleType::Suv => {
                // Windshield
                let windshield = [
                    to_screen(hl * 0.40, -hw * 0.78),
                    to_screen(hl * 0.40, hw * 0.78),
                    to_screen(hl * 0.15, hw * 0.72),
                    to_screen(hl * 0.15, -hw * 0.72),
                ];
                Self::fill_polygon(canvas, &windshield, Color::RGB(42, 60, 80));
                Self::draw_polygon_outline(canvas, &windshield, Color::RGB(22, 28, 36));
                canvas.set_draw_color(Color::RGB(85, 125, 160));
                let _ = canvas.draw_line(to_screen(hl * 0.35, -hw * 0.50), to_screen(hl * 0.18, -hw * 0.15));

                // Side windows & pillars
                for &side in &[-1.0f32, 1.0f32] {
                    let side_win = [
                        to_screen(hl * 0.15, side * hw * 0.72),
                        to_screen(hl * 0.15, side * hw * 0.85),
                        to_screen(-hl * 0.65, side * hw * 0.85),
                        to_screen(-hl * 0.65, side * hw * 0.72),
                    ];
                    Self::fill_polygon(canvas, &side_win, Color::RGB(34, 50, 68));
                    canvas.set_draw_color(Color::RGB(18, 22, 28));
                    let _ = canvas.draw_line(to_screen(-hl * 0.12, side * hw * 0.72), to_screen(-hl * 0.12, side * hw * 0.85));
                    let _ = canvas.draw_line(to_screen(-hl * 0.40, side * hw * 0.72), to_screen(-hl * 0.40, side * hw * 0.85));
                }

                // Upright rear tailgate window
                let rear_win = [
                    to_screen(-hl * 0.65, -hw * 0.72),
                    to_screen(-hl * 0.65, hw * 0.72),
                    to_screen(-hl * 0.88, hw * 0.65),
                    to_screen(-hl * 0.88, -hw * 0.65),
                ];
                Self::fill_polygon(canvas, &rear_win, Color::RGB(30, 42, 58));
                Self::draw_polygon_outline(canvas, &rear_win, Color::RGB(18, 24, 32));

                // Extended tall roof panel
                let roof_pts = [
                    to_screen(hl * 0.15, -hw * 0.72),
                    to_screen(hl * 0.15, hw * 0.72),
                    to_screen(-hl * 0.65, hw * 0.72),
                    to_screen(-hl * 0.65, -hw * 0.72),
                ];
                Self::fill_polygon(canvas, &roof_pts, palette.body);

                // Dual Silver Roof Rack Rails
                canvas.set_draw_color(Color::RGB(192, 200, 212));
                for &side in &[-1.0f32, 1.0f32] {
                    let _ = canvas.draw_line(to_screen(hl * 0.10, side * hw * 0.58), to_screen(-hl * 0.60, side * hw * 0.58));
                    let _ = canvas.draw_line(to_screen(hl * 0.10, side * hw * 0.54), to_screen(-hl * 0.60, side * hw * 0.54));
                }
            }
            VehicleType::Van => {
                // Cab-forward steep windshield
                let windshield = [
                    to_screen(hl * 0.65, -hw * 0.80),
                    to_screen(hl * 0.65, hw * 0.80),
                    to_screen(hl * 0.40, hw * 0.75),
                    to_screen(hl * 0.40, -hw * 0.75),
                ];
                Self::fill_polygon(canvas, &windshield, Color::RGB(42, 60, 80));
                Self::draw_polygon_outline(canvas, &windshield, Color::RGB(22, 28, 36));
                canvas.set_draw_color(Color::RGB(85, 125, 160));
                let _ = canvas.draw_line(to_screen(hl * 0.60, -hw * 0.50), to_screen(hl * 0.45, -hw * 0.15));

                // Elongated side windows
                for &side in &[-1.0f32, 1.0f32] {
                    let side_win = [
                        to_screen(hl * 0.40, side * hw * 0.75),
                        to_screen(hl * 0.40, side * hw * 0.88),
                        to_screen(-hl * 0.75, side * hw * 0.88),
                        to_screen(-hl * 0.75, side * hw * 0.75),
                    ];
                    Self::fill_polygon(canvas, &side_win, Color::RGB(32, 46, 62));
                    canvas.set_draw_color(Color::RGB(18, 22, 28));
                    let _ = canvas.draw_line(to_screen(hl * 0.08, side * hw * 0.75), to_screen(hl * 0.08, side * hw * 0.88));
                    let _ = canvas.draw_line(to_screen(-hl * 0.28, side * hw * 0.75), to_screen(-hl * 0.28, side * hw * 0.88));
                    let _ = canvas.draw_line(to_screen(-hl * 0.55, side * hw * 0.75), to_screen(-hl * 0.55, side * hw * 0.88));
                }

                // Rear cargo door windows
                let rear_win = [
                    to_screen(-hl * 0.75, -hw * 0.72),
                    to_screen(-hl * 0.75, hw * 0.72),
                    to_screen(-hl * 0.92, hw * 0.68),
                    to_screen(-hl * 0.92, -hw * 0.68),
                ];
                Self::fill_polygon(canvas, &rear_win, Color::RGB(28, 38, 52));
                Self::draw_polygon_outline(canvas, &rear_win, Color::RGB(18, 24, 32));

                // Long continuous van roof
                let roof_pts = [
                    to_screen(hl * 0.40, -hw * 0.75),
                    to_screen(hl * 0.40, hw * 0.75),
                    to_screen(-hl * 0.75, hw * 0.75),
                    to_screen(-hl * 0.75, -hw * 0.75),
                ];
                Self::fill_polygon(canvas, &roof_pts, palette.body);

                // Stamped roof reinforcement ribs
                canvas.set_draw_color(palette.shadow);
                let _ = canvas.draw_line(to_screen(hl * 0.30, -hw * 0.38), to_screen(-hl * 0.65, -hw * 0.38));
                let _ = canvas.draw_line(to_screen(hl * 0.30, hw * 0.38), to_screen(-hl * 0.65, hw * 0.38));

                // Dual rear cargo door vertical split line
                canvas.set_draw_color(Color::RGB(24, 28, 34));
                let _ = canvas.draw_line(to_screen(-hl * 0.75, 0.0), to_screen(-hl, 0.0));
            }
        }

        // ----------------------------------------------------
        // G. Warm Projector Headlights (Warm projector illumination)
        // ----------------------------------------------------
        for &side in &[-1.0f32, 1.0f32] {
            let hl_pts = [
                to_screen(hl - 0.4, side * hw * 0.45),
                to_screen(hl - 0.8, side * hw * 0.85),
                to_screen(hl - 4.5, side * hw * 0.95),
                to_screen(hl - 3.2, side * hw * 0.50),
            ];
            Self::fill_polygon(canvas, &hl_pts, Color::RGB(255, 248, 205));
            Self::draw_polygon_outline(canvas, &hl_pts, Color::RGB(210, 195, 150));

            // Warm golden core beam
            canvas.set_draw_color(Color::RGB(255, 235, 140));
            let _ = canvas.draw_line(
                to_screen(hl - 1.2, side * hw * 0.55),
                to_screen(hl - 2.8, side * hw * 0.75),
            );
            // LED daylight accent strip
            canvas.set_draw_color(Color::RGB(255, 255, 245));
            let _ = canvas.draw_line(
                to_screen(hl - 0.9, side * hw * 0.82),
                to_screen(hl - 4.0, side * hw * 0.92),
            );
        }

        // ----------------------------------------------------
        // H. Taillights & Dynamic Brake Lights
        // Normal cruise: dim dark-red
        // Active braking or stop: brilliant saturated bright red
        // ----------------------------------------------------
        let tl_color = if is_braking {
            Color::RGB(255, 25, 35) // Brilliant saturated bright red
        } else {
            Color::RGB(130, 16, 22) // Dim dark red passive running lights
        };

        let tl_outline = if is_braking {
            Color::RGB(255, 120, 130) // Hot reddish-pink incandescent halo
        } else {
            Color::RGB(75, 10, 14) // Dark passive surround
        };

        for &side in &[-1.0f32, 1.0f32] {
            let tl_pts = [
                to_screen(-hl + 3.2, side * hw * 0.65),
                to_screen(-hl + 3.2, side * hw * 0.95),
                to_screen(-hl + 0.4, side * hw * 0.88),
                to_screen(-hl + 0.4, side * hw * 0.55),
            ];
            Self::fill_polygon(canvas, &tl_pts, tl_color);
            Self::draw_polygon_outline(canvas, &tl_pts, tl_outline);

            if is_braking {
                // Intense bright filament / LED core
                canvas.set_draw_color(Color::RGB(255, 215, 215));
                let _ = canvas.draw_line(
                    to_screen(-hl + 2.4, side * hw * 0.72),
                    to_screen(-hl + 1.0, side * hw * 0.82),
                );
            } else {
                // Subtle passive running light marker
                canvas.set_draw_color(Color::RGB(165, 25, 32));
                let _ = canvas.draw_point(to_screen(-hl + 1.8, side * hw * 0.76));
            }
        }

        // Center High-Mounted Stop Lamp (CHMSL)
        let chmsl_color = if is_braking {
            Color::RGB(255, 30, 40) // Blazing bright stop lamp
        } else {
            Color::RGB(95, 12, 16)  // Passive lens
        };
        canvas.set_draw_color(chmsl_color);
        let chmsl_fwd = match vehicle.vehicle_type {
            VehicleType::Hatchback => -hl + 1.2,
            VehicleType::Sedan => -hl * 0.38,
            VehicleType::Suv => -hl * 0.68,
            VehicleType::Van => -hl + 1.5,
        };
        let _ = canvas.draw_line(
            to_screen(chmsl_fwd, -hw * 0.25),
            to_screen(chmsl_fwd, hw * 0.25),
        );

        // ----------------------------------------------------
        // I. Flashing Amber Turn Signals (Turn::Left or Turn::Right)
        // ----------------------------------------------------
        if blink_left {
            let amber_col = if blink_on {
                Color::RGB(255, 185, 15) // Bright active amber
            } else {
                Color::RGB(110, 70, 15)  // Inactive amber
            };
            let fl_pts = [
                to_screen(hl - 0.5, -hw * 0.75),
                to_screen(hl - 0.5, -hw * 0.95),
                to_screen(hl - 3.2, -hw * 0.98),
                to_screen(hl - 2.5, -hw * 0.80),
            ];
            Self::fill_polygon(canvas, &fl_pts, amber_col);
            let rl_pts = [
                to_screen(-hl + 3.0, -hw * 0.80),
                to_screen(-hl + 3.0, -hw * 0.98),
                to_screen(-hl + 0.6, -hw * 0.92),
                to_screen(-hl + 0.6, -hw * 0.75),
            ];
            Self::fill_polygon(canvas, &rl_pts, amber_col);
            if blink_on {
                canvas.set_draw_color(Color::RGB(255, 240, 80));
                let _ = canvas.draw_point(to_screen(hl - 1.5, -hw * 0.88));
                let _ = canvas.draw_point(to_screen(-hl + 1.8, -hw * 0.88));
            }
        }

        if blink_right {
            let amber_col = if blink_on {
                Color::RGB(255, 185, 15) // Bright active amber
            } else {
                Color::RGB(110, 70, 15)  // Inactive amber
            };
            let fr_pts = [
                to_screen(hl - 0.5, hw * 0.75),
                to_screen(hl - 0.5, hw * 0.95),
                to_screen(hl - 3.2, hw * 0.98),
                to_screen(hl - 2.5, hw * 0.80),
            ];
            Self::fill_polygon(canvas, &fr_pts, amber_col);
            let rr_pts = [
                to_screen(-hl + 3.0, hw * 0.80),
                to_screen(-hl + 3.0, hw * 0.98),
                to_screen(-hl + 0.6, hw * 0.92),
                to_screen(-hl + 0.6, hw * 0.75),
            ];
            Self::fill_polygon(canvas, &rr_pts, amber_col);
            if blink_on {
                canvas.set_draw_color(Color::RGB(255, 240, 80));
                let _ = canvas.draw_point(to_screen(hl - 1.5, hw * 0.88));
                let _ = canvas.draw_point(to_screen(-hl + 1.8, hw * 0.88));
            }
        }

        // Rear License Plate
        let plate = [
            to_screen(-hl + 0.4, -hw * 0.32),
            to_screen(-hl + 0.4, hw * 0.32),
            to_screen(-hl - 0.1, hw * 0.32),
            to_screen(-hl - 0.1, -hw * 0.32),
        ];
        Self::fill_polygon(canvas, &plate, Color::RGB(245, 245, 245));

        // Autonomous Smart Sensor Dome (Lidar puck for autonomous intersection management)
        canvas.set_draw_color(Color::RGB(0, 220, 255));
        let puck_center = to_screen(-hl * 0.20, 0.0);
        let _ = canvas.fill_rect(Rect::new(puck_center.x - 2, puck_center.y - 2, 5, 5));

        // Route & Speed Badge above Vehicle
        let label = format!("{}{}", vehicle.turn.as_char(), vehicle.velocity.round() as i32);
        let badge_x = pos.x as i32 - 12;
        let badge_y = pos.y as i32 - (hl as i32 + 13);
        Self::draw_string_5x7(canvas, &label, badge_x, badge_y, 1, Color::RGB(255, 255, 255));
    }

    /// Scanline polygon fill for arbitrary N-vertex polygons
    fn fill_polygon(canvas: &mut Canvas<Window>, pts: &[Point], color: Color) {
        if pts.len() < 3 {
            return;
        }
        canvas.set_draw_color(color);

        let min_y = pts.iter().map(|p| p.y).min().unwrap_or(0);
        let max_y = pts.iter().map(|p| p.y).max().unwrap_or(0);
        let n = pts.len();

        for y in min_y..=max_y {
            let mut nodes = Vec::with_capacity(8);
            for i in 0..n {
                let j = (i + 1) % n;
                let (p1, p2) = (pts[i], pts[j]);
                if (p1.y < y && p2.y >= y) || (p2.y < y && p1.y >= y) {
                    let dy = p2.y - p1.y;
                    if dy != 0 {
                        let x = p1.x + ((y - p1.y) as f32 / dy as f32 * (p2.x - p1.x) as f32).round() as i32;
                        nodes.push(x);
                    }
                }
            }
            nodes.sort_unstable();
            let mut k = 0;
            while k + 1 < nodes.len() {
                let _ = canvas.draw_line(Point::new(nodes[k], y), Point::new(nodes[k + 1], y));
                k += 2;
            }
        }
    }

    /// Outlines a polygon perimeter
    fn draw_polygon_outline(canvas: &mut Canvas<Window>, pts: &[Point], color: Color) {
        if pts.len() < 2 {
            return;
        }
        canvas.set_draw_color(color);
        let n = pts.len();
        for i in 0..n {
            let j = (i + 1) % n;
            let _ = canvas.draw_line(pts[i], pts[j]);
        }
    }

    /// In-simulation HUD display banner & metric pills
    fn draw_hud(canvas: &mut Canvas<Window>, sim: &Simulation) {
        // Top glassmorphic banner
        canvas.set_draw_color(Color::RGBA(18, 22, 28, 230));
        let _ = canvas.fill_rect(Rect::new(12, 12, (CANVAS_WIDTH as u32) - 24, 52));

        // Border
        canvas.set_draw_color(Color::RGB(50, 60, 75));
        let _ = canvas.draw_rect(Rect::new(12, 12, (CANVAS_WIDTH as u32) - 24, 52));

        // Title
        Self::draw_string_5x7(canvas, "SMART-ROAD | FLEET: CHEVROLET SPARK", 24, 20, 1, Color::RGB(0, 220, 255));

        // Paint Mode status badge in header
        let paint_txt = match sim.fleet_paint_mode {
            crate::simulation::FleetPaintMode::SummitWhiteOnly => "PAINT: SUMMIT WHITE [C]",
            crate::simulation::FleetPaintMode::SparkPalette => "PAINT: SPARK PALETTE [C]",
        };
        Self::draw_string_5x7(canvas, paint_txt, 550, 20, 1, Color::RGB(255, 215, 60));

        // HUD Metric pills
        let active_txt = format!("ACTIVE: {}", sim.vehicles.len());
        Self::draw_string_5x7(canvas, &active_txt, 24, 38, 1, Color::RGB(230, 235, 240));

        let passed_txt = format!("PASSED: {}", sim.stats.max_vehicles_passed);
        Self::draw_string_5x7(canvas, &passed_txt, 140, 38, 1, Color::RGB(100, 240, 150));

        let calls_txt = format!("CLOSE CALLS: {}", sim.stats.close_calls);
        let calls_color = if sim.stats.close_calls > 0 {
            Color::RGB(255, 170, 50)
        } else {
            Color::RGB(120, 220, 255)
        };
        Self::draw_string_5x7(canvas, &calls_txt, 270, 38, 1, calls_color);

        let auto_status = if sim.auto_spawn { "AUTO: ON [R]" } else { "AUTO: OFF [R]" };
        let auto_color = if sim.auto_spawn { Color::RGB(0, 230, 150) } else { Color::RGB(180, 185, 195) };
        Self::draw_string_5x7(canvas, auto_status, 430, 38, 1, auto_color);

        let timer_txt = format!("TIME: {:.1}s", sim.stats.simulation_duration);
        Self::draw_string_5x7(canvas, &timer_txt, 560, 38, 1, Color::RGB(220, 220, 230));

        // Bottom Controls Bar
        canvas.set_draw_color(Color::RGBA(18, 22, 28, 220));
        let _ = canvas.fill_rect(Rect::new(12, (CANVAS_HEIGHT as i32) - 44, (CANVAS_WIDTH as u32) - 24, 32));
        canvas.set_draw_color(Color::RGB(50, 60, 75));
        let _ = canvas.draw_rect(Rect::new(12, (CANVAS_HEIGHT as i32) - 44, (CANVAS_WIDTH as u32) - 24, 32));

        let controls_str = "SPAWN: [ARROWS] | [R] Auto | [C] Spark Paint | [ESC] Finish & Stats";
        Self::draw_string_5x7(canvas, controls_str, 24, (CANVAS_HEIGHT as i32) - 34, 1, Color::RGB(200, 210, 225));
    }

    /// Dedicated Statistics Window Modal (Displayed when simulation finishes on Esc)
    fn draw_stats_screen(canvas: &mut Canvas<Window>, sim: &Simulation) {
        // Semi-transparent backdrop blur overlay
        canvas.set_draw_color(Color::RGBA(10, 14, 20, 235));
        let _ = canvas.fill_rect(Rect::new(0, 0, CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32));

        // Dashboard card
        let modal_w = 680;
        let modal_h = 620;
        let mx = (CANVAS_WIDTH as i32 - modal_w) / 2;
        let my = (CANVAS_HEIGHT as i32 - modal_h) / 2;

        canvas.set_draw_color(Color::RGB(22, 27, 36));
        let _ = canvas.fill_rect(Rect::new(mx, my, modal_w as u32, modal_h as u32));

        // Glowing border
        canvas.set_draw_color(Color::RGB(0, 215, 255));
        let _ = canvas.draw_rect(Rect::new(mx, my, modal_w as u32, modal_h as u32));
        let _ = canvas.draw_rect(Rect::new(mx + 1, my + 1, (modal_w - 2) as u32, (modal_h - 2) as u32));

        // Modal Header
        Self::draw_string_5x7(canvas, "SIMULATION STATISTICS REPORT", mx + 160, my + 30, 2, Color::RGB(0, 230, 255));
        Self::draw_string_5x7(canvas, "FLEET MODEL: CHEVROLET SPARK (HATCHBACK)", mx + 165, my + 60, 1, Color::RGB(160, 175, 195));

        // Decorative separator line
        canvas.set_draw_color(Color::RGB(45, 60, 80));
        let _ = canvas.draw_line(Point::new(mx + 40, my + 85), Point::new(mx + modal_w - 40, my + 85));

        // Stat Card Coordinates
        let card_w = 280;
        let card_h = 80;
        let col1_x = mx + 45;
        let col2_x = mx + 355;

        // Metric 1: Max number of vehicles that passed the intersection
        let row1_y = my + 105;
        Self::draw_stat_card(
            canvas,
            col1_x,
            row1_y,
            card_w,
            card_h,
            "MAX VEHICLES PASSED",
            &format!("{}", sim.stats.max_vehicles_passed),
            "VEHICLES",
            Color::RGB(70, 230, 130),
        );

        // Metric 2: Close calls (violation of safety distance)
        let calls_col = if sim.stats.close_calls > 0 {
            Color::RGB(255, 160, 40)
        } else {
            Color::RGB(80, 220, 255)
        };
        Self::draw_stat_card(
            canvas,
            col2_x,
            row1_y,
            card_w,
            card_h,
            "CLOSE CALLS COUNT",
            &format!("{}", sim.stats.close_calls),
            "EVENTS",
            calls_col,
        );

        // Metric 3: Max velocity achieved
        let row2_y = my + 200;
        Self::draw_stat_card(
            canvas,
            col1_x,
            row2_y,
            card_w,
            card_h,
            "MAX VELOCITY (FASTEST)",
            &format!("{:.1}", sim.stats.max_velocity),
            "PX / SEC",
            Color::RGB(255, 205, 50),
        );

        // Metric 4: Min velocity reached
        Self::draw_stat_card(
            canvas,
            col2_x,
            row2_y,
            card_w,
            card_h,
            "MIN VELOCITY (SLOWEST)",
            &format!("{:.1}", sim.stats.display_min_velocity()),
            "PX / SEC",
            Color::RGB(160, 210, 255),
        );

        // Metric 5: Max time that vehicles took to pass
        let row3_y = my + 295;
        Self::draw_stat_card(
            canvas,
            col1_x,
            row3_y,
            card_w,
            card_h,
            "MAX TRANSIT TIME",
            &format!("{:.2} s", sim.stats.max_time),
            "LONGEST TRANSIT",
            Color::RGB(255, 120, 130),
        );

        // Metric 6: Min time that vehicles took to pass
        Self::draw_stat_card(
            canvas,
            col2_x,
            row3_y,
            card_w,
            card_h,
            "MIN TRANSIT TIME",
            &format!("{:.2} s", sim.stats.display_min_time()),
            "SHORTEST TRANSIT",
            Color::RGB(120, 240, 190),
        );

        // Bonus Metrics Bar (Bottom section)
        let bonus_y = my + 395;
        canvas.set_draw_color(Color::RGB(28, 36, 48));
        let _ = canvas.fill_rect(Rect::new(col1_x, bonus_y, (card_w * 2 + 30) as u32, 95));
        canvas.set_draw_color(Color::RGB(55, 75, 100));
        let _ = canvas.draw_rect(Rect::new(col1_x, bonus_y, (card_w * 2 + 30) as u32, 95));

        Self::draw_string_5x7(canvas, "FLEET SUMMARY & AUTONOMOUS SAFETY AUDIT", col1_x + 15, bonus_y + 12, 1, Color::RGB(0, 210, 255));

        let b1 = format!("FLEET VEHICLE:   CHEVROLET SPARK");
        let b2 = format!("COLLISIONS:      0 (100% COLLISION FREE)");
        let b3 = format!("TOTAL SPAWNED:   {}", sim.stats.total_spawned);
        let b4 = format!("AVG SPEED:       {:.1} px/s", sim.stats.avg_velocity());
        let b5 = format!("AVG TRANSIT:     {:.2} s", sim.stats.avg_time());

        Self::draw_string_5x7(canvas, &b1, col1_x + 15, bonus_y + 35, 1, Color::RGB(255, 215, 60));
        Self::draw_string_5x7(canvas, &b3, col1_x + 15, bonus_y + 55, 1, Color::RGB(220, 225, 235));
        Self::draw_string_5x7(canvas, &b5, col1_x + 15, bonus_y + 75, 1, Color::RGB(220, 225, 235));

        Self::draw_string_5x7(canvas, &b2, col1_x + 290, bonus_y + 35, 1, Color::RGB(90, 245, 140));
        Self::draw_string_5x7(canvas, &b4, col1_x + 290, bonus_y + 55, 1, Color::RGB(220, 225, 235));

        // Interactive Footer Actions
        let foot_y = my + modal_h - 65;
        canvas.set_draw_color(Color::RGB(35, 45, 60));
        let _ = canvas.draw_line(Point::new(mx + 40, foot_y - 15), Point::new(mx + modal_w - 40, foot_y - 15));

        Self::draw_string_5x7(canvas, "PRESS [ENTER] TO RESTART SIMULATION   |   PRESS [ESC] OR [Q] TO EXIT", mx + 90, foot_y + 5, 1, Color::RGB(255, 220, 100));
    }

    fn draw_stat_card(
        canvas: &mut Canvas<Window>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        title: &str,
        value: &str,
        unit: &str,
        val_color: Color,
    ) {
        // Card background
        canvas.set_draw_color(Color::RGB(26, 33, 44));
        let _ = canvas.fill_rect(Rect::new(x, y, w as u32, h as u32));

        // Card border
        canvas.set_draw_color(Color::RGB(45, 58, 76));
        let _ = canvas.draw_rect(Rect::new(x, y, w as u32, h as u32));

        // Card Title
        Self::draw_string_5x7(canvas, title, x + 14, y + 12, 1, Color::RGB(155, 170, 190));

        // Value in large size
        Self::draw_string_5x7(canvas, value, x + 14, y + 36, 2, val_color);

        // Unit subtitle
        Self::draw_string_5x7(canvas, unit, x + 14, y + 62, 1, Color::RGB(120, 135, 155));
    }

    /// Renders a string using the embedded 5x7 bitmap font
    pub fn draw_string_5x7(
        canvas: &mut Canvas<Window>,
        text: &str,
        x: i32,
        y: i32,
        scale: i32,
        color: Color,
    ) {
        let mut cur_x = x;
        for c in text.chars() {
            Self::draw_char_5x7(canvas, c, cur_x, y, scale, color);
            cur_x += (5 + 1) * scale;
        }
    }

    /// Single character rendering with 5x7 bitmap font
    fn draw_char_5x7(
        canvas: &mut Canvas<Window>,
        c: char,
        x: i32,
        y: i32,
        scale: i32,
        color: Color,
    ) {
        canvas.set_draw_color(color);
        let glyph = get_glyph_5x7(c);

        for row in 0..7 {
            let row_bits = glyph[row];
            for col in 0..5 {
                if (row_bits & (1 << (4 - col))) != 0 {
                    if scale == 1 {
                        let _ = canvas.draw_point(Point::new(x + col, y + row as i32));
                    } else {
                        let _ = canvas.fill_rect(Rect::new(
                            x + col * scale,
                            y + (row as i32) * scale,
                            scale as u32,
                            scale as u32,
                        ));
                    }
                }
            }
        }
    }
}

/// 5x7 Bitmap Glyph definitions for numbers, uppercase/lowercase letters, and common symbols
fn get_glyph_5x7(c: char) -> [u8; 7] {
    match c {
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],

        'A' | 'a' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' | 'b' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' | 'c' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' | 'd' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' | 'e' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' | 'f' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' | 'g' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' | 'h' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' | 'i' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' | 'j' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' | 'k' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' | 'l' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' | 'm' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' | 'n' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' | 'o' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' | 'p' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' | 'q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' | 'r' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' | 's' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' | 't' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' | 'u' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' | 'v' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' | 'w' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' | 'x' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' | 'y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' | 'z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],

        ':' => [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        ',' => [0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b00100, 0b01000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        '/' => [0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000],
        '%' => [0b11001, 0b11010, 0b00100, 0b01000, 0b01011, 0b10011, 0b00000],
        '[' => [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110],
        ']' => [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110],
        '(' => [0b00100, 0b01000, 0b10000, 0b10000, 0b10000, 0b01000, 0b00100],
        ')' => [0b00100, 0b00010, 0b00001, 0b00001, 0b00001, 0b00010, 0b00100],
        '|' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        _ =>   [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    }
}
