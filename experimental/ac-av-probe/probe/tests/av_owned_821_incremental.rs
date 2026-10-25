//! PLAN-821 V-13 incremental (R1-F07): the four sub-cases over the REAL
//! owned install channel — `auto_av_probe::owned_session::
//! OwnedFunctionSession` (one frontend compile per revision, automatic
//! HIR-identity edit detection, real production counters at the real
//! production points, and the FORMAL `incremental_gate` degradation
//! detector). The unit-level stale-link refusal (V-13(3)) replays
//! alongside through the raw D828 interfaces.

use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::ExecOutcome;
use auto_av_core::function_code::FunctionCode;
use auto_av_core::program_snapshot::ProgramSnapshot;
use auto_av_core::{abc_verify, mir_verify};
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

/// V-13 (2): two PURE body edits through the real session — the edit
/// channel AUTOMATICALLY invalidates only the edited function; the
/// formal gate is green at each step; untouched residents keep their
/// objects and generations; both revisions run exact oracles with a
/// clean owned ledger.
#[test]
fn av_owned_821_v13_two_fn_local_body_updates() {
    let mut s =
        auto_av_probe::owned_session::OwnedFunctionSession::warmup(THREE_FN_SRC, "d_main", &[])
            .expect("warmup");
    assert_eq!(s.frontends, 1, "cold warm-up = ONE frontend compile");
    assert_eq!(s.run_i32(), 1201);
    let main_before = s.code_of("d_main");
    let plain_before = s.code_of("d_plain");

    // ---- edit 1: bump's body (+1 -> +2). Only bump re-produces.
    let (a, b, ins) = edit_of(THREE_FN_SRC, "n: u.n + 1", "n: u.n + 2");
    let before = s.production();
    let invalidated = s.edit(a, b, &ins).expect("edit installs");
    assert_eq!(
        invalidated,
        vec!["d_bump".to_string()],
        "automatic detection"
    );
    s.incremental_gate(&before, &["d_bump"])
        .expect("the formal gate is green on the incremental set");
    assert_eq!(s.frontends, 2, "one frontend compile per revision");
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
            value: auto_av_core::contract::Value::I32(1301),
            ..
        }
    ));
    assert_eq!(trace.arena.as_ref().unwrap().live(), 0);

    // ---- edit 2: main's body (*100 -> *10). Only main re-produces.
    let src1 = THREE_FN_SRC.replace("n: u.n + 1", "n: u.n + 2");
    let (a, b, ins) = edit_of(&src1, "c.n * 100", "c.n * 10");
    let before = s.production();
    let invalidated = s.edit(a, b, &ins).expect("edit installs");
    assert_eq!(invalidated, vec!["d_main".to_string()]);
    s.incremental_gate(&before, &["d_main"])
        .expect("gate green on main-only");
    assert_eq!(s.run_i32(), 131);
}

/// V-13 (3) session face: an INCONSISTENT intermediate edit is refused
/// by the frontend (the state keeps the prior revision and still runs);
/// a CONSISTENT signature edit changes the module's type context — the
/// automatic invalidation set is then the WHOLE module (every unit's
/// module-wide import table staled), and the gate is green expecting
/// exactly that set.
#[test]
fn av_owned_821_v13_signature_edits_refused_or_whole_context() {
    let mut s =
        auto_av_probe::owned_session::OwnedFunctionSession::warmup(THREE_FN_SRC, "d_main", &[])
            .expect("warmup");
    assert_eq!(s.run_i32(), 1201);

    // (a) The bare signature edit alone leaves bump's body referring to
    // `u.n` on a Str param — the FRONTEND refuses the intermediate; the
    // session keeps revision 1 and still runs. (The precise-refusal
    // front.)
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
    // intermediate typechecks): signature + body + call site together.
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
    let before = s.production();
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
    s.incremental_gate(&before, &["d_bump", "d_main", "d_plain"])
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
fn av_owned_821_v13_forced_whole_recompute_reddens_the_gate() {
    let mut s =
        auto_av_probe::owned_session::OwnedFunctionSession::warmup(THREE_FN_SRC, "d_main", &[])
            .expect("warmup");
    assert_eq!(s.run_i32(), 1201);
    // A normal body edit first: gate green on the incremental set.
    let (a, b, i) = edit_of(THREE_FN_SRC, "n: u.n + 1", "n: u.n + 2");
    let before = s.production();
    s.edit(a, b, &i).expect("edit");
    s.incremental_gate(&before, &["d_bump"])
        .expect("incremental is green");
    // The forced whole path: production advances EVERYWHERE.
    let before = s.production();
    s.force_whole_recompute().expect("forced whole runs");
    let err = s
        .incremental_gate(&before, &["d_bump"])
        .expect_err("the formal gate CATCHES the whole recompute");
    assert!(
        err.contains("outside the invalidated set"),
        "the gate names the leakage: {err}"
    );
    s.incremental_gate(&before, &["d_bump", "d_main", "d_plain"])
        .expect("expecting the whole set is green (a declared recompute is legal)");
    assert_eq!(s.run_i32(), 1301, "the program still runs correctly");
}

/// V-13 (3) unit face: a stale caller unit does not LINK against a new
/// signature (raw D828 interfaces — the same refusal the session-level
/// front prevents up-stream).
#[test]
fn av_owned_821_v13_stale_caller_unit_link_refused() {
    let build = |src: &str| -> std::collections::BTreeMap<String, Arc<FunctionCode>> {
        let compiled =
            auto_ac_prototype::source::compile_text_record_source(src, "u.at", &[]).unwrap();
        let bundle = compiled.checked.bundle();
        let module = bundle.modules.first().unwrap();
        let snap = auto_av_core::contract::fnv1a64(src.as_bytes());
        let mut out = std::collections::BTreeMap::new();
        for d in &module.defs {
            if matches!(d.kind, auto_ac_prototype::hir::DefKind::Function { .. }) {
                let u = auto_av_probe::hir_adapter::lower_function_owned_unit(
                    &bundle,
                    &d.id_text,
                    &[],
                    snap,
                )
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
fn av_owned_821_v13_shared_scalar_session_still_warms() {
    let src = "fn main() int {\n    return 6\n}\n";
    let mut s = auto_av_probe::session::warmup(src, "d_main", &[]).expect("scalar warmup");
    let (out, _) = s.run(&ExecLimits::default(), &CancelFlag::new(false));
    let _ = &mut s;
    assert!(
        matches!(out, ExecOutcome::Success { .. }),
        "the scalar session still runs: {out:?}"
    );
    let _ = execute_program;
}
