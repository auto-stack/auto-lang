# Chapter 3 — Types & Ownership

This is where Auto proves it isn't "Rust-like" — it's *Rust*. Structs, enums,
and the ownership/borrow vocabulary map one-to-one, and a2r emits the Rust
you'd write by hand.

<Listing file="script-to-ship/ch03-types-ownership/03_point.at" view="scriptship" caption="A struct, a borrowing reader, and an untouched owner" />

## What maps to what

| Auto | Rust (a2r output) |
|------|-------------------|
| `type Point { x int, y int }` | `struct Point { x: i32, y: i32 }` |
| `Point(3, 4)` | `Point { x: 3, y: 4 }` |
| reading fields (`p.x`) without consuming | `&p` semantics — borrow |
| `p.view` | `&p` (explicit shared borrow) |
| `p.mut` | `&mut p` (explicit mutable borrow) |
| `p.take` | move |

In the listing, `norm_sq` reads `p.x` and `p.y` but never consumes `p`, so
after the call `a` is still usable (we call `show(a)` again). Hit **Transpile
to Rust** and you'll see a2r emit a borrow-shaped Rust function that agrees.

## Why this is the load-bearing claim

A language that "looks like Rust" but quietly GC's everything, or copies on
every pass, is not Rust's script layer — it's a dialect that diverges at
ship time. Auto's ownership keywords exist precisely so that the script's
memory behavior and the shipped Rust's memory behavior are the *same* behavior.
The parity tests in `parity/libs/` exercise exactly this: the same source, run
through AutoVM and through a2r-compiled Rust, must produce identical output —
including the cases where ownership decides whether a move or a borrow happens.

## The honest boundary

Auto's VM is a 32-bit interpreter; the integer types are `i32`/`u32`/`f64`
etc., matching Rust's width. Where the VM has known boundary limitations
(e.g. user-defined structs crossing a module boundary in certain Result
shapes — see `parity/docs/known-divergences.md` DIV-URL-VM-1), the parity
libraries work around them in-source and document the workaround. Those are
VM implementation limits, not language-semantics divergences: the a2r output
is unaffected and behaves as the Rust reads.

Next: [Errors →](ch04-errors)
