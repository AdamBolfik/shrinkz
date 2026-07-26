use super::types::{Axis, Rect, Vec2, WallView};

/// Completed solid wall segment (axis-aligned).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidWall {
    pub axis: Axis,
    pub fixed: f32,
    pub start: f32,
    pub end: f32,
}

impl SolidWall {
    pub fn to_view(self, thickness: f32) -> WallView {
        WallView {
            axis: self.axis,
            fixed: self.fixed,
            start: self.start.min(self.end),
            end: self.start.max(self.end),
            thickness,
        }
    }

    pub fn as_rect(self, thickness: f32) -> Rect {
        let half = thickness * 0.5;
        let start = self.start.min(self.end);
        let end = self.start.max(self.end);
        match self.axis {
            Axis::Horizontal => {
                Rect::new(start, self.fixed - half, (end - start).max(0.0), thickness)
            }
            Axis::Vertical => {
                Rect::new(self.fixed - half, start, thickness, (end - start).max(0.0))
            }
        }
    }
}

/// Bidirectional wall currently growing from an origin.
///
/// Each half is independent:
/// - while still extending, a ball hit destroys that half and costs a life
/// - once a half reaches solid geometry it is committed as a permanent solid (balls bounce)
#[derive(Debug, Clone, PartialEq)]
pub struct GrowingWall {
    pub origin: Vec2,
    pub axis: Axis,
    pub neg_extent: f32,
    pub pos_extent: f32,
    pub neg_done: bool,
    pub pos_done: bool,
    pub neg_alive: bool,
    pub pos_alive: bool,
    /// Already written into the permanent solid wall list.
    pub neg_committed: bool,
    pub pos_committed: bool,
}

impl GrowingWall {
    pub fn new(origin: Vec2, axis: Axis) -> Self {
        Self {
            origin,
            axis,
            neg_extent: 0.0,
            pos_extent: 0.0,
            neg_done: false,
            pos_done: false,
            neg_alive: true,
            pos_alive: true,
            neg_committed: false,
            pos_committed: false,
        }
    }

    /// True when each half is either destroyed or committed as permanent solid.
    pub fn is_fully_resolved(&self) -> bool {
        (!self.neg_alive || self.neg_committed) && (!self.pos_alive || self.pos_committed)
    }

    pub fn neg_range(&self) -> (f32, f32) {
        match self.axis {
            Axis::Horizontal => (self.origin.x - self.neg_extent, self.origin.x),
            Axis::Vertical => (self.origin.y - self.neg_extent, self.origin.y),
        }
    }

    pub fn pos_range(&self) -> (f32, f32) {
        match self.axis {
            Axis::Horizontal => (self.origin.x, self.origin.x + self.pos_extent),
            Axis::Vertical => (self.origin.y, self.origin.y + self.pos_extent),
        }
    }

    pub fn fixed(&self) -> f32 {
        match self.axis {
            Axis::Horizontal => self.origin.y,
            Axis::Vertical => self.origin.x,
        }
    }

    /// In-progress halves only (committed halves are drawn from the solid wall list).
    pub fn to_view(&self, thickness: f32) -> Option<WallView> {
        let origin_along = match self.axis {
            Axis::Horizontal => self.origin.x,
            Axis::Vertical => self.origin.y,
        };

        let mut min_v = f32::MAX;
        let mut max_v = f32::MIN;
        let mut any = false;

        if self.neg_alive && !self.neg_committed && self.neg_extent > 0.0 {
            min_v = min_v.min(origin_along - self.neg_extent);
            max_v = max_v.max(origin_along);
            any = true;
        }
        if self.pos_alive && !self.pos_committed && self.pos_extent > 0.0 {
            min_v = min_v.min(origin_along);
            max_v = max_v.max(origin_along + self.pos_extent);
            any = true;
        }

        if !any || (max_v - min_v) <= f32::EPSILON {
            return None;
        }

        Some(WallView {
            axis: self.axis,
            fixed: self.fixed(),
            start: min_v,
            end: max_v,
            thickness,
        })
    }

