//! PLAN-843 结构拆分：run* 执行入口族（自 lib.rs 原样搬移，仅组合）。
//! PLAN-884 T-03：同构桥接机制单源收敛到 [`run_bridge`]；本文件只保留
//! 公开签名、引擎选择 facade（G4）与受审计的参数绑定/元组投影。
use super::*;

mod run_bridge;

pub fn run(code: &str) -> AutoResult<String> {
    // Plan 081 Phase 1: AutoVM is now the default (no feature flag required)
    // Use execution engine selector to get the engine (with env override support)
    let engine = execution_engine::ExecutionEngine::get();
    execution_engine::execute_with_engine(engine, code)
}

/// Plan 177: Run AutoLang code with stdout capture for testing
///
/// Returns (result_value, captured_stdout)
pub fn run_with_capture(code: &str) -> AutoResult<(String, String)> {
    let engine = execution_engine::ExecutionEngine::get();
    execution_engine::execute_with_engine_capture(engine, code)
}

/// P574 T6:VM 执行线程栈。历史硬编码 4MB 已被 516KB aavm lib 拼合源的
/// 解释栈需求越过(探针定量:4MB 爆/5MB 过,递归有限);且显式 stack_size
/// 会绕过 RUST_MIN_STACK——Plan 423 抬测试线程栈护栏的本意对此失效。
/// 改为 RUST_MIN_STACK 可覆盖(字节),缺省 16MB,下限保底 4MB。
pub fn vm_thread_stack_size() -> usize {
    std::env::var("RUST_MIN_STACK")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n >= 4 * 1024 * 1024)
        .unwrap_or(16 * 1024 * 1024)
}

/// Plan 773 F-08: run failure observed at the production stage boundary,
/// with the typed VM trap code attached when the failure IS a run-stage trap
/// (decided where the typed `VMError` still exists). `run_trap_code: None`
/// means the failure did not come from a VM trap (I/O, missing files,
/// synthetic deadlines, panics) — the test-case adapters treat exactly
/// those as infrastructure ERRORs, never as language negatives.
#[derive(Debug)]
pub struct StagedRunFailure {
    pub error: crate::error::AutoError,
    /// `VMError` variant name from the engine's run-task loop, if any.
    pub run_trap_code: Option<&'static str>,
}

/// Plan 773 F-08: staged twin of [`run_with_capture_and_path`] — identical
/// execution, but a failing run carries the typed trap code so the run-stage
/// oracle channel is type-derived instead of message-text-parsed. The old
/// API keeps its exact signature and behavior.
pub fn run_with_capture_and_path_staged(
    code: &str,
    path: &str,
) -> Result<(String, String), StagedRunFailure> {
    run_bridge::run_staged_on_dedicated_thread(code, path)
}

/// Run AutoLang code with stdout capture, using the source file path for
/// module resolution (so `use db` resolves `db.at` next to the source file).
pub fn run_with_capture_and_path(code: &str, path: &str) -> AutoResult<(String, String)> {
    run_with_capture_and_path_staged(code, path).map_err(|f| f.error)
}

/// Run AutoLang code with stdout capture and return the disassembled bytecode.
pub fn run_with_capture_and_bytecode(
    code: &str,
) -> AutoResult<(String, String, Vec<crate::vm::disasm::DisasmLine>)> {
    run_with_capture_and_bytecode_with_meta(code).map(|(r, out, bc, _)| (r, out, bc))
}

/// Like [`run_with_capture_and_bytecode`], also returning the symbol tables
/// (strings/functions/natives) for resolving disassembly operand references.
pub fn run_with_capture_and_bytecode_with_meta(
    code: &str,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_with_capture_and_bytecode_with_deadline(code, None)
}

/// PLAN-746 (PG-MEM-1): like [`run_with_capture_and_bytecode_with_meta`] but
/// installs a cooperative execution deadline. Playground /api/run uses this
/// to bound runaway programs; `None` keeps unlimited execution.
pub fn run_with_capture_and_bytecode_with_deadline(
    code: &str,
    deadline: Option<std::time::Instant>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_with_capture_and_bytecode_with_input(code, deadline, None)
}

/// Like [`run_with_capture_and_bytecode_with_deadline`] with request-local
/// stdin. `Some("")` means immediate EOF; `None` preserves process stdin.
pub fn run_with_capture_and_bytecode_with_input(
    code: &str,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_bridge::run_on_dedicated_thread(code, true, None, deadline, input)
}

/// Run AutoLang code with stdout capture and source path, returning bytecode.
pub fn run_with_capture_and_path_and_bytecode(
    code: &str,
    path: &str,
) -> AutoResult<(String, String, Vec<crate::vm::disasm::DisasmLine>)> {
    run_with_capture_and_path_and_bytecode_with_meta(code, path).map(|(r, out, bc, _)| (r, out, bc))
}

/// Like [`run_with_capture_and_path_and_bytecode`], also returning [`crate::vm::disasm::BytecodeMeta`].
pub fn run_with_capture_and_path_and_bytecode_with_meta(
    code: &str,
    path: &str,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_with_capture_and_path_and_bytecode_with_deadline(code, path, None)
}

/// PLAN-746 (PG-MEM-1): path variant of [`run_with_capture_and_bytecode_with_deadline`]
/// (playground project runs — `use db` resolution needs the source path).
pub fn run_with_capture_and_path_and_bytecode_with_deadline(
    code: &str,
    path: &str,
    deadline: Option<std::time::Instant>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_with_capture_and_path_and_bytecode_with_input(code, path, deadline, None)
}

/// Path-aware execution with a deadline and request-local stdin.
pub fn run_with_capture_and_path_and_bytecode_with_input(
    code: &str,
    path: &str,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_bridge::run_on_dedicated_thread(code, true, Some(path), deadline, input)
}

/// PLAN-746 (PG-MEM-1 / SD-04): join an execution thread without panicking.
/// A panic inside the VM execution thread (e.g. a parser bug on hostile
/// input) surfaces as a clean `Err` instead of an `unwrap` panic bubbling
/// up through the caller (previously axum turned that into a bare 500).
pub fn join_execution<T: Send + 'static>(
    handle: std::thread::JoinHandle<AutoResult<T>>,
) -> AutoResult<T> {
    match handle.join() {
        Ok(r) => r,
        Err(_) => Err(crate::error::AutoError::Msg(
            "execution thread panicked".to_string(),
        )),
    }
}

/// Run AutoLang code using AutoVM (bytecode VM)
///
/// **Plan 068 Phase 9**: Primary execution engine for AutoLang
///
/// This function compiles AutoLang code to ABC bytecode and executes it
/// on the AutoVM virtual machine. AutoVM is faster than the evaluator and
/// provides consistent behavior across PC and MCU environments.
///
/// # Examples
/// ```ignore
/// let result = run_autovm("1 + 2").unwrap();  // Returns "3"
/// ```
pub fn run_autovm(code: &str) -> AutoResult<String> {
    // 专用线程 + LocalSet 桥接（capture=false：stdout 透传进程，不捕获）。
    // 栈策略统一为 vm_thread_stack_size()（RUST_MIN_STACK≥4MiB 可覆盖，缺省 16MiB）。
    run_bridge::run_on_dedicated_thread(code, false, None, None, None).map(|(r, _, _, _)| r)
}

/// Plan 177: Run AutoVM with stdout capture for testing
pub fn run_autovm_capture(code: &str) -> AutoResult<(String, String)> {
    run_bridge::run_on_dedicated_thread(code, true, None, None, None)
        .map(|(r, stdout, _, _)| (r, stdout))
}

pub fn parse(code: &str) -> AutoResult<ast::Code> {
    // Plan 091: Universe removed
    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code);
    parser.parse().map_err(|e| e.to_string().into())
}

/// Parse code and return proper AutoError (not converted to string)
/// This is used by the LSP to get detailed error information
pub fn parse_preserve_error(code: &str) -> Result<ast::Code, error::AutoError> {
    // Plan 091: Universe removed
    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code);
    parser.parse()
}

static EXTERNAL_BACK_ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// 设置外部后端根(宿主编排层在 run_file 前调用;路径须为绝对路径)。
pub fn set_external_back_root(root: std::path::PathBuf) {
    let _ = EXTERNAL_BACK_ROOT.set(root);
}

/// PLAN-635 D4: does this pac.at text declare a dependency with the given
/// name? Matches both `dep "name" { ... }` and `dep name { ... }` forms with
/// a word-boundary check so `dep settings` does not match `dep settings_v2`.
pub fn pac_declares_dep(content: &str, dep_name: &str) -> bool {
    for form in [format!("dep \"{}\"", dep_name), format!("dep {}", dep_name)] {
        if let Some(pos) = content.find(&form) {
            let after = content[pos + form.len()..].chars().next();
            // The name must end at a non-identifier char (whitespace, `{`, `,`, `:`).
            let boundary = after.is_some_and(|c| !(c.is_alphanumeric() || c == '_' || c == '.'));
            if boundary {
                return true;
            }
        }
    }
    false
}

/// PLAN-635 D5: if this pac.at declares a workspace whose members include a
/// directory matching `dep_name` (member entry's last path segment), return
/// that entry's relative path. Members syntax: `members: ["common", "apps/x"]`
/// (single- or multi-line).
pub fn pac_workspace_member_dir(content: &str, dep_name: &str) -> Option<String> {
    let members_pos = content.find("members")?;
    let list_start = content[members_pos..].find('[')? + members_pos;
    let list_end = content.find(']').filter(|e| *e > list_start)?;
    let list = &content[list_start + 1..list_end];
    for entry in list.split(',') {
        let entry = entry.trim().trim_matches(|c| c == '"' || c == '\'');
        if entry.is_empty() {
            continue;
        }
        let last = entry.rsplit(['/']).next().unwrap_or(entry);
        if last == dep_name {
            return Some(entry.to_string());
        }
    }
    None
}

/// PLAN-647 guardrail B (contract Q5): version-plane keys inside a pac.at
/// `dep` declaration are a violation. The version plane is not enabled —
/// blueprints are a single rolling source and cross-repo alignment is
/// git-level — so a `version:`/`pin:`/… key would read like a constraint
/// while being silently ignored. Returns the offending message (points at
/// contract Q5) or None when the declaration is clean.
pub fn pac_dep_version_violation(content: &str, dep_name: &str) -> Option<String> {
    const VERSION_KEYS: [&str; 6] = ["version", "pin", "rev", "tag", "branch", "commit"];
    for form in [format!("dep \"{dep_name}\""), format!("dep {dep_name}")] {
        let Some(pos) = content.find(&form) else {
            continue;
        };
        // Same name-boundary discipline as pac_declares_dep: the matched name
        // must end at a non-identifier char (`bpsx` must not hit `dep bps`).
        let after = content[pos + form.len()..].chars().next();
        if !after.is_some_and(|c| !(c.is_alphanumeric() || c == '_' || c == '.')) {
            continue;
        }
        // Scan only this declaration's `{ ... }` block.
        let Some(brace) = content[pos..].find('{') else {
            continue;
        };
        let brace = pos + brace;
        let Some(block_len) = content[brace..].find('}') else {
            continue;
        };
        let block = &content[brace..brace + block_len];
        for key in VERSION_KEYS {
            for prefix in [format!("{key}:"), format!("{key} =")] {
                let Some(kpos) = block.find(&prefix) else {
                    continue;
                };
                // Word-start check so `rev:` doesn't hit `prev:`.
                let before = block[..kpos].chars().last();
                if !before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    return Some(format!(
                        "pac.at dep `{dep_name}` declares `{key}` — the version plane is \
                         not enabled (single-source ruling: docs/specs/blueprint/contract.md \
                         Q5); declare `path` only, cross-repo alignment is carried by git"
                    ));
                }
            }
        }
    }
    None
}

fn module_path_candidates(rel: &str) -> Vec<String> {
    const MAX_VARIANTS: usize = 4;
    let mut out = vec![rel.to_string()];
    let segs: Vec<&str> = rel.split(std::path::MAIN_SEPARATOR).collect();
    let eligible: Vec<usize> = (0..segs.len()).filter(|&i| segs[i].contains('_')).collect();
    if eligible.is_empty() || eligible.len() > 16 {
        return out;
    }
    for mask in 1..(1usize << eligible.len()) {
        if out.len() > MAX_VARIANTS {
            break;
        }
        let cand: Vec<String> = segs
            .iter()
            .enumerate()
            .map(|(i, s)| match eligible.iter().position(|&e| e == i) {
                Some(bit) if mask & (1 << bit) != 0 => s.replace('_', "-"),
                _ => s.to_string(),
            })
            .collect();
        out.push(cand.join(std::path::MAIN_SEPARATOR_STR));
    }
    out
}

