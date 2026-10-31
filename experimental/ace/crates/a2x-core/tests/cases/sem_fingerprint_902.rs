//! PLAN-902 V-07: the semantic fingerprint's dependency-invalidations.
//!
//! Every "differs" leg is a change the §5.4 key inputs name explicitly —
//! callee interface (signature + modes + kind), nominal record identity
//! (name + ordered field shapes), transfer flags. Every "stable" leg is a
//! pure layout/ordering change the plan forbids from invalidating (def
//! table reorder, unrelated additions, unused locals). Scalar fingerprints
//! keep their frozen bytes (the worker's 892 corpus/golden and store tests
//! pin those); all legs here run on the 902 domains.

use ace_a2x_core::{semantic_fingerprint, SemanticView};
use ace_core::source::{
    compile_parts_with, compile_record_source, compile_text_record_source, SourceProfile,
};

const RUNTIME_TEXT_CAP: &str = "ac.runtime.text.v1";

fn fp_record(src: &str, focus: &str) -> String {
    let compiled = compile_record_source(src, "fp.at", &[])
        .unwrap_or_else(|ds| panic!("record compile: {ds:?}"));
    let view =
        SemanticView::from_checked(&compiled.checked).unwrap_or_else(|r| panic!("view: {r}"));
    let f = view
        .functions()
        .iter()
        .find(|f| f.source_name == focus)
        .unwrap_or_else(|| panic!("`{focus}` not in view"));
    semantic_fingerprint(&view, f).expect("fingerprint")
}

fn fp_text_record(src: &str, focus: &str) -> String {
    let compiled = compile_text_record_source(src, "fp.at", &[RUNTIME_TEXT_CAP.to_string()])
        .unwrap_or_else(|ds| panic!("text-record compile: {ds:?}"));
    let view =
        SemanticView::from_checked(&compiled.checked).unwrap_or_else(|r| panic!("view: {r}"));
    let f = view
        .functions()
        .iter()
        .find(|f| f.source_name == focus)
        .unwrap_or_else(|| panic!("`{focus}` not in view"));
    semantic_fingerprint(&view, f).expect("fingerprint")
}

const CALLER_BASE: &str = "type Counter { value int }\n\
    fn bump(mut x Counter, d int) int { x.value = x.value + d\n    return x.value }\n\
    fn main() int {\n    var c Counter = Counter(20)\n    let r int = bump(c.mut, 1)\n    return r + c.value\n}";

#[test]
fn callee_signature_change_invalidates_caller() {
    let base = fp_record(CALLER_BASE, "main");
    // callee gains a parameter (call updated) — the CALLER's fingerprint
    // must move even though main's own text shape is analogous
    let changed = fp_record(
        "type Counter { value int }\n\
         fn bump(mut x Counter, d int, e int) int { x.value = x.value + d + e\n    return x.value }\n\
         fn main() int {\n    var c Counter = Counter(20)\n    let r int = bump(c.mut, 1, 0)\n    return r + c.value\n}",
        "main",
    );
    assert_ne!(
        base, changed,
        "callee signature change must invalidate the caller"
    );
}

#[test]
fn callee_mode_change_invalidates_caller() {
    let base = fp_record(CALLER_BASE, "main");
    // callee's record param degrades to a view (call site marker removed)
    let changed = fp_record(
        "type Counter { value int }\n\
         fn bump(x Counter, d int) int { return x.value + d }\n\
         fn main() int {\n    var c Counter = Counter(20)\n    let r int = bump(c, 1)\n    return r + c.value\n}",
        "main",
    );
    assert_ne!(
        base, changed,
        "callee param-mode change must invalidate the caller"
    );
}

