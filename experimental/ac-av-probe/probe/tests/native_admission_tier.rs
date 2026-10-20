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
    let run = auto_av_probe::native_reference::build_and_run(SRC, &[], "d_main", &workdir, None)
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
        src,
        &caps,
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
        src,
        &caps,
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
    let workdir = std::env::temp_dir().join(format!("av798-nadm-budget-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    std::env::set_var("AV798_NATIVE_FAKE_ELAPSED_MS", "61000");
    let out = auto_av_probe::native_reference::build_and_run_with_caps(
        src,
        &[],
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

/// F798-R7-04 replay: an 82-byte structured object (full header + section
/// header + one symbol + closing string table) whose section declares raw
/// data at an out-of-range pointer — rejected at the section-window check.
#[test]
fn native_admission_section_data_out_of_range_rejected() {
    let mut fake = vec![0u8; 20 + 40 + 18 + 4];
    fake[0..2].copy_from_slice(&0x8664u16.to_le_bytes()); // AMD64
    fake[2..4].copy_from_slice(&1u16.to_le_bytes()); // 1 section
    fake[8..12].copy_from_slice(&(20u32 + 40).to_le_bytes()); // sym ptr after section table
    fake[12..16].copy_from_slice(&1u32.to_le_bytes()); // 1 symbol
    let sh = 20;
    fake[sh + 16..sh + 20].copy_from_slice(&16u32.to_le_bytes()); // SizeOfRawData
    fake[sh + 20..sh + 24].copy_from_slice(&u32::MAX.to_le_bytes()); // raw ptr
    let st = 20 + 40 + 18;
    fake[st..st + 4].copy_from_slice(&4u32.to_le_bytes()); // strtab closes
    let r = ref_for_coff(&fake);
    assert!(matches!(
        admit_native_ref(&r, &fake, &ctx()),
        Err(NativeAdmissionError::MalformedObject(w)) if w.contains("raw-data window overruns")
    ));
}

/// F798-R7-01 replay: an in-flight linker stall past the REMAINING build
/// budget is killed with its tree — fake 59s pre-stage + a real 12s-stalling
/// "linker" (AC741_RUST_LLD override) must return Budget within ~2s of the
/// 1s remaining budget, never run the stall to completion.
#[cfg(windows)]
#[test]
fn native_reference_inflight_linker_stall_killed_at_remaining_budget() {
    let _env_guard = NATIVE_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let slow = std::env::temp_dir().join(format!("av798-slow-lld-{}.cmd", std::process::id()));
    std::fs::write(
        &slow,
        "@echo off\r\nping -n 13 127.0.0.1 > nul\r\nexit /b 9\r\n",
    )
    .unwrap();
    let workdir = std::env::temp_dir().join(format!("av798-nadm-stall-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    std::env::set_var("AV798_NATIVE_FAKE_ELAPSED_MS", "59000");
    std::env::set_var("AC741_RUST_LLD", &slow);
    let started = std::time::Instant::now();
    let out = auto_av_probe::native_reference::build_and_run_with_caps(
        SRC,
        &[],
        "d_main",
        &workdir,
        None,
        1024 * 1024,
        1024 * 1024,
    );
    let wall = started.elapsed();
    std::env::remove_var("AV798_NATIVE_FAKE_ELAPSED_MS");
    std::env::remove_var("AC741_RUST_LLD");
    let _ = std::fs::remove_file(&slow);
    assert!(
        matches!(
            out,
            Err(auto_av_probe::native_reference::NativeError::Budget(_))
        ),
        "the stalled build is a Budget kill, not a completed link: {:?}",
        out
    );
    assert!(
        wall < std::time::Duration::from_secs(6),
        "killed near the ~1s remaining budget (took {wall:?}), not after the 12s stall"
    );
    let _ = std::fs::remove_dir_all(&workdir);
}

/// F798-R8-03: cold worker preparation is BOUNDED — a stalled build
/// command is killed at its (shortened) preparation wall; a missing
/// override surfaces a controlled error; the ready path succeeds.
#[cfg(windows)]
#[test]
fn native_worker_preparation_bounded_and_killed_on_stall() {
    let _env_guard = NATIVE_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let slow = std::env::temp_dir().join(format!("av798-prep-stall-{}.cmd", std::process::id()));
    std::fs::write(
        &slow,
        "@echo off\r\nping -n 20 127.0.0.1 > nul\r\nexit /b 1\r\n",
    )
    .unwrap();
    let root = std::env::temp_dir().join(format!("av798-prep-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let exe = root
        .join("target")
        .join("release")
        .join("av_probe_worker.exe");
    std::env::set_var("AV798_NATIVE_PREP_WALL_MS", "1500");
    std::env::set_var("AV798_NATIVE_PREP_CARGO", &slow);
    let started = std::time::Instant::now();
    let out = auto_av_probe::native_reference::bounded_worker_build(&root, &exe);
    let wall = started.elapsed();
    std::env::remove_var("AV798_NATIVE_PREP_WALL_MS");
    std::env::remove_var("AV798_NATIVE_PREP_CARGO");
    let _ = std::fs::remove_file(&slow);
    assert!(
        out.as_ref()
            .err()
            .map(|e| e.contains("preparation exceeded"))
            .unwrap_or(false),
        "stalled preparation is killed at its wall: {:?}",
        out
    );
    assert!(
        wall < std::time::Duration::from_secs(5),
        "killed near the 1.5s wall (took {wall:?}), not the 20s stall"
    );
    // Missing override: controlled error (not a panic, not a hang).
    std::env::set_var("AV798_NATIVE_WORKER", root.join("definitely-missing.exe"));
    let miss = auto_av_probe::native_reference::native_worker_binary();
    std::env::remove_var("AV798_NATIVE_WORKER");
    assert!(miss.is_err());
    assert!(miss.unwrap_err().contains("missing file"));
    // Ready path: the real worker resolves.
    let ready = auto_av_probe::native_reference::native_worker_binary();
    assert!(ready.is_ok(), "ready control: {:?}", ready.err());
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R9-02: the preparation supervisor drains BOTH streams under caps —
/// an over-cap stderr FAILS (never a truncated success), a large stdout
/// under the cap succeeds (no pipe backpressure false-timeout), and a
/// stalled preparation is killed promptly.
#[cfg(windows)]
#[test]
fn native_worker_preparation_dual_stream_controls() {
    let _env_guard = NATIVE_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = std::env::temp_dir().join(format!("av798-prep2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("target").join("release")).unwrap();
    let exe = root
        .join("target")
        .join("release")
        .join("av_probe_worker.exe");

    // Control A: over-cap stderr (2 MiB) must FAIL even with exit 0 and
    // the product present.
    let overflow = root.join("overflow.cmd");
    std::fs::write(
        &overflow,
        "@echo off\r\ntype nul > \"%CD%\\target\\release\\av_probe_worker.exe\"\r\npowershell -NoProfile -Command \"$s='A'*1024; 1..2048 | ForEach-Object { [Console]::Error.WriteLine($s) }\"\r\nexit /b 0\r\n",
    )
    .unwrap();
    std::env::set_var("AV798_NATIVE_PREP_WALL_MS", "60000");
    std::env::set_var("AV798_NATIVE_PREP_CARGO", &overflow);
    let out = auto_av_probe::native_reference::bounded_worker_build(&root, &exe);
    assert!(
        out.as_ref()
            .err()
            .map(|e| e.contains("stream cap"))
            .unwrap_or(false),
        "2MiB stderr over the cap fails closed: {:?}",
        out
    );

    // Control B: 128 KiB stdout under the cap succeeds (the pipe is
    // drained — no backpressure false-timeout).
    let stdout_ok = root.join("stdout-ok.cmd");
    std::fs::write(
        &stdout_ok,
        "@echo off\r\ntype nul > \"%CD%\\target\\release\\av_probe_worker.exe\"\r\npowershell -NoProfile -Command \"$s='B'*1024; 1..128 | ForEach-Object { [Console]::Out.WriteLine($s) }\"\r\nexit /b 0\r\n",
    )
    .unwrap();
    let _ = std::fs::remove_file(&exe);
    std::env::set_var("AV798_NATIVE_PREP_CARGO", &stdout_ok);
    let started = std::time::Instant::now();
    let out = auto_av_probe::native_reference::bounded_worker_build(&root, &exe);
    let wall = started.elapsed();
    assert!(out.is_ok(), "128KiB stdout drains and succeeds: {:?}", out);
    assert!(
        wall < std::time::Duration::from_secs(20),
        "no backpressure stall (took {wall:?})"
    );

    std::env::remove_var("AV798_NATIVE_PREP_WALL_MS");
    std::env::remove_var("AV798_NATIVE_PREP_CARGO");
    let _ = std::fs::remove_dir_all(&root);
}
