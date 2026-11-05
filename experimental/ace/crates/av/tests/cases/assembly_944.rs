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

#[test]
fn rust_assembly_effect_and_restored_first_body_edit_use_real_fragment_producers() {
    use super::module_sources_950::rust_value;
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::ModuleA2xTarget;
    let temp = Temp::new();
    let store_path = temp.0.join("store");
    let (mut inputs, caps, av_manifest) = fixture();
    let mut query = SourceQuerySession::new("pkg", "app");
    let product =
        compile_assembly_product(&inputs, "app", &caps, &av_manifest, &mut query, B).unwrap();
    let manifest = Manifest::freeze(
        av_manifest.semantic().clone(),
        ModuleA2xTarget::consumer_for_product(&product, &RustTargetProvider::v1(), vec![]).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest.fingerprint(), av_manifest.fingerprint());
    assert_ne!(
        manifest.consumer_fingerprint(),
        av_manifest.consumer_fingerprint()
    );
    let mut rust = ModuleA2xTarget::cold_assembly(
        &store_path,
        "assembly944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &product,
        &manifest,
    )
    .unwrap();
    let mut source = ModuleSession::from_product(&product, query).unwrap();
    let mut store = ModuleStore::open(&store_path, "assembly944").unwrap();
    let mut dispatched = false;
    assert!(rust
        .run_authorized(&mut |_, _| Err("denied".into()), |_| {
            dispatched = true;
            Ok(())
        })
        .is_err());
    assert!(!dispatched);
    rust.run_authorized(&mut |_, _| Ok(()), |a| {
        rust_value(a, &temp.0.join("cold"), 42);
        Ok(())
    })
    .unwrap();
    let old = source
        .prepare_current_receipt(
            &mut store,
            &BTreeMap::from([("a2x-rust".into(), rust.receipt().unwrap())]),
        )
        .unwrap();
    store.publish(&old, None, false).unwrap();
    let local = source
        .bound
        .origins
        .iter()
        .find(|(_, o)| o.1 == "local")
        .unwrap()
        .0
        .clone();
    let old_local = rust.fragment_receipts()[&local].clone();
    let mut semantic = manifest.semantic().clone();
    semantic
        .declarations
        .iter_mut()
        .find(|d| d.module == "std")
        .unwrap()
        .requirements
        .push("console".into());
    let effect = Manifest::freeze(semantic, manifest.consumer().clone()).unwrap();
    let contexts = manifest_contexts(&source.bound, &effect).unwrap();
    let mut batch = source
        .prepare_batch(None, contexts.clone(), &BTreeSet::new(), B)
        .unwrap();
    let before = rust.stats().clone();
    let edit = rust.prepare_batch(&batch, &effect).unwrap();
    assert_eq!(rust.stats().lower - before.lower, 2);
    assert_eq!(rust.stats().print - before.print, 2);
    let stale = rust.prepare_batch(&batch, &effect).unwrap();
    let receipt = source
        .prepare_batch_receipt(
            &mut store,
            &mut batch,
            BTreeMap::from([("a2x-rust".into(), rust.prepared_receipt(&edit).unwrap())]),
        )
        .unwrap();
    source.validate_batch(&batch).unwrap();
    rust.validate(&edit).unwrap();
    store.publish(&receipt, Some(&old), false).unwrap();
    source.commit_batch(batch).unwrap();
    rust.commit(edit).unwrap();
    assert!(rust.validate(&stale).is_err());
    assert_eq!(rust.fragment_receipts()[&local], old_local);
    rust.run_authorized(
        &mut |_, r| {
            assert_eq!(r, ["console"]);
            Ok(())
        },
        |a| {
            rust_value(a, &temp.0.join("effect"), 42);
            Ok(())
        },
    )
    .unwrap();
    drop(rust);
    drop(source);
    drop(store);
    drop(product);
    let mut store = ModuleStore::open(&store_path, "assembly944").unwrap();
    let recovery = store
        .restore_with_contexts(&receipt, &inputs, "app", &caps, "pkg", "app", B, &contexts)
        .unwrap();
    assert_eq!(recovery.session.counters().solver_runs, 0);
    assert!(ModuleA2xTarget::restore(
        &store_path,
        "assembly944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &recovery.targets["a2x-rust"],
        &recovery.bound
    )
    .is_err());
    let mut rust = ModuleA2xTarget::restore_assembly(
        &store_path,
        "assembly944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &recovery.targets["a2x-rust"],
        &recovery.bound,
        &effect,
    )
    .unwrap();
    assert_eq!(rust.stats().lower, 0);
    assert_eq!(rust.stats().print, 0);
    assert!(rust
        .run_authorized(&mut |_, _| Err("fresh grant required".into()), |_| Err::<
            (),
            String,
        >(
            "must not dispatch".into()
        ))
        .is_err());
    let mut source = ModuleSession::from_recovery(recovery);
    let fragment = "pub fn answer() int { return 43 }";
    inputs[0].source = fragment.into();
    let mut semantic = effect.semantic().clone();
    semantic
        .sources
        .iter_mut()
        .find(|s| s.source_id == "modules/std.at")
        .unwrap()
        .content_hash = ace_core::runtime_contract::sha256_hex(fragment.as_bytes());
    let changed = Manifest::freeze(semantic, effect.consumer().clone()).unwrap();
    let mut batch = source
        .prepare_batch(
            Some(("std", "answer", fragment)),
            contexts.clone(),
            &BTreeSet::new(),
            B,
        )
        .unwrap();
    assert_eq!(source.query.counters().solver_runs, 1);
    let edit = rust.prepare_batch(&batch, &changed).unwrap();
    assert_eq!(rust.stats().lower, 1);
    assert_eq!(rust.stats().print, 1);
    let next = source
        .prepare_batch_receipt(
            &mut store,
            &mut batch,
            BTreeMap::from([("a2x-rust".into(), rust.prepared_receipt(&edit).unwrap())]),
        )
        .unwrap();
    source.validate_batch(&batch).unwrap();
    rust.validate(&edit).unwrap();
    store.publish(&next, Some(&receipt), false).unwrap();
    source.commit_batch(batch).unwrap();
    rust.commit(edit).unwrap();
    assert_eq!(rust.fragment_receipts()[&local], old_local);
    rust.run_authorized(&mut |_, _| Ok(()), |a| {
        rust_value(a, &temp.0.join("first-edit"), 43);
        Ok(())
    })
    .unwrap();
    drop(rust);
    drop(source);
    drop(store);
    let store = ModuleStore::open(&store_path, "assembly944").unwrap();
    let recovery = store
        .restore_with_contexts(&next, &inputs, "app", &caps, "pkg", "app", B, &contexts)
        .unwrap();
    assert_eq!(recovery.session.counters().solver_runs, 0);
    let rust = ModuleA2xTarget::restore_assembly(
        &store_path,
        "assembly944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &recovery.targets["a2x-rust"],
        &recovery.bound,
        &changed,
    )
    .unwrap();
    assert_eq!(rust.stats().lower, 0);
    assert_eq!(rust.stats().print, 0);
    rust.run_authorized(&mut |_, _| Ok(()), |a| {
        rust_value(a, &temp.0.join("next-read"), 43);
        Ok(())
    })
    .unwrap();
}

