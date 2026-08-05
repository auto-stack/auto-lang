# Plan 365: AutoUI Pluggable Host Architecture (COSMIC Replication on Windows)

## Background

Two constraints collide in the COSMIC replication effort (Plan 364):

1. **Development machines run Windows 11**; Pop!_OS lives in a VMware guest. COSMIC
   components target Linux/Wayland, so naive "transpile and run" does not work on
   the dev machine.
2. The **AutoUI separation architecture** (docs/design/20-autoui-separation-architecture.md,
   from earlier external design discussions) already solves cross-platform
   rendering in principle: apps produce a declarative tree (AutoTree) and render
   commands; a host process owns all GPU resources.

Key realization recorded in this plan: **the existing `VNode`/`VTree` in
`crates/auto-lang/src/ui/vnode.rs` IS the AutoTree**. It already carries stable
`VNodeId`, parent/children links, `VNodeProps`, a stable logical `path`, and
`source_span` (vnode.rs:276-295) — exactly the fields design doc 20 specifies
(NodeId, kind, props, SourceMap). Existing render backends (iced/, gpui/,
headless/, interpreter/) are, in design-doc terms, in-process hosts. No new IR
will be created.

The missing analysis from doc 20: it covers **rendering** only. COSMIC
components' Linux-only surface is mostly **system services** (DBus, Wayland
protocols, /proc, PAM, PipeWire), which is orthogonal to rendering and needs its
own abstraction.

## Decisions

### D1. AutoTree = existing VNode/VTree, no third IR

`ui/vnode.rs` is the single UI intermediate representation. Extensions needed by
doc 20 (diff/patch protocol, RenderCommand lowering) are added **to** VTree and
its backends, never as a parallel tree. AURA remains the compile-time IR that
lowers into VTree at runtime (aura_view_builder.rs, snapshot_builder.rs).

### D2. One app core, three pluggable hosts

```
                App core (Auto, platform-neutral)
      state machine + messages + AutoTree + system-port calls
                            │
      ┌─────────────────────┼───────────────────────┐
      ▼                     ▼                       ▼
 Host ① Dev host      Host ② COSMIC host      Host ③ AutoOS host
 (Windows/Linux)      (Linux deliverable)     (doc-20 full vision)
 VTree → wgpu/winit   VTree → libcosmic       VTree → RenderCommand
 in-process, mock     Element, in-process     → RenderQueue → shared
 system ports         real Wayland citizen    compositor — DEFERRED
```

- **Host ① (dev)**: today's dynamic iced renderer
  (`crates/auto-lang/src/ui/iced/`) running on Windows with wgpu. System ports
  are satisfied by scriptable mocks. This is the day-to-day COSMIC-replication
  environment.
- **Host ② (target)**: a new libcosmic backend that lowers VTree to
  libcosmic/iced `Element`s, in-process, producing real Wayland citizens
  (layer-shell, xdg activation, cosmic-protocols via Plan 364 glue). Replicated
  components can be dropped into a real COSMIC session and verified against the
  upstream compositor.
- **Host ③ (AutoOS)**: the shared compositor + RenderQueue from design doc 20.
  **Deferred** until the AutoOS phase; it is an optimization for the
  100-apps-shared-GPU scenario and is not needed by hosts ①/②, which render
  in-process. App cores require no changes when host ③ arrives.

  Windows implementation shape for the separated host (design input for W5):
  the host is **not a compositor on Windows** — DWM is. `autoui-host` is a
  DWM *client*: a winit multi-window process holding the single shared wgpu
  context (font atlas, texture pool, pipeline cache), creating one OS window
  per app and executing that app's RenderCommand stream into the window
  surface; window stacking/decorations/focus/final composition come free from
  Windows. Its components: window registry (app connection → window →
  surface), GpuBackend (wgpu; text via cosmic-text, shared font atlas),
  RenderQueue transport (Windows named shared memory `CreateFileMapping` +
  named events instead of memfd/eventfd; `shared_memory` crate or iceoryx as
  starting point), a low-frequency control channel (localhost socket/named
  pipe for handshake and window management), and an input router (winit
  events → AutoUI event protocol → app downlink queue). ~80% of its code —
  the RenderCommand executor, font atlas, texture pool — is built and
  equivalence-tested earlier, in migration Stage 1's in-process loopback;
  Stage 2 wraps it with the transport layer. On Linux the same binary runs as
  a winit client for debugging; only the AutoOS phase grows a smithay-based
  true-compositor variant (Linux-only).

  Resilience requirements (design input for W5, protocol decisions needed
  BEFORE RenderQueue is built): the shared host is an accepted single point
  of failure — same risk profile as a Wayland compositor or Chrome's GPU
  process — so the goal is fast stateless recovery, not SPOF elimination.
  (a) Apps own their state and AutoTree; the host holds only rebuildable GPU
  resources, so a host crash must never kill apps: apps detect control-channel
  loss, wait, and reconnect. (b) Protocol-level: reconnect handshake carries a
  host generation counter; generation mismatch triggers a Full-frame resend
  (doc 20 §4.2 Full/Incremental already supports this); RenderQueue ownership
  lives app-side or is app-recreatable. (c) A watchdog (supervisor process /
  Job Object on Windows, systemd Restart on Linux) restarts the host.
  (d) Host hardening: all RenderCommand parsing is bounds-checked (untrusted
  input from every app), wgpu device-lost recovers in-process without exit.
  (e) Optional sharding by criticality (shell apps vs user apps on separate
  hosts) to shrink blast radius. (f) Ultimate fallback: the permanent
  in-process path lets an app degrade to self-rendering when no host is
  reachable. None of this applies to hosts ①/② (in-process) — it only
  constrains W5's protocol design.

