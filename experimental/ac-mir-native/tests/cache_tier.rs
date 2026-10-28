//! PLAN-818 T-04 cache tier (`mir_native_cache_`): function-object keys,
//! dependency-precise invalidation, restore/regeneration, the no-MIR hit,
//! and controlled admission rejections at the consumption gate
//! (V-04/V-05/V-06; AC-04).

mod common;

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use ac_mn::cache::{
    coff_unit_key, get_unit_coff, put_link_manifest, put_unit_coff, unit_fp, CoffMeta,
    LinkManifest, LinkUnitEntry,
};
use ac_mn::config::{COFF_LAYER, LINK_LAYER, MN_ABI, MN_TARGET};
use ac_mn::driver::MnDriver;
use ac_mn::{emit, worker, MnError};

fn temp_root(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mn8-t04-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn workdir_of(root: &std::path::Path, round: &str) -> std::path::PathBuf {
    root.join(format!("w-{round}"))
}

/// V-04: body-only edits across 30 DIFFERENT revisions recompile exactly
/// the edited unit; every unchanged unit (and only those) hits; every
/// round still REALLY links and runs; the cached result equals the
/// no-cache result for sampled revisions.
#[test]
#[cfg(windows)]
fn mir_native_cache_body_edit_30_revisions_reuse_and_oracle() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let root = temp_root("body30");
    let mut d = MnDriver::with_store(&root, "body30").expect("store");
    let gen = |k: i32| {
        (
            format!(
                "fn bump(x int) int {{\n    return x + {k}\n}}\n\nfn main() int {{\n    let a = bump(10)\n    let b = bump(20)\n    return a + b\n}}\n"
            ),
            30 + 2 * k, // independent closed form: (10+k)+(20+k)
        )
    };
    for k in 0..30 {
        let (src, want) = gen(k);
        let r = d.request(
            &src,
            "d_main",
            &[],
            &workdir_of(&root, &format!("r{k}")),
            None,
        );
        assert!(r.ok, "rev {k} failed: {:?}", r.error);
        assert_eq!(r.exit_code, Some(want), "rev {k} independent oracle");
        let emitted = *r.obs.counters.get("codegen_emit_units").unwrap_or(&0);
        let hits = *r.obs.counters.get("cache_hit_coff").unwrap_or(&0);
        if k == 0 {
            assert_eq!(emitted, 2, "fresh store compiles both units");
            assert_eq!(hits, 0);
        } else {
            assert_eq!(
                emitted, 1,
                "rev {k}: ONLY the edited unit recompiles (symbol scheme keeps the caller reusable)"
            );
            assert_eq!(hits, 1, "rev {k}: the unchanged unit hits");
        }
        assert!(
            *r.obs.counters.get("link_invocations").unwrap_or(&0) >= 1
                && *r.obs.counters.get("run_invocations").unwrap_or(&0) >= 1,
            "rev {k}: a hit STILL links and runs (never a cached PASS)"
        );
        // Sampled no-cache oracle: the same revision through the
        // ephemeral (no-store) path must agree.
        if k % 10 == 7 {
            let (src2, _) = gen(k);
            let mut e = MnDriver::ephemeral();
            let w = root.join(format!("ephemeral-{k}"));
            let r2 = e.request(&src2, "d_main", &[], &w, None);
            assert!(r2.ok, "ephemeral rev {k}: {:?}", r2.error);
            assert_eq!(r2.exit_code, Some(want), "rev {k}: cache == no-cache");
            let _ = std::fs::remove_dir_all(&w);
        }
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// V-05: a callee SIGNATURE change invalidates the callee AND every
/// caller (the caller's fp embeds the callee's structural signature);
/// the new closure links and runs correctly.
#[test]
#[cfg(windows)]
fn mir_native_cache_callee_signature_change_invalidates_callers() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    let root = temp_root("sigchg");
    let mut d = MnDriver::with_store(&root, "sigchg").expect("store");
    let v1 =
        "fn add1(v int) int {\n    return v + 1\n}\n\nfn main() int {\n    return add1(41)\n}\n";
    let r1 = d.request(v1, "d_main", &[], &workdir_of(&root, "v1"), None);
    assert!(r1.ok, "{:?}", r1.error);
    assert_eq!(r1.exit_code, Some(42));
    assert_eq!(*r1.obs.counters.get("codegen_emit_units").unwrap_or(&0), 2);
    // Signature change: the callee takes a second parameter; the caller's
    // `fn:add1` import signature changes too.
    let v2 = "fn add1(v int, w int) int {\n    return v + w\n}\n\nfn main() int {\n    return add1(40, 2)\n}\n";
    let r2 = d.request(v2, "d_main", &[], &workdir_of(&root, "v2"), None);
    assert!(r2.ok, "{:?}", r2.error);
    assert_eq!(r2.exit_code, Some(42), "new closure still correct");
    assert_eq!(
        *r2.obs.counters.get("codegen_emit_units").unwrap_or(&0),
        2,
        "callee AND caller recompile (signature invalidation)"
    );
    assert_eq!(*r2.obs.counters.get("cache_hit_coff").unwrap_or(&0), 0);
    let _ = std::fs::remove_dir_all(&root);
}

/// V-05: reordering function declarations changes NO unit fingerprint —
/// every unit hits (codegen forbidden as the hard control), references do
/// not cross, and the result is identical.
#[test]
#[cfg(windows)]
fn mir_native_cache_function_reorder_all_hits_and_stable_symbols() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    let root = temp_root("reorder");
    let mut d = MnDriver::with_store(&root, "reorder").expect("store");
    let v1 = "fn f(x int) int {\n    return x * 2\n}\n\nfn g(y int) int {\n    return f(y) + 1\n}\n\nfn main() int {\n    return g(5)\n}\n";
    let r1 = d.request(v1, "d_main", &[], &workdir_of(&root, "v1"), None);
    assert!(r1.ok, "{:?}", r1.error);
    assert_eq!(r1.exit_code, Some(11), "f(5)*2+1");
    assert_eq!(*r1.obs.counters.get("codegen_emit_units").unwrap_or(&0), 3);
    // Reorder g before f (forward reference): same units, same fps.
    let v2 = "fn g(y int) int {\n    return f(y) + 1\n}\n\nfn f(x int) int {\n    return x * 2\n}\n\nfn main() int {\n    return g(5)\n}\n";
    d.forbid_codegen = true; // hard control: a miss would abort here
    let r2 = d.request(v2, "d_main", &[], &workdir_of(&root, "v2"), None);
    assert!(r2.ok, "{:?}", r2.error);
    assert_eq!(r2.exit_code, Some(11), "reordered closure identical");
    assert_eq!(
        *r2.obs.counters.get("codegen_emit_units").unwrap_or(&0),
        0,
        "reordering recompiles nothing"
    );
    assert_eq!(*r2.obs.counters.get("cache_hit_coff").unwrap_or(&0), 3);
    // The successful link IS the symbol-stability proof: unit symbols do
    // not depend on declaration order, so no reference crosses.
    let _ = std::fs::remove_dir_all(&root);
}