pub fn resolve_module_path(base_dir: &std::path::Path, module: &str) -> Option<std::path::PathBuf> {
    // Plan 061:外部后端链接 —— `back.X` 优先映射到后端项目根
    // (`back.api` → `<root>/api.at`,`back.shell` → `<root>/shell.at`;
    // mod.at 形态同样支持)。未配置或不命中则回落常规解析(本地 back/)。
    if let Some(root) = EXTERNAL_BACK_ROOT.get() {
        let mapped = if module == "back" {
            "api".to_string()
        } else if let Some(rest) = module.strip_prefix("back.") {
            rest.replace('.', std::path::MAIN_SEPARATOR_STR)
        } else {
            String::new()
        };
        if !mapped.is_empty() {
            for cand in module_path_candidates(&mapped) {
                let as_file = root.join(format!("{}.at", cand));
                if as_file.exists() {
                    return Some(as_file);
                }
                let as_mod = root.join(&cand).join("mod.at");
                if as_mod.exists() {
                    return Some(as_mod);
                }
            }
        }
    }
    let rel = module.replace('.', std::path::MAIN_SEPARATOR_STR);
    // Helper to probe a candidate directory for {rel}.at / {rel}/mod.at —
    // literal form first, then kebab variants of underscore segments
    // (PLAN-649 SD-01; candidates without underscores are literal-only, so
    // existing behavior is untouched).
    let probe = |dir: &std::path::Path| -> Option<std::path::PathBuf> {
        for cand in module_path_candidates(&rel) {
            let as_file = dir.join(format!("{}.at", cand));
            if as_file.exists() {
                return Some(as_file);
            }
            let as_mod = dir.join(&cand).join("mod.at");
            if as_mod.exists() {
                return Some(as_mod);
            }
        }
        None
    };
    // Direct resolution relative to base_dir.
    if let Some(p) = probe(base_dir) {
        return Some(p);
    }
    if let Some(p) = crate::util::find_aavm_module(&module.replace('.', "/"), base_dir) {
        return Some(p);
    }
    // Plan 327 Phase B: fall back to the parent directory so a sibling layout
    // like src/{front,back}/ resolves. `app.at` in src/front/ imports
    // `back.api` (src/back/api.at); base_dir is src/front/, so the direct probe
    // misses it. Trying the parent (src/) finds src/back/api.at. This mirrors
    // the script path, which seeds CompileSession.source_dirs with the source
    // dir's parent.
    if let Some(parent) = base_dir.parent() {
        if let Some(p) = probe(parent) {
            return Some(p);
        }
    }

    // 向上遍历查找 deps 目录 (如 src/front/ -> src/ -> root/deps/) 或 pac.at 中的本地 path 依赖
    let (dep_name, _) = match module.find('.') {
        Some(dot) => (&module[..dot], &module[dot + 1..]),
        None => (module, ""),
    };
    // PLAN-635 D4/D5: deps/ 与 workspace members 的候选探测序列（与本地
    // path 依赖一致）。声明门控（pnpm 式严格隔离）：deps/<name> 仅在 pac.at
    // `dep` 声明（或 workspace member）时可达；物化但未声明 = 幽灵依赖，
    // 阻断解析并给修复指引。
    let probe_pkg = |pkg_root: &std::path::Path| -> Option<std::path::PathBuf> {
        let (_, sub) = match module.find('.') {
            Some(dot) => (&module[..dot], &module[dot + 1..]),
            None => (module, ""),
        };
        if sub.is_empty() {
            let candidates = [
                pkg_root.join("src").join("front").join("app.at"),
                pkg_root.join("src").join("front").join("mod.at"),
                pkg_root.join("src").join("mod.at"),
                pkg_root.join("mod.at"),
                pkg_root.join(format!("{}.at", dep_name)),
            ];
            candidates.into_iter().find(|c| c.exists())
        } else {
            let sub_rel = sub.replace('.', std::path::MAIN_SEPARATOR_STR);
            // PLAN-649 SD-01: literal sub-path candidates first, then kebab
            // variants of underscore segments (`bps.feedback.empty_state...`
            // reaches `feedback/empty-state/`). Layout priority (front/back/
            // src/flat) is preserved within each candidate form.
            for cand in module_path_candidates(&sub_rel) {
                let candidates = [
                    pkg_root
                        .join("src")
                        .join("front")
                        .join(format!("{}.at", cand)),
                    pkg_root
                        .join("src")
                        .join("front")
                        .join(&cand)
                        .join("mod.at"),
                    pkg_root
                        .join("src")
                        .join("back")
                        .join(format!("{}.at", cand)),
                    pkg_root.join("src").join("back").join(&cand).join("mod.at"),
                    pkg_root.join("src").join(format!("{}.at", cand)),
                    pkg_root.join("src").join(&cand).join("mod.at"),
                    pkg_root.join("front").join(format!("{}.at", cand)),
                    pkg_root.join("front").join(&cand).join("mod.at"),
                    pkg_root.join(format!("{}.at", cand)),
                    pkg_root.join(&cand).join("mod.at"),
                ];
                if let Some(hit) = candidates.into_iter().find(|c| c.exists()) {
                    return Some(hit);
                }
            }
            None
        }
    };
    let mut curr_dir = Some(base_dir);
    for _ in 0..4 {
        let Some(d) = curr_dir else { break };
        let pac_content = std::fs::read_to_string(d.join("pac.at")).ok();
        let dep_declared = pac_content
            .as_deref()
            .is_some_and(|c| pac_declares_dep(c, dep_name));
        let member_entry = pac_content
            .as_deref()
            .and_then(|c| pac_workspace_member_dir(c, dep_name));

        // PLAN-647 guardrail B: a version-ish key on this dep's declaration
        // fails the resolution outright (even when `path:` would resolve) —
        // resolving anyway would validate a fake constraint.
        if dep_declared {
            if let Some(pac) = &pac_content {
                if let Some(msg) = pac_dep_version_violation(pac, dep_name) {
                    eprintln!("error: {msg}");
                    return None;
                }
            }
        }

        let deps_candidate = d.join("deps");
        if deps_candidate.is_dir() {
            if dep_declared {
                if let Some(p) = probe_pkg(&deps_candidate.join(dep_name)) {
                    return Some(p);
                }
            } else if !dep_name.is_empty()
                && member_entry.is_none()
                && deps_candidate.join(dep_name).exists()
            {
                // PLAN-659 T-09：空名模块探测不触发幽灵依赖守卫——
                // deps/"" 恒为 deps/ 本体（exists 恒真），画廊 boot 期
                // 空名解析批次曾把该守卫刷成 "dependency ''" 噪音 ×N。
                eprintln!(
                    "error: dependency '{}' is materialized at {} but not declared in pac.at — declare it (`dep \"{}\" {{ path: ... }}`) or add it to workspace members",
                    dep_name,
                    deps_candidate.display(),
                    dep_name
                );
            }
        }

        // PLAN-635 D5: workspace members resolve like deps/<name>.
        if let Some(entry) = &member_entry {
            let member_root = d.join(entry);
            if let Some(p) = probe_pkg(&member_root) {
                return Some(p);
            }
        }

        // pac.at 中的本地 path 依赖（dep "<name>" { path: ... } 声明形态，
        // 天然过门控）。
        if let Some(content) = &pac_content {
            if let Some(pos) = content
                .find(&format!("dep \"{}\"", dep_name))
                .or_else(|| content.find(&format!("dep {}", dep_name)))
            {
                let slice = &content[pos..];
                if let Some(path_pos) = slice.find("path:") {
                    let path_slice = &slice[path_pos + 5..];
                    let path_line = path_slice
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches(|c| c == '"' || c == '\'');
                    if !path_line.is_empty() {
                        let local_dep_dir = d.join(path_line);
                        if local_dep_dir.is_dir() {
                            if let Some(p) = probe_pkg(&local_dep_dir) {
                                return Some(p);
                            }
                        }
                    }
                }
            }
        }
        curr_dir = d.parent();
    }

    None
}

/// Plan 442 A3 (ext link): load `use.web` ext imports for the VM render
/// target. Previously UseWeb was ignored entirely — pure-Auto helpers
/// imported via `use.web X from "…/port.at"` never got a definition, and any
/// handler calling them left an unresolved CALL reloc that killed the whole
/// VmBridge link ("Undefined symbol"). This function:
///   1. loads `.at` ext sources through the port-adapter chain
///      (X.at → X.vm.at → X.web.at) — their pure-Auto fns become real module
///      symbols, aliased to the adapter module's qualified name (mirroring
///      how scanner-resolved use imports alias their items);
///   2. synthesizes no-op platform stubs for remaining ext symbols (TS/npm
///      sources) with the arity observed at call sites — unless
///      `AUTO_VM_EXT_STUBS=0` restores the strict hard-link-error behavior.
/// Shared with the plan370/plan442 test support so tests exercise the
/// production logic.
pub fn read_auto_lib(project_root: &std::path::Path) -> AutoResult<String> {
    let mut lib_code = String::new();
    for file in AUTO_LIB_FILES {
        let path = project_root.join(file);
        if path.exists() {
            lib_code.push_str(&std::fs::read_to_string(&path)?);
            lib_code.push('\n');
        }
    }
    Ok(lib_code)
}

// PLAN-752: cached AAVM v1 bootstrap lib (AUTO_LIB_FILES full manifest) for the
// playground prepend_lib path. Mirrors the golden is_bootstrap concat
// (run_vm_file_test: `format!("{}\n{}", lib, src)`), single source of truth
// stays the pub(crate) manifest above.
static BOOTSTRAP_LIB: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

/// Concatenated `packages/aavm/lib-legacy/*.at` bootstrap lib (~188KB, dependency
/// order per AUTO_LIB_FILES). `None` when no manifest file resolves (caller
/// treats prepend_lib as unavailable). Result is cached after first call.
pub fn bootstrap_lib_source() -> Option<&'static str> {
    BOOTSTRAP_LIB
        .get_or_init(|| {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let project_root = root
                .parent() // crates/
                .and_then(|p| p.parent()) // repo root
                .unwrap_or(root);
            read_auto_lib(project_root).ok().filter(|s| !s.is_empty())
        })
        .as_deref()
}

