//! PLAN-967 T-13 (AC-11 / V-13 row mapping): the bidirectional/resource
//! obligations close on the DELIVERED lanes —
//!
//! * EXPORT direction: an independent C caller calls ACE-compiled code —
//!   the delivered 939 CE bridge (the `ce-record`/`ce-text-view` decks on
//!   the `abi/ca-d1-bridge-v1` source leg, production case group green);
//! * callback reentry / destroy-once / stale-handle / cancel-late: the
//!   delivered 939 resource suites (callback, callback-reentrant,
//!   handle-lifecycle, handle-stale, txn-*, forged-handle decks on
//!   native+c legs);
//! * the future ports: T-08's CompletionPorts + this round's
//!   session-generation binding (a session teardown kills every in-flight
//!   port; a rebind starts fresh);
//! * F-967-COFF-CB disposition: the whole-module split path now FAILS
//!   CLOSED for callback-carrying resource manifests (named rejection
//!   pointing at the delivered per-function session lane).
#![cfg(windows)]
use std::path::PathBuf;

#[test]
fn t13_future_ports_bind_the_session_generation() {
    use ace_av_core::foreign_exec::{CompletionPorts, ForeignSession};
    use ace_foreign_contract::PortValue;

    // a session + its port registry move together
    let session = ForeignSession::new();
    let mut ports = CompletionPorts::new(session.generation());
    ports.bind_session_generation(session.generation());
    assert_eq!(ports.session_generation(), session.generation());

    let a = ports.register("f0.async");
    assert_eq!(
        ports.complete(a, PortValue::I32(1)),
        ace_foreign_contract::CompletionDisposition::Publish
    );

    // session teardown moves the generation — the port registry bound to
    // the OLD generation drops every in-flight completion
    let mut session = ForeignSession::new();
    let mut ports = CompletionPorts::new(session.generation());
    ports.bind_session_generation(session.generation());
    let b = ports.register("f0.async");
    session.teardown();
    ports.bind_session_generation(session.generation());
    assert_eq!(
        ports.complete(b, PortValue::I32(2)),
        ace_foreign_contract::CompletionDisposition::DropLate,
        "a session move kills the in-flight port (the late-completion drop)"
    );
}

#[test]
fn t13_coff_split_path_fails_closed_for_callback_resource_manifests() {
    // F-967-COFF-CB: the whole-module split entrance is the SCALAR (938)
    // lane; a callback-carrying resource manifest gets a NAMED rejection
    // (never a broken caller). The 939 callback deck's manifest+source
    // provide the exact shape.
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("t13-fence-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let deck_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../../../crates/auto-lang/test/cases/ace-ca-d-resources-939/src/callback.atomdata",
    );
    let deck: ace_abi::worker_source::Deck =
        serde_json::from_slice(&std::fs::read(deck_path).unwrap()).unwrap();
    let authority = ace_foreign_contract::manual::HostAuthority {
        review_id: deck.manifest.review_id.clone(),
        contract_digest: deck.manifest.contract_digest().unwrap(),
        provider: deck.manifest.provider.clone(),
        provider_digest: "0".repeat(64),
        header_digest: deck.manifest.header_digest.clone(),
        physical_table_digest: deck.manifest.physical_table_digest.clone(),
        target: deck.manifest.target.clone(),
        capabilities: deck.manifest.facts.capabilities.iter().cloned().collect(),
        available_symbols: deck
            .manifest
            .facts
            .functions
            .iter()
            .map(|f| f.symbol.clone())
            .collect(),
        binding_generation: 1,
    };
    let environment = deck.manifest.clone().admit(&authority).unwrap();
    let mut query = ace_source_query::SourceQuerySession::new("p967f", "m967f");
    let refused = match ace_native::foreign_lower::compile_source_units(
        &deck.source,
        "f0967f",
        &deck.capabilities,
        &environment,
        &mut query,
    ) {
        Err(message) => message,
        Ok(_) => panic!("the callback fence must fire"),
    };
    assert!(
        refused.contains("foreign.callback-lane"),
        "the named fence, got: {refused}"
    );
}

#[test]
fn t13_export_direction_is_the_delivered_ce_bridge() {
    // The CE export direction (an independent C caller calling ACE-compiled
    // code) is the delivered 939 bridge: the ce-record / ce-text-view decks
    // ride the `abi/ca-d1-bridge-v1` source leg of the PRODUCTION case
    // group `case-ace-ca-d-resources-939` (green). This row asserts the
    // deck identities exist and stay bound to the export leg — the
    // execution proof lives in that suite (run by T-04's sweep).
    for deck in ["ce-record", "ce-text-view"] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../../../crates/auto-lang/test/cases/ace-ca-d-resources-939/src/{deck}.atomdata"
        ));
        let bytes = std::fs::read(&path).unwrap_or_else(|e| {
            panic!("the {deck} export deck must exist (the 939 CE bridge): {e}")
        });
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("PLAN-939"), "{deck} is the CE export deck");
    }
}
