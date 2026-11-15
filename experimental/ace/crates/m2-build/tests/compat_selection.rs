//! `ace.m2.p978.compat-selection` — V-ID-REJECT / V-ID-RUN / V-ID-SELECT
//! (PLAN-978 T-03/T-08): the runtime-compatibility axes reject BEFORE
//! dispatch; the building rustc identity is NOT an admission axis; a
//! precise pin never auto-upgrades.

mod common;

use ace_m2_build::compat::{
    check_compatibility, ConsumerRequirements, ProviderOffer, SymbolContract, TargetAxis,
};
use ace_m2_build::selection::{admit_selection, ProviderArtifact, ProviderSelection};

fn consumer() -> ConsumerRequirements {
    ConsumerRequirements::native_program("sf_main", "sig-hash-1")
}

fn offer() -> ProviderOffer {
    let c = consumer();
    ProviderOffer {
        target: c.target.clone(),
        abi_domain: c.abi_domain.clone(),
        interface_schema: c.interface_schema.clone(),
        offered_symbols: c.required_symbols.clone(),
        semantic_owner: c.semantic_owner.clone(),
        threading: c.threading.clone(),
        runtime_dependencies: c.runtime_dependencies.clone(),
    }
}

/// V-ID-RUN leg 1: an offer built by a DIFFERENT rustc (modeled as an
/// offer that carries no rustc axis at all) with every real axis intact
/// is COMPATIBLE — no rustc-equality refusal.
#[test]
fn p978_rustc_identity_is_not_an_admission_axis() {
    let mut o = offer();
    // Same real axes, "different toolchain provenance" — nothing about
    // the offer encodes a rustc identity, so admission holds.
    o.runtime_dependencies = consumer().runtime_dependencies.clone();
    assert!(check_compatibility(&consumer(), &o).is_ok());
}

/// V-ID-REJECT: every axis mismatch is a NAMED pre-dispatch rejection —
/// target os/arch/env/bits/endianness, ABI domain, interface schema,
/// required symbol presence/signature, semantic owner, threading,
/// runtime dependency. Same-CPU/repr(C)/same-major NEVER suffice.
#[test]
fn p978_compat_axes_reject_named_before_dispatch() {
    let cases: Vec<(&'static str, ProviderOffer)> = vec![
        ("m2build.compat.target-os", {
            let mut o = offer();
            o.target.os = "linux".into();
            o
        }),
        ("m2build.compat.target-arch", {
            let mut o = offer();
            o.target.arch = "aarch64".into();
            o
        }),
        ("m2build.compat.target-env", {
            let mut o = offer();
            o.target.env = "gnu".into();
            o
        }),
        ("m2build.compat.target-bits", {
            let mut o = offer();
            o.target.bits = 32;
            o
        }),
        ("m2build.compat.target-endianness", {
            let mut o = offer();
            o.target.endianness = "big".into();
            o
        }),
        ("m2build.compat.abi-domain", {
            let mut o = offer();
            o.abi_domain = "rust".into();
            o
        }),
        ("m2build.compat.interface-schema", {
            let mut o = offer();
            o.interface_schema = "ace-stdlib-assembly/9".into();
            o
        }),
        ("m2build.compat.required-symbol", {
            let mut o = offer();
            o.offered_symbols.clear();
            o
        }),
        ("m2build.compat.symbol-signature", {
            let mut o = offer();
            o.offered_symbols[0].signature_hash = "different".into();
            o
        }),
        ("m2build.compat.semantic-owner", {
            let mut o = offer();
            o.semantic_owner = "someone.else".into();
            o
        }),
        ("m2build.compat.threading", {
            let mut o = offer();
            o.threading = "multi".into();
            o
        }),
        ("m2build.compat.runtime-dep", {
            let mut o = offer();
            o.runtime_dependencies.clear();
            o
        }),
    ];
    for (expected_code, bad_offer) in cases {
        let err =
            check_compatibility(&consumer(), &bad_offer).expect_err("axis mismatch must reject");
        assert_eq!(err.code, expected_code, "axis detail: {}", err.detail);
    }
    // A legal near-neighbor still passes (the accepted control).
    assert!(check_compatibility(&consumer(), &offer()).is_ok());
}

/// V-ID-SELECT: the precise pin never auto-upgrades — a NEW on-disk hash
/// (intact required subset) is a pin rejection until the selection is
/// explicitly updated; a wrong grant never admits.
#[test]
fn p978_selection_pin_and_authorization() {
    let sel = ProviderSelection::new(
        "f0-text-provider",
        7,
        vec![ProviderArtifact {
            provider_id: "f0-text-provider".into(),
            path: "providers/p.dll".into(),
            sha256: "aaa".into(),
            kind: "cdylib".into(),
        }],
        "aaa",
        "run:grant-1",
    );
    // Exact pin + valid grant: admitted.
    let ok = admit_selection(&sel, "aaa", true, true).unwrap();
    assert_eq!(ok.admitted_sha256, "aaa");
    assert_eq!(ok.generation, 7);
    // Same bytes, intact subset, WRONG grant: refused.
    let err = admit_selection(&sel, "aaa", true, false).unwrap_err();
    assert_eq!(err.code, "m2build.selection.authorization");
    // NEW hash (optional symbols added, subset intact), pin unchanged:
    // refused — an explicit selection update is required.
    let err = admit_selection(&sel, "bbb", true, true).unwrap_err();
    assert_eq!(err.code, "m2build.selection.pin");
    // Explicit re-pin of the new hash admits (the update path).
    let updated = ProviderSelection::new(
        "f0-text-provider",
        8,
        vec![ProviderArtifact {
            provider_id: "f0-text-provider".into(),
            path: "providers/p.dll".into(),
            sha256: "bbb".into(),
            kind: "cdylib".into(),
        }],
        "bbb",
        "run:grant-1",
    );
    assert!(admit_selection(&updated, "bbb", true, true).is_ok());
    // Required subset broken: refused even with a matching pin.
    let err = admit_selection(&updated, "bbb", false, true).unwrap_err();
    assert_eq!(err.code, "m2build.selection.required-subset");
    // No artifacts declared: refused.
    let empty = ProviderSelection::new("x", 1, vec![], "aaa", "g");
    let err = admit_selection(&empty, "aaa", true, true).unwrap_err();
    assert_eq!(err.code, "m2build.selection.artifact-missing");
}

/// TargetAxis::host records the real host facts (the Windows validation
/// host; Linux axes exist as data — their RUN legs are deferred to
/// PLAN-1063, never asserted here).
#[test]
fn p978_host_axis_recorded() {
    let axis = TargetAxis::host();
    if cfg!(target_os = "windows") {
        assert_eq!(axis.os, "windows");
        assert_eq!(axis.bits, 64);
    }
    assert!(!axis.arch.is_empty());
}

/// SymbolContract identity: required symbols are name+signature pairs —
/// a name-only match never equals a contract.
#[test]
fn p978_symbol_contract_needs_signature() {
    let a = SymbolContract {
        name: "sf_main".into(),
        signature_hash: "h1".into(),
    };
    let b = SymbolContract {
        name: "sf_main".into(),
        signature_hash: "h2".into(),
    };
    assert_ne!(a, b);
}
