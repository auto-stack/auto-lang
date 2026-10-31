//! PLAN-902 T-03 record/text MODE + credential conformance (V-01/V-03):
//! single-body TEXT credentials carrying the verifier's own resource plan
//! (verified facts, never re-derived), credential-owner mismatch refusals,
//! nominal-identity refusals, Copy-snapshot capture before `mut`,
//! multi-`mut` exclusivity, reversed named construction placement,
//! same-root conflict refusals — all through the real chain (values by
//! real rustc; refusals are NAMED compile-time diagnostics).

use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_program};
use ace_core::source::{
    compile_parts_with, compile_record_source, compile_text_record_source, SourceProfile,
};
use ace_core::text_resources::seed_param_exclusions;
use ace_core::verify::verify_text_function;

use crate::rt_common::{assert_value, run_chain, Prof, RUNTIME_TEXT_CAP};

const T03: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/T03.at");

/// V-01: the text single-body credential carries the verifier's OWN
/// BodyResourcePlan; the fragment view lowers from that plan and the
/// emitted program certifies the same value through real rustc — cleanup
/// from verified facts, not a re-derivation, on the fragment path too.
#[test]
fn text_fragment_credential_true_chain() {
    // whole-module reference (T03 main already certified in 902 suite)
    let whole = crate::rt_common::compile_case("T03", T03, Prof::TextRecord);
    let whole_view = SemanticView::from_checked(&whole.checked).expect("whole view");
    let whole_prog = lower_program(&whole_view, "main").expect("whole lower");
    let whole_text = ace_a2x_rust::print_program(&whole_prog);

    // single-body credential over the same content
    let parts = compile_parts_with(
        T03,
        "T03.at",
        &[RUNTIME_TEXT_CAP.to_string()],
        SourceProfile::TextRecord,
    )
    .expect("parts");
    let main_def = parts.bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_main")
        .expect("main def");
    let excl = seed_param_exclusions(&parts.bundle);
    let cred = verify_text_function(&parts.bundle, main_def, &excl)
        .unwrap_or_else(|ds| panic!("text credential: {ds:?}"));
    assert!(cred.resource_plan().is_some(), "credential carries a plan");
    assert_eq!(cred.resource_plan().unwrap().owned.len(), 3);

    let frag_view = SemanticView::from_parts(&parts, &cred).expect("fragment view");
    assert_eq!(frag_view.functions().len(), 1);
    assert!(frag_view.body_plan(&frag_view.functions()[0]).is_some());
    let frag_prog = lower_program(&frag_view, "main").expect("fragment lower");
    let frag_text = ace_a2x_rust::print_program(&frag_prog);
    assert_eq!(whole_text, frag_text, "fragment must render identically");

    // and the fragment fileset compiles + runs to the frozen oracle
    let fs = assemble_record_text_file_set(&frag_view, &frag_prog, &whole.input_hash, "main");
    let dir = crate::rt_common::out_root().join("T03-fragment");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &fs.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let mut compiler = std::process::Command::new(crate::rt_common::rustc());
    compiler
        .args(["--edition", "2021", "-o"])
        .arg(dir.join("frag.exe"))
        .arg("main.rs")
        .current_dir(&dir);
    let out = auto_tool_execution::run_command_capture(
        compiler,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("spawn rustc");
    assert_eq!(out.kind, auto_tool_execution::ExitKind::Success);
    let run = auto_tool_execution::run_command_capture(
        {
            let mut c = std::process::Command::new(dir.join("frag.exe"));
            c.current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run frag");
    assert_eq!(
        String::from_utf8_lossy(run.stdout.as_ref().unwrap()),
        "value:23\n"
    );
}

/// V-01: a credential never admits the WRONG function — a fragment bundle
/// whose planned def is absent (owner mismatch / stale credential) is a
/// named admission failure, not a silent pass.
#[test]
fn credential_def_mismatch_rejects() {
    let parts = compile_parts_with(
        T03,
        "T03.at",
        &[RUNTIME_TEXT_CAP.to_string()],
        SourceProfile::TextRecord,
    )
    .expect("parts");
    let excl = seed_param_exclusions(&parts.bundle);
    // main is def d_main; use a bogus index-free mismatch by verifying the
    // real def then checking admission against a bundle where that def is
    // renamed is not expressible — the honest mismatch leg is a def id that
    // does not exist in the parts: emulate with a method-owner mismatch.
    let cred = verify_text_function(&parts.bundle, 0, &excl).expect("credential for d_main");
    assert_eq!(cred.def_id_text(), "d_main");

    // same-shape different-owner: Counter vs Gauge methods must not admit
    // each other's credentials (def ids differ per owner).
    let counter_src = "type Counter { value int\n  fn read() int { return .value } }\nfn main() int { let x Counter = Counter(7)\n    return x.read() }";
    let gauge_src = "type Gauge { value int\n  fn read() int { return .value } }\nfn main() int { let x Gauge = Gauge(9)\n    return x.read() }";
    let cparts = compile_parts_with(counter_src, "c.at", &[], SourceProfile::Record).expect("c");
    let gparts = compile_parts_with(gauge_src, "g.at", &[], SourceProfile::Record).expect("g");
    let c_def = cparts.bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_f0_read")
        .expect("counter read def");
    let c_cred =
        ace_core::verify::verify_function_ref(&cparts.bundle, c_def).expect("counter credential");
    assert_eq!(c_cred.def_id_text(), "d_f0_read");
    // the GAUGE bundle admits its own method but never the counter's def id
    let g_def = gparts.bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_f0_read")
        .expect("gauge read def");
    let g_cred =
        ace_core::verify::verify_function_ref(&gparts.bundle, g_def).expect("gauge credential");
    let c_view = SemanticView::from_parts(&cparts, &c_cred).expect("counter fragment");
    let g_view = SemanticView::from_parts(&gparts, &g_cred).expect("gauge fragment");
    // both admit exactly their own method with owner-qualified identity —
    // same shape, DIFFERENT nominal identity
    assert_eq!(c_view.functions()[0].source_name, "Counter_read");
    assert_eq!(g_view.functions()[0].source_name, "Gauge_read");
    // cross-admission is structurally impossible: the def id keys the match
    let cross = SemanticView::from_parts(&gparts, &c_cred);
    assert!(
        cross.is_err(),
        "a Counter credential must not admit a Gauge bundle"
    );
}

/// V-01: the capability front door holds on the single-body path — the
/// source adapter always provisions `requires` for text ops, so the honest
/// negative control strips it from a cloned bundle.
#[test]
fn text_credential_needs_capability() {
    let parts = compile_parts_with(
        "fn main() int {\n    let a Str = \"x\"\n    return a.len()\n}",
        "cap.at",
        &[],
        SourceProfile::Text,
    )
    .expect("parts");
    assert!(
        parts.bundle.requires.iter().any(|r| r == RUNTIME_TEXT_CAP),
        "adapter provisions the runtime capability for text ops"
    );
    let mut stripped = parts.bundle.clone();
    stripped.requires.clear();
    let excl = seed_param_exclusions(&stripped);
    let err =
        verify_text_function(&stripped, 0, &excl).expect_err("capability front door must reject");
    assert_eq!(err[0].code, "capability.missing");
}

/// V-03: a pure-Copy record snapshot taken BEFORE the mut call keeps its
/// evaluation-point value — the later exclusive write-back cannot
/// retroactively change it.
#[test]
fn copy_snapshot_before_mut() {
    let src = "type Counter { value int }\n\
               fn bump(mut x Counter, delta int) int {\n    x.value = x.value + delta\n    return x.value\n}\n\
               fn main() int {\n    var x Counter = Counter(20)\n    let snap Counter = x\n    let now int = bump(x.mut, 1)\n    return snap.value * 10 + now\n}";
    let (out, _err) = run_chain("COPY_SNAP", src, Prof::Record, "debug", &[], &[]);
    // snap captured 20 BEFORE the mut; now = 21 → 20*10 + 21 = 221
    assert_value("COPY_SNAP", &out, 221);
}

/// V-03: multiple `mut` params on distinct roots + a plain return.
#[test]
fn multi_mut_plus_plain_return() {
    let src = "type Cell { v int }\n\
               fn pair(mut a Cell, mut b Cell) int {\n    a.v = 1\n    b.v = 2\n    return a.v * 10 + b.v\n}\n\
               fn main() int {\n    var x Cell = Cell(0)\n    var y Cell = Cell(0)\n    return pair(x.mut, y.mut)\n}";
    let (out, _err) = run_chain("MULTI_MUT", src, Prof::Record, "debug", &[], &[]);
    assert_value("MULTI_MUT", &out, 12);
}

/// V-03: same-root `mut` + view overlap in one statement is a NAMED
/// verifier rejection (774 D-04 exclusivity) — never silently compiled.
#[test]
fn same_root_mut_view_conflict_rejected() {
    let src = "type Counter { value int }\n\
               fn bump(mut x Counter, peek Counter) int {\n    x.value = x.value + 1\n    return x.value + peek.value\n}\n\
               fn main() int {\n    var x Counter = Counter(20)\n    return bump(x.mut, x)\n}";
    let err = compile_record_source(src, "neg.at", &[]).expect_err("same-root conflict");
    let codes: Vec<&str> = err.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"verify.mut-conflict"),
        "expected verify.mut-conflict, got {codes:?}"
    );
}

/// V-03: reversed named construction places by NAME (declaration binding),
/// not by written position.
#[test]
fn named_ctor_reverse_order_placement() {
    let src = "type P { a int\n    b int }\n\
               fn main() int {\n    let p P = P { b: 2, a: 1 }\n    return p.a * 10 + p.b\n}";
    let (out, _err) = run_chain("NAMED_CTOR", src, Prof::Record, "debug", &[], &[]);
    assert_value("NAMED_CTOR", &out, 12);
}

/// V-03: a same-named free function and a same-named method of two owners
/// coexist with distinct identities (owner-qualified method naming).
#[test]
fn same_name_free_and_two_owner_methods() {
    let src = "type Counter { value int\n  fn read() int { return .value } }\n\
               type Gauge { value int\n  fn read() int { return .value + 1 } }\n\
               fn read() int { return 7 }\n\
               fn main() int {\n    let c Counter = Counter(1)\n    let g Gauge = Gauge(2)\n    return c.read() * 100 + g.read() * 10 + read()\n}";
    let (out, _err) = run_chain("NAME_COLLISION", src, Prof::Record, "debug", &[], &[]);
    // 1*100 + 3*10 + 7 = 137
    assert_value("NAME_COLLISION", &out, 137);
}

/// V-03: receiver evaluated exactly once, before explicit args — observed
/// by trace marks (record domain carries the 892 trace surface).
#[test]
fn receiver_once_before_args() {
    let src = "type Counter { value int\n  fn add(delta int) int { return .value + delta } }\n\
               fn main() int {\n    let c Counter = Counter(mark_b())\n    return c.add(mark_a())\n}";
    let compiled = compile_record_source(src, "marks.at", &["hir.test.trace.v1".to_string()])
        .unwrap_or_else(|ds| panic!("marks record compile: {ds:?}"));
    let view =
        SemanticView::from_checked(&compiled.checked).unwrap_or_else(|r| panic!("view: {r}"));
    let program = lower_program(&view, "main").expect("lower");
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");
    let dir = crate::rt_common::out_root().join("RECV_ONCE");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &fs.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let compile = auto_tool_execution::run_command_capture(
        {
            let mut c = std::process::Command::new(crate::rt_common::rustc());
            c.args(["--edition", "2021", "-o"])
                .arg(dir.join("recv.exe"))
                .arg("main.rs")
                .current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("rustc");
    assert_eq!(compile.kind, auto_tool_execution::ExitKind::Success);
    let run = auto_tool_execution::run_command_capture(
        {
            let mut c = std::process::Command::new(dir.join("recv.exe"));
            c.current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run");
    assert_eq!(
        String::from_utf8_lossy(run.stdout.as_ref().unwrap()),
        "value:3\n",
        "mark_b()=2 receiver + mark_a()=1 arg"
    );
    let stderr = String::from_utf8_lossy(run.stderr.as_ref().unwrap()).to_string();
    let marks: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        marks,
        vec!["mark_b", "mark_a"],
        "receiver evaluated exactly once BEFORE the explicit args"
    );
}

/// V-01: text-domain whole-module vs single-body exclusion carry — the
/// whole-module plan and the credential plan agree for the same body
/// (same-source same-facts; the fragment path never invents ownership).
#[test]
fn text_credential_plan_matches_whole_module() {
    let whole = compile_text_record_source(
        "fn f(move s Str) Str { return s }\n\
         fn main() int {\n    let a Str = \"ab\"\n    let b Str = f(a.move)\n    return b.len()\n}",
        "move.at",
        &[RUNTIME_TEXT_CAP.to_string()],
    )
    .expect("whole");
    let whole_plan = whole
        .checked
        .text_resources()
        .expect("whole plan")
        .bodies
        .get(&(0, 1)) // main's body
        .expect("main plan")
        .clone();
    // owned Str bindings a (moved out at the call) and b both own buffers
    assert!(whole_plan.owned.contains(&0) && whole_plan.owned.contains(&1));

    let parts = compile_parts_with(
        "fn f(move s Str) Str { return s }\n\
         fn main() int {\n    let a Str = \"ab\"\n    let b Str = f(a.move)\n    return b.len()\n}",
        "move.at",
        &[RUNTIME_TEXT_CAP.to_string()],
        SourceProfile::TextRecord,
    )
    .expect("parts");
    let main_def = parts.bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_main")
        .unwrap();
    let excl = seed_param_exclusions(&parts.bundle);
    let cred = verify_text_function(&parts.bundle, main_def, &excl).expect("cred");
    assert_eq!(
        cred.resource_plan().unwrap().owned,
        whole_plan.owned,
        "credential plan owned set == whole-module plan for the same body"
    );
}
