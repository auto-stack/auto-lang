# PNG image resources and 2D sprite frames (PLAN-004)

A 2D leaf may carry one accepted static PNG visual. The document owns the image
identity, the default frame and the flip; the real Auto profile chooses the
frame rectangle and the horizontal flip on every fixed tick, and the character,
camera, runtime objects, triggers and visuals are prepared, accepted and
published as one transaction. Collision geometry is the object's scene box: a
frame crop or a flip never moves, resizes or re-shapes the entity.

## Finite image domain

Admission is structural and precedes every decoder allocation and all IO; only
an already-accepted read reaches the decoder, and the decoded length is checked
again against the prediction.

| Property | Accepted |
|---|---|
| Container | one static PNG, 8-bit |
| Colour type | 2 (truecolour RGB) or 6 (truecolour + alpha) |
| Filter/compression | method 0/0, no interlacing |
| Sides | 1..=2048 pixels each |
| Decoded RGBA | `width*height*4 <= 16MiB` |
| Original bytes | `<= 8MiB` per resource |
| Sampling | nearest, clamp to edge, no mipmaps; sRGB RGBA8 texture into the sRGB surface, blended in linear space |

Rejected by name before decoding: APNG (`acTL`), 16-bit, palette/grayscale
colour types, truncated or trailing bytes, wrong signature, a chunk chain that
exceeds the file, zero or over-2048 sides, and any header/decoded dimension
disagreement. A rejected input leaves the decoder attempt counter unchanged.

## Frame, flip and bounds

A frame is an integer pixel rectangle `x,y,w,h` with `w,h>=1` inside the image.
`x+w>width` or `y+h>height` is a named rejection at every boundary — document
binding, admitted block and the UV conversion — and is never a wrap or an
overflow panic. The flip mirrors the U axis only: the world rectangle, the
entity box, the collision rules and every solid/trigger result are unchanged by
a frame or flip change.

UVs are `x/width .. (x+w)/width` on U and `y/height .. (y+h)/height` on V in
the screen-style 2-D camera; a flipped frame swaps the two U endpoints. The
display size is the scene's explicit object scale.

## Public profile

`autographics.sprites2d/1` (number 109, profile 20) composes the unchanged
triggers2d surface (character/1 + runtime_camera2d/1 + objects2d/1 +
triggers2d/1) with one sprite unit. Every earlier profile keeps its exact
assembled bytes. The public entry selects it by name:

```powershell
autographics-authoring --open C:/project/scene.agscene --behavior main.at --behavior-api sprites2d
```

The `--behavior-api` value list is derived from the profile names, so a profile
cannot be added without also becoming selectable.

| Function | Result |
|---|---|
| `ag_sprite_frame(id,x,y,w,h,flip)` | 1/0; writes one bound leaf's frame and flip in one call |
| `ag_sprite_slot(token,x,y,w,h,flip)` | 1/0; writes one live runtime instance's frame and flip |
| `ag_sprite_bound(id)` | 0/1 legal absence; an unbound id answers 0 without a sticky error |

`id` is the stable document id; `token` is the reserved-slot handle. Each write
costs one of the callback's 32 shared credits and at most 32 sprite writes are
accepted per callback. A write is all-or-nothing: the frame and the flip of one
visual never half-apply.

## Host admission

The sprite block rides every tick after the rounds block as a fixed table for
that Run: four header ints (`status`, bound count, run, tick), one row per
bound leaf and eight slot rows, seven values each (`node/template ordinal`,
`asset ordinal`, `x, y, w, h`, `flip`). The host independently re-validates the
node ordinal and asset ordinal against the frozen target table, the registry
identity and dimensions, the frame bounds, the run/tick, the slot generation of
the accepted instance table, and that every live instance of a bound template
carries its row. Any failure rejects the whole candidate and keeps the last
complete group: a new position never publishes beside an old frame.

Named sticky codes: 41 (target not bound / not a 2-D leaf / incomplete
configuration), 42 (frame domain or flip argument), 43 (per-callback sprite
write budget), 44 (group or non-2-D target).

## Persistence and closure

Scene schema 5 adds `objects[].sprite = null | {asset, frame:[x,y,w,h], flip}`.
Schemas 1–4 keep their exact meaning: a legacy file that carries the field or a
non-model resource is rejected instead of silently upgraded. Bindings are
limited to 2-D leaves and to eight per scene; a ninth is rejected.

An image is a project resource in the same `assets/` directory as models,
stored as `<lowercase64sha>.png`; its identity is the complete original SHA256,
byte length, `png` format and its declared dimensions. Models stay `.glb` and a
model reference must name a model payload; an image is never treated as an
empty mesh model. Image and model payloads share one budget: eight original
content identities, 32MiB raw bytes, 64MiB decoded CPU payload and geometry,
and 64MiB GPU textures and mesh arrays, counted once per content identity
across accepted, history and candidate states. The 8-file/32MiB disk budget
includes residuals and temporary reservations; foreign or incomplete files are
reported and left untouched; a corrupted or missing blob rejects the reopen and
restoring the exact bytes recovers the project.

## Lifetime

Pause does not advance the visual; Step advances exactly one neutral tick
through the real phase; Stop revokes the runtime owner and restores the editing
preview with the document's default frame; a new Run starts a fresh Auto
session on its initialization frame. A failed preparation keeps the whole
last-good group. Textures are cached by content identity: a warm frame or flip
change rewrites instance data only, never pixels, and the last owner's release
frees the texture.

## Measured price

The composed managed source is 80367 bytes (api 76518) against the unchanged
96KiB envelope; the sprite block adds 74 host round-trips for two bound leaves
(116 at the eight-leaf ceiling), about 19% of a Run's reads. Debug-build whole
ticks measure 5.5–9.5ms median and 7.8–8.6ms p95 on an idle workstation, with
the 2s per-tick bound unchanged. GPU timings are unavailable and excluded.

## Boundaries

Desktop Windows x64 only; images are imported from a local disk path, never
downlinked. The supported domain is deliberately finite — a required asset that
does not fit returns for a new plan instead of raising the recorded budgets.
G09 gallery content, sound/HUD, 3-D textures/UV and node animation are separate
plans. Windowless GPU capture and the public CLI entry are the accepted evidence
paths; OS-focus automation pauses rather than bypassing its gate.

Provenance: auto-ui PLAN-004 (r1), archived at
`auto-ui/docs/plans/archived/004-autographics-public-images-and-sprite-frames.md`;
delivery `e0fce13810f9e1af28182950d62cfce8722aa30a` on auto-ui master (rebased
from reviewed `0e6e361` with an identical `experimental/**` tree and a fully
equal range-diff), independent final review pass bound to `0e6e361`. Both
Cargo.lock files, the SDK pin and the frozen snapshot are unchanged; ag-examples
is untouched and continues to consume the delivered 002 closure.