    /// Solid segment for a completed half that has not yet been committed.
    pub fn uncommitted_solid_half(&self, negative: bool) -> Option<SolidWall> {
        if negative {
            if !(self.neg_alive && self.neg_done && !self.neg_committed) {
                return None;
            }
            let (a, b) = self.neg_range();
            if (b - a).abs() <= f32::EPSILON {
                return None;
            }
            Some(SolidWall {
                axis: self.axis,
                fixed: self.fixed(),
                start: a,
                end: b,
            })
        } else {
            if !(self.pos_alive && self.pos_done && !self.pos_committed) {
                return None;
            }
            let (a, b) = self.pos_range();
            if (b - a).abs() <= f32::EPSILON {
                return None;
            }
            Some(SolidWall {
                axis: self.axis,
                fixed: self.fixed(),
                start: a,
                end: b,
            })
        }
    }

}

/// Max growth distances from origin along free axis until solid or playfield edge.
pub fn free_axis_limits(
    origin: Vec2,
    axis: Axis,
    playfield: Rect,
    solids: &[SolidWall],
    thickness: f32,
) -> (f32, f32) {
    let half_t = thickness * 0.5;
    match axis {
        Axis::Horizontal => {
            let y = origin.y;
            let mut neg = origin.x - playfield.left();
            let mut pos = playfield.right() - origin.x;
            for w in solids {
                let rect = w.as_rect(thickness);
                if y + half_t < rect.top() || y - half_t > rect.bottom() {
                    continue;
                }
                if rect.right() <= origin.x + 0.01 {
                    neg = neg.min((origin.x - rect.right()).max(0.0));
                }
                if rect.left() >= origin.x - 0.01 {
                    pos = pos.min((rect.left() - origin.x).max(0.0));
                }
            }
            (neg.max(0.0), pos.max(0.0))
        }
        Axis::Vertical => {
            let x = origin.x;
            let mut neg = origin.y - playfield.top();
            let mut pos = playfield.bottom() - origin.y;
            for w in solids {
                let rect = w.as_rect(thickness);
                if x + half_t < rect.left() || x - half_t > rect.right() {
                    continue;
                }
                if rect.bottom() <= origin.y + 0.01 {
                    neg = neg.min((origin.y - rect.bottom()).max(0.0));
                }
                if rect.top() >= origin.y - 0.01 {
                    pos = pos.min((rect.top() - origin.y).max(0.0));
                }
            }
            (neg.max(0.0), pos.max(0.0))
        }
    }
}

/// Collision with a still-growing (not yet solid) wall half — destroys on hit.
pub fn circle_hits_growing_wall_half(
    center: Vec2,
    radius: f32,
    wall: &GrowingWall,
    half_negative: bool,
    thickness: f32,
) -> bool {
    if half_negative {
        if !wall.neg_alive || wall.neg_done || wall.neg_committed || wall.neg_extent <= 0.0 {
            return false;
        }
    } else if !wall.pos_alive || wall.pos_done || wall.pos_committed || wall.pos_extent <= 0.0 {
        return false;
    }
    let (start, end) = if half_negative {
        wall.neg_range()
    } else {
        wall.pos_range()
    };
    if (end - start).abs() < f32::EPSILON {
        return false;
    }
    let solid = SolidWall {
        axis: wall.axis,
        fixed: wall.fixed(),
        start,
        end,
    };
    circle_hits_solid(center, radius, solid, thickness)
}

pub fn circle_hits_solid(center: Vec2, radius: f32, wall: SolidWall, thickness: f32) -> bool {
    let rect = wall.as_rect(thickness);
    let closest = closest_point_on_rect(center, rect);
    let dx = center.x - closest.x;
    let dy = center.y - closest.y;
    dx * dx + dy * dy <= radius * radius
}

fn closest_point_on_rect(p: Vec2, r: Rect) -> Vec2 {
    Vec2::new(
        p.x.clamp(r.left(), r.right()),
        p.y.clamp(r.top(), r.bottom()),
    )
}

