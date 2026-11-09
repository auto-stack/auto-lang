//! Explicit original eight release variants, using complete worker output and
//! independent references. The existing contained executor owns every process.
use ace_a2x_core::sha256::sha256_hex;
use ace_core::link::{run_deadlined_captured, CaptureCaps};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FLAGS: [&str; 5] = ["--edition", "2021", "-O", "-C", "debug-assertions=off"];
const BASE: &str = "crates/auto-lang/test/cases/ace-m1-expanded";

pub(super) fn plain(path: &Path) {
    for parent in path.ancestors() {
        let metadata = std::fs::symlink_metadata(parent).unwrap();
        assert!(!metadata.file_type().is_symlink());
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            assert_eq!(metadata.file_attributes() & 0x400, 0, "linked release path");
        }
    }
}

pub(super) fn bytes(path: &Path) -> Vec<u8> {
    plain(path);
    assert!(std::fs::metadata(path).unwrap().len() <= 4 << 20);
    std::fs::read(path).unwrap()
}

pub(super) fn save(path: &Path, value: &Value) {
    let raw = serde_json::to_vec_pretty(value).unwrap();
    assert!(raw.len() <= 4 << 20);
    std::fs::write(path, raw).unwrap();
}

pub(super) fn hex(raw: &[u8]) -> String {
    raw.iter().map(|b| format!("{b:02x}")).collect()
}

pub(super) fn capture(command: &mut Command, phase_ms: u64, whole: Instant) -> Value {
    let argv: Vec<_> = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|s| s.to_str().expect("release UTF-8 argv").to_string())
        .collect();
    let cwd = command.get_current_dir().unwrap().display().to_string();
    let left = Duration::from_secs(360).saturating_sub(whole.elapsed());
    let deadline = Duration::from_millis(phase_ms).min(left);
    assert!(!deadline.is_zero(), "whole original certification budget");
    let now = Instant::now();
    let result = run_deadlined_captured(
        command,
        deadline,
        CaptureCaps {
            stdout_max: 1 << 20,
            stderr_max: 1 << 20,
        },
    );
    let mut value = json!({"argv":argv,"cwd":cwd,"deadline_ms":deadline.as_millis(),
                          "wall_ms":now.elapsed().as_millis()});
    match result {
        Ok(run) => {
            value["terminal"] = json!({"exit_code":run.exit_code,
                "stdout_hex":hex(&run.stdout),"stderr_hex":hex(&run.stderr)});
        }
        Err(error) => value["error"] = format!("{error:?}").into(),
    }
    value
}

pub(super) fn successful(value: &Value) {
    assert_eq!(value["terminal"]["exit_code"], 0, "{value}");
    assert!(value.get("error").is_none(), "{value}");
}

