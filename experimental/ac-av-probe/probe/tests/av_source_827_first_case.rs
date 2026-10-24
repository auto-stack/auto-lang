//! PLAN-827 first case (V-01): the REAL AP-01 source file — frozen bytes
//! read from the ac-core fixtures tree — through the full landed pipeline
//! (source profile → Checked HIR → shared MIR → ABC → AV), plus the store
//! round (put → hit, AV still executed every time) and a TRUE new-process
//! restore (`av_probe_worker restore-run` re-opens the store, re-verifies
//! every envelope, re-executes). Independent oracles: the in-source gcd
//! groups are hand-audited (contract-decision §3) and the native reference
//! build/run cross-checks the same bytes on a separate channel.
//!
//! Identity discipline (V-04 first axis): the seeds' SHA256 receipts are
//! frozen constants; the corpus copies under
//! `crates/auto-lang/test/cases/av-source-827/src/` must be byte-identical
//! to the originals — verified from BOTH directions every run.

use auto_av_probe::driver::Driver;

/// The read-only originals (frozen at PLAN-827 T-01 against the catalog
/// receipts; any drift here is a seed-identity violation, not a test
/// adjustment).
const AP01_ORIGINAL: &str = "../../ac-core/fixtures/scalar-source/valid/p-gcd-euclid.at";
const AP22_ORIGINAL: &str = "../../ac-core/fixtures/scalar-source/valid/p-binop-args.at";
const AP23_ORIGINAL: &str = "../../ac-core/fixtures/scalar-source/valid/p-short-circuit-trap.at";
const AP01_SHA256: &str = "6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3";
const AP22_SHA256: &str = "46d6a431a5c1b9bbe6e2e87b8732c09a9c584b58eb92799fdb4ab073f423e625";
const AP23_SHA256: &str = "d6871b77c9b02e797c77a2cf34900e005fe97062bf1fc95ffcdf1cf7707052a9";
/// The corpus copy (formal-recipe path input).
const AP01_CORPUS_COPY: &str =
    "../../../crates/auto-lang/test/cases/av-source-827/src/p-gcd-euclid.at";

fn manifest_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let d = Sha256::digest(bytes);
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn read_seed(rel: &str, frozen_sha: &str) -> String {
    let bytes =
        std::fs::read(manifest_path(rel)).unwrap_or_else(|e| panic!("read seed {rel}: {e}"));
    let sha = sha256_hex(&bytes);
    assert_eq!(
        sha, frozen_sha,
        "seed {rel} drifted from the PLAN-827 frozen receipt (was {frozen_sha})"
    );
    String::from_utf8(bytes).unwrap_or_else(|e| panic!("seed {rel} is not UTF-8: {e}"))
}

/// V-04 identity axis: originals match the frozen receipts AND the corpus
/// copy is byte-identical to the AP-01 original (checked both ways).
#[test]
fn av_source_827_seeds_identity() {
    let ap01 = read_seed(AP01_ORIGINAL, AP01_SHA256);
    let _ap22 = read_seed(AP22_ORIGINAL, AP22_SHA256);
    let _ap23 = read_seed(AP23_ORIGINAL, AP23_SHA256);
    let copy = std::fs::read(manifest_path(AP01_CORPUS_COPY)).expect("corpus copy of AP-01 exists");
    assert_eq!(
        sha256_hex(&copy),
        AP01_SHA256,
        "corpus copy must be byte-identical to the AP-01 original"
    );
    assert_eq!(
        copy,
        ap01.into_bytes(),
        "corpus copy bytes == original bytes"
    );
}

/// V-01: the real AP-01 file, ephemeral driver — typed i32 0 with the full
/// pipeline actually exercised (one real lowering, both functions' ABC
/// units emitted, AV executed).
#[test]
fn av_source_827_first_case_real_file_ephemeral() {
    let src = read_seed(AP01_ORIGINAL, AP01_SHA256);
    let mut d = Driver::ephemeral();
    let r = d.request(&src, "d_main", &[]);
    assert!(r.diag.is_none(), "pipeline rejected AP-01: {:?}", r.diag);
    assert_eq!(r.value_i32(), Some(0), "hand-audited gcd groups all pass");
    assert_eq!(
        *r.counters.get("mir_lower_modules").unwrap_or(&0),
        1,
        "one real whole-module lowering"
    );
    assert_eq!(
        *r.counters.get("abc_units_emitted").unwrap_or(&0),
        2,
        "gcd + main: two ABC units emitted"
    );
    assert!(
        *r.counters.get("av_execute").unwrap_or(&0) >= 1,
        "AV executed"
    );
}

