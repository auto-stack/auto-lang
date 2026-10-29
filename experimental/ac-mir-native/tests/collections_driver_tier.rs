mod common;
use ac_artifact_probe::store::ManagedStore;
use ac_mn::{
    collections_observe::{decode, Outcome},
    driver::{CacheMode, MnDriver},
};
use std::path::PathBuf;
fn directory(tag: &str) -> PathBuf {
    let path = common::repo_root()
        .join("docs/plans/attachments/889")
        .join(format!(
            "driver-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
const SOURCE:&str="enum Token { Data(Str)\nEnd }\nfn make() Token { return Token.Data(\"hello\") }\nfn main() int { var xs=List<Str>.new()\n xs.push(\"中文\")\n let t=make()\n is t { Token.Data(s) -> { return xs.get(0).len()+s.len()+35 }\nToken.End -> { return 0 } } }";
#[test]
fn collections_driver_fresh_hit_restart_and_read_only_are_real_native_runs() {
    let dir = directory("cache");
    let cache = dir.join("store");
    let mut driver = MnDriver::with_store(&cache, "p889").unwrap();
    let fresh = driver.request_collections(SOURCE, "d_main", &dir.join("output"), None);
    assert!(fresh.ok, "{:?}", fresh.error);
    assert_eq!(
        decode(fresh.exit_code, &fresh.stderr).unwrap().outcome,
        Outcome::Return(42)
    );
    assert_eq!(fresh.obs.counters.get("codegen_emit_units"), Some(&2));
    let id = fresh.manifest_artifact_id.unwrap();
    driver.forbid_codegen = true;
    let hit = driver.request_collections(SOURCE, "d_main", &dir.join("output"), None);
    assert!(hit.ok, "{:?}", hit.error);
    assert_eq!(hit.obs.counters.get("cache_hit_coff"), Some(&2));
    assert!(!hit.obs.counters.contains_key("codegen_emit_units"));
    assert_eq!(hit.obs.counters.get("runtime_cargo_invocations"), Some(&0));
    assert_eq!(hit.exit_code, Some(42));
    drop(driver);
    let mut restarted = MnDriver::with_store(&cache, "p889").unwrap();
    restarted.forbid_codegen = true;
    let run = restarted.request_restored_collections(&id, &dir.join("restart"), None);
    assert!(run.ok, "{:?}", run.error);
    assert_eq!(run.exit_code, Some(42));
    assert_eq!(
        run.obs.counters.get("restored_mir_units_readmitted"),
        Some(&2)
    );
    assert!(!run.obs.counters.contains_key("codegen_emit_units"));
    assert!(!run.obs.counters.contains_key("frontend_real_body_plans"));
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_mn-worker"));
    child
        .arg("collections-restore-run")
        .arg("--store")
        .arg(&cache)
        .arg("--workdir")
        .arg(dir.join("new-process"))
        .arg("--manifest-id")
        .arg(&id)
        .args(["--project", "p889", "--read-only"]);
    let child = auto_ac_prototype::link::run_deadlined_captured(
        &mut child,
        ac_mn::config::REQUEST_BUDGET,
        auto_ac_prototype::link::CaptureCaps {
            stdout_max: ac_mn::config::STREAM_CAP,
            stderr_max: ac_mn::config::STREAM_CAP,
        },
    )
    .unwrap();
    assert_eq!(
        child.exit_code,
        Some(0),
        "{}",
        String::from_utf8_lossy(&child.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&child.stdout).unwrap();
    assert_eq!(response["protocol"], "mncollections-restore/1");
    assert_eq!(response["exit_code"], 42);
    assert_eq!(response["counters"]["restored_mir_units_readmitted"], 2);
    assert!(response["counters"].get("codegen_emit_units").is_none());
    let store = ManagedStore::open_existing(&cache).unwrap();
    let roots = store.root_artifacts("p889", "soft").unwrap();
    let blocks = store.space().blocks_count;
    restarted.cache_mode = CacheMode::ReadOnly;
    let ro = restarted.request_restored_collections(&id, &dir.join("ro"), None);
    assert!(ro.ok, "{:?}", ro.error);
    assert_eq!(ro.exit_code, Some(42));
    assert!(ro.manifest_artifact_id.is_none());
    assert_eq!(store.root_artifacts("p889", "soft").unwrap(), roots);
    assert_eq!(store.space().blocks_count, blocks);
    // A read-only source miss still emits and executes, without new pages or roots.
    restarted.forbid_codegen = false;
    let miss = restarted.request_collections(
        &SOURCE.replace("+35", "+36"),
        "main",
        &dir.join("ro-miss"),
        None,
    );
    assert!(miss.ok, "{:?}", miss.error);
    assert_eq!(miss.exit_code, Some(43));
    assert_eq!(miss.obs.counters.get("codegen_emit_units"), Some(&1));
    assert_eq!(store.space().blocks_count, blocks);
    assert_eq!(store.root_artifacts("p889", "soft").unwrap(), roots);
    drop(restarted);
    let mut next_process = MnDriver::with_store(&cache, "p889").unwrap();
    let published = next_process.request_collections(
        &SOURCE.replace("+35", "+36"),
        "main",
        &dir.join("next-publication"),
        None,
    );
    assert!(published.ok, "{:?}", published.error);
    assert_eq!(
        store.root_artifacts("p889", "soft").unwrap(),
        vec![published.manifest_artifact_id.unwrap()]
    );
}
#[test]
fn collections_failed_candidate_keeps_previous_pe_receipt_and_root() {
    let dir = directory("rollback");
    let cache = dir.join("store");
    let output = dir.join("output");
    let mut driver = MnDriver::with_store(&cache, "p889").unwrap();
    let good = driver.request_collections(SOURCE, "d_main", &output, None);
    assert!(good.ok, "{:?}", good.error);
    let exe = std::fs::read(output.join("collections.exe")).unwrap();
    let receipt = std::fs::read(output.join(".mn-collections-link.txt")).unwrap();
    let store = ManagedStore::open_existing(&cache).unwrap();
    let roots = store.root_artifacts("p889", "soft").unwrap();
    for source in [
        "fn main(hi int) int { return hi }",
        "fn main() int { return missing() }",
    ] {
        let bad = driver.request_collections(source, "d_main", &output, None);
        assert!(!bad.ok);
        assert!(!bad.obs.counters.contains_key("codegen_emit_units"));
        assert_eq!(std::fs::read(output.join("collections.exe")).unwrap(), exe);
        assert_eq!(
            std::fs::read(output.join(".mn-collections-link.txt")).unwrap(),
            receipt
        );
        assert_eq!(store.root_artifacts("p889", "soft").unwrap(), roots);
    }
    let resource=driver.request_collections("fn main() int { var xs=List<int>.new()\n var i=0\n loop { if i>=65537 { break }\n xs.push(i)\n i=i+1 }\n return 42 }","d_main",&output,None);
    assert!(
        !resource.ok,
        "resource overflow must fail before publication"
    );
    assert_eq!(resource.obs.counters.get("run_invocations"), Some(&1));
    assert_eq!(std::fs::read(output.join("collections.exe")).unwrap(), exe);
    assert_eq!(
        std::fs::read(output.join(".mn-collections-link.txt")).unwrap(),
        receipt
    );
    assert_eq!(store.root_artifacts("p889", "soft").unwrap(), roots);
    // A user directory at the receipt destination fails final placement after
    // a real native build/run; the prior PE and root still survive unchanged.
    std::fs::remove_file(output.join(".mn-collections-link.txt")).unwrap();
    std::fs::create_dir(output.join(".mn-collections-link.txt")).unwrap();
    let failed = driver.request_collections(&SOURCE.replace("+35", "+36"), "d_main", &output, None);
    assert!(!failed.ok);
    assert_eq!(failed.obs.counters.get("run_invocations"), Some(&1));
    assert_eq!(std::fs::read(output.join("collections.exe")).unwrap(), exe);
    assert!(output.join(".mn-collections-link.txt").is_dir());
    assert_eq!(store.root_artifacts("p889", "soft").unwrap(), roots);
}

#[test]
fn collections_restore_rejects_all_forged_identity_axes_before_link_or_codegen() {
    use ac_mn::collections_artifacts as art;
    let dir = directory("admission");
    let cache = dir.join("store");
    let output = dir.join("output");
    let mut driver = MnDriver::with_store(&cache, "p889").unwrap();
    let good = driver.request_collections(SOURCE, "d_main", &output, None);
    assert!(good.ok, "{:?}", good.error);
    let store = ManagedStore::open_existing(&cache).unwrap();
    let manifest =
        art::get_manifest(&store, good.manifest_artifact_id.as_deref().unwrap()).unwrap();
    let exe = std::fs::read(output.join("collections.exe")).unwrap();
    let roots = store.root_artifacts("p889", "soft").unwrap();
    for case in 0..23 {
        let mut forged = manifest.clone();
        match case {
            0 => forged.schema.push('!'),
            1 => forged.profile.push('!'),
            2 => forged.abi.push('!'),
            3 => forged.target.push('!'),
            4 => forged.codegen_build.push('!'),
            5 => forged.owner_module.push('!'),
            6 => forged.entry = "absent".into(),
            7 => forged.entry_symbol.push('!'),
            8 => forged.runtime.abi.push('!'),
            9 => forged.runtime.build_key.push('!'),
            10 => forged.runtime.content_hash.push('!'),
            11 => forged.runtime_blob.hash.push('!'),
            12 => forged.runtime_blob.pages[0] = manifest.units[0].mir.pages[0].clone(),
            13 => forged.units[0].meta.runtime.push('!'),
            14 => forged.units[0].meta.flags.push('!'),
            15 => forged.units[0].meta.mir_schema.push('!'),
            16 => forged.units[0].meta.owner_schema.push('!'),
            17 => forged.units[0].coff_hash.push('!'),
            18 => forged.units[0].coff_len += 1,
            19 => forged.units[0].mir.pages[0] = manifest.runtime_blob.pages[0].clone(),
            20 => forged.units[0].mir = art::put_blob(&store, "mir", b"not MIR").unwrap(),
            21 => forged.units.push(forged.units[0].clone()),
            22 => forged.lld.push('!'),
            _ => unreachable!(),
        }
        let id = art::put_manifest(&store, &forged).unwrap();
        let rejected = driver.request_restored_collections(&id, &output, None);
        assert!(!rejected.ok, "case {case} accepted");
        assert!(
            !rejected.obs.counters.contains_key("link_invocations"),
            "case {case}"
        );
        assert!(!rejected.obs.counters.contains_key("codegen_emit_units"));
        assert_eq!(
            std::fs::read(output.join("collections.exe")).unwrap(),
            exe,
            "case {case}"
        );
        assert_eq!(
            store.root_artifacts("p889", "soft").unwrap(),
            roots,
            "case {case}"
        );
    }
}

#[test]
fn collections_product_driver_runs_all_original_sources_and_owned_combinations() {
    let dir = directory("source-domain");
    let mut driver = MnDriver::ephemeral();
    for (id, value) in [
        ("L01", 42),
        ("L02", 42),
        ("L03", 0),
        ("L04", 42),
        ("L05", 42),
        ("L06", 70),
        ("L07", 42),
        ("V01", 42),
        ("V02", 42),
        ("V03", 3),
        ("V04", 42),
        ("V05", 42),
        ("V06", 42),
        ("V07", 42),
        ("LSTR864", 51),
        ("FIN864A", 42),
        ("FIN864B", 53),
    ] {
        let source = std::fs::read_to_string(
            common::repo_root().join(format!("crates/auto-lang/test/cases/ace-m1/src/{id}.at")),
        )
        .unwrap();
        let result = driver.request_collections(&source, "main", &dir.join(id), None);
        assert!(result.ok, "{id}: {:?}", result.error);
        let observed = decode(result.exit_code, &result.stderr).unwrap();
        assert_eq!(
            observed.outcome,
            if id == "L06" {
                Outcome::Trap(21)
            } else {
                Outcome::Return(value)
            },
            "{id}"
        );
    }
    for (tag, source) in [
        (
            "list-variant",
            include_str!("../../ac-core/tests/fixtures/collections_889/list_variant.at"),
        ),
        (
            "nested-variant",
            include_str!("../../ac-core/tests/fixtures/collections_889/nested_variant.at"),
        ),
        (
            "optional-variant",
            include_str!("../../ac-core/tests/fixtures/collections_889/optional_variant.at"),
        ),
    ] {
        let result = driver.request_collections(source, "main", &dir.join(tag), None);
        assert!(result.ok, "{tag}: {:?}", result.error);
        assert_eq!(
            decode(result.exit_code, &result.stderr).unwrap().outcome,
            Outcome::Return(42)
        );
    }
    let returned = driver.request_collections(
        "fn main() int { return 70 }",
        "main",
        &dir.join("return70"),
        None,
    );
    assert!(returned.ok, "{:?}", returned.error);
    assert_eq!(
        decode(returned.exit_code, &returned.stderr)
            .unwrap()
            .outcome,
        Outcome::Return(70)
    );
}

#[test]
fn collections_cli_builds_new_profile_and_preserves_output_on_refusal() {
    let dir = directory("cli");
    let input = dir.join("case.at");
    let output = dir.join("case.exe");
    std::fs::write(&input, SOURCE).unwrap();
    let invoke = |backend: &str| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_ac-native"));
        command
            .arg("build-source")
            .arg(&input)
            .args([
                "--source-profile",
                "collections",
                "--entry",
                "main",
                "--output",
            ])
            .arg(&output)
            .env("AC_NATIVE_BACKEND", backend);
        auto_ac_prototype::link::run_deadlined_captured(
            &mut command,
            ac_mn::config::REQUEST_BUDGET,
            auto_ac_prototype::link::CaptureCaps {
                stdout_max: ac_mn::config::STREAM_CAP,
                stderr_max: ac_mn::config::STREAM_CAP,
            },
        )
        .unwrap()
    };
    let built = invoke("mn");
    assert_eq!(
        built.exit_code,
        Some(0),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let pe = ac_mn::link_driver::run_pe(
        &output,
        ac_mn::config::RUN_BUDGET,
        auto_ac_prototype::link::CaptureCaps {
            stdout_max: ac_mn::config::STREAM_CAP,
            stderr_max: ac_mn::config::STREAM_CAP,
        },
    )
    .unwrap();
    assert_eq!(
        decode(pe.exit_code, &pe.stderr).unwrap().outcome,
        Outcome::Return(42)
    );
    let exe = std::fs::read(&output).unwrap();
    let receipt = std::fs::read(dir.join("case.ac-link.txt")).unwrap();
    assert!(String::from_utf8_lossy(&receipt).contains("mn.collections.cell/2"));
    for backend in ["legacy", "collections=legacy", "collections=typo"] {
        assert_ne!(invoke(backend).exit_code, Some(0));
        assert_eq!(std::fs::read(&output).unwrap(), exe);
        assert_eq!(
            std::fs::read(dir.join("case.ac-link.txt")).unwrap(),
            receipt
        );
    }
    std::fs::write(&input, "fn main() int { return absent() }").unwrap();
    assert_ne!(invoke("mn").exit_code, Some(0));
    assert_eq!(std::fs::read(&output).unwrap(), exe);
}

#[test]
fn collections_paged_coff_hit_keeps_required_closure_and_bounds_page_reads() {
    use ac_mn::{cache, collections_artifacts as art};
    let dir = directory("pages");
    let cache_root = dir.join("store");
    let store = ManagedStore::open(&cache_root).unwrap();
    let source = "fn main() int { return 42 }";
    let cold = ac_mn::collections_frontend::cold(source, "mn.at").unwrap();
    let unit = &cold.units[0];
    let runtime = ac_mn::collections_runtime::prepare(None).unwrap();
    let meta = cache::CoffMeta::for_collections_unit(unit, cache::unit_fp(unit), &runtime.identity)
        .unwrap();
    let mut coff = ac_mn::collections_lower::emit_unit_coff(unit)
        .unwrap()
        .bytes;
    // Extend COFF's string table with unused empty strings. The exact object
    // still closes at its declared table end and has the same actual code.
    let symbols = u32::from_le_bytes(coff[8..12].try_into().unwrap()) as usize;
    let count = u32::from_le_bytes(coff[12..16].try_into().unwrap()) as usize;
    let strings = symbols + count * 18;
    coff.resize(ac_mn::config::PAGE_BYTES * 2 + 13, 0);
    let string_len = (coff.len() - strings) as u32;
    coff[strings..strings + 4].copy_from_slice(&string_len.to_le_bytes());
    auto_av_probe::coff_admission::check_coff_relocatable(&coff).unwrap();
    let (_, pages) = cache::put_unit_coff(&store, None, &meta, &coff).unwrap();
    assert_eq!(pages.len(), 2);
    let mut driver = MnDriver::with_store(&cache_root, "p889").unwrap();
    driver.forbid_codegen = true;
    let run = driver.request_collections(source, "main", &dir.join("output"), None);
    assert!(run.ok, "{:?}", run.error);
    assert_eq!(run.exit_code, Some(42));
    let manifest = art::get_manifest(&store, run.manifest_artifact_id.as_deref().unwrap()).unwrap();
    assert_eq!(manifest.units[0].extra_coff_pages, pages);
    let mut restored = MnDriver::with_store(&cache_root, "p889").unwrap();
    restored.forbid_codegen = true;
    let run = restored.request_restored_collections(
        run.manifest_artifact_id.as_deref().unwrap(),
        &dir.join("restored"),
        None,
    );
    assert!(run.ok, "{:?}", run.error);
    assert_eq!(run.exit_code, Some(42));
    let mut forged = manifest;
    forged.units[0].extra_coff_pages.pop();
    let id = art::put_manifest(&store, &forged).unwrap();
    assert!(
        !restored
            .request_restored_collections(&id, &dir.join("bad"), None)
            .ok
    );
    use ac_artifact_probe::{codec::ChunkKind, identity::QueryKey, store::PutRequest};
    let id = store
        .put_artifact(PutRequest {
            layer: art::MANIFEST_LAYER.id,
            key: QueryKey::new(
                &art::MANIFEST_LAYER,
                vec![("manifest".into(), "oversized-page-control".into())],
            ),
            kind: ChunkKind::LayoutData,
            payload: vec![0; ac_mn::config::PAGE_BYTES + 1],
            edges: vec![],
            project: None,
        })
        .unwrap();
    assert!(art::get_blob(
        &store,
        &art::Blob {
            kind: "mir".into(),
            hash: String::new(),
            len: 1,
            pages: vec![id]
        },
        "mir"
    )
    .is_err());
}

#[test]
fn collections_runtime_override_is_bound_to_prepared_library_bytes() {
    let dir = directory("runtime-identity");
    let original = ac_mn::collections_runtime::prepare(None).unwrap();
    let copied = dir.join("runtime.lib");
    std::fs::copy(&original.library, &copied).unwrap();
    assert_eq!(
        ac_mn::collections_runtime::prepare(Some(&copied))
            .unwrap()
            .identity,
        original.identity
    );
    let mut bytes = std::fs::read(&copied).unwrap();
    let middle = bytes.len() / 2;
    bytes[middle] ^= 1;
    std::fs::write(&copied, &bytes).unwrap();
    let error = ac_mn::collections_runtime::prepare(Some(&copied))
        .err()
        .expect("same-size altered runtime must reject");
    assert_eq!(error.code(), "mn.admit.runtime-content");
    std::fs::write(&copied, b"wrong runtime").unwrap();
    assert!(ac_mn::collections_runtime::prepare(Some(&copied)).is_err());
}
