//! F798-R3-09 / F798-R3-11 tier: the finite generation / native-artifact
//! admission model (PLAN-798 §5.3 T-08) with a REAL-COFF positive control
//! (landed Cranelift lowering) and per-axis negatives; plus the bounded
//! native-run failure counter-control (capped streams terminate the run
//! as a Run error, never a truncated success).

use auto_ac_prototype::source::compile_scalar_source;
use auto_av_probe::native_admission::{
    admit_native_ref, check_coff_relocatable, content_hash, mint_and_admit_lowered, ref_for_coff,
    NativeAdmissionCtx, NativeAdmissionError, NativeArtifactRef, ObjectFormat, ADMITTED_ABI,
    ADMITTED_BACKEND, ADMITTED_NATIVE_PROFILE, ADMITTED_RECIPE, ADMITTED_TARGET,
};

const SRC: &str = "fn main() int {\n    return 7\n}\n";

/// Serializes tests that mutate process env (the budget fault seam)
/// against tests that run real builds reading the same env.
static NATIVE_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn checked() -> auto_ac_prototype::verify::CheckedModule {
    compile_scalar_source(SRC, "adm.at", &[])
        .expect("frontend")
        .checked
}

fn ctx() -> NativeAdmissionCtx {
    NativeAdmissionCtx::default()
}

/// Positive control: a REAL relocatable COFF produced by the landed
/// backend mints + admits under the current context.
#[test]
fn native_admission_real_coff_positive() {
    let (r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).expect("admit");
    assert_eq!(r.object_len, bytes.len() as u64);
    // The minted hash is the blake3 of the exact bytes.
    assert_eq!(r.object_sha256, content_hash(&bytes));
    // Structural COFF properties hold on the real artifact.
    check_coff_relocatable(&bytes).expect("real object is relocatable");
}

/// Wrong generation / unknown profile / unknown capability are rejected —
/// never forward-granted.
#[test]
fn native_admission_identity_axes_rejected() {
    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).expect("admit");

    r.generation = "plan799-gen-9".into();
    assert_eq!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::UnknownGeneration(
            "plan799-gen-9".into()
        ))
    );
    // An explicit legacy-generation entry admits it (compatibility axis is
    // an admission decision, not a silent default).
    let mut legacy = NativeAdmissionCtx::default();
    legacy.legacy_generations.insert("plan799-gen-9".into());
    assert!(admit_native_ref(&r, &bytes, &legacy).is_ok());

    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).unwrap();
    r.profile = "native.unknown/9".into();
    assert_eq!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::UnknownProfile(
            "native.unknown/9".into()
        ))
    );

    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).unwrap();
    r.capabilities = vec!["native.ffi".into()];
    assert_eq!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::UnknownCapability("native.ffi".into()))
    );

    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).unwrap();
    r.target = "wasm32-unknown-unknown".into();
    assert!(matches!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::UnknownTarget(_))
    ));

    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).unwrap();
    r.abi = "sysv".into();
    assert!(matches!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::UnknownAbi(_))
    ));

    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).unwrap();
    r.backend = "gcc".into();
    assert!(matches!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::UnknownBackend(_))
    ));

    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).unwrap();
    r.recipe = "other/recipe".into();
    assert!(matches!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::UnknownRecipe(_))
    ));
}

