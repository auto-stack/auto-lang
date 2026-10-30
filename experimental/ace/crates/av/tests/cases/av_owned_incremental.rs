//! PLAN-821 V-13 incremental (R1-F07/R2-F07'/R3-F07): the four sub-cases
//! over the REAL owned install channel — `ace_av::owned_session::
//! OwnedFunctionSession`. A PURE single-body edit rides the REAL D828
//! resident-function frontend (one fn-item parse + `prepare_candidate_
//! document` re-check/emission against the retained environment — the
//! unchanged units' parse/recheck/emit production work is ZERO); an
//! interface/context edit takes the whole-module channel where every unit
//! re-frontends. The FORMAL `incremental_gate` is channel-aware and
//! reddens a real whole frontend on a body edit. The unit-level
//! stale-link refusal (V-13(3)) replays alongside through the raw D828
//! interfaces.

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute_program, CancelFlag, ExecLimits};
use ace_av_core::contract::ExecOutcome;
use ace_av_core::function_code::FunctionCode;
use ace_av_core::program_snapshot::ProgramSnapshot;
use ace_av_core::{abc_verify, mir_verify};
use std::sync::Arc;

const THREE_FN_SRC: &str = "\
type Note { body Str; n int }
fn bump(move u Note) Note {
    return Note { body: \"B\", n: u.n + 1 }
}
fn plain(move a Note) Note {
    return Note { body: \"C\", n: a.n + 10 }
}
fn main() int {
    let a Note = Note { body: \"A\", n: 1 }
    let b Note = bump(a.move)
    let c Note = plain(b.move)
    return c.n * 100 + c.body.len()
}
";

/// Byte-range edit from a literal (must match exactly once).
fn edit_of(src: &str, from: &str, to: &str) -> (u64, u64, Vec<u8>) {
    let hits = src.matches(from).count();
    assert_eq!(hits, 1, "discriminating literal `{from}` ({hits} hits)");
    let pos = src.find(from).unwrap() as u64;
    (pos, pos + from.len() as u64, to.as_bytes().to_vec())
}

/// Capture the gate's BEFORE state: production, resident snapshot,
/// frontend count.
fn before(
    s: &ace_av::owned_session::OwnedFunctionSession,
) -> (
    std::collections::BTreeMap<String, ace_av::owned_session::Production>,
    ace_av::owned_session::GateSnapshot,
    u32,
) {
    (s.production(), s.gate_snapshot(), s.frontends())
}
fn gate(
    s: &ace_av::owned_session::OwnedFunctionSession,
    b: &(
        std::collections::BTreeMap<String, ace_av::owned_session::Production>,
        ace_av::owned_session::GateSnapshot,
        u32,
    ),
    expected: &[&str],
) -> Result<(), String> {
    s.incremental_gate(&b.0, &b.1, b.2, expected)
}

/// V-13 (2): two PURE body edits through the real session — each edit
/// rides the SINGLE-BODY channel (the D828 resident-function frontend):
/// ZERO whole-module frontend compiles, the edited unit alone advances
/// its frontend production (parse/recheck/emit), the untouched units'
/// production is UNCHANGED (zero parse/recheck/emit work), and the formal
/// gate is green at each step with objects and generations preserved.
#[test]
fn av_owned_v13_two_fn_local_body_updates() {
    let mut s = ace_av::owned_session::OwnedFunctionSession::warmup(THREE_FN_SRC, "d_main", &[])
        .expect("warmup");
    assert_eq!(s.frontends, 1, "cold warm-up = ONE whole-module frontend");
    assert_eq!(s.run_i32(), 1201);
    let main_before = s.code_of("d_main");
    let plain_before = s.code_of("d_plain");

    // ---- edit 1: bump's body (+1 -> +2). Only bump re-produces; NO
    // whole-module frontend runs (the D828 single-body channel).
    let (a, b, ins) = edit_of(THREE_FN_SRC, "n: u.n + 1", "n: u.n + 2");
    let bg = before(&s);
    let invalidated = s.edit(a, b, &ins).expect("edit installs");
    assert_eq!(
        invalidated,
        vec!["d_bump".to_string()],
        "automatic detection"
    );
    assert_eq!(
        s.last_channel(),
        ace_av::owned_session::EditChannel::SingleBody,
        "a pure body edit rides the single-function channel"
    );
    gate(&s, &bg, &["d_bump"]).expect("the formal gate is green on the incremental set");
    assert_eq!(
        s.frontends, 1,
        "a single-body edit runs ZERO whole-module frontends"
    );
    let prod = s.production();
    assert_eq!(
        (
            prod["d_main"].parse,
            prod["d_main"].recheck,
            prod["d_main"].emit
        ),
        (1, 1, 1),
        "d_main's frontend production is the COLD round only (zero body-edit work)"
    );
    assert_eq!(
        (
            prod["d_bump"].parse,
            prod["d_bump"].recheck,
            prod["d_bump"].emit
        ),
        (2, 2, 2),
        "d_bump's frontend production advanced exactly once"
    );
    assert!(
        Arc::ptr_eq(&s.code_of("d_main"), &main_before),
        "main's resident object is reused"
    );
    assert!(
        Arc::ptr_eq(&s.code_of("d_plain"), &plain_before),
        "plain's resident object is reused"
    );
    assert_eq!(s.code_of("d_main").generation(), 1);
    assert_eq!(s.code_of("d_plain").generation(), 1);
    assert_eq!(s.code_of("d_bump").generation(), 2);
    let (out, trace) = s.run();
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(1301),
            ..
        }
    ));
    assert_eq!(trace.arena.as_ref().unwrap().live(), 0);

    // ---- edit 2: main's body (*100 -> *10). Only main re-produces.
    let src1 = THREE_FN_SRC.replace("n: u.n + 1", "n: u.n + 2");
    let (a, b, ins) = edit_of(&src1, "c.n * 100", "c.n * 10");
    let bg = before(&s);
    let invalidated = s.edit(a, b, &ins).expect("edit installs");
    assert_eq!(invalidated, vec!["d_main".to_string()]);
    gate(&s, &bg, &["d_main"]).expect("gate green on main-only");
    assert_eq!(s.frontends, 1, "still zero whole-module frontends");
    assert_eq!(s.run_i32(), 131);
}

