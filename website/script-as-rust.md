---
title: Auto as Rust's Script Layer
layout: page
sidebar: false
---

<script setup>
import ScriptShipView from './.vitepress/theme/components/ScriptShipView.vue'
const heroAuto = [
  'fn fib(n int) int {',
  '    if n < 2 { return n }',
  '    return fib(n - 1) + fib(n - 2)',
  '}',
  '',
  'fn main() {',
  '    var line = ""',
  '    for i in 0..12 {',
  '        if i > 0 { line = line + ", " }',
  '        line = line + fib(i).to(str)',
  '    }',
  '    print("fib: " + line)',
  '}',
].join('\n')
</script>

# Auto is Rust's Script Layer

> **Python taught the world that fast iteration wins. Rust taught the world
> that safety wins. Auto refuses to choose.**

You (or an AI) write Auto. The AutoVM runs it instantly — no compile step, so
the iterate-refresh loop is seconds, not minutes. When the work is done, `a2r`
transpiles that same source into short, idiomatic Rust that links against
`a2r-std` and ships with native performance and memory safety. The compiler
guarantees the script's behavior matches the shipped Rust's behavior.

## See it: one program, two execution modes

Edit the Auto on the left. Hit **Run in VM** to execute instantly (no
compile). Hit **Transpile to Rust** to see the exact Rust `a2r` produces.
Hit **Run Both & Compare** to watch the two backends agree in real time.

<ScriptShipView
  :auto="heroAuto"
  :compare-run="true"
  caption="The whole pitch in one block: script now, Rust at ship, identical output."
/>

## The three acts

**Dev** — write Auto, run it in the VM, iterate in seconds. No compile waiting
on every turn. AI gets to be wrong many times, fast, because being wrong is
cheap.

**Ship** — `a2r` turns the same source into Rust you'd have written by hand:
real `trait` / `impl` / `Box<dyn>`, generics, ownership, `Result` + `?`. Link
`a2r-std`, `cargo build --release`, deploy.

**Bridge** — the transpiler is on the hook for agreement. AutoVM output ==
transpiled-Rust output. Not a claim: [verified by 232 three-way parity
tests](https://github.com/zhaopuming/auto-lang/blob/masterhttps://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html)
across seven real libraries.

## Why this beats "Python now, rewrite in Rust later"

| | Python + C/C++ (or Rust) | Auto + Rust (a2r) |
|---|---|---|
| **Ecosystem** | Two split ecosystems; FFI is a bridge with real cliffs | One ecosystem — Auto fully supports Rust's patterns + std + crates, `a2r-std` is the thin mirror |
| **Capability parity** | Python lacks types/ownership/zero-cost abstractions; C/Rust lack ergonomics | Auto and Rust agree on what a program *means* (same trait/generic/ownership/async semantics) |
| **Migration cost** | Python → C/C++ is a complete rewrite project, heavy AI involvement | Auto → Rust is mechanical transpilation; the compiler guarantees behavior |
| **Behavior consistency** | None — Python and C disagree on numerics, concurrency, memory routinely | Enforced — the parity harness fails the build on divergence |
| **AI assistance** | Python is easy to generate; the C/Rust rewrite is a second mountain | Auto is easy to generate (script mode tolerates imperfection); the Rust step is deterministic |

The core difference: with Python+C, the rewrite is a *design* problem (the two
languages don't agree on semantics). With Auto+Rust, the "rewrite" is a
*compiler* step — and compilers are more reliable than AI rewrites.

## Evidence, not promises

Auto's "VM and Rust behave identically" claim is backed by an automated
three-way parity harness: AutoVM vs a2r-transpiled Rust vs native Rust, on
real libraries. This is the part that distinguishes a credible tool from
marketing copy.

**L1 — verified three-way today (232 test cases):**

| Library | Cases | What it exercises |
|---------|-------|-------------------|
| base64 | 33/33 | byte/string loops, error handling |
| url | 30/30 | record types, Result, module boundaries |
| serde_json | 56/56 | recursive data, tag/enum, generics |
| regex | 45/45 | pattern matching, backtracking |
| cli_app | 32/32 | pure std text processing (wc-style) |
| trait_advanced | 10/10 | spec/trait dispatch (L1 subset) |
| tokio | 13/13 | async spawn/join, channels |

See the live [parity dashboard](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html) for the
full matrix and per-library details.

**Honest boundaries (L3 — roadmap, not yet verified):**

These are documented openly in
[known-divergences](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/known-divergences.md), not hidden:

- **Associated types** in specs — Auto's grammar has no construct yet (language gap).
- **Default method bodies that return a value** — a2r wrapping bug (void defaults work).
- **Generic spec implementations** — a2r drops the concrete type argument.
- **Bounded generic functions** (`fn f<T has Spec>`) — bound syntax + VM dispatch gap.
- **reqwest / http_client_sync parity** — needs an in-process mock-server harness.

Auto does not pretend to be finished where it isn't. The L1 list is what's
verified; the L3 list is on the roadmap, and every chapter in the tour tells
you which tier a feature is in.

## Start here

→ **[From Script to Ship — the interactive tour](/docs/script-to-ship/README)** —
six chapters, every block runnable in your browser.

→ **[Parity dashboard](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html)** — the evidence.

→ **[Script-to-Ship demos](/examples/script-to-ship-demos/)** — runnable
single-file examples (serde_json, regex, wc) you can clone and ship.

```bash
# Dev — interpret instantly, no compile
auto main.at

# Ship — transpile to Rust, then cargo build for release
auto trans --path main.at rust
```
