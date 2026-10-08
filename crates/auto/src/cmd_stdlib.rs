//! PLAN-738 T-06：`auto stdlib inspect` — 标准库装配清点/单次装配
//! manifest/核心门检查（AC-01/06/07，SD-01/06）。
//!
//! 语义（计划 §5.6）：
//! - inventory（默认）：全库清点，parse 失败逐项诊断不删分母；有失败时
//!   完整 JSON + `status:"partial"` + 退出码 3（非零，不隐藏错误）。
//! - actual（`--actual <file.at>`）：对该文件 use 闭包做一次真实 dry 装配
//!   （resolve_uses，不执行 main/业务网络/文件操作），输出层选择 + 内容
//!   指纹 + provider 目录声明；验证等级不伪造 executed。
//! - `--check`：核心被请求闭包的 missing/conflict/unverified 违规 →
//!   退出码 1；明确 unsupported 被调用也非零，未引用 unsupported 是
//!   inventory 项（输出 inventory_notes，不计违规）。
//!
//! 退出码：0 = 通过；1 = check 违规；2 = 用法/IO/解析错误；3 = inventory
//! partial（parse 失败存在）。
//!
//! Dynamic IDs avoid the reserved production catalog; unrelated aliases and
//! missing producer signatures remain explicit check diagnostics.

use clap::Subcommand;
use serde_json::{json, Value};

use auto_lang::stdlib_assembly::loader;
use auto_lang::stdlib_assembly::model::{AssemblyDiagnostic, AssemblyTarget, Environment};
use auto_lang::stdlib_assembly::providers;
use auto_lang::stdlib_assembly::validate;

pub const EXIT_OK: i32 = 0;
pub const EXIT_CHECK_FAILED: i32 = 1;
pub const EXIT_ERROR: i32 = 2;
pub const EXIT_INVENTORY_PARTIAL: i32 = 3;

#[derive(Subcommand, Debug)]
pub enum StdlibAction {
    /// 全库清点 / 单次装配 manifest / 核心门检查（稳定 JSON）
    Inspect {
        /// 模块过滤（点分名，如 io、auto.http、encoding.base64）
        #[arg(long)]
        module: Option<String>,
        /// 执行/发射目标：vm | rust | c
        #[arg(long, default_value = "vm")]
        target: String,
        /// 运行环境：native | browser
        #[arg(long, default_value = "native")]
        environment: String,
        /// actual 模式：对该 .at 文件的 use 闭包做真实 dry 装配（不执行 main）
        #[arg(long)]
        actual: Option<String>,
        /// 核心门检查：违规非零退出（1）
        #[arg(long)]
        check: bool,
    },
}

/// `format`：全局 `--format`（OutputFormat）——JSON 是稳定契约面（默认
/// text 时也输出 JSON，除非显式 `--format json`；text 为人类摘要）。
pub fn run(action: StdlibAction, format: Option<crate::OutputFormat>) -> miette::Result<()> {
    match action {
        StdlibAction::Inspect {
            module,
            target,
            environment,
            actual,
            check,
        } => {
            let (code, value) = inspect(
                module.as_deref(),
                &target,
                &environment,
                actual.as_deref(),
                check,
            );
            let rendered = match format {
                Some(crate::OutputFormat::Text) => render_text(&value),
                _ => serde_json::to_string_pretty(&value).unwrap_or_default(),
            };
            println!("{rendered}");
            if code != EXIT_OK {
                std::process::exit(code);
            }
            Ok(())
        }
    }
}

fn parse_target(s: &str) -> Result<AssemblyTarget, String> {
    match s {
        "vm" => Ok(AssemblyTarget::Vm),
        "rust" => Ok(AssemblyTarget::Rust),
        "c" => Ok(AssemblyTarget::C),
        other => Err(format!("unknown target `{other}` (vm | rust | c)")),
    }
}

