//! PLAN-934 F-934-R1-01 (AC-05): the facts restore lane exercised through
//! the REAL PLAN-929 restore interface.
//!
//! §5.3 requires the restore lane to consume "929 or an equivalently
//! released source + CST/declaration-context interface". This test starts
//! from that interface instead of a bare parse:
//!
//! ```text
//! 929 cold UnitCatalog → encode/decode manifest → UnitCatalog::from_restored
//!   → restored source snapshot → decl-env dump/encode/decode/restore
//!   (profile + caps + source-sha identity agreement, wrong sha refused)
//!   → facts pages admitted by restore_verify (no solver)
//!   → restore_bind (pages only) → emit → REAL HIR verification
//!   → seed → first good edit solves exactly one body → first bad edit
//!     refused at admission → repair produces a clean page
//! ```

use ace_core::source::cst::units::{self, UnitCatalog};
use ace_core::source::cst::{FrontBudget, FRONT_CAPS_VERSION};
use ace_core::source::decl_env::{
    decl_env_dump, decl_env_restore, decode_decl_env, encode_decl_env,
};
use ace_core::source::{
    emit_resolved_parts, env_fn_names, parse_program_with, resolve_env, SourceProfile,
};
use ace_core::verify::verify_scalar;
use ace_source_query::{restore_bind, restore_verify, InferBodyFacts, SourceQuerySession};
use std::collections::BTreeMap;

const SRC: &str =
    "fn leaf(x int) int {\n    return x + 1\n}\n\nfn main() int {\n    return leaf(41)\n}\n";
const PROFILE: SourceProfile = SourceProfile::Scalar;
const PACKAGE: &str = "pkg";
const MODULE: &str = "m";
const CAPS: &[String] = &[];

/// Cold fact pages of every free function (the producer side of a restore).
fn cold_pages(src: &str) -> BTreeMap<String, InferBodyFacts> {
    let program = parse_program_with(src, PROFILE).expect("parse");
    let mut session = SourceQuerySession::new(PACKAGE, MODULE);
    let mut pages = BTreeMap::new();
    for item in &program.items {
        if let ace_core::source::ast::Item::Fn(f) = item {
            let facts = session
                .infer_body(&program, &f.name.name)
                .expect("cold facts")
                .facts;
            pages.insert(f.name.name.clone(), facts);
        }
    }
    pages
}

