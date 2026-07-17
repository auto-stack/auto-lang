# Chapter 5 — Traits & Generics

Auto's word for `trait` is `spec`. Declare it, implement it on types with
`type T as Spec { ... }`, and dispatch through a spec-typed parameter — a2r
emits Rust `trait` / `impl` / `Box<dyn>`.

<Listing file="script-to-ship/ch05-traits-generics/05_shapes.at" view="scriptship" caption="A spec with two implementations, dispatched dynamically" />

## What works today (L1)

The listing shows the verified subset: a spec with required methods, two
implementations, and a function `area_of(s Shape)` that takes the spec type
and dispatches dynamically. a2r turns this into a Rust `trait Shape`, two
`impl Shape for ...` blocks, and a `Box<dyn Shape>` parameter. This is real
dynamic dispatch, verified three-way against native Rust in the parity suite
(`parity/libs/trait_advanced/`, L1 10/10).

## Honest boundaries (L3)

Auto's spec system is younger than Rust's trait system, and some advanced
forms are not yet supported on both backends. These are documented as open
gaps in `parity/docs/known-divergences.md` §"trait_advanced (D2)", not hidden:

- **Associated types** — Auto's spec grammar has no `type Item;` construct
  (language gap; L3).
- **Default method bodies that return a value** — a2r wraps the body so the
  return type mismatches (a2r gap; void default methods work).
- **Generic spec implementations** — a2r drops the concrete type argument on
  `impl Comparable<i32> for T` (a2r gap).
- **Bounded generic functions** (`fn max<T has Comparable>`) — the bound
  syntax is rejected and the VM can't dispatch through a type parameter.

The point of listing these openly: Auto does not pretend to be finished where
it isn't. The L1 baseline (the listing above) is what's verified; the L3 items
are on the roadmap. When a chapter uses a feature, it tells you which tier
it's in.

Next: [Ship: Release →](ch06-ship-release)
