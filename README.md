# Shrinkz

Modern territory-claim arcade game inspired by classic bounce-and-claim titles. Claim at least **75%** of the playfield by drawing walls while bouncing balls try to stop you.

Built with **Bevy 0.19** (Rust). Original code and assets — product name is **Shrinkz**.

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

## Requirements

- Rust toolchain (1.85+ recommended; developed with 1.96)
- A GPU-capable desktop environment for the native window

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

## Web / WASM (local)

Install a WASM target and a static server tool:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli   # or: cargo install trunk
```

### Option A — trunk (recommended when available)

```bash
# from repo root after adding an index.html + Trunk.toml (see below)
trunk serve
```

A minimal `index.html` for trunk:

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Shrinkz</title>
    <style>
      html, body, canvas { margin: 0; width: 100%; height: 100%; background: #14161c; }
    </style>
  </head>
  <body>
    <link data-trunk rel="rust" data-bin="shrinkz" data-wasm-opt="z" />
  </body>
</html>
```

And `Trunk.toml`:

```toml
[build]
public_url = "./"
```

Then open the URL trunk prints (usually `http://127.0.0.1:8080`).

### Option B — wasm-bindgen manual

```bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir dist --target web target/wasm32-unknown-unknown/release/shrinkz.wasm
# serve dist/ with any static file server, e.g.:
python3 -m http.server -d dist 8080
```

**Web controls:** left click uses the axis toggle; **Shift+click** or **right-click** for vertical when the browser allows it.

No remote host is required for v1 — local serve only.

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
| Local WASM | pipeline documented; smoke after trunk/wasm-bindgen install |

## License

MIT OR Apache-2.0
