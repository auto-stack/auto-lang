//! Actual module target assembly mode; executable identity and authorization
//! are observed at the original AV dispatch path, not model-only counters.
#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc"))]
use ace_av::modules::{assembly_consumer, ModuleAvTarget};
use ace_core::source::{
    cst::FrontBudget,
    modules::{bind_modules, ModuleInput},
};
use ace_semantics_contract::assembly::{
    Declaration, Manifest, SemanticAssembly, Signature, SCALAR_MODULES,
};
use ace_source_query::{
    assembly::{
        compile_assembly_product, manifest_contexts, source_inputs, ASSEMBLY_HOST,
        ASSEMBLY_PLATFORM,
    },
    modules::{ModuleSession, ModuleStore},
    SourceQuerySession,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
const B: FrontBudget = FrontBudget::WORK_OWNED_64MIB;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        Self(
            std::env::var_os("AUTO_PLAN_ARTIFACTS")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join(format!(
                    "assembly944-av-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                )),
        )
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn fixture() -> (Vec<ModuleInput>, Vec<String>, Manifest) {
    let inputs = vec![
        ModuleInput {
            id: "std".into(),
            source: "pub fn answer() int { return 42 }".into(),
        },
        ModuleInput {
            id: "app".into(),
            source: "use std.answer\nfn local() int { return 7 } fn main() int { return answer() }"
                .into(),
        },
    ];
    let caps = vec![SCALAR_MODULES.into()];
    let bound = bind_modules(&inputs, "app", &caps, B).unwrap();
    let declarations = [("std", "answer"), ("app", "local"), ("app", "main")]
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
        host: ASSEMBLY_HOST.into(),
        platform: ASSEMBLY_PLATFORM.into(),
        declarations,
        references: vec![],
        sources: source_inputs(&bound),
    };
    (
        inputs,
        caps,
        Manifest::freeze(semantic, assembly_consumer(vec![])).unwrap(),
    )
}
#[test]
fn av_assembly_effect_update_preserves_unrelated_code_and_rechecks_authority_after_restore() {
    let temp = Temp::new();
    let (inputs, caps, manifest) = fixture();
    let mut query = SourceQuerySession::new("pkg", "app");
    let product =
        compile_assembly_product(&inputs, "app", &caps, &manifest, &mut query, B).unwrap();
    let mut av = ModuleAvTarget::cold_assembly(&temp.0, &product, &manifest).unwrap();
    let mut source = ModuleSession::from_product(&product, query).unwrap();
    let mut store = ModuleStore::open(&temp.0, "assembly944").unwrap();
    assert!(av.run().is_err());
    assert_eq!(
        av.run_authorized(&mut |host, requirements| {
            assert_eq!(host, ASSEMBLY_HOST);
            assert!(requirements.is_empty());
            Ok(())
        })
        .unwrap(),
        42
    );
    let old = source
        .prepare_current_receipt(
            &mut store,
            &BTreeMap::from([("av".into(), av.receipt().unwrap())]),
        )
        .unwrap();
    store.publish(&old, None, false).unwrap();
    let local = av
        .current()
        .code()
        .iter()
        .find(|c| c.name().ends_with("_local"))
        .unwrap()
        .clone();
    let mut semantic = manifest.semantic().clone();
    semantic
        .declarations
        .iter_mut()
        .find(|d| d.module == "std")
        .unwrap()
        .requirements
        .push("console".into());
    let effect = Manifest::freeze(semantic, assembly_consumer(vec![])).unwrap();
    let contexts = manifest_contexts(&source.bound, &effect).unwrap();
    let before = av.lowered;
    let mut batch = source
        .prepare_batch(None, contexts.clone(), &BTreeSet::new(), B)
        .unwrap();
    let edit = av.prepare_batch(&batch, &effect).unwrap();
    assert_eq!(av.lowered - before, 2);
    let stale = av.prepare_batch(&batch, &effect).unwrap();
    let receipt = source
        .prepare_batch_receipt(
            &mut store,
            &mut batch,
            BTreeMap::from([("av".into(), av.prepared_receipt(&edit).unwrap())]),
        )
        .unwrap();
    source.validate_batch(&batch).unwrap();
    av.validate(&edit).unwrap();
    store.publish(&receipt, Some(&old), false).unwrap();
    source.commit_batch(batch).unwrap();
    av.commit(edit).unwrap();
    assert!(
        av.validate(&stale).is_err(),
        "same source hash does not authorize an old assembly candidate"
    );
    assert!(Arc::ptr_eq(
        &local,
        av.current()
            .code()
            .iter()
            .find(|c| c.name().ends_with("_local"))
            .unwrap()
    ));
    let mut checks = 0;
    assert!(av
        .run_authorized(&mut |_, r| {
            checks += 1;
            assert_eq!(r, ["console"]);
            Err("revoked".into())
        })
        .is_err());
    assert_eq!(checks, 1);
    assert_eq!(
        av.run_authorized(&mut |_, r| {
            assert_eq!(r, ["console"]);
            Ok(())
        })
        .unwrap(),
        42
    );
    drop(av);
    drop(source);
    drop(store);
    drop(product);
    let store = ModuleStore::open(&temp.0, "assembly944").unwrap();
    let (recovery, syntax) = ace_core::source::parser::capture_syntax_work(|| {
        store
            .restore_with_contexts(&receipt, &inputs, "app", &caps, "pkg", "app", B, &contexts)
            .unwrap()
    });
    assert!(syntax.tokenized_ranges.is_empty());
    assert!(syntax.parsed_functions.is_empty());
    assert_eq!(recovery.session.counters().solver_runs, 0);
    assert!(ModuleAvTarget::restore(&temp.0, &recovery.targets["av"], &recovery.bound).is_err());
    assert!(ModuleAvTarget::restore_assembly(
        &temp.0,
        &recovery.targets["av"],
        &recovery.bound,
        &manifest
    )
    .is_err());
    let av = ModuleAvTarget::restore_assembly(
        &temp.0,
        &recovery.targets["av"],
        &recovery.bound,
        &effect,
    )
    .unwrap();
    assert_eq!(av.lowered, 0);
    assert!(av
        .run_authorized(&mut |_, _| Err("restored grants are absent".into()))
        .is_err());
    assert_eq!(av.run_authorized(&mut |_, _| Ok(())).unwrap(), 42);
}
