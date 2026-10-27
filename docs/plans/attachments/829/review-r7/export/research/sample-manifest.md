# AGP Frozen Sample Manifest (T-01, PLAN-829)

Authoritative oracle inventory for the two frozen game samples replicated by
`experimental/autographics-prototype/` (AGP). Downstream tasks (T-02..T-09)
MUST treat this file plus `testdata/` and `assets/` as the frozen oracle.
The implementation must not modify this oracle; corrections require a plan
revision with evidence (see `research/evidence-notes.md`).

Download date: **2026-10-20 UTC** (all integrity checks below executed the same day).

Network access note: `raw.githubusercontent.com` was unreachable from the
work machine (connection reset); all frozen content was obtained via `git
fetch --depth 1 origin <commit>` from `github.com` (which is reachable) into
scratch clones, then copied out. Blob hashes were verified with
`git hash-object` inside the cloned trees.

---

## 1. AG-2D-01 — Bevy Breakout

### 1.1 Frozen source identity

| Field | Value |
|---|---|
| Repository | `bevyengine/bevy` |
| Commit | `efe61cc5b9872f3f9e1ee7553325612c094ccc97` (fetched 2026-10-20; commit title "Fix JsonSchemaBevyType and registry.schema ... (#26209)") |
| File | `examples/showcase/breakout.rs` |
| Recorded blob (Plan §4.3) | `b9bfb8b4b2cd19aa15b460f1323ad5c2af0e5954` |
| Measured `git hash-object` | `b9bfb8b4b2cd19aa15b460f1323ad5c2af0e5954` — **MATCH** |
| Size / sha256 | 14085 bytes / `e33be56b22bc60ffa518c67ae34b23aead0c0f3e346e9b355609df19540d4848` |
| Companion module | `examples/showcase/stepping.rs` (compiled via `mod stepping;`), blob `a59a5b6cbc2170284da898f72ea0957c5cc0f126`, sha256 `7541abadd9c9f7d697c58c4b92229edf8401ed66da44df989001f94ce35c82e8`. Stepping UI is inert without the `bevy_debug_stepping` cargo feature (`stepping.rs` line ~36: `if cfg!(not(feature = "bevy_debug_stepping")) { return; }`); it is a debug tool, **not part of the gameplay profile** (consistent with Plan V-03). |
| Local copy | `testdata/breakout/breakout.rs`, `testdata/breakout/stepping.rs` |

### 1.2 Gameplay rules parsed from the actual source

All values below are quoted from `breakout.rs` at the frozen commit (line
numbers refer to that file).

**Entities (spawned in `setup`, lines 169–287):**

- `Camera2d` (line 176).
- `Paddle` + `Collider`: sprite color `srgb(0.3,0.3,0.7)`, size `120.0 x 20.0`
  (`PADDLE_SIZE`, line 11), translation `(0, BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR, 0)`
  = `(0, -300 + 60 = -240, 0)` (line 183).
- `Ball`: mesh circle, color `srgb(1.0,0.5,0.5)`, diameter `30.0`
  (`BALL_DIAMETER`, line 19), starting position `(0.0, -50.0, 1.0)`
  (`BALL_STARTING_POSITION`, line 18), z=1 "renders on top".
  Initial velocity = `INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED`
  (line 203): direction `(0.5,-0.5)`, speed `400.0` → velocity ≈
  `(282.8427, -282.8427)` units/s.
- Scoreboard UI: root `Text::new("Score: ")` with child `TextSpan`, font size
  `33.0 px` (`SCOREBOARD_FONT_SIZE`, line 39), text color `srgb(0.5,0.5,1.0)`,
  span color `srgb(1.0,0.5,0.5)`, absolutely positioned at top-left padding
  `5.0 px` (lines 207–229). The span is overwritten each frame with
  `score.to_string()` (line 328). No custom font is loaded — Bevy's built-in
  default font is used.
- 4 walls (`Wall`, `Collider`): thickness `10.0` (`WALL_THICKNESS`), left
  center x `-450`, right `450`, bottom center y `-300`, top `300` (lines 23–29),
  color `srgb(0.8,0.8,0.8)`. Wall sprite size = arena span + thickness
  (left/right: `10 x 610`; top/bottom: `910 x 10`), centered on the arena.
