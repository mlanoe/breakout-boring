# breakout-boring

A port of Bevy's official **Breakout** showcase example
([`examples/showcase/breakout.rs`](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/showcase/breakout.rs),
v0.19.0) to the [Boring language](../boring) — a real, visual game
(window, sprites, sound, keyboard input), not a headless demo.

The defining feature: **100% Boring-authored.** There is no
hand-written `src/lib.rs` or `src/main.rs` at all. Every component,
resource and event, every constant, all pure logic, all Bevy systems,
every entity spawn, the `App` wiring, and the real program entry point
(`fn main()`) come from a single source file,
[`boring/breakout.br`](boring/breakout.br). `src/boring_gen.rs` is
generated from it via `boring build --emit-rust` and serves as both
Cargo targets' crate root directly.

```bash
cargo run     # opens a window — arrow keys move the paddle
cargo test    # headless MinimalPlugins tests of the collision/score logic
```

## Layout

| Path | Role |
|---|---|
| [`boring/breakout.br`](boring/breakout.br) | The entire game: components (`Paddle`/`Ball`/`Brick`/`Collider`/`Wall`/`ScoreboardUi`), the `BallCollided` event, resources (`Velocity`/`Score`/`CollisionSound`), the `WallLocation`/`Collision` enums, collision/geometry logic, every `Startup`/`Update` system and observer, all constants, `build_app`, `run_game`, and `fn main()` |
| [`src/boring_gen.rs`](src/boring_gen.rs) | Generated output of `boring build --emit-rust` (see [`boring/regen.sh`](boring/regen.sh)) — never hand-edited. Both `[lib]` and `[[bin]]` in `Cargo.toml` point their `path` at this one file |
| [`tests/ecs_integration.rs`](tests/ecs_integration.rs) | Hand-written Rust: headless `MinimalPlugins` integration tests for the collision/scoring systems |
| [`Cargo.toml`](Cargo.toml) | Hand-written: `bevy = "0.19"` (full default features) |
| [`boring.toml`](boring.toml) | Hand-written: project entry point plus `[external_types]`/`[derives]` includes shared with other Boring+Bevy games via [`boring-bevylib`](../boring-bevylib) |

## Building `boring`

This project depends on the `boring` compiler being built from source:

```bash
git -C boring worktree add ../.boring-main-worktree main
cd .boring-main-worktree && cargo build --release
export BORING_BIN="$(pwd)/target/release/boring"
```

## Regenerating `src/boring_gen.rs`

```bash
cd breakout-boring/boring
./regen.sh          # uses $BORING_BIN, defaults to `boring` on PATH
```

`regen.sh` runs `boring build --emit-rust` in project mode (no file
argument — `main` and `[external_types]`/`[derives]` are read from
[`boring.toml`](boring.toml)). `boring.toml` pulls in the shared
external-type and derive-macro whitelists from
[`../boring-bevylib`](../boring-bevylib) rather than repeating them
locally. `--emit-rust` only ever prints transpiled Rust text — it never
touches `Cargo.toml`, which stays entirely hand-written.

## Verifying it works

`cargo test` runs `tests/ecs_integration.rs`: a headless `MinimalPlugins`
`App` (no window) that spawns a ball and a brick, runs the collision
systems for a couple of ticks, and asserts the brick despawned, the
score incremented exactly once, and the ball's velocity reflected on the
correct axis. `cargo run` opens the real window (arrow keys move the
paddle) for a visual check.

## Dropped from the original

The `stepping` module (`mod stepping; ... stepping::SteppingPlugin`) —
Bevy's debug-stepping dev tool, gated behind the `bevy_debug_stepping`
feature. Unrelated to gameplay and to this project's purpose.
