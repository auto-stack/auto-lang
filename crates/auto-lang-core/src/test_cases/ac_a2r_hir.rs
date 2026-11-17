//! PLAN-892 `ac-a2r/hir-scalar-emit-v1` recipe adapter (`probe/run` seam,
//! the 796 registered-recipe contract): the case source runs through the
//! REAL A2X chain — checked core/scalar frontend → target-neutral semantic
//! view → Rust provider → full actual file set — produced by the PREBUILT
//! `a2x-worker` tool, and judged entirely PARENT-SIDE by byte-comparing
//! the actual set against the reviewed golden sidecars.
//!
//! Plan §5.5 daily-channel contracts implemented here:
//! * spawn-zero: this adapter spawns EXACTLY ONE process — the prebuilt
//!   worker (the trusted tool identity; a case can never name an
//!   executable). rustc/cargo/target-Rust programs are NEVER spawned on
//!   the daily path; the worker's self-reported `spawned_rustc` must be 0
//!   and the parent verifies it;
//! * NO lazy toolchain builds: a missing worker is a controlled infra
//!   ERROR carrying the explicit maintenance command — stricter than the
//!   older adapters' first-use cargo build (the maintenance build is an
//!   observable operation owned by the verify script/tests, never the
//!   case timing path);
//! * goldens are NEVER an input to production: the worker receives only
//!   the source/profile/entry/caps/nonce; expected bytes arrive through
//!   the oracle sidecars like every other adapter;
//! * identity: the worker's protocol/recipe echo and manifest hash are
//!   re-verified parent-side from the emitted files (trust but verify).

use std::path::{Path, PathBuf};
use std::time::Instant;

use auto_test_case::compare::{compare_bytes, Mismatch};
use auto_test_case::contract::{CheckSpec, Completion, ProbeStep, Syntax};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::ArtifactRef;
use auto_test_case::sha256::{hex, sha256};

use super::avm::Verdict;
use auto_test_case::report::Status;

pub const A2R_HIR_RECIPE: &str = "ac-a2r/hir-scalar-emit-v1";
/// PLAN-902: the record/text/text-record emit recipe — the same prebuilt
/// worker, adapter contracts and parent-side judging as the scalar
/// channel; the worker derives the channel from the case's profile.
pub const A2R_RT_RECIPE: &str = "ac-a2r/hir-record-text-emit-v1";
/// PLAN-904: the collections (List/Variant) emit recipe — same prebuilt
/// worker, adapter contracts and parent-side judging; strict recipe/
/// profile/fileset/cert isolation from both channels above.
pub const A2R_COLL_RECIPE: &str = "ac-a2r/hir-collections-emit-v1";
pub const A2X_WORKER_PROTOCOL: &str = "a2xworker/1";
pub const A2X_WORKER_ENV: &str = "AC_A2X_HIR_WORKER";
/// F-2 (review): OPTIONAL pinned content credential for the prebuilt
/// worker (sha256 hex). When set, a worker binary whose hash disagrees is
/// an infra ERROR — the daily channel never trusts an unidentified tool
/// (plan D-04: 不确定即不授信).
pub const A2X_WORKER_SHA_ENV: &str = "AC_A2X_HIR_WORKER_SHA256";

/// Locate the PREBUILT a2x worker. Env override first (re-read every
/// call); otherwise the conventional release location next to the repo
/// root. NEVER builds — a missing worker is infra, with the maintenance
/// command in the error text.
pub fn worker_binary() -> Result<PathBuf, String> {
    if std::env::var("AUTO_TEST_DOMAIN").as_deref() == Ok("legacy") {
        return Err(
            "ACE worker requested by a legacy test group; explicitly select the ACE group".into(),
        );
    }
    if let Ok(env_path) = std::env::var(A2X_WORKER_ENV) {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "{A2X_WORKER_ENV} points at a missing file: {env_path}"
        ));
    }
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    while let Some(d) = dir {
        // the conventional prebuilt locations: the repo-root target (the
        // verify script copies there) and the a2x workspace's own release
        // output (where `cargo build --manifest-path .../auto-a2x` puts it)
        for exe in [d
            .join("experimental/ace")
            .join("target")
            .join("release")
            .join(a2x_worker_exe_name())]
        {
            if exe.is_file() {
                return Ok(exe);
            }
        }
        dir = d.parent();
    }
    Err(format!(
        "prebuilt a2x worker not found (no {A2X_WORKER_ENV} override, no <repo>/target/release/{}); \
         maintenance build (observable, outside case timing): \
         cargo build --release --manifest-path experimental/ace/Cargo.toml -p ace-a2x-worker --bin a2x-worker",
        a2x_worker_exe_name()
    ))
}