/// V-01/V-07 first case: same-store put → hit (hit still executes AV,
/// never a cached PASS) → TRUE new-process restore via the real worker
/// binary (store re-opened from disk, every envelope re-verified, closure
/// re-assembled, entry re-executed).
#[test]
fn av_source_827_first_case_store_round_hit_new_worker() {
    let src = read_seed(AP01_ORIGINAL, AP01_SHA256);
    let root = std::env::temp_dir().join(format!("av827-first-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = Driver::with_store(&root, "av827-first").expect("store");
    let r1 = d.request(&src, "d_main", &[]);
    assert!(r1.ok(), "fresh (miss) request: {:?}", r1.diag);
    assert_eq!(r1.value_i32(), Some(0));
    assert_eq!(
        *r1.counters.get("cache_hit_abc").unwrap_or(&0),
        0,
        "fresh = all misses"
    );
    let r2 = d.request(&src, "d_main", &[]);
    assert!(r2.ok(), "hit request: {:?}", r2.diag);
    assert_eq!(r2.value_i32(), Some(0));
    assert_eq!(
        *r2.counters.get("mir_build").unwrap_or(&0),
        0,
        "hit: no lower"
    );
    assert_eq!(
        *r2.counters.get("abc_emit").unwrap_or(&0),
        0,
        "hit: no emit"
    );
    assert!(
        *r2.counters.get("av_execute").unwrap_or(&0) > 0,
        "a hit STILL executes AV"
    );

    // New worker process: restore by manifest artifact id, re-verify, run.
    let manifest = r2.manifest.as_ref().expect("manifest");
    let store = auto_av_probe::cache::open_store(&root).unwrap();
    let key = auto_av_probe::cache::manifest_key(
        &manifest.owner,
        &manifest.entry,
        &manifest.fingerprint(),
    );
    let art = store
        .get_artifact("A09", &key)
        .unwrap()
        .expect("manifest artifact");
    drop(store);
    let worker = env!("CARGO_BIN_EXE_av_probe_worker");
    let out = std::process::Command::new(worker)
        .args([
            "restore-run",
            "--store",
            root.to_str().unwrap(),
            "--owner",
            &manifest.owner,
            "--entry",
            &manifest.entry,
            "--manifest-artifact",
            &art.artifact_id,
            "--snapshot",
            &format!("{:016x}", r2.snapshot),
        ])
        .output()
        .expect("spawn worker");
    assert!(
        out.status.success(),
        "restore-run failed: {}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let parsed: auto_av_probe::worker_restore::RestoreOutcome =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("worker JSON");
    assert!(parsed.executed_ok, "{:?}", parsed.error);
    assert_eq!(parsed.value, Some(0), "restored execution result");
    assert_eq!(
        parsed.units_verified, 2,
        "both units re-verified in the new process"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// V-01 independent oracle channel: the SAME frozen bytes through the
/// native reference build/run must exit 0 (separate channel from the AV
/// guest result). The native channel is REQUIRED (plan §5.5/AC-06): when
/// the tool cannot be resolved the test FAILS — never a silent pass or an
/// ignored skip (R4-F01).
static NATIVE_TOOL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
#[cfg(windows)]
fn av_source_827_first_case_native_cross_check() {
    let _guard = NATIVE_TOOL_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let src = read_seed(AP01_ORIGINAL, AP01_SHA256);
    if let Err(e) = auto_ac_prototype::link::find_rust_lld() {
        panic!(
            "REQUIRED native oracle unavailable (rust-lld: {e:?}); \
             this case cannot pass without the independent native channel"
        );
    }
    let workdir = std::env::temp_dir().join(format!("av827-native-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let run = auto_av_probe::native_reference::build_and_run(&src, &[], "d_main", &workdir, None)
        .expect("native build+run");
    assert_eq!(
        run.exit_code, 0,
        "native exit must match the hand-audited 0"
    );
    let _ = std::fs::remove_dir_all(&workdir);
}

/// R4-F01 missing-tool control (first-case branch): with the SUPPORTED
/// `AC741_RUST_LLD` override pointing at a nonexistent file, tool
/// resolution ERRORS without a silent fallback — exactly the branch the
/// required native test above panics on. The control never touches a real
/// tool and restores the environment.
#[test]
#[cfg(windows)]
fn av_source_827_native_missing_tool_is_red_first_case_control() {
    let _guard = NATIVE_TOOL_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let saved = std::env::var("AC741_RUST_LLD").ok();
    std::env::set_var("AC741_RUST_LLD", "plan827-absent-review-linker.exe");
    let resolved = auto_ac_prototype::link::find_rust_lld();
    match saved {
        Some(v) => std::env::set_var("AC741_RUST_LLD", v),
        None => std::env::remove_var("AC741_RUST_LLD"),
    }
    assert!(
        resolved.is_err(),
        "a broken override must make resolution fail — the branch the required native test panics on"
    );
}
