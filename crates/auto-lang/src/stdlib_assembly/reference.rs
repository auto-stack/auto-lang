//! Proof for the callee selected at a real VM emission/dispatch point.
use std::collections::BTreeSet;

use super::model::{LayerKind, ParseStatus, SymbolKind, VerificationLevel};
use crate::vm::native::NativeInterface;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReferenceProof {
    pub module: String,
    pub symbol: String,
    pub declaration: String,
    pub declaration_hash: u64,
    pub producer: String,
    pub native_id: Option<u16>,
    pub callee: String,
    pub verification: VerificationLevel,
    pub public_signature: super::model::LogicalSignature,
    pub selected_adapter: crate::vm::native::NativeContract,
}

/// Walk linked instruction boundaries from the actual task entry, following
/// branches, direct calls, closures, futures and generator/task entries. This
/// happens against the final interface before the first business instruction.
pub fn verify_linked_native_closure(
    flash: &crate::vm::virt_memory::VirtualFlash,
    entry: usize,
    interface: &NativeInterface,
    strings: &[Vec<u8>],
) -> Result<Vec<ReferenceProof>, String> {
    use crate::vm::opcode::OpCode;
    let mut pending = vec![entry];
    let mut visited = BTreeSet::new();
    let mut natives = BTreeSet::new();
    while let Some(ip) = pending.pop() {
        if ip >= flash.memory.len() || !visited.insert(ip) {
            continue;
        }
        let byte = flash.read_u8(ip);
        if !OpCode::is_valid(byte) {
            return Err(format!("STDASSEMBLY.INVALID_BYTECODE: {ip}"));
        }
        let op = OpCode::from(byte);
        let operands = match op {
            OpCode::CREATE_FUTURE => 5 + 2 * flash.read_u8(ip + 5) as usize,
            _ => crate::vm::disasm::Disassembler::new(flash).instruction_operands(op, ip),
        };
        let next = ip + 1 + operands;
        if next > flash.memory.len() {
            return Err(format!(
                "STDASSEMBLY.INVALID_BYTECODE: truncated {op:?} at {ip}"
            ));
        }
        match op {
            OpCode::CALL_NAT | OpCode::CALL_NAT_COUNTED => {
                natives.insert(flash.read_u16(ip + 1));
            }
            OpCode::CALL
            | OpCode::CLOSURE
            | OpCode::CREATE_FUTURE
            | OpCode::CREATE_GENERATOR
            | OpCode::SPAWN => {
                pending.push(flash.read_u32(ip + 1) as usize);
            }
            OpCode::CALL_SPEC => {
                if let Some(name) = strings.get(flash.read_u32(ip + 1) as usize) {
                    let method = String::from_utf8_lossy(name);
                    for (name, address) in &flash.exports_by_name {
                        if name == method.as_ref() || name.ends_with(&format!(".{method}")) {
                            pending.push(*address as usize);
                        }
                    }
                }
            }
            OpCode::JMP | OpCode::JMP_IF_Z | OpCode::JMP_IF_NZ | OpCode::PUSH_HANDLER => {
                let delta = i16::from_le_bytes([flash.read_u8(ip + 1), flash.read_u8(ip + 2)]);
                pending.push(next.wrapping_add_signed(delta as isize));
            }
            OpCode::JMP_L | OpCode::JMP_FAR => {
                pending.push(next.wrapping_add_signed(flash.read_i32(ip + 1) as isize));
            }
            _ => {}
        }
        if !matches!(
            op,
            OpCode::HALT | OpCode::RET | OpCode::JMP | OpCode::JMP_L | OpCode::JMP_FAR
        ) {
            pending.push(next);
        }
    }
    let mut proofs = Vec::new();
    for id in natives {
        if let Some(proof) = interface.verify_core_reference(id)? {
            proofs.push(proof);
        }
    }
    Ok(proofs)
}

