//! Export actual audited standard-operation evidence to the backend-free
//! assembly contract. This never compiles ACE source with the legacy parser.
use super::{
    contracts, loader,
    model::{LayerKind, VerificationLevel},
    reference, validate,
};
use crate::vm::native::NativeInterface;
use ace_semantics_contract::assembly::{OperationReference, ReferenceLevel, Signature};

pub fn current_operation_reference(
    operation: &str,
    interface: &NativeInterface,
) -> Result<OperationReference, String> {
    let root = loader::repo_stdlib_root().map_err(|e| e.to_string())?;
    let mut inventory = loader::scan_inventory(&root);
    contracts::audit_providers(&mut inventory, interface);
    if inventory
        .diagnostics
        .iter()
        .any(|d| d.code == contracts::INVALID)
    {
        return Err("assembly.operation-contract".into());
    }
    let op = inventory
        .contracts
        .operations
        .iter()
        .find(|o| o.id == operation)
        .ok_or("assembly.unknown-operation")?;
    if op.visibility != "public" || op.layer != LayerKind::Public {
        return Err("assembly.operation-not-public".into());
    }
    let symbol = inventory
        .modules
        .iter()
        .find(|m| m.module == op.module)
        .into_iter()
        .flat_map(|m| &m.layers)
        .filter(|l| l.kind == LayerKind::Public)
        .flat_map(|l| &l.symbols)
        .find(|s| s.name == op.symbol && s.is_pub)
        .ok_or("assembly.missing-public-declaration")?;
    let name = validate::public_native_name(&op.module, symbol);
    let id = interface
        .resolve(&name)
        .ok_or("assembly.reference-unbound")?;
    let proof = reference::verify_native_reference(id, interface)?
        .ok_or("assembly.reference-unverified")?;
    let s = &proof.public_signature;
    if proof.module != op.module
        || proof.symbol != op.symbol
        || proof.verification != VerificationLevel::SignatureChecked
        || s.parameters != op.parameters
        || s.parameter_modes != op.modes
        || s.returns != op.returns
    {
        return Err("assembly.operation-signature-drift".into());
    }
    // file!() in existing adapter contracts includes the build checkout root.
    // Strip only the known source prefix; preserve the real relative producer
    // location and all contract contents. Native numeric IDs are not operation
    // identities and are deliberately absent from this portable record.
    let raw = proof.producer.replace('\\', "/");
    let marker = "crates/auto-lang-core/";
    let pos = raw.find(marker).ok_or("assembly.nonportable-adapter")?;
    let producer = raw[pos..].replace(':', "#");
    let mut adapter = proof.selected_adapter;
    adapter.producer = producer.clone();
    let contract = serde_json::to_vec(&adapter).map_err(|e| e.to_string())?;
    Ok(OperationReference {
        verification: ReferenceLevel::SignatureChecked,
        operation: op.id.clone(),
        semantic_version: op.semantic_version,
        module: op.module.clone(),
        symbol: op.symbol.clone(),
        signature: Signature {
            parameters: s.parameters.clone(),
            modes: s.parameter_modes.clone(),
            returns: s.returns.clone(),
        },
        requirements: op.requirements.clone(),
        source_id: proof.declaration,
        declaration_hash: format!("{:016x}", proof.declaration_hash),
        adapter: proof.callee,
        producer,
        adapter_contract_hash: blake3::hash(&contract).to_hex().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_public_reference_retains_proof_without_claiming_ace_execution() {
        let mut interface = NativeInterface::production();
        let proof = current_operation_reference("io.read_line", &interface).unwrap();
        assert_eq!(proof.module, "io");
        assert_eq!(proof.symbol, "read_line");
        assert_eq!(proof.signature.returns, "str");
        assert_eq!(proof.requirements, ["stdin"]);
        assert!(!proof.producer.contains("D:/"));
        assert!(proof.producer.starts_with("crates/auto-lang-core/"));
        assert!(current_operation_reference("text.byte_length", &interface).is_err());
        assert!(current_operation_reference("missing", &interface).is_err());
        // Replacing a final selected adapter with a contradictory independent
        // contract must fail the same actual public reference path.
        interface.declare_contract_identity("auto.io.read_line", &[], &["i32"], "i32");
        assert!(current_operation_reference("io.read_line", &interface).is_err());
    }
}