### D3. System ports are a separate, orthogonal layer

Every Linux-only system dependency is accessed through a port interface with
two implementations:

- **Linux adapter**: real zbus / wayland-client / /proc code (Plan 364 glue
  crates: `auto-cosmic-dbus`, protocol bindings).
- **Windows dev mock**: scriptable fake (recorded fixtures, synthetic events)
  so app logic is fully exercisable on Windows.

Initial port list (from the COSMIC component analysis): NotificationsPort,
AudioPort (settings-daemon varlink), DisplayPort (cosmic-randr), NetworkPort
(NM), BluetoothPort (BlueZ), PowerPort (UPower/logind), SessionPort
(systemd/logind/greetd), PortalPort (xdg-desktop-portal), SecretsPort.

### D4. RenderQueue / shared-memory IPC is explicitly out of scope

Hosts ① and ② render in-process. The lock-free shared-memory RenderQueue,
dirty-rect incremental frames, and the <30 MB self-hosted Vulkan compositor
(doc 20 §5-§6, §9) are deferred to the AutoOS phase with entry conditions:
≥3 replicated apps running on host ②, and measured memory/latency budgets that
justify the split.

### D5. Wayland-bound components keep a Linux verification loop

Host ① mocks cannot verify protocol behavior. Wayland-bound components
(cosmic-bg, panel, applets, notifications, osd, launcher, randr, …) are
integration-tested on real Wayland via **WSL2 + WSLg** on the Windows dev
machine (WSLg provides a real Wayland compositor; nested cosmic-comp via its
winit backend covers cosmic-private protocols), with the VMware Pop!_OS guest
as the final validation environment.

## Work Items

| # | Item | Status | Difficulty | Files | Acceptance |
|---|------|--------|-----------|-------|------------|
| W1 | Unify host backend interface over VTree | ✅ | ⭐⭐ | crates/auto-lang/src/ui/{host.rs (new), mod.rs, app.rs} | `HostBackend` enum + `run` + `default_for_features`; `App::run` delegates to it; headless + iced compile; existing examples unchanged |
| W2 | Dev-host mock framework for system ports | ✅ | ⭐⭐ | new: crates/auto-cosmic/{ports,demo}/ | a demo app (clock+battery applet logic) runs on Windows driven by scripted mock events |
| W3 | libcosmic host backend (VTree → Element) | 🔨 scaffold | ⭐⭐⭐⭐ | crates/auto-cosmic/host-libcosmic/ (excluded from Windows workspace) | crate structure + lowering design doc; real VTree→Element impl requires libcosmic (Linux), built incrementally per replicated component |
| W4 | Linux port adapters | 🔨 scaffold | ⭐⭐⭐ | crates/auto-cosmic/ports-linux/ (excluded from Windows workspace) | adapter signatures + service-mapping doc; real zbus/UPower impls require Linux, built per COSMIC component |
| W5 | (Deferred) RenderCommand/RenderQueue/compositor per doc 20 | ⏸ Deferred | ⭐⭐⭐⭐⭐ | new crates | entry conditions in D4 met |

### Dependency order

W1 first (defines the seam). W2 enables the Windows daily loop immediately.
W3 starts after Plan 364 W1-W3 (attribute macros, fn attrs, generic bounds)
land. W4 proceeds per COSMIC component being replicated. W5 is parked.

## Migration & Continuity (standalone apps → separated architecture)

The eventual move from in-process rendering to the doc-20 separated
architecture is **additive below VTree**. App sources, AURA, VTree, state
management, and event routing never change; only the stage after "VTree is
built" gains new options:

```
.at source → AURA → VTree → ┬ in-process iced renderer (KEPT permanently)
                            ├ in-process RenderCommand executor (loopback)
                            └ RenderQueue → separate host process
```

Rules:

1. **The in-process path is never deleted.** It remains host ① (dev host) and
   the debug/fallback mode even in the AutoOS endgame. Which path runs is a
   `pac.at` backend setting, not a code change.
2. **Staged rollout, each stage independently shippable:**
   - Stage 0 (now): examples run on the in-process iced renderer. No change.
   - Stage 1: `VTree → RenderCommand` lowering + in-process loopback executor
     (no IPC). Examples switch paths via flag, default unchanged. Acceptance:
     equivalence tests (below) pass on every example.
   - Stage 2: RenderQueue transport, two processes, single app, Windows first
     (wgpu/winit are already cross-platform). Opt-in per example.
   - Stage 3: multi-app shared host (font atlas / texture pool
     centralization) — the actual memory payoff of doc 20. Standalone
     in-process mode still available.
3. **Machine-checkable invariants, not eyeballing:**
   - VTree snapshots (`vtree_atom.rs` VTreeAtomBuilder) of each example must
     be byte-identical before and after each stage — proves "app side zero
     change" as a test, not a promise.
   - RenderCommand streams are serialized and golden-compared: same VTree
     snapshot → same RenderCommand sequence (Stage 1's core test).
   - `ui/headless/` (Plan 174) builds VTrees in memory without a window and
     is the runner for all equivalence tests.
   - `ui/debug/` tooling (inspector, hit_test, source_map) operates on VTree
     and keeps working under every host.
4. **CI gate:** "all examples run in in-process mode" is a permanent CI job;
   each new stage adds a job, never replaces one.

### Relation to other plans

- Plan 364 (a2r COSMIC readiness): unchanged; its `auto-cosmic-ui` glue crate
  is now specified as host ② (W3 here).
- Design doc 20: §3 (AutoTree) already implemented as ui/vnode.rs; §4-§6
  (RenderCommand/RenderQueue/compositor) deferred per D4; §8 (DevTools)
  partially exists under ui/debug/.
- COSMIC roadmap: phases and component order unchanged; host ① shortens the
  feedback loop for every GUI component, WSL2 covers protocol verification.

## Out of scope

- RenderQueue/shared-memory IPC, dirty-rect protocol (W5, deferred)
- cosmic-comp itself (stays upstream Rust)
- VM-backend GUI work (COSMIC replication is a2r-only, per Plan 364)

---

## Progress log

### W1 — landed (unified `HostBackend` interface)

**Design**: a `HostBackend` enum (not a trait object — the per-backend bounds
differ: iced requires `C::Msg: Send`, gpui takes a title arg, and the backend
element types are foreign). Each variant is `#[cfg(feature = "ui-*")]`-gated.
New file `crates/auto-lang/src/ui/host.rs`:

- `HostBackend::run::<C>(self)` — unified entry; delegates to the existing
  per-backend `run_app`/`run_headless`. Requires `C::Msg: Send` (iced's
  constraint, propagated to the method since Rust checks bounds at function
  scope, not per-match-arm). This is acceptable: GUI message types are
  conventionally `Send`, and the headless/gpui paths work with any `Send` msg.
- `HostBackend::default_for_features()` — auto-detect (Headless > Iced > Gpui
  priority, matching the historical `App::run` cfg-ladder).
- `App::run::<C>()` rewritten to delegate to
  `HostBackend::default_for_features()?.run::<C>()`.

**Preserved**: all per-backend direct entry points (`iced::run_app`,
`gpui::run_app`, `headless::run_headless`) remain public and unchanged —
examples that call them directly still work. W1 is additive.

**Verification**:
- `cargo build --features ui-headless` ✅ + `cargo build --features ui-iced` ✅
- `cargo build --features ui-gpui` — 3 pre-existing E0004 errors in
  `gpui/renderer.rs` + `style/gpui_adapter.rs` (incomplete `View`/`StyleClass`
  matches), confirmed identical on clean master; NOT introduced by W1.
- Host tests 2/2 (default-picks-headless, headless-runs-component).
- a2r regression: 301/0 (zero new failures).

**Follow-up (not in W1)**: de-ice `Component::subscription` — moving it to an
`IcedComponent` extension trait would also require updating the a2r codegen
(`ui_gen/rust.rs:983` generates `fn subscription` inside `impl Component`).
Recorded as a separate cleanup.

### W2 — landed (dev-host mock framework for system ports)

**Delivered**: two new workspace crates under `crates/auto-cosmic/`:

1. **`auto-cosmic-ports`** — the system-port abstraction layer:
   - `SystemPort` trait (base for all ports) + concrete port traits:
     `PowerPort` (battery/AC, UPower on Linux), `ClockPort` (wall-clock time),
     `NotificationsPort` (desktop notifications, FreeDesktop D-Bus on Linux).
   - `mock` module: scriptable mock impls (`MockPowerPort`, `MockClockPort`,
     `MockNotificationsPort`) — push scripted states/events in, the app reads
     them out. Mutex-protected, `Send`-safe, `#[derive(Debug)]`.

2. **`auto-cosmic-demo`** — a `ClockBatteryApplet` (clock + battery indicator)
   implementing `auto_lang::ui::Component`. Driven by `Arc<MockClockPort>` +
   `Arc<MockPowerPort>`, rendered via `HostBackend::Headless` (the W1 seam).
   Platform-neutral: the same component runs under the libcosmic host (W3) on
   Linux with real ports (W4).

**Verification**: `cargo run -p auto-cosmic-demo` on Windows produces:
```
ClockBatteryApplet: 12:00:00, battery 78%, AC=false
After 2 min: 12:02:00, battery 76%
```
Clock advance (+120s → 12:02:00) and battery change (78%→76%) both work;
headless render via `HostBackend` exits cleanly. Ports crate tests 3/3.

**Design note**: W2 ports are deliberately minimal (3 ports: Power, Clock,
Notifications). The plan's full list (Audio, Display, Network, Bluetooth,
Session, Portal, Secrets) will be added incrementally as replicated COSMIC
components exercise them — per the plan's "W4 proceeds per COSMIC component
being replicated" directive.

