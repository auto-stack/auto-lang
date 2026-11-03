//! PLAN-925 CA-B (T-06): METHOD warm edits on the C record-text channel —
//! the narrow method fragment (`static fn ..` / `fn ..` + owner binding)
//! rides the single-body credential chain (graft keeps the retained
//! receiver/owner shape; the method def is `d_f{slot}_{name}`), with REAL
//! phase accounting: one fragment parse (never a whole-source parse), one
//! prepare/verify, exactly one lower/print and two persists for the edited
//! method; every unchanged sibling method, free function and caller keeps
//! its resident bytes (P0/IO0) — a method body edit no longer rides the
//! cold path. Declaration changes (signature) are NAMED rejections on this
//! path (D-05: declarations take the cold front end); owners never
//! cross-wire; A→B→A revisits land on the store.

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::{A2xSession, ProducerStats};
use ace_core::source::SourceProfile;
use std::path::PathBuf;

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c-rt-method")
}

fn open_session(test: &str) -> A2xSession {
    A2xSession::open(
        &out_root().join(format!("store-{}", std::process::id())).join(test),
        "a2c-rt-method",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .expect("open C rt session")
}

/// Counter with a static constructor, two view methods, a same-named free
/// function and a caller — the frozen 881/902 method shape.
const BASE: &str = "type Counter {\n    value int\n    static fn new(n int) Counter { return Counter(n) }\n    fn read() int { return .value }\n    fn add(delta int) int { return .value + delta }\n}\nfn read() int { return 7 }\nfn main() int {\n    let c Counter = Counter.new(20)\n    return c.read() * 100 + c.add(1) * 10 + read()\n}";

fn compile_run(dir: &PathBuf, files: &[(String, String)]) -> String {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("case.exe");
    ace_a2x_worker::crun::compile_c_fileset(dir, &exe).unwrap_or_else(|e| panic!("cl: {e}"));
    let run = ace_a2x_worker::crun::run_exe(&exe).expect("run");
    String::from_utf8_lossy(&run.stdout).to_string()
}

fn fragment_of<'a>(generated: &'a str, c_name: &str) -> &'a str {
    // the DEFINITION block of one C function in generated.c: the origin-
    // marked block whose definition line opens THAT function's body
    // (`<ret> <name>(<params>) {` — call sites inside other bodies are
    // indented and never end a line with `) {`)
    let header = format!(" {}(", c_name);
    generated
        .split("
// origin:")
        .find(|b| {
            b.lines()
                .any(|l| !l.starts_with(' ') && l.contains(&header) && l.ends_with(") {"))
        })
        .unwrap_or_else(|| panic!("fragment {} missing", c_name))
}

fn delta(after: &ProducerStats, before: &ProducerStats) -> (u64, u64, u64, u64, u64, u64, u64) {
    (
        after.parse - before.parse,
        after.prepare_candidate - before.prepare_candidate,
        after.verify - before.verify,
        after.lower - before.lower,
        after.print - before.print,
        after.persist - before.persist,
        after.whole_fallback - before.whole_fallback,
    )
}

