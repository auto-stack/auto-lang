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
    if let Ok(root) = std::env::var("AUTO_STDLIB_ROOT") {
        let root = PathBuf::from(root);
        return root
            .canonicalize()
            .and_then(|p| {
                if p.is_dir() {
                    Ok(p)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotADirectory,
                        "not a directory",
                    ))
                }
            })
            .map_err(|e| crate::error::AutoError::Msg(format!("invalid AUTO_STDLIB_ROOT: {e}")));
    }
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
    let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/auto");
    if bundled.is_dir() {
        return Ok(bundled.canonicalize().unwrap_or(bundled));
    }
    let installed = dirs::home_dir()
        .into_iter()
        .map(|home| home.join(".auto/libs/stdlib/auto"))
        .chain([
            PathBuf::from("/usr/local/lib/auto/stdlib/auto"),
            PathBuf::from("/usr/lib/auto/stdlib/auto"),
        ]);
    for root in installed {
        if root.is_dir() {
            return Ok(root.canonicalize()?);
        }
    }
    Err(crate::error::AutoError::Msg(
        "stdlib/auto not found (CARGO_MANIFEST_DIR or CWD)".to_string(),
    ))
}

/// PLAN-738 T-06（AC-07，SD-01/05）：stdlib 装配内容指纹——生成 API 收据
/// （734 generation.json）与复用新鲜度门（736）消费的 stdlib 来源身份。
///
/// 链式 FNV-1a 64 吸收：inventory schema 版本 + provider 目录 schema 版本
/// + 装配目标 + 全部层（模块名, 层内容指纹）按扫描序。内容级身份——改任一
/// 层或目录声明（bump schema）即变；parse 失败层同样按内容入链（分母不删）。
/// 目标入链：target 变化即身份变化（§5.4：target/features 改变指纹；
/// profile/运行配置不在此——那是 736 的 config_hash）。
pub fn stdlib_assembly_fingerprint(
    root: &Path,
    target: super::model::AssemblyTarget,
) -> AutoResult<u64> {
    if !root.is_dir() {
        return Err(crate::error::AutoError::Msg(format!(
            "stdlib root not found: {}",
            root.display()
        )));
    }
    let inv = scan_inventory(root);
    if inv
        .diagnostics
        .iter()
        .any(|d| d.code == super::validate::code::READ_FAIL)
    {
        return Err(crate::error::AutoError::Msg(
            "stdlib inventory incomplete; refusing assembly fingerprint".into(),
        ));
    }
    let mut acc: u64 = 0xcbf29ce484222325;
    let mut mix = |v: u64| {
        acc ^= v;
        acc = acc.wrapping_mul(0x100000001b3);
    };
    mix(INVENTORY_SCHEMA_VERSION as u64);
    mix(super::providers::catalog_schema_version() as u64);
    mix(fnv1a64(include_str!(
        "../../../../stdlib/assembly-providers.json"
    )));
    if let Some(catalog) = root
        .parent()
        .map(|p| p.join("assembly-providers.json"))
        .filter(|p| p.exists())
    {
        mix(fnv1a64(&std::fs::read_to_string(catalog)?));
    }
    mix(fnv1a64(&format!(
        "ui-iced={};streaming-http={};python={}",
        cfg!(feature = "ui-iced"),
        cfg!(feature = "streaming-http"),
        cfg!(feature = "python")
    )));
    // Host implementations are build inputs, independently of the .at API.
    mix(fnv1a64(include_str!("../a2r_std.rs")));
    mix(fnv1a64(include_str!("../vm/ffi/stdlib.rs")));
    mix(fnv1a64(include_str!("../vm/native_catalog.rs")));
    mix(match target {
        super::model::AssemblyTarget::Vm => 1,
        super::model::AssemblyTarget::Rust => 2,
        super::model::AssemblyTarget::C => 3,
    });
    for m in &inv.modules {
        for l in &m.layers {
            mix(fnv1a64(&m.module));
            mix(fnv1a64(&l.file));
            mix(l.kind as u64);
            mix(l.content_hash);
        }
    }
    Ok(acc)
}

