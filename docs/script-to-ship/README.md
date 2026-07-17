# From Script to Ship — Auto as Rust's Script Layer

This is a **workflow tour**, not a language tutorial. It shows how Auto lets you
develop like a scripter (skip the compile, refresh to see results) and ship like
a Rustacean (native performance + memory safety) — **from the same source code**.

> If you're new to the Auto *language* itself (syntax, types, control flow),
> start with the [Language Tour](../tour/README) first. This tour assumes you
> can read basic Auto.

## The one-paragraph pitch

Auto is Rust's script layer. You (or an AI) write Auto; the AutoVM runs it
instantly with no compile step, so the iterate-refresh loop is seconds, not
minutes. When the work is done, the `a2r` transpiler turns that same source
into short, idiomatic Rust that links against `a2r-std` and ships with native
performance and memory safety. The compiler guarantees the script's behavior
matches the shipped Rust's behavior.

## The three acts

Every chapter is built around three acts:

- **Dev** — write Auto, run it in the VM, iterate in seconds (no compile).
- **Ship** — `a2r` transpiles the same source to Rust; `cargo build` for release.
- **Bridge** — the transpiler guarantees VM output == Rust output. This is
  not a claim; it's [verified by parity tests](../../parity/docs/parity-dashboard.html).

## Evidence, not promises

Auto's "VM and Rust behave identically" claim is backed by an automated
three-way parity harness (`parity/`): AutoVM vs a2r-transpiled Rust vs native
Rust, on real libraries. See the live
[parity dashboard](../../parity/docs/parity-dashboard.html) for current coverage
(L1 = verified three-way, L2 = VM-stable, L3 = roadmap). Each chapter links the
relevant L1 evidence.

## Chapters

1. [Hello, Script & Ship](ch01-hello-script-ship) — the smallest closed loop:
   one program, two execution modes, identical output.
2. [AI in the Loop](ch02-ai-in-the-loop) — why script mode is the right shape
   for AI-driven development.
3. [Types & Ownership](ch03-types-ownership) — structs, enums, and how Auto's
   `view`/`mut`/`take` map to Rust's `&`/`&mut`/move.
4. [Errors](ch04-errors) — Auto's `!` functions and `.?` propagation → Rust's
   `Result` and `?`.
5. [Traits & Generics](ch05-traits-generics) — Auto's `spec` → Rust's `trait` /
   `impl` / `Box<dyn>`. Includes honest boundaries (what a2r supports today).
6. [Ship: Release](ch06-ship-release) — the `a2r` command line, linking
   `a2r-std`, and the performance/safety payoff.

## How to read

Each runnable block is a `<ScriptShipView>`: edit the Auto on the left, hit
**Run in VM** to execute instantly, hit **Transpile to Rust** to see the exact
Rust a2r produces, and (where shown) **Run Both & Compare** to watch the two
backends agree in real time.