/// Inventory claims do not enter this proof. The native ID comes from actual
/// emission, and the contract comes from the final selected interface.
pub fn verify_native_reference(
    id: u16,
    interface: &NativeInterface,
) -> Result<Option<ReferenceProof>, String> {
    let names: BTreeSet<String> = {
        let registry = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
        crate::vm::native_catalog::NATIVE_ID_ENTRIES
            .iter()
            .filter(|(_, native)| *native == id)
            .map(|(name, _)| name.to_string())
            .chain(
                registry
                    .get_function_names()
                    .into_iter()
                    .filter(|name| registry.get_id(name) == Some(id)),
            )
            .collect()
    };
    let modules: BTreeSet<&str> = names
        .iter()
        .filter_map(|name| {
            let module = name.strip_prefix("auto.")?.split('.').next()?;
            let module = if module == "http_stream" {
                "http"
            } else {
                module
            };
            super::validate::CORE_MODULES
                .contains(&module)
                .then_some(module)
        })
        .collect();
    if modules.is_empty() {
        return Ok(None);
    }
    let root = super::loader::repo_stdlib_root().map_err(|e| e.to_string())?;
    for module in modules {
        let declaration = format!("stdlib/auto/{module}.at");
        let text = std::fs::read_to_string(root.join(format!("{module}.at")))
            .map_err(|e| format!("STDINV.READ_FAIL: {declaration}: {e}"))?;
        let layer = super::loader::parse_layer(&text, LayerKind::Public, declaration.clone());
        if let ParseStatus::Failed { error } = &layer.parse {
            return Err(format!("STDINV.PARSE_FAIL: {declaration}: {error}"));
        }
        for symbol in layer
            .symbols
            .iter()
            .filter(|s| s.is_pub && matches!(s.kind, SymbolKind::Fn | SymbolKind::Method))
        {
            // These IDs implement the legacy string-wire JSON helpers, not
            // JsonValue receiver methods. An ID alias cannot grant a public
            // method proof to a different calling convention.
            if module == "json"
                && symbol.name.starts_with("JsonValue.")
                && interface
                    .contract(id)
                    .and_then(|contract| contract.receiver.as_ref())
                    .is_none()
            {
                continue;
            }
            let native_name = super::validate::public_native_name(module, symbol);
            let selected_id = crate::vm::native_registry::BIGVM_NATIVES
                .lock()
                .unwrap()
                .peek_qualified(&native_name)
                .or_else(|| interface.resolve(&native_name));
            if selected_id != Some(id) {
                continue;
            }
            let label = format!("{module}.{} ({declaration}, native #{id})", symbol.name);
            if let Some((_, previous, next)) = interface
                .binding_conflicts
                .iter()
                .find(|(n, _, _)| *n == id)
            {
                return Err(format!(
                    "STDASSEMBLY.NATIVE_ID_CONFLICT: {label}: {previous} / {next}"
                ));
            }
            if interface.get(id).is_none() {
                return Err(format!("STDASSEMBLY.PROVIDER_CLAIM_NO_CALLEE: {label}"));
            }
            let contract = interface.contract(id).ok_or_else(|| format!(
                "STDASSEMBLY.SIGNATURE_UNVERIFIED: {label}: selected callee lacks an independent adapter contract"))?;
            super::validate::signature_matches(module, symbol, contract, LayerKind::Public)
                .map_err(|reason| format!("STDASSEMBLY.SIGNATURE_DRIFT: {label}: {reason}"))?;
            return Ok(Some(ReferenceProof {
                module: module.into(),
                symbol: symbol.name.clone(),
                declaration,
                declaration_hash: layer.content_hash,
                producer: contract.producer.clone(),
                native_id: Some(id),
                callee: native_name,
                verification: VerificationLevel::SignatureChecked,
                public_signature: symbol.signature.clone().unwrap(),
                selected_adapter: contract.clone(),
            }));
        }
    }
    // Legacy private intrinsics are outside the public core contract. They
    // do not acquire a public SignatureChecked claim through this path.
    Ok(None)
}

