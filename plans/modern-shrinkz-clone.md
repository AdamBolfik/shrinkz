# Shrinkz — Modern Territory-Claim Arcade (Bevy)

## 1. Problem

There is no maintained, open, OS-agnostic game in the classic “bounce-and-claim” genre (popularized by JezzBall-style titles) that feels modern while keeping the classic rules. Original Windows Entertainment Pack games are abandoned and awkward to run today. We will build **Shrinkz**, a new game that:

- Runs natively on **desktop** (macOS, Windows, Linux) first
- Targets **web (WASM)** in the same first product phase (local serve is enough for v1)
- Defers **Android** until after **polish** (sound, particles, settings)
- Uses **faithful classic rules** with **modern visuals** (not a pixel-perfect remake of any commercial title)

The project lives at `/Users/adam/Code/shrinkz` as a new Bevy (Rust) game. Product name, crate, and window title are all **Shrinkz** / `shrinkz` — never “JezzBall” / `jezzball`.

## 2. Research Findings

### Classic rules (authoritative for v1)

Genre rules derived from public descriptions of bounce-and-claim games (Qix descendants with bouncing balls). Implementation must be original code and original assets.

| Rule | Behavior |
|------|----------|
| Goal | Claim **≥ 75%** of the playfield area |
| Walls | Horizontal or vertical segments grown from a point |
| Growth | Wall builds **both directions** from the click origin until each half hits an existing boundary/wall |
| Hit in progress | Ball collides with unfinished wall half → that half is destroyed; player loses **1 life** |
| Claim | When a closed region contains **no balls**, it fills as claimed territory |
| Lives | At level start, lives = number of balls on that level |
| Progression | Clear level → next level with **+1 ball**; lives reset to ball count |
| Cap | Ball count caps at **50** (classic-style); configurable via `max_balls` |
| Game over | Lives reach 0 |
| Timer | **Optional** in v1 — off by default; can be enabled via config / settings |
| Score | **In v1** — accumulate score for claimed area, level clears, and remaining lives/time bonuses (see §4) |
| Control | Left click / primary = horizontal; right click / secondary = vertical; **also** Shift+click for vertical and an on-screen H/V toggle (both) |

### Stack decision (confirmed)

- **Engine:** Bevy (Rust)
- **v1 targets:** Desktop + web (local `trunk serve` or equivalent)
- **Post-v1 order:** **Polish first**, then Android packaging
- **Fidelity:** Faithful classic rules, modern visuals
- **Bevy version:** Pin latest stable at scaffold time (no preference)
- **Toolchain present:** Rust 1.96 / cargo 1.96 on this machine

### Bevy platform reality (planning constraints)

- **Desktop:** first-class, lowest friction
- **Web (WASM):** supported; expect larger download; size optimization for release WASM; local serve only in v1
- **Android:** deferred past polish; NDK/Gradle packaging and touch UX are a later milestone — do **not** block v1

### Architecture implication

Shrinkz is almost entirely **2D geometry + state machine**. Keep **simulation pure and Bevy-free** so physics, wall growth, region claiming, scoring, and level rules are unit-testable without a window. Bevy owns input, rendering, audio, and scene lifecycle only.

### Legal / naming

- Do **not** copy Microsoft (or any third-party) assets, trademarks, or binaries
- Ship only as **Shrinkz** — no “JezzBall” branding in UI, package name, or marketing strings
- Genre inspiration is fine; assets, names, and code are original

### Greenfield

No existing game code in this repo. `hello_rust` is unrelated. This plan scaffolds the whole project under `shrinkz/`.

### Decisions from review (locked)

| # | Topic | Decision |
|---|--------|----------|
| 1 | Name | **Shrinkz** (not JezzBall / jezzball) |
| 2 | Timer | **Optional** (off by default) |
| 3 | Score | **Add in v1** |
| 4 | Bevy pin | Latest stable at scaffold |
| 5 | Web host | **Local only** for v1 |
| 6 | After desktop+web | **Polish**, then Android later |
| 7 | Vertical wall input | **Both** Shift+click and on-screen H/V toggle (plus right-click when available) |