/// R3-F07 permanent counterexample: a main-only body edit in a
/// two-function module — the untouched function's frontend production
/// stays at the cold round (zero parse/recheck/emit work), no
/// whole-module frontend runs, and the SAME gate reddens both lying
/// expectations: claiming the whole set invalidated (a whole ride) or
/// expecting the wrong unit.
#[test]
fn av_owned_r3_main_only_body_edit_zero_untouched_frontend_work() {
    const SRC: &str = "type Note { body Str; n int }\nfn read(move p Note) int { return p.n }\nfn main() int {\nlet a Note = Note { body: \"A\", n: 7 }\nreturn read(a.move)\n}\n";
    let mut s =
        ace_av::owned_session::OwnedFunctionSession::warmup(SRC, "d_main", &[]).expect("warmup");
    assert_eq!(s.run_i32(), 7);
    let read_before = s.code_of("d_read");
    let bg = before(&s);
    let pos = SRC.find("n: 7").unwrap() as u64;
    let invalidated = s
        .edit(pos, pos + 4, b"n: 8")
        .expect("the main-only body edit installs");
    assert_eq!(invalidated, vec!["d_main".to_string()]);
    assert_eq!(
        s.last_channel(),
        ace_av::owned_session::EditChannel::SingleBody
    );
    assert_eq!(s.frontends, 1, "no whole-module frontend ran");
    gate(&s, &bg, &["d_main"]).expect("the honest expectation is green");
    assert!(Arc::ptr_eq(&s.code_of("d_read"), &read_before));
    let prod = s.production();
    assert_eq!(
        (
            prod["d_read"].parse,
            prod["d_read"].recheck,
            prod["d_read"].emit
        ),
        (1, 1, 1),
        "the untouched function's frontend production is the COLD round only"
    );
    assert_eq!(s.run_i32(), 8);

    // Counter-control A: claiming the WHOLE set was invalidated (the
    // whole-ride expectation) must redden — d_read produced nothing.
    let err = gate(&s, &bg, &["d_main", "d_read"])
        .expect_err("an over-claiming expectation reddens the same gate");
    assert!(
        err.contains("produced NOTHING"),
        "the gate names the unproduced unit: {err}"
    );
}

