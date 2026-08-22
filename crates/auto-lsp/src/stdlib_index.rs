//! Plan 416 5-C: stdlib completion index.
//!
//! Parses `stdlib/auto/<mod>.at` declarations once (lazily, via
//! [`auto_lang::util::find_stdlib`]) and serves:
//!   * module-name completions (`json`, `fs`, …) — [`stdlib_module_completions`]
//!   * per-module member completions (`json.parse`, …) —
//!     [`stdlib_member_completions`]
//!
//! Only plain `<mod>.at` files are indexed (double-extension variants like
//! `builder.c.at` / `env.vm.at` are backend-specific copies of the same
//! surface; the plain `.at` is the declaration source of truth).

use std::collections::HashMap;
use std::sync::OnceLock;
use tower_lsp_server::ls_types::*;

/// A indexed stdlib declaration: fn or type.
#[derive(Debug, Clone)]
struct StdlibEntry {
    label: String,
    detail: String,
    kind: CompletionItemKind,
}

/// Module name → indexed declarations.
#[derive(Debug, Default)]
struct StdlibIndex {
    modules: HashMap<String, Vec<StdlibEntry>>,
}

static INDEX: OnceLock<StdlibIndex> = OnceLock::new();

fn build_index() -> StdlibIndex {
    let mut index = StdlibIndex::default();
    let std_dir = match auto_lang::util::find_std_lib() {
        Ok(d) => std::path::PathBuf::from(d.as_str()),
        Err(_) => return index, // stdlib not installed — completions degrade to empty
    };
    let entries = match std::fs::read_dir(&std_dir) {
        Ok(e) => e,
        Err(_) => return index,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        // Plain `<mod>.at` only — skip `mod.c.at`, `mod.vm.at`, dirs, non-.at.
        if !file_name.ends_with(".at") {
            continue;
        }
        let module = file_name.trim_end_matches(".at").to_string();
        if module.is_empty() || module.contains('.') {
            continue;
        }
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut parser = auto_lang::parser::Parser::new(&source);
        let code = match parser.parse() {
            Ok(c) => c,
            Err(_) => continue, // unreadable module — skip silently
        };
        let mut decls = Vec::new();
        for stmt in code.stmts {
            match stmt {
                auto_lang::ast::Stmt::Fn(f) => {
                    let params: Vec<String> = f
                        .params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, p.ty.unique_name()))
                        .collect();
                    decls.push(StdlibEntry {
                        label: f.name.to_string(),
                        detail: format!(
                            "fn {}({}) {}",
                            f.name,
                            params.join(", "),
                            f.ret.unique_name()
                        ),
                        kind: CompletionItemKind::FUNCTION,
                    });
                }
                auto_lang::ast::Stmt::TypeDecl(t) => {
                    decls.push(StdlibEntry {
                        label: t.name.to_string(),
                        detail: format!("type {}", t.name),
                        kind: CompletionItemKind::STRUCT,
                    });
                }
                _ => {}
            }
        }
        if !decls.is_empty() {
            index.modules.insert(module, decls);
        }
    }
    index
}

fn index() -> &'static StdlibIndex {
    INDEX.get_or_init(build_index)
}

fn to_item(e: &StdlibEntry) -> CompletionItem {
    CompletionItem {
        label: e.label.clone(),
        kind: Some(e.kind),
        detail: Some(e.detail.clone()),
        ..Default::default()
    }
}

/// Completion items for the stdlib module names themselves.
pub fn stdlib_module_completions() -> Vec<CompletionItem> {
    let mut mods: Vec<&String> = index().modules.keys().collect();
    mods.sort();
    mods.into_iter()
        .map(|m| CompletionItem {
            label: m.clone(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some(format!("stdlib module auto.{}", m)),
            ..Default::default()
        })
        .collect()
}

/// Completion items for a stdlib module's members (fns + types).
pub fn stdlib_member_completions(module: &str) -> Vec<CompletionItem> {
    match index().modules.get(module) {
        Some(entries) => entries.iter().map(to_item).collect(),
        None => Vec::new(),
    }
}

/// True when `name` is a known stdlib module (member completions apply).
pub fn is_stdlib_module(name: &str) -> bool {
    index().modules.contains_key(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan 416 5-C: on a dev checkout the stdlib dir resolves and the index
    /// finds at least the long-standing modules with callable fns. Degrades
    /// (empty) when the stdlib directory is absent — the count assertions are
    /// therefore gated on the directory having been found at all.
    #[test]
    fn test_stdlib_index_modules_and_members() {
        if auto_lang::util::find_std_lib().is_err() {
            eprintln!("stdlib not found — skipping (degraded environment)");
            return;
        }
        let mods = stdlib_module_completions();
        assert!(!mods.is_empty(), "stdlib modules indexed");
        for expect in ["json", "fs", "time"] {
            assert!(
                mods.iter().any(|m| m.label == expect),
                "module {} indexed (got {:?})",
                expect,
                mods.iter().map(|m| m.label.clone()).collect::<Vec<_>>()
            );
        }
        assert!(is_stdlib_module("json"));
        assert!(!is_stdlib_module("definitely_not_a_module_xyz"));

        // Every indexed module advertises at least one member completion.
        for m in mods {
            let members = stdlib_member_completions(&m.label);
            assert!(!members.is_empty(), "module {} has members", m.label);
        }

        // json is a stable module with parse/decode-family functions.
        let json = stdlib_member_completions("json");
        let labels: Vec<&str> = json.iter().map(|i| i.label.as_str()).collect();
        assert!(
            labels
                .iter()
                .any(|l| l.contains("decode") || l.contains("parse")),
            "json members include decode/parse family: {:?}",
            labels
        );
    }
}

#[cfg(test)]
mod complete_integration_tests {
    use super::*;
    use tower_lsp_server::ls_types::Position;

    /// Plan 416 5-C: end-to-end through the public completion entry — a `.`
    /// trigger after a stdlib module name yields that module's fns; the
    /// keyword baseline contains the authoritative lexer keywords.
    #[test]
    fn test_complete_stdlib_member_and_keywords() {
        if auto_lang::util::find_std_lib().is_err() {
            eprintln!("stdlib not found — skipping (degraded environment)");
            return;
        }
        // `json.` member completion (dot already typed on the line).
        let items = crate::completion::complete(
            "fn main() {\n    var v = json.\n}\n",
            Position {
                line: 1,
                character: 19,
            },
            "test://x.at",
            Some('.'),
        );
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(
            labels
                .iter()
                .any(|l| l.contains("decode") || l.contains("parse")),
            "json member completions surfaced: {:?}",
            &labels[..labels.len().min(10)]
        );

        // Keyword baseline includes authoritative lexer keywords (e.g. `yield`).
        let kws = crate::completion::complete(
            "fn main() {\n    \n}\n",
            Position {
                line: 1,
                character: 4,
            },
            "test://x.at",
            None,
        );
        let labels: Vec<&str> = kws.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"yield"), "lexer keyword `yield` offered");
        assert!(labels.contains(&"fn"), "curated keyword `fn` kept");
    }
}
