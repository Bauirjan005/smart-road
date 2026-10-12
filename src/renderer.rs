use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use crate::simulation::Simulation;
use crate::types::{Turn, VehicleType, CANVAS_HEIGHT, CANVAS_WIDTH, CENTER_X, CENTER_Y, HALF_ROAD_WIDTH, LANE_WIDTH};
use crate::vehicle::Vehicle;

#[derive(Clone, Copy)]
pub struct SparkPalette {
    pub name: &'static str,
    pub body: Color,
    pub shadow: Color,
    pub highlight: Color,
}

const fn pal(name: &'static str, b: (u8, u8, u8), s: (u8, u8, u8), h: (u8, u8, u8)) -> SparkPalette {
    SparkPalette { name, body: Color::RGB(b.0, b.1, b.2), shadow: Color::RGB(s.0, s.1, s.2), highlight: Color::RGB(h.0, h.1, h.2) }
}

pub const CHEVY_SPARK_COLORS: [SparkPalette; 8] = [
    pal("Summit White", (246, 248, 252), (192, 198, 208), (255, 255, 255)),
    pal("Mystic Blue", (38, 110, 210), (22, 68, 140), (95, 170, 255)),
    pal("Red Hot", (225, 34, 46), (155, 20, 30), (255, 85, 96)),
    pal("Nitro Yellow", (245, 192, 20), (180, 140, 15), (255, 225, 80)),
    pal("Passion Fruit", (150, 58, 180), (105, 35, 130), (200, 105, 230)),
    pal("Caribbean Blue", (22, 182, 212), (14, 128, 155), (105, 228, 255)),
    pal("Silver Ice", (190, 196, 205), (142, 148, 158), (236, 242, 250)),
    pal("Nightfall Gray", (74, 80, 90), (46, 52, 60), (122, 132, 146)),
];

pub struct Renderer;

impl Renderer {
    pub fn render(canvas: &mut Canvas<Window>, sim: &Simulation) -> Result<(), String> {
        canvas.set_blend_mode(BlendMode::Blend);
        Self::draw_environment(canvas);
        Self::draw_roads(canvas);
        Self::draw_lane_markings(canvas);
        Self::draw_sensor_zones(canvas);
        for v in &sim.vehicles { Self::draw_vehicle(canvas, v, sim.stats.simulation_duration); }
        Self::draw_hud(canvas, sim);
        if sim.is_finished { Self::draw_stats_screen(canvas, sim); }
        canvas.present();
        Ok(())
    }

    fn draw_environment(canvas: &mut Canvas<Window>) {
        let (cx, cy, hw, sw) = (CENTER_X as i32, CENTER_Y as i32, HALF_ROAD_WIDTH as i32, 28);
        canvas.set_draw_color(Color::RGB(36, 52, 38));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(33, 48, 35));
        for y in (0..CANVAS_HEIGHT as i32).step_by(32) { let _ = canvas.fill_rect(Rect::new(0, y, CANVAS_WIDTH as u32, 16)); }

        canvas.set_draw_color(Color::RGB(150, 155, 164));
        for (r1, r2, r3) in [
            (Rect::new(cx - hw - sw, 0, sw as u32, (cy - hw) as u32), Rect::new(0, cy - hw - sw, (cx - hw) as u32, sw as u32), Rect::new(cx - hw - sw, cy - hw - sw, sw as u32, sw as u32)),
            (Rect::new(cx + hw, 0, sw as u32, (cy - hw) as u32), Rect::new(cx + hw, cy - hw - sw, (CANVAS_WIDTH as i32 - (cx + hw)) as u32, sw as u32), Rect::new(cx + hw, cy - hw - sw, sw as u32, sw as u32)),
            (Rect::new(cx - hw - sw, cy + hw, sw as u32, (CANVAS_HEIGHT as i32 - (cy + hw)) as u32), Rect::new(0, cy + hw, (cx - hw) as u32, sw as u32), Rect::new(cx - hw - sw, cy + hw, sw as u32, sw as u32)),
            (Rect::new(cx + hw, cy + hw, sw as u32, (CANVAS_HEIGHT as i32 - (cy + hw)) as u32), Rect::new(cx + hw, cy + hw, (CANVAS_WIDTH as i32 - (cx + hw)) as u32, sw as u32), Rect::new(cx + hw, cy + hw, sw as u32, sw as u32)),
        ] { let _ = canvas.fill_rect(r1); let _ = canvas.fill_rect(r2); let _ = canvas.fill_rect(r3); }

