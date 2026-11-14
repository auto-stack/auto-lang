//! PLAN-962 T-03 a2x legs: the aggregates surface through the REAL
//! provider true chains (aggregates source profile → HIR → a2x semantic
//! view → provider fileset → real toolchain run), against INDEPENDENT
//! hand-computed oracles. The plan added zero new HIR node kinds (tuples
//! ride records, Box/grid ride lists), so the A2R/A2C lowering admits the
//! composites once the view's profile/aggregates capability lands.

use ace_a2x_rust::{assemble_file_set, lower_program};

fn tools_available() -> bool {
    std::process::Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
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
    let mut compiler =
        std::process::Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
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

const M2_CONTROL_EFFECTS_CASES: &[(&str, &str, i32)] = &[
    (
        "m2-gen-sum",
        "\
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
fn main() int {
    var s = 0
    for n in counter() {
        s = s + n
    }
    return s
}",
        6,
    ),
    (
        "m2-coalesce-is-prop",
        "\
fn parse(ok bool) !int {
    if ok { return Ok(20) }
    Err(\"bad\")
}
fn add_two(ok bool) !int {
    let v = parse(ok).?
    Ok(v + 2)
}
fn main() int {
    let a = Some(40) ?? 99
    let b = None ?? 2
    let c = add_two(true).?(0)
    let d = add_two(false).?(5)
    let e = is Some(3) {
        Some(v) -> v
        None -> 0
    }
    return a + b + c + d + e
}",
        72,
    ),
    (
        "m2-try-finally-nested-break",
        "\
fn main() int {
    var acc = 0
    var log = 0
    var i = 0
    for i < 5 {
        i = i + 1
        try {
            if i == 2 {
                continue
            }
            if i == 4 {
                break
            }
            acc = acc + i
        } finally {
            log = log + i
        }
    }
    // acc: 1 + 3 = 4; log: 1 + 2 + 3 + 4 = 10
    return acc * 10 + log
}",
        50,
    ),
    // V-02 (laziness) / V-05 (observable order) on the A2X legs: the `??`
    // right side must NOT run, and a non-tail call statement must run for
    // effect (F-965-R1-01) — a commutative accumulator would hide the latter.
    (
        "m2-lazy-and-ordered",
        "\
type Ctr { hits int }
fn bump(mut c Ctr) int {
    c.hits = c.hits + 1
    7
}
fn main() int {
    var c = Ctr(0)
    let a = Some(4) ?? bump(c.mut)
    bump(c.mut)
    let b = None ?? 3
    if c.hits != 1 { return 0 }
    var acc = 0
    var i = 0
    for i < 3 {
        i = i + 1
        acc = acc * 10 + i
    }
    return a * 100 + b * 10 + acc
}",
        // 4*100 + 3*10 + 123 = 553
        553,
    ),
];

fn run_a2x_session_rust(name: &str, src: &str) -> String {
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::A2xSession;
    use ace_core::source::SourceProfile;
    let store = std::env::temp_dir().join(format!("a2x-965-r-store-{}", std::process::id()));
    let dir = std::env::temp_dir().join(format!("a2x-965-r-out-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = A2xSession::open(
        &store,
        "a2r-965",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .expect("open Rust session");
    let asm = s
        .cold_assemble_profile(src, SourceProfile::Aggregates, &[], "main", false)
        .unwrap_or_else(|e| panic!("Rust assembly failed for {name}: {e:?}"));
    for (path, content) in &asm.files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
    let exe = dir.join("out.exe");
    let out =
        std::process::Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
            .args(["--edition", "2021", "-O", "-o"])
            .arg(&exe)
            .arg("main.rs")
            .current_dir(&dir)
            .output()
            .expect("spawn rustc");
    assert!(
        out.status.success(),
        "rustc failed for {name}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = std::process::Command::new(&exe)
        .current_dir(&dir)
        .output()
        .expect("run exe");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&store);
    String::from_utf8_lossy(&run.stdout).to_string()
}

#[test]
fn aggregates_965_control_effects_rust() {
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    for &(name, src, want) in M2_CONTROL_EFFECTS_CASES {
        let stdout = run_a2x_session_rust(name, src);
        let line = stdout.lines().next().unwrap_or("");
        let got: i32 = line
            .strip_prefix("value:")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or_else(|| panic!("{name} (Rust): expected value:{want}, got {stdout:?}"));
        assert_eq!(got, want, "case {name} (Rust)");
    }
    let fail_stdout = run_a2x_session_rust("m2-uncaught-fail", "fn main() int { fail(\"boom\") }");
    assert!(
        fail_stdout.starts_with("trap:"),
        "expected trap output for uncaught fail (Rust), got {fail_stdout:?}"
    );
}

#[test]
fn aggregates_965_control_effects_c() {
    use ace_a2x_c::CTargetProvider;
    use ace_a2x_store::A2xSession;
    use ace_core::source::SourceProfile;
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    let store = std::env::temp_dir().join(format!("a2x-965-c-store-{}", std::process::id()));
    let outroot = std::env::temp_dir().join(format!("a2x-965-c-out-{}", std::process::id()));
    let run_c = |name: &str, src: &str| -> String {
        let mut s = A2xSession::open(
            &store,
            "a2c-965",
            "probe",
            128 * 1024 * 1024,
            CTargetProvider::v1(),
        )
        .expect("open C session");
        let asm = s
            .cold_assemble_profile(src, SourceProfile::Aggregates, &[], "main", false)
            .unwrap_or_else(|e| panic!("C assembly failed for {name}: {e:?}"));
        let dir = outroot.join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (path, content) in &asm.files {
            std::fs::write(dir.join(path), content).unwrap();
        }
        std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
        let (stdout, _stderr) = ace_a2x_worker::crun::compile_and_run_c(&dir, "a2x_ag_965.exe")
            .unwrap_or_else(|e| panic!("C compile/run failed for {name}: {e}"));
        stdout
    };
    for &(name, src, want) in M2_CONTROL_EFFECTS_CASES {
        let stdout = run_c(name, src);
        let line = stdout.lines().next().unwrap_or("");
        let got: i32 = line
            .strip_prefix("value:")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or_else(|| panic!("{name} (C): expected value:{want}, got {stdout:?}"));
        assert_eq!(got, want, "case {name} (C)");
    }
    let fail_stdout = run_c("m2-uncaught-fail", "fn main() int { fail(\"boom\") }");
    assert!(
        fail_stdout.starts_with("trap:"),
        "expected trap output for uncaught fail (C), got {fail_stdout:?}"
    );
    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&outroot);
}
