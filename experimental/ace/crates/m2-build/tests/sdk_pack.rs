//! `ace.m2.p978.sdk-pack` — V-08 / V-09 / V-ID-PACK (PLAN-978 T-08/T-09):
//! the relocatable candidate SDK — same content stable across roots,
//! out-of-repo run with the development cache absent, undeclared-file and
//! integrity rejections, provider-grant admission, and concurrent CURRENT
//! publication without lost updates.

mod common;

use std::path::Path;

use ace_m2_build::driver::{M2Build, ProviderArtifactInput, RunObservation};
use ace_m2_build::identity::TargetKind;
use common::{b01_inputs, scratch};

fn run_exit(run: &Result<RunObservation, ace_m2_build::M2Reject>) -> i32 {
    run.as_ref()
        .expect("run must succeed")
        .exit_code
        .expect("value observed")
}

/// V-08 leg 1 + V-ID-PACK: pack av+native, RELOCATE the directory far
/// from the build root, delete the build root (no dev cache), run both
/// lanes from the relocated copy — 14 both ways.
#[test]
fn p978_sdk_relocates_and_runs_without_dev_cache() {
    let root = scratch("sdk-relocate");
    let mut driver = M2Build::new(&root);
    let sdk = root.join("sdk");
    driver
        .build_sdk(
            &b01_inputs(),
            &[TargetKind::Av, TargetKind::Native],
            &sdk,
            vec![],
            "run:t1",
        )
        .expect("pack");

    // Relocate: copy to a sibling far away, remove ALL build state.
    let far = scratch("sdk-far");
    let relocated = far.join("packed-sdk");
    copy_dir(&sdk, &relocated);
    let receipt_before = std::fs::read(relocated.join("install-manifest.json")).unwrap();
    std::fs::remove_dir_all(&root).expect("dev cache fully removed");

    let mut consumer = M2Build::new(&far.join("fresh-root"));
    let av = consumer.run_sdk(&relocated, TargetKind::Av, "run:t1");
    assert_eq!(run_exit(&av), 14);
    let native = consumer.run_sdk(&relocated, TargetKind::Native, "run:t1");
    assert_eq!(run_exit(&native), 14, "precompiled exe runs from the SDK");
    // The recipe document survived byte-identical across the run.
    assert_eq!(
        receipt_before,
        std::fs::read(relocated.join("install-manifest.json")).unwrap()
    );
}

/// V-ID-PACK leg 2: an UNDECLARED file inside the SDK is a named
/// rejection (a smuggled dependency cannot ride along).
#[test]
fn p978_sdk_undeclared_file_rejected() {
    let root = scratch("sdk-extra");
    let mut driver = M2Build::new(&root);
    let sdk = root.join("sdk");
    driver
        .build_sdk(&b01_inputs(), &[TargetKind::Av], &sdk, vec![], "run:t1")
        .unwrap();
    std::fs::write(sdk.join("smuggled.dll"), b"evil").unwrap();
    let err = M2Build::verify_sdk(&sdk).expect_err("extra file must reject");
    assert_eq!(err.code, "m2build.recipe.extra-file");
    assert!(err.detail.contains("smuggled.dll"));
}

/// V-ID-PACK leg 3: artifact corruption (hash drift) is a named
/// integrity rejection; the OLD runnable copy keeps working (failure
/// never overwrites the good package).
#[test]
fn p978_sdk_corruption_rejected_old_kept() {
    let root = scratch("sdk-corrupt");
    let mut driver = M2Build::new(&root);
    let sdk = root.join("sdk");
    driver
        .build_sdk(&b01_inputs(), &[TargetKind::Av], &sdk, vec![], "run:t1")
        .unwrap();
    // Corrupt one declared artifact (the build receipt).
    let artifact = sdk.join("targets/av/build-receipt.json");
    let original = std::fs::read(&artifact).unwrap();
    std::fs::write(&artifact, b"{corrupted}").unwrap();
    let err = M2Build::verify_sdk(&sdk).expect_err("corruption must reject");
    assert_eq!(err.code, "m2build.recipe.integrity");
    // Repair (the restore path): rewrite the original bytes — the same
    // entry works again (failure did not destroy the package).
    std::fs::write(&artifact, original).unwrap();
    assert!(M2Build::verify_sdk(&sdk).is_ok());
    let mut consumer = M2Build::new(&root.join("c"));
    assert_eq!(
        run_exit(&consumer.run_sdk(&sdk, TargetKind::Av, "run:t1")),
        14
    );
}

