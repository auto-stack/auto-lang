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
  boundary.** When a function in one module returns `Ok(Url { ... })` and a
  caller in another module destructures it via `Ok(u)`, the struct value is
  corrupted (field reads return the wrong value). Workaround: the url
  replication returns only `str`/`int` primitives across the module boundary
  (no `Url` record), so each accessor re-slices the raw URL string.
  - AutoVM: n/a (design avoids the path)
  - a2r: n/a
  - Rust: n/a
  - 状态: documented (workaround in place)

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

