# generators — yield iterator parity (Plan 359 D2 / Plan 417-D2)

Three-way parity (AutoVM vs a2r-transpiled Rust vs native Rust oracle) for
Auto's generator syntax: `fn f() ~Iter<T> { yield x; ... }` consumed by `for`.

## Sub-scenarios

| # | Scenario | Auto entry | Rust oracle |
|---|----------|-----------|-------------|
| A | plain multi-yield generator | `three_yields` | `std::iter::from_fn` over a vec |
| B | bounded loop generator + empty edge | `counter(start, count)` | `from_fn` with captured state |
| C | conditional-yield (filter-in-generator) | `evens_up_to(limit)` | `(0..limit).step_by(2)` |

## Findings (2026-08-22)

- **`~Iter<T>` a2r lowering was broken before Plan 417-D2**: the generator
  body is wrapped in `async_stream::stream!` (producing a `Stream`), but the
  declared return said `impl Iterator` — never compiled; the golden
  `001_simple_yield` was left unregistered for this reason. 417-D2 unifies
  `~Iter`/`~Stream` onto the `impl futures::Stream` lowering (the for-in
  consumption rewrite and async-main wrapping ride the same detection).
- **Lazy chains** (generator → map → filter → take) have no Auto syntax yet —
  roadmap item, not covered here (no divergence to record: the feature does
  not exist on the Auto side).
- VM runs the same sources natively (yield is a VM coroutine primitive);
  a2r output and the native oracle agree on all six TAP checks.
