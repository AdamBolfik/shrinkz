//! Shrinkz game library — pure simulation and shared types.

pub mod sim;

pub use sim::{
    Axis, GameCommand, GameConfig, GameSession, GameSnapshot, Phase, Rect, Vec2, WallView,
};