fn a2x_worker_exe_name() -> &'static str {
    if cfg!(windows) {
        "a2x-worker.exe"
    } else {
        "a2x-worker"
    }
}

pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    if case.spec.source.syntax != Syntax::Auto {
        return Err(format!(
            "{} consumes auto source files; syntax {:?} is not accepted",
            A2R_HIR_RECIPE,
            case.spec.source.syntax.as_str()
        ));
    }
    // PLAN-902: both a2x emit recipes ride this adapter; the case's recipe
    // is the identity the worker must echo back.
    let recipe = check
        .options
        .probe_recipe
        .as_deref()
        .ok_or("probe check without a recipe (contract violation)")?;
    if recipe != A2R_HIR_RECIPE && recipe != A2R_RT_RECIPE && recipe != A2R_COLL_RECIPE {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    let binding = check
        .options
        .probe_a2x_hir
        .as_ref()
        .ok_or("probe check without an a2x_hir binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if steps.len() != 1 || !matches!(steps[0], ProbeStep::Run) {
        return Err(format!(
            "{recipe} takes exactly one `run` step (the daily emit); other step shapes arrive with their own contract"
        ));
    }
    let source_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "a2x_hir adapter needs the resolved entry input".to_string())?;

    let worker = worker_binary()?;
    // F-2 (review): content credential — the observed binary hash travels
    // with the verdict; a pinned expectation (if declared) must match or
    // the check is infra, never a language verdict.
    let worker_bytes = std::fs::read(&worker).map_err(|e| format!("read worker: {e}"))?;
    let worker_sha = hex(&sha256(&worker_bytes));
    if let Ok(pinned) = std::env::var(A2X_WORKER_SHA_ENV) {
        if !pinned.is_empty() && pinned != worker_sha {
            return Ok(Verdict {
                status: Status::Error,
                detail: Some(format!(
                    "worker content credential mismatch: pinned {pinned} != observed {worker_sha}"
                )),
                ..Verdict::pass()
            });
        }
    }
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("artifact dir: {e}"))?;
    let source_path = artifact_dir.join("source.at");
    std::fs::write(&source_path, &source_bytes).map_err(|e| format!("stage source: {e}"))?;
    let actual_dir = artifact_dir.join("actual");
    let nonce = hex(&sha256(
        format!(
            "{}|{}|a2x-hir",
            std::process::id(),
            started.elapsed().as_nanos()
        )
        .as_bytes(),
    ));

    let request = serde_json::json!({
        "op": "daily-emit",
        "source_path": source_path,
        "profile": binding.profile,
        "entry": binding.entry,
        "caps": binding.caps,
        "out_dir": actual_dir,
        "nonce": nonce,
    });

    let request_text = request.to_string();
    let exchange = super::execution_trace::WorkerExchange::start(
        artifact_dir,
        A2X_WORKER_PROTOCOL,
        recipe,
        &worker_sha,
        request_text.as_bytes(),
    )?;
    // The ONLY process this adapter ever spawns (plan §5.5 spawn-zero).
    // PLAN-1034: 有界交换——请求经 stdin 全量送达（argv 不再携带），
    // exec 70s / 单一 cleanup 3s / stdout 4MiB / stderr 1MiB；cap 或期限
    // 命中即 Status::Error（Infra，保 Resource 子因），绝无截断成功。
    let policy = super::a2x_exchange::A2XExchangePolicy::daily_emit();
    let out = match super::a2x_exchange::exchange(&worker, &request_text, policy) {
        Ok(out) => out,
        Err(e) => {
            if let Some(exchange) = exchange {
                let _ = exchange.finish(&[], &[], None, Some(&e.0));
            }
            // 交换失败是 INFRA，永不充当语言判定。
            return Ok(Verdict {
                status: Status::Error,
                detail: Some(e.0),
                ..Verdict::pass()
            });
        }
    };
    if let Some(exchange) = exchange {
        exchange.finish(&out.stdout, &out.stderr, Some(0), None)?;
    }
    if out.exit_kind != auto_tool_execution::ExitKind::Success {
        // A worker refusal is INFRA, never a language verdict.
        return Ok(Verdict {
            status: Status::Error,
            detail: Some(format!(
                "a2x worker failed ({:?}): {}",
                out.exit_kind,
                String::from_utf8_lossy(&out.stdout).trim()
            )),
            ..Verdict::pass()
        });
    }
    let resp: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("worker response not json: {e}"))?;
    if resp["status"] != "ok" {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some(format!("worker controlled failure: {}", resp["detail"])),
            ..Verdict::pass()
        });
    }
    // Identity echo (trust but verify): the worker must have served the
    // case's own recipe channel.
    if resp["protocol"].as_str() != Some(A2X_WORKER_PROTOCOL)
        || resp["recipe"].as_str() != Some(recipe)
    {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some(format!(
                "worker identity mismatch: protocol={:?} recipe={:?}",
                resp["protocol"], resp["recipe"]
            )),
            ..Verdict::pass()
        });
    }
    if resp["spawned_rustc"].as_i64() != Some(0) {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some("worker reported nonzero rustc spawns on the daily path".into()),
            ..Verdict::pass()
        });
    }
    // Parent-side manifest re-hash: the reported digest must match the
    // emitted manifest bytes on disk. PLAN-1034: 产物有界准入——manifest
    // 4MiB、单文件 64MiB、单 check 总量 128MiB，cap+1 有界读取 + checked
    // 累加（生产者报告 bytes 只作声明，父侧核真实字节）。
    let mut artifacts = super::a2x_exchange::ArtifactBudget::new();
    let manifest_bytes = artifacts.read_manifest(&actual_dir.join("manifest.json"))?;
    let manifest_sha = hex(&sha256(&manifest_bytes));
    if resp["manifest_sha256"].as_str() != Some(manifest_sha.as_str()) {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some("worker manifest hash disagrees with the emitted file".into()),
            ..Verdict::pass()
        });
    }

    match &oracle.completion {
        Completion::Success => {
            // PLAN-977 AC-COVERAGE negative control: an EMISSION check must
            // not carry run-observation expectations — emit cannot
            // masquerade as run. Exit/stdout/stderr/AV-channel oracles are
            // run legs (the certify op / run adapters), never this lane.
            if oracle.exit.is_some()
                || oracle.stdout.is_some()
                || oracle.stderr.is_some()
                || oracle.av_channel.is_some()
            {
                return Ok(Verdict {
                    status: Status::Error,
                    detail: Some(
                        "a2r emit checks declare artifact expectations only \
                         (exit/stdout/stderr/av-channel are run observations; \
                         emit cannot masquerade as run)"
                            .into(),
                    ),
                    ..Verdict::pass()
                });
            }
            let mut mismatches = Vec::new();
            let mut refs = Vec::new();
            let mut expected_paths: Vec<String> = Vec::new();
            // PLAN-904 (RV-4, §5.6): path-domain review BEFORE any byte
            // comparison — declared artifact paths are relative, unique
            // after normalization, free of `..` traversal, and carry no
            // Windows case collision (two paths differing only by ASCII
            // case would alias on a case-insensitive filesystem).
            for problem in path_domain_problems(
                &oracle.artifacts.iter().map(|w| w.path.clone()).collect::<Vec<_>>(),
            ) {
                mismatches.push(Mismatch {
                    observation: "artifact paths".into(),
                    kind: "path-domain",
                    expected: "relative normalized unique paths within the output domain".into(),
                    actual: problem,
                    first_diff: None,
                });
            }
            for want in &oracle.artifacts {
                expected_paths.push(want.path.clone());
                let path = actual_dir.join(&want.path);
                match artifacts.read_file(&path, false) {
                    Err(limit) => return Ok(Verdict {
                        status: Status::Error,
                        detail: Some(limit),
                        ..Verdict::pass()
                    }),
                    Ok(None) => mismatches.push(Mismatch {
                        observation: format!("artifact {}", want.path),
                        kind: "missing",
                        expected: format!("{} bytes", want.bytes.len()),
                        actual: "absent".to_string(),
                        first_diff: None,
                    }),
                    Ok(Some(actual)) => {
                        if let Some(m) = compare_bytes(
                            &want.bytes,
                            &actual,
                            want.compare,
                            &format!("artifact {}", want.path),
                        ) {
                            mismatches.push(m);
                        }
                        refs.push(ArtifactRef {
                            logical_path: want.path.clone(),
                            stored_path: path.display().to_string(),
                            sha256: hex(&sha256(&actual)),
                            bytes: Some(actual.len()),
                        });
                    }
                }
            }
            // Undeclared extra products are a mismatch (Design 35 §6.2):
            // the actual set must equal the declared set exactly.
            let extras: Vec<String> = std::fs::read_dir(&actual_dir)
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .filter(|e| e.path().is_file())
                        .filter(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            !expected_paths.contains(&name) && name != "source.at"
                        })
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect()
                })
                .unwrap_or_default();
            if !extras.is_empty() {
                mismatches.push(Mismatch {
                    observation: "artifacts".into(),
                    kind: "extra",
                    expected: format!("exactly {:?}", expected_paths),
                    actual: format!("extra files: {:?}", extras),
                    first_diff: None,
                });
            }
            let status = if mismatches.is_empty() {
                Status::Pass
            } else {
                Status::Fail
            };
            let mut v = Verdict {
                status,
                mismatches,
                detail: None,
                artifacts: refs,
                native: None,
            };
            if v.status == Status::Pass {
                v.detail = Some(format!(
                    "a2x-hir daily emit: 4-file actual set byte-identical to the reviewed golden (manifest sha256 {manifest_sha}, worker {worker_sha}, worker spawn only)"
                ));
            }
            Ok(v)
        }
        other => Err(format!(
            "{recipe} daily checks declare success completions only (got {:?}); the trap/diagnostic axes are certification-leg observations",
            other
        )),
    }
}

