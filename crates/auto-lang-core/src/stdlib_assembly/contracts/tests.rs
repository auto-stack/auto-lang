use super::*;
use crate::stdlib_assembly::{loader, model::*, providers};

fn real() -> (std::path::PathBuf, StdlibInventory) {
    let root = loader::repo_stdlib_root().unwrap();
    let inv = loader::scan_inventory(&root);
    (root, inv)
}

#[test]
fn real_inventory_consumes_contracts_and_preserves_denominators() {
    let (_, mut inv) = real();
    let errors: Vec<_> = inv
        .diagnostics
        .iter()
        .filter(|d| d.code == INVALID)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(inv.contracts.operations.len(), 17);
    assert_eq!(inv.contracts.compatibility.len(), 17);
    // PLAN-947 T-08: the console/1 four-end ledger for the public say
    // binding is machine-pinned — AV and native are real supported lanes
    // (corpus ace-m2/p947-console on both recipes); Rust/C hosts stay
    // missing_capability with the 973 four-end owner, never silently
    // widened.
    let say = inv
        .contracts
        .compatibility
        .iter()
        .find(|c| c.operation == "console.say")
        .expect("console.say compatibility entry");
    assert_eq!(say.ace_av, "supported");
    assert_eq!(say.ace_native, "supported");
    assert_eq!(say.hir_rust, "missing_capability");
    assert_eq!(say.hir_c, "missing_capability");
    assert_eq!(say.owner, "PLAN-947");
    assert!(say.gap.contains("973"));
    assert_eq!(
        inv.files_total,
        inv.modules.iter().map(|m| m.layers.len()).sum::<usize>()
    );
    let failed = inv
        .modules
        .iter()
        .flat_map(|m| &m.layers)
        .filter(|l| matches!(l.parse, ParseStatus::Failed { .. }))
        .count();
    assert_eq!(
        failed,
        inv.contracts
            .rows
            .iter()
            .filter(|r| r.symbol.is_none())
            .count()
    );
    assert!(inv
        .contracts
        .rows
        .iter()
        .any(|r| r.status == "missing_contract"));
    let interface = crate::vm::native::NativeInterface::production();
    audit_providers(&mut inv, &interface);
    for name in ["auto.str.byte_len", "Str.byte_len", "auto.str.parse_int"] {
        assert!(
            inv.contracts
                .native_bindings
                .iter()
                .any(|b| b.name == name && b.bound),
            "{name}"
        );
    }
    assert!(inv
        .contracts
        .native_bindings
        .iter()
        .all(|b| b.verification != "executed"));
    let errors: Vec<_> = inv
        .diagnostics
        .iter()
        .filter(|d| d.code == INVALID)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    assert!(!providers::load_catalog().unwrap().providers.is_empty());
    println!("CONTRACT943 files={} failed_layers={} callable_and_failed_rows={} native_alias_rows={} operations={}", inv.files_total, failed, inv.contracts.rows.len(), inv.contracts.native_bindings.len(), inv.contracts.operations.len());
    if let Ok(path) = std::env::var("PLAN943_INVENTORY_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&inv).unwrap()).unwrap();
    }
}

