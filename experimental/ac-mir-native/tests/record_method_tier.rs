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
    let workdir = std::env::temp_dir().join(format!("mn881-old-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let old =
        auto_av_probe::native_reference::build_and_run_record(R08, &[], "d_main", &workdir, None)
            .expect("old-native record build+run");
    let _ = std::fs::remove_dir_all(&workdir);
    assert_eq!(
        old.exit_code, 42,
        "old-native exit (stderr {:?})",
        old.stderr
    );
    assert_eq!((mn_v, av_v), (42, 42), "three legs agree on 42");
}

/// V-09 P-02 through mn: view receiver + explicit mut record param —
/// the by-value t_MutRet write-back ABI compiles and runs natively.
#[test]
fn mn_method_p02_view_receiver_mut_param() {
    if !tools_available() {
        eprintln!("skip: link tools unavailable");
        return;
    }
    let src = "type P { v int
    fn mix(mut q P, d int) int { q.v = q.v + d; return .v * 10 + q.v }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(7)
    var q = P.mk(2)
    let r = p.mix(q.mut, 3)
    if p.v != 7 { return 90 + p.v }
    if q.v != 5 { return 80 + q.v }
    return r
}
";
    let r = run_mn(src, &[], "p02-mn");
    assert!(r.ok, "mn failed: {:?}", r.error);
    assert_eq!(
        r.exit_code,
        Some(75),
        "P-02 = 75 (stderr {:?})",
        String::from_utf8_lossy(&r.stderr)
    );
}

/// V-09 P-05 through mn: TWO mut params — wb binds exactly a/b natively.
#[test]
fn mn_method_p05_two_muts_wb_exact() {
    if !tools_available() {
        eprintln!("skip: link tools unavailable");
        return;
    }
    let src = "type P { v int
    static fn mixed(ro P, mut a P, k int, mut b P) int {
        a.v = a.v + k;
        b.v = b.v + 2 * k;
        return ro.v * 100 + a.v * 10 + b.v
    }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var ro = P.mk(1)
    var a = P.mk(2)
    var b = P.mk(3)
    let r = P.mixed(ro, a.mut, 1, b.mut)
    let diff = ro.v * 100 + a.v * 10 + b.v - r
    if diff != 0 { return 60 }
    if ro.v != 1 { return 70 + ro.v }
    return r
}
";
    let r = run_mn(src, &[], "p05-mn");
    assert!(r.ok, "mn failed: {:?}", r.error);
    assert_eq!(
        r.exit_code,
        Some(135),
        "P-05 = 135 (stderr {:?})",
        String::from_utf8_lossy(&r.stderr)
    );
}

/// V-12 (mn leg): old/new products coexist by content — a method body
/// edit flips the exit, and the ORIGINAL source still resolves to its
/// OWN cached product (the store keys by content fingerprint, never by
/// position).
#[test]
fn mn_method_v12_old_and_new_products() {
    if !tools_available() {
        eprintln!("skip: link tools unavailable");
        return;
    }
    let root = std::env::temp_dir().join(format!("mn881-v12-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = ac_mn::driver::MnDriver::with_store(&root, "v12").expect("store");
    let run = |d: &mut ac_mn::driver::MnDriver, src: &str, tag: &str| {
        let wd = root.join(tag);
        let r = d.request(src, "d_main", &[], &wd, None);
        r
    };
    let r1 = run(&mut d, R08, "r1");
    assert!(
        r1.ok && r1.exit_code == Some(42),
        "first run 42: {:?}",
        r1.error
    );
    let edited = R08.replace(
        "fn read() int { return .value }",
        "fn read() int { return .value + 1 }",
    );
    let r2 = run(&mut d, &edited, "r2");
    assert!(
        r2.ok && r2.exit_code == Some(43),
        "edited run 43: {:?}",
        r2.error
    );
    // The ORIGINAL source re-runs to 42 — its cached products survived
    // the edit round untouched (no positional keying, no overwrite).
    let r3 = run(&mut d, R08, "r3");
    assert!(
        r3.ok && r3.exit_code == Some(42),
        "original re-runs 42: {:?}",
        r3.error
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// V-03 through mn: the 782 Token microprogram (methods + clone + moves).
#[test]
fn mn_method_token_microprogram() {
    if !tools_available() {
        eprintln!("skip: link tools unavailable");
        return;
    }
    let src = r#"type Token {
    text Str
    kind int
    line int
    at int
    len int
    fn is_ident() bool { return .kind == 1 }
    static fn ident(move text Str, line int, at int) Token {
        let n int = text.len()
        return Token { text: text.move, kind: 1, line: line, at: at, len: n }
    }
}
fn weigh(move t Token) int { return t.len + t.line + t.at }
fn main() int {
    let s Str = "keyword"
    let a Token = Token.ident(s.move, 3, 12)
    if a.is_ident() == false { return 100 }
    if a.len != 7 { return 101 }
    var b Token = a.clone()
    b.at = 40
    if a.at != 12 { return 102 }
    let w int = weigh(b.move)
    return w + a.text.len()
}
"#;
    let r = run_mn(src, &[], "tok-mn");
    assert!(r.ok, "mn failed: {:?}", r.error);
    assert_eq!(
        r.exit_code,
        Some(57),
        "Token = 57 (stderr {:?})",
        String::from_utf8_lossy(&r.stderr)
    );
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