- Bricks (`Brick`, `Collider`): size `100 x 30` (`BRICK_SIZE`, line 31), color
  `srgb(0.5,0.5,1.0)`. Grid geometry (lines 238–262):
  - `total_width = (450-(-450)) - 2*20 = 860`; `bottom_edge = -240 + 270 = 30`;
    `total_height = 300 - 30 - 20 = 250`.
  - `n_columns = floor(860 / (100+5)) = 8`; `n_rows = floor(250 / (30+5)) = 7`.
  - `left_edge = 0 - (8/2*100) - (7/2*5) = -417.5`; `offset_x = -417.5 + 50 = -367.5`;
    `offset_y = 30 + 15 = 45`.
  - Grid = **7 rows × 8 columns = 56 bricks**, first brick center `(-367.5, 45)`,
    step `(105, 35)`.
- Background clear color `srgb(0.9,0.9,0.9)` (line 42, `ClearColor`).

**Movement (per `Update` schedule, systems chained in order
`apply_velocity` → `move_paddle` → `check_for_collisions`, lines 63–67):**

- `apply_velocity` (316): every entity with `Velocity` moves
  `translation += velocity * delta_secs` (frame-time scaled; see oracle §1.5
  for the fixed-timestep freeze).
- `move_paddle` (289): direction = `+1` if `ArrowRight` pressed, `-1` if
  `ArrowLeft` (both may cancel). New x = `x + direction * 500.0 * delta_secs`
  (`PADDLE_SPEED` = 500). Then clamped to
  `[LEFT_WALL + 10/2 + 120/2 + 10, RIGHT_WALL - 10/2 - 120/2 - 10]` =
  **`[-390, 390]`** (`PADDLE_PADDING` = 10, lines 310–313).

**Collision / bounce semantics (`check_for_collisions`, lines 331–382):**

- Ball is a `BoundingCircle` (center = translation.xy, radius `15.0`); each
  collider is an `Aabb2d` (center = translation.xy, half-size = scale.xy/2).
- On overlap: `BallCollided` event is triggered (→ sound, see 1.4), and the
  collision side is computed in `ball_collision` (402–422): closest point on
  the AABB to the ball center; if `|offset.x| > |offset.y|` → side is `Left`
  when `offset.x < 0` else `Right`; else `Top` when `offset.y > 0` else
  `Bottom`.
- Reflection is **conditional on velocity direction** (prevents sticking):
  `Left` ⇒ flip x if `vx > 0`; `Right` ⇒ flip x if `vx < 0`;
  `Top` ⇒ flip y if `vy < 0`; `Bottom` ⇒ flip y if `vy > 0` (lines 364–379).
- If the collider is a `Brick`: it is despawned and `Score += 1` (lines 353–356).
- The ball loops over ALL colliders each frame; multiple contacts in one frame
  can each trigger reflection and `BallCollided`.
- **Bottom-wall bounce confirmed; there is NO ball-loss, lives, or game-over
  logic anywhere in the frozen source. Do not invent one.** The host `reset`
  entry (Plan §5.2) is an AGP extension, not a sample rule.

**Scoring:** `Score(usize)` resource starts at 0 (line 58). +1 per brick
despawn. Scoreboard span updated every frame from the resource. No win
condition; clearing all 56 bricks leaves the ball bouncing forever.

### 1.3 Input bindings (actual)

- `ArrowLeft` / `ArrowRight` — paddle movement (lines 296–302). Keyboard only.
- No mouse input, no other keys. Known profile difference: none in the sample
  itself (Bevy examples take no gamepad input here); Plan §1 restricts AGP to
  keyboard+mouse anyway.

### 1.4 Audio triggers (exact event → sound mapping)

- Asset loaded once at setup: `sounds/breakout_collision.ogg`
  (`asset_server.load`, line 179), stored in `CollisionSound` resource.
- Every frame in which the ball overlaps ANY collider (wall, paddle, or brick —
  each overlapping collider separately), `commands.trigger(BallCollided)` is
  issued (line 350). The observer `play_collision_sound` (384–390) spawns
  `(AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN)` — i.e. one
  one-shot playback instance per trigger, despawned when finished.
