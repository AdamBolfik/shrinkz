# Shrinkz

Modern territory-claim arcade game inspired by classic bounce-and-claim titles. Claim at least **75%** of the playfield by drawing walls while bouncing balls try to stop you.

Built with **Bevy 0.19** (Rust). Original code and assets — product name is **Shrinkz**.

## Play in the browser

**Live:** [https://adambolfik.github.io/shrinkz/](https://adambolfik.github.io/shrinkz/)

Pushes to `main` build WASM with Trunk and deploy via GitHub Pages (see `.github/workflows/deploy-pages.yml`).

First time enabling Pages on this repo: **Settings → Pages → Source → GitHub Actions**.

## Rules

- **Left click** (or primary button): start a wall using the preferred axis (see toggle).
- **Right click**: vertical wall.
- **Shift+click**: vertical wall (works when right-click is awkward).
- **Axis toggle** button: switch preferred axis between **H** (horizontal) and **V** (vertical) for primary click.
- Walls grow in **both directions** from the click until they hit a solid edge or finished wall.
- If a ball hits an unfinished wall half, that half is destroyed and you lose a **life**.
- Closed regions with **no balls** fill in as claimed territory.
- Clear a level by claiming **≥ 75%**. Next level adds one more ball; lives reset to the ball count.
- Ball count caps at **50**.
- **Score** increases when you claim area and when you clear a level (level bonus + lives remaining).
- Optional **timer** is off by default (`GameConfig.timer_enabled`); when on, expiry is game over.
- **P** pause / resume · **R** restart level (keeps score) · **N** new game (score resets)
- **T** or **Theme** button — cycle color themes (Midnight, Arcade, Neon, Paper, Forest, Sunset)

## Requirements

- Rust toolchain (1.85+ recommended; developed with 1.96)
- A GPU-capable desktop environment for the native window
- For web: `wasm32-unknown-unknown` target and [Trunk](https://trunkrs.dev/)

## Run (desktop)

```bash
cargo run
```

Release build:

```bash
cargo run --release
```

## Test

```bash
cargo test
```

Simulation rules are covered by behavioral tests in `tests/sim_session.rs` (no window required).

## Web / WASM

### Local

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
# open http://127.0.0.1:8080
```

Release bundle (writes `dist/`):

```bash
trunk build --release
```

### GitHub Pages (CI)

Workflow: `.github/workflows/deploy-pages.yml`

- Triggers on push to `main` (and manual `workflow_dispatch`)
- Builds with `trunk build --release` and `TRUNK_PUBLIC_URL=/shrinkz/`
- Deploys the `dist/` folder to GitHub Pages

**Web controls:** left click uses the axis toggle; **Shift+click** or **right-click** for vertical when the browser allows it.

## Project layout

```
src/
  lib.rs          # library root
  main.rs         # Bevy app (input, render, HUD)
  sim/            # pure game rules (no Bevy)
    types.rs
    geometry.rs
    session.rs
tests/
  sim_session.rs  # behavioral sim tests
.github/workflows/
  deploy-pages.yml
plans/
  modern-shrinkz-clone.md
```

## Future

1. **Polish** — sound, claim/hit particles, settings UI (timer on/off, volume).
2. **Android** — package the same Bevy crate (NDK / mobile tooling); reuse axis toggle + tap.

## Verified on

| Check | Status |
|-------|--------|
| `cargo build` (macOS) | pass |
| `cargo test` (B1–B15, B19–B22) | pass |
| Desktop window (`cargo run`) | manual — run locally |
| GitHub Pages (Trunk WASM) | CI on push to `main` |

## License

MIT OR Apache-2.0