## 3. Proposed Solution

**Approach.** Create a single Rust package (`shrinkz`) using Bevy for presentation. Implement gameplay in a pure `sim` module (balls, walls, field partition, levels, lives, score, optional timer). Drive the sim from fixed-timestep Bevy systems. Render with simple 2D primitives first (modern clean look), HUD for %, lives, level, and score. Ship desktop binary, then local WASM. After v1 playable: polish (audio, particles, settings UI including optional timer). Android remains a later phase, documented lightly only.

**Seams.**

| Boundary | Crossing |
|----------|----------|
| Input → Game commands | Pointer / toggle → `StartWall { origin, axis }` / `SetAxis` / `Pause` / `Restart` |
| Sim → View model | Pure `GameSnapshot` each frame for draw + HUD |
| View → Screen | Bevy 2D camera + shapes/UI nodes |
| Platform | `cfg` only where desktop vs web input differ |

**Impact.** A playable Shrinkz on desktop and local browser; score and optional timer in the rules; polish path clear; Android not blocking.

**Design constraints.**

- **SRP per public unit** — sim owns rules; Bevy owns I/O and draw
- **Public surface minimalism** — only command apply + snapshot APIs public from `sim`
- **DRY** — one wall-growth, claim, and score path for all levels
- **KISS** — no multiplayer, accounts, power-ups, or editor in v1
- **Behavior coverage** — every public sim / input surface maps to §8

**Rejected alternatives.** Flutter/Flame and Godot rejected by stack choice. Android-before-polish rejected. Shipping under JezzBall name rejected.

## 4. Simulation Layer (`sim`)

Pure Rust, no Bevy imports. **World units: playfield origin top-left, y-down**, matching 2D screen space.

### `GameSession`

- **Purpose:** Own the mutable run of one play session (menu → playing → level clear → game over).
- **Public surface:**
  - `new(config: GameConfig) -> GameSession` — fresh session at level 1. Covers B1, B19.
  - `apply(&mut self, command: GameCommand, dt: Duration)` — advance sim one step with optional command. Covers B2–B12, B14–B15, B19–B22.
  - `snapshot(&self) -> GameSnapshot` — read-only view for rendering/HUD. Covers B13.
- **Private internals:** `tick_balls`, `tick_walls`, `resolve_wall_hits`, `claim_empty_regions`, `award_claim_score`, `check_level_clear`, `begin_level`, `tick_optional_timer`.
- **Invariants & rules:**
  - Deterministic given seed + command stream; prefer fixed 45°-style velocities.
  - `apply` never panics on out-of-bounds clicks: clamps or ignores invalid origins.
  - Claiming is idempotent within a tick.
  - Score never decreases; only resets on `RestartGame`.

### `GameConfig`

- **Purpose:** Tunable constants for a build / settings.
- **Public surface:**
  - Geometry/tuning: `playfield: Rect`, `claim_ratio_to_clear: f32` (default `0.75`), `wall_growth_speed: f32`, `ball_speed: f32`, `ball_radius: f32`, `wall_thickness: f32`, `max_balls: u32` (default `50`).
  - Timer: `timer_enabled: bool` (default `false`), `level_time_limit: Duration` (used only when timer enabled).
  - Score weights: `score_per_area_unit: u64`, `level_clear_bonus_base: u64`, `life_remaining_bonus: u64`, `time_remaining_bonus_per_second: u64` (time bonus applies only when timer enabled).
  - Covers B1, B8, B19–B22.
- **Private internals:** none.
- **Invariants & rules:** `claim_ratio_to_clear` in `(0, 1]`; speeds positive; when `timer_enabled` is false, timer fields are ignored by the sim.

### `GameCommand`

- **Purpose:** Discrete player/system intents the sim understands.
- **Public surface:** enum variants:
  - `StartWall { origin: Vec2, axis: Axis }` — begin bidirectional wall. Covers B3, B4.
  - `SetPreferredAxis(Axis)` — update sticky axis for toggle-driven play (sim may store preferred axis for snapshot/UI only if walls still pass axis explicitly — prefer UI-owned toggle; if UI-owned, this variant is optional and can live only in the input layer). **Decision: preferred axis is UI-owned**; sim always receives explicit `axis` on `StartWall`. Covers B17, B18 via input layer.
  - `Pause` / `Resume` — pause gate. Covers B14.
  - `RestartLevel` / `RestartGame` — reset. Covers B15, B19.