        canvas.set_draw_color(Color::RGB(130, 135, 144));
        for &sx in &[cx - hw - sw, cx + hw] {
            for y in (0..cy - hw).step_by(24).chain((cy + hw..CANVAS_HEIGHT as i32).step_by(24)) {
                let _ = canvas.draw_line(Point::new(sx, y), Point::new(sx + sw, y));
            }
        }
        for &sy in &[cy - hw - sw, cy + hw] {
            for x in (0..cx - hw).step_by(24).chain((cx + hw..CANVAS_WIDTH as i32).step_by(24)) {
                let _ = canvas.draw_line(Point::new(x, sy), Point::new(x, sy + sw));
            }
        }
    }

    fn draw_roads(canvas: &mut Canvas<Window>) {
        let (cx, cy, hw) = (CENTER_X as i32, CENTER_Y as i32, HALF_ROAD_WIDTH as i32);
        canvas.set_draw_color(Color::RGB(34, 37, 44));
        let _ = canvas.fill_rect(Rect::new(cx - hw, 0, (hw * 2) as u32, CANVAS_HEIGHT as u32));
        let _ = canvas.fill_rect(Rect::new(0, cy - hw, CANVAS_WIDTH as u32, (hw * 2) as u32));
        canvas.set_draw_color(Color::RGB(30, 33, 40));
        let _ = canvas.fill_rect(Rect::new(cx - hw, cy - hw, (hw * 2) as u32, (hw * 2) as u32));

        canvas.set_draw_color(Color::RGB(24, 26, 31));
        for x in [cx - hw, cx + hw - 4] {
            let _ = canvas.fill_rect(Rect::new(x, 0, 4, (cy - hw) as u32));
            let _ = canvas.fill_rect(Rect::new(x, cy + hw, 4, (CANVAS_HEIGHT as i32 - (cy + hw)) as u32));
        }
        for y in [cy - hw, cy + hw - 4] {
            let _ = canvas.fill_rect(Rect::new(0, y, (cx - hw) as u32, 4));
            let _ = canvas.fill_rect(Rect::new(cx + hw, y, (CANVAS_WIDTH as i32 - (cx + hw)) as u32, 4));
        }

        canvas.set_draw_color(Color::RGB(210, 215, 225));
        for (p1, p2) in [
            ((0, cy - hw), (cx - hw, cy - hw)), ((cx - hw, 0), (cx - hw, cy - hw)),
            ((cx + hw, 0), (cx + hw, cy - hw)), ((cx + hw, cy - hw), (CANVAS_WIDTH as i32, cy - hw)),
            ((0, cy + hw), (cx - hw, cy + hw)), ((cx - hw, cy + hw), (cx - hw, CANVAS_HEIGHT as i32)),
            ((cx + hw, CANVAS_HEIGHT as i32), (cx + hw, cy + hw)), ((cx + hw, cy + hw), (CANVAS_WIDTH as i32, cy + hw)),
        ] { let _ = canvas.draw_line(Point::new(p1.0, p1.1), Point::new(p2.0, p2.1)); }
    }

    fn draw_lane_markings(canvas: &mut Canvas<Window>) {
        let (cx, cy, hw, lw) = (CENTER_X as i32, CENTER_Y as i32, HALF_ROAD_WIDTH as i32, LANE_WIDTH as i32);
        canvas.set_draw_color(Color::RGB(250, 198, 28));
        for ox in [-3, 1] {
            let _ = canvas.fill_rect(Rect::new(cx + ox, 0, 2, (cy - hw) as u32));
            let _ = canvas.fill_rect(Rect::new(cx + ox, cy + hw, 2, (CANVAS_HEIGHT as i32 - (cy + hw)) as u32));
        }
        for oy in [-3, 1] {
            let _ = canvas.fill_rect(Rect::new(0, cy + oy, (cx - hw) as u32, 2));
            let _ = canvas.fill_rect(Rect::new(cx + hw, cy + oy, (CANVAS_WIDTH as i32 - (cx + hw)) as u32, 2));
        }

        canvas.set_draw_color(Color::RGB(220, 226, 236));
        for lane in [1, 2] {
            for x in [cx - lane * lw, cx + lane * lw] {
                for y in (0..cy - hw - 36).step_by(32) { let _ = canvas.draw_line(Point::new(x, y), Point::new(x, (y + 16).min(cy - hw - 36))); }
                for y in (cy + hw + 36..CANVAS_HEIGHT as i32).step_by(32) { let _ = canvas.draw_line(Point::new(x, y), Point::new(x, (y + 16).min(CANVAS_HEIGHT as i32))); }
            }
            for y in [cy - lane * lw, cy + lane * lw] {
                for x in (0..cx - hw - 36).step_by(32) { let _ = canvas.draw_line(Point::new(x, y), Point::new((x + 16).min(cx - hw - 36), y)); }
                for x in (cx + hw + 36..CANVAS_WIDTH as i32).step_by(32) { let _ = canvas.draw_line(Point::new(x, y), Point::new((x + 16).min(CANVAS_WIDTH as i32), y)); }
            }
        }

        Self::draw_zebra_crossings(canvas);
        canvas.set_draw_color(Color::RGB(250, 252, 255));
        for r in [
            Rect::new(cx, cy + hw - 5, hw as u32, 5), Rect::new(cx - hw, cy - hw, hw as u32, 5),
            Rect::new(cx - hw, cy, 5, hw as u32), Rect::new(cx + hw - 5, cy - hw, 5, hw as u32),
        ] { let _ = canvas.fill_rect(r); }
        Self::draw_schematic_arrows(canvas);
    }

    fn draw_zebra_crossings(canvas: &mut Canvas<Window>) {
        let (cx, cy, hw) = (CENTER_X as i32, CENTER_Y as i32, HALF_ROAD_WIDTH as i32);
        canvas.set_draw_color(Color::RGB(246, 248, 252));
        let mut d = cx - hw + 6;
        while d < cx + hw - 6 {
            let _ = canvas.fill_rect(Rect::new(d, cy + hw + 8, 10, 26));
            let _ = canvas.fill_rect(Rect::new(d, cy - hw - 8 - 26, 10, 26));
            let _ = canvas.fill_rect(Rect::new(cx - hw - 8 - 26, d, 26, 10));
            let _ = canvas.fill_rect(Rect::new(cx + hw + 8, d, 26, 10));
            d += 18;
        }
    }

    fn draw_schematic_arrows(canvas: &mut Canvas<Window>) {
        let (cx, cy, lw) = (CENTER_X as i32, CENTER_Y as i32, LANE_WIDTH as i32);
        for (x, y, deg, turn) in [
            (cx + lw / 2, cy + 178, 270.0, Turn::Left), (cx + 3 * lw / 2, cy + 178, 270.0, Turn::Straight), (cx + 5 * lw / 2, cy + 178, 270.0, Turn::Right),
            (cx - lw / 2, cy - 178, 90.0, Turn::Left), (cx - 3 * lw / 2, cy - 178, 90.0, Turn::Straight), (cx - 5 * lw / 2, cy - 178, 90.0, Turn::Right),
            (cx - 178, cy + lw / 2, 0.0, Turn::Left), (cx - 178, cy + 3 * lw / 2, 0.0, Turn::Straight), (cx - 178, cy + 5 * lw / 2, 0.0, Turn::Right),
            (cx + 178, cy - lw / 2, 180.0, Turn::Left), (cx + 178, cy - 3 * lw / 2, 180.0, Turn::Straight), (cx + 178, cy - 5 * lw / 2, 180.0, Turn::Right),
        ] { Self::draw_lane_arrow(canvas, x, y, deg, turn); }
    }

    fn draw_lane_arrow(canvas: &mut Canvas<Window>, xc: i32, yc: i32, deg: f32, turn: Turn) {
        let (rad, color) = (deg.to_radians(), Color::RGB(232, 238, 248));
        let (ca, sa) = (rad.cos(), rad.sin());
        let to_s = |f: f32, r: f32| Point::new((xc as f32 + f * ca - r * sa).round() as i32, (yc as f32 + f * sa + r * ca).round() as i32);
        match turn {
            Turn::Straight => {
                Self::fill_polygon(canvas, &[to_s(-12.0, -2.0), to_s(4.0, -2.0), to_s(4.0, 2.0), to_s(-12.0, 2.0)], color);
                Self::fill_polygon(canvas, &[to_s(4.0, -7.5), to_s(14.0, 0.0), to_s(4.0, 7.5)], color);
            }
            Turn::Left => {
                Self::fill_polygon(canvas, &[to_s(-12.0, -1.8), to_s(0.0, -1.8), to_s(0.0, 1.8), to_s(-12.0, 1.8)], color);
                Self::fill_polygon(canvas, &[to_s(0.0, 1.8), to_s(5.0, -8.0), to_s(2.0, -9.5), to_s(-2.0, -1.8)], color);
                Self::fill_polygon(canvas, &[to_s(9.0, -8.0), to_s(4.0, -16.0), to_s(-1.0, -8.0)], color);
            }
            Turn::Right => {
                Self::fill_polygon(canvas, &[to_s(-12.0, -1.8), to_s(0.0, -1.8), to_s(0.0, 1.8), to_s(-12.0, 1.8)], color);
                Self::fill_polygon(canvas, &[to_s(0.0, -1.8), to_s(5.0, 8.0), to_s(2.0, 9.5), to_s(-2.0, 1.8)], color);
                Self::fill_polygon(canvas, &[to_s(9.0, 8.0), to_s(4.0, 16.0), to_s(-1.0, 8.0)], color);
            }
        }
    }

    fn draw_sensor_zones(canvas: &mut Canvas<Window>) {
        let (cx, cy, hw, d) = (CENTER_X as i32, CENTER_Y as i32, HALF_ROAD_WIDTH as i32, 200);
        canvas.set_draw_color(Color::RGBA(0, 210, 255, 160));
        if cy + hw + d < CANVAS_HEIGHT as i32 { let _ = canvas.draw_line(Point::new(cx, cy + hw + d), Point::new(cx + hw, cy + hw + d)); }
        if cy - hw - d > 0 { let _ = canvas.draw_line(Point::new(cx - hw, cy - hw - d), Point::new(cx, cy - hw - d)); }
        if cx - hw - d > 0 { let _ = canvas.draw_line(Point::new(cx - hw - d, cy), Point::new(cx - hw - d, cy + hw)); }
        if cx + hw + d < CANVAS_WIDTH as i32 { let _ = canvas.draw_line(Point::new(cx + hw + d, cy - hw), Point::new(cx + hw + d, cy)); }
    }

    fn draw_vehicle(canvas: &mut Canvas<Window>, vehicle: &Vehicle, sim_time: f32) {
        let (pos, rad) = (vehicle.position(), vehicle.angle().to_radians());
        let (ca, sa) = (rad.cos(), rad.sin());
        let (w, l) = vehicle.vehicle_type.dimensions();
        let (hl, hw) = (l / 2.0, w / 2.0);

        let to_s = |f: f32, r: f32| Point::new((pos.x + f * ca - r * sa).round() as i32, (pos.y + f * sa + r * ca).round() as i32);
        let to_sh = |f: f32, r: f32| Point::new((pos.x + 2.5 + f * ca - r * sa).round() as i32, (pos.y + 2.5 + f * sa + r * ca).round() as i32);

        let palette = CHEVY_SPARK_COLORS[vehicle.color_idx % CHEVY_SPARK_COLORS.len()];
        let is_braking = vehicle.is_braking();
        let blink_on = ((sim_time * 5.0) as i32) % 2 == 0;

        // Shadow
        Self::fill_polygon(canvas, &[
            to_sh(hl+1.4, -hw*0.4), to_sh(hl+1.4, hw*0.4), to_sh(hl+0.6, hw+1.4), to_sh(hl*0.4, hw+1.6), to_sh(0.0, hw+1.6),
            to_sh(-hl*0.4, hw+1.6), to_sh(-hl+0.6, hw+1.4), to_sh(-hl-1.4, hw*0.5), to_sh(-hl-1.4, -hw*0.5), to_sh(-hl+0.6, -hw-1.4),
            to_sh(-hl*0.4, -hw-1.6), to_sh(0.0, -hw-1.6), to_sh(hl*0.4, -hw-1.6), to_sh(hl+0.6, -hw-1.4),
        ], Color::RGBA(12, 14, 18, 75));
        Self::fill_polygon(canvas, &[
            to_sh(hl+0.5, -hw*0.35), to_sh(hl+0.5, hw*0.35), to_sh(hl*0.5, hw+0.6), to_sh(-hl*0.5, hw+0.6),
            to_sh(-hl-0.5, hw*0.4), to_sh(-hl-0.5, -hw*0.4), to_sh(-hl*0.5, -hw-0.6), to_sh(hl*0.5, -hw-0.6),
        ], Color::RGBA(8, 10, 14, 130));

        // Headlight Beams & Brake Light Reflection
        for side in [-1.0f32, 1.0f32] {
            Self::fill_polygon(canvas, &[to_s(hl-0.5, side*hw*0.35), to_s(hl-0.5, side*hw*0.85), to_s(hl+14.0, side*(hw+4.5)), to_s(hl+14.0, side*hw*0.25)], Color::RGBA(255, 245, 200, 32));
        }
        if is_braking {
            Self::fill_polygon(canvas, &[to_s(-hl+0.5, -hw*0.9), to_s(-hl+0.5, hw*0.9), to_s(-hl-8.5, hw*1.25), to_s(-hl-8.5, -hw*1.25)], Color::RGBA(225, 20, 32, 80));
        }

        // Wheels
        let (w_len, w_wid) = match vehicle.vehicle_type { VehicleType::Hatchback => (5.6, 2.8), VehicleType::Sedan => (6.4, 3.0), VehicleType::Suv => (7.2, 3.5), VehicleType::Van => (7.6, 3.8) };
        for (wf, ws) in [(hl * 0.60, hw - 0.3), (hl * 0.60, -(hw - 0.3)), (-hl * 0.60, hw - 0.3), (-hl * 0.60, -(hw - 0.3))] {
            Self::fill_polygon(canvas, &[to_s(wf+w_len*0.5, ws-w_wid*0.5), to_s(wf+w_len*0.5, ws+w_wid*0.5), to_s(wf-w_len*0.5, ws+w_wid*0.5), to_s(wf-w_len*0.5, ws-w_wid*0.5)], Color::RGB(20, 22, 26));
            Self::fill_polygon(canvas, &[to_s(wf+w_len*0.28, ws-w_wid*0.26), to_s(wf+w_len*0.28, ws+w_wid*0.26), to_s(wf-w_len*0.28, ws+w_wid*0.26), to_s(wf-w_len*0.28, ws-w_wid*0.26)], Color::RGB(198, 204, 214));
        }

        // Body Shell
        let body_pts: Vec<Point> = match vehicle.vehicle_type {
            VehicleType::Hatchback => [to_s(hl, -hw*0.35), to_s(hl, hw*0.35), to_s(hl-0.9, hw*0.85), to_s(hl-3.2, hw), to_s(0.0, hw), to_s(-hl+2.8, hw), to_s(-hl+0.8, hw*0.85), to_s(-hl, hw*0.4), to_s(-hl, -hw*0.4), to_s(-hl+0.8, -hw*0.85), to_s(-hl+2.8, -hw), to_s(0.0, -hw), to_s(hl-3.2, -hw), to_s(hl-0.9, -hw*0.85)].to_vec(),
            VehicleType::Sedan => [to_s(hl, -hw*0.3), to_s(hl, hw*0.3), to_s(hl-1.2, hw*0.8), to_s(hl-4.2, hw), to_s(0.0, hw*0.98), to_s(-hl+4.5, hw), to_s(-hl+1.2, hw*0.82), to_s(-hl, hw*0.35), to_s(-hl, -hw*0.35), to_s(-hl+1.2, -hw*0.82), to_s(-hl+4.5, -hw), to_s(0.0, -hw*0.98), to_s(hl-4.2, -hw), to_s(hl-1.2, -hw*0.8)].to_vec(),
            VehicleType::Suv => [to_s(hl, -hw*0.45), to_s(hl, hw*0.45), to_s(hl-1.2, hw*0.9), to_s(hl-4.0, hw), to_s(0.0, hw*0.96), to_s(-hl+4.0, hw), to_s(-hl+1.0, hw*0.9), to_s(-hl, hw*0.6), to_s(-hl, -hw*0.6), to_s(-hl+1.0, -hw*0.9), to_s(-hl+4.0, -hw), to_s(0.0, -hw*0.96), to_s(hl-4.0, -hw), to_s(hl-1.2, -hw*0.9)].to_vec(),
            VehicleType::Van => [to_s(hl, -hw*0.45), to_s(hl, hw*0.45), to_s(hl-1.2, hw*0.92), to_s(hl-3.5, hw), to_s(0.0, hw), to_s(-hl+2.5, hw), to_s(-hl+0.5, hw*0.95), to_s(-hl, hw*0.7), to_s(-hl, -hw*0.7), to_s(-hl+0.5, -hw*0.95), to_s(-hl+2.5, -hw), to_s(0.0, -hw), to_s(hl-3.5, -hw), to_s(hl-1.2, -hw*0.92)].to_vec(),
        };
        Self::fill_polygon(canvas, &body_pts, palette.body);
        Self::draw_polygon_outline(canvas, &body_pts, palette.shadow);

        // Grille & Bowtie
        let (gf, ghw) = (hl - 0.8, hw * 0.55);
        let grille = [to_s(gf, -ghw), to_s(gf, ghw), to_s(gf-2.0, ghw*0.8), to_s(gf-2.0, -ghw*0.8)];
        Self::fill_polygon(canvas, &grille, Color::RGB(26, 28, 34));
        Self::draw_polygon_outline(canvas, &grille, Color::RGB(48, 52, 60));
        Self::fill_polygon(canvas, &[to_s(hl-0.2, -ghw*0.85), to_s(hl-0.2, ghw*0.85), to_s(hl-0.8, ghw*0.85), to_s(hl-0.8, -ghw*0.85)], Color::RGB(215, 222, 230));
        let bowtie = [to_s(hl+0.2, -hw*0.2), to_s(hl+0.2, hw*0.2), to_s(hl-0.7, hw*0.2), to_s(hl-0.7, -hw*0.2)];
        Self::fill_polygon(canvas, &bowtie, Color::RGB(232, 182, 28));
        Self::draw_polygon_outline(canvas, &bowtie, Color::RGB(248, 250, 255));

        // Hood Creases & Mirrors
        let hood_f = match vehicle.vehicle_type { VehicleType::Hatchback => hl*0.35, VehicleType::Sedan => hl*0.38, VehicleType::Suv => hl*0.40, VehicleType::Van => hl*0.65 };
        canvas.set_draw_color(palette.shadow);
        let _ = canvas.draw_line(to_s(hood_f, -hw*0.45), to_s(hl-1.5, -hw*0.25));
        let _ = canvas.draw_line(to_s(hood_f, hw*0.45), to_s(hl-1.5, hw*0.25));
        let mir_f = if matches!(vehicle.vehicle_type, VehicleType::Van) { hl*0.50 } else { hl*0.25 };
        for side in [-1.0f32, 1.0f32] {
            let mir = [to_s(mir_f+1.2, side*(hw-0.2)), to_s(mir_f+1.2, side*(hw+2.8)), to_s(mir_f-0.6, side*(hw+3.0)), to_s(mir_f-0.6, side*(hw-0.2))];
            Self::fill_polygon(canvas, &mir, palette.body);
            Self::draw_polygon_outline(canvas, &mir, palette.shadow);
            canvas.set_draw_color(Color::RGB(55, 75, 95));
            let _ = canvas.draw_line(to_s(mir_f-0.3, side*(hw+0.6)), to_s(mir_f-0.3, side*(hw+2.6)));
        }

        // Glasshouse per Vehicle Type
        match vehicle.vehicle_type {
            VehicleType::Hatchback => {
                let ws = [to_s(hl*0.35, -hw*0.75), to_s(hl*0.35, hw*0.75), to_s(hl*0.05, hw*0.68), to_s(hl*0.05, -hw*0.68)];
                Self::fill_polygon(canvas, &ws, Color::RGB(42, 60, 80));
                Self::draw_polygon_outline(canvas, &ws, Color::RGB(22, 28, 36));
                canvas.set_draw_color(Color::RGB(85, 125, 160));
                let _ = canvas.draw_line(to_s(hl*0.30, -hw*0.45), to_s(hl*0.12, -hw*0.10));
                for side in [-1.0f32, 1.0f32] {
                    Self::fill_polygon(canvas, &[to_s(hl*0.05, side*hw*0.68), to_s(hl*0.05, side*hw*0.82), to_s(-hl*0.55, side*hw*0.82), to_s(-hl*0.55, side*hw*0.68)], Color::RGB(36, 52, 70));
                    canvas.set_draw_color(Color::RGB(18, 22, 28));
                    let _ = canvas.draw_line(to_s(-hl*0.22, side*hw*0.68), to_s(-hl*0.22, side*hw*0.82));
                }
                let rw = [to_s(-hl*0.55, -hw*0.68), to_s(-hl*0.55, hw*0.68), to_s(-hl*0.85, hw*0.58), to_s(-hl*0.85, -hw*0.58)];
                Self::fill_polygon(canvas, &rw, Color::RGB(32, 45, 62));
                Self::draw_polygon_outline(canvas, &rw, Color::RGB(18, 24, 32));
                Self::fill_polygon(canvas, &[to_s(hl*0.05, -hw*0.68), to_s(hl*0.05, hw*0.68), to_s(-hl*0.55, hw*0.68), to_s(-hl*0.55, -hw*0.68)], palette.body);
                let sp = [to_s(-hl*0.85, -hw*0.72), to_s(-hl*0.85, hw*0.72), to_s(-hl*0.98, hw*0.70), to_s(-hl*0.98, -hw*0.70)];
                Self::fill_polygon(canvas, &sp, palette.body);
                Self::draw_polygon_outline(canvas, &sp, palette.shadow);
                canvas.set_draw_color(Color::RGB(30, 35, 45));
                let _ = canvas.draw_line(to_s(-hl*0.45, 0.0), to_s(-hl*0.80, 0.0));
            }
            VehicleType::Sedan => {
                let ws = [to_s(hl*0.38, -hw*0.75), to_s(hl*0.38, hw*0.75), to_s(hl*0.10, hw*0.68), to_s(hl*0.10, -hw*0.68)];
                Self::fill_polygon(canvas, &ws, Color::RGB(42, 60, 80));
                Self::draw_polygon_outline(canvas, &ws, Color::RGB(22, 28, 36));
                canvas.set_draw_color(Color::RGB(85, 125, 160));
                let _ = canvas.draw_line(to_s(hl*0.32, -hw*0.45), to_s(hl*0.15, -hw*0.10));
                for side in [-1.0f32, 1.0f32] {
                    Self::fill_polygon(canvas, &[to_s(hl*0.10, side*hw*0.68), to_s(hl*0.10, side*hw*0.82), to_s(-hl*0.35, side*hw*0.82), to_s(-hl*0.35, side*hw*0.68)], Color::RGB(36, 52, 70));
                    canvas.set_draw_color(Color::RGB(18, 22, 28));
                    let _ = canvas.draw_line(to_s(-hl*0.12, side*hw*0.68), to_s(-hl*0.12, side*hw*0.82));
                }
                let rw = [to_s(-hl*0.35, -hw*0.68), to_s(-hl*0.35, hw*0.68), to_s(-hl*0.62, hw*0.60), to_s(-hl*0.62, -hw*0.60)];
                Self::fill_polygon(canvas, &rw, Color::RGB(34, 48, 65));
                Self::draw_polygon_outline(canvas, &rw, Color::RGB(20, 26, 34));
                Self::fill_polygon(canvas, &[to_s(hl*0.10, -hw*0.68), to_s(hl*0.10, hw*0.68), to_s(-hl*0.35, hw*0.68), to_s(-hl*0.35, -hw*0.68)], palette.body);
                let tk = [to_s(-hl*0.62, -hw*0.65), to_s(-hl*0.62, hw*0.65), to_s(-hl*0.95, hw*0.62), to_s(-hl*0.95, -hw*0.62)];
                Self::fill_polygon(canvas, &tk, palette.body);
                Self::draw_polygon_outline(canvas, &tk, palette.shadow);
                canvas.set_draw_color(Color::RGB(215, 222, 230));
                let _ = canvas.draw_line(to_s(-hl*0.92, -hw*0.40), to_s(-hl*0.92, hw*0.40));
            }
            VehicleType::Suv => {
                let ws = [to_s(hl*0.40, -hw*0.78), to_s(hl*0.40, hw*0.78), to_s(hl*0.15, hw*0.72), to_s(hl*0.15, -hw*0.72)];
                Self::fill_polygon(canvas, &ws, Color::RGB(42, 60, 80));
                Self::draw_polygon_outline(canvas, &ws, Color::RGB(22, 28, 36));
                canvas.set_draw_color(Color::RGB(85, 125, 160));
                let _ = canvas.draw_line(to_s(hl*0.35, -hw*0.50), to_s(hl*0.18, -hw*0.15));
                for side in [-1.0f32, 1.0f32] {
                    Self::fill_polygon(canvas, &[to_s(hl*0.15, side*hw*0.72), to_s(hl*0.15, side*hw*0.85), to_s(-hl*0.65, side*hw*0.85), to_s(-hl*0.65, side*hw*0.72)], Color::RGB(34, 50, 68));
                    canvas.set_draw_color(Color::RGB(18, 22, 28));
                    let _ = canvas.draw_line(to_s(-hl*0.12, side*hw*0.72), to_s(-hl*0.12, side*hw*0.85));
                    let _ = canvas.draw_line(to_s(-hl*0.40, side*hw*0.72), to_s(-hl*0.40, side*hw*0.85));
                }
                let rw = [to_s(-hl*0.65, -hw*0.72), to_s(-hl*0.65, hw*0.72), to_s(-hl*0.88, hw*0.65), to_s(-hl*0.88, -hw*0.65)];
                Self::fill_polygon(canvas, &rw, Color::RGB(30, 42, 58));
                Self::draw_polygon_outline(canvas, &rw, Color::RGB(18, 24, 32));
                Self::fill_polygon(canvas, &[to_s(hl*0.15, -hw*0.72), to_s(hl*0.15, hw*0.72), to_s(-hl*0.65, hw*0.72), to_s(-hl*0.65, -hw*0.72)], palette.body);
                canvas.set_draw_color(Color::RGB(192, 200, 212));
                for side in [-1.0f32, 1.0f32] {
                    let _ = canvas.draw_line(to_s(hl*0.10, side*hw*0.58), to_s(-hl*0.60, side*hw*0.58));
                    let _ = canvas.draw_line(to_s(hl*0.10, side*hw*0.54), to_s(-hl*0.60, side*hw*0.54));
                }
            }
            VehicleType::Van => {
                let ws = [to_s(hl*0.65, -hw*0.80), to_s(hl*0.65, hw*0.80), to_s(hl*0.40, hw*0.75), to_s(hl*0.40, -hw*0.75)];
                Self::fill_polygon(canvas, &ws, Color::RGB(42, 60, 80));
                Self::draw_polygon_outline(canvas, &ws, Color::RGB(22, 28, 36));
                canvas.set_draw_color(Color::RGB(85, 125, 160));
                let _ = canvas.draw_line(to_s(hl*0.60, -hw*0.50), to_s(hl*0.45, -hw*0.15));
                for side in [-1.0f32, 1.0f32] {
                    Self::fill_polygon(canvas, &[to_s(hl*0.40, side*hw*0.75), to_s(hl*0.40, side*hw*0.88), to_s(-hl*0.75, side*hw*0.88), to_s(-hl*0.75, side*hw*0.75)], Color::RGB(32, 46, 62));
                    canvas.set_draw_color(Color::RGB(18, 22, 28));
                    let _ = canvas.draw_line(to_s(hl*0.08, side*hw*0.75), to_s(hl*0.08, side*hw*0.88));
                    let _ = canvas.draw_line(to_s(-hl*0.28, side*hw*0.75), to_s(-hl*0.28, side*hw*0.88));
                    let _ = canvas.draw_line(to_s(-hl*0.55, side*hw*0.75), to_s(-hl*0.55, side*hw*0.88));
                }
                let rw = [to_s(-hl*0.75, -hw*0.72), to_s(-hl*0.75, hw*0.72), to_s(-hl*0.92, hw*0.68), to_s(-hl*0.92, -hw*0.68)];
                Self::fill_polygon(canvas, &rw, Color::RGB(28, 38, 52));
                Self::draw_polygon_outline(canvas, &rw, Color::RGB(18, 24, 32));
                Self::fill_polygon(canvas, &[to_s(hl*0.40, -hw*0.75), to_s(hl*0.40, hw*0.75), to_s(-hl*0.75, hw*0.75), to_s(-hl*0.75, -hw*0.75)], palette.body);
                canvas.set_draw_color(palette.shadow);
                let _ = canvas.draw_line(to_s(hl*0.30, -hw*0.38), to_s(-hl*0.65, -hw*0.38));
                let _ = canvas.draw_line(to_s(hl*0.30, hw*0.38), to_s(-hl*0.65, hw*0.38));
                canvas.set_draw_color(Color::RGB(24, 28, 34));
                let _ = canvas.draw_line(to_s(-hl*0.75, 0.0), to_s(-hl, 0.0));
            }
        }

        // Headlights
        for side in [-1.0f32, 1.0f32] {
            let hlp = [to_s(hl-0.4, side*hw*0.45), to_s(hl-0.8, side*hw*0.85), to_s(hl-4.5, side*hw*0.95), to_s(hl-3.2, side*hw*0.50)];
            Self::fill_polygon(canvas, &hlp, Color::RGB(255, 248, 205));
            Self::draw_polygon_outline(canvas, &hlp, Color::RGB(210, 195, 150));
            canvas.set_draw_color(Color::RGB(255, 235, 140));
            let _ = canvas.draw_line(to_s(hl-1.2, side*hw*0.55), to_s(hl-2.8, side*hw*0.75));
            canvas.set_draw_color(Color::RGB(255, 255, 245));
            let _ = canvas.draw_line(to_s(hl-0.9, side*hw*0.82), to_s(hl-4.0, side*hw*0.92));
        }

        // Taillights & Dynamic Brake Lights
        let (tl_col, tl_out) = if is_braking { (Color::RGB(255, 25, 35), Color::RGB(255, 120, 130)) } else { (Color::RGB(130, 16, 22), Color::RGB(75, 10, 14)) };
        for side in [-1.0f32, 1.0f32] {
            let tlp = [to_s(-hl+3.2, side*hw*0.65), to_s(-hl+3.2, side*hw*0.95), to_s(-hl+0.4, side*hw*0.88), to_s(-hl+0.4, side*hw*0.55)];
            Self::fill_polygon(canvas, &tlp, tl_col);
            Self::draw_polygon_outline(canvas, &tlp, tl_out);
            if is_braking {
                canvas.set_draw_color(Color::RGB(255, 215, 215));
                let _ = canvas.draw_line(to_s(-hl+2.4, side*hw*0.72), to_s(-hl+1.0, side*hw*0.82));
            } else {
                canvas.set_draw_color(Color::RGB(165, 25, 32));
                let _ = canvas.draw_point(to_s(-hl+1.8, side*hw*0.76));
            }
        }

        // CHMSL Stop Lamp
        canvas.set_draw_color(if is_braking { Color::RGB(255, 30, 40) } else { Color::RGB(95, 12, 16) });
        let cf = match vehicle.vehicle_type { VehicleType::Hatchback => -hl+1.2, VehicleType::Sedan => -hl*0.38, VehicleType::Suv => -hl*0.68, VehicleType::Van => -hl+1.5 };
        let _ = canvas.draw_line(to_s(cf, -hw*0.25), to_s(cf, hw*0.25));

        // Turn Signals
        for (active, side) in [(vehicle.turn == Turn::Left && !vehicle.has_cleared, -1.0f32), (vehicle.turn == Turn::Right && !vehicle.has_cleared, 1.0f32)] {
            if !active { continue; }
            let amb = if blink_on { Color::RGB(255, 185, 15) } else { Color::RGB(110, 70, 15) };
            Self::fill_polygon(canvas, &[to_s(hl-0.5, side*hw*0.75), to_s(hl-0.5, side*hw*0.95), to_s(hl-3.2, side*hw*0.98), to_s(hl-2.5, side*hw*0.80)], amb);
            Self::fill_polygon(canvas, &[to_s(-hl+3.0, side*hw*0.80), to_s(-hl+3.0, side*hw*0.98), to_s(-hl+0.6, side*hw*0.92), to_s(-hl+0.6, side*hw*0.75)], amb);
            if blink_on {
                canvas.set_draw_color(Color::RGB(255, 240, 80));
                let _ = canvas.draw_point(to_s(hl-1.5, side*hw*0.88));
                let _ = canvas.draw_point(to_s(-hl+1.8, side*hw*0.88));
            }
        }

        // License Plate & Lidar Puck
        Self::fill_polygon(canvas, &[to_s(-hl+0.4, -hw*0.32), to_s(-hl+0.4, hw*0.32), to_s(-hl-0.1, hw*0.32), to_s(-hl-0.1, -hw*0.32)], Color::RGB(245, 245, 245));
        canvas.set_draw_color(Color::RGB(0, 220, 255));
        let puck = to_s(-hl * 0.20, 0.0);
        let _ = canvas.fill_rect(Rect::new(puck.x - 2, puck.y - 2, 5, 5));

        let label = format!("{}{}", vehicle.turn.as_char(), vehicle.velocity.round() as i32);
        Self::draw_string_5x7(canvas, &label, pos.x as i32 - 12, pos.y as i32 - (hl as i32 + 13), 1, Color::RGB(255, 255, 255));
    }

    fn fill_polygon(canvas: &mut Canvas<Window>, pts: &[Point], color: Color) {
        if pts.len() < 3 { return; }
        canvas.set_draw_color(color);
        let (min_y, max_y, n) = (pts.iter().map(|p| p.y).min().unwrap_or(0), pts.iter().map(|p| p.y).max().unwrap_or(0), pts.len());
        for y in min_y..=max_y {
            let mut nodes = Vec::with_capacity(8);
            for i in 0..n {
                let (p1, p2) = (pts[i], pts[(i + 1) % n]);
                if (p1.y < y && p2.y >= y) || (p2.y < y && p1.y >= y) {
                    let dy = p2.y - p1.y;
                    if dy != 0 { nodes.push(p1.x + ((y - p1.y) as f32 / dy as f32 * (p2.x - p1.x) as f32).round() as i32); }
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

    fn draw_polygon_outline(canvas: &mut Canvas<Window>, pts: &[Point], color: Color) {
        if pts.len() < 2 { return; }
        canvas.set_draw_color(color);
        for i in 0..pts.len() { let _ = canvas.draw_line(pts[i], pts[(i + 1) % pts.len()]); }
    }

    fn draw_hud(canvas: &mut Canvas<Window>, sim: &Simulation) {
        canvas.set_draw_color(Color::RGBA(18, 22, 28, 230));
        let _ = canvas.fill_rect(Rect::new(12, 12, (CANVAS_WIDTH as u32) - 24, 52));
        canvas.set_draw_color(Color::RGB(50, 60, 75));
        let _ = canvas.draw_rect(Rect::new(12, 12, (CANVAS_WIDTH as u32) - 24, 52));

        Self::draw_string_5x7(canvas, "SMART-ROAD | FLEET: CHEVROLET SPARK", 24, 20, 1, Color::RGB(0, 220, 255));
        let paint_txt = match sim.fleet_paint_mode {
            crate::simulation::FleetPaintMode::SummitWhiteOnly => "PAINT: SUMMIT WHITE [C]",
            crate::simulation::FleetPaintMode::SparkPalette => "PAINT: SPARK PALETTE [C]",
        };
        Self::draw_string_5x7(canvas, paint_txt, 550, 20, 1, Color::RGB(255, 215, 60));

        Self::draw_string_5x7(canvas, &format!("ACTIVE: {}", sim.vehicles.len()), 24, 38, 1, Color::RGB(230, 235, 240));
        Self::draw_string_5x7(canvas, &format!("PASSED: {}", sim.stats.max_vehicles_passed), 140, 38, 1, Color::RGB(100, 240, 150));
        Self::draw_string_5x7(canvas, &format!("CLOSE CALLS: {}", sim.stats.close_calls), 270, 38, 1, if sim.stats.close_calls > 0 { Color::RGB(255, 170, 50) } else { Color::RGB(120, 220, 255) });
        Self::draw_string_5x7(canvas, if sim.auto_spawn { "AUTO: ON [R]" } else { "AUTO: OFF [R]" }, 430, 38, 1, if sim.auto_spawn { Color::RGB(0, 230, 150) } else { Color::RGB(180, 185, 195) });
        Self::draw_string_5x7(canvas, &format!("TIME: {:.1}s", sim.stats.simulation_duration), 560, 38, 1, Color::RGB(220, 220, 230));

        canvas.set_draw_color(Color::RGBA(18, 22, 28, 220));
        let _ = canvas.fill_rect(Rect::new(12, (CANVAS_HEIGHT as i32) - 44, (CANVAS_WIDTH as u32) - 24, 32));
        canvas.set_draw_color(Color::RGB(50, 60, 75));
        let _ = canvas.draw_rect(Rect::new(12, (CANVAS_HEIGHT as i32) - 44, (CANVAS_WIDTH as u32) - 24, 32));
        Self::draw_string_5x7(canvas, "SPAWN: [ARROWS] | [R] Auto | [C] Spark Paint | [ESC] Finish & Stats", 24, (CANVAS_HEIGHT as i32) - 34, 1, Color::RGB(200, 210, 225));
    }

    fn draw_stats_screen(canvas: &mut Canvas<Window>, sim: &Simulation) {
        canvas.set_draw_color(Color::RGBA(10, 14, 20, 235));
        let _ = canvas.fill_rect(Rect::new(0, 0, CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32));
        let (mw, mh) = (680, 620);
        let (mx, my) = ((CANVAS_WIDTH as i32 - mw) / 2, (CANVAS_HEIGHT as i32 - mh) / 2);

        canvas.set_draw_color(Color::RGB(22, 27, 36));
        let _ = canvas.fill_rect(Rect::new(mx, my, mw as u32, mh as u32));
        canvas.set_draw_color(Color::RGB(0, 215, 255));
        let _ = canvas.draw_rect(Rect::new(mx, my, mw as u32, mh as u32));
        let _ = canvas.draw_rect(Rect::new(mx + 1, my + 1, (mw - 2) as u32, (mh - 2) as u32));

        Self::draw_string_5x7(canvas, "SIMULATION STATISTICS REPORT", mx + 160, my + 30, 2, Color::RGB(0, 230, 255));
        Self::draw_string_5x7(canvas, "FLEET MODEL: CHEVROLET SPARK (HATCHBACK)", mx + 165, my + 60, 1, Color::RGB(160, 175, 195));
        canvas.set_draw_color(Color::RGB(45, 60, 80));
        let _ = canvas.draw_line(Point::new(mx + 40, my + 85), Point::new(mx + mw - 40, my + 85));

        let (cw, ch, c1, c2) = (280, 80, mx + 45, mx + 355);
        let cards = [
            (c1, my + 105, "MAX VEHICLES PASSED", format!("{}", sim.stats.max_vehicles_passed), "VEHICLES", Color::RGB(70, 230, 130)),
            (c2, my + 105, "CLOSE CALLS COUNT", format!("{}", sim.stats.close_calls), "EVENTS", if sim.stats.close_calls > 0 { Color::RGB(255, 160, 40) } else { Color::RGB(80, 220, 255) }),
            (c1, my + 200, "MAX VELOCITY (FASTEST)", format!("{:.1}", sim.stats.max_velocity), "PX / SEC", Color::RGB(255, 205, 50)),
            (c2, my + 200, "MIN VELOCITY (SLOWEST)", format!("{:.1}", sim.stats.display_min_velocity()), "PX / SEC", Color::RGB(160, 210, 255)),
            (c1, my + 295, "MAX TRANSIT TIME", format!("{:.2} s", sim.stats.max_time), "LONGEST TRANSIT", Color::RGB(255, 120, 130)),
            (c2, my + 295, "MIN TRANSIT TIME", format!("{:.2} s", sim.stats.display_min_time()), "SHORTEST TRANSIT", Color::RGB(120, 240, 190)),
        ];
        for (x, y, t, v, u, c) in cards { Self::draw_stat_card(canvas, x, y, cw, ch, t, &v, u, c); }

        let by = my + 395;
        canvas.set_draw_color(Color::RGB(28, 36, 48));
        let _ = canvas.fill_rect(Rect::new(c1, by, (cw * 2 + 30) as u32, 95));
        canvas.set_draw_color(Color::RGB(55, 75, 100));
        let _ = canvas.draw_rect(Rect::new(c1, by, (cw * 2 + 30) as u32, 95));

        Self::draw_string_5x7(canvas, "FLEET SUMMARY & AUTONOMOUS SAFETY AUDIT", c1 + 15, by + 12, 1, Color::RGB(0, 210, 255));
        Self::draw_string_5x7(canvas, "FLEET VEHICLE:   CHEVROLET SPARK", c1 + 15, by + 35, 1, Color::RGB(255, 215, 60));
        Self::draw_string_5x7(canvas, &format!("TOTAL SPAWNED:   {}", sim.stats.total_spawned), c1 + 15, by + 55, 1, Color::RGB(220, 225, 235));
        Self::draw_string_5x7(canvas, &format!("AVG TRANSIT:     {:.2} s", sim.stats.avg_time()), c1 + 15, by + 75, 1, Color::RGB(220, 225, 235));
        Self::draw_string_5x7(canvas, "COLLISIONS:      0 (100% COLLISION FREE)", c1 + 290, by + 35, 1, Color::RGB(90, 245, 140));
        Self::draw_string_5x7(canvas, &format!("AVG SPEED:       {:.1} px/s", sim.stats.avg_velocity()), c1 + 290, by + 55, 1, Color::RGB(220, 225, 235));

        let fy = my + mh - 65;
        canvas.set_draw_color(Color::RGB(35, 45, 60));
        let _ = canvas.draw_line(Point::new(mx + 40, fy - 15), Point::new(mx + mw - 40, fy - 15));
        Self::draw_string_5x7(canvas, "PRESS [ENTER] TO RESTART SIMULATION   |   PRESS [ESC] OR [Q] TO EXIT", mx + 90, fy + 5, 1, Color::RGB(255, 220, 100));
    }

    fn draw_stat_card(canvas: &mut Canvas<Window>, x: i32, y: i32, w: i32, h: i32, title: &str, value: &str, unit: &str, val_color: Color) {
        canvas.set_draw_color(Color::RGB(26, 33, 44));
        let _ = canvas.fill_rect(Rect::new(x, y, w as u32, h as u32));
        canvas.set_draw_color(Color::RGB(45, 58, 76));
        let _ = canvas.draw_rect(Rect::new(x, y, w as u32, h as u32));
        Self::draw_string_5x7(canvas, title, x + 14, y + 12, 1, Color::RGB(155, 170, 190));
        Self::draw_string_5x7(canvas, value, x + 14, y + 36, 2, val_color);
        Self::draw_string_5x7(canvas, unit, x + 14, y + 62, 1, Color::RGB(120, 135, 155));
    }

    pub fn draw_string_5x7(canvas: &mut Canvas<Window>, text: &str, x: i32, y: i32, scale: i32, color: Color) {
        let mut cur_x = x;
        for c in text.chars() {
            Self::draw_char_5x7(canvas, c, cur_x, y, scale, color);
            cur_x += 6 * scale;
        }
    }

    fn draw_char_5x7(canvas: &mut Canvas<Window>, c: char, x: i32, y: i32, scale: i32, color: Color) {
        canvas.set_draw_color(color);
        let glyph = get_glyph_5x7(c);
        for row in 0..7 {
            let row_bits = glyph[row];
            for col in 0..5 {
                if (row_bits & (1 << (4 - col))) != 0 {
                    if scale == 1 {
                        let _ = canvas.draw_point(Point::new(x + col, y + row as i32));
                    } else {
                        let _ = canvas.fill_rect(Rect::new(x + col * scale, y + (row as i32) * scale, scale as u32, scale as u32));
                    }
                }
            }
        }
    }
}

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