/// V-07: a VIEW method body edit is a true warm single-body edit — phase
/// meters, resident identity of every other unit, real cl value.
#[test]
fn a2c_rt_view_method_body_warm_edit_phases() {
    let mut s = open_session("view_method");
    let cold = s
        .cold_assemble_profile(BASE, SourceProfile::Record, &[], "main", false)
        .expect("cold");
    let gen0 = cold
        .files
        .iter()
        .find(|(p, _)| p == "generated.c")
        .map(|(_, c)| c.clone())
        .unwrap();
    // 20*100 + 21*10 + 7 = 2217
    let out = compile_run(&out_root().join("VIEW_METHOD_0"), &cold.files);
    assert_eq!(out, "value:2217\n");
    let before = s.stats().clone();
    let warm = s
        .warm_edit_method(
            "Counter",
            "read",
            "fn read() int { return .value * 2 }",
            "main",
            false,
        )
        .expect("method warm edit");
    let after = s.stats().clone();
    let (parse, prep, verify, lower, print, persist, whole) = delta(&after, &before);
    assert_eq!(parse, 1, "exactly ONE fragment parse (no whole-source parse)");
    assert_eq!(prep, 1, "one prepare");
    assert_eq!(verify, 1, "one single-body verify");
    assert_eq!(lower, 1, "exactly the edited method lowers");
    assert_eq!(print, 1, "exactly the edited method prints");
    assert_eq!(persist, 2, "one semantic + one fragment put");
    assert_eq!(whole, 0, "a method body edit never rides the cold path now");
    let gen1 = warm
        .files
        .iter()
        .find(|(p, _)| p == "generated.c")
        .map(|(_, c)| c.clone())
        .unwrap();
    // every OTHER unit keeps its resident bytes (P0/IO0 for the unchanged)
    for c_name in ["mt_Counter_new", "mt_Counter_add", "fn_read", "fn_main"] {
        assert_eq!(
            fragment_of(&gen0, c_name),
            fragment_of(&gen1, c_name),
            "unchanged unit `{}` must keep its resident fragment",
            c_name
        );
    }
    assert_ne!(
        fragment_of(&gen0, "mt_Counter_read"),
        fragment_of(&gen1, "mt_Counter_read"),
        "the edited method re-produced"
    );
    // 40*100 + 21*10 + 7 = 4217 through real cl
    let out = compile_run(&out_root().join("VIEW_METHOD_1"), &warm.files);
    assert_eq!(out, "value:4217\n");
}

/// V-07: a STATIC method body edit takes the same warm path; the
/// same-named free function and the sibling view methods stay resident.
#[test]
fn a2c_rt_static_method_body_warm_edit() {
    let mut s = open_session("static_method");
    let cold = s
        .cold_assemble_profile(BASE, SourceProfile::Record, &[], "main", false)
        .expect("cold");
    let gen0 = cold
        .files
        .iter()
        .find(|(p, _)| p == "generated.c")
        .map(|(_, c)| c.clone())
        .unwrap();
    let before = s.stats().clone();
    let warm = s
        .warm_edit_method(
            "Counter",
            "new",
            "static fn new(n int) Counter { return Counter(n + 1) }",
            "main",
            false,
        )
        .expect("static method warm edit");
    let (parse, _, _, lower, _, _, whole) = delta(&s.stats().clone(), &before);
    assert_eq!((parse, lower, whole), (1, 1, 0));
    let gen1 = warm
        .files
        .iter()
        .find(|(p, _)| p == "generated.c")
        .map(|(_, c)| c.clone())
        .unwrap();
    for c_name in ["mt_Counter_read", "mt_Counter_add", "fn_read", "fn_main"] {
        assert_eq!(fragment_of(&gen0, c_name), fragment_of(&gen1, c_name), "{}", c_name);
    }
    // 21*100 + 22*10 + 7 = 2327
    let out = compile_run(&out_root().join("STATIC_METHOD_1"), &warm.files);
    assert_eq!(out, "value:2327\n");
}