#[test]
fn contract_mutations_are_rejected_by_real_inventory() {
    let (root, inv) = real();
    let (ops, compat) = decode(OPERATIONS, COMPATIBILITY).unwrap();
    assert!(validate(&ops, &compat, &inv.modules).is_empty());
    for mutation in [
        "unit",
        "same_width_signature",
        "mode",
        "missing_id",
        "duplicate_id",
        "dispatch",
        "visibility",
        "native_identity",
        "missing_native",
        "missing_native_wrong_return",
        "candidate_signature",
        "candidate_mode",
        "byte_return_float",
    ] {
        let mut mutated = ops.clone();
        let byte = mutated
            .operations
            .iter_mut()
            .find(|op| op.id == "text.byte_length")
            .unwrap();
        match mutation {
            "unit" => byte.unit = "unicode_scalars".into(),
            "same_width_signature" => {
                mutated
                    .operations
                    .iter_mut()
                    .find(|op| op.id == "math.min")
                    .unwrap()
                    .parameters[0] = "u64".into()
            }
            "mode" => byte.modes[0] = "Move".into(),
            "missing_id" => byte.id.clear(),
            "dispatch" => byte.dispatch = "dynamic_call_spec".into(),
            "native_identity" => byte.native = Some("auto.user.byte_len".into()),
            "missing_native" => byte.native = None,
            "missing_native_wrong_return" => {
                byte.native = None;
                byte.returns = "float".into();
            }
            "byte_return_float" => byte.returns = "float".into(),
            "candidate_signature" => {
                let op = mutated
                    .operations
                    .iter_mut()
                    .find(|op| op.id == "list.length")
                    .unwrap();
                op.parameters = vec!["str".into()];
                op.modes = vec!["Move".into()];
                op.returns = "float".into();
            }
            "candidate_mode" => {
                let op = mutated
                    .operations
                    .iter_mut()
                    .find(|op| op.id == "list.length")
                    .unwrap();
                op.parameters = vec!["str".into()];
                op.modes = vec!["Move".into()];
            }
            "visibility" => {
                mutated
                    .operations
                    .iter_mut()
                    .find(|op| op.id == "math.min")
                    .unwrap()
                    .visibility = "private".into()
            }
            "duplicate_id" => mutated.operations.push(mutated.operations[0].clone()),
            _ => unreachable!(),
        }
        // Exercise the actual disk decode -> scan -> validation path, not only
        // a string search or a test-only validator.
        let temp = tempfile::tempdir().unwrap();
        let stdlib = temp.path().join("stdlib");
        std::fs::create_dir_all(stdlib.join("auto")).unwrap();
        std::fs::create_dir(stdlib.join("contracts")).unwrap();
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .map(Result::unwrap)
            .filter(|e| e.file_type().is_file())
        {
            let relative = entry.path().strip_prefix(&root).unwrap();
            let dest = stdlib.join("auto").join(relative);
            std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
            std::fs::copy(entry.path(), dest).unwrap();
        }
        std::fs::write(
            stdlib.join("contracts/operations.toml"),
            toml::to_string(&mutated).unwrap(),
        )
        .unwrap();
        std::fs::write(stdlib.join("contracts/compatibility.toml"), COMPATIBILITY).unwrap();
        let result = loader::scan_inventory(&stdlib.join("auto"));
        assert!(
            loader::stdlib_assembly_fingerprint(&stdlib.join("auto"), AssemblyTarget::Vm).is_err(),
            "{mutation}: invalid contract must not yield an admissible assembly identity"
        );
        let expected = if mutation.starts_with("candidate_") {
            "list.length"
        } else if matches!(mutation, "same_width_signature" | "visibility") {
            "math.min"
        } else {
            "text.byte_length"
        };
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == INVALID && d.message.contains(expected)),
            "{mutation}: {:?}",
            result
                .diagnostics
                .iter()
                .filter(|d| d.code == INVALID)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn failed_layer_inline_and_spec_methods_remain_in_inventory() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("broken.at"), "pub fn broken(").unwrap();
    std::fs::write(temp.path().join("user.at"), "pub fn byte_len(s str) int { return 7 }\ntype Example {\n pub fn value() int { return 9 }\n}\nspec Clone { fn clone() Self }\n").unwrap();
    let inv = loader::scan_inventory(temp.path());
    assert_eq!(inv.files_total, 2);
    assert!(inv
        .contracts
        .rows
        .iter()
        .any(|r| r.module == "broken" && r.symbol.is_none()));
    let layer = &inv.module("user").unwrap().layers[0];
    assert!(
        matches!(layer.parse, ParseStatus::Parsed { .. }),
        "{:?}",
        layer.parse
    );
    for symbol in ["byte_len", "Example.value", "Clone.clone"] {
        assert!(
            layer.symbols.iter().any(|s| s.name == symbol),
            "{symbol}: {:?}",
            layer.symbols
        );
    }
    assert!(
        inv.contracts.rows.iter().all(|r| r.operation.is_none()),
        "User homonyms must not acquire stdlib identities"
    );
    assert!(!inv.diagnostics.iter().any(|d| d.code == INVALID));
}

