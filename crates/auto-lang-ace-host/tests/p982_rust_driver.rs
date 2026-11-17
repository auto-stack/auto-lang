//! PLAN-982（982/RUST I，第二步）：**宿主驱动面**的永久用例。
//!
//! 产品当前仍走旧 `transpile_rust*`；本桥提供"源码 → 受检 HIR → view →
//! lower → FileSet"的唯一边界。这里钉住四件事：
//! 1. 真实源码产出**冻结三件套**（wrapper/runtime/generated）与 schema manifest；
//! 2. 同输入两次调用**逐字节一致**（下游逐文件 diff 的前提）；
//! 3. 失败**具名**（前端 `ace.rust/compile`、入口 `ace.rust/lower`），不返回空集；
//! 4. 入口名是降级后的 Rust 名（`fn_main`），调用方不需要自己猜。

use auto_lang_ace_host::rust_transpile::transpile_scalar_source;

const SRC: &str = r#"
fn add(a int, b int) int { return a + b }

fn main() int {
    var total = add(2, 3)
    return total
}
"#;

#[test]
fn p982_rust_host_driver_renders_the_frozen_triple() {
    let fileset = transpile_scalar_source(SRC, "p982-rust", "main").expect("rendered");
    assert_eq!(
        fileset
            .files
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        vec!["main.rs", "a2x_runtime.rs", "generated.rs"]
    );
    assert!(fileset.files.iter().all(|f| !f.text.trim().is_empty()));
    assert_eq!(fileset.entry, "fn_main");
    assert!(
        fileset.manifest.contains("auto.a2x.rust.fileset/1"),
        "manifest schema: {}",
        fileset.manifest
    );
    assert!(
        fileset.files[2].text.contains("fn_main"),
        "generated.rs carries the lowered entry"
    );
}

#[test]
fn p982_rust_host_driver_is_deterministic() {
    let a = transpile_scalar_source(SRC, "p982-rust", "main").expect("first");
    let b = transpile_scalar_source(SRC, "p982-rust", "main").expect("second");
    assert_eq!(a, b, "two renders must be byte-identical");
}

#[test]
fn p982_rust_host_driver_names_its_refusals() {
    // 前端拒绝：具名且不返回空集。
    let err = transpile_scalar_source("fn main() int { this is not auto }", "p982", "main")
        .expect_err("frontend refuses");
    assert_eq!(err.code, "ace.rust/compile", "{err}");
    assert!(!err.message.is_empty());

    // 入口拒绝：具名 `ace.rust/lower`，绝不静默挑第一个函数。
    let err = transpile_scalar_source(SRC, "p982-rust", "no_such_fn").expect_err("entry");
    assert_eq!(err.code, "ace.rust/lower", "{err}");
}