/// 扫描并清点 stdlib 全部 `.at` 层。
pub fn scan_inventory(root: &Path) -> StdlibInventory {
    let mut at_files: Vec<PathBuf> = Vec::new();
    let mut diagnostics: Vec<AssemblyDiagnostic> = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        match entry {
            Ok(e)
                if e.file_type().is_file()
                    && e.path().extension().and_then(|x| x.to_str()) == Some("at") =>
            {
                at_files.push(e.into_path())
            }
            Ok(_) => {}
            Err(e) => diagnostics.push(AssemblyDiagnostic {
                code: super::validate::code::READ_FAIL.into(),
                module: String::new(),
                file: e
                    .path()
                    .and_then(|p| p.strip_prefix(root).ok())
                    .map(|p| portable_file(&p.to_string_lossy().replace('\\', "/"))),
                message: format!(
                    "inventory traversal incomplete: {}",
                    e.io_error()
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "directory cycle".into())
                ),
            }),
        }
    }
    at_files.sort();

    let files_total = at_files.len();
    let mut modules: Vec<ModuleInventory> = Vec::new();

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
                let bytes = std::fs::read(path).unwrap_or_default();
                push_layer(
                    &mut modules,
                    module_path,
                    LayerInventory {
                        kind,
                        file: portable_file(&rel),
                        content_hash: bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
                            (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
                        }),
                        parse: ParseStatus::Failed {
                            error: format!("read failed: {e}"),
                        },
                        symbols: Vec::new(),
                    },
                );
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
    format!(
        "{}|{}|{}",
        d.code,
        d.module,
        d.file.as_deref().unwrap_or("")
    )
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
pub fn parse_layer(content: &str, kind: LayerKind, file: String) -> LayerInventory {
    let shared: std::sync::Arc<std::sync::RwLock<crate::types::TypeStore>> =
        std::sync::Arc::new(std::sync::RwLock::new(crate::types::TypeStore::new()));
    parse_layer_with_store(content, kind, file, shared)
}

pub fn parse_layer_with_store(
    content: &str,
    kind: LayerKind,
    file: String,
    shared: std::sync::Arc<std::sync::RwLock<crate::types::TypeStore>>,
) -> LayerInventory {
    let content_hash = fnv1a64(content);
    let mut parser = crate::parser::Parser::new_with_type_store(content, shared);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            return LayerInventory {
                kind,
                file,
                content_hash,
                parse: ParseStatus::Failed {
                    error: format!("{e:?}"),
                },
                symbols: Vec::new(),
            };
        }
    };

    let mut symbols: Vec<SymbolEntry> = Vec::new();
    for stmt in &ast.stmts {
        match stmt {
            crate::ast::Stmt::Fn(f) => push_fn(&mut symbols, f, None, content),
            crate::ast::Stmt::TypeDecl(t) => {
                symbols.push(SymbolEntry {
                    name: t.name.to_string(),
                    kind: SymbolKind::Type,
                    is_pub: t.is_pub,
                    is_vm_decl: false,
                    arity: 0,
                    signature: None,
                    has_body: false,
                    source_span: None,
                    verification: VerificationLevel::Declared,
                });
                for member in &t.members {
                    symbols.push(SymbolEntry {
                        name: format!("{}.{}", t.name, member.name),
                        kind: SymbolKind::Field,
                        is_pub: t.is_pub,
                        is_vm_decl: false,
                        arity: 0,
                        signature: Some(super::model::LogicalSignature {
                            parameters: Vec::new(),
                            parameter_modes: Vec::new(),
                            returns: member.ty.unique_name().to_string(),
                            is_static: false,
                            has_self: false,
                            generics: Vec::new(),
                            attributes: member.attrs.iter().map(ToString::to_string).collect(),
                        }),
                        has_body: member.value.is_some(),
                        source_span: None,
                        verification: VerificationLevel::Declared,
                    });
                }
                for m in &t.methods {
                    push_fn(&mut symbols, m, Some(t.name.as_str()), content);
                }
            }
            crate::ast::Stmt::Ext(e) => {
                for m in &e.methods {
                    push_fn(&mut symbols, m, Some(e.target.as_str()), content);
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

fn push_fn(symbols: &mut Vec<SymbolEntry>, f: &crate::ast::Fn, owner: Option<&str>, content: &str) {
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
        signature: Some(super::model::LogicalSignature {
            parameters: f
                .params
                .iter()
                .map(|p| p.ty.unique_name().to_string())
                .collect(),
            parameter_modes: f.params.iter().map(|p| format!("{:?}", p.mode)).collect(),
            returns: if matches!(f.ret, crate::ast::Type::Unknown) {
                f.ret_name
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| f.ret.unique_name().to_string())
            } else {
                f.ret.unique_name().to_string()
            },
            is_static: f.is_static,
            has_self: (owner.is_some() || f.parent.is_some()) && !f.is_static,
            generics: f.type_params.iter().map(|p| p.name.to_string()).collect(),
            attributes: f.attrs.iter().map(ToString::to_string).collect(),
        }),
        has_body: !f.body.stmts.is_empty() || source_has_body(content, f.span),
        source_span: f.span,
        verification: VerificationLevel::Declared,
    });
}

pub(crate) fn source_has_body(content: &str, span: Option<(usize, usize)>) -> bool {
    let Some((offset, length)) = span else {
        return false;
    };
    let Some(source) = content.get(offset..offset.saturating_add(length)) else {
        return false;
    };
    let mut lexer = crate::lexer::Lexer::new(source);
    let mut parentheses = 0usize;
    while let Ok(token) = lexer.next() {
        use crate::token::TokenKind;
        match token.kind {
            TokenKind::LParen => parentheses += 1,
            TokenKind::RParen => parentheses = parentheses.saturating_sub(1),
            TokenKind::LBrace if parentheses == 0 => return true,
            TokenKind::EOF => break,
            _ => {}
        }
    }
    false
}
