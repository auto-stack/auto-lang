# Chapter 6 — Ship: Release

Everything so far has been the Dev act. This chapter is the Ship act: turning
the Auto source you iterated on into the Rust you deploy.

<Listing file="script-to-ship/ch06-ship-release/06_fib.at" view="scriptship" compare="true" caption="A small program — transpile, read the Rust, compare outputs" />

## The two commands

```bash
# Dev (what you've been doing in every chapter):
auto main.at

# Ship:
auto trans --path main.at rust      # emits main.a2r.rs
# then, with a2r-std on the path:
cargo build --release               # native binary, no VM
```

The `compare="true"` block above adds a **Run Both & Compare** button. It runs
the AutoVM and then compiles + runs the a2r-transpiled Rust, and checks the
two outputs agree. That green checkmark is the whole promise of the language:
the thing you shipped behaves like the thing you iterated on.

## What a2r-std is for

Auto's standard library (`auto.io`, `auto.fs`, `auto.http`, ...) is mirrored
on the Rust side by the `a2r-std` crate. When a2r emits `use crate::io::print`,
that resolves to `a2r_std::io::print` at cargo-build time. So the same call
that the VM executes natively, the shipped binary executes via a thin Rust
wrapper — same behavior, Rust performance.

## The payoff

You kept the iteration speed of a script and the delivery properties of Rust.
No second project to port Python to C++. No "rewrite it in Rust" phase where
behavior silently drifts. The transpiler is on the hook for agreement, and the
[parity dashboard](../../parity/docs/parity-dashboard.html) shows exactly which
libraries and patterns that agreement has been verified on — 232 test cases
across seven real libraries at L1, today.

## Where to go next

- **[Language Tour](../tour/README)** — the language reference (syntax, types,
  control flow) if you haven't read it.
- **[Parity dashboard](../../parity/docs/parity-dashboard.html)** — live
  evidence: which libraries are L1-verified, which are roadmap.
- **[Script-to-Ship demos](../../examples/script-to-ship-demos/)** — runnable
  single-file demos (serde_json, regex, wc) you can clone and ship.

← Back to the [Script-to-Ship overview](README)
