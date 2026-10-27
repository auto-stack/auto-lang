# AGP 3D Feasibility — ag3d-risk-slice results (T-05, PLAN-829)

Purpose: de-risk the hard parts of AG-3D-01 (Godot "Squash the Creeps") on
real frozen assets BEFORE T-06 builds the game. Probe case `ag3d-risk-slice`
(windowed, exit 0 on this machine); report + keyframes under
`scratch/829/ag3d-risk-slice/`. Verdict per risk item, then measured facts,
then what remains for T-06. Nothing here is downgraded: failures would be
recorded as failures.

## 1. Verdicts

| Risk item | Verdict | Evidence |
|---|---|---|
| GLB loading (gltf 1.4) of the frozen models | **GO** | `player.glb` (3 meshes: 273/687/273 verts, materials pupil/body/eye) and `mob.glb` (273/974/273, mob_eye/mob_body/pupil) load; material baseColorFactors match the frozen tree; node transforms (incl. the mob's rotation quat (-0.5,-0.5,0.5,0.5) + scale (0.953,1,1), which the T-01 dump missed — see §4) are baked into vertices. Failure taxonomy verified: `missing_file`, `bad_model_parse` (`threed_contract::glb_failure_taxonomy_missing_and_corrupt`); textured models rejected as `unsupported_extension` by design. |
| Identity-keyed GPU mesh cache | **GO** | `MeshStore` keyed by (model sha256, mesh name); 8 meshes uploaded once for the whole run; uploads happen only on first use (probe: `uploaded_meshes = 8`, no per-frame rebuild). |
| Lit mesh rendering + depth | **GO** | Perspective AND orthographic (explicit 0..1-z matrices, y-up; unit-tested) — the sample camera is **orthographic size=24 far=40** (`Main.tscn` lines 101–106), implemented as `Camera3d::sample_camera`. Depth buffer per target size. 20 draw instances → 20 grouped draw calls per frame (probe stats). |
| Directional shadow mapping | **GO (after one real bug)** | 2048² Depth32Float shadow map, comparison-sampler PCF, Lambert×shadow fragment. Keyframes `anim_t45_shadow_on` vs `anim_t45_shadow_off` are hash-distinct AND visually verified (soft shadows fall lower-left of every character, matching the light from the frozen `DirectionalLight3D` transform, Main.tscn lines 84–91). Bug found by viewing keyframes: the shadow camera was placed opposite the light — back-face culling emptied the depth map (uniform darkening); fixed by placing the camera AT the light. |
| Inline bob animation | **GO** | Frozen from `Player.tscn` lines 17–43 / `Mob.tscn` lines 12–38 (identical tracks): length 1.2 s, loop, linear interp; position y keys (0.22, 0.329753)/(0.7, 0.660351)/(1.18, 0.349734); rotation x keys (0.1, −5.0326°)/(0.46, 10°)/(1.18, −10°). Applied as Pivot/Character local transform; deterministic (`render3d::anim` unit tests + distinct animation-phase keyframes t0/t45/t90). |
| Collision scheme | **GO (pure-state)** | The sample's actual scheme works without a physics engine: sphere(r=0.792278) player body, mob OBB (1.35822×1.08835×2.20058, yaw-only), ground plane, arena inward planes at the frozen Wall positions; top-contact normal·UP>0.1 ⇒ squash+bounce 16, other contacts fatal; 120 Hz fixed step with floor slop (needed: exact rest gives pen==0 and `is_on_floor` flaps without it). Query evidence in the probe report: `TopLandingSquash` / `Fatal` / rest_y=0.792278 / arena clamp (13.208, 15.208). |
| Environment look | **GO** | Sky-horizon clear color (0.64625, 0.65575, 0.67075, Main.tscn lines 17–19), 60×60 ground, four decorative cylinders (24 segments, albedo (0.635294, 0.211765, 0.0235294)) at the frozen world positions (14,1,−15)/(−13,1,−15)/(−13,1,16)/(14,1,16) — visible in keyframes. |

**Overall: GO for T-06.** Every load-bearing technique is proven on the real
frozen assets on the target profile.

## 2. Measured facts (this machine, from the probe report)

- Adapter: NVIDIA GeForce RTX 4060 Ti, **Vulkan**, driver NVIDIA 617.14
  (same device/token family as the T-02 gpu-foundation run).
- Window: real window opened (mode `window`); offscreen keyframes 900×600.
- Frame content: 20 instances (ground + 4 cylinders + player + 4 mobs × 3
  primitives), 20 draw calls (one per mesh group), 8 uploaded meshes.
- Keyframes: 4 distinct sha256 (`anim_t0_shadow_on` 9d329f19…,
  `anim_t45_shadow_on` 68880551…, `anim_t45_shadow_off` d236fb36…,
  `anim_t90_shadow_on` 31dd859b…); shadow on/off distinct ✓.
- **GPU timing: unavailable** (`gpu_timing_available: false`) — no timestamp
  queries wired; no FPS numbers are claimed anywhere in this document.
- CPU-side wall time: the whole 93-frame windowed run (incl. 4 readback
  captures) completes in well under a second of real time; no per-frame
  allocation growth was observed (meshes/atlas uploaded once). A P50/P95
  study is deliberately left to T-07 (V-07) with its measurement contract.

## 3. Honest gaps / T-06 items

1. **Mob spawn height — CORRECTED 2026-10-21 (review r6 F829-R6-01)**: the
   earlier claim here (and in the T-06 sim) that mobs spawn at y=1
   "matching the frozen source" was a LOCAL/GLOBAL coordinate misread.
   `Main.gd:37` passes `mob_spawn_location.position` — the PathFollow3D's
   LOCAL position (SpawnLocation local y=0; curve points y=0) — and
   `Mob.gd` adds the mob directly under Main, so SpawnPath's transform y=1
   (Main.tscn line 128) never reaches the mob. Mob.gd has no gravity, so
   mobs keep y=0 for their whole flight. Verified against the real Godot
   4.4.1 fixture running the unchanged frozen scripts (four seeds, all
   `actual_spawned_mob_y=0`; review-r6 `source-spawn-stdout` receipt). The
   T-06 game sim, the independent Python generator, the frozen golden
   trace and the content frames were corrected and re-frozen TOGETHER
   under this source expectation (source-fidelity correction, not tuning).
   Related coordinate paths rechecked in r5: the player is integrated as
   the SPHERE CENTER (source: the sphere rests with its center at radius
   height — node origin at center − 0.771765, a 2 cm model-origin nuance
   that remains the documented approximation); the MobDetector cylinders
   stay player-local as implemented and validated by the F03/R4 probes;
   the spawn rectangle x/z is unaffected (SpawnPath x/z = 0).
2. **WorldBoundaryShape3D semantics**: Godot's one-sided infinite planes were
   approximated by inward-facing planes at the frozen shape positions
   (Main.tscn lines 59–82). Player containment is proven at gameplay level in
   T-06 (arena-clamped player in the canonical trace, zero mismatches over
   3600 ticks); the approximation stands as documented.
3. **No skeletal animation anywhere**: confirmed again — both GLBs have zero
   animations/textures; the only animation is the inline float-bob. Nothing
   skeletal needs building for this sample.
4. **Physics is custom, not Jolt — validated in T-06**: the 120 Hz step with
   the single-pass resolve approximation reproduces the independent Python
   reference BIT-EXACTLY (max position diff 0.0 over the full 3600-tick
   canonical run incl. 3 squashes + bounce impulses and the death at t=1402).
   The approximation stands as the frozen oracle implementation.
5. **RNG streams — DONE in T-06**: driver-owned `DeterministicRng` with the
   frozen seed; the canonical spawn stream is frozen in
   `testdata/squash/golden_squash_trace.json` and matches the independent
   generator bit-for-bit. (r6 re-freeze: seed 0x1B, 45 spawns, spawn
   height y=0 per the corrected source call path.)
6. **Text/HUD for squash**: not part of this slice (score label reuses the
   T-04 text pipeline; verified there).
7. **Audio — DONE in T-06**: the looping "House In a Forest Loop.ogg"
   (autoplay, loop=true per the frozen .import) plays through kira/cpal from
   session start; the probe report records real playback state `Playing`.

## 4. Evidence-note correction (T-01 follow-up)

The T-01/T-05 GLB inspection initially reported the mob node as
"translation + scale only". The node ALSO carries rotation quaternion
(-0.5, -0.5, 0.5, 0.5). The asset loader bakes the full node transform
(rotation + non-uniform scale via inverse-transpose normals); the T-01
manifest's asset table remains correct in substance (no animations, no
textures) and this adds the rotation fact — noted here rather than editing
the frozen manifest tables.