- **Private internals:** none.
- **Invariants & rules:** Only one wall in progress at a time; a second `StartWall` while building is ignored.

### `Axis`

- **Purpose:** Wall orientation.
- **Public surface:** `Horizontal`, `Vertical`. Covers B3, B4.
- **Invariants & rules:** See §5 input mapping.

### `GameSnapshot`

- **Purpose:** Everything the view needs to draw one frame without mutably touching sim.
- **Public surface:** `phase: Phase`, `level: u32`, `lives: u32`, `score: u64`, `balls: Vec<BallView>`, `walls: Vec<WallView>`, `claimed: Vec<Rect>` (or equivalent region rep), `claimed_ratio: f32`, `wall_in_progress: Option<WallView>`, `timer: Option<TimerView>` (`remaining` when timer enabled, else `None`). Covers B13, B19–B22.
- **Private internals:** conversion from internal entities.
- **Invariants & rules:** Snapshot is pure data (cloneable); no handles into sim.

### Scoring (load-bearing)

- **On claim:** add `floor(claimed_area_delta * score_per_area_unit)` (or integer grid cells × weight). Covers B19.
- **On level clear:** add `level_clear_bonus_base * level` plus `lives * life_remaining_bonus`; if timer enabled, add `floor(remaining_secs) * time_remaining_bonus_per_second`. Covers B20.
- **On wall hit / life loss:** no score penalty (classic-friendly).
- **On game over:** final score remains visible; no further awards until `RestartGame`. Covers B12, B19.

### Optional timer (load-bearing)

- When `timer_enabled`: each level starts with `level_time_limit`; `dt` counts down only in `Phase::Playing` and not while paused.
- On expiry: treat as game over **or** lose one life and small time refill — **v1 rule: timer expiry → `Phase::GameOver`** (simple). Covers B21, B22.
- When disabled: `snapshot.timer == None`; no expiry path.

### Geometry / claim algorithm (private to `sim`, load-bearing)

- **Wall growth:** two half-segments from origin along `axis` until solid or destroyed.
- **Ball motion:** constant velocity; bounce on solid walls (invert colliding axis).
- **Hit unfinished wall:** destroy that half; decrement lives; lives == 0 → `GameOver`.
- **Region claim:** closed region with zero ball centers → claimed; award claim score; balls only in unclaimed space.
- **Level clear:** `claimed_ratio >= claim_ratio_to_clear` → award clear bonuses → advance with `balls = min(level, max_balls)`, `lives = balls`, timer reset if enabled.

## 5. Application Layer (Bevy)

### `ShrinkzApp` plugin group

- **Purpose:** Register plugins, resources, and schedules for one game window titled Shrinkz.
- **Public surface:** `run()` from `main`. Covers B16.
- **Private internals:** schedule labels `SimTick`, `Draw`, `Ui`.
- **Invariants & rules:** Fixed timestep for `SimTick` (e.g. 60 Hz).

### Input plugin

- **Purpose:** Translate platform pointer and UI toggle into `GameCommand`s.
- **Public surface:** systems that emit at most one `StartWall` per click. Covers B3, B4, B17, B18.
- **Private internals:** `cursor_to_playfield`, `button_to_axis`, `AxisToggle` resource (UI-owned preferred axis).
- **Invariants & rules:**
  - **Desktop:** left → horizontal; right → vertical.
  - **All platforms:** **Shift+click** forces vertical (or toggles override — **Shift+click → Vertical** regardless of toggle).
  - **On-screen H/V toggle:** sets preferred axis; primary click without Shift uses preferred axis (default Horizontal). Right-click still forces Vertical when available.
  - Clicks outside playfield ignored.
  - Android (later): reuse toggle + tap; not built in v1.

### Render plugin

