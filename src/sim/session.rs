use std::time::Duration;

use super::geometry::{
    bounce_ball, circle_hits_growing_wall_half, compute_claim_state, free_axis_limits, GrowingWall,
    SolidWall,
};
use super::types::{
    Axis, BallView, GameCommand, GameConfig, GameSnapshot, Phase, Rect, TimerView, Vec2, WallView,
};

#[derive(Debug, Clone)]
struct Ball {
    position: Vec2,
    velocity: Vec2,
}

/// Owns one Shrinkz play session from start through game over.
#[derive(Debug, Clone)]
pub struct GameSession {
    config: GameConfig,
    phase: Phase,
    level: u32,
    lives: u32,
    score: u64,
    balls: Vec<Ball>,
    solids: Vec<SolidWall>,
    claimed: Vec<Rect>,
    free: Vec<Rect>,
    claimed_ratio: f32,
    growing: Option<GrowingWall>,
    timer_remaining: Option<Duration>,
    /// Claimed ratio before last claim, for score deltas.
    previous_claimed_ratio: f32,
}

impl GameSession {
    /// Create a fresh session at level 1.
    pub fn new(config: GameConfig) -> Self {
        let mut session = Self {
            config,
            phase: Phase::Playing,
            level: 1,
            lives: 0,
            score: 0,
            balls: Vec::new(),
            solids: Vec::new(),
            claimed: Vec::new(),
            free: Vec::new(),
            claimed_ratio: 0.0,
            growing: None,
            timer_remaining: None,
            previous_claimed_ratio: 0.0,
        };
        session.begin_level(1, true);
        session
    }

    /// Advance simulation by `dt`, optionally applying one command first.
    pub fn apply(&mut self, command: Option<GameCommand>, dt: Duration) {
        if let Some(cmd) = command {
            self.apply_command(cmd);
        }

        match self.phase {
            Phase::Playing => self.tick(dt),
            Phase::Paused => {}
            Phase::LevelClear => {
                self.advance_after_clear();
            }
            Phase::GameOver => {}
        }
    }

    /// Read-only view for rendering and HUD.
    pub fn snapshot(&self) -> GameSnapshot {
        let balls = self
            .balls
            .iter()
            .map(|b| BallView {
                position: b.position,
                radius: self.config.ball_radius,
            })
            .collect();

        let walls: Vec<WallView> = self
            .solids
            .iter()
            .map(|w| w.to_view(self.config.wall_thickness))
            .collect();

        let wall_in_progress = self
            .growing
            .as_ref()
            .and_then(|g| g.to_view(self.config.wall_thickness));

        let timer = if self.config.timer_enabled {
            self.timer_remaining.map(|remaining| TimerView { remaining })
        } else {
            None
        };

        GameSnapshot {
            phase: self.phase,
            level: self.level,
            lives: self.lives,
            score: self.score,
            balls,
            walls,
            claimed: self.claimed.clone(),
            free: self.free.clone(),
            claimed_ratio: self.claimed_ratio,
            wall_in_progress,
            timer,
            playfield: self.config.playfield,
        }
    }

    fn apply_command(&mut self, command: GameCommand) {
        match command {
            GameCommand::StartWall { origin, axis } => {
                if self.phase != Phase::Playing {
                    return;
                }
                if self.growing.is_some() {
                    return;
                }
                if !self.config.playfield.contains_point(origin) {
                    return;
                }
                // Do not start a wall inside already-claimed space
                if self.point_in_claimed(origin) {
                    return;
                }
                self.growing = Some(GrowingWall::new(origin, axis));
            }
            GameCommand::Pause => {
                if self.phase == Phase::Playing {
                    self.phase = Phase::Paused;
                }
            }
            GameCommand::Resume => {
                if self.phase == Phase::Paused {
                    self.phase = Phase::Playing;
                }
            }
            GameCommand::RestartLevel => {
                let level = self.level;
                let score = self.score;
                self.begin_level(level, false);
                self.score = score;
                self.phase = Phase::Playing;
            }
            GameCommand::RestartGame => {
                let config = self.config.clone();
                *self = Self::new(config);
            }
        }
    }

    fn tick(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();
        if dt_secs <= 0.0 {
            return;
        }

        if self.config.timer_enabled {
            if let Some(remaining) = self.timer_remaining.as_mut() {
                if *remaining <= dt {
                    *remaining = Duration::ZERO;
                    self.phase = Phase::GameOver;
                    return;
                }
                *remaining = remaining.saturating_sub(dt);
            }
        }

        // Substep wall growth so a ball cannot tunnel through a segment without a hit.
        self.grow_walls_with_hit_checks(dt_secs);
        self.commit_finished_wall_halves();
        self.tick_balls(dt_secs);
        self.resolve_wall_hits();
        self.commit_finished_wall_halves();
        self.clear_growing_if_resolved();
        self.refresh_claims(true);
        self.check_level_clear();
    }