fn parse_environment(s: &str) -> Result<Environment, String> {
    match s {
        "native" => Ok(Environment::Native),
        "browser" => Ok(Environment::Browser),
        other => Err(format!("unknown environment `{other}` (native | browser)")),
    }
}

/// 模块过滤名归一：`auto.http` → `http`（stem）；其余保持点分名。
fn normalize_module(m: &str) -> String {
    m.strip_prefix("auto.").unwrap_or(m).to_string()
}

/// Production registration and inventory share the explicit stdlib root.
fn production_surfaces() -> Result<
    (
        auto_lang::stdlib_assembly::model::StdlibInventory,
        std::sync::MutexGuard<'static, auto_lang::vm::native_registry::AutoVMNativeRegistry>,
        auto_lang::vm::native::NativeInterface,
    ),
    String,
> {
    let stdlib_root = loader::repo_stdlib_root().map_err(|e| e.to_string())?;
    auto_lang::vm::native_registry::register_builtin_natives();

    let mut shims = auto_lang::vm::native::NativeInterface::new();
    shims.register_std_shims();
    auto_lang::vm::ffi::stdlib::register_stdlib_ffi(&mut shims);
    shims.build_from_inventory();

    let inv = loader::scan_inventory(&stdlib_root);
    Ok((
        inv,
        auto_lang::vm::native_registry::BIGVM_NATIVES
            .lock()
            .unwrap(),
        shims,
    ))
}

/// 入口（测试消费）：返回 (退出码, JSON)。
pub fn inspect(
    module: Option<&str>,
    target: &str,
    environment: &str,
    actual: Option<&str>,
    check: bool,
) -> (i32, Value) {
    let target = match parse_target(target) {
        Ok(t) => t,
        Err(e) => return error_value(EXIT_ERROR, &e),
    };
    let environment = match parse_environment(environment) {
        Ok(e) => e,
        Err(e) => return error_value(EXIT_ERROR, &e),
    };

    if let Some(path) = actual {
        return inspect_actual(path, target, environment, module, check);
    }

    // ---- inventory 模式 ----
    let (inv, registry, shims) = match production_surfaces() {
        Ok(x) => x,
        Err(e) => return error_value(EXIT_ERROR, &e),
    };
    let status = if inv.diagnostics.is_empty() {
        "ok"
    } else {
        "partial"
    };
    let modules: Vec<Value> = inv
        .modules
        .iter()
        .filter(|m| {
            module
                .map(|f| m.module == normalize_module(f))
                .unwrap_or(true)
        })
        .map(|m| serde_json::to_value(m).unwrap_or(Value::Null))
        .collect();
    let base = json!({
        "mode": "inventory",
        "schema_version": inv.schema_version,
        "root": inv.root,
        "status": status,
        "files_total": inv.files_total,
        "modules": modules,
        "diagnostics": inv.diagnostics,
    });
    if check {
        let (code, mut value) =
            check_core(module, target, environment, None, &inv, &registry, &shims);
        // check 输出保留 inventory 分母信息（完整分母不漏项）
        value["files_total"] = json!(inv.files_total);
        value["inventory_diagnostics"] = json!(inv.diagnostics);
        (code, value)
    } else {
        let code = if status == "partial" {
            EXIT_INVENTORY_PARTIAL
        } else {
            EXIT_OK
        };
        (code, base)
    }
}