- **Purpose:** Draw playfield, claimed regions, balls, walls, cursor guide from `GameSnapshot`.
- **Public surface:** draw systems. Covers B13, B16.
- **Private internals:** mesh builders, color palette.
- **Invariants & rules:** Modern flat/minimal; original Shrinkz look; camera letterboxes on resize.

### HUD / menu plugin

- **Purpose:** Show level, lives, claim %, **score**, optional timer, pause, game over, level clear; H/V toggle control.
- **Public surface:** Bevy UI bound to snapshot + `AxisToggle`. Covers B8, B11–B15, B17–B22.
- **Private internals:** button handlers → commands / toggle resource.
- **Invariants & rules:** Readable at 1280×720 and smaller web canvases; pause freezes sim; game over shows **final score** and level reached.

## 6. Platform Packaging

### Desktop (v1)

- **Purpose:** Native Shrinkz binary.
- **Public surface:** `cargo run --release`; README install/build.
- **Invariants & rules:** Verify on developer macOS.

### Web / WASM (v1)

- **Purpose:** Local browser playable build.
- **Public surface:** `trunk serve` (or Bevy-friendly equivalent) documented in README — **no remote host required**.
- **Invariants & rules:** Document controls (right-click, Shift+click, H/V toggle); WASM size opts for release.

### Polish (post-v1, before Android)

- **Purpose:** Feel and presentation: sound effects, light particles on claim/hit, settings (timer on/off, volume), juice.
- **Not required for v1 acceptance** but is the **next milestone after desktop+web**.

### Android (later — after polish)

- **Purpose:** Touch package of the same crate.
- **v1 deliverable:** brief README “Future: Android” only — **no APK**.
- **Invariants & rules:** Do not block v1 or polish.

## 7. Frontend / UX (player-facing)

### Play screen

- **Purpose:** Primary gameplay view branded Shrinkz.
- **Public surface:** playfield + HUD (level, lives, %, score, timer if on) + H/V toggle. Covers B2–B13, B17–B22.
- **Private internals:** resize letterbox; optional grid.
- **Invariants & rules:** Instant claim fill in v1; in-progress wall distinct color.

### Title / game over screens

- **Purpose:** Start and end session.
- **Public surface:** Start, Restart; show score on game over. Covers B1, B12, B15, B19.
- **Invariants & rules:** Restart returns to level 1 and score 0.

## 8. Test Plan

Sim tests are pure Rust unit tests (no window). Prefer `cargo test` under `src/sim/` or `tests/sim_*.rs`.

### `tests/sim_session.rs` (or `src/sim/*` unit tests)

- **B1: A new session starts at level 1 with one ball, one life, and score 0.**
  - **Test:** `new_session_starts_at_level_one_with_one_ball_one_life_and_zero_score`
  - **Given** default `GameConfig`,
  - **When** `GameSession::new` is called,
  - **Then** snapshot has `level == 1`, `balls.len() == 1`, `lives == 1`, `score == 0`, `claimed_ratio == 0`, `timer == None` when timer disabled.
  - **Observable outcome:** snapshot fields above.

- **B2: Balls bounce off solid playfield edges and stay inside.**
  - **Test:** `balls_remain_inside_playfield_after_many_ticks`
  - **Given** a session with one ball near a corner aimed outward,
  - **When** `apply` runs for many ticks with no commands,
  - **Then** every ball center remains inside the playfield inset by radius.
  - **Observable outcome:** all `BallView` positions in-bounds.

- **B3: Horizontal StartWall begins bidirectional growth along X.**
  - **Test:** `start_wall_horizontal_begins_bidirectional_growth`
  - **Given** empty field,
  - **When** `StartWall { axis: Horizontal, origin: center }` is applied over ticks,
  - **Then** `wall_in_progress` grows along X only.
  - **Observable outcome:** geometry of `wall_in_progress`.

- **B4: Vertical StartWall grows along Y.**
  - **Test:** `start_wall_vertical_grows_along_y`
  - **Given** empty field,
  - **When** `StartWall { axis: Vertical, origin: center }` over ticks,
  - **Then** length increases along Y only.
  - **Observable outcome:** geometry axis of `wall_in_progress`.