- Exact mapping: **ball∩collider contact ⇒ `breakout_collision.ogg` one-shot,
  no volume/pitch variation**. A frame with ball touching both a wall and a
  brick triggers the sound twice.

### 1.5 Breakout replay/oracle freeze proposal (binding for T-03/T-04)

- Simulation: fixed timestep `dt = 1/60 s`, systems in frozen order
  (move ball → move paddle from queued input → collisions → scoreboard).
  Input recorded as per-tick `ArrowLeft/ArrowRight` booleans.
- Determinism: the sample is RNG-free; golden trace must match bit-for-bit on
  positions/score for a given input trace when using f32 arithmetic in the
  same operation order; cross-check tolerance for a reimplementation:
  position `|Δ| ≤ 1e-3` units per axis, velocity direction angle `≤ 1e-3 rad`,
  score exact integer, collision event sequence exact.
- Brick count fixed at 56 with the grid formula above; oracle asserts initial
  layout exactly (56 centers).
- Image comparison (golden frames): per-pixel comparison in sRGB 8-bit;
  proposed freeze — mean absolute channel diff ≤ 2/255, 99.9th-percentile
  absolute diff ≤ 8/255, against renderer-approved golden keyframes
  (background, initial frame, post-first-bounce, post-first-brick, all-bricks
  cleared). Non-bitwise across GPUs by design (Plan V-03 allows).
- Audio: oracle asserts trigger count == collision-contact count per frame and
  that the collision ogg is the only audio asset on the playback path.

---

## 2. AG-3D-01 — Godot "Squash the Creeps"

### 2.1 Frozen source identity

| Field | Value |
|---|---|
| Repository | `godotengine/godot-demo-projects` |
| Commit | `b761b4cd5718a7118f2c95eb55f181d0a5f004f1` (fetched 2026-10-20) |
| Directory | `3d/squash_the_creeps/` |
| README blob (Plan §4.3) | recorded `8006f2783acf3ad8baceb63a80269b36b0c16138` — measured `8006f2783acf3ad8baceb63a80269b36b0c16138` — **MATCH** |
| Main.tscn blob (Plan §4.3) | recorded `105c56d67284326f5efdb5caf426c0f063f9f255` — measured `105c56d67284326f5efdb5caf426c0f063f9f255` — **MATCH** |
| Directory enumeration | via `git ls-tree` on the fetched commit (equivalent to the GitHub API tree enumeration); full inventory in §2.5 |
| Local text copies | `testdata/squash_the_creeps/` (scripts, scenes, project.godot, theme, README, .import metadata, font license) |

### 2.2 Gameplay rules parsed from the actual source

Values quoted from `Player.gd`, `Mob.gd`, `Main.gd`, `ScoreLabel.gd`,
`Main.tscn`, `Mob.tscn`, `Player.tscn`, `MusicPlayer.tscn`, `project.godot`.

**Player (`Player.gd` + `Player.tscn`):**

- `CharacterBody3D`, speed `14` m/s, `jump_impulse = 20`, `bounce_impulse = 16`,
  `fall_acceleration = 75` m/s² (exported vars, lines 6–12).
- Per physics tick: build `direction` from `move_left/right/forward/back`
  actions (x/z ±1, combined then normalized); when moving,
  `basis = Basis.looking_at(direction)` (snap facing) and
  `AnimationPlayer.speed_scale = 4`, else `speed_scale = 1` (lines 16–33).
- `velocity.x = direction.x * 14`, `velocity.z = direction.z * 14` (horizontal
  velocity is set absolutely, no inertia).
- Jump: if `is_on_floor()` and `jump` just-pressed → `velocity.y += 20`.
- Gravity: `velocity.y -= 75 * delta` every tick, then `move_and_slide()`.
- Squash check after sliding: iterate slide collisions; if collider is in
  group `mob` and `Vector3.UP.dot(collision_normal) > 0.1` (landed on top) →
  `mob.squash()`, `velocity.y = 16`, loop broken (max 1 squash/point per tick)
  (lines 53–62).
- Visual: `rotation.x = PI/6 * velocity.y / jump_impulse` — character pitches
  with vertical velocity (line 65).
