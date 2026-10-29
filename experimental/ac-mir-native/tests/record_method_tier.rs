//! PLAN-881 record/text-record METHOD tier (`mn_method_`): type-bound
//! static/view methods through the SHARED lowering onto the mn pipeline
//! (escalating frontend → lower_checked_data/owned → verify → unit split →
//! CLIF/COFF → PE link → run), differentially against the AV semantic
//! baseline and the landed old HIR-native reference. Oracles are frozen
//! independently (hand-audited): R08 = 42, the owned Token row = 21.

mod common;

use common::{run_mn, tools_available};

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

/// R08 through mn: the method rows the 871 ledger tracked as
/// `gap:mn-lower-method` must compile, link and run to the frozen 42.
#[test]
fn mn_method_r08_runs() {
    if !tools_available() {
        eprintln!("skip: link tools unavailable");
        return;
    }
    let r = run_mn(R08, &[], "r08");
    assert!(r.ok, "mn request failed: {:?}", r.error);
    assert_eq!(
        r.exit_code,
        Some(42),
        "R08 exit (stderr {:?})",
        String::from_utf8_lossy(&r.stderr)
    );
}

/// Three-leg differential on R08: mn == AV == old HIR-native == 42.
#[test]
fn mn_method_r08_differential() {
    if !tools_available() {
        eprintln!("skip: link tools unavailable");
        return;
    }
    let mn = run_mn(R08, &[], "r08-diff");
    assert!(mn.ok, "mn failed: {:?}", mn.error);
    let mn_v: i32 = match mn.exit_code {
        Some(v) => v,
        None => panic!("mn exit missing: {:?}", mn.error),
    };
    let av = auto_av_probe::data_driver::request_data(None, "r08-diff", R08, "d_main", &[])
        .expect("av data request")
        .av
        .expect("av outcome");
    let av_v = match av {
        auto_av_core::contract::ExecOutcome::Success {
            value: auto_av_core::contract::Value::I32(v),
            ..
        } => v,
        other => panic!("av outcome: {other:?}"),
    };
    let workdir =
        std::env::temp_dir().join(format!("mn881-old-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let old = auto_av_probe::native_reference::build_and_run_record(
        R08,
        &[],
        "d_main",
        &workdir,
        None,
    )
    .expect("old-native record build+run");
    let _ = std::fs::remove_dir_all(&workdir);
    assert_eq!(old.exit_code, 42, "old-native exit (stderr {:?})", old.stderr);
    assert_eq!((mn_v, av_v), (42, 42), "three legs agree on 42");
}

/// Owned-profile methods through mn: static construction with a moved
/// `Str` + view receiver field read. Hand-audited: 'A中' = 4 UTF-8 bytes,
/// 4 * 4 + 5 = 21.
#[test]
fn mn_method_owned_static_and_view() {
    if !tools_available() {
        eprintln!("skip: link tools unavailable");
        return;
    }
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
    let r = run_mn(src, &[], "owned-m");
    assert!(r.ok, "mn owned methods failed: {:?}", r.error);
    assert_eq!(
        r.exit_code,
        Some(21),
        "owned static+view through mn = 21 (stderr {:?})",
        String::from_utf8_lossy(&r.stderr)
    );
}