/// V-04: cross-project — content-identical units share cache keys across
/// project namespaces (the OwnerScope design); the second project hits
/// everything with codegen forbidden.
#[test]
#[cfg(windows)]
fn mir_native_cache_cross_project_shared_units() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    let root = temp_root("xproj");
    let src = "fn leaf(v int) int {\n    if v < 0 {\n        return 0 - v\n    }\n    return v + 7\n}\n\nfn main() int {\n    return leaf(0 - 5) + leaf(3)\n}\n";
    let mut p1 = MnDriver::with_store(&root, "proj-one").expect("store");
    let r1 = p1.request(src, "d_main", &[], &workdir_of(&root, "p1"), None);
    assert!(r1.ok, "{:?}", r1.error);
    assert_eq!(r1.exit_code, Some(5 + 10), "leaf(-5)=5, leaf(3)=10");
    assert_eq!(*r1.obs.counters.get("codegen_emit_units").unwrap_or(&0), 2);
    let mut p2 = MnDriver::with_store(&root, "proj-two").expect("store");
    p2.forbid_codegen = true; // a miss would abort — the hit must be real
    let r2 = p2.request(src, "d_main", &[], &workdir_of(&root, "p2"), None);
    assert!(r2.ok, "{:?}", r2.error);
    assert_eq!(r2.exit_code, Some(15), "cross-project result identical");
    assert_eq!(*r2.obs.counters.get("codegen_emit_units").unwrap_or(&0), 0);
    assert_eq!(*r2.obs.counters.get("cache_hit_coff").unwrap_or(&0), 2);
    let _ = std::fs::remove_dir_all(&root);
}