/// V-08: a SIGNATURE change is a declaration-context change — a NAMED
/// rejection on the warm path (never a silent stale binding); a
/// non-existent owner / method is a named rejection; a same-named method
/// of ANOTHER owner never cross-wires.
#[test]
fn a2c_rt_method_decl_change_and_owner_rejections() {
    let mut s = open_session("decl_neg");
    s.cold_assemble_profile(BASE, SourceProfile::Record, &[], "main", false)
        .expect("cold");
    // signature change: the retained slot's shape is the authority
    let err = s
        .warm_edit_method(
            "Counter",
            "read",
            "fn read(k int) int { return .value + k }",
            "main",
            false,
        )
        .expect_err("signature change must reject on the warm path");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(r) => {
            assert!(r.code.starts_with("a2x."), "named rejection, got {}", r);
        }
        other => panic!("expected Reject, got {:?}", other),
    }
    // unknown owner
    let err = s
        .warm_edit_method("Gauge", "read", "fn read() int { return 1 }", "main", false)
        .expect_err("unknown owner must reject");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(r) => {
            assert!(r.detail.contains("not resident"), "got {}", r);
        }
        other => panic!("expected Reject, got {:?}", other),
    }
    // receiver-shape mismatch: editing the VIEW method `read` with a
    // `static fn` fragment is a named rejection (the graft keeps the
    // retained receiver shape)
    let err = s
        .warm_edit_method(
            "Counter",
            "read",
            "static fn read() int { return 1 }",
            "main",
            false,
        )
        .expect_err("receiver shape change must reject");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Reject(_)));

    // two owners with a same-named method: editing one leaves the other's
    // fragment byte-identical
    let two = "type Counter { value int\n  fn read() int { return .value } }\ntype Gauge { value int\n  fn read() int { return .value + 1 } }\nfn main() int {\n    let c Counter = Counter(1)\n    let g Gauge = Gauge(2)\n    return c.read() * 10 + g.read()\n}";
    let mut s2 = open_session("two_owners");
    let cold = s2
        .cold_assemble_profile(two, SourceProfile::Record, &[], "main", false)
        .expect("cold");
    let gen0 = cold
        .files
        .iter()
        .find(|(p, _)| p == "generated.c")
        .map(|(_, c)| c.clone())
        .unwrap();
    let warm = s2
        .warm_edit_method("Gauge", "read", "fn read() int { return .value + 5 }", "main", false)
        .expect("gauge edit");
    let gen1 = warm
        .files
        .iter()
        .find(|(p, _)| p == "generated.c")
        .map(|(_, c)| c.clone())
        .unwrap();
    assert_eq!(fragment_of(&gen0, "mt_Counter_read"), fragment_of(&gen1, "mt_Counter_read"));
    assert_ne!(fragment_of(&gen0, "mt_Gauge_read"), fragment_of(&gen1, "mt_Gauge_read"));
    // 1*10 + 7 = 17
    let out = compile_run(&out_root().join("TWO_OWNERS_1"), &warm.files);
    assert_eq!(out, "value:17\n");
}

/// V-08: A→B→A — revisiting a method body lands on the STORE (fragment
/// hit, zero lowering), and the caller never re-lowers across the whole
/// sequence (a callee body is not a caller dependency).
#[test]
fn a2c_rt_method_aba_store_hit_and_caller_stability() {
    let mut s = open_session("aba");
    s.cold_assemble_profile(BASE, SourceProfile::Record, &[], "main", false)
        .expect("cold");
    let b = "fn read() int { return .value * 3 }";
    let a = "fn read() int { return .value }";
    s.warm_edit_method("Counter", "read", b, "main", false)
        .expect("B");
    let before = s.stats().clone();
    s.warm_edit_method("Counter", "read", a, "main", false)
        .expect("A again");
    let after = s.stats().clone();
    assert_eq!(after.lower - before.lower, 0, "A revisit lowers nothing");
    assert_eq!(
        after.store_hit_fragment - before.store_hit_fragment,
        1,
        "A revisit is a store fragment hit"
    );
    assert_eq!(after.whole_fallback - before.whole_fallback, 0);
}

/// V-07 (含 type 源的自由函数暖片段, PLAN-904 relaxation on the C channel):
/// a free-function body edit inside a type-bearing source is a true
/// fragment edit here too.
#[test]
fn a2c_rt_free_fn_in_type_source_warm_edit() {
    let mut s = open_session("free_in_type");
    s.cold_assemble_profile(BASE, SourceProfile::Record, &[], "main", false)
        .expect("cold");
    let before = s.stats().clone();
    let warm = s
        .warm_edit("read", "fn read() int { return 9 }", "main", false)
        .expect("free fn warm edit in a type-bearing source");
    let (parse, _, _, lower, _, _, whole) = delta(&s.stats().clone(), &before);
    assert_eq!((parse, lower, whole), (1, 1, 0));
    // 20*100 + 21*10 + 9 = 2219
    let out = compile_run(&out_root().join("FREE_IN_TYPE_1"), &warm.files);
    assert_eq!(out, "value:2219\n");
}

