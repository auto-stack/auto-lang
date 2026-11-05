//! The versioned assembly contract enters the ordinary finite-module lane.
//! No parser, solver, module binder or HIR emitter is duplicated here.
use crate::{
    catalog::SignatureCatalog,
    modules::{compile_bound_modules, ModuleProduct},
    SourceQuerySession,
};
use ace_core::source::{
    ast::{Item, ParamMode},
    cst::FrontBudget,
    modules::{bind_modules, BoundModules, ModuleInput},
};
use ace_semantics_contract::assembly::{Manifest, Signature, SourceInput};
use std::collections::BTreeMap;

pub fn source_inputs(bound: &BoundModules) -> Vec<SourceInput> {
    let mut sources: Vec<_> = bound
        .segments
        .iter()
        .map(|s| SourceInput {
            source_id: format!("modules/{}.at", s.module),
            content_hash: ace_core::runtime_contract::sha256_hex(s.source.as_bytes()),
        })
        .collect();
    sources.sort_by(|a, b| a.source_id.cmp(&b.source_id));
    sources
}

/// Compare every explicit declaration with the actual admitted source graph.
/// Missing declarations are rejected rather than silently treated as pure.
pub fn manifest_contexts(
    bound: &BoundModules,
    manifest: &Manifest,
) -> Result<BTreeMap<String, String>, String> {
    manifest.validate()?;
    let s = manifest.semantic();
    if s.sources != source_inputs(bound) {
        return Err("assembly.source-inputs".into());
    }
    if s.declarations.len() != bound.origins.len() {
        return Err("assembly.declaration-coverage".into());
    }
    let catalog = SignatureCatalog::build(&bound.program, "assembly", "modules")
        .map_err(|e| e.to_string())?;
    let mut result = BTreeMap::new();
    for declaration in &s.declarations {
        let (owner, _) = bound
            .origins
            .iter()
            .find(|(_, o)| o.0 == declaration.module && o.1 == declaration.symbol)
            .ok_or("assembly.unknown-declaration")?;
        let facts = catalog.get(owner).ok_or("assembly.declaration-signature")?;
        let item = bound
            .program
            .items
            .iter()
            .find_map(|i| match i {
                Item::Fn(f) if &f.name.name == owner => Some(f),
                _ => None,
            })
            .ok_or("assembly.declaration-source")?;
        let actual = Signature {
            parameters: facts
                .params
                .iter()
                .map(|(_, t)| t.as_str().to_string())
                .collect(),
            modes: item
                .params
                .iter()
                .map(|p| {
                    match p.mode {
                        ParamMode::Default => "View",
                        ParamMode::Mut => "Mut",
                        ParamMode::Move => "Take",
                    }
                    .into()
                })
                .collect(),
            returns: facts.ret.as_str().into(),
        };
        if declaration.signature != actual {
            return Err(format!(
                "assembly.signature-mismatch: {}.{}",
                declaration.module, declaration.symbol
            ));
        }
        if result
            .insert(owner.clone(), declaration.version())
            .is_some()
        {
            return Err("assembly.duplicate-owner".into());
        }
    }
    for provider in &manifest.consumer().providers {
        if provider
            .owners
            .iter()
            .any(|owner| !bound.origins.contains_key(owner))
        {
            return Err("assembly.provider-owner".into());
        }
    }
    Ok(result)
}