/// V-06: the no-MIR hit — a persisted closure links and runs from the
/// manifest + object bytes alone; the restore path receives NO source and
/// structurally has no MIR at all (818 never persists MIR; A08 belongs to
/// the 798 producer).
#[test]
#[cfg(windows)]
fn mir_native_cache_no_mir_manifest_hit() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    let root = temp_root("nomir");
    let src = "fn helper(v int) int {\n    return v * 3\n}\n\nfn main() int {\n    return helper(14)\n}\n";
    let mut d = MnDriver::with_store(&root, "nomir").expect("store");
    let r = d.request(src, "d_main", &[], &workdir_of(&root, "w"), None);
    assert!(r.ok, "{:?}", r.error);
    assert_eq!(r.exit_code, Some(42));
    let manifest_id = r.manifest_artifact_id.clone().expect("manifest id");
    drop(d); // release nothing in particular — the worker opens its own
    let out = worker::restore_run_by_id(
        &root,
        &manifest_id,
        &workdir_of(&root, "restore"),
        None, // no source, no MIR, no trace lib needed for this source
    )
    .expect("manifest-only restore links and runs");
    assert_eq!(out.exit_code, Some(42), "restored PE result identical");
    assert_eq!(out.units_admitted, 2);
    let _ = std::fs::remove_dir_all(&root);
}

/// Compile a two-function source into verified units (the same public
/// pipeline the driver uses) — shared by the admission tests.
fn two_units() -> Vec<auto_av_core::mir::MirModule> {
    let src = "fn helper(v int) int {\n    return v + 5\n}\n\nfn main() int {\n    return helper(37)\n}\n";
    let compiled =
        auto_ac_prototype::source::compile_scalar_source(src, "mn.at", &[]).expect("compile");
    let mir =
        auto_mir_lower::lower_checked(&compiled.checked, "d_main", 0).expect("lower");
    let checked = auto_av_core::mir_verify::verify_mir(&mir).expect("verify");
    auto_av_probe::cache::split_units(checked.module()).expect("split")
}

