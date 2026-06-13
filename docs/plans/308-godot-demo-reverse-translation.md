# Plan 308: Reverse-Translate Godot Demo Projects into a2gd Test Cases

## Context

a2gd currently has only hand-crafted test fixtures (synthetic `.at` → `.expected.gd`/`.tscn`).
We want **real-world validation**: take official Godot demo projects (cloned at
`D:\github\godot-demo-projects`) and "reverse-translate" each into Auto (`.at`) source,
then turn each into an a2gd regression test. This proves the Auto→Godot round-trip on
code that was *not* written to flatter the transpiler, and locks in current capabilities.

**Decisions confirmed with user:**
- **Round-trip contract** — Auto is the source of truth. `.expected.gd`/`.tscn` = a2gd's
  reviewed actual output, *not* a byte-identical copy of the original demo.
- **Defer GDScript sugar** — translate idiomatically around `$`, `&""`, `^""`, ternary
  (document as future gaps; do not add parser features now).
- **Scope: 4 curated demos** maximizing feature coverage.

Source-of-truth references: `crates/auto-lang/src/trans/gdscript.rs` (`test_a2gd` ≈ line 1792),
`crates/auto-lang/src/trans/tscn.rs` (`test_a2tscn` ≈ line 521).

---

## Capability map (verified)

**a2gd GDScript — supported:** `extends`, `signal`, `@export`, `@onready` (parser.rs:6513),
`func _ready/_process/_physics_process` (pass-through), untyped vars, `Vector2(...)`/`.ZERO`/`PI`,
method calls (`.normalized()`, `.clamp()`, `.emit()`, `move_and_slide()`, `queue_free()`,
`instantiate()`, `set_deferred()`, `Input.is_action_pressed(...)`, `Input.get_axis(...)`
all pass through), enums w/ values, typed `Array[T]`, comments, `await`, `preload(...)` as
a call expression.

**Gaps (translate around):**

| GDScript | Idiomatic Auto | Reason |
|---|---|---|
| `$Node` / `$"Path"` | `get_node("Node")` | no `$` sugar |
| `&"name"` StringName | `"name"` | no `&` literal |
| `^"path"` NodePath | `"path"` / `get_node` | no `^` literal |
| `x is Type` (type-check) | `is x { as Type -> ... }` pattern-match form | Auto `is` = match keyword |
| `a if c else b` (ternary) | `if/else` statement | no expression-if |
| `var x := expr` | `var x = expr` | Auto infers |

**tscn generator — supported:** header (auto `load_steps`, deterministic FNV-1a uid),
`ext_resource` inference by extension (`.gd/.png/.wav/.ttf/.tscn`), inline sub_resources
(`Type { props }`), basic nodes, nested parent-path derivation, `instance`, `connect`.
**Gaps (omit from translations, note):** complex sub_resource values (arrays-of-objects
like `SpriteFrames.animations`), `PackedVector2Array`/`PackedByteArray`/`Curve2D._data`,
node metadata (`unique_id`, `groups`, `collision_mask`, `gravity_scale`, `process_callback`).

**Structural note:** a2gd derives `extends <Type>` from the scene root type
(gdscript.rs:1660). So every script fixture must include a `scene <Name> : <Type>` block
(even minimal). All target demos pair a scene with a script naturally, **except**
`ball_factory.gd` (attached to the huge `scene_instancing.tscn`) → wrap in minimal
`scene BallFactory : Node2D`.

**Fixture placement (harness constraint):** the tscn harness `test_a2tscn` hardcodes the
`tscn/` prefix, so ALL fixtures live under `test/a2gd/tscn/godot_demos/<demo>/NN_name/`.
The `.gd` harness `test_a2gd` takes an arbitrary case path, so it references the same file
as `test_a2gd("tscn/godot_demos/<demo>/NN_name")`. Dir names MUST keep a numeric prefix
(`001_ball`) — both harnesses derive the `.at` basename from `split('_')[1..]`.

---

## The 4 demos & fixtures

