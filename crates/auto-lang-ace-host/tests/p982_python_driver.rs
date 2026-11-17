//! PLAN-982（982/PY，第一步）：A2Py 宿主驱动的端到端证据。
//!
//! 断言：
//! 1. 一段真实 Auto 源码能渲染成 A2Py file set（manifest 带 schema 标识）；
//! 2. 同输入两次渲染**逐字节一致**（files + manifest）；
//! 3. 产出的目录能**真的用 python 跑起来**并打印源语义（本机有 python 3.14）。

use std::path::PathBuf;
use std::process::Command;

const SRC: &str = r#"
fn add(a int, b int) int { return a + b }

fn main() int {
    var total = add(2, 3)
    return total
}
"#;

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "p982-py-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("scratch");
    dir
}

#[test]
fn p982_python_driver_renders_a_runnable_program() {
    let fileset =
        auto_lang_ace_host::python_transpile::transpile_scalar_source(SRC, "p982-py", "main")
            .expect("the A2Py driver renders");
    assert!(
        !fileset.files.is_empty(),
        "the file set is not empty: {:?}",
        fileset.files.iter().map(|f| &f.name).collect::<Vec<_>>()
    );
    assert!(
        fileset.manifest.contains("python") || fileset.manifest.contains("a2x"),
        "the manifest carries its schema: {}",
        fileset.manifest
    );

    // 确定性：同输入两次，逐字节一致。
    let again =
        auto_lang_ace_host::python_transpile::transpile_scalar_source(SRC, "p982-py", "main")
            .expect("render twice");
    assert_eq!(fileset, again, "the driver is deterministic");

    // 实跑：写盘后用 python 执行入口。
    let dir = scratch("run");
    for file in &fileset.files {
        let path = dir.join(&file.name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("dir");
        }
        std::fs::write(&path, &file.text).expect("write");
    }
    let entry = fileset
        .files
        .iter()
        .map(|f| &f.name)
        .find(|n| n.ends_with("main.py"))
        .unwrap_or_else(|| {
            panic!(
                "no python entry among {:?}",
                fileset.files.iter().map(|f| &f.name).collect::<Vec<_>>()
            )
        });
    let out = Command::new("python")
        .arg("-I")
        .arg("-B")
        .arg(entry)
        .current_dir(&dir)
        .output()
        .expect("spawn python");
    assert!(
        out.status.success(),
        "the generated program must run:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains('5'),
        "the program prints the source semantics (5), got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