#[test]
fn decode_and_policy_boundaries_fail_closed() {
    assert!(decode(&" ".repeat(MAX_BYTES + 1), COMPATIBILITY).is_err());
    assert!(decode(
        &OPERATIONS.replacen("schema_version = 1", "schema_version = 99", 1),
        COMPATIBILITY
    )
    .is_err());
    assert!(decode(&format!("unknown = true\n{OPERATIONS}"), COMPATIBILITY).is_err());
    assert!(decode("schema_version=1", COMPATIBILITY).is_err());
    let (ops, mut compat) = decode(OPERATIONS, COMPATIBILITY).unwrap();
    compat.entries.pop();
    assert!(!validate(&ops, &compat, &real().1.modules).is_empty());
}

#[test]
fn native_alias_control_and_unverified_candidate_proofs() {
    let (_, mut inv) = real();
    let policy_only = inv
        .contracts
        .operation_proofs
        .iter()
        .find(|p| p.operation == "list.length")
        .unwrap();
    assert_eq!(
        policy_only.signature_status,
        "policy_only_signature_unverified"
    );
    let native = inv
        .contracts
        .operation_proofs
        .iter()
        .find(|p| p.operation == "text.byte_length")
        .unwrap();
    assert_eq!(
        native.signature_status,
        "producer_signature_checked_semantics_unverified"
    );
    inv.contracts
        .operations
        .iter_mut()
        .find(|op| op.id == "text.byte_length")
        .unwrap()
        .native = Some("Str.byte_len".into());
    let interface = crate::vm::native::NativeInterface::production();
    audit_providers(&mut inv, &interface);
    assert!(!inv.diagnostics.iter().any(|d| d.code == INVALID));
}

#[test]
fn all_registered_aliases_survive_one_native_id() {
    let temporary = tempfile::tempdir().unwrap();
    let mut inv = loader::scan_inventory(temporary.path());
    let mut interface = crate::vm::native::NativeInterface::new();
    interface.register(6800, |_task, _vm| Ok(()));
    interface.register_name("user.fixture.original", 6800);
    interface.register_name("user.fixture.alias", 6800);
    interface.register_name("user.fixture.third", 6800);
    // Neither spelling belongs to the catalogue or the global registry.
    // The previous id_to_name() projection necessarily dropped one of them.
    audit_providers(&mut inv, &interface);
    for name in [
        "user.fixture.original",
        "user.fixture.alias",
        "user.fixture.third",
    ] {
        assert!(inv
            .contracts
            .native_bindings
            .iter()
            .any(|b| b.name == name && b.id == Some(6800) && b.bound && b.producer.is_none()));
    }
    let interface = crate::vm::native::NativeInterface::production();
    audit_providers(&mut inv, &interface);
    let reported: BTreeSet<_> = inv
        .contracts
        .native_bindings
        .iter()
        .map(|b| b.name.as_str())
        .collect();
    assert!(interface
        .registered_names()
        .all(|name| reported.contains(name)));
    for name in interface.registered_names() {
        let row = inv
            .contracts
            .native_bindings
            .iter()
            .find(|b| b.name == name)
            .unwrap();
        let id = interface.resolve(name).unwrap();
        assert_eq!(row.id, Some(id));
        assert_eq!(row.bound, interface.get(id).is_some());
        assert_eq!(
            row.producer.as_deref(),
            interface.contract(id).map(|c| c.producer.as_str())
        );
    }
    let first = serde_json::to_value(&inv.contracts.native_bindings).unwrap();
    audit_providers(&mut inv, &interface);
    assert_eq!(
        first,
        serde_json::to_value(&inv.contracts.native_bindings).unwrap()
    );
}