/// Advance a ball with edge and solid-wall bounces.
pub fn bounce_ball(
    mut pos: Vec2,
    mut vel: Vec2,
    radius: f32,
    playfield: Rect,
    solids: &[SolidWall],
    thickness: f32,
    dt: f32,
) -> (Vec2, Vec2) {
    pos = pos.add(vel.mul(dt));

    if pos.x - radius < playfield.left() {
        pos.x = playfield.left() + radius;
        vel.x = vel.x.abs();
    } else if pos.x + radius > playfield.right() {
        pos.x = playfield.right() - radius;
        vel.x = -vel.x.abs();
    }
    if pos.y - radius < playfield.top() {
        pos.y = playfield.top() + radius;
        vel.y = vel.y.abs();
    } else if pos.y + radius > playfield.bottom() {
        pos.y = playfield.bottom() - radius;
        vel.y = -vel.y.abs();
    }

    for w in solids {
        if !circle_hits_solid(pos, radius, *w, thickness) {
            continue;
        }
        let rect = w.as_rect(thickness);
        match w.axis {
            Axis::Horizontal => {
                if pos.y < w.fixed {
                    pos.y = rect.top() - radius;
                    vel.y = -vel.y.abs();
                } else {
                    pos.y = rect.bottom() + radius;
                    vel.y = vel.y.abs();
                }
            }
            Axis::Vertical => {
                if pos.x < w.fixed {
                    pos.x = rect.left() - radius;
                    vel.x = -vel.x.abs();
                } else {
                    pos.x = rect.right() + radius;
                    vel.x = vel.x.abs();
                }
            }
        }
    }

    (pos, vel)
}

/// Result of region claim analysis for one frame.
#[derive(Debug, Clone)]
pub struct ClaimState {
    /// Open chambers where balls can still move (drawn as free space).
    pub free: Vec<Rect>,
    /// Filled territory (physics bounce); renderer paints playfield claimed then carves free.
    pub claimed: Vec<Rect>,
    pub claimed_ratio: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ClaimCell {
    Open,
    Barrier,
    Free,
}

/// Flood-fill free space from balls; everything else is claimed.
///
/// Barriers are wall segments only (no previous-claim feedback). Free rects are merged
/// solid blocks so the renderer can paint the playfield claimed and carve free on top.
pub fn compute_claim_state(
    playfield: Rect,
    barriers: &[SolidWall],
    balls: &[(Vec2, f32)],
    wall_thickness: f32,
    grid_columns: u32,
) -> ClaimState {
    let cols = grid_columns.max(16) as usize;
    let cell_w = playfield.width / cols as f32;
    let rows = ((playfield.height / cell_w).round() as usize).max(16);
    let cell_h = playfield.height / rows as f32;

    let mut grid = vec![ClaimCell::Open; cols * rows];
    let idx = |c: usize, r: usize| -> usize { r * cols + c };

    // Mark ONLY cells that overlap the true wall thickness. Over-dilating painted free-side
    // cells as claimed, so the fill looked like it stopped short of (or bled past) walls.
    for w in barriers {
        let rect = w.as_rect(wall_thickness);
        let c0 = ((rect.left() - playfield.left()) / cell_w).floor() as i32;
        let c1 = ((rect.right() - playfield.left()) / cell_w).ceil() as i32;
        let r0 = ((rect.top() - playfield.top()) / cell_h).floor() as i32;
        let r1 = ((rect.bottom() - playfield.top()) / cell_h).ceil() as i32;
        for r in r0.max(0)..r1.min(rows as i32) {
            for c in c0.max(0)..c1.min(cols as i32) {
                let cell_rect = Rect::new(
                    playfield.left() + c as f32 * cell_w,
                    playfield.top() + r as f32 * cell_h,
                    cell_w,
                    cell_h,
                );
                if rects_overlap_expanded(cell_rect, rect, 0.01) {
                    grid[idx(c as usize, r as usize)] = ClaimCell::Barrier;
                }
            }
        }
        // Ensure a continuous 4-connected barrier along the wall centerline (1 cell thick)
        // without expanding into free space beyond the wall slab.
        mark_wall_centerline(
            &mut grid,
            *w,
            playfield,
            cell_w,
            cell_h,
            cols,
            rows,
            idx,
        );
    }

    let mut stack: Vec<(usize, usize)> = Vec::new();
    for (pos, _) in balls {
        let c = ((pos.x - playfield.left()) / cell_w)
            .floor()
            .clamp(0.0, (cols - 1) as f32) as usize;
        let r = ((pos.y - playfield.top()) / cell_h)
            .floor()
            .clamp(0.0, (rows - 1) as f32) as usize;
        let i = idx(c, r);
        if grid[i] == ClaimCell::Open {
            grid[i] = ClaimCell::Free;
            stack.push((c, r));
        } else if grid[i] == ClaimCell::Barrier {
            for (dc, dr) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let nc = c as i32 + dc;
                let nr = r as i32 + dr;
                if nc < 0 || nr < 0 || nc >= cols as i32 || nr >= rows as i32 {
                    continue;
                }
                let ni = idx(nc as usize, nr as usize);
                if grid[ni] == ClaimCell::Open {
                    grid[ni] = ClaimCell::Free;
                    stack.push((nc as usize, nr as usize));
                }
            }
        }
    }

