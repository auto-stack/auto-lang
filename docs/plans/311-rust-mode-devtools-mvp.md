# F12 DevTools for rust mode (render=rust)

> **For Claude:** Primarily `crates/auto-lang/src/ui/iced/renderer.rs` + a small change to `crates/auto-man/src/rust_ui.rs` (generated `main.rs`). Build with `cargo build -p auto`; regression `cargo test -p auto-lang --lib 'ui::debug'`. End-to-end manual: `auto r -r rust` in any `examples/ui/*` (force rust render regardless of `pac.at`).

## Status

- **Phase 1 (MVP) — DONE & merged.** Tree-selection DevTools works for the standard rust launch path (`run_app_devtools`). Verified: `auto r -r rust` on `012-stopwatch` regenerates `main.rs` with `run_app_devtools::<App>()`, cargo links, app launches, F12 opens the panel, tree + 检视 sections populate.
- **Phase 2 — TODO (this section).** Two gaps users hit immediately: (a) async-init apps (`015-notes`) get no F12; (b) no canvas "检视" highlight — selecting a VNode does nothing on the app.

## Context

VM-mode F12 DevTools is working; rust mode (`render=rust`) has **none of it**. Pressing F12 does nothing because rust mode runs through `run_app::<C>()` ([renderer.rs:6027](../../crates/auto-lang/src/ui/iced/renderer.rs#L6027)), whose `.subscription(|c| c.subscription())` defaults to `Subscription::none()` ([component.rs:51](../../crates/auto-lang/src/ui/component.rs#L51)) — no keyboard listener, so F12 never reaches a handler. There is also no DevTools state, no VTree capture, no inspector panel. This is a missing feature, not a bug.

User decision (confirmed): **MVP first**, reusing as much of the VM DevTools machinery as cleanly possible, then add rust-specific niceties later.

## Reuse map (the answer to "can we reuse VM components?")

| Layer | Reusable? | How |
|---|---|---|
| `view_to_vtree_with_paths` ([vnode_converter.rs:92](../../crates/auto-lang/src/ui/vnode_converter.rs#L92)) | ✅ fully | call with `span_for = \|_\| None` (rust has no source spans) |
| `LayoutCollector` + `backfill_bounds` + `InspectorCache`/`BoxModel`/`ComputedNode` | ✅ fully | id-based, independent of DynamicState |
| box-model diagram, `kv_row`, `render_collapsible_section`, `with_selected_vnode`, `placeholder_panel`, `tab_style_fn` | ✅ fully | pure helpers |
| `DEBUG_*` string conventions + prefix-parsing dispatch | ✅ via extraction | extract VM `update()`'s prefix dispatch into a pure `apply_debug_event(&mut DevToolsState, &str)` shared by both modes |
| `into_iced` (AbstractView→iced) | ✅ via thread-local | inject a debug-ctx thread-local so into_iced records per-node id↔VNodeId + computed style + box insets (see Task 4) |
| `wrap_debug` + `render_dynamic_view` (VM converters) | ❌ not directly | AuraNodeId/debug_id_map/span_map coupled — rust has none. MVP uses tree-selection, not canvas overlays, so the heavy overlay path is unneeded. |
| Panel renderers (`render_inspector_layout_tab` etc.) | ❌ for MVP | take `&DynamicState`; MVP writes thin rust-side shells over the shared primitives. Unifying via a shared trait is a documented follow-up. |

**Key enabler:** `View::map_msg` ([view.rs:1106](../../crates/auto-lang/src/ui/view.rs#L1106)) recursively remaps every handler in the tree, so the wrapper can lift `inner.view(): View<C::Msg>` to `View<WrapperMsg>` cleanly.

## Architecture

A **`DevToolsWrapper<C>`** Component that wraps the user's `C`, owns a `DevToolsState`, and exposes F12 + a tree+inspector panel. Injected at the rust-mode entry point.

```
DevToolsWrapper<C: Component> { inner: C, dt: DevToolsState }
enum WrapperMsg<C: Component> { Inner(C::Msg), Debug(String) }   // Debug mirrors IcedMessage.event strings → shared dispatch
```

- `view()`: `inner.view().map_msg(WrapperMsg::Inner)` → build VTree → set thread-local debug ctx → `into_iced()` (records id↔VNodeId + computed style + box insets into cache) → render app element + DevTools panel.
- `on()`: `Inner(m)` → `inner.on(m)`; `Debug(s)` → shared `apply_debug_event`.
- `subscription()`: F12 keyboard sub + window-level `listen_with` (modifiers, mouse_moved for divider, resized) → `Debug(...)`; plus a `LayoutCollector` operation task when `needs_bounds`.

**MVP scope — tree-selection only** (no canvas hover/click overlays): select nodes by clicking the element tree. This sidesteps the entire AuraNode-coupled `wrap_debug` overlay path. Canvas hover/select + Source/AutoUI tabs are explicit follow-ups (need overlays / VM build-time data rust mode lacks).

## Phase 1 tasks (DONE — MVP)

> Tasks 1–6 below are the implemented MVP. Kept as the historical record; no further work here. See **Phase 2** below for the open follow-ups.

### Task 1 — `DevToolsState` (the reusable field subset)

### Task 1 — `DevToolsState` (the reusable field subset)
New struct in renderer.rs holding only the DevTools-relevant fields (NOT extracted from `DynamicState` — duplicated definitions to avoid churning the working VM path): `debug_mode`, `inspect_mode`, `selected_vnode`, `hovered_vnode`, `current_modifiers`, `inspector_subtab`, `inspector_sections`, `devtools_open`, `live_vtree`, `live_cache`, `window_size`, `devtools_panel_width`, `inspector_split_ratio`, `dragging_inner_divider`, `inspector_scroll_id`, `elements_scroll_id`, `needs_bounds`, + an id counter and an `InspectorCache` scratch. All `RefCell` where mutable, `Default` = debug off, panel closed.

### Task 2 — Extract `apply_debug_event` (shared dispatch)
Pull the `DEBUG_*` prefix-parsing arms out of VM `update()` ([renderer.rs:~2320-2470](../../crates/auto-lang/src/ui/iced/renderer.rs)) into a pure `fn apply_debug_event(dt: &mut DevToolsState, event: &str) -> bool` returning `ui_changed`. Handles: `__toggle_debug`, `__inspector_subtab_*`, `__inspector_section_*`, `__vnode_select_*`, `__mouse_moved`/`__mouse_released` (divider drag + bounds trigger), `__window_resized`, `__modifiers_changed`. **Re-wire VM `update()` to call it** (DynamicState → a `DevToolsState` view via a small adapter, OR keep VM's inline copy and only share for rust). Lowest-risk: keep VM inline, have rust use the extracted fn (share the fn body by copying the logic into the new pure fn and having BOTH call it once VM is verified — decide during impl; the pure fn is the unit of reuse).

### Task 3 — `DevToolsWrapper<C>` + `WrapperMsg<C>`
Implement `Component` for `DevToolsWrapper<C>`. `view()` builds the panel via new `render_rust_devtools_panel(&dt, app_element)` reusing `render_collapsible_section` + box/computed/props shells. `on()` routes messages. `subscription()` wires F12 + window events (mirror VM's `keyboard_subscription` + `listen_with` at [renderer.rs:1816](../../crates/auto-lang/src/ui/iced/renderer.rs#L1816) and [:3069](../../crates/auto-lang/src/ui/iced/renderer.rs#L3069), emitting `Debug(...)`).

### Task 4 — Thread-local debug ctx in `into_iced` (the conversion-reuse trick)
Add `thread_local! { static DEBUG_RECORD_CTX: Cell<Option<Rc<DebugRecordCtx>>> }` near `INSPECT_CAPTURE`. When `Some`, `into_iced` (at each widget/container arm's exit) wraps the built element in an id'd `container` (the bounds probe pattern from [renderer.rs:5382](../../crates/auto-lang/src/ui/iced/renderer.rs#L5382)) and records into the ctx's `InspectorCache`: id↔VNodeId (via a path stack the converter pushes/pops on container descent), `computed_style` (from the variant's props), and declared box insets (reuse `debug_style_insets`). When `None` (all existing callers, tests, VM path) → no-op, zero behavior change. **This is the main implementation sub-task**; if the per-arm injection proves too invasive, fallback is a thin separate recursive converter `convert_with_debug_ids` mirroring `into_iced`'s match — same outcome, more code, isolated.

### Task 5 — Bounds flow
In the wrapper's `update`/`view` cycle: when `needs_bounds`, run `LayoutCollector::new()` operation (as VM does at [renderer.rs:3009](../../crates/auto-lang/src/ui/iced/renderer.rs#L3009)), then `backfill_bounds(&mut cache, &bounds)` (reused). Re-trigger after each view rebuild while panel open.

### Task 6 — Entry point + codegen
Add `pub fn run_app_devtools::<C>()` (mirrors `run_app` but instantiates `DevToolsWrapper::<C>::default()` and runs the wrapper). Update the generated `main.rs` template in `rust_ui.rs` ([rust_ui.rs:647-653](../../crates/auto-man/src/rust_ui.rs)) to emit `auto_lang::ui::iced::run_app_devtools::<{main_widget}>()` instead of `run_app::<…>()`. Keep plain `run_app` for tests/non-UI.

## Phase 2 tasks (TODO — async-init F12 + canvas 检视)

Two user-reported gaps. **P2-A** makes F12 reach async-init apps; **P2-B** adds the canvas highlight + real geometry. P2-B's pieces (id map → restyle highlight → measured bounds) are ordered so each ships testable value on its own.

### P2-A — `run_app_with_task_devtools` (async-init apps get F12)

**Why:** `015-notes` (and any app with an `__InitLoaded` init API) codegens to `run_app_with_task` ([rust_ui.rs:626-645](../../crates/auto-man/src/rust_ui.rs#L626)), the branch Phase 1 deliberately left unwired → no F12 subscription, no panel.

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs` (new entry + wrapper constructor)
- Modify: `crates/auto-man/src/rust_ui.rs` (async-init branch)
- Modify: `crates/auto-lang/src/ui/iced/mod.rs` (re-export)

**Steps:**
1. Add a non-`Default` constructor on `DevToolsWrapper<C>`: `fn from_inner(inner: C) -> Self { Self { inner, dt: DevToolsState::default() } }`. (Async-init builds `C` via boot, not `Default`, so the existing `Default`-derived path can't be reused.)
2. Add `pub fn run_app_with_task_devtools<C>(boot)` mirroring `run_app_with_task` ([renderer.rs:6059](../../crates/auto-lang/src/ui/iced/renderer.rs#L6059)) but: the iced state-init closure calls `boot()` to get `(C, Task<C::Msg>)`, wraps as `DevToolsWrapper::from_inner(c)`, and lifts the boot `Task<C::Msg>` to `Task<WrapperMsg<C>>` via `.map(WrapperMsg::Inner)`. Reuse `devtools_update` / `devtools_view` / `devtools_subscription`.
3. Re-export `run_app_with_task_devtools` from `mod.rs`.
4. In `rust_ui.rs`, change the async-init branch ([rust_ui.rs:638](../../crates/auto-man/src/rust_ui.rs#L638)) from `run_app_with_task(move || {...})` to `run_app_with_task_devtools(move || {...})`.

**Verify:** `auto r -r rust` in `015-notes` → F12 opens the panel (previously: nothing).

### P2-B — Canvas 检视 highlight + real geometry (the deferred Task 4/5)

**Why:** Selecting a VNode does nothing on the app; box model shows "(尺寸待测量)". Users expect the VM experience: selected widget outlined in orange/yellow on the canvas, real `W × H` in the box model.

**Approach decision (key):** Do NOT chase absolute-position overlays (iced has no clean overlay API). Instead mirror how VM achieves highlight — **restyle the selected widget during conversion**. Since the app re-renders every frame, `into_iced` checks a thread-local "selected VNodeId" and, when converting the matching node, applies an orange border to that widget's style. No geometry needed for the highlight itself. Measured bounds are a separate, additive step that only improves the box-model numbers.

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs` (thread-local ctx + into_iced hooks + bounds op + panel tweaks)

#### P2-B-1 — Thread-local conversion ctx + VNodeId↔path in `into_iced`
Add `thread_local! { static RUST_DEBUG_CTX: Cell<Option<Rc<RustDebugCtx>>> }` (near `INSPECT_CAPTURE`). `RustDebugCtx { selected: RefCell<Option<VNodeId>>, id_map: RefCell<HashMap<VNodeId, iced::widget::Id>>, path_stack: RefCell<Vec<u16>>, cache: ... }`.
- In `into_iced`, on entering each widget/container arm: push the child index onto `path_stack`, compute `VNodeId = id_from_path(&path_stack)`, and — **if `selected == Some(vid)`** — apply an orange border (`.style(|t| container::Style { border: iced::Border { color: orange, width: 2.0, .. }, .. })`) by wrapping the built element. Pop on exit. Children descent must push/pop in the SAME order `view_to_vtree_with_paths` walks them (Column/Row→children, Container/Scrollable→[child], List→items, Table→cells, Tabs→contents) — reuse the order from the existing `view_children` helper.
- When ctx is `None` (all VM callers, tests, non-devtools rust) → zero behavior change. Gate every hook on `RUST_DEBUG_CTX.with(|c| c.get().is_some())`.
- The wrapper's `view_element` sets the ctx (selected VNodeId + empty id_map + cache writer) before `app_view.into_iced()`, clears after.

#### P2-B-2 — Restyle highlight (the "yellow box")
Driven entirely by P2-B-1's selected-check. No new render pass. Selected widget gains an orange border each frame while the panel is open and a node is selected. (Color matches the tree's selected-node orange `rgb(0.85,0.4,0.1)` for consistency.)

#### P2-B-3 — Measured bounds (replaces "(尺寸待测量)")
After `into_iced` fills `id_map` (VNodeId→iced Id), run a `LayoutCollector`-style widget operation over `app_el` to read each Id's bounds (pattern from [renderer.rs:~3009](../../crates/auto-lang/src/ui/iced/renderer.rs#L3009) VM path), then map back VNodeId→`Rect` and `backfill_bounds(&mut cache, &bounds)` (reuse [inspector_cache.rs](../../crates/auto-lang/src/ui/iced/inspector_cache.rs)). Wire via a `Task` from `devtools_update` so it runs after each view rebuild while the panel is open, OR compute inline if the operation API permits a synchronous read post-layout. Box model's `content` rect then shows real `W × H`; drop the "(尺寸待测量)" row.
- **Fallback if synchronous read isn't available in iced 0.14:** ship P2-B-1+B-2 (highlight via restyle, no geometry) and keep "(尺寸待测量)" — the highlight is the user-visible win; bounds are a polish follow-up. Document which path landed.

#### P2-B-4 — Canvas click-to-select (optional, after B-1/B-2)
Clicking a widget on the canvas selects its VNode (reverse of tree→highlight). Wrap each app widget's `mouse_area` (when ctx active) to emit `__vnode_select_<vid>` on click. Reuses the id map from B-1. Skip if it churns too much of `into_iced`; tree-select + highlight already covers the core ask.

**Verify (Phase 2):**
1. `cargo build -p auto` + `cargo test -p auto-lang --lib 'ui::debug'` (≥80) green; add a unit test that drives `into_iced` with a ctx set and asserts the selected node's element carries the highlight border / id_map is populated.
2. `auto r -r rust` in `015-notes` → F12 opens (P2-A).
3. `auto r -r rust` in `012-stopwatch` → select a VNode → that widget gets an orange outline on the canvas (P2-B-2); box model shows real `W × H` if B-3 landed.
4. `render: "vm"` apps unchanged (ctx is `None` on the VM path — no regression).

## Later follow-ups (not in Phase 2)
- Unify VM and rust panel renderers behind a shared `DevToolsAccess` trait (removes the thin rust shells).
- Source / AutoUI tabs (need VM build-time data rust mode doesn't carry).

## Verification
1. `cargo build -p auto` — compiles; VM `into_iced` callers unaffected (thread-local defaults to None).
2. `cargo test -p auto-lang --lib 'ui::debug'` — 80 tests still pass.
3. Regenerate + run `examples/ui/015-notes` with `render: "rust"`:
   - UI opens normally (app renders identically to before when F12 off).
   - Press **F12** → DevTools panel opens on the right (previously: nothing).
   - Left pane: element tree (VTree) of the live view. Click a node → selected (orange path), right pane populates.
   - Right pane 检视 tab: collapsible ▾ 盒模型 / Computed / Properties sections show real data (box-model diagram + numeric rows; computed style k/v; VNode props k/v).
   - Drag the Tree|Inspector divider; resize window — panel tracks.
   - F12 again → panel closes; app fully native.
4. Confirm `render: "vm"` still works identically (no regression from Task 2/4).
