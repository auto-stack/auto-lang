//! Native runtime ABI/resource probes. Source -> MIR coverage lives separately.
mod common;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

fn libraries() -> &'static [PathBuf] {
    static LIBRARIES: OnceLock<Vec<PathBuf>> = OnceLock::new();
    LIBRARIES.get_or_init(|| {
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/collections");
        let output = Command::new("cargo")
            .args(["build", "--manifest-path"])
            .arg(runtime.join("Cargo.toml"))
            .arg("--release")
            .arg("--locked")
            .env_remove("CARGO_TARGET_DIR")
            .output()
            .expect("build container runtime");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        vec![runtime.join("target/release/ac_collections_runtime.lib")]
    })
}

fn runtime_probe(tag: &str, config: Option<&str>) -> ac_mn::link_driver::PeRun {
    assert!(
        common::tools_available(),
        "native tools are required; no skip pass"
    );
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = common::repo_root().join(format!(
        "docs/plans/attachments/889/runtime-{tag}-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&work).unwrap();
    let object = work.join("caller.obj");
    let mut compiler = Command::new("rustc");
    compiler
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/collections_runtime_probe.rs"),
        )
        .args([
            "--crate-type=lib",
            "--emit=obj",
            "--edition=2021",
            "-C",
            "panic=abort",
            "-C",
            "opt-level=1",
            "-o",
        ])
        .arg(&object);
    if let Some(config) = config {
        compiler.args(["--cfg", config]);
    }
    let output = compiler.output().expect("compile native ABI caller");
    std::fs::write(work.join("compiler.log"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = std::fs::read(&object).unwrap();
    let run = ac_mn::link_driver::link_and_run(
        &[],
        &bytes,
        "ac_start",
        &work.join("pe"),
        libraries(),
        "runtime-only probe, not source/MIR execution",
    )
    .expect("real native runtime PE");
    std::fs::write(work.join("stderr.log"), &run.stderr).unwrap();
    run
}

fn zero_live(stderr: &[u8]) {
    balance(stderr).unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(stderr)));
    let text = String::from_utf8_lossy(stderr);
    for field in [
        "arena_live",
        "arena_commit",
        "arena_invalid",
        "text_metadata",
    ] {
        assert!(text.contains(&format!("MN-COLL {field}=0\n")), "{text}");
    }
}

#[test]
fn runtime_allocator_takeover_refuses_live_and_retained_old_heap_storage() {
    let run = runtime_probe("allocator-switch", Some("mn_probe_allocator_switch"));
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    zero_live(&run.stderr);
}

#[test]
fn runtime_guest_exact_eight_mib_and_plus_one_rejection() {
    let run = runtime_probe("guest-limits", Some("mn_probe_guest"));
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    zero_live(&run.stderr);
    let text = String::from_utf8_lossy(&run.stderr);
    let peak: u64 = text
        .lines()
        .find_map(|line| line.strip_prefix("MN-COLL arena_peak="))
        .unwrap()
        .parse()
        .unwrap();
    assert!(peak > 8 << 20 && peak <= 32 << 20, "{text}");
}

#[test]
fn runtime_physical_quota_fragment_reuse_and_coalescing() {
    let run = runtime_probe("arena-quota", Some("mn_probe_arena"));
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    zero_live(&run.stderr);
    assert!(String::from_utf8_lossy(&run.stderr).contains("MN-COLL arena_peak=4096\n"));
}

#[test]
fn runtime_cancellation_rolls_back_text_chunks_and_partial_list_clone() {
    let run = runtime_probe("cancel", Some("mn_probe_cancel"));
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    zero_live(&run.stderr);
}

#[test]
fn runtime_text_metadata_initial_and_growth_failure_are_atomic() {
    let run = runtime_probe("metadata", Some("mn_probe_metadata"));
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    zero_live(&run.stderr);
}

#[test]
fn runtime_list_exact_limit_and_plus_one_rejection() {
    let run = runtime_probe("list-limits", Some("mn_probe_limits"));
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    zero_live(&run.stderr);
}

fn balance(stderr: &[u8]) -> Result<(), String> {
    let text = String::from_utf8_lossy(stderr);
    for observation in [
        "MN-COLL live=0\n",
        "MN-COLL bytes=0\n",
        "MN-COLL guest=0\n",
        "MN-COLL invalid=0\n",
    ] {
        if !text.contains(observation) {
            return Err(format!("missing {observation}"));
        }
    }
    let summaries: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("SUMMARY "))
        .collect();
    if summaries.len() != 1 {
        return Err("missing/duplicate text summary".into());
    }
    let fields: std::collections::BTreeMap<_, _> = summaries[0]
        .split_whitespace()
        .filter_map(|field| field.split_once('='))
        .map(|(k, v)| (k, v.parse::<u64>()))
        .collect();
    let get = |key| fields.get(key).and_then(|n| n.as_ref().ok()).copied();
    if get("live") != Some(0)
        || get("live_bytes") != Some(0)
        || get("allocs").is_none()
        || get("allocs") != get("frees")
    {
        return Err("text buffers not balanced".into());
    }
    if !text
        .lines()
        .any(|line| line.starts_with("FREESTAT ") && line.ends_with(" invalid=0"))
    {
        return Err("missing/invalid text release observations".into());
    }
    Ok(())
}

#[test]
fn runtime_ownership_clone_and_failure_atomicity() {
    let run = runtime_probe("primitives", None);
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    zero_live(&run.stderr);
}

#[test]
fn runtime_trap_cleans_resource_callers_and_normal70_is_distinct() {
    let trap = runtime_probe("trap", Some("mn_probe_trap"));
    assert_eq!(trap.exit_code, Some(70));
    zero_live(&trap.stderr);
    assert!(String::from_utf8_lossy(&trap.stderr).contains("MN-COLL-TRAP code=21\n"));
    let normal = runtime_probe("normal70", Some("mn_probe_normal70"));
    assert_eq!(normal.exit_code, Some(70));
    zero_live(&normal.stderr);
    assert!(!String::from_utf8_lossy(&normal.stderr).contains("MN-COLL-TRAP"));
}

#[test]
fn runtime_deep_construct_clone_rollback_and_drop_on_64k_stack() {
    let run = runtime_probe("deep64k", Some("mn_probe_deep"));
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    zero_live(&run.stderr);
}

#[test]
fn runtime_omitted_frame_release_control_has_nonzero_live() {
    let run = runtime_probe("leak-control", Some("mn_probe_leak"));
    // The PE deliberately returns the same value. Resource observations must
    // distinguish the actual omitted release from a normal return42.
    assert_eq!(run.exit_code, Some(42));
    let text = String::from_utf8_lossy(&run.stderr);
    assert!(text.contains("MN-COLL live="));
    assert!(
        !text.contains("MN-COLL live=0\n"),
        "missing release went unobserved: {text}"
    );
    assert!(!text.contains("MN-COLL bytes=0\n"));
}

#[test]
fn runtime_missing_text_owner_is_rejected_even_when_container_balance_is_zero() {
    let run = runtime_probe("text-leak-control", Some("mn_probe_text_leak"));
    assert_eq!(run.exit_code, Some(42));
    assert!(String::from_utf8_lossy(&run.stderr).contains("MN-COLL live=0\n"));
    assert_eq!(
        balance(&run.stderr).unwrap_err(),
        "text buffers not balanced"
    );
}