    while let Some((c, r)) = stack.pop() {
        for (dc, dr) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
            let nc = c as i32 + dc;
            let nr = r as i32 + dr;
            if nc < 0 || nr < 0 || nc >= cols as i32 || nr >= rows as i32 {
                continue;
            }
            let ni = idx(nc as usize, nr as usize);
            if grid[ni] == ClaimCell::Open {
                grid[ni] = ClaimCell::Free;
                stack.push((nc as usize, nr as usize));
            }
        }
    }

    let free_cells = grid.iter().filter(|c| **c == ClaimCell::Free).count() as f32;
    let total = (cols * rows) as f32;
    let claimed_ratio = ((total - free_cells) / total).clamp(0.0, 1.0);

    let mut free =
        merge_matching_cells(&grid, |c| c == ClaimCell::Free, cols, rows, playfield, cell_w, cell_h);
    // Snap free chamber edges to walls / playfield so fill meets lines cleanly.
    snap_free_rects_to_geometry(&mut free, playfield, barriers, wall_thickness);

    // Claimed bounce surfaces: complement of free within the playfield (axis-aligned).
    let claimed = claimed_rects_from_free(playfield, &free);

    ClaimState {
        free,
        claimed,
        claimed_ratio,
    }
}

fn mark_wall_centerline(
    grid: &mut [ClaimCell],
    wall: SolidWall,
    playfield: Rect,
    cell_w: f32,
    cell_h: f32,
    cols: usize,
    rows: usize,
    idx: impl Fn(usize, usize) -> usize,
) {
    let start = wall.start.min(wall.end);
    let end = wall.start.max(wall.end);
    match wall.axis {
        Axis::Vertical => {
            let c = ((wall.fixed - playfield.left()) / cell_w)
                .floor()
                .clamp(0.0, (cols - 1) as f32) as usize;
            let r0 = ((start - playfield.top()) / cell_h).floor() as i32;
            let r1 = ((end - playfield.top()) / cell_h).ceil() as i32;
            for r in r0.max(0)..r1.min(rows as i32) {
                grid[idx(c, r as usize)] = ClaimCell::Barrier;
            }
        }
        Axis::Horizontal => {
            let r = ((wall.fixed - playfield.top()) / cell_h)
                .floor()
                .clamp(0.0, (rows - 1) as f32) as usize;
            let c0 = ((start - playfield.left()) / cell_w).floor() as i32;
            let c1 = ((end - playfield.left()) / cell_w).ceil() as i32;
            for c in c0.max(0)..c1.min(cols as i32) {
                grid[idx(c as usize, r)] = ClaimCell::Barrier;
            }
        }
    }
}

