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
            });
        }

        return Some(UseStatement {
            module: module.to_string(),
            items,
            is_wildcard: false,
            alias,
            is_c_import: false,
            c_header: None,
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
        });
    }

    Some(UseStatement {
        module: module_part.to_string(),
        items: Vec::new(),
        is_wildcard: false,
        alias,
        is_c_import: false,
        c_header: None,
    })
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
}