/// V-06: admission rejections at the consumption gate — self-consistent
/// WRONG-ADDRESS artifacts, same-key conflicts preserving the existing
/// payload (failure keeps the old commit), missing manifest targets, sha
/// mismatches, wrong-target manifests, and clean axis misses.
#[test]
fn mir_native_cache_bad_admission_rejections() {
    let root = temp_root("admit");
    let store = ManagedStore::open(&root).expect("store");
    let units = two_units();
    // Units are in MODULE order (helper declared first) — find by name.
    let unit_main = units
        .iter()
        .find(|u| u.funcs[0].name == "d_main")
        .expect("d_main");
    let unit_helper = units
        .iter()
        .find(|u| u.funcs[0].name == "d_helper")
        .expect("d_helper");
    let meta_main = CoffMeta::for_unit(unit_main, unit_fp(unit_main)).unwrap();
    let meta_helper = CoffMeta::for_unit(unit_helper, unit_fp(unit_helper)).unwrap();

    // Real helper COFF persisted under its own key.
    let coff = emit::emit_unit_coff(unit_helper).expect("emit helper");
    let (id_helper, _pages) = put_unit_coff(&store, None, &meta_helper, &coff.bytes).unwrap();

    // A hit returns the identity of the exact artifact whose bytes passed
    // envelope/COFF admission, rather than reopening the key for an id.
    let (admitted_id, admitted_bytes) = ac_mn::cache::get_unit_coff_with_id(&store, &meta_helper)
        .expect("admitted lookup")
        .expect("persisted helper hit");
    assert_eq!(admitted_id, id_helper);
    assert_eq!(admitted_bytes, coff.bytes);
    assert!(ac_mn::cache::get_unit_coff_with_id(&store, &meta_main)
        .expect("clean miss")
        .is_none());

    // (a) Self-consistent wrong address: the HELPER's enveloped payload
    // stored under the MAIN unit's key. Internally consistent, wrong
    // address — the consumption gate must reject, never link it as main.
    let stored = store
        .get_artifact(COFF_LAYER.id, &coff_unit_key(&meta_helper))
        .expect("get helper payload")
        .expect("helper payload present");
    store
        .put_artifact(PutRequest {
            layer: COFF_LAYER.id,
            key: coff_unit_key(&meta_main),
            kind: ChunkKind::LayoutData,
            payload: stored.payload.clone(),
            edges: vec![],
            project: None,
        })
        .expect("tampered put under the main key (fresh key)");
    match get_unit_coff(&store, &meta_main) {
        Err(MnError::Admit(kind, msg)) => {
            assert_eq!(kind, "unit", "wrong-address payload: {msg}");
        }
        other => panic!("wrong-address artifact must Admit-reject: {other:?}"),
    }

    // Same-key conflict (V-06): a SECOND, different payload under the
    // helper's existing key is a controlled store conflict — uncertainty
    // reported, nothing overwritten, the existing commit survives.
    let coff_main = emit::emit_unit_coff(unit_main).expect("emit main");
    match put_unit_coff(&store, None, &meta_helper, &coff_main.bytes) {
        Err(MnError::Infra(msg)) => {
            assert!(
                msg.contains("key conflict"),
                "controlled store conflict classification: {msg}"
            );
        }
        other => panic!("same-key different-content must Conflict: {other:?}"),
    }
    // The existing helper payload survives the failed publish.
    assert!(
        get_unit_coff(&store, &meta_helper).is_ok(),
        "failure keeps the old commit"
    );

    // Crafted manifests under synthetic keys (no edges — these are
    // adversarial documents, not driver output).
    let put_crafted = |man: &LinkManifest, tag: &str| -> String {
        let key = QueryKey::new(&LINK_LAYER, vec![("probe".to_string(), tag.to_string())]);
        store
            .put_artifact(PutRequest {
                layer: LINK_LAYER.id,
                key,
                kind: ChunkKind::LayoutData,
                payload: man.render().into_bytes(),
                edges: vec![],
                project: None,
            })
            .expect("crafted manifest put")
    };
    let base_man = |entry: LinkUnitEntry, target: &str| LinkManifest {
        owner: meta_helper.owner.clone(),
        entry: "d_helper".to_string(),
        entry_symbol: meta_helper.symbol.clone(),
        abi: MN_ABI.to_string(),
        target: target.to_string(),
        units: vec![entry],
        lld_sha: "probe".to_string(),
        sdk_tag: "probe".to_string(),
        kernel32_sha: "probe".to_string(),
        trace_lib_sha: None,
    };
    let real_entry = |artifact_id: String, sha: String| LinkUnitEntry {
        meta: meta_helper.clone(),
        sha256: sha,
        len: coff.bytes.len(),
        artifact_id,
        extra_page_ids: vec![],
    };

    // (b) Manifest references a MISSING artifact id.
    let man_missing = base_man(
        real_entry("art-does-not-exist".into(), coff_sha(&coff.bytes)),
        MN_TARGET,
    );
    let id_missing = put_crafted(&man_missing, "missing-id");
    match worker::restore_run_by_id(&root, &id_missing, &workdir_of(&root, "b"), None) {
        Err(MnError::Admit(kind, _)) => assert_eq!(kind, "address"),
        other => panic!("missing manifest target must Admit-reject: {other:?}"),
    }

    // (c) Manifest sha does not bind the stored bytes.
    let man_badsha = base_man(
        real_entry(id_helper.clone(), format!("{:064x}", 0u32)),
        MN_TARGET,
    );
    let id_badsha = put_crafted(&man_badsha, "bad-sha");
    match worker::restore_run_by_id(&root, &id_badsha, &workdir_of(&root, "c"), None) {
        Err(MnError::Admit(kind, msg)) => {
            assert_eq!(kind, "sha256", "{msg}");
        }
        other => panic!("sha mismatch must Admit-reject: {other:?}"),
    }

    // (d) Manifest claims a foreign target — the axes are rejected BEFORE
    // any object is read; no host machine code is ever produced.
    let man_target = base_man(
        real_entry(id_helper.clone(), coff_sha(&coff.bytes)),
        "x86_64-unknown-linux-gnu",
    );
    let id_target = put_crafted(&man_target, "bad-target");
    match worker::restore_run_by_id(&root, &id_target, &workdir_of(&root, "d"), None) {
        Err(MnError::Admit(kind, _)) => assert_eq!(kind, "manifest"),
        other => panic!("foreign-target manifest must Admit-reject: {other:?}"),
    }

    // (e) A foreign TARGET axis on the unit meta is a CLEAN MISS — a
    // different key space, never a host-machine-code fallback.
    let mut meta_foreign = meta_helper.clone();
    meta_foreign.target = "aarch64-pc-windows-msvc".to_string();
    assert!(
        matches!(get_unit_coff(&store, &meta_foreign), Ok(None)),
        "foreign-target unit key must miss cleanly"
    );

    // Control: the REAL two-unit closure restores and runs from the
    // manifest alone on an UNPOISONED store (this store's d_main key is
    // deliberately tampered above and must stay that way) — the entry is
    // the ZERO-PARAM d_main (the wrapper contract); the one-param helper
    // rides alongside.
    if common::tools_available() {
        let croot = temp_root("admit-control");
        let cstore = ManagedStore::open(&croot).expect("control store");
        let coff_main = emit::emit_unit_coff(unit_main).expect("emit main");
        let (id_main, _a) = put_unit_coff(&cstore, None, &meta_main, &coff_main.bytes).unwrap();
        let (id_h2, _b) = put_unit_coff(&cstore, None, &meta_helper, &coff.bytes).unwrap();
        let real_man = LinkManifest {
            owner: meta_main.owner.clone(),
            entry: "d_main".to_string(),
            entry_symbol: meta_main.symbol.clone(),
            abi: MN_ABI.to_string(),
            target: MN_TARGET.to_string(),
            units: vec![
                LinkUnitEntry {
                    meta: meta_main.clone(),
                    sha256: coff_sha(&coff_main.bytes),
                    len: coff_main.bytes.len(),
                    artifact_id: id_main,
                    extra_page_ids: vec![],
                },
                LinkUnitEntry {
                    meta: meta_helper.clone(),
                    sha256: coff_sha(&coff.bytes),
                    len: coff.bytes.len(),
                    artifact_id: id_h2,
                    extra_page_ids: vec![],
                },
            ],
            lld_sha: "probe".to_string(),
            sdk_tag: "probe".to_string(),
            kernel32_sha: "probe".to_string(),
            trace_lib_sha: None,
        };
        let real_id = put_link_manifest(&cstore, None, &real_man).unwrap();
        let out = worker::restore_run_by_id(&croot, &real_id, &workdir_of(&croot, "ok"), None)
            .expect("real manifest restores");
        assert_eq!(out.exit_code, Some(42), "helper(37) = 42");
        assert_eq!(out.units_admitted, 2);
        let _ = std::fs::remove_dir_all(&croot);
    }
    let _ = std::fs::remove_dir_all(&root);
}

