//! Store 依赖提取（PLAN-815 T-04 自 auto-lang lib.rs 根族迁入；vue 生成
//! 消费的 UI 分析便利面）。旧包经 lib.rs 再导出保持
//! `auto_lang::extract_store_deps_from_file` 路径。

use auto_lang_core::ast;
use auto_lang_core::session::CompilerSession;
use auto_lang_core::Parser;

/// Check if a use statement targets a `store` module.
///
/// Plan 351/370: supports two forms:
/// - legacy:  `use store: Name`        (path exactly "store")
/// - unified: `use notes_store: Name`  (path contains "store", e.g. `*_store`)
///
/// This mirrors the looser detection in auto-man/vue.rs so that widgets built
/// via ui_build_shadcn_with_sub_widgets (the App root path) get store_deps
/// populated the same way as components built via auto-man's generator.
fn is_store_use_stmt(use_stmt: &ast::Use) -> bool {
    let path_match = use_stmt.paths.len() == 1
        && (use_stmt.paths[0].as_str() == "store" || use_stmt.paths[0].as_str().contains("store"));
    let mp_match = use_stmt.module_path.as_ref().map_or(false, |mp| {
        let d = mp.display();
        d == "store" || d.contains("store")
    });
    path_match || mp_match
}

/// Extract store names from `use store: <Name>` declarations.
pub fn extract_store_imports_from_ast(code: &ast::Code) -> Vec<String> {
    let mut stores = Vec::new();
    for stmt in &code.stmts {
        if let ast::Stmt::Use(ref use_stmt) = stmt {
            if is_store_use_stmt(use_stmt) {
                stores.extend(use_stmt.items.iter().map(|s| s.as_str().to_string()));
            }
        }
    }
    stores
}

/// Parse an .at file and extract store dependency names from `use store: <Name>` declarations.
/// Convenience for downstream tools (auto-man) that need store_deps when re-generating
/// individual components.
pub fn extract_store_deps_from_file(path: &str) -> Vec<String> {
    use auto_lang_core::session::CompilerSession;
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let session = CompilerSession::ui().with_backend("vue");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);
    let mut deps = match parser.parse() {
        Ok(ast) => extract_store_imports_from_ast(&ast),
        Err(_) => Vec::new(),
    };
    // PLAN-048 (auto-musk A 线): 跨 store 限定调用 (`ForgeStore.X(...)`) 也构成
    // store 依赖——否则 vue 产物裸发 `ForgeStore` 无 import(TS2304)。按生态命名
    // 约定(XxxStore)扫全文的限定引用;去重后并入。
    // PLAN-675 (T-04 附带修复): 扫描前剥注释——启发式扫的是裸文本，注释里的
    // 示例标识符一并命中（022 app.at 首行 `// … (cf. 018 app.at
    // \`use book_store: BooksStore\`).` 曾整体进 store_deps → 生成幽灵
    // `useBooksStore` import → vite ENOENT 全构建断链；022 standalone 与
    // 画廊 routable 臂同病）。字符串面内的 `//` 不当注释剥。
    let scan_text = strip_line_block_comments(&code);
    let mut hits: Vec<String> = Vec::new();
    let mut cur = String::new();
    for ch in scan_text.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else {
            // 首字母大写且以 Store 结尾的标识符(生态命名约定:XxxStore)。
            let starts_uc = cur
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false);
            if starts_uc && cur.len() > 5 && cur.ends_with("Store") && !deps.contains(&cur) {
                hits.push(cur.clone());
            }
            cur.clear();
        }
    }
    for h in hits {
        if !deps.contains(&h) {
            deps.push(h);
        }
    }
    deps
}

pub use auto_lang_core::strip_line_block_comments;

#[cfg(test)]
mod store_deps_scan_tests {
    /// PLAN-675: 注释里的示例 store 名不进依赖（022 幽灵 useBooksStore 回归），
    /// 真实 use 声明与代码内限定调用照常命中。
    #[test]
    fn extract_store_deps_ignores_comment_examples() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.at");
        std::fs::write(
            &path,
            r#"// store import at the top (cf. 018 app.at `use book_store: BooksStore`).
use board_store: BoardStore

widget App {
    view {
        // AnotherStore.X() mentioned in a comment must not count.
        button "go" .OnGo -> { store.Init() }
    }
}
"#,
        )
        .unwrap();
        let deps = super::extract_store_deps_from_file(path.to_str().unwrap());
        assert!(
            deps.iter().any(|d| d == "BoardStore"),
            "real use-decl must hit: {deps:?}"
        );
        assert!(
            !deps
                .iter()
                .any(|d| d == "BooksStore" || d == "AnotherStore"),
            "comment examples must not leak into deps: {deps:?}"
        );
    }
}
