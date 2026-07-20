# Known Divergences

This file records all accepted and open divergences between AutoVM, a2r, and
native Rust for replicated libraries.

## Format

Each entry has:
- **DIV-NNNN**: unique ID
- **库**: library name
- **用例**: test case name
- **AutoVM 行为**: what AutoVM produces
- **a2r 行为**: what a2r transpiled Rust produces
- **Rust 原生行为**: what native Rust produces
- **偏差类型**: 可接受 / 待修复 / 已修复
- **状态**: accepted / open / fixed
- **原因**: explanation

---

(No divergences yet — _dummy is fully consistent across all three backends.)

## url (P1)

All 30 url test cases are consistent across AutoVM, a2r and native Rust
(100%). No test-case-level divergences are accepted.

The following are **library/tooling limitations discovered during the url
replication and worked around in the implementation**, recorded here for
future reference. They are not test-case divergences (every case passes on
all three backends); they shaped the API design.

### VM limitations (AutoVM)

- **DIV-URL-VM-1 — user-defined structs do not reliably cross the module
  boundary.** ~~When a function in one module returns `Ok(Url { ... })` and a
  caller in another module destructures it via `Ok(u)`, the struct value is
  corrupted (field reads return the wrong value). Workaround: the url
  replication returns only `str`/`int` primitives across the module boundary
  (no `Url` record), so each accessor re-slices the raw URL string.~~
  **Fixed in Plan 359 (Bug B1):** a `Url` struct now crosses the module
  boundary through `Result Ok(...)` with all fields readable. The url library
  was rewritten to return `Result[Url, str]` from `parse()`; the test reads
  struct fields (`u.scheme`) directly. The free-function accessors remain as
  thin conveniences but are no longer a workaround.
  - AutoVM: fixed (struct fields readable across boundary)
  - a2r: works (parity runner's `wrap_as_module` now promotes struct fields to
    `pub` so cross-module field reads compile)
  - Rust: n/a
  - 状态: fixed (Plan 359 cleanup)

