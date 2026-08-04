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
| W1 | Dotted pass-through annotations (`#[zbus.interface]` → `#[zbus::interface]`) | ✅ | ⭐ Low | parser.rs:6785-6894 | `#[zbus.interface(...)]` on impl parses and round-trips; single unknown ident still errors |
| W2 | `Fn.attrs` field + function-level attribute output | ✅ | ⭐ Low | ast/fun.rs:17-36, rust.rs:7060 area | `#[tokio.main]` / arbitrary attrs on fn emit to Rust |
| W3 | Multi-bound `#[with(T as A + B)]` + struct/trait/impl-level bound output | ✅ | ⭐⭐ Low-Mid | ast/types.rs:370, parser.rs:7288-7329/7643-7655, rust.rs:957-963 + 13 output sites | bounds emit at all sites; `T: A + B`; spec-as-constraint bypasses the `Box<dyn>` special case (`rust_bound_name`, rust.rs:957) |
| W4 | `~{}` full statement support, test-driven | ✅ | ⭐⭐ Mid | rust.rs:3031-3067 → delegate to stmt() (rust.rs:7838) | new tests under test/a2r/ for If/For/Is/Break/Continue inside `~{}` pass; no silent drops (unknown stmt = compile error) |
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

---

## Progress log

### W1 + W2 — landed (commit `9b905dd0`)

- **W1 (D1)**: dotted annotation path parsing (`#[zbus.interface]` →
  `#[zbus::interface]`). New `is_annotation_dotted_path` lookahead in the
  parser (with a streaming-token rewind fix); dotted annotations enter a
  separate `impl_attrs` bucket so they don't merge into struct-level
  `derive`. New fields `TypeDecl.impl_attrs` / `Ext.attrs` / `Fn.attrs` and
  render sites: ext/impl block → `#[attr]` prefixed before `impl`;
  type's own methods → before `impl Type {`; `merge_ext_blocks` folds ext
  annotations into the target type's impl block. Annotation dispatch gained
  Ext/Impl branches (previously "Expected ... after annotation").
- **W2**: fn-level attribute output — `#[tokio.main]` → `#[tokio::main]`
  prefixed before `fn`. `Fn.attrs` field added (with `new`/`with_ret_name`/
  `Default` defaults); `fn_decl` renders `#[attr]` before `fn`.
- Golden: `16_interop/018_dotted_attrs` covers impl-block / type-method / fn
  three states. Verification: a2r suite 292 passed; remaining lib failures
  are the pre-existing baseline (17 dstr + 1 route).

### W3 — landed (commit `e01f0f84`, folded into the Plan-018 C8 const work)

W3 (multi-bound `#[with(T as A + B)]`) was implemented alongside the C8 const
keyword work rather than as a standalone Plan-364 commit. This is the reason
the C8 commit widened the transpiler hot path enough to push the deep-recursion
cookbook tests over the libtest stack budget (see the next section). All four
pieces are in place and verified:

- **AST** (`ast/types.rs:370`): `TypeParam.constraint` is `Vec<Type>` (not the
  single-`Option` the original plan assumed); `Display` joins with ` + `.
- **Parser** (`parser.rs:7288` `parse_with_params`): after `as`, parses
  `Type (+ Type)*` (7318-7326). Repeated `#[with(T as A, T as B)]` aggregates
  into the same param's constraint Vec via the same-name merge at
  `parser.rs:7643-7655` (`extend`, not overwrite).
- **Spec-as-constraint bypass** (`rust.rs:957` `rust_bound_name`): a bound
  type renders its bare name — `Type::Spec` → `spec.name`, `Type::User` →
  qualified name — so `#[with(T as Greeter)]` emits `T: Greeter`, never
  `T: Box<dyn Greeter>`. This is the D2 "spec-as-constraint bypasses the
  `Box<dyn>` special case" acceptance criterion.
- **Output sites**: 13 emit sites in `rust.rs` (8441 fn-level + struct/trait/
  impl/where-clause sites at 9967/10194/10251/10282/10383/10510/10652/10683/
  11066/11327/11411/11523) — well beyond the 6 the plan named. Each renders
  `T: A + B` by iterating the constraint Vec with ` + ` separators.
- **Golden**: `16_interop/019_multi_bound` covers fn-level multi-bound, the
  `#[with(T as A, T as B)]` aggregation, type-level (`struct Pair<K: Debug>`),
  and spec-as-constraint (`T: Greeter`). Test passes.

Verification: `cargo test -p auto-lang --lib --features test-trans
test_16_interop_019_multi_bound` → ok.

### W4 — landed (`~{}` full statement support, test-driven)

**Defect fixed**: the `~{}` (async block) emission at `rust.rs:3031-3067`
hand-matched only `Expr/Store/Return/Reply` and **silently dropped** every
other statement class via `_ => {}`. So `~{ if x { ... } }`,
`~{ for i in ... { } }`, `~{ break }`, `~{ continue }` vanished from the
generated Rust — silently wrong code, exactly the COSMIC `Subscription::run`
shape (`~{ for evt in stream { ... } }`) that D5 targets.

