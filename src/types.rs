use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point2D) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::ZERO
        }
    }

    pub fn lerp(&self, other: &Point2D, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl Add for Point2D {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point2D {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    North,
    South,
    East,
    West,
}

impl Origin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Origin::North => "North",
            Origin::South => "South",
            Origin::East => "East",
            Origin::West => "West",
        }
    }

    pub fn incoming_heading(&self) -> f32 {
        match self {
            Origin::North => 90.0,   // Facing down (+y)
            Origin::South => 270.0,  // Facing up (-y)
            Origin::West => 0.0,     // Facing right (+x)
            Origin::East => 180.0,   // Facing left (-x)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Turn {
    Right,
    Straight,
    Left,
}

impl Turn {
    pub fn as_char(&self) -> char {
        match self {
            Turn::Right => 'r',
            Turn::Straight => 's',
            Turn::Left => 'l',
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Turn::Right => "Right (r)",
            Turn::Straight => "Straight (s)",
            Turn::Left => "Left (l)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VelocityTier {
    Stopped,
    Slow,
    Medium,
    Fast,
}

impl VelocityTier {
    pub fn speed(&self) -> f32 {
        match self {
            VelocityTier::Stopped => 0.0,
            VelocityTier::Slow => 55.0,
            VelocityTier::Medium => 105.0,
            VelocityTier::Fast => 165.0,
        }
    }

    pub fn all_speeds() -> [f32; 3] {
        [55.0, 105.0, 165.0]
    }
}

// Canvas & geometric constants
pub const CANVAS_WIDTH: f32 = 900.0;
pub const CANVAS_HEIGHT: f32 = 900.0;
pub const CENTER_X: f32 = CANVAS_WIDTH / 2.0;   // 450.0
pub const CENTER_Y: f32 = CANVAS_HEIGHT / 2.0;  // 450.0

pub const LANE_WIDTH: f32 = 36.0;
pub const NUM_LANES_PER_DIR: usize = 3;
pub const HALF_ROAD_WIDTH: f32 = (NUM_LANES_PER_DIR as f32) * LANE_WIDTH; // 108.0

pub const VEHICLE_LENGTH: f32 = 36.0;
pub const VEHICLE_WIDTH: f32 = 20.0;

// Safety and detection margins
pub const SAFETY_DISTANCE: f32 = 32.0;
pub const CLOSE_CALL_THRESHOLD: f32 = SAFETY_DISTANCE;
pub const DETECTION_DISTANCE_AHEAD: f32 = 200.0; // Distance before stop-line where smart intersection manages vehicle

// Physics acceleration & deceleration rates
pub const DEFAULT_ACCEL: f32 = 120.0; // px / s^2
pub const DEFAULT_DECEL: f32 = 250.0; // px / s^2

// Vehicle make and model specification
pub const CAR_MAKE: &str = "Chevrolet";
pub const CAR_MODEL: &str = "Spark";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VehicleType {
    Hatchback,
    Sedan,
    Suv,
    Van,
}

impl VehicleType {
    /// Returns (width, length) in pixels as specified:
    /// - Hatchback: 16x25 px
    /// - Sedan: 17x30 px
    /// - Suv: 19x33 px
    /// - Van: 20x38 px
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            VehicleType::Hatchback => (16.0, 25.0),
            VehicleType::Sedan => (17.0, 30.0),
            VehicleType::Suv => (19.0, 33.0),
            VehicleType::Van => (20.0, 38.0),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            VehicleType::Hatchback => "Hatchback",
            VehicleType::Sedan => "Sedan",
            VehicleType::Suv => "SUV",
            VehicleType::Van => "Van",
        }
    }

    pub fn from_id(id: usize) -> Self {
        match id % 4 {
            0 => VehicleType::Hatchback,
            1 => VehicleType::Sedan,
            2 => VehicleType::Suv,
            _ => VehicleType::Van,
        }
    }
}

