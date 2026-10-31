//! PLAN-894 T-05 native tier: the owned-TEXT source incremental session
//! (`OwnedSourceMnSession`) over the real mn pipeline — V-06 (true
//! body-level increments with structural production oracles), V-07
//! (candidate failure atomicity — the old program keeps running) and
//! V-08's native restart leg (persisted required-data decode + verify,
//! no trusted flags, then a legal edit continues). The AV-side twin
//! lives in `ac-av-probe/probe/tests/text_source_894_av.rs`.

#[path = "../common/mod.rs"]
mod common;

use ace_native::owned_source_incremental::OwnedSourceMnSession;
use std::collections::BTreeMap;

/// The §5.3 supplement source: text_job carries the T03 algorithm (23
/// with the `c = c + "中"` replacement), cmp_job the T05 algorithm (42);
/// main = 65. disjoint() is built but never called.
const SUPPLEMENT: &str = "\
fn text_job() int {
    let a Str = \"ab\"
    let b Str = a.clone()
    var c Str = b.move
    c = c + \"中\"
    return a.len() * 10 + c.len()
}
fn cmp_job() int {
    let a str = \"A\\0B\"
    if a == \"A\" { return 1 }
    if a != \"A\\0B\" { return 2 }
    return 42
}
fn disjoint() int { return 100 }
fn main() int { return text_job() + cmp_job() }
";

