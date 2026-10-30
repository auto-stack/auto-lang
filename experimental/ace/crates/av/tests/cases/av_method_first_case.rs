//! PLAN-881 T-02 first-case probes (`av_method_881`): record-domain
//! type-bound methods through the SHARED lowering — whole-module and
//! single-function-unit paths onto AV, data and owned profiles, plus one
//! REAL single-method-body frontend edit/atomic publish through the 828
//! session machinery.
//!
//! Oracles are frozen independently of the implementation under test:
//! R08 (ACE-M1) = 42; the owned Token row = 57 (782 v12 microprogram
//! arithmetic, hand-audited); the edit turns 42 -> 43 with every other
//! unit's production identity unchanged.

use ace_av::data_driver::request_data;
use ace_av::data_session::DataIncrementalSession;
use ace_av::owned_driver::request_owned;
use ace_av_core::contract::Value;

const R08: &str = "\
type Counter {
    value int
    static fn new(n int) Counter { return Counter(n) }
    fn read() int { return .value }
}
fn main() int {
    let x Counter = Counter.new(42)
    return x.read()
}
";

/// R08 through the whole-module data path (AV semantic baseline).
#[test]
fn method_r08_whole_module_av_data() {
    let r = request_data(None, "r08", R08, "d_main", &[])
        .expect("R08 lowers, verifies, admits and runs");
    let av = r.av.expect("av outcome present");
    assert!(
        matches!(
            av,
            ace_av_core::contract::ExecOutcome::Success {
                value: Value::I32(42),
                ..
            }
        ),
        "R08 must return 42, got {av:?}"
    );
}

/// A legal method that is NEVER CALLED must not sink the module either
/// (the old Pass-1 scan rejected any Method def, called or not).
#[test]
fn method_uncalled_legal_method_still_lowers() {
    let src = "\
type P {
    v int
    static fn mk(n int) P { return P { v: n } }
    fn unused_helper() int { return .v * 2 }
}
fn main() int {
    let p = P.mk(21)
    return p.v
}
";
    let r = request_data(None, "uncalled", src, "d_main", &[]).expect("uncalled method lowers");
    let av = r.av.expect("av outcome present");
    assert!(
        matches!(
            av,
            ace_av_core::contract::ExecOutcome::Success {
                value: Value::I32(21),
                ..
            }
        ),
        "uncalled method must not block the module, got {av:?}"
    );
}

/// R08 through the UNIT path: warm up the data session (every def —
/// methods included — cold-lowered as its own unit, snapshot-linked,
/// one cold run), then read the run back.
#[test]
fn method_r08_unit_path_session() {
    let sess = DataIncrementalSession::warmup(R08, "d_main", &[])
        .expect("session warms up with method units");
    let v = sess.run().expect("linked units run");
    assert_eq!(v, Value::I32(42), "R08 through units must be 42");
    // The method units are residents under their owner-based keys.
    let order = sess.fn_order();
    assert!(
        order.contains(&"m.Counter.new".to_string()),
        "static method unit resident, order={order:?}"
    );
    assert!(
        order.contains(&"m.Counter.read".to_string()),
        "view method unit resident, order={order:?}"
    );
}

/// One REAL single-method-body frontend edit + atomic publish: `read`
/// returns `.value + 1`; the run flips 42 -> 43 and the structural gate
/// proves only the method's own unit was produced.
#[test]
fn method_method_body_edit_atomic_publish() {
    let mut sess = DataIncrementalSession::warmup(R08, "d_main", &[]).expect("session warms up");
    let edited = R08.replace(
        "fn read() int { return .value }",
        "fn read() int { return .value + 1 }",
    );
    assert_ne!(edited, R08, "edit actually changes the source");
    let report = sess
        .edit_body(&edited, "m.Counter.read")
        .expect("method body edit succeeds");
    assert_eq!(report.changed_def, "m.Counter.read");
    let v = sess.run().expect("post-edit run");
    assert_eq!(v, Value::I32(43), "edited method returns .value + 1");
    // Only the edited method's lower counter moved.
    let read_lowered = report
        .lowered
        .get("lower:m.Counter.read")
        .copied()
        .unwrap_or(0);
    let new_lowered = report
        .lowered
        .get("lower:m.Counter.new")
        .copied()
        .unwrap_or(0);
    let main_lowered = report.lowered.get("lower:d_main").copied().unwrap_or(0);
    assert_eq!(read_lowered, 2, "read re-lowered exactly once more");
    assert_eq!(new_lowered, 1, "static method unit NOT re-lowered");
    assert_eq!(main_lowered, 1, "main unit NOT re-lowered");
}

/// Owned (text-record) profile: static construction with a moved `Str`,
/// view receiver reading a field — the 782-shaped method surface through
/// the SHARED owned lowering. Hand-audited oracle: 'A中' is 4 UTF-8
/// bytes, so score = 4 * 4 + 5 = 21.
#[test]
fn method_owned_static_and_view_methods() {
    let src = "\
type Tok {
    label Str
    n int
    static fn make(move s Str, k int) Tok { return Tok { label: s.move, n: k } }
    fn score() int { return .label.len_bytes() * 4 + .n }
}
fn main() int {
    let t = Tok.make(\"A中\", 5)
    return t.score()
}
";
    let r = request_owned(None, "tok", src, "d_main", &[]).expect("owned methods lower");
    let av = r.av.clone().expect("av outcome present");
    assert!(
        matches!(
            av,
            ace_av_core::contract::ExecOutcome::Success {
                value: Value::I32(21),
                ..
            }
        ),
        "owned static+view methods: len_bytes('A中')=4, 4*4+5=21, got {av:?}"
    );
}
