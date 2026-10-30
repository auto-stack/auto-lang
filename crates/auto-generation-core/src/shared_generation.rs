//! Restricted, single-source language generation mechanisms shared by HTTP
//! and non-HTTP consumers. No server runtime, generated HTTP templates, IPC
//! assembly, UI dependency or delegation back to the mixed API facade lives here.
//! PLAN-890 T-02: this module is the bounded future shared-carrier source;
//! physical package relocation follows the full T-02 freeze.
use auto_lang_core::api::{ApiAttrs, ApiEndpoint, ApiField, ApiModule, ApiParam, ApiType};
use auto_val::AutoResult;
use std::path::Path;

/// 首次物化绑定：把实际 lock 身份写回收据的 workspace_lock 字段（保持
/// 其余字段逐字不动——serde_json 值级更新，时间戳等不重写）。
pub fn bind_workspace_lock(record_path: &Path, record: &mut serde_json::Value, lock: &str) -> bool {
    let Some(value) = record.get_mut("workspace_lock") else {
        return false;
    };
    if value.as_str() == Some(lock) {
        return true;
    }
    *value = serde_json::Value::String(lock.to_string());
    std::fs::write(record_path, record.to_string()).is_ok()
}

/// workspace lock 新鲜度（PLAN-738 T-12/R5-01）：双方一致才新鲜；首次
/// 物化（absent→实际值）由调用方在绑定收据后判新鲜；absent/absent=
/// 尚无 lock 物化，新鲜；其余（值漂移/lock 消失/读取失败→None 不可
/// 核验/收据缺字段）一律陈旧。
pub fn lock_freshness(recorded: Option<&str>, current: Option<&str>) -> bool {
    match (recorded, current) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// 装配新鲜度判定（PLAN-738 T-06；纯函数便于单测）。
/// - 双 None：不能核验装配身份，必须重新生成；
/// - 单侧缺失：陈旧——旧产物无该字段而当前可算=门升级后首查（再生一次
///   写入新字段收敛）；收据有而当前不可算=无法核验，保守再生；
/// - 双方在：十六进制指纹串相等才新鲜。
pub fn assembly_freshness(recorded: Option<&str>, current: Option<&str>) -> bool {
    match (recorded, current) {
        (None, None) => false,
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// Try to parse API file with full AST parsing
pub fn try_full_parse(api_content: &str) -> Option<ApiModule> {
    use auto_lang_core::api::ApiExtractor;

    let mut parser = auto_lang_core::Parser::from(api_content);
    let ast = parser.parse().ok()?;

    let extractor = ApiExtractor::new();
    let module = extractor.extract("api", &ast.stmts);

    // Only return if we found endpoints
    if module.endpoints.is_empty() && module.types.is_empty() {
        None
    } else {
        Some(module)
    }
}

/// Generate Rust server code (Axum-based)
/// Plan musk-022 CRUD 智能扩展: transpile db.at to a db.rs module via a2r.
/// Reuses the Tauri-backend precedent (tauri_backend::transpile_at_to_rust).
pub fn transpile_db_to_rs(content: &str) -> AutoResult<String> {
    transpile_back_module_to_rs_with_proofs("db", content, None).map(|(code, _)| code)
}

/// PLAN-681 T-01: transpile a `src/back/*.at` module to Rust (companion
/// modules like fsys.at share db.at's 16MB-stack transpile pipeline; the
/// module name feeds the transpiler's module-id slot and error strings).
pub fn transpile_back_module_to_rs(module_name: &str, content: &str) -> AutoResult<String> {
    transpile_back_module_to_rs_with_proofs(module_name, content, None).map(|(code, _)| code)
}

/// PLAN-738 T-11：back 模块转译携带引用闭包证明。生成服务的 back 产物经
/// `qualify_a2r_std` 链接 `auto_lang::` 内嵌镜像——核心 producer 引用按
/// Embedded 运行形态收集（原实现用 Standalone 默认并丢弃证明）。
/// PLAN-885 G2：`source_dir` 传入 back 目录，使 register_import_signatures
/// 能解析 `use works: …` 导入的函数签名/字段类型（此前只查 CWD，导入
/// 函数 fn_ret_types 缺失 → `join_path(Env.get(..), ..)` 等 &str 形参
/// 调用漏 .as_str() → E0308）。
pub fn transpile_back_module_to_rs_with_proofs(
    module_name: &str,
    content: &str,
    source_dir: Option<std::path::PathBuf>,
) -> AutoResult<(
    String,
    Vec<auto_lang_core::stdlib_assembly::reference::ReferenceProof>,
)> {
    use auto_val::AutoStr;
    // Plan 399 §7: parse + transpile on a 16MB stack. db.at with deep nesting
    // (for + if/else + multi-field struct literals) overflows Windows's 1MB
    // main-thread stack (parser frames are large; cf. run_autovm lib.rs:341).
    // This mirrors the repo's established pattern. UTF-8-safety fixes in
    // strip_collection_new (post_process) handle the second §7 bug separately.
    // Error is converted to String inside the thread (AutoError is !Send).
    let content = content.to_string();
    let module = module_name.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || -> Result<(String, Vec<auto_lang_core::stdlib_assembly::reference::ReferenceProof>), String> {
            let (mut sink, references) = auto_lang_core::trans::rust::transpile_rust_full_with_options(
                AutoStr::from(module.as_str()),
                &content,
                None,
                source_dir.clone(),
                Some(auto_lang_core::stdlib_assembly::host::RustRuntime::Embedded),
                true,
            )
            .map_err(|e| format!("Failed to transpile {}.at: {}", module, e))?;
            let rust_code = String::from_utf8(sink.done().map_err(|e| e.to_string())?.to_vec())
                .map_err(|e| format!("Invalid UTF-8 in {}.rs output: {}", module, e))?;
            // AutoError is !Send; proofs cross the thread boundary as plain data.
            Ok((rust_code, references))
        })
        .map_err(|e| format!("Failed to spawn {}.rs transpile thread: {}", module_name, e))?;
    handle
        .join()
        .map_err(|_| format!("{}.rs transpile thread panicked", module_name))?
        .map_err(|e| e.into())
}

/// Plan musk-022 CRUD 扩展: post-process a2r's db.rs output to fix the known
/// backend-context gaps (a2r transpiles for a generic module; the HTTP backend
/// has specific shape). All rewrites below are mechanical, safe string fixes.
///
/// NOTE (Plan 399 调研 2026-08-07): these are **workarounds**, not redundant —
/// the corresponding a2r root causes are NOT fixed in `trans/rust.rs`. Verified:
/// - `use api:` → `crate::api` : a2r `use_stmt` has no api→types remap
///   (`rust.rs:11490` maps any non-stdlib bare module to `crate::<name>`).
/// - `List<T>.new(EXPR)` wrapper : a2r `call` has no List/Array interception
///   (`rust.rs:3659`); `GenName` emits `List<T>` verbatim.
/// - `&[T]` return over a MutexGuard : a2r `Type::Slice` always emits `&[T]`
///   regardless of return position (`rust.rs:1184`), no owned-Vec rewrite.
/// - str param → String field missing `.to_string()` : no such logic in a2r.
/// Do NOT delete these rewrites unless the a2r root cause is fixed first.
/// Generated backends use the embedded standard library from auto-lang-core.
/// Qualify the transpiler preamble and paths while preserving standalone faces.
pub fn qualify_a2r_std(code: String) -> String {
    // Plan 742 R3 (F-742-R3-01): single-pass, priority-ordered scan over
    // code regions only — NO placeholder/restore round-trip. A restore step
    // cannot distinguish a placeholder it inserted from a legal business
    // string of the same spelling (`"__AUTO_A2R_STD_QUAL__"` is a legal
    // str), so the r2 marker scheme rewrote literal payloads. Priority per
    // position: already-qualified > standalone-only faces > generic
    // qualification; `use` lines also qualify the bare crate name.
    // PLAN-668 R-24 / Plan 742: `a2r_std::sys` / `a2r_std::sqlite` are
    // standalone-crate-only faces (the embedded facade has neither) — they
    // keep their direct form, the sys one untouched and sqlite with the
    // standalone-crate `::` prefix.
    code.split('\n')
        .map(|l| {
            if !l.contains("a2r_std") {
                return l.to_string();
            }
            let mask = auto_lang_core::trans::rust::RustTrans::code_region_mask(l);
            let is_use_line = l.trim_start().starts_with("use ");
            let bytes = l.as_bytes();
            // Plan 742 R4 (F-742-R4-01): code-region identity is NOT module
            // identity — a match must start at an identifier boundary. A
            // legal companion named `custom_a2r_std` ends with the stdlib
            // token; without this check its qualified path
            // (`crate::custom_a2r_std::`) was rewritten into
            // `crate::custom_auto_lang::a2r_std::` (E0433).
            let at_ident_boundary = |i: usize| {
                i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_')
            };
            let mut out = String::with_capacity(l.len());
            let mut i = 0;
            while i < bytes.len() {
                if mask[i] && at_ident_boundary(i) {
                    // Canonical paths stay idempotent; old generated paths
                    // migrate in code regions only, preserving literal bytes.
                    let mut consumed = false;
                    for (prefix, replacement) in [
                        ("auto_lang_core::a2r_std", "auto_lang_core::a2r_std"),
                        ("auto_lang::a2r_std", "auto_lang_core::a2r_std"),
                    ] {
                        if let Some(after) = l[i..].strip_prefix(prefix) {
                            if after
                                .as_bytes()
                                .first()
                                .map_or(true, |b| !(b.is_ascii_alphanumeric() || *b == b'_'))
                            {
                                out.push_str(replacement);
                                i += prefix.len();
                                consumed = true;
                                break;
                            }
                        }
                    }
                    if consumed {
                        continue;
                    }
                    // Standalone-only faces — keep their direct form.
                    if l[i..].starts_with("a2r_std::sys::") {
                        out.push_str("a2r_std::sys::");
                        i += "a2r_std::sys::".len();
                        continue;
                    }
                    if l[i..].starts_with("a2r_std::sqlite") {
                        out.push_str("::a2r_std::sqlite");
                        i += "a2r_std::sqlite".len();
                        continue;
                    }
                    // Generic qualification.
                    if l[i..].starts_with("a2r_std::") {
                        out.push_str("auto_lang_core::a2r_std::");
                        i += "a2r_std::".len();
                        continue;
                    }
                    // `use a2r_std;` / `use a2r_std::*;` — bare crate name.
                    if is_use_line && l[i..].starts_with("a2r_std") {
                        let after = &l[i + "a2r_std".len()..];
                        if after.starts_with(';') || after.starts_with("::*") {
                            out.push_str("auto_lang_core::a2r_std");
                            i += "a2r_std".len();
                            continue;
                        }
                    }
                }
                let ch = l[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Plan 742 R2 (F-742-R2-01): plain-text replacement confined to code
/// regions of each line — strings/chars/comments keep their payload bytes.
/// Line-oriented (split on '\n' keeps the trailing newline shape); the
/// needle must not span lines (all current callers are single-line forms).
// PLAN-815 T-04 迁移归因：在 auto-man 即已无调用方（历史后处理族残员），
// 按单算法保真随迁，不在移动提交中裁剪冻结面。
#[allow(dead_code)]
fn replace_in_code_regions(code: &str, from: &str, to: &str) -> String {
    if !code.contains(from) {
        return code.to_string();
    }
    code.split('\n')
        .map(|l| {
            if !l.contains(from) {
                return l.to_string();
            }
            let mask = auto_lang_core::trans::rust::RustTrans::code_region_mask(l);
            let bytes = l.as_bytes();
            let mut out = String::with_capacity(l.len());
            let mut i = 0;
            while i < bytes.len() {
                if mask[i] && l[i..].starts_with(from) {
                    out.push_str(to);
                    i += from.len();
                } else {
                    let ch = l[i..].chars().next().unwrap();
                    out.push(ch);
                    i += ch.len_utf8();
                }
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Plan 742 R2 (F-742-R2-01): whole-text code-region mask (per-line masks
/// concatenated, '\n' bytes masked out). Single-line comment/string state —
/// multi-line block comments are outside the emitted-surface boundary
/// documented on the line-level scanner.
pub fn code_region_mask_text(text: &str) -> Vec<bool> {
    text.split('\n')
        .flat_map(|l| {
            let mut m = auto_lang_core::trans::rust::RustTrans::code_region_mask(l);
            m.push(false); // the '\n' separator itself
            m
        })
        .collect()
}

/// PLAN-681 T-01: light post-process for companion `src/back/*.at` modules
/// (fsys.at shape — stateless fn wrappers). Only the module-shape fixups that
/// are shape-generic: a2r_std re-qualification, api→types import remap and
/// `List<T>.new` stripping. The db-only heuristics in [`post_process_db_rs`]
/// (MutexGuard deref, borrowed-iterator clones, id widening) target CRUD
/// state shapes a companion module does not have — deliberately NOT applied.
pub fn post_process_companion_rs(code: String) -> String {
    let mut code = qualify_a2r_std(code);
    code = code.replace("use crate::api::", "use crate::types::");
    code = strip_collection_new(&code);
    code
}

/// PLAN-885 G4: 伴生模块中与 api 同形同名的 struct 提升为公共 types 引用。
/// `db.rs` 经 `use api: … Work, SaveResult` → `use crate::types::…` 使用公共
/// 类型，而伴生模块（works/drafts）自行定义了同形 `Work`/`SaveResult` ——
/// 生成后出现 `types::Work` vs `works::Work` 两个不同类型（E0308）。
/// 字段完全一致时，把伴生模块的 struct 定义替换为 `pub use crate::types::X;`
/// （类型统一为公共面，db 包装与伴生返回同一 Rust 类型）。
pub fn promote_api_types_in_companion(
    code: &str,
    api_types: &[auto_lang_core::api::ApiType],
) -> String {
    use std::collections::HashSet;
    // api 类型名 → 字段名（按顺序）映射。
    let api_fields: HashSet<(&str, Vec<String>)> = api_types
        .iter()
        .map(|t| {
            (
                t.name.as_str(),
                t.fields.iter().map(|f| f.name.clone()).collect(),
            )
        })
        .collect();
    if api_fields.is_empty() {
        return code.to_string();
    }
    let mut out = String::with_capacity(code.len());
    let mut rest = code;
    loop {
        // 找下一个 `pub struct NAME {` 块起点（含前一行的 derive 注解）。
        let Some(marker_start) = rest.find("pub struct ") else {
            out.push_str(rest);
            break;
        };
        // 保留 marker 之前的文本。
        out.push_str(&rest[..marker_start]);
        let after_marker = &rest[marker_start + "pub struct ".len()..];
        let Some(name_end) = after_marker.find(|c: char| c.is_whitespace() || c == '{') else {
            out.push_str(&rest[marker_start..]);
            break;
        };
        let type_name = &after_marker[..name_end];
        let Some(open_idx) = after_marker.find('{') else {
            out.push_str(&rest[marker_start..]);
            break;
        };
        let block_start = marker_start;
        let body_start = marker_start + "pub struct ".len() + open_idx + 1;
        // 括号平衡扫描块尾。
        let mut depth = 1usize;
        let mut i = body_start;
        while i < rest.len() {
            match rest.as_bytes()[i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if depth != 0 {
            out.push_str(&rest[block_start..]);
            break;
        }
        let block_text = &rest[block_start..=i];
        // 提取字段名（`    pub name: Ty,` 行）。
        let fields: Vec<String> = block_text
            .lines()
            .filter_map(|l| {
                let t = l.trim_start();
                t.strip_prefix("pub ").and_then(|r| {
                    r.find(':')
                        .map(|c| r[..c].trim().to_string())
                        .filter(|f| !f.is_empty())
                })
            })
            .collect();
        let matches_api = api_fields
            .iter()
            .any(|(name, api_fs)| *name == type_name && *api_fs == fields);
        if matches_api {
            // 整块（含其上方连续的属性/doc 注释行）替换为 pub use。
            // 伴生模块对同名类型常带 `#[derive(...)]`/`/// …` 属性，pub use
            // 不能承接 derive（E0774）——向上吞掉从块起点到上一个空行/非
            // 属性行的所有连续 `#[…]`/`///` 行。注意 out 以 `\n` 结尾，
            // rfind 需先跳过结尾换行。
            loop {
                let end = out.trim_end_matches('\n').len();
                if end == 0 {
                    break;
                }
                let Some(dl) = out[..end].rfind('\n') else {
                    break;
                };
                let tail = &out[dl + 1..end];
                let trimmed = tail.trim_start();
                if trimmed.starts_with("#[") || trimmed.starts_with("///") {
                    out.truncate(dl);
                } else {
                    break;
                }
            }
            out.push_str(&format!("pub use crate::types::{};\n", type_name));
        } else {
            out.push_str(block_text);
            out.push('\n');
        }
        rest = &rest[i + 1..];
    }
    out
}

pub fn post_process_db_rs(mut code: String) -> String {
    code = qualify_a2r_std(code);
    code = code.replace("use crate::api::", "use crate::types::");
    // Strip `List<T>.new(EXPR)` -> `EXPR` (a2r leaves the wrapper; List=Vec, the
    // array literal is already vec![...]). Bracket-balanced over the .new(...) parens.
    code = strip_collection_new(&code);
    // Plan 399 Phase 11.3: &[T] return + global clone now handled in a2r
    // (rust_return_type_name emits Vec<T>; write_return_expr adds .clone() for
    // global var returns). Keeping fix_borrowed_slice_returns as a no-op fallback
    // is unnecessary once a2r is verified — comment out to confirm.
    // code = fix_borrowed_slice_returns(&code);
    // Plan 399 §3/P11.6: over-deref before method calls. Generalize the hardcoded
    // MESSAGES to any global + any mutating method: `*VAR.lock().unwrap().METHOD(`
    // → drop `*`. a2r over-dereferences the MutexGuard before a method call
    // (push/insert/extend/etc. need &mut self via DerefMut, not a moved value).
    // P11.6 a2r根治(改 Expr::Ident)回归面广,保留这个后处理正则作为完整覆盖.
    {
        use regex::Regex;
        if let Ok(re) = Regex::new(
            r"\*(\w+)\.lock\(\)\.unwrap\(\)\.(push|insert|extend|pop|remove|retain|clear|sort_by|sort|swap|truncate|drain|splice|resize)\(",
        ) {
            code = re.replace_all(&code, "$1.lock().unwrap().$2(").to_string();
        }
    }
    // Param-to-field &str -> String: a2r passes &str fn params into String struct
    // fields without .to_string(). Regex: `<field>: <param>,` where both are the same
    // bare ident and field is a known String field (skip id/bool/time/count).
    code = append_tostring_for_str_fields(&code);
    // id field type widening: backend types.rs uses i64 for int, but a2r emits i32
    // guards. `id: *NEXTID.lock().unwrap()` -> add `as i64` for the id field.
    code = code.replace(
        "id: *NEXTID.lock().unwrap()",
        "id: *NEXTID.lock().unwrap() as i64",
    );
    // Plan 399 Phase 11.1: a2r now emits i64 for `int` (rust_type_name Type::Int
    // => i64 + as i64 casts). The blunt code.replace("i32","i64") is no longer
    // needed — comment out to confirm 015/017 still compile.
    // code = code.replace("i32", "i64");
    // Plan 399 §3: a2r emits `let results: Vec<T> = ...` without `mut` but then
    // calls results.push(). Add `mut` to `let NAME:` that is followed (in the same
    // fn) by `NAME.push`. Simple per-line heuristic: `let X = vec![]` / `let X:` → `let mut X`.
    // Plan 399 Phase 11.5: a2r now scans the fn body for mutated `let` bindings
    // (scan_mutated_bindings) and emits `let mut` — this post-process is removed.
    // code = add_mut_to_let_collections(&code);
    // Plan 399 §3: a2r iterates borrowed (`for note in &*G.lock()`) but moves
    // fields out of the shared reference in struct literals (`tags: note.tags`)
    // and returns (`return Some(note)`).
    // Plan 399 Phase 11.4: a2r now clones borrowed-iter field reads in struct
    // ctors (write_expr_for_struct_field). The return-Some(note) case still
    // needs this post-process (a2r doesn't yet clone bare iter-var returns) —
    // keep it until that's covered.
    code = append_clone_for_borrowed_fields(&code);
    code
}

/// Plan 399 §3: in `Type { ..., field: note.x, ... }` struct literals, `note` is
/// a `&Note` (borrowed iterator), so moving `note.tags`/`note.title` errors.
/// Append `.clone()` to every `<ident>.<ident>` read inside struct-ctor fields.
/// Also fix `return Some(<ident>)` over a borrow → `Some(<ident>.clone())`.
fn append_clone_for_borrowed_fields(code: &str) -> String {
    use regex::Regex;
    let mut out = code.to_string();
    // `field: name.attr` (not already cloned, not a method call) → add .clone()
    // Matches `word.word` followed by `,` or ` }` (struct field end). Avoids
    // `word.word(...)` calls and already-`.clone()`d reads.
    let re = match Regex::new(r"(\b\w+:\s*)(\w+)\.(\w+)(,|\s*\})") {
        Ok(r) => r,
        Err(_) => return out,
    };
    out = re
        .replace_all(&out, "${1}${2}.${3}.clone()${4}")
        .to_string();
    // `return Some(name)` → `return Some(name.clone())` when name is a bare ident
    // (covers `for note in &*G.lock() { ... return Some(note) }`).
    let re2 = match Regex::new(r"return Some\((\w+)\)") {
        Ok(r) => r,
        Err(_) => return out,
    };
    out = re2.replace_all(&out, "return Some($1.clone())").to_string();
    out
}

/// Plan 399 §3: add `mut` to `let NAME = vec![...]` / `let NAME: Vec<...> = ...`
/// when the binding is later mutated (a2r omits `mut`). Conservative: only
/// prefixes `let ` → `let mut ` for lines that look like a fresh Vec binding.
// PLAN-815 T-04 迁移归因：在 auto-man 即已无调用方（历史后处理族残员），
// 按单算法保真随迁，不在移动提交中裁剪冻结面。
#[allow(dead_code)]
fn add_mut_to_let_collections(code: &str) -> String {
    use regex::Regex;
    let re = match Regex::new(r"(?m)^(\s*)let (\w+)(:\s*Vec<| =\s*vec!\[)") {
        Ok(r) => r,
        Err(_) => return code.to_string(),
    };
    re.replace_all(code, "${1}let mut ${2}${3}").to_string()
}

/// For `Type { field: field, ... }` where `field` is a &str param assigned to a
/// String struct field, append `.to_string()`. Scans for `ident: ident,` pairs
/// (no regex backref — regex crate lacks it) where the ident is a str param.
fn append_tostring_for_str_fields(code: &str) -> String {
    use std::collections::HashSet;
    let mut str_params: HashSet<String> = HashSet::new();
    // Plan 399 §3: slice params (&[String]) assigned to Vec<String> fields need
    // .to_vec() (a2r passes them straight: `tags: tags` where tags: &[String]).
    let mut slice_params: HashSet<String> = HashSet::new();
    for line in code.lines() {
        let l = line.trim_start();
        if !(l.starts_with("pub fn ") || l.starts_with("fn ")) {
            continue;
        }
        if let Some(open) = l.find('(') {
            if let Some(close) = l[open..].find(')') {
                for p in l[open + 1..open + close].split(',') {
                    let tok: Vec<&str> = p
                        .trim()
                        .split(|c: char| c == ':' || c.is_whitespace())
                        .filter(|t| !t.is_empty())
                        .collect();
                    if tok.len() >= 2 && (tok[1] == "str" || tok[1] == "&str") {
                        str_params.insert(tok[0].to_string());
                    }
                    // Slice param: `name &[String]` or `name &[...]`.
                    if tok.len() >= 2 && tok[1].starts_with("&[") {
                        slice_params.insert(tok[0].to_string());
                    }
                }
            }
        }
    }
    let skip: HashSet<&str> = [
        "id",
        "mine",
        "done",
        "pinned",
        "time",
        "count",
        "unread",
        "active_id",
    ]
    .iter()
    .copied()
    .collect();
    // Scan the whole code; for each word w that is a str param, replace `w: w,` -> `w: w.to_string(),`.
    // Process by iterating over str_params (small set) and doing targeted replace.
    let mut out = code.to_string();
    for name in &str_params {
        if skip.contains(name.as_str()) {
            continue;
        }
        // Plan 399 §3: a2r emits `field: field` for str params; add .to_string().
        // Match both `,` (mid-struct) and ` }` (last field, no trailing comma).
        for sep in [",", " }"] {
            let needle = format!("{}: {}{}", name, name, sep);
            let repl = format!("{}: {}.to_string(){}", name, name, sep);
            out = out.replace(&needle, &repl);
        }
    }
    // Plan 399 §3: slice params (`tags: &[String]`) → `field: tags.to_vec()`.
    for name in &slice_params {
        for sep in [",", " }"] {
            let needle = format!("{}: {}{}", name, name, sep);
            let repl = format!("{}: {}.to_vec(){}", name, name, sep);
            out = out.replace(&needle, &repl);
        }
    }
    out
}

/// Strip `List<T>.new(...)` / `Array<T>.new(...)` wrappers, leaving the inner expr.
/// Plan 399 §7: made UTF-8 safe — `List</Array<` are ASCII so byte-indexing for
/// matching is fine, but the byte cursor must skip whole UTF-8 sequences when
/// it lands on a non-ASCII lead byte (e.g. emoji in seed data like 📄), else
/// `code[i..]` slices into a multi-byte char and panics.
fn strip_collection_new(code: &str) -> String {
    // Plan 742 R2 (F-742-R2-01): constructs must sit in code regions — a
    // business string spelling `List<T>.new([...])` is literal text.
    let mask = code_region_mask_text(code);
    let mut out = String::with_capacity(code.len());
    let bytes = code.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Only attempt the List/Array match at an ASCII lead byte — landing mid
        // multi-byte char means we just copy the byte run through.
        if mask[i]
            && bytes[i] < 0x80
            && (code[i..].starts_with("List<") || code[i..].starts_with("Array<"))
        {
            let is_array = code[i..].starts_with("Array<");
            let lt_at = if is_array { i + 5 } else { i + 4 };
            if let Some(gt_rel) = code[lt_at..].find('>') {
                let gt_at = lt_at + gt_rel;
                if mask[gt_at]
                    && code
                        .get(gt_at + 1..)
                        .map_or(false, |s| s.starts_with(".new("))
                {
                    let paren_open = gt_at + 5;
                    if let Some(paren_close) = balance_paren(code, paren_open) {
                        out.push_str(&code[paren_open + 1..paren_close]);
                        i = paren_close + 1;
                        continue;
                    }
                }
            }
        }
        // Advance one whole UTF-8 char so the cursor stays on a char boundary.
        let ch_len = utf8_len(bytes[i]);
        out.push_str(&code[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Byte length of the UTF-8 char whose lead byte is `b`. Plan 399 §7.
fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b < 0xC0 {
        1
    }
    // continuation byte (shouldn't be a lead, stay safe)
    else if b < 0xE0 {
        2
    } else if b < 0xF0 {
        3
    } else {
        4
    }
}

/// Find the matching ')' for the '(' at `open`, balancing () [] {} and "".
fn balance_paren(code: &str, open: usize) -> Option<usize> {
    let bytes = code.as_bytes();
    let mut depth = 1i32;
    let mut in_str = false;
    let mut j = open + 1;
    while j < bytes.len() {
        let c = bytes[j] as char;
        if in_str {
            if c == '"' {
                in_str = false;
            }
            j += 1;
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// `pub fn f() -> &[T] { return *G.lock().unwrap(); }` -> `pub fn f() -> Vec<T>
/// { return G.lock().unwrap().clone(); }`. The borrowed-slice-over-guard form
/// doesn't compile; return an owned clone instead.
// PLAN-815 T-04 迁移归因：在 auto-man 即已无调用方（历史后处理族残员），
// 按单算法保真随迁，不在移动提交中裁剪冻结面。
#[allow(dead_code)]
fn fix_borrowed_slice_returns(code: &str) -> String {
    let mut out = code.to_string();
    // Return type: `&[T]` -> `Vec<T>`
    while let Some(idx) = out.find("-> &[") {
        if let Some(end) = out[idx..].find("]") {
            let abs_end = idx + end;
            // Replace "-> &[T]" (5 chars "-> &[") with "-> Vec<[T]"
            out.replace_range(
                idx..=abs_end,
                &format!("-> Vec<{}>", &out[idx + 5..abs_end]),
            );
        } else {
            break;
        }
    }
    // Body: `return *G.lock().unwrap();` -> `return G.lock().unwrap().clone();`
    // Plan 399 §3: generalize the hardcoded MESSAGES/NOTES to any global (e.g.
    // FOLDERS) — `return *<UPPER>.lock().unwrap();` over a guard can't move.
    {
        use regex::Regex;
        if let Ok(re) = Regex::new(r"return \*(\w+)\.lock\(\)\.unwrap\(\);") {
            out = re
                .replace_all(&out, "return $1.lock().unwrap().clone();")
                .to_string();
        }
    }
    out
}

/// Determine the primary type from an ApiModule (first defined type)
pub fn primary_type_name_pub(api_module: &auto_lang_core::api::ApiModule) -> Option<String> {
    api_module.types.first().map(|t| t.name.clone())
}

/// Extract the set of `pub fn NAME` names from a transpiled db.rs source.
/// Used to decide whether an HTTP handler can delegate to `db::NAME(...)`
/// instead of the `State<Db>` CRUD template.
pub fn extract_db_fn_names(db_rs: &str) -> std::collections::HashSet<String> {
    use regex::Regex;
    let mut set = std::collections::HashSet::new();
    // Match `pub fn name(` at the start of a line (a2r emits this form).
    let re = Regex::new(r"(?m)^\s*pub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(").expect("valid regex");
    for cap in re.captures_iter(db_rs) {
        if let Some(m) = cap.get(1) {
            set.insert(m.as_str().to_string());
        }
    }
    set
}

// ============================================================================
// Lenient API Extraction (Plan 132)
// ============================================================================

/// Extract API definitions leniently - skip unresolvable module references
///
/// This function uses regex-based parsing to extract API definitions without
/// requiring full module resolution. This is useful when `back/api.at` contains
/// `use db` statements where the db module isn't available during extraction.
pub fn extract_api_lenient(api_content: &str) -> Option<ApiModule> {
    use regex::Regex;

    let mut module = ApiModule::new("api".to_string());

    // Extract type definitions using regex
    // Pattern: pub type Name = { fields }
    let type_pattern = Regex::new(r"pub\s+type\s+(\w+)\s*=\s*\{([^}]+)\}").ok()?;

    for cap in type_pattern.captures_iter(api_content) {
        let name = cap.get(1)?.as_str().to_string();
        let fields_str = cap.get(2)?.as_str();

        let fields = parse_fields(fields_str);
        module.types.push(ApiType {
            name,
            fields,
            doc: None,
            alias_of: None,
        });
    }

    // Extract #[api] function definitions
    // Pattern: #[api(...)] pub fn name(params) return_type {
    // Note: return_type may be followed by { or whitespace
    let fn_pattern =
        Regex::new(r#"#\[api\(([^]]*)\]\s*pub\s+fn\s+(\w+)\s*\(([^)]*)\)\s*(\S+)?"#).ok()?;

    for cap in fn_pattern.captures_iter(api_content) {
        let annotation_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let fn_name = cap.get(2)?.as_str().to_string();
        let params_str = cap.get(3).map(|m| m.as_str()).unwrap_or("");
        // Return type may have trailing { which we need to strip
        let return_type_raw = cap.get(4).map(|m| m.as_str()).unwrap_or("void");
        let return_type = return_type_raw.trim_end_matches('{').trim().to_string();
        let return_type = if return_type.is_empty() {
            "void".to_string()
        } else {
            return_type
        };

        // Extract method from annotation (e.g., method = "GET")
        let method_pattern = Regex::new(r#"method\s*=\s*"(\w+)""#).ok()?;
        let method = method_pattern
            .captures(annotation_str)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "GET".to_string());

        // Extract path from annotation (e.g., path = "/api/users")
        let path_pattern = Regex::new(r#"path\s*=\s*"([^"]+)""#).ok()?;
        let path = path_pattern
            .captures(annotation_str)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| format!("/api/{}", fn_name));

        let params = parse_params(params_str);
        let mut attrs = ApiAttrs::new();
        attrs.method = Some(method);
        attrs.path = Some(path);
        let mut endpoint = ApiEndpoint::new(fn_name.clone(), attrs);
        endpoint.params = params;
        endpoint.return_type = return_type;

        module.endpoints.push(endpoint);
    }

    Some(module)
}

/// Parse type fields from a string like "id: int\nname: str" or the
/// space-separated form "id int\nname str" (Plan 317 supports both).
/// Plan 043 M5 B-4: colon-less fields (e.g. `commands []ToolEntry`,
/// `rows [][]RenderedCell`) were previously dropped, so the generated
/// interface silently lost fields and callers failed with TS2339.
#[doc(hidden)]
pub fn parse_fields(fields_str: &str) -> Vec<ApiField> {
    fields_str
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }

            // Split on ':' to get name and type (canonical form)
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            let (name, ty) = if parts.len() == 2 && !parts[1].trim().is_empty() {
                (parts[0].trim(), parts[1].trim())
            } else {
                // Colon-less "name type" form: split on the first whitespace.
                // A bare name (no type) is skipped, matching the colon path.
                let mut ws = line.splitn(2, char::is_whitespace);
                match (ws.next(), ws.next()) {
                    (Some(n), Some(t)) => (n, t.trim()),
                    _ => return None,
                }
            };
            if name.is_empty() {
                return None;
            }
            Some(ApiField {
                name: name.to_string(),
                ty: ty.to_string(),
                optional: false,
                default: None,
            })
        })
        .collect()
}

/// Parse function parameters from a string like "id int, name str"
#[doc(hidden)]
pub fn parse_params(params_str: &str) -> Vec<ApiParam> {
    if params_str.trim().is_empty() {
        return Vec::new();
    }

    params_str
        .split(',')
        .filter_map(|param| {
            let parts: Vec<&str> = param.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                Some(ApiParam {
                    name: parts[0].to_string(),
                    ty: parts[1].to_string(),
                    optional: false,
                    default: None,
                })
            } else {
                None
            }
        })
        .collect()
}