/// V-13 (3) session face: an INCONSISTENT intermediate edit is refused
/// by the frontend (the state keeps the prior revision and still runs);
/// a CONSISTENT signature edit changes the module's type context — the
/// automatic invalidation set is then the WHOLE module (every unit's
/// module-wide import table staled), the whole-module channel runs (ONE
/// whole frontend) and the gate is green expecting exactly that set.
#[test]
fn av_owned_v13_signature_edits_refused_or_whole_context() {
    let mut s = ace_av::owned_session::OwnedFunctionSession::warmup(THREE_FN_SRC, "d_main", &[])
        .expect("warmup");
    assert_eq!(s.run_i32(), 1201);

    // (a) The bare signature edit alone leaves bump's body referring to
    // `u.n` on a Str param — the FRONTEND refuses the intermediate; the
    // session keeps revision 1 and still runs. (The signature drift is
    // the retained environment's own interface gate → the whole channel;
    // its frontend rejects the inconsistent intermediate.)
    let (a, b, i) = edit_of(
        THREE_FN_SRC,
        "fn bump(move u Note) Note {",
        "fn bump(move u Str) Note {",
    );
    assert!(
        s.edit(a, b, &i).is_err(),
        "the inconsistent intermediate refuses"
    );
    assert_eq!(s.revision(), 1, "a refused edit keeps the revision");
    assert_eq!(s.run_i32(), 1201, "the prior snapshot still runs");

    // (b) The consistent retyping as ONE ATOMIC BATCH (no single-site
    // intermediate typechecks): signature + body + call site together —
    // TWO declarations met, the whole-module channel owns it.
    let (e_sig, e_sig_end, i_sig) = edit_of(
        THREE_FN_SRC,
        "fn bump(move u Note) Note {",
        "fn bump(move u Str) Note {",
    );
    let (e_body, e_body_end, i_body) = edit_of(
        THREE_FN_SRC,
        "return Note { body: \"B\", n: u.n + 1 }",
        "return Note { body: u.move, n: 1 }",
    );
    let (e_call, e_call_end, i_call) = edit_of(
        THREE_FN_SRC,
        "let a Note = Note { body: \"A\", n: 1 }
    let b Note = bump(a.move)",
        "let f Str = \"X\"
    let b Note = bump(f.move)",
    );
    let bg = before(&s);
    let inv = s
        .edit_batch(&[
            (e_sig, e_sig_end, i_sig),
            (e_body, e_body_end, i_body),
            (e_call, e_call_end, i_call),
        ])
        .expect("the atomic retyping installs");
    assert_eq!(
        inv,
        vec![
            "d_bump".to_string(),
            "d_main".to_string(),
            "d_plain".to_string(),
        ],
        "a type-context change invalidates the WHOLE module (module-wide import tables)"
    );
    assert_eq!(
        s.last_channel(),
        ace_av::owned_session::EditChannel::WholeModule,
        "an interface edit rides the whole-module channel"
    );
    gate(&s, &bg, &["d_bump", "d_main", "d_plain"])
        .expect("the gate is green on the precisely-whole set");
    // bump("X") -> Note{body:"X",n:1}; plain -> Note{body:"C",n:11};
    // 11*100 + len("X") = 1101.
    assert_eq!(
        s.run_i32(),
        1101,
        "the re-linked program runs the new interface"
    );
}

/// V-13 (4): a FORCED whole recompute is CAUGHT by the formal gate —
/// with an incremental expectation it reddens; only expecting the whole
/// set passes.
#[test]
fn av_owned_v13_forced_whole_recompute_reddens_the_gate() {
    let mut s = ace_av::owned_session::OwnedFunctionSession::warmup(THREE_FN_SRC, "d_main", &[])
        .expect("warmup");
    assert_eq!(s.run_i32(), 1201);
    // A normal body edit first: gate green on the incremental set.
    let (a, b, i) = edit_of(THREE_FN_SRC, "n: u.n + 1", "n: u.n + 2");
    let bg = before(&s);
    s.edit(a, b, &i).expect("edit");
    gate(&s, &bg, &["d_bump"]).expect("incremental is green");
    // The forced whole path: production advances EVERYWHERE.
    let bg = before(&s);
    s.force_whole_recompute().expect("forced whole runs");
    let err = gate(&s, &bg, &["d_bump"]).expect_err("the formal gate CATCHES the whole recompute");
    assert!(
        err.contains("outside the invalidated set"),
        "the gate names the leakage: {err}"
    );
    gate(&s, &bg, &["d_bump", "d_main", "d_plain"])
        .expect("expecting the whole set is green (a declared recompute is legal)");
    assert_eq!(s.run_i32(), 1301, "the program still runs correctly");
}