#[test]
fn two_consumer_host_publishes_one_root_and_publication_failure_preserves_both_executables() {
    use super::module_sources_950::rust_value;
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::ModuleA2xTarget;
    use ace_source_query::assembly::{AssemblyHost, AssemblyTarget};
    let temp = Temp::new();
    let path = temp.0.join("store");
    let (inputs, caps, av_manifest) = fixture();
    let mut query = SourceQuerySession::new("pkg", "app");
    let product =
        compile_assembly_product(&inputs, "app", &caps, &av_manifest, &mut query, B).unwrap();
    let rust_manifest = Manifest::freeze(
        av_manifest.semantic().clone(),
        ModuleA2xTarget::consumer_for_product(&product, &RustTargetProvider::v1(), vec![]).unwrap(),
    )
    .unwrap();
    let av = ModuleAvTarget::cold_assembly(&path, &product, &av_manifest).unwrap();
    let rust = ModuleA2xTarget::cold_assembly(
        &path,
        "host944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &product,
        &rust_manifest,
    )
    .unwrap();
    let source = ModuleSession::from_product(&product, query).unwrap();
    let store = ModuleStore::open(&path, "host944").unwrap();
    let mut host = AssemblyHost::publish_initial(source, store, av, rust).unwrap();
    let old = host.receipt().clone();
    let files = host.targets().1.current().files.clone();
    let old_av = host.targets().0.current().clone();
    let before = host.source().query.counters().solver_runs;
    assert!(host
        .prepare(None, B, |bound| {
            let mut semantic = av_manifest.semantic().clone();
            semantic.sources = source_inputs(bound);
            semantic
                .declarations
                .iter_mut()
                .find(|d| d.module == "std")
                .unwrap()
                .requirements
                .push("console".into());
            Ok((
                Manifest::freeze(semantic, av_manifest.consumer().clone())?,
                rust_manifest.clone(),
            ))
        })
        .err()
        .unwrap()
        .contains("common-identity"));
    assert_eq!(host.source().query.counters().solver_runs, before);
    // Empty-body/link-only candidates still belong to one transient host.
    // Matching root values never substitute for that ownership identity.
    let other_store = ModuleStore::open(&path, "host944").unwrap();
    let other_recovery = other_store
        .restore_with_contexts(
            &old,
            &inputs,
            "app",
            &caps,
            "pkg",
            "app",
            B,
            host.source().query.declaration_contexts(),
        )
        .unwrap();
    let other_av = ModuleAvTarget::restore_assembly(
        &path,
        &other_recovery.targets["av"],
        &other_recovery.bound,
        &av_manifest,
    )
    .unwrap();
    let other_rust = ModuleA2xTarget::restore_assembly(
        &path,
        "host944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &other_recovery.targets["a2x-rust"],
        &other_recovery.bound,
        &rust_manifest,
    )
    .unwrap();
    let mut other = AssemblyHost::from_recovery(
        ModuleSession::from_recovery(other_recovery),
        other_store,
        other_av,
        other_rust,
        old.clone(),
    )
    .unwrap();
    let lowered = host.targets().0.lowered;
    let printed = host.targets().1.stats().print;
    let foreign = host
        .prepare(None, B, |_| {
            let mut consumer = av_manifest.consumer().clone();
            consumer
                .providers
                .push(ace_semantics_contract::assembly::ProviderInput {
                    source_id: "providers/link-only".into(),
                    content_hash: ace_core::runtime_contract::sha256_hex(b"link-only-input"),
                    owners: vec![],
                });
            Ok((
                Manifest::freeze(av_manifest.semantic().clone(), consumer)?,
                rust_manifest.clone(),
            ))
        })
        .unwrap();
    assert_eq!(host.targets().0.lowered, lowered);
    assert_eq!(host.targets().1.stats().print, printed);
    assert!(other
        .publish(foreign, false)
        .unwrap_err()
        .contains("foreign-publication"));
    drop(other);
    let a = av_manifest.clone();
    let r = rust_manifest.clone();
    let factory = |bound: &ace_core::source::modules::BoundModules| {
        let mut semantic = a.semantic().clone();
        semantic.sources = source_inputs(bound);
        Ok((
            Manifest::freeze(semantic.clone(), a.consumer().clone())?,
            Manifest::freeze(semantic, r.consumer().clone())?,
        ))
    };
    assert!(host
        .prepare(
            Some(("std", "answer", "pub fn answer() int { return missing() }")),
            B,
            &factory
        )
        .is_err());
    let cancelled = host
        .prepare(
            Some(("std", "answer", "pub fn answer() int { return 43 }")),
            B,
            &factory,
        )
        .unwrap();
    assert!(host.publish(cancelled, true).is_err());
    let failed = host
        .prepare(
            Some(("std", "answer", "pub fn answer() int { return 43 }")),
            B,
            &factory,
        )
        .unwrap();
    let managed = ace_artifacts::store::ManagedStore::open(&path).unwrap();
    managed.db().execute_batch("CREATE TRIGGER fail_assembly944 BEFORE DELETE ON roots WHEN OLD.kind='source-modules-current' BEGIN SELECT RAISE(ABORT,'assembly944 transaction failure'); END;").unwrap();
    assert!(host
        .publish(failed, false)
        .unwrap_err()
        .contains("assembly944 transaction failure"));
    managed
        .db()
        .execute_batch("DROP TRIGGER fail_assembly944")
        .unwrap();
    assert_eq!(
        managed
            .root_artifacts("host944", "source-modules-current")
            .unwrap(),
        vec![old.artifact.clone()]
    );
    assert_eq!(host.receipt(), &old);
    assert_eq!(host.targets().1.current().files, files);
    assert!(Arc::ptr_eq(host.targets().0.current(), &old_av));
    assert_eq!(
        host.targets().0.run_authorized(&mut |_, _| Ok(())).unwrap(),
        42
    );
    host.targets()
        .1
        .run_authorized(&mut |_, _| Ok(()), |a| {
            rust_value(a, &temp.0.join("preserved"), 42);
            Ok(())
        })
        .unwrap();
    drop(host);
    drop(product);
    drop(managed);
    let store = ModuleStore::open(&path, "host944").unwrap();
    let contexts = BTreeMap::from(
        av_manifest
            .semantic()
            .declarations
            .iter()
            .map(|d| {
                let owner = if d.module == "app" && d.symbol == "main" {
                    "main".into()
                } else {
                    format!("m{}_{}_{}", d.module.len(), d.module, d.symbol)
                };
                (owner, d.version())
            })
            .collect::<BTreeMap<_, _>>(),
    );
    let recovery = store
        .restore_with_contexts(&old, &inputs, "app", &caps, "pkg", "app", B, &contexts)
        .unwrap();
    let av = ModuleAvTarget::restore_assembly(
        &path,
        &recovery.targets["av"],
        &recovery.bound,
        &av_manifest,
    )
    .unwrap();
    let rust = ModuleA2xTarget::restore_assembly(
        &path,
        "host944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &recovery.targets["a2x-rust"],
        &recovery.bound,
        &rust_manifest,
    )
    .unwrap();
    assert_eq!(av.lowered, 0);
    assert_eq!(rust.stats().lower, 0);
    let mut host =
        AssemblyHost::from_recovery(ModuleSession::from_recovery(recovery), store, av, rust, old)
            .unwrap();
    let current = host
        .prepare(
            Some(("std", "answer", "pub fn answer() int { return 43 }")),
            B,
            &factory,
        )
        .unwrap();
    host.publish(current, false).unwrap();
    assert_eq!(
        host.targets().0.run_authorized(&mut |_, _| Ok(())).unwrap(),
        43
    );
    host.targets()
        .1
        .run_authorized(&mut |_, _| Ok(()), |a| {
            rust_value(a, &temp.0.join("installed"), 43);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        host.targets().0.current_manifest().unwrap().fingerprint(),
        host.targets().1.current_manifest().unwrap().fingerprint()
    );
}

#[test]
fn provider_bytes_are_owner_and_consumer_scoped_and_restored_first_edit_executes_new_implementation(
) {
    use super::module_sources_950::rust_value;
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::ModuleA2xTarget;
    use ace_semantics_contract::assembly::ProviderInput;
    use ace_source_query::assembly::{AssemblyHost, AssemblyTarget};
    let temp = Temp::new();
    let path = temp.0.join("store");
    let (mut inputs, caps, initial_av) = fixture();
    let mut query = SourceQuerySession::new("pkg", "app");
    let product =
        compile_assembly_product(&inputs, "app", &caps, &initial_av, &mut query, B).unwrap();
    let initial_rust = Manifest::freeze(
        initial_av.semantic().clone(),
        ModuleA2xTarget::consumer_for_product(&product, &RustTargetProvider::v1(), vec![]).unwrap(),
    )
    .unwrap();
    let av = ModuleAvTarget::cold_assembly(&path, &product, &initial_av).unwrap();
    let rust = ModuleA2xTarget::cold_assembly(
        &path,
        "provider944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &product,
        &initial_rust,
    )
    .unwrap();
    let source = ModuleSession::from_product(&product, query).unwrap();
    let mut host = AssemblyHost::publish_initial(
        source,
        ModuleStore::open(&path, "provider944").unwrap(),
        av,
        rust,
    )
    .unwrap();
    let std_owner = host
        .source()
        .bound
        .origins
        .iter()
        .find(|(_, o)| o.0 == "std")
        .unwrap()
        .0
        .clone();
    let local = host
        .targets()
        .0
        .current()
        .code()
        .iter()
        .find(|c| c.name().ends_with("_local"))
        .unwrap()
        .clone();
    let old_code = host
        .targets()
        .0
        .current()
        .code()
        .iter()
        .find(|c| c.name() == format!("d_{std_owner}"))
        .unwrap()
        .clone();
    let rust_fragments = host.targets().1.fragment_receipts();
    let before_av = host.targets().0.lowered;
    let before_rust = host.targets().1.stats().clone();
    let before_solver = host.source().query.counters().solver_runs;
    let provider_file = temp.0.join("answer.provider");
    std::fs::write(
        &provider_file,
        b"physical provider revision two, same operation behavior",
    )
    .unwrap();
    let make = |bound: &ace_core::source::modules::BoundModules| {
        let mut semantic = initial_av.semantic().clone();
        semantic.sources = source_inputs(bound);
        let input = ProviderInput {
            source_id: "providers/answer.provider".into(),
            content_hash: ace_core::runtime_contract::sha256_hex(
                &std::fs::read(&provider_file).unwrap(),
            ),
            owners: vec![std_owner.clone()],
        };
        Ok((
            Manifest::freeze(semantic.clone(), assembly_consumer(vec![input]))?,
            Manifest::freeze(semantic, initial_rust.consumer().clone())?,
        ))
    };
    let candidate = host.prepare(None, B, &make).unwrap();
    assert_eq!(host.source().query.counters().solver_runs, before_solver);
    assert_eq!(
        host.targets().0.lowered - before_av,
        1,
        "only the AV provider owner is actually lowered"
    );
    assert_eq!(host.targets().1.stats().lower, before_rust.lower);
    assert_eq!(host.targets().1.stats().print, before_rust.print);
    host.publish(candidate, false).unwrap();
    assert_eq!(host.targets().1.fragment_receipts(), rust_fragments);
    assert!(Arc::ptr_eq(
        &local,
        host.targets()
            .0
            .current()
            .code()
            .iter()
            .find(|c| c.name().ends_with("_local"))
            .unwrap()
    ));
    assert!(!Arc::ptr_eq(
        &old_code,
        host.targets()
            .0
            .current()
            .code()
            .iter()
            .find(|c| c.name() == format!("d_{std_owner}"))
            .unwrap()
    ));
    assert_eq!(
        host.targets().0.run_authorized(&mut |_, _| Ok(())).unwrap(),
        42
    );
    host.targets()
        .1
        .run_authorized(&mut |_, _| Ok(()), |a| {
            rust_value(a, &temp.0.join("provider-only"), 42);
            Ok(())
        })
        .unwrap();
    let receipt = host.receipt().clone();
    let contexts = host.source().query.declaration_contexts().clone();
    let (expected_av, expected_rust) = make(&host.source().bound).unwrap();
    drop(host);
    drop(product);
    let store = ModuleStore::open(&path, "provider944").unwrap();
    let recovery = store
        .restore_with_contexts(&receipt, &inputs, "app", &caps, "pkg", "app", B, &contexts)
        .unwrap();
    assert_eq!(recovery.session.counters().solver_runs, 0);
    assert!(ModuleAvTarget::restore_assembly(
        &path,
        &recovery.targets["av"],
        &recovery.bound,
        &initial_av
    )
    .is_err());
    let av = ModuleAvTarget::restore_assembly(
        &path,
        &recovery.targets["av"],
        &recovery.bound,
        &expected_av,
    )
    .unwrap();
    let rust = ModuleA2xTarget::restore_assembly(
        &path,
        "provider944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &recovery.targets["a2x-rust"],
        &recovery.bound,
        &expected_rust,
    )
    .unwrap();
    assert_eq!(av.lowered, 0);
    assert_eq!(rust.stats().lower, 0);
    assert_eq!(rust.stats().print, 0);
    let mut host = AssemblyHost::from_recovery(
        ModuleSession::from_recovery(recovery),
        store,
        av,
        rust,
        receipt,
    )
    .unwrap();
    let fragment = "pub fn answer() int { return 44 }";
    std::fs::write(&provider_file, fragment.as_bytes()).unwrap();
    let candidate = host
        .prepare(Some(("std", "answer", fragment)), B, &make)
        .unwrap();
    assert_eq!(host.source().query.counters().solver_runs, 1);
    assert_eq!(host.targets().0.lowered, 1);
    assert_eq!(host.targets().1.stats().lower, 1);
    assert_eq!(host.targets().1.stats().print, 1);
    host.publish(candidate, false).unwrap();
    assert_eq!(
        host.targets().0.run_authorized(&mut |_, _| Ok(())).unwrap(),
        44
    );
    host.targets()
        .1
        .run_authorized(&mut |_, _| Ok(()), |a| {
            rust_value(a, &temp.0.join("provider-and-body"), 44);
            Ok(())
        })
        .unwrap();
    inputs[0].source = fragment.into();
    let next = host.receipt().clone();
    let (a, r) = make(&host.source().bound).unwrap();
    assert_eq!(a.fingerprint(), r.fingerprint());
    drop(host);
    let store = ModuleStore::open(&path, "provider944").unwrap();
    let recovery = store
        .restore_with_contexts(&next, &inputs, "app", &caps, "pkg", "app", B, &contexts)
        .unwrap();
    let av = ModuleAvTarget::restore_assembly(&path, &recovery.targets["av"], &recovery.bound, &a)
        .unwrap();
    let rust = ModuleA2xTarget::restore_assembly(
        &path,
        "provider944",
        "pkg",
        256 * 1024 * 1024,
        RustTargetProvider::v1(),
        &recovery.targets["a2x-rust"],
        &recovery.bound,
        &r,
    )
    .unwrap();
    assert_eq!(recovery.session.counters().solver_runs, 0);
    assert_eq!(av.lowered, 0);
    assert_eq!(rust.stats().lower, 0);
    assert_eq!(
        av.current_manifest().unwrap().consumer().providers[0].content_hash,
        ace_core::runtime_contract::sha256_hex(fragment.as_bytes())
    );
    assert_eq!(av.run_authorized(&mut |_, _| Ok(())).unwrap(), 44);
    rust.run_authorized(&mut |_, _| Ok(()), |a| {
        rust_value(a, &temp.0.join("next-provider-read"), 44);
        Ok(())
    })
    .unwrap();
}