/// Find the project root by walking up from a path, looking for `auto/` dir or `Cargo.toml`.
pub fn find_project_root(from: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut dir = from;
    loop {
        if dir.join("packages/aavm").is_dir() || dir.join("Cargo.toml").is_file() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Plan 262: Run a single VM file-based test case.
/// Returns a `FileTestReport` with pass/fail outcome.
fn extract_value_from_vm(
    vm: &crate::vm::engine::AutoVM,
    bits: i32,
    visited: &mut std::collections::HashSet<u64>,
) -> Value {
    if bits < 0 {
        // Tagged string index: indices are stored as -(index+1)
        let str_idx = (-bits - 1) as usize;
        let strings = vm.strings.read().unwrap();
        if let Some(str_bytes) = strings.get(str_idx) {
            return Value::Str(String::from_utf8_lossy(str_bytes).to_string().into());
        }
        return Value::Nil;
    }

    let id = bits as u64;

    if !visited.insert(id) {
        // Cycle detected
        return Value::Nil;
    }

    // 1. Check if it's an object ID (Plan 390 §15 H3b: ObjectData in
    // heap_objects).
    if let Some(obj_ref) = vm.get_heap_object(id) {
        let obj_guard = obj_ref.read().unwrap();
        if let Some(obj_data) = obj_guard
            .as_any()
            .downcast_ref::<crate::vm::types::ObjectData>()
        {
            let mut result_obj = Obj::new();
            for (key, val) in &obj_data.fields {
                let extracted = extract_auto_val_value(vm, val, visited);
                result_obj.set(key.clone(), extracted);
            }
            visited.remove(&id);
            return Value::Obj(Box::new(result_obj));
        }
    }

    // 2. Check if it's a node ID
    // Plan 390 §15 H3a: nodes live in heap_objects — resolve via
    // get_heap_object + downcast instead of the deleted `nodes` registry.
    if let Some(node_ref) = vm.get_heap_object(id) {
        let node_data = node_ref.read().unwrap();
        if let Some(node) = node_data.as_any().downcast_ref::<Node>() {
            let result = Value::node(extract_node_deep(vm, node, visited));
            visited.remove(&id);
            return result;
        }
    }

    // 3. Check if it's an array ID (Plan 390 §15 H3b: ListData<Value> in
    // heap_objects — same storage as CREATE_ARRAY).
    if let Some(list_ref) = vm.get_heap_object(id) {
        let array_data = list_ref.read().unwrap();
        if let Some(list) = array_data
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<Value>>()
        {
            let mut items = Vec::new();
            for val in list.elems.iter() {
                items.push(extract_auto_val_value(vm, val, visited));
            }
            visited.remove(&id);
            return Value::Array(Array::from_vec(items));
        }
    }

    visited.remove(&id);
    // Fallback to integer (for non-heap values)
    Value::Int(bits)
}

fn extract_auto_val_value(
    vm: &crate::vm::engine::AutoVM,
    val: &Value,
    visited: &mut std::collections::HashSet<u64>,
) -> Value {
    match val {
        Value::VmRef(vm_ref) => extract_value_from_vm(vm, vm_ref.id as i32, visited),
        Value::Int(bits) => {
            // Check if this is a tagged string index (negative value)
            if *bits < 0 {
                let str_idx = (-bits - 1) as usize;
                let strings = vm.strings.read().unwrap();
                if let Some(str_bytes) = strings.get(str_idx) {
                    return Value::Str(String::from_utf8_lossy(str_bytes).to_string().into());
                }
            }
            val.clone()
        }
        Value::Array(arr) => {
            let mut items = Vec::new();
            for v in &arr.values {
                items.push(extract_auto_val_value(vm, v, visited));
            }
            Value::Array(Array::from_vec(items))
        }
        Value::Obj(obj) => {
            let mut result_obj = Obj::new();
            for (key, val) in obj.iter() {
                result_obj.set(key.clone(), extract_auto_val_value(vm, val, visited));
            }
            Value::Obj(Box::new(result_obj))
        }
        Value::Node(node) => Value::node(extract_node_deep(vm, node, visited)),
        _ => val.clone(),
    }
}

fn extract_node_deep(
    vm: &crate::vm::engine::AutoVM,
    node: &Node,
    visited: &mut std::collections::HashSet<u64>,
) -> Node {
    let mut result = node.clone();
    // Resolve props
    let props = node.props_clone();
    for (key, val) in props.iter() {
        result.set_prop(key.clone(), extract_auto_val_value(vm, val, visited));
    }
    // TODO: Resolve args and kids if they contain VmRefs
    result
}

/// let result = eval_config_with_vm(config, &Obj::new()).unwrap();
/// ```
pub fn eval_config_with_vm(code: &str, args: &Obj) -> AutoResult<Value> {
    crate::install_ui_host_capabilities();
    use crate::parser::CompileDest;
    use crate::vm::codegen::Codegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::virt_memory::VirtualFlash;

    // Note: Plan 091 - Universe parameter removed, AutoVM uses its own state
    // Note: Do NOT preprocess macros here — pac.at is config code, not UI code.
    // The `app` keyword in pac.at means a node definition (app (id: "main") {...}),
    // not a UI macro (which would expand to `type ... is App {...}`).

    // 1. Parse the code
    let mut parser = Parser::from(code);
    // Plan 364: Config mode enables tolerant manifest parsing (bare-name nodes,
    // relaxed undefined checks) and context-variable injection (e.g. `port`).
    parser.compile_dest = CompileDest::Config;
    if let Some(port) = args.get_str("port") {
        parser.define_config_globals(port.as_str());
    }
    let ast = parser.parse()?;

    // 2. Compile to bytecode using the unified Codegen in config_mode.
    // Plan 364 Step 5: replaces the old compile-time-only ConfigCodegen with a
    // runtime-evaluating VM Config mode (accumulation opcodes + Script control
    // flow), so f-string templates, object field access, and for-over-array
    // work at runtime.
    //
    // Plan 013: inject `on`/`off` as built-in config globals (true/false) so
    // manifest toggles can read naturally, e.g. pac.at `shadcn: off`. Without
    // these, a bareword `off`/`on` value aborts the whole config eval with
    // "Undefined variable". Caller-supplied args win (only set when absent).
    let mut args_with_builtins = args.clone();
    if !args_with_builtins.has("on") {
        args_with_builtins.set("on", Value::Bool(true));
    }
    if !args_with_builtins.has("off") {
        args_with_builtins.set("off", Value::Bool(false));
    }
    let mut codegen = Codegen::new_for_config();
    codegen.compile_config_program(&ast, &args_with_builtins)?;

    // 3. Perform linking (resolve function calls)
    let strings = codegen.strings.clone();
    let exports = codegen.exports.clone();
    let relocs = codegen.relocs.clone();

    for reloc in &relocs {
        if let Some(&addr) = exports.get(&reloc.symbol_name) {
            let bytes = addr.to_le_bytes();
            let offset = reloc.offset as usize;
            for (i, b) in bytes.iter().enumerate() {
                codegen.code[offset + i] = *b;
            }
        } else {
            return Err(AutoError::Msg(format!(
                "Undefined symbol in config: {}",
                reloc.symbol_name
            )));
        }
    }

    // 4. Load into VM and execute
    let bytecode = codegen.code.clone();
    let object_keys = codegen.object_keys.clone();
    let object_types = codegen.object_types.clone();
    // Plan 197 Task 9: Extract generic registry for VM
    let generic_registry = std::mem::take(&mut codegen.generic_registry);
    // Plan 327 Phase 1: take task handler registry for VM
    let task_handler_registry = std::mem::take(&mut codegen.task_handler_registry);

    let rt = get_global_runtime();
    rt.block_on(async {
        let flash = VirtualFlash::new_with_code_and_keys(bytecode, object_keys, object_types);
        let mut vm = AutoVM::new(flash, 32768); // 32KB RAM for config (deeply nested manifests)
        vm.load_strings(strings);
        vm.load_generic_registry(generic_registry);
        vm.load_task_handler_registry(task_handler_registry); // Plan 327 Phase 1

        // 5. Execute from entry point (default to 0 for config)
        let entry_point = exports.get("main").copied().unwrap_or(0) as usize;

        let task_id = vm.spawn_task(entry_point, 32768); // 32KB task stack (deeply nested manifests)

        // Run the VM to completion
        vm.run_task_loop().await;

        // 6. Get the result from the VM stack.
        // Plan 364 Step 5: the result is the root Node (POP_ACCUM leaves its
        // id on the stack). No Obj->root conversion is needed anymore.
        if let Some(task_arc) = vm.tasks.get(&task_id).map(|r| r.value().clone()) {
            let mut task = task_arc.lock().await;

            if task.ram.sp == 0 {
                // No return value - return Nil
                return Ok(Value::Nil);
            }

            // Pop the result value from the stack
            let result_i32 = task.ram.pop_i32();

            // Materialize the result from the VM heap
            let mut visited = std::collections::HashSet::new();
            let materialized_result = extract_value_from_vm(&vm, result_i32, &mut visited);

            Ok(materialized_result)
        } else {
            // Task not found - return Nil
            Ok(Value::Nil)
        }
    })
}

// ---- PLAN-814：UI 宿主装配钩子（原 REPL 隐式扇入点之一，具名收口） ----
// 纯 SDK 无 UI 实现：钩子未注册时 no-op；上层 auto-lang 在装配点注册真实
// 安装器（实现留上层）。SDK 不依赖 inventory/linker 偶然性拉 UI。
static UI_HOST_INSTALL_HOOK: OnceLock<fn()> = OnceLock::new();

pub fn set_ui_host_install_hook(install: fn()) {
    let _ = UI_HOST_INSTALL_HOOK.set(install);
}

// ---- PLAN-841 T-05：中立外部标识符存在性检查端口 ----
// 领域自有符号族（如导入的 style recipe）不在 Core 词法/作用域内；其
// owner（auto-ui-compiler 的 recipe 注册表）注册类型化 `fn(&str) -> bool`
// 存在性检查，Core 解析器在 name-checker 末位咨询。未注册=零效果；不
// 载荷任何 UI 数据/类型进 Core（对照 PLAN-841 合同"中立扩展只承载具名
// 契约"）。替代已退役的 core 侧 design_tokens 直连查询。
static EXTERNAL_IDENT_CHECKER: OnceLock<fn(&str) -> bool> = OnceLock::new();

pub fn set_external_ident_checker(check: fn(&str) -> bool) {
    let _ = EXTERNAL_IDENT_CHECKER.set(check);
}

pub(crate) fn external_ident_checker() -> Option<fn(&str) -> bool> {
    EXTERNAL_IDENT_CHECKER.get().copied()
}

/// PLAN-814 R3-01/R3-04：上层兼容装配的进程期自注册收集点。语义边界
/// （对照 r2 §2 "不靠 linker inventory 偶然拉入初始化"）：SDK **不提交**
/// 任何安装器——纯 SDK 消费者零 submit、钩子缺席=显式拒绝/回退语义不
/// 变；上层 auto-lang 被链接即其 submit 生效（确定性自注册，覆盖不经
/// run 族漏斗的直达调用——native dispatch/旧公开入口）。
pub struct UiCompatInstaller(pub fn());

pub fn install_ui_host_capabilities() {
    if let Some(install) = UI_HOST_INSTALL_HOOK.get() {
        install();
        return;
    }
    // 钩子未注册：尝试上层自注册安装器（无 submit 时为空——纯 SDK 形态）。
    for installer in inventory::iter::<UiCompatInstaller> {
        (installer.0)();
    }
}

// ---- PLAN-814：i18n lookup 钩子（原 shim_i18n_t 直连 crate::ui 的残边收口） ----
// 纯 SDK：未注册钩子时返回 None（shim 回退原 key）。上层 auto-lang 在装配点
// 注册 ui::i18n_lookup::lookup，行为与旧 ui-iced 门内实现一致。
static I18N_LOOKUP_HOOK: OnceLock<fn(&str) -> Option<String>> = OnceLock::new();

pub fn set_i18n_lookup_hook(lookup: fn(&str) -> Option<String>) {
    let _ = I18N_LOOKUP_HOOK.set(lookup);
}

pub fn i18n_lookup(key: &str) -> Option<String> {
    I18N_LOOKUP_HOOK.get().and_then(|f| f(key))
}

pub fn store_name_snake(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                let next_lower = chars.get(i + 1).map_or(false, |n| n.is_ascii_lowercase());
                if prev.is_ascii_lowercase() || prev.is_ascii_digit() || next_lower {
                    out.push('_');
                }
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

// ---- T-02 终批根项（runners/FFI init/错误定位/remap/结果提取）----
/// Plan 262: Run a single VM file-based test case.
/// Returns a `FileTestReport` with pass/fail outcome.
pub fn run_vm_file_test(case: &test_runner::FileTestCase) -> test_runner::FileTestReport {
    #[cfg(feature = "test-cases")]
    let trace = crate::test_cases::execution_trace::Invocation::start(|| {
        let oracles = [
            case.expected_out.as_deref(),
            case.expected_result.as_deref(),
            case.expected_error.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        serde_json::json!({"family":"legacy-vm","id":case.name,
            "source":crate::test_cases::execution_trace::file_identity(&case.source_file),
            "oracles":oracles.into_iter().map(crate::test_cases::execution_trace::file_identity).collect::<Vec<_>>()})
    });
    let report = run_vm_file_test_impl(case);
    #[cfg(feature = "test-cases")]
    if let Some(trace) = trace {
        trace.finish(serde_json::json!({"status":if matches!(&report.outcome,test_runner::TestOutcome::Passed){"PASS"}else{"FAIL"},
            "outcome":format!("{:?}",report.outcome),"duration_ms":report.duration_ms}));
    }
    report
}

fn run_vm_file_test_impl(case: &test_runner::FileTestCase) -> test_runner::FileTestReport {
    use std::time::Instant;

    let start = Instant::now();
    let case_name = case.name.clone();

    // Read source file
    let src = match std::fs::read_to_string(&case.source_file) {
        Ok(s) => s,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("failed to read source: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Prepend auto/lib for bootstrap tests
    let src = if case.is_bootstrap {
        let project_root = match find_project_root(&case.dir) {
            Some(r) => r,
            None => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(
                        "cannot find project root for bootstrap test".into(),
                    ),
                    duration_ms: start.elapsed().as_millis(),
                    stdout: String::new(),
                };
            }
        };
        match read_auto_lib(&project_root) {
            Ok(lib) => format!("{}\n{}", lib, src),
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "failed to read auto lib: {}",
                        e
                    )),
                    duration_ms: start.elapsed().as_millis(),
                    stdout: String::new(),
                };
            }
        }
    } else {
        src
    };

    // Error expectation test
    if case.expected_error.is_some() {
        let result = run(&src);
        let duration_ms = start.elapsed().as_millis();
        return match result {
            Err(_) => test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Passed,
                duration_ms,
                stdout: String::new(),
            },
            Ok(v) => test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!(
                    "expected error but got: {:?}",
                    v
                )),
                duration_ms,
                stdout: String::new(),
            },
        };
    }

    // Normal execution with capture
    // Plan 560 T14：文件测试走带 path 通道（硬化门控对 .at 文件生效、
    // .as 豁免——内联 run_with_capture 不门控，见 execute 段注记）。
    let src_path_str = case.source_file.to_string_lossy().to_string();
    let (result, stdout) = match run_with_capture_and_path(&src, &src_path_str) {
        Ok(r) => r,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("execution error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let duration_ms = start.elapsed().as_millis();

    // Check stdout against .expected.out
    if let Some(ref expected_path) = case.expected_out {
        let expected = match std::fs::read_to_string(expected_path) {
            Ok(e) => e,
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "failed to read expected.out: {}",
                        e
                    )),
                    duration_ms,
                    stdout: stdout.clone(),
                };
            }
        };
        if stdout != expected {
            let wrong_path = expected_path.with_extension("wrong.out");
            let _ = std::fs::write(&wrong_path, &stdout);
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed("stdout mismatch".into()),
                duration_ms,
                stdout,
            };
        }
    }

    // Check return value against .expected.result
    if let Some(ref expected_path) = case.expected_result {
        let expected = match std::fs::read_to_string(expected_path) {
            Ok(e) => e,
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "failed to read expected.result: {}",
                        e
                    )),
                    duration_ms,
                    stdout: stdout.clone(),
                };
            }
        };
        if result != expected {
            let wrong_path = expected_path.with_extension("wrong.result");
            let _ = std::fs::write(&wrong_path, &result);
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed("result mismatch".into()),
                duration_ms,
                stdout,
            };
        }
    }

    test_runner::FileTestReport {
        name: case_name,
        outcome: test_runner::TestOutcome::Passed,
        duration_ms,
        stdout,
    }
}

/// Plan 263 Phase 3: Run a single a2ts transpiler file-based test case.
/// Reads .at source, transpiles to TypeScript, compares against .expected.ts.
pub fn run_a2ts_file_test(case: &test_runner::A2tsTestCase) -> test_runner::FileTestReport {
    #[cfg(feature = "test-cases")]
    let trace = crate::test_cases::execution_trace::Invocation::start(|| {
        let oracles = vec![case.expected_file.as_path()];
        serde_json::json!({"family":"legacy-a2ts","id":case.name,
            "source":crate::test_cases::execution_trace::file_identity(&case.source_file),
            "oracles":oracles.into_iter().map(crate::test_cases::execution_trace::file_identity).collect::<Vec<_>>()})
    });
    let report = run_a2ts_file_test_impl(case);
    #[cfg(feature = "test-cases")]
    if let Some(trace) = trace {
        trace.finish(serde_json::json!({"status":if matches!(&report.outcome,test_runner::TestOutcome::Passed){"PASS"}else{"FAIL"},
            "outcome":format!("{:?}",report.outcome),"duration_ms":report.duration_ms}));
    }
    report
}

fn run_a2ts_file_test_impl(case: &test_runner::A2tsTestCase) -> test_runner::FileTestReport {
    use std::time::Instant;

    let start = Instant::now();
    let case_name = case.name.clone();

    let stem = case
        .source_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("test");

    // Read source
    let src = match std::fs::read_to_string(&case.source_file) {
        Ok(s) => s,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("failed to read source: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Parse and transpile
    let _scope = crate::scope_manager::ScopeManager::new();
    let mut parser = crate::parser::Parser::from(src.as_str());
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("parse error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let mut sink = crate::trans::Sink::new(stem.into());
    let mut trans = crate::trans::typescript::TypeScriptTrans::new(stem.into());
    let ts_code = match trans.trans(ast, &mut sink) {
        Ok(_) => match sink.done() {
            Ok(bytes) => bytes.to_vec(),
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "sink finalize error: {}",
                        e
                    )),
                    duration_ms: start.elapsed().as_millis(),
                    stdout: String::new(),
                };
            }
        },
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("transpiler error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Read expected
    let expected = match std::fs::read(&case.expected_file) {
        Ok(b) => b,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!(
                    "failed to read expected.ts: {}",
                    e
                )),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let duration_ms = start.elapsed().as_millis();

    if ts_code != expected {
        let wrong_path = case.expected_file.with_extension("wrong.ts");
        let _ = std::fs::write(&wrong_path, &ts_code);
        test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Failed("transpiler output mismatch".into()),
            duration_ms,
            stdout: String::from_utf8_lossy(&ts_code).to_string(),
        }
    } else {
        test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Passed,
            duration_ms,
            stdout: String::new(),
        }
    }
}

