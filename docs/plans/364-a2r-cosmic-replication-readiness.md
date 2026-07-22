# Plan 364: a2r Readiness for COSMIC Desktop Replication

## Background

The Auto language will be used to replicate the Pop!_OS COSMIC desktop environment
(Rust + iced/libcosmic, ~25 components, Linux/Wayland target). An analysis of the
COSMIC codebase and the current a2r transpiler identified four capability gaps.
This plan records the design decisions for closing them and the concrete work items.

Key constraints discovered during analysis:

- COSMIC's application skeleton (iced `Application`, zbus interfaces, wayland
  Dispatch handlers) relies on attribute macros (`#[zbus::interface]`, serde
  derives), generic trait bounds, and `'static` move closures
  (`Subscription::run`).
- Named lifetimes are **not** needed: COSMIC uses `'_` (~1026 occurrences),
  `'static` (806), and named `<'a>` mostly inside custom widget internals
  (cosmic-files/src/mouse_area.rs etc.). zbus/wayland/application code is
  ~100% covered by lifetime elision + owned types.
- a2r already has partial foundations: fn-level `#[with(T as Spec)]` bounds
  (rust.rs:7116-7129), pass-through annotations `derive/serde/tokio/allow/cfg`
  (Plan 159 6B-2, parser.rs:6849-6868), `~Stream<T>` →
  `impl futures::Stream<Item=T>` (rust.rs:7093-7144, untested), and postfix
  `.move` (`Expr::Move`, rust.rs:1633-1637).

**Scope decision**: COSMIC replication targets the **a2r backend only**. The VM
backend's true-concurrency gaps (spawn degrades to inline execution,
DIV-CONC-1; channels unreachable from language level, DIV-CONC-2) are design-level
and explicitly out of scope here. VM is used for pure-logic tests only.

---

## Decisions

### D1. Dotted annotation paths for attribute macros

Auto's path syntax uses `.` (e.g. `zbus.interface`), not Rust's `::`.
Attribute macros with paths are written as:

```auto
#[zbus.interface(name = "com.system76.CosmicSession")]
impl CosmicSession { ... }
```

Transpiled output:

```rust
#[zbus::interface(name = "com.system76.CosmicSession")]
impl CosmicSession { ... }
```

Rules:

- Annotation name parsing is extended from a single `Ident` to `Ident (Dot Ident)*`
  (parser.rs:6785-6894). Single unknown identifiers still error; any **dotted**
  annotation name falls into the pass-through (raw_attrs) branch unconditionally.
- The `.` → `::` conversion happens **at the parser side** before storing into
  `attrs` (reuses the same convention as `qualify_type_name`, rust.rs:900-960);
  file/store attrs (GDScript) are unaffected.
- No grammar conflict: `.` never appears in annotation-name position today.

### D2. Multiple trait bounds: `#[with(T as A + B)]`

The user's proposal `#[with(T as A | B)]` is parseable (`|` never appears in
type position; parser.rs:6984-6992 currently rejects it), but **rejected on
semantic grounds**: in Auto, `|` already means "or / alternative" (is-branch
pattern multi-match, parser.rs:6306), whereas trait bounds mean "and" (T must
implement A **and** B). Reusing `|` for "and" would be misleading.

Chosen syntax: `#[with(T as A + B)]` — `+` is equally free in type position and
matches Rust intuition.

Implementation: `TypeParam.constraint` changes from `Option<Box<Type>>` to
`Vec<Type>` (ast/types.rs:370); `parse_with_params` (parser.rs:6967-6972)
accepts `Type (+ Type)*`; fn-level bound output (rust.rs:7124-7126) joins with
` + `. Repeated `#[with(T as A, T as B)]` remains valid sugar (existing
same-name merge logic at parser.rs:7042-7043 aggregates into the same Vec).

### D3. Lifetimes: owned-style only (Route A), no language change

No named lifetime support will be added for COSMIC replication. Conventions:

- a2r continues to emit elision-friendly references (`&T`, `&self`) and lets
  rustc infer; `Element<'static, Message>` is the widget-tree style (COSMIC
  itself uses this, e.g. cosmic-applibrary/src/app.rs:361).
- Borrow-holding custom widgets (`struct MouseArea<'a, ...>`) are **rewritten
  as owned designs** during replication, not transpiled literally.
- Estimated coverage: ~85-90% of COSMIC code; the remainder (cosmic-comp
  render layer multi-lifetime traits) stays in upstream Rust anyway.
- This supersedes the deferred "lifetime annotation" item excluded from
  Plan 242 — it stays excluded.

### D4. Move closures: explicit `move` prefix keyword (Option B)

Current state: postfix `.move` (`Expr::Move`) only skips the call-site
auto-clone (rust.rs:6052 requires `Expr::Ident`); it does **not** affect
closure capture. Writing `x.move` inside a closure body does not turn the
outer closure into a move closure — it produces borrow errors instead.
Closures have no capture-mode field (ast/fun.rs:472-481); the only `move`
emission is a hardcoded `thread::spawn` special case (rust.rs:5353-5358).

