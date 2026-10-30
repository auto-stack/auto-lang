#[path = "../common/mod.rs"]
mod common;
use ace_native::{
    collections_incremental::CollectionsIncrementalSession,
    collections_observe::{decode, Outcome},
    driver::MnDriver,
};
fn directory(tag: &str) -> std::path::PathBuf {
    let path = common::repo_root()
        .join("docs/plans/attachments/889")
        .join(format!(
            "incremental-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
const OLD:&str="fn helper() int { var xs=List<int>.new()\n xs.push(42)\n return xs.get(0) }\nfn main() int { return helper() }";
const NEW:&str="fn helper() int { var xs=List<Str>.new()\n xs.push(\"中文\")\n let value ?int=Some(41)\n is value { Some(n) -> { return n+xs.get(0).len() } None -> { return 0 } } }\nfn main() int { return helper() }";
#[test]
fn collections_native_single_body_probe_preserves_real_producers_and_history() {
    let dir = directory("first-probe");
    let driver = MnDriver::with_store(&dir.join("store"), "p889").unwrap();
    let (mut session, first) =
        CollectionsIncrementalSession::new(OLD, "main", driver, &dir.join("output"), None).unwrap();
    assert_eq!(
        decode(first.exit_code, &first.stderr).unwrap().outcome,
        Outcome::Return(42)
    );
    let identity = session.identities();
    let original_fp = session.unit_fingerprints();
    for (source, expected, codegen) in [(NEW, 43, 1), (OLD, 42, 0)] {
        let run = session.apply_source(source).unwrap();
        assert_eq!(
            decode(run.exit_code, &run.stderr).unwrap().outcome,
            Outcome::Return(expected)
        );
        assert_eq!(
            run.work
                .obs
                .counters
                .get("codegen_emit_units")
                .copied()
                .unwrap_or(0),
            codegen
        );
        assert_eq!(
            session.identities()["main"],
            identity["main"],
            "main source/MIR/COFF producers must be exactly the retained objects"
        );
        assert_eq!(
            run.work
                .syntax
                .parsed_functions
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            vec!["helper"]
        );
        assert_eq!(
            run.work.bodies.keys().cloned().collect::<Vec<_>>(),
            vec!["helper"]
        );
        let body = run.work.bodies["helper"];
        assert_eq!(
            (body.resolve, body.typecheck, body.plan, body.bind),
            (1, 1, 1, 1)
        );
        assert_eq!(run.work.obs.counters.get("mir_lower:d_helper"), Some(&1));
        assert!(!run.work.obs.counters.contains_key("mir_lower:d_main"));
        assert_eq!(session.unit_fingerprints()["d_main"], original_fp["d_main"]);
        if source == OLD {
            assert_eq!(session.unit_fingerprints(), original_fp);
        }
    }
    assert_eq!(session.revision(), 2);
    let hit = session.apply_source(OLD).unwrap();
    assert!(hit.work.syntax.parsed_functions.is_empty() && hit.work.bodies.is_empty());
    assert!(!hit.work.obs.counters.contains_key("codegen_emit_units"));
    assert_eq!(hit.exit_code, Some(42));
}

#[test]
fn collections_incremental_utf8_coordinates_cancel_stale_and_failures_keep_current() {
    let dir = directory("coordinates-and-rollback");
    let cache = dir.join("store");
    let output = dir.join("output");
    let (mut session, _) = CollectionsIncrementalSession::new(
        OLD,
        "main",
        MnDriver::with_store(&cache, "p889").unwrap(),
        &output,
        None,
    )
    .unwrap();
    let main_identity = session.identities()["main"];
    let old_spans = session.mapped_spans("main").unwrap();
    for (source, value) in [
        (NEW.to_string(), 43),
        (NEW.replace("var xs", "// 中文🙂\n var xs"), 43),
        (OLD.to_string(), 42),
    ] {
        let run = session.apply_source(&source).unwrap();
        assert_eq!(run.exit_code, Some(value));
        assert_eq!(session.identities()["main"], main_identity);
        let spans = session.mapped_spans("main").unwrap();
        assert_eq!(spans.len(), old_spans.len());
        for ((old_instruction, old), (new_instruction, new)) in old_spans.iter().zip(&spans) {
            assert_eq!(old_instruction, new_instruction);
            assert_eq!(
                &OLD[old.start as usize..old.end as usize],
                &source[new.start as usize..new.end as usize]
            );
            assert!(new.start as usize >= source.rfind("fn main").unwrap());
        }
        if source.contains('🙂') {
            let store = ace_artifacts::store::ManagedStore::open_existing(&cache).unwrap();
            let manifest = ace_native::collections_artifacts::get_manifest(
                &store,
                session.manifest_id().unwrap(),
            )
            .unwrap();
            assert_eq!(manifest.schema, "ac889/a10-collections/3");
            assert_eq!(
                ace_native::collections_artifacts::get_blob(&store, &manifest.source, "source")
                    .unwrap(),
                source.as_bytes()
            );
            let coordinate = manifest
                .coordinates
                .iter()
                .find(|c| c.unit == "d_main")
                .unwrap();
            assert_eq!(coordinate.current_start, source.rfind("fn main").unwrap());
            assert_ne!(coordinate.current_start, coordinate.mint_start);
            let mut restarted = MnDriver::with_store(&cache, "p889-readback").unwrap();
            restarted.forbid_codegen = true;
            let restored = restarted.request_restored_collections(
                session.manifest_id().unwrap(),
                &dir.join("restart-current-coordinates"),
                None,
            );
            assert!(restored.ok, "{:?}", restored.error);
            assert_eq!(restored.exit_code, Some(43));
            assert_eq!(
                restored.obs.counters.get("restored_source_coordinates"),
                Some(&2)
            );
        }
    }
    let original_source = session.source().to_string();
    let ids = session.identities();
    let fp = session.manifest_fingerprint().to_string();
    let manifest = session.manifest_id().unwrap().to_string();
    let exe = std::fs::read(output.join("collections.exe")).unwrap();
    let receipt = std::fs::read(output.join(".mn-collections-link.txt")).unwrap();
    let cancelled = session.prepare_source(NEW).unwrap();
    assert_eq!(session.source(), original_source);
    cancelled.cancel();
    assert_eq!(session.identities(), ids);
    assert_eq!(session.manifest_fingerprint(), fp);
    let bad = NEW.replace("Some(41)", "Some(\"wrong\")");
    assert!(session.prepare_source(&bad).is_err());
    assert_eq!(session.source(), original_source);
    assert_eq!(session.identities(), ids);
    assert_eq!(session.manifest_id(), Some(manifest.as_str()));
    session.test_omit_runtime(true);
    let bad_link = session
        .apply_source(NEW)
        .err()
        .expect("real omitted-runtime link must fail");
    assert!(matches!(bad_link, ace_native::MnError::Infra(_)));
    assert_eq!(
        session.last_work().obs.counters.get("link_invocations"),
        Some(&1)
    );
    assert!(!session
        .last_work()
        .obs
        .counters
        .contains_key("run_invocations"));
    session.test_omit_runtime(false);
    assert_eq!(session.source(), original_source);
    assert_eq!(session.identities(), ids);
    assert_eq!(session.manifest_fingerprint(), fp);
    assert_eq!(std::fs::read(output.join("collections.exe")).unwrap(), exe);
    assert_eq!(
        std::fs::read(output.join(".mn-collections-link.txt")).unwrap(),
        receipt
    );
    let first = session.prepare_source(NEW).unwrap();
    let stale = session
        .prepare_source(&NEW.replace("Some(41)", "Some(42)"))
        .unwrap();
    assert_eq!(session.commit_prepared(first).unwrap().exit_code, Some(43));
    let current = session.manifest_fingerprint().to_string();
    assert!(session.commit_prepared(stale).is_err());
    assert_eq!(session.source(), NEW);
    assert_eq!(session.manifest_fingerprint(), current);
    let (mut foreign, _) = CollectionsIncrementalSession::new(
        OLD,
        "main",
        MnDriver::with_store(&cache, "other-session").unwrap(),
        &dir.join("other-output"),
        None,
    )
    .unwrap();
    let token = foreign.prepare_source(NEW).unwrap();
    assert!(session.commit_prepared(token).is_err());
    assert_eq!(session.source(), NEW);
    assert_eq!(foreign.apply_source(OLD).unwrap().exit_code, Some(42));
    session.close();
    assert!(session.identities().is_empty());
    assert!(session.prepare_source(OLD).is_err());
}

#[test]
fn collections_declaration_invalidation_rebuilds_only_incompatible_native_units() {
    let old="type Item { name Str; code int }\nenum Token { Data(Item)\nEnd }\nfn make() Token { return Token.Data(Item(\"中\",41)) }\nfn main() int { let t=make()\n is t { Token.Data(p) -> { return p.name.len()+p.code } Token.End -> { return 0 } } }\nfn untouched() int { return 5 }";
    let new = old.replace("Data(Item)\nEnd", "End\nData(Item)");
    let dir = directory("declaration-invalidation");
    let (mut session, first) = CollectionsIncrementalSession::new(
        old,
        "main",
        MnDriver::with_store(&dir.join("store"), "p889").unwrap(),
        &dir.join("output"),
        None,
    )
    .unwrap();
    assert_eq!(first.exit_code, Some(42));
    let identities = session.identities();
    let fp = session.unit_fingerprints();
    assert!(
        session.prepare_source(&new).is_err(),
        "body path must not silently switch to whole frontend"
    );
    let prepared = session.prepare_declaration_change(&new).unwrap();
    assert!(prepared.is_declaration_invalidation());
    let result = session.commit_prepared(prepared).unwrap();
    assert_eq!(result.exit_code, Some(42));
    assert_eq!(
        result
            .work
            .obs
            .counters
            .get("declaration_frontend_invalidations"),
        Some(&1)
    );
    assert_eq!(result.work.syntax.parsed_functions.len(), 3);
    assert_eq!(result.work.obs.counters.get("codegen_emit_units"), Some(&2));
    assert_ne!(session.unit_fingerprints()["d_make"], fp["d_make"]);
    assert_ne!(session.unit_fingerprints()["d_main"], fp["d_main"]);
    let now = session.identities();
    assert_eq!(now["untouched"].1, identities["untouched"].1);
    assert_eq!(now["untouched"].2, identities["untouched"].2);
    assert_eq!(
        session.unit_fingerprints()["d_untouched"],
        fp["d_untouched"]
    );
}

#[test]
fn collections_incremental_actual_whole_work_and_same_content_copy_controls_turn_red() {
    let dir = directory("counter-controls");
    let output = dir.join("output");
    let (mut session, _) = CollectionsIncrementalSession::new(
        OLD,
        "main",
        MnDriver::with_store(&dir.join("store"), "p889").unwrap(),
        &output,
        None,
    )
    .unwrap();
    let identity = session.identities();
    let fp = session.manifest_fingerprint().to_string();
    for fault in 0..3 {
        session.negative_controls.whole_frontend = fault == 0;
        session.negative_controls.whole_lower = fault == 1;
        session.negative_controls.copy_unchanged_code = fault == 2;
        let error = session
            .prepare_source(NEW)
            .err()
            .expect("a real degeneration must fail the structural gate");
        let work = session.last_work();
        if fault == 0 {
            assert_eq!(
                work.obs.counters.get("negative_control_whole_frontend"),
                Some(&1)
            );
            assert_eq!(work.syntax.parsed_functions.get("main"), Some(&1));
        }
        if fault == 1 {
            assert_eq!(
                work.obs.counters.get("negative_control_lower:main"),
                Some(&1)
            );
            assert_eq!(work.bodies["main"].plan, 1);
        }
        if fault == 2 {
            assert_eq!(
                work.obs.counters.get("negative_control_same_content_copy"),
                Some(&1)
            );
            assert!(error.message().contains("lost producer identity"));
        }
        assert_eq!(session.source(), OLD);
        assert_eq!(session.identities(), identity);
        assert_eq!(session.manifest_fingerprint(), fp);
    }
    session.negative_controls = Default::default();
    assert_eq!(session.apply_source(NEW).unwrap().exit_code, Some(43));
}

#[test]
fn collections_incremental_runtime_budget_and_deadline_keep_previous_publication() {
    let dir = directory("execution-failures");
    let cache = dir.join("store");
    let output = dir.join("output");
    let (mut session, _) = CollectionsIncrementalSession::new(
        OLD,
        "main",
        MnDriver::with_store(&cache, "p889").unwrap(),
        &output,
        None,
    )
    .unwrap();
    let identities = session.identities();
    let fp = session.manifest_fingerprint().to_string();
    let id = session.manifest_id().unwrap().to_string();
    let exe = std::fs::read(output.join("collections.exe")).unwrap();
    let receipt = std::fs::read(output.join(".mn-collections-link.txt")).unwrap();
    let resource="fn helper() int { var xs=List<int>.new()\n var i=0\n loop { if i>=65537 { break }\n xs.push(i)\n i=i+1 }\n return 42 }\nfn main() int { return helper() }";
    let error = session
        .apply_source(resource)
        .err()
        .expect("MAX_LIST+1 must fail");
    assert_eq!(error.code(), "mn.resource.execution-budget");
    let run = session
        .last_failure_run()
        .expect("runtime failure must retain its real raw evidence");
    let observed = decode(run.exit_code, &run.stderr).unwrap();
    assert_eq!(observed.outcome, Outcome::Trap(22));
    assert_eq!(observed.resources["live"], 0);
    assert_eq!(observed.resources["registered_bytes"], 0);
    let infinite="fn helper() int { var flag=true\n loop { if flag == false { break } }\n return 42 }\nfn main() int { return helper() }";
    let error = session
        .apply_source(infinite)
        .err()
        .expect("contained infinite loop must hit real deadline");
    assert!(matches!(error, ace_native::MnError::Infra(_)), "{error:?}");
    assert_eq!(
        session.last_work().obs.counters.get("run_invocations"),
        Some(&1)
    );
    assert!(
        session.last_failure_run().is_none(),
        "a killed process has no observed cleanup summary"
    );
    assert_eq!(session.source(), OLD);
    assert_eq!(session.revision(), 0);
    assert_eq!(session.identities(), identities);
    assert_eq!(session.manifest_fingerprint(), fp);
    assert_eq!(session.manifest_id(), Some(id.as_str()));
    assert_eq!(std::fs::read(output.join("collections.exe")).unwrap(), exe);
    assert_eq!(
        std::fs::read(output.join(".mn-collections-link.txt")).unwrap(),
        receipt
    );
    assert_eq!(session.apply_source(OLD).unwrap().exit_code, Some(42));
}

#[test]
fn collections_frozen_resident_source_load_preserves_115_unchanged_producers() {
    let load = ace_native::collections_workload::generate(0);
    assert_eq!(
        load.source,
        include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/N889Load.at")
    );
    let dir = directory("frozen-first-probe");
    let baseline = ace_av::mempeek::sample_private_bytes();
    let sampler = ace_native::resources::TreePeakSampler::start(5);
    let started = std::time::Instant::now();
    let (mut session, first) = CollectionsIncrementalSession::new(
        &load.source,
        "main",
        MnDriver::with_store(&dir.join("store"), "p889").unwrap(),
        &dir.join("output"),
        None,
    )
    .unwrap();
    let cold_ms = started.elapsed().as_millis();
    assert_eq!(first.exit_code, Some(load.expected));
    let identities = session.identities();
    assert_eq!(identities.len(), 116);
    for revision in [1, 10] {
        let load = ace_native::collections_workload::generate(revision);
        let run = session.apply_source(&load.source).unwrap();
        assert_eq!(run.exit_code, Some(load.expected));
        assert_eq!(run.work.obs.counters.get("codegen_emit_units"), Some(&1));
        assert_eq!(
            run.work
                .syntax
                .parsed_functions
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            vec!["collections_helper"]
        );
        assert_eq!(
            run.work.bodies.keys().cloned().collect::<Vec<_>>(),
            vec!["collections_helper"]
        );
        for (name, identity) in session.identities() {
            if name != "collections_helper" {
                assert_eq!(identity, identities[&name], "{name}");
            }
        }
        assert!(run.work.obs.ledger.violations().is_empty());
        println!(
            "frozen-native body revision={revision} wall_ms={} {}",
            run.work.elapsed_ms,
            run.work.obs.ledger.render()
        );
    }
    let peak = sampler.stop();
    let delta = peak.own_peak.saturating_sub(baseline);
    println!("frozen-native cold_ms={cold_ms} baseline={baseline} peak={} delta={delta} tree={} ticks={} child_ticks={} {} {}",peak.own_peak,peak.tree_peak,peak.ticks,peak.child_ticks,first.work.obs.ledger.render(),first.work.obs.render());
    assert!(baseline > 0 && peak.ticks > 0 && peak.child_ticks > 0 && peak.own_peak > 0);
    assert!(delta <= 128 << 20);
    session.close();
}

#[test]
fn collections_prepared_runtime_drift_refuses_before_publication() {
    let dir = directory("prepared-runtime-drift");
    let store = dir.join("store");
    let output = dir.join("output");
    // Corrupt only an owned copy. A process-local mutex cannot protect
    // sibling nextest processes using the shared product runtime.
    let shared = ace_native::collections_runtime::prepare(None)
        .unwrap()
        .library;
    let shared_bytes = std::fs::read(&shared).unwrap();
    let library = dir.join("owned-collections-runtime.lib");
    std::fs::copy(&shared, &library).unwrap();
    let (mut session, first) = CollectionsIncrementalSession::new(
        OLD,
        "main",
        MnDriver::with_store(&store, "p889").unwrap(),
        &output,
        Some(&library),
    )
    .unwrap();
    assert_eq!(first.exit_code, Some(42));
    let prepared = session
        .prepare_source(&OLD.replace("push(42)", "push(43)"))
        .unwrap();
    let old_source = session.source().to_owned();
    let old_id = session.identities();
    let old_manifest = session.manifest_id().map(str::to_owned);
    let exe = std::fs::read(&first.build.exe_path).unwrap();
    let receipt = std::fs::read(&first.build.receipt_path).unwrap();
    let db = ace_artifacts::store::ManagedStore::open_existing(&store).unwrap();
    let roots = db.root_artifacts("p889", "soft").unwrap();
    struct Restore {
        path: std::path::PathBuf,
        bytes: Vec<u8>,
    }
    impl Drop for Restore {
        fn drop(&mut self) {
            std::fs::write(&self.path, &self.bytes).unwrap();
        }
    }
    let saved = Restore {
        bytes: std::fs::read(&library).unwrap(),
        path: library.clone(),
    };
    let mut bad = saved.bytes.clone();
    let mid = bad.len() / 2;
    bad[mid] ^= 1;
    std::fs::write(&library, bad).unwrap();
    let error = session
        .commit_prepared(prepared)
        .err()
        .expect("same-sized changed library must refuse a previously prepared candidate");
    assert!(error.code().contains("runtime"), "{error:?}");
    assert_eq!(session.source(), old_source);
    assert_eq!(session.identities(), old_id);
    assert_eq!(session.manifest_id(), old_manifest.as_deref());
    assert_eq!(std::fs::read(&first.build.exe_path).unwrap(), exe);
    assert_eq!(std::fs::read(&first.build.receipt_path).unwrap(), receipt);
    assert_eq!(db.root_artifacts("p889", "soft").unwrap(), roots);
    let run = ace_native::link_driver::run_pe(
        &first.build.exe_path,
        ace_native::config::RUN_BUDGET,
        ace_core::link::CaptureCaps {
            stdout_max: ace_native::config::STREAM_CAP,
            stderr_max: ace_native::config::STREAM_CAP,
        },
    )
    .unwrap();
    assert_eq!(run.exit_code, Some(42));
    drop(saved);
    let run = session
        .apply_source(&OLD.replace("push(42)", "push(43)"))
        .unwrap();
    assert_eq!(run.exit_code, Some(43));
    assert_eq!(run.work.obs.counters.get("cache_hit_coff"), Some(&1));
    assert_eq!(std::fs::read(&shared).unwrap(), shared_bytes);
    session.close();
}
