//! PLAN-1116 T-01 seam probes: where do the generator/stream (1089),
//! task/actor (1090), and `use.rs` foreign (1093) surfaces STOP on the
//! ordinary-language A2R chain today? Each probe walks the real chain
//! (ordinary Auto frontend → checked HIR → SemanticView → language-channel
//! lowering → fileset → rustc → run) and records the first named outcome
//! instead of panicking, so the plan can bind producers and gaps.

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_core::SemanticView;
use ace_a2x_rust::{
    assemble_language_file_set, canonical_execution_config, lower_language_program,
};
use ace_core::source::compile_language_source;

const LANG_CAP: &str = "ac.source.language.av/v1";

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Walk the chain for one case; return a one-line outcome description.
fn probe(name: &str, src: &str, caps: &[&str]) -> String {
    let caps_vec: Vec<String> = caps.iter().map(|c| c.to_string()).collect();
    let compiled = match ace_core::source::adapter::compile_language_source_prepared(
        src,
        "main.at",
        &caps_vec,
        |program, projected_source, has_instances| {
            if has_instances {
                ace_source_query::instantiate_auto_program(program, "main.at", &caps_vec)?;
            }
            if src.contains('#') {
                let mut service =
                    ace_source_cte::CtService::new(ace_source_cte::MemoryEnvelopeSink::new());
                ace_source_cte::product::prepare_auto_ct(
                    &mut service,
                    program,
                    projected_source,
                    "main.at",
                    &caps_vec,
                )
            } else {
                Ok(())
            }
        },
    ) {
        Ok(c) => c,
        Err(ds) => {
            return format!(
                "frontend-diagnostic {}",
                ds.iter()
                    .map(|d| format!("{:?}:{}:{}", d.stage, d.code, d.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
    };
    let view = match SemanticView::from_checked(&compiled.checked) {
        Ok(v) => v,
        Err(r) => return format!("view-reject {r}"),
    };
    let program = match lower_language_program(&view, "main") {
        Ok(p) => p,
        Err(r) => return format!("lower-reject {r}"),
    };
    let cfg = canonical_execution_config("debug", 16_777_216, None, false, false);
    let fs = assemble_language_file_set(
        &view,
        &program,
        &compiled.input_hash,
        "1116-t01",
        &cfg,
        "main",
    );
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2x-out/lang-1116-t01")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("a2x_case.exe");
    let mut compiler = Command::new(rustc());
    compiler
        .args(["--edition", "2021"])
        .arg("-o")
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir);
    let compile_out =
        auto_tool_execution::run_command_capture(compiler, auto_tool_execution::TreeScope::ProcessTree, None)
            .expect("spawn rustc");
    if compile_out.kind != auto_tool_execution::ExitKind::Success {
        let full = String::from_utf8_lossy(compile_out.stderr.as_ref().map(Vec::as_slice).unwrap_or(b""))
            .to_string();
        let errors: Vec<&str> = full
            .lines()
            .filter(|l| l.starts_with("error"))
            .chain(full.lines().filter(|l| l.contains("panicked")))
            .take(4)
            .collect();
        let head = if errors.is_empty() {
            full.lines().take(4).collect::<Vec<_>>().join(" | ")
        } else {
            errors.join(" | ")
        };
        return format!("rustc-fail {head}");
    }
    let mut command = Command::new(&exe);
    command.current_dir(&dir);
    let run = auto_tool_execution::run_command_capture(command, auto_tool_execution::TreeScope::ProcessTree, None)
        .expect("run exe");
    let stdout = String::from_utf8_lossy(run.stdout.as_ref().map(Vec::as_slice).unwrap_or(b""))
        .lines()
        .take(3)
        .collect::<Vec<_>>()
        .join(" | ");
    let stderr = String::from_utf8_lossy(run.stderr.as_ref().map(Vec::as_slice).unwrap_or(b""))
        .lines()
        .take(2)
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        "ran kind={:?} stdout=[{stdout}] stderr=[{stderr}]",
        run.kind
    )
}

#[test]
fn t01_probe_generator_sum() {
    // 1089 corpus: indirect-generator (exit 6 on AV).
    let src = "fn counter() ~Iter<int> { yield 1; yield 2; yield 3 }\n\
               fn make() ~Iter<int> { return counter() }\n\
               fn main() int { var n = 0; for x in make() { n = n + x }; return n }";
    println!("PROBE generator-sum => {}", probe("gen-sum", src, &[LANG_CAP]));
}

#[test]
#[ignore = "PLAN-1116 defect③: a generator finally referencing a module global awaits the \
            package-binding seam (single-file mode lacks the global scope); tracked in the plan"]
fn t01_probe_generator_close_finally() {
    // 1089 corpus: close-finally-once (exit 42 on AV).
    let src = "var closed = 0\n\
               fn values() ~Iter<int> {\n\
                try { yield 19; yield 23 } finally { closed = closed + 1 }\n\
               }\n\
               fn main() int {\n\
                for n in values() { break }\n\
                if closed == 1 { return 42 }; return 0\n\
               }";
    println!("PROBE generator-close => {}", probe("gen-close", src, &[LANG_CAP, "ac.source.modules.packages/v1", "console.stdout", "console.stderr"]));
}

#[test]
fn t01_probe_task_ask_reply() {
    // 1090 corpus: ask-reply (exit 42 on AV).
    let src = "task Q { on { n int -> { reply(n + 1) } } }\n\
               fn main() int { let h = Task.spawn(\"Q\", 1); let n = await h.ask(41); h.stop(); return n }";
    println!("PROBE task-ask => {}", probe("task-ask", src, &[LANG_CAP]));
}

#[test]
fn t01_probe_async_block() {
    // 1090 corpus: async-block-value (exit 42 on AV).
    let src = "fn main() int { let f = ~{ 40 + 2 }; return f.await }";
    println!("PROBE async-block => {}", probe("async-block", src, &[LANG_CAP]));
}

#[test]
fn t02_probe_go_once() {
    // 1090 corpus: go-once (lazy future fn; stdout "queued\n42", exit 0).
    let src = "fn work(n int) ~int { print(n); return n }\n\
               fn main() int {\n\
                let f = work(42)\n\
                print(\"queued\")\n\
                f.go\n\
                return 0\n\
               }";
    println!("PROBE go-once => {}", probe("go-once", src, &[LANG_CAP, "console.stdout", "console.stderr"]));
}

#[test]
fn t02_probe_future_list() {
    // 1090 corpus: future-list (exit 42).
    let src = "fn work(n int) ~int { return n + 1 }\n\
               fn make() List<Future<int>> { return [work(18), work(22)] }\n\
               fn main() int { let xs = make(); return xs[0].await + xs[1].await }";
    println!("PROBE future-list => {}", probe("future-list", src, &[LANG_CAP]));
}

#[test]
fn t02_probe_ordinary_go() {
    // 1090 corpus: ordinary-go (direct go; stdout "42").
    let src = "fn worker(n int) { print(n) }\nfn main() { go worker(42) }";
    println!("PROBE ordinary-go => {}", probe("ordinary-go", src, &[LANG_CAP, "console.stdout", "console.stderr"]));
}

#[test]
fn t02_probe_owned_capture() {
    // 1090 corpus: owned-capture (Str move capture; stdout "stays owned", exit 42).
    let src = "fn main() int {\n\
               let text Str = \"stays owned\"\n\
               let f = ~{\n\
                 sleep(25)\n\
                 print(text)\n\
                 42\n\
               }\n\
               return f.await\n\
               }";
    println!("PROBE owned-capture => {}", probe("owned-capture", src, &[LANG_CAP, "console.stdout", "console.stderr"]));
}

#[test]
fn t02_probe_start_default() {
    // 1090 corpus: start-default (task field init; exit 42).
    let src = "task T {\n\
               count = 40\n\
               on { n int -> { reply(count + n) } }\n\
               }\n\
               fn main() int { let h = Task.spawn(\"T\", 1); return h.ask(2).await }";
    println!("PROBE start-default => {}", probe("start-default", src, &[LANG_CAP]));
}

#[test]
fn t02_probe_handler_spawn() {
    // 1090 corpus: handler-spawn (nested spawn in handler; exit 42).
    let src = "task Leaf { on { n int -> { reply(n + 1) } } }\n\
               task Root {\n\
                on {\n\
                 n int -> {\n\
                  let child = Task.spawn(\"Leaf\", 1)\n\
                  let value = child.ask(n).await\n\
                  child.stop()\n\
                  reply(value)\n\
                 }\n\
                }\n\
               }\n\
               fn main() int { let h = Task.spawn(\"Root\", 1); return h.ask(41).await }";
    println!("PROBE handler-spawn => {}", probe("handler-spawn", src, &[LANG_CAP]));
}

#[test]
fn t02_probe_generator_corpus() {
    // 1089 corpus family (single-file rows; the module-global finally row
    // awaits the package seam — see t01_probe_generator_close_finally).
    let cases: &[(&str, &str, &str)] = &[
        ("loop-local", "fn seq() ~Iter<int> {\n var i = 0\n for i < 10 {\n  yield i * 2\n  i += 1\n  if i == 3 { break }\n  if i == 1 { continue }\n  yield i * 3\n }\n}\nfn main() int {\n var n = 0\n for e in seq() { n = n + e }\n return n + 30\n}", "value:42"),
        ("owned-string-drain", "fn words() ~Iter<Str> { yield \"20\"; yield \"22\" }\nfn main() int {\n var trace = \"\"\n for word in words() { trace = trace + word }\n if trace == \"2022\" { return 42 }; return 0\n}", "value:42"),
        ("payload-owner-early-stop", "enum Entry {\n Row(List<Str>),\n Other,\n}\nfn records() ~Iter<Entry> {\n var first = List<Str>.new()\n first.push(\"first\".clone())\n yield Entry.Row(first)\n var second = List<Str>.new()\n second.push(\"second\".clone())\n yield Entry.Row(second)\n fail(\"third-next-must-not-run\")\n}\nfn main() int {\n var visits = 0\n for entry in records() {\n  let word = is entry {\n   Entry.Row(xs) -> xs.remove(0),\n   Entry.Other -> \"\".clone(),\n  }\n  if visits == 0 && word != \"first\" { return -1 }\n  if visits == 1 && word != \"second\" { return -2 }\n  visits += 1\n  if visits == 2 { break }\n }\n return 40 + visits\n}", "value:42"),
        ("branch-forwarded-iterator", "fn seq(value int) ~Iter<int> {\n let marker = \"x\".clone()\n yield value + marker.len() - 1\n}\nfn choose(move it ~Iter<int>, yes bool) ~Iter<int> {\n if yes { return it.move }\n return seq(0)\n}\nfn main() int {\n let first = seq(42)\n var var_it = choose(first.move, true)\n return var_it.next() ?? 0\n}", "value:42"),
        ("borrow-mut-prefix", "type Tally { value int }\nfn series(mut tally Tally, limit int) ~Iter<int> {\n    tally.value = tally.value + 2\n    let n = tally.value\n    yield n\n    yield limit\n}\nfn main() int {\n    var tally = Tally(18)\n    var sum = 0\n    for n in series(tally.mut, 2) {\n        sum += n\n        if tally.value != 20 { return -1 }\n    }\n    return sum + tally.value\n}", "value:42"),
        ("borrow-before-loop-suspension", "type Number { value int }\nfn series(n Number) ~Iter<int> {\n let initial = n.value\n var i = 0\n for i < 2 {\n  yield initial + i * 2\n  i += 1\n }\n}\nfn main() int {\n let n = Number(20)\n var result = 0\n for value in series(n) { result += value }\n return result\n}", "value:42"),
        ("owned-list-items", "fn values() ~Iter<List<Str>> {\n    var first = List<Str>.new()\n    first.push(\"twenty\".clone())\n    yield first\n    var second = List<Str>.new()\n    second.push(\"twenty-two\".clone())\n    yield second\n}\nfn main() int {\n    var it = values()\n    let a = it.next()\n    let b = it.next()\n    let c = it.next()\n    let d = it.next()\n    it.close()\n    let first = is a { Some(xs) -> xs.remove(0) == \"twenty\"; None -> false }\n    let second = is b { Some(xs) -> xs.remove(0) == \"twenty-two\"; None -> false }\n    let empty = is c { Some(xs) -> false; None -> true }\n    let empty_again = is d { Some(xs) -> false; None -> true }\n    if first && second && empty && empty_again { return 42 }\n    return 0\n}", "value:42"),
        ("cancel-during-suspension", "fn seq() ~Iter<Str> {\n var retained = List<Str>.new()\n retained.push(\"twenty\".clone())\n retained.push(\"twenty-two\".clone())\n yield retained.remove(0)\n yield retained.remove(0)\n yield \"unexpected-after-cancel\".clone()\n}\nfn main() int {\n var count = 0\n for text in seq() {\n  print(text)\n  count += 1\n  if count == 2 { break }\n }\n return 42\n}", "value:42"),
    ];
    let mut failures = Vec::new();
    for (name, src, want) in cases {
        let out = probe(name, src, &[LANG_CAP, "console.stdout", "console.stderr"]);
        let got = out.lines().find(|l| l.contains("stdout=")).unwrap_or(&out);
        let ok = got.contains(want);
        println!("PROBE gen-{name} => {out}");
        if !ok {
            failures.push(format!("{name}: want {want} in {got}"));
        }
    }
    assert!(failures.is_empty(), "generator corpus failures:\n{}", failures.join("\n"));
}

#[test]
fn t02_probe_negative_rows() {
    // 1089 frozen negative rows: the shared frontend's named rejections
    // must surface identically on the A2R chain (same front door).
    let rows: &[(&str, &str, &str)] = &[
        ("yield-neighbor", "fn main() int { yield 42; return 0 }", "source.yield-outside-generator"),
        ("no-borrow-across-yield", "type Box { value int }\nfn series(box Box) ~Iter<int> {\n    yield 20\n    yield box.value\n}\nfn main() int {\n    let box = Box(22)\n    for n in series(box) { return n }\n    return 0\n}", "source.generator-borrow-param"),
        ("borrow-crosses-frame", "type Box { value int }\nfn series(box Box) ~Iter<int> { yield box.value }\nfn make() ~Iter<int> {\n    let box = Box(42)\n    return series(box)\n}\nfn main() int { return 0 }", "source.generator-borrow-escape"),
    ];
    let mut failures = Vec::new();
    for (name, src, want_code) in rows {
        let out = probe(name, src, &[LANG_CAP, "ac.source.modules.packages/v1", "console.stdout", "console.stderr"]);
        println!("PROBE neg-{name} => {out}");
        let ok = out.contains(want_code);
        if !ok {
            failures.push(format!("{name}: want code {want_code} in {out}"));
        }
    }
    assert!(failures.is_empty(), "negative row failures:\n{}", failures.join("\n"));
}

#[test]
fn t02_probe_fifo_capacity() {
    // 1090 corpus: fifo-capacity (stdout "19\n23", exit 42).
    let src = "task Echo { on { n int -> { print(n) } } }\n\
               fn main() int { let h = Task.spawn(\"Echo\", 1); h.send(19); h.send(23); h.stop(); return 42 }";
    let out = probe("fifo-capacity", src, &[LANG_CAP, "console.stdout", "console.stderr"]);
    println!("PROBE fifo-capacity => {out}");
    assert!(out.contains("value:42") && out.contains("19 | 23"), "fifo-capacity: {out}");
}

#[test]
fn t02_probe_spawn_outside_main() {
    // 1090 corpus: spawn-outside-main (exit 42).
    let src = "task Q { on { n int -> { reply(n + 1) } } }\n\
               fn request() int { let h = Task.spawn(\"Q\", 1); let n = await h.ask(41); h.stop(); return n }\n\
               fn main() int { return request() }";
    let out = probe("spawn-outside-main", src, &[LANG_CAP]);
    println!("PROBE spawn-outside-main => {out}");
    assert!(out.contains("value:42"), "spawn-outside-main: {out}");
}


#[test]
#[ignore = "PLAN-1116 family-B debt: a generator capturing a List VIEW param across             suspension needs true shared-handle aliasing (the interleaved two-generator             oracle [1,2] rules out copy-in/out); see the plan T-02 record"]
fn t02_probe_generator_listview_debt() {
    let cases: &[(&str, &str, &str)] = &[
        ("close-throws-then-terminal", "fn seq(events List<int>) ~Iter<int> {\n try { yield 40; yield 41 } finally { events.push(2); fail(\"close!\") }\n}\nfn main() int {\n var events = List<int>.new()\n var g = seq(events)\n let initial = g.next() ?? 0\n try { g.close(); return -1 } catch e { if e != \"close!\" { return -2 } }\n try { g.close() } catch e { return -3 }\n if (g.next() ?? -1) != -1 { return -4 }\n if events.len() != 1 { return -5 }\n return initial + events.get(0)\n}", "value:42"),
        ("closed-before-first-next", "fn seq(events List<int>) ~Iter<int> {\n events.push(20)\n try { yield 40; yield 41 } finally { events.push(2) }\n}\nfn main() int {\n var events = List<int>.new()\n var g = seq(events)\n g.close(); g.close()\n if (g.next() ?? -1) != -1 || events.len() != 0 { return 0 }\n return 42\n}", "value:42"),
        ("drain-finally-once", "fn seq(events List<int>) ~Iter<int> {\n try { yield 19; yield 23 } finally { events.push(9) }\n}\nfn main() int {\n var events = List<int>.new()\n var it = seq(events)\n var n = 0\n for v in it { n = n + v }\n it.close()\n let extra = it.next() ?? 0\n if events.len() != 1 || events[0] != 9 { return 0 }\n return n + extra\n}", "value:42"),
        ("early-break-outer-finally", "fn seq(events List<int>) ~Iter<Str> {\n try {\n  let before = \"kept\".clone()\n  yield before.clone()\n  yield before + \"!\"\n  yield before + \"?\"\n } finally {\n  events.push(2)\n }\n}\nfn main() int {\n var events = List<int>.new()\n var n = 0\n for word in seq(events) {\n  n += word.len()\n  if n == 9 { break }\n }\n if n == 9 && events.len() == 1 && events[0] == 2 { return 42 }\n return 0\n}", "value:42"),
        ("two-finally-producers-interleaved", "fn one(events List<int>) ~Iter<int> {\n try { yield 19; yield 99 } finally { events.push(1) }\n}\nfn two(events List<int>) ~Iter<int> {\n try { yield 23; yield 98 } finally { events.push(2) }\n}\nfn main() int {\n var events = List<int>.new()\n var a = one(events)\n var b = two(events)\n let x = a.next() ?? 0\n let y = b.next() ?? 0\n a.close()\n b.close()\n if events.len() != 2 || events[0] != 1 || events[1] != 2 { return -1 }\n return x + y\n}", "value:42"),
        ("nested-close", "fn seq(events List<int>) ~Iter<int> {\n try {\n  try { yield 42 } finally { events.push(1); fail(\"inner\") }\n } finally { events.push(2) }\n}\nfn main() int {\n var events = List<int>.new()\n var g = seq(events)\n let n = g.next() ?? 0\n try { g.close(); return -1 } catch e { if e != \"inner\" { return -2 } }\n if events.len() != 2 || events.get(0) != 1 || events.get(1) != 2 { return -3 }\n return n\n}", "value:42"),
        ("nested-close-errors-from-calls", "fn leaf(events List<int>) int { events.push(2); fail(\"inner-call\") }\nfn seq(events List<int>) ~Iter<int> {\n try {\n  try { yield 42; yield 99 } finally { leaf(events) }\n } finally { events.push(3); fail(\"outer-wins\") }\n}\nfn main() int {\n var events = List<int>.new()\n var it = seq(events)\n let n = it.next() ?? 0\n try { it.close(); return -1 }\n catch e { if e != \"outer-wins\" { return -2 } }\n it.close()\n if events.len() != 2 || events[0] != 2 || events[1] != 3 { return -3 }\n if (it.next() ?? -1) != -1 { return -4 }\n return n\n}", "value:42"),
    ];
    let mut failures = Vec::new();
    for (name, src, want) in cases {
        let out = probe(name, src, &[LANG_CAP, "console.stdout", "console.stderr"]);
        println!("PROBE debt-{name} => {out}");
        if !out.contains(want) {
            failures.push(format!("{name}: want {want} in {out}"));
        }
    }
    assert!(failures.is_empty(), "list-view debt rows still failing:
{}", failures.join("
"));
}

#[test]
fn t01_probe_use_rs_foreign() {    // 1093/1113 canonical surface: `use.rs` + a frozen provider decl.
    let src = "use.rs f0_text\n\
               fn main() int { return text_byte_len(\"A中\u{1F600}\u{0}\") }";
    println!("PROBE use-rs => {}", probe("use-rs", src, &[LANG_CAP]));
}
