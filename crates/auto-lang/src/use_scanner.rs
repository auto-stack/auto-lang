//! Use 语句扫描器
//!
//! 用于预处理阶段快速扫描源码中的 `use` 语句，无需完整解析。
//!
//! # Example
//!
//! ```
//! use auto_lang::use_scanner::{scan_use_statements, UseStatement};
//!
//! let source = r#"
//! use std.io
//! use std.fs: read, write
//! use std.math.*
//! "#;
//!
//! let uses = scan_use_statements(source);
//! assert_eq!(uses.len(), 3);
//! ```

use std::collections::HashSet;

/// Use 语句信息
#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement {
    /// 模块路径，如 "std.io"
    pub module: String,
    /// 导入的项，如 ["read", "write"]
    pub items: Vec<String>,
    /// 是否通配符导入 (use std.io.*)
    pub is_wildcard: bool,
    /// 别名 (use std.io as io)
    pub alias: Option<String>,
    /// 是否是 C 导入 (use c <stdio.h>)
    pub is_c_import: bool,
    /// C 头文件路径 (仅当 is_c_import 为 true)
    pub c_header: Option<String>,
    /// Plan 092: 是否是 Rust 导入 (use.rust serde::json)
    pub is_rust_import: bool,
    /// Plan 214: 是否是 Python 导入 (use.py json5::{dumps, loads})
    pub is_python_import: bool,
    /// Plan 167: 是否是 pub use
    pub is_pub: bool,
}

impl UseStatement {
    /// 创建新的 use 语句
    pub fn new(module: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            items: Vec::new(),
            is_wildcard: false,
            alias: None,
            is_c_import: false,
            c_header: None,
            is_rust_import: false,
            is_python_import: false,
            is_pub: false,
        }
    }

    /// 创建通配符导入
    pub fn wildcard(module: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            items: Vec::new(),
            is_wildcard: true,
            alias: None,
            is_c_import: false,
            c_header: None,
            is_rust_import: false,
            is_python_import: false,
            is_pub: false,
        }
    }

    /// 创建带项的导入
    pub fn with_items(module: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            module: module.into(),
            items,
            is_wildcard: false,
            alias: None,
            is_c_import: false,
            c_header: None,
            is_rust_import: false,
            is_python_import: false,
            is_pub: false,
        }
    }

    /// 创建 C 头文件导入
    pub fn c_import(header: impl Into<String>) -> Self {
        Self {
            module: String::new(),
            items: Vec::new(),
            is_wildcard: false,
            alias: None,
            is_c_import: true,
            c_header: Some(header.into()),
            is_rust_import: false,
            is_python_import: false,
            is_pub: false,
        }
    }

    /// Plan 092: 创建 Rust crate 导入
    pub fn rust_import(module: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            module: module.into(),
            items,
            is_wildcard: false,
            alias: None,
            is_c_import: false,
            c_header: None,
            is_rust_import: true,
            is_python_import: false,
            is_pub: false,
        }
    }

    /// Plan 214: 创建 Python module 导入
    pub fn python_import(module: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            module: module.into(),
            items,
            is_wildcard: false,
            alias: None,
            is_c_import: false,
            c_header: None,
            is_rust_import: false,
            is_python_import: true,
            is_pub: false,
        }
    }
}

