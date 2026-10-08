//! PLAN-738：全库 inventory 装载器。扫描 stdlib 根下全部 `.at` 文件（公共
//! + 目标层），用**实际语言 parser**（`Parser::new_with_type_store`，生产
//! 装载同款构造）提取公开符号；解析失败逐项记 diagnostic，不删分母。
//!
//! 便携源 ID = stdlib 根相对路径（正斜杠）；模块身份 = 剥层后缀的相对路径
//! 点分（`encoding/base64.rs.at` → `encoding.base64`）。仅扫描 `.at`——
//! 生成 C 产物（.c/.h）属构建输出，不进符号分母。

use std::path::{Path, PathBuf};

use crate::error::AutoResult;

use super::model::{
    fnv1a64, AssemblyDiagnostic, LayerInventory, LayerKind, ModuleInventory, ParseStatus,
    StdlibInventory, SymbolEntry, SymbolKind, VerificationLevel, INVENTORY_SCHEMA_VERSION,
};

/// 便携惯例根值（manifest 的 stdlib 来源身份字段）。
pub const PORTABLE_ROOT: &str = "stdlib/auto";

/// 定位仓库 stdlib 根（测试/工具进程确定性：CARGO_MANIFEST_DIR 上两级到
/// 仓根）。**不用 `find_std_lib()`**——实勘其项目根分支拼出
/// `<root>/stdlib/stdlib/auto` 恒不命中，cargo 构建下会静默解析到
/// `~/.auto/libs/stdlib/auto`（见 738-stdlib-decision §1/E3 来源身份裂缝）。
pub fn repo_stdlib_root() -> AutoResult<std::path::PathBuf> {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let root = PathBuf::from(manifest_dir).join("../../stdlib/auto");
        if root.is_dir() {
            return Ok(root.canonicalize().unwrap_or(root));
        }
    }
    let cwd_rel = PathBuf::from("stdlib/auto");
    if cwd_rel.is_dir() {
        return Ok(cwd_rel.canonicalize().unwrap_or(cwd_rel));
    }
    Err(crate::error::AutoError::Msg(
        "stdlib/auto not found (CARGO_MANIFEST_DIR or CWD)".to_string(),
    ))
}

/// 扫描并清点 stdlib 全部 `.at` 层。
pub fn scan_inventory(root: &Path) -> StdlibInventory {
    let mut at_files: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".at"))
                .unwrap_or(false)
        })
        .collect();
    at_files.sort();

    let files_total = at_files.len();
    let mut modules: Vec<ModuleInventory> = Vec::new();
    let mut diagnostics: Vec<AssemblyDiagnostic> = Vec::new();

    for path in &at_files {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let Some((kind, base)) = LayerKind::from_file_name(file_name) else {
            continue;
        };
        // 模块身份：基名所在目录 + 基名（目录模块 mod.at → 目录名）
        let dir_rel = Path::new(&rel)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let module_path = if base == "mod" {
            dir_rel.clone()
        } else if dir_rel.is_empty() {
            base.clone()
        } else {
            format!("{dir_rel}.{base}")
        };

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                diagnostics.push(AssemblyDiagnostic {
                    code: super::validate::code::READ_FAIL.to_string(),
                    module: module_path.clone(),
                    file: Some(portable_file(&rel)),
                    message: format!("read failed: {e}"),
                });
                continue;
            }
        };

        let layer = parse_layer(&content, kind, portable_file(&rel));
        if let super::model::ParseStatus::Failed { error } = &layer.parse {
            diagnostics.push(AssemblyDiagnostic {
                code: super::validate::code::PARSE_FAIL.to_string(),
                module: module_path.clone(),
                file: Some(portable_file(&rel)),
                message: error.clone(),
            });
        }
        push_layer(&mut modules, module_path, layer);
    }

    // 覆盖诊断：有目标层却无公共层的模块（§5.1 缺口记录）
    for d in super::validate::coverage_checks(&modules) {
        diagnostics.push(d);
    }
    // 输出顺序与扫描顺序解耦：模块按身份排序（stable JSON 契约）
    modules.sort_by(|a, b| a.module.cmp(&b.module));
    diagnostics.sort_by(|a, b| diag_sort(a).cmp(&diag_sort(b)));

    let inv = StdlibInventory {
        schema_version: INVENTORY_SCHEMA_VERSION,
        root: PORTABLE_ROOT.to_string(),
        files_total,
        modules,
        diagnostics,
    };
    inv.assert_sorted();
    inv
}