### W3 — scaffold (libcosmic host backend — Linux-only)

**Delivered**: `crates/auto-cosmic/host-libcosmic/` — the crate structure,
lowering design, and `run_libcosmic::<C>()` entry signature. The actual
VTree→libcosmic-Element lowering requires the `libcosmic` crate
(Linux/Wayland-only) and is **excluded from the Windows workspace build** via
`Cargo.toml` `exclude`. On Linux, the crate's `src/linux.rs` documents the full
lowering map (all 18 `VNodeKind` → libcosmic widgets) and event-routing strategy.

**Why scaffold, not full impl**: W3 is ⭐⭐⭐⭐ difficulty (the plan's highest) —
it must cover all 18 widget variants, libcosmic theming, and cosmic-protocols
integration (layer-shell for panels). It is built **incrementally, driven by
replicated COSMIC components** — the plan says "widget coverage driven by
cosmic-monitor replication." There are no replicated components yet (that is the
post-365 work: cosmic-screenshot → cosmic-session → cosmic-monitor), so the
scaffold is the correct deliverable at this stage. The first real component
replication will fill in the lowering.

**Build**: Linux/WSL2 only. `compile_error!` on non-Linux prevents accidental
Windows build. On Linux: uncomment libcosmic dep in Cargo.toml, implement the
lowering in `src/linux.rs`.

### W4 — scaffold (Linux port adapters — Linux-only)

**Delivered**: `crates/auto-cosmic/ports-linux/` — adapter signatures and a
service-mapping doc (`src/linux.rs`) covering `LinuxPowerPort` (UPower),
`LinuxClockPort` (clock_gettime), `LinuxNotificationsPort` (FreeDesktop D-Bus).
The `LinuxClockPort::now_secs()` impl is already functional (uses
`std::time::SystemTime`); the UPower and D-Bus adapters require `zbus`
(Linux-only) and are stubbed with TODO markers.

**Excluded from Windows workspace** the same way as W3.

**Why scaffold**: same reason as W3 — the real adapters are validated against
real D-Bus services on WSL2 (W4 acceptance: "pass integration test on WSL2"),
which requires a Linux environment. The W2 mock impls remain the primary path
on Windows. The adapter stubs document the exact service interfaces so a Linux
developer can fill them in without re-deriving the design.

### Plan 365 status after W1–W4

| WI | Status | Notes |
|----|--------|-------|
| W1 | ✅ landed | `HostBackend` unified interface |
| W2 | ✅ landed | mock framework + demo applet (Windows-verified) |
| W3 | 🔨 scaffold | libcosmic host (Linux, incremental per component) |
| W4 | 🔨 scaffold | Linux port adapters (Linux, incremental per component) |
| W5 | ⏸ deferred | RenderQueue/compositor (AutoOS phase) |

The cross-platform foundation (W1–W2) is complete and Windows-verified. W3/W4
are Linux deliverables whose real implementations are driven by COSMIC component
replication — the next phase of work beyond Plan 365. The scaffolds make that
work a "fill in the lowering/adapters" task rather than a "design from scratch"
task.