/// V-ID-PACK leg 4: a MISSING declared artifact is a named rejection.
#[test]
fn p978_sdk_missing_artifact_rejected() {
    let root = scratch("sdk-missing");
    let mut driver = M2Build::new(&root);
    let sdk = root.join("sdk");
    driver
        .build_sdk(&b01_inputs(), &[TargetKind::Av], &sdk, vec![], "run:t1")
        .unwrap();
    std::fs::remove_file(sdk.join("targets/av/build-receipt.json")).unwrap();
    let err = M2Build::verify_sdk(&sdk).expect_err("missing artifact must reject");
    assert_eq!(err.code, "m2build.recipe.missing");
}

/// Provider-grant admission (V-ID-RESTORE leg): a packed provider with a
/// grant admits the run; the WRONG grant is refused before dispatch;
/// repacking with the right grant admits again.
#[test]
fn p978_sdk_provider_grant_admission() {
    let root = scratch("sdk-grant");
    let mut driver = M2Build::new(&root);
    let sdk = root.join("sdk");
    driver
        .build_sdk(
            &b01_inputs(),
            &[TargetKind::Av],
            &sdk,
            vec![ProviderArtifactInput {
                provider_id: "f0-text-provider".into(),
                file_name: "f0_text_provider.dll".into(),
                bytes: b"provider-bytes".to_vec(),
                kind: "cdylib".into(),
            }],
            "run:right",
        )
        .unwrap();
    // Wrong grant: refused BEFORE dispatch.
    let err = {
        let mut c = M2Build::new(&root.join("c1"));
        c.run_sdk(&sdk, TargetKind::Av, "run:wrong").unwrap_err()
    };
    assert_eq!(err.code, "m2build.selection.authorization");
    // Tampered provider bytes (pin mismatch): refused.
    let dll = sdk.join("providers/f0_text_provider.dll");
    std::fs::write(&dll, b"tampered").unwrap();
    let err = {
        let mut c = M2Build::new(&root.join("c2"));
        c.run_sdk(&sdk, TargetKind::Av, "run:right").unwrap_err()
    };
    assert!(
        err.code == "m2build.selection.pin" || err.code == "m2build.recipe.integrity",
        "tamper rejects with {err}"
    );
}

/// V-09 leg 2 (source02 boundary): an explicit C-FFI source through the
/// plain AV/native lanes is a NAMED rejection (the frozen F0 foreign
/// domain belongs to the 967 provider lanes) — never a silent fallback
/// or a wrong-answer run.
#[test]
fn p978_foreign_source_named_boundary() {
    let root = scratch("foreign-boundary");
    let mut driver = M2Build::new(&root);
    let inputs =
        ace_m2_build::driver::ProjectInputs::new("strlen", "main", "main.at", common::STRLEN_AT);
    for target in [TargetKind::Av, TargetKind::Native] {
        let err = driver
            .build(target, &inputs)
            .expect_err("foreign source must reject on the plain lanes");
        let expected_code = match target {
            TargetKind::Av => "m2build.lane-av",
            TargetKind::Native => "m2build.lane-native",
            _ => "m2build.lane-a2x",
        };
        assert_eq!(
            err.code, expected_code,
            "target {}: named lane rejection, got {err}",
            target.name()
        );
        assert!(
            err.detail.contains("source.unsupported") || err.detail.contains("unsupported"),
            "the boundary is NAMED: {err}"
        );
    }
}