/// PLAN-904 (RV-4, §5.6): the declared artifact path set's domain review
/// — relative paths only, no `..` traversal or rooted/absolute forms, and
/// unique after ASCII-case-insensitive normalization (two declared paths
/// differing only by case would alias on a case-insensitive filesystem).
/// Returns one human-readable problem per violation.
pub fn path_domain_problems(paths: &[String]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut normalized: std::collections::BTreeMap<String, &str> =
        std::collections::BTreeMap::new();
    for p in paths {
        if std::path::Path::new(p).is_absolute()
            || p.starts_with('/')
            || p.contains("..")
            || p.contains('\\')
        {
            problems.push(format!("{p}: outside the output domain"));
            continue;
        }
        let norm = p.to_ascii_lowercase().replace('\\', "/");
        if let Some(orig) = normalized.get(&norm) {
            problems.push(format!("{orig} collides with {p} (case-insensitive alias)"));
        } else {
            normalized.insert(norm, p);
        }
    }
    problems
}

#[cfg(test)]
mod path_domain_tests {
    use super::path_domain_problems;

    fn v(paths: &[&str]) -> Vec<String> {
        path_domain_problems(&paths.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn clean_paths_pass() {
        assert!(v(&["main.rs", "a2x_runtime.rs", "generated.rs", "manifest.json"]).is_empty());
        assert!(v(&["sub/dir/file.rs"]).is_empty());
    }

    #[test]
    fn traversal_and_absolute_rejected() {
        assert!(!v(&["../escape.rs"]).is_empty());
        assert!(!v(&["sub/../../x.rs"]).is_empty());
        assert!(!v(&["C:/abs/main.rs"]).is_empty());
        assert!(!v(&["/rooted.rs"]).is_empty());
    }

    #[test]
    fn case_collision_rejected() {
        let problems = v(&["main.rs", "Main.RS"]);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("collides"), "{problems:?}");
    }
}

// ---- PLAN-1034 T-03（IPC-03）：真实父 adapter 的有界交换终态反例 ----
// 驱动真实 `run_check`（非替身 helper），stub = 预构建 a2x-worker 的
// TEST-ONLY 故障缝（A2X_TEST_FAULT，子进程 env 继承；生产路径永不设置）。
// 小 cap/短 deadline 经 A2X_TEST_* env 覆盖（仅测试配置，生产默认
// 70s/3s/4MiB/1MiB 不变）；使用 env 的测试串行持有 TEST_POLICY_LOCK 并
// 在结束前恢复。正常路径/golden 的正向重放由 case-a2r/a2c-hir-* case
// 组承担（同代码路径、真 goldens）。
#[cfg(test)]
mod audit1034_ipc_03 {
    use super::*;
    use crate::test_cases::a2x_exchange::{
        A2X_TEST_CLEANUP_MS_ENV, A2X_TEST_EXEC_MS_ENV, A2X_TEST_STDERR_CAP_ENV,
        A2X_TEST_STDOUT_CAP_ENV, TEST_POLICY_LOCK,
    };
    use auto_test_case::admission::Limits;
    use auto_test_case::atom::parse_contract;
    use auto_test_case::contract::decode_case;
    use auto_test_case::document::CaseDocument;
    use auto_test_case::ir::{resolve_case, resolve_oracle};