/// 扫描源码中的所有 use 语句
///
/// 使用简单的字符串匹配，不进行完整解析。
/// 适用于预处理阶段快速获取依赖信息。
///
/// # 支持的语法
///
/// - `use std.io` - 导入整个模块
/// - `use std.io: say, read` - 导入特定项
/// - `use std.io.*` - 通配符导入
/// - `use std.io as io` - 别名导入
/// - `use c <stdio.h>` - C 头文件导入
///
/// # Example
///
/// ```
/// use auto_lang::use_scanner::scan_use_statements;
///
/// let source = "use std.io\nuse std.fs: read";
/// let uses = scan_use_statements(source);
/// assert_eq!(uses.len(), 2);
/// ```
pub fn scan_use_statements(source: &str) -> Vec<UseStatement> {
    let mut statements = Vec::new();
    let mut seen_modules = HashSet::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // 跳过注释和空行
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#") {
            continue;
        }

        // 检查是否是 use 语句
        if let Some(rest) = trimmed.strip_prefix("use ") {
            if let Some(stmt) = parse_use_line(rest) {
                // 去重：相同模块只记录一次
                if seen_modules.insert(stmt.module.clone()) {
                    statements.push(stmt);
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("use.") {
            // Plan 092/214: Handle use.rust / use.py without space
            if let Some(stmt) = parse_use_line(&format!(".{}", rest)) {
                if seen_modules.insert(stmt.module.clone()) {
                    statements.push(stmt);
                }
            }
        }
    }

    statements
}

/// 解析单行 use 语句
fn parse_use_line(line: &str) -> Option<UseStatement> {
    let line = line.trim();

    // 移除行尾注释
    let line = if let Some(pos) = line.find("//") {
        &line[..pos]
    } else {
        line
    };
    let line = line.trim();

    if line.is_empty() {
        return None;
    }

    // Plan 092: Rust crate 导入: use.rust serde::json::{from_str, to_string}
    if line.starts_with(".rust ") || line.starts_with(".rust\t") {
        let rest = line[5..].trim();  // Skip ".rust "
        return parse_rust_import(rest);
    }

    // Plan 214: Python module 导入: use.py json5::{dumps, loads}
    if line.starts_with(".py ") || line.starts_with(".py\t") {
        let rest = line[3..].trim();  // Skip ".py "
        return parse_python_import(rest);
    }

    // C 头文件导入: use c <stdio.h>
    if line.starts_with("c ") || line.starts_with("c<") {
        let header_part = if line.starts_with("c ") {
            line[2..].trim()
        } else {
            line[1..].trim()
        };

        // 提取 <...> 中的内容
        if header_part.starts_with('<') && header_part.contains('>') {
            let end = header_part.find('>')?;
            let header = &header_part[1..end];
            return Some(UseStatement::c_import(header));
        }
        return None;
    }

    // Plan 167: pub use — check for "pub " prefix
    let (line, is_pub) = if line.starts_with("pub ") {
        (&line[4..], true)
    } else {
        (line, false)
    };

    // 检查是否有别名: use std.io as io
    let (module_part, alias) = if let Some(as_pos) = line.find(" as ") {
        let module = line[..as_pos].trim();
        let alias_name = line[as_pos + 4..].trim();
        (module, Some(alias_name.to_string()))
    } else {
        (line, None)
    };

    // 检查是否有项导入: use std.io: say, read
    if let Some(colon_pos) = module_part.find(':') {
        let module = module_part[..colon_pos].trim();
        let items_str = &module_part[colon_pos + 1..];

        // 解析项列表
        let items: Vec<String> = items_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 检查是否是通配符
        if items.len() == 1 && items[0] == "*" {
            return Some(UseStatement {
                module: module.to_string(),
                items: Vec::new(),
                is_wildcard: true,
                alias,
                is_c_import: false,
                c_header: None,
                is_rust_import: false,
                is_python_import: false,
                is_pub,
            });
        }

        return Some(UseStatement {
            module: module.to_string(),
            items,
            is_wildcard: false,
            alias,
            is_c_import: false,
            c_header: None,
            is_rust_import: false,
            is_python_import: false,
            is_pub,
        });
    }

    // 简单导入: use std.io
    // 检查是否以 .* 结尾（通配符）
    if module_part.ends_with(".*") {
        let module = &module_part[..module_part.len() - 2];
        return Some(UseStatement {
            module: module.to_string(),
            items: Vec::new(),
            is_wildcard: true,
            alias,
            is_c_import: false,
            c_header: None,
            is_rust_import: false,
            is_python_import: false,
            is_pub,
        });
    }

    Some(UseStatement {
        module: module_part.to_string(),
        items: Vec::new(),
        is_wildcard: false,
        alias,
        is_c_import: false,
        c_header: None,
        is_rust_import: false,
        is_python_import: false,
        is_pub,
    })
}

/// Plan 092: Parse Rust crate import
///
/// Parses: `serde::json::{from_str, to_string}` or `serde::json`
fn parse_rust_import(line: &str) -> Option<UseStatement> {
    let line = line.trim();

    if line.is_empty() {
        return None;
    }

    // Find items in braces: {item1, item2}
    let (module_part, items) = if let Some(brace_start) = line.find('{') {
        let module = line[..brace_start].trim();
        let rest = &line[brace_start + 1..];

        let items_str = if let Some(brace_end) = rest.find('}') {
            &rest[..brace_end]
        } else {
            rest
        };

        // Remove trailing :: before items
        let module = module.trim_end_matches(':').trim();

        let items: Vec<String> = items_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        (module, items)
    } else {
        // No braces - just module path
        (line, Vec::new())
    };

    // Remove trailing :: from module
    let module = module_part.trim_end_matches(':').trim();

    Some(UseStatement::rust_import(module, items))
}

/// Plan 214/300.1: Parse Python module import
///
/// Parses: `a.b.c: x, y` (from a.b.c import x, y) or `a.b.c` (import a.b.c)
fn parse_python_import(line: &str) -> Option<UseStatement> {
    let line = line.trim();

    if line.is_empty() {
        return None;
    }

    // Plan 300.1: colon-separated items (Pythonic style)
    // use.py a.b.c: x, y  → module="a.b.c", items=["x", "y"]
    // use.py a.b.c         → module="a.b.c", items=[]
    let (module_part, items) = if let Some(colon_pos) = line.find(':') {
        let module = line[..colon_pos].trim();
        let items_str = &line[colon_pos + 1..];
        let items: Vec<String> = items_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        (module, items)
    } else {
        (line, Vec::new())
    };

    Some(UseStatement::python_import(module_part, items))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_use() {
        let source = "use std.io";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "std.io");
        assert!(!uses[0].is_wildcard);
        assert!(uses[0].items.is_empty());
    }

    #[test]
    fn test_use_with_items() {
        let source = "use std.io: say, read";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "std.io");
        assert_eq!(uses[0].items, vec!["say", "read"]);
        assert!(!uses[0].is_wildcard);
    }

    #[test]
    fn test_use_wildcard() {
        let source = "use std.io.*";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "std.io");
        assert!(uses[0].is_wildcard);
    }

    #[test]
    fn test_use_wildcard_colon() {
        let source = "use std.io: *";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "std.io");
        assert!(uses[0].is_wildcard);
    }

    #[test]
    fn test_use_alias() {
        let source = "use std.io as io";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "std.io");
        assert_eq!(uses[0].alias, Some("io".to_string()));
    }

    #[test]
    fn test_c_import() {
        let source = "use c <stdio.h>";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert!(uses[0].is_c_import);
        assert_eq!(uses[0].c_header, Some("stdio.h".to_string()));
    }

    #[test]
    fn test_multiple_uses() {
        let source = r#"
use std.io
use std.fs: read, write
use std.math.*
"#;
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 3);
        assert_eq!(uses[0].module, "std.io");
        assert_eq!(uses[1].module, "std.fs");
        assert_eq!(uses[2].module, "std.math");
        assert!(uses[2].is_wildcard);
    }

    #[test]
    fn test_deduplication() {
        let source = r#"
use std.io
use std.io
use std.io
"#;
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
    }

    #[test]
    fn test_skip_comments() {
        let source = r#"
// This is a comment
use std.io
// Another comment
#use fake.io
"#;
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "std.io");
    }

    #[test]
    fn test_inline_comment() {
        let source = "use std.io  // import io module";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "std.io");
    }

    // Plan 092: Rust import tests

    #[test]
    fn test_rust_import_simple() {
        let source = "use.rust serde::json";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert!(uses[0].is_rust_import);
        assert_eq!(uses[0].module, "serde::json");
        assert!(uses[0].items.is_empty());
    }

    #[test]
    fn test_rust_import_with_items() {
        let source = "use.rust serde::json::{from_str, to_string}";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert!(uses[0].is_rust_import);
        assert_eq!(uses[0].module, "serde::json");
        assert_eq!(uses[0].items, vec!["from_str", "to_string"]);
    }

    #[test]
    fn test_rust_import_deep_path() {
        let source = "use.rust tokio::net::TcpStream";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert!(uses[0].is_rust_import);
        assert_eq!(uses[0].module, "tokio::net::TcpStream");
    }

    #[test]
    fn test_mixed_imports() {
        let source = r#"
use std.io
use.rust serde::json::{from_str}
use c <stdio.h>
"#;
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 3);

        // First is Auto import
        assert!(!uses[0].is_rust_import);
        assert!(!uses[0].is_c_import);

        // Second is Rust import
        assert!(uses[1].is_rust_import);
        assert!(!uses[1].is_c_import);

        // Third is C import
        assert!(!uses[2].is_rust_import);
        assert!(uses[2].is_c_import);
    }

    // Plan 131: Module path prefix tests (pac. and super.)

    #[test]
    fn test_scan_pac_import() {
        let source = "use pac.db";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "pac.db");
    }

    #[test]
    fn test_scan_super_import() {
        let source = "use super.utils";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "super.utils");
    }

    #[test]
    fn test_scan_pac_deep_path() {
        let source = "use pac.api.handlers.user";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "pac.api.handlers.user");
    }

    #[test]
    fn test_scan_super_with_items() {
        let source = "use super.utils: load, save";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].module, "super.utils");
        assert_eq!(uses[0].items, vec!["load", "save"]);
    }

    #[test]
    fn test_scan_plan131_mixed_imports() {
        let source = r#"
use pac.db
use super.utils
use local_mod
"#;
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 3);
        assert_eq!(uses[0].module, "pac.db");
        assert_eq!(uses[1].module, "super.utils");
        assert_eq!(uses[2].module, "local_mod");
    }

    // Plan 214: Python import tests

    #[test]
    fn test_py_import_simple() {
        let source = "use.py json";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert!(uses[0].is_python_import);
        assert_eq!(uses[0].module, "json");
        assert!(uses[0].items.is_empty());
    }

    #[test]
    fn test_py_import_with_items() {
        let source = "use.py json5: dumps, loads";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert!(uses[0].is_python_import);
        assert_eq!(uses[0].module, "json5");
        assert_eq!(uses[0].items, vec!["dumps", "loads"]);
    }

    #[test]
    fn test_py_import_deep_path() {
        let source = "use.py os.path";
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 1);
        assert!(uses[0].is_python_import);
        assert_eq!(uses[0].module, "os.path");
    }

    #[test]
    fn test_mixed_all_import_types() {
        let source = r#"
use std.io
use.rust serde::json::{from_str}
use.py json: dumps, loads
use c <stdio.h>
"#;
        let uses = scan_use_statements(source);
        assert_eq!(uses.len(), 4);

        assert!(!uses[0].is_rust_import);
        assert!(!uses[0].is_python_import);
        assert!(!uses[0].is_c_import);

        assert!(uses[1].is_rust_import);
        assert!(!uses[1].is_python_import);

        assert!(uses[2].is_python_import);
        assert!(!uses[2].is_rust_import);

        assert!(uses[3].is_c_import);
        assert!(!uses[3].is_python_import);
    }
}
