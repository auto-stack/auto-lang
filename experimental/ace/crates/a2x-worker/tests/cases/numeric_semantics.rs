//! PLAN-961 numeric semantics through the REAL provider true chains: the
//! numeric source profile → HIR → a2x semantic view → both providers
//! (Rust fileset + real rustc run; C fileset + real MSVC cl run), each
//! against an INDEPENDENT hand-computed oracle (values / trap names).

use ace_a2x_rust::{assemble_file_set, lower_program};

fn tools_available() -> bool {
    std::process::Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[derive(Debug)]
pub(crate) enum Expect {
    Value(i32),
    Trap(&'static str),
}

pub(crate) struct Case {
    pub(crate) name: String,
    pub(crate) src: String,
    pub(crate) expect: Expect,
}

fn v(x: i32) -> Expect {
    Expect::Value(x)
}

fn t(x: &'static str) -> Expect {
    Expect::Trap(x)
}

/// The numeric corpus (V-01..V-05 + V-08 anchors observable through the
/// entry channel): i64 magnitude arithmetic, u8 width checks, shifts,
/// conversions, float lanes with the frozen ±0-div trap.
pub(crate) fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "n01-i64-magnitude".into(),
            src: "fn main() int {\n    let a = 2147483648i64\n    let b = a + 1i64\n    return b.as(i32)\n}".into(),
            // 2147483649 narrows to i32: TRAPS by the frozen checked
            // matrix — value observable stays in-range below instead.
            expect: t("Overflow"),
        },
        Case {
            name: "n01b-i64-in-range".into(),
            src: "fn main() int {\n    let a = 2000000000i64\n    let b = a + 12i64\n    return b.as(i32)\n}".into(),
            expect: v(2000000012),
        },
        Case {
            name: "n02-u8-width".into(),
            src: "fn main() int {\n    let a = 200u8\n    let b = 55u8\n    let c = a + b\n    return c.as(i32)\n}".into(),
            expect: v(255),
        },
        Case {
            name: "n03-u8-overflow".into(),
            src: "fn main() int {\n    let a = 200u8\n    let b = 100u8\n    let c = a + b\n    let _ = c\n    return 0\n}".into(),
            expect: t("Overflow"),
        },
        Case {
            name: "n04-shift-width-trap".into(),
            src: "fn main() int {\n    let x = 1i64\n    let r = x.shl(64i64)\n    let _ = r\n    return 0\n}".into(),
            expect: t("InvalidScalar"),
        },
        Case {
            name: "n05-shift-value".into(),
            src: "fn main() int {\n    let x = 1i64\n    let f = x.shl(62i64)\n    let g = f.shr(60i64)\n    return g.as(i32)\n}".into(),
            expect: v(4),
        },
        Case {
            name: "n06-i64-min-div".into(),
            src: "fn main() int {\n    let mn = -9223372036854775808i64\n    let m1 = -1i64\n    return (mn / m1).as(i32)\n}".into(),
            expect: t("DivOverflow"),
        },
        Case {
            name: "n07-float-div-zero".into(),
            src: "fn main() int {\n    let z = 0.0d\n    let v = 1.5d / z\n    let _ = v\n    return 0\n}".into(),
            expect: t("DivByZero"),
        },
        Case {
            name: "n08-narrow-trap".into(),
            src: "fn main() int {\n    let a = 300i64\n    return a.as(u8).as(i32)\n}".into(),
            expect: t("Overflow"),
        },
        Case {
            name: "n09-sext-value".into(),
            src: "fn main() int {\n    let a = -1i8\n    let b = a.as(i64)\n    return b.as(i32)\n}".into(),
            expect: v(-1),
        },
        Case {
            name: "n10-trunc-zero".into(),
            src: "fn main() int {\n    let a = -0.9d\n    return a.as(i64).as(i32)\n}".into(),
            expect: v(0),
        },
        Case {
            name: "n11-f64-to-i64-range".into(),
            src: "fn main() int {\n    let a = 9223372036854775807i64\n    let f = a.as(f64)\n    let b = f * f\n    let c = b * f\n    return c.as(i64).as(i32)\n}".into(),
            expect: t("InvalidScalar"),
        },
        Case {
            name: "n12-bit-ops".into(),
            src: "fn main() int {\n    let a = 12i64\n    let b = 10i64\n    let c = a & b\n    let d = a | b\n    let e = a ^ b\n    return (c + d + e).as(i32)\n}".into(),
            expect: v((12 & 10) + (12 | 10) + (12 ^ 10)),
        },
    ]
}