- `die()`: emits `hit`, `queue_free()`; `_on_MobDetector_body_entered` →
  `die()` — the `MobDetector` Area3D (collision_mask = 2 = enemies layer) with
  one cylinder + three small cylinder shapes detects any mob body touching the
  player's sides/upper area ⇒ **any non-top contact kills the player**.
- Collision shapes: main sphere radius `0.792278` at y `0.771765`.
- Player collision_mask = `2147483654` (layers 2 "enemies", 3 "world",
  and an unused high bit): collides physically with mobs and the world.
- Animation: `AnimationPlayer` autoplay `float` — an **inline** 1.2 s looping
  animation defined in `Player.tscn` (tracks on `Pivot/Character:position` —
  keys at 0.22/0.7/1.18 s, values y `0.329753 → 0.660351 → 0.349734` — and
  `rotation_degrees` x `-5.0326 → 10 → -10`, keys at 0.1/0.46/1.18 s). The GLB
  model contains **no** animations (verified by parsing the GLB JSON chunk);
  the bobbing comes entirely from this scene-level animation.

**Mob (`Mob.gd` + `Mob.tscn`):**

- Spawned by `Main._on_mob_timer_timeout` every `MobTimer.wait_time = 0.5` s
  (Timer `autostart = true`, Main.tscn lines 137–139). Spawn point: mob scene
  instantiated, `SpawnPath/SpawnLocation.progress_ratio = randf()` (uniform on
  the SpawnPath rectangle Curve3D with points (14,0,-15), (-13,0,-15),
  (-13,0,16), (14,0,16), at y=1), then `mob.initialize(spawn_pos, player_pos)`.
- `initialize` (Mob.gd 16–32): `look_at_from_position(start, (player.x,
  start.y, player.z), UP)` — aim horizontally at the player; then
  `rotate_y(randf_range(-PI/4, PI/4))` (±45°); speed `randf_range(10.0, 18.0)`
  (`min_speed`/`max_speed`); `velocity = FORWARD * speed` rotated by Y.
  `AnimationPlayer.speed_scale = random_speed / 10`.
- Movement: `move_and_slide()` only — constant velocity, no gravity, no
  avoidance. Mob collision_layer = 2 (enemies), collision_mask = `2147483648`
  (no defined layer) ⇒ mobs physically collide with nothing; the player and
  the MobDetector Area handle all interactions.
- `squash()`: emits `squashed`, `queue_free()`.
- `_on_visible_on_screen_notifier_screen_exited` → `queue_free()` — mobs
  leaving the screen are freed.
- Mob collision shapes: main box `1.35822 x 1.08835 x 2.20058` at y `0.52793`
  plus 4 small boxes (`0.418579 x 0.569824 x 0.833984`) for the "creeps".
- Mob animation: same inline `float` 1.2 s loop as the player, autoplay.

**Score / win-lose (`ScoreLabel.gd`, `Main.gd`):**

- `score = 0`, `_on_Mob_squashed(): score += 1; text = "Score: %s" % score`.
  Initial label text `"Score: 0"` (Main.tscn line 158). Score increments
  exactly once per squashed mob (the player loop `break` guarantees 1 point
  per mob even with multiple contacts).
- Player hit → `Main._on_player_hit`: `MobTimer.stop()`, show
  `UserInterface/Retry` (fullscreen `ColorRect` black α=0.447059 with centered
  label **"Press Space or Enter to retry"**, font size 32). The player node is
  freed by `die()`. Game world freezes (no player) except existing mobs.
- Retry input: `event.is_action_pressed("ui_accept")` while Retry visible →
  `get_tree().reload_current_scene()` (full scene reload). `ui_accept` is
  Godot's built-in default (Space / Enter) — it is NOT overridden in
  `project.godot`.
- No win condition; play continues until hit.

**Camera / environment (`Main.tscn`):**

- `CameraPivot` rotated 45° down (x basis ~ sin/cos 45°); child `Camera3D`:
  `projection = 1` (orthogonal), `size = 24.0`, `fov = 48.6` (unused in ortho),
  `far = 40.0`, positioned z=19 from pivot (lines 98–106). Fixed camera; no
  camera movement anywhere in the scripts.
