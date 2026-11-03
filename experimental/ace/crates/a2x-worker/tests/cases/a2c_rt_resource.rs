//! PLAN-925 CA-B (T-05): the C record-text RESOURCE closure —
//! verifier-planned cleanup at every exit (loops/branches/short-circuit),
//! deep-clone precommit atomicity under an exhaustive per-allocation
//! fault matrix, replace atomicity, independent backing and the
//! same-process retry — all against the real C runtime ledger.
//!
//! The in-process probe TU `#include`s generated.c so it shares the
//! runtime's static allocation ledger and fault knob: it faults every
//! allocation of a nested-record clone (1..N), checks the typed trap, the
//! partial recovery balance, the untouched source, N+1, the independent
//! backing, the cleared-fault retry, and the mut-param replace commit —
//! the oracle the plan requires instead of "re-run the program and infer".

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;
use std::path::PathBuf;
use std::process::Command;

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c-rt-res")
}

fn open_session(test: &str) -> A2xSession {
    A2xSession::open(
        &out_root()
            .join(format!("store-{}", std::process::id()))
            .join(test),
        "a2c-rt-res",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .expect("open C rt session")
}

fn materialize(s: &mut A2xSession, name: &str, src: &str, prof: SourceProfile) -> PathBuf {
    let asm = s
        .cold_assemble_profile(src, prof, &[], "main", false)
        .unwrap_or_else(|e| panic!("assembly {name}: {e:?}"));
    let dir = out_root().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    dir
}

fn run_exe(exe: &PathBuf, dir: &PathBuf, env: &[(&str, &str)]) -> (String, String, bool) {
    let mut command = Command::new(exe);
    command.current_dir(dir);
    for (k, v) in env {
        command.env(k, v);
    }
    let run = auto_tool_execution::run_command_capture(
        command,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run exe");
    (
        String::from_utf8_lossy(run.stdout.as_ref().expect("stdout")).to_string(),
        String::from_utf8_lossy(run.stderr.as_ref().expect("stderr")).to_string(),
        run.kind == auto_tool_execution::ExitKind::Success,
    )
}

/// (alloc_count, free_count) from an A2X_RT_EVENTS ledger.
fn ledger(stderr: &str) -> (u64, u64) {
    let mut allocs = 0u64;
    let mut frees = 0u64;
    for line in stderr.lines().filter(|l| l.starts_with("rt:")) {
        if line.starts_with("rt:alloc:") {
            allocs += 1;
        } else if line.starts_with("rt:free:") {
            frees += 1;
        }
    }
    (allocs, frees)
}

fn run_program(s: &mut A2xSession, name: &str, src: &str, prof: SourceProfile) -> (String, String) {
    let dir = materialize(s, name, src, prof);
    let exe = dir.join("case.exe");
    ace_a2x_worker::crun::compile_c_fileset(&dir, &exe)
        .unwrap_or_else(|e| panic!("cl {name}: {e}"));
    let (out, err, ok) = run_exe(&exe, &dir, &[("A2X_RT_EVENTS", "1")]);
    assert!(ok, "wrapper exit non-zero for {name}");
    (out, err)
}

const PROBE_C: &str = r##"#include "generated.c"

static int failures = 0;
#define CHECK(cond, msg) do { if (!(cond)) { printf("FAIL:%s\n", msg); failures++; } else { printf("ok:%s\n", msg); } } while (0)

static a2x_str_view v_of(const char* lit) {
    a2x_str_view v; v.data = (const uint8_t*)lit; v.len = strlen(lit); return v;
}
static a2x_str mk(const char* lit) {
    a2x_stro_result r = a2x_str_clone(v_of(lit));
    if (r.trap != A2X_TRAP_NONE) { printf("FAIL:mk\n"); failures++; }
    return r.value;
}
static int bytes_eq(a2x_str s, const char* lit) {
    a2x_str_view sv; sv.data = s.data; sv.len = s.len;
    return a2x_str_eq(sv, v_of(lit));
}

static int probe_main(void) {
    long fault;
    a2x_rec_Wrap w;
    memset(&w, 0, sizeof w);
    /* source record: 3 owned buffers (head "aa", inner.label "t", tail "bb") */
    w.f_head = mk("aa"); w.f_inner.f_label = mk("t"); w.f_tail = mk("bb");

    /* deep clone = 3 allocations in declaration order: head, inner.label, tail */
    for (fault = 1; fault <= 3; fault++) {
        long ord0 = a2x_rt_alloc_ordinal, free0 = a2x_rt_free_count;
        a2x_res_a2x_rec_Wrap r;
        a2x_rt_config();
        a2x_rt_cfg_fault = ord0 + fault;
        r = a2x_rt_clone_a2x_rec_Wrap(&w);
        CHECK(r.trap == A2X_TRAP_RESOURCE, "typed Resource trap at the faulted allocation");
        CHECK(a2x_rt_alloc_ordinal - ord0 == fault, "attempts reached exactly the fault point");
        CHECK(a2x_rt_free_count - free0 == fault - 1, "partial buffers fully recovered (balance)");
        CHECK(bytes_eq(w.f_head, "aa") && bytes_eq(w.f_inner.f_label, "t") && bytes_eq(w.f_tail, "bb"),
              "source record intact at the fault point");
    }
    /* N+1: no fault fires; the clone owns INDEPENDENT backing */
    {
        long ord0 = a2x_rt_alloc_ordinal;
        a2x_res_a2x_rec_Wrap r;
        a2x_rt_cfg_fault = ord0 + 4;
        r = a2x_rt_clone_a2x_rec_Wrap(&w);
        CHECK(r.trap == A2X_TRAP_NONE, "N+1 does not fault");
        CHECK(a2x_rt_alloc_ordinal - ord0 == 3, "exactly the clone's allocations");
        CHECK(r.value.f_head.data != w.f_head.data, "clone head has its own buffer");
        a2x_str_free(&r.value.f_head);
        CHECK(bytes_eq(w.f_head, "aa"), "source readable after freeing the clone's head");
        a2x_rt_free_a2x_rec_Wrap(&r.value);
    }
    /* clear the fault and RETRY in the same process */
    {
        a2x_res_a2x_rec_Wrap r;
        a2x_rt_cfg_fault = 0;
        r = a2x_rt_clone_a2x_rec_Wrap(&w);
        CHECK(r.trap == A2X_TRAP_NONE, "same-process retry after clearing the fault");
        CHECK(bytes_eq(r.value.f_head, "aa") && bytes_eq(r.value.f_inner.f_label, "t") && bytes_eq(r.value.f_tail, "bb"),
              "retry clone carries the source bytes");
        a2x_rt_free_a2x_rec_Wrap(&r.value);
    }
    /* replace atomicity through the LOWERED mut-param function: fault the
       RHS concat -> the OLD head buffer survives untouched and nothing
       dies; clearing the fault and retrying commits the replacement */
    {
        long free0 = a2x_rt_free_count, ord0 = a2x_rt_alloc_ordinal;
        a2x_result t, ok;
        a2x_rt_cfg_fault = ord0 + 1;
        t = fn_grow(&w);
        CHECK(t.trap == A2X_TRAP_RESOURCE, "grow traps at the faulted concat");
        CHECK(a2x_rt_free_count - free0 == 0, "nothing died on the faulted replace");
        CHECK(bytes_eq(w.f_head, "aa"), "old buffer alive and untouched (precommit)");
        a2x_rt_cfg_fault = 0;
        ok = fn_grow(&w);
        CHECK(ok.trap == A2X_TRAP_NONE && ok.value == 302, "retry commits: aa! (3)*100 + bb (2)");
        CHECK(bytes_eq(w.f_head, "aa!"), "writeback visible after the commit");
        CHECK(a2x_rt_free_count - free0 == 1, "exactly the old head died at commit");
        a2x_rt_free_a2x_rec_Wrap(&w);
    }
    return failures;
}

int main(void) {
    int f;
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
    f = probe_main();
    if (f == 0) printf("probe:PASS\n"); else printf("probe:FAIL(%d)\n", f);
    return f == 0 ? 0 : 1;
}
"##;

/// V-04: the in-process deep-clone/replace fault matrix (see the module
/// doc). The probe shares the runtime ledger with the generated code.
#[test]
fn a2c_rt_deep_clone_fault_matrix_in_process() {
    let src = "type Inner {\n    label Str\n}\ntype Wrap {\n    head Str\n    inner Inner\n    tail Str\n}\nfn grow(mut w Wrap) int {\n    w.head = w.head + \"!\"\n    return w.head.len() * 100 + w.tail.len()\n}\nfn main() int {\n    var w Wrap = Wrap { head: \"aa\", inner: Inner { label: \"t\" }, tail: \"bb\" }\n    return grow(w.mut)\n}";
    let mut s = open_session("fault_matrix");
    let dir = materialize(&mut s, "FAULT_MATRIX", src, SourceProfile::TextRecord);
    // the probe needs the wrapper's includes beside generated.c
    let probe = format!(
        "#include <fcntl.h>\n#include <io.h>\n#include <stdio.h>\n#include <string.h>\n{}",
        PROBE_C
    );
    std::fs::write(dir.join("probe.c"), probe).unwrap();
    let exe = dir.join("probe.exe");
    ace_a2x_worker::crun::compile_c_standalone(&dir.join("probe.c"), &exe)
        .unwrap_or_else(|e| panic!("probe compile: {e}"));
    let (out, _err, ok) = run_exe(&exe, &dir, &[]);
    assert!(
        ok && out.contains("probe:PASS") && !out.contains("FAIL"),
        "probe findings (exit ok={ok}):\n{out}"
    );
    // the whole program also certifies the frozen value through main.c
    let main_exe = dir.join("case.exe");
    ace_a2x_worker::crun::compile_c_fileset(&dir, &main_exe).expect("cl fileset");
    let (out, err, _) = run_exe(&main_exe, &dir, &[("A2X_RT_EVENTS", "1")]);
    assert_eq!(out, "value:302\n");
    let (a, f) = ledger(&err);
    assert_eq!(a, f, "program ledger balances (no leak / no double free)");
}

/// V-05: every control-flow exit closes its planned buffers — loops with
/// break/continue, returns inside branches, a short-circuit side owning a
/// temp. The event ledger must BALANCE (allocs == frees) and the value
/// oracles pin the semantics.
#[test]
fn a2c_rt_exit_paths_ledger_balanced() {
    let mut s = open_session("exit_paths");
    let (out, err) = run_program(
        &mut s,
        "EXIT_LOOP",
        "fn main() int {\n    var total int = 0\n    var i int = 0\n    for true {\n        let piece Str = \"ab\"\n        i = i + 1\n        if i == 2 {\n            continue\n        }\n        total = total + piece.len()\n        if i == 4 {\n            break\n        }\n    }\n    return total * 10 + i\n}",
        SourceProfile::Text,
    );
    assert_eq!(out, "value:64\n", "EXIT_LOOP");
    let (a, f) = ledger(&err);
    assert_eq!(
        (a, f),
        (4, 4),
        "every iteration buffer freed on continue/fall-through/break"
    );
    let (out, err) = run_program(
        &mut s,
        "EXIT_BRANCH_RETURN",
        "fn pick(n int) int {\n    let a Str = \"abc\"\n    let b Str = \"de\"\n    if n > 1 {\n        return a.len() * 10\n    }\n    return b.len()\n}\nfn main() int {\n    return pick(2) + pick(1)\n}",
        SourceProfile::Text,
    );
    assert_eq!(out, "value:32\n", "EXIT_BRANCH_RETURN");
    let (a, f) = ledger(&err);
    assert_eq!((a, f), (4, 4), "branch returns free both planned buffers");
    let (out, err) = run_program(
        &mut s,
        "EXIT_SHORT_CIRCUIT",
        "fn main() int {\n    let a Str = \"xy\"\n    let hit bool = a.to(str) == \"x\" || (a + \"z\").len() == 3\n    if hit {\n        return 1\n    }\n    return a.len()\n}",
        SourceProfile::Text,
    );
    assert_eq!(out, "value:1\n", "EXIT_SHORT_CIRCUIT");
    let (a, f) = ledger(&err);
    assert_eq!(
        (a, f),
        (2, 2),
        "the short-circuit side's owned temp freed with its side"
    );
}
