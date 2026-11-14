//! PLAN-964（T-03，native 端）：同一 mn 管线真实构建+运行，exit code =
//! main 结果。按原支持域：闭包（Move 快照捕获）与 FStr（TextConcat +
//! i32 显示）在 native 域执行；对象/nil（动态值面）为具名 mn 拒绝
//! （负控，沿 native 拒 DYN 先例——"按原支持域"臂）。

use ace_native::driver::MnDriver;

fn run_native_checked(
    src: &str,
    tag: &str,
    compile: fn(
        &str,
        &str,
        &[String],
    ) -> Result<ace_core::verify::CheckedModule, Vec<ace_core::atom_text::Diagnostic>>,
) -> i32 {
    let checked = compile(src, "p964.at", &[]).expect("source compiles");
    let tag = format!("mn964-{tag}-{}", std::process::id());
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&checked, "d_main", &mn_dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&mn_dir);
    assert!(mn.ok, "mn leg failed: {:?}", mn.error);
    mn.exit_code.unwrap_or(-1)
}

const CLOSURE_CAPTURE_HOF: &str = "\
fn scale(factor int) fn(int) int {
    return (y int) => y * factor
}
fn main() int {
    let f = scale(3)
    let r0 = f(1)
    let r1 = f(2)
    let r2 = f(3)
    return r0 * 100 + r1 * 10 + r2
}
";

const FSTRING: &str = "\
fn main() int {
    let name = \"AutoLang\"
    let age = 1
    let greeting = f\"Hello, $name\"
    let info = f\"Name: $name, Age: $age\"
    let x = 10
    let y = 20
    let result = f\"Result: ${x + y}\"
    let msg = f\"The value is $x and ${y * 2}\"
    let a = 5
    let b = 3
    let sum = f\"Sum: ${a + b}, Product: ${a * b}\"
    if greeting != \"Hello, AutoLang\" { return 0 }
    if info != \"Name: AutoLang, Age: 1\" { return 0 }
    if result != \"Result: 30\" { return 0 }
    if msg != \"The value is 10 and 40\" { return 0 }
    if sum != \"Sum: 8, Product: 15\" { return 0 }
    return 42
}
";

/// source02（native 端）：factor=3 捕获 + HOF → exit=369（[3,6,9]）。
#[test]
fn native_closure_capture_hof_exit_369() {
    let exit = run_native_checked(
        CLOSURE_CAPTURE_HOF,
        "hof",
        |s, p, c| ace_core::source::compile_closures_source(s, p, c).map(|x| x.checked),
    );
    assert_eq!(exit, 369, "mapped [3,6,9] oracle on the mn leg");
}

/// source03（native 端）：FStr 插值 + i32 显示 → exit=42。
#[test]
fn native_fstring_interpolation_exit_42() {
    let exit = run_native_checked(
        FSTRING,
        "fstr",
        |s, p, c| ace_core::source::compile_aggregates_source(s, p, c).map(|x| x.checked),
    );
    assert_eq!(exit, 42, "fstring oracle on the mn leg");
}

/// 对象（动态值面）按原支持域 = 具名 mn 拒绝（负控）。
#[test]
fn native_object_surface_is_named_rejection() {
    let src = "fn main() int { var obj = { x: 1 }; obj[\"y\"] = 2; return obj[\"x\"] }";
    let checked = ace_core::source::compile_aggregates_source(src, "p964.at", &[])
        .expect("aggregates source compiles")
        .checked;
    let tag = format!("mn964-obj-{}", std::process::id());
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&checked, "d_main", &mn_dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&mn_dir);
    assert!(!mn.ok, "object surface must be a named mn rejection");
    let msg = format!("{:?}", mn.error);
    assert!(
        msg.contains("object") || msg.contains("dyn"),
        "expected a dynamic/object rejection, got {msg}"
    );
}
#[test]
fn probe_native_fstr_mir() {
    let src = "fn main() int {\n    let name = \"AutoLang\"\n    let x = 10\n    let y = 20\n    let result = f\"Result: ${x + y}\"\n    if result != \"Result: 30\" { return 0 }\n    return 42\n}\n";
    let compiled = ace_core::source::compile_aggregates_source(src, "p964.at", &[]).expect("compile");
    let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 964).expect("lower");
    println!("PROFILE {}", mir.profile);
    for (i, t) in mir.types.iter().enumerate() {
        println!("T{i}: {t:?}");
    }
    for (fi, f) in mir.funcs.iter().enumerate() {
        println!("F{fi} name={} slots:", f.name);
        for (si, s) in f.slots.iter().enumerate() {
            println!("  slot{si} ty={}", s.ty);
        }
    }
}