- **B5: Completing a wall that isolates an empty region claims that region.**
  - **Test:** `completed_wall_claims_region_with_no_balls`
  - **Given** a finishing wall closes a ball-free pocket,
  - **When** the wall completes,
  - **Then** `claimed_ratio` increases and that pocket appears in `claimed`.
  - **Observable outcome:** `claimed_ratio` and claimed regions.

- **B6: A region that still contains a ball is not claimed.**
  - **Test:** `region_containing_a_ball_is_not_claimed`
  - **Given** a closed region containing a ball,
  - **When** walls finish,
  - **Then** that region remains unclaimed.
  - **Observable outcome:** ball still in free space; ratio excludes that area.

- **B7: A ball hitting an unfinished wall half destroys that half and costs a life.**
  - **Test:** `ball_destroying_unfinished_wall_half_decrements_lives`
  - **Given** growing wall and ball on collision course,
  - **When** collision occurs,
  - **Then** that half is gone and `lives` decreases by 1.
  - **Observable outcome:** lives and walls in snapshot.

- **B8: Reaching the claim ratio clears the level and advances with one more ball.**
  - **Test:** `reaching_claim_ratio_advances_level_and_adds_a_ball`
  - **Given** session forced to `claimed_ratio >= 0.75`,
  - **When** clear check runs,
  - **Then** level 2, balls == 2, lives == 2, claimed resets.
  - **Observable outcome:** snapshot level/balls/lives/claimed_ratio.

- **B9: Ball count per level is min(level, max_balls).**
  - **Test:** `ball_count_caps_at_configured_max`
  - **Given** `max_balls == 3` and session at level 5,
  - **When** level 5 begins,
  - **Then** `balls.len() == 3` and `lives == 3`.
  - **Observable outcome:** ball and life counts.

- **B10: Second StartWall ignored while a wall is already in progress.**
  - **Test:** `second_start_wall_ignored_while_building`
  - **Given** wall in progress,
  - **When** another `StartWall` applied,
  - **Then** in-progress wall origin/axis unchanged.
  - **Observable outcome:** `wall_in_progress` continuity.

- **B11: Lives reaching zero ends the game.**
  - **Test:** `zero_lives_enters_game_over_phase`
  - **Given** `lives == 1` and unfinished-wall hit,
  - **When** hit resolves,
  - **Then** `phase == GameOver`.
  - **Observable outcome:** `phase` in snapshot.

- **B12: RestartGame returns to level 1 with score 0.**
  - **Test:** `restart_game_resets_to_level_one_and_zero_score`
  - **Given** mid-run session with score > 0 at level ≥ 2,
  - **When** `RestartGame` applied,
  - **Then** level 1, one ball, one life, score 0, claimed 0.
  - **Observable outcome:** snapshot equals fresh session shape.

- **B13: Snapshot is a complete read-only view for rendering.**
  - **Test:** `snapshot_exposes_phase_hud_and_entities_without_mutating_session`
  - **Given** any session,
  - **When** `snapshot` called twice with no `dt` progress,
  - **Then** snapshots match and session unchanged.
  - **Observable outcome:** equality of successive snapshots.

- **B14: Pause freezes simulation time.**
  - **Test:** `pause_prevents_ball_motion`
  - **Given** moving ball,
  - **When** `Pause` then ticks then `Resume`,
  - **Then** position unchanged during pause; advances after resume.
  - **Observable outcome:** ball positions across snapshots.

- **B15: RestartLevel resets the current chamber only and keeps score.**
  - **Test:** `restart_level_keeps_level_and_score_resets_chamber`
  - **Given** level 3 mid-claim with score S,
  - **When** `RestartLevel` applied,
  - **Then** level remains 3, lives restored to ball count, claimed_ratio 0, score remains S.
  - **Observable outcome:** snapshot level/lives/claimed_ratio/score.

- **B19: Claiming area increases score.**
  - **Test:** `claiming_empty_region_increases_score`
  - **Given** a session about to claim a known-area empty region,
  - **When** claim completes,
  - **Then** `score` increases by the configured area weight for that area.
  - **Observable outcome:** `score` delta on snapshot.

