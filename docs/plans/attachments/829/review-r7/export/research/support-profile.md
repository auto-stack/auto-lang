# AGP Support Profile (T-02, PLAN-829)

Scope: what the T-02 foundation actually supports and validates on the frozen
Windows x64 profile. Performance budgets and load verification come later
(T-07, V-07); this file freezes the support surface, dependency decisions,
failure taxonomy and the real measured device facts from the gpu-foundation
probe. Nothing here upgrades the root workspace or touches production UI.

## 1. Version alignment record (re-verified 2026-10-20 on worktree entry)

| Item | Plan preflight claim | Measured in production root Cargo.lock (main checkout, read-only) | AGP pin | Verdict |
|---|---|---|---|---|
| wgpu | 27.0.1 | **27.0.1** (`wgpu-hal` 27.0.4) | `wgpu = "27.0.1"` → locked **27.0.1** | MATCH |
| winit | 0.30.13 | **0.30.13** | `winit = "0.30.13"` → locked **0.30.13** | MATCH |
| iced / iced_wgpu | 0.14 | **0.14.0** (iced_widget 0.14.2) | not a dependency in T-02 (deferred to T-08) | MATCH |

- The production stack reaches wgpu **through iced_wgpu** (root comment in
  `crates/auto-lang/Cargo.toml`: "经 iced_wgpu 取 wgpu … 保证与 iced 渲染器用
  同一个 wgpu 版本实例"). AGP depends on wgpu 27.0.1 directly — the same
  instance version, so the T-08 Iced seam can share one Device/Queue.
- wgpu 27.0.1 is also the newest 27.x (28/29/30 exist but would diverge from
  production); the "use the production version even if not newest" rule
  applies and is recorded here.
- Root `[patch.crates-io]`: `iced_widget = { path = "crates/patches/iced_widget" }`
  (PLAN-043 container-layout race hardening, iced_widget 0.14.2 minimal
  surgery). **AGP uses STOCK iced_widget 0.14.2 — the patch is NOT applied**
  (review r1 F09, 2026-10-21): the container-layout race fix is not
  necessary for AGP's minimal static spike, verified by running the
  canonical iced case green with the unpatched crate. AGP therefore has
  ZERO path dependencies. Production retains the patch; the semantic
  difference is recorded, not hidden.
- Workspace boundary: root `Cargo.toml` members are an **explicit list** (no
  `experimental/*` glob), so AGP cannot be absorbed. AGP/Cargo.toml carries an
  empty `[workspace]` table making it a standalone workspace; root
  Cargo.toml/Cargo.lock are **unmodified** (root Cargo.lock is gitignored
  library convention; AGP un-ignores only its own lock via a local
  `.gitignore` negation).

## 2. Bounded dependency selection (≤3 candidates per category)

Whole-path criterion: the two frozen samples' actual needs (see
`research/sample-manifest.md`), not generic engine capability.