- `DirectionalLight3D`: transform tilted ~45°, `shadow_enabled = true`,
  `shadow_bias = 0.04`, `shadow_blur = 1.5`,
  `directional_shadow_max_distance = 40.0` (lines 84–91). **Real-time
  directional shadows are a sample requirement** (Plan AC-04: must not be
  silently dropped).
- `WorldEnvironment`: `background_mode = 2` (sky), procedural sky with
  horizon color `0.64625/0.65575/0.67075`, `tonemap_mode = 4`.
- `Main._ready`: on the `gl_compatibility` renderer only, shadow quality is
  raised and a second dimmed light added — a compatibility workaround that is
  a no-op on the Forward+ profile (`project.godot` renderer not pinned there;
  README states "Renderer: Forward+").
- Ground: `StaticBody3D` box `60 x 2 x 60` at y=-1, layer 4 ("world").
- 4 invisible world-boundary planes (`WorldBoundaryShape3D`, "walls") keep the
  player inside the arena (layer 4). Walls collision transform at z offset +5.
- Decorative cylinders: 4 `CylinderMesh` (24 segments, albedo
  `0.635294/0.211765/0.0235294`) around the arena.
- Physics (project.godot): `physics_ticks_per_second = 120` (**fixed 120 Hz
  physics**), `physics_engine = "Jolt Physics"`,
  `physics_interpolation = true`; MSAA 3D = 2 (4x); directional soft shadow
  filter quality = 3.

### 2.3 Input bindings (actual, from `project.godot [input]`)

| Action | Bindings in frozen project |
|---|---|
| `move_left` | Left Arrow (keycode 4194319), physical **A**, joypad axis 0 negative |
| `move_right` | Right Arrow (4194321), physical **D**, joypad axis 0 positive |
| `move_forward` | Up Arrow (4194320), physical **W**, joypad axis 1 negative |
| `move_back` | Down Arrow (4194322), physical **S**, joypad axis 1 positive |
| `jump` | physical **Space**, **right mouse button**, joypad button 0 |
| retry | `ui_accept` (Godot built-in default: Space / Enter) |

- Keyboard profile used by AGP (Plan §1): arrows + WASD + Space. **Known
  profile differences (recorded, not replicated):** gamepad left stick /
  A-Cross button, and right-mouse jump. README also documents them.
- Note: movement is 4-way digital (no analog scaling); `deadzone 0.2` only
  matters for the gamepad profile.

### 2.4 Audio (actual playback path — verified, see evidence note E1)

- `MusicPlayer.tscn`: an `AudioStreamPlayer` with `autoplay = true` playing
  `res://art/House In a Forest Loop.ogg`. It is instanced as an **autoload
  singleton**: `project.godot [autoload] MusicPlayer="*res://MusicPlayer.tscn"`.
  So the music **does actually play** in the frozen sample (looping —
  `.import` param `loop=true`), despite not appearing as a node in `Main.tscn`.
  No other audio exists in the sample: no squash/sound effects, no death
  sound. This is the complete audio denominator for AG-3D-01.

### 2.5 Full asset inventory of the frozen tree (all files verified to exist)

Paths relative to `3d/squash_the_creeps/`. Columns: size (bytes) / sha256 /
git blob. Download URL form:
`https://raw.githubusercontent.com/godotengine/godot-demo-projects/b761b4cd5718a7118f2c95eb55f181d0a5f004f1/3d/squash_the_creeps/<path>`
(reachable hosts may vary; the git remote `https://github.com/godotengine/godot-demo-projects.git`
at the same commit is the verified retrieval path).

**Scripts & scenes (copied to `testdata/squash_the_creeps/`):**