/// V-13 (3) unit face: a stale caller unit does not LINK against a new
/// signature (raw D828 interfaces — the same refusal the session-level
/// front prevents up-stream).
#[test]
fn av_owned_v13_stale_caller_unit_link_refused() {
    let build = |src: &str| -> std::collections::BTreeMap<String, Arc<FunctionCode>> {
        let compiled = ace_core::source::compile_text_record_source(src, "u.at", &[]).unwrap();
        let bundle = compiled.checked.bundle();
        let module = bundle.modules.first().unwrap();
        let snap = ace_av_core::contract::fnv1a64(src.as_bytes());
        let mut out = std::collections::BTreeMap::new();
        for d in &module.defs {
            if matches!(d.kind, ace_core::hir::DefKind::Function { .. }) {
                let u = ace_lowering::lower_function_owned_unit(&bundle, &d.id_text, &[], snap)
                    .unwrap();
                mir_verify::verify_mir(&u).unwrap();
                let wire = encode_abc(&u).unwrap();
                let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &u.owner).unwrap();
                let _ = decode_abc(&wire.to_flat());
                out.insert(
                    d.id_text.clone(),
                    Arc::new(FunctionCode::admit_unit(loaded, 1).unwrap()),
                );
            }
        }
        out
    };
    let base = build(THREE_FN_SRC);
    let code: Vec<Arc<FunctionCode>> = base.values().cloned().collect();
    ProgramSnapshot::link(code, "d_main", 1).expect("the consistent set links");

    // ONE new-signature bump unit mixed into the OLD set: the stale
    // `fn:bump` imports (main's, plain's — module-wide tables) fail the
    // checked link, never a silent cross-identity reuse.
    let new_src = THREE_FN_SRC
        .replace("fn bump(move u Note) Note {", "fn bump(move u Str) Note {")
        .replace(
            "return Note { body: \"B\", n: u.n + 1 }",
            "return Note { body: u.move, n: 1 }",
        )
        .replace(
            "let a Note = Note { body: \"A\", n: 1 }
    let b Note = bump(a.move)",
            "let f Str = \"X\"
    let b Note = bump(f.move)",
        );
    let new_bump = build(&new_src).get("d_bump").unwrap().clone();
    let stale: Vec<Arc<FunctionCode>> = base
        .iter()
        .map(|(k, v)| {
            if k == "d_bump" {
                new_bump.clone()
            } else {
                v.clone()
            }
        })
        .collect();
    let err =
        ProgramSnapshot::link(stale, "d_main", 2).expect_err("the stale caller must not link");
    assert!(
        err.to_lowercase().contains("signature"),
        "the refusal names the signature axis: {err}"
    );
}

/// V-13 (1): the shared scalar baseline — the 828 formal-session campaign
/// replays in the script's incremental scope; this face pins that the
/// owned landing left the scalar incremental core runnable.
#[test]
fn av_owned_v13_shared_scalar_session_still_warms() {
    let src = "fn main() int {\n    return 6\n}\n";
    let mut s = ace_av::session::warmup(src, "d_main", &[]).expect("scalar warmup");
    let (out, _) = s.run(&ExecLimits::default(), &CancelFlag::new(false));
    let _ = &mut s;
    assert!(
        matches!(out, ExecOutcome::Success { .. }),
        "the scalar session still runs: {out:?}"
    );
    let _ = execute_program;
}

