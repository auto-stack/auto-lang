//! Original 902 four optimized variants, complete emitted programs versus independent references.
//! The scalar observer's contained capture is shared; source/profile/flags remain distinct.
use super::scalar_release_941::{bytes, capture, hex, plain, save, successful};
use ace_a2x_core::sha256::sha256_hex;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
const FLAGS: [&str; 3] = ["--edition", "2021", "-O"];
const BASE: &str = "crates/auto-lang/test/cases/ace-m1-expanded";

#[test]
fn original_four_release_variants() {
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
    let root = home.join(format!(
        "record-text-release-941-{}-{stamp}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let worker = PathBuf::from(env!("CARGO_BIN_EXE_a2x-worker"));
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let cases = [
        ("r07", "record", 42),
        ("r08", "record", 42),
        ("t03", "text-record", 23),
        ("t05", "text", 42),
    ];
    let mut index = json!({"schema":"ace.m1.record-text-release-index/1","flags":FLAGS,
        "producer":{"role":"ace-a2x-worker::conformance::record_text_release_941::original_four_release_variants",
        "source_sha256":sha256_hex(include_bytes!("record_text_release_941.rs")),
        "table_sha256":sha256_hex(include_bytes!("record_text_902.rs")),
        "reference_contract_sha256":sha256_hex(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/../../../../crates/auto-lang/test/cases/ace-m1-expanded/record-text-reference-contracts.tsv"))),
        "worker_path":worker,"execution_path":std::env::current_exe().unwrap(),"rustc":rustc},"required":4,"cases":[]});
    let mut vv = Command::new(&rustc);
    vv.arg("-vV").current_dir(&root);
    index["compiler_identity"] = capture(&mut vv, 120_000, Instant::now());
    save(&root.join("index.json"), &index);
    successful(&index["compiler_identity"]);
    for (name, profile, expected) in cases {
        let whole = Instant::now();
        let directory = root.join(name);
        let actual = directory.join("actual");
        let reference = directory.join("reference");
        std::fs::create_dir_all(&actual).unwrap();
        std::fs::create_dir(&reference).unwrap();
        let original = repo
            .join("crates/auto-lang/test/cases/ace-m1/src")
            .join(format!("{}.at", name.to_uppercase()));
        let source = repo
            .join("crates/auto-lang/test/cases/a2c-hir-record-text-925/src")
            .join(format!("{}.at", name.to_uppercase()));
        let source_bytes = bytes(&source);
        assert_eq!(source_bytes, bytes(&original), "original 902 source bytes");
        let independent = repo
            .join(BASE)
            .join("record-text-reference")
            .join(format!("record-text-base--{name}.rs"));
        let reference_bytes = bytes(&independent);
        std::fs::write(reference.join("main.rs"), &reference_bytes).unwrap();
        let caps: Vec<&str> = if profile == "record" {
            vec![]
        } else {
            vec!["ac.runtime.text.v1"]
        };
        let request = json!({"op":"daily-emit","source_path":source,"profile":profile,
            "entry":"main","caps":caps,"out_dir":actual,
            "nonce":sha256_hex(format!("{}-{stamp}-{}",std::process::id(),name).as_bytes())});
        let mut record = json!({"schema":"ace.m1.record-text-release-execution/1","case":name,
            "profile":profile,"source_path":source,"source_sha256":sha256_hex(&source_bytes),
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
            let line = format!("value:{expected}\n");
            let marks = String::new();
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
        let reference = json!({"case":name,"path":format!("{}/receipt.json",name),
                               "sha256":sha256_hex(&bytes(&receipt))});
        index["cases"].as_array_mut().unwrap().push(reference);
        save(&root.join("index.json"), &index);
    }
    println!(
        "four original release variants observed: {}",
        root.display()
    );
    if artifact_home.is_none() {
        plain(&root);
        std::fs::remove_dir_all(root).unwrap();
    }
}
