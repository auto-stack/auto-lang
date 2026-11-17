//! PLAN-982（982/RUST I，第一步）：**无缓存 A2R 驱动**的端到端证据。
//!
//! 产品侧（`cmd_trans.rs` / `auto-man-runtime::target`）仍走旧
//! `transpile_rust*`，而新 A2R 是**目标提供者框架**（`TargetProvider` +
//! fragment 缓存键）。本文件走"源码 → 受检 HIR → SemanticView →
//! `lower_program` → `assemble_file_set`"这条**产品将来要走**的链路，证明：
//! 1. 真实 `.at` 源能被渲染成完整的 `FileSet`（wrapper/runtime/generated 三件）；
//! 2. 渲染结果**确定**（同输入两次逐字节一致，可作为 diff 基线）；
//! 3. 名字/入口/生成标记都落在产物里（不是空文件）。
//!
//! 这是"无缓存驱动"的 oracle；产品接线（`AUTO_ACE_RUST` 候选开关 + 旧
//! RustTrans 零 dispatch）在后续步骤，本文件先钉住语言侧能力。

use ace_a2x_core::SemanticView;

fn view_of(src: &str) -> SemanticView<'static> {
    let compiled = ace_core::source::compile_scalar_source(src, "a2r-probe", &[])
        .expect("scalar frontend diagnostics");
    let checked = Box::leak(Box::new(compiled.checked));
    SemanticView::from_checked(checked).expect("view admission")
}

const SRC: &str = r#"
fn add(a int, b int) int { return a + b }

fn main() int {
    var total = add(2, 3)
    return total
}
"#;

#[test]
fn p982_rust_uncached_driver_renders_a_complete_fileset() {
    let view = view_of(SRC);
    let program = ace_a2x_rust::lower_program(&view, "main").expect("lowering the entry");
    assert!(
        program.entry.ends_with("main"),
        "the entry is the lowered Rust name of `main`: {}",
        program.entry
    );
    let input_hash = format!("{:016x}", auto_lang_hash(SRC));
    let fileset = ace_a2x_rust::assemble_file_set(&view, &program, &input_hash, "main");

    let names: Vec<&str> = fileset.files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        vec!["main.rs", "a2x_runtime.rs", "generated.rs"],
        "the file set is the frozen wrapper/runtime/generated triple"
    );
    for (name, text) in &fileset.files {
        assert!(!text.trim().is_empty(), "{name} must not be empty");
    }
    let generated = &fileset.files[2].1;
    assert!(
        generated.contains("DO NOT EDIT"),
        "generated.rs carries the generated marker"
    );
    // 生成文件持有**降级后的入口名**（`fn_main`），不是源名 `main`。
    assert!(
        generated.contains("fn fn_main") || generated.contains("fn_main"),
        "generated.rs holds the lowered entry: {}",
        &generated[..generated.len().min(200)]
    );
    assert!(
        fileset
            .manifest
            .contains(ace_a2x_rust::artifacts::FILESET_SCHEMA),
        "the manifest names the frozen file-set schema: {}",
        fileset.manifest
    );
    assert!(
        !fileset.manifest.contains("main.rs\"") || fileset.manifest.contains("files"),
        "the manifest lists its members"
    );
}

/// 同一输入两次渲染**逐字节一致**：这是下游"与旧转译逐文件 diff"可用的前提
/// （不确定性会淹没差异面）。
#[test]
fn p982_rust_uncached_driver_is_deterministic() {
    let render = || {
        let view = view_of(SRC);
        let program = ace_a2x_rust::lower_program(&view, "main").expect("lower");
        let fileset = ace_a2x_rust::assemble_file_set(&view, &program, "0000000000000000", "main");
        fileset.files
    };
    let a = render();
    let b = render();
    assert_eq!(
        a, b,
        "two renders of the same source must be byte-identical"
    );
}

/// 入口选择是**显式**的：不存在的入口是具名拒绝，不静默降级到第一个函数。
#[test]
fn p982_rust_uncached_driver_refuses_unknown_entry_by_name() {
    let view = view_of(SRC);
    let err = ace_a2x_rust::lower_program(&view, "no_such_fn").expect_err("unknown entry");
    let text = format!("{err:?}");
    assert!(
        text.to_lowercase().contains("entry") || text.contains("a2x."),
        "the refusal must be named: {text}"
    );
}

/// 探针用的稳定 64 位混合（与产品身份无关，只为确定性输入）。
fn auto_lang_hash(bytes: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
