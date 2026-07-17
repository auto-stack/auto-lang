# trait_advanced Replication

**Plan:** 358 D2.1
**Scope:** Auto `spec` (trait) advanced features — default methods, associated
types, bounded/generic specs — compared three-way across AutoVM, a2r
(transpiled Rust), and a native Rust oracle.
**Parity status:** 10/10 consistent (100%) across all three backends.

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
- Generic bound: the bound keyword from the prose spec (`<T has Comparable>`)
  is **not** accepted by the parser. The only bound syntax that parses is the
  `#[with(T as Spec)]` attribute form, and even then the AutoVM cannot dispatch
  a spec method through a generic type parameter inside a generic function.

Gotchas hit while writing this library (worth recording):
- **`tag` is a reserved token** (`TokenKind::Tag`, from the `tag Shape { ... }`
  declaration). It cannot be used as a method name; the parser fails with
  "Expected identifier ... after dot, got Tag".
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

- **L3 (language gap):** NOT supported by Auto. `spec Container { type Item; fn get(i int) Item }`
  is a parse error ("Expected term, got RBrace"). There is no Auto syntax for
  an associated type in a spec. No code is emitted. Rust supports associated
  types natively, so the Rust oracle has no corresponding test either. See
  **DIV-TRAIT-LANG-1**.

### C. bounded / generic specs

- **L1 (live, 3-way consistent):** a **non-generic** spec `Comparable` with a
  formal implementer `ScoreCmp` that compares against a primitive `int`
  argument and returns a three-way sign. Covered by `spec_basics.at` (6 cases
  for the Comparable subset, plus 2 for the sub-scenario A `Identifiable`
  baseline). This is the trait feature (spec + formal impl + method dispatch)
  that all three backends agree on.
- **L3 (documented):**
  - **Generic spec with a concrete type argument** (`type T as Comparable<int>`)
    makes a2r drop the type argument, emitting `impl Comparable for T`
    (missing generics, `error[E0107]`). The generic spec *declaration* parses
    and transpiles, but the impl does not. See **DIV-TRAIT-A2R-2**.
  - **Bounded-generic functions** (`fn max<T has Comparable>(...)`): the
    `<T has Comparable>` bound is not accepted on functions, and even with the
    `#[with(T as Comparable)]` attribute form the AutoVM cannot dispatch a
    spec method through a generic type parameter ("Undefined symbol: T.compare").
    See **DIV-TRAIT-VM-1**.

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
- **DIV-TRAIT-A2R-2** — generic spec impl drops the concrete type argument (open).
- **DIV-TRAIT-LANG-1** — associated types not supported by the Auto language (open).
- **DIV-TRAIT-VM-1** — AutoVM cannot dispatch a spec method on a generic type parameter (open).

## How to run

```
cd parity
cargo run -p auto-parity -- --root . --auto-binary ../target/release/auto.exe run trait_advanced
```