/// Plan 263 Phase 3: Run a single a2c transpiler file-based test case.
/// Reads .at source, transpiles to C, compares against .expected.c and .expected.h.
pub fn run_a2c_file_test(case: &test_runner::A2cTestCase) -> test_runner::FileTestReport {
    #[cfg(feature = "test-cases")]
    let trace = crate::test_cases::execution_trace::Invocation::start(|| {
        let oracles = [
            Some(case.expected_c.as_path()),
            Some(case.expected_h.as_path()),
            case.expected_error.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        serde_json::json!({"family":"legacy-a2c","id":case.name,
            "source":crate::test_cases::execution_trace::file_identity(&case.source_file),
            "oracles":oracles.into_iter().map(crate::test_cases::execution_trace::file_identity).collect::<Vec<_>>()})
    });
    let report = run_a2c_file_test_impl(case);
    #[cfg(feature = "test-cases")]
    if let Some(trace) = trace {
        trace.finish(serde_json::json!({"status":if matches!(&report.outcome,test_runner::TestOutcome::Passed){"PASS"}else{"FAIL"},
            "outcome":format!("{:?}",report.outcome),"duration_ms":report.duration_ms}));
    }
    report
}

fn run_a2c_file_test_impl(case: &test_runner::A2cTestCase) -> test_runner::FileTestReport {
    use std::time::Instant;

    let start = Instant::now();
    let case_name = case.name.clone();

    let stem = case
        .source_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("test");

    // Read source
    let src = match std::fs::read_to_string(&case.source_file) {
        Ok(s) => s,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("failed to read source: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Check if this is an error test
    if case.expected_error.is_some() {
        let result = crate::trans::c::transpile_c(stem, &src);
        let duration_ms = start.elapsed().as_millis();
        return match result {
            Err(_) => test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Passed,
                duration_ms,
                stdout: String::new(),
            },
            Ok(_) => test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(
                    "expected error but transpilation succeeded".into(),
                ),
                duration_ms,
                stdout: String::new(),
            },
        };
    }

    // Normal transpile
    let mut ccode = match crate::trans::c::transpile_c(stem, &src) {
        Ok(c) => c,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("transpiler error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let actual_src = match ccode.done() {
        Ok(bytes) => bytes.to_vec(),
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!(
                    "transpiler finalize error: {}",
                    e
                )),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };
    let actual_header = ccode.header;

    let duration_ms = start.elapsed().as_millis();

    // Compare .c output
    let expected_c = std::fs::read(&case.expected_c).unwrap_or_default();
    if actual_src != expected_c {
        let wrong_path = case.expected_c.with_extension("wrong.c");
        let _ = std::fs::write(&wrong_path, &actual_src);
        return test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Failed("C source mismatch".into()),
            duration_ms,
            stdout: String::from_utf8_lossy(&actual_src).to_string(),
        };
    }

    // Compare .h output
    let expected_h = std::fs::read(&case.expected_h).unwrap_or_default();
    if actual_header != expected_h {
        let wrong_path = case.expected_h.with_extension("wrong.h");
        let _ = std::fs::write(&wrong_path, &actual_header);
        return test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Failed("C header mismatch".into()),
            duration_ms,
            stdout: String::from_utf8_lossy(&actual_header).to_string(),
        };
    }

    test_runner::FileTestReport {
        name: case_name,
        outcome: test_runner::TestOutcome::Passed,
        duration_ms,
        stdout: String::new(),
    }
}

/// Plan 263: Run a single a2r transpiler file-based test case.
/// Reads .at source, transpiles to Rust, compares against .expected.rs.
pub fn run_a2r_file_test(case: &test_runner::A2rTestCase) -> test_runner::FileTestReport {
    #[cfg(feature = "test-cases")]
    let trace = crate::test_cases::execution_trace::Invocation::start(|| {
        let oracles = vec![case.expected_file.as_path()];
        serde_json::json!({"family":"legacy-a2r","id":case.name,
            "source":crate::test_cases::execution_trace::file_identity(&case.source_file),
            "oracles":oracles.into_iter().map(crate::test_cases::execution_trace::file_identity).collect::<Vec<_>>()})
    });
    let report = run_a2r_file_test_impl(case);
    #[cfg(feature = "test-cases")]
    if let Some(trace) = trace {
        trace.finish(serde_json::json!({"status":if matches!(&report.outcome,test_runner::TestOutcome::Passed){"PASS"}else{"FAIL"},
            "outcome":format!("{:?}",report.outcome),"duration_ms":report.duration_ms}));
    }
    report
}

fn run_a2r_file_test_impl(case: &test_runner::A2rTestCase) -> test_runner::FileTestReport {
    use std::time::Instant;

    let start = Instant::now();
    let case_name = case.name.clone();

    // Extract stem from source file name for transpile_rust
    let stem = case
        .source_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("test");

    // Read source
    let src = match std::fs::read_to_string(&case.source_file) {
        Ok(s) => s,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("failed to read source: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Transpile
    // Plan 610 ⑥: relative use.c JSON manifests resolve against the case dir.
    let actual = match crate::trans::rust::transpile_rust_with_source_dir(
        case.source_file
            .parent()
            .unwrap_or(std::path::Path::new(".")),
        stem,
        &src,
    ) {
        Ok(mut rcode) => match rcode.done() {
            Ok(bytes) => bytes.to_vec(),
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "transpiler finalize error: {}",
                        e
                    )),
                    duration_ms: start.elapsed().as_millis(),
                    stdout: String::new(),
                };
            }
        },
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("transpiler error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Read expected
    let expected = match std::fs::read(&case.expected_file) {
        Ok(b) => b,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!(
                    "failed to read expected.rs: {}",
                    e
                )),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let duration_ms = start.elapsed().as_millis();

    if actual != expected {
        let wrong_path = case.expected_file.with_extension("wrong.rs");
        let _ = std::fs::write(&wrong_path, &actual);
        test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Failed("transpiler output mismatch".into()),
            duration_ms,
            stdout: String::from_utf8_lossy(&actual).to_string(),
        }
    } else {
        test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Passed,
            duration_ms,
            stdout: String::new(),
        }
    }
}

/// Plan 359 B1: Remap CREATE_OBJ `key_index` (u16) operands in bytecode after
/// the object_keys/object_types pools are merged across modules.
///
/// When dep modules' object pools are appended to the main module's pool, every
/// CREATE_OBJ instruction in a dep module still holds its original 0-based
/// `key_index`, which now points into the MAIN module's portion of the merged
/// pool (wrong keys/types → corrupt objects, or field_count/types length
/// mismatches that overflow at runtime). This walker decodes instruction
/// lengths precisely (mirroring `remap_string_indices`) and rewrites the u16
/// operand of each CREATE_OBJ using `obj_remap` (old_index → new_index).
pub(crate) fn remap_obj_indices(code: &mut Vec<u8>, obj_remap: &[u32]) {
    use crate::vm::opcode::OpCode;
    if obj_remap.is_empty() {
        return;
    }
    let mut last_const_i32: i32 = 0;
    let mut i = 0;
    while i < code.len() {
        let op_byte = code[i];
        if !OpCode::is_valid(op_byte) {
            i += 1;
            continue;
        }
        let op = OpCode::from(op_byte);
        let operand_pos = i + 1;
        if let OpCode::CONST_I32 = op {
            if operand_pos + 4 <= code.len() {
                last_const_i32 = i32::from_le_bytes([
                    code[operand_pos],
                    code[operand_pos + 1],
                    code[operand_pos + 2],
                    code[operand_pos + 3],
                ]);
            }
        }
        let (operand_len, idxs) =
            operand_span_with_pool_indices(op, code, operand_pos, last_const_i32);
        for (off, space) in idxs {
            if space == IdxSpace::Obj {
                let pos = operand_pos + off;
                if pos + 4 <= code.len() {
                    let old = u32::from_le_bytes([
                        code[pos],
                        code[pos + 1],
                        code[pos + 2],
                        code[pos + 3],
                    ]) as usize;
                    if old < obj_remap.len() {
                        let bytes = obj_remap[old].to_le_bytes();
                        code[pos] = bytes[0];
                        code[pos + 1] = bytes[1];
                        code[pos + 2] = bytes[2];
                        code[pos + 3] = bytes[3];
                    }
                }
            }
        }
        i += 1 + operand_len;
    }
}

/// Plan 355: dep 模块字符串池并入主池后,重写字节码中的池索引。
/// remap 为 u32(池 u32 化);跨度模型与 remap_obj_indices 共享。
/// PLAN-814 R4/R3-05：随 VM 迁入 SDK 的 codegen 工具（原 auto-lang
/// pub(crate)）；aavm 上层测试（aavm2_m4）跨 crate 引用需要 pub 面。
pub fn remap_string_indices(code: &mut Vec<u8>, remap: &[u32]) {
    use crate::vm::opcode::OpCode;
    let mut last_const_i32: i32 = 0;
    let mut i = 0;
    while i < code.len() {
        let op_byte = code[i];
        if !OpCode::is_valid(op_byte) {
            i += 1;
            continue;
        }
        let op = OpCode::from(op_byte);
        let operand_pos = i + 1;
        if let OpCode::CONST_I32 = op {
            if operand_pos + 4 <= code.len() {
                last_const_i32 = i32::from_le_bytes([
                    code[operand_pos],
                    code[operand_pos + 1],
                    code[operand_pos + 2],
                    code[operand_pos + 3],
                ]);
            }
        }
        let (operand_len, idxs) =
            operand_span_with_pool_indices(op, code, operand_pos, last_const_i32);
        for (off, space) in idxs {
            if space == IdxSpace::Str {
                let pos = operand_pos + off;
                if pos + 4 <= code.len() {
                    let raw = u32::from_le_bytes([
                        code[pos],
                        code[pos + 1],
                        code[pos + 2],
                        code[pos + 3],
                    ]);
                    // 0xFFFF "无 id" 哨兵不重映射(PUSH_ACCUM/CREATE_NODE
                    // 的可选 id 位;吸收自 417-followup 的 apply_u32_opt)。
                    if raw == 0xFFFF {
                        continue;
                    }
                    let old = raw as usize;
                    if old < remap.len() {
                        let bytes = remap[old].to_le_bytes();
                        code[pos] = bytes[0];
                        code[pos + 1] = bytes[1];
                        code[pos + 2] = bytes[2];
                        code[pos + 3] = bytes[3];
                    }
                }
            }
        }
        i += 1 + operand_len;
    }
}

#[cfg(not(feature = "python"))]
pub fn init_py_ffi(
    _session: &compile::CompileSession,
) -> Option<crate::vm::native::NativeInterface> {
    None
}