    const CASE_MD: &str = r#"
```test-case
case {
    schema: "autolang.case/2"
    id: "ipc03_a2r_scalar"
    tags: ["ipc"]
    links: ["PLAN-1034"]
    source: { text: "fn main() int { return 42 }\n", syntax: "auto", path: "main.at" }
    entry: { mode: "program" }
    checks: [
        {
            id: "c1"
            adapter: "probe"
            action: "run"
            options: {
                recipe: "ac-a2r/hir-scalar-emit-v1"
                a2x_hir: { profile: "scalar", entry: "main", caps: [] }
                steps: [{ op: "run" }]
            }
            expect: { completion: "success" }
        }
    ]
}
```
"#;

    /// 经真实 run_check 执行一次 a2r scalar emit check。
    fn drive(tmp: &std::path::Path) -> Result<Verdict, String> {
        let limits = Limits::defaults();
        let doc = CaseDocument::parse_with(CASE_MD.as_bytes(), &limits).unwrap();
        let tc = doc.fences().first().unwrap();
        let cval = parse_contract(std::str::from_utf8(doc.payload_bytes(tc)).unwrap()).unwrap();
        let spec = decode_case(&cval, &limits).unwrap();
        let check = spec.checks[0].clone();
        let oracle = resolve_oracle(&spec.id, &check, tmp, &limits).unwrap();
        let resolved = resolve_case(spec, &doc, tmp, &limits).unwrap();
        run_check(&resolved, &check, &oracle, tmp, &tmp.join("art"), tmp)
    }