    fn tick_balls(&mut self, dt_secs: f32) {
        let playfield = self.effective_play_bounds();
        let solids = self.solids.clone();
        let thickness = self.config.wall_thickness;
        let radius = self.config.ball_radius;

        for ball in &mut self.balls {
            let (pos, vel) = bounce_ball(
                ball.position,
                ball.velocity,
                radius,
                playfield,
                &solids,
                thickness,
                dt_secs,
            );
            // Stay out of filled territory (bounce on free-chamber boundary).
            let (pos, vel) = bounce_against_claimed(pos, vel, radius, &self.claimed, dt_secs);
            ball.position = pos;
            ball.velocity = vel;
        }
    }

    /// Grow each unfinished half in small steps; after every step, destroy any half
    /// a ball is touching (classic: hit before the segment reaches a wall → life lost).
    fn grow_walls_with_hit_checks(&mut self, dt_secs: f32) {
        if self.growing.is_none() {
            return;
        }
        let total = self.config.wall_growth_speed * dt_secs;
        if total <= 0.0 {
            return;
        }
        // ~2px substeps so balls cannot skip through a segment in one jump.
        let substep = 2.0_f32;
        let steps = (total / substep).ceil().max(1.0) as u32;
        let each = total / steps as f32;
        for _ in 0..steps {
            self.grow_walls_by(each);
            self.resolve_wall_hits();
            if self.growing.is_none() || self.phase == Phase::GameOver {
                break;
            }
            // Stop growing a half that just finished this substep after hit checks.
            if self
                .growing
                .as_ref()
                .is_some_and(|g| g.is_fully_resolved())
            {
                break;
            }
        }
    }

    fn grow_walls_by(&mut self, distance: f32) {
        let Some(growing) = self.growing.as_mut() else {
            return;
        };
        // Only solids + playfield + free-chamber bounds. Do NOT clamp against claimed
        // rects: those are rebuilt every frame and can falsely bleed into open chambers,
        // which marked halves "done" mid-air with no life loss.
        let (neg_max, pos_max) = free_axis_limits(
            growing.origin,
            growing.axis,
            self.config.playfield,
            &self.solids,
            self.config.wall_thickness,
        );
        let (neg_max, pos_max) =
            clamp_growth_to_free_chamber(growing, neg_max, pos_max, &self.free);

        if growing.neg_alive && !growing.neg_done && !growing.neg_committed {
            growing.neg_extent = (growing.neg_extent + distance).min(neg_max);
            if growing.neg_extent >= neg_max - 0.01 {
                growing.neg_extent = neg_max;
                growing.neg_done = true;
            }
        }
        if growing.pos_alive && !growing.pos_done && !growing.pos_committed {
            growing.pos_extent = (growing.pos_extent + distance).min(pos_max);
            if growing.pos_extent >= pos_max - 0.01 {
                growing.pos_extent = pos_max;
                growing.pos_done = true;
            }
        }
    }

    /// Promote any half that has reached solid geometry into permanent walls.
    /// Completed halves bounce balls; they are never destroyed by hits.
    fn commit_finished_wall_halves(&mut self) {
        let parts = {
            let Some(growing) = self.growing.as_mut() else {
                return;
            };
            let mut parts = Vec::new();
            // Mark done even when extent is zero (origin already on a boundary).
            if growing.neg_alive && growing.neg_done && !growing.neg_committed {
                if let Some(part) = growing.uncommitted_solid_half(true) {
                    parts.push(part);
                }
                growing.neg_committed = true;
            }
            if growing.pos_alive && growing.pos_done && !growing.pos_committed {
                if let Some(part) = growing.uncommitted_solid_half(false) {
                    parts.push(part);
                }
                growing.pos_committed = true;
            }
            parts
        };
        self.solids.extend(parts);
    }