/// actual 模式：真实 dry 装配（resolve_uses）→ 层选择 + 指纹 manifest。
fn inspect_actual(
    path: &str,
    target: AssemblyTarget,
    environment: Environment,
    module: Option<&str>,
    check: bool,
) -> (i32, Value) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return error_value(EXIT_ERROR, &format!("cannot read {path}: {e}")),
    };
    let mut session = auto_lang::compile::CompileSession::new();
    session.set_assembly_target(target).ok();
    session.assembly.environment = environment;
    if let Some(parent) = std::path::Path::new(path).parent() {
        session.add_source_dir(parent.to_path_buf());
    }
    if let Err(e) = session.resolve_uses(&source) {
        return error_value(EXIT_ERROR, &format!("assembly failed: {e}"));
    }

    let provider_schema = providers::catalog_schema_version();
    let modules: Vec<Value> = session
        .layer_selections
        .iter()
        .filter(|sel| {
            module
                .map(|f| normalize_module(&sel.module) == normalize_module(f))
                .unwrap_or(true)
        })
        .map(|sel| {
            let public_hash = std::fs::read_to_string(&sel.public_file)
                .map(|s| auto_lang::stdlib_assembly::model::fnv1a64(&s))
                .unwrap_or(0);
            let context_hash = sel
                .context_file
                .as_ref()
                .and_then(|f| std::fs::read_to_string(f).ok())
                .map(|s| auto_lang::stdlib_assembly::model::fnv1a64(&s));
            json!({
                "module": sel.module,
                "target": sel.target,
                "public_file": portable_source_id(&sel.public_file, path),
                "public_fnv1a64": public_hash,
                "context_file": sel.context_file.as_ref().map(|f| portable_source_id(f, path)),
                "context_fnv1a64": context_hash,
                "candidate_files": sel.candidate_files.iter().map(|f| portable_source_id(f, path)).collect::<Vec<_>>(),
                "context_byte_boundary": sel.context_byte_boundary,
            })
        })
        .collect();

    let catalog = providers::load_catalog().ok();
    let claims: Vec<Value> = catalog
        .as_ref()
        .map(|c| {
            c.providers
                .iter()
                .filter(|p| {
                    let t = match target {
                        AssemblyTarget::Vm => "vm",
                        AssemblyTarget::Rust => "rust",
                        AssemblyTarget::C => "c",
                    };
                    p.target == t
                        && session
                            .layer_selections
                            .iter()
                            .any(|sel| normalize_module(&sel.module) == p.module)
                        && module
                            .map(|f| p.module == normalize_module(f))
                            .unwrap_or(true)
                })
                .map(|p| serde_json::to_value(p).unwrap_or(Value::Null))
                .collect()
        })
        .unwrap_or_default();

    let mut value = json!({
        "mode": "actual",
        "schema_version": provider_schema,
        "target": target,
        "environment": environment,
        "root": loader::PORTABLE_ROOT,
        "modules": modules,
        "provider_claims": claims,
        "diagnostics": [],
    });

    if check {
        let (_, registry, shims) = match production_surfaces() {
            Ok(x) => x,
            Err(e) => return error_value(EXIT_ERROR, &e),
        };
        let mut inv = auto_lang::stdlib_assembly::model::StdlibInventory {
            schema_version: auto_lang::stdlib_assembly::model::INVENTORY_SCHEMA_VERSION,
            root: loader::PORTABLE_ROOT.into(),
            files_total: 0,
            modules: Vec::new(),
            diagnostics: Vec::new(),
        };
        let snapshot = std::sync::Arc::new(std::sync::RwLock::new(
            session.type_store().read().unwrap().clone(),
        ));
        for selection in &session.layer_selections {
            let mut layers = Vec::new();
            for (file, kind) in std::iter::once((
                &selection.public_file,
                auto_lang::stdlib_assembly::model::LayerKind::Public,
            ))
            .chain(
                selection
                    .context_file
                    .iter()
                    .map(|f| (f, auto_lang::stdlib_assembly::model::LayerKind::Vm)),
            ) {
                let source = match std::fs::read_to_string(file) {
                    Ok(source) => source,
                    Err(e) => {
                        return error_value(
                            EXIT_ERROR,
                            &format!("cannot read selected source: {e}"),
                        )
                    }
                };
                layers.push(loader::parse_layer_with_store(
                    &source,
                    kind,
                    portable_source_id(file, path),
                    snapshot.clone(),
                ));
            }
            inv.files_total += layers.len();
            inv.modules
                .push(auto_lang::stdlib_assembly::model::ModuleInventory {
                    module: normalize_module(&selection.module),
                    layers,
                });
        }
        let closure: Vec<String> = session
            .layer_selections
            .iter()
            .map(|s| normalize_module(&s.module))
            .collect();
        let closure_refs: Vec<&str> = closure.iter().map(String::as_str).collect();
        let (code, mut check_value) = check_core(
            module,
            target,
            environment,
            Some(&closure_refs),
            &inv,
            &registry,
            &shims,
        );
        check_value["mode"] = json!("actual+check");
        check_value["target"] = json!(target);
        check_value["environment"] = json!(environment);
        check_value["manifest"] = value.take();
        return (code, check_value);
    }

    (EXIT_OK, value)
}

