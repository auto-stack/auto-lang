# AGP — AutoGraphics Prototype (PLAN-829)

An **independent** wgpu-based 2D/3D game-prototype workspace that replicates
two frozen game samples with deterministic replay evidence:

- **AG-2D-01 — Bevy Breakout** (`bevyengine/bevy@efe61cc5…`,
  `examples/showcase/breakout.rs`) — faithful replica with a frozen golden
  trace (bit-exact replay).
- **AG-3D-01 — Godot "Squash the Creeps"**
  (`godotengine/godot-demo-projects@b761b4cd…`, `3d/squash_the_creeps/`) —
  faithful replica with a frozen golden trace (bit-exact replay).
- **Iced shared-device spike** — real iced 0.14 widgets + our 2D/3D views on
  ONE wgpu device/queue.

This is a **research prototype, not production software**. Production
integration (real Auto source entry, production AutoUI/nesting/RQHost, Forge
& distribution) is **pending** — see
[`research/integration-handoff.md`](research/integration-handoff.md).

## Repository position

Standalone workspace: `experimental/autographics-prototype/` carries its own
`[workspace]`, `Cargo.lock`, and assets. **Zero path dependencies** — the
iced_widget crate comes straight from crates.io (stock 0.14.2; the
production PLAN-043 patch is not necessary for this minimal spike, see
`research/integration-handoff.md`). The repository root workspace/lock and
all production crates are untouched (verified by the
`independent-consumer` probe). No junctions/symlinks.

## Build / test / run

All commands run from the repository root (or anywhere — paths relative):

```bash
# fast checks (scoped to AGP only; never root suites)
cargo check --manifest-path experimental/autographics-prototype/Cargo.toml --locked
cargo  test --manifest-path experimental/autographics-prototype/Cargo.toml --locked

# probe runner (10 cases; windowed cases open real windows on this desktop)
PROBE=experimental/autographics-prototype/target/release/graphics-probe.exe
cargo build --manifest-path experimental/autographics-prototype/Cargo.toml --locked --release

$PROBE --list
$PROBE --case gpu-foundation            --replay canonical --report scratch/829/gpu-foundation --mode auto
$PROBE --case shared-runtime            --replay canonical --report scratch/829/shared-runtime --mode auto
$PROBE --case ag2d-breakout             --replay canonical --report scratch/829/ag2d-breakout
$PROBE --case ag3d-risk-slice           --replay canonical --report scratch/829/ag3d-risk-slice
$PROBE --case ag3d-squash               --replay canonical --report scratch/829/ag3d-squash
$PROBE --case lifecycle                 --replay canonical --report scratch/829/lifecycle
$PROBE --case metrics                   --replay canonical --report scratch/829/metrics
$PROBE --case verifier-negative-controls --replay canonical --report scratch/829/negative
$PROBE --case iced-shared-device        --replay canonical --report scratch/829/iced-shared-device
$PROBE --case independent-consumer      --replay canonical --report scratch/829/independent-consumer
```

Each case writes `report.json` + keyframe PNGs; exit 0 = pass, non-zero =
fail (no skips-as-pass).

## Interactive play

```bash
cargo run --manifest-path experimental/autographics-prototype/Cargo.toml --locked --release --bin graphics-probe -- --case ag2d-breakout --replay live
cargo run --manifest-path experimental/autographics-prototype/Cargo.toml --locked --release --bin graphics-probe -- --case ag3d-squash  --replay live
cargo run --manifest-path experimental/autographics-prototype/Cargo.toml --locked --release --bin iced_spike
```

- Breakout: ArrowLeft/ArrowRight paddle; R = reset (host extension).
- Squash: WASD/arrows move, Space jump, Enter retry (when dead), R reset.
- iced spike: "Step graphics" button changes the 2D palette.

## Evidence locations

- Probe reports/keyframes: `scratch/829/<case>/` (gitignored; regenerate with
  the commands above).
- Frozen oracles: `testdata/breakout/golden_trace.json`,
  `testdata/squash/golden_squash_trace.json` + generators (provenance in
  file headers).
- Research: `research/sample-manifest.md` (frozen oracle),
  `support-profile.md`, `3d-feasibility.md`, `measurements.md`,
  `integration-handoff.md`, `result.md`.

## Asset provenance & licenses

| Asset | Source | License |
|---|---|---|
| `assets/breakout/sounds/breakout_collision.ogg` | bevy assets (PR #4331) | not recorded in Bevy CREDITS (see manifest §3 gap note) |
| `assets/breakout/fonts/FiraMono-*` | frozen Bevy tree | SIL OFL 1.1 |
| `assets/squash_the_creeps/**` | frozen godot-demo-projects | MIT (project); music CC-BY 3.0 (HorrorPen); Montserrat SIL OFL 1.1 |

sha256 for everything under `testdata/SHA256SUMS` and `assets/SHA256SUMS`.

## Directory map

```
src/core/       G-base runtime: identity, fixed-tick clock, actions/events,
                scene deltas + replay log, deterministic RNG (GPU-agnostic)
src/renderer/   wgpu foundation: device/targets, quad pipeline, resource
                cache, capture, failure taxonomy
src/render2d/   fontdue glyph-atlas text
src/render3d/   camera, lit mesh pipeline + shadow mapping, bob animation,
                collision, procedural environment
src/assets/     GLB loading (gltf 1.4), classified failures
src/games/      breakout/ + squash/ (drivers over the G-base runtime)
src/host/       probe cases, verifier gates, iced spike, reports
src/bin/        graphics-probe (runner), iced_spike
testdata/       frozen sources + golden traces + generators + golden
                content frames (incl. godot-engine-reference/)
research/       manifests, feasibility, measurements, handoff, result
```