/// Process addresses are not portable artifacts: a reference CLAIMING a
/// linked image (or any non-relocatable format) over real object bytes is
/// rejected; content-binding mismatches are rejected.
#[test]
fn native_admission_relocatability_and_binding() {
    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).expect("admit");

    r.format = ObjectFormat::LinkedImage;
    assert!(matches!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::NotRelocatable(_))
    ));
    r.format = ObjectFormat::Other("raw-bin".into());
    assert!(matches!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::NotRelocatable(_))
    ));

    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).unwrap();
    r.object_sha256 = content_hash(b"other bytes");
    assert!(matches!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::HashMismatch { .. })
    ));

    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).unwrap();
    r.object_len += 1;
    assert!(matches!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::LengthMismatch { .. })
    ));

    let (mut r, bytes) = mint_and_admit_lowered(&checked(), "d_main", &ctx()).unwrap();
    r.has_manifest = false;
    assert_eq!(
        admit_native_ref(&r, &bytes, &ctx()),
        Err(NativeAdmissionError::MissingManifest)
    );

    // The frozen admitted axes are the reference pair (compile-time pin).
    assert_eq!(ADMITTED_TARGET, "x86_64-pc-windows-msvc");
    assert_eq!(ADMITTED_ABI, "msvc-x64");
    assert_eq!(ADMITTED_BACKEND, "cranelift");
    assert_eq!(ADMITTED_NATIVE_PROFILE, "native.msvc-x64/1");
    assert_eq!(ADMITTED_RECIPE, "plan798/native-reference");
}