/// Plan 214/300: Initialize Python FFI bridge if there are use.py imports
///
/// Creates a PyFfiBridge, imports the requested Python modules,
/// registers each function as a native shim with auto-type marshalling,
/// and returns the NativeInterface for merging into the VM.
///
/// Plan 300: Uses Python `inspect.signature()` to detect param count,
/// then registers with `PySignature::all_auto(count)` for runtime
/// NanoValue tag-based type detection instead of hardcoded string→string.
#[cfg(feature = "python")]
pub fn init_py_ffi(
    session: &compile::CompileSession,
) -> Option<crate::vm::native::NativeInterface> {
    let py_imports = session.py_imports();
    if py_imports.is_empty() {
        return None;
    }

    let mut bridge = match crate::py_ffi::PyFfiBridge::new() {
        Ok(b) => b,
        Err(e) => {
            log::warn!("Failed to create PyFfiBridge: {:?}", e);
            return None;
        }
    };

    // Import each Python module and register its functions
    for (module_name, functions) in py_imports {
        if let Err(e) = bridge.import_module(module_name) {
            log::warn!("Failed to import Python module '{}': {:?}", module_name, e);
            continue;
        }

        // Plan 300: For bare module imports (empty items), discover callables via dir()
        if functions.is_empty() {
            let discovered = bridge.discover_module_callables(module_name);
            for (func_name, param_count) in discovered {
                // Plan 567 T17: 裸模块发现面同法灌注定预言机知识。
                let ret_anno = bridge.inspect_return_annotation(module_name, &func_name);
                let sig = {
                    let mut sig = crate::py_ffi_types::PySignature::all_auto(param_count);
                    if let Some(t) = &ret_anno {
                        sig.returns = t.clone();
                    }
                    sig
                };
                if let Some(t) = &ret_anno {
                    crate::py_ffi_types::record_return_annotation(&func_name, t.clone());
                }
                if let Ok(native_id) = bridge.register_function(module_name, &func_name, sig) {
                    // Register with module-qualified name so codegen can find it
                    let qualified = format!("py.{}.{}", module_name, func_name);
                    if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
                        registry.register_with_id(&qualified, native_id);
                    }
                }
            }
            continue;
        }

        // Named imports — use inspect to get param count for callables; register
        // non-callable attributes (e.g. math.pi) as zero-arg constants (Plan 369 Task 11).
        for func_name in functions {
            // Plan 369 Task 11: constants are non-callable module attributes.
            if !bridge.is_callable(module_name, &func_name) {
                // Plan 567 T17: 常量通道注解（模块级 `x: float = ...`）。
                if let Some(t) = bridge.inspect_constant_annotation(module_name, &func_name) {
                    crate::py_ffi_types::record_return_annotation(&func_name, t);
                }
                match bridge.register_constant(module_name, &func_name) {
                    Ok(native_id) => {
                        log::info!(
                            "Registered Python const: {}.{} (native_id={})",
                            module_name,
                            func_name,
                            native_id
                        );
                        let qualified = format!("py.{}", func_name);
                        if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
                            registry.register_with_id(&qualified, native_id);
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to register Python const {}.{}: {:?}",
                            module_name,
                            func_name,
                            e
                        );
                    }
                }
                continue;
            }
            let param_count = bridge.inspect_param_count(module_name, &func_name, 1);
            // Plan 567 T17（W3 注解预言机）: 读返回注解——有承诺则签名带
            // 类型（shim 出口 D4 强制），并登记全局知识表（codegen 消费：
            // py_return_types 灌注 + nullable lint）。无注解 = Auto 零变化。
            let ret_anno = bridge.inspect_return_annotation(module_name, &func_name);
            let sig = {
                let mut sig = crate::py_ffi_types::PySignature::all_auto(param_count);
                if let Some(t) = &ret_anno {
                    sig.returns = t.clone();
                }
                sig
            };
            if let Some(t) = &ret_anno {
                crate::py_ffi_types::record_return_annotation(&func_name, t.clone());
                log::info!(
                    "PyOracle: {}.{} -> {:?} (annotated)",
                    module_name,
                    func_name,
                    t
                );
            }
            match bridge.register_function(module_name, func_name, sig) {
                Ok(native_id) => {
                    log::info!(
                        "Registered Python FFI: {}.{} (native_id={}, params={})",
                        module_name,
                        func_name,
                        native_id,
                        param_count
                    );

                    // Also register in BIGVM_NATIVES so codegen can find it
                    let qualified = format!("py.{}", func_name);
                    if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
                        registry.register_with_id(&qualified, native_id);
                    }
                }
                Err(e) => {
                    log::warn!(
                        "Failed to register Python function {}.{}: {:?}",
                        module_name,
                        func_name,
                        e
                    );
                }
            }
        }
    }

    // Plan 369 Task 12: register py_call / py_getattr built-ins so users can
    // dispatch method calls and attribute access on opaque Python objects
    // returned by earlier py-FFI calls (e.g. datetime.date.isoformat()).
    bridge.register_object_shims();
    if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
        registry.register_with_id("py.py_call", crate::py_ffi::NATIVE_PY_CALL);
        registry.register_with_id("py.py_getattr", crate::py_ffi::NATIVE_PY_GETATTR);
        // Plan 539 W0 (DIV-PY-KWARGS-1): keyword-argument method-call channel.
        registry.register_with_id("py.py_call_kw", crate::py_ffi::NATIVE_PY_CALL_KW);
        // Plan 539 W0 (DIV-PY-EXCEPT-1): May-valued method-call channel.
        registry.register_with_id("py.py_call_may", crate::py_ffi::NATIVE_PY_CALL_MAY);
        // Plan 539 W0 (DIV-PY-ITER-1): iteration protocol pair.
        registry.register_with_id("py.py_iter", crate::py_ffi::NATIVE_PY_ITER);
        registry.register_with_id("py.py_next", crate::py_ffi::NATIVE_PY_NEXT);
        // Plan 539 W1 (T11-T14): inference idiom builtins.
        registry.register_with_id("py.py_matmul", crate::py_ffi::NATIVE_PY_MATMUL);
        registry.register_with_id("py.py_getitem", crate::py_ffi::NATIVE_PY_GETITEM);
        registry.register_with_id("py.py_setitem", crate::py_ffi::NATIVE_PY_SETITEM);
        registry.register_with_id("py.py_slice", crate::py_ffi::NATIVE_PY_SLICE);
        registry.register_with_id("py.py_call0", crate::py_ffi::NATIVE_PY_CALL0);
        registry.register_with_id("py.py_with", crate::py_ffi::NATIVE_PY_WITH);
        registry.register_with_id("py.py_enter", crate::py_ffi::NATIVE_PY_ENTER);
        registry.register_with_id("py.py_exit", crate::py_ffi::NATIVE_PY_EXIT);
        registry.register_with_id("py.py_item_kw", crate::py_ffi::NATIVE_PY_ITEM_KW);
        registry.register_with_id("py.py_float", crate::py_ffi::NATIVE_PY_FLOAT);
        // Plan 539 W3 (T21): Auto closure as Python callable.
        registry.register_with_id("py.py_callable", crate::py_ffi::NATIVE_PY_CALLABLE);
        // Plan 555 T04: 分发组合子配套三桥（B2 桥半/B6/D8）。
        registry.register_with_id("py.py_setattr", crate::py_ffi::NATIVE_PY_SETATTR);
        registry.register_with_id("py.py_len", crate::py_ffi::NATIVE_PY_LEN);
        registry.register_with_id("py.py_type_name", crate::py_ffi::NATIVE_PY_TYPE_NAME);
        // Plan 560 T04: B7 contains + B8 裸模块句柄。
        registry.register_with_id("py.py_contains", crate::py_ffi::NATIVE_PY_CONTAINS);
        registry.register_with_id("py.py_module", crate::py_ffi::NATIVE_PY_MODULE);
        // Plan 560 T06 (D7): GIL str()。
        registry.register_with_id("py.py_str", crate::py_ffi::NATIVE_PY_STR);
        // Plan 560 T07 (C6)：幂。
        registry.register_with_id("py.py_pow", crate::py_ffi::NATIVE_PY_POW);
        // Plan 560 T08 (C3/C4/C7)。
        registry.register_with_id("py.py_truthy", crate::py_ffi::NATIVE_PY_TRUTHY);
        registry.register_with_id("py.py_is", crate::py_ffi::NATIVE_PY_IS);
        // Plan 567 T06 (P560-D2): may 值通道变体。
        registry.register_with_id("py.py_getattr_may", crate::py_ffi::NATIVE_PY_GETATTR_MAY);
        registry.register_with_id("py.py_getitem_may", crate::py_ffi::NATIVE_PY_GETITEM_MAY);
        // Plan 567 T07 (P539-D5): kwargs×may 组合。
        registry.register_with_id("py.py_call_kw_may", crate::py_ffi::NATIVE_PY_CALL_KW_MAY);
        // Plan 567 T12 (P560-D1): with-as catch 臂再抛通道。
        registry.register_with_id("py.py_raise", crate::py_ffi::NATIVE_PY_RAISE);
        // Plan 567 T18 (W3 D4): GIL int() 显式标量提取。
        registry.register_with_id("py.py_int", crate::py_ffi::NATIVE_PY_INT);
        // Plan 602 (D2): Python 类派生工厂（exec 模板 + 回调方法挂载）。
        registry.register_with_id("py.py_subclass", crate::py_ffi::NATIVE_PY_SUBCLASS);
    }

    let mut native_interface = crate::vm::native::NativeInterface::new();
    native_interface.merge(bridge.native_interface());

    Some(native_interface)
}

