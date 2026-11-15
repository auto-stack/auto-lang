//! PLAN-969 AUDIT-08 (1024 F08): the profile × consumer FACT-AUTHORITY
//! table, pinned by permanent guards at the real product seams.
//!
//! Table (current authority, named — 1024 F08's deliverable):
//!
//! | Profile  | Product consumers                                  | Type authority / bind                     | Retained legacy derivation (named, not deleted) |
//! |----------|----------------------------------------------------|-------------------------------------------|--------------------------------------------------|
//! | Scalar   | `compile_scalar_product` (AV query_entry, A2X…)    | SQ facts (PLAN-934) `bind_with_facts`     | `core::compile_scalar_source` = diagnostic oracle ONLY (verdicts, never HIR) |
//! | Core     | `compile_core_product` (AV query_entry, native rung)| SQ facts (PLAN-999)                        | `core::compile_scalar_source` oracle discipline, same |
//! | TextRecord | `compile_text_record_product` (969 facade/foreign lane) | SQ facts + foreign admission (938/967) | `core::compile_text_record_source` (native ladder rung) — dual entry NAMED |
//! | Record   | `compile_record_product` (963 instances lane)      | EMPTY fact supply; the domain's own resolve/verify | `core::compile_record_source` (native rung) |
//! | Numeric/Aggregates | native ladder rungs                       | own core compile families (NO solver fact page yet — NAMED uncovered) | they ARE the current authority |
//!
//! Guarded invariants (the AUDIT-08 oracle):
//! * facts rejection at ITS profile is a TYPED rejection — the entry
//!   never silently accepts via legacy derivation, and escalation is the
//!   caller's EXPLICIT gate, not a hidden re-route;
//! * a genuine type error keeps the canonical typed diagnostics;
//! * the uncovered lanes (Record/Numeric/Aggregates) answer through
//!   their own named families — not by deleting or silently re-routing.

use ace_core::atom_text::{Stage, SurfaceFact};
use ace_source_query::{
    compile_core_product, compile_record_product, compile_scalar_product,
    compile_text_record_product, SourceQuerySession,
};

fn scalar_session() -> SourceQuerySession {
    SourceQuerySession::new("p969-audit08", "p969.at")
}

/// Scalar keeps facts authority: a Core-owned construct (`while`) is a
/// TYPED surface rejection at the scalar profile — never silently
/// accepted through any legacy derivation.
#[test]
fn audit08_scalar_facts_authority_typed_rejection() {
    let src = "fn main() int { var i = 0\nwhile i < 1 { i = i + 1 }\nreturn i }";
    let mut session = scalar_session();
    let err = match compile_scalar_product(
        src,
        "p969.at",
        &[],
        "p969-audit08",
        "p969.at",
        &mut session,
    ) {
        Ok(_) => panic!("scalar owns its rejection — no silent acceptance"),
        Err(ds) => ds,
    };
    assert!(
        err.iter()
            .all(|d| d.code == "source.unsupported" && d.stage == Stage::Text),
        "the scalar face reports its own surface rejection: {err:?}"
    );
    // PLAN-969 (AUDIT-03): the rejection carries the STRUCTURED surface
    // fact — escalation ladders route on this, never on the message.
    assert!(
        err.iter()
            .all(|d| d.surface == Some(SurfaceFact::ProfileSurface)),
        "the projector tags the profile-surface rejection: {err:?}"
    );
}

/// The explicit shared gate escalates the SAME source to the Core fact
/// lane and compiles it — escalation is the caller's gate, visible, and
/// the Core lane is also facts-authoritative.
#[test]
fn audit08_core_facts_lane_compiles_the_escalated_source() {
    let src = "fn main() int { var i = 0\nwhile i < 1 { i = i + 1 }\nreturn i }";
    let mut session = scalar_session();
    let product =
        match compile_core_product(src, "p969.at", &[], "p969-audit08", "p969.at", &mut session) {
            Ok(product) => product,
            Err(ds) => panic!("the Core fact lane compiles the while source: {ds:?}"),
        };
    assert_eq!(
        product.compiled.profile, "auto.source.core-i32.draft",
        "the compiled artifact names the profile that owns the authority"
    );
}

/// A genuine type error keeps the canonical typed diagnostics at the
/// owning profile (no silent success, no re-route).
#[test]
fn audit08_typed_error_stays_canonical() {
    let src = "fn main() int {\n    let x int = 1\n    x = 2\n    return x\n}\n";
    let mut session = scalar_session();
    let err = match compile_scalar_product(
        src,
        "p969.at",
        &[],
        "p969-audit08",
        "p969.at",
        &mut session,
    ) {
        Ok(_) => panic!("the immutable write is a typed rejection — no silent success"),
        Err(ds) => ds,
    };
    assert!(
        err.iter()
            .any(|d| d.code == "verify.immutable-write" && d.stage == Stage::Verify),
        "the canonical verify diagnostic answers: {err:?}"
    );
}

/// The TextRecord product lane (the 969 facade/foreign lane's seam) is
/// facts-authoritative for ordinary text-record source, and the Record
/// lane (empty supply) answers through its own resolve/verify — the
/// uncovered lanes are NAMED, neither deleted nor silently re-routed.
#[test]
fn audit08_textrecord_and_record_lanes_answer_their_own() {
    // TextRecord: a plain record-shaped source compiles through the fact
    // lane (the 969 bridge's exact seam).
    let src = "use.rs f0_text\nfn main() int { return 7 }";
    let mut session = scalar_session();
    if let Err(ds) =
        compile_text_record_product(src, "p969.at", &[], "p969-audit08", "p969.at", &mut session)
    {
        panic!("the text-record fact lane compiles ordinary text-record source: {ds:?}");
    }
    // Record: the instances lane has NO solver fact page (empty supply);
    // ordinary record source still compiles through its own family.
    let record_src = "type Pair { left int; right int }\nfn main() int { return 1 }";
    let mut session2 = scalar_session();
    if let Err(ds) = compile_record_product(
        record_src,
        "p969.at",
        &[],
        "p969-audit08",
        "p969.at",
        &mut session2,
    ) {
        panic!(
            "the record lane answers through its own resolve/verify (named uncovered-by-facts lane): {ds:?}"
        );
    }
}
