# Chapter 1 — Hello, Script & Ship

This is the whole idea in one block. One Auto program, two ways to run it,
identical output.

<Listing file="script-to-ship/ch01-hello-script-ship/01_hello.at" view="scriptship" caption="The smallest closed loop: one source, two execution modes" />

## What just happened

Hit **Run in VM**. The AutoVM interprets the source directly — no compiler in
the loop, no build step. You changed the code, you hit run, you saw output.
That is the **Dev** act: iteration latency in seconds.

Now hit **Transpile to Rust**. The right pane shows the exact Rust that `a2r`
emits for this source. It is not a sketch or an approximation — it is the
output of `auto trans --path main.at rust`. Notice it reads like Rust you
would have written by hand: `fn greet(name: &str) -> String`, `for i in
1..=10`, `println!`. That is the **Ship** act: the deliverable is Rust.

If your block has **Run Both & Compare**, try it: the VM and the transpiled
Rust (compiled and run) print the same thing. That is the **Bridge** act, and
it is the part that distinguishes Auto from "Python now, rewrite in C later":
there is no rewrite, and the compiler is on the hook for agreement.

## The two commands behind the buttons

```bash
# Dev — interpret instantly, no compile
auto main.at

# Ship — transpile to Rust, then cargo build for release
auto trans --path main.at rust     # writes main.a2r.rs
```

That is the entire workflow. The rest of this tour elaborates on each act and
shows it across the Rust patterns you actually use.

## Why this matters

Python taught the world that fast iteration wins. Rust taught the world that
safety wins. The usual answer is to use both — write it in Python, then pay
the rewrite tax to move it to C/C++ or Rust. Auto refuses the premise: the
thing you iterate on *is* the thing you ship, in a language whose semantics
line up with Rust's.

Next: [AI in the Loop →](ch02-ai-in-the-loop)