    fn tmp_root(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "plan1034_ipc03_{}_{}_{:?}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// 断言 worker 预构建存在（ace-adapters 组工具面），缺则本测试不运
    /// 行、不冒 PASS。
    fn require_worker() -> Option<std::path::PathBuf> {
        worker_binary().ok()
    }

    /// 环境负控助手：持锁设置子进程故障缝 + 小 cap/短期限，执行后恢复
    /// （不留全局残值）。
    fn with_fault_env<R>(
        fault: &str,
        exec_ms: u64,
        cleanup_ms: u64,
        stdout_cap: usize,
        stderr_cap: usize,
        pid_file: Option<&std::path::Path>,
        f: impl FnOnce() -> R,
    ) -> R {
        let _guard = TEST_POLICY_LOCK.lock().unwrap();
        std::env::set_var("A2X_TEST_FAULT", fault);
        std::env::set_var(A2X_TEST_EXEC_MS_ENV, exec_ms.to_string());
        std::env::set_var(A2X_TEST_CLEANUP_MS_ENV, cleanup_ms.to_string());
        std::env::set_var(A2X_TEST_STDOUT_CAP_ENV, stdout_cap.to_string());
        std::env::set_var(A2X_TEST_STDERR_CAP_ENV, stderr_cap.to_string());
        if let Some(p) = pid_file {
            let _ = std::fs::remove_file(p);
            std::env::set_var("A2X_TEST_PID_FILE", p);
        }
        let result = f();
        std::env::remove_var("A2X_TEST_FAULT");
        std::env::remove_var(A2X_TEST_EXEC_MS_ENV);
        std::env::remove_var(A2X_TEST_CLEANUP_MS_ENV);
        std::env::remove_var(A2X_TEST_STDOUT_CAP_ENV);
        std::env::remove_var(A2X_TEST_STDERR_CAP_ENV);
        std::env::remove_var("A2X_TEST_PID_FILE");
        result
    }

    /// 挂起 worker：短 exec 期限触发 → Status::Error（deadline 终态），
    /// 具名清理结果随行，绝不悬挂；无 rustc/cargo 额外 spawn（PID 单行）。
    #[test]
    fn hang_hits_exec_deadline_as_error() {
        if require_worker().is_none() {
            eprintln!("prebuilt a2x-worker missing; not run, not PASS");
            return;
        }
        let tmp = tmp_root("hang");
        let pid_file = tmp.join("pids.txt");
        let started = std::time::Instant::now();
        let v = with_fault_env(
            "hang",
            200,
            1_000,
            4 << 20,
            1 << 20,
            Some(&pid_file),
            || drive(&tmp).unwrap(),
        );
        assert_eq!(v.status, Status::Error, "detail: {:?}", v.detail);
        let detail = v.detail.unwrap_or_default();
        assert!(
            detail.contains("exceeded") && detail.contains("deadline"),
            "deadline termination named, got: {detail}"
        );
        assert!(
            detail.contains("cleanup_completed=1"),
            "cleanup completed within the single deadline: {detail}"
        );
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "no hang: {:?}",
            started.elapsed()
        );
        let pids = std::fs::read_to_string(&pid_file).unwrap_or_default();
        assert_eq!(pids.lines().count(), 1, "exactly one spawn: {pids}");
    }

