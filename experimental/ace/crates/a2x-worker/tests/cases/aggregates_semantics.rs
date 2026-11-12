//! PLAN-962 T-03 a2x legs: the aggregates surface through the REAL
//! provider true chains (aggregates source profile → HIR → a2x semantic
//! view → provider fileset → real toolchain run), against INDEPENDENT
//! hand-computed oracles. The plan added zero new HIR node kinds (tuples
//! ride records, Box/grid ride lists), so the A2R/A2C lowering admits the
//! composites once the view's profile/aggregates capability lands.

use ace_a2x_rust::{assemble_file_set, lower_program};

fn tools_available() -> bool {
    std::process::Command::new(
        std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()),
    )
    .arg("--version")
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}

/// The Rust true chain over the arena-free aggregate shapes (tuple +
/// destructuring + named ctor + range loop). Arena-carrying shapes
/// (lists/Box/grid) consume the CollectionsBodyFacts free schedule on
/// the collections family lane (store_coll precedent) — the fileset lane
/// here covers the record/tuple emission surface.
#[test]
fn aggregates_rust_true_chain() {
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    for (name, src, want) in [
        (
            "ag-tuple-destructure",
            "fn main() int {\n    let t = (19, 23)\n    let (x, y) = t\n    return x + y\n}",
            42,
        ),
        // NAMED PENDING (plan 9): tuple as a CALL ARGUMENT on the C provider
        // needs temp materialization (cl C2440 const-rec-pointer ABI); the
        // Rust provider passed this case (unit-6).
        (
            "ag-named-ctor",
            "fn main() int {\n    let p = Pair { right: 2, left: 40 }\n    return p.left + p.right\n}\ntype Pair { left int; right int }",
            42,
        ),
        (
            "ag-range-sum",
            "fn main() int {\n    var s = 0\n    for i in 1 .. 5 {\n        s = s + i\n    }\n    for j in 1 ..= 3 {\n        s = s + j * 10\n    }\n    return s\n}",
            70,
        ),
    ] {
        let got = run_rust(src);
        assert_eq!(got, want, "case {name}");
    }
}

fn run_rust(src: &str) -> i32 {
    use ace_a2x_core::SemanticView;
    let caps: Vec<String> = vec![];
    let compiled = ace_core::source::compile_aggregates_source(src, "probe", &caps)
        .map_err(|d| format!("compile: {}", d[0].message))
        .unwrap();
    let view = SemanticView::from_checked(&compiled.checked)
        .map_err(|r| format!("view: {r:?}"))
        .unwrap();
    let program = lower_program(&view, "main")
        .map_err(|r| format!("lower: {r:?}"))
        .unwrap();
    let fs = assemble_file_set(&view, &program, &compiled.input_hash, "main");

    let dir = std::env::temp_dir().join(format!("a2x-ag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &fs.manifest).unwrap();

    let exe = dir.join("out.exe");
    let mut compiler = std::process::Command::new(
        std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()),
    );
    compiler
        .arg("--edition")
        .arg("2021")
        .arg("-O")
        .arg("-o")
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir);
    let out = compiler.output().expect("spawn rustc");
    if !out.status.success() {
        let s = String::from_utf8_lossy(&out.stderr).to_string();
        let _ = std::fs::remove_dir_all(&dir);
        panic!("rustc failed:\n{s}");
    }
    let run = std::process::Command::new(&exe)
        .current_dir(&dir)
        .output()
        .expect("run exe");
    let _ = std::fs::remove_dir_all(&dir);
    let stdout = String::from_utf8_lossy(&run.stdout).to_string();
    // wrapper prints `value:N` / `trap:N`
    let line = stdout.lines().next().unwrap_or("");
    let n: i32 = line
        .trim_start_matches("value:")
        .parse()
        .unwrap_or_else(|_| panic!("no value in {stdout:?}"));
    n
}

/// PLAN-962 T-03 a2x-c leg (real MSVC cl): the same arena-free aggregate
/// cases through the C provider session assembly → cl compile+run → the
/// same independent oracle table (mirrors the numeric C true chain).
#[test]
fn aggregates_c_true_chain() {
    use ace_a2x_c::CTargetProvider;
    use ace_a2x_store::A2xSession;
    use ace_core::source::SourceProfile;
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    let store = std::env::temp_dir().join(format!("a2x-ag-c-store-{}", std::process::id()));
    let outroot = std::env::temp_dir().join(format!("a2x-ag-c-out-{}", std::process::id()));
    for (name, src, want) in [
        (
            "ag-tuple-destructure",
            "fn main() int {\n    let t = (19, 23)\n    let (x, y) = t\n    return x + y\n}",
            42,
        ),
        // NAMED PENDING (plan 9): tuple as a CALL ARGUMENT on the C provider
        // needs temp materialization (cl C2440 const-rec-pointer ABI); the
        // Rust provider passed this case (unit-6).
        (
            "ag-named-ctor",
            "fn main() int {\n    let p = Pair { right: 2, left: 40 }\n    return p.left + p.right\n}\ntype Pair { left int; right int }",
            42,
        ),
        (
            "ag-range-sum",
            "fn main() int {\n    var s = 0\n    for i in 1 .. 5 {\n        s = s + i\n    }\n    for j in 1 ..= 3 {\n        s = s + j * 10\n    }\n    return s\n}",
            70,
        ),
    ] {
        let mut s = A2xSession::open(
            &store,
            "a2c-ag",
            "probe",
            128 * 1024 * 1024,
            CTargetProvider::v1(),
        )
        .expect("open C session");
        let asm = s
            .cold_assemble_profile(src, SourceProfile::Aggregates, &[], "main", false)
            .unwrap_or_else(|e| panic!("C assembly failed for {name} (dir kept): {e:?}"));
        let dir = outroot.join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (path, content) in &asm.files {
            std::fs::write(dir.join(path), content).unwrap();
        }
        std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
        let (stdout, _stderr) =
            ace_a2x_worker::crun::compile_and_run_c(&dir, "a2x_ag.exe")
                .unwrap_or_else(|e| panic!("C compile/run failed for {name}: {e}"));
        let line = stdout.lines().next().unwrap_or("").to_string();
        let got: i32 = line
            .strip_prefix("value:")
            .map(|v| v.trim().parse().unwrap())
            .unwrap_or_else(|| panic!("case {name}: unexpected C output {stdout:?}"));
        assert_eq!(got, want, "case {name} (C)");
    }
    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&outroot);
}
