//! PLAN-982（982/WEBTARGET，第一步）：A2JS 宿主驱动的端到端证据。
//!
//! 断言：渲染非空、manifest 带 schema、**同输入逐字节确定**，并且产物能**用 node
//! 真跑**出源语义（本机 node 25；不需要 `tsc`，因为驱动选的是 JavaScript provider）。

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
        "p982-js-{tag}-{}-{}",
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
fn p982_script_driver_renders_a_runnable_program() {
    let fileset =
        auto_lang_ace_host::script_transpile::transpile_scalar_source(SRC, "p982-js", "main")
            .expect("the A2JS driver renders");
    assert!(!fileset.files.is_empty(), "the file set is not empty");
    assert!(
        fileset.manifest.contains("a2x") || fileset.manifest.contains("script"),
        "the manifest carries its schema: {}",
        fileset.manifest
    );

    let again =
        auto_lang_ace_host::script_transpile::transpile_scalar_source(SRC, "p982-js", "main")
            .expect("render twice");
    assert_eq!(fileset, again, "the driver is deterministic");

    let dir = scratch("run");
    for file in &fileset.files {
        let path = dir.join(&file.name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("dir");
        }
        std::fs::write(&path, &file.text).expect("write");
    }
    let names: Vec<&str> = fileset.files.iter().map(|f| f.name.as_str()).collect();
    let entry = names
        .iter()
        .find(|n| n.ends_with(".mjs") || n.ends_with(".js"))
        .unwrap_or_else(|| panic!("no script entry among {names:?}"));
    let out = Command::new("node")
        .arg(entry)
        .current_dir(&dir)
        .output()
        .expect("spawn node");
    assert!(
        out.status.success(),
        "the generated script must run:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains('5'),
        "the script prints the source semantics (5), got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
/// TS（强类型）通道：只验证**渲染 + 确定性**；本机无 `tsc`，执行面**不声称**。
#[test]
fn p982_script_driver_renders_typescript_deterministically() {
    let fileset =
        auto_lang_ace_host::script_transpile::transpile_typescript_source(SRC, "p982-ts", "main")
            .expect("the TypeScript provider renders");
    assert!(!fileset.files.is_empty(), "the TS file set is not empty");
    assert!(
        fileset.files.iter().any(|f| f.name.ends_with(".ts")),
        "the TS file set carries a `.ts` file: {:?}",
        fileset.files.iter().map(|f| &f.name).collect::<Vec<_>>()
    );
    let again =
        auto_lang_ace_host::script_transpile::transpile_typescript_source(SRC, "p982-ts", "main")
            .expect("render twice");
    assert_eq!(fileset, again, "the TypeScript channel is deterministic");
}