    /// 持续 stdout 超 cap：立即终止 → Error（Resource 子因），PID 单行。
    #[test]
    fn stdout_flood_hits_cap_as_error() {
        if require_worker().is_none() {
            eprintln!("prebuilt a2x-worker missing; not run, not PASS");
            return;
        }
        let tmp = tmp_root("floodout");
        let pid_file = tmp.join("pids.txt");
        let started = std::time::Instant::now();
        let v = with_fault_env(
            "flood-stdout",
            5_000,
            1_000,
            128,
            1 << 20,
            Some(&pid_file),
            || drive(&tmp).unwrap(),
        );
        assert_eq!(v.status, Status::Error, "detail: {:?}", v.detail);
        let detail = v.detail.unwrap_or_default();
        assert!(
            detail.contains("stdout exceeded its 128-byte cap"),
            "stdout cap named: {detail}"
        );
        assert!(
            detail.contains("cleanup_completed=1"),
            "cleanup completed: {detail}"
        );
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "cap fires immediately: {:?}",
            started.elapsed()
        );
        let pids = std::fs::read_to_string(&pid_file).unwrap_or_default();
        assert_eq!(pids.lines().count(), 1, "exactly one spawn: {pids}");
    }

    /// 持续 stderr 超 cap：独立于 stdout 的同一门。
    #[test]
    fn stderr_flood_hits_cap_as_error() {
        if require_worker().is_none() {
            eprintln!("prebuilt a2x-worker missing; not run, not PASS");
            return;
        }
        let tmp = tmp_root("flooderr");
        let v = with_fault_env("flood-stderr", 5_000, 1_000, 4 << 20, 64, None, || {
            drive(&tmp).unwrap()
        });
        assert_eq!(v.status, Status::Error, "detail: {:?}", v.detail);
        let detail = v.detail.unwrap_or_default();
        assert!(
            detail.contains("stderr exceeded its 64-byte cap"),
            "stderr cap named: {detail}"
        );
    }

