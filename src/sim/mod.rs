//! Pure game simulation for Shrinkz — no engine dependencies.

mod geometry;
mod session;
mod types;

pub use session::GameSession;
pub use types::{
    Axis, GameCommand, GameConfig, GameSnapshot, Phase, Rect, Vec2, WallView,
};