Rejected alternative (Option A): extending the hardcoded function-name list to
include `Subscription::run` — zero parser change but fragile and non-general.

Chosen design, consistent with Auto's explicit-`.move` philosophy:

```auto
let cb = move (msg: Message) ~Stream<Event> {
    ...
}
```

- Add `is_move: bool` to `Closure` AST (ast/fun.rs:472).
- Parser accepts the existing `TokenKind::Move` (parser.rs:514) as a closure
  prefix keyword.
- a2r emits `move |params| body` at rust.rs:2412 (Closure) and 2320 (Lambda).
- VM codegen's closure compilation accepts and ignores the flag (VM closures
  are environment-capturing already); all `Closure` construction sites updated.

### D5. `~{}` async blocks: unify on the standard statement emitter

Current state: `~{}` → `async move {}` handles only
`Stmt::Expr/Store/Return/Reply`; everything else is **silently dropped**
(rust.rs:2657-2693, `_ => {}`). Dropped: `If`, `For`, `Try`, `Is`, `Block`,
`Break`, `Continue`, `MacroCall`, destructuring `let`, and more.

Fix: build a local `Sink` inside async-block emission and delegate to the
unified `stmt()` entry (rust.rs:6514), which already implements If
(rust.rs:7541), For (rust.rs:7388/7491), etc. This also converges the
duplicated statement emission in Lambda (rust.rs:2343-2373) and Block
(rust.rs:2376-2408). Development is **test-driven**: each statement class gets
a failing test first (see W4).

---

## Work Items

| # | Item | Status | Difficulty | Files | Acceptance |
|---|------|--------|-----------|-------|------------|
| W1 | Dotted pass-through annotations (`#[zbus.interface]` → `#[zbus::interface]`) | ⏳ | ⭐ Low | parser.rs:6785-6894 | `#[zbus.interface(...)]` on impl parses and round-trips; single unknown ident still errors |
| W2 | `Fn.attrs` field + function-level attribute output | ⏳ | ⭐ Low | ast/fun.rs:17-36, rust.rs:7060 area | `#[tokio.main]` / arbitrary attrs on fn emit to Rust |
| W3 | Multi-bound `#[with(T as A + B)]` + struct/trait/impl-level bound output | ⏳ | ⭐⭐ Low-Mid | ast/types.rs:370, parser.rs:6967-6972/7042, rust.rs:7124-7126 + 8351/8502/8521/8610/8688/9420 | bounds emit at all 6 sites; `T: A + B`; spec-as-constraint bypasses the `Box<dyn>` special case (rust.rs:836) |
| W4 | `~{}` full statement support, test-driven | ⏳ | ⭐⭐ Mid | rust.rs:2657-2693 → delegate to stmt() (rust.rs:6514) | new tests under test/a2r/ for If/For/Try/Is/Break/Continue inside `~{}` pass; no silent drops (unknown stmt = compile error) |
| W5 | `move` closure prefix keyword | ⏳ | ⭐⭐ Mid | ast/fun.rs:472, parser.rs (closure syntax), rust.rs:2412/2320, vm/codegen.rs | `move (x) => ...` emits `move \|x\| ...`; `.go`/`~{}` cases unchanged; existing tests unaffected |
| W6 | `~Stream<T>` parity coverage | ⏳ | ⭐ Low | parity/libs/tokio_stream/ (new), parity/crates/auto-parity/src/runner.rs:229-252 | parity runner Cargo template gains `futures`, `async-stream`, tokio `sync` feature; 3-way (VM-skip / a2r / native) tests pass |
| W7 | Local path dependencies in generated Cargo.toml | ⏳ | ⭐ Low | rust.rs:12405 (dep scanner output), dep_scanner.rs | `dep` supports `{ path = "..." }` so Auto projects can depend on local glue crates (auto-cosmic-dbus/-ui); monorepo template (Auto app + local Rust glue) builds end-to-end |

### Dependency order

W1 + W2 + W3 first (small, unblock zbus/serde/generic COSMIC glue). W7 lands
with them (needed by the first replicated component's build). W4 + W5 +
W6 before the first GUI component (iced `Subscription`). All are
prerequisites for cosmic-monitor; W1-W3+W7 alone suffice for cosmic-screenshot
and cosmic-session.

### Testing conventions

- Every work item lands with `test/a2r/` cases; stream/channel behavior lands
  as a new `parity/libs/tokio_stream/` package (VM-side cases marked
  a2r-only via an explicit skip annotation, extending the runner's existing
  divergence classification).
- COSMIC replication milestones (cosmic-screenshot → cosmic-session →
  cosmic-monitor) double as integration tests; features they exercise are
  backported into parity.

## Out of scope

- Named lifetime parameters (`<'a>`), lifetime relationships (`'a: 'b`)
- VM true concurrency (DIV-CONC-1/2)
- cosmic-comp (stays upstream Rust; replication is component-level replacement
  validated against the real compositor)