#[test]
fn wide_math_private_helpers_and_legacy_dispatches_are_not_reclassified() {
    let (_, inv) = real();
    for name in ["math.abs", "math.min", "math.max"] {
        let op = inv
            .contracts
            .operations
            .iter()
            .find(|o| o.id == name)
            .unwrap();
        assert!(op.parameters.iter().all(|p| p == "i64"));
        assert_eq!(op.returns, "i64");
        let policy = inv
            .contracts
            .compatibility
            .iter()
            .find(|p| p.operation == name)
            .unwrap();
        // PLAN-945: the i64 public family stays missing_capability on every
        // end — the packages lane rejects it by name, no narrowing adapter.
        assert_eq!(policy.ace_av, "missing_capability");
        assert_eq!(policy.ace_native, "missing_capability");
        assert_eq!(policy.hir_rust, "missing_capability");
        assert_eq!(policy.hir_c, "missing_capability");
        assert!(policy.gap.contains("ace.stdlib-unwired"), "{}", policy.gap);
    }
    for name in ["math.private_square", "math.private_cube"] {
        assert_eq!(
            inv.contracts
                .operations
                .iter()
                .find(|o| o.id == name)
                .unwrap()
                .visibility,
            "private"
        );
    }
    let units: BTreeSet<_> = inv
        .contracts
        .operations
        .iter()
        .filter(|o| o.symbol == "str.len")
        .map(|o| (&o.dispatch, &o.unit))
        .collect();
    assert_eq!(units.len(), 2);
    // PLAN-947/PLAN-945 honestly-delivered wirings only: console.say (AV +
    // native, corpus evidence) and the 945 stdlib slice (byte_len/equal/
    // clone/square/cube via stdlib/ace/std + the four-end conformance).
    // Everything else keeps the 943-era all-gap invariant — wide math in
    // particular must never be silently reclassified.
    let wired_945 = [
        "text.byte_length",
        "text.equal",
        "text.clone",
        "math.private_square",
        "math.private_cube",
    ];
    assert!(inv.contracts.compatibility.iter().all(|c| {
        c.operation == "console.say"
            || wired_945.contains(&c.operation.as_str())
            || (c.ace_av != "supported"
                && c.ace_native != "supported"
                && c.hir_rust != "supported"
                && c.hir_c != "supported")
    }));
    let say = inv
        .contracts
        .compatibility
        .iter()
        .find(|c| c.operation == "console.say")
        .unwrap();
    assert_eq!(
        (say.ace_av.as_str(), say.ace_native.as_str()),
        ("supported", "supported")
    );
    assert_eq!(
        (say.hir_rust.as_str(), say.hir_c.as_str()),
        ("missing_capability", "missing_capability")
    );
}

#[test]
fn partial_contract_pair_and_fingerprint_changes_are_observed() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("auto");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(temp.path().join("contracts")).unwrap();
    std::fs::write(root.join("sample.at"), "pub fn test() int { return 1 }").unwrap();
    let a = loader::stdlib_assembly_fingerprint(&root, AssemblyTarget::Vm).unwrap();
    std::fs::write(
        temp.path().join("contracts/operations.toml"),
        "schema_version=1\noperations=[]",
    )
    .unwrap();
    assert!(loader::scan_inventory(&root)
        .diagnostics
        .iter()
        .any(|d| d.code == INVALID));
    assert!(loader::stdlib_assembly_fingerprint(&root, AssemblyTarget::Vm).is_err());
    std::fs::write(
        temp.path().join("contracts/compatibility.toml"),
        "schema_version=1\nentries=[]",
    )
    .unwrap();
    let b = loader::stdlib_assembly_fingerprint(&root, AssemblyTarget::Vm).unwrap();
    assert_ne!(a, b);
}
