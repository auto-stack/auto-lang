# Chapter 2 — AI in the Loop

The "Auto as Rust's script layer" story is built around AI doing the writing.
This chapter is about *why script mode is the right substrate for that* —
and it's not marketing, it's latency.

<Listing file="script-to-ship/ch02-ai-in-the-loop/02_wordcount.at" view="scriptship" caption="A small self-checking program — the shape of an AI turn" />

## The loop an AI actually runs

When an AI writes code, it doesn't write once and stop. It writes, runs,
reads the output, notices the edge case it missed, rewrites, runs again.
That loop is the unit of productivity. Its cost is the time per iteration.

- **Writing Rust directly**: every turn pays the compile tax. For a non-trivial
  crate that's tens of seconds to minutes, *per AI turn*. The AI spends more
  time waiting on `cargo build` than thinking.
- **Writing Auto**: the AutoVM interprets instantly — no compile step. The AI
  hits `/api/run`, gets stdout back in milliseconds, and corrects immediately.

Edit the block above: change the `sample` string, or add a new tricky input,
and hit **Run in VM**. That sub-second round-trip is the whole point. The AI
gets to be wrong many times, fast, because being wrong is cheap.

## Why "script" doesn't mean "Python"

Python is also fast to iterate on. So why not just have the AI write Python
and rewrite in Rust later? Because that rewrite is the expensive part — a
whole second project where an AI has to translate dynamic, untyped, GC'd
semantics into static, ownership-tracked, zero-cost-abstraction Rust. The two
languages don't agree on what a program *means*, so the translation is a design
problem, not a mechanical one.

Auto agrees with Rust on the meaning. The `count_words` function above has the
same types, the same control flow, the same integer semantics as the Rust a2r
will emit from it. Hit **Transpile to Rust** and look: there is no rewrite left
to do. The iteration substrate and the shipping substrate are the same language.

## The honest part

Script mode tolerates AI imperfection precisely *because* the code is
disposable mid-iteration: a wrong attempt costs a sub-second rerun, nothing is
committed. What survives is the final Auto source, and that source is what a2r
turns into auditable Rust. The AI is not trusted with the release — the
transpiler and the Rust compiler are. The AI is trusted with the prototype,
where fast failure is a feature.

Next: [Types & Ownership →](ch03-types-ownership)