/// Plan 212b Task 4: Initialize Rust FFI bridge at runtime
///
/// Scans the CompileSession for Rust dependencies and imports,
/// loads compiled libraries, and registers functions as native shims.
///
/// Returns a NativeInterface with registered Rust FFI shims, or None
/// if there are no Rust imports.
pub fn init_rust_ffi(
    session: &compile::CompileSession,
) -> Option<crate::vm::native::NativeInterface> {
    let rust_imports = session.rust_imports();
    if rust_imports.is_empty() {
        return None;
    }

    let mut native_interface = crate::vm::native::NativeInterface::new();

    // Create a RustFfiBridge to load and register Rust functions
    let mut bridge = match crate::ffi::RustFfiBridge::new() {
        Ok(b) => b,
        Err(e) => {
            log::warn!("Failed to create RustFfiBridge: {:?}", e);
            return None;
        }
    };

    // Load compiled libraries for each crate
    for (crate_name, functions) in rust_imports {
        // PLAN-592: 过滤类型导入(首字母大写)——类型走方法 shim 包(dispatch 3000),
        // 不进自由函数 wrapper;不过滤会以计数错配 wrapper 版本键(v3_{count}),
        // 并对 auto_<Type> 符号产生必败的 GetProcAddress 告警。同时去重:
        // collect_rust_imports/resolve_uses 两阶段可能重复登记同一导入名,
        // 计数漂移会使版本键探错 wrapper。
        let mut seen_fns = std::collections::HashSet::new();
        let free_fns: Vec<&String> = functions
            .iter()
            .filter(|f| !f.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
            .filter(|f| seen_fns.insert((*f).clone()))
            .collect();
        // The wrapper library name follows the sandbox naming convention
        let wrapper_name = format!("{}_wrapper", crate_name.replace('-', "_"));

        // Try to load the compiled wrapper library
        if let Some(sandbox) = session.sandbox() {
            // Plan 430 C2: 方法 shim 包(独立于自由函数 wrapper;early-continue 之前加载)
            if let Some((methods_lib, _manifest_json)) = sandbox.find_methods_pack(&crate_name) {
                match unsafe { libloading::Library::new(&methods_lib) } {
                    Ok(lib) => {
                        let lib = std::sync::Arc::new(lib);
                        match crate::vm::ffi::dep_methods::read_shim_manifest(&lib) {
                            Ok(manifest) => {
                                crate::vm::ffi::dep_methods::register_pack(
                                    &crate_name,
                                    lib,
                                    &manifest,
                                );
                            }
                            Err(e) => {
                                log::warn!(
                                    "plan430: shim manifest read failed for {}: {:?}",
                                    crate_name,
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "plan430: methods pack load failed for {} from {}: {}",
                            crate_name,
                            methods_lib.display(),
                            e
                        );
                    }
                }
            }

            // Try v3 cache first (Phase 3C-v2 with sig_code), then v2, then v1。
            // PLAN-592 修复:版本键两侧统一为**导入自由函数数**。
            // PLAN-596 T4 再修:mono 实例使 compile 侧 shim 数(基名+实例)与本侧
            // 导入名数恒差——计数键失效。改为**覆盖校验扫描**:v3_* 候选按
            // mtime 降序逐个装载,读 sig manifest 校验"每个导入名有裸条目或
            // __<label> 实例条目",全覆盖才采用(592 的错变体陷阱由覆盖校验
            // 排除);无覆盖候选再落 v2/v1 旧键。
            let sig_hash_len = free_fns.len();
            let v2_path = sandbox.crate_library_path(&wrapper_name, &format!("v2_{sig_hash_len}"));
            let v1_path = sandbox.crate_library_path(&wrapper_name, "1");
            let lib_ext = if cfg!(target_os = "windows") {
                ".dll"
            } else if cfg!(target_os = "macos") {
                ".dylib"
            } else {
                ".so"
            };
            let v3_prefix = format!("{}-v3_", wrapper_name.replace('-', "_"));
            let mut v3_candidates: Vec<(std::path::PathBuf, std::time::SystemTime)> = Vec::new();
            if let Ok(rd) = std::fs::read_dir(sandbox.crates_path()) {
                for entry in rd.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with(&v3_prefix) && name.ends_with(lib_ext) {
                        let mtime = entry
                            .metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        v3_candidates.push((entry.path(), mtime));
                    }
                }
            }
            v3_candidates.sort_by(|a, b| b.1.cmp(&a.1)); // 新→旧

            // 旁路校验:独立 libloading 读候选的 auto__sig_manifest(不进
            // bridge——bridge 对同名 crate 只允许一次装载,先载后验会卡死
            // 扫描;胜者确定后 bridge 一次性装载)。读后立即卸载候选探针。
            let mut lib_path: Option<(std::path::PathBuf, u8)> = None;
            let mut manifest: Option<String> = None;
            'scan: for (cand, _) in &v3_candidates {
                let Ok(probe) = (unsafe { libloading::Library::new(cand) }) else {
                    continue;
                };
                let mjson = (|| -> Option<String> {
                    unsafe {
                        let sym: Result<
                            libloading::Symbol<
                                unsafe extern "C" fn() -> *const std::os::raw::c_char,
                            >,
                            _,
                        > = probe.get(b"auto__sig_manifest");
                        let f = sym.ok()?;
                        let p = f();
                        if p.is_null() {
                            return None;
                        }
                        let c = std::ffi::CStr::from_ptr(p);
                        Some(c.to_string_lossy().into_owned())
                    }
                })();
                drop(probe);
                let Some(mjson) = mjson else { continue };
                let sig_map: std::collections::HashMap<String, String> =
                    crate::ffi::parse_manifest_json(&mjson)
                        .into_iter()
                        .collect();
                let covered = free_fns.iter().all(|f| {
                    sig_map.contains_key(f.as_str())
                        || sig_map.keys().any(|k| k.starts_with(&format!("{}__", f)))
                });
                if covered {
                    lib_path = Some((cand.clone(), 3));
                    manifest = Some(mjson);
                    break 'scan;
                }
            }
            let (lib_path, cache_gen) = match lib_path {
                Some(p) => p,
                None => {
                    // 无覆盖的 v3 候选:退回 592 的精确计数键与 v2/v1(非 mono 场景)
                    let v3_path =
                        sandbox.crate_library_path(&wrapper_name, &format!("v3_{sig_hash_len}"));
                    if v3_path.exists() {
                        (v3_path, 3)
                    } else if v2_path.exists() {
                        (v2_path, 2)
                    } else if v1_path.exists() {
                        (v1_path, 1)
                    } else {
                        log::info!(
                            "Wrapper library not found for {} (no covering v3_* candidate; tried v3_{}/v2/v1)",
                            crate_name,
                            sig_hash_len
                        );
                        continue;
                    }
                }
            };

            // 胜者(或 fallback 精确键)交 bridge 一次性装载——旁路探针不进
            // bridge,此处无论 manifest 是否已取到都必须装载。
            if let Err(e) = bridge.load_rust_library(&crate_name, &lib_path) {
                log::warn!(
                    "Failed to load Rust library {} from {}: {:?}",
                    crate_name,
                    lib_path.display(),
                    e
                );
                continue;
            }
            if cache_gen >= 3 && manifest.is_none() {
                manifest = bridge.load_sig_manifest(&crate_name);
            }
            let manifest = if cache_gen >= 3 { manifest } else { None };

            if let Some(ref manifest_json) = manifest {
                // Parse manifest: {"func_name":"sig_code",...}
                let parsed = crate::ffi::parse_manifest_json(manifest_json);
                let sig_map: std::collections::HashMap<String, String> =
                    parsed.into_iter().collect();

                for func_name in free_fns.iter() {
                    // PLAN-596 T4:mono 实例注册——裸名无条目时枚举
                    // "name__<label>" 实例变体逐个注册(rust.{inst},call 侧
                    // codegen 按实参形态路由到实例名)。
                    if !sig_map.contains_key(func_name.as_str()) {
                        let inst_names: Vec<String> = sig_map
                            .keys()
                            .filter(|k| k.starts_with(&format!("{}__", func_name)))
                            .cloned()
                            .collect();
                        if !inst_names.is_empty() {
                            for inst in inst_names {
                                let Some(sig_code) = sig_map.get(&inst) else {
                                    continue;
                                };
                                let exported = crate::ffi::build_exported_name(&inst, sig_code);
                                let sig = crate::ffi::sig_code_to_signature(sig_code);
                                match bridge.register_function_with_export(
                                    crate_name,
                                    &inst,
                                    &exported,
                                    sig.clone(),
                                ) {
                                    Ok(native_id) => {
                                        log::info!(
                                            "Registered Rust FFI (mono inst): {}::{} (native_id={}, sig={:?})",
                                            crate_name, inst, native_id, sig
                                        );
                                        let qualified = format!("rust.{inst}");
                                        if let Ok(mut registry) =
                                            crate::vm::native_registry::BIGVM_NATIVES.lock()
                                        {
                                            registry.register_with_id(&qualified, native_id);
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!(
                                            "Failed to register mono instance {}::{}: {:?}",
                                            crate_name,
                                            inst,
                                            e
                                        );
                                    }
                                }
                            }
                            continue;
                        }
                    }
                    let (exported_name, signature) =
                        if let Some(sig_code) = sig_map.get(func_name.as_str()) {
                            let exported = crate::ffi::build_exported_name(func_name, sig_code);
                            let sig = crate::ffi::sig_code_to_signature(sig_code);
                            (exported, sig)
                        } else {
                            // Fallback to known_signature or default String→String
                            let sig = crate::ffi::resolve_signature(crate_name, func_name)
                                .unwrap_or_else(|| {
                                    crate::ffi::RustSignature::new()
                                        .param(crate::ffi::RustType::String)
                                        .returns(crate::ffi::RustType::String)
                                });
                            (format!("auto_{}", func_name), sig)
                        };

                    match bridge.register_function_with_export(
                        crate_name,
                        func_name,
                        &exported_name,
                        signature.clone(),
                    ) {
                        Ok(native_id) => {
                            log::info!(
                                "Registered Rust FFI: {}::{} (native_id={}, sig={:?})",
                                crate_name,
                                func_name,
                                native_id,
                                signature
                            );

                            let qualified = format!("rust.{}", func_name);
                            if let Ok(mut registry) =
                                crate::vm::native_registry::BIGVM_NATIVES.lock()
                            {
                                registry.register_with_id(&qualified, native_id);
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to register Rust function {}::{}: {:?}",
                                crate_name,
                                func_name,
                                e
                            );
                        }
                    }
                }
            } else {
                // No manifest — use known_signature or default (legacy v1/v2 path)
                for func_name in free_fns.iter() {
                    let signature = crate::ffi::resolve_signature(crate_name, func_name)
                        .unwrap_or_else(|| {
                            crate::ffi::RustSignature::new()
                                .param(crate::ffi::RustType::String)
                                .returns(crate::ffi::RustType::String)
                        });

                    match bridge.register_function(crate_name, func_name, signature.clone()) {
                        Ok(native_id) => {
                            log::info!(
                                "Registered Rust FFI: {}::{} (native_id={}, sig={:?})",
                                crate_name,
                                func_name,
                                native_id,
                                signature
                            );

                            let qualified = format!("rust.{}", func_name);
                            if let Ok(mut registry) =
                                crate::vm::native_registry::BIGVM_NATIVES.lock()
                            {
                                registry.register_with_id(&qualified, native_id);
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to register Rust function {}::{}: {:?}",
                                crate_name,
                                func_name,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    // Merge the bridge's native interface into our result
    native_interface.merge(bridge.native_interface());

    Some(native_interface)
}

pub fn extract_undefined_symbol(error_msg: &str) -> Option<&str> {
    error_msg
        .strip_prefix("Undefined symbol: ")
        .and_then(|s| s.split(" in ").next())
}

/// Find the source span of a `use ... : symbol` statement in source code
pub fn find_use_symbol_span(source: &str, error_msg: &str) -> miette::SourceSpan {
    let symbol = error_msg
        .strip_prefix("Undefined symbol: ")
        .and_then(|s| s.split(" in ").next())
        .unwrap_or("");

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") && trimmed.contains(symbol) {
            let line_start = source[..]
                .split('\n')
                .take(line_num)
                .map(|l| l.len() + 1)
                .sum::<usize>();
            return miette::SourceSpan::new(line_start.into(), line.len().into());
        }
    }
    miette::SourceSpan::new(0usize.into(), source.len().min(40).into())
}

/// 链接器重映射共用的操作数跨度模型(2026-08-22 池 u32 化重写)。
/// 返回 (操作数字节长度, [(池索引偏移, 索引空间)])。
/// 两份重映射 walker(字符串池 / object_keys 池)共用本表,消灭此前
/// 双拷贝各自漂移的病灶(历史上 CALL_SPEC/CREATE_OBJ 宽度就漂过)。
/// 变长操作数(CLOSURE/CREATE_FUTURE/BUILD_FSTR/IS_VARIANT/NEW_INSTANCE)
/// 在此统一解析。
pub(crate) fn operand_span_with_pool_indices(
    op: crate::vm::opcode::OpCode,
    code: &[u8],
    pos: usize,
    last_const_i32: i32,
) -> (usize, Vec<(usize, IdxSpace)>) {
    use crate::vm::opcode::OpCode;
    let none: Vec<(usize, IdxSpace)> = Vec::new();
    match op {
        // 单枚 u32 字符串池索引
        OpCode::LOAD_STR
        | OpCode::LOAD_GLOBAL
        | OpCode::STORE_GLOBAL
        | OpCode::GET_FIELD
        | OpCode::CAPTURE_VAR
        | OpCode::LOAD_CAPTURED
        | OpCode::STORE_CAPTURED
        | OpCode::ACCUM_PAIR => (4, vec![(0, IdxSpace::Str)]),
        // 两枚 u32 字符串池索引(name, id;id 可为 0xFFFF 哨兵)
        OpCode::PUSH_ACCUM => (8, vec![(0, IdxSpace::Str), (4, IdxSpace::Str)]),
        // u32 method_name + u8 argc
        OpCode::CALL_SPEC => (5, vec![(0, IdxSpace::Str)]),
        // u32 name_idx + u8 argc + u32 id_idx(两枚池索引)
        OpCode::CREATE_NODE => (9, vec![(0, IdxSpace::Str), (5, IdxSpace::Str)]),
        // u32 key_index(object_keys 池)+ u8 field_count
        OpCode::CREATE_OBJ => (5, vec![(0, IdxSpace::Obj)]),
        // CALL_NAT: u16 native id —— 池合并后 native id 不变,只跳过
        OpCode::CALL_NAT => (2, none),
        // 1 字节定长
        OpCode::CONST_U8
        | OpCode::PUSH_BOOL
        | OpCode::POP_N
        | OpCode::RESERVE_STACK
        | OpCode::RET
        | OpCode::ERROR_PROPAGATE
        | OpCode::LOAD_LOCAL
        | OpCode::STORE_LOCAL
        | OpCode::LOAD_STATE_FIELD
        | OpCode::STORE_STATE_FIELD
        | OpCode::CREATE_ARRAY
        | OpCode::CREATE_TUPLE
        | OpCode::CREATE_OK
        | OpCode::GET_GENERIC_FIELD
        | OpCode::SET_GENERIC_FIELD
        | OpCode::GET_TUPLE_FIELD => (1, none),
        OpCode::FN_PROLOG => (2, none),
        // 4 字节定长
        OpCode::CONST_I32
        | OpCode::CONST_F32
        | OpCode::JMP_FAR
        | OpCode::JMP_L
        | OpCode::CALL
        | OpCode::LOAD_REF
        | OpCode::STORE_REF
        | OpCode::LOAD_MUT_REF
        | OpCode::STORE_MUT_REF => (4, none),
        OpCode::CONST_I64 | OpCode::CONST_U64 | OpCode::CONST_F64 => (8, none),
        OpCode::SPAWN | OpCode::CREATE_GENERATOR => (5, none),
        OpCode::SLEEP | OpCode::JOIN | OpCode::SEND => (4, none),
        OpCode::SOURCE_LINE => (2, none),
        OpCode::JMP | OpCode::JMP_IF_Z | OpCode::JMP_IF_NZ | OpCode::PUSH_HANDLER => (2, none),
        // CLOSURE: u32 body_addr + u8 capture_count + u8 n_args
        //          + capture_count × (u32 名索引 + u16 slot)
        OpCode::CLOSURE => {
            let mut idxs = Vec::new();
            if pos + 5 < code.len() {
                let count = code[pos + 4] as usize;
                for k in 0..count {
                    idxs.push((6 + k * 6, IdxSpace::Str));
                }
                (6 + count * 6, idxs)
            } else {
                (6, idxs)
            }
        }
        // CREATE_FUTURE: u32 body_offset + u8 capture_count + count × u32 名索引
        OpCode::CREATE_FUTURE => {
            let mut idxs = Vec::new();
            if pos + 4 < code.len() {
                let count = code[pos + 4] as usize;
                for k in 0..count {
                    idxs.push((5 + k * 4, IdxSpace::Str));
                }
                (5 + count * 4, idxs)
            } else {
                (5, idxs)
            }
        }
        // 变长:count 字节 + count 个 tag
        OpCode::BUILD_FSTR => {
            let parts = if pos < code.len() {
                code[pos] as usize
            } else {
                0
            };
            (1 + parts, none)
        }
        // u16 len + len 字节
        OpCode::IS_VARIANT => {
            let len = if pos + 2 <= code.len() {
                u16::from_le_bytes([code[pos], code[pos + 1]]) as usize
            } else {
                0
            };
            (2 + len, none)
        }
        // name_len 来自前一条 CONST_I32
        OpCode::NEW_INSTANCE => (last_const_i32.max(0) as usize, none),
        // 无操作数
        _ => (0, none),
    }
}

/// Internal AutoVM execution function (async)
/// Plan 177: capture parameter enables stdout capture for testing
/// 池索引空间:字符串池 vs object_keys 池(两份重映射各自的关注点)。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum IdxSpace {
    Str,
    Obj,
}

/// Format a Value for display in object literals
pub(crate) fn format_value_for_display(vm: &crate::vm::engine::AutoVM, val: &Value) -> String {
    match val {
        Value::Int(i) => {
            // Check if it's a tagged string index
            if *i < 0 && *i > -1000000 && *i != -2147483648 && *i != -2147483647 {
                let str_idx = (-i - 1) as usize;
                let strings = vm.strings.read().unwrap();
                if let Some(bytes) = strings.get(str_idx) {
                    return format!("\"{}\"", String::from_utf8_lossy(bytes));
                }
            }
            i.to_string()
        }
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => format!("\"{}\"", s.as_str()),
        Value::Nil => "nil".to_string(),
        Value::VmRef(vm_ref) => {
            // Recursively format VmRef values.
            // Plan 390 §15 H3b: objects/arrays/nodes all live in heap_objects.
            let id = vm_ref.id as u64;
            if let Some(obj_arc) = vm.get_heap_object(id) {
                let obj = obj_arc.read().unwrap();
                if let Some(obj) = obj.as_any().downcast_ref::<crate::vm::types::ObjectData>() {
                    let fields: Vec<String> = obj
                        .fields
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, format_value_for_display(vm, v)))
                        .collect();
                    format!("{{{}}}", fields.join(", "))
                } else if let Some(arr) = obj
                    .as_any()
                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                {
                    let items: Vec<String> = arr
                        .elems
                        .iter()
                        .map(|v| format_value_for_display(vm, v))
                        .collect();
                    format!("[{}]", items.join(", "))
                } else {
                    format!("<ref:{}>", id)
                }
            } else {
                format!("<ref:{}>", id)
            }
        }
        Value::Array(arr) => {
            let items: Vec<String> = arr
                .values
                .iter()
                .map(|v| format_value_for_display(vm, v))
                .collect();
            format!("[{}]", items.join(", "))
        }
        Value::Obj(obj) => {
            let fields: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value_for_display(vm, v)))
                .collect();
            format!("{{{}}}", fields.join(", "))
        }
        _ => val.repr().to_string(),
    }
}

// ---- T-02：trans C/Rust 直译腿（纯 trans 后端，无 ui_gen）----
/// Transpile to Rust with incremental compilation support
///
/// This function uses CompileSession to enable incremental compilation,
/// caching results between calls for faster subsequent transpilations.
///
/// # Arguments
/// * `session` - Mutable reference to CompileSession (maintains cache)
/// * `path` - Path to the AutoLang source file
///
/// # Returns
/// Ok(String) with success message indicating transpiled file names
///
/// # Example
/// ```no_run
/// use auto_lang::{trans_rust_with_session, compile::CompileSession};
///
/// let mut session = CompileSession::new();
/// let result = trans_rust_with_session(&mut session, "test.at").unwrap();
/// println!("{}", result);
/// ```
pub fn trans_rust_with_session(session: &mut CompileSession, path: &str) -> AutoResult<String> {
    // PLAN-738 T-03 (AC-02): declare the assembly target BEFORE any loading or
    // emission (see trans_c_with_session). The standalone re-parse below uses
    // CompileDest::TransRust — the session target must agree with it, and any
    // VM-layer bytecode preprocessing must never bind as the Rust target's
    // implementation.
    session.set_assembly_target(crate::stdlib_assembly::model::AssemblyTarget::Rust)?;
    let code = std::fs::read_to_string(path)?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        session.add_source_dir(parent.to_path_buf());
    }
    session.resolve_uses(&code)?;

    // Compile source with incremental support (for dirty-tracking / caching)
    let frag_ids = session.compile_source(&code, path)?;

    // Plan 204 Phase 6A: Use .a2r.rs suffix to avoid overwriting .rs files
    let rsname = path.replace(".at", ".a2r.rs");
    if session.layer_selections.iter().any(|selection| {
        selection
            .context_file
            .as_ref()
            .is_some_and(|file| file.ends_with(".rs.at"))
    }) {
        let (output, references) = crate::stdlib_assembly::emission::emit_rust_assembly(
            session,
            &code,
            std::path::Path::new(path),
        )?;
        session.assembly_references.extend(references);
        let manifest =
            session.freeze_assembly_manifest("rust-emission", std::path::Path::new(path), &code)?;
        std::fs::write(&rsname, output)?;
        std::fs::write(
            format!("{rsname}.assembly.json"),
            serde_json::to_vec(&manifest)
                .map_err(|error| crate::error::AutoError::Msg(error.to_string()))?,
        )?;
        return Ok(format!(
            "[trans] {path} -> {rsname} (selected Rust Auto layers)"
        ));
    }

    let fname = AutoPath::new(path).filename();

    // Re-parse for transpilation using the full pipeline (not incremental),
    // so that type declarations (structs, enums, etc.) are emitted.
    let mut parser = Parser::from(code.as_str());
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let mut ast = parser.parse()?;

    // Run CTEE (Compile-Time Execution Engine) to transform AST
    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast)?;

    // Full transpilation via RustTrans::trans()
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::rust::RustTrans::new(fname);
    // Plan 610 ⑥: relative `use.c "<file>.json"` manifests resolve against the
    // transpiled file's directory.
    trans.source_dir = std::path::Path::new(path).parent().map(|p| p.to_path_buf());
    // Plan 376U: A2R_CRATE_ROOT=1 marks this file as a crate root (lib.at /
    // */mod.at). Such files emit top-level `use` as `pub use` (re-exports) and
    // the `#![allow(...)]` crate pragma, so they can serve as an auto-generated
    // lib.rs / aggregator without hand-editing.
    if std::env::var("A2R_CRATE_ROOT")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        trans.is_crate_root = true;
        trans.emit_allow_pragma = true;
    }

    // Plan 376D: Build a shared TypeStore from ALL sibling .at files (including
    // subdirectories) so the type inference engine can resolve cross-module types.
    {
        // PLAN-010 T3: only treat the two-levels-up directory as the "crate
        // src root" when it IS literally named `src` (the repo layout this
        // heuristic was built for). For arbitrary CLI paths the old
        // unconditional parent().parent() walked unrelated trees (a sample
        // in $TMP scanned the entire temp dir) — the `auto trans` hang.
        let file_dir = std::path::Path::new(path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let src_root = match file_dir.parent() {
            Some(pp) if pp.file_name().and_then(|n| n.to_str()) == Some("src") => pp.to_path_buf(),
            _ => file_dir.clone(),
        };
        let mut type_store = crate::types::TypeStore::new();
        // Scan the source root recursively for .at files
        // PLAN-010 T3: the scan MUST be bounded. The two-levels-up "crate
        // src root" assumption makes arbitrary CLI paths (e.g. a sample in
        // $TMP) walk enormous unrelated trees parsing every .at file — the
        // `auto trans` multi-minute hang (009 DEBTS obs. #3). Bound depth,
        // file count, and skip build/VC litter.
        const SCAN_MAX_DEPTH: usize = 6;
        const SCAN_MAX_FILES: usize = 400;
        const SCAN_SKIP_DIRS: &[&str] = &[
            ".git",
            "target",
            "node_modules",
            ".wt",
            ".worktrees",
            ".cargo",
            "dist",
            "build",
            ".venv",
            "__pycache__",
        ];
        fn scan_at_files(
            dir: &std::path::Path,
            store: &mut crate::types::TypeStore,
            depth: usize,
            budget: &mut usize,
        ) {
            if depth > SCAN_MAX_DEPTH || *budget == 0 {
                return;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if *budget == 0 {
                        return;
                    }
                    let entry_path = entry.path();
                    let skipped_dir = entry_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| SCAN_SKIP_DIRS.contains(&n))
                        .unwrap_or(false);
                    if entry_path.is_dir() {
                        if !skipped_dir {
                            scan_at_files(&entry_path, store, depth + 1, budget);
                        }
                    } else if entry_path.extension().map(|e| e == "at").unwrap_or(false) {
                        // PLAN-738 T-03 (AC-02): VM/C target layers must not
                        // leak into the Rust emission type context — .vm.at /
                        // .c.at are foreign layers here. .rs.at is the Rust
                        // target's selected (mirror-signature) layer — kept,
                        // alongside the public .at files.
                        let scan_fname = entry_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");
                        if scan_fname.ends_with(".vm.at") || scan_fname.ends_with(".c.at") {
                            continue;
                        }
                        *budget -= 1;
                        if let Ok(code) = std::fs::read_to_string(&entry_path) {
                            let mut p = crate::parser::Parser::from(code.as_str());
                            p.set_dest(crate::parser::CompileDest::TransRust);
                            p.skip_check = true;
                            if let Ok(ast) = p.parse() {
                                for stmt in &ast.stmts {
                                    use crate::ast::Stmt;
                                    match stmt {
                                        Stmt::Fn(fd) => {
                                            store.register_fn_decl(fd);
                                        }
                                        Stmt::TypeDecl(td) => {
                                            store.register_type_decl(td);
                                        }
                                        Stmt::SpecDecl(sd) => {
                                            store.register_spec_decl(sd);
                                        }
                                        Stmt::EnumDecl(ed) => {
                                            store.register_enum_decl(ed.clone());
                                        }
                                        Stmt::Ext(ext) => {
                                            store.register_ext_methods(ext);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        scan_at_files(&src_root, &mut type_store, 0, &mut SCAN_MAX_FILES);
        trans.set_shared_type_store(Some(std::sync::Arc::new(std::sync::RwLock::new(
            type_store,
        ))));
    }

    // Pre-populate struct_fields from sibling .at files in the same directory.
    // This allows cross-file positional struct construction (e.g., Pos(1, 0, 0)
    // in lexer.at when Pos is defined in pos.at) to use actual field names
    // instead of falling back to field0/field1/field2.
    if let Some(parent) = std::path::Path::new(path).parent() {
        // Plan 380: recurse into subdirectories too (e.g. builtin_roles/mod.at
        // defines `load_builtin` used by src/roles.at) and pre-populate
        // fn_ret_types so the spec-bound-ident / .await heuristics see
        // cross-module return types on the single-file CLI path.
        let mut at_files: Vec<std::path::PathBuf> = Vec::new();
        let mut stack: Vec<(std::path::PathBuf, usize)> = vec![(parent.to_path_buf(), 0)];
        // PLAN-010 T3: same bounded-scan policy as the crate-root scan —
        // unbounded recursion over arbitrary CLI paths is the trans hang.
        const SIB_SCAN_MAX_DEPTH: usize = 6;
        const SIB_SCAN_MAX_FILES: usize = 400;
        const SIB_SCAN_SKIP_DIRS: &[&str] = &[
            ".git",
            "target",
            "node_modules",
            ".wt",
            ".worktrees",
            ".cargo",
            "dist",
            "build",
            ".venv",
            "__pycache__",
        ];
        while let Some((dir, depth)) = stack.pop() {
            if depth > SIB_SCAN_MAX_DEPTH || at_files.len() >= SIB_SCAN_MAX_FILES {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let ep = entry.path();
                    let skipped_dir = ep
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| SIB_SCAN_SKIP_DIRS.contains(&n))
                        .unwrap_or(false);
                    if ep.is_dir() {
                        if !skipped_dir {
                            stack.push((ep, depth + 1));
                        }
                    } else if ep.extension().map(|e| e == "at").unwrap_or(false) {
                        at_files.push(ep);
                        if at_files.len() >= SIB_SCAN_MAX_FILES {
                            break;
                        }
                    }
                }
            }
        }
        // PLAN-010 T3: cap the number and size of parsed siblings — parsing
        // hundreds of large stray .at files (e.g. in a temp dir) is the
        // multi-minute `auto trans` hang.
        const SIB_PARSE_MAX: usize = 50;
        const SIB_PARSE_MAX_BYTES: u64 = 1_000_000;
        let mut sib_parsed = 0usize;
        for entry_path in at_files {
            if sib_parsed >= SIB_PARSE_MAX {
                break;
            }
            if entry_path == std::path::Path::new(path) {
                continue;
            }
            let too_big = std::fs::metadata(&entry_path)
                .map(|m| m.len() > SIB_PARSE_MAX_BYTES)
                .unwrap_or(true);
            if too_big {
                continue;
            }
            if let Ok(sibling_code) = std::fs::read_to_string(&entry_path) {
                sib_parsed += 1;
                let mut sib_parser = Parser::from(sibling_code.as_str());
                sib_parser.set_dest(crate::parser::CompileDest::TransRust);
                sib_parser.skip_check = true;
                if let Ok(sib_ast) = sib_parser.parse() {
                    for stmt in &sib_ast.stmts {
                        match stmt {
                            crate::ast::Stmt::TypeDecl(td) => {
                                let field_names: Vec<auto_val::AutoStr> =
                                    td.members.iter().map(|m| m.name.clone()).collect();
                                if !field_names.is_empty()
                                    && !trans.struct_fields().contains_key(&td.name)
                                {
                                    trans
                                        .struct_fields_mut()
                                        .insert(td.name.clone(), field_names);
                                }
                            }
                            // Plan 380: pre-populate fn_ret_types from
                            // sibling fns (unqualified + Type.method) so
                            // the spec-bound-ident / .await heuristics
                            // work on the single-file CLI path.
                            crate::ast::Stmt::Fn(fd) => {
                                let fname: auto_val::AutoStr = fd.name.clone();
                                if !trans.fn_ret_types_mut().contains_key(&fname) {
                                    trans.fn_ret_types_mut().insert(fname, fd.ret.clone());
                                }
                                if let Some(p) = &fd.parent {
                                    let qualified: auto_val::AutoStr =
                                        format!("{}.{}", p, fd.name).into();
                                    if !trans.fn_ret_types_mut().contains_key(&qualified) {
                                        trans.fn_ret_types_mut().insert(qualified, fd.ret.clone());
                                    }
                                }
                            }
                            // Plan 371 (defect A): pre-populate spec names from
                            // sibling .at files so cross-module specs (e.g.
                            // `use role_def: Role` then `role Role`) resolve to
                            // Type::Spec (-> Box<dyn X>) on the single-file path.
                            crate::ast::Stmt::SpecDecl(sd) => {
                                let name = sd.name.clone();
                                if !trans.spec_decls_mut().contains_key(&name) {
                                    trans.spec_decls_mut().insert(name, sd.methods.clone());
                                }
                            }
                            // Plan 372 follow-up: pre-populate enum names from
                            // sibling .at files so cross-module enum errors like
                            // `Err(AgentError::Config(...))` don't get wrongly
                            // Box::new'd (AgentError lives in error.at, not the
                            // file being transpiled).
                            crate::ast::Stmt::EnumDecl(ed) => {
                                trans.known_enum_names_mut().insert(ed.name.clone());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // Plan 372 follow-up: also scan the grandparent directory (one level up)
        // to catch enums/specs defined in sibling directories (e.g. error.at in
        // src/ when transpiling orchestration/driver.at in src/orchestration/).
        if let Some(grandparent) = parent.parent() {
            // PLAN-010 T3: same parse budget/size guard as the sibling scan.
            const GP_PARSE_MAX: usize = 20;
            const GP_PARSE_MAX_BYTES: u64 = 1_000_000;
            let mut gp_parsed = 0usize;
            if let Ok(entries) = std::fs::read_dir(grandparent) {
                for entry in entries.flatten() {
                    if gp_parsed >= GP_PARSE_MAX {
                        break;
                    }
                    let entry_path = entry.path();
                    if entry_path.extension().map(|e| e == "at").unwrap_or(false) {
                        if entry_path == std::path::Path::new(path) {
                            continue;
                        }
                        let too_big = std::fs::metadata(&entry_path)
                            .map(|m| m.len() > GP_PARSE_MAX_BYTES)
                            .unwrap_or(true);
                        if too_big {
                            continue;
                        }
                        if let Ok(sibling_code) = std::fs::read_to_string(&entry_path) {
                            gp_parsed += 1;
                            let mut sib_parser = Parser::from(sibling_code.as_str());
                            sib_parser.set_dest(crate::parser::CompileDest::TransRust);
                            sib_parser.skip_check = true;
                            if let Ok(sib_ast) = sib_parser.parse() {
                                for stmt in &sib_ast.stmts {
                                    match stmt {
                                        crate::ast::Stmt::EnumDecl(ed) => {
                                            trans.known_enum_names_mut().insert(ed.name.clone());
                                        }
                                        crate::ast::Stmt::SpecDecl(sd) => {
                                            let name = sd.name.clone();
                                            if !trans.spec_decls_mut().contains_key(&name) {
                                                trans
                                                    .spec_decls_mut()
                                                    .insert(name, sd.methods.clone());
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    trans.trans(ast, &mut sink)?;

    // Plan 355: Apply the same post-processing fixes that the in-process
    // `transpile_rust()` path applies. Without this, the `auto trans` CLI
    // emitted raw code with known-correctable defects (e.g. `return None;`
    // in void fns, double semicolons after `static` decls), so a2r test
    // binaries built from CLI-transpiled sources failed to compile while the
    // identical code compiled fine when transpiled via `transpile_rust()`.
    // PLAN-009 T1 (F1): thread the restricted-name set so fix_non_ord_derives
    // never re-widens AST-restricted derives on the CLI path either.
    crate::trans::rust::RustTrans::post_process_with(&mut sink.body, &trans.ord_restricted_names());

    // Write output file
    session
        .assembly_references
        .extend(trans.assembly_references.iter().cloned());
    let manifest =
        session.freeze_assembly_manifest("rust-emission", std::path::Path::new(path), &code)?;
    let source_bytes = sink.done()?;
    if !source_bytes.is_empty() {
        std::fs::write(&rsname, source_bytes)?;
        std::fs::write(
            format!("{rsname}.assembly.json"),
            serde_json::to_vec(&manifest)
                .map_err(|error| crate::error::AutoError::Msg(error.to_string()))?,
        )?;

        // Plan 204 Phase 6B: Basic output validation
        let source_str = String::from_utf8_lossy(source_bytes);
        validate_rust_output(&rsname, &source_str);
    }

    Ok(format!(
        "[trans] {} -> {} ({} fragments tracked)",
        path,
        rsname,
        frag_ids.len()
    ))
}

/// Transpile to C with incremental compilation support
///
/// This function uses CompileSession to enable incremental compilation,
/// caching results between calls for faster subsequent transpilations.
///
/// # Arguments
/// * `session` - Mutable reference to CompileSession (maintains cache)
/// * `path` - Path to the AutoLang source file
///
/// # Returns
/// Ok(String) with success message indicating transpiled file names
///
/// # Example
/// ```no_run
/// use auto_lang::{trans_c_with_session, compile::CompileSession};
///
/// let mut session = CompileSession::new();
/// let result = trans_c_with_session(&mut session, "test.at").unwrap();
/// println!("{}", result);
/// ```
pub fn trans_c_with_session(session: &mut CompileSession, path: &str) -> AutoResult<String> {
    // PLAN-738 T-03 (AC-02): declare the assembly target BEFORE any loading or
    // emission. This entry has a zero-module-load surface today (decision
    // report E1④); the explicit target guarantees any future resolve_uses
    // consumer on this session selects the C layer — never the VM layer — as
    // its type context.
    session.set_assembly_target(crate::stdlib_assembly::model::AssemblyTarget::C)?;
    let code = std::fs::read_to_string(path)?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        session.add_source_dir(parent.to_path_buf());
    }
    session.resolve_uses(&code)?;

    // Compile source with incremental support
    let frag_ids = session.compile_source(&code, path)?;

    let artifacts = crate::stdlib_assembly::emission::emit_c_assembly(
        session,
        &code,
        std::path::Path::new(path),
    )?;
    let manifest =
        session.freeze_assembly_manifest("c-emission", std::path::Path::new(path), &code)?;
    let output = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    // All emission/strict checks finish before any generated artifact is written.
    for artifact in &artifacts {
        std::fs::write(
            output.join(format!("{}.c", artifact.stem)),
            &artifact.source,
        )?;
        std::fs::write(
            output.join(format!("{}.h", artifact.stem)),
            &artifact.header,
        )?;
    }
    std::fs::write(
        output.join(format!(
            "{}.assembly.json",
            std::path::Path::new(path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
        )),
        serde_json::to_vec(&manifest)
            .map_err(|error| crate::error::AutoError::Msg(error.to_string()))?,
    )?;
    Ok(format!(
        "[trans] {} -> {} ({} fragments, {} C artifacts)",
        path,
        path.replace(".at", ".c"),
        frag_ids.len(),
        artifacts.len()
    ))
}

/// Transpile AutoLang file to Rust
///
/// **Plan 091**: Now uses CompileSession internally (no Universe dependency)
pub fn trans_rust(path: &str) -> AutoResult<String> {
    let mut session = CompileSession::new();
    trans_rust_with_session(&mut session, path)
}

/// Evaluate config code using AutoVM (Plan 081 Phase 2)
///
/// **Replaces** `eval_config_with_scope` which uses the deprecated Interpreter.
/// Uses ConfigCodegen to compile to bytecode, then executes with AutoVM.
///
/// # Arguments
/// * `code` - Configuration source code
/// * `args` - Arguments to pass to the config
/// * `univ` - Universe for variable storage
///
/// # Returns
/// * The config value (typically a Node representing the parsed config)
///
/// # Example
/// ```no_run
/// use auto_lang::{eval_config_with_vm};
/// use auto_val::Obj;
///
/// let config = r#"
/// name: "myapp"
/// version: "0.1.0"
/// "#;
pub fn trans_c(path: &str) -> AutoResult<String> {
    let mut session = CompileSession::new();
    trans_c_with_session(&mut session, path)
}

// ---- T-02：vm/abt/assembly 测试支持根项 ----
/// Plan 204 Phase 6B: Basic structural validation of transpiled Rust output.
///
/// Checks bracket matching and basic structural validity.
/// Logs warnings if validation fails but does not block writing.
pub fn validate_rust_output(path: &str, content: &str) {
    let mut paren_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut in_string = false;
    let mut in_char = false;
    let mut escape_next = false;

    for ch in content.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if ch == '\\' && (in_string || in_char) {
            escape_next = true;
            continue;
        }
        if ch == '"' && !in_char {
            in_string = !in_string;
            continue;
        }
        if ch == '\'' && !in_string {
            in_char = !in_char;
            continue;
        }
        if in_string || in_char {
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            _ => {}
        }
    }

    let mut warnings = Vec::new();
    if paren_depth != 0 {
        warnings.push(format!("unbalanced parentheses (depth: {})", paren_depth));
    }
    if brace_depth != 0 {
        warnings.push(format!("unbalanced braces (depth: {})", brace_depth));
    }
    if bracket_depth != 0 {
        warnings.push(format!("unbalanced brackets (depth: {})", bracket_depth));
    }

    if !warnings.is_empty() {
        eprintln!(
            "[a2r warning] Output validation failed for {}: {}",
            path,
            warnings.join(", ")
        );
    }
}

/// Plan 226 Phase 6: Create an AutoVM from ABT text, ready to run.
///
/// Parses ABT text, assembles it to bytecode, and constructs a VM with
/// standard native shims (print, assert, etc.) loaded.
///
/// Returns `(vm, stdout_capture, main_entry_point)`.
pub fn create_vm_from_abt(
    abt_text: &str,
) -> AutoResult<(
    crate::vm::engine::AutoVM,
    std::sync::Arc<std::sync::RwLock<String>>,
    usize,
)> {
    crate::install_ui_host_capabilities();
    use crate::vm::abt::{asm, parser};
    use crate::vm::engine::AutoVM;
    use crate::vm::native::NativeInterface;
    use crate::vm::virt_memory::VirtualFlash;

    // 1. Parse ABT text
    let abt_program = parser::parse(abt_text)
        .map_err(|e| crate::error::AutoError::Msg(format!("ABT parse error: {}", e)))?;

    // 2. Assemble to CompiledPackage
    let pkg = asm::assemble(&abt_program)
        .map_err(|e| crate::error::AutoError::Msg(format!("ABT assemble error: {}", e)))?;

    // 3. Create VirtualFlash from CompiledPackage
    let flash = VirtualFlash::from_vec_with_metadata(
        pkg.bytecode,
        pkg.exports.clone(),
        pkg.object_keys,
        pkg.object_types,
    );

    // 4. Ensure built-in natives are registered (normally done by TypeStore/CompileSession)
    crate::vm::native_registry::register_builtin_natives();

    // 5. Create VM with stdout capture
    let (mut vm, output_buffer) = AutoVM::new_with_capture(flash, 8192);

    // 6. Load string pool
    vm.load_strings(pkg.string_pool);

    // 7. Register standard native shims + all stdlib FFI
    let mut ni = NativeInterface::new();
    ni.register_std_shims();
    crate::vm::ffi::stdlib::register_stdlib_ffi(&mut ni);
    vm.merge_native_interface(&ni);

    // 8. Determine entry point (main export, or start of bytecode)
    let main_entry = pkg
        .exports
        .get("main")
        .or_else(|| pkg.exports.get("test"))
        .copied()
        .unwrap_or(0) as usize;

    Ok((vm, output_buffer, main_entry))
}

/// Plan 225: Compile source and create a VM with stdout capture ready for debugging.
/// The caller is responsible for setting the debugger controller, spawning, and running tasks.
/// Returns (vm, stdout_capture_buffer, entry_point, result_type)
pub fn create_vm_from_source(
    code: &str,
) -> AutoResult<(
    crate::vm::engine::AutoVM,
    std::sync::Arc<std::sync::RwLock<String>>,
    usize,
    crate::vm::codegen::ObjectType,
)> {
    crate::install_ui_host_capabilities();
    use crate::vm::codegen::Codegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::loader::Linker;
    use crate::vm::opcode::OpCode;
    use crate::vm::virt_memory::VirtualFlash;

    // Plan 442 C2: axum serve adapter state reset (see execute_autovm_with_path).
    crate::vm::ffi::axum_adapter::reset();
    // PLAN-669: #[api] param-sig side channel — same reset discipline.
    let http_metadata_scope =
        crate::vm::host_capabilities::http_service::CompilationMetadataGuard::begin();
    crate::vm::ffi::http_server::clear_api_param_sigs();

    let mut session = compile::CompileSession::new();
    session.collect_rust_imports(code)?;
    session.collect_py_imports(code)?;
    session.resolve_deps(code)?;
    session.resolve_uses(code)?;

    let rust_ffi_native_interface = init_rust_ffi(&session);
    let py_ffi_native_interface = init_py_ffi(&session);

    let mut parser = Parser::new_with_type_store(code, session.type_store());
    let mut ast = parser.parse()?;

    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast)?;

    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    codegen.assembly_context = session.assembly;
    // PLAN-013 T1: 与脚本路径同款——dep 模块(如 stdlib term.vm.at)声明的
    // #[vm] fn 名播种进根 codegen,裸调用才不会被 Plan 347 影子抑制改道 reloc。
    for name in &session.vm_fn_names {
        codegen.vm_fn_names.insert(name.clone());
    }
    let (type_decls, other_stmts): (Vec<_>, Vec<_>) = ast.stmts.iter().partition(|stmt| {
        matches!(
            stmt,
            crate::ast::Stmt::TypeDecl(_)
                | crate::ast::Stmt::Ext(_)
                | crate::ast::Stmt::EnumDecl(_)
        )
    });

    for stmt in &type_decls {
        codegen.compile_stmt(stmt)?;
    }

    if !other_stmts.is_empty() {
        let n_locals = 16;
        codegen.emit_op(OpCode::FN_PROLOG);
        codegen.emit_byte(0);
        codegen.emit_byte(n_locals as u8);
        codegen.emit_op(OpCode::RESERVE_STACK);
        codegen.emit_byte(n_locals as u8);
        for (i, stmt) in ast.stmts.iter().enumerate() {
            if matches!(
                stmt,
                crate::ast::Stmt::TypeDecl(_)
                    | crate::ast::Stmt::Ext(_)
                    | crate::ast::Stmt::EnumDecl(_)
            ) {
                continue;
            }
            if i < ast.source_lines.len() {
                codegen.emit_source_line(ast.source_lines[i]);
            }
            codegen.compile_stmt(stmt)?;
        }
    }

    codegen.code.push(OpCode::HALT as u8);

    let strings = codegen.strings.clone();
    let mut linker = Linker::new();
    let dep_modules = session.take_compiled_modules();
    for module in dep_modules {
        linker.add_module(module);
    }

    let object_keys = codegen.object_keys.clone();
    let object_types = codegen.object_types.clone();
    let result_type = codegen.last_expr_type.clone();
    let generic_registry = std::mem::take(&mut codegen.generic_registry);
    // Plan 327 Phase 1: take task handler registry before finish() consumes codegen,
    // so HANDLE_MSG opcode can find `task` `on { }` handlers at runtime.
    let task_handler_registry = std::mem::take(&mut codegen.task_handler_registry);
    // Plan 312: Extract API routes before finish() consumes codegen
    let api_routes = codegen.api_routes.clone();
    session
        .assembly_references
        .extend(codegen.assembly_references.iter().cloned());
    session.freeze_assembly_manifest("vm-batch", std::path::Path::new("<main>"), code.as_ref())?;
    let main_module = codegen.finish("<main>".to_string());
    linker.add_entry_module(main_module);

    let (linked_code, global_symbols) = linker.link().map_err(|e| {
        let span = if let Some(pos) = e.source_pos {
            crate::error::pos_to_span(pos)
        } else {
            find_use_symbol_span(code, &e.message)
        };
        let help = if e.source_pos.is_some() {
            Some(format!(
                "Use a `use` statement to import '{}' from a module, or check for typos",
                e.symbol
            ))
        } else {
            extract_undefined_symbol(&e.message)
                .map(|s| format!("Check if '{}' is defined and exported in the module", s))
        };
        crate::error::AutoError::MsgWithSource(crate::error::MsgWithSource {
            source: miette::NamedSource::new("<script>", code.to_string()),
            message: e.message.clone(),
            span,
            help,
        })
    })?;

    let main_entry = if let Some(&addr) = global_symbols
        .get("main")
        .or_else(|| global_symbols.get("test"))
    {
        addr as usize
    } else {
        let dep_size: usize = linker
            .modules
            .iter()
            .take(linker.modules.len() - 1)
            .map(|m| m.code.len())
            .sum();
        dep_size
    };

    let flash = VirtualFlash::from_vec_with_metadata(
        linked_code,
        global_symbols,
        object_keys,
        object_types,
    );

    let (mut vm, output_buffer) = AutoVM::new_with_capture(flash, 8192);
    vm.load_strings(strings);
    vm.load_generic_registry(generic_registry);
    vm.load_task_handler_registry(task_handler_registry); // Plan 327 Phase 1
                                                          // Plan 312: Register #[api] routes for HTTP server dispatch
    crate::vm::host_capabilities::http_service::VmHttpBridge::bind_program_metadata(
        &mut vm,
        http_metadata_scope.finish(&api_routes),
    );
    crate::vm::ffi::stdlib::register_http_routes(api_routes);

    if let Some(rust_ni) = rust_ffi_native_interface {
        vm.merge_native_interface(&rust_ni);
    }
    if let Some(py_ni) = py_ffi_native_interface {
        vm.merge_native_interface(&py_ni);
    }

    Ok((vm, output_buffer, main_entry, result_type))
}

/// PLAN-675: 注释剥离（store 依赖启发式扫描的预处理，lib.rs/api.rs 两处
/// 扫描共用）——`//` 行注释与 `/* */` 块注释整体移除，换行保留；双引号
/// 字符串面内的 `//`、`/*` 不触发（.at 字符串字面量双引号定界）。
pub fn strip_line_block_comments(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let mut chars = code.chars().peekable();
    let mut in_str = false;
    let mut str_escape = false;
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if str_escape {
                str_escape = false;
            } else if c == '\\' {
                str_escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                out.push(c);
            }
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                for n in chars.by_ref() {
                    if n == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                while let Some(n) = chars.next() {
                    if n == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                    if n == '\n' {
                        out.push('\n');
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Drain and return stashed store composable files (filename, content).
pub fn drain_store_extra_files() -> Vec<(String, String)> {
    let result = STORE_EXTRA_FILES.with(|cell| std::mem::take(&mut *cell.borrow_mut()));
    result
}

/// Emit a use-import compile-time diagnostic (PLAN-664 P-15/P-16): stderr +
/// thread-local sink for test assertions.
pub fn use_diag(msg: String) {
    USE_DIAGS.with(|cell| cell.borrow_mut().push(msg.clone()));
    eprintln!("{}", msg);
}

/// Take (drain) accumulated use-import diagnostics — test entry.
pub fn take_use_diags() -> Vec<String> {
    USE_DIAGS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// Plan 260: Run all `#[test]` functions in an Auto source file.
/// Synchronous wrapper around test_code.
pub fn test_file(path: &str) -> AutoResult<test_runner::TestResult> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| crate::error::AutoError::Msg(format!("Failed to read file: {}", e)))?;
    let rt = get_global_runtime();
    rt.block_on(test_code(&code))
}