    fn resolve_wall_hits(&mut self) {
        let Some(growing) = self.growing.as_ref() else {
            return;
        };
        let radius = self.config.ball_radius;
        let thickness = self.config.wall_thickness;

        let mut hit_neg = false;
        let mut hit_pos = false;
        for ball in &self.balls {
            if circle_hits_growing_wall_half(ball.position, radius, growing, true, thickness) {
                hit_neg = true;
            }
            if circle_hits_growing_wall_half(ball.position, radius, growing, false, thickness) {
                hit_pos = true;
            }
        }

        if !hit_neg && !hit_pos {
            return;
        }

        let mut lives_to_lose = 0u32;
        if let Some(growing) = self.growing.as_mut() {
            // Destroy only unfinished halves; committed solids stay (already in solids).
            if hit_neg && growing.neg_alive && !growing.neg_done && !growing.neg_committed {
                growing.neg_alive = false;
                growing.neg_extent = 0.0;
                lives_to_lose += 1;
            }
            if hit_pos && growing.pos_alive && !growing.pos_done && !growing.pos_committed {
                growing.pos_alive = false;
                growing.pos_extent = 0.0;
                lives_to_lose += 1;
            }
        }

        for _ in 0..lives_to_lose {
            self.lose_life();
            if self.phase == Phase::GameOver {
                break;
            }
        }

        if self.phase == Phase::GameOver {
            // Keep any halves already committed as solid walls; drop the in-progress remainder.
            self.growing = None;
        }
    }

    fn clear_growing_if_resolved(&mut self) {
        let resolved = self
            .growing
            .as_ref()
            .is_some_and(|g| g.is_fully_resolved());
        if resolved {
            self.growing = None;
        }
    }

    fn refresh_claims(&mut self, award_score: bool) {
        let ball_data: Vec<(Vec2, f32)> = self
            .balls
            .iter()
            .map(|b| (b.position, self.config.ball_radius))
            .collect();

        // Only committed solids seal chambers. Unfinished growing segments must not
        // count as barriers — they can false-seal pockets and create phantom claimed
        // that then freezes wall growth mid-air.
        let state = compute_claim_state(
            self.config.playfield,
            &self.solids,
            &ball_data,
            self.config.wall_thickness,
            self.config.claim_grid_columns,
        );

        if award_score && state.claimed_ratio > self.previous_claimed_ratio {
            let area_delta =
                (state.claimed_ratio - self.previous_claimed_ratio) * self.config.playfield.area();
            let points = (area_delta.max(0.0) * self.config.score_per_area_unit as f32) as u64;
            self.score = self.score.saturating_add(points);
        }

        self.claimed = state.claimed;
        self.free = state.free;
        self.claimed_ratio = state.claimed_ratio;
        self.previous_claimed_ratio = state.claimed_ratio;
    }

    fn check_level_clear(&mut self) {
        if self.phase != Phase::Playing {
            return;
        }
        if self.claimed_ratio + f32::EPSILON >= self.config.claim_ratio_to_clear {
            self.award_level_clear_bonus();
            self.phase = Phase::LevelClear;
        }
    }

    fn award_level_clear_bonus(&mut self) {
        let clear = self.config.level_clear_bonus_base.saturating_mul(self.level as u64);
        let lives = self
            .config
            .life_remaining_bonus
            .saturating_mul(self.lives as u64);
        let mut bonus = clear.saturating_add(lives);
        if self.config.timer_enabled {
            if let Some(remaining) = self.timer_remaining {
                let secs = remaining.as_secs();
                bonus = bonus.saturating_add(
                    secs.saturating_mul(self.config.time_remaining_bonus_per_second),
                );
            }
        }
        self.score = self.score.saturating_add(bonus);
    }

    fn advance_after_clear(&mut self) {
        let next = self.level.saturating_add(1);
        self.begin_level(next, false);
        self.phase = Phase::Playing;
    }

    fn lose_life(&mut self) {
        if self.lives == 0 {
            self.phase = Phase::GameOver;
            return;
        }
        self.lives -= 1;
        if self.lives == 0 {
            self.phase = Phase::GameOver;
        }
    }

    fn begin_level(&mut self, level: u32, reset_score: bool) {
        self.level = level.max(1);
        let ball_count = self.ball_count_for_level(self.level);
        self.lives = ball_count;
        self.balls = spawn_balls(
            ball_count,
            self.config.playfield,
            self.config.ball_speed,
            self.config.ball_radius,
        );
        self.solids.clear();
        self.claimed.clear();
        self.free.clear();
        self.claimed_ratio = 0.0;
        self.previous_claimed_ratio = 0.0;
        self.growing = None;
        if reset_score {
            self.score = 0;
        }
        self.timer_remaining = if self.config.timer_enabled {
            Some(self.config.level_time_limit)
        } else {
            None
        };
        self.phase = Phase::Playing;
        // Establish free chamber covering the playfield before the first frame.
        self.refresh_claims(false);
    }

    fn ball_count_for_level(&self, level: u32) -> u32 {
        level.min(self.config.max_balls).max(1)
    }

    fn effective_play_bounds(&self) -> Rect {
        self.config.playfield
    }

    fn point_in_claimed(&self, p: Vec2) -> bool {
        if !self.config.playfield.contains_point(p) {
            return false;
        }
        // Claimed if not inside any open free chamber.
        if self.free.is_empty() && self.claimed.is_empty() {
            return false;
        }
        !self.free.iter().any(|r| r.contains_point(p))
    }
}