**Fix (D5 directive)**: replaced the hand-match with delegation to the unified
`stmt()` entry (`rust.rs:7838`), which handles 22 variants with an
`_ => Err(...)` catch-all (loud failure, no silent drops). The bridge between
the two output styles (`stmt()` writes to a `Sink`; the async-block arm has
`out: impl Write`) uses a fresh `Sink::dummy()` per statement, drained into
`out` after each `stmt()` call. A fresh sink per statement is required because
`stmt()` internally calls `sink.record()` (e.g. inside `emit_loop_body`),
which slices `body[record_pos..]` — reusing one sink without resetting
`record_pos` after `clear()` slices out of bounds.

**Separator logic**: `stmt()`'s `Store/Return/Reply/Break/Continue` arms
already emit their own trailing `;`; only `Expr` omits it (callers add it).
The new code appends `;` only for `Expr`, and a space separator between
statements, matching the old single-line `async move { x; y; }` style.

**Scope note — `Try`/`Block`**: these two variants are NOT arms in `stmt()`
(they hit `_ => Err`). Before W4 they were silently dropped inside `~{}`;
after W4 they produce a clear transpiler error. Full a2r `try/catch` lowering
(a separate, larger feature — the whole try→`Result`/`?` transformation) is
out of W4 scope and tracked separately. `stmt()` does not yet emit `Try`/`Block`
at the fn-body level either.

**Golden**: `16_interop/020_async_block_stmts` — `~{ var ...; for ...;
if ...; expr }` exercising Store/For/If/Expr inside an async block. Test-driven:
written first (red — For/If dropped), then implementation made it green.

**Test harness note**: the `~{ for ... }` shape drives the same deep-transpiler-
recursion that overflows the 2 MB libtest worker stack (see the stack-overflow
section below). A `test_a2r_deep` helper (16 MB dedicated thread, mirroring
`test_cookbook_deep`) was added for this case.

Verification: `cargo test -p auto-lang --lib --features test-trans
tests::a2r_tests::` → **296 passed, 0 failed, 0 overflows** (stable across
2 runs). Existing async tests (`005_async_move`, `001_async_fn`) unchanged.
Bonus: eliminating the stack-overflow crashes also resolved 3 previously-
flaky golden tests (`rand_custom` 006/010, `log_custom` 004) that had been
collateral damage of overflow-induced process crashes — the full a2r suite is
now green.

### Incidental: deep-recursion stack-overflow class (mitigated, not a regression)

During W1/W2 + Plan-018 C8 landing, several cookbook a2r tests that drive deep
recursion in the transpiler's hot path (type inference / chained-method /
iterator / nested-index lowering) began overflowing the **2 MB libtest worker
thread stack** on Windows debug (`STATUS_STACK_OVERFLOW`, 0xc00000fd). This is
a stack-budget issue, not a functional regression — the cases transpile
correctly under a larger stack.

Affected cases (all confirmed failing identically on the clean `master`
baseline):

- `test_cookbook_file_003_recursive_size` — newly overflowed after C8/W1/W2
  widened the hot-path frames (this was the trigger for the investigation).
- `test_cookbook_science_mathematics_linear_algebra_001_add_matrices` — same
  class; previously latent.
- `test_cookbook_science_mathematics_linear_algebra_002_multiply_matrices` —
  already had an ad-hoc inline 4 MB-thread wrapper; now unified.

**Why `build.rs`'s `/STACK:64M` does not help here**: that linker flag governs
the `auto.exe` main thread (and the test binary's main thread), which fixed
the large-`.at`-file parse overflow (Plan 018 `specs.at`, ~1100 lines). It
does **not** govern the per-test worker threads that libtest spawns — those
default to 2 MB and are controlled by `RUST_MIN_STACK` / explicit
`thread::Builder::stack_size`. The two stack budgets are independent.

**Mitigation**: added `test_cookbook_deep(case)` in `a2r_tests.rs` — spawns a
dedicated 16 MB thread, runs the transpile, propagates the result. Routed the
three deep cases through it (and removed the ad-hoc inline wrapper on 002).
Verification: a2r suite now runs to completion with zero stack overflows;
292 pass / 3 fail, where the 3 failures (`rand_custom` 006/010,
`log_custom` 004) are pre-existing golden mismatches on the baseline,
unrelated to this work.

**Known, deliberately out of scope here**: `perf_benchmark_tests::
benchmark_nested_loops` overflows via the *evaluator/VM* execution path (not
the a2r transpiler), so it is a separate stack-budget problem from the a2r
ones mitigated above. It is left for a VM-stack follow-up.

**Root-cause follow-up (deferred)**: the durable fix is to shrink the
transpiler's per-frame size on the recursive hot path (fn_decl / type_decl /
stmt / Pratt expression lowering), not to keep raising test-thread stacks.
Tracked here as the long-term direction; the test-thread mitigation is the
pragmatic interim.