### 1. `instancing/` — scene sub_resources + scene instancing script
- **`ball/ball.at`** (scene-only → `.expected.tscn`): RigidBody2D root; children Sprite2D
  (texture ext_resource), CollisionShape2D (CircleShape2D inline sub_resource); root
  `physics_material_override = PhysicsMaterial { bounce = 0.4 }` inline. Tests inline
  sub_resources + ext_resource collection + nested nodes. No script.
- **`ball_factory/ball_factory.at`** (script → `.expected.gd` only): minimal
  `scene BallFactory : Node2D` wrapper; `#[export] var ball_scene PackedScene = preload("res://ball.tscn")`;
  `func _unhandled_input(input_event InputEvent)`; `input_event is InputEventMouseButton`
  → `is input_event { as InputEventMouseButton -> ... }`; `spawn()` calls `instantiate()`,
  `add_child()`, `get_global_mouse_position()`.

### 2. `hexagonal_map/` — CharacterBody2D math + Input
- **`troll/troll.at`** (combined → `.gd` + `.tscn`): scene `: CharacterBody2D`; `const`
  (MOTION_SPEED etc.); `func _physics_process(delta f64)`; `Input.get_axis("move_left", "move_right")`
  (plain strings for `&""`); Vector2 math, `velocity`, `move_and_slide()`. Scene: Sprite2D,
  Shadow Sprite2D (`modulate`, `show_behind_parent`, `scale`, `skew`), CollisionShape2D
  (CircleShape), Camera2D (`process_callback = 0`).

### 3. `kinematic_character/` — physics + @onready
- **`player/player.at`** (combined → `.gd` + `.tscn`): scene `: CharacterBody2D`; `const`;
  `#[onready] var gravity = float(ProjectSettings.get_setting("physics/2d/default_gravity"))`;
  `func _physics_process(delta f64)`; `Input.get_axis`, `move_toward`, `clamp`, `is_on_floor`,
  `Input.is_action_just_pressed("jump")`. Scene: Sprite2D + CollisionShape2D (RectangleShape).

### 4. `dodge_the_creeps/` — the showcase (signals + emit + Vector2)
- **`mob/mob.at`** (combined): scene `: RigidBody2D`; `func _ready()` picks random
  animation (`Array(get_node("AnimatedSprite2D").sprite_frames.get_animation_names()).pick_random()`);
  `screen_exited` signal connect → `queue_free()`. Scene: AnimatedSprite2D (bare — SpriteFrames
  animations array is a deferred gap), CollisionShape2D (CapsuleShape), VisibleOnScreenNotifier2D
  + `connect screen_exited`.
- **`player/player.at`** (combined): scene `: Area2D`; `signal hit`; `#[export] var speed = 400`;
  `func _ready()`, `func _process(delta)`; `Input.is_action_pressed("move_right")` ×4;
  `Vector2.ZERO`, `.normalized()`, `.clamp()`, `.length()`; `get_node("AnimatedSprite2D")` for `$`;
  `hit.emit()`; ternary `rotation = PI if velocity.y > 0 else 0` → if/else; `set_deferred("disabled", true)`.
  Scene: AnimatedSprite2D (bare), CollisionShape2D, `connect body_entered from "." to "." method "_on_body_entered"`.

*(hud/main from dodge deferred: hud has multi-control UI + Timer; main has Path2D with
Curve2D `_data` PackedVector2Array — both blocked by complex-sub_resource/Array gaps.
Listed as documented follow-up.)*

---

## Idiomatic translation worked-example (dodge player `_process`, partial)

Original GDScript:
```gdscript
if velocity.length() > 0:
	velocity = velocity.normalized() * speed
	$AnimatedSprite2D.play()
rotation = PI if velocity.y > 0 else 0
hit.emit()
```
Auto translation:
```auto
if velocity.length() > 0 {
    velocity = velocity.normalized() * speed
    get_node("AnimatedSprite2D").play()
}
if velocity.y > 0 {
    rotation = PI
} else {
    rotation = 0
}
hit.emit()
```
`.expected.gd` will therefore contain `get_node("AnimatedSprite2D").play()` and an
`if/else` block — diverging from the original's `$` and ternary, which is the
documented round-trip tradeoff.