/// R4-F07 permanent counterexamples: CONSECUTIVE variable-length edits —
/// the declaration coordinates must follow the CURRENT source (the
/// edited function's end and every later declaration shift by the edit's
/// delta at publish), in BOTH directions and across functions.
#[test]
fn av_owned_r4_consecutive_variable_length_edits() {
    const SRC: &str = "type Note { body Str; n int }\nfn read(move p Note) int { return p.n }\nfn main() int {\nlet a Note = Note { body: \"A\", n: 7 }\nreturn read(a.move)\n}\n";
    // Grow then edit the SAME function again (positive delta; the second
    // edit addresses CURRENT-source coordinates).
    let mut s = ace_av::owned_session::OwnedFunctionSession::warmup(SRC, "d_main", &[]).unwrap();
    let p = SRC.find("n: 7").unwrap() as u64;
    s.edit(p, p + 4, b"n: 1234").expect("grow edit installs");
    assert_eq!(s.run_i32(), 1234);
    let src1 = SRC.replacen("n: 7", "n: 1234", 1);
    let p = src1.find("n: 1234").unwrap() as u64;
    let bg = before(&s);
    let inv = s
        .edit(p, p + 7, b"n: 8")
        .expect("the second edit addresses the CURRENT source");
    assert_eq!(inv, vec!["d_main".to_string()]);
    assert_eq!(
        s.last_channel(),
        ace_av::owned_session::EditChannel::SingleBody
    );
    gate(&s, &bg, &["d_main"]).expect("the gate stays green on the shifted coordinates");
    assert_eq!(s.run_i32(), 8);

    // Shrink control: a shorter replacement then another edit of the same fn.
    let src2 = src1.replacen("n: 1234", "n: 8", 1);
    let p = src2.find("n: 8").unwrap() as u64;
    s.edit(p, p + 4, b"n: 12").expect("grow again");
    assert_eq!(s.run_i32(), 12);

    // An EARLIER function's length change shifts a LATER function's
    // coordinates; the later edit must still route the single-body path.
    let mut s = ace_av::owned_session::OwnedFunctionSession::warmup(SRC, "d_main", &[]).unwrap();
    let p = SRC.find("return p.n").unwrap() as u64;
    s.edit(p, p + 10, b"return p.n + 100")
        .expect("the earlier fn grows");
    assert_eq!(s.run_i32(), 107);
    let src1 = SRC.replacen("return p.n", "return p.n + 100", 1);
    let p = src1.find("n: 7").unwrap() as u64;
    let bg = before(&s);
    let inv = s
        .edit(p, p + 4, b"n: 8")
        .expect("the LATER fn's coordinates shifted with the earlier edit");
    assert_eq!(inv, vec!["d_main".to_string()]);
    assert_eq!(
        s.last_channel(),
        ace_av::owned_session::EditChannel::SingleBody,
        "no silent whole-module fallback after a length change"
    );
    gate(&s, &bg, &["d_main"]).expect("gate green");
    assert_eq!(s.run_i32(), 108);

    // A rename touches TWO declarations (decl head + call site) — the
    // whole-module channel owns it, and the rebuilt coordinates serve
    // the NEXT pure body edit on the single-body path again.
    let mut s = ace_av::owned_session::OwnedFunctionSession::warmup(SRC, "d_main", &[]).unwrap();
    let (a1, a2) = (
        SRC.find("read(move").unwrap() as u64,
        SRC.find("return read(").unwrap() as u64,
    );
    s.edit_batch(&[
        (a1, a1 + 4, b"redd".to_vec()),
        (a2 + 7, a2 + 11, b"redd".to_vec()),
    ])
    .expect("the consistent rename installs via the whole-module channel");
    assert_eq!(
        s.last_channel(),
        ace_av::owned_session::EditChannel::WholeModule
    );
    assert_eq!(s.run_i32(), 7);
    // After that whole-module round the fast path is rebuilt and still
    // serves pure body edits with CURRENT coordinates.
    let src1 =
        SRC.replacen("read(move", "redd(move", 1)
            .replacen("return read(", "return redd(", 1);
    let p = src1.find("n: 7").unwrap() as u64;
    let inv = s
        .edit(p, p + 4, b"n: 9")
        .expect("body edit after the whole round");
    assert_eq!(inv, vec!["d_main".to_string()]);
    assert_eq!(
        s.last_channel(),
        ace_av::owned_session::EditChannel::SingleBody
    );
    assert_eq!(s.run_i32(), 9);
}