/// Expand free rect edges that sit near a wall or playfield border so chambers
/// meet solid geometry flush (removes grid stair-step / gap artifacts).
fn snap_free_rects_to_geometry(
    free: &mut Vec<Rect>,
    playfield: Rect,
    barriers: &[SolidWall],
    wall_thickness: f32,
) {
    let snap = wall_thickness.max(2.0);
    for rect in free.iter_mut() {
        // Snap to playfield edges
        if (rect.left() - playfield.left()).abs() < snap {
            let right = rect.right();
            rect.x = playfield.left();
            rect.width = right - rect.x;
        }
        if (rect.right() - playfield.right()).abs() < snap {
            rect.width = playfield.right() - rect.x;
        }
        if (rect.top() - playfield.top()).abs() < snap {
            let bottom = rect.bottom();
            rect.y = playfield.top();
            rect.height = bottom - rect.y;
        }
        if (rect.bottom() - playfield.bottom()).abs() < snap {
            rect.height = playfield.bottom() - rect.y;
        }

        // Snap to vertical walls
        for w in barriers {
            let half = wall_thickness * 0.5;
            match w.axis {
                Axis::Vertical => {
                    let wall_left = w.fixed - half;
                    let wall_right = w.fixed + half;
                    let w_start = w.start.min(w.end);
                    let w_end = w.start.max(w.end);
                    // Vertical overlap with wall span
                    if rect.bottom() <= w_start || rect.top() >= w_end {
                        continue;
                    }
                    if (rect.right() - wall_left).abs() < snap && rect.right() <= w.fixed + snap {
                        rect.width = (wall_left - rect.x).max(0.0);
                    }
                    if (rect.left() - wall_right).abs() < snap && rect.left() >= w.fixed - snap {
                        let right = rect.right();
                        rect.x = wall_right;
                        rect.width = (right - rect.x).max(0.0);
                    }
                }
                Axis::Horizontal => {
                    let wall_top = w.fixed - half;
                    let wall_bottom = w.fixed + half;
                    let w_start = w.start.min(w.end);
                    let w_end = w.start.max(w.end);
                    if rect.right() <= w_start || rect.left() >= w_end {
                        continue;
                    }
                    if (rect.bottom() - wall_top).abs() < snap && rect.bottom() <= w.fixed + snap {
                        rect.height = (wall_top - rect.y).max(0.0);
                    }
                    if (rect.top() - wall_bottom).abs() < snap && rect.top() >= w.fixed - snap {
                        let bottom = rect.bottom();
                        rect.y = wall_bottom;
                        rect.height = (bottom - rect.y).max(0.0);
                    }
                }
            }
        }
    }
    free.retain(|r| r.width > 0.5 && r.height > 0.5);
}

/// Claimed territory for physics: playfield strips not covered by free chambers.
///
/// Built by subtracting free rects from the playfield via successive splits so balls
/// bounce on claimed edges (same silhouettes the player sees as fill).
fn claimed_rects_from_free(playfield: Rect, free: &[Rect]) -> Vec<Rect> {
    let mut claimed = vec![playfield];
    for hole in free {
        let mut next = Vec::new();
        for piece in claimed {
            next.extend(subtract_rect(piece, *hole));
        }
        claimed = next;
    }
    claimed
        .into_iter()
        .filter(|r| r.width > 0.5 && r.height > 0.5)
        .collect()
}

/// Return rectangles covering `outer` minus `hole` (axis-aligned subtract).
fn subtract_rect(outer: Rect, hole: Rect) -> Vec<Rect> {
    let x0 = outer.left().max(hole.left());
    let x1 = outer.right().min(hole.right());
    let y0 = outer.top().max(hole.top());
    let y1 = outer.bottom().min(hole.bottom());
    if x0 >= x1 || y0 >= y1 {
        return vec![outer];
    }

    let mut parts = Vec::new();
    // Top band
    if outer.top() < y0 {
        parts.push(Rect::new(
            outer.x,
            outer.y,
            outer.width,
            y0 - outer.top(),
        ));
    }
    // Bottom band
    if y1 < outer.bottom() {
        parts.push(Rect::new(
            outer.x,
            y1,
            outer.width,
            outer.bottom() - y1,
        ));
    }
    // Middle-left
    if outer.left() < x0 {
        parts.push(Rect::new(outer.x, y0, x0 - outer.left(), y1 - y0));
    }
    // Middle-right
    if x1 < outer.right() {
        parts.push(Rect::new(x1, y0, outer.right() - x1, y1 - y0));
    }
    parts
}