#[test]
fn original_eight_release_variants() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap();
    let artifact_home = std::env::var_os("AUTO_PLAN_ARTIFACTS").map(PathBuf::from);
    let home = artifact_home.clone().unwrap_or_else(std::env::temp_dir);
    plain(&home);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = home.join(format!("scalar-release-941-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let worker = PathBuf::from(env!("CARGO_BIN_EXE_a2x-worker"));
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let cases: Vec<_> = super::scalar_semantics::cases()
        .into_iter()
        .filter(|case| case.release_too)
        .collect();
    assert_eq!(cases.len(), 8, "original release set changed");
    let mut index = json!({"schema":"ace.m1.scalar-release-index/1","flags":FLAGS,
        "producer":{"role":"ace-a2x-worker::conformance::scalar_release_941::original_eight_release_variants",
        "source_sha256":sha256_hex(include_bytes!("scalar_release_941.rs")),
        "table_sha256":sha256_hex(include_bytes!("scalar_semantics.rs")),
        "reference_contract_sha256":sha256_hex(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/../../../../crates/auto-lang/test/cases/ace-m1-expanded/scalar-reference-contracts.tsv"))),
        "worker_path":worker,"execution_path":std::env::current_exe().unwrap(),"rustc":rustc},"required":8,"cases":[]});
    let mut vv = Command::new(&rustc);
    vv.arg("-vV").current_dir(&root);
    index["compiler_identity"] = capture(&mut vv, 120_000, Instant::now());
    save(&root.join("index.json"), &index);
    successful(&index["compiler_identity"]);
    for case in cases {
        let whole = Instant::now();
        let directory = root.join(&case.name);
        let actual = directory.join("actual");
        let reference = directory.join("reference");
        std::fs::create_dir_all(&actual).unwrap();
        std::fs::create_dir(&reference).unwrap();
        let source = repo
            .join(BASE)
            .join("scalar-src")
            .join(format!("{}.at", case.name));
        assert_eq!(sha256_hex(&bytes(&source)), sha256_hex(case.src.as_bytes()));
        let independent = repo
            .join(BASE)
            .join("scalar-reference")
            .join(format!("{}.rs", case.name));
        let reference_bytes = bytes(&independent);
        std::fs::write(reference.join("main.rs"), &reference_bytes).unwrap();
        let caps: Vec<_> = if case.trace {
            vec!["hir.test.trace.v1"]
        } else {
            vec![]
        };
        let request = json!({"op":"daily-emit","source_path":source,"profile":case.profile,
            "entry":"main","caps":caps,"out_dir":actual,
            "nonce":sha256_hex(format!("{}-{stamp}-{}",std::process::id(),case.name).as_bytes())});
        let mut record = json!({"schema":"ace.m1.scalar-release-execution/1","case":case.name,
            "profile":case.profile,"source_path":source,"source_sha256":sha256_hex(case.src.as_bytes()),
            "reference_path":independent,"reference_sha256":sha256_hex(&reference_bytes),
            "request":request,"flags":FLAGS,"actual_files":{},"processes":{}});
        let receipt = directory.join("receipt.json");
        let mut daily = Command::new(&worker);
        daily.arg(request.to_string()).current_dir(&actual);
        record["daily"] = capture(&mut daily, 360_000, whole);
        save(&receipt, &record);
        successful(&record["daily"]);
        for name in ["main.rs", "a2x_runtime.rs", "generated.rs", "manifest.json"] {
            record["actual_files"][name] = sha256_hex(&bytes(&actual.join(name))).into();
        }
        for (leg, work) in [("actual", &actual), ("reference", &reference)] {
            let exe = work.join("release.exe");
            let mut compile = Command::new(&rustc);
            compile
                .args(FLAGS)
                .arg("-o")
                .arg(&exe)
                .arg("main.rs")
                .current_dir(work);
            record["processes"][leg] = json!({"compile":capture(&mut compile,120_000,whole)});
            save(&receipt, &record);
            successful(&record["processes"][leg]["compile"]);
            let mut run = Command::new(&exe);
            run.current_dir(work);
            record["processes"][leg]["run"] = capture(&mut run, 30_000, whole);
            save(&receipt, &record);
            successful(&record["processes"][leg]["run"]);
            let line = match &case.expect {
                super::scalar_semantics::Expect::Value(value) => format!("value:{value}\n"),
                super::scalar_semantics::Expect::Trap(trap) => format!("trap:{trap}\n"),
            };
            let marks: String = case.marks.iter().map(|mark| format!("{mark}\n")).collect();
            assert_eq!(
                record["processes"][leg]["run"]["terminal"]["stdout_hex"],
                hex(line.as_bytes())
            );
            assert_eq!(
                record["processes"][leg]["run"]["terminal"]["stderr_hex"],
                hex(marks.as_bytes())
            );
        }
        record["whole_wall_ms"] = (whole.elapsed().as_millis() as u64).into();
        save(&receipt, &record);
        let reference = json!({"case":case.name,"path":format!("{}/receipt.json",case.name),
                               "sha256":sha256_hex(&bytes(&receipt))});
        index["cases"].as_array_mut().unwrap().push(reference);
        save(&root.join("index.json"), &index);
    }
    println!(
        "eight original release variants observed: {}",
        root.display()
    );
    if artifact_home.is_none() {
        plain(&root);
        std::fs::remove_dir_all(root).unwrap();
    }
}