/// Dry compilation uses the actual emitters and performs no business execution.
/// Dependency declarations come from the resolved session, not a catalog scan.
pub fn compile_actual_references(
    session: &crate::compile::CompileSession,
    source: &str,
    path: &std::path::Path,
) -> crate::AutoResult<Vec<ReferenceProof>> {
    use crate::stdlib_assembly::model::AssemblyTarget;
    let snapshot = std::sync::Arc::new(std::sync::RwLock::new(
        session.type_store().read().unwrap().clone(),
    ));
    let mut parser = crate::parser::Parser::new_with_type_store(source, snapshot.clone());
    parser.set_dest(match session.assembly.target {
        AssemblyTarget::Vm => crate::parser::CompileDest::Interp,
        AssemblyTarget::Rust => crate::parser::CompileDest::TransRust,
        AssemblyTarget::C => crate::parser::CompileDest::TransC,
    });
    let ast = parser.parse()?;
    let mut references = session.assembly_references.clone();
    match session.assembly.target {
        AssemblyTarget::Vm => {
            let mut generator = crate::vm::codegen::Codegen::new_with_type_store(snapshot);
            generator.assembly_context = session.assembly;
            for statement in &ast.stmts {
                generator.compile_stmt(statement)?;
            }
            references.extend(generator.assembly_references);
        }
        AssemblyTarget::Rust => {
            let (_, proofs) = super::emission::emit_rust_assembly(session, source, path)?;
            references.extend(proofs);
        }
        AssemblyTarget::C => {
            super::emission::emit_c_assembly(session, source, path)?;
        }
    }
    Ok(references)
}

#[cfg(test)]
mod plan738_reference {
    use super::*;

    #[test]
    fn final_interface_is_checked_before_any_business_instruction() {
        use crate::vm::{
            engine::AutoVM, opcode::OpCode, task::AutoTask, virt_memory::VirtualFlash,
        };
        // Print is the first instruction. The later public parse call loses
        // its producer contract after code emission, before VM execution.
        let flash = VirtualFlash::new_with_code(vec![
            OpCode::CALL_NAT as u8,
            0x01,
            0x00,
            OpCode::CALL_NAT as u8,
            0x6e,
            0x07,
            OpCode::HALT as u8,
        ]);
        let (mut vm, output) = AutoVM::new_with_capture(flash, 1024);
        std::sync::Arc::make_mut(&mut vm.native_interface).register_static(1902, |_, _| Ok(()));
        let error = vm
            .run_one_instruction(&mut AutoTask::new(1, 1024, 0))
            .unwrap_err();
        assert!(
            format!("{error:?}").contains("SIGNATURE_UNVERIFIED"),
            "{error:?}"
        );
        assert!(output.read().unwrap().is_empty());
    }

    #[test]
    fn legacy_wire_json_helpers_do_not_inherit_receiver_proofs() {
        let native = NativeInterface::production();
        assert!(native.verify_core_reference(1906).unwrap().is_none());
        let (result, _) = crate::run_with_capture("json.get(\"{\\\"x\\\":42}\", \"x\")").unwrap();
        assert!(result.contains("42"), "{result}");
    }

    #[test]
    fn production_emission_refuses_unproved_tcp_read_before_execution() {
        let error = crate::run_with_capture("net.tcp_stream_read(0, [0])").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("STDASSEMBLY.SIGNATURE_UNVERIFIED"),
            "{error:?}"
        );
        assert!(error.to_string().contains("TcpStream.read"), "{error:?}");
    }

    #[test]
    fn selected_json_override_materializes_real_heap_value() {
        let (result, _) =
            crate::run_with_capture("let d = json.parse(\"{\\\"answer\\\":42}\")\nd.answer")
                .unwrap();
        assert_eq!(result, "42");
        let interface = NativeInterface::production();
        let proof = interface.verify_core_reference(1902).unwrap().unwrap();
        assert_eq!(proof.symbol, "parse");
        assert_eq!(proof.selected_adapter.returns, "JsonValue?");
        assert!(proof.producer.contains("vm/native.rs"));
    }

    #[test]
    fn final_merge_invalidates_cached_signature_proof() {
        let mut interface = NativeInterface::production();
        assert!(interface.verify_core_reference(1902).unwrap().is_some());
        let mut untyped = NativeInterface::new();
        untyped.register_static(1902, |_, _| Ok(()));
        interface.merge(&untyped);
        assert!(interface
            .verify_core_reference(1902)
            .unwrap_err()
            .contains("SIGNATURE_UNVERIFIED"));
    }
}