/// A REAL linked image (process addresses) is structurally rejected: the
/// PE the native reference produces is not admissible as a relocatable
/// artifact reference.
#[cfg(windows)]
#[test]
fn native_admission_real_pe_rejected() {
    let _env_guard = NATIVE_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // Build a tiny PE through the landed pipeline (same shape the trace
    // parity leg uses, without the trace lib).
    let workdir = std::env::temp_dir().join(format!("av798-nadm-pe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let run = auto_av_probe::native_reference::build_and_run(&checked(), "d_main", &workdir, None)
        .expect("native build+run");
    assert_eq!(run.exit_code, 7);
    let pe_bytes = std::fs::read(&run.exe_path).expect("read PE");
    // Process-image bytes never pass the relocatable-object check.
    assert!(matches!(
        check_coff_relocatable(&pe_bytes),
        Err(NativeAdmissionError::MalformedObject(_))
    ));
    // And a reference minted AS IF the PE were the artifact is rejected on
    // the format axis even with honest content binding.
    let mut r = NativeArtifactRef {
        generation: "plan798-gen-1".into(),
        profile: ADMITTED_NATIVE_PROFILE.into(),
        capabilities: vec![],
        format: ObjectFormat::CoffRelocatable,
        target: ADMITTED_TARGET.into(),
        abi: ADMITTED_ABI.into(),
        backend: ADMITTED_BACKEND.into(),
        flags: vec![],
        recipe: ADMITTED_RECIPE.into(),
        object_sha256: content_hash(&pe_bytes),
        object_len: pe_bytes.len() as u64,
        has_manifest: true,
    };
    assert!(matches!(
        admit_native_ref(&r, &pe_bytes, &ctx()),
        Err(NativeAdmissionError::MalformedObject(_))
    ));
    r.format = ObjectFormat::LinkedImage;
    let _ = r; // (already rejected structurally above)
    let _ = std::fs::remove_dir_all(&workdir);
}

/// F798-R3-11 failure counter-control: the native run uses the RAW-BYTES
/// capped executor — a stream past its cap terminates the whole run as a
/// Run error, never a truncated success. The trace program writes mark
/// bytes to stderr ("aba"), so a zero stderr cap must fail.
#[cfg(windows)]
#[test]
fn native_reference_capped_stream_fails_closed() {
    let _env_guard = NATIVE_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let src = "fn main() int {\n    let a = mark_a()\n    return a\n}\n";
    let caps = vec!["hir.test.trace.v1".to_string()];
    let compiled = compile_scalar_source(src, "cap.at", &caps).expect("frontend");
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let support = repo.join("experimental/ac-core/test-support");
    let lib = support.join("target/debug/ac_trace_support.lib");
    if !lib.is_file() {
        let out = std::process::Command::new("cargo")
            .args(["build", "--locked"])
            .current_dir(&support)
            .output()
            .expect("spawn cargo for test-support");
        assert!(
            out.status.success(),
            "test-support build failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let workdir = std::env::temp_dir().join(format!("av798-nadm-cap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    // Uncapped control: real run, stderr carries the mark bytes.
    let ok = auto_av_probe::native_reference::build_and_run_with_caps(
        &compiled.checked,
        "d_main",
        &workdir,
        Some(&lib),
        1024 * 1024,
        1024 * 1024,
    )
    .expect("uncapped control runs");
    assert_eq!(ok.stderr, "a");
    let workdir2 = std::env::temp_dir().join(format!("av798-nadm-cap0-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir2);
    let capped = auto_av_probe::native_reference::build_and_run_with_caps(
        &compiled.checked,
        "d_main",
        &workdir2,
        Some(&lib),
        1024 * 1024,
        0,
    );
    assert!(
        matches!(capped, Err(auto_av_probe::native_reference::NativeError::Run(ref m)) if m.contains("cap") || m.contains("OutputCap")),
        "zero stderr cap fails closed: {:?}",
        capped
    );
    let _ = std::fs::remove_dir_all(&workdir);
    let _ = std::fs::remove_dir_all(&workdir2);
}

/// F798-R6-06 replay: a 20-byte COFF header stub claiming sections and an
/// out-of-range symbol pointer never admits — the header's claimed table
/// ranges must fit the offered bytes.
#[test]
fn native_admission_truncated_coff_header_rejected() {
    let mut fake = vec![0u8; 20];
    fake[0..2].copy_from_slice(&0x8664u16.to_le_bytes()); // AMD64
    fake[2..4].copy_from_slice(&1u16.to_le_bytes()); // 1 section
    fake[8..12].copy_from_slice(&u32::MAX.to_le_bytes()); // symbol pointer
    fake[12..16].copy_from_slice(&1u32.to_le_bytes()); // 1 symbol record
    let r = ref_for_coff(&fake);
    assert!(matches!(
        admit_native_ref(&r, &fake, &ctx()),
        Err(NativeAdmissionError::MalformedObject(_))
    ));
    assert!(matches!(
        check_coff_relocatable(&fake),
        Err(NativeAdmissionError::MalformedObject(w)) if w.contains("section table overruns")
    ));
    // A zero-symbol-count header is rejected too (linkable objects carry
    // at least one symbol record).
    let mut nosym = vec![0u8; 20];
    nosym[0..2].copy_from_slice(&0x8664u16.to_le_bytes());
    nosym[2..4].copy_from_slice(&1u16.to_le_bytes());
    assert!(matches!(
        check_coff_relocatable(&nosym),
        Err(NativeAdmissionError::MalformedObject(w)) if w.contains("symbol")
    ));
}

/// F798-R6-07 counter-control: a pre-link stage that already consumed the
/// shared build clock makes the stage admission REFUSE to start the link
/// (deterministic fake-clock seam — no real 60s stall).
#[cfg(windows)]
#[test]
fn native_reference_pre_stage_overrun_refuses_to_start_link() {
    let _env_guard = NATIVE_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let src = "fn main() int {
    return 7
}
";
    let compiled = compile_scalar_source(src, "budget.at", &[]).expect("frontend");
    let workdir = std::env::temp_dir().join(format!("av798-nadm-budget-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    std::env::set_var("AV798_NATIVE_FAKE_ELAPSED_MS", "61000");
    let out = auto_av_probe::native_reference::build_and_run_with_caps(
        &compiled.checked,
        "d_main",
        &workdir,
        None,
        1024 * 1024,
        1024 * 1024,
    );
    std::env::remove_var("AV798_NATIVE_FAKE_ELAPSED_MS");
    assert!(
        matches!(
            out,
            Err(auto_av_probe::native_reference::NativeError::Budget(_))
        ),
        "a pre-link stage over the shared clock refuses to start the link: {:?}",
        out
    );
    let _ = std::fs::remove_dir_all(&workdir);
}
