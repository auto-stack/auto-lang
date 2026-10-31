//! PLAN-815 T-04 R1-F03 根级路径合同——PLAN-900 修订。
//!
//! store-deps 族（extract_store_deps_from_file/extract_store_imports_from_ast）
//! 的 auto-lang 根级转发随本 crate 去 auto-ui 依赖退役（单源
//! auto_ui_compiler::store_deps；行为回归在 auto-ui-compiler 侧）。
//! strip_line_block_comments 单源在 SDK（auto_lang_core），本路径保持。

#[test]
fn historic_root_fns_resolve() {
    let _ = auto_lang::strip_line_block_comments;
}