- **DIV-URL-VM-2 — `Err` string payload of a `Result` is corrupted when read
  via an `is`/`match` binding in the AutoVM.** The bound value comes back as a
  small negative integer (a leaked tag marker) rather than the message
  string. Workaround: error test cases assert *that* parsing failed
  (`is_err`), not the message content (same pattern as base64's `check_err`).
  - AutoVM: Err message unreadable (returns e.g. `-2`)
  - a2r: message reads correctly
  - Rust: message reads correctly
  - 偏差类型: 可接受 (accepted — tests check pass/fail, not message text)
  - 状态: accepted

- **DIV-URL-VM-3 — instance methods on a `type` combined with a static
  `fn new` constructor miscompute field reads** (`[GET_FIELD] non-i32`).
  Workaround: the url replication uses free functions, not type methods.
  - 状态: documented (design avoids the path)

### a2r transpiler (fixed in Plan 355)

- **DIV-URL-A2R-1 — `Result` Ok-type inference assumed `String`** for any
  un-annotated function returning `Ok(...)`, so `Ok(Url { ... })` produced
  `Result<String, String>` and failed to compile. Fixed: the transpiler now
  infers the Ok payload type (struct construction → `Result<Url, String>`;
  string payload → `Result<String, String>` as before). Fixed in
  `crates/auto-lang/src/trans/rust.rs`.
  - 状态: fixed

- **DIV-URL-A2R-2 — `use auto.url: ...` was routed to the non-existent
  `a2r_std::url`** because `url` appeared in the transpiler's hardcoded
  stdlib-module list (although `a2r-std` has no `url` module). This broke
  imports of the replicated `url` library in parity tests. Fixed: `url`
  removed from the stdlib-module lists so the import resolves to
  `crate::url` (the replicated library). Fixed in
  `crates/auto-lang/src/trans/rust.rs`.
  - 状态: fixed

### url crate representation differences (by design)

These are deliberate simplifications of the parser, not bugs. The test suite
is constructed so all three backends agree:

- Optional port/query/fragment use sentinels (`-1` / `""`) instead of
  `Option`; absence is tested separately from presence.
- No default-port stripping (tests use only non-default ports like 8080/8443).
- No percent-decoding or host lower-casing (scheme is lower-cased; hosts in
  tests are already lower-case).

## rusqlite (P3)

All 65 rusqlite test cases are consistent across AutoVM, a2r and native
rusqlite 0.31.0 (100%). No test-case-level divergences are accepted. The
replication covers rusqlite's deterministic query layer — the `FromSql` and
`ToSql` value coercions (type dispatch, integral range checks, Integer->Real /
Integer->bool widening, Blob/Text/Option handling). The native oracle drives
each value through a real in-memory SQLite `SELECT ?1` so the Value->Rust
mapping goes through genuine rusqlite 0.31.0.

The following are **library/tooling limitations discovered during the rusqlite
replication and worked around in the implementation**, recorded here for
future reference. They are not test-case divergences (every included case
passes on all three backends); they shaped the scope and API design.

### Scope limitation: no FFI path for Connection/Statement

- **DIV-RUSQLITE-1 — `use.rust rusqlite::Connection` is not viable in the
  current VM.** `use.rust` `RustFfiBridge` marshals only `VMConvertible`
  primitives (i32, u32, bool, i64, u64, f64, String, Vec<...>, Option, tuples).
  `Connection` and `Statement` are opaque, stateful handles with no
  `VMConvertible` impl and no `RustStdlibObject` shim. Consequently the
  Plan-355 brief's literal "call rusqlite via `use.rust`" path cannot work for
  the connection/query types. The replication instead follows the proven
  parity-library pattern (base64 / url / serde_json / regex / sha2): a pure-Auto
  reimplementation of the deterministic slice, compared three-way against a
  native oracle. SQL execution / query planning is SQLite's job, not
  rusqlite's, and is non-deterministic w.r.t. storage, so it is out of scope.
  - AutoVM: n/a (design avoids the path)
  - a2r: n/a
  - Rust: n/a
  - 偏差类型: 可接受 (accepted — out of scope by design)
  - 状态: documented

### VM limitations (AutoVM), worked around

- **DIV-RUSQLITE-VM-1 — `Result`-wrapped *float* payload corrupts when it
  crosses the module boundary.** A lib function returning `Ok(f)` where `f` is
  `float`, read in another module via an `is`/`match` binding, comes back with
  a mangled bit pattern (`5.0` compares unequal to the literal `5.0`). This is
  the same boundary-corruption family as DIV-URL-VM-1/2, now affecting floats
  in `Result`. Plain (non-`Result`) int / float / str / bool returns cross
  cleanly (bool fixed in Plan 359 / B4 — see the rusqlite cleanup). The parser
  also does not accept the `Result[float, str]` generic syntax that a
  float-returning coercion would need, so the status/value split is still
  load-bearing for the f64/f32 coercions.
  Workaround: each `FromSql` coercion is split into two plain-primitive
  functions — `<name>_status(v) int` (0=Ok, 1=InvalidType, 2=OutOfRange) and
  `<name>_value(v) <T>` — neither of which returns a `Result`. The native
  oracle mirrors this API.
  - AutoVM: float-in-Result corrupted across module boundary
  - a2r: not affected (direct calls, no module boundary)
  - Rust: n/a
  - 偏差类型: 可接受 (accepted — worked around; no test-case divergence)
  - 状态: accepted

- **DIV-RUSQLITE-VM-2 — the 32-bit VM `int` cannot represent i32/u32
  out-of-range boundary values.** The VM `int` is 32-bit signed and silently
  wraps (it has no i64), so the literal `2147483648` (one past `i32::MAX`)
  wraps to `-2147483648`, and `4294967295` (`u32::MAX`) is unrepresentable.
  rusqlite's `FromSql for i32`/`u32` *would* return `OutOfRange` for these, but
  the value cannot be constructed in the VM to test that path. Workaround:
  those specific boundary cases are excluded from the suite; the i32/u32
  in-range and InvalidType paths are covered, and the i8/i16/u8/u16 coercions
  exercise the `OutOfRange` path with values (200, 256, 32768, 65536, -1, ...)
  that ARE representable in a 32-bit int. All three backends agree on every
  included case.
  - AutoVM: cannot construct the out-of-range i32/u32 inputs (32-bit wrap)
  - a2r: same (transpiles the same Auto source; `i32 > i32::MAX` is a
    compile-time-no-op warning)
  - Rust: returns `OutOfRange` correctly (real i64)
  - 偏差类型: 可接受 (accepted — excluded from suite; documented)
  - 状态: accepted

### rusqlite API representation differences (by design)

These are deliberate modelling choices, not bugs. The suite is constructed so
all three backends agree:

- SQLite `Value` is modelled as an opaque `Val { kind, ival, sval, fval }`
  struct (tag + one payload per variant) rather than an enum-with-payload,
  because Auto has no such construct. Callers never read `Val` fields directly.
- `FromSql` error variants are encoded as int status codes (0/1/2) rather than
  the `FromSqlError` enum, because Auto has no enums and the VM corrupts Err
  string payloads (DIV-URL-VM-2).
- `bool` results (`from_bool_value`, `option_is_none`) are returned as plain
  `bool`. (Plan 359 / B4 fixed bool crossing the Auto module boundary, so the
  native oracle and the Auto side now both yield/compare real `bool`s; the
  earlier 0/1-int encoding is no longer needed.)

## Concurrency (Plan 359 Phase 5)

These are language-level limitations of the Auto concurrency model. They are
not test-case divergences; they describe what the runtime supports today.

- **DIV-CONC-1 — `~{ ... }.go` spawn is synchronous-inline (timing).**
  `~{ body }.go` no longer crashes (Plan 359 G1 fixed a stack underflow in the
  `SPAWN_GO` handler and a stray POP in the `Expr::Go` codegen). However, the
  `Expr::AsyncBlock` codegen currently compiles the body **inline** in the
  caller's code stream and passes a placeholder offset (`0`) to `CREATE_FUTURE`.
  As a result the spawned body executes synchronously at the spawn point, and
  `SPAWN_GO` cannot start a real background task (it guards against `body_offset
  == 0` to avoid restarting at address 0). Fire-and-forget programs therefore
  run without crashing, but the spawn does not yet provide true concurrent
  execution. Fixing this requires compiling async-block bodies out-of-line into
  a separate code region and recording the real offset in the Future.

- **DIV-CONC-2 — channels have no Auto-level syntax (language design gap).**
  The VM defines `CHAN_NEW` (0x85), `SEND` (0x86), `RECV` (0x87), and
  `TRY_RECV` (0x88) opcodes with full engine handlers and a Tokio-backed
  `AutoChannel` runtime (`crates/auto-lang/src/vm/channel.rs`), but **no Auto
  surface syntax generates them**. There is no `chan` keyword, no `<-` send/recv
  operator, and no `Channel.new()`/`.send()`/`.recv()` builtin that the parser
  or codegen recognizes. Adding channels is therefore a language-design task
  (deciding syntax + semantics), not a bug fix. Until such syntax exists,
  channel programs cannot be written in Auto source; the opcodes are only
  reachable by hand-assembled ABT. This is tracked as a known limitation.



## trait_advanced (Plan 359 D2)

Three-way parity library `parity/libs/trait_advanced/` is **L1 100% (10/10)**
on its baseline subset: a non-generic spec with required methods, void
default methods, and a non-generic `Comparable` spec with concrete
implementations. The library also probes advanced trait features; the
current status of each:

- **DIV-TRAIT-A2R-1 — value-returning spec default method (FIXED).**
  Previously a2r wrapped the default body as `{ expr; }` -> unit, failing
  E0308. Fixed in Plan 359: `spec_decl` now delegates `Expr::Block` default
  bodies to the generic `body()` emitter, which keeps the tail expression.
  Verified by `crates/auto-lang/test/a2r/12_specs/004_default_body`.
  - 状态: fixed.

- **DIV-TRAIT-A2R-2 — generic spec implementation drops concrete type args
  (open).** `type ScoreCmp as Comparable<i32>` transpiles to
  `impl Comparable for ScoreCmp` (missing `<i32>`), failing E0107. Root
  cause: the spec-impl generator at `trans/rust.rs` reads
  `type_decl.specs` (names only), never `type_decl.spec_impls` (which
  carries `SpecImpl.type_args`). Fix: ~15-20 lines to thread the concrete
  args through. The L1 baseline keeps `Comparable` non-generic to avoid
  this. 状态: open (L3 roadmap).

- **DIV-TRAIT-A2R-3 — spec method bodies do not emit the `self.` prefix
  (open, surfaced during A2R-1 verification).** A spec method that reads a
  field (`return title`) or calls a sibling method (`prefix()`) transpiles
  without `self.`, failing E0425 ("cannot find value/function"). This is a
  deeper a2r gap in spec-method scope resolution, not exercised by the
  existing `12_specs/001-003` tests (whose bodies only call `print`).
  状态: open (L3 roadmap; affects any spec method body that accesses self).

- **DIV-TRAIT-VM-1 — bounded-generic functions (open, VM side).** AutoVM
  cannot dispatch a spec method on a generic type parameter, and the
  `<T has Spec>` bound syntax is unsupported. 状态: open (L3).

- **DIV-TRAIT-VM-2 — VM trait checker requires re-declaration of default
  methods (open, VM side).** Implementers must re-declare every default-
  bodied spec method even though the language intends inheritance. Worked
  around in the library by re-declaring. 状态: open (L3).

- **DIV-TRAIT-LANG-1 — associated types not supported (open, language).**
  Auto's spec grammar has no `type Item;` construct. 状态: open (L3).
