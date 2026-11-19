//! PLAN-982（982/RUST · round 200）：**包级 A2R 多文件工程**——跨模块 `use`
//! 的真实多模块工程经 `transpile_package_rust` 渲染成完整 file set，写入
//! **干净目录**后冷构建、运行，值 oracle 对拍源语义。
//!
//! 验收（排程令第 1 项的语言侧腿）：①跨模块语义（main 用 util 的 pub fn，
//! 值经两模块计算）；②干净目录冷构建＋运行（`rustc` 直编 single-crate 形
//! ——fileset 三件套＋包装自带 `mod` 结构，无需外部 Cargo.toml）；③确定性
//! （同输入两次渲染逐字节一致）；④坏第二模块 ⇒ 具名拒（`ace.rust/compile`），
//! 绝不吐半成品。man 侧工程面（`auto build/gen` 写盘＋RustTrans 零派发收据）
//! 由 `plan982_rust_multifile_cli` 持有。

use auto_lang_ace_host::rust_transpile::transpile_package_rust;

const UTIL: &str = "pub fn twice(n int) int { return n * 2 }\n";
const MAIN: &str = "use util.twice\nfn main() int { return twice(21) }\n";

#[test]
fn p982_rust_package_multifile_cold_builds_and_runs() {
    let fileset = transpile_package_rust(
        &[
            ("util".to_string(), UTIL.to_string()),
            ("main".to_string(), MAIN.to_string()),
        ],
        "main",
    )
    .expect("the two-module project transpiles");
    eprintln!(
        "p982 a2r package files: {:?}",
        fileset
            .files
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        !fileset.files.is_empty(),
        "the package lane emits a real file set"
    );

    // 干净目录写盘（完整工程）。
    let dir = std::env::temp_dir().join(format!(
        "p982-a2r-pkg-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("clean dir");
    for file in &fileset.files {
        let path = dir.join(&file.name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("parent");
        }
        std::fs::write(&path, &file.text).expect("write file");
    }

    // 冷构建＋运行：main.rs 包装是文件集入口（`mod generated`/`mod a2x_runtime`）。
    let entry_path = dir.join("main.rs");
    assert!(
        entry_path.exists(),
        "the file set carries its main.rs wrapper"
    );
    let exe = dir.join("app.exe");
    let build = std::process::Command::new("rustc")
        .arg("--edition")
        .arg("2021")
        .arg("-O")
        .arg(&entry_path)
        .arg("-o")
        .arg(&exe)
        .output()
        .expect("rustc");
    assert!(
        build.status.success(),
        "the generated project cold-builds alone:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let run = std::process::Command::new(&exe).output().expect("run");
    let stdout = String::from_utf8_lossy(&run.stdout).to_string();
    eprintln!("p982 a2r package run: {stdout:?}");
    assert!(
        stdout.contains("value:42"),
        "the cross-module semantics run (twice(21)): {stdout:?}"
    );

    // 确定性：同输入两次渲染逐字节一致。
    let again = transpile_package_rust(
        &[
            ("util".to_string(), UTIL.to_string()),
            ("main".to_string(), MAIN.to_string()),
        ],
        "main",
    )
    .expect("second render");
    assert_eq!(
        fileset.files, again.files,
        "the package render is byte-deterministic"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn p982_rust_package_multifile_bad_module_refuses_named() {
    let err = transpile_package_rust(
        &[
            (
                "util".to_string(),
                "pub fn twice(n int { return 1 }\n".to_string(),
            ),
            ("main".to_string(), MAIN.to_string()),
        ],
        "main",
    )
    .expect_err("a broken second module must refuse");
    eprintln!("p982 a2r package bad module: {err:?}");
    assert_eq!(err.0, "ace.rust/compile", "the refusal is named: {err:?}");
    assert!(
        err.1.contains("util") || err.1.contains("main"),
        "the diagnostic carries the module identity: {err:?}"
    );
}
