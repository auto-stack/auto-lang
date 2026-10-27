# AGP Integration Handoff (T-08, AC-08)

Status date: 2026-10-21. Scope: what PLAN-829 verified vs what remains
PENDING for production integration. Nothing below reads as done unless a
probe report is cited. I1–I3 are explicitly pending; the independent Iced
probe does not advance them.

## 1. Seam table (IH-01..IH-05, Plan §5.4)

### IH-01 GPU / Iced same-device seam
- **Owner/consumer**: auto-ui renderer/host owner (production) ↔
  AutoGraphics renderer owner (post-migration).
- **Entry gate**: migration-completion receipt from the migration owner —
  current M814-A (compiler/SDK subset) does NOT suffice; the runtime/
  renderer handoff receipt is required.
- **Delivered by THIS plan** (probe `iced-shared-device`, windowed, exit 0):
  - Real iced 0.14.0 application; genuine widgets (button + label) and our
    2D/3D graphics views in ONE window, one wgpu **27.0.1** instance
    (Cargo.lock: iced_wgpu 0.14.0 → wgpu 27.0.1; AGP lock identical version).
  - **Same device/queue**: view content is rendered with the `&Device /
    &Queue` that iced_wgpu hands to `Primitive::prepare` (mpv-precedent
    pattern, independent implementation). Proof is functional: content is
    bound and drawn inside iced's pass on the prepare-handed device.
    HONEST LIMITATION (r1 repair): wgpu 27 uses process-global resource ids
    and does NOT reject cross-device binding at the API level (observed via
    an error scope and recorded); it exposes no device-global id either.
    The runtime invariant is therefore architectural — the render path
    never constructs a second device — enforced by the permanent audit
    test `f05_render_path_has_no_second_device`, plus a token gate that
    fires on any target recorded with a foreign device identity.
  - **Final composite (r1 F05)**: GDI window capture (PrintWindow /
    CopyFromScreen) hangs on the presenting wgpu window from a non-owner
    process in this environment (attempted, recorded), so composite
    evidence is in-band: blit-draw counters incremented only adjacent to
    the real composite draw, sampled at each key moment; the review's
    blank-blit mutation turns the gate RED (recorded).
  - **Borrow/lifecycle**: the spike borrows the device per-frame in
    `prepare`; persistent view textures are created once per size and
    re-created only on resize (lifecycle case: no per-frame recreation).
  - **Texture format/alpha**: view textures are `Rgba8UnormSrgb`;
    composited via a blit pass with alpha forced to 1 (opaque graphics
    under the controls row); iced's own layer blends UI alpha above it.
  - **Depth**: 3D view renders its own depth buffer offscreen (Depth32Float)
    before compositing — no depth coupling with iced's UI pass.
  - **Submission ordering**: content encode/submit happens inside
    `prepare` (before iced's own pass encodes), so ordering is
    content-then-UI on the same queue every frame.
  - **Resize**: `window::resize` task → resize event → views re-created at
    new bounds (keyframes `initial/resized` distinct hashes); DPI:
    `Viewport::scale_factor()` recorded (this machine: 2.0).
  - **Exit**: clean exit 0 (report written before close).
  - **Iced holds no game business state**: sim clock + press count live in
    the app/shared cell; widgets only render and emit messages. Confirmed
    by construction and noted in the report.
  - **Negative controls**: blank/undriven frame caught (frozen content
    hashes equal); second device caught (bind test). Gates in
    `src/host/gates.rs`, shared with the V-08 verifier.
- **Still missing (production)**: wiring into the real AutoUI element tree;
  production window management (multi-window, dock); production theming;
  input routing policies (focus/IME); the migration-owner receipt.

### IH-02 Auto SDK seam
- **Owner/consumer**: language SDK owner + UI/graphics adapter owner.
- **Entry gate**: actually-published SDK/profile with the public symbols;
  no production API is presumed (per Plan §10.3).
- **Delivered by THIS plan**: the G-base runtime contract that an SDK would
  drive is implemented and verified (T-03: fixed-tick clock, action/event
  queues, replayable change log, session/generation identity; T-06 canonical
  replay drives it end-to-end).
- **Still missing**: the real `.at` game entry driving this core through a
  published SDK (I1, pending); action/event schema publication; adapter
  ownership rules (core must not depend on VM/AST).

### IH-03 RQHost seam
- **Owner/consumer**: migrated host/protocol owner.
- **Entry gate**: migration completion + protocol version/instance receipts.
- **Delivered by THIS plan**: nothing for RQHost specifically (out of scope
  by design — the Plan's §4.2 evidence stands: current rqhost is CPU
  record/replay with default software drawing).
- **Still missing**: GPU-backed RQHost protocol, backpressure/disconnect,
  resource retirement rules (Plan §5.4 IH-03 rows).

### IH-04 Nesting / Forge seam
- **Owner/consumer**: AutoUI input/layout owner + Forge owner.
- **Entry gate**: migration of the production UI stack.
- **Delivered by THIS plan**: single-window widget-over-graphics compositing
  (UI over graphics, clipping via iced layers). NOT delivered: arbitrary
  nesting (UI→3D→UI, 3D→UI→2D), coordinate/ray/UV pick-through, focus/IME
  interplay, save/reopen consistency.
- **Still missing**: everything in the nesting matrix; Forge editing loop.

### IH-05 Distribution / repository boundary
- **Owner/consumer**: auto-ui/auto-man release owner.
- **Entry gate**: release pipeline receipts.
- **Delivered by THIS plan**: AGP remains a standalone workspace (root
  lock/build untouched — `git status` on root files is clean); dependency
  closure is registry-only — ZERO path-sourced packages, no `[patch]`
  entries (r1 F09: the vendored iced_widget patch was removed after stock
  iced_widget 0.14.2 verified green unpatched 2026-10-21; production
  retains the PLAN-043 patch, AGP does not need it); no junctions/symlinks;
  assets are sha256-pinned; probe reports carry adapter/lock facts.
- **Still missing**: production relocation/A→B consumer replay (V-09 covers
  the AGP-side independent consumer), signing, update channel.

## 2. I1–I3 (explicitly PENDING)

### I1 — Real Auto-source game entry (PENDING)
- Owner: language SDK owner + AutoGraphics adapter owner.
- Entry conditions: IH-02 published SDK/profile; two `.at` samples authored;
  migration receipts where they touch SDK symbols.
- Acceptance: both frozen games driven by `.at` code through the public SDK
  controlling the same core, handling real actions/events; AGP Rust drivers
  are NOT credited toward I1.
- Current status: **not started**; no `.at` integration exists in this plan.

### I2 — Production AutoUI / nesting / RQHost (PENDING)
- Owner: auto-ui renderer/host owner + protocol owner.
- Entry conditions: migration completion receipts (M814-B and successors);
  IH-01 receipt promoted from probe to production host.
- Acceptance: real host scenarios UI→3D→UI and 3D→UI→2D; RQHost GPU path;
  save/reopen consistency; the nested-input matrix.
- Current status: **not started**; the iced spike is explicitly not the
  production AutoUI.

### I3 — Forge and distribution (PENDING)
- Owner: Forge owner + release owner.
- Entry conditions: I2 plus Forge editing-loop design receipts.
- Acceptance: edit/save/undo in a real running scene; relocation-resilient
  release closure.
- Current status: **not started**.

## 3. IH-01 measured facts (V-06 report, `scratch/829/iced-shared-device/`)

| Fact | Value |
|---|---|
| Adapter (diagnostic device, same selection) | NVIDIA GeForce RTX 4060 Ti, Vulkan, driver 617.14 |
| iced / iced_wgpu / wgpu instances | 0.14.0 / 0.14.0 / **27.0.1** (same as production lock) |
| iced_widget | 0.14.2 STOCK from crates.io (r1 F09: the production PLAN-043 patch is not necessary for this minimal static spike; verified green unpatched 2026-10-21; production retains the patch) |
| DPI scale | recorded from `Viewport::scale_factor()` (2.0 on this machine) |
| Window | created 1000×800 logical, resized to 860×700 via `window::resize` |
| Real click | PowerShell user32 click at the button → `ButtonPressed` through the real iced event path → graphics state changed (after-button keyframes differ) |
| Keyframes | initial / after_button / resized / blank_control / pre_close, 2D+3D, distinct where driven |
| Negative controls | blank/undriven caught; second device caught (bind test) |
| GPU timing | unavailable (explicit) |