#[test]
fn facts_restore_lane_runs_on_the_929_restore_seam() {
    // ---- 1. 929 cold catalog, frozen manifest, real restore --------------
    let cold =
        UnitCatalog::build(SRC, PROFILE, FrontBudget::WORK_OWNED_64MIB).expect("929 cold catalog");
    let manifest = units::decode_manifest(&units::encode_manifest(
        &cold.to_manifest(FRONT_CAPS_VERSION),
    ))
    .expect("manifest round-trip");
    let restored = UnitCatalog::from_restored(cold.src(), &manifest, PROFILE, FRONT_CAPS_VERSION)
        .expect("929 restore adopts the current source + manifest");
    assert_eq!(
        restored.src(),
        SRC,
        "restored snapshot is the current source"
    );
    assert_eq!(
        restored.profile, PROFILE,
        "restored catalog keeps the profile"
    );
    for owner in ["leaf", "main"] {
        assert!(
            restored.fns.contains_key(owner),
            "restored catalog owns `{owner}`"
        );
    }

    // The facts lane's input is the RESTORED source, not a fresh test string.
    let program = parse_program_with(restored.src(), PROFILE).expect("parse restored source");

    // ---- 2. 929 declaration context: identity agreement ------------------
    let source_sha = "fnv-or-sha-of-the-current-source";
    let env = resolve_env(&program, CAPS, PROFILE).expect("declaration context");
    let dump = decl_env_dump(&env, FRONT_CAPS_VERSION, source_sha);
    let decoded = decode_decl_env(&encode_decl_env(&dump)).expect("decl env round-trip");
    let restored_env = decl_env_restore(&decoded, PROFILE, FRONT_CAPS_VERSION, source_sha)
        .expect("decl env restore");
    assert_eq!(
        env_fn_names(&restored_env),
        env_fn_names(&env),
        "restored declaration context keeps the module's function surface"
    );
    assert!(
        decl_env_restore(&decoded, PROFILE, FRONT_CAPS_VERSION, "another-source").is_err(),
        "a foreign source identity is refused, never silently adopted"
    );

    // ---- 3. facts pages admitted against the restored source ------------
    let pages = cold_pages(restored.src());
    for (name, page) in &pages {
        restore_verify(&program, PACKAGE, MODULE, name, Some(page))
            .unwrap_or_else(|e| panic!("page `{name}` must admit: {e}"));
    }
    assert!(
        restore_verify(&program, PACKAGE, MODULE, "leaf", None).is_err(),
        "a missing required page refuses the lane (explicit cold is another request)"
    );

    // ---- 4. pages-only bind emits HIR that the REAL verifier accepts -----
    let (resolved, applied, supply) =
        restore_bind(&program, CAPS, PACKAGE, MODULE, &pages, PROFILE).expect("pages-only bind");
    assert_eq!(applied.len(), 2, "every owner bound from pages");
    assert_eq!(supply.admitted_bodies(), 2);
    let parts = emit_resolved_parts(&resolved, restored.src(), "m.at", PROFILE).expect("emit");
    verify_scalar(parts.bundle).expect("the restored HIR passes real verification");

    // ---- 5. seed → first good edit, first bad edit, repair ---------------
    let mut session = SourceQuerySession::new(PACKAGE, MODULE);
    for page in pages.values() {
        session.seed(&program, page).expect("seed an admitted page");
    }
    assert_eq!(
        session.counters().solver_runs,
        0,
        "seeding restored pages runs no solver"
    );

    // Good edit: only leaf's body changes (+1 -> +2) — exactly one solve,
    // the unchanged caller stays warm.
    let edited = SRC.replace("x + 1", "x + 2");
    let edited_program = parse_program_with(&edited, PROFILE).expect("parse edit");
    let leaf = session
        .infer_body(&edited_program, "leaf")
        .expect("leaf edit");
    assert!(!leaf.hit, "the edited body re-solves");
    let main = session.infer_body(&edited_program, "main").expect("main");
    assert!(main.hit, "the unchanged caller stays warm");
    assert_eq!(
        session.counters().solver_runs,
        1,
        "exactly one solve after restore + first good edit"
    );

    // First BAD edit: the produced page carries the solver's diagnostic and
    // is refused at the facts admission, not by a side checker.
    let bad = SRC.replace("x + 1", "true");
    let bad_program = parse_program_with(&bad, PROFILE).expect("parse bad edit");
    let bad_page = session
        .infer_body(&bad_program, "leaf")
        .expect("bad page produced");
    let raw: &InferBodyFacts = &bad_page.facts;
    assert!(
        restore_verify(&bad_program, PACKAGE, MODULE, "leaf", Some(raw)).is_err(),
        "a diagnostic page is refused at restore admission"
    );
}

#[test]
fn facts_repair_after_the_bad_edit_restores_a_clean_page() {
    // Same seam, repair leg: bad page refused, fixed source yields a page
    // that admits again (the session stays usable across the failure).
    let program = parse_program_with(SRC, PROFILE).expect("parse");
    let pages = cold_pages(SRC);
    let mut session = SourceQuerySession::new(PACKAGE, MODULE);
    for page in pages.values() {
        session.seed(&program, page).expect("seed");
    }
    let bad = SRC.replace("x + 1", "true");
    let bad_program = parse_program_with(&bad, PROFILE).expect("parse bad");
    let bad_page = session.infer_body(&bad_program, "leaf").expect("bad page");
    assert!(restore_verify(&bad_program, PACKAGE, MODULE, "leaf", Some(&bad_page.facts)).is_err());

    let repaired = session.infer_body(&program, "leaf").expect("repair");
    restore_verify(&program, PACKAGE, MODULE, "leaf", Some(&repaired.facts))
        .expect("the repaired page is clean and admits again");
}