/// Ordinary source -> actual namespace/visibility -> declaration/effect keys ->
/// the existing fact-driven checked product. Invalid assembly inputs never
/// reach a semantic body producer or either backend.
pub fn compile_assembly_product(
    inputs: &[ModuleInput],
    entry: &str,
    caps: &[String],
    manifest: &Manifest,
    query: &mut SourceQuerySession,
    budget: FrontBudget,
) -> Result<ModuleProduct, String> {
    let bound = bind_modules(inputs, entry, caps, budget).map_err(|e| format!("{e:?}"))?;
    let contexts = manifest_contexts(&bound, manifest)?;
    query
        .set_declaration_contexts(&bound.program, contexts)
        .map_err(|e| e.to_string())?;
    compile_bound_modules(bound, caps, query).map_err(|e| format!("{e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_semantics_contract::assembly::{
        Consumer, Declaration, SemanticAssembly, AV_PRODUCER, SCALAR_MODULES,
    };
    fn fixture() -> (Vec<ModuleInput>, Vec<String>, Manifest) {
        let inputs = vec![ModuleInput {id:"std".into(),source:"pub fn answer() int { return 42 }".into()},
            ModuleInput {id:"app".into(),source:"use std.answer\nfn answer() int { return 7 } fn main() int { return std.answer() }".into()}];
        let caps = vec![SCALAR_MODULES.into()];
        let bound = bind_modules(&inputs, "app", &caps, FrontBudget::WORK_OWNED_64MIB).unwrap();
        let declarations = [("std", "answer"), ("app", "answer"), ("app", "main")]
            .into_iter()
            .map(|(module, symbol)| Declaration {
                module: module.into(),
                symbol: symbol.into(),
                semantic_version: 1,
                signature: Signature {
                    parameters: vec![],
                    modes: vec![],
                    returns: "i32".into(),
                },
                requirements: vec![],
            })
            .collect();
        let semantic = SemanticAssembly {
            profile: SCALAR_MODULES.into(),
            host: "explicit-finite/1".into(),
            platform: "x86_64-pc-windows-msvc".into(),
            declarations,
            references: vec![],
            sources: source_inputs(&bound),
        };
        let consumer = Consumer {
            producer: AV_PRODUCER.into(),
            abi: "scalar-i32/1".into(),
            runtime: "av/1".into(),
            toolchain: "av/1".into(),
            providers: vec![],
        };
        (inputs, caps, Manifest::freeze(semantic, consumer).unwrap())
    }
    #[test]
    fn assembly_contract_uses_real_module_bindings_and_rejects_stale_or_missing_declarations_before_solve(
    ) {
        let (inputs, caps, manifest) = fixture();
        let mut query = SourceQuerySession::new("pkg", "app");
        let product = compile_assembly_product(
            &inputs,
            "app",
            &caps,
            &manifest,
            &mut query,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        let contexts = manifest_contexts(&product.bound, &manifest).unwrap();
        let std_owner = product
            .bound
            .origins
            .iter()
            .find(|(_, o)| o.0 == "std")
            .unwrap()
            .0;
        let local_owner = product
            .bound
            .origins
            .iter()
            .find(|(_, o)| o.0 == "app" && o.1 == "answer")
            .unwrap()
            .0;
        assert_ne!(std_owner, local_owner);
        assert_ne!(contexts[std_owner], contexts[local_owner]);
        assert_eq!(query.counters().solver_runs, 3);
        for mutation in ["missing", "signature", "mode", "source", "namespace"] {
            let mut semantic = manifest.semantic().clone();
            match mutation {
                "missing" => {
                    semantic.declarations.pop();
                }
                "signature" => semantic.declarations[0].signature.returns = "bool".into(),
                "mode" => {
                    semantic.declarations[0]
                        .signature
                        .parameters
                        .push("i32".into());
                    semantic.declarations[0].signature.modes.push("Mut".into());
                }
                "source" => semantic.sources[0].content_hash = "stale".into(),
                _ => semantic.declarations[0].module = "unknown".into(),
            }
            let bad = Manifest::freeze(semantic, manifest.consumer().clone()).unwrap();
            let mut query = SourceQuerySession::new("pkg", "app");
            assert!(
                compile_assembly_product(
                    &inputs,
                    "app",
                    &caps,
                    &bad,
                    &mut query,
                    FrontBudget::WORK_OWNED_64MIB
                )
                .is_err(),
                "{mutation}"
            );
            assert_eq!(query.counters().solver_runs, 0, "{mutation}");
        }
    }
}