/// The Rust true chain (real rustc): wrapper prints `value:N`/`trap:N`.
#[test]
fn numeric_rust_true_chain() {
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    for case in cases() {
        let got = run_rust(&case);
        match (&case.expect, got) {
            (Expect::Value(w), Ok(g)) => assert_eq!(*w, g, "case {}", case.name),
            (Expect::Trap(w), Err(g)) => assert_eq!(*w, g, "case {}", case.name),
            (w, g) => panic!("case {}: expected {w:?}, got {g:?}", case.name),
        }
    }
}

fn run_rust(case: &Case) -> Result<i32, String> {
    use ace_a2x_core::SemanticView;
    let caps: Vec<String> = vec![];
    let compiled = ace_core::source::compile_numeric_source(&case.src, "probe", &caps)
        .map_err(|d| format!("compile: {}", d[0].message))?;
    let view = SemanticView::from_checked(&compiled.checked).map_err(|r| format!("view: {r:?}"))?;
    let program = lower_program(&view, "main").map_err(|r| format!("lower: {r:?}"))?;
    let fs = assemble_file_set(&view, &program, &compiled.input_hash, "main");

    let dir = std::env::temp_dir().join(format!("a2x-num-{}-{}", case.name, std::process::id()));
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
        panic!("case {}: rustc failed:\n{}", case.name, s);
    }
    let run = std::process::Command::new(&exe)
        .current_dir(&dir)
        .output()
        .expect("run exe");
    let stdout = String::from_utf8_lossy(&run.stdout).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    let line = stdout.lines().next().unwrap_or("").to_string();
    if let Some(v) = line.strip_prefix("value:") {
        return Ok(v.trim().parse().unwrap());
    }
    if let Some(t) = line.strip_prefix("trap:") {
        return Err(t.trim().to_string());
    }
    Err(format!("unexpected output: {stdout:?}"))
}

/// The C true chain (real MSVC cl): numeric profile through the C
/// provider session assembly → cl compile+run → same oracle table.
#[test]
fn numeric_c_true_chain() {
    use ace_a2x_c::CTargetProvider;
    use ace_a2x_store::A2xSession;
    use ace_core::source::SourceProfile;
    if !tools_available() {
        eprintln!("infra-blocked — skipped");
        return;
    }
    let store = std::env::temp_dir().join(format!("a2x-num-c-store-{}", std::process::id()));
    let outroot = std::env::temp_dir().join(format!("a2x-num-c-out-{}", std::process::id()));
    for case in cases() {
        let mut s = A2xSession::open(
            &store,
            "a2c-num",
            "probe",
            128 * 1024 * 1024,
            CTargetProvider::v1(),
        )
        .expect("open C session");
        let asm = s
            .cold_assemble_profile(&case.src, SourceProfile::Numeric, &[], "main", false)
            .unwrap_or_else(|e| panic!("C assembly failed for {} (dir kept): {:?}", case.name, e));
        let dir = outroot.join(&case.name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (path, content) in &asm.files {
            std::fs::write(dir.join(path), content).unwrap();
        }
        std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
        let (stdout, _stderr) = ace_a2x_worker::crun::compile_and_run_c(&dir, "a2x_num.exe")
            .unwrap_or_else(|e| panic!("C compile/run failed for {}: {}", case.name, e));
        let line = stdout.lines().next().unwrap_or("").to_string();
        let got = if let Some(v) = line.strip_prefix("value:") {
            Ok::<i32, String>(v.trim().parse().unwrap())
        } else if let Some(t) = line.strip_prefix("trap:") {
            Err(t.trim().to_string())
        } else {
            panic!("case {}: unexpected C output {stdout:?}", case.name)
        };
        match (&case.expect, got) {
            (Expect::Value(w), Ok(g)) => assert_eq!(*w, g, "case {} (C)", case.name),
            (Expect::Trap(w), Err(g)) => assert_eq!(*w, g, "case {} (C)", case.name),
            (w, g) => panic!("case {} (C): expected {w:?}, got {g:?}", case.name),
        }
    }
    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&outroot);
}