fn spawn_balls(count: u32, playfield: Rect, speed: f32, radius: f32) -> Vec<Ball> {
    let mut balls = Vec::with_capacity(count as usize);
    let cx = playfield.center().x;
    let cy = playfield.center().y;
    // Classic-style diagonal velocities at 45° multiples
    let dirs = [
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, 0.5),
        Vec2::new(-0.5, 1.0),
        Vec2::new(0.5, -1.0),
        Vec2::new(-1.0, -0.5),
    ];
    for i in 0..count {
        let dir = dirs[(i as usize) % dirs.len()].normalized().mul(speed);
        // Spread spawn slightly so balls don't stack
        let ox = ((i % 5) as f32 - 2.0) * (radius * 3.0);
        let oy = ((i / 5) as f32 - 1.0) * (radius * 3.0);
        let mut pos = Vec2::new(cx + ox, cy + oy);
        pos.x = pos
            .x
            .clamp(playfield.left() + radius, playfield.right() - radius);
        pos.y = pos
            .y
            .clamp(playfield.top() + radius, playfield.bottom() - radius);
        balls.push(Ball {
            position: pos,
            velocity: dir,
        });
    }
    balls
}

fn bounce_against_claimed(
    mut pos: Vec2,
    mut vel: Vec2,
    radius: f32,
    claimed: &[Rect],
    _dt: f32,
) -> (Vec2, Vec2) {
    for rect in claimed {
        // Expand rect by radius for circle-AABB
        let left = rect.left() - radius;
        let right = rect.right() + radius;
        let top = rect.top() - radius;
        let bottom = rect.bottom() + radius;
        if pos.x < left || pos.x > right || pos.y < top || pos.y > bottom {
            continue;
        }
        // Push out along smallest penetration
        let pen_left = pos.x - left;
        let pen_right = right - pos.x;
        let pen_top = pos.y - top;
        let pen_bottom = bottom - pos.y;
        let min_pen = pen_left.min(pen_right).min(pen_top).min(pen_bottom);
        if (min_pen - pen_left).abs() < f32::EPSILON {
            pos.x = left;
            vel.x = -vel.x.abs();
        } else if (min_pen - pen_right).abs() < f32::EPSILON {
            pos.x = right;
            vel.x = vel.x.abs();
        } else if (min_pen - pen_top).abs() < f32::EPSILON {
            pos.y = top;
            vel.y = -vel.y.abs();
        } else {
            pos.y = bottom;
            vel.y = vel.y.abs();
        }
    }
    (pos, vel)
}

/// Limit growth to the free chamber that contains the click origin.
///
/// Free rects come from ball flood-fill (open space). Using them avoids phantom
/// claimed geometry stopping a half mid-chamber without a ball hit.
fn clamp_growth_to_free_chamber(
    growing: &GrowingWall,
    mut neg_max: f32,
    mut pos_max: f32,
    free: &[Rect],
) -> (f32, f32) {
    let origin = growing.origin;
    let mut found = false;
    // Union of free rects that contain the origin (handles adjacent free tiles).
    let mut min_along = f32::MAX;
    let mut max_along = f32::MIN;

    match growing.axis {
        Axis::Horizontal => {
            for rect in free {
                if !rect.contains_point(origin)
                    && !(origin.y >= rect.top()
                        && origin.y <= rect.bottom()
                        && origin.x >= rect.left() - 0.5
                        && origin.x <= rect.right() + 0.5)
                {
                    continue;
                }
                // Origin on this free band (y overlap is enough for horizontal growth
                // if x is near the rect — prefer strict contains).
                if origin.y < rect.top() || origin.y > rect.bottom() {
                    continue;
                }
                if origin.x < rect.left() - 1.0 || origin.x > rect.right() + 1.0 {
                    continue;
                }
                found = true;
                min_along = min_along.min(rect.left());
                max_along = max_along.max(rect.right());
            }
            if found {
                neg_max = neg_max.min((origin.x - min_along).max(0.0));
                pos_max = pos_max.min((max_along - origin.x).max(0.0));
            }
        }
        Axis::Vertical => {
            for rect in free {
                if origin.x < rect.left() || origin.x > rect.right() {
                    continue;
                }
                if origin.y < rect.top() - 1.0 || origin.y > rect.bottom() + 1.0 {
                    continue;
                }
                found = true;
                min_along = min_along.min(rect.top());
                max_along = max_along.max(rect.bottom());
            }
            if found {
                neg_max = neg_max.min((origin.y - min_along).max(0.0));
                pos_max = pos_max.min((max_along - origin.y).max(0.0));
            }
        }
    }
    (neg_max, pos_max)
}