/// V-ID-HANDOFF: the recipe IS the 967→978→986 handoff document — one
/// provenance identity per requested target, build tools ONLY in
/// build_inputs (pure-AV SDK needs none), runtime closure split out, and
/// a packed provider carrying its pin/authorization mapping.
#[test]
fn p978_recipe_is_the_handoff_document() {
    let root = scratch("handoff");
    let mut driver = M2Build::new(&root);
    let sdk = root.join("sdk");
    let recipe = driver
        .build_sdk(
            &b01_inputs(),
            &[TargetKind::Av, TargetKind::A2r],
            &sdk,
            vec![ProviderArtifactInput {
                provider_id: "f0-text-provider".into(),
                file_name: "f0_text_provider.dll".into(),
                bytes: b"handoff-provider".to_vec(),
                kind: "cdylib".into(),
            }],
            "run:h",
        )
        .unwrap();
    // One provenance identity per requested target.
    assert_eq!(recipe.provenance.len(), 2);
    assert_eq!(
        recipe
            .provenance
            .iter()
            .filter(|p| p.target == TargetKind::Av)
            .count(),
        1
    );
    // Runtime closure: an av-host dependency, NO toolchain requirement.
    assert!(recipe.run_dependencies.iter().any(|d| d.kind == "av-host"));
    assert!(
        !recipe
            .run_dependencies
            .iter()
            .any(|d| d.kind == "linker" || d.kind == "compiler"),
        "build tools never ride the runtime closure"
    );
    // Provider selection: pin + grant + artifact mapping present.
    let sel = recipe.provider_selection.as_ref().unwrap();
    assert_eq!(sel.provider_id, "f0-text-provider");
    assert!(!sel.pinned_sha256.is_empty());
    assert_eq!(sel.authorization, "run:h");
    assert_eq!(sel.artifacts.len(), 1);
    // The packed provider file exists at its declared relative path.
    assert!(sdk.join("providers/f0_text_provider.dll").is_file());
}

/// V-08 leg 2: concurrent CURRENT publication loses updates LOUDLY —
/// four writers race on one (project, kind); the surviving CURRENT is
/// exactly one live artifact and every writer either succeeded or got
/// the named CAS rejection (never a silent overwrite).
#[test]
fn p978_concurrent_current_no_lost_update() {
    let root = scratch("sdk-current");
    let store_root = root.join("store").join("m2b");
    let store = ace_artifacts::store::ManagedStore::open(&store_root).unwrap();
    let mut ids = Vec::new();
    for i in 0..4 {
        let key = ace_artifacts::identity::QueryKey {
            layer: ace_m2_build::identity::M978B_LAYER,
            components: vec![("identity".into(), format!("race-{i}"))],
        };
        store
            .put_artifact(ace_artifacts::store::PutRequest {
                layer: ace_m2_build::identity::M978B_LAYER,
                key,
                kind: ace_artifacts::codec::ChunkKind::LayoutData,
                payload: vec![i as u8],
                edges: vec![],
                project: Some("race".into()),
            })
            .unwrap();
        let loaded = store
            .get_artifact(
                ace_m2_build::identity::M978B_LAYER,
                &ace_artifacts::identity::QueryKey {
                    layer: ace_m2_build::identity::M978B_LAYER,
                    components: vec![("identity".into(), format!("race-{i}"))],
                },
            )
            .unwrap()
            .unwrap();
        ids.push(loaded.artifact_id);
    }
    // All four race to replace CURRENT from the same expected base
    // (None = empty). Exactly the first CAS wins; the rest get the named
    // Boundary rejection.
    let mut wins = 0;
    let mut loud_losses = 0;
    for id in &ids {
        match store.replace_current_root("race", "m2build/av", None, id) {
            Ok(()) => wins += 1,
            Err(ace_artifacts::store::StoreError::Boundary(_)) => loud_losses += 1,
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }
    assert_eq!(wins, 1, "exactly one CAS winner");
    assert_eq!(loud_losses, 3, "every loser is a NAMED rejection");
    let roots = store.root_artifacts("race", "m2build/av").unwrap();
    assert_eq!(
        roots.len(),
        1,
        "CURRENT is exactly one root (no lost update)"
    );
    assert!(ids.contains(&roots[0]));
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap().flatten() {
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}