fn rects_overlap_expanded(a: Rect, b: Rect, pad: f32) -> bool {
    a.left() < b.right() + pad
        && a.right() > b.left() - pad
        && a.top() < b.bottom() + pad
        && a.bottom() > b.top() - pad
}

fn merge_matching_cells(
    grid: &[ClaimCell],
    matches: impl Fn(ClaimCell) -> bool,
    cols: usize,
    rows: usize,
    playfield: Rect,
    cell_w: f32,
    cell_h: f32,
) -> Vec<Rect> {
    let idx = |c: usize, r: usize| r * cols + c;
    let mut visited = vec![false; cols * rows];
    let mut rects = Vec::new();

    for r0 in 0..rows {
        for c0 in 0..cols {
            let i0 = idx(c0, r0);
            if visited[i0] || !matches(grid[i0]) {
                continue;
            }
            let mut c1 = c0 + 1;
            while c1 < cols && matches(grid[idx(c1, r0)]) && !visited[idx(c1, r0)] {
                c1 += 1;
            }
            let mut r1 = r0 + 1;
            'grow: while r1 < rows {
                for c in c0..c1 {
                    let i = idx(c, r1);
                    if visited[i] || !matches(grid[i]) {
                        break 'grow;
                    }
                }
                r1 += 1;
            }
            for r in r0..r1 {
                for c in c0..c1 {
                    visited[idx(c, r)] = true;
                }
            }
            rects.push(Rect::new(
                playfield.left() + c0 as f32 * cell_w,
                playfield.top() + r0 as f32 * cell_h,
                (c1 - c0) as f32 * cell_w,
                (r1 - r0) as f32 * cell_h,
            ));
        }
    }
    rects
}

#[cfg(test)]
mod geometry_tests {
    use super::*;

    #[test]
    fn free_axis_limits_span_full_playfield_without_solids() {
        let pf = Rect::new(0.0, 0.0, 100.0, 50.0);
        let origin = Vec2::new(40.0, 25.0);
        let (neg, pos) = free_axis_limits(origin, Axis::Horizontal, pf, &[], 4.0);
        assert!((neg - 40.0).abs() < 0.01);
        assert!((pos - 60.0).abs() < 0.01);
    }

    #[test]
    fn sealed_side_of_vertical_wall_is_fully_claimed() {
        let pf = Rect::new(0.0, 0.0, 100.0, 100.0);
        let barriers = [SolidWall {
            axis: Axis::Vertical,
            fixed: 40.0,
            start: 0.0,
            end: 100.0,
        }];
        let balls = [(Vec2::new(70.0, 50.0), 4.0)];
        let state = compute_claim_state(pf, &barriers, &balls, 4.0, 50);
        assert!(
            state.claimed_ratio > 0.25,
            "left sealed side should claim, ratio={}",
            state.claimed_ratio
        );
        for rect in &state.free {
            assert!(
                rect.left() >= 35.0,
                "free rect should not extend deep into sealed left side: {rect:?}"
            );
        }
    }
}

#[cfg(test)]
mod hit_tests {
    use super::*;

    #[test]
    fn growing_half_hits_ball_on_segment() {
        let wall = GrowingWall {
            origin: Vec2::new(70.0, 100.0),
            axis: Axis::Horizontal,
            neg_extent: 0.0,
            pos_extent: 40.0,
            neg_done: false,
            pos_done: false,
            neg_alive: true,
            pos_alive: true,
            neg_committed: false,
            pos_committed: false,
        };
        let hit = circle_hits_growing_wall_half(Vec2::new(100.0, 100.0), 4.0, &wall, false, 4.0);
        assert!(hit, "ball on growing horizontal half should hit");
    }
}