- **B20: Level clear awards level and lives bonuses (and does not decrease score).**
  - **Test:** `level_clear_adds_clear_and_life_bonuses`
  - **Given** session at clear threshold with known lives and config weights,
  - **When** level clears,
  - **Then** score increases by at least clear bonus + lives bonus.
  - **Observable outcome:** `score` after clear vs before.

- **B21: With timer disabled, snapshot has no timer and expiry never fires.**
  - **Test:** `disabled_timer_is_absent_and_never_expires`
  - **Given** `timer_enabled == false`,
  - **When** many ticks elapse,
  - **Then** `timer == None` and phase stays `Playing` (absent other failures).
  - **Observable outcome:** `timer` and `phase`.

- **B22: With timer enabled, expiry ends the game.**
  - **Test:** `enabled_timer_expiry_enters_game_over`
  - **Given** `timer_enabled == true` and `level_time_limit` short,
  - **When** enough `dt` elapses without pause,
  - **Then** `phase == GameOver`.
  - **Observable outcome:** `phase`; timer remaining ≤ 0 before transition.

### Manual / smoke

- **B16: Desktop window runs and renders Shrinkz.** Manual: `cargo run`.
- **B17: Left/right click and Shift+click map to wall axes on desktop.** Manual.
- **B18: On-screen H/V toggle changes preferred axis for primary click.** Manual.
- **B23: Local WASM build loads and accepts click-to-wall.** Manual after web pipeline.

## 9. Implementation Tasks

- [x] **Scaffold Bevy project `shrinkz`** — `Cargo.toml`, `src/main.rs`, pin latest stable Bevy, window title Shrinkz, `cargo run` empty window.
- [x] **Add `sim` module skeleton** — `GameConfig`, `GameSession`, `GameCommand`, `GameSnapshot`, `Phase`, `Axis`, score/timer fields.
- [x] **Implement ball motion and edge bounce** — constant velocity + solid boundary collisions.
- [x] **Implement bidirectional wall growth** — horizontal/vertical halves until solid hit.
- [x] **Implement unfinished-wall hit** — destroy half, decrement lives, game over at 0.
- [x] **Implement region claim** — close empty regions; update `claimed_ratio`.
- [x] **Implement scoring** — claim area points; level-clear and lives bonuses; score on snapshot.
- [x] **Implement optional timer** — off by default; when on, countdown + expiry → game over.
- [x] **Implement level clear and advance** — 75% threshold, +1 ball, lives reset, max ball cap, clear score awards.
- [x] **Implement pause/restart commands** — pause gate; restart level (keep score); restart game (score 0).
- [x] **Wire Bevy fixed-timestep sim systems** — apply commands + `dt` each tick.
- [x] **Wire input** — left/right click, Shift+click → vertical, H/V toggle resource for primary click.
- [x] **Render playfield, balls, walls, claimed regions** — modern flat Shrinkz palette; camera letterbox.
- [x] **HUD and end states** — level, lives, %, score, optional timer, pause, clear, game over, restart, H/V toggle.
- [x] **Local Web/WASM pipeline** — documented `trunk serve` (or equivalent); no remote host.
- [x] **README** — Shrinkz rules, controls (click / Shift / toggle), run desktop, run local web, test command, future polish then Android.
- [x] **Behavioral tests for sim** — cover B1–B15, B19–B22 (see §8).
- [x] **Manual smoke checklist** — B16–B18, B23 in README “Verified on” or PR notes.

## 10. Questions for the Reviewer

None remaining from the prior set — all answered and locked in §2.

If anything drifts during implementation, reopen only:

1. Exact score weight numbers (can tune after first playable).
2. Whether timer expiry should cost a life instead of hard game over (currently hard game over).

---

**Plan updated at** `shrinkz/plans/modern-shrinkz-clone.md`  
(Project directory renamed from `jezzball/` → `shrinkz/`.)

Please **confirm in writing that you approve** this revised plan before implementation (or `/implement`) begins.