| Category | Candidates | Decision | Why (whole-path fit) |
|---|---|---|---|
| Math | glam 0.34 / nalgebra / cgmath | **glam 0.34** | f32-first, matches the samples' Transform idioms (both Bevy and Godot samples think in plain f32 TRS); tiny; nalgebra rejected (heavier, deref-heavy API, no sample need), cgmath rejected (less maintained). |
| Text (Breakout HUD) | glyphon / wgpu_glyph / fontdue+custom atlas | **fontdue 0.9.5 + custom atlas** | glyphon **has no wgpu-27 line** (0.9→wgpu 25, 0.10→wgpu 28; verified against the crates index) — rejected on version alignment, the deciding constraint. wgpu_glyph rejected (stale, pre-wgpu-0.20 era). fontdue is pure-Rust TTF rasterization; the HUD need ("Score: N", 33 px) is one glyph atlas + quads. Font: Montserrat-Medium.ttf already frozen in `assets/` (SIL OFL 1.1). |
| Audio | kira 0.9 / rodio / miniaudio | **kira 0.9.6 (ogg feature)** | Samples need exactly: one-shot SFX per collision (Breakout) + one looping music stream (Squash). kira gives an explicit command-based AudioManager with static/streaming sounds — deterministic control for replay evidence. rodio rejected (less explicit instance control; sink-centric API fights the replay-oracle requirement). miniaudio rejected (unsafe C bindings, unnecessary). OGG decode via symphonia 0.5 (kira `ogg` feature). |
| Model loading (T-05) | gltf 1.4 / custom minimal GLB parser / (quick_gltf—nonexistent) | **gltf 1.4.1** | The frozen GLBs are tiny (52–64 KB, untextured, animation-less), but a hand parser risks spec-surface bugs on T-05's timeline; gltf crate is the standard, robust GLB path. Custom parser recorded as fallback if gltf 1.4's dependency weight becomes a problem. |
| Collision 2D | custom AABB/circle solver / parry2d / (rapier2d) | **custom solver** (T-03/T-04) | Breakout needs exactly one primitive: BoundingCircle vs Aabb2d with side classification — 30 lines, already specified numerically by the oracle. parry2d rejected (dependency surface far exceeds the single primitive needed). |
| Collision 3D | custom (sphere/box vs planes+boxes) / parry3d / rapier3d | **custom solver** (T-05/T-06) | Squash needs: character sphere/capsule vs ground plane + 4 world-boundary planes + mob OBB overlap + top-contact normal test. All primitives, no broadphase. parry3d rejected (heavy; broadphase/narrowphase machinery unused). rapier3d rejected (full physics engine — sample semantics are simpler than a physics engine's). |
| Scene storage | plain struct-of-vec arena + generational ids / hecs / bevy_ecs | **plain arena** (`core::Scene`) | The Plan forbids building an ECS framework; the samples have < 100 live objects. Arena + `IdAllocator` gives stable identity + O(1) lookup without archetype machinery. hecs/bevy_ecs rejected (archetype ECS, query infrastructure unused). |

## 3. Supported surface actually validated (T-02)

- **Window target**: winit 0.30 window, swapchain via `wgpu::Surface`,
  Fifo present mode, sRGB BGRA/RGBA (adapter-selected), resize → reconfigure,
  minimize (0×0) → presentation suspended without error, restore → presented.
- **Offscreen target**: RGBA8-sRGB texture, arbitrary resize, GPU→CPU readback
  (256-byte row pitch) to PNG keyframes with sha256.
- **Two targets / one device**: window swapchain + offscreen (or two
  offscreens, headless fallback) render through a single `GpuDevice`; the
  probe reports a `DeviceToken` per target and asserts identity.
- **Resource cache**: bounded (LRU, capacity configurable), keyed by
  (owner-identity, role); generational validation on every lookup; explicit
  release; observable counters (created/hits/misses/evicted/released/
  stale_rejected).
- **Text/models/audio**: dependencies locked (§2) but not yet wired — support
  is claimed only from T-04/T-05/T-06 implementation evidence.
- **Model/texture formats**: not yet validated (T-05 loads the frozen GLBs).
  Texture support today: RGBA8 upload path only.

## 4. View / frame / resource-queue strategy

- One `wgpu::Queue` per host; all frame submissions are ordered on it.
  2D frames go through a single storage-buffer quad pass
  (`QuadPipeline::render` is the only 2D submission point).
- Per-frame uploads are `queue.write_buffer` diffs (camera matrix + instance
  list), never full resource recreation. Resizes recreate only the target
  texture/surface config — pipeline and device untouched.
- The cache never recreates entries whose key+identity is unchanged; probe
  asserts `created` counter is flat across the lifecycle (§6).
- Draw calls per 2D frame: 1 pass, up to 4096 quads (instance buffer).

## 5. Failure taxonomy (observable, classified)

| Classification | Trigger | Behavior (last-good preserved) |
|---|---|---|
| `adapter_unavailable` | no GPU adapter | probe exits 3, no report body |
| `device_lost` | request_device failure / GPU reset | error returned; probe aborts with code |
| `surface_lost` | swapchain `Lost`/`Outdated` | one reconfigure from last valid config, then retry once; else classified |
| `zero_size_target` | 0×0 configure/resize (minimize) | rejected; presentation suspends; last config kept |
| `missing_file` | asset path absent | rejected before any GPU work |
| `bad_shader` | WGSL validation failure | caught via device error scope → classified error, not a panic |
| `stale_identity` | freed/reused/cross-session id | change rejected, scene/cache untouched |
| `cache_full` | bounded cache exhaustion (non-reusable key) | LRU eviction first; classified if impossible |
| `resource_released` | double release | second release rejected |
| `report_io` | report/keyframe write failure | probe exits 3 |

Policy: **a failed operation never clobbers the last-good scene** — frames
that fail return the error to the caller; the previous presented frame stays
visible. Late/zero-size presentations are suspension, not failure.

## 6. Measured device facts (from the probe report on THIS machine)

Probe: `graphics-probe --case gpu-foundation --replay canonical --report
scratch/829/gpu-foundation` (run 2026-10-20, release; report artifacts under
`scratch/829/gpu-foundation/` and `scratch/829/gpu-foundation-headless/`).
Values below are copied from `report.json` — not fabricated.

- **Adapter**: NVIDIA GeForce RTX 4060 Ti (DiscreteGpu), vendor 4318, device 10245
- **Backend**: Vulkan — driver `NVIDIA 617.14`
- **Device identity**: token `dev0:5041b2d12f80b95e` identical across BOTH
  targets (window swapchain + offscreen) — V-01 same-device requirement holds
- **Window**: REAL window created (winit 0.30, mode `window`); initial inner
  1600×1200 physical @ **scale factor 2.0** (800×600 logical); 13 window
  frames presented, 12 offscreen frames rendered
- **Lifecycle observed**: resize 800×600 → 1024×640 logical ([2048,1280]
  physical events observed) → minimize ([0,0] observed, presentation
  suspended, last config kept) → restore ([2048,1280] observed, presented
  again). Zero-size never errors — it suspends.
- **Keyframes**: 3 offscreen PNG captures (frames 0/6/11), sha256
  `0f915b70…`, `f78ef71d…`, `5c23f788…` — three distinct hashes (moving-quad
  evidence). The headless fallback run produced **bit-identical** keyframe
  hashes on the same adapter (deterministic offscreen rendering).
- **Resources**: cache capacity 8, 4 entries created up front; post-lifecycle
  re-lookup produced 4 hits / 0 new creations — untouched entries survived
  (no per-frame rebuild). Live entries 4.
- **Failure injection (all 5 rejected with the expected classification,
  last-good preserved)**: `zero_size_target`, `missing_file`,
  `bad_shader` (WGSL validation via device error scope), `stale_identity`
  (freed/reused owner id into cache), `resource_released` (double release).
- **gpu_timing_available: false** — no timestamp queries in T-02 (explicitly
  recorded rather than fabricated).
- **Exit**: `pass` (windowed and headless both).

## 7. Platform / input profile exercised

- Windows x64 native window (winit 0.30), keyboard/mouse session profile
  (Plan §1). Gamepad/browser/Linux/mobile: not exercised, recorded as known
  profile differences.
- DPI: scale factor recorded by the probe (§6).
- **GPU timing: unavailable** — timestamp queries are not wired in T-02; the
  report carries `gpu_timing_available: false` explicitly rather than
  fabricating numbers. CPU-side frame counts and wall-clock run time only.

## 8. Resource budgets

- Cache capacity: default 8 entries (probe), sized per game at T-03+.
- 2D instance budget: 4096 quads/frame.
- Keyframe captures: probe-only; each 800×600 RGBA readback ≈ 1.9 MB CPU.
- No FPS/vram claims are made in T-02 (Plan §5.3: baselines recorded first,
  budgets frozen here, load verification in T-07).

## 9. Deviations / notes

- glyphon unusable at wgpu 27 (§2) — text decision differs from the most
  obvious candidate; recorded with index evidence.
- `wgpu::PollType` (wgpu 27) replaces `Maintain`; `PipelineLayoutDescriptor`
  keeps `push_constant_ranges`; `RenderPassColorAttachment` gained
  `depth_slice`; `RenderPipelineDescriptor` uses `multiview` — API drift
  against older wgpu documented for future upgrades.
- glam 0.34 removed `Mat4::orthographic_rh`; AGP writes the 2D ortho matrix
  explicitly (renderer-owned, 20 lines) to avoid projection-API drift.
- kira locked at 0.9.x although 0.12 exists: 0.9 is the mature symphonia-0.5
  line with the API T-04/T-06 will target; no wgpu coupling makes staleness
  low-risk. Re-evaluation gate: if T-06 hits a kira 0.9 limitation, bump with
  evidence in the plan file.
