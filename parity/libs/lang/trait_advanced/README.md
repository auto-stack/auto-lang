# trait_advanced Replication

**Plan:** 358 D2.1
**Scope:** Auto `spec` (trait) advanced features — default methods, associated
types, bounded/generic specs — compared three-way across AutoVM, a2r
(transpiled Rust), and a native Rust oracle.
**Parity status:** 14/14 consistent (100%) across all three backends.

This is an **honest-boundary** library. Its goal is to surface where Auto's
spec system and the a2r transpiler do and do not support Rust-trait-style
advanced features, not to maximise a pass count by avoiding hard cases. Each
sub-scenario is carried at the level the toolchain actually supports; gaps are
recorded in `parity/docs/known-divergences.md`.

## Auto spec syntax (verified)

Confirmed against `CLAUDE.md`, the existing a2r tests under
`crates/auto-lang/test/a2r/{12_specs,13_delegation}/`, the language spec
(`docs/language/specification.md` §"Specs (Traits)"), and the tour chapters
`docs/tour/ch08-methods` and `ch09-generics`:

- Declaration: `spec Name { fn method() }`. Return type uses a space, not an
  arrow: `fn m() int`, never `fn m() -> int`.
- Generic spec: `spec Comparable<T> { fn compare(other T) int }`.
- Default method: a spec method may carry a body, e.g.
  `spec L { fn greet() { print("hi") } }`. The body lives in `SpecMethod.body`
  (Plan 019 Stage 8.5) and a2r emits it into the generated Rust trait.
- Implementation: `type T as SpecName { ... }` (formal), or methods supplied
  via `ext T { ... }` (inherent). `has field Type for SpecName` delegates a
  spec to a field.
- Generic bound (Plan 417-E3): `fn max_of<T has Comparable>(...)` parses into
  the fn's `type_params` constraints, a2r emits `<T: Comparable>`, and the
  AutoVM dispatches the generic receiver's method calls dynamically on the
  runtime type. The older `#[with(T as Spec)]` attribute form keeps working.
  Call-site bound *checking* (rejecting an argument type that does not
  implement the bound) is not yet enforced — recorded in
  KNOWN-DEBT-AND-RISKS.md.

Gotchas hit while writing this library (worth recording):
- **`tag` is a reserved token** (`TokenKind::Tag`, from the `tag Shape { ... }`
  declaration). It cannot be used as a method name; the parser fails with
  "Expected identifier ... after dot, got Tag".
- **Sub-scenario B wrapper locals are named `data` on purpose.** a2r's
  `fix_vec_i32_index` text post-pass rewrites `x.get(i)` into bracket
  indexing for any receiver name not on its hash-map allowlist — including
  user types with a `get` method. Renaming the local to anything off the
  allowlist (e.g. `bx`) silently miscompiles the call (E0608 at build time).
  Root fix is type-awareness in that post-pass; see the debt entry
  (`KNOWN-DEBT-AND-RISKS.md`, Plan 417-E2).
- **Doc-comment scanning misreads code-like punctuation.** Backticks and
  braced/parenthesised code fragments inside `///` or `//` comments confuse
  Auto's comment handling and surface as spurious parse errors elsewhere in
  the file. Comments here are kept prose-only.

## Sub-scenarios

### A. spec default methods

- **L1 (live, 3-way consistent):** a spec `Announcer` with a **void** default
  method `announce` (body `print("[ANN] " + self.label())`) composed from the
  required method `label`. An implementer `Robot` re-declares `announce`
  (mirroring the default), because the AutoVM trait checker does not yet skip
  default-bodied methods. The void form is exactly what a2r compiles
  correctly. Covered by `default_methods_probe.at` (2 cases).
- **L3 (documented, not live):** a **value-returning** default method (e.g.
  `fn greet() str { "hi " + self.who() }`) is miscompiled by a2r — it wraps the
  default body as a statement block so the method returns unit, conflicting
  with the declared return type. See **DIV-TRAIT-A2R-1**. Verified out of band
  but not included as a live test, because any library containing such a
  default method fails to compile under a2r entirely and would spoil the L1
  baseline for every other case.

### B. associated types