fn diag_sort(d: &AssemblyDiagnostic) -> String {
    format!("{}|{}|{}", d.code, d.module, d.file.as_deref().unwrap_or(""))
}

fn portable_file(rel: &str) -> String {
    format!("{PORTABLE_ROOT}/{rel}")
}

fn push_layer(modules: &mut Vec<ModuleInventory>, module_path: String, layer: LayerInventory) {
    if let Some(m) = modules.iter_mut().find(|m| m.module == module_path) {
        m.layers.push(layer);
        m.layers.sort_by_key(|l| l.kind);
    } else {
        modules.push(ModuleInventory {
            module: module_path,
            layers: vec![layer],
        });
    }
}

/// 解析单层并提取公开符号（生产装载同款构造）。
fn parse_layer(content: &str, kind: LayerKind, file: String) -> LayerInventory {
    let content_hash = fnv1a64(content);
    let shared: std::sync::Arc<std::sync::RwLock<crate::types::TypeStore>> =
        std::sync::Arc::new(std::sync::RwLock::new(crate::types::TypeStore::new()));
    let mut parser = crate::parser::Parser::new_with_type_store(content, shared);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            return LayerInventory {
                kind,
                file,
                content_hash,
                parse: ParseStatus::Failed { error: format!("{e:?}") },
                symbols: Vec::new(),
            };
        }
    };

    let mut symbols: Vec<SymbolEntry> = Vec::new();
    for stmt in &ast.stmts {
        match stmt {
            crate::ast::Stmt::Fn(f) => push_fn(&mut symbols, f, None),
            crate::ast::Stmt::TypeDecl(t) => {
                symbols.push(SymbolEntry {
                    name: t.name.to_string(),
                    kind: SymbolKind::Type,
                    is_pub: true, // stdlib 面即公开面；非 pub 类型不予登记位（T-04 细化）
                    is_vm_decl: false,
                    arity: 0,
                    verification: VerificationLevel::Declared,
                });
                for m in &t.methods {
                    push_fn(&mut symbols, m, Some(t.name.as_str()));
                }
            }
            crate::ast::Stmt::Ext(e) => {
                for m in &e.methods {
                    push_fn(&mut symbols, m, Some(e.target.as_str()));
                }
            }
            _ => {}
        }
    }
    let symbol_count = symbols.len();
    symbols.sort_by(|a, b| a.name.cmp(&b.name));
    LayerInventory {
        kind,
        file,
        content_hash,
        parse: ParseStatus::Parsed { symbol_count },
        symbols,
    }
}

fn push_fn(symbols: &mut Vec<SymbolEntry>, f: &crate::ast::Fn, owner: Option<&str>) {
    // 归一身份：限定方法 `Owner.name`（parser 的 parent 或 ext/type owner）；
    // 顶层裸名 fn 保持裸名（net.at 的 `pub fn TcpListener.accept` 由 parent
    // 归一为 `TcpListener.accept`）。
    let name = match (&f.parent, owner) {
        (Some(p), _) => format!("{p}.{}", f.name),
        (None, Some(o)) => format!("{o}.{}", f.name),
        (None, None) => f.name.to_string(),
    };
    symbols.push(SymbolEntry {
        name,
        kind: if f.parent.is_some() || owner.is_some() {
            SymbolKind::Method
        } else {
            SymbolKind::Fn
        },
        is_pub: f.is_pub,
        is_vm_decl: matches!(f.kind, crate::ast::FnKind::VmFunction),
        arity: f.params.len(),
        verification: VerificationLevel::Declared,
    });
}
