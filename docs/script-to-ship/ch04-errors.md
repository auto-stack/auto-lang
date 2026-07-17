# Chapter 4 — Errors

Auto's error model is Rust's error model: `Result`, the `?` operator, explicit
propagation. The spelling differs slightly; the semantics are identical.

<Listing file="script-to-ship/ch04-errors/04_safe_div.at" view="scriptship" caption="A !int function (Result) with .? propagation" />

## The spelling

| Auto | Rust |
|------|------|
| `fn f() !int` | `fn f() -> Result<i32, String>` |
| `return Err("...")` | `return Err("...".to_string())` |
| `return a / b` (in a `!int` fn) | `return Ok(a / b)` |
| `x.?` | `x?` |
| `is r { Ok(v) -> ..., Err(e) -> ... }` | `match r { Ok(v) => ..., Err(e) => ... }` |

In the listing, `half_of` calls `safe_div(...).?` — on success it divides by 2,
on failure it propagates the same `Err` to its own caller. Transpile it and
watch a2r emit `?`. The `main` then pattern-matches on the result with `is`,
exactly mirroring a Rust `match`.

## Why Result, not exceptions

Auto has no exceptions. This is deliberate: a script that can fail by surprise
at any depth is not a substrate you can confidently transpile to Rust and
ship. By forcing errors into the type system (`!T` return types), Auto makes
every failure point visible to the compiler — and to a2r, which needs that
visibility to emit Rust `Result` rather than `panic!`. The script is easy to
iterate on *because* its failure modes are honest and typed, not in spite of it.

Next: [Traits & Generics →](ch05-traits-generics)