- **L1 (live, 3-way consistent, since Plan 417-E2 2026-08-22):** a spec
  declares an associated type with a `type Item` member and references it in
  method signatures; the implementer binds it by name at the impl clause
  (`type IntBox as Container<Item=int>`). The binding substitutes through the
  trait checker (signature conformance), the AutoVM method compilation, and
  the a2r emission (`type Item;` + `Self::Item` in the trait, `type Item =
  i64;` in the impl). Covered by `assoc_types.at` (4 cases). See
  **DIV-TRAIT-LANG-1** (flipped ✅).

### C. bounded / generic specs

- **L1 (live, 3-way consistent):**
  - A **non-generic** spec `Comparable` with a formal implementer `ScoreCmp`
    that compares against a primitive `int` argument and returns a three-way
    sign. Covered by `spec_basics.at` (6 cases for the Comparable subset,
    plus 2 for the sub-scenario A `Identifiable` baseline). This is the trait
    feature (spec + formal impl + method dispatch) that all three backends
    agree on.
  - **Bounded-generic functions** `fn max_of<T has Comparable>(a T, b T) T`
    (Plan 417-E3, formerly DIV-TRAIT-VM-1): the `has` bound parses into the
    fn's type parameters, a2r emits the native Rust bound `<T: Comparable>`,
    and the AutoVM dispatches the generic receiver's method call dynamically
    on the runtime type (CALL_SPEC on the heap tag). The reversed-compare
    implementer `ScoreDesc` proves the dispatch follows each receiver's own
    type. Covered by `bounded_generics.at` (4 cases).
- **L3 (documented):** none remain — the last item, generic spec with a
  concrete type argument (`type T as Comparable<int>`), was fixed across the
  a2r impl emission (Plan 359), the VM trait checker's signature comparison,
  and the a2r trait-declaration emission (Plan 417-followup). See
  **DIV-TRAIT-A2R-2** for the full history.

## API

Entry points are primitive-in / primitive-out so the parity tests never pass a
user struct across the module boundary (a known AutoVM hazard, see
DIV-URL-VM-1) and never trip a2r struct-ownership codegen (E0507/E0382).

- `device_ident(serial int) str` — sub-scenario A baseline, `Identifiable::ident` on `Device`.
- `channel_ident(name str) str` — sub-scenario A baseline, `Identifiable::ident` on `Channel`.
- `announce_robot(id int) str` — sub-scenario A default-method path on `Announcer`.
- `robot_label(id int) str` — sub-scenario A required method `Announcer::label` on `Robot`.
- `max_score_val(a int, b int) int` — sub-scenario C, max via `Comparable` on `ScoreCmp`.
- `score_cmp(a int, b int) int` — sub-scenario C, three-way `Comparable` on `ScoreCmp`.
- `bounded_max_val(a int, b int) int` — sub-scenario C, bounded-generic `max_of<T has Comparable>` on `ScoreCmp` (Plan 417-E3).
- `bounded_max_desc(a int, b int) int` — sub-scenario C, same generic fn through the reversed-compare `ScoreDesc` implementer.

## Implementation notes

- Spec methods take primitive parameters and return primitives, mirroring how
  the url/rusqlite parity libs avoid the AutoVM struct-boundary hazard and the
  a2r struct-ownership borrow-checker gaps. The trait dispatch (required
  method, default method, multiple impls, formal `as Spec` impl) is fully
  exercised.
- The AutoVM trait checker requires the implementer to re-declare every
  default-bodied method; the re-declaration mirrors the default body. The
  default body is still emitted into the Rust trait by a2r, so the
  default-method feature is genuinely present in the generated code.

## Known divergences

See `parity/docs/known-divergences.md` §"trait_advanced (D2)" for:
- **DIV-TRAIT-A2R-1** — value-returning default method miscompiled by a2r (open).
- **DIV-TRAIT-A2R-2** — generic spec impl drops the concrete type argument (fixed: Plan 359 a2r impl emission + 417-followup checker/trait-declaration halves).
- **DIV-TRAIT-LANG-1** — associated types: fixed 2026-08-22 (Plan 417-E2); sub-scenario B now runs live at L1.

## How to run

```
cd parity
cargo run -p auto-parity -- --root . --auto-binary ../target/release/auto.exe run trait_advanced
```