fn fresh_root(tag: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("mn894-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// (unit name → (fp, generation)) snapshot — the structural identity.
fn identity(s: &OwnedSourceMnSession) -> BTreeMap<String, (String, u64)> {
    s.unit_names()
        .iter()
        .map(|n| {
            let e = s.unit_entry(n).unwrap();
            (n.clone(), (e.fp.clone(), e.generation))
        })
        .collect()
}

fn tools_ready() -> bool {
    common::tools_available()
}

#[test]
#[cfg(windows)]
fn owned_warmup_and_single_body_edit_65_66() {
    if !tools_ready() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — not run");
        return;
    }
    let root = fresh_root("warm-edit");
    let (mut s, cold) =
        OwnedSourceMnSession::warmup(SUPPLEMENT, "d_main", &[], &root).expect("warmup");
    assert_eq!(cold.exit_code, Some(65), "cold value (23 + 42)");
    // every unit produced exactly once in the cold round
    for name in s.unit_names() {
        let p = s.production().get(name).unwrap();
        assert_eq!(
            (p.parse, p.recheck, p.emit, p.lower, p.coff, p.cache_put),
            (1, 1, 1, 1, 1, 1),
            "{name}"
        );
    }
    let before = identity(&s);
    // ---- body edit: text_job `中` -> `中x` (len 3 -> 4): 23 -> 24, 65 -> 66
    let pos = s.src().find("c + \"中\"").expect("needle");
    let byte_pos = s.src()[..pos].len() + "c + \"".len();
    let obs = s.edit(
        byte_pos as u64,
        (byte_pos + "中".len()) as u64,
        "中x".as_bytes().to_vec(),
    );
    assert!(
        obs.ok,
        "edit applies: {:?}",
        obs.refused_reason.clone().or(obs.infra.clone())
    );
    assert_eq!(obs.exit_code, Some(66), "24 + 42");
    assert_eq!(obs.changed_fn.as_deref(), Some("d_text_job"));
    assert_eq!(obs.codegen_units, 1, "exactly one COFF emitted");
    assert_eq!(obs.fallback_whole_lower, 0, "no whole-module fallback");
    assert_eq!(obs.whole_module_stage_delta, 0, "zero whole-module work");
    assert_eq!(obs.unaffected_stage_delta, 0, "zero untouched-unit work");
    // structural identity: ONLY d_text_job moved (fp changed, generation 0->1)
    let after = identity(&s);
    for (name, (fp, gen)) in &after {
        let (ofp, ogen) = before.get(name).unwrap();
        if name == "d_text_job" {
            assert_ne!(fp, ofp, "d_text_job fp moved");
            assert_eq!(*gen, 1, "d_text_job generation bumped once");
        } else {
            assert_eq!((fp, gen), (ofp, ogen), "{name}: identity frozen");
        }
    }
    // production: text_job one more round; every other unit unchanged
    for name in s.unit_names() {
        let p = s.production().get(name).unwrap();
        if name == "d_text_job" {
            assert_eq!(
                (p.parse, p.lower, p.coff),
                (2, 2, 2),
                "target re-produced once"
            );
        } else {
            assert_eq!((p.parse, p.lower, p.coff), (1, 1, 1), "{name}: untouched");
        }
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
#[cfg(windows)]
fn owned_a_to_b_to_a_history_reuse() {
    if !tools_ready() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — not run");
        return;
    }
    let root = fresh_root("aba");
    let (mut s, _) = OwnedSourceMnSession::warmup(SUPPLEMENT, "d_main", &[], &root).unwrap();
    let base = identity(&s);
    let pos = s.src().find("c + \"中\"").unwrap();
    let byte_pos = s.src()[..pos].len() + "c + \"".len();
    // A -> B
    let b = s.edit(
        byte_pos as u64,
        (byte_pos + "中".len()) as u64,
        "中x".as_bytes().to_vec(),
    );
    assert_eq!(b.exit_code, Some(66));
    assert!(!b.reused_history_unit);
    // B -> A (byte positions identical: the edit was length-preserving
    // in bytes? "中" 3 bytes vs "中x" 4 bytes — recompute from src)
    let pos2 = s.src().find("c + \"中x\"").unwrap();
    let byte_pos2 = s.src()[..pos2].len() + "c + \"".len();
    let a = s.edit(
        byte_pos2 as u64,
        (byte_pos2 + "中x".len()) as u64,
        "中".as_bytes().to_vec(),
    );
    assert!(a.ok, "back-edit applies");
    assert_eq!(a.exit_code, Some(65), "value back to 65");
    assert!(a.reused_history_unit, "the ORIGINAL body is a content hit");
    assert_eq!(a.codegen_units, 0, "zero COFF production on a content hit");
    // identity: d_text_job back to the ORIGINAL fp, generation NOT bumped
    let after = identity(&s);
    let (fp, gen) = after.get("d_text_job").unwrap();
    let (ofp, ogen) = base.get("d_text_job").unwrap();
    assert_eq!(
        (fp, gen),
        (ofp, ogen),
        "A→B→A restores the original unit identity"
    );
    // production honesty: parse/recheck/emit ran again, lower/coff did NOT
    let p = s.production().get("d_text_job").unwrap();
    assert_eq!(
        (p.parse, p.recheck, p.emit),
        (3, 3, 3),
        "three real frontend rounds"
    );
    assert_eq!(
        (p.lower, p.coff, p.cache_put),
        (2, 2, 2),
        "only two real unit productions"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
#[cfg(windows)]
fn owned_cmp_job_edit_is_independent() {
    if !tools_ready() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — not run");
        return;
    }
    let root = fresh_root("cmp");
    let (mut s, _) = OwnedSourceMnSession::warmup(SUPPLEMENT, "d_main", &[], &root).unwrap();
    let before = identity(&s);
    // cmp_job: `return 42` -> `return 52`: 23 + 52 = 75
    let pos = s.src().find("return 42").unwrap();
    let byte_pos = s.src()[..pos].len() + "return ".len();
    let obs = s.edit(byte_pos as u64, (byte_pos + 2) as u64, b"52".to_vec());
    assert!(obs.ok, "cmp_job edit applies");
    assert_eq!(obs.exit_code, Some(75));
    assert_eq!(obs.changed_fn.as_deref(), Some("d_cmp_job"));
    let after = identity(&s);
    for (name, v) in &after {
        let o = before.get(name).unwrap();
        if name == "d_cmp_job" {
            assert_eq!(v.1, 1, "cmp_job bumped");
        } else {
            assert_eq!(
                v, o,
                "{name} frozen (text_job untouched by a static-return edit)"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
#[cfg(windows)]
fn owned_failed_candidate_keeps_old_program_running() {
    if !tools_ready() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — not run");
        return;
    }
    let root = fresh_root("fail");
    let (mut s, _) = OwnedSourceMnSession::warmup(SUPPLEMENT, "d_main", &[], &root).unwrap();
    let before = identity(&s);
    // A compile-broken body edit: `return a.len() * 10 + c.len()` ->
    // `return a` (type mismatch) inside text_job.
    let pos = s.src().find("return a.len() * 10 + c.len()").unwrap();
    let byte_pos = s.src()[..pos].len() + "return ".len();
    let obs = s.edit(
        byte_pos as u64,
        (byte_pos + "a.len() * 10 + c.len()".len()) as u64,
        b"a".to_vec(),
    );
    assert!(!obs.ok, "the broken edit is refused");
    assert!(
        obs.refused_reason.is_some(),
        "named refusal: {:?}",
        obs.refused_reason
    );
    assert_eq!(obs.attempt_prepare, 1, "the attempt is honestly counted");
    assert_eq!(obs.exit_code, None, "no PE claims the broken program");
    // the old program still runs at 65
    let rerun = s.rerun();
    assert!(rerun.ok);
    assert_eq!(
        rerun.exit_code,
        Some(65),
        "the prior revision keeps running"
    );
    // identity untouched
    assert_eq!(identity(&s), before, "no unit moved on a refused edit");
    // a FOLLOWING legal edit recovers (66)
    let pos = s.src().find("c + \"中\"").unwrap();
    let byte_pos = s.src()[..pos].len() + "c + \"".len();
    let ok = s.edit(
        byte_pos as u64,
        (byte_pos + "中".len()) as u64,
        "中x".as_bytes().to_vec(),
    );
    assert!(ok.ok, "legal edit after failure recovers");
    assert_eq!(ok.exit_code, Some(66));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
#[cfg(windows)]
fn owned_cross_process_restore_and_continuation() {
    if !tools_ready() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — not run");
        return;
    }
    let root = fresh_root("restore");
    let (mut s, _) = OwnedSourceMnSession::warmup(SUPPLEMENT, "d_main", &[], &root).unwrap();
    let pos = s.src().find("c + \"中\"").unwrap();
    let byte_pos = s.src()[..pos].len() + "c + \"".len();
    let b = s.edit(
        byte_pos as u64,
        (byte_pos + "中".len()) as u64,
        "中x".as_bytes().to_vec(),
    );
    assert_eq!(b.exit_code, Some(66));
    let before = identity(&s);
    // ---- new process: decode + verify + run -----------------------
    drop(s);
    let (mut s2, obs) = OwnedSourceMnSession::restore(&root).expect("restore");
    assert!(obs.ok);
    assert_eq!(obs.exit_code, Some(66), "restored program reproduces 66");
    assert_eq!(
        identity(&s2),
        before,
        "restored identity equals the persisted one"
    );
    for name in s2.unit_names() {
        assert_eq!(
            s2.production_sum(name),
            0,
            "{name}: restored counters start at zero"
        );
    }
    // a legal edit continues; A→B→A hits the warm session's history
    let pos2 = s2.src().find("c + \"中x\"").unwrap();
    let byte_pos2 = s2.src()[..pos2].len() + "c + \"".len();
    let a = s2.edit(
        byte_pos2 as u64,
        (byte_pos2 + "中x".len()) as u64,
        "中".as_bytes().to_vec(),
    );
    assert!(a.ok);
    assert_eq!(a.exit_code, Some(65));
    assert!(
        a.reused_history_unit,
        "the persisted history still serves content hits"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
#[cfg(windows)]
fn owned_restore_refuses_corrupted_store() {
    if !tools_ready() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — not run");
        return;
    }
    let root = fresh_root("corrupt");
    let (s, _) = OwnedSourceMnSession::warmup(SUPPLEMENT, "d_main", &[], &root).unwrap();
    let _ = s;
    // Corrupt the persisted manifest's recorded exit (a wrong artifact
    // set must refuse — no trusted flags).
    let mpath = root.join("owned-894-session.json");
    let raw = std::fs::read_to_string(&mpath).unwrap();
    let bad = raw.replace("\"last_exit\": 65", "\"last_exit\": 12345");
    assert_ne!(bad, raw, "manifest patch applied");
    std::fs::write(&mpath, bad).unwrap();
    let r = OwnedSourceMnSession::restore(&root);
    assert!(r.is_err(), "a drifted manifest refuses the restore");
    eprintln!("restore refusal: {:?}", r.err());
    let _ = std::fs::remove_dir_all(&root);
}
