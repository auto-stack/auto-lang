//! PLAN-815 T-04 R1-F03 兼容断言：store-deps 族迁 auto-ui-compiler 后的
//! 三个 auto-lang 根级公开路径必须保持可从 crate 外部导入（编译即
//! oracle；行为验证在 auto-ui-compiler store_deps 回归）。

#[test]
fn historic_root_fns_resolve() {
    let _ = auto_lang::extract_store_deps_from_file;
    let _ = auto_lang::extract_store_imports_from_ast;
    let _ = auto_lang::strip_line_block_comments;
}