| path | bytes | sha256 |
|---|---|---|
| Main.gd | 1581 | `459f8f8735731072598b30e283fc62117063539b6162afb9979c774dc7d9613f` |
| Mob.gd | 1276 | `12bc135f704ed11bfc3dce6acafbd795502e615679bd49e1de92a895f87e33e4` |
| Player.gd | 2561 | `e138ba5f4ded1cd2ef2360457a8a00755b6125a8369d89f478044cd0013ab90b` |
| ScoreLabel.gd | 95 | `b4e10f3b95a80c34b67d5e8873501ab3d7733abfd4ca91f5570143290e44e9c9` |
| Main.tscn | 7227 | `87c2140064ade9d98a101edb9ec0d2b08536757eb9f47b58dd2e85d48fe29819` |
| Mob.tscn | 3402 | `9bef41c3c459f65906b8711b51b8e9a7a06c9c205270598f0ee72ad5d6be643a` |
| Player.tscn | 3438 | `9e7ad8257a301521ef535968a69941972e05f56a6eb63a181d4c9bd53f44e2ab` |
| MusicPlayer.tscn | 273 | `a8724735c32f9644549ea60f3fa9167dc5012404139f10b4e31872430bb5db53` |
| project.godot | 5194 | `5908d6498c89cd83fe13c692bd76360f0149ae08b718d7f471cf3a56e54067b4` |
| ui_theme.tres | 210 | `7a764260a2dec2c0bbcf2a471064044c33e40bc702c37884cc69df0cd17eadd1` |
| README.md | 1900 | `62a1bd30dc3fca56ff9c98ef40c304fe7a5ed75f24502894fa5623dd0e7ec0b0` |
| fonts/LICENSE.txt | 4470 | `7a05af5e0552b46fbca086d777a251897082e77b840ab59199916a83cbab24f7` |
| art/body.tres | 211 | `e4b0ac534f88f514fc3cbab87561ea183196ec3c1ba538eee3179b8d190e89de` |
| art/eye.tres | 306 | `7cd9238df895fe5f3d983a3c627339886afd50a9383adc09f4011a4086c02459` |
| art/mob_body.tres | 200 | `103e38579b2378f54ceecfd64bf1bfae719686124d6fc6d1763ca557f2c3bb9` |
| art/mob_eye.tres | 313 | `90b3cc479b3537725aa188034855e1b04d5b06dc350efaae89217f88cd507d30` |
| art/pupil.tres | 240 | `8e23bb2a5573c9ab5edc309b2479a102d68fb4e4f7c4995364fa0ec431f5a81a` |
| art/mob.glb.import | 1131 | `501f1c1896f860bbaad942146af5dc3555418af8621c2f25a7ca1cc3f8bff878` |
| art/player.glb.import | 1140 | `3824f1f5aa25998a56fcb575ff4b64dd2a4438416893b2bb1c19790cdebea8a0` |
| art/House In a Forest Loop.ogg.import | 260 | `99a39f5e2910b2f7e0f71d152ede1345613b68ece4497fcec459766657659a0` |

The five `art/*.tres` material files are **unreferenced** by any scene, script,
or .import in the frozen tree (grep-verified) — the GLBs embed their own
materials. Kept in testdata for completeness; not gameplay inputs.

**Binary / large files (hashes recorded here; needed subset downloaded to `assets/`):**

| path | bytes | sha256 | license | in `assets/` |
|---|---|---|---|---|
| art/mob.glb | 63828 | `8a82adb5bfba8c22f9c1a0d692e4d25c2f9055cb7e6f6acd6f61ac169079818c` | no per-file license; Godot demo project assets per repo root (MIT for code; see note N1) | yes |
| art/player.glb | 52392 | `05d8588e8032a674e309ccb4d04578fb578b8da1629096b1036f628593801942` | same as above | yes |
| art/House In a Forest Loop.ogg | 1430743 | `32680f2b2631eb9f6e2902ebbbec1b43ab82b4dbf154fdc9bfbfa17b03270504` | **CC-BY 3.0**, © 2012 HorrorPen (per frozen README §License) | yes |
| fonts/Montserrat-Medium.ttf | 260236 | `92b3d3c6e135eb1dc95f88e6ca75bd6113d9eb3261a95ca39f733e3897e53675` | **SIL OFL 1.1**, © 2011 The Montserrat Project Authors (frozen `fonts/LICENSE.txt`) | yes |
| art/mob.blend | 140616 | `2adaff536949f384f30f926d7798816d9353feca7a771acbe8adadb46dd971f9` | source art; not needed at runtime | no |
| art/player.blend | 162488 | `1c7f85c3935c2aed31d63bab9c3cf9ad4cfc7c4d44b888608ea826d177322afa` | source art; not needed at runtime | no |
| icon.webp | 13490 | `3361a4b47413c65ef2c846a9170f603757fc0c89aba25e3274b7cf7d9258bf7e` | project icon; not used in gameplay | no |
| icon.webp.import | 919 | `0f8159c229ad2363e692c6410abeb4700c3e9da6826a59e50c3e6ee2a6d08f5c` | — | no |
| fonts/Montserrat-Medium.ttf.import | 780 | `7a489ff4f0bc087c5d27b386ca5b317d7f58a387f3416e60c2820dd8c087b2dc` | — | no |
| screenshots/.gdignore | 0 | (empty) | — | no |
| screenshots/squash_the_creeps.webp | 50164 | `75f8644f9a93bff9074c4c73bf8faf0b875cb1077b2a1f01d3f55715d08e3d02` | screenshot; reference only | no |
| *.gd.uid (5 files, 19–20 bytes each) | — | see git tree | editor uid metadata | no |
| .gitignore | 14 | `b3e95a48ca3f4e98ea98039246ad051a2c157aaecc1a99f6607766e493afe5d3` | — | no |