#[test]
fn record_shape_and_name_changes_invalidate() {
    let base = fp_record(CALLER_BASE, "main");
    // field TYPE change
    let field_ty = fp_record(
        "type Counter { on bool }\n\
         fn bump(mut x Counter, d int) int { return 1 }\n\
         fn main() int {\n    var c Counter = Counter(true)\n    let r int = bump(c.mut, 1)\n    return r\n}",
        "main",
    );
    assert_ne!(base, field_ty, "record field type change must invalidate");
    // field NAME change (same types)
    let field_name = fp_record(
        "type Counter { count int }\n\
         fn bump(mut x Counter, d int) int { x.count = x.count + d\n    return x.count }\n\
         fn main() int {\n    var c Counter = Counter(20)\n    let r int = bump(c.mut, 1)\n    return r + c.count\n}",
        "main",
    );
    assert_ne!(base, field_name, "record field name change must invalidate");
    // record NAME change, identical shape everywhere (nominal identity)
    let renamed = fp_record(
        "type Gauge { value int }\n\
         fn bump(mut x Gauge, d int) int { x.value = x.value + d\n    return x.value }\n\
         fn main() int {\n    var c Gauge = Gauge(20)\n    let r int = bump(c.mut, 1)\n    return r + c.value\n}",
        "main",
    );
    assert_ne!(
        base, renamed,
        "record rename must invalidate (nominal identity)"
    );
}

#[test]
fn method_vs_free_function_kind_marker() {
    // a method and a same-shaped free function with identical bodies are
    // different semantic objects (the kind leads the 902 sig line)
    let as_method = fp_record(
        "type P { v int\n    static fn make(n int) P { return P(n) } }\n\
         fn main() int {\n    let p P = P.make(3)\n    return p.v\n}",
        "P_make",
    );
    let as_free = fp_record(
        "type P { v int }\n\
         fn make(n int) P { return P(n) }\n\
         fn main() int {\n    let p P = make(3)\n    return p.v\n}",
        "make",
    );
    assert_ne!(as_method, as_free, "method vs free function must differ");
}

#[test]
fn layout_reorders_are_stable() {
    let base = fp_record(CALLER_BASE, "main");
    // adding an UNRELATED function before main (def/table reorder)
    let grown = fp_record(
        "type Counter { value int }\n\
         fn extra(n int) int { return n }\n\
         fn bump(mut x Counter, d int) int { x.value = x.value + d\n    return x.value }\n\
         fn main() int {\n    var c Counter = Counter(20)\n    let r int = bump(c.mut, 1)\n    return r + c.value\n}",
        "main",
    );
    assert_eq!(base, grown, "unrelated additions must not invalidate");
}

#[test]
fn transfer_flag_change_invalidates() {
    let base = fp_text_record(
        "fn f(move s Str) Str { return s }\n\
         fn main() int {\n    let a Str = \"ab\"\n    let b Str = f(a.move)\n    return b.len()\n}",
        "main",
    );
    // no transfer: the callee takes a view instead (both programs legal;
    // the caller's semantics differ — ownership vs borrow)
    let changed = fp_text_record(
        "fn f(s Str) Str { return s.clone() }\n\
         fn main() int {\n    let a Str = \"ab\"\n    let b Str = f(a)\n    return b.len()\n}",
        "main",
    );
    assert_ne!(base, changed, "transfer-mode change must invalidate");
}

#[test]
fn credential_context_binding_rejects_same_shape_foreign_bundle() {
    // V-01/V-07 join: a whole-module verified Counter context must not
    // accept a same-shape Gauge credential carrier at fragment admission.
    let counter = "type Counter { value int\n  fn read() int { return .value } }\n\
                   fn main() int { let x Counter = Counter(7)\n    return x.read() }";
    let gauge = "type Gauge { value int\n  fn read() int { return .value } }\n\
                 fn main() int { let x Gauge = Gauge(9)\n    return x.read() }";
    let cparts = compile_parts_with(counter, "c.at", &[], SourceProfile::Record).unwrap();
    let gparts = compile_parts_with(gauge, "g.at", &[], SourceProfile::Record).unwrap();
    let c_def = cparts.bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_f0_read")
        .unwrap();
    let c_cred = ace_core::verify::verify_function_ref(&cparts.bundle, c_def).expect("credential");
    let err = SemanticView::from_parts(&gparts, &c_cred)
        .expect_err("foreign same-shape bundle must reject the credential");
    assert_eq!(err.code, "a2x.credential");
}