/// 核心门：被请求核心闭包的 status 违规 + 按模块前缀分诊的 ID 冲突组。
fn check_core(
    module: Option<&str>,
    target: AssemblyTarget,
    environment: Environment,
    requested: Option<&[&str]>,
    inv: &auto_lang::stdlib_assembly::model::StdlibInventory,
    registry: &auto_lang::vm::native_registry::AutoVMNativeRegistry,
    shims: &auto_lang::vm::native::NativeInterface,
) -> (i32, Value) {
    let module_filter: Option<Vec<String>> = module
        .map(|m| vec![normalize_module(m)])
        .or_else(|| requested.map(|mods| mods.iter().map(|m| normalize_module(m)).collect()));
    let mods: Option<Vec<&str>> = module_filter
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());

    let validations = validate::validate_core_vm_bindings(inv, registry, shims, environment);
    let requested_modules = mods.as_deref().unwrap_or(validate::CORE_MODULES);
    let mut violations =
        validate::requested_diagnostics(inv, target, environment, requested_modules, &validations);

    // ID 相撞分诊（T-04 遗留 b）：冲突组按 `auto.<module>.` 前缀归入受影响
    // 模块；无 --module 时全量上报（全局注册面事实）。
    for g in validate::id_alias_conflict_groups(registry)
        .into_iter()
        .filter(|_| target == AssemblyTarget::Vm)
    {
        let relevant = match &module_filter {
            None => true,
            Some(list) => g
                .names
                .iter()
                .any(|n| list.iter().any(|m| n.starts_with(&format!("auto.{m}.")))),
        };
        if relevant {
            violations.push(AssemblyDiagnostic {
                code: validate::code::NATIVE_ID_CONFLICT.to_string(),
                module: format!("id:{}", g.id),
                file: None,
                message: format!(
                    "multiple unrelated names share native id {}: {:?}",
                    g.id, g.names
                ),
            });
        }
    }

    let status = if violations.is_empty() {
        "pass"
    } else {
        "fail"
    };
    let checked_modules: Vec<&str> = mods.unwrap_or_else(|| validate::CORE_MODULES.to_vec());
    let value = json!({
        "mode": "check",
        "target": target,
        "environment": environment,
        "checked_modules": checked_modules,
        "status": status,
        "violations": violations,

    });
    let code = if violations.is_empty() {
        EXIT_OK
    } else {
        EXIT_CHECK_FAILED
    };
    (code, value)
}

fn error_value(code: i32, msg: &str) -> (i32, Value) {
    (
        code,
        json!({
            "mode": "error",
            "status": "error",
            "message": msg,
        }),
    )
}