**Textures: none.** Both GLBs are untextured (0 images); all color comes from
flat embedded materials (`mob_eye`, `mob_body`, `pupil` / `pupil`, `body`,
`eye` — names verified by parsing the GLB JSON chunk). No PNG/JPG/WebP
textures exist anywhere in the frozen directory except the project icon and
screenshot (non-gameplay).

**Note N1 (license boundary):** the frozen `3d/squash_the_creeps/` directory
contains no LICENSE file of its own; the frozen README credits the music
(CC-BY 3.0) and font (SIL OFL 1.1). The godot-demo-projects repo root license
(MIT) covers the project as a whole. GLB models carry no explicit per-file
license in the tree. Flagged for the T-09 independent-consumer/license review;
no asset is blocked for prototype use on this evidence.

### 2.6 Squash replay/oracle freeze proposal (binding for T-03/T-05/T-06)

- Fixed timestep: physics `dt = 1/120 s` (frozen `physics_ticks_per_second`),
  one input sample per physics tick; rendering decoupled. Godot physics
  interpolation is on in the sample, but the oracle asserts physics-state
  equality at tick boundaries only.
- Determinism/seed: the sample uses Godot's global RNG (`randf`,
  `randf_range`) — not reproducible across engines. Oracle freeze: the AGP
  driver MUST use its own seeded PRNG; for the canonical trace the expected
  per-spawn tuple (spawn position on the rectangle path, ±45° rotation,
  speed in [10,18]) is recorded in the trace itself and the oracle compares
  driver state against it, not against Godot's RNG stream.
  *(Corrections, dated: T-06 first froze the seed as `0x8293D01` and this
  section said "30 ticks"; the actual frozen values since T-06 are seed
  `0x1B` and a spawn every **60** physics ticks (= 0.5 s at 120 Hz) —
  recorded here to match `testdata/squash/golden_squash_trace.json`. Review
  r1 (F07) caught the drift; this note reconciles the binding contract.)*
- Movement tolerances: position `|Δ| ≤ 1e-2` m per axis per tick boundary;
  vertical velocity `|Δ| ≤ 1e-1` m/s during jump arcs (integrator differences);
  facing angle `≤ 1e-2` rad. Score exact integer; squash counted exactly once
  per mob; mob freed exactly once (squash XOR screen-exit XOR despawn).
- Death semantics: ANY mob contact that is not a top landing (normal·UP ≤ 0.1)
  touching the MobDetector volumes ⇒ death; Retry overlay visible; MobTimer
  stopped; `ui_accept` ⇒ full reset (score 0, mobs cleared, player restored).
- Spawn cadence oracle: one mob every **60** physics ticks (= 0.5 s at
  120 Hz), first spawn after 0.5 s; spawn path rectangle corners frozen
  (§2.2). *(Correction, dated 2026-10-21 review r2 F829-R2-04: this bullet
  still said "30 physics ticks" three lines below its own 60-tick
  correction; the frozen trace and all tests use 60.)*