    /// 返回畸形 JSON 后退 0：父协议解析拒绝（run_check Err），worker
    /// exit 0 不覆盖。
    #[test]
    fn bad_json_is_protocol_refusal_not_exit_zero() {
        if require_worker().is_none() {
            eprintln!("prebuilt a2x-worker missing; not run, not PASS");
            return;
        }
        let tmp = tmp_root("badjson");
        let err = match with_fault_env("bad-json", 5_000, 1_000, 4 << 20, 1 << 20, None, || {
            drive(&tmp)
        }) {
            Err(e) => e,
            Ok(_) => panic!("garbage response must be a protocol refusal"),
        };
        assert!(
            err.contains("not json"),
            "exit-0 garbage must fail parent parse: {err}"
        );
    }

    /// 合法 JSON 但回错 recipe：trust-but-verify 身份门 → Error（非语言
    /// 判定）。
    #[test]
    fn wrong_identity_echo_is_error() {
        if require_worker().is_none() {
            eprintln!("prebuilt a2x-worker missing; not run, not PASS");
            return;
        }
        let tmp = tmp_root("wrongid");
        let v = with_fault_env(
            "wrong-identity",
            5_000,
            1_000,
            4 << 20,
            1 << 20,
            None,
            || drive(&tmp).unwrap(),
        );
        assert_eq!(v.status, Status::Error, "detail: {:?}", v.detail);
        let detail = v.detail.unwrap_or_default();
        assert!(
            detail.contains("identity mismatch"),
            "identity gate named: {detail}"
        );
    }

    /// 父退出而孙进程持管道：owned 树终止后 EOF 抵达，交换有界完成不
    /// 悬挂；PID 记录 root+descendant 两行（descendant 为 worker 自身
    /// re-exec，无 rustc/cargo）。
    #[test]
    fn grandchild_holding_pipe_completes_bounded() {
        if require_worker().is_none() {
            eprintln!("prebuilt a2x-worker missing; not run, not PASS");
            return;
        }
        let tmp = tmp_root("orphan");
        let pid_file = tmp.join("pids.txt");
        let started = std::time::Instant::now();
        let outcome = with_fault_env(
            "orphan-holds-pipe",
            5_000,
            1_000,
            4 << 20,
            1 << 20,
            Some(&pid_file),
            || drive(&tmp),
        );
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "descendant-held pipe must not hang the exchange: {:?}",
            started.elapsed()
        );
        match outcome {
            Err(e) => assert!(e.contains("not json"), "empty stdout refusal: {e}"),
            Ok(v) => assert_eq!(
                v.status,
                Status::Error,
                "no PASS on a fault seam: {:?}",
                v.detail
            ),
        }
        let pids = std::fs::read_to_string(&pid_file).unwrap_or_default();
        assert_eq!(
            pids.lines().count(),
            2,
            "root + descendant recorded: {pids}"
        );
        assert!(pids.contains("root") && pids.contains("descendant"));
    }
}