/// text 摘要（非契约面；JSON 是稳定输出）。
fn render_text(value: &Value) -> String {
    let mode = value["mode"].as_str().unwrap_or("?");
    match mode {
        "check" => format!(
            "check {}: {} violation(s) across {:?}",
            value["status"].as_str().unwrap_or("?"),
            value["violations"].as_array().map(|a| a.len()).unwrap_or(0),
            value["checked_modules"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default(),
        ),
        "inventory" => format!(
            "inventory {} ({} files, {} diagnostics)",
            value["status"].as_str().unwrap_or("?"),
            value["files_total"].as_u64().unwrap_or(0),
            value["diagnostics"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
        ),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CLI 族约定串行（CWD 钉仓根 + 全局注册面）——plan §6.3
    /// `--test-threads=1`。组内显式串行锁兜底（防被别的滤串并行拉起）。
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_lock<T>(f: impl FnOnce() -> T) -> T {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        f()
    }

    #[test]
    fn inventory_full_partial_status_and_denominator() {
        with_lock(|| {
            let (code, value) = inspect(None, "vm", "native", None, false);
            // 冻结基线：39 处 isolated-parse 失败 → partial → 非零（不隐藏）
            assert_eq!(code, EXIT_INVENTORY_PARTIAL, "parse 失败存在必须非零");
            assert_eq!(value["status"], "partial");
            assert_eq!(value["mode"], "inventory");
            let modules = value["modules"].as_array().unwrap();
            assert!(
                modules.iter().any(|m| m["module"] == "io"),
                "六核心 io 必须在册"
            );
            assert!(value["files_total"].as_u64().unwrap() > 0, "分母必须非零");
        });
    }

    #[test]
    fn inventory_module_filter() {
        with_lock(|| {
            let (_, value) = inspect(Some("auto.io"), "vm", "native", None, false);
            let modules = value["modules"].as_array().unwrap();
            assert_eq!(modules.len(), 1, "过滤后恰一模块: {modules:?}");
            assert_eq!(modules[0]["module"], "io");
        });
    }

    #[test]
    fn actual_manifest_two_layer_fixture() {
        with_lock(|| {
            let tmp = tempfile::tempdir().unwrap();
            std::fs::write(
                tmp.path().join("main.at"),
                "use proto: *\n\nfn main() {\n    let x = 1\n}\n",
            )
            .unwrap();
            std::fs::write(
                tmp.path().join("proto.at"),
                "pub fn answer() int {\n    return 42\n}\n",
            )
            .unwrap();
            std::fs::write(
                tmp.path().join("proto.vm.at"),
                "#[vm]\npub fn vm_only() int;\n",
            )
            .unwrap();
            let main_path = tmp.path().join("main.at");

            let (code, value) = inspect(
                None,
                "vm",
                "native",
                Some(main_path.to_str().unwrap()),
                false,
            );
            assert_eq!(code, EXIT_OK, "actual 装配应零违规: {}", value);
            assert_eq!(value["mode"], "actual");
            let modules = value["modules"].as_array().unwrap();
            let proto = modules
                .iter()
                .find(|m| m["module"] == "proto")
                .expect("proto 应在 actual manifest");
            assert!(
                proto["context_file"]
                    .as_str()
                    .unwrap()
                    .ends_with("proto.vm.at"),
                "VM 目标应记录选定层: {proto}"
            );
            assert!(
                proto["public_fnv1a64"].as_u64().unwrap() != 0,
                "公共段指纹必须非零"
            );
            assert_eq!(value["schema_version"].as_u64(), Some(1));
        });
    }

    #[test]
    fn check_sse_stub_is_nonzero() {
        with_lock(|| {
            let (code, value) = inspect(Some("sse"), "vm", "native", None, true);
            assert_eq!(
                code, EXIT_CHECK_FAILED,
                "sse parse_sse DeclaredStub 必须非零"
            );
            let violations = value["violations"].as_array().unwrap();
            assert!(
                violations
                    .iter()
                    .any(|v| v["code"] == validate::code::PROVIDER_CLAIM_NO_CALLEE),
                "应有 stub 违规: {violations:?}"
            );
        });
    }

    #[test]
    fn rust_net_and_unknown_module_cannot_pass_vm_checks() {
        with_lock(|| {
            let (code, value) = inspect(Some("auto.net"), "rust", "native", None, true);
            assert_eq!(code, EXIT_CHECK_FAILED);
            assert_eq!(value["target"], "rust");
            assert!(value["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["code"] == validate::code::PROVIDER_UNSUPPORTED));
            let (code, value) = inspect(Some("not_a_module"), "vm", "native", None, true);
            assert_eq!(code, EXIT_CHECK_FAILED);
            assert!(value["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["code"] == validate::code::NO_PUBLIC_LAYER));
        });
    }

    #[test]
    fn actual_non_core_closure_excludes_unrelated_core_failures() {
        with_lock(|| {
            let tmp = tempfile::tempdir().unwrap();
            let main = tmp.path().join("main.at");
            std::fs::write(&main, "use jsonx: *\nfn main() { let x = answer() }\n").unwrap();
            std::fs::write(
                tmp.path().join("jsonx.at"),
                "pub fn answer() int { return 42 }\n",
            )
            .unwrap();
            let (code, value) = inspect(None, "vm", "native", Some(main.to_str().unwrap()), true);
            assert_eq!(code, EXIT_OK, "{value}");
            assert_eq!(value["checked_modules"], json!(["jsonx"]));
            let source = value["manifest"]["modules"][0]["public_file"]
                .as_str()
                .unwrap();
            assert!(!source.contains(tmp.path().to_str().unwrap()));
        });
    }

    #[test]
    fn check_browser_io_unsupported_nonzero() {
        with_lock(|| {
            let (code, value) = inspect(Some("io"), "vm", "browser", None, true);
            assert_eq!(code, EXIT_CHECK_FAILED, "Browser io 全族 Unsupported 非零");
            let violations = value["violations"].as_array().unwrap();
            assert!(
                violations
                    .iter()
                    .any(|v| v["code"] == validate::code::PROVIDER_UNSUPPORTED),
                "应有 Unsupported 违规: {violations:?}"
            );
        });
    }

    #[test]
    fn check_http_reports_unverified_producers_without_id_collisions() {
        with_lock(|| {
            let (code, value) = inspect(Some("http"), "vm", "native", None, true);
            assert_eq!(code, EXIT_CHECK_FAILED, "未验证的生产者必须非零");
            let violations = value["violations"].as_array().unwrap();
            assert!(
                violations
                    .iter()
                    .any(|v| v["code"] == validate::code::SIGNATURE_DRIFT),
                "http 必须报告缺独立签名: {violations:?}"
            );
            assert!(!violations
                .iter()
                .any(|v| v["code"] == validate::code::NATIVE_ID_CONFLICT));
        });
    }

    #[test]
    fn check_json_missing_signature_evidence_is_nonzero() {
        with_lock(|| {
            // Canonical names now resolve; missing independent producer
            // signatures must still fail instead of becoming supported.
            let (code, value) = inspect(Some("json"), "vm", "native", None, true);
            assert_eq!(
                code, EXIT_CHECK_FAILED,
                "json 未验证的签名必须诚实非零: {}",
                value
            );
            let violations = value["violations"].as_array().unwrap();
            assert!(
                violations
                    .iter()
                    .any(|v| v["code"] == validate::code::SIGNATURE_DRIFT),
                "应有 Unverified(SIGNATURE_DRIFT) 违规: {violations:?}"
            );
        });
    }

    #[test]
    fn usage_error_unknown_target() {
        let (code, value) = inspect(None, "wasm", "native", None, false);
        assert_eq!(code, EXIT_ERROR);
        assert_eq!(value["mode"], "error");
    }
}

fn portable_source_id(file: &str, entry: &str) -> String {
    let path = std::path::Path::new(file)
        .canonicalize()
        .unwrap_or_else(|_| file.into());
    if let Ok(root) = loader::repo_stdlib_root() {
        if let Ok(relative) = path.strip_prefix(root) {
            return format!(
                "stdlib/auto/{}",
                relative.to_string_lossy().replace('\\', "/")
            );
        }
    }
    let entry = std::path::Path::new(entry)
        .canonicalize()
        .unwrap_or_else(|_| entry.into());
    let relative = entry
        .parent()
        .and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or_else(|| std::path::Path::new(path.file_name().unwrap_or_default()));
    format!("source/{}", relative.to_string_lossy().replace('\\', "/"))
}