- Image comparison: keyframes (start, first spawn, mid-jump, squash moment,
  death overlay). Proposed freeze — mean abs channel diff ≤ 3/255, 99th
  percentile ≤ 12/255 (3D shading variance is larger than the 2D case),
  shadows must be present (a shadowless frame fails outright, not by
  tolerance). Cross-GPU bitwise equality is NOT required (Plan V-04).
- Audio oracle: looping playback of `House In a Forest Loop.ogg` from session
  start is the ONLY audio event; oracle asserts the stream is playing (real
  playback-path evidence per Plan §6, not a trigger count) and that no other
  audio path exists.

---

## 3. Downloaded asset summary (under `assets/`)

| File (in `assets/`) | Sample | bytes | sha256 (matches §1/§2.5) |
|---|---|---|---|
| `breakout/sounds/breakout_collision.ogg` | AG-2D-01 | 4930 | `a83896f7dfb64b6a8ad24beb44bfa6665b4275d208ec38de894776dac866d7cf` (bevy blob `0211d70cfb845fc9a88e32b0caf038f435a98d29`) |
| `squash_the_creeps/art/mob.glb` | AG-3D-01 | 63828 | `8a82adb5bfba8c22f9c1a0d692e4d25c2f9055cb7e6f6acd6f61ac169079818c` |
| `squash_the_creeps/art/player.glb` | AG-3D-01 | 52392 | `05d8588e8032a674e309ccb4d04578fb578b8da1629096b1036f628593801942` |
| `squash_the_creeps/art/House In a Forest Loop.ogg` | AG-3D-01 | 1430743 | `32680f2b2631eb9f6e2902ebbbec1b43ab82b4dbf154fdc9bfbfa17b03270504` |
| `squash_the_creeps/fonts/Montserrat-Medium.ttf` | AG-3D-01 | 260236 | `92b3d3c6e135eb1dc95f88e6ca75bd6113d9eb3261a95ca39f733e3897e53675` |
| `squash_the_creeps/fonts/LICENSE.txt` | AG-3D-01 | 4470 | `7a05af5e0552b46fbca086d777a251897082e77b840ab59199916a83cbab24f7` |

Total: **6 files, 1,817,029 bytes (~1.8 MB)** — far below the ~150 MB budget.
Breakout uses **no other assets** (no fonts, no images — verified from the
frozen source: the only `asset_server.load` call in breakout.rs is the
collision sound, line 179).

**License gap (recorded, not blocking):** `breakout_collision.ogg` has no
entry in the frozen Bevy `CREDITS.md` (verified by full read + `git log -S`),
while Bevy's README states example assets "typically fall under different open
licenses" with details in CREDITS.md. The file was introduced upstream by
bevyengine/bevy PR #4331 (2022-04-14) without a license note. Treated as a
research-only oracle asset pending T-09 license review. Bevy source code
itself is MIT OR Apache-2.0.

`sha256sum -c` result (2026-10-20): `assets/SHA256SUMS` and
`testdata/SHA256SUMS` — **all entries OK**.

## 4. Integrity verification record

| Check | Recorded (Plan/preflight) | Measured | Result |
|---|---|---|---|
| breakout.rs git blob | `b9bfb8b4b2cd19aa15b460f1323ad5c2af0e5954` | `b9bfb8b4b2cd19aa15b460f1323ad5c2af0e5954` | MATCH |
| squash README git blob | `8006f2783acf3ad8baceb63a80269b36b0c16138` | `8006f2783acf3ad8baceb63a80269b36b0c16138` | MATCH |
| squash Main.tscn git blob | `105c56d67284326f5efdb5caf426c0f063f9f255` | `105c56d67284326f5efdb5caf426c0f063f9f255` | MATCH |
| `sha256sum -c testdata/SHA256SUMS` | — | 22 files | all OK |
| `sha256sum -c assets/SHA256SUMS` | — | 6 files | all OK |

No mismatches. Download/verification method: shallow git fetch of both frozen
commits from `github.com` (raw.githubusercontent.com unreachable from this
machine), `git hash-object` inside the fetched trees, sha256 via `sha256sum`.
