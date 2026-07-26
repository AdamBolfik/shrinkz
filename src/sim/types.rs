use std::time::Duration;

/// 2D vector in playfield space (origin top-left, y-down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(self) -> Self {
        let len = self.length();
        if len <= f32::EPSILON {
            Self::ZERO
        } else {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        }
    }

    pub fn mul(self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

/// Axis-aligned rectangle (origin top-left, y-down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains_point(self, p: Vec2) -> bool {
        p.x >= self.x
            && p.y >= self.y
            && p.x <= self.x + self.width
            && p.y <= self.y + self.height
    }

    pub fn area(self) -> f32 {
        self.width * self.height
    }

    pub fn left(self) -> f32 {
        self.x
    }

    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn top(self) -> f32 {
        self.y
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    pub fn center(self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Playing,
    Paused,
    LevelClear,
    GameOver,
}

/// Tunable constants for a Shrinkz build or settings profile.
#[derive(Debug, Clone, PartialEq)]
pub struct GameConfig {
    pub playfield: Rect,
    pub claim_ratio_to_clear: f32,
    pub wall_growth_speed: f32,
    pub ball_speed: f32,
    pub ball_radius: f32,
    pub wall_thickness: f32,
    pub max_balls: u32,
    pub timer_enabled: bool,
    pub level_time_limit: Duration,
    pub score_per_area_unit: u64,
    pub level_clear_bonus_base: u64,
    pub life_remaining_bonus: u64,
    pub time_remaining_bonus_per_second: u64,
    /// Grid resolution used for region claim flood-fill (cells across width).
    pub claim_grid_columns: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            playfield: Rect::new(0.0, 0.0, 800.0, 600.0),
            claim_ratio_to_clear: 0.75,
            wall_growth_speed: 220.0,
            ball_speed: 160.0,
            ball_radius: 8.0,
            wall_thickness: 6.0,
            max_balls: 50,
            timer_enabled: false,
            level_time_limit: Duration::from_secs(90),
            score_per_area_unit: 1,
            level_clear_bonus_base: 500,
            life_remaining_bonus: 100,
            time_remaining_bonus_per_second: 10,
            claim_grid_columns: 240,
        }
    }
}

/// Discrete player or system intents the simulation understands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameCommand {
    StartWall { origin: Vec2, axis: Axis },
    Pause,
    Resume,
    RestartLevel,
    RestartGame,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallView {
    pub position: Vec2,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallView {
    pub axis: Axis,
    /// Fixed coordinate: y for horizontal walls, x for vertical walls.
    pub fixed: f32,
    pub start: f32,
    pub end: f32,
    pub thickness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimerView {
    pub remaining: Duration,
}

/// Read-only view of the session for rendering and HUD.
#[derive(Debug, Clone, PartialEq)]
pub struct GameSnapshot {
    pub phase: Phase,
    pub level: u32,
    pub lives: u32,
    pub score: u64,
    pub balls: Vec<BallView>,
    pub walls: Vec<WallView>,
    /// Solid filled territory (physics). Renderer usually paints the whole playfield
    /// as claimed and carves `free` on top instead of drawing these rects.
    pub claimed: Vec<Rect>,
    /// Remaining open chambers where balls bounce.
    pub free: Vec<Rect>,
    pub claimed_ratio: f32,
    pub wall_in_progress: Option<WallView>,
    pub timer: Option<TimerView>,
    pub playfield: Rect,
}