fn coff_sha(bytes: &[u8]) -> String {
    ac_mn::cache::content_hash(bytes)
}

/// The unit metas derive ONLY from the verified unit content: the same
/// source produces identical fingerprints across independent compilations
/// (key stability); a callee BODY edit changes exactly the callee's fp
/// (the caller's fn: import signature is untouched — dependency-precise
/// invalidation), while the symbol stays put.
#[test]
fn mir_native_cache_fingerprint_axes() {
    let units = two_units();
    let by_name = |n: &str| units.iter().find(|u| u.funcs[0].name == n).unwrap();
    let (m_main, m_helper) = (
        CoffMeta::for_unit(by_name("d_main"), unit_fp(by_name("d_main"))).unwrap(),
        CoffMeta::for_unit(by_name("d_helper"), unit_fp(by_name("d_helper"))).unwrap(),
    );
    // Independent recompile of the SAME source: same fps.
    let again = two_units();
    let by_name2 = |n: &str| again.iter().find(|u| u.funcs[0].name == n).unwrap();
    assert_eq!(
        m_main.fp,
        CoffMeta::for_unit(by_name2("d_main"), unit_fp(by_name2("d_main")))
            .unwrap()
            .fp,
        "same source, same fingerprint"
    );
    assert_eq!(
        m_helper.fp,
        CoffMeta::for_unit(by_name2("d_helper"), unit_fp(by_name2("d_helper")))
            .unwrap()
            .fp
    );
    assert_eq!(
        m_main.symbol,
        CoffMeta::for_unit(by_name2("d_main"), unit_fp(by_name2("d_main")))
            .unwrap()
            .symbol
    );
    // A callee BODY edit: callee fp moves, caller fp and both symbols stay.
    let src_edited = "fn helper(v int) int {\n    return v + 6\n}\n\nfn main() int {\n    return helper(37)\n}\n";
    let compiled = auto_ac_prototype::source::compile_scalar_source(src_edited, "mn.at", &[])
        .expect("compile");
    let mir = auto_mir_lower::lower_checked(&compiled.checked, "d_main", 0).unwrap();
    let checked = auto_av_core::mir_verify::verify_mir(&mir).unwrap();
    let units2 = auto_av_probe::cache::split_units(checked.module()).unwrap();
    let e_helper = units2
        .iter()
        .find(|u| u.funcs[0].name == "d_helper")
        .unwrap();
    let e_main = units2.iter().find(|u| u.funcs[0].name == "d_main").unwrap();
    let e_helper = CoffMeta::for_unit(e_helper, unit_fp(e_helper)).unwrap();
    assert_ne!(m_helper.fp, e_helper.fp, "callee body edit moves its fp");
    assert_eq!(m_helper.symbol, e_helper.symbol, "symbol stays");
    assert_eq!(
        m_main.fp,
        CoffMeta::for_unit(e_main, unit_fp(e_main)).unwrap().fp,
        "caller fp untouched by a callee BODY edit (signature unchanged)"
    );
}

/// Tool identity must cover every byte, including a short final read, while
/// agreeing with the existing whole-buffer BLAKE3 identity at block boundaries.
#[test]
fn mir_native_cache_streamed_tool_identity_matches_full_bytes() {
    let root = temp_root("streamed-tool-identity");
    for n in [0usize, 1, 65535, 65536, 65537, 131089] {
        let mut bytes: Vec<u8> = (0..n).map(|i| (i.wrapping_mul(37) % 251) as u8).collect();
        if let Some(last) = bytes.last_mut() {
            *last = 255;
        }
        let path = root.join(format!("tool-{n}.bin"));
        std::fs::write(&path, &bytes).unwrap();
        let got = ac_mn::link_driver::file_identity(&path).unwrap();
        assert_eq!(got, ac_mn::cache::content_hash(&bytes), "length {n}");
        assert_eq!(got, ac_mn::link_driver::file_identity(&path).unwrap());
        if n > 65536 {
            assert_ne!(got, ac_mn::cache::content_hash(&bytes[..65536]));
        }
    }
    assert!(ac_mn::link_driver::file_identity(&root.join("missing.bin")).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