---

## Per-fixture TDD workflow (repeat for each)

1. Write the `<name>.at` translation (idiomatic per the table above).
2. Run the new test → harness writes `.wrong.gd`/`.wrong.tscn`.
3. **Review** `.wrong.*` against the original demo's *intent* (not byte-identity);
   confirm a2gd output is correct GDScript/Godot.
4. Rename `.wrong.*` → `.expected.*`.
5. Add the test function(s):
   - `.gd` asserts → `test_godot_demo_<name>` in `gdscript.rs` tests block, calling
     `test_a2gd("tscn/godot_demos/<demo>/NN_name")`.
   - `.tscn` asserts → `test_godot_demo_<name>_scene` in `tscn.rs` tests block, calling
     `test_a2tscn("godot_demos/<demo>/NN_name")`.

## Files to add/modify
- **New fixture dirs** under `crates/auto-lang/test/a2gd/tscn/godot_demos/`:
  `instancing/{001_ball,002_ball_factory}/`, `hexagonal_map/001_troll/`,
  `kinematic_character/001_player/`, `dodge_the_creeps/{001_mob,002_player}/` —
  each with `.at` + `.expected.*`.
- **`crates/auto-lang/src/trans/gdscript.rs`** — add ~6 `test_godot_demo_*` fns in the tests module.
- **`crates/auto-lang/src/trans/tscn.rs`** — add ~5 `test_godot_demo_*_scene` fns.
- **This file** (`docs/plans/308-godot-demo-reverse-translation.md`) — committed plan +
  "Documented gaps" appendix below recording round-trip divergences.

## Execution order
instancing → hexagonal_map → kinematic_character → dodge_the_creeps (ascending feature
complexity; each demo committed separately). Run `cargo build -p auto` before the first
test run (CLAUDE.md rule), `cargo test -p auto-lang --lib -- trans` after each demo.

---

## Verification

- `cargo test -p auto-lang --lib -- trans` → all green (new demo tests + existing 308).
- Each `.expected.gd` is valid GDScript that could run in Godot (extends correct type,
  lifecycle funcs present, method calls well-formed) — spot-check 2–3 by eye against the
  original demo.
- Each `.expected.tscn` has a valid `[gd_scene]` header, ext/sub resources, and a node
  tree matching the scene block — spot-check troll + dodge player.
- No existing test regresses (the 6 cookbook `Array[T]` expectations from Plan 306 Phase 5
  remain the baseline).

## Out of scope (explicit)
- Adding `$` / `&""` / `^""` / ternary to the Auto parser (deferred — user decision).
- Complex inline sub_resources (SpriteFrames.animations arrays, PackedVector2Array,
  Curve2D._data), node metadata (groups/unique_id/collision_mask), dodge hud & main scenes.
- Translating the giant `scene_instancing.tscn` / `kinematic_character/world.tscn`
  (TileMapLayer PackedByteArray, Animation libraries) — out of reach.

## Documented gaps (future work)
These divergences between the real Godot demos and our Auto translations are recorded for
a future plan that closes them:
1. **GDScript sugar**: `$` node-path, `&"StringName"`, `^"NodePath"` literals, expression
   ternary — would require parser + a2gd changes; translations currently use `get_node()`,
   plain strings, and `if/else`.
2. **Complex sub_resource values**: tscn `render_value` falls to debug `{:?}` for
   `Expr::Array`/`Expr::Object` (tscn.rs:331). Blocks SpriteFrames `.animations`,
   Animation libraries, Curve2D `_data`.
3. **Packed arrays**: `PackedVector2Array`, `PackedByteArray`, `PackedColorArray`,
   `PackedFloat32Array` — no rendering path.
4. **Node metadata**: `unique_id`, `groups=[...]`, `collision_mask`, `gravity_scale`,
   `process_callback`, `unique_name_in_owner` — not in the SceneNode AST.
5. **`is` type-check ergonomics**: GDScript `x is Type` as a boolean expression maps to
   Auto's statement-form `is x { as Type -> ... }`, which is workable but verbose.
