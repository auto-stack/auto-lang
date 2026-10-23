//! Rust UI (ICED/GPUI) project generation utilities
//!
//! This module generates Rust code from AURA widget definitions,
//! targeting ICED or GPUI backends via the auto_lang::ui runtime.
//!
//! Workflow:
//! 1. Read .at files from a `front/` directory
//! 2. Parse with AURA pipeline (CompilerSession::ui with "rust" backend)
//! 3. Extract WidgetDecl AST nodes -> AuraWidget
//! 4. Generate Rust code via RustGenerator
//! 5. Wrap in main() with backend selection (ICED/GPUI)
//! 6. Write to `rust/<name>.rs`

use std::fs;
use std::path::{Path, PathBuf};

use auto_lang_core::api::types::{ApiEndpoint, ApiModule};

use auto_lang_core::session::CompilerSession;
use auto_lang_core::Parser;
use auto_ui_compiler::ui_gen::rust::RustGenerator;
use auto_ui_compiler::ui_gen::BackendGenerator;
use colored::Colorize;

use auto_val::AutoResult;

// PLAN-815 T-04: shared rust-workspace layout moved to auto-api-gen
// (single source; re-export keeps every historic crate::rust_ui path).
pub use auto_api_gen::rust_workspace::{
    back_member_name, ensure_shared_workspace, find_lang_repo_root, front_member_name,
    get_rust_workspace_dir, resolve_rust_workspace_dir, shared_cargo_target_dir,
    workspace_lock_identity, write_shared_cargo_config,
};
// PLAN-815 T-04: backend reuse freshness gate moved to auto-api-gen
pub use auto_api_gen::api_gen::backend_generation_is_fresh;

/// Generate Rust UI code from .at files in a project directory.
///
/// Resolve the front/ source directory for a project.
/// Front dir for a given source (PLAN-815 T-06 R2 / F-815-T06-R1-01):
/// the request's declaration wins (node form > prop form); legacy and
/// undeclared requests keep the multi-location probe chain.
fn front_dir_for(project_dir: &Path, source: &PacSource) -> PathBuf {
    source
        .declared_front_dir(project_dir)
        .unwrap_or_else(|| find_front_dir(project_dir))
}

fn find_front_dir(project_dir: &Path) -> PathBuf {
    if project_dir.join("src").join("front").exists() {
        project_dir.join("src").join("front")
    } else if project_dir.join("source").join("front").exists() {
        project_dir.join("source").join("front")
    } else if project_dir.join("front").exists() {
        project_dir.join("front")
    } else {
        project_dir.join("src").join("front")
    }
}

/// Check if the generated Rust project needs to be regenerated.
/// Returns (needs_full_regen, needs_code_regen).
fn needs_regeneration(project_dir: &Path, rust_dir: &Path, source: &PacSource) -> (bool, bool) {
    let cargo_toml = rust_dir.join("Cargo.toml");
    let main_rs = rust_dir.join("src").join("main.rs");

    if !cargo_toml.exists() || !main_rs.exists() {
        return (true, true);
    }

    // Check if any .at source file is newer than main.rs
    let front_dir = front_dir_for(project_dir, source);
    if let Ok(at_files) = collect_at_files(&front_dir) {
        if let Ok(main_meta) = fs::metadata(&main_rs) {
            if let Ok(main_time) = main_meta.modified() {
                for at_file in &at_files {
                    if let Ok(at_meta) = fs::metadata(at_file) {
                        if let Ok(at_time) = at_meta.modified() {
                            if at_time > main_time {
                                return (false, true);
                            }
                        }
                    }
                }
            }
        }
    }

    // 014:侧车源(term.rs 等)同样参与指纹——此前侧车更新不触发 regen,
    // 生成 crate 一直吃旧拷贝(at-app 泵扩面实测踩中)。
    let sidecar = auto_api_gen::sidecar::load_sidecar(project_dir);
    if let Ok(main_meta) = fs::metadata(&main_rs) {
        if let Ok(main_time) = main_meta.modified() {
            for (_, source_rel) in &sidecar.modules {
                let source = project_dir.join(source_rel);
                if let Ok(at_meta) = fs::metadata(&source) {
                    if let Ok(at_time) = at_meta.modified() {
                        if at_time > main_time {
                            return (false, true);
                        }
                    }
                }
            }
        }
    }

    // Check if default feature in Cargo.toml matches expected
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        if !content.contains("default = [\"ui-iced\"]") {
            return (true, true);
        }
    }

    (false, false)
}

/// PLAN-039 T-14（组件传型）：WidgetDecl view 树是否含 PascalCase
/// 子件引用（CHILD_MSG_TYPES 预注册的口径面）。
fn view_decl_has_pascal_child(node: &auto_lang_core::ast::ui::ViewNode) -> bool {
    use auto_lang_core::ast::ui::ViewNode;
    match node {
        ViewNode::Element { tag, children, .. } => {
            if tag
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                return true;
            }
            children.iter().any(view_decl_has_pascal_child)
        }
        ViewNode::Component { .. } => true,
        ViewNode::ForLoop { body, .. } => body.iter().any(view_decl_has_pascal_child),
        ViewNode::Conditional {
            then_body,
            else_body,
            ..
        } => {
            then_body.iter().any(view_decl_has_pascal_child)
                || else_body
                    .as_ref()
                    .map(|e| e.iter().any(view_decl_has_pascal_child))
                    .unwrap_or(false)
        }
        _ => false,
    }
}

/// PLAN-039 T-14（批次 E，组件传型）：预扫各 .at 文件 widget props
/// 声明序 → WIDGET_PROP_ORDERS 预注册（app.at 字母序先于 components/
/// 编译，调用点按声明序重排需要跨文件视图；生成期组件自注册仍在 =
/// 双保险，幂等）。klondike CardFace 参数错位株的配套面。
fn collect_component_prop_orders(at_files: &[std::path::PathBuf]) {
    for at_path in at_files {
        if let Ok(code) = std::fs::read_to_string(at_path) {
            let session = CompilerSession::ui().with_backend("rust");
            let mut parser = Parser::from(code.as_str()).with_session(session);
            if let Ok(ast) = parser.parse() {
                for stmt in &ast.stmts {
                    if let auto_lang_core::ast::Stmt::WidgetDecl(widget_decl) = stmt {
                        let order: Vec<String> = widget_decl
                            .props
                            .iter()
                            .map(|p| p.name.as_str().to_string())
                            .collect();
                        if !order.is_empty() {
                            auto_ui_compiler::ui_gen::rust::WIDGET_PROP_ORDERS.with(|po| {
                                po.borrow_mut()
                                    .insert(widget_decl.name.as_str().to_string(), order);
                            });
                        }
                        // PLAN-039 T-14（组件传型）：子组件 prop 型预注册
                        // （构造参数按目标 prop 型强转的查表面）。
                        {
                            let mut prop_tys = std::collections::HashMap::new();
                            for pd in &widget_decl.props {
                                let kind = match pd.ty {
                                    auto_lang_core::ast::Type::Int
                                    | auto_lang_core::ast::Type::I64
                                    | auto_lang_core::ast::Type::Uint
                                    | auto_lang_core::ast::Type::U64 => "int",
                                    auto_lang_core::ast::Type::Bool => "bool",
                                    auto_lang_core::ast::Type::StrFixed(_)
                                    | auto_lang_core::ast::Type::StrOwned
                                    | auto_lang_core::ast::Type::StrSlice => "str",
                                    _ => "",
                                };
                                if !kind.is_empty() {
                                    prop_tys.insert(pd.name.as_str().to_string(), kind.to_string());
                                }
                            }
                            if !prop_tys.is_empty() {
                                auto_ui_compiler::ui_gen::rust::CHILD_PROP_TYPES.with(|m| {
                                    m.borrow_mut()
                                        .insert(widget_decl.name.as_str().to_string(), prop_tys);
                                });
                            }
                        }
                        // PLAN-039 T-14（组件传型）：子组件 Msg 型预注册——
                        // 无 msg 块**且 view 无 PascalCase 子件引用** → "()"；
                        // 有子件引用 → {}Msg（生成期 child gate 同口径——
                        // CardFace 无 msg 但有 CardSuit/CourtBadge 子件）。
                        let has_child_ref = widget_decl
                            .view
                            .as_ref()
                            .map(|v| view_decl_has_pascal_child(&v.root))
                            .unwrap_or(false);
                        let msg_ty = if widget_decl.messages.is_empty() && !has_child_ref {
                            "()".to_string()
                        } else {
                            format!("{}Msg", widget_decl.name.as_str())
                        };
                        auto_ui_compiler::ui_gen::rust::CHILD_MSG_TYPES.with(|m| {
                            m.borrow_mut()
                                .insert(widget_decl.name.as_str().to_string(), msg_ty);
                        });
                    }
                }
            }
        }
    }
}

/// PLAN-039 T-13（批次 E，E-D5-A 配套）：注册返回类型化 user 型的 api
/// 桩函数集到 RustGenerator 全局表（scan 收格门——typed 返回的调用
/// 局部不收 Value 格，`r.cards` 直达字段）。两 build 入口在逐文件
/// 编译前调用（thread-local 进程内持久，幂等重注册）。
fn register_api_typed_fns_for(project_dir: &Path) {
    if let Some(module) = parse_api_module(project_dir) {
        let known: std::collections::HashSet<&str> =
            module.types.iter().map(|t| t.name.as_str()).collect();
        let typed_fns: std::collections::HashSet<String> = module
            .endpoints
            .iter()
            .filter(|e| {
                let ret = e.return_type.trim();
                let base = ret.trim_start_matches("[]").trim_end_matches(']');
                !ret.is_empty() && ret != "void" && known.contains(base)
            })
            .map(|e| e.fn_name.clone())
            .collect();
        let mut primer = RustGenerator::new();
        primer.register_api_typed_fns(typed_fns);
    }
}

/// Plan 374 / Plan 371 Task 22a: pre-scan `.at` files for `StoreDecl`s.
///
/// Shared by both `generate_rust_ui` (full regen) and `regenerate_code_only`
/// (incremental regen). Previously the incremental path skipped this and
/// passed an empty store list, starving the store-composable pipeline
/// (struct/enum generation + `store.X` rewriting registration) and producing
/// 86 errors on store-composable apps like 015-notes.
fn collect_store_decls(at_files: &[std::path::PathBuf]) -> Vec<auto_lang_core::ast::ui::StoreDecl> {
    let mut all_stores: Vec<auto_lang_core::ast::ui::StoreDecl> = Vec::new();
    for at_path in at_files {
        if let Ok(code) = std::fs::read_to_string(at_path) {
            let session = CompilerSession::ui().with_backend("rust");
            let mut parser = Parser::from(code.as_str()).with_session(session);
            if let Ok(ast) = parser.parse() {
                for stmt in &ast.stmts {
                    if let auto_lang_core::ast::Stmt::StoreDecl(ref store) = stmt {
                        all_stores.push(store.clone());
                    }
                }
            }
        }
    }
    all_stores
}

/// Plan 371 Task 22c: scan ALL `.at` files for each component's own scalar
/// state fields (name, rust_type), so a parent in one file can see a child's
/// fields defined in another file.
fn collect_component_state_fields(
    at_files: &[std::path::PathBuf],
) -> std::collections::HashMap<String, Vec<(String, String)>> {
    let mut map: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    for at_path in at_files {
        if let Ok(code) = std::fs::read_to_string(at_path) {
            let session = CompilerSession::ui().with_backend("rust");
            let mut parser = Parser::from(code.as_str()).with_session(session);
            if let Ok(ast) = parser.parse() {
                for stmt in &ast.stmts {
                    if let auto_lang_core::ast::Stmt::WidgetDecl(widget_decl) = stmt {
                        let fields: Vec<(String, String)> = widget_decl
                            .model
                            .as_ref()
                            .map(|m| {
                                m.fields
                                    .iter()
                                    .filter_map(|v| {
                                        let ty = match v.ty {
                                            auto_lang_core::ast::Type::Bool => Some("bool"),
                                            auto_lang_core::ast::Type::Int => Some("i32"),
                                            auto_lang_core::ast::Type::Float
                                            | auto_lang_core::ast::Type::Double => Some("f64"),
                                            auto_lang_core::ast::Type::StrOwned
                                            | auto_lang_core::ast::Type::StrFixed(_) => {
                                                Some("String")
                                            }
                                            _ => None,
                                        };
                                        let ty = ty.or_else(|| match &v.init {
                                            auto_lang_core::ast::Expr::Bool(_) => Some("bool"),
                                            auto_lang_core::ast::Expr::Int(_) => Some("i32"),
                                            auto_lang_core::ast::Expr::Float(_, _)
                                            | auto_lang_core::ast::Expr::Double(_, _) => {
                                                Some("f64")
                                            }
                                            auto_lang_core::ast::Expr::Str(_) => Some("String"),
                                            _ => None,
                                        });
                                        ty.map(|t| (v.name.to_string(), t.to_string()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        map.insert(widget_decl.name.to_string(), fields);
                    }
                }
            }
        }
    }
    map
}

/// Semantic info about a child component, collected via cross-file pre-scan.
/// Used by the Rust generator to replace hardcoded special-case logic
/// (Init forwarding, prop writeback, delete handling) with .at-source-driven
/// general code. Mirrors `collect_component_state_fields` in structure.
/// Defined in `auto_lang::ui_gen::rust` (the generator crate) and re-exported
/// here so `auto-man` can use it without a reverse dependency.
pub use auto_ui_compiler::ui_gen::rust::ComponentSemantics;

/// Cross-file pre-scan: for each widget, collect which props its handlers write.
/// Scans `on` blocks for assignments `self.<prop>.<field> = ...` where `<prop>`
/// matches a declared widget prop (so private state like `editing` is excluded).
pub fn collect_component_semantics(
    at_files: &[std::path::PathBuf],
) -> std::collections::HashMap<String, ComponentSemantics> {
    use auto_lang_core::ast::Stmt;

    let mut map: std::collections::HashMap<String, ComponentSemantics> =
        std::collections::HashMap::new();
    for at_path in at_files {
        if let Ok(code) = std::fs::read_to_string(at_path) {
            let session = CompilerSession::ui().with_backend("rust");
            let mut parser = Parser::from(code.as_str()).with_session(session);
            if let Ok(ast) = parser.parse() {
                for stmt in &ast.stmts {
                    if let Stmt::WidgetDecl(widget_decl) = stmt {
                        let prop_names: std::collections::HashSet<String> = widget_decl
                            .props
                            .iter()
                            .map(|p| p.name.to_string())
                            .collect();
                        let mut semantics = ComponentSemantics::default();
                        // Scan handler bodies for `self.<prop>.<field> = ...` assignments.
                        if let Some(on_block) = &widget_decl.on {
                            for handler in &on_block.handlers {
                                scan_written_props(
                                    &handler.body,
                                    &prop_names,
                                    &mut semantics.written_props,
                                );
                            }
                        }
                        // Dedup while preserving order.
                        semantics.written_props.sort();
                        semantics.written_props.dedup();
                        map.insert(widget_decl.name.to_string(), semantics);
                    }
                }
            }
        }
    }
    map
}

/// Recursively scan statements for prop-write assignments.
/// Matches `self.<prop>.<field> = rhs` (two-level Dot on left, Op::Asn),
/// where `<prop>` is one of the widget's declared props.
fn scan_written_props(
    body: &auto_lang_core::ast::Body,
    prop_names: &std::collections::HashSet<String>,
    out: &mut Vec<String>,
) {
    use auto_lang_core::ast::{Expr, Stmt};
    use auto_val::Op;
    for stmt in &body.stmts {
        match stmt {
            Stmt::Expr(Expr::Bina(left, op, _)) if matches!(op, Op::Asn) => {
                // left = self.<prop>.<field>  → Dot(Dot(Ident("self"), prop), field)
                if let Expr::Dot(inner, _field) = left.as_ref() {
                    if let Expr::Dot(obj, prop) = inner.as_ref() {
                        if let Expr::Ident(name) = obj.as_ref() {
                            if name.as_str() == "self" || name.as_str() == "." {
                                let prop_str = prop.as_str().to_string();
                                if prop_names.contains(&prop_str) && !out.contains(&prop_str) {
                                    out.push(prop_str);
                                }
                            }
                        }
                    }
                }
            }
            // Recurse into control-flow constructs that contain nested bodies.
            Stmt::Expr(Expr::If(if_stmt)) => {
                for branch in &if_stmt.branches {
                    scan_written_props(&branch.body, prop_names, out);
                }
                if let Some(ref else_body) = if_stmt.else_ {
                    scan_written_props(else_body, prop_names, out);
                }
            }
            Stmt::For(for_stmt) => {
                scan_written_props(&for_stmt.body, prop_names, out);
            }
            Stmt::Block(inner_body) => {
                scan_written_props(inner_body, prop_names, out);
            }
            _ => {}
        }
    }
}

/// Regenerate only main.rs (skip Cargo.toml to preserve cargo cache).
fn regenerate_code_only(project_dir: &Path, rust_dir: &Path, src_: &PacSource) -> AutoResult<()> {
    let front_dir = front_dir_for(project_dir, src_);
    // PLAN-681 T-05（F2 bps 依赖解析面）：widget 补集进编译集 + fn 模块
    // 内嵌（resolve_bps_dependencies 头注）。
    let (bps_widget_files, bps_fn_code) = {
        let front = collect_at_files(&front_dir).unwrap_or_default();
        resolve_bps_dependencies(project_dir, &front, src_)
    };
    let mut at_files = collect_at_files(&front_dir)?;
    at_files.extend(bps_widget_files);
    if at_files.is_empty() {
        return Ok(());
    }

    let pac_path = project_dir.join("pac.at");
    let project_name = if pac_path.exists() {
        src_.name(&pac_path)
    } else {
        "MyApp".to_string()
    };

    // Plan 374 / Plan 371 Task 22a: pre-scan all .at files for StoreDecls,
    // exactly like `generate_rust_ui` does. Previously this passed `&[]`,
    // which starved `compile_at_file`'s store registration + struct/enum
    // generation — producing 86 "no field `store`" / "cannot find type
    // `StoreMsg`" errors on incremental regen of store-composable apps.
    let all_stores = collect_store_decls(&at_files);
    if !all_stores.is_empty() {
        println!(
            "  {} {} store composable(s) found",
            "Found".bright_green(),
            all_stores.len()
        );
    }
    // PLAN-039 T-13（E-D5-A 配套）：api 桩 typed 返回集注册（scan 门）。
    register_api_typed_fns_for(project_dir);
    // Plan 371 Task 22c: cross-file component state fields.
    let component_fields = collect_component_state_fields(&at_files);
    // PLAN-039 T-14（组件传型）：props 声明序预注册（构造参数重排）。
    collect_component_prop_orders(&at_files);
    // Plan 371 L1: cross-file component semantics (written props).
    let component_semantics = collect_component_semantics(&at_files);

    let mut all_components = String::new();
    let mut all_api_imports: Vec<String> = Vec::new();
    for at_path in &at_files {
        match compile_at_file(
            at_path,
            &all_stores,
            &component_fields,
            &component_semantics,
        ) {
            Ok((code, api_imports)) => {
                all_components.push_str(&code);
                all_components.push('\n');
                all_api_imports.extend(api_imports);
            }
            Err(e) => {
                let file_name = at_path.file_name().unwrap_or_default().to_string_lossy();
                println!(
                    "{} Failed to compile {}: {}",
                    "Warning:".bright_yellow(),
                    file_name,
                    e
                );
            }
        }
    }

    if all_components.trim().is_empty() {
        return Ok(());
    }

    // Deduplicate API imports and generate API client once
    deduplicate_imports(&mut all_api_imports);
    if !all_api_imports.is_empty() {
        all_components.push('\n');
        all_components.push_str(&generate_api_client(project_dir, &all_api_imports));
    }
    // PLAN-681 T-05（F2a）：bps use-fn 依赖模块内嵌。
    if !bps_fn_code.is_empty() {
        all_components.push('\n');
        all_components.push_str(&bps_fn_code);
    }

    let full_code = wrap_example(&project_name, &all_components, project_dir, src_);
    let main_rs = rust_dir.join("src").join("main.rs");
    fs::write(&main_rs, &full_code)
        .map_err(|e| format!("Failed to write {}: {}", main_rs.display(), e))?;

    // PLAN-013 T2: regen 只写 main.rs,侧车 mod 声明须重挂。014 修正:
    // 模块文件同样以工程源为准重拷(源是唯一真身;此前 regen 免复制,
    // 侧车源更新后生成 crate 一直吃旧文件,at-app term.rs 泵扩面实测
    // 踩中)。Cargo.toml 也按 pac 现值重写(exe_name 等 014 配置 regen
    // 即生效),随后 apply_sidecar_to_crate 重注依赖 + mod 声明。
    let cargo_toml = generate_cargo_toml(&project_name, project_dir, src_);
    fs::write(rust_dir.join("Cargo.toml"), &cargo_toml).map_err(|e| {
        format!(
            "Failed to write {}: {}",
            rust_dir.join("Cargo.toml").display(),
            e
        )
    })?;

    let sidecar = src_.sidecar_spec(project_dir);
    if !sidecar.is_empty() {
        auto_api_gen::sidecar::apply_sidecar_to_crate(&sidecar, project_dir, rust_dir)?;
    }

    Ok(())
}

/// `project_dir` is the workspace root (where pac.at lives).
/// `output_dir` overrides the default `rust/` output directory.
/// `_project` is reserved for future full-project scaffolding.
pub fn generate_rust_ui(
    project_dir: &Path,
    output_dir: Option<&Path>,
    _project: bool,
) -> AutoResult<()> {
    let source = PacSource::legacy();
    generate_rust_ui_inner(project_dir, output_dir, _project, &source)
}

/// 请求路径（PLAN-809 T-06c 生产入口）。
pub fn generate_rust_ui_with_request(
    project_dir: &Path,
    output_dir: Option<&Path>,
    project: bool,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    let source = PacSource::from_request(req);
    generate_rust_ui_inner(project_dir, output_dir, project, &source)
}

fn generate_rust_ui_inner(
    project_dir: &Path,
    output_dir: Option<&Path>,
    _project: bool,
    source: &PacSource,
) -> AutoResult<()> {
    println!("{}", "Generating Rust UI code".bright_cyan());

    // Determine output directory — use shared workspace at examples/rust-workspace/
    let ws_dir = ensure_shared_workspace(project_dir);
    let member_name = front_member_name(project_dir);
    let default_output = ws_dir.join(&member_name);
    let output = output_dir
        .map(|p| p.to_path_buf())
        .unwrap_or(default_output);

    fs::create_dir_all(&output).map_err(|e| format!("Failed to create output directory: {}", e))?;

    // PLAN-792：编译相位抽取为 compile_rust_ui_app（开发/交付共享）；
    // None = 无 .at/无 WidgetDecl（既有早退语义，输出字节与直写路径一致）。
    let Some(compiled) = compile_rust_ui_app(project_dir, source)? else {
        return Ok(());
    };
    let project_name = compiled.project_name;
    let main_widget = compiled.main_widget;
    let full_code = compiled.main_rs;

    // Write output as a Cargo project
    let src_dir = output.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src directory: {}", e))?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, &full_code)
        .map_err(|e| format!("Failed to write {}: {}", main_rs.display(), e))?;

    // Generate Cargo.toml with workspace dependencies
    let cargo_toml = generate_cargo_toml(&project_name, project_dir, source);
    let cargo_path = output.join("Cargo.toml");
    fs::write(&cargo_path, &cargo_toml)
        .map_err(|e| format!("Failed to write {}: {}", cargo_path.display(), e))?;

    // PLAN-013 T2: pac.at rust_sidecar 供给(用户 .rs 模块 + Cargo 依赖)。
    let sidecar = source.sidecar_spec(project_dir);
    if !sidecar.is_empty() {
        auto_api_gen::sidecar::apply_sidecar_to_crate(&sidecar, project_dir, &output)?;
        println!(
            "{}",
            "  rust_sidecar applied (modules + deps)".bright_green()
        );
    }

    // Note: no per-member .cargo/config.toml needed — the workspace-level
    // .cargo/config.toml sets target-dir for all members.

    // Update workspace members to include the newly created project
    let _ = ensure_shared_workspace(project_dir);

    println!();
    println!("{} {}", "  Generated".bright_green(), output.display());
    println!("{} {} (main widget)", "  Entry".bright_green(), main_widget);
    println!();
    println!(
        "{}",
        "  Rust UI project generated successfully!"
            .bright_green()
            .bold()
    );

    Ok(())
}

/// PLAN-792：编译相位产物（开发落点与交付落点共享的纯计算结果）。
pub(crate) struct CompiledRustUiApp {
    pub(crate) project_name: String,
    /// 入口 widget 名（打印用）。
    pub(crate) main_widget: String,
    /// 生成的 main.rs 最终内容（含 __InitLoaded 载荷类型改写后的形态）。
    pub(crate) main_rs: String,
}

/// Phase A（纯计算）：front 收集 → 组件编译 → wrap_example → api 载荷类型
/// 改写。开发模式（generate_rust_ui）与交付模式（generate_rust_ui_delivery）
/// 共享。PLAN-792 抽取自 generate_rust_ui 直写路径：`__InitLoaded` 改写从
/// "写盘后读回"改为内存内完成，最终落盘字节与抽取前一致。
///
/// 返回 `Ok(None)` = 无 .at 文件或无 WidgetDecl（既有早退语义）。
pub(crate) fn compile_rust_ui_app(
    project_dir: &Path,
    src_: &PacSource,
) -> AutoResult<Option<CompiledRustUiApp>> {
    let front_dir = front_dir_for(project_dir, src_);

    if !front_dir.exists() {
        return Err(format!("Front directory not found: {}", front_dir.display()).into());
    }

    // Collect .at files
    // PLAN-681 T-05（F2 bps 依赖解析面）：widget 补集进编译集 + fn 模块
    // 内嵌（resolve_bps_dependencies 头注）。
    let (bps_widget_files, bps_fn_code) = {
        let front = collect_at_files(&front_dir)?;
        resolve_bps_dependencies(project_dir, &front, src_)
    };
    let mut at_files = collect_at_files(&front_dir)?;
    at_files.extend(bps_widget_files);
    if at_files.is_empty() {
        println!(
            "{}",
            "  No .at files found in front directory".bright_yellow()
        );
        return Ok(None);
    }

    println!(
        "{} {} files found",
        "  Found".bright_green(),
        at_files.len()
    );

    // Get project name from pac.at
    let pac_path = project_dir.join("pac.at");
    let project_name = if pac_path.exists() {
        src_.name(&pac_path)
    } else {
        "MyApp".to_string()
    };

    // Plan 374 Task 2-3: Pre-scan all .at files to collect StoreDecls.
    let all_stores = collect_store_decls(&at_files);
    if !all_stores.is_empty() {
        println!(
            "  {} {} store composable(s) found",
            "Found".bright_green(),
            all_stores.len()
        );
    }

    // PLAN-039 T-13（E-D5-A 配套）：api 桩 typed 返回集注册（scan 门）。
    register_api_typed_fns_for(project_dir);

    // Plan 371 Task 22c: cross-file component state fields.
    let component_fields = collect_component_state_fields(&at_files);
    // PLAN-039 T-14（组件传型）：props 声明序预注册（构造参数重排）。
    collect_component_prop_orders(&at_files);
    // Plan 371 L1: cross-file component semantics (written props).
    let component_semantics = collect_component_semantics(&at_files);

    // Compile each .at file and collect generated components
    let mut all_components = String::new();
    let mut all_api_imports: Vec<String> = Vec::new();
    for at_path in &at_files {
        let file_name = at_path.file_name().unwrap_or_default().to_string_lossy();
        println!("  {} {}", "Parsing".bright_cyan(), file_name);

        match compile_at_file(
            at_path,
            &all_stores,
            &component_fields,
            &component_semantics,
        ) {
            Ok((code, api_imports)) => {
                all_components.push_str(&code);
                all_components.push('\n');
                all_api_imports.extend(api_imports);
            }
            Err(e) => {
                println!(
                    "{} Failed to compile {}: {}",
                    "Warning:".bright_yellow(),
                    file_name,
                    e
                );
            }
        }
    }

    if all_components.trim().is_empty() {
        println!(
            "{}",
            "  No components generated (no WidgetDecl nodes found)".bright_yellow()
        );
        return Ok(None);
    }

    // Deduplicate API imports and generate API client once
    deduplicate_imports(&mut all_api_imports);
    if !all_api_imports.is_empty() {
        all_components.push('\n');
        all_components.push_str(&generate_api_client(project_dir, &all_api_imports));
    }
    // PLAN-681 T-05（F2a）：bps use-fn 依赖模块内嵌。
    if !bps_fn_code.is_empty() {
        all_components.push('\n');
        all_components.push_str(&bps_fn_code);
    }

    // Wrap in main() boilerplate
    let main_widget = extract_main_widget(&all_components);
    let mut full_code = wrap_example(&project_name, &all_components, project_dir, src_);

    // PLAN-013 T2: __InitLoaded 载荷类型修正——ui_gen 硬编码
    // Vec<serde_json::Value>(JSON API 客户端世界);merged db 吸收时
    // init fn 返回具体类型(如 Vec<String>),按 api 契约改写 msg 变体。
    // PLAN-792：改写移入纯计算相位（内存内），落盘字节与写后读回一致。
    {
        let api_module = parse_api_module(project_dir);
        if let Some(module) = &api_module {
            if let Some(db) = merged_db_impl(project_dir, module) {
                if let Some(init_fn) = extract_init_api_func(&full_code) {
                    if let Some(ep) = module.endpoints.iter().find(|e| e.fn_name == init_fn) {
                        if let Some(rust_ty) = merged_scalar_rust_ty(&ep.return_type) {
                            let rewritten = full_code.replace(
                                "__InitLoaded(Vec<serde_json::Value>)",
                                &format!("__InitLoaded({rust_ty})"),
                            );
                            if rewritten != full_code {
                                full_code = rewritten;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Some(CompiledRustUiApp {
        project_name,
        main_widget,
        main_rs: full_code,
    }))
}

/// Extract API function names from `use back.api: fn1, fn2, ...` statements.
/// PLAN-627: 模块形态 `use back.api`（无符号清单）自 api.at 契约枚举
/// （`auto_lang::config::api_contract_fn_names_for_front`，与 ui_gen/api.rs
/// 双写同源语义）。
fn extract_api_imports_from_ast(ast: &auto_lang_core::ast::Code, at_path: &Path) -> Vec<String> {
    let mut imports = Vec::new();
    let mut module_form = false;
    for stmt in &ast.stmts {
        if let auto_lang_core::ast::Stmt::Use(ref use_stmt) = stmt {
            if is_api_use(use_stmt) {
                if use_stmt.items.is_empty() {
                    module_form = true;
                }
                imports.extend(use_stmt.items.iter().map(|s| s.as_str().to_string()));
            }
        }
    }
    if module_form {
        for name in auto_lang_core::config::api_contract_fn_names_for_front(at_path) {
            if !imports.contains(&name) {
                imports.push(name);
            }
        }
    }
    imports
}

/// Check if a `use` statement targets `back.api`
fn is_api_use(use_stmt: &auto_lang_core::ast::Use) -> bool {
    if use_stmt.paths.len() == 2
        && use_stmt.paths[0].as_str() == "back"
        && use_stmt.paths[1].as_str() == "api"
    {
        return true;
    }
    if let Some(ref mp) = use_stmt.module_path {
        if mp.display() == "back.api" {
            return true;
        }
    }
    false
}

/// Compile a single .at file to Rust UI code.
/// Returns (generated_code, api_imports) so callers can deduplicate API stubs.
fn compile_at_file(
    at_path: &Path,
    stores: &[auto_lang_core::ast::ui::StoreDecl],
    component_fields: &std::collections::HashMap<String, Vec<(String, String)>>,
    component_semantics: &std::collections::HashMap<String, ComponentSemantics>,
) -> AutoResult<(String, Vec<String>)> {
    let code = fs::read_to_string(at_path)
        .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

    // Parse with UI scenario targeting rust backend
    let session = CompilerSession::ui().with_backend("rust");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);
    let ast = parser
        .parse()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let mut output = String::new();
    let mut generator = RustGenerator::new();

    // PLAN-039 T-12（批次 E）：预填 store 字段类型表——app.at 与
    // *_store.at 分文件编译（字母序 app 在前），`.store.<field>.<sub>`
    // 降链判定需要跨 widget 的字段型视图（表全局幂等，逐文件重填无扰）。
    // shim 家族注入在 wrap_example（main.rs 组装级一次——per-file 会撞
    // 多文件拼接的重复定义）。
    for store in stores {
        generator.prime_store_field_types(store);
    }

    // Extract API imports from `use back.api: ...` statements
    let api_imports = extract_api_imports_from_ast(&ast, at_path);

    // Plan 374 Task 1: Register view fn fragments BEFORE extracting widgets.
    auto_ui_compiler::aura::extract::clear_view_fragments();
    for stmt in &ast.stmts {
        if let auto_lang_core::ast::Stmt::ViewFragmentDecl(frag) = stmt {
            auto_ui_compiler::aura::extract::register_view_fragment(frag);
        }
    }

    // Plan 374 Task 2: Register store names for store.X rewriting (all files),
    // but only GENERATE the store struct in the file that declares it.
    // PLAN-681 T-05（E0618 收口）：预填 store msg 载荷表——on-only 根
    // handler 的载荷推断源（app.at 字母序先编译，须跨文件预填）。
    auto_ui_compiler::ui_gen::rust::register_store_msg_payloads(stores);
    for store in stores {
        generator.register_store("store", store.name.as_str());
    }
    // Check if THIS file contains any StoreDecl — only generate from here.
    let file_has_store = ast
        .stmts
        .iter()
        .any(|s| matches!(s, auto_lang_core::ast::Stmt::StoreDecl(_)));
    if file_has_store {
        for store in stores {
            let fake_decl = auto_lang_core::ast::ui::WidgetDecl {
                name: store.name.clone(),
                messages: store.messages.clone(),
                model: store.model.clone(),
                computed: store.computed.clone(),
                view: None,
                named_views: Vec::new(),
                on: store.on.clone(),
                bind: None,
                props: Vec::new(),
                routes: None,
                lifecycle: Vec::new(),
                style: None,
                ext_imports: Vec::new(),
                watch: Vec::new(),
                expose: Vec::new(),
                setup: None,
                actions: None,
                timer: None,
            };
            match auto_ui_compiler::aura::extract_widget_from_decl(&fake_decl) {
                Ok(aura_widget) => {
                    let rust_code = generator
                        .generate(&aura_widget)
                        .map_err(|e| e.to_string())?;
                    output.push_str(&rust_code);
                    output.push('\n');
                }
                Err(e) => {
                    eprintln!("  Warning: store '{}' extraction failed: {}", store.name, e);
                }
            }
        }
    }

    // Plan 371 Task 22c: register every component's own scalar state fields.
    for (name, fields) in component_fields.iter() {
        generator.register_component_state(name, fields.clone());
    }
    // Plan 371 L1: register every component's semantics (written props).
    for (name, sem) in component_semantics.iter() {
        generator.register_component_semantics(name, sem.clone());
    }

    // Extract AURA widgets from AST
    for stmt in &ast.stmts {
        if let auto_lang_core::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let mut aura_widget = auto_ui_compiler::aura::extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            aura_widget.api_imports = api_imports.clone();

            let rust_code = generator
                .generate(&aura_widget)
                .map_err(|e| e.to_string())?;

            output.push_str(&rust_code);
            output.push('\n');
        }
    }

    // Plan 374: Generate Rust type aliases from Auto type declarations.
    // User-defined types (like Note) are aliased to serde_json::Value since
    // the code generator uses Value-based dynamic field access (["field"]).
    for stmt in &ast.stmts {
        if let auto_lang_core::ast::Stmt::TypeDecl(type_decl) = stmt {
            output.push_str(&generate_type_struct(type_decl));
            output.push('\n');
        }
    }

    // PLAN-039 T-12（批次 E）：store 文件级自由 fn 的 rust 轨发射（vue
    // 轨 module_fns 通道 parity）——`fn esc(s str) str` 株（kanban 写端点
    // body JSON 转义）此前静默丢弃，生成 handler 报 cannot find function。
    // #[api]/test fn 由 extract_module_fn 门跳过；参数/返回按声明型直译，
    // 函数体复用 handler 语句翻译链（join/postprocess 全含）。
    for stmt in &ast.stmts {
        if let auto_lang_core::ast::Stmt::Fn(fn_decl) = stmt {
            if auto_ui_compiler::aura::extract::extract_module_fn(fn_decl).is_some() {
                output.push_str(&generate_module_fn_rust(&generator, fn_decl));
                output.push('\n');
            }
        }
    }

    // NOTE: API stubs are NOT generated here — callers collect all imports
    // across files and generate stubs once to avoid duplicates.

    Ok((output, api_imports))
}

/// Deduplicate API imports, preserving order.
fn deduplicate_imports(imports: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    imports.retain(|s| seen.insert(s.clone()));
}

/// PLAN-013 T2: merged 模式的 db.at 吸收件(转译码 + 可覆盖的 fn 名集)。
struct MergedDbImpl {
    code: String,
    fns: std::collections::HashSet<String>,
}

/// PLAN-681 T-02（route A 前半）：伴生模块 + 端点体模块的 merged 内嵌件。
/// 标量契约(无 primary pub type)且无 db.at 时——门控与 back 侧
/// api_gen.rs generate_rust_server 的 route A 同款；fsys/TreeIcon 等
/// 伴生实现与剥属性行转译的 api 端点体进 front 进程内嵌入。
struct MergedRouteAImpl {
    api_impl: String,
    companions: Vec<(String, String)>, // (mod name, rust source)
}

fn merged_route_a_impl(
    project_dir: &Path,
    module: &auto_lang_core::api::ApiModule,
) -> Option<MergedRouteAImpl> {
    if auto_api_gen::api_gen::primary_type_name_pub(module).is_some() {
        return None;
    }
    if project_dir.join("src").join("back").join("db.at").exists() {
        return None;
    }
    let back_dir = project_dir.join("src").join("back");
    let mut companions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&back_dir) {
        let mut stems: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "at").unwrap_or(false))
            .filter_map(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| s.trim_end_matches(".at").to_string())
            })
            .filter(|stem| stem != "api" && stem != "db")
            .collect();
        stems.sort();
        for stem in stems {
            let content = std::fs::read_to_string(back_dir.join(format!("{}.at", stem))).ok()?;
            let rs = auto_api_gen::api_gen::transpile_back_module_to_rs(&stem, &content).ok()?;
            companions.push((stem, auto_api_gen::api_gen::post_process_companion_rs(rs)));
        }
    }
    let api_at = std::fs::read_to_string(back_dir.join("api.at")).ok()?;
    let stripped: String = api_at
        .lines()
        .filter(|l| !l.trim_start().starts_with("#[api"))
        .collect::<Vec<_>>()
        .join("\n");
    let api_impl =
        auto_api_gen::api_gen::transpile_back_module_to_rs("api_impl", &stripped).ok()?;
    let api_impl = auto_api_gen::api_gen::post_process_companion_rs(api_impl);
    Some(MergedRouteAImpl {
        api_impl,
        companions,
    })
}

/// 载入并转译 src/back/db.at;含「引用工程外类型面」(crate::types /
/// super:: 依赖,015-notes 类结构型 db)的转译件不可吸收——避免 front
/// crate 里悬空引用;此时返回 None,endpoint 落回 JSON CRUD 脚手架。
fn merged_db_impl(
    project_dir: &Path,
    module: &auto_lang_core::api::ApiModule,
) -> Option<MergedDbImpl> {
    let db_file = project_dir.join("src").join("back").join("db.at");
    let content = std::fs::read_to_string(db_file).ok()?;
    let _ = module;
    let db_rs = auto_api_gen::api_gen::transpile_db_to_rs(&content).ok()?;
    let rs = auto_api_gen::api_gen::post_process_db_rs(db_rs);
    if rs.contains("crate::types") || rs.contains("super::") || rs.contains("crate::api::") {
        return None;
    }
    let fns = auto_api_gen::api_gen::extract_db_fn_names(&rs);
    if fns.is_empty() {
        return None;
    }
    Some(MergedDbImpl { code: rs, fns })
}

/// api.at 类型面的标量子集 → Rust 类型;[]str / List<str> 映射 Vec<String>。
/// 结构型/未知类型返回 None(endpoint 不可吸收)。
fn merged_scalar_rust_ty(ty: &str) -> Option<&'static str> {
    match ty.trim() {
        "int" | "i64" => Some("i64"),
        "str" | "String" => Some("String"),
        "bool" => Some("bool"),
        "float" | "f32" => Some("f32"),
        "double" | "f64" => Some("f64"),
        "void" | "()" => Some("()"),
        "[]str" | "[]String" | "List<str>" | "List<String>" => Some("Vec<String>"),
        "[]int" | "List<int>" | "List<i64>" => Some("Vec<i64>"),
        _ => None,
    }
}

/// db 吸收的 endpoint 委托 fn;不可覆盖(无同名 db fn / 非标量面)→ None。
fn merged_db_delegate(
    db: &MergedDbImpl,
    endpoint: &auto_lang_core::api::ApiEndpoint,
) -> Option<String> {
    if !db.fns.contains(endpoint.fn_name.as_str()) {
        return None;
    }
    let mut sig = Vec::new();
    let mut call_args = Vec::new();
    for p in &endpoint.params {
        let ty = merged_scalar_rust_ty(&p.ty)?;
        // PLAN-019:参数面与 014 返回面同型校正——UI 模型 int 是 i32,
        // db 侧 .at int 转译是 i64;垫片参数取 i32、调用处 as i64 升位,
        // 否则 `api_f(self.int_field)` 编不下(app.at 多 Pane 槽位实测,
        // 014 "返回面统一降位" 的镜像面)。
        match ty {
            "i64" => {
                sig.push(format!("{}: i32", p.name));
                call_args.push(format!("{} as i64", p.name));
            }
            "String" => {
                // a2r str 形参是 &str(String 按引用传);标量按值。
                sig.push(format!("{}: String", p.name));
                call_args.push(format!("&{}", p.name));
            }
            other => {
                sig.push(format!("{}: {}", p.name, other));
                call_args.push(p.name.clone());
            }
        }
    }
    let ret_ty = merged_scalar_rust_ty(&endpoint.return_type)?;
    let args = call_args.join(", ");
    let ret_clause: String;
    let call;
    if ret_ty == "i64" {
        // 014 修正:UI 模型 int 是 i32(collect prop types Type::Int),
        // db 侧 .at int 转译是 i64——返回面统一降位,否则
        // `self.x = api_int()` 编不过(at-app 光标/几何回读实测)。
        ret_clause = " -> i32".to_string();
        call = format!("db::{}({}) as i32", endpoint.fn_name, args);
    } else if ret_ty == "()" {
        ret_clause = String::new();
        call = format!("db::{}({})", endpoint.fn_name, args);
    } else {
        ret_clause = format!(" -> {ret_ty}");
        call = format!("db::{}({})", endpoint.fn_name, args);
    };
    Some(format!(
        "fn {}({}){} {{\n    {}\n}}\n\n",
        endpoint.fn_name,
        sig.join(", "),
        ret_clause,
        call
    ))
}

/// Generate API client functions for Rust UI.
/// Parses the API definition from src/back/api.at and generates reqwest HTTP calls (Plan 388 W1: migrated from ureq).
/// Falls back to heuristic stubs if the API file can't be parsed.
fn generate_api_client(project_dir: &Path, api_imports: &[String]) -> String {
    // AUTO_HTTP_PORT lets multiple `auto run` instances coexist; default 8080.
    let base_url = crate::util::http_base_url();

    // Try to parse api.at to get real endpoint definitions
    let api_module = parse_api_module(project_dir);

    // PLAN-039 T-13（批次 E，E-D5-A 类型化株）：back api.at `pub type` →
    // Rust struct 发射（serde derive + Default）——kanban BoardsStore 的
    // `Vec<Card>`/`Meta` 字段此前找不到类型本体（E0425 ×4：类型声明在
    // back 文件，compile_at_file 只扫 front .at 的 TypeDecl 且 Value 化）。
    // 类型化发射后 `.cards[i].column` 直达字段（E-D2 字段表配套）。
    let api_type_structs = api_module
        .as_ref()
        .map(generate_api_type_structs)
        .unwrap_or_default();

    // Plan 347: In merged mode (AUTO_VM_MERGE != "0", the default), generate
    // in-process direct-call functions instead of HTTP client code.
    let merged_mode = std::env::var("AUTO_VM_MERGE").as_deref() != Ok("0");

    if merged_mode {
        if let Some(module) = &api_module {
            return format!(
                "{}{}",
                api_type_structs,
                generate_merged_api_client(module, project_dir)
            );
        }
        // Fallback to stubs if no api.at found
        return generate_api_stubs(api_imports);
    }

    // Split mode: generate HTTP client functions.
    // PLAN-021 T-09 根修:本函数唯一消费方 = rust 轨前台 crate(单 exe)。
    // api:rust 此前经 AUTO_VM_MERGE=0 强制走下方 split-HTTP——生成
    // Value 型 HTTP wrapper,从零构建必炸(auto-term 253 错)且使单 exe
    // 依赖外部 back 进程。017 merged 承诺 = db.at 吸收进 front crate
    // 进程内直调:凡 db.at 可吸收(全部端点有同名 db fn 且标量面)一律
    // 优先走 merged 直调;吸收不可用才回落 split-HTTP。
    if let Some(module) = &api_module {
        if merged_db_impl(project_dir, module).is_some() {
            return format!(
                "{}{}",
                api_type_structs,
                generate_merged_api_client(module, project_dir)
            );
        }
    }
    if let Some(module) = &api_module {
        let mut code = String::new();
        code.push_str(&api_type_structs);
        // Plan 349 step 1: Generate a TLS-aware HTTP client helper.
        code.push_str(&generate_http_client_helper());
        // Plan 349 step 2-3: Generate upload/download utility functions.
        code.push_str(&generate_http_utility_functions());
        // Plan 350 step 4: Generate WebSocket client functions.
        code.push_str(&generate_ws_functions());

        for endpoint in &module.endpoints {
            code.push_str(&generate_endpoint_fn(endpoint, &base_url));
            code.push('\n');
        }
        return code;
    }

    // Fallback: heuristic stubs based on function name convention
    generate_api_stubs(api_imports)
}

/// PLAN-039 T-13（批次 E，E-D5-A 类型化株）：back api.at `pub type` →
/// Rust struct 发射（serde derive + Default）。字段型走 auto_type_to_rust
/// （`[]Card` → `Vec<Card>` 递归；user 型透传=同批 struct 前置声明序由
/// ApiModule.types 的声明序保证——.at 文件内 CardsResult 引用 Card 时
/// Card 已先声明，Rust 项序无关）。Default：数值 0/串空/bool false/
/// Vec 空——kanban 空态引导（empty 判据）与 back 缺席容差兼容。
fn generate_api_type_structs(module: &ApiModule) -> String {
    let mut code = String::new();
    for t in &module.types {
        let fields: Vec<String> = t
            .fields
            .iter()
            .map(|f| format!("    pub {}: {},", f.name, auto_type_to_rust(&f.ty)))
            .collect();
        code.push_str(&format!(
            "#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]\npub struct {} {{\n{}\n}}\n\n",
            t.name,
            fields.join("\n")
        ));
    }
    code
}

/// Parse the API module from src/back/api.at
fn parse_api_module(project_dir: &Path) -> Option<ApiModule> {
    let back_dir = if project_dir.join("src").join("back").exists() {
        project_dir.join("src").join("back")
    } else if project_dir.join("back").exists() {
        project_dir.join("back")
    } else {
        return None;
    };

    let api_file = back_dir.join("api.at");
    if !api_file.exists() {
        return None;
    }

    let content = fs::read_to_string(&api_file).ok()?;
    auto_api_gen::api_gen::try_full_parse(&content)
        .or_else(|| auto_api_gen::api_gen::extract_api_lenient(&content))
}

/// Plan 349 step 1 / Plan 388 W1: Generate a TLS-aware HTTP client helper.
/// The generated API client uses `reqwest::blocking` so TLS can be configured
/// via a `ClientBuilder` (ureq cannot configure certificates per-request).
/// Set AUTO_TLS_SKIP_VERIFY=1 to skip certificate verification (dev/test);
/// set AUTO_TLS_CA_CERT=/path/to/ca.pem to add a custom CA (PEM). The client is
/// built once (OnceLock) and reused; per-call cloning is cheap (Arc-backed).
/// Plan 349 步骤 7/8 (W6): AUTO_COOKIE_STORE=1 / AUTO_GZIP=1 / AUTO_BROTLI=1 enable
/// cookie persistence and response decompression, mirroring the VM-side
/// RequestBuilder.cookie_store/gzip/brotli flags.
fn generate_http_client_helper() -> String {
    r#"// Plan 349/388: TLS-aware reqwest blocking client helper.
// Set AUTO_TLS_SKIP_VERIFY=1 to skip certificate verification (dev/test).
// Set AUTO_TLS_CA_CERT=/path/to/ca.pem to add a custom CA (PEM).
// Plan 349 步骤 7/8 (W6): Set AUTO_COOKIE_STORE=1 / AUTO_GZIP=1 / AUTO_BROTLI=1 to
// enable cookie persistence and response decompression.
fn _tls_skip_verify() -> bool {
    std::env::var("AUTO_TLS_SKIP_VERIFY").as_deref() == Ok("1")
}

fn _http_client() -> reqwest::blocking::Client {
    use std::sync::OnceLock;
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        let mut builder = reqwest::blocking::Client::builder();
        if _tls_skip_verify() {
            builder = builder.danger_accept_invalid_certs(true);
        }
        if let Ok(ca_path) = std::env::var("AUTO_TLS_CA_CERT") {
            if let Ok(pem) = std::fs::read(&ca_path) {
                if let Ok(cert) = reqwest::Certificate::from_pem(&pem) {
                    builder = builder.add_root_certificate(cert);
                }
            }
        }
        // Plan 349 步骤 7/8 (W6): cookie store + compression, env-driven (default off).
        if std::env::var("AUTO_COOKIE_STORE").as_deref() == Ok("1") {
            builder = builder.cookie_store(true);
        }
        if std::env::var("AUTO_GZIP").as_deref() == Ok("1") {
            builder = builder.gzip(true);
        }
        if std::env::var("AUTO_BROTLI").as_deref() == Ok("1") {
            builder = builder.brotli(true);
        }
        // Fall back to a plain client only if the builder itself fails (e.g. a
        // malformed PEM): failures are then *stricter* (verification stays on,
        // custom CA absent → connection errors), never silently weaker.
        builder.build().unwrap_or_default()
    })
    .clone()
}

"#
    .to_string()
}

fn generate_endpoint_fn(endpoint: &ApiEndpoint, base_url: &str) -> String {
    let fn_name = &endpoint.fn_name;
    let method = endpoint.method().to_uppercase();
    let path = endpoint.path();
    let path = path.strip_prefix('/').unwrap_or(&path);

    // Separate path params from body params
    let full_path = format!("/{}", path); // keep for :param matching
    let path_params: Vec<_> = endpoint
        .params
        .iter()
        .filter(|p| full_path.contains(&format!(":{}", p.name)))
        .collect();
    let body_params: Vec<_> = endpoint
        .params
        .iter()
        .filter(|p| !full_path.contains(&format!(":{}", p.name)))
        .collect();

    // Build URL — if path has :param, use format!()
    let has_path_params = !path_params.is_empty();
    let url_expr = if has_path_params {
        let mut url_fmt = path.to_string();
        let mut format_args: Vec<String> = Vec::new();
        for p in &path_params {
            url_fmt = url_fmt.replace(&format!(":{}", p.name), "{}");
            format_args.push(p.name.clone());
        }
        format!(
            "&format!(\"{}/{}\", {})",
            base_url,
            url_fmt,
            format_args.join(", ")
        )
    } else {
        format!("\"{}/{}\"", base_url, path)
    };

    // Function parameters
    let params: Vec<String> = endpoint
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty)))
        .collect();
    let param_list = params.join(", ");

    // Return type — typed per the api.at declaration (PLAN-021 T-09 根修)。
    // 此前除 bool 外一律 serde_json::Value,而调用点生成按 .at 声明类型
    // (int→i32/[]str→Vec<String>)消费 → 生成的 crate 从零构建必炸
    // (auto-term 253 错;曾长期被 skip-if-exists 的化石 main.rs 掩盖)。
    // JSON 线格式即声明类型的 serde 形(back api.rs 按同一声明序列化),
    // 直接反序列化为该类型。未知/自定义类型名回落 Value(Plan 388 的
    // Note 类消费面不受扰)。bool 特例保留:写结果作为 UI 控制流值。
    let return_type = &endpoint.return_type;
    let is_void = return_type == "void";
    let is_option = return_type.starts_with("?");

    // Boolean write results are control-flow values for the UI (for example,
    // a score save acknowledgement). Keep them typed and wait for the HTTP
    // response so the Store can distinguish success from a retryable failure.
    // Other scalar/value writes retain the historical non-blocking placeholder
    // behavior because they may represent server-assigned IDs.
    let is_bool = return_type == "bool";
    let (rust_return_type, value_type) = if is_void {
        (String::new(), String::new())
    } else {
        let rt = auto_type_to_rust(return_type);
        (rt.clone(), rt)
    };

    // Generate function body based on HTTP method
    let ureq_method = method.to_lowercase();
    // DELETE never needs a return value in the UI — treat as void
    // Option-returning PUT/PATCH are also fire-and-forget (non-blocking) — treat as void
    let is_fire_and_forget = is_void || method == "DELETE" || (method != "GET" && is_option);
    let body = if method == "GET" {
        generate_get_fn_body(ureq_method, url_expr, is_fire_and_forget, &value_type)
    } else if method == "DELETE" {
        generate_delete_fn_body(url_expr)
    } else {
        // POST, PUT — send JSON body
        generate_write_fn_body(ureq_method, url_expr, &body_params, is_void, &value_type)
    };

    if is_fire_and_forget {
        format!("fn {}({}) {{\n{}}}\n", fn_name, param_list, body)
    } else {
        format!(
            "fn {}({}) -> {} {{\n{}}}\n",
            fn_name, param_list, rust_return_type, body
        )
    }
}

/// Generate body for GET requests (reqwest::blocking, Plan 388 W1)
fn generate_get_fn_body(
    method: String,
    url_expr: String,
    is_void: bool,
    return_type: &str,
) -> String {
    if is_void {
        format!(
            "    let _ = _http_client().{}({}).send();\n",
            method, url_expr
        )
    } else if return_type.starts_with("Vec<") {
        format!(
            "    _http_client().{}({})\n        .send().ok()\n        .and_then(|r| r.json::<{}>().ok())\n        .unwrap_or_default()\n",
            method, url_expr, return_type
        )
    } else if return_type.starts_with("Option<") {
        // PLAN-021 T-09:声明类型直反序列化(serde 原生支持 Option<T>),
        // `.ok().flatten()` 归一外层 Result——此前硬编码 Value 使返回面
        // 与声明(Option<T>)错位。
        format!(
            "    _http_client().{}({})\n        .send().ok()\n        .and_then(|r| r.json::<{}>().ok())\n        .flatten()\n",
            method, url_expr, return_type
        )
    } else {
        format!(
            "    _http_client().{}({})\n        .send().ok()\n        .and_then(|r| r.json::<{}>().ok())\n        .unwrap_or_default()\n",
            method, url_expr, return_type
        )
    }
}

/// Generate body for DELETE requests (non-blocking via background thread)
fn generate_delete_fn_body(url_expr: String) -> String {
    let url_owned = if url_expr.starts_with('&') || url_expr.starts_with("format!") {
        format!(
            "    let url = {}.to_string();\n",
            url_expr.trim_start_matches('&')
        )
    } else {
        format!("    let url = {}.to_string();\n", url_expr)
    };
    format!(
        "{}    std::thread::spawn(move || {{ let _ = _http_client().delete(&url).send(); }});\n",
        url_owned
    )
}

/// Generate body for POST/PUT requests (with JSON body)
///
/// For void return types (e.g., update_note): non-blocking via background thread.
/// For value return types (e.g., create_note): returns a local JSON placeholder and
/// spawns a background thread for the actual HTTP call. The returned placeholder
/// contains the params so the UI can display immediately.
fn generate_write_fn_body(
    method: String,
    url_expr: String,
    body_params: &[&auto_lang_core::api::types::ApiParam],
    is_void: bool,
    return_type: &str,
) -> String {
    let json_fields: Vec<String> = body_params
        .iter()
        .map(|p| format!("\"{}\": {}", p.name, p.name))
        .collect();
    let json_body = format!("serde_json::json!({{{}}})", json_fields.join(", "));

    if is_void {
        // Non-blocking: spawn background thread for fire-and-forget
        let url_owned = if url_expr.starts_with('&') || url_expr.starts_with("format!") {
            format!(
                "    let url = {}.to_string();\n",
                url_expr.trim_start_matches('&')
            )
        } else {
            format!("    let url = {}.to_string();\n", url_expr)
        };
        format!(
            "{}    let body = {};\n    std::thread::spawn(move || {{ let _ = _http_client().{}(&url).json(&body).send(); }});\n",
            url_owned, json_body, method
        )
    } else if return_type.starts_with("Vec<") {
        // Vec return — keep blocking (rare for POST/PUT)
        format!(
            "    _http_client().{}({})\n        .json(&{})\n        .send().ok()\n        .and_then(|r| r.json::<{}>().ok())\n        .unwrap_or_default()\n",
            method, url_expr, json_body, return_type
        )
    } else if return_type.starts_with("Option<") {
        // Option return (e.g., update_note) — non-blocking: fire-and-forget in background thread
        let url_owned = if url_expr.starts_with('&') || url_expr.starts_with("format!") {
            format!(
                "    let url = {}.to_string();\n",
                url_expr.trim_start_matches('&')
            )
        } else {
            format!("    let url = {}.to_string();\n", url_expr)
        };
        format!(
            "{}    let body = {};\n    std::thread::spawn(move || {{ let _ = _http_client().{}(&url).json(&body).send(); }});\n",
            url_owned, json_body, method
        )
    } else if return_type == "bool" {
        // Boolean POST/PUT results drive UI state (save success/failure), so
        // parse the actual server response instead of returning a placeholder.
        format!(
            "    _http_client().{}({})\n        .json(&{})\n        .send().ok()\n        .and_then(|r| r.json::<bool>().ok())\n        .unwrap_or_default()\n",
            method, url_expr, json_body
        )
    } else if matches!(
        return_type.trim(),
        "int" | "i32" | "i64" | "str" | "String" | "float" | "f32" | "double" | "f64"
    ) {
        // PLAN-021 T-09:声明标量返回(int/str 等)→ 阻塞反序列化为该
        // 类型——.at 调用方按声明消费返回值(term_pump_input 的泵送计数
        // 等),占位 Value 会破坏类型契约。语义 = 与 GET 同款同步读。
        format!(
            "    _http_client().{}({})\n        .json(&{})\n        .send().ok()\n        .and_then(|r| r.json::<{}>().ok())\n        .unwrap_or_default()\n",
            method, url_expr, json_body, return_type
        )
    } else {
        // Value/未知自定义类型返回(如 create_note → Value/Note)
        // Non-blocking: return a local placeholder with the params, POST in background.
        // The actual server-generated ID won't be available, but the UI works immediately.
        let local_fields: Vec<String> = body_params
            .iter()
            .map(|p| format!("\"{}\": {}", p.name, p.name))
            .collect();
        let local_json = format!("serde_json::json!({{{}}})", local_fields.join(", "));
        let url_owned = if url_expr.starts_with('&') || url_expr.starts_with("format!") {
            format!(
                "    let url = {}.to_string();\n",
                url_expr.trim_start_matches('&')
            )
        } else {
            format!("    let url = {}.to_string();\n", url_expr)
        };
        format!(
            "{}    let body = {};\n    let local_result = {local_json};\n    std::thread::spawn(move || {{ let _ = _http_client().{}(&url).json(&body).send(); }});\n    local_result\n",
            url_owned, json_body, method, local_json = local_json
        )
    }
}

/// Convert Auto type string to Rust type for API function signatures
fn auto_type_to_rust(ty: &str) -> String {
    match ty {
        "int" => "i32".to_string(),
        "i64" => "i64".to_string(),
        "str" => "String".to_string(),
        "bool" => "bool".to_string(),
        "void" => "()".to_string(),
        s if s.starts_with("[]") => {
            let inner = &s[2..];
            format!("Vec<{}>", auto_type_to_rust(inner))
        }
        s if s.starts_with("?") => {
            let inner = &s[1..];
            format!("Option<{}>", auto_type_to_rust(inner))
        }
        s => s.to_string(),
    }
}

/// Build the RHS expression that stores a merged-client parameter into a JSON
/// object field. `[]str`/array params (`Vec<String>`) become a JSON array so
/// the field keeps its array shape (e.g. Note.tags); scalars use Value::from.
fn merged_param_to_value(ty: &str, param_name: &str) -> String {
    if ty.starts_with("[]") {
        format!(
            "serde_json::Value::Array({}.iter().map(|s| serde_json::Value::from(s.clone())).collect())",
            param_name
        )
    } else {
        format!("serde_json::Value::from({}.clone())", param_name)
    }
}

/// Generate heuristic stub functions when API module can't be parsed.
fn generate_api_stubs(api_imports: &[String]) -> String {
    let mut code = String::new();
    code.push_str("// API function stubs (no api.at found — using heuristic placeholders)\n");
    for fn_name in api_imports {
        let lower = fn_name.to_lowercase();
        if lower.starts_with("list_") || lower.starts_with("list") {
            code.push_str(&format!(
                "fn {}() -> Vec<serde_json::Value> {{ vec![] }}\n\n",
                fn_name
            ));
        } else if lower.starts_with("create_") {
            code.push_str(&format!(
                "fn {}(_title: String, _body: String) -> serde_json::Value {{ serde_json::json!({{\"id\": 0, \"title\": _title, \"body\": _body, \"time\": \"now\"}}) }}\n\n",
                fn_name
            ));
        } else if lower.starts_with("update_") {
            code.push_str(&format!(
                "fn {}(_id: i32, _title: String, _body: String) {{ }}\n\n",
                fn_name
            ));
        } else if lower.starts_with("delete_") {
            code.push_str(&format!("fn {}(_id: i32) {{ }}\n\n", fn_name));
        } else if lower.starts_with("get_") {
            code.push_str(&format!(
                "fn {}(_id: i32) -> Option<serde_json::Value> {{ None }}\n\n",
                fn_name
            ));
        } else {
            code.push_str(&format!("fn {}() {{ }}\n\n", fn_name));
        }
    }
    code
}

/// Plan 347: Generate JSON initial data for the merged-mode global store.
/// Returns a string like `[json!({"id":0,"title":"Welcome",...}), ...]`
/// (without the outer `vec![]` wrapper).
fn generate_json_initial_data(module: &auto_lang_core::api::ApiModule) -> String {
    let primary_type = auto_api_gen::api_gen::primary_type_name_pub(module);
    let primary_type = match primary_type {
        Some(t) => t,
        None => return "[]".to_string(),
    };
    let api_type = match module.types.iter().find(|t| t.name == primary_type) {
        Some(t) => t,
        None => return "[]".to_string(),
    };

    let mut items = vec![];
    for i in 0..3i64 {
        let fields: Vec<String> = api_type
            .fields
            .iter()
            .map(|f| {
                let val = match f.ty.as_str() {
                    "int" | "i64" => format!("{}", i),
                    "bool" => match f.name.as_str() {
                        // pinned: first note pinned so the "Pinned" tab isn't empty.
                        "pinned" => if i == 0 { "true" } else { "false" }.to_string(),
                        _ => "false".to_string(),
                    },
                    s if s.starts_with("[]") => {
                        // Array field (e.g. tags) — emit a non-empty JSON array so
                        // tag filters and renders have data to show.
                        let sample = match f.name.as_str() {
                            "tags" => match i {
                                0 => "intro",
                                1 => "home",
                                _ => "work",
                            },
                            _ => "item",
                        };
                        format!("[\"{}\"]", sample)
                    }
                    _ => {
                        let sample = match f.name.as_str() {
                            "title" | "name" => match i {
                                0 => "Welcome",
                                1 => "Shopping List",
                                _ => "Meeting Notes",
                            },
                            "body" | "description" | "content" => match i {
                                0 => "This is your notes app. Click on any note to view it.",
                                1 => "Milk, Eggs, Bread, Cheese",
                                _ => "Q3 roadmap discussion with the team",
                            },
                            "time" | "date" | "created_at" => match i {
                                0 => "Just now",
                                1 => "2 hours ago",
                                _ => "Yesterday",
                            },
                            // folder: spread the 3 seed notes across the categories
                            // ("" / "personal" / "work") so each folder tab shows at
                            // least one note. Without this all notes get a generic
                            // value and folder-filtered lists render empty.
                            "folder" | "category" => match i {
                                0 => "",
                                1 => "personal",
                                _ => "work",
                            },
                            _ => "Sample",
                        };
                        format!("\"{}\"", sample)
                    }
                };
                format!("\"{}\": {}", f.name, val)
            })
            .collect();
        items.push(format!("serde_json::json!({{{}}})", fields.join(", ")));
    }
    format!("[{}]", items.join(", "))
}

/// Plan 347: Generate in-process API functions for Rust+Rust merged mode.
/// Instead of HTTP calls, functions operate on a global `static DATA: Mutex<Vec<Value>>`.
/// Function signatures match the HTTP version so widget call sites don't change.
///
/// PLAN-013 T2: 当 src/back/db.at 存在、可转译、且 endpoint 全部落在
/// 「db 同名实现 + 标量/[]str 参数返回面」时,把 db 转译件以 `mod db`
/// 嵌入,endpoint fn 直接委托 `db::<fn>(…)`——merged 模式从此运行
/// .at 真实现而非 JSON CRUD 脚手架(未覆盖的 endpoint 仍走原脚手架)。
///
/// PLAN-648 T-03 契约裁定:API_DATA JSON CRUD 原型 = **明示保留**的
/// demo 脚手架兜底,非缺陷待清——优先级恒为 ①db.at 吸收(真实现)
/// ②/api/viewer/* 媒体面(宿主共享控制面)③CRUD 脚手架(仅覆盖前
/// 两臂都不覆盖的端点)。在库消费方 = 无 db.at 的演示应用
/// (examples/ui/020-music-player、031-image-viewer);带 db.at 的
/// 应用(auto-term、013/015 等)全部走吸收臂。行为锁定测试:
/// merged_api_client_db_absorption_wins_over_crud /
/// merged_api_client_crud_fallback_for_uncovered_endpoints。
fn generate_merged_api_client(
    module: &auto_lang_core::api::ApiModule,
    project_dir: &Path,
) -> String {
    let mut code = String::new();
    code.push_str("// API functions (auto-generated, in-process merged mode — no HTTP)\n\n");

    let db_impl = merged_db_impl(project_dir, module);
    if let Some(db) = &db_impl {
        code.push_str(
            "\n// PLAN-013 T2: db.at 吸收(转译嵌入)——merged 模式的后端真实现;勿手改\npub mod db {\n#![allow(unused)]\n",
        );
        code.push_str(&db.code);
        code.push_str("\n}\n\n");
    }

    // PLAN-681 T-02（route A 前半）：伴生模块 + 端点体内嵌（db 吸收同款）。
    // api_impl 内 `crate::fsys` 路径在 front 上下文解析到下方兄弟内嵌
    // 模块（同为 main.rs 顶层 item）。
    let route_a = merged_route_a_impl(project_dir, module);
    if let Some(ra) = &route_a {
        code.push_str(
            "\n// PLAN-681 route A: companion + endpoint-body modules (in-process merged)\n",
        );
        for (name, src) in &ra.companions {
            code.push_str(&format!(
                "pub mod {} {{\n#![allow(unused)]\n{}\n}}\n\n",
                name, src
            ));
        }
        code.push_str("pub mod api_impl {\n#![allow(unused)]\n");
        code.push_str(&ra.api_impl);
        code.push_str("\n}\n\n");
    }

    // Generate JSON initial data (not strong-typed structs).
    let initial_items = generate_json_initial_data(module);
    code.push_str("use std::sync::{LazyLock, Mutex};\n");
    code.push_str("use serde_json::Value;\n\n");
    code.push_str(&format!(
        "static API_DATA: LazyLock<Mutex<Vec<Value>>> = LazyLock::new(|| {{\n    Mutex::new(vec!{})\n}});\n",
        initial_items
    ));
    code.push_str(
        "static API_NEXT_ID: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(100));\n\n",
    );

    for endpoint in &module.endpoints {
        let fn_name = &endpoint.fn_name;
        let method = endpoint.method().to_uppercase();
        let params: Vec<&str> = endpoint.params.iter().map(|p| p.name.as_str()).collect();
        let path_params: Vec<_> = endpoint
            .params
            .iter()
            .filter(|p| endpoint.path().contains(&format!(":{}", p.name)))
            .collect();
        let body_params: Vec<_> = endpoint
            .params
            .iter()
            .filter(|p| !endpoint.path().contains(&format!(":{}", p.name)))
            .collect();

        // PLAN-013 T2: db 吸收臂——同名 db 实现存在且标量面覆盖 → 委托。
        if let Some(emit) = db_impl
            .as_ref()
            .and_then(|db| merged_db_delegate(db, endpoint))
        {
            code.push_str(&emit);
            continue;
        }

        // PLAN-681 T-02（route A）：真契约桩——参数/返回取 ApiEndpoint 标量
        // 面（str→String 以 &x 委派、int→i64），体委派内嵌 api_impl（端点
        // 体转译=fsys 委派链单源）。非标量参数/返回的端点落后续臂。
        if route_a.is_some()
            && endpoint
                .params
                .iter()
                .all(|p| merged_scalar_rust_ty(&p.ty).is_some())
            && merged_scalar_rust_ty(endpoint.return_type.trim()).is_some()
        {
            let sig = endpoint
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, merged_scalar_rust_ty(&p.ty).unwrap()))
                .collect::<Vec<_>>()
                .join(", ");
            let ret = merged_scalar_rust_ty(endpoint.return_type.trim()).unwrap();
            let call = endpoint
                .params
                .iter()
                .map(|p| {
                    if merged_scalar_rust_ty(&p.ty) == Some("String") {
                        format!("&{}", p.name)
                    } else {
                        p.name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            code.push_str(&format!(
                "fn {}({}) -> {} {{\n    api_impl::{}({})\n}}\n\n",
                fn_name, sig, ret, fn_name, call
            ));
            continue;
        }

        // PLAN-039 T-13（批次 E，E-D5-A）：声明对齐臂——返回型是本批
        // back `pub type`（或其 Vec）时，签名/返回按 back 声明直译
        // （path+body 参数全量、str→String——通用 CRUD 桩此前 path 参数
        // 恒 i32 且 POST/PUT 丢 path 参数，kanban 六端点全面错位=E0061
        // ×5 + Option 索引 ×7 的根因，P666-D1 关联归因）。体 = 空态
        // Default（编译轨生成门形态；此类 app 运行轨在役 = VM 解释，
        // 空态与 kanban empty 判据兼容）。
        {
            let known_types: std::collections::HashSet<&str> =
                module.types.iter().map(|t| t.name.as_str()).collect();
            let ret = endpoint.return_type.trim();
            let base = ret.trim_start_matches("[]").trim_end_matches(']');
            if !ret.is_empty() && ret != "void" && known_types.contains(base) {
                let sig = endpoint
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let rt = auto_type_to_rust(ret);
                let default_expr = if rt.starts_with("Vec<") {
                    "Vec::new()".to_string()
                } else {
                    format!("{}::default()", rt)
                };
                // 空参端点（list_boards）不发 discard——`(,)` 是语法错。
                let discard = if endpoint.params.is_empty() {
                    String::new()
                } else {
                    format!(
                        "    let _ = ({});\n",
                        endpoint
                            .params
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                code.push_str(&format!(
                    "fn {}({}) -> {} {{\n{}    {}\n}}\n\n",
                    fn_name, sig, rt, discard, default_expr
                ));
                continue;
            }
        }

        // Plan 547: the image viewer's merged Rust arm shares the host-owned
        // media control plane with VM instead of manufacturing JSON CRUD
        // records. Keep the generated signatures strongly typed so the Rust
        // UI can use session ids, URI strings, and name vectors directly.
        if endpoint.path().starts_with("/api/viewer/") {
            let sig = body_params
                .iter()
                .map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let body = match fn_name.as_str() {
                "open_file" | "open_directory" | "open_path" => format!(
                    "fn {}({}) -> String {{ auto_media_core::image_pipeline::open_media_session(path) }}\n\n",
                    fn_name, sig
                ),
                "snapshot" => format!(
                    "fn {}({}) -> String {{ auto_media_core::image_pipeline::media_session_snapshot(&session) }}\n\n",
                    fn_name, sig
                ),
                "current_uri" => format!(
                    "fn {}({}) -> String {{ auto_media_core::image_pipeline::media_session_uri(&session) }}\n\n",
                    fn_name, sig
                ),
                "names" => format!(
                    "fn {}({}) -> Vec<String> {{ auto_media_core::image_pipeline::media_session_names(&session) }}\n\n",
                    fn_name, sig
                ),
                "navigate" => format!(
                    "fn {}({}) -> String {{ auto_media_core::image_pipeline::navigate_media_session(&session, delta) }}\n\n",
                    fn_name, sig
                ),
                "request_view" => format!(
                    "fn {}({}) -> String {{ auto_media_core::image_pipeline::request_media_view(&session, index, viewport_width, viewport_height, quality) }}\n\n",
                    fn_name, sig
                ),
                "close_session" => format!(
                    "fn {}({}) -> bool {{ auto_media_core::image_pipeline::close_media_session(&session) }}\n\n",
                    fn_name, sig
                ),
                "image_stats" => format!(
                    "fn {}({}) -> String {{ auto_media_core::image_pipeline::media_session_stats(&session) }}\n\n",
                    fn_name, sig
                ),
                _ => String::new(),
            };
            if !body.is_empty() {
                code.push_str(&body);
                continue;
            }
        }

        match method.as_str() {
            "GET" => {
                // PLAN-710 D-7（T-00 普查回补）：GET 桩按契约签名收口——
                // 此前的 by-id 模板（`fn f(id: i32) -> Option<Value>`）与
                // query 参数契约（`env_str(name str) str` 族）错位，调用点
                // E0061/E0308 + Option Display E0599（auto-edit 018 普查中
                // 被「占位符类型推断塌方」误并档的独立残面——普查报告
                // docs/reports/p710-census.md D-7）。修形：参数/返回取
                // 契约声明（auto_type_to_rust），体沿用 POST 同款请求记录
                // 架构 + 返回缺省（str→""/bool→false/int→0/其它→Null）。
                // 运行期真值由 merged back 服务端供（L2 下游件边界）。
                let sig = endpoint
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret_ty = {
                    let r = endpoint.return_type.trim();
                    if r.is_empty() || r == "void" {
                        "()".to_string()
                    } else {
                        auto_type_to_rust(r)
                    }
                };
                let default_expr = match ret_ty.as_str() {
                    "String" => "String::new()".to_string(),
                    "bool" => "false".to_string(),
                    "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "0".to_string(),
                    "()" => String::new(),
                    _ => "serde_json::Value::Null".to_string(),
                };
                let discard = if endpoint.params.is_empty() {
                    String::new()
                } else {
                    format!(
                        "    let _ = ({});\n",
                        endpoint
                            .params
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                code.push_str(&format!(
                    "fn {}({}) -> {} {{\n{}    {{\n        let mut data = API_DATA.lock().unwrap();\n        let id = {{ let mut next = API_NEXT_ID.lock().unwrap(); *next += 1; *next }};\n        data.push(serde_json::json!({{\"id\": id}}));\n    }}\n    {}\n}}\n\n",
                    fn_name, sig, ret_ty, discard, default_expr
                ));
            }
            "POST" => {
                // PLAN-710 D-7 回补延伸：POST 返回型按契约收口——bool 返回
                // （sync_copy/sync_delete 族）此前恒发 Value（→ 调用点
                // `ok = sync_copy(...)` E0308 expected bool found Value）；
                // bool 契约发 bool 桩（记录请求 + false 缺省），str/其它
                // 维持 Value 形（envelope JSON 契约——regex_replace 先例，
                // json.to_value 包裹消费）。
                let post_ret = {
                    let r = endpoint.return_type.trim();
                    if r == "bool" {
                        "bool".to_string()
                    } else if r == "str" || r == "string" {
                        "String".to_string()
                    } else {
                        "Value".to_string()
                    }
                };
                let body_fields: Vec<String> = body_params
                    .iter()
                    .map(|p| format!("\"{}\": {}", p.name, merged_param_to_value(&p.ty, &p.name)))
                    .collect();
                if post_ret == "bool" {
                    let discard = if body_params.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "    let _ = ({});\n",
                            body_params
                                .iter()
                                .map(|p| p.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    };
                    code.push_str(&format!(
                        "fn {}({}) -> bool {{\n{}    {{\n        let mut data = API_DATA.lock().unwrap();\n        let id = {{ let mut next = API_NEXT_ID.lock().unwrap(); *next += 1; *next }};\n        data.push(serde_json::json!({{\"id\": id}}));\n    }}\n    false\n}}\n\n",
                        fn_name,
                        body_params.iter().map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty))).collect::<Vec<_>>().join(", "),
                        discard
                    ));
                } else if post_ret == "String" {
                    // PLAN-710 D-7：str 契约 POST 桩发 String（记录项序列化
                    // 回传——`json.to_value(regex_replace(...))` 消费链按
                    // 契约 parse；Value 直返型此前错位 = &Value/&str
                    // E0308——regex_replace 株）。
                    code.push_str(&format!(
                        "fn {}({}) -> String {{\n    let mut data = API_DATA.lock().unwrap();\n    let id = {{ let mut next = API_NEXT_ID.lock().unwrap(); *next += 1; *next }};\n    let item = serde_json::json!({{\"id\": id, {}}});\n    data.push(item.clone());\n    serde_json::to_string(&item).unwrap_or_default()\n}}\n\n",
                        fn_name,
                        body_params.iter().map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty))).collect::<Vec<_>>().join(", "),
                        body_fields.join(", ")
                    ));
                } else {
                    code.push_str(&format!(
                        "fn {}({}) -> Value {{\n    let mut data = API_DATA.lock().unwrap();\n    let id = {{ let mut next = API_NEXT_ID.lock().unwrap(); *next += 1; *next }};\n    let item = serde_json::json!({{\"id\": id, {}}});\n    data.push(item.clone());\n    item\n}}\n\n",
                        fn_name,
                        body_params.iter().map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty))).collect::<Vec<_>>().join(", "),
                        body_fields.join(", ")
                    ));
                }
            }
            "PUT" => {
                // PUT is fire-and-forget in the UI (callers don't use the
                // return value), so return () — matching split mode's
                // fire-and-forget PUT. Param types come from the declared type
                // (e.g. []str → Vec<String>), and array params are stored as
                // JSON arrays rather than a single Value::from(String).
                let id_param = path_params.first().map(|p| p.name.as_str()).unwrap_or("id");
                let body_fields: Vec<String> = body_params
                    .iter()
                    .map(|p| {
                        let rhs = merged_param_to_value(&p.ty, &p.name);
                        format!("item[\"{}\"] = {}", p.name, rhs)
                    })
                    .collect();
                code.push_str(&format!(
                    "fn {}({}: i32, {}) {{\n    let mut data = API_DATA.lock().unwrap();\n    if let Some(item) = data.iter_mut().find(|n| n[\"id\"].as_i64() == Some({} as i64)) {{\n        {};\n    }}\n}}\n\n",
                    fn_name, id_param,
                    body_params.iter().map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty))).collect::<Vec<_>>().join(", "),
                    id_param,
                    body_fields.join("; ")
                ));
            }
            "PATCH" => {
                // PATCH: partial update — same structure as PUT (id + body fields).
                let id_param = path_params.first().map(|p| p.name.as_str()).unwrap_or("id");
                let body_fields: Vec<String> = body_params
                    .iter()
                    .map(|p| {
                        let rhs = merged_param_to_value(&p.ty, &p.name);
                        format!("item[\"{}\"] = {}", p.name, rhs)
                    })
                    .collect();
                let all_params: Vec<String> = path_params
                    .iter()
                    .map(|p| format!("{}: i32", p.name))
                    .chain(
                        body_params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty))),
                    )
                    .collect();
                let path_filter: Vec<String> = path_params
                    .iter()
                    .map(|p| format!("n[\"{}\"].as_i64() == Some({} as i64)", p.name, p.name))
                    .collect();
                code.push_str(&format!(
                    "fn {}({}) {{\n    let mut data = API_DATA.lock().unwrap();\n    if let Some(item) = data.iter_mut().find(|n| {}) {{\n        {};\n    }}\n}}\n\n",
                    fn_name,
                    all_params.join(", "),
                    path_filter.join(" && "),
                    body_fields.join("; ")
                ));
            }
            "DELETE" => {
                if path_params.is_empty() {
                    // No path param (e.g. DELETE /api/todos/completed) — operate on all
                    // matching items. Use body params as filter criteria if present.
                    if body_params.is_empty() {
                        // Clear all (unlikely but safe).
                        code.push_str(&format!(
                            "fn {}() {{\n    let mut data = API_DATA.lock().unwrap();\n    data.clear();\n}}\n\n",
                            fn_name
                        ));
                    } else {
                        // Retain items NOT matching the body filter (e.g. done != true).
                        let retain_cond: Vec<String> = body_params
                            .iter()
                            .map(|p| {
                                format!(
                                    "n[\"{}\"] != {}",
                                    p.name,
                                    merged_param_to_value(&p.ty, &p.name)
                                )
                            })
                            .collect();
                        let params_sig: Vec<String> = body_params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty)))
                            .collect();
                        code.push_str(&format!(
                            "fn {}({}) {{\n    let mut data = API_DATA.lock().unwrap();\n    data.retain(|n| {});\n}}\n\n",
                            fn_name,
                            params_sig.join(", "),
                            retain_cond.join(" && ")
                        ));
                    }
                } else {
                    // PLAN-039 T-13（E-D5-A）：签名按声明全参（path 参数
                    // 全量、型按声明 str→String——`delete_card(id str,
                    // cid str)` 此前 `(id: i32)` 丢 cid=E0061 株）；
                    // 体维持 fire-and-forget 内存删（编译轨生成门形态），
                    // id 过滤按首个 path 参数声明型选串/数比较。
                    let all_sig: Vec<String> = endpoint
                        .params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, auto_type_to_rust(&p.ty)))
                        .collect();
                    let first_path = path_params.first();
                    let id_param = first_path.map(|p| p.name.as_str()).unwrap_or("id");
                    let id_is_str = first_path.map(|p| p.ty.trim() == "str").unwrap_or(false);
                    let retain = if id_is_str {
                        format!("n[\"id\"].as_str() != Some({}.as_str())", id_param)
                    } else {
                        format!("n[\"id\"].as_i64() != Some({} as i64)", id_param)
                    };
                    let extra_discard = path_params
                        .iter()
                        .skip(1)
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let discard = if extra_discard.is_empty() {
                        String::new()
                    } else {
                        format!("    let _ = ({});\n", extra_discard)
                    };
                    code.push_str(&format!(
                        "fn {}({}) {{\n{}    let mut data = API_DATA.lock().unwrap();\n    data.retain(|n| {});\n}}\n\n",
                        fn_name,
                        all_sig.join(", "),
                        discard,
                        retain
                    ));
                }
            }
            _ => {
                code.push_str(&format!("fn {}() {{}}\n\n", fn_name));
            }
        }
    }

    code
}

/// Plan 349 step 2 / Plan 388 W2/W3: Generate multipart upload + download
/// (resume + progress) functions for a2r. Mirrors the VM natives
/// `http.upload` / `http.download` / `http.download_resume` /
/// `http.download_with_progress` and the chainable `RequestBuilder.multipart_*`.
fn generate_http_utility_functions() -> String {
    r#"// Plan 349/388: File upload (multipart) + download utilities (a2r)

fn upload_file(url: &str, file_path: &str) -> serde_json::Value {
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let form = match reqwest::blocking::multipart::Form::new()
            .file("file", &file_path) { Ok(f) => f, Err(_) => return serde_json::Value::Null };
        let resp = match _http_client()
            .post(&url)
            .multipart(form)
            .send() { Ok(r) => r, Err(_) => return serde_json::Value::Null };
        let text = match resp.text() { Ok(t) => t, Err(_) => return serde_json::Value::Null };
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
    }).join().unwrap_or(serde_json::Value::Null)
}

fn upload_file_with_fields(url: &str, file_path: &str, fields: &serde_json::Value) -> serde_json::Value {
    let url = url.to_string();
    let file_path = file_path.to_string();
    let fields = fields.clone();
    std::thread::spawn(move || {
        let mut form = reqwest::blocking::multipart::Form::new();
        if let Some(obj) = fields.as_object() {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    form = form.text(k.clone(), s.to_string());
                }
            }
        }
        if let Ok(part) = reqwest::blocking::multipart::Part::file(&file_path) {
            form = form.part("file", part);
        }
        let resp = match _http_client()
            .post(&url)
            .multipart(form)
            .send() { Ok(r) => r, Err(_) => return serde_json::Value::Null };
        let text = match resp.text() { Ok(t) => t, Err(_) => return serde_json::Value::Null };
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
    }).join().unwrap_or(serde_json::Value::Null)
}

// Plan 388 W2: chainable multipart builder — mirrors the VM's
// RequestBuilder.multipart_file(field, path) / multipart_text(field, value).
// Usage: multipart_form().text("title", "x").file("upload", "a.bin").send(url)
fn multipart_form() -> MultiPart {
    MultiPart { fields: Vec::new(), files: Vec::new() }
}

struct MultiPart {
    fields: Vec<(String, String)>,
    files: Vec<(String, String)>,
}

impl MultiPart {
    fn text(mut self, field: &str, value: &str) -> Self {
        self.fields.push((field.to_string(), value.to_string()));
        self
    }

    fn file(mut self, field: &str, path: &str) -> Self {
        self.files.push((field.to_string(), path.to_string()));
        self
    }

    fn send(self, url: &str) -> serde_json::Value {
        let url = url.to_string();
        let fields = self.fields;
        let files = self.files;
        std::thread::spawn(move || {
            let mut form = reqwest::blocking::multipart::Form::new();
            for (k, v) in fields {
                form = form.text(k, v);
            }
            for (k, path) in files {
                match reqwest::blocking::multipart::Part::file(&path) {
                    Ok(part) => form = form.part(k, part),
                    Err(_) => return serde_json::Value::Null,
                }
            }
            let resp = match _http_client()
                .post(&url)
                .multipart(form)
                .send() { Ok(r) => r, Err(_) => return serde_json::Value::Null };
            let text = match resp.text() { Ok(t) => t, Err(_) => return serde_json::Value::Null };
            serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
        }).join().unwrap_or(serde_json::Value::Null)
    }
}

fn download_file(url: &str, file_path: &str) -> bool {
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let resp = match _http_client().get(&url).send() { Ok(r) => r, Err(_) => return false };
        use std::io::Write;
        let mut file = match std::fs::File::create(&file_path) { Ok(f) => f, Err(_) => return false };
        match resp.bytes() {
            Ok(b) => file.write_all(&b).is_ok(),
            Err(_) => false,
        }
    }).join().unwrap_or(false)
}

fn download_file_resume(url: &str, file_path: &str, offset: u64) -> bool {
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let range = format!("bytes={}-", offset);
        let resp = match _http_client()
            .get(&url).header("Range", &range).send() { Ok(r) => r, Err(_) => return false };
        use std::io::Write;
        let mut file = match std::fs::OpenOptions::new().append(true).open(&file_path) {
            Ok(f) => f, Err(_) => return false
        };
        match resp.bytes() {
            Ok(b) => file.write_all(&b).is_ok(),
            Err(_) => false,
        }
    }).join().unwrap_or(false)
}

// Plan 388 W3: non-blocking download with progress events — mirrors the VM's
// http.download_with_progress iterator. Returns an mpsc::Receiver the caller
// polls; each event is {"done": bool, "written": u64, "total": u64, ...}.
fn download_with_progress(url: &str, file_path: &str) -> std::sync::mpsc::Receiver<serde_json::Value> {
    let (tx, rx) = std::sync::mpsc::channel::<serde_json::Value>();
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let mut resp = match _http_client().get(&url).send() {
            Ok(r) => r,
            Err(_) => { let _ = tx.send(serde_json::json!({"done": true, "error": "request failed"})); return; }
        };
        let total = resp.content_length().unwrap_or(0);
        let file = match std::fs::File::create(&file_path) {
            Ok(f) => f,
            Err(_) => { let _ = tx.send(serde_json::json!({"done": true, "error": "open file"})); return; }
        };
        struct ProgressWriter {
            inner: std::fs::File,
            tx: std::sync::mpsc::Sender<serde_json::Value>,
            total: u64,
            written: u64,
        }
        impl std::io::Write for ProgressWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let n = self.inner.write(buf)?;
                self.written += n as u64;
                let _ = self.tx.send(serde_json::json!({
                    "done": false, "written": self.written, "total": self.total
                }));
                Ok(n)
            }
            fn flush(&mut self) -> std::io::Result<()> { self.inner.flush() }
        }
        let mut writer = ProgressWriter { inner: file, tx: tx.clone(), total, written: 0 };
        let ok = resp.copy_to(&mut writer).is_ok();
        let _ = tx.send(serde_json::json!({
            "done": true, "written": writer.written, "total": total, "ok": ok
        }));
    });
    rx
}

"#.to_string()
}

/// Plan 350 step 4 / Plan 388 W4: Generate WebSocket client functions for a2r
/// (ws_connect / ws_send / ws_on_message / ws_close — mirrors the VM's
/// `ws.connect/send/on_message/close`). Incoming text/binary frames are queued
/// per connection and drained by `ws_on_message` (non-blocking, UI-pollable).
fn generate_ws_functions() -> String {
    r#"// Plan 350/388: WebSocket client (a2r)
use std::sync::Mutex;
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref WS_CONNS: Mutex<HashMap<i32, WsConn>> = Mutex::new(HashMap::new());
    static ref WS_NEXT_ID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);
}

struct WsConn {
    sender: Option<std::sync::mpsc::Sender<String>>,
    receiver: std::sync::mpsc::Receiver<String>,
    close: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

fn ws_connect(url: &str) -> i32 {
    let id = WS_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let (msg_tx, msg_rx) = std::sync::mpsc::channel::<String>();
    let close = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let close_reader = close.clone();
    let url = url.to_string();

    std::thread::spawn(move || {
        use tungstenite::Message;
        let (mut socket, _) = match tungstenite::connect(&url) {
            Ok(pair) => pair,
            Err(_) => return,
        };
        // Short read timeout so the loop can service outgoing messages while
        // waiting for incoming frames — a plain blocking read() would deadlock
        // an echo conversation until the peer speaks first (the outgoing
        // channel would never be polled). Applied to the underlying TcpStream
        // for plain and native-TLS (wss://) streams; the `_` arm covers
        // rustls-enabled builds (MaybeTlsStream is #[non_exhaustive]; the
        // Rustls variant is feature-gated and only exists with rustls-tls).
        use tungstenite::stream::MaybeTlsStream;
        let timeout = Some(std::time::Duration::from_millis(50));
        match socket.get_ref() {
            MaybeTlsStream::Plain(tcp) => { let _ = tcp.set_read_timeout(timeout); }
            MaybeTlsStream::NativeTls(tls) => { let _ = tls.get_ref().set_read_timeout(timeout); }
            _ => {}
        }
        loop {
            if close_reader.load(std::sync::atomic::Ordering::SeqCst) { break; }
            // Check for outgoing messages (non-blocking).
            if let Ok(msg) = rx.try_recv() {
                if socket.send(Message::Text(msg.into())).is_err() { break; }
            }
            match socket.read() {
                // Deliver incoming frames to the ws_on_message queue.
                Ok(Message::Text(t)) => { let _ = msg_tx.send(t); }
                Ok(Message::Binary(b)) => { let _ = msg_tx.send(String::from_utf8_lossy(&b).into_owned()); }
                Ok(Message::Close(_)) => break,
                // Read timeout / would-block: no frame ready yet — not a
                // connection error, keep servicing outgoing + waiting.
                Err(e) => {
                    use tungstenite::Error;
                    let is_timeout = matches!(&e, Error::Io(ioe)
                        if ioe.kind() == std::io::ErrorKind::WouldBlock
                        || ioe.kind() == std::io::ErrorKind::TimedOut);
                    if !is_timeout { break; }
                }
                _ => {}
            }
        }
        // Dropping `socket` closes the TCP/WS connection (ws_close relies on this).
    });

    WS_CONNS.lock().unwrap().insert(id, WsConn { sender: Some(tx), receiver: msg_rx, close });
    id
}

fn ws_send(handle: i32, message: &str) -> bool {
    WS_CONNS.lock().unwrap()
        .get(&handle)
        .and_then(|conn| conn.sender.as_ref())
        .and_then(|tx| tx.send(message.to_string()).ok())
        .is_some()
}

fn ws_on_message(handle: i32) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(conn) = WS_CONNS.lock().unwrap().get(&handle) {
        while let Ok(m) = conn.receiver.try_recv() {
            out.push(m);
        }
    }
    out
}

fn ws_close(handle: i32) {
    if let Some(conn) = WS_CONNS.lock().unwrap().get(&handle) {
        // Flag the reader thread to exit (dropping the socket → connection
        // actually closes), then sever the send path.
        conn.close.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    WS_CONNS.lock().unwrap().remove(&handle);
}

"#.to_string()
}

/// Wrap generated components in a main() function with ICED/GPUI backend selection.
fn wrap_example(
    project_name: &str,
    components: &str,
    project_dir: &Path,
    src_: &PacSource,
) -> String {
    let main_widget = extract_main_widget(components);
    let main_msg = format!("{}Msg", main_widget);

    // 剥重复 import + computed 属性字段式访问修补（shell-lib 目标同源
    // 提取件——prepare_generated_components）。
    let cleaned = prepare_generated_components(components);

    // Detect async init: look for __InitLoaded variant in generated code
    let async_init_func = extract_init_api_func(cleaned.trim());

    let iced_entry = if let Some(ref func_name) = async_init_func {
        // Async init: use run_app_with_task_devtools with boot task that loads
        // data in background (Plan 311 P2-A: DevTools-wired counterpart of
        // run_app_with_task, so F12 works for init-API apps like 015-notes).
        // The async {} wrapper ensures spawn_blocking is only called when Iced's Tokio
        // runtime polls the future — NOT eagerly in main() before the runtime starts.
        format!(
            r#"println!("Running with Iced backend");
        let __init = std::cell::RefCell::new(Some(
            iced::Task::perform(
                async {{ tokio::task::spawn_blocking(|| {func_name}()).await.unwrap_or_default() }},
                |r| {main_msg}::__InitLoaded(r)
            )
        ));
        return auto_lang::ui::iced::run_app_with_task_devtools(move || {{
            let task = __init.borrow_mut().take().unwrap_or_else(iced::Task::none);
            ({main_widget}::default(), task)
        }});"#,
            func_name = func_name,
            main_msg = main_msg,
            main_widget = main_widget,
        )
    } else {
        // No async init: use standard run_app. Wrapped in run_app_devtools so
        // F12 opens the rust-mode DevTools inspector (Plan 311).
        format!(
            r#"println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app_devtools::<{main_widget}>();"#,
            main_widget = main_widget,
        )
    };

    // Plan 020 T-05 —— native 桌面客户端臂：孵化参数在册（
    // `--autodesk-client=<pipe>` 直连 / `--autodesk-incubate` broker 孵化）
    // → `client_entry::run_native_client` 协议 client（三态裁决：spawn
    // `--autodesk-render=` 透传 > auto 缺省 = 覆盖扫描制（PLAN-026 T-06
    // 翻转：Covered → queue；NotCovered → 降级 independent 留痕）；
    // pac `desktop_render:` 档由宿主 spawn 侧读取透传，生成物无
    // pac 位置感知）。无标记 = 独立窗现行行为零变化（I1 零删除不变式）。
    // v1 边界：async-init App 经孵化臂以 `default()` 态起（初始化 API
    // 加载不接协议 client 面——超覆盖 App 走 auto 降级 independent）。
    // PLAN-031 T-06：`--autodesk-rqhost` 标记 → `ClientTarget::Rqhost`
    // （rendezvous 采纳内建 + exit-on-EOF 策略档——宿主 spawn 注入
    // `auto run -r rust -q` 旗标族；旧生成物不识此旗标安全忽略）。
    // PLAN-693 T-01/T-03：三模式底座 CLI 前置于旧孵化臂——
    // `--render-mode independent|rq|desktop` + `-q`/`--rq` 糖 +
    // `--desktop-endpoint` + `--window`/`--title`；底座优先级链 CLI >
    // AUTO_VM_RENDER env > pac（spawn 透传 `--autodesk-render=` > exe 旁
    // pac.at）> independent（F-3 关闭口径：remote 族恒显式选用）。rq 臂 =
    // exe 侧 ensure 孵化（`auto run -q` 宿主语义内联）；desktop 臂 = 不
    // 孵化直连（端点缺席报错）；显式 CLI independent 覆盖孵化注入（CLI
    // 顶优先）。同批根修：PLAN-683 给 ClientOpts 加 `remote` 字段时本
    // 生成器未同步——生成 exe 自此 E0063 不可编（日档无生成物编译门，
    // 潜伏在案），旧臂补 `remote: false` 归位。
    let native_client_gate = format!(
        r#"let __autodesk_args: Vec<String> = std::env::args().collect();
        let __has_client = __autodesk_args.iter().any(|a| a.starts_with("--autodesk-client="));
        let __has_incubate = __autodesk_args.iter().any(|a| a == "--autodesk-incubate");
        // PLAN-039 T-07（D2-A）：`--autodesk-launcher` = 宿主召唤链的
        // 编译轨 spawn 形态（spawn_launcher_outproc 编译产物优先臂）——
        // 与 incubate 同走 broker client（pipe 经 --autodesk-broker 传入）。
        let __has_launcher = __autodesk_args.iter().any(|a| a == "--autodesk-launcher");
        // PLAN-693：三模式 CLI 解析（未知参数容错透传；显式已知参数畸形
        // = 报错退出留痕）。
        let __rc = match auto_lang::ui::desktop_protocol::render_cli::parse_render_cli(
            &__autodesk_args,
        ) {{
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        }};
        // CLI --window/--title = 既有 AUTO_VM_WINDOW/TITLE 的 CLI 形
        //（env 覆写——pac 烘焙的 env_inits 在前，CLI 在后覆盖）。
        if let Some((w, h)) = __rc.window {{
            std::env::set_var("AUTO_VM_WINDOW", format!("{{}}x{{}}", w, h));
        }}
        if let Some(t) = &__rc.title {{
            std::env::set_var("AUTO_VM_TITLE", t);
        }}
        // pac 腿：spawn 透传（宿主把 pac desktop_render / --render 注入为
        // `--autodesk-render=`，注入在前用户在后——rev 取后值）> exe 旁
        // pac.at（auto build 部署随行形态）。
        let __pac_leg = __autodesk_args
            .iter()
            .rev()
            .find_map(|a| a.strip_prefix("--autodesk-render=").map(|s| s.to_string()))
            .or_else(auto_lang::ui::desktop_protocol::render_cli::sidecar_pac_desktop_render);
        let (__base, __base_log) = auto_lang::ui::desktop_protocol::render_cli::resolve_render_base(
            __rc.mode,
            std::env::var("AUTO_VM_RENDER").ok().as_deref(),
            __pac_leg.as_deref(),
        );
        if let Some(l) = &__base_log {{
            eprintln!("{{}}", l);
        }}
        match __base {{
            // rq 模式（`-q` 语义内联）：exe 侧孵化保活（探活→spawn auto
            // rqhost→退避）→ rendezvous 采纳 + remote 帧宿主。
            auto_lang::ui::desktop_protocol::render_cli::RenderBase::Rq => {{
                // Box<dyn Error> 的 From<_> 非单射（anyhow/miette 等）——
                // `?`+Into::into 推断二义（E0283 实测）；return 位显式
                // into（旧臂同款，目标型钉死）。
                if let Err(e) = auto_lang::ui::desktop_protocol::rqhost::ensure_rqhost_ready() {{
                    return Err(e.into());
                }}
                let __wellknown = auto_lang::ui::desktop_protocol::rqhost::wellknown_pipe();
                eprintln!("[render] rq 模式：采纳 rqhost daemon（{{__wellknown}}）");
                // --app386= Hello app_name 覆盖（PLAN-694 T-01 认领对称性
                // ——宿主 spawn 按 `--app386=<目录名>` 传身份，认领按目录
                // 名匹配同源；旧臂同款语义）。
                let mut __app_name = "{project_name_snake}".to_string();
                for __a in &__autodesk_args {{
                    if let Some(v) = __a.strip_prefix("--app386=") {{
                        __app_name = v.to_string();
                    }}
                }}
                let (__w, __h) = auto_lang::ui::desktop_protocol::rqhost::vm_window_size();
                let __component = {main_widget}::default();
                let __opts = auto_lang::ui::desktop_protocol::client_entry::ClientOpts {{
                    app_name: __app_name.clone(),
                    title: std::env::var("AUTO_VM_TITLE").unwrap_or_else(|_| __app_name.clone()),
                    width: __w,
                    height: __h,
                    frame_mode: auto_lang::ui::desktop_protocol::message::FrameMode::Commands,
                    auto_downgraded: false,
                    remote: true,
                }};
                return auto_lang::ui::desktop_protocol::client_entry::run_native_client(
                    __component,
                    __opts,
                    auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Rqhost {{
                        wellknown: __wellknown,
                        app_name: __app_name,
                    }},
                )
                .map_err(Into::into);
            }}
            // desktop 模式：虚拟桌面合成器端点——不孵化（桌面已在前），
            // 端点缺席 = 报错列探测过的 wellknown（AC-03）。
            auto_lang::ui::desktop_protocol::render_cli::RenderBase::Desktop => {{
                let Some(__endpoint) = __rc.desktop_endpoint else {{
                    return Err(format!(
                        "desktop 模式需要 --desktop-endpoint <pipe>（未指定；探测 wellknown: {{}}——请先启动虚拟桌面或补参）",
                        auto_lang::ui::desktop_protocol::rqhost::wellknown_pipe(),
                    )
                    .into());
                }};
                eprintln!("[render] desktop 模式：采纳桌面合成器端点（不孵化）");
                // --app386= Hello app_name 覆盖（PLAN-694 T-01 认领对称性
                // ——宿主 spawn 按 `--app386=<目录名>` 传身份，认领按目录
                // 名匹配同源；旧臂同款语义）。
                let mut __app_name = "{project_name_snake}".to_string();
                for __a in &__autodesk_args {{
                    if let Some(v) = __a.strip_prefix("--app386=") {{
                        __app_name = v.to_string();
                    }}
                }}
                let (__w, __h) = auto_lang::ui::desktop_protocol::rqhost::vm_window_size();
                let __component = {main_widget}::default();
                let __opts = auto_lang::ui::desktop_protocol::client_entry::ClientOpts {{
                    app_name: __app_name.clone(),
                    title: std::env::var("AUTO_VM_TITLE").unwrap_or_else(|_| __app_name.clone()),
                    width: __w,
                    height: __h,
                    frame_mode: auto_lang::ui::desktop_protocol::message::FrameMode::Commands,
                    auto_downgraded: false,
                    remote: true,
                }};
                return auto_lang::ui::desktop_protocol::client_entry::run_native_client(
                    __component,
                    __opts,
                    auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Desktop {{
                        endpoint: __endpoint,
                        app_name: __app_name,
                    }},
                )
                .map_err(Into::into);
            }}
            // independent（缺省/显式）：直落独立窗（下行 iced_entry）。
            auto_lang::ui::desktop_protocol::render_cli::RenderBase::Independent => {{}}
        }}
        // 旧孵化注入臂（PLAN-020/031/039 既有宿主编排面零变化；显式 CLI
        // 底座已在上方截走——independent 覆盖孵化注入 = CLI 顶优先语义）。
        if __rc.mode.is_none() && (__has_client || __has_incubate || __has_launcher) {{
            let mut __pipe: Option<String> = None;
            let mut __broker = auto_lang::ui::desktop_protocol::broker::BROKER_PIPE.to_string();
            let mut __render: Option<String> = None;
            let mut __app_name = "{project_name_snake}".to_string();
            for __a in &__autodesk_args {{
                if let Some(v) = __a.strip_prefix("--autodesk-client=") {{
                    __pipe = Some(v.to_string());
                }} else if let Some(v) = __a.strip_prefix("--autodesk-broker=") {{
                    __broker = v.to_string();
                }} else if let Some(v) = __a.strip_prefix("--autodesk-render=") {{
                    __render = Some(v.to_string());
                }} else if let Some(v) = __a.strip_prefix("--app386=") {{
                    __app_name = v.to_string();
                }}
            }}
            if let Some(arg) = __render.as_deref() {{
                if auto_lang::ui::desktop_protocol::coverage::RenderMode::parse(arg).is_none() {{
                    eprintln!("[autodesk-client] 未知 --autodesk-render={{arg}}（auto|queue|independent），回退 auto");
                }}
            }}
            let __mode = auto_lang::ui::desktop_protocol::coverage::RenderMode::resolve(
                __render.as_deref(),
                None,
            );
            // PLAN-026 T-06 翻转：组件先行构造（auto 裁决 = 覆盖扫描制
            // ——queue 优先 + NotCovered 降级 independent 留痕）。
            let __component = {main_widget}::default();
            let (__frame_mode, __downgraded, __log) =
                auto_lang::ui::desktop_protocol::client_entry::resolve_native_frame_mode(
                    __mode,
                    "{main_widget}",
                    &__component.view(),
                );
            if let Some(__l) = &__log {{
                eprintln!("[autodesk-client] {{__l}}");
            }}
            let __rqhost = __autodesk_args.iter().any(|a| a == "--autodesk-rqhost");
            let __target = if __rqhost {{
                // PLAN-031：rqhost 采纳（well-known rendezvous + exit-on-EOF）。
                auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Rqhost {{
                    wellknown: __broker.clone(),
                    app_name: __app_name.clone(),
                }}
            }} else {{ match __pipe {{
                Some(p) => auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Direct(p),
                None => auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Broker {{
                    broker_pipe: __broker,
                }},
            }} }};
            let __opts = auto_lang::ui::desktop_protocol::client_entry::ClientOpts {{
                app_name: __app_name.clone(),
                title: __app_name,
                width: 480.0,
                height: 320.0,
                frame_mode: __frame_mode,
                auto_downgraded: __downgraded,
                // PLAN-683 字段（headless 宿主分岔）——旧孵化编排臂恒
                // false（remote 族走上方三模式臂；E0063 根修归位）。
                remote: false,
            }};
            return auto_lang::ui::desktop_protocol::client_entry::run_native_client(
                __component,
                __opts,
                __target,
            )
            .map_err(Into::into);
        }}"#,
        main_widget = main_widget,
        project_name_snake = to_snake_case(project_name),
    );

    let pac_path = project_dir.join("pac.at");
    let pac_window = src_.window_env(&pac_path);
    let pac_title = src_.title_env(&pac_path);

    let mut env_inits = String::new();
    if let Some(win) = pac_window {
        env_inits.push_str(&format!(
            r#"        if std::env::var("AUTO_VM_WINDOW").is_err() {{
            std::env::set_var("AUTO_VM_WINDOW", "{}");
        }}
"#,
            win
        ));
    }
    if let Some(t) = pac_title {
        env_inits.push_str(&format!(
            r#"        if std::env::var("AUTO_VM_TITLE").is_err() {{
            std::env::set_var("AUTO_VM_TITLE", "{}");
        }}
"#,
            t.replace('\\', "\\\\").replace('"', "\\\"")
        ));
    }

    format!(
        r#"// Auto-generated from Auto language by a2rust-ui

use auto_ui_runtime::ui::{{Component, View}};

// 014 内存哨兵:记账分配器(存活字节 + 大块分配点回溯;超限冻结时
// 自动落盘分配报告到 %TEMP%/auto-term-mem-report.txt)。
#[global_allocator]
static GUARD_ALLOC: auto_lang::ui::mem_guard::GuardAlloc = auto_lang::ui::mem_guard::GuardAlloc;

// PLAN-039 T-12（批次 E）：内建方法 shim 家族——main.rs 组装级注入
// 一次（per-file 注入会撞多 .at 文件拼接的重复定义）。
{at_shims}

{cleaned}

fn main() -> auto_lang::ui::AppResult<()> {{
    #[cfg(feature = "ui-iced")]
    {{
{env_inits}        // Plan 020 T-05：孵化参数在册 → native 协议 client 臂（返回即走）；
        // 无标记 → 独立窗（下行 iced_entry 现行行为零变化）。
        {native_client_gate}
        // PLAN-039 T-04（批次 E 连带）：深视图 app 的栈扩容走 PE
        // /STACK（workspace .cargo/config rustflags）——winit 的
        // Windows EventLoop 要求主线程（any_thread 不可用，线程包裹
        // 形态在 tetris 实测确定性 panic，弃）。
        {iced_entry}
    }}
    // PLAN-691 后随（2026-09-22 用户裁定 auto-lang 侧移除 ui-gpui 休眠
    // feature）：本生成器残留 emission 令全新工程 cargo resolve 即败
    //（`auto-lang does not have that feature`——[features] 表声明
    // `auto-lang/ui-gpui` 被 cargo 惰性校验），GPUI 臂随 feature 一并
    // 摘除，main 收敛为 iced 单底座。
    #[cfg(not(feature = "ui-iced"))]
    {{
        Err("No backend enabled! Use --features ui-iced".into())
    }}}}
"#,
        cleaned = cleaned.trim(),
        at_shims = AT_BUILTIN_METHOD_SHIMS,
        env_inits = env_inits,
        native_client_gate = native_client_gate,
        iced_entry = iced_entry,
    )
}

/// Extract the main widget name from generated components.
/// Looks for "App" struct first, then falls back to the first `pub struct` found.
fn extract_main_widget(components: &str) -> String {
    // Look for "pub struct App"
    for line in components.lines() {
        let trimmed = line.trim();
        if trimmed == "pub struct App {" {
            return "App".to_string();
        }
    }

    // Fallback: find first pub struct
    for line in components.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub struct ") {
            if let Some(name) = rest.split_whitespace().next() {
                // Remove trailing brace if present
                let name = name.trim_end_matches('{').trim();
                return name.to_string();
            }
        }
    }

    // Last resort
    "App".to_string()
}

/// Detect async init by looking for `__InitLoaded` in the generated code.
/// If found, extract the API function name (e.g., "list_notes") that should be
/// called in the background boot task.
fn extract_init_api_func(components: &str) -> Option<String> {
    // Only look for async init if __InitLoaded variant exists
    if !components.contains("__InitLoaded") {
        return None;
    }

    // Find the API function to call: look for `fn list_*()` or `fn get_*()` definitions.
    // These are the GET endpoints that load data for Init.
    for line in components.lines() {
        let trimmed = line.trim();
        // Match patterns like: fn list_notes() -> Vec<...> {
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            if let Some(paren_pos) = rest.find('(') {
                let func_name = &rest[..paren_pos];
                let lower = func_name.to_lowercase();
                if lower.starts_with("list_") || lower.starts_with("get_") {
                    return Some(func_name.to_string());
                }
            }
        }
    }
    None
}

/// Collect all .at files in a directory (non-recursive).
/// PLAN-039 T-04：递归收集 src/front 下全部 .at（跳过 pac.at）——原
/// 实现只读平铺层，`components/`/`pages/` 子目录件全漏（klondike
/// CardSuit/CardFace/CourtBadge 缺失 = 186 错中「cannot find type」族的
/// 根因）。app 仓结构约定（auto-os AGENTS.md §3：src/front = app.at +
/// store + pages/）本就含子目录。确定性：全路径排序。
/// PLAN-681 T-05（F2 bps 依赖解析面）：pac.at `dep <name> { path: "…" }`
/// 声明解析（行式扫描——parse_pac_name 同族；path 相对工程根）。
fn parse_pac_dep_roots(pac_path: &Path, project_dir: &Path) -> Vec<(String, PathBuf)> {
    let mut deps = Vec::new();
    let Ok(code) = fs::read_to_string(pac_path) else {
        return deps;
    };
    let mut current: Option<String> = None;
    for line in code.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("dep ") {
            if let Some(name) = rest.split('{').next().map(|s| s.trim().to_string()) {
                if !name.is_empty() {
                    current = Some(name);
                }
            }
        } else if let (Some(name), Some(rest)) = (&current, t.strip_prefix("path:")) {
            let p = rest.trim().trim_matches('"').trim_matches('\'');
            let root = if Path::new(p).is_absolute() {
                PathBuf::from(p)
            } else {
                project_dir.join(p)
            };
            deps.push((name.clone(), root));
            current = None;
        } else if t.starts_with('}') {
            current = None;
        }
    }
    deps
}

/// PLAN-681 T-05（F2）：`use <dep>.a.b.mod: fn…` 导入收集（符号级）。
/// 返回 (模块文件相对 dep 根的分段路径, 导入符号表)；按模块去重合并。
fn collect_dep_fn_imports(
    at_files: &[PathBuf],
    dep_names: &[String],
) -> Vec<(Vec<String>, Vec<String>)> {
    let mut by_module: std::collections::BTreeMap<Vec<String>, Vec<String>> =
        std::collections::BTreeMap::new();
    for at_path in at_files {
        let Ok(code) = fs::read_to_string(at_path) else {
            continue;
        };
        let session = CompilerSession::ui().with_backend("rust");
        let mut parser = Parser::from(code.as_str()).with_session(session);
        let Ok(ast) = parser.parse() else { continue };
        for stmt in &ast.stmts {
            if let auto_lang_core::ast::Stmt::Use(ref use_stmt) = stmt {
                if let Some(first) = use_stmt.paths.first() {
                    if dep_names.iter().any(|d| d == first.as_str()) && use_stmt.paths.len() > 1 {
                        let segs: Vec<String> = use_stmt
                            .paths
                            .iter()
                            .skip(1)
                            .map(|s| s.as_str().to_string())
                            .collect();
                        let entry = by_module.entry(segs).or_default();
                        for item in &use_stmt.items {
                            let name = item.as_str().to_string();
                            if !entry.contains(&name) {
                                entry.push(name);
                            }
                        }
                    }
                }
            }
        }
    }
    by_module.into_iter().collect()
}

/// PLAN-681 T-05（F2 主入口）：伴生依赖解析——①`use <dep>.…: fn` 的支撑
/// 模块转译内嵌（`mod __bps_*` + 星 use 再导出，调用点裸名零变化）；
/// ②front 源内裸 widget 名本地未声明时，依赖包内同名 widget 的 .at 源
/// 补进编译集（vue 契约须显式 use、VM 轨宽松——rust 轨按 dep 声明 + 名
/// 字面命中补源；机制通用，零消费方标识符硬编码）。
/// 返回 (widget 补集文件, fn 模块内嵌代码)。
fn resolve_bps_dependencies(
    project_dir: &Path,
    at_files: &[PathBuf],
    src_: &PacSource,
) -> (Vec<PathBuf>, String) {
    let pac_path = project_dir.join("pac.at");
    let deps = src_.dep_roots(&pac_path, project_dir);
    if deps.is_empty() {
        return (Vec::new(), String::new());
    }
    let dep_names: Vec<String> = deps.iter().map(|(n, _)| n.clone()).collect();

    // 本地 widget 名集 + 各 front 源文本（名字面命中用）。
    let mut local_widget_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut front_texts: Vec<String> = Vec::new();
    for at_path in at_files {
        let Ok(code) = fs::read_to_string(at_path) else {
            continue;
        };
        front_texts.push(code.clone());
        let session = CompilerSession::ui().with_backend("rust");
        let mut parser = Parser::from(code.as_str()).with_session(session);
        if let Ok(ast) = parser.parse() {
            for stmt in &ast.stmts {
                if let auto_lang_core::ast::Stmt::WidgetDecl(ref wd) = stmt {
                    local_widget_names.insert(wd.name.as_str().to_string());
                }
            }
        }
    }

    // ② widget 补集：依赖包 .at 按名命中（非本地声明且 front 源字面出现）。
    let mut widget_files: Vec<PathBuf> = Vec::new();
    for (_dep_name, root) in &deps {
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    // reference/ = 文档参考工程（demo 壳），不进编译集。
                    if name != "reference" {
                        stack.push(path);
                    }
                } else if path.extension().map(|e| e == "at").unwrap_or(false)
                    && path.file_name().map(|f| f != "pac.at").unwrap_or(true)
                {
                    let Ok(code) = fs::read_to_string(&path) else {
                        continue;
                    };
                    let session = CompilerSession::ui().with_backend("rust");
                    let mut parser = Parser::from(code.as_str()).with_session(session);
                    let Ok(ast) = parser.parse() else { continue };
                    let mut hit = false;
                    for stmt in &ast.stmts {
                        if let auto_lang_core::ast::Stmt::WidgetDecl(ref wd) = stmt {
                            let name = wd.name.as_str();
                            if !local_widget_names.contains(name)
                                && front_texts.iter().any(|t| {
                                    t.split(|c: char| !c.is_alphanumeric() && c != '_')
                                        .any(|w| w == name)
                                })
                            {
                                hit = true;
                            }
                        }
                    }
                    if hit && !widget_files.contains(&path) {
                        widget_files.push(path);
                    }
                }
            }
        }
    }

    // ① fn 模块内嵌。
    let mut fn_code = String::new();
    for (segs, _symbols) in collect_dep_fn_imports(at_files, &dep_names) {
        // 各 dep 根下按分段路径定位模块文件。
        let rel = segs.iter().collect::<PathBuf>();
        let mut module_src: Option<(String, String)> = None; // (stem, content)
        for (dep_name, root) in &deps {
            let candidate = root.join(&rel).with_extension("at");
            if candidate.exists() {
                if let Ok(content) = fs::read_to_string(&candidate) {
                    module_src = Some((format!("__bps_{}_{}", dep_name, segs.join("_")), content));
                    break;
                }
            }
        }
        let Some((mod_name, content)) = module_src else {
            continue;
        };
        // PLAN-681 T-05（N3）：依赖模块源先过 BARE_FN_SIGS 注册（返回型/
        // 形参 kind/bare List 元素形状）——ui_gen 的 computed 返回型推断、
        // 调用实参收口、view 迭代 Value 判定按表消费（先注册后转译,顺序
        // 无关——registry 是 thread_local 静态表）。
        RustGenerator::new().register_bare_fn_sigs_from_source(&content);
        match auto_api_gen::api_gen::transpile_back_module_to_rs(&mod_name, &content) {
            Ok(rs) => {
                let rs = auto_api_gen::api_gen::post_process_companion_rs(rs);
                if !fn_code.is_empty() {
                    fn_code.push('\n');
                }
                fn_code.push_str(&format!(
                    "// PLAN-681 T-05 (F2a): bps use-fn 依赖内联 {}\n",
                    segs.join(".")
                ));
                fn_code.push_str(&format!(
                    "mod {} {{\n#![allow(unused)]\n{}\n}}\nuse {}::*;\n",
                    mod_name, rs, mod_name
                ));
            }
            Err(e) => {
                eprintln!(
                    "  ⚠ bps 模块 {} 转译失败（内联跳过）: {}",
                    segs.join("."),
                    e
                );
            }
        }
    }

    (widget_files, fn_code)
}

fn collect_at_files(dir: &Path) -> AutoResult<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk(&path, files)?;
                continue;
            }
            if path.extension().map(|e| e == "at").unwrap_or(false) {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                // Skip pac.at (project config)
                if file_name == "pac.at" {
                    continue;
                }
                files.push(path);
            }
        }
        Ok(())
    }

    walk(dir, &mut files).map_err(|e| format!("Failed to read dir: {}", e))?;

    files.sort();
    Ok(files)
}

/// Parse project name from pac.at file.
fn parse_pac_name(pac_path: &Path) -> Option<String> {
    let content = fs::read_to_string(pac_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// pac.at `exe_name: "..."` — 生成应用的可执行产物名(014 at-app:
/// `auto-term.exe` 才是正名;缺省 = 包名默认 bin)。合法字符 [A-Za-z0-9_-]。
fn parse_pac_exe_name(pac_path: &Path) -> Option<String> {
    let content = fs::read_to_string(pac_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("exe_name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                let valid = !value.is_empty()
                    && value
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
                if valid {
                    return Some(value.to_string());
                }
                eprintln!("⚠ pac.at exe_name '{value}' 含非法字符(允许 [A-Za-z0-9_-]),忽略");
            }
        }
    }
    None
}

/// Parse window setting from pac.at file (e.g. "fit" or "1280x800").
fn parse_pac_window(pac_path: &Path) -> Option<String> {
    let content = fs::read_to_string(pac_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("window:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Parse display title from pac.at file (prefers title_zh if available).
fn parse_pac_title(pac_path: &Path) -> Option<String> {
    let content = fs::read_to_string(pac_path).ok()?;
    let mut title = None;
    let mut title_zh = None;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("title_zh:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..]
                    .trim()
                    .trim_end_matches(',')
                    .trim_matches('"')
                    .trim_matches('\'');
                if !value.is_empty() {
                    title_zh = Some(value.to_string());
                }
            }
        } else if line.starts_with("title:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..]
                    .trim()
                    .trim_end_matches(',')
                    .trim_matches('"')
                    .trim_matches('\'');
                if !value.is_empty() {
                    title = Some(value.to_string());
                }
            }
        }
    }
    title_zh.or(title)
}

/// pac.at 读源缝（PLAN-809 T-06c / I-PROJECT）。
///
/// rust_ui 的 pac.at 数据字段读取统一经此类型：legacy 路径懒解析文件
/// （行为冻结），请求路径直接供值（生产分派传入 Automan 构造的
/// GenerationRequest——生成器不再回读 pac.at/AutoConfig）。
pub struct PacSource<'a> {
    req: Option<&'a auto_project_model::generation_request::GenerationRequest>,
}

impl<'a> PacSource<'a> {
    pub fn legacy() -> Self {
        Self { req: None }
    }

    pub fn from_request(
        req: &'a auto_project_model::generation_request::GenerationRequest,
    ) -> Self {
        Self { req: Some(req) }
    }

    #[allow(dead_code)]
    pub fn is_request(&self) -> bool {
        self.req.is_some()
    }

    /// PLAN-815 T-06 R1（F-815-T06-R1-01）：请求声明的前端源目录
    /// （`app("front")` 节点形式优先于 `front:` prop——镜像
    /// parse_workspace_path；无声明返回 None=调用方默认 src/front）。
    /// legacy 臂无请求 → None（默认布局语义冻结）。
    pub fn declared_front_dir(&self, project_dir: &Path) -> Option<std::path::PathBuf> {
        let req = self.req?;
        crate::util::request_front_dir(project_dir, req)
    }

    /// T-05 R3（R3-01）：请求携带的 theme{} lite 声明（单源数据——供
    /// internal-vm-exec 执行子进程同链 compose 激活；legacy 返回 None）。
    pub fn theme_decl_lite(
        &self,
    ) -> Option<&auto_project_model::generation_request::ThemeDeclLite> {
        self.req.and_then(|r| r.theme_decl.as_ref())
    }

    pub fn name(&self, pac_path: &Path) -> String {
        match self.req {
            Some(r) => r
                .identity
                .name
                .clone()
                .unwrap_or_else(|| "MyApp".to_string()),
            None => parse_pac_name(pac_path).unwrap_or_else(|| "MyApp".to_string()),
        }
    }

    pub fn exe_name(&self, pac_path: &Path) -> Option<String> {
        match self.req {
            Some(r) => r.exe_name.clone(),
            None => parse_pac_exe_name(pac_path),
        }
    }

    pub fn window_env(&self, pac_path: &Path) -> Option<String> {
        match self.req {
            Some(r) => match r.runtime.window {
                Some(auto_project_model::descriptor::WindowSpec::Size(w, h)) => {
                    Some(format!("{}x{}", w as i64, h as i64))
                }
                Some(auto_project_model::descriptor::WindowSpec::Fit) => Some("fit".to_string()),
                None => None,
            },
            None => parse_pac_window(pac_path),
        }
    }

    pub fn title_env(&self, pac_path: &Path) -> Option<String> {
        // parse_pac_title 语义：title_zh 优先于 title（无 locale 门）。
        match self.req {
            Some(r) => r
                .display
                .title_zh
                .clone()
                .or_else(|| r.display.title.clone()),
            None => parse_pac_title(pac_path),
        }
    }

    pub fn dep_roots(&self, pac_path: &Path, project_dir: &Path) -> Vec<(String, PathBuf)> {
        match self.req {
            Some(r) => r
                .dep_roots
                .iter()
                .map(|d| {
                    let path = match &d.path {
                        Some(p) => {
                            let rel = std::path::Path::new(p);
                            if rel.is_absolute() {
                                rel.to_path_buf()
                            } else {
                                project_dir.join(rel)
                            }
                        }
                        None => project_dir.join(format!("deps/{}", d.name)),
                    };
                    (d.name.clone(), path)
                })
                .collect(),
            None => parse_pac_dep_roots(pac_path, project_dir),
        }
    }

    pub fn api_is_rust(&self, pac_path: &Path) -> bool {
        match self.req {
            Some(r) => r.runtime.api.as_deref() == Some("rust"),
            None => {
                pac_path.is_file() && {
                    auto_lang_core::config::AutoConfig::from_file(pac_path, &auto_val::Obj::new())
                        .ok()
                        .map(|c| c.root.get_prop("api").to_astr() == "rust")
                        .unwrap_or(false)
                }
            }
        }
    }

    pub fn external_backend(&self, project_dir: &Path) -> AutoResult<Option<auto_val::AutoStr>> {
        match self.req {
            Some(r) => Ok(r
                .external_backend
                .clone()
                .map(auto_val::AutoStr::from)
                .filter(|s| !s.is_empty())),
            None => external_backend_of(project_dir),
        }
    }

    /// rust_sidecar 声明（R2-F02：legacy=load_sidecar 文件读；请求路径=
    /// req.raw_nodes 中 rust_sidecar 节点经 core 单源解析，不回读 pac.at）。
    pub fn sidecar_spec(&self, project_dir: &Path) -> auto_api_gen::sidecar::SidecarSpec {
        match self.req {
            Some(r) => {
                // R4-02：完整支持域=节点形态 + 对象/嵌套 prop 形态
                // （req.raw 中 key=="rust_sidecar" 的 Value::Obj/Node——
                // core sidecar_decl_from_root 的 prop 兜底臂同域）。
                let sidecar_nodes: Vec<auto_val::Node> = r
                    .raw_nodes
                    .iter()
                    .filter(|n| n.head == "rust_sidecar")
                    .map(|n| {
                        let mut node = auto_val::Node::new("rust_sidecar");
                        if let Some(name) = &n.name {
                            node.args.add_pos(auto_val::Value::Str(name.clone().into()));
                        }
                        for prop in &n.props {
                            node.set_prop(prop.key.as_str(), prop.value.clone());
                        }
                        node
                    })
                    .collect();
                // R5：prop 形态（Obj/嵌套 Node）**直接挂 root 的 prop**——
                // core sidecar_decl_from_root 的兜底臂读 root.get_prop
                // （R4-02 错把 prop 包进 wrapper kid 再挂 kids，兜底臂永不
                // 命中）。节点形态仍挂 kids（主臂 get_nodes）。
                let mut root = auto_val::Node::new("root");
                for p in r.raw.iter().filter(|p| p.key == "rust_sidecar") {
                    if let Some(v) = &p.value {
                        root.set_prop("rust_sidecar", v.clone());
                    }
                }
                for n in sidecar_nodes {
                    root.add_kid(n);
                }
                auto_api_gen::sidecar::from_decl(&auto_man_core::sidecar_decl_from_root(&root))
            }
            None => auto_api_gen::sidecar::load_sidecar(project_dir),
        }
    }

    /// theme{} 声明激活（compose + set_theme_composed——语言侧效果，
    /// 本方法属 adapter；请求路径从 ThemeDeclLite 还原语言声明）。
    pub fn apply_theme_decl(&self, project_dir: &Path) -> Option<String> {
        match self.req {
            Some(r) => {
                let lite = r.theme_decl.as_ref()?;
                let decl = auto_lang_core::design_tokens::decl::ThemeDecl {
                    name: lite.name.clone(),
                    extends: lite.extends.clone(),
                    mode: lite.mode.clone(),
                    colors: lite.colors.clone(),
                };
                match auto_lang_core::design_tokens::decl::compose(
                    &decl,
                    &std::collections::BTreeMap::new(),
                ) {
                    Ok(theme) => {
                        let name = theme.name.clone();
                        auto_ui_compiler::style::theme::set_theme_composed(std::sync::Arc::new(
                            theme,
                        ));
                        Some(name)
                    }
                    Err(e) => {
                        println!(
                            "{} theme{{}} 合成失败：{e} —— 回退内置缺省",
                            "⚠".bright_yellow()
                        );
                        None
                    }
                }
            }
            None => apply_pac_theme_decl(project_dir),
        }
    }
}

/// Convert CamelCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Generate Cargo.toml content for the Rust UI project (workspace member version).
///
/// No `[workspace]` section — this project is a member of the shared workspace
/// at `examples/rust-workspace/`. Dependencies use `workspace = true` to inherit
/// from the workspace-level `[workspace.dependencies]`.
pub(crate) fn generate_cargo_toml(
    project_name: &str,
    project_dir: &Path,
    src_: &PacSource,
) -> String {
    let snake_name = to_snake_case(project_name);
    // 014: pac.at `exe_name`（pac_bin_block——交付清单同源复用）。
    let bin_block = pac_bin_block(project_dir, src_);

    format!(
        r#"[package]
name = "{snake_name}"
version = "0.1.0"
edition = "2021"
{bin_block}
[features]
ui-iced = ["auto-lang/ui-iced"]
default = ["ui-iced", "auto-lang/default"]

[dependencies]
auto-lang.workspace = true
# PLAN-814 T-05 canonical 双直依赖（import_policy.rs；PLAN-815 T-07 子批2
# 接线）：rust 生成组件只 import `auto_ui_runtime::ui`/`auto_lang_core`，
# 成员须直依赖两包（auto-ui-runtime default 含 ui-iced）。legacy 壳层
# （wrap_example 头/mem_guard/内建 shim）仍发射 auto_lang::，auto-lang
# 直依赖保留至壳层发射 canonical 化。
auto-ui-runtime.workspace = true
auto-lang-core.workspace = true
# PLAN-811：viewer 桥函数直调媒体基础公开入口（auto_media_core::）。
auto-media-core.workspace = true
serde_json.workspace = true
# PLAN-039 T-13（E-D5-A）：back `pub type` struct 发射的 serde derive 依赖。
serde.workspace = true
# PLAN-018:a2r 全局 List 字面量(非 const 初始化)走 once_cell::Lazy 形态
# (trans/rust.rs global_lazy_used;api_gen.rs db 路径同款先例)。
once_cell = "1"
reqwest = {{ version = "0.12", features = ["blocking", "json", "multipart", "cookies", "gzip", "brotli"] }}
tungstenite = {{ version = "0.24", features = ["native-tls"] }}
lazy_static = "1"
tokio.workspace = true
iced.workspace = true
"#
    )
}

/// a2r fix 共用件（wrap_example 原内联体提取——shell-lib 目标同源复用）：
/// ① 剥 RustGenerator 已发的重复 import；② computed 属性字段式访问修补
/// （文件级扫描 `pub fn NAME(&self)` → `self.NAME()` 调用点改写——跨
/// widget computed 同捕）。
fn prepare_generated_components(components: &str) -> String {
    // Strip duplicate imports — RustGenerator already emits them. Both the
    // canonical header (PLAN-814 T-05: `auto_ui_runtime::ui`) and the legacy
    // facade form are stripped: the shell header (wrap_example) provides the
    // single import, and a leftover per-block copy collides as E0252.
    let mut cleaned = components
        .trim()
        .replace("use auto_ui_runtime::ui::{Component, View};\n", "")
        .replace("use auto_ui_runtime::ui::{Component, View};", "")
        .replace("use auto_lang::ui::{Component, View};\n", "")
        .replace("use auto_lang::ui::{Component, View};", "");

    // a2r fix: computed properties are generated as methods (fn name(&self)),
    // but all code accesses them as fields. Scan for `pub fn NAME(&self)` to
    // discover all computed methods, then add () to field-style accesses.
    // This runs at file level so cross-widget computed (store.history accessed
    // from App) is caught regardless of widget generation order.
    let computed_methods: Vec<String> = {
        let mut names = Vec::new();
        for line in cleaned.lines() {
            let trimmed = line.trim();
            // Match: pub fn NAME(&self) -> ... {
            if trimmed.starts_with("pub fn ") && trimmed.contains("(&self)") {
                if let Some(name) = trimmed
                    .strip_prefix("pub fn ")
                    .and_then(|s| s.split('(').next())
                {
                    // Skip obvious non-computed methods (view, on, new, state_snapshot, etc.)
                    if !["view", "on", "new", "state_snapshot", "default", "clone"].contains(&name)
                    {
                        names.push(name.to_string());
                    }
                }
            }
        }
        names
    };
    for cname in &computed_methods {
        for prefix in [format!("self.{}", cname), format!("self.store.{}", cname)] {
            let with_call = format!("{}()", prefix);
            if cleaned.contains(&with_call) {
                continue;
            }
            cleaned = cleaned.replace(&format!("{}.", prefix), &format!("{}().", with_call));
            cleaned = cleaned.replace(&format!("{},", prefix), &format!("{},", with_call));
            cleaned = cleaned.replace(&format!("{})", prefix), &format!("{})", with_call));
            cleaned = cleaned.replace(&format!("{} ", prefix), &format!("{} ", with_call));
        }
    }
    cleaned
}

// ==================== PLAN-036 T-02：shell pack 无窗组件库 ====================
//
// D7 定案：五件 .at（auto-os/shell 权威源）→ lib crate `crates/shell-pack`
// （入库 workspace 成员——每次 cargo build 即真编译门；027 §5.1 D5 入库
// 裁定复活）。产物 = 五组件 + `SHELL_MANIFEST` const（shell_projection
// 同形）+ `mount_face(id, w, h)` 工厂 + 每组件 `ShellStateAccess` 生成
// 实现（string-key 写态 lowering / 召唤事件 dispatch / 命令读走——D8
// 接缝的编译臂）。freshness 门（`test_shell_pack_lib_freshness`）以本
// 生成器对拍入库物——pack 改动后须 `regen_shell_pack`（#[ignore] 测试）
// 重生成 + 提交。

/// 五件清单（与 auto-lang `shell_projection::SHELL_MANIFEST` 同源——
/// id/ widget / pack 文件名）。
const SHELL_LIB_FACES: [(&str, &str, &str); 5] = [
    ("shell", "Desktop", "shell.at"),
    ("desktop", "DesktopSurface", "desktop.at"),
    ("switcher", "Switcher", "switcher.at"),
    (
        "notification_center",
        "NotificationCenter",
        "notification_center.at",
    ),
    ("dashboard", "DashboardPanel", "dashboard.at"),
];

/// 召唤事件全集（`shell_projection::ShellEvent` 名——dispatch 生成臂的
/// 匹配面）。
const SHELL_LIB_EVENTS: [&str; 9] = [
    "RebuildMru",
    "RebuildNotes",
    "RunningSync",
    "ApplyFilter",
    "RebuildFaces",
    // PLAN-036 T-04（D6）：overlay 键盘动词事件位（宿主截获 → 快照
    /// events 下行 → child dispatch → Msg variant）。
    "Advance",
    "Back",
    "Pick",
    "Escape",
];

/// 面元数据（ShellStateAccess 生成实现的材料）。
struct ShellFaceInfo {
    id: &'static str,
    widget: &'static str,
    file: &'static str,
    /// 根 widget 状态字段（名 → rust 类型——镜像 ui_gen/rust.rs 状态推断）。
    states: Vec<(String, String)>,
    /// msg 变体 ∩ 召唤事件全集。
    events: Vec<String>,
}

/// 状态类型推断（忠实镜像 ui_gen/rust.rs：generate_rust 的 state_types
/// 填充[Unknown 初值推断] + `auto_type_to_rust` 递归映射）。返回 None =
/// 不生成写臂（typed 集合等非常用形态——运行时 write 返 false，parity
/// 冒烟可见；不误配字段类型）。
fn shell_state_rust_type(state: &auto_ui_compiler::aura::AuraStateDef) -> Option<String> {
    use auto_lang_core::ast::{Expr, Type};
    if matches!(state.type_info, Type::Unknown) {
        return match &state.initial {
            Expr::Array(_) => Some("Vec<serde_json::Value>".into()),
            Expr::Object(_) => Some("serde_json::Value".into()),
            Expr::Str(_) => Some("String".into()),
            Expr::Int(_) => Some("i32".into()),
            Expr::Float(..) | Expr::Double(..) => Some("f64".into()),
            Expr::Bool(_) => Some("bool".into()),
            // 兜底 = auto_type_to_rust(Unknown)。
            _ => Some("serde_json::Value".into()),
        };
    }
    match &state.type_info {
        Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => Some("String".into()),
        Type::Int => Some("i32".into()),
        Type::Uint => Some("u32".into()),
        Type::I64 => Some("i64".into()),
        Type::U64 => Some("u64".into()),
        Type::Float => Some("f32".into()),
        Type::Double => Some("f64".into()),
        Type::Bool => Some("bool".into()),
        // 未定形元素集合（List/Array/Slice 的 Unknown 内型）→ 生成器实际
        // 产出 Vec<serde_json::Value>（auto_type_to_rust 递归 Unknown 臂）。
        Type::List(inner) | Type::Array(auto_lang_core::ast::ArrayType { elem: inner, .. }) => {
            match inner.as_ref() {
                Type::Unknown => Some("Vec<serde_json::Value>".into()),
                _ => None,
            }
        }
        Type::RuntimeArray(_) => None,
        Type::Unknown => Some("serde_json::Value".into()),
        _ => None,
    }
}

/// 解析 pack 五件的面元数据（每件根 WidgetDecl——`extract_widget_from_
/// decl` 同 vocab 门路径）。
fn collect_shell_face_infos(pack_dir: &Path) -> AutoResult<Vec<ShellFaceInfo>> {
    let mut infos = Vec::new();
    for (id, widget, file) in SHELL_LIB_FACES {
        let path = pack_dir.join(file);
        let src = fs::read_to_string(&path)
            .map_err(|e| format!("shell pack 读取失败 {}: {e}", path.display()))?;
        let session = CompilerSession::ui();
        let mut parser = Parser::from(src.as_str()).with_session(session);
        let ast = parser
            .parse()
            .map_err(|e| format!("shell pack 解析失败 {file}: {e:?}"))?;
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                auto_lang_core::ast::Stmt::WidgetDecl(d) if d.name == widget => Some(d),
                _ => None,
            })
            .ok_or_else(|| format!("shell pack {file} 缺根 widget {widget}"))?;
        let aura = auto_ui_compiler::aura::extract_widget_from_decl(decl)
            .map_err(|e| format!("shell pack {file} widget 提取失败: {e}"))?;
        let states = aura
            .state_vars
            .iter()
            .filter_map(|s| shell_state_rust_type(s).map(|ty| (s.name.clone(), ty)))
            .collect();
        let events = aura
            .messages
            .iter()
            .flat_map(|m| m.variants.iter().map(|v| v.name.clone()))
            .filter(|v| SHELL_LIB_EVENTS.contains(&v.as_str()))
            .collect();
        infos.push(ShellFaceInfo {
            id,
            widget,
            file,
            states,
            events,
        });
    }
    Ok(infos)
}

/// 生成 `ShellStateAccess for {Widget}`（D8 编译臂写态 lowering）。
fn gen_shell_state_access(info: &ShellFaceInfo) -> String {
    let mut code = format!("\nimpl ShellStateAccess for {} {{\n", info.widget);
    // shell_write：标量键 → 类型化字段赋值（auto_val → 字段类型反序列化）。
    code.push_str("    fn shell_write(&mut self, key: &str, value: auto_ui_runtime::ui::auto_val::Value) -> bool {\n");
    code.push_str("        match key {\n");
    for (field, ty) in &info.states {
        let arm = match ty.as_str() {
            "Vec<serde_json::Value>" => format!(
                "            \"{f}\" => {{ if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() {{ self.{f} = v; }} true }}\n",
                f = field
            ),
            _ => format!(
                "            \"{f}\" => {{ if let Ok(v) = value.deserialize_into::<{ty}>() {{ self.{f} = v; }} true }}\n",
                f = field,
                ty = ty
            ),
        };
        code.push_str(&arm);
    }
    code.push_str("            _ => false,\n        }\n    }\n");
    // shell_write_vec：数组键 → Vec<serde_json::Value> 字段。
    code.push_str("    fn shell_write_vec(&mut self, key: &str, values: Vec<auto_ui_runtime::ui::auto_val::Value>) -> bool {\n");
    code.push_str("        match key {\n");
    for (field, ty) in &info.states {
        if ty == "Vec<serde_json::Value>" {
            code.push_str(&format!(
                "            \"{f}\" => {{ self.{f} = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }}\n",
                f = field
            ));
        }
    }
    code.push_str("            _ => false,\n        }\n    }\n");
    // shell_dispatch：召唤事件 → Msg variant on() 派发（027 "宿主写状态
    // 不触发 handler"语义的编译等价——事件名即变体名）。
    code.push_str("    fn shell_dispatch(&mut self, event: &str) {\n        match event {\n");
    for ev in &info.events {
        code.push_str(&format!(
            "            \"{ev}\" => self.on({w}Msg::{ev}),\n",
            w = info.widget
        ));
    }
    code.push_str("            _ => {}\n        }\n    }\n");
    // shell_read_str：String 字段读（__desktop_cmd 读走面）。
    code.push_str(
        "    fn shell_read_str(&self, key: &str) -> Option<String> {\n        match key {\n",
    );
    for (field, ty) in &info.states {
        if ty == "String" {
            code.push_str(&format!(
                "            \"{f}\" => Some(self.{f}.clone()),\n",
                f = field
            ));
        }
    }
    code.push_str("            _ => None,\n        }\n    }\n");
    code.push_str("}\n");
    code
}

/// 生成 lib crate 源（lib.rs + Cargo.toml；freshness 门与 regen 共用——
/// 纯函数无 FS 副作用）。
pub fn generate_shell_pack_lib_sources(pack_dir: &Path) -> AutoResult<(String, String)> {
    let at_files: Vec<PathBuf> = SHELL_LIB_FACES
        .iter()
        .map(|(_, _, f)| pack_dir.join(f))
        .collect();
    let all_stores = collect_store_decls(&at_files);
    let component_fields = collect_component_state_fields(&at_files);
    let component_semantics = collect_component_semantics(&at_files);

    let mut all_components = String::new();
    for at_path in &at_files {
        let file_name = at_path.file_name().unwrap_or_default().to_string_lossy();
        println!("  {} {}", "Parsing".bright_cyan(), file_name);
        let (code, _) = compile_at_file(
            at_path,
            &all_stores,
            &component_fields,
            &component_semantics,
        )?;
        all_components.push_str(&code);
        all_components.push('\n');
    }
    if all_components.trim().is_empty() {
        return Err("shell pack 生成物为空（无 WidgetDecl 组件）".into());
    }
    let cleaned = prepare_generated_components(&all_components);

    let infos = collect_shell_face_infos(pack_dir)?;

    // —— lib.rs ——
    let mut lib = String::new();
    lib.push_str("// Auto-generated from Auto language by a2rust-ui（shell pack 无窗组件库）\n");
    lib.push_str("// DO NOT EDIT - changes will be overwritten\n");
    lib.push_str("// PLAN-036 T-02（D7 定案）：源 = auto-os/shell 五件（入库产物——freshness\n");
    lib.push_str(
        "// 门 `test_shell_pack_lib_freshness` 对拍钉住；pack 改动后跑 regen_shell_pack\n",
    );
    lib.push_str("//（#[ignore] 测试）重生成 + 提交）。\n");
    lib.push_str("#![allow(dead_code, non_snake_case, non_camel_case_types, unused_imports, unused_variables, unused_mut, clippy::all)]\n\n");
    lib.push_str("use auto_ui_runtime::ui::{Component, View};\n");
    lib.push_str("use auto_ui_runtime::ui::desktop_protocol::shell_client::{FaceProjector, ShellStateAccess, ShellSurface};\n\n");
    lib.push_str(&cleaned);
    lib.push_str("\n// —— SHELL_MANIFEST（shell_projection 同形装配清单——D5）——\n");
    lib.push_str(
        "pub use auto_ui_runtime::ui::shell_projection::{ShellFace, ShellManifest, ShellMount};\n\n",
    );
    lib.push_str("pub const SHELL_MANIFEST: ShellManifest = ShellManifest {\n    crate_name: \"shell-pack\",\n    faces: &[\n");
    for info in &infos {
        let mount = if matches!(info.id, "shell" | "desktop") {
            "ShellMount::ResidentBoot"
        } else {
            "ShellMount::LazyOverlay"
        };
        lib.push_str(&format!(
            "        ShellFace {{ id: {:?}, widget: {:?}, mount: {} }},\n",
            info.id, info.widget, mount
        ));
    }
    lib.push_str("    ],\n};\n");
    for info in &infos {
        lib.push_str(&gen_shell_state_access(info));
    }
    // —— mount_face 工厂（D5：id → 装配面；ensure_covered 过门 = 029 前提）——
    lib.push_str("\nfn mount<C: Component + ShellStateAccess + Default + 'static>(\n    width: f32,\n    height: f32,\n) -> Option<Box<dyn ShellSurface>> {\n    let face = FaceProjector::new(C::default(), width, height);\n    face.ensure_covered().ok()?;\n    Some(Box::new(face))\n}\n\n");
    lib.push_str("/// 面装配工厂（id = SHELL_MANIFEST 面 id；尺寸 = 面局部 viewport）。\n");
    lib.push_str("pub fn mount_face(id: &str, width: f32, height: f32) -> Option<Box<dyn ShellSurface>> {\n    match id {\n");
    for info in &infos {
        lib.push_str(&format!(
            "        {:?} => mount::<{}>(width, height),\n",
            info.id, info.widget
        ));
    }
    lib.push_str("        _ => None,\n    }\n}\n");
    lib.push_str("\n#[cfg(test)]\nmod tests;\n");

    // —— Cargo.toml ——
    let cargo = r#"[package]
name = "shell-pack"
version = "0.1.0"
edition = "2021"
# PLAN-036 T-02：a2r 生成物（入库——027 §5.1 D5）；真编译门 = workspace
# 成员编译（每次 cargo build）+ auto-man `test_shell_pack_lib_freshness`
# 字节对拍。lib.rs 为生成物（DO NOT EDIT）；tests.rs 手写（冒烟/写覆盖/
# boot 时延度量行——T-03）。

[dependencies]
# PLAN-814 T-05：shell-pack 生成物 canonical import（lib.rs 同步再生）。
auto-ui-runtime = { path = "../auto-ui-runtime", features = ["ui-iced"] }
auto-lang-core = { path = "../auto-lang-core" }
serde_json.workspace = true
# a2r 全局 List 字面量（once_cell::Lazy 形态——generate_cargo_toml 同注）。
once_cell = "1"
"#
    .to_string();

    Ok((lib, cargo))
}

/// 生成 shell-pack lib crate（权威源 = auto-os/shell；解析序与词汇门同
/// [resolve_os_top_dir]——AUTO_OS_ROOT > 兄弟/主检出）。
pub fn generate_shell_pack_lib(output_dir: &Path) -> AutoResult<()> {
    println!(
        "{}",
        "Generating shell-pack lib crate (a2r windowless target)".bright_cyan()
    );
    let pack_dir = auto_lang_core::os_paths::resolve_os_top_dir(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."),
        "shell",
    )
    .ok_or("auto-os/shell 未解析（solo 检出——AUTO_OS_ROOT 或兄弟/主检出任一在位方可生成）")?;
    let (lib, cargo) = generate_shell_pack_lib_sources(&pack_dir)?;
    let src_dir = output_dir.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create {}: {e}", src_dir.display()))?;
    let lib_rs = src_dir.join("lib.rs");
    fs::write(&lib_rs, &lib).map_err(|e| format!("Failed to write {}: {e}", lib_rs.display()))?;
    let cargo_path = output_dir.join("Cargo.toml");
    fs::write(&cargo_path, &cargo)
        .map_err(|e| format!("Failed to write {}: {e}", cargo_path.display()))?;
    println!(
        "{} shell-pack → {} (lib {} bytes)",
        "  Wrote".bright_green(),
        output_dir.display(),
        lib.len()
    );
    Ok(())
}

/// Write `.cargo/config.toml` with shared target-dir pointing to workspace root's target/.

// ==================== PLAN-792：显式可搬动交付模式 ====================
//
// 目标（docs/plans/792-generated-ui-independent-build.md）：
// 在没有开发主检出/临时 worktree 路径兜底的独立消费者中，真实生成、全新
// 解析并构建一个最小 Iced 工程，再 locked 重放；依赖/资源来源完整可追溯。
//
// 布局（镜像生产仓根布局——auto-lang build.rs 以 `../../crates` 与根
// Cargo.lock 为编译期输入，非镜像布局会在 build.rs unwrap 处红）：
//
//   <delivery_root>/
//     Cargo.toml            # 受管 workspace 根：members=[app+crates/*]、
//                           # default-members=[app]、[workspace.dependencies]
//                           # 表副本、[patch.crates-io] iced_widget（消费根
//                           # patch——不写在依赖包内）
//     Cargo.lock            # 交付阶段 fresh 解析一次后冻结（--locked 重放）
//     app/                  # 生成的应用 member（与开发模式同一生成管线）
//     crates/<closure>      # 受管源码 crate（字节级副本，仅 auto-lang 清单
//                           # 做投影修剪：剥交付闭包外 optional path 依赖）
//     crates/patches/iced_widget/  # 本地 patch 随包发行（PLAN-043 走查）
//     stdlib/、docs/schema/ # 仓库根级编译期输入（include_str! 越界面）
//     assets/               # 资源 allowlist（src/front 非 .at 文件 + hash）
//     delivery-manifest.json # producer 身份/闭包规模/资源与 pin 清单

/// 交付闭包内受管源码 crate（相对 crates/；含 auto-lang 全库——语言源码
/// 尚未拆完，属如实披露的受管包含，非"纯 core/独立 UI 仓"）。
pub const DELIVERY_CRATES: [&str; 26] = [
    "auto-lang",
    "auto-val",
    "auto-atom",
    "auto-lex",
    // PLAN-793 落地后 auto-lang 的必需 path 依赖（诊断基础；其自身依赖
    // auto-val/auto-lex 已在闭包）——merge 整合期闭包追踪（R5 预警兑现）。
    "auto-diagnostics",
    // PLAN-799 落地后 auto-lang 的必需 path 依赖（AST 模型单一来源；其
    // 自身依赖 auto-val/auto-lex 已在闭包）——后合入者按合流合同窄接线
    //（docs/plans/reports/model-decoupling-parallel-order-20261016.md）。
    "auto-ast",
    // PLAN-801 落地后 auto-lang 的必需 path 依赖（方言端口 API；其自身
    // 依赖 auto-ast/auto-lex/auto-diagnostics 已在闭包）——沿 792 合流
    // 合同窄接线（extraction-next-parallel-order-20261017 §文件所有权）。
    "auto-parser-api",
    // PLAN-806 落地后 auto-lang 的必需 path 依赖（UI msg/model 声明文法
    // 单一份实现；其自身依赖 auto-parser-api 已在闭包）——沿 792 合流
    // 合同窄接线。
    "auto-ui-dialect",
    // PLAN-807 落地后 auto-lang 的必需 path 依赖（文本基础：rope/分页/
    // 快照/diff 单一来源；其唯一三方依赖 imara-diff 走 registry）——沿
    // 792 合流合同窄接线，不登记则交付根 fresh 解析红。
    "auto-text-core",
    // PLAN-808 落地后 auto-lang 的必需 path 依赖（terminal 域核心：owned
    // cell/geometry 合同 + DLL 服务 + 34 方法端口；自身闭包仅 libloading，
    // 由 terminal-core-consumer/verify-deps.sh 钉住）。
    "auto-terminal-core",
    // PLAN-811 落地后 auto-lang 的必需 path 依赖（媒体基础：图像资产
    // 管线/媒体文件服务；其三方依赖 image/kamadak-exif/blake3/fastrand
    // 走 registry）——沿 792 合流合同窄接线，不登记则交付根 fresh 解析红。
    "auto-media-core",
    "auto-cache",
    "shim-metadata",
    "auto-bindgen",
    "auto-macros",
    "a2r-std",
    // PLAN-814 T-02 落地后 auto-lang 的必需 path 依赖（纯语言 SDK：非 UI
    // 语言栈单源；闭包与上述成员同构）——沿 792 合流合同窄接线。
    "auto-lang-core",
    // PLAN-814 T-03 落地后 auto-lang 的必需 path 依赖（UI 编译域：style/
    // aura/a2ui/四生成器/registry；其依赖 auto-lang-core 已在闭包）——沿
    // 792 合流合同窄接线，不登记则交付根 fresh 解析红。
    "auto-ui-compiler",
    // PLAN-814 T-01 骨架起 auto-lang 的必需 path 依赖（UI 运行域骨架——
    // B/T-04 迁入真实 renderer/宿主 adapter；当前为文档骨架，闭包零额外面）。
    "auto-ui-runtime",
    // PLAN-815 T-04 起受管交付随包发行 UI 工具链（合同 §1 增补
    // auto-tool-protocol/auto-ui-tools；工具的生成主体闭包新增
    // auto-api-gen（api 生成单算法）与 auto-man-core（rust_sidecar 声明
    // 单源）；R1-F01 补 auto-project-model（协议 crate 的 path 依赖）与
    // auto-toolchain-core（auto-man-core 的 path 依赖）——六者缺一则
    // 交付根 fresh metadata/lock 即红（闭包完备性由 delivery 测试静态钉住）。
    "auto-tool-protocol",
    "auto-ui-tools",
    // PLAN-815 T-05 R1（F01）：owned 子进程执行设施（std-only）——工具
    // 托管 spawn 的树终止/收尸原语，交付根复算生成/运行闭包必需。
    "auto-tool-execution",
    "auto-api-gen",
    "auto-man-core",
    "auto-project-model",
    "auto-toolchain-core",
];

/// 随包发行的本地 patch（相对 crates/patches/）。
pub const DELIVERY_PATCHES: [&str; 1] = ["iced_widget"];

/// 仓库根级编译期输入（auto-lang include_str! 越界引用面：stdlib_assembly
/// 的 loader/manifest/providers + native_registry + aura/schema_loader——
/// 缺一即受管 build 红，受管阶段判红不回退主检出）。
const DELIVERY_ROOT_INPUTS: [&str; 5] = [
    "stdlib/assembly-providers.json",
    "stdlib/auto/image.at",
    "stdlib/auto/image.rs.at",
    "stdlib/auto/image.vm.at",
    "docs/schema/aura.at",
];

/// 投影修剪 spec：vendored manifest 剥除的 optional path 依赖行（不在交付
/// 闭包内且非默认 feature 启用；cargo 对成员的 optional path 依赖仍读
/// manifest，不剥则解析红）+ feature 行改写（保留 feature 名维持 cfg 面
/// 与零新警告，仅剥 dep:/转发引用）。
///
/// PLAN-814 SD-08：producer 包图更新——auto-lang 的 test-cases 行自 T-01
/// 起带 SDK 转发项（auto-lang-core/test-cases）；auto-lang-core 自身清单
/// 有独立 test-cases/auto-test-case 面。按 manifest 分 spec，漂移判红
/// （编辑计数 ≠ spec 全量）与幂等（再交付零编辑）按各自 spec 判定。
struct DeliveryProjectionSpec {
    strip_deps: &'static [&'static str],
    feature_rewrites: &'static [(&'static str, &'static str)],
}

const PROJECTION_AUTO_LANG: DeliveryProjectionSpec = DeliveryProjectionSpec {
    // PLAN-815 T-05 门禁附带修复（披露）：814-T-04 把跨仓 autodown-core
    // optional path 依赖与 dep: 特征形态随迁 auto-ui-runtime——本 spec 的
    // autodown 剥除/改写两项陈旧化（master 端到端即红），移除；runtime
    // 清单的对应投影见 PROJECTION_AUTO_UI_RUNTIME。
    strip_deps: &["auto-test-case = { path = \"../auto-test-case\", optional = true }"],
    feature_rewrites: &[(
        "test-cases = [\"dep:auto-test-case\", \"auto-lang-core/test-cases\"]",
        "test-cases = []",
    )],
};

const PROJECTION_AUTO_UI_RUNTIME: DeliveryProjectionSpec = DeliveryProjectionSpec {
    // 814-T-04 随迁的跨仓 optional 依赖（原 auto-lang 位置），投影语义
    // 原样承接：剥 autodown-core 路径行，autodown 特征改写为无 dep: 形态。
    strip_deps: &[
        "autodown-core = { path = \"../../../auto-down/autodown/packages/engine/rust\", optional = true }",
    ],
    feature_rewrites: &[(
        "autodown = [\"ui-iced\", \"dep:autodown-core\"]",
        "autodown = [\"ui-iced\"]",
    )],
};

const PROJECTION_AUTO_LANG_CORE: DeliveryProjectionSpec = DeliveryProjectionSpec {
    strip_deps: &["auto-test-case = { path = \"../auto-test-case\", optional = true }"],
    feature_rewrites: &[("test-cases = [\"dep:auto-test-case\"]", "test-cases = []")],
};

/// 历史漂移家族（779/785 记录的 fresh-lock 失败面：wgpu-hal 宏边界 E0277
/// 源于图形栈新补丁流入）：交付应用清单按 producer 已验证的应用锁
/// （examples/rust-workspace/Cargo.lock）钉扎单版本槽成员。iced_widget 不
/// 入列（patch 单源控制）；多版本槽成员（如 windows/windows-core 三槽并存）
/// 无法以清单直依赖钉扎（TOML 单键），由冻结锁控制并显式登记。
pub const DELIVERY_PIN_FAMILY: [&str; 14] = [
    "iced",
    "iced_core",
    "iced_futures",
    "iced_graphics",
    "iced_renderer",
    "iced_runtime",
    "iced_tiny_skia",
    "iced_wgpu",
    "wgpu",
    "wgpu-core",
    "wgpu-hal",
    "wgpu-types",
    // 多槽恒定成员（0.57/0.58/0.62 三槽并存）：清单无法对同名多槽包重复
    // 声明直依赖——永不进 pins，恒落 pins_multi_slot_lock_frozen 披露。
    "windows",
    "windows-core",
];

/// 交付目录拷贝时跳过的目录名（与 auto-lang build.rs 哈希漫步的跳过面同源：
/// 构建闭包不含这三类产物）。
const DELIVERY_COPY_SKIP_DIRS: [&str; 3] = ["target", ".git", "node_modules"];

/// 受管文件记录（delivery-manifest.json 的 crates/root_inputs/resources 条目）。
#[derive(Clone)]
struct DeliveryFileRecord {
    rel: String,
    bytes: u64,
    blake3: String,
}

/// 单个受管目录（crate/patch/资源树）的拷贝结果。
struct DeliveryCopiedDir {
    records: Vec<DeliveryFileRecord>,
    digest: String,
}

/// 递归拷贝 `src` → `dst`，跳过 `skip` 命名的目录；返回逐文件记录与目录
/// digest（对排序后的 "rel\0len\nblake3\n" 清单再 blake3——目录身份与机器/
/// 检出无关）。
fn delivery_copy_dir(src: &Path, dst: &Path, skip: &[&str]) -> AutoResult<DeliveryCopiedDir> {
    let mut records: Vec<DeliveryFileRecord> = Vec::new();
    fn walk(
        src: &Path,
        dst: &Path,
        rel: &str,
        skip: &[&str],
        records: &mut Vec<DeliveryFileRecord>,
    ) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            let child_rel = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            if path.is_dir() {
                if skip.contains(&name.as_str()) {
                    continue;
                }
                walk(&path, &dst.join(&name), &child_rel, skip, records)?;
            } else {
                let content = fs::read(&path)?;
                fs::write(dst.join(&name), &content)?;
                records.push(DeliveryFileRecord {
                    rel: child_rel,
                    bytes: content.len() as u64,
                    blake3: blake3::hash(&content).to_hex().to_string(),
                });
            }
        }
        Ok(())
    }
    walk(src, dst, "", skip, &mut records)
        .map_err(|e| format!("delivery: 拷贝 {} 失败: {e}", src.display()))?;
    records.sort_by(|a, b| a.rel.cmp(&b.rel));
    let digest = delivery_dir_digest(&records);
    Ok(DeliveryCopiedDir { records, digest })
}

/// 目录 digest：对排序后逐文件 "rel\0len\nblake3\n" 串接再 blake3。
fn delivery_dir_digest(records: &[DeliveryFileRecord]) -> String {
    let mut acc = String::new();
    for r in records {
        acc.push_str(&r.rel);
        acc.push('\0');
        acc.push_str(&r.bytes.to_string());
        acc.push('\n');
        acc.push_str(&r.blake3);
        acc.push('\n');
    }
    blake3::hash(acc.as_bytes()).to_hex().to_string()
}

/// 交付模式 producer 源根解析序（仅用于定位**待拷贝源**，解析结果不进入
/// 生成物）：AUTO_DELIVERY_SOURCE_ROOT → AUTO_LANG_ROOT → **CLI 可执行
/// 文件邻接 sdk/**（PLAN-816 B-R4-07：安装形态默认接线——安装目录随附
/// SDK 闭包（双二进制+sdk/ 布局，install-manifest），生产链不借实施树；
/// 显式 env 覆盖保持开发优先）→ CWD 仓库漫步（既有 find_lang_repo_root，
/// 兄弟/主检出）→ 编译期 CARGO_MANIFEST_DIR 上溯。候选必须含
/// crates/auto-lang 与根 Cargo.toml，否则受管判红。
fn delivery_producer_root() -> AutoResult<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(p) = std::env::var_os("AUTO_DELIVERY_SOURCE_ROOT") {
        candidates.push(PathBuf::from(p));
    }
    if let Some(p) = std::env::var_os("AUTO_LANG_ROOT") {
        candidates.push(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("sdk"));
        }
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(root) = find_lang_repo_root(&cwd) {
        candidates.push(root);
    }
    if let Some(root) = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
    {
        candidates.push(root.to_path_buf());
    }
    for c in candidates {
        if c.join("crates").join("auto-lang").is_dir() && c.join("Cargo.toml").is_file() {
            return Ok(c);
        }
    }
    Err("delivery: producer 源根未解析（需含 crates/auto-lang 与根 Cargo.toml；可用 AUTO_DELIVERY_SOURCE_ROOT 钉死）".into())
}

/// producer git 身份（best-effort：非 git 检出记录 null）。
fn delivery_git_identity(root: &Path) -> (Option<String>, bool) {
    let run = |args: &[&str]| -> Option<String> {
        std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let head = run(&["rev-parse", "HEAD"]).filter(|s| !s.is_empty());
    let dirty = run(&["status", "--porcelain"])
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    (head, dirty)
}

/// producer ui_gen 指纹（与 auto-ui-compiler build.rs 的
/// AUTO_UI_GEN_FINGERPRINT 同文件集：crates/auto-ui-compiler/src/ui_gen/**
/// ——PLAN-814 T-03 迁移后单源所在；blake3 over 排序 rel+内容，本指纹为
/// 交付 manifest 的 producer 身份戳，算法独立于 build.rs 的 FNV 门）。
fn delivery_ui_gen_fingerprint(root: &Path) -> AutoResult<String> {
    let dir = root
        .join("crates")
        .join("auto-ui-compiler")
        .join("src")
        .join("ui_gen");
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    fn walk(dir: &Path, rel: &str, out: &mut Vec<(String, Vec<u8>)>) {
        let Ok(rd) = fs::read_dir(dir) else {
            return;
        };
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let child_rel = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            let path = entry.path();
            if path.is_dir() {
                walk(&path, &child_rel, out);
            } else if let Ok(content) = fs::read(&path) {
                out.push((child_rel, content));
            }
        }
    }
    walk(&dir, "", &mut entries);
    if entries.is_empty() {
        return Err("delivery: producer ui_gen 源目录为空（源根可疑）".into());
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut acc = Vec::new();
    for (rel, content) in &entries {
        acc.extend_from_slice(rel.as_bytes());
        acc.push(0);
        acc.extend_from_slice(&(content.len() as u64).to_le_bytes());
        acc.push(0xFF);
        acc.extend_from_slice(content);
    }
    Ok(blake3::hash(&acc).to_hex().to_string())
}

/// 文本抽取根 manifest 的一个 `[section]`（头行到下一个顶层 `[` 前，字节
/// 原样——表副本与 producer 逐字节同源）。
fn delivery_extract_toml_section(root_manifest: &str, header: &str) -> AutoResult<String> {
    let mut out: Option<String> = None;
    let mut capturing = false;
    for line in root_manifest.lines() {
        if capturing {
            if line.starts_with('[') {
                break;
            }
            out.as_mut().unwrap().push_str(line);
            out.as_mut().unwrap().push('\n');
        } else if line.trim() == header {
            out = Some(format!("{header}\n"));
            capturing = true;
        }
    }
    out.ok_or_else(|| format!("delivery: producer 根 manifest 缺 {header} 段").into())
}

/// 从 producer 应用锁（examples/rust-workspace/Cargo.lock）提取
/// `[[package]]` name→versions。
fn delivery_lock_versions(lock_text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut name: Option<String> = None;
    for line in lock_text.lines() {
        let t = line.trim();
        if t.starts_with("[[package]]") {
            name = None;
        } else if let Some(v) = t.strip_prefix("name = ") {
            name = Some(v.trim_matches('"').to_string());
        } else if let Some(v) = t.strip_prefix("version = ") {
            if let Some(n) = &name {
                out.push((n.clone(), v.trim_matches('"').to_string()));
                name = None;
            }
        }
    }
    out
}

/// 交付 pin 集：单版本槽家族成员 → `=ver` 精确钉（直依赖边，仅约束解析
/// 不增 feature）；多版本槽成员 → 显式登记为"冻结锁控制"（清单无法对同
/// 名多槽包重复声明直依赖）。
fn delivery_pins(producer_root: &Path) -> AutoResult<(Vec<(String, String)>, Vec<String>)> {
    // 根 Cargo.lock（本地实际输入——Cargo.lock 整体被 .gitignore 忽略、
    // examples/rust-workspace/ 同样不保证在位；消费其内容并以 SHA-256
    // 记入 manifest 绑定身份，图形栈版本与应用锁经比对一致）。
    let lock_path = producer_root.join("Cargo.lock");
    let lock_text = fs::read_to_string(&lock_path).map_err(|e| {
        format!(
            "delivery: producer 应用锁不可读（{}）: {e}",
            lock_path.display()
        )
    })?;
    let versions = delivery_lock_versions(&lock_text);
    let mut pins = Vec::new();
    let mut multi_slot = Vec::new();
    for fam in DELIVERY_PIN_FAMILY {
        let mut slots: Vec<String> = versions
            .iter()
            .filter(|(n, _)| n == fam)
            .map(|(_, v)| v.clone())
            .collect();
        slots.sort();
        slots.dedup();
        match slots.len() {
            0 => {}
            1 => pins.push((fam.to_string(), format!("={}", slots[0]))),
            _ => multi_slot.push(fam.to_string()),
        }
    }
    if pins.is_empty() {
        return Err("delivery: pin 家族在 producer 应用锁中零命中（锁可疑）".into());
    }
    Ok((pins, multi_slot))
}

/// 交付根 workspace manifest（消费根 patch 就位于此，不写在依赖包内）。
fn generate_delivery_root_manifest(producer_root: &Path, app_member: &str) -> AutoResult<String> {
    let root_manifest = fs::read_to_string(producer_root.join("Cargo.toml"))
        .map_err(|e| format!("delivery: producer 根 manifest 不可读: {e}"))?;
    let ws_deps = delivery_extract_toml_section(&root_manifest, "[workspace.dependencies]")?;
    let profile_dev = delivery_extract_toml_section(&root_manifest, "[profile.dev]")?;
    let mut members: Vec<String> = Vec::new();
    members.push(format!("    \"app\""));
    for c in DELIVERY_CRATES {
        members.push(format!("    \"crates/{c}\""));
    }
    let members_toml = members.join(",\n");
    Ok(format!(
        r#"# PLAN-792 受管交付根（显式可搬动交付模式——生成物，勿手改）。
# 镜像生产仓布局：members 含应用与受管源码闭包（[workspace.dependencies]
# 表为 producer 根表逐字节副本）；plain `cargo build` 只建 app。
[workspace]
members = [
{members_toml}
]
default-members = ["{app_member}"]
resolver = "2"

{ws_deps}
{profile_dev}
# 顶层 patch（PLAN-043 走查 2026-09-25）：iced_widget 本地补丁随包发行——
# patch 写在消费根，不写在依赖包内；删此段则 --locked 重放确定性红。
[patch.crates-io]
iced_widget = {{ path = "crates/patches/iced_widget" }}
"#
    ))
}

/// 交付应用 manifest：自包含（无 workspace 继承）——语言 crate 以受管本地
/// source bundle 相对路径解析；registry 依赖规格与开发模式
/// generate_cargo_toml 同源同规格。
fn generate_delivery_app_manifest(
    project_name: &str,
    project_dir: &Path,
    pins: &[(String, String)],
    src_: &PacSource,
) -> String {
    let snake_name = to_snake_case(project_name);
    let bin_block = pac_bin_block(project_dir, src_);
    // 与基础依赖同名的 pin（如 iced——TOML 单键不可重复声明）直接并入基础
    // 行用精确版本；pin 块只列其余家族成员。
    const BASE_KEYS: [&str; 9] = [
        "serde_json",
        "serde",
        "once_cell",
        "reqwest",
        "tungstenite",
        "lazy_static",
        "tokio",
        "iced",
        "auto-lang",
    ];
    let pin_of = |n: &str| pins.iter().find(|(pn, _)| pn == n).map(|(_, v)| v.clone());
    let iced_ver = pin_of("iced").unwrap_or_else(|| "0.14".to_string());
    let pin_lines: Vec<String> = pins
        .iter()
        .filter(|(n, _)| !BASE_KEYS.contains(&n.as_str()))
        .map(|(n, v)| format!("{n} = {{ version = \"{v}\", default-features = false }}"))
        .collect();
    let pin_block = if pin_lines.is_empty() {
        String::new()
    } else {
        format!(
            "# PLAN-792 drift pins（779/785 fresh-lock 失败家族；来源 = producer\n# 根 Cargo.lock 已验证版本，default-features=false 仅约束解析不增\n# feature 面；iced 等与基础依赖同名者并入基础行精确版本）。\n{}\n",
            pin_lines.join("\n")
        )
    };
    format!(
        r#"# PLAN-792 交付应用 member（生成物）——自包含清单：不继承 workspace、
# 不隐式搜索开发主检出/编译期 CARGO_MANIFEST_DIR；所有本地 path 依赖均在
# 交付根内。
[package]
name = "{snake_name}"
version = "0.1.0"
edition = "2021"
{bin_block}
[features]
ui-iced = ["auto-lang/ui-iced"]
default = ["ui-iced", "auto-lang/default"]

[dependencies]
auto-lang = {{ path = "../crates/auto-lang" }}
# PLAN-814 T-05 canonical 双直依赖（import_policy.rs；PLAN-815 T-07 子批2
# 接线）：交付 app 的生成组件只 import `auto_ui_runtime::ui`/
# `auto_lang_core`——直接依赖同交付根内两包（闭包成员 814 T-02/T-03 已
# 随包；缺此两行则 fresh/locked 真实构建 E0433，792 driver V01 在案）。
# auto-ui-runtime default 含 ui-iced，无需 feature 转发。
auto-ui-runtime = {{ path = "../crates/auto-ui-runtime" }}
auto-lang-core = {{ path = "../crates/auto-lang-core" }}
# PLAN-811：viewer 桥函数直接消费媒体基础公开入口（同交付根内源闭包）。
auto-media-core = {{ path = "../crates/auto-media-core" }}
serde_json = "1"
# PLAN-039 T-13（E-D5-A）：back `pub type` struct 发射的 serde derive 依赖。
serde = {{ version = "1", features = ["derive"] }}
# PLAN-018:a2r 全局 List 字面量(非 const 初始化)走 once_cell::Lazy 形态
# (trans/rust.rs global_lazy_used;api_gen.rs db 路径同款先例)。
once_cell = "1"
reqwest = {{ version = "0.12", features = ["blocking", "json", "multipart", "cookies", "gzip", "brotli"] }}
tungstenite = {{ version = "0.24", features = ["native-tls"] }}
lazy_static = "1"
tokio = {{ version = "1", features = ["rt"] }}
iced = {{ version = "{iced_ver}", features = ["tokio", "advanced"] }}
{pin_block}"#
    )
}

/// pac.at `exe_name` → 显式 `[[bin]]` 块（014；generate_cargo_toml 同源）。
fn pac_bin_block(project_dir: &Path, src_: &PacSource) -> String {
    src_.exe_name(&project_dir.join("pac.at"))
        .map(|exe| format!("\n[[bin]]\nname = \"{exe}\"\npath = \"src/main.rs\"\n"))
        .unwrap_or_default()
}

/// 投影修剪 vendored auto-lang/Cargo.toml：剥闭包外 optional path 依赖行
///（连带其紧邻上行注释），feature 行改写为无 dep: 引用形态。幂等：已修剪
/// 的清单二次运行零改动。
fn delivery_apply_projection(
    vendored_manifest: &Path,
    spec: &DeliveryProjectionSpec,
) -> AutoResult<Vec<String>> {
    let text = fs::read_to_string(vendored_manifest)
        .map_err(|e| format!("delivery: 读 vendored manifest 失败: {e}"))?;
    let mut edits = Vec::new();
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        if spec.strip_deps.iter().any(|m| line.contains(m)) {
            // Consecutively pop out all immediately adjacent comment lines above (belonging to the stripped dep's explanation block)——avoid dangling comments in the vendored copy
            while out
                .last()
                .map(|l: &String| l.trim().starts_with('#'))
                .unwrap_or(false)
            {
                out.pop();
            }
            edits.push(format!("strip dep: {}", line.trim()));
            continue;
        }
        let mut line = line.to_string();
        for &(from, to) in spec.feature_rewrites {
            if line.trim() == from {
                line = line.replace(from, to);
                edits.push(format!("rewrite feature: {from} -> {to}"));
            }
        }
        out.push(line);
    }
    let joined = out.join("\n");
    let joined = if text.ends_with('\n') {
        format!("{joined}\n")
    } else {
        joined
    };
    if joined != text {
        fs::write(vendored_manifest, &joined)
            .map_err(|e| format!("delivery: 写投影 manifest 失败: {e}"))?;
    }
    let expected = spec.strip_deps.len() + spec.feature_rewrites.len();
    if edits.len() == expected {
        return Ok(edits);
    }
    // 已投影形态（再交付/幂等重放）：strip 标记与改写前形态全缺席 + 改写后
    // 形态全在位 → Ok（零编辑）；否则判 producer 清单结构漂移。
    let already_projected = spec.strip_deps.iter().all(|m| !text.contains(m))
        && spec
            .feature_rewrites
            .iter()
            .all(|(from, _)| !text.contains(from))
        && spec
            .feature_rewrites
            .iter()
            .all(|(_, to)| text.contains(to));
    if already_projected {
        return Ok(Vec::new());
    }
    Err(format!(
        "delivery: 投影修剪不完整（期望 {expected} 处，实际 {} 处）——producer 清单结构漂移，受管判红",
        edits.len()
    )
    .into())
}

/// 交付根守卫：空目录或既往交付（有 delivery-manifest.json 标记）才允许；
/// 外来非空目录显式拒绝，不覆盖他者内容。（R1-F02：换装语义——不再在
/// 生成前清理既有交付；旧包经 staging 成功后整体换出。）
fn delivery_guard_root(delivery_root: &Path) -> AutoResult<()> {
    if !delivery_root.exists() {
        return Ok(());
    }
    if !delivery_root.is_dir() {
        return Err(format!("delivery: 交付根不是目录：{}", delivery_root.display()).into());
    }
    let marker = delivery_root.join("delivery-manifest.json");
    if marker.is_file() {
        return Ok(());
    }
    let empty = fs::read_dir(delivery_root)
        .map(|mut it| it.next().is_none())
        .unwrap_or(false);
    if empty {
        return Ok(());
    }
    Err(format!(
        "delivery: 交付根非空且无 delivery-manifest.json（外来目录，拒绝覆盖）：{}",
        delivery_root.display()
    )
    .into())
}

/// R1-F02：producer 输入预检——闭包 crate manifest/patch/根级编译输入/根
/// manifest 与根锁逐项在位检查。任何缺失在触碰旧交付或建 staging 之前
/// 判红（失败保留旧交付字节，满足"失败不切换任何旧生产产物"）。
fn delivery_preflight_producer(producer_root: &Path) -> AutoResult<()> {
    let mut missing: Vec<String> = Vec::new();
    for c in DELIVERY_CRATES {
        if !producer_root
            .join("crates")
            .join(c)
            .join("Cargo.toml")
            .is_file()
        {
            missing.push(format!("crates/{c}/Cargo.toml"));
        }
    }
    for p in DELIVERY_PATCHES {
        if !producer_root
            .join("crates")
            .join("patches")
            .join(p)
            .join("Cargo.toml")
            .is_file()
        {
            missing.push(format!("crates/patches/{p}/Cargo.toml"));
        }
    }
    for rel in DELIVERY_ROOT_INPUTS {
        if !producer_root.join(rel).is_file() {
            missing.push(rel.to_string());
        }
    }
    for rel in ["Cargo.toml", "Cargo.lock"] {
        if !producer_root.join(rel).is_file() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "delivery: producer 预检失败（缺 {} 项输入：{}）——受管判红，既有交付不受影响",
            missing.len(),
            missing.join("、")
        )
        .into());
    }
    Ok(())
}

/// staging/换出残桩的兄弟路径（与交付根同父同卷——rename 不跨卷）。
fn delivery_sibling_path(delivery_root: &Path, suffix: &str) -> PathBuf {
    let name = delivery_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("delivery");
    delivery_root
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(".{name}.{suffix}"))
}

/// staging 目录内置属主标记（R2-F01：唯一名之外的归属证明——含此标记
/// 的目录方可被生成器删除）。
const DELIVERY_STAGING_MARKER: &str = ".delivery-staging.marker";

/// R3-F01：换出事务标记——backup 目录由本次换出创建后立即写入；陈残 GC
/// 仅回收同时具备本标记与 delivery-manifest.json 的既往残桩（可验证的
/// 事务身份，而非"任何交付包都有的清单文件名"）。
const DELIVERY_SWAP_MARKER: &str = ".delivery-swap.marker";

/// R2-F01：本次运行独占的 staging 路径（pid+纳秒唯一名——绝不复用固定
/// 名，也就绝不凭名字触碰同级外来目录）。若唯一名意外命中已存目录：
/// 含属主标记（既往崩溃残留）才清理，否则判红拒绝。
fn delivery_new_staging(delivery_root: &Path) -> AutoResult<PathBuf> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let staging = delivery_sibling_path(
        delivery_root,
        &format!("delivery-staging-{}-{nanos}", std::process::id()),
    );
    if staging.exists() {
        if staging.join(DELIVERY_STAGING_MARKER).is_file() {
            fs::remove_dir_all(&staging).map_err(|e| {
                format!(
                    "delivery: 清理同号 staging 残留失败（{}）: {e}",
                    staging.display()
                )
            })?;
        } else {
            return Err(format!(
                "delivery: staging 唯一名意外命中外来目录（{}）——拒绝触碰",
                staging.display()
            )
            .into());
        }
    }
    Ok(staging)
}

/// R2-F01：staging 属主守卫——本次运行创建的 staging 在任何失败/提前返回
/// 路径上都被清理（只可能删到本次独占名）；成功换装后路径已不存在，Drop
/// 为 no-op。
struct StagingGuard(PathBuf);
impl Drop for StagingGuard {
    fn drop(&mut self) {
        if self.0.is_dir() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

/// R1-F02：换装——staging 全部生成且校验成功后整体替换旧交付。
/// 旧包先换出到 backup 位，新包换入；换入失败回滚旧包后显式 Err（staging
/// 保留供诊断）。backup 清理失败只告警（新包已生效、旧包完整残留非混合态）。
/// R3-F01：backup 用**本次操作独占的唯一名**（pid+ns），换出后立刻写入
/// 换出事务标记——清理只作用于本次换出创建的路径；**任何既存目录（含
/// 带合法 delivery-manifest.json 的其他应用交付包）都不被认领/删除**。
/// 陈残 GC 只回收同时具备换出标记+清单的既往残桩（可验证的事务身份）。
fn delivery_swap_in(staging: &Path, delivery_root: &Path) -> AutoResult<()> {
    if !delivery_root.exists() {
        fs::rename(staging, delivery_root).map_err(|e| format!("delivery: 交付根落位失败: {e}"))?;
        return Ok(());
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let trash = delivery_sibling_path(
        delivery_root,
        &format!("delivery-old-{}-{nanos}", std::process::id()),
    );
    if trash.exists() {
        // 唯一名意外命中既存目录（外来）——拒绝触碰，不删不覆盖。
        return Err(format!(
            "delivery: backup 唯一名意外命中外来目录（{}）——拒绝删除/覆盖；可手动移开后重试",
            trash.display()
        )
        .into());
    }
    fs::rename(delivery_root, &trash)
        .map_err(|e| format!("delivery: 旧交付换出失败（旧包未动）: {e}"))?;
    // 换出事务标记：该目录自此可证明为本线换出残桩（GC 依据；回滚时移除
    // 以免污染恢复后的旧包）。
    let marker = trash.join(DELIVERY_SWAP_MARKER);
    if let Err(e) = fs::write(&marker, format!("pid={}\n", std::process::id())) {
        let _ = fs::rename(&trash, delivery_root);
        return Err(format!("delivery: 写换出标记失败（旧包已回滚）: {e}").into());
    }
    match fs::rename(staging, delivery_root) {
        Ok(()) => {}
        Err(e) => {
            let _ = fs::remove_file(&marker);
            if fs::rename(&trash, delivery_root).is_ok() {
                return Err(format!(
                    "delivery: 新交付换入失败（旧包已回滚原位；staging 保留供诊断）: {e}"
                )
                .into());
            }
            return Err(format!(
                "delivery: 新交付换入且回滚失败（旧包在 {}，staging 保留）: {e}",
                trash.display()
            )
            .into());
        }
    }
    if let Err(e) = fs::remove_dir_all(&trash) {
        eprintln!(
            "⚠ delivery: 旧交付清理失败（新包已生效；旧包完整残留于 {}，带换出标记可被后续 GC）: {e}",
            trash.display()
        );
    }
    Ok(())
}

/// R3-F01/R4-F01：既往换出残桩 GC——仅在**成功替代产物已在交付根落位**
/// （`delivery_root` 含 delivery-manifest.json）时，回收同时具备换出事务
/// 标记与清单的 `.{name}.delivery-old-*` 目录。
///
/// 换出事务标记只证明「本线曾把旧根换出」，**不证明新包已成功提交**；
/// 根缺失/未完成时备份可能是唯一旧包（待恢复态），绝不可回收。清单+标记
/// 双条件仍必要（排除无标记外来目录），但不再是充分条件。
fn delivery_gc_stale_backups(delivery_root: &Path) {
    // 成功替代证明：交付根在位且是完整交付包。缺此证明即视为可能存在
    // 待恢复的唯一旧备份——直接返回，不触碰任何 backup。
    let replacement_ok =
        delivery_root.is_dir() && delivery_root.join("delivery-manifest.json").is_file();
    if !replacement_ok {
        return;
    }
    let parent = delivery_root.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(
        ".{}.delivery-old-",
        delivery_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("delivery")
    );
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with(&prefix) {
            continue;
        }
        let path = entry.path();
        let swap_marker = path.join(DELIVERY_SWAP_MARKER).is_file();
        let manifest = path.join("delivery-manifest.json").is_file();
        if swap_marker && manifest {
            if let Err(e) = fs::remove_dir_all(&path) {
                eprintln!("⚠ delivery: 陈残换出桩清理失败（{}）: {e}", path.display());
            }
        } else {
            eprintln!(
                "⚠ delivery: 命名像换出残桩但缺事务标记/清单（{}）——不删除，请人工确认",
                path.display()
            );
        }
    }
}

/// R2-F02/R3-F02：绝对形态判定——除 `is_absolute` 外，Windows 根相对
///（`/x` 无盘符——join 语义丢弃基目录落当前盘根）与盘符/verbatim 前缀
/// 同属"丢弃解析基"的外部形态。
fn delivery_path_is_absolute_like(p: &Path) -> bool {
    p.is_absolute()
        || matches!(
            p.components().next(),
            Some(std::path::Component::RootDir) | Some(std::path::Component::Prefix(_))
        )
}

/// 词法包含判定：value 相对 base_depth（manifest 目录相对交付根的层数）
/// 解析后是否仍在交付根内。ParentDir 在根层再向外弹、或任何 RootDir/
/// Prefix 分量（丢弃基目录）即越界。
fn delivery_path_escapes_root(base_depth: usize, value: &str) -> bool {
    let mut depth = base_depth;
    for comp in Path::new(value).components() {
        match comp {
            std::path::Component::ParentDir => {
                if depth == 0 {
                    return true;
                }
                depth -= 1;
            }
            std::path::Component::CurDir => {}
            std::path::Component::RootDir | std::path::Component::Prefix(_) => return true,
            _ => depth += 1,
        }
    }
    false
}

/// R3-F02：递归收集 TOML 值树中所有键名为 `path` 的字符串值——依赖/
/// target/patch/workspace 等表的路径引用经解析器归一（空格、单/双引号、
/// 内联表写法同结论），不再做固定文本模板匹配。
fn delivery_collect_toml_paths(value: &toml::Value, out: &mut Vec<String>) {
    match value {
        toml::Value::Table(t) => {
            for (k, v) in t {
                if k == "path" {
                    if let toml::Value::String(s) = v {
                        out.push(s.clone());
                    }
                }
                delivery_collect_toml_paths(v, out);
            }
        }
        toml::Value::Array(a) => {
            for v in a {
                delivery_collect_toml_paths(v, out);
            }
        }
        _ => {}
    }
}

/// 对一组 path 值应用边界规则（绝对/根相对 + 词法越界），violator 返回
/// Some(诊断)。
fn delivery_check_path_values<'a>(
    base_depth: usize,
    owner: &str,
    paths: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    for v in paths {
        if delivery_path_is_absolute_like(Path::new(v)) {
            return Some(format!(
                "{owner} 使用外部绝对/根相对 path（{v}）——交付模式显式拒绝（包内仅受管闭包源；registry 依赖用 `name:version` 形态）"
            ));
        }
        if delivery_path_escapes_root(base_depth, v) {
            return Some(format!(
                "{owner} 的相对 path 越出交付根（{v}）——交付模式显式拒绝（受管闭包 crate 用 path=\"../crates/<name>\" 形态）"
            ));
        }
    }
    None
}

/// R1-F03/R3-F02：交付模式 sidecar 边界——按**实际 TOML 语义**校验每个
/// 依赖表达式（sidecar::fill/apply 会把 `{...}` 形态值原样写入
/// Cargo.toml，`{path='…'}` 等合法变体与 `{ path = "…" }` 同结论）；
/// registry 版本依赖与项目内 .rs 模块照常。相对值按 app/（根下一层）
/// 解析。生成后的全 manifest 边界自检为权威兜底。
fn delivery_validate_sidecar(sidecar: &auto_api_gen::sidecar::SidecarSpec) -> AutoResult<()> {
    for (name, spec) in &sidecar.deps {
        // 解析为 TOML 值（`__dep__ = <spec>` 载体）；解析失败 = 非 TOML
        // 表达式的版本串（写入形态带引号、cargo 不当 path 消费），由
        // 生成后边界自检兜底。
        let candidate = format!("__dep__ = {spec}");
        let Ok(parsed) = toml::from_str::<toml::Value>(&candidate) else {
            continue;
        };
        let mut paths: Vec<String> = Vec::new();
        delivery_collect_toml_paths(&parsed, &mut paths);
        if let Some(diag) = delivery_check_path_values(
            1,
            &format!("rust_sidecar 依赖 {name}"),
            paths.iter().map(|s| s.as_str()),
        ) {
            return Err(format!("delivery: {diag}").into());
        }
    }
    Ok(())
}

/// R1-F03 强化：交付边界自检——bundle 内全部 Cargo.toml 的每个
/// `path = "…"` 值必须相对、且按其 manifest 目录词法解析后落在交付根内
///（覆盖任何来源的越界源：sidecar/未来注入路径）；并保留 producer 绝对
/// 路径残留扫描。staging 内通过后才允许换装。
fn delivery_boundary_selfcheck(delivery_root: &Path, producer_root: &Path) -> AutoResult<()> {
    let mut manifests: Vec<PathBuf> = vec![
        delivery_root.join("Cargo.toml"),
        delivery_root.join("app").join("Cargo.toml"),
    ];
    fn collect_manifests(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if matches!(name.as_str(), "target" | ".git" | "node_modules") {
                    continue;
                }
                collect_manifests(&path, out);
            } else if name == "Cargo.toml" {
                out.push(path);
            }
        }
    }
    collect_manifests(&delivery_root.join("crates"), &mut manifests);
    let producer_abs = producer_root.to_string_lossy().to_lowercase();
    for m in &manifests {
        let Ok(text) = fs::read_to_string(m) else {
            continue;
        };
        if text.to_lowercase().contains(&producer_abs) {
            return Err(format!(
                "delivery: {} 残留 producer 绝对路径——重定位不安全，受管判红",
                m.display()
            )
            .into());
        }
        // R3-F02：按 TOML 语义解析后遍历全部 path 值（依赖/target/patch/
        // workspace 表；解析器归一空格/单双引号/内联表——不做固定文本
        // 匹配）。无法解析的 manifest 无法验证边界 → 受管判红（fail-closed）。
        let parsed: toml::Value = toml::from_str(&text).map_err(|e| {
            format!(
                "delivery: {} 无法按 TOML 解析（{e}）——边界不可验证，受管判红",
                m.display()
            )
        })?;
        let mut paths: Vec<String> = Vec::new();
        delivery_collect_toml_paths(&parsed, &mut paths);
        let base_depth = m
            .parent()
            .and_then(|p| p.strip_prefix(delivery_root).ok())
            .map(|rel| rel.components().count())
            .unwrap_or(0);
        if let Some(diag) = delivery_check_path_values(
            base_depth,
            &format!("{}", m.display()),
            paths.iter().map(|s| s.as_str()),
        ) {
            return Err(format!("delivery: {diag}").into());
        }
    }
    Ok(())
}

/// crate 名唯一性断言：bundle 内 crates/ 全部实际成员（含多余目录——按
/// package.name 判）的包名不得重复（同包 source 重复 = 身份冲突，显式
/// 拒绝）；应用名也不得与 vendored 撞名。
fn delivery_assert_unique_crates(delivery_root: &Path, app_package: &str) -> AutoResult<()> {
    let crates_dir = delivery_root.join("crates");
    let mut manifest_dirs: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if path.file_name().and_then(|n| n.to_str()) == Some("patches") {
                if let Ok(patches) = fs::read_dir(&path) {
                    for p in patches.flatten() {
                        if p.path().is_dir() {
                            manifest_dirs.push(p.path());
                        }
                    }
                }
            } else {
                manifest_dirs.push(path);
            }
        }
    }
    manifest_dirs.sort();
    let mut seen: Vec<String> = Vec::new();
    for dir in &manifest_dirs {
        let manifest = dir.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let text = fs::read_to_string(&manifest)
            .map_err(|e| format!("delivery: 读 {} 失败: {e}", manifest.display()))?;
        let value: toml::Value = toml::from_str(&text)
            .map_err(|e| format!("delivery: 解析 {} 失败: {e}", manifest.display()))?;
        let name = value
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| format!("delivery: {} 缺 package.name", manifest.display()))?;
        if seen.iter().any(|s| s == name) {
            return Err(format!(
                "delivery: 同包 source 重复（{} 在 bundle 内出现多次）——身份冲突显式拒绝",
                name
            )
            .into());
        }
        seen.push(name.to_string());
    }
    if seen.iter().any(|s| s == app_package) {
        return Err(format!(
            "delivery: 应用包名 {app_package} 与受管 crate 撞名——身份冲突显式拒绝"
        )
        .into());
    }
    Ok(())
}

/// 资源 allowlist：src/front 下非 .at 文件递归收集入 assets/（运行期 CWD
/// 相对解析——消费侧从 assets/ 运行，与开发模式 src/front 行为对齐）。
fn delivery_copy_resources(
    front_dir: &Path,
    delivery_root: &Path,
) -> AutoResult<Vec<serde_json::Value>> {
    let mut out = Vec::new();
    if !front_dir.is_dir() {
        return Ok(out);
    }
    fn walk(
        dir: &Path,
        rel: &str,
        dst_base: &Path,
        origin_base: &Path,
        out: &mut Vec<serde_json::Value>,
    ) -> AutoResult<()> {
        for entry in fs::read_dir(dir).map_err(|e| format!("delivery: 读资源目录失败: {e}"))?
        {
            let entry = entry.map_err(|e| format!("delivery: 遍历资源目录失败: {e}"))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let child_rel = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            let path = entry.path();
            if path.is_dir() {
                walk(&path, &child_rel, dst_base, origin_base, out)?;
            } else if path.extension().and_then(|e| e.to_str()) != Some("at") {
                let content = fs::read(&path)
                    .map_err(|e| format!("delivery: 读资源 {} 失败: {e}", path.display()))?;
                let dst = dst_base.join(&child_rel);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        format!("delivery: 建资源目录 {} 失败: {e}", parent.display())
                    })?;
                }
                fs::write(&dst, &content)
                    .map_err(|e| format!("delivery: 写资源 {} 失败: {e}", dst.display()))?;
                use sha2::{Digest, Sha256};
                out.push(serde_json::json!({
                    "rel": child_rel,
                    "origin_rel": path.strip_prefix(origin_base)
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .unwrap_or_default(),
                    "bytes": content.len(),
                    // R1-F01：资源 hash 用 SHA-256（随包 verify.py 经 python
                    // hashlib 独立可校验；crate digest 仍为 blake3 producer 身份）。
                    "sha256": format!("{:x}", Sha256::digest(&content)),
                }));
            }
        }
        Ok(())
    }
    walk(
        front_dir,
        "",
        &delivery_root.join("assets"),
        front_dir,
        &mut out,
    )?;
    out.sort_by(|a, b| a["rel"].as_str().cmp(&b["rel"].as_str()));
    Ok(out)
}

/// 重定位自检：cargo 可见的生成清单（根/应用 manifest 与 Cargo.lock）不得
/// 残留 producer 绝对路径（受管判红，不静默出包）。
/// 随包发行的受管资源校验入口（R1-F01：消费 delivery-manifest.json 的
/// resources allowlist——SHA-256 逐资源核对 + assets/ 未登记文件判红；
/// 无 producer/原工程也可独立执行）。
const DELIVERY_VERIFY_PY: &str = r#"#!/usr/bin/env python3
"""PLAN-792 交付包受管资源校验（随包发行；无 producer/原工程也可执行）。

消费 delivery-manifest.json 的 resources allowlist：
  1) 逐资源存在 + 字节数 + SHA-256 与记录相等（等长篡改判红）；
  2) assets/ 下不得有未登记文件（清空 allowlist 逃逸判红）。
退出码：0=通过；1=发现问题（逐条诊断）。
用法：python verify.py   （在交付根内）
"""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def main() -> int:
    manifest_path = ROOT / "delivery-manifest.json"
    if not manifest_path.is_file():
        print("[verify-delivery] FAIL delivery-manifest.json 缺失")
        return 1
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        print(f"[verify-delivery] FAIL delivery-manifest.json 不可读: {e}")
        return 1
    problems = []
    listed = set()
    for r in manifest.get("resources", []):
        rel = r.get("rel", "")
        listed.add(rel)
        f = ROOT / "assets" / rel
        if not f.is_file():
            problems.append(f"资源缺失: assets/{rel}")
            continue
        data = f.read_bytes()
        actual = hashlib.sha256(data).hexdigest()
        recorded = str(r.get("sha256", ""))
        if actual != recorded:
            problems.append(
                f"hash 不符: assets/{rel} 记录={recorded} 实际={actual}"
            )
        elif len(data) != int(r.get("bytes", -1)):
            problems.append(
                f"字节数不符: assets/{rel} 记录={r.get('bytes')} 实际={len(data)}"
            )
    assets_dir = ROOT / "assets"
    if assets_dir.is_dir():
        for f in sorted(assets_dir.rglob("*")):
            if f.is_file():
                rel = f.relative_to(assets_dir).as_posix()
                if rel not in listed:
                    problems.append(f"未登记资源: assets/{rel}")
    for p in problems:
        print(f"[verify-delivery] FAIL {p}")
    if problems:
        return 1
    print(f"[verify-delivery] PASS {len(listed)} 资源校验一致（SHA-256=清单记录）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
"#;

/// PLAN-792：显式可搬动交付模式生成入口。
///
/// 与开发模式（generate_rust_ui → rust-workspace 共享工作区 + workspace
/// 继承清单）共享同一编译相位 compile_rust_ui_app；差异仅在落点与清单形态：
/// 交付根为镜像生产仓布局的受管 bundle（见本节头注），manifest 不隐式搜索
/// D:/autostack 或编译期 CARGO_MANIFEST_DIR。锁由交付管线在受管阶段 fresh
/// 解析一次后冻结（本函数不自动跑 cargo；消费侧 `cargo build --locked`）。
///
/// R1-F02：全部生成/校验在独立 staging 完成，成功后整体换装——准备或
/// 生成失败保留既有交付字节（失败不切换任何旧生产产物）。
pub fn generate_rust_ui_delivery(project_dir: &Path, delivery_root: &Path) -> AutoResult<()> {
    let source = PacSource::legacy();
    generate_rust_ui_delivery_inner(project_dir, delivery_root, &source)
}

/// 请求路径（PLAN-809 T-06c 生产入口）。
pub fn generate_rust_ui_delivery_with_request(
    project_dir: &Path,
    delivery_root: &Path,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    let source = PacSource::from_request(req);
    generate_rust_ui_delivery_inner(project_dir, delivery_root, &source)
}

fn generate_rust_ui_delivery_inner(
    project_dir: &Path,
    delivery_root: &Path,
    source: &PacSource,
) -> AutoResult<()> {
    println!(
        "{}",
        "Generating Rust UI delivery bundle (PLAN-792 explicit delivery mode)".bright_cyan()
    );

    let producer_root = delivery_producer_root()?;
    let front_dir = front_dir_for(project_dir, source);
    if !front_dir.exists() {
        return Err(format!("Front directory not found: {}", front_dir.display()).into());
    }

    let Some(compiled) = compile_rust_ui_app(project_dir, source)? else {
        return Err(
            "delivery: 前端无 .at 组件（受管交付拒绝空应用——受管阶段判红，不静默出空包)".into(),
        );
    };

    // R1-F02：producer 完整性预检（触碰旧交付/staging 之前）。
    delivery_preflight_producer(&producer_root)?;

    // R1-F03：sidecar 外部 path 显式拒绝（registry 版本依赖/项目内模块照常）。
    let sidecar = source.sidecar_spec(project_dir);
    delivery_validate_sidecar(&sidecar)?;

    // 守卫（外来非空目录拒绝；换装语义下不清理既有交付）。
    delivery_guard_root(delivery_root)?;
    let final_root = delivery_root.to_path_buf();
    // R3-F01/R4-F01：陈残换出桩 GC——仅当交付根已是完整替代产物时回收
    // 冗余残桩；根缺失/未完成（待恢复唯一旧包）时 GC 为 no-op。
    delivery_gc_stale_backups(delivery_root);

    // R1-F02/R2-F01：独立 staging 生成——成功前不动旧交付。staging 用
    // 本次运行独占的唯一名（pid+ns）+ 内置属主标记，绝不凭固定名认领同级
    // 目录；失败路径由 StagingGuard 清理（只删本次确实拥有的路径）。
    let staging = delivery_new_staging(delivery_root)?;
    let _staging_guard = StagingGuard(staging.clone());
    if let Some(parent) = staging.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("delivery: 建 staging 父目录失败: {e}"))?;
    }
    fs::create_dir_all(&staging).map_err(|e| format!("delivery: 建 staging 失败: {e}"))?;
    fs::write(
        staging.join(DELIVERY_STAGING_MARKER),
        format!(
            "auto-man delivery staging (PLAN-792) pid={}\n",
            std::process::id()
        ),
    )
    .map_err(|e| format!("delivery: 写 staging 标记失败: {e}"))?;
    // 生成体一律落 staging（下方 delivery_root 遮蔽为 staging 路径）。
    let delivery_root = staging.as_path();

    // 受管源码闭包
    let mut crate_entries: Vec<serde_json::Value> = Vec::new();
    let mut total_files: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut projection_edits: Vec<String> = Vec::new();
    let mut projection_edits_core: Vec<String> = Vec::new();
    let mut projection_edits_runtime: Vec<String> = Vec::new();
    for name in DELIVERY_CRATES {
        let src = producer_root.join("crates").join(name);
        if !src.is_dir() {
            return Err(
                format!("delivery: producer 缺受管 crate 源（crates/{name}）——受管判红").into(),
            );
        }
        let dst = delivery_root.join("crates").join(name);
        let copied = delivery_copy_dir(&src, &dst, &DELIVERY_COPY_SKIP_DIRS)?;
        total_files += copied.records.len() as u64;
        total_bytes += copied.records.iter().map(|r| r.bytes).sum::<u64>();
        let mut edits: Vec<String> = Vec::new();
        if name == "auto-lang" {
            edits = delivery_apply_projection(&dst.join("Cargo.toml"), &PROJECTION_AUTO_LANG)?;
            projection_edits = edits.clone();
        } else if name == "auto-lang-core" {
            edits = delivery_apply_projection(&dst.join("Cargo.toml"), &PROJECTION_AUTO_LANG_CORE)?;
            projection_edits_core = edits.clone();
        } else if name == "auto-ui-runtime" {
            edits =
                delivery_apply_projection(&dst.join("Cargo.toml"), &PROJECTION_AUTO_UI_RUNTIME)?;
            projection_edits_runtime = edits.clone();
        }
        crate_entries.push(serde_json::json!({
            "name": name,
            "kind": "source",
            "origin_rel": format!("crates/{name}"),
            "files": copied.records.len(),
            "bytes": copied.records.iter().map(|r| r.bytes).sum::<u64>(),
            "digest": copied.digest,
            "projection_edits": edits,
        }));
        println!(
            "  {} crates/{name}（{} 文件，digest {}..）",
            "Vendored".bright_green(),
            copied.records.len(),
            &copied.digest[..12.min(copied.digest.len())]
        );
    }
    for name in DELIVERY_PATCHES {
        let src = producer_root.join("crates").join("patches").join(name);
        if !src.is_dir() {
            return Err(format!(
                "delivery: producer 缺受管 patch 源（crates/patches/{name}）——受管判红"
            )
            .into());
        }
        let dst = delivery_root.join("crates").join("patches").join(name);
        let copied = delivery_copy_dir(&src, &dst, &DELIVERY_COPY_SKIP_DIRS)?;
        total_files += copied.records.len() as u64;
        total_bytes += copied.records.iter().map(|r| r.bytes).sum::<u64>();
        crate_entries.push(serde_json::json!({
            "name": name,
            "kind": "patch",
            "origin_rel": format!("crates/patches/{name}"),
            "files": copied.records.len(),
            "bytes": copied.records.iter().map(|r| r.bytes).sum::<u64>(),
            "digest": copied.digest,
        }));
    }

    // 仓库根级编译期输入（include_str! 越界面——缺一即 build 红）
    let mut root_input_entries: Vec<serde_json::Value> = Vec::new();
    for rel in DELIVERY_ROOT_INPUTS {
        let src = producer_root.join(rel);
        if !src.is_file() {
            return Err(format!("delivery: producer 缺根级编译输入（{rel}）——受管判红").into());
        }
        let dst = delivery_root.join(rel);
        let content = fs::read(&src).map_err(|e| format!("delivery: 读 {rel} 失败: {e}"))?;
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("delivery: 建 {} 失败: {e}", parent.display()))?;
        }
        fs::write(&dst, &content).map_err(|e| format!("delivery: 写 {rel} 失败: {e}"))?;
        total_files += 1;
        total_bytes += content.len() as u64;
        root_input_entries.push(serde_json::json!({
            "rel": rel,
            "bytes": content.len(),
            "blake3": blake3::hash(&content).to_hex().to_string(),
        }));
    }

    // 根 manifest（消费根 patch）+ pin 集
    let app_package = to_snake_case(&compiled.project_name);
    // default-members 用 member 路径 "app"（固定应用位；包名是 ui_minimal 等）。
    let root_manifest = generate_delivery_root_manifest(&producer_root, "app")?;
    let root_manifest_path = delivery_root.join("Cargo.toml");
    fs::write(&root_manifest_path, &root_manifest)
        .map_err(|e| format!("delivery: 写交付根 manifest 失败: {e}"))?;

    let (pins, multi_slot) = delivery_pins(&producer_root)?;

    // 应用 member（main.rs 与开发模式同一编译相位产物）
    let app_dir = delivery_root.join("app");
    fs::create_dir_all(app_dir.join("src"))
        .map_err(|e| format!("delivery: 建应用目录失败: {e}"))?;
    let app_manifest =
        generate_delivery_app_manifest(&compiled.project_name, project_dir, &pins, source);
    fs::write(app_dir.join("Cargo.toml"), &app_manifest)
        .map_err(|e| format!("delivery: 写应用 manifest 失败: {e}"))?;
    fs::write(app_dir.join("src").join("main.rs"), &compiled.main_rs)
        .map_err(|e| format!("delivery: 写应用 main.rs 失败: {e}"))?;

    // PLAN-013 T2：rust_sidecar 供给（与开发模式同调用面；写入交付 app
    // member——外部 path 已在 R1-F03 前置拒绝，此处仅 registry 版本依赖
    // 与项目内 .rs 模块）。
    if !sidecar.is_empty() {
        auto_api_gen::sidecar::apply_sidecar_to_crate(&sidecar, project_dir, &app_dir)?;
        println!(
            "{}",
            "  rust_sidecar applied to delivery app member (modules + deps)".bright_green()
        );
    }

    // 资源 allowlist
    let resources = delivery_copy_resources(&front_dir, delivery_root)?;
    if resources.is_empty() {
        let _ = fs::remove_dir(delivery_root.join("assets"));
    }
    total_files += resources.len() as u64;
    for r in &resources {
        total_bytes += r["bytes"].as_u64().unwrap_or(0);
    }

    // 身份断言 + 交付边界自检（R1-F03：全部 manifest 的 path 越界/绝对
    // 路径扫描——覆盖 sidecar 与任何未来注入源；producer 残留扫描保留）。
    delivery_assert_unique_crates(delivery_root, &app_package)?;
    delivery_boundary_selfcheck(delivery_root, &producer_root)?;

    // R1-F01：随包受管资源校验入口（消费清单 sha256；无 producer 可执行）。
    fs::write(delivery_root.join("verify.py"), DELIVERY_VERIFY_PY)
        .map_err(|e| format!("delivery: 写 verify.py 失败: {e}"))?;

    // delivery-manifest.json（producer 身份/闭包规模/资源与 pin 清单）
    let (git_head, git_dirty) = delivery_git_identity(&producer_root);
    let ui_gen_fp = delivery_ui_gen_fingerprint(&producer_root)?;
    use sha2::{Digest, Sha256};
    let root_lock_sha256 = format!(
        "{:x}",
        Sha256::digest(
            &fs::read(producer_root.join("Cargo.lock"))
                .map_err(|e| format!("delivery: 读 producer 根锁失败: {e}"))?
        )
    );
    let main_blake3 = blake3::hash(compiled.main_rs.as_bytes())
        .to_hex()
        .to_string();
    let manifest = serde_json::json!({
        "schema": "generated-ui-delivery/1",
        "producer": {
            "generator": "auto-man rust_ui delivery mode (PLAN-792)",
            "source_root": producer_root.to_string_lossy().replace('\\', "/"),
            "git_head": git_head,
            "git_dirty": git_dirty,
            "ui_gen_fingerprint": ui_gen_fp,
            "app_lock_source": "Cargo.lock (workspace root)",
            // R2：根锁为本地输入（非 git 跟踪）——绑定实际内容身份。
            "root_lock_sha256": root_lock_sha256,
        },
        "layout": {
            "workspace_root": "Cargo.toml",
            "app_member": "app",
            "crates_dir": "crates",
            "resources_dir": if resources.is_empty() { serde_json::Value::Null } else { serde_json::json!("assets") },
            "run_note": "运行期本地资产按 CWD 相对解析：从 assets/ 运行应用二进制（开发模式为 src/front，行为对齐）",
        },
        "verification": {
            "entry": "verify.py",
            "resources_algo": "sha-256",
            "note": "python verify.py（交付根内）——缺资源/等长篡改/坏 hash/未登记资源判红",
        },
        "crates": crate_entries,
        "root_inputs": root_input_entries,
        "app": {
            "package": app_package,
            "project_name": compiled.project_name,
            "main_widget": compiled.main_widget,
            "main_rs_blake3": main_blake3,
            "main_rs_bytes": compiled.main_rs.len(),
        },
        "resources": resources,
        "pins": pins.iter().cloned().collect::<std::collections::BTreeMap<String, String>>(),
        "pins_multi_slot_lock_frozen": multi_slot,
        "projection_edits_auto_lang": projection_edits,
        "projection_edits_auto_lang_core": projection_edits_core,
        "projection_edits_auto_ui_runtime": projection_edits_runtime,
        "disclosure": {
            "total_files": total_files,
            "total_bytes": total_bytes,
            "includes_language_source": true,
            "note": "受管包含语言源码依赖（auto-lang 全库等 10 crate——语言尚未拆完，非纯 core/独立 UI 仓）；iced_widget 为本地 patch 单源（消费根 [patch.crates-io]）",
        },
    });
    let manifest_path = delivery_root.join("delivery-manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap_or_default(),
    )
    .map_err(|e| format!("delivery: 写 delivery-manifest.json 失败: {e}"))?;

    // R1-F02：staging 全部生成且校验通过——整体换装（失败路径已在上方
    // 显式 Err 返回并保留旧交付）。staging 属主标记随使命完成移除（换装
    // 后的交付根不带运行期标记）。
    let _ = fs::remove_file(staging.join(DELIVERY_STAGING_MARKER));
    delivery_swap_in(&staging, &final_root)?;
    // R4-F01：成功发布后回收冗余残桩——含根缺失待恢复态留下的旧备份
    // （此刻已被新包替代）。失败路径不走这里，唯一旧包得以保留。
    delivery_gc_stale_backups(&final_root);
    // 成功换装：staging 路径已不存在，守卫 Drop 为 no-op。

    println!(
        "  {} 交付根 {}（受管 {} 文件 / {:.1} MiB；pin {} 项，多槽锁冻结 {} 项）",
        "Delivered".bright_green(),
        final_root.display(),
        total_files,
        total_bytes as f64 / (1024.0 * 1024.0),
        pins.len(),
        multi_slot.len()
    );
    println!(
        "{}",
        "  下一步（受管阶段）：cargo generate-lockfile 全新解析一次后冻结；资源校验 python verify.py；消费侧 cargo build --locked"
            .bright_cyan()
    );
    Ok(())
}

/// PLAN-734 T-04：新鲜度失配时绕过复用臂的再生入口（等价默认路径）。
/// PLAN-736 T-05/AC-02：health 优先就绪探针。
/// - Some(true)  = 服务模式 ready（/__/health 200，身份可读）
/// - Some(false) = 端口有 HTTP 应答但未 ready（503=服务模式启动中；
///   404=legacy 产物无控制面——按既有 TCP 语义视为可继续）
/// - None        = 端口无 HTTP 应答（连接失败/超时——继续等待）
/// 相比纯 TCP connect：503（bind 成功但初始化未完/旧实例 draining）不再
/// 被误判为 ready；返回体含身份（instance/bound/config_hash）供上层核对。
pub fn probe_service_ready(port: u16) -> std::option::Option<bool> {
    use std::io::{Read, Write};
    let addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().ok()?;
    let mut stream =
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(800)).ok()?;
    stream
        .write_all(
            b"GET /__auto/health/ready HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        )
        .ok()?;
    let mut buf = Vec::new();
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(800)));
    let _ = stream.take(4096).read_to_end(&mut buf);
    let head = String::from_utf8_lossy(&buf);
    let status: u16 = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if status == 0 {
        return None; // 非 HTTP 应答（可能他者占口）——按未就绪继续等
    }
    Some(status == 200 || status == 404)
}

fn start_api_server_fresh(project_dir: &Path) -> Result<Option<std::process::Child>, String> {
    // 递归调用自身但复用臂已被调用方语义绕开——直接内联默认路径太长；
    // 以临时屏蔽 AUTO_REUSE_BACKEND 的方式走完整默认流程。
    let prev = std::env::var("AUTO_REUSE_BACKEND").ok();
    std::env::remove_var("AUTO_REUSE_BACKEND");
    let result = start_api_server(project_dir);
    if let Some(v) = prev {
        std::env::set_var("AUTO_REUSE_BACKEND", v);
    }
    result
}

pub fn start_api_server(project_dir: &Path) -> Result<Option<std::process::Child>, String> {
    // daemon-ensure 语义（auto-os apps.manifest / pac.at back_port daemon 链）：
    // AUTO_REUSE_BACKEND=1 且端口已有活监听时直接复用——不杀（Plan 354 的
    // kill 会打断产品链先起好的真后端）、不重生成、不再 spawn 生成的
    // axum stub。缺省行为不变（kill + 重生成 + spawn）。
    if std::env::var("AUTO_REUSE_BACKEND").as_deref() == Ok("1") {
        let port = crate::util::http_port();
        let probe_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        if std::net::TcpStream::connect_timeout(&probe_addr, std::time::Duration::from_secs(1))
            .is_ok()
        {
            // PLAN-734 T-04：ready 新鲜度校验——在跑后端的 generation.json 源
            // 指纹与当前 api.at/db.at 不一致时拒绝复用（旧产物不冒充当前契约），
            // 回落到默认 kill+重生成+spawn 路径。
            if backend_generation_is_fresh(project_dir) {
                println!();
                println!(
                    "  {} Backend port {} already serving — reusing it (AUTO_REUSE_BACKEND=1)",
                    "✓".bright_green(),
                    port
                );
            } else {
                println!(
                    "  ⛔ Backend port {} serves a STALE generation (source hash mismatch) — regenerating",
                    port
                );
                return start_api_server_fresh(project_dir);
            }
            if std::env::var_os("AUTO_HTTP_BASE").is_none() {
                std::env::set_var("AUTO_HTTP_BASE", format!("http://127.0.0.1:{}", port));
            }
            return Ok(None);
        }
    }
    // Backend lives in the resolved workspace ({name}-back/ member; Stage B P-2:
    // in-framework = shared examples/rust-workspace, out-of-framework =
    // project-local rust-workspace).
    let ws_dir = resolve_rust_workspace_dir(project_dir);
    let back_name = back_member_name(project_dir);
    let api_backend_dir = ws_dir.join(&back_name);
    // Generate or update the Rust backend from #[api] declarations (idempotent).
    if let Err(e) = auto_api_gen::api_gen::generate_api(project_dir, "rust") {
        return Err(format!("Failed to generate Rust backend: {}", e));
    }
    if !api_backend_dir.join("Cargo.toml").exists() {
        return Err(format!(
            "backend Cargo.toml missing under {}",
            api_backend_dir.display()
        ));
    }

    println!();
    println!(
        "{}",
        "▶ Starting API backend server (Rust axum)...".bright_cyan()
    );

    // Plan 354: Kill any process occupying the backend port before starting.
    let port = crate::util::http_port();
    crate::util::kill_process_on_port(port);

    let cargo_toml = api_backend_dir.join("Cargo.toml");

    // Plan 328: Sanitize Cargo package name — cargo rejects names starting
    // with a digit (e.g. 015-notes-back). Fix in-place if stale Cargo.toml
    // has an unsanitized name.
    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("name = \"") {
                if let Some(name) = rest.strip_suffix("\"") {
                    if name
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    {
                        let fixed = content.replace(
                            &format!("name = \"{}\"", name),
                            &format!("name = \"app-{}\"", name),
                        );
                        let _ = std::fs::write(&cargo_toml, fixed);
                        println!(
                            "  {} Fixed package name: {} → app-{}",
                            "⚠".bright_yellow(),
                            name,
                            name
                        );
                    }
                    break;
                }
            }
        }
    }

    println!("  cargo run --manifest-path {}", cargo_toml.display());
    // PLAN-815 T-05 hosted mode: the cargo api-server child is tracked
    // (piped stdout forwarded to the host log stream with URL sniffing;
    // cancel kills it; the registry owns it, so nothing is handed back).
    // Legacy path: inherit stdio, child returned to the caller as before.
    let mut tracked: Option<std::sync::Arc<crate::proc_host::TrackedChild>> = None;
    let api_server: Result<Option<std::process::Child>, String> =
        if crate::proc_host::current().is_some() {
            let spec = crate::proc_host::TrackedSpawn::new(
                "cargo",
                vec![
                    "run".to_string(),
                    "--manifest-path".to_string(),
                    cargo_toml.to_string_lossy().to_string(),
                ],
            );
            tracked = crate::proc_host::spawn_tracked(spec, "api-server")
                .map_err(|e| format!("Failed to start API server: {e}"))
                .ok();
            Ok(None)
        } else {
            std::process::Command::new("cargo")
                .args(["run", "--manifest-path", cargo_toml.to_str().unwrap_or(".")])
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .spawn()
                .map(Some)
                .map_err(|e| format!("Failed to start API server: {e}"))
        };

    match api_server {
        Ok(mut child) => {
            println!(
                "  {} API server starting (PID: {})...",
                "✓".bright_green(),
                child
                    .as_ref()
                    .map(|c| c.id())
                    .or_else(|| tracked.as_ref().map(|t| t.id))
                    .unwrap_or(0)
            );

            // Wait for the server to become ready by polling the port.
            // PLAN-026 T-05: 60s 一次性等待 + "continuing anyway" 吞错 →
            // 有界重试门(累计上限 120s,冷构建 back >60s 场景实测 98s;
            // 超限 kill + 显式报错,不再带着无监听端口进入组件构建/Init
            // —— lab-premerge2 卡死坑收口)。
            println!("  Waiting for API server to be ready...");
            let max_wait = std::time::Duration::from_secs(120);
            let start = std::time::Instant::now();
            let mut ready = false;

            // AUTO_HTTP_PORT (default 8080) — must match the backend's bind address.
            let port = crate::util::http_port();
            let probe_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

            while start.elapsed() < max_wait {
                std::thread::sleep(std::time::Duration::from_millis(500));
                // PLAN-815 T-05: hosted cancel during the ready window
                // kills the owned child and bails with the sentinel.
                if crate::proc_host::cancelled() {
                    if let Some(t) = &tracked {
                        t.kill();
                    } else if let Some(c) = child.as_mut() {
                        let _ = c.kill();
                    }
                    return Err(crate::proc_host::CANCELLED_ERR.to_string());
                }
                // PLAN-736 T-05：health 优先（200=service ready；404=legacy
                // 产物可继续；503=bind 成功但未 ready——不再假 ready），
                // TCP connect 兜底（health 通道异常时不阻塞 legacy 面）。
                match probe_service_ready(port) {
                    Some(is_ready) => {
                        ready = is_ready;
                        if ready {
                            break;
                        }
                        continue;
                    }
                    None => {}
                }
                match std::net::TcpStream::connect_timeout(
                    &probe_addr,
                    std::time::Duration::from_secs(1),
                ) {
                    Ok(_) => {
                        ready = true;
                        break;
                    }
                    Err(_) => continue,
                }
            }

            if ready {
                println!(
                    "  {} API server is ready on http://127.0.0.1:{}",
                    "✓".bright_green(),
                    port
                );
                Ok(child)
            } else {
                if let Some(t) = &tracked {
                    t.kill();
                } else if let Some(c) = child.as_mut() {
                    let _ = c.kill();
                }
                Err(format!(
                    "API backend not ready within {}s (port {}); aborting before UI init — check backend build/run above",
                    max_wait.as_secs(), port
                ))
            }
        }
        Err(e) => Err(format!("Failed to start API server: {}", e)),
    }
}

/// PLAN-815 T-05 R1: hosted-aware cargo invocation — owned tree with
/// cancel-aware reap when a RunIo is installed (cancelled waits surface
/// the [`crate::proc_host::CANCELLED_ERR`] sentinel so the final report
/// classifies Cancelled, F-815-T05-R1-05), plain `.status()` otherwise
/// (legacy behavior frozen).
fn cargo_status(
    program: &str,
    args: &[String],
    envs: &[(&str, String)],
) -> std::io::Result<std::process::ExitStatus> {
    if crate::proc_host::current().is_some() {
        let mut spec = crate::proc_host::TrackedSpawn::new(program, args.to_vec());
        for (key, value) in envs {
            spec = spec.with_env(key, value);
        }
        let tracked = crate::proc_host::spawn_tracked(spec, "cargo")?;
        let status = crate::proc_host::wait_tracked(&tracked)?;
        if !status.success() && crate::proc_host::cancelled() {
            return Err(std::io::Error::other(crate::proc_host::CANCELLED_ERR));
        }
        return Ok(status);
    }
    let mut command = std::process::Command::new(program);
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.status()
}

/// Stop an API server child process (if running).
pub fn stop_api_server(child: &mut Option<std::process::Child>) {
    if let Some(c) = child {
        let _ = c.kill();
        println!("  {} API server (Rust) stopped", "✓".bright_green());
    }
}

/// `auto build` for rust-mode UI projects: regenerate as needed, then
/// `cargo build` the shared-workspace member. Mirrors `run_rust_ui`'s
/// regeneration logic without running the app.
pub fn build_rust_ui(project_dir: &Path) -> AutoResult<()> {
    let source = PacSource::legacy();
    build_rust_ui_inner(project_dir, &source)
}

/// 请求路径（PLAN-809 R1-F02：run/build 生产再生成同样经 I-PROJECT）。
pub fn build_rust_ui_with_request(
    project_dir: &Path,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    let source = PacSource::from_request(req);
    build_rust_ui_inner(project_dir, &source)
}

fn build_rust_ui_inner(project_dir: &Path, source: &PacSource) -> AutoResult<()> {
    let ws_dir = resolve_rust_workspace_dir(project_dir);
    let member_name = front_member_name(project_dir);
    let rust_dir = ws_dir.join(&member_name);
    let (full, code) = needs_regeneration(project_dir, &rust_dir, source);

    if full {
        println!("{}", "Generating Rust UI project...".bright_cyan());
        generate_rust_ui_inner(project_dir, None, false, source)?;
    } else if code {
        println!(
            "{}",
            "Regenerating Rust UI code (source changed)...".bright_cyan()
        );
        regenerate_code_only(project_dir, &rust_dir, source)?;
    }

    // PLAN-681 R-1（复审 needs_fix 修正）：build 路径补 back crate 刷新。
    // generate_api（伴生模块转译 + route A——axum 后端生成）此前仅 run 路径
    // start_api_server 触发，back 源变更在 build 下只再生成 front 成员，
    // 陈旧空体桩照常编译绿（消费方 back 停 9-21 桩版实证）。该入口自注
    // idempotent；无契约纯前工程经 resolve_back_api 门控跳过。
    if auto_lang_core::config::resolve_back_api(project_dir).is_some() {
        println!(
            "{}",
            "Refreshing Rust backend crate (idempotent)...".bright_cyan()
        );
        auto_api_gen::api_gen::generate_api(project_dir, "rust")
            .map_err(|e| format!("Failed to generate Rust backend: {}", e))?;
    }

    // Resolve to an ABSOLUTE manifest path (ws_dir may be CWD-relative) and
    // pin CARGO_TARGET_DIR to the repo-wide shared target: cargo discovers
    // .cargo/config.toml from the CWD (NOT the manifest dir), so without the
    // env a build launched from the project dir would fall back to a cold
    // workspace-local target and recompile the whole tree (onig_sys included).
    let cargo_toml = std::fs::canonicalize(rust_dir.join("Cargo.toml")).map_err(|e| {
        format!(
            "generated Cargo.toml missing under {}: {}",
            rust_dir.display(),
            e
        )
    })?;

    println!(
        "{}",
        "Building Rust UI project (backend: rust-ui)".bright_cyan()
    );
    let build_args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        cargo_toml.to_string_lossy().to_string(),
    ];
    let target_dir = shared_cargo_target_dir(project_dir);
    let status = cargo_status(
        "cargo",
        &build_args,
        &[("CARGO_TARGET_DIR", target_dir.to_string_lossy().to_string())],
    )?;

    if !status.success() {
        return Err(format!("Cargo build failed with status: {}", status).into());
    }
    Ok(())
}

pub fn run_rust_ui(project_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    let source = PacSource::legacy();
    run_rust_ui_inner(project_dir, args, &source)
}

/// 请求路径（PLAN-809 R1-F02）。
pub fn run_rust_ui_with_request(
    project_dir: &Path,
    args: Vec<String>,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    let source = PacSource::from_request(req);
    run_rust_ui_inner(project_dir, args, &source)
}

fn run_rust_ui_inner(project_dir: &Path, args: Vec<String>, source: &PacSource) -> AutoResult<()> {
    // Rust project lives in the resolved workspace (Stage B P-2 landing order).
    let ws_dir = resolve_rust_workspace_dir(project_dir);
    let member_name = front_member_name(project_dir);
    let rust_dir = ws_dir.join(&member_name);
    let (full, code) = needs_regeneration(project_dir, &rust_dir, source);

    if full {
        println!("{}", "Generating Rust UI project...".bright_cyan());
        generate_rust_ui_inner(project_dir, None, false, source)?;
    } else if code {
        println!(
            "{}",
            "Regenerating Rust UI code (source changed)...".bright_cyan()
        );
        regenerate_code_only(project_dir, &rust_dir, source)?;
    }

    // Plan 347: In merged mode, skip the backend HTTP server — API functions
    // are in-process (generated as direct-call code in main.rs).
    let merged_mode = std::env::var("AUTO_VM_MERGE").as_deref() != Ok("0");
    let mut _api_child = if merged_mode {
        println!();
        println!(
            "  {} rust+rust merged mode: backend runs in-process (no HTTP server)",
            "✓".bright_green()
        );
        None
    } else {
        // PLAN-026 T-05: ready 失败显式中止(不进组件构建/Init)。
        match start_api_server(project_dir) {
            Ok(child) => child,
            Err(e) => return Err(e.into()),
        }
    };

    println!(
        "{}",
        if merged_mode {
            "Running Rust UI app (backend: rust-ui, merged in-process)".bright_cyan()
        } else {
            "Running Rust UI app (backend: rust-ui, split over HTTP)".bright_cyan()
        }
    );

    // Set CWD to src/front/ so local assets (e.g. avatar.png) can be found
    // by the Iced renderer's load_image_bytes(). The cargo subprocess uses
    // --manifest-path instead of current_dir so it can find Cargo.toml, but
    // the final binary (profile-card.exe) inherits this CWD for asset resolution.
    //
    // IMPORTANT: resolve the manifest to an ABSOLUTE path BEFORE changing CWD.
    // `rust_dir` is relative to the repo root (get_rust_workspace_dir returns
    // "examples/rust-workspace"), so after set_current_dir(front_dir) the
    // relative path no longer resolves and `cargo run --manifest-path` fails
    // with "manifest path does not exist".
    let cargo_toml = {
        let rel = rust_dir.join("Cargo.toml");
        // Prefer canonicalize (resolves symlinks/relative parts); fall back to
        // anchoring the relative path to the pre-CWD-change current directory.
        std::fs::canonicalize(&rel).unwrap_or_else(|_| {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            cwd.join(&rel)
        })
    };

    // PLAN-815 T-06 R2：运行 CWD 消费请求声明的前端目录。
    let front_dir = front_dir_for(project_dir, source);
    let original_dir = std::env::current_dir().ok();
    let _ = std::env::set_current_dir(&front_dir);

    let mut run_args = vec![
        "run".to_string(),
        "--manifest-path".to_string(),
        cargo_toml.to_string_lossy().to_string(),
    ];
    // PLAN-031 T-06：`--` 分隔符——以 `-` 开头的透传参数（如 rqhost 注入
    // 的 --autodesk-* 旗标）不加分隔符会被 cargo 自身吞掉报错；无参时
    // 不加（保持既有命令行形状）。
    if !args.is_empty() {
        run_args.push("--".to_string());
    }
    for arg in &args {
        run_args.push(arg.clone());
    }
    // Pin the shared repo target (see build_rust_ui): CWD is src/front for
    // runtime asset resolution, so the workspace's .cargo/config.toml is
    // NOT discovered from here and a cold local target would recompile the
    // whole tree (onig_sys included).
    let target_dir = shared_cargo_target_dir(project_dir);
    let status = cargo_status(
        "cargo",
        &run_args,
        &[("CARGO_TARGET_DIR", target_dir.to_string_lossy().to_string())],
    )?;

    // Restore original CWD
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(dir);
    }

    stop_api_server(&mut _api_child);

    if !status.success() {
        // F-815-T05-R1-05: a cancel-killed cargo tree is a cancellation,
        // not a cargo failure — the sentinel keeps the final report honest.
        if crate::proc_host::cancelled() {
            return Err(crate::proc_host::CANCELLED_ERR.into());
        }
        return Err(format!("Cargo run failed with status: {}", status).into());
    }

    Ok(())
}

// ── Plan 061:外部后端(pac.at `back: { project }`)装载编排 ─────────────

/// 读 pac.at 的 `back.project` 配置(无 pac.at / 无配置 → None)。
/// 探针参数与 Automan::parse_pac 一致(port=win32 占位,Config 求值用)。
fn external_backend_of(project_dir: &Path) -> AutoResult<Option<auto_val::AutoStr>> {
    use auto_val::{AutoStr, Obj, Value};
    let pac_path = project_dir.join("pac.at");
    if !pac_path.is_file() {
        return Ok(None);
    }
    let mut probe = Obj::new();
    probe.set("port", Value::Str(AutoStr::from("win32")));
    let config = auto_lang_core::config::AutoConfig::from_file(&pac_path, &probe)?;
    // PLAN-815 T-04: 读取经 auto_man_core 声明归一单源（Pac::new 对
    // external_backend 是 decl 字段直拷——冻结 oracle 不变；不引 Man Pac）。
    let decl = auto_man_core::pac_from_decl(&config.root, &config.args);
    Ok(decl.external_backend.filter(|s| !s.is_empty()))
}

/// PLAN-609 T-A1: pac.at `theme: {}` 块声明 → 合成主题激活（VM 腿 boot）。
/// 与 vue 腿（`vue.rs::from_workspace` T-06）同口径：pac 单块解析无具名
/// 声明集，`extends` 仅可引用内置。声明缺席/pac.at 缺失或解析失败 →
/// 零变化（None）；compose 失败告警回退内置缺省（vue 腿同款容错）。
/// 激活只换 ACTIVE_THEME 槽，DARK_MODE/ACCENT_NAME 不动（调用点保证在
/// env 种子之后）。返回激活的合成主题名（测试断言面）。
fn apply_pac_theme_decl(project_dir: &Path) -> Option<String> {
    let pac_path = project_dir.join("pac.at");
    let config =
        match auto_lang_core::config::AutoConfig::from_file(&pac_path, &auto_val::Obj::new()) {
            Ok(c) => c,
            Err(_) => return None,
        };
    // PLAN-815 T-04: 声明读取经 auto_man_core 单源（Pac::new 对 theme_decl
    // 是 decl 字段直拷+语言类型直映；resolve() 只动 props/targets/依赖
    // 下载，不触及 theme_decl——冻结 oracle 不变，不引 Man Pac）。
    let decl_lite = auto_man_core::pac_from_decl(&config.root, &config.args).theme_decl?;
    let decl = auto_lang_core::design_tokens::decl::ThemeDecl {
        name: decl_lite.name.clone(),
        extends: decl_lite.extends.clone(),
        mode: decl_lite.mode.clone(),
        colors: decl_lite.colors.clone(),
    };
    match auto_lang_core::design_tokens::decl::compose(&decl, &std::collections::BTreeMap::new()) {
        Ok(theme) => {
            let name = theme.name.clone();
            auto_ui_compiler::style::theme::set_theme_composed(std::sync::Arc::new(theme));
            Some(name)
        }
        Err(e) => {
            println!(
                "{} theme{{}} 合成失败：{e} —— 回退内置缺省",
                "⚠".bright_yellow()
            );
            None
        }
    }
}

/// Generate a Rust type alias from an Auto TypeDecl.
/// Uses serde_json::Value as the underlying type to maintain compatibility
/// with the dynamic-typing code generator (field access via ["field"]).
fn generate_type_struct(type_decl: &auto_lang_core::ast::TypeDecl) -> String {
    format!("pub type {} = serde_json::Value;\n", type_decl.name)
}

/// PLAN-039 T-12（批次 E，E-D4）：`.at` 内建方法的 Value 接收者 shim 家族
/// ——每个生成 main.rs 文件级注入一次（#allow(dead_code) 零增警）。语义
/// 与 VM 宽松面逐条对齐：len 串按字符/数组按元素；slice 按字符截取
/// （start/end 容差钳制）；str 数值降串不带引号（as_str 直通，其余走
/// Value Display）；lower/upper/trim 走 as_str 缺省空串。
const AT_BUILTIN_METHOD_SHIMS: &str = r#"#[allow(dead_code)]
fn __at_num(v: &serde_json::Value) -> i32 {
    v.as_i64().unwrap_or(0) as i32
}
#[allow(dead_code)]
fn __at_len(v: &serde_json::Value) -> i32 {
    if let Some(a) = v.as_array() {
        a.len() as i32
    } else if let Some(s) = v.as_str() {
        s.chars().count() as i32
    } else {
        v.as_i64().unwrap_or(0) as i32
    }
}
#[allow(dead_code)]
fn __at_slice_v(v: &serde_json::Value, a: i32, b: i32) -> String {
    let s = v.as_str().unwrap_or_default();
    s.chars().skip(a.max(0) as usize).take((b - a).max(0) as usize).collect()
}
#[allow(dead_code)]
fn __at_slice(s: &str, a: i32, b: i32) -> String {
    s.chars().skip(a.max(0) as usize).take((b - a).max(0) as usize).collect()
}
#[allow(dead_code)]
fn __at_str(v: &serde_json::Value) -> String {
    match v.as_str() {
        Some(s) => s.to_string(),
        None => v.to_string(),
    }
}
#[allow(dead_code)]
fn __at_lower(v: &serde_json::Value) -> String {
    v.as_str().unwrap_or_default().to_lowercase()
}
#[allow(dead_code)]
fn __at_upper(v: &serde_json::Value) -> String {
    v.as_str().unwrap_or_default().to_uppercase()
}
#[allow(dead_code)]
fn __at_trim(v: &serde_json::Value) -> String {
    v.as_str().unwrap_or_default().trim().to_string()
}
"#;

/// PLAN-039 T-12（批次 E）：store 文件级自由 fn → Rust 自由 fn。参数/
/// 返回按声明型直译（type_to_rust_str）；函数体复用 handler 语句翻译
/// 链（generate_handler_body：join_stmt_block + postprocess 全含）。
fn generate_module_fn_rust(
    generator: &auto_ui_compiler::ui_gen::rust::RustGenerator,
    fn_decl: &auto_lang_core::ast::Fn,
) -> String {
    let params: Vec<String> = fn_decl
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name.as_str(), type_to_rust_str(&p.ty)))
        .collect();
    let ret = match &fn_decl.ret {
        auto_lang_core::ast::Type::Void => String::new(),
        t => format!(" -> {}", type_to_rust_str(t)),
    };
    let body = generator.generate_handler_body(&auto_ui_compiler::aura::LogicPayload::AstStmts(
        fn_decl.body.stmts.clone(),
    ));
    format!(
        "#[allow(unused)]\nfn {}({}){} {{\n    {}\n}}\n",
        fn_decl.name.as_str(),
        params.join(", "),
        ret,
        body
    )
}

/// Convert an Auto Type to a Rust type string (standalone, mirrors RustGenerator::auto_type_to_rust).
fn type_to_rust_str(ty: &auto_lang_core::ast::Type) -> String {
    use auto_lang_core::ast::Type;
    match ty {
        Type::Int => "i32".to_string(),
        Type::Uint => "u32".to_string(),
        Type::I64 => "i64".to_string(),
        Type::U64 => "u64".to_string(),
        Type::Float => "f32".to_string(),
        Type::Double => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => "String".to_string(),
        Type::Void => "()".to_string(),
        Type::Array(arr) => format!("Vec<{}>", type_to_rust_str(&arr.elem)),
        Type::RuntimeArray(arr) => format!("Vec<{}>", type_to_rust_str(&arr.elem)),
        Type::List(inner) => format!("Vec<{}>", type_to_rust_str(inner)),
        Type::Slice(sl) => format!("Vec<{}>", type_to_rust_str(&sl.elem)),
        Type::Map(k, v) => format!(
            "std::collections::HashMap<{}, {}>",
            type_to_rust_str(k),
            type_to_rust_str(v)
        ),
        Type::User(td) => td.name.to_string(),
        Type::Unknown => "serde_json::Value".to_string(),
        _ => "serde_json::Value".to_string(),
    }
}

#[cfg(test)]
mod tests {
    /// T-07 子批3（F-815-T07-B2-R1-03）`external_backend` 字段消费钉：
    /// 请求在场即权威——值**或缺席**都由请求定，不回读磁盘主声明（无第
    /// 二真源）；legacy 臂（无请求）才读 pac.at `back: { project }`。
    /// run 面会话消费（backend 根=project_root.join(值)，托管形态经
    /// `AUTO_UI_TOOL_VM_BACK_ROOT` 每请求精确条目传 VM 会话子进程）见
    /// run_vm.rs 装配段与 proc_host
    /// `vm_session_env_is_replaced_per_request`（R4-01）。
    #[test]
    fn external_backend_request_is_authoritative_over_disk_decl() {
        use auto_project_model::descriptor::SourceInfo;
        use auto_project_model::{parse_document, GenerationRequest};
        let dir = tempfile::tempdir().expect("tempdir");
        // 磁盘主声明：back -> ./disk-back、exe_name -> disk-bin（与请求
        // 快照不同值，区分性 oracle）。
        std::fs::write(
            dir.path().join("pac.at"),
            "name: \"back-demo\"\nrender: \"vue\"\nexe_name: \"disk-bin\"\nback: { project: \"./disk-back\" }\n",
        )
        .unwrap();
        let parse = |text: &str| {
            GenerationRequest::from_descriptor(
                &parse_document(text, SourceInfo::from_path(dir.path().join("pac.at")))
                    .expect("parse"),
            )
        };
        let with_request_back =
            parse("name: \"back-demo\"\nrender: \"vue\"\nback: { project: \"./request-back\" }\n");
        assert_eq!(
            with_request_back.external_backend.as_deref(),
            Some("./request-back")
        );
        let without_back = parse("name: \"back-demo\"\nrender: \"vue\"\n");
        // 请求在场：值即请求值；请求未声明=缺席（不落回磁盘读取）。
        assert_eq!(
            PacSource::from_request(&with_request_back)
                .external_backend(dir.path())
                .unwrap()
                .as_deref(),
            Some("./request-back")
        );
        assert_eq!(
            PacSource::from_request(&without_back)
                .external_backend(dir.path())
                .unwrap(),
            None,
            "evaluated request is authoritative — no disk fallback read"
        );
        // legacy 臂（无请求）：读磁盘主声明。
        assert_eq!(
            PacSource::legacy()
                .external_backend(dir.path())
                .unwrap()
                .as_deref(),
            Some("./disk-back")
        );
        // 同款请求权威模式钉（T-07 子批3 矩阵 §exe_name）：rust [[bin]]
        // 命名经 PacSource::exe_name（rust_ui.rs:3250）——请求在场即权威。
        let with_exe_name = parse(
            "name: \"back-demo\"\nrender: \"vue\"\nexe_name: \"req-bin\"\nback: { project: \"./x\" }\n",
        );
        assert_eq!(with_exe_name.exe_name.as_deref(), Some("req-bin"));
        let pac_file = dir.path().join("pac.at"); // exe_name 取 pac 文件路径
        assert_eq!(
            PacSource::from_request(&with_exe_name)
                .exe_name(&pac_file)
                .as_deref(),
            Some("req-bin")
        );
        assert_eq!(
            PacSource::from_request(&without_back).exe_name(&pac_file),
            None,
            "request arm carries the request's absence — no disk fallback"
        );
        assert_eq!(
            PacSource::legacy().exe_name(&pac_file).as_deref(),
            Some("disk-bin")
        );
    }

    /// T-07 子批3 R1 复审修正（F-815-T07-B2-R1-03 收口）：
    /// `runtime.window` / `display.title_zh|title` / `runtime.api` /
    /// `dep_roots` 四个 PacSource 读取面的**请求臂消费**钉——生成/运行
    /// 链消费点 = wrap_example（rust_ui.rs:2727-2728 → 生成 main.rs 的
    /// AUTO_VM_WINDOW/AUTO_VM_TITLE 初始化）、run_vm.rs:297（api_is_rust →
    /// VM/Rust 后端默认与 split 分支 → generate_api("rust")+start_api_server）、
    /// resolve_bps_dependencies（rust_ui.rs:2940 → rust 轨 dep 源补充）。
    /// 请求在场即权威（值或缺席都不落盘读）；legacy 臂才读盘。
    #[test]
    fn pac_source_window_title_api_dep_roots_request_arms() {
        use auto_project_model::descriptor::SourceInfo;
        use auto_project_model::{parse_document, GenerationRequest};
        let dir = tempfile::tempdir().expect("tempdir");
        // 盘上声明与请求快照不同值（区分性 oracle）
        std::fs::write(
            dir.path().join("pac.at"),
            "name: \"env-demo\"\nrender: \"rust\"\nwindow: \"640x480\"\ntitle: \"DISK-TITLE\"\napi: \"rust\"\ndep \"disk-dep\" { path: \"./disk-deps\" }\n",
        )
        .unwrap();
        let parse = |text: &str| {
            GenerationRequest::from_descriptor(
                &parse_document(text, SourceInfo::from_path(dir.path().join("pac.at")))
                    .expect("parse"),
            )
        };
        let pac_file = dir.path().join("pac.at");
        let req = parse(
            "name: \"env-demo\"\nrender: \"rust\"\nwindow: \"901x617\"\ntitle_zh: \"REQUEST-ZH\"\ntitle: \"REQUEST-EN\"\ndep \"req-dep\" { path: \"./req-deps\" }\n",
        );
        let src = PacSource::from_request(&req);
        // runtime.window（Size→"WxH"）；display 链 title_zh 优先于 title
        assert_eq!(src.window_env(&pac_file).as_deref(), Some("901x617"));
        assert_eq!(src.title_env(&pac_file).as_deref(), Some("REQUEST-ZH"));
        // 请求未声明 api → false（权威缺席，不落盘读盘上 api:"rust"）
        assert!(!src.api_is_rust(&pac_file));
        // dep_roots 请求臂：相对路径按 project_dir 锚定
        let deps = src.dep_roots(&pac_file, dir.path());
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, "req-dep");
        assert_eq!(deps[0].1, dir.path().join("req-deps"));
        // 无 path 的 dep 默认锚 deps/<name>
        let bare = parse("name: \"env-demo\"\nrender: \"rust\"\ndep \"bare\" {}\n");
        let deps2 = PacSource::from_request(&bare).dep_roots(&pac_file, dir.path());
        assert_eq!(deps2.len(), 1);
        assert_eq!(deps2[0].1, dir.path().join("deps").join("bare"));
        // Fit 窗口归一
        let fit = parse("name: \"env-demo\"\nrender: \"rust\"\nwindow: \"fit\"\n");
        assert_eq!(
            PacSource::from_request(&fit)
                .window_env(&pac_file)
                .as_deref(),
            Some("fit")
        );
        // legacy 臂读盘（行为冻结对照）
        let legacy = PacSource::legacy();
        assert_eq!(legacy.window_env(&pac_file).as_deref(), Some("640x480"));
        assert_eq!(legacy.title_env(&pac_file).as_deref(), Some("DISK-TITLE"));
        assert!(legacy.api_is_rust(&pac_file));
    }

    /// PLAN-815 T-04 R1-F01 永久反例：交付闭包完备性静态钉——对
    /// DELIVERY_CRATES 中每个 crate，读其 Cargo.toml 的 [dependencies]/
    /// [build-dependencies] path 项；凡指向本 workspace 兄弟 crate 的
    /// path 依赖必须在 DELIVERY_CRATES 内。R1 首轮 23 项清单漏
    /// auto-project-model/auto-toolchain-core 致交付根 fresh metadata
    /// exit 101——本测试把该缺陷类钉死在源内（真实 bundle 的隔离
    /// metadata 验收另由 driver/收尾证明承载）。
    #[test]
    fn delivery_crates_path_closure_is_complete() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        for name in DELIVERY_CRATES {
            let manifest_path = root.join(name).join("Cargo.toml");
            let manifest = std::fs::read_to_string(&manifest_path)
                .unwrap_or_else(|e| panic!("{} manifest unreadable: {e}", name));
            let parsed: toml::Value = toml::from_str(&manifest)
                .unwrap_or_else(|e| panic!("{} manifest unparsable: {e}", name));
            // 投影剥除豁免面：vendored manifest 中被 DeliveryProjectionSpec
            // 逐行剥除的 optional path 依赖（如 auto-lang 的
            // auto-test-case/autodown-core——前者闭包外 optional，后者指向
            // 仓外 auto-down）不经交付解析，无需随包。
            let stripped: Vec<&str> = [
                PROJECTION_AUTO_LANG.strip_deps,
                PROJECTION_AUTO_LANG_CORE.strip_deps,
                PROJECTION_AUTO_UI_RUNTIME.strip_deps,
            ]
            .into_iter()
            .flatten()
            .copied()
            .collect();
            let manifest_text = &manifest;
            for section in ["dependencies", "build-dependencies"] {
                let Some(deps) = parsed.get(section).and_then(|v| v.as_table()) else {
                    continue;
                };
                for (dep, spec) in deps {
                    let Some(path) = spec.get("path").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    if !path.replace(char::from(92), "/").starts_with("../") {
                        continue; // 同 crate 内相对资源（tests/fixtures 等）不经 workspace 解析
                    }
                    let target = path.trim_start_matches("../");
                    if target.contains("..") {
                        continue; // 越出 workspace 的外部 path（如 auto-down）经投影剥除受管
                    }
                    if stripped
                        .iter()
                        .any(|line| line.contains(&format!("{dep} = ")))
                    {
                        continue; // 投影 spec 已剥除该行
                    }
                    let target = target.split('/').next().unwrap_or(target);
                    if target == name {
                        continue;
                    }
                    assert!(
                        DELIVERY_CRATES.contains(&target),
                        "{name} 的 path 依赖 {target} 不在 DELIVERY_CRATES 且无投影剥除——交付根 fresh metadata 将 exit 101（R1-F01 缺陷类）"
                    );
                }
            }
            let _ = manifest_text;
        }
    }

    use super::*;
    // PLAN-815 T-04: freshness pure helpers moved to auto-api-gen (api_gen).
    use auto_api_gen::api_gen::{assembly_freshness, lock_freshness};

    /// PLAN-815 T-04 随迁：image stdlib 共享 runtime feature 的 UI 侧断言
    /// （原 api_gen::image_stdlib_codegen_uses_shared_runtime_feature 的
    /// rust_ui 半部——api 侧留 auto-api-gen）。
    #[test]
    fn image_stdlib_ui_cargo_uses_shared_runtime_feature() {
        let ui_cargo = generate_cargo_toml("image-ui", Path::new("."), &PacSource::legacy());
        assert!(ui_cargo.contains("auto-lang/ui-iced"));
    }

    /// PLAN-738 R5-01：workspace lock 新鲜度真值表（严格比较）。首次
    /// 物化（absent→实际值）**不在纯函数层豁免**——由 backend_generation_
    /// is_fresh 的绑定分支处理（判新鲜的同时把实际 lock 身份写回收据，
    /// 一次性收敛；R5 反例：纯 absent 豁免会永久放行后续漂移）。旧收据
    /// 缺字段（None）/lock 消失/值漂移均保守再生。
    #[test]
    fn lock_freshness_truth_table() {
        assert!(lock_freshness(Some("a1b2"), Some("a1b2")));
        assert!(!lock_freshness(Some("a1b2"), Some("c3d4")));
        assert!(!lock_freshness(Some("a1b2"), Some("absent")));
        // 纯函数层 absent≠实值：首启豁免只经门内绑定分支（见
        // review738_r5_lock_binding_state_machine 状态机测试）。
        assert!(!lock_freshness(Some("absent"), Some("c3d4")));
        assert!(lock_freshness(Some("absent"), Some("absent")));
        assert!(!lock_freshness(None, Some("a1b2")));
        assert!(!lock_freshness(Some("absent"), None));
        assert!(!lock_freshness(None, None));
    }

    /// PLAN-738 R5-01：lock 一次绑定状态机——真实 generate_api + 隔离
    /// workspace：生成时 absent → 首次物化 fresh 且收据绑定实际 lock 身份
    /// → 依赖版本漂移/lock 删除/收据缺字段均陈旧。反例源自 R5 补充复审
    /// 探针（738-review-r5-probe.py）。
    #[test]
    fn review738_r5_lock_binding_state_machine() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("project");
        let isolated_workspace = dir.path().join("workspace");
        std::fs::create_dir_all(project.join("src/back")).unwrap();
        std::fs::write(
            project.join("pac.at"),
            "{ name: \"review738-r5-lock\", scene: \"workspace\" }",
        )
        .unwrap();
        std::fs::write(
            project.join("src/back/api.at"),
            "#[api(method = \"GET\", path = \"/api/answer\")]
pub fn answer() int { return 42 }
",
        )
        .unwrap();
        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(value) => std::env::set_var("AUTO_RUST_WORKSPACE", value),
                    None => std::env::remove_var("AUTO_RUST_WORKSPACE"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("AUTO_RUST_WORKSPACE"));
        std::env::set_var("AUTO_RUST_WORKSPACE", &isolated_workspace);
        auto_api_gen::api_gen::generate_api(&project, "rust").unwrap();
        let ws = ensure_shared_workspace(&project);
        let receipt_path = ws.join(back_member_name(&project)).join("generation.json");
        let read_lock_field = || {
            serde_json::from_str::<serde_json::Value>(
                &std::fs::read_to_string(&receipt_path).unwrap(),
            )
            .unwrap()["workspace_lock"]
                .as_str()
                .unwrap()
                .to_string()
        };
        assert_eq!(read_lock_field(), "absent", "生成时尚无 lock");
        assert!(backend_generation_is_fresh(&project), "无 lock 物化时新鲜");
        let lock_path = ws.join("Cargo.lock");
        let lock_v1 = "version = 4

[[package]]
name = \"review-runtime\"
version = \"0.1.0\"
";
        std::fs::write(&lock_path, lock_v1).unwrap();
        assert!(
            backend_generation_is_fresh(&project),
            "首次物化（确定性派生）新鲜"
        );
        let bound = read_lock_field();
        assert_eq!(
            bound,
            format!("{:x}", auto_api_gen::api_gen::fnv1a(lock_v1.as_bytes())),
            "首次物化必须把实际 lock 身份绑定进收据"
        );
        assert!(backend_generation_is_fresh(&project), "绑定后同内容仍新鲜");
        let lock_v2 = "version = 4

[[package]]
name = \"review-runtime\"
version = \"0.2.0\"
";
        std::fs::write(&lock_path, lock_v2).unwrap();
        assert!(
            !backend_generation_is_fresh(&project),
            "依赖版本漂移必须拒绝旧收据"
        );
        std::fs::remove_file(&lock_path).unwrap();
        assert!(!backend_generation_is_fresh(&project), "lock 删除必须陈旧");
        std::fs::write(&lock_path, lock_v1).unwrap();
        let mut receipt = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&receipt_path).unwrap(),
        )
        .unwrap();
        receipt.as_object_mut().unwrap().remove("workspace_lock");
        std::fs::write(&receipt_path, receipt.to_string()).unwrap();
        assert!(
            !backend_generation_is_fresh(&project),
            "旧收据缺字段=不可核验，保守陈旧"
        );
    }

    /// P738-R9-01 fail-closed + R11-01 三态身份：NotFound=Ok(None)（调用
    /// 方归 absent）；成功读取（含零字节）=Ok(Some(实际内容指纹))——空
    /// 文件也是实际身份 811c9dc5，不与 absent 合并；权限/IO 错误=Err
    /// （不可核验，不冒充 absent）。
    #[test]
    fn workspace_lock_identity_three_states() {
        assert_eq!(
            workspace_lock_identity(Ok(vec![1, 2])).unwrap(),
            Some(format!("{:x}", auto_api_gen::api_gen::fnv1a(&[1, 2])))
        );
        // R11 反例锚：零字节文件=实际内容身份，两端同口径。
        assert_eq!(
            workspace_lock_identity(Ok(Vec::new())).unwrap(),
            Some("811c9dc5".to_string())
        );
        assert_eq!(
            workspace_lock_identity(Err(std::io::Error::from(std::io::ErrorKind::NotFound)))
                .unwrap(),
            None
        );
        assert!(workspace_lock_identity(Err(std::io::Error::from(
            std::io::ErrorKind::PermissionDenied
        )))
        .is_err());
        // 端到端语义：recorded=absent × 读错误（→None）必须陈旧。
        assert!(!lock_freshness(Some("absent"), None));
    }

    /// P738-R11-01：空 lock 生成/消费身份对拍矩阵——生成端收据与复用门
    /// 对同一空文件必须记同一实际身份（811c9dc5）；absent→空文件一次绑定；
    /// 同空文件再生成后未变输入不得判 stale（R11 反例转正：修复前门把空
    /// 文件归 absent，与收据指纹永不相等 → 永久陈旧）；此后非空漂移/
    /// 删除/非 NotFound 读错误均陈旧。真实 generate_api +
    /// backend_generation_is_fresh，隔离 workspace（沿 R5 状态机测试形态）。
    #[test]
    fn review751_empty_lock_identity_matrix() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("project");
        let isolated_workspace = dir.path().join("workspace");
        std::fs::create_dir_all(project.join("src/back")).unwrap();
        std::fs::write(
            project.join("pac.at"),
            "{ name: \"plan751-empty-lock\", scene: \"workspace\" }",
        )
        .unwrap();
        std::fs::write(
            project.join("src/back/api.at"),
            "#[api(method = \"GET\", path = \"/api/answer\")]
pub fn answer() int { return 42 }
",
        )
        .unwrap();
        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(value) => std::env::set_var("AUTO_RUST_WORKSPACE", value),
                    None => std::env::remove_var("AUTO_RUST_WORKSPACE"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("AUTO_RUST_WORKSPACE"));
        std::env::set_var("AUTO_RUST_WORKSPACE", &isolated_workspace);
        auto_api_gen::api_gen::generate_api(&project, "rust").unwrap();
        let ws = ensure_shared_workspace(&project);
        let receipt_path = ws.join(back_member_name(&project)).join("generation.json");
        let read_lock_field = || {
            serde_json::from_str::<serde_json::Value>(
                &std::fs::read_to_string(&receipt_path).unwrap(),
            )
            .unwrap()["workspace_lock"]
                .as_str()
                .unwrap()
                .to_string()
        };
        let lock_path = ws.join("Cargo.lock");
        // 1) 生成时尚无 lock：收据 absent，门新鲜。
        assert_eq!(read_lock_field(), "absent", "生成时尚无 lock");
        assert!(backend_generation_is_fresh(&project), "无 lock 物化时新鲜");
        // 2) 空文件出现：门新鲜（一次绑定），收据绑定实际身份 811c9dc5。
        std::fs::write(&lock_path, b"").unwrap();
        assert!(
            backend_generation_is_fresh(&project),
            "absent→空文件首次物化必须新鲜（一次绑定）"
        );
        assert_eq!(
            read_lock_field(),
            format!("{:x}", auto_api_gen::api_gen::fnv1a(b"")),
            "空文件必须绑定实际内容身份（811c9dc5），不得归 absent"
        );
        assert!(
            backend_generation_is_fresh(&project),
            "绑定后同空文件仍新鲜"
        );
        // 3) R11 反例转正：同一空文件上再生成——收据仍记 811c9dc5（两端
        //    一致），业务与 lock 均未变，门必须新鲜。
        auto_api_gen::api_gen::generate_api(&project, "rust").unwrap();
        assert_eq!(
            read_lock_field(),
            format!("{:x}", auto_api_gen::api_gen::fnv1a(b"")),
            "生成端对空文件必须记同一实际身份（两端一致）"
        );
        assert!(
            backend_generation_is_fresh(&project),
            "空 lock 再生成成功后未变输入不得判 stale"
        );
        // 4) 非空漂移：陈旧。
        std::fs::write(&lock_path, b"version = 4\n").unwrap();
        assert!(
            !backend_generation_is_fresh(&project),
            "空→非空内容漂移必须陈旧"
        );
        // 5) 删除：陈旧（已绑定实际身份后消失≠absent/absent）。
        std::fs::remove_file(&lock_path).unwrap();
        assert!(!backend_generation_is_fresh(&project), "lock 删除必须陈旧");
        // 6) 非 NotFound 读错误（目录占位）：不可核验=保守陈旧（R9-01）。
        std::fs::create_dir(&lock_path).unwrap();
        assert!(
            !backend_generation_is_fresh(&project),
            "读错误（非 NotFound）不可核验，保守陈旧"
        );
        std::fs::remove_dir(&lock_path).unwrap();
    }

    /// Only two known equal fingerprints prove assembly freshness.
    #[test]
    fn assembly_freshness_truth_table() {
        assert!(!assembly_freshness(None, None), "缺少两侧身份不能证明新鲜");
        assert!(
            assembly_freshness(Some("aaaa00000000bbbb"), Some("aaaa00000000bbbb")),
            "指纹相等=新鲜"
        );
        assert!(
            !assembly_freshness(Some("aaaa000000000000"), Some("bbbb000000000000")),
            "指纹漂移（改 stdlib 但 api.at 不变）=陈旧"
        );
        assert!(
            !assembly_freshness(None, Some("aaaa000000000000")),
            "旧产物无 assembly 字段而当前可算=陈旧（门升级首查再生收敛）"
        );
        assert!(
            !assembly_freshness(Some("aaaa000000000000"), None),
            "收据有而当前不可算=无法核验，保守陈旧"
        );
    }

    /// PLAN-031 T-06：生成 main 的 rqhost 臂在场——`--autodesk-rqhost`
    /// 标记解析 + `ClientTarget::Rqhost`（rendezvous 采纳 + exit-on-EOF
    /// 策略档）；直连/broker 既有臂零变化（I2 内容面）。
    #[test]
    fn generated_main_has_rqhost_gate() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("p031-app");
        std::fs::create_dir_all(project.join("src").join("front")).unwrap();
        std::fs::write(
            project.join("src").join("front").join("app.at"),
            "widget P031Counter { model { var count int = 0 } view { text `c: ${.count}` } }\n",
        )
        .unwrap();
        let components = "use auto_lang::ui::{Component, View};\npub struct P031Counter { pub count: i64 }\nimpl Component for P031Counter { type Msg = i64; fn view(&self) -> View<Self::Msg> { View::new() } }\n";
        let main_rs = wrap_example("p031-app", components, &project, &PacSource::legacy());
        assert!(main_rs.contains(r##"--autodesk-rqhost""##), "标记解析在场");
        assert!(
            main_rs.contains("ClientTarget::Rqhost"),
            "Rqhost 目标臂在场"
        );
        // 既有直连/broker 臂不回退（I2）。
        assert!(main_rs.contains("ClientTarget::Direct"));
        assert!(main_rs.contains("ClientTarget::Broker"));
    }

    /// PLAN-693 T-01/T-03：生成 main 的三模式 CLI 臂在场——`--render-mode`/
    /// `-q`/`--rq` 解析与优先级链单源调用、rq 臂（exe 侧 ensure 孵化 +
    /// remote 帧宿主）、desktop 臂（`--desktop-endpoint` 缺席报错 + 不孵化
    /// 目标）；旧孵化臂零变化（I2）+ ClientOpts `remote:` 字段归位
    ///（PLAN-683 E0063 潜伏缺口根修 pin——生成物无日档编译门，字符串
    /// 锚点即最小防线）。
    #[test]
    fn generated_main_has_tri_mode_cli_gate() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("p693-app");
        std::fs::create_dir_all(project.join("src").join("front")).unwrap();
        std::fs::write(
            project.join("src").join("front").join("app.at"),
            "widget P693App { model { var count int = 0 } view { text `c: ${.count}` } }\n",
        )
        .unwrap();
        let components = "use auto_lang::ui::{Component, View};\npub struct P693App { pub count: i64 }\nimpl Component for P693App { type Msg = i64; fn view(&self) -> View<Self::Msg> { View::new() } }\n";
        let main_rs = wrap_example("p693-app", components, &project, &PacSource::legacy());
        // 三模式 CLI 解析 + 优先级链单源调用（CLI > env > pac > 独立轨）。
        assert!(
            main_rs.contains("render_cli::parse_render_cli"),
            "解析单源在场"
        );
        assert!(
            main_rs.contains("render_cli::resolve_render_base"),
            "优先级链单源在场"
        );
        // rq 臂：exe 侧孵化保活 + Rqhost 采纳。
        assert!(
            main_rs.contains("rqhost::ensure_rqhost_ready"),
            "exe 侧孵化在场"
        );
        assert!(main_rs.contains("ClientTarget::Rqhost"));
        // desktop 臂：不孵化直连 + 端点缺席报错（AC-03 生成面）。
        assert!(main_rs.contains("ClientTarget::Desktop"));
        assert!(main_rs.contains("--desktop-endpoint"));
        assert!(main_rs.contains("请先启动虚拟桌面"));
        // remote 字段归位（E0063 根修 pin）：rq/desktop 两臂 true + 旧臂 false。
        assert_eq!(
            main_rs.matches("remote: true").count(),
            2,
            "rq/desktop 臂 remote 帧宿主"
        );
        assert!(main_rs.contains("remote: false"), "旧孵化臂 E0063 归位");
        // PLAN-694 T-01 认领对称性 pin：rq/desktop 两臂消费 `--app386=`
        // 覆盖 Hello app_name（宿主 spawn 认领按目录名匹配同源——旧臂
        // 同款语义；缺席臂 = 认领错配潜伏缺口）。
        assert_eq!(
            main_rs.matches(r#"strip_prefix("--app386=")"#).count(),
            3,
            "rq/desktop/旧臂三方言同消费 --app386"
        );
        // 既有直连/broker 臂不回退（I2）。
        assert!(main_rs.contains("ClientTarget::Direct"));
        assert!(main_rs.contains("ClientTarget::Broker"));
    }

    /// PLAN-609 T-A1 三段：①theme{} 声明经 boot 路径激活合成主题
    /// （theme_name=声明名、resolve 走合成体、mode/accent 不被清写）；
    /// ②无声明零变化；③坏 extends compose 失败容错回退。
    /// ACTIVE_THEME/DARK_MODE/ACCENT_NAME 均 thread-local —— nextest
    /// 每测独立进程天然隔离。
    #[test]
    fn vm_boot_pac_theme_decl_activates_composed() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("p609-app");
        std::fs::create_dir_all(project.join("src").join("front")).unwrap();
        std::fs::write(
            project.join("pac.at"),
            "name: \"p609-app\"\ntheme: {\n    name: \"brand-x\"\n    extends: \"stella\"\n    mode: \"auto\"\n    colors: {\n        primary: \"#8b5cf6\"\n    }\n}\n",
        )
        .unwrap();

        // ① 声明激活：合成名上槽，resolve 走合成体（VM face == 合成体 face;
        // hex 声明经 HSL 域往返,字节保真在合成体本体断言）。
        let activated = apply_pac_theme_decl(&project).expect("声明应激活");
        assert_eq!(activated, "brand-x", "激活名=声明名");
        assert_eq!(
            auto_ui_compiler::style::theme::theme_name(),
            "brand-x",
            "theme_name 返回合成主题名"
        );
        let composed =
            auto_ui_compiler::style::theme::active_composed().expect("合成主题在 ACTIVE_THEME 槽");
        let expected_primary =
            auto_lang_core::design_tokens::decl::normalize_value("#8b5cf6").expect("声明值合法");
        assert!(
            composed.light.iter().any(|(t, v)| *t
                == auto_ui_compiler::style::theme::registry::TokenName::Primary
                && v == &expected_primary),
            "合成体 primary = 声明色规范化值（normalize hex→HSL 串）"
        );
        assert_eq!(
            auto_ui_compiler::style::theme::active_theme_rgb(
                auto_ui_compiler::style::theme::registry::TokenName::Primary,
                false,
            ),
            composed.resolve_rgb(
                auto_ui_compiler::style::theme::registry::TokenName::Primary,
                false,
            ),
            "VM resolve 走合成体（与合成体自身解析同值）"
        );

        // ①b 优先级成文：声明激活不清写 mode/accent（env 种子语义位不动）。
        assert!(
            auto_ui_compiler::style::theme::dark_mode(),
            "DARK_MODE 不被声明清写"
        );
        assert_eq!(
            auto_ui_compiler::style::theme::accent_name(),
            "indigo",
            "ACCENT_NAME 不被声明清写"
        );
    }

    #[test]
    fn vm_boot_no_theme_decl_zero_change() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("p609-plain");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(project.join("pac.at"), "name: \"p609-plain\"\n").unwrap();

        let before = auto_ui_compiler::style::theme::theme_name();
        assert_eq!(apply_pac_theme_decl(&project), None, "无声明 → 无激活");
        assert_eq!(
            auto_ui_compiler::style::theme::theme_name(),
            before,
            "无声明零变化（内置缺省链不动）"
        );

        // pac.at 缺失同律（run_vm_ui 对独立于 pac 的路径零变化）。
        let empty = tmp.path().join("p609-nopac");
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(apply_pac_theme_decl(&empty), None, "pac.at 缺失零变化");
    }

    #[test]
    fn vm_boot_bad_theme_decl_falls_back() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("p609-bad");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(
            project.join("pac.at"),
            "name: \"p609-bad\"\ntheme: {\n    name: \"broken\"\n    extends: \"no-such-base\"\n}\n",
        )
        .unwrap();

        let before = auto_ui_compiler::style::theme::theme_name();
        assert_eq!(
            apply_pac_theme_decl(&project),
            None,
            "compose 失败（extends 非内置）→ 不激活"
        );
        assert_eq!(
            auto_ui_compiler::style::theme::theme_name(),
            before,
            "坏声明回退内置缺省，不 panic 不换槽"
        );
    }

    /// Stage B P-2：落点解析序三分支——env 权威 / 框架内项目=共享 ws（零
    /// 变化）/ 仓外项目=project-local。框架内检测用 worktree 内真实路径
    /// （CWD 上行探测命中本仓根），仓外用 temp 目录（必然在框架根之外）。
    #[test]
    fn rust_workspace_resolution_order() {
        std::env::remove_var("AUTO_RUST_WORKSPACE");
        // 仓外：temp 项目 → project-local。
        let tmp = tempfile::tempdir().unwrap();
        let out_project = tmp.path().join("auto-out-app");
        std::fs::create_dir_all(&out_project).unwrap();
        assert_eq!(
            resolve_rust_workspace_dir(&out_project),
            out_project.join("rust-workspace"),
            "仓外项目落 project-local"
        );
        // 框架内：本仓（worktree）下的真实路径（包根 current_dir）→ 共享
        // 工作区（零变化）。
        let in_project = std::env::current_dir().unwrap();
        assert_eq!(
            resolve_rust_workspace_dir(&in_project),
            get_rust_workspace_dir(),
            "框架内项目保持共享工作区"
        );
        // env 设置即权威。
        std::env::set_var("AUTO_RUST_WORKSPACE", "D:/custom-ws");
        assert_eq!(
            resolve_rust_workspace_dir(&out_project),
            PathBuf::from("D:/custom-ws")
        );
        std::env::remove_var("AUTO_RUST_WORKSPACE");
    }

    /// Stage B P-2：仓外项目 ensure——ws 落 project-local + rel 锚点从实际
    /// 落点计算 + **框架共享工作区 Cargo.toml 字节不变**（V4 零污染）。
    #[test]
    fn ensure_workspace_lands_project_local() {
        std::env::remove_var("AUTO_RUST_WORKSPACE");
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("kanban-fake");
        std::fs::create_dir_all(&project).unwrap();
        let fw_ws_cargo = get_rust_workspace_dir().join("Cargo.toml");
        let fw_before = std::fs::read_to_string(&fw_ws_cargo).ok();

        let ws = ensure_shared_workspace(&project);
        assert_eq!(ws, project.join("rust-workspace"), "落点 project-local");
        let manifest =
            std::fs::read_to_string(ws.join("Cargo.toml")).expect("workspace Cargo.toml written");
        assert!(manifest.contains("[workspace]"), "虚拟 manifest 在案");
        // rel 锚点：从 temp 落点指向 auto-lang（temp 布局无兄弟 auto-lang，
        // 走 fallback 字面量——断言其存在即可；真实布局由 586/583 实证覆盖）。
        assert!(
            manifest.contains("auto-lang = { path = "),
            "path 依赖注入在案"
        );
        assert!(
            std::fs::read_to_string(ws.join(".cargo").join("config.toml"))
                .map(|c| c.contains("target-dir"))
                .unwrap_or(false),
            "target-dir 共享缓存在案"
        );
        // V4：框架共享工作区零触碰。
        let fw_after = std::fs::read_to_string(&fw_ws_cargo).ok();
        assert_eq!(fw_before, fw_after, "框架 ws Cargo.toml 字节不变");
        // 幂等：二次 ensure 落点不变。
        assert_eq!(ensure_shared_workspace(&project), ws);
    }

    /// PLAN-039 T-05 发现臂⑤：无框架祖先的 app 仓布局（temp 落点同构）
    /// target-dir 落 workspace 本地默认 "target"——幻影 fallback
    /// （"../../autostack/auto-lang/target"）废除（生成物重定向到不存
    /// 在布局，desktop_exe 约定路径随之失配）。
    #[test]
    fn app_repo_layout_targets_workspace_local_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("minesweeper-fake");
        std::fs::create_dir_all(&project).unwrap();
        let ws = ensure_shared_workspace(&project);
        let config = std::fs::read_to_string(ws.join(".cargo").join("config.toml"))
            .expect("config.toml written");
        assert!(
            config.contains("target-dir = \"target\""),
            "app 仓布局落本地默认 target:\n{config}"
        );
        assert!(
            !config.contains("autostack"),
            "幻影路径不得再出现:\n{config}"
        );
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MyApp"), "my_app");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("App"), "app");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("lowercase"), "lowercase");
    }

    #[test]
    fn test_extract_main_widget_prefers_app() {
        let code = r#"
pub struct Counter {
    pub count: i32,
}

pub struct App {
    pub title: String,
}
"#;
        assert_eq!(extract_main_widget(code), "App");
    }

    #[test]
    fn test_extract_main_widget_fallback_first_struct() {
        let code = r#"
pub struct Counter {
    pub count: i32,
}

pub struct Timer {
    pub seconds: i32,
}
"#;
        assert_eq!(extract_main_widget(code), "Counter");
    }

    #[test]
    fn test_extract_main_widget_empty() {
        let code = "// no structs here";
        assert_eq!(extract_main_widget(code), "App");
    }

    #[test]
    fn test_parse_pac_name() {
        let dir = std::env::temp_dir().join("auto_test_pac");
        fs::create_dir_all(&dir).ok();
        let pac_path = dir.join("pac.at");
        fs::write(&pac_path, r#"name: "TestProject""#).ok();
        assert_eq!(parse_pac_name(&pac_path), Some("TestProject".to_string()));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_gen_015_notes_rust() {
        // Quick generation test for 015-notes Rust UI code
        let project_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples")
            .join("ui")
            .join("015-notes");
        if !project_dir.exists() {
            eprintln!("Skipping: project dir not found at {:?}", project_dir);
            return;
        }
        let result = super::generate_rust_ui(&project_dir, None, false);
        match result {
            Ok(()) => println!("Generation succeeded!"),
            Err(e) => panic!("Generation failed: {}", e),
        }
    }

    /// PLAN-648 T-03 生成器行为锁定 ①:db.at 吸收臂优先于 CRUD 原型。
    /// api.at 端点在 db.at 有同名实现且标量面 → 生成的 merged client
    /// 直接委托 `db::<fn>`,零 API_DATA 运行时触碰(路由语义 = 真实现)。
    #[test]
    fn merged_api_client_db_absorption_wins_over_crud() {
        let tmp = tempfile::tempdir().unwrap();
        let back = tmp.path().join("src").join("back");
        std::fs::create_dir_all(&back).unwrap();
        std::fs::write(
            back.join("api.at"),
            r#"
#[api(method = "GET", path = "/api/ping")]
pub fn ping() int {
    return 0
}
"#,
        )
        .unwrap();
        std::fs::write(
            back.join("db.at"),
            r#"
pub fn ping() int {
    return 42
}
"#,
        )
        .unwrap();
        let module = super::parse_api_module(tmp.path()).expect("api module parses");
        let code = super::generate_merged_api_client(&module, tmp.path());
        assert!(
            code.contains("db::ping"),
            "absorption arm should delegate ping to db::ping, got:\n{}",
            code
        );
        // 端点 fn 区段(static 声明之后)不得触碰 API_DATA 运行时。
        let endpoint_section = code.split("static API_NEXT_ID").nth(1).unwrap_or("");
        assert!(
            !endpoint_section.contains("API_DATA"),
            "absorbed endpoint must not touch API_DATA, got:\n{}",
            code
        );
    }

    /// PLAN-648 T-03 生成器行为锁定 ②:CRUD 原型 = 明示保留的 demo
    /// 脚手架兜底(消费方 = 无 db.at 演示应用,如 020-music-player /
    /// 031-image-viewer)。吸收臂不覆盖的端点回落 API_DATA 脚手架,
    /// 形态锁定:GET 无参 = 全量列表、POST = 增记录。
    #[test]
    fn merged_api_client_crud_fallback_for_uncovered_endpoints() {
        let tmp = tempfile::tempdir().unwrap();
        let back = tmp.path().join("src").join("back");
        std::fs::create_dir_all(&back).unwrap();
        // 无 db.at —— 全部端点走 CRUD 兜底。
        std::fs::write(
            back.join("api.at"),
            r#"
#[api(method = "GET", path = "/api/items")]
pub fn list_items() []str {
    return []
}

#[api(method = "POST", path = "/api/items")]
pub fn create_item(name str) str {
    return ""
}
"#,
        )
        .unwrap();
        let module = super::parse_api_module(tmp.path()).expect("api module parses");
        let code = super::generate_merged_api_client(&module, tmp.path());
        assert!(
            code.contains("static API_DATA"),
            "CRUD fallback must keep the API_DATA scaffold, got:\n{}",
            code
        );
        assert!(
            code.contains("fn list_items() -> Vec<Value>"),
            "GET no-param fallback = list-all over API_DATA, got:\n{}",
            code
        );
        assert!(
            code.contains("fn create_item(") && code.contains("API_NEXT_ID"),
            "POST fallback = id-allocating insert over API_DATA, got:\n{}",
            code
        );
    }

    /// Stage B P-2 V4：仓外项目完整生成链——member 落 project-local，框架
    /// 共享工作区与仓内 examples 零触碰（015-notes 回查由 test_gen_015_notes_rust
    /// 既有锚承载：框架内项目仍走共享 ws）。
    #[test]
    fn generate_rust_ui_out_of_repo_lands_project_local() {
        std::env::remove_var("AUTO_RUST_WORKSPACE");
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples")
            .join("ui")
            .join("001-helloworld");
        if !src.exists() {
            eprintln!("Skipping: fixture not found at {:?}", src);
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("helloworld-out");
        fs::create_dir_all(project.join("src").join("front")).unwrap();
        for f in ["pac.at", "src/front/app.at"] {
            fs::copy(src.join(f), project.join(f)).unwrap();
        }
        let fw_ws_cargo = get_rust_workspace_dir().join("Cargo.toml");

        generate_rust_ui(&project, None, false).expect("out-of-repo generation succeeds");
        // member 落 project-local。
        let member = project.join("rust-workspace").join("helloworld-out");
        assert!(
            member.join("Cargo.toml").exists(),
            "member lands project-local at {}",
            member.display()
        );
        // V4：框架共享工作区零触碰。PLAN-659 T-08：字节不变断言在并行
        // 全套下与**同档其他仓内生成测试**（共享写 `examples/rust-workspace/
        // Cargo.toml`）存在跨进程读写竞态（P642-D15② 并行红机理）——改为
        // 成员缺席断言：本测试的 member 不得出现在框架工作区（对无关
        // 并行写入者免疫，回归语义不变：错落框架仓即被捕获）。
        let fw_after = fs::read_to_string(&fw_ws_cargo).unwrap_or_default();
        assert!(
            !fw_after.contains("helloworld-out"),
            "framework ws must not gain this project's member; Cargo.toml tail:\n{}",
            fw_after.chars().rev().take(400).collect::<String>()
        );
    }

    #[test]
    fn test_collect_at_files_skips_pac() {
        let dir = std::env::temp_dir().join("auto_test_collect");
        fs::create_dir_all(&dir).ok();
        fs::write(dir.join("app.at"), "").ok();
        fs::write(dir.join("pac.at"), "name: test").ok();
        fs::write(dir.join("other.at"), "").ok();

        let files = collect_at_files(&dir).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"app.at".to_string()));
        assert!(names.contains(&"other.at".to_string()));
        assert!(!names.contains(&"pac.at".to_string()));

        fs::remove_dir_all(&dir).ok();
    }

    // --- Plan 388 W1: API client migrated from ureq to reqwest::blocking + TLS ---

    #[test]
    fn test_w1_http_client_helper_emits_tls_client() {
        // The split-mode helper must build a reqwest::blocking client honouring
        // AUTO_TLS_SKIP_VERIFY / AUTO_TLS_CA_CERT (OnceLock-cached), so endpoint
        // functions can configure TLS (ureq could not).
        let helper = generate_http_client_helper();
        assert!(
            helper.contains("fn _http_client() -> reqwest::blocking::Client"),
            "missing _http_client"
        );
        assert!(
            helper.contains("AUTO_TLS_SKIP_VERIFY"),
            "missing skip-verify env"
        );
        assert!(helper.contains("AUTO_TLS_CA_CERT"), "missing CA env");
        assert!(
            helper.contains("danger_accept_invalid_certs"),
            "missing skip-verify wiring"
        );
        assert!(
            helper.contains("add_root_certificate"),
            "missing custom CA wiring"
        );
        assert!(helper.contains("OnceLock"), "client should be built once");
    }

    #[test]
    fn test_w1_get_body_uses_reqwest_client() {
        // GET body must route through _http_client() and reqwest's .send()/.json(),
        // never emit `ureq::`.
        // PLAN-021 T-09:Option 返回改声明类型直反序列化 + .flatten() 归一
        // (此前硬编码 Value 使返回面与声明错位)。
        for (ret, expected) in [
            ("serde_json::Value", "_http_client().get("),
            (
                "Vec<serde_json::Value>",
                "r.json::<Vec<serde_json::Value>>().ok()",
            ),
            (
                "Option<serde_json::Value>",
                "r.json::<Option<serde_json::Value>>().ok()",
            ),
        ] {
            let body = generate_get_fn_body(
                "get".into(),
                "&format!(\"http://x/{}\", id)".into(),
                false,
                ret,
            );
            assert!(
                body.contains(expected),
                "GET({ret}) missing {expected}: {body}"
            );
            assert!(
                !body.contains("ureq"),
                "GET({ret}) still emits ureq: {body}"
            );
        }
        let option = generate_get_fn_body(
            "get".into(),
            "&format!(\"http://x/{}\", id)".into(),
            false,
            "Option<serde_json::Value>",
        );
        assert!(
            option.contains(".flatten()"),
            "GET(Option) must flatten: {option}"
        );
        let void = generate_get_fn_body("get".into(), "\"http://x\"".into(), true, "");
        assert!(
            void.contains("_http_client().get(\"http://x\").send()"),
            "void GET: {void}"
        );
        assert!(!void.contains("ureq"), "void GET still emits ureq: {void}");
    }

    #[test]
    fn test_w1_write_and_delete_bodies_use_reqwest_client() {
        // POST/PUT fire-and-forget + blocking bodies, and DELETE, must use
        // _http_client() and reqwest .json(&body)/.send().
        let params: Vec<&auto_lang_core::api::types::ApiParam> = vec![];
        let void = generate_write_fn_body("post".into(), "\"http://x\"".into(), &params, true, "");
        assert!(
            void.contains("_http_client().post(&url).json(&body).send()"),
            "void POST: {void}"
        );
        assert!(!void.contains("ureq"), "void POST still emits ureq: {void}");

        let blocking = generate_write_fn_body(
            "post".into(),
            "\"http://x\"".into(),
            &params,
            false,
            "serde_json::Value",
        );
        // Value-returning POST is non-blocking: placeholder + background thread.
        assert!(
            blocking.contains("_http_client().post(&url).json(&body).send()"),
            "blocking POST: {blocking}"
        );
        assert!(
            blocking.contains("local_result"),
            "blocking POST must return placeholder: {blocking}"
        );
        assert!(
            !blocking.contains("ureq"),
            "blocking POST still emits ureq: {blocking}"
        );

        // PLAN-021 T-09:声明标量返回(int)的 POST 改阻塞反序列化——
        // 调用方按声明消费返回值,占位 Value 破坏类型契约。
        let scalar_blocking =
            generate_write_fn_body("post".into(), "\"http://x\"".into(), &params, false, "i32");
        assert!(
            scalar_blocking.contains("r.json::<i32>().ok()"),
            "scalar POST must block+deserialize: {scalar_blocking}"
        );
        assert!(
            !scalar_blocking.contains("local_result"),
            "scalar POST must not return placeholder: {scalar_blocking}"
        );

        let bool_result =
            generate_write_fn_body("post".into(), "\"http://x\"".into(), &params, false, "bool");
        // Boolean results are control-flow values (for example, save
        // acknowledgements), so the generated client must parse the server
        // response instead of returning a placeholder from a background task.
        assert!(
            bool_result.contains("r.json::<bool>().ok()"),
            "bool POST must parse response: {bool_result}"
        );
        assert!(
            !bool_result.contains("local_result"),
            "bool POST must not return placeholder: {bool_result}"
        );
        assert!(
            !bool_result.contains("std::thread::spawn"),
            "bool POST must wait for result: {bool_result}"
        );

        let del = generate_delete_fn_body("\"http://x/1\"".into());
        assert!(
            del.contains("_http_client().delete(&url).send()"),
            "DELETE: {del}"
        );
        assert!(!del.contains("ureq"), "DELETE still emits ureq: {del}");
    }

    // --- Plan 388 W2/W3: multipart chain + download progress ---

    #[test]
    fn test_w2_multipart_chainable_builder() {
        let utils = generate_http_utility_functions();
        // Chainable builder mirroring VM RequestBuilder.multipart_file/multipart_text
        assert!(
            utils.contains("fn multipart_form() -> MultiPart"),
            "missing multipart_form"
        );
        assert!(
            utils.contains("struct MultiPart"),
            "missing MultiPart struct"
        );
        assert!(
            utils.contains("fn text(mut self, field: &str, value: &str) -> Self"),
            "missing .text()"
        );
        assert!(
            utils.contains("fn file(mut self, field: &str, path: &str) -> Self"),
            "missing .file()"
        );
        assert!(
            utils.contains("fn send(self, url: &str) -> serde_json::Value"),
            "missing .send()"
        );
        assert!(
            utils.contains(".multipart(form)"),
            "send must attach multipart form"
        );
        // Existing generic uploads kept
        assert!(utils.contains("fn upload_file("), "upload_file must remain");
        assert!(
            utils.contains("fn upload_file_with_fields("),
            "upload_file_with_fields must remain"
        );
        // Plan 388 review: all HTTP clients must be TLS-aware (_http_client),
        // not raw reqwest builders that ignore AUTO_TLS_*.
        assert!(
            !utils.contains("reqwest::blocking::Client::new"),
            "utils must not build raw clients"
        );
        assert!(
            !utils.contains("reqwest::blocking::get"),
            "utils must not use blocking::get"
        );
        assert!(
            utils.contains("_http_client()"),
            "utils must use TLS-aware _http_client"
        );
    }

    #[test]
    fn test_w3_download_progress_iterator() {
        let utils = generate_http_utility_functions();
        assert!(utils.contains("fn download_with_progress(url: &str, file_path: &str) -> std::sync::mpsc::Receiver<serde_json::Value>"),
            "missing download_with_progress");
        assert!(
            utils.contains("fn download_file(url: &str, file_path: &str) -> bool"),
            "download_file must remain"
        );
        assert!(
            utils.contains("fn download_file_resume("),
            "download_file_resume must remain"
        );
        assert!(
            utils.contains("copy_to"),
            "progress must stream via copy_to (not buffer-all)"
        );
        assert!(
            utils.contains("content_length()"),
            "progress must report total"
        );
        // Plan 388 review: downloads must go through the TLS-aware client too.
        assert!(
            utils.contains("_http_client().get"),
            "download must use TLS-aware _http_client"
        );
    }

    // --- Plan 388 W4: WebSocket client ---

    #[test]
    fn test_w4_ws_on_message_delivers_frames() {
        let ws = generate_ws_functions();
        for f in ["ws_connect", "ws_send", "ws_on_message", "ws_close"] {
            assert!(ws.contains(&format!("fn {}(", f)), "missing {f}");
        }
        // Reader thread must forward frames to a per-conn queue consumed by ws_on_message
        assert!(
            ws.contains("msg_tx.send"),
            "reader must deliver frames to a queue"
        );
        assert!(
            ws.contains("receiver: std::sync::mpsc::Receiver<String>"),
            "WsConn must hold the incoming queue"
        );
        assert!(
            ws.contains("fn ws_on_message(handle: i32) -> Vec<String>"),
            "ws_on_message must drain to Vec<String>"
        );
        // No unused Arc import
        assert!(
            !ws.contains("use std::sync::{Arc, Mutex}"),
            "Arc import must be gone"
        );
        // Plan 388 review: read timeout must cover the MaybeTlsStream variants
        // that exist in generated projects (plain ws:// + native-TLS wss://,
        // matching the `native-tls` tungstenite feature in the Cargo template) —
        // otherwise the outgoing-channel deadlock persists for TLS connections.
        // The `_` arm satisfies #[non_exhaustive] (and rustls-enabled builds).
        assert!(
            ws.contains("MaybeTlsStream::Plain(tcp) => { let _ = tcp.set_read_timeout(timeout); }"),
            "plain ws:// must get a read timeout"
        );
        assert!(ws.contains("MaybeTlsStream::NativeTls(tls) => { let _ = tls.get_ref().set_read_timeout(timeout); }"),
            "native-TLS wss:// must get a read timeout");
        assert!(
            ws.contains("_ => {}"),
            "match must be exhaustive (non_exhaustive enum)"
        );
        // Plan 388 review: ws_close must actually terminate the reader thread
        // (close flag → socket dropped), not just sever the send path.
        assert!(
            ws.contains("close_reader.load"),
            "reader must check the close flag"
        );
        assert!(
            ws.contains("conn.close.store(true"),
            "ws_close must set the close flag"
        );
    }

    #[test]
    fn test_w6_helper_emits_cookie_gzip_brotli_env() {
        // Plan 349 步骤 7/8 (W6): the generated _http_client() helper must wire cookie
        // store + gzip + brotli behind env flags, mirroring the VM-side
        // RequestBuilder.cookie_store/gzip/brotli flags. Defaults stay off so
        // existing behavior is unchanged.
        let helper = generate_http_client_helper();
        assert!(
            helper.contains("AUTO_COOKIE_STORE"),
            "missing cookie env flag"
        );
        assert!(
            helper.contains("cookie_store(true)"),
            "cookie_store not wired"
        );
        assert!(helper.contains("AUTO_GZIP"), "missing gzip env flag");
        assert!(helper.contains(".gzip(true)"), "gzip not wired");
        assert!(helper.contains("AUTO_BROTLI"), "missing brotli env flag");
        assert!(helper.contains(".brotli(true)"), "brotli not wired");
    }

    #[test]
    fn test_w6_cargo_toml_has_compression_cookie_features() {
        // Plan 349 步骤 7/8 (W6): generated Cargo.toml must declare the reqwest features
        // the helper relies on (cookies + gzip + brotli), alongside the
        // pre-existing blocking/json/multipart.
        let toml = generate_cargo_toml(
            "test_proj",
            std::path::Path::new("/tmp/test"),
            &PacSource::legacy(),
        );
        assert!(
            toml.contains("\"cookies\""),
            "missing cookies feature: [{}]",
            toml
        );
        assert!(
            toml.contains("\"gzip\""),
            "missing gzip feature: [{}]",
            toml
        );
        assert!(
            toml.contains("\"brotli\""),
            "missing brotli feature: [{}]",
            toml
        );
    }

    #[test]
    fn test_plan015_title_zh_never_enters_cargo_toml() {
        // PLAN-015 反向钉：展示名字段（title_zh）与产物名解耦——pac 只声明
        // title/title_zh 时不得生成 [[bin]]（产物名仍走包名缺省链）；
        // exe_name 声明臂行为不变（Plan 014）。
        let tmp = std::env::temp_dir().join("plan015-title-zh-cargo-probe");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("pac.at"),
            "name: \"calc\"\ntitle: \"Calculator\"\ntitle_zh: \"计算器\"\n",
        )
        .unwrap();
        let toml = generate_cargo_toml("calc", &tmp, &PacSource::legacy());
        assert!(
            !toml.contains("[[bin]]"),
            "title_zh 不得触发 [[bin]]: [{}]",
            toml
        );
        std::fs::write(
            tmp.join("pac.at"),
            "name: \"calc\"\ntitle_zh: \"计算器\"\nexe_name: \"auto-term\"\n",
        )
        .unwrap();
        let toml = generate_cargo_toml("calc", &tmp, &PacSource::legacy());
        assert!(
            toml.contains("[[bin]]\nname = \"auto-term\""),
            "exe_name 显式臂不变: [{}]",
            toml
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// PLAN-036 T-02：freshness 门——以权威源（auto-os/shell）重生成对拍
    /// 入库物 `crates/shell-pack/src/lib.rs`（字节一致；solo 检出跳过）。
    /// pack 改动后跑 `regen_shell_pack`（#[ignore]）重生成 + 提交，否则
    /// 本门红。
    #[test]
    fn test_shell_pack_lib_freshness() {
        let Some(pack_dir) = auto_lang_core::os_paths::resolve_os_top_dir(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."),
            "shell",
        ) else {
            eprintln!("test_shell_pack_lib_freshness: SKIPPED — auto-os/shell 未解析(solo 检出)");
            return;
        };
        let lib_rs =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/shell-pack/src/lib.rs");
        let Ok(committed) = std::fs::read_to_string(&lib_rs) else {
            panic!(
                "crates/shell-pack/src/lib.rs 缺席——先跑 regen_shell_pack（#[ignore] 测试）生成入库物"
            );
        };
        let (generated, _) = generate_shell_pack_lib_sources(&pack_dir)
            .expect("shell pack lib 生成失败（词汇/结构缺口——先修 a2r 臂）");
        assert!(
            generated == committed,
            "shell-pack 入库物过期（pack 已改未 regen）：重跑 regen_shell_pack（#[ignore]）+ 提交"
        );
    }

    /// PLAN-036 T-02：regen 入口——生成 shell-pack lib crate 到入库位
    /// （`crates/shell-pack/`；覆盖 lib.rs/Cargo.toml，tests.rs 手写不动）。
    /// 显式跑：`cargo test -p auto-man regen_shell_pack -- --ignored --nocapture`。
    #[test]
    #[ignore = "显式 regen（pack 改动后手动跑——写入库位）"]
    fn regen_shell_pack() {
        let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/shell-pack");
        generate_shell_pack_lib(&out).expect("shell pack regen 失败");
    }

    // ==================== PLAN-792 交付模式定向测试 ====================

    /// 测试用 producer 源根（编译期 CARGO_MANIFEST_DIR 上溯到仓根——与
    /// delivery_producer_root 的末位候选同源，测试内显式钉死避免 CWD 依赖）。
    fn test_producer_root() -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .to_path_buf();
        assert!(root.join("crates").join("auto-lang").is_dir());
        root
    }

    /// PLAN-792：pin 提取——单版本槽精确钉扎、多版本槽显式登记（清单无法
    /// 对同名多槽包重复声明直依赖）、锁缺失受管判红。
    #[test]
    fn delivery_pins_single_and_multi_slot() {
        let dir = tempfile::tempdir().unwrap();
        let lock = dir.path().join("Cargo.lock");
        let _ = &lock;
        std::fs::write(
            &lock,
            "[[package]]\nname = \"wgpu-hal\"\nversion = \"27.0.4\"\n\n\
             [[package]]\nname = \"windows-core\"\nversion = \"0.57.0\"\n\n\
             [[package]]\nname = \"windows-core\"\nversion = \"0.62.2\"\n",
        )
        .unwrap();
        let (pins, multi) = delivery_pins(dir.path()).unwrap();
        assert!(
            pins.contains(&("wgpu-hal".to_string(), "=27.0.4".to_string())),
            "单槽 wgpu-hal 精确钉扎：{pins:?}"
        );
        assert_eq!(multi, vec!["windows-core".to_string()], "多槽登记");
        assert!(!pins.iter().any(|(n, _)| n == "windows-core"));

        let missing = dir.path().join("absent");
        assert!(delivery_pins(&missing).is_err(), "锁缺失受管判红");
    }

    /// PLAN-792：交付根 manifest 装配——[patch.crates-io] 在消费根（不写
    /// 在依赖包内）、default-members 只含 app、[workspace.dependencies] 表
    /// 为 producer 根表字节副本。
    #[test]
    fn delivery_root_manifest_shape() {
        let dir = tempfile::tempdir().unwrap();
        let root_manifest = dir.path().join("Cargo.toml");
        std::fs::write(
            &root_manifest,
            "[workspace]\nmembers = [\"crates/x\"]\n\n\
             [workspace.dependencies]\nserde_json = \"1\"\ntoml = \"0.8\"\n\n\
             [profile.dev]\ndebug = \"line-tables-only\"\n\n\
             [patch.crates-io]\niced_widget = { path = \"crates/patches/iced_widget\" }\n",
        )
        .unwrap();
        let manifest = generate_delivery_root_manifest(dir.path(), "app").unwrap();
        assert!(
            manifest.contains(
                "[patch.crates-io]\niced_widget = { path = \"crates/patches/iced_widget\" }"
            ),
            "消费根 patch 就位"
        );
        assert!(
            manifest.contains("default-members = [\"app\"]"),
            "plain cargo build 只建 app"
        );
        assert!(
            manifest.contains("[workspace.dependencies]\nserde_json = \"1\"\ntoml = \"0.8\"\n"),
            "表副本逐字节同源"
        );
        assert!(
            manifest.contains("[profile.dev]\ndebug = \"line-tables-only\""),
            "profile 副本随包"
        );
        assert!(
            manifest.contains("\"crates/auto-lang\""),
            "受管闭包成员在列"
        );
        // 缺段判红
        let bare = dir.path().join("bare");
        std::fs::create_dir_all(&bare).unwrap();
        std::fs::write(bare.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        assert!(generate_delivery_root_manifest(&bare, "app").is_err());
    }

    /// PLAN-792：投影修剪——剥闭包外 optional path 依赖 + feature 行改写保
    /// 留 feature 名；幂等（二次运行零改动）；计数不足受管判红。
    #[test]
    fn delivery_projection_strip_and_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let m = dir.path().join("Cargo.toml");
        // T-05 fact update: the auto-lang manifest no longer carries the cross-repo
        // autodown-core optional dep (814-T-04 moved it to auto-ui-runtime), so this
        // spec keeps only the test-protocol strip/rewrite; the runtime shape is
        // verified by the runtime-spec arm below.
        let original = "[features]\n\
             test-cases = [\"dep:auto-test-case\", \"auto-lang-core/test-cases\"]\n\n\
             [dependencies]\n# comment\n\
             auto-test-case = { path = \"../auto-test-case\", optional = true }\n\
             auto-val = { path = \"../auto-val\" }\n";
        std::fs::write(&m, original).unwrap();
        let edits = delivery_apply_projection(&m, &PROJECTION_AUTO_LANG).unwrap();
        assert_eq!(edits.len(), 2, "1 strip + 1 rewrite");
        let text = std::fs::read_to_string(&m).unwrap();
        assert!(
            !text.contains("auto-test-case"),
            "test-protocol dep stripped"
        );
        assert!(text.contains("test-cases = []"), "feature name kept");
        assert!(
            text.contains("auto-val = { path = \"../auto-val\" }"),
            "in-closure dependency untouched"
        );
        // Idempotent: a second pass over the trimmed manifest makes zero edits.
        let stripped = std::fs::read_to_string(&m).unwrap();
        delivery_apply_projection(&m, &PROJECTION_AUTO_LANG).unwrap();
        assert_eq!(std::fs::read_to_string(&m).unwrap(), stripped);
        // Under-count is red (a producer whose structure drifted).
        let drift = dir.path().join("drift.toml");
        std::fs::write(
            &drift,
            "[dependencies]\nauto-val = { path = \"../auto-val\" }\n",
        )
        .unwrap();
        assert!(delivery_apply_projection(&drift, &PROJECTION_AUTO_LANG).is_err());
        // PLAN-815 T-05: runtime spec — cross-repo autodown-core strip + feature
        // rewrite (the shape moved over from auto-lang by 814-T-04), idempotent,
        // drift-red, each holding on its own.
        let rt = dir.path().join("runtime.toml");
        std::fs::write(
            &rt,
            "[features]\nautodown = [\"ui-iced\", \"dep:autodown-core\"]\n\n[dependencies]\n# comment\nautodown-core = { path = \"../../../auto-down/autodown/packages/engine/rust\", optional = true }\n",
        )
        .unwrap();
        let rt_edits = delivery_apply_projection(&rt, &PROJECTION_AUTO_UI_RUNTIME).unwrap();
        assert_eq!(rt_edits.len(), 2, "1 strip + 1 rewrite");
        let rt_text = std::fs::read_to_string(&rt).unwrap();
        assert!(
            !rt_text.contains("autodown-core"),
            "cross-repo optional dep stripped"
        );
        assert!(
            rt_text.contains("autodown = [\"ui-iced\"]"),
            "feature name kept"
        );
        let rt_stripped = std::fs::read_to_string(&rt).unwrap();
        delivery_apply_projection(&rt, &PROJECTION_AUTO_UI_RUNTIME).unwrap();
        assert_eq!(std::fs::read_to_string(&rt).unwrap(), rt_stripped);
        let rt_drift = dir.path().join("runtime-drift.toml");
        std::fs::write(
            &rt_drift,
            "[dependencies]\nauto-val = { path = \"../auto-val\" }\n",
        )
        .unwrap();
        assert!(delivery_apply_projection(&rt_drift, &PROJECTION_AUTO_UI_RUNTIME).is_err());
        // PLAN-814 SD-08：SDK spec——旧形态 test-cases 行 + auto-test-case
        // 剥除（2 编辑）、幂等、漂移判红各自成立。
        let sdk = dir.path().join("sdk.toml");
        std::fs::write(
            &sdk,
            "[features]\ntest-cases = [\"dep:auto-test-case\"]\n\n[dependencies]\nauto-test-case = { path = \"../auto-test-case\", optional = true }\n",
        )
        .unwrap();
        let sdk_edits = delivery_apply_projection(&sdk, &PROJECTION_AUTO_LANG_CORE).unwrap();
        assert_eq!(sdk_edits.len(), 2, "1 strip + 1 rewrite");
        let sdk_text = std::fs::read_to_string(&sdk).unwrap();
        assert!(
            !sdk_text.contains("auto-test-case"),
            "SDK 测试协议 dep 剥除"
        );
        assert!(sdk_text.contains("test-cases = []"), "feature 名保留");
        delivery_apply_projection(&sdk, &PROJECTION_AUTO_LANG_CORE).unwrap();
        assert_eq!(std::fs::read_to_string(&sdk).unwrap(), sdk_text, "SDK 幂等");
        assert!(delivery_apply_projection(&drift, &PROJECTION_AUTO_LANG_CORE).is_err());
    }

    /// PLAN-792 端到端：generate_rust_ui_delivery 真实生成——镜像布局
    /// （crates/ + 根级编译输入）、自包含应用清单（path 依赖 + pin 块）、
    /// 资源 allowlist + hash、delivery-manifest.json 身份字段、外来目录
    /// 显式拒绝、再交付（标记在案）重生成。不跑 cargo（锁/构建由交付
    /// driver 的 V01/V02 覆盖——本测试是生成器内容门）。
    #[test]
    fn delivery_bundle_end_to_end() {
        let dir = tempfile::tempdir().unwrap();
        let producer = test_producer_root();
        let project = dir.path().join("proj");
        let front = project.join("src").join("front");
        std::fs::create_dir_all(&front).unwrap();
        std::fs::write(
            project.join("pac.at"),
            "{ name: \"ui-minimal\", scene: \"ui\", render: \"rust\", window: \"fit\" }",
        )
        .unwrap();
        std::fs::write(
            front.join("app.at"),
            "widget App {\n    model {\n        var count int = 0\n    }\n    view {\n        col {\n            text `Count: ${.count}`\n            image (src: \"avatar.png\") {\n                style: \"w-16 h-16\"\n            }\n            button \"+1\" { onclick: () => {.count += 1} }\n            style: \"items-center gap-4 p-6\"\n        }\n    }\n}\n",
        )
        .unwrap();
        std::fs::write(front.join("avatar.png"), b"\x89PNG-fixture-bytes").unwrap();

        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(v) => std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", v),
                    None => std::env::remove_var("AUTO_DELIVERY_SOURCE_ROOT"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("AUTO_DELIVERY_SOURCE_ROOT"));
        std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", &producer);

        let delivery = dir.path().join("delivery");
        generate_rust_ui_delivery(&project, &delivery).expect("交付生成失败");

        // 镜像布局 + 根级编译输入
        assert!(delivery.join("crates/auto-lang/src/lib.rs").is_file());
        assert!(delivery
            .join("crates/patches/iced_widget/src/lib.rs")
            .is_file());
        for rel in DELIVERY_ROOT_INPUTS {
            assert!(delivery.join(rel).is_file(), "根级编译输入缺失：{rel}");
        }
        // 根 manifest：消费根 patch + default-members + app 成员
        let root_manifest = std::fs::read_to_string(delivery.join("Cargo.toml")).unwrap();
        assert!(root_manifest.contains("[patch.crates-io]"));
        assert!(root_manifest.contains("default-members = [\"app\"]"));
        assert!(root_manifest.contains("\"app\""));
        // 投影修剪生效
        let vendored =
            std::fs::read_to_string(delivery.join("crates/auto-lang/Cargo.toml")).unwrap();
        assert!(
            !vendored.lines().any(|l| l.contains("autodown-core = ")),
            "跨仓 optional dep 边剥除"
        );
        assert!(
            !vendored.lines().any(|l| l.contains("auto-test-case = ")),
            "测试协议 dep 边剥除"
        );
        assert!(
            !vendored.contains("../../../auto-down"),
            "跨仓 path 引用零残留（重定位安全）"
        );
        assert!(vendored.contains("test-cases = []"));
        // 应用清单：自包含 path 依赖 + pin 块；无 workspace 继承
        let app_manifest = std::fs::read_to_string(delivery.join("app/Cargo.toml")).unwrap();
        assert!(app_manifest.contains("auto-lang = { path = \"../crates/auto-lang\" }"));
        assert!(!app_manifest.contains(".workspace = true"));
        assert!(
            app_manifest.contains("wgpu-hal = { version = \"=27."),
            "drift pin 在列"
        );
        // 生成 main.rs：handler + 本地资产引用（与开发模式同一编译相位）
        let main_rs = std::fs::read_to_string(delivery.join("app/src/main.rs")).unwrap();
        assert!(main_rs.contains("avatar.png"), "本地资产引用");
        assert!(main_rs.contains("count"), "handler 状态字段");
        // 资源 allowlist + hash（R1-F01：sha256——随包 verify.py 可独立校验）
        let asset = delivery.join("assets/avatar.png");
        assert!(asset.is_file(), "资源入包");
        let verify_entry = delivery.join("verify.py");
        assert!(verify_entry.is_file(), "随包受管校验入口 verify.py 在案");
        assert!(
            std::fs::read_to_string(&verify_entry)
                .unwrap()
                .contains("sha256"),
            "verify.py 消费清单 sha256"
        );
        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(delivery.join("delivery-manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest["schema"], "generated-ui-delivery/1");
        assert_eq!(manifest["verification"]["entry"], "verify.py");
        let resources = manifest["resources"].as_array().unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0]["rel"], "avatar.png");
        let content = std::fs::read(&asset).unwrap();
        use sha2::{Digest, Sha256};
        assert_eq!(
            resources[0]["sha256"].as_str().unwrap(),
            format!("{:x}", Sha256::digest(&content)),
            "资源 sha256 与文件实际值相等"
        );
        // producer 身份字段（git/ui_gen 指纹/来源可追溯）
        assert!(manifest["producer"]["ui_gen_fingerprint"].is_string());
        assert!(manifest["producer"]["source_root"].is_string());
        // 闭包规模如实披露（受管包含语言源码——非纯 core）
        assert_eq!(manifest["disclosure"]["includes_language_source"], true);
        assert!(
            manifest["disclosure"]["total_bytes"].as_u64().unwrap() > 1_000_000,
            "auto-lang 全库规模明示"
        );
        assert!(manifest["pins"]["wgpu-hal"].is_string(), "pin 表在案");

        // 再交付：标记在案 → 唯一名 staging 重生成+整体换装成功（旧包换出）
        generate_rust_ui_delivery(&project, &delivery).expect("再交付失败");
        assert!(
            !delivery_sibling_path(&delivery, "delivery-old").exists(),
            "换出残桩已清理"
        );
        let staging_leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .contains(".delivery-staging-")
            })
            .collect();
        assert!(
            staging_leftovers.is_empty(),
            "成功/失败路径均不残留 staging"
        );
        assert!(
            !delivery.join(DELIVERY_STAGING_MARKER).exists(),
            "staging 属主标记不随交付包发行"
        );

        // 外来目录显式拒绝（非空且无标记）
        let foreign = dir.path().join("foreign");
        std::fs::create_dir_all(&foreign).unwrap();
        std::fs::write(foreign.join("mine.txt"), b"keep").unwrap();
        assert!(generate_rust_ui_delivery(&project, &foreign).is_err());
        assert!(foreign.join("mine.txt").is_file(), "外来内容不被覆盖");
    }

    /// R2-F01/R3-F01 永久用例：外来同级目录字节保留——①固定名 staging
    /// 位（.delivery.delivery-staging，历史保留名）上的外来内容不被触碰；
    /// ②**另一应用的合法交付包**（带自己的 delivery-manifest.json，R3 反例
    /// 复放）放在历史固定换出名（.delivery.delivery-old）上：再交付成功
    /// 且该应用数据完整存活（本轮起 backup 用唯一名+事务标记，固定名
    /// 从此不被认领）；③陈残 GC：带换出标记+清单的模式名残桩被回收、
    /// 仅带清单（无事务标记）的模式名外来目录不被删除；④正常换装恢复。
    #[test]
    fn delivery_foreign_sibling_dirs_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let producer = test_producer_root();
        let project = dir.path().join("proj");
        let front = project.join("src").join("front");
        std::fs::create_dir_all(&front).unwrap();
        std::fs::write(
            project.join("pac.at"),
            "{ name: \"uiminimal\", scene: \"ui\", render: \"rust\", window: \"fit\" }",
        )
        .unwrap();
        std::fs::write(
            front.join("app.at"),
            "widget App {\n    view {\n        col {\n            text `hi`\n        }\n    }\n}\n",
        )
        .unwrap();

        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(v) => std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", v),
                    None => std::env::remove_var("AUTO_DELIVERY_SOURCE_ROOT"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("AUTO_DELIVERY_SOURCE_ROOT"));
        std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", &producer);

        let delivery = dir.path().join("delivery");
        // ① 固定名 staging 位上的外来内容（R2 反例 1 复放）
        let legacy_stage = delivery_sibling_path(&delivery, "delivery-staging");
        std::fs::create_dir_all(&legacy_stage).unwrap();
        std::fs::write(legacy_stage.join("mine.txt"), b"unrelated sibling data").unwrap();
        generate_rust_ui_delivery(&project, &delivery).expect("外来固定名 staging 不应阻断首交");
        assert_eq!(
            std::fs::read(legacy_stage.join("mine.txt")).unwrap(),
            b"unrelated sibling data",
            "外来 staging 位内容字节保留"
        );

        // ② 另一应用的合法交付包占据历史固定换出名（R3 反例复放：有效
        // manifest + 该应用用户数据——"marker 存在 ≠ 本次操作归属"）
        let other_app = dir.path().join("other-app");
        let other_proj = dir.path().join("other-proj");
        let other_front = other_proj.join("src").join("front");
        std::fs::create_dir_all(&other_front).unwrap();
        std::fs::write(
            other_proj.join("pac.at"),
            "{ name: \"unrelated_app\", scene: \"ui\", render: \"rust\", window: \"fit\" }",
        )
        .unwrap();
        std::fs::write(
            other_front.join("app.at"),
            "widget App {\n    view {\n        col {\n            text `hi`\n        }\n    }\n}\n",
        )
        .unwrap();
        generate_rust_ui_delivery(&other_proj, &other_app).expect("他应用交付生成失败");
        let legacy_trash = delivery_sibling_path(&delivery, "delivery-old");
        std::fs::rename(&other_app, &legacy_trash).unwrap();
        std::fs::create_dir_all(legacy_trash.join("app")).unwrap();
        std::fs::write(
            legacy_trash.join("app").join("unrelated-user-data.txt"),
            b"another application's valid delivery",
        )
        .unwrap();
        let other_manifest_before =
            std::fs::read(legacy_trash.join("delivery-manifest.json")).unwrap();
        generate_rust_ui_delivery(&project, &delivery).expect("固定名他应用交付不应阻断再交付");
        assert!(
            legacy_trash.join("delivery-manifest.json").is_file(),
            "历史固定名上的他应用交付完整存活"
        );
        assert!(
            legacy_trash
                .join("app")
                .join("unrelated-user-data.txt")
                .is_file(),
            "他应用用户数据存活（不凭清单文件名认领）"
        );
        // 他应用交付字节原样（身份未被冒认修改）
        assert_eq!(
            std::fs::read(legacy_trash.join("delivery-manifest.json")).unwrap(),
            other_manifest_before,
            "固定名上的他应用交付字节不变"
        );

        // ③ 陈残 GC：带换出标记+清单的模式名残桩被回收；仅清单无标记的
        // 模式名外来目录不被删除
        let stale = dir.path().join(".delivery.delivery-old-999999-1");
        std::fs::create_dir_all(stale.join("app")).unwrap();
        std::fs::write(
            stale.join("delivery-manifest.json"),
            "{\"schema\":\"generated-ui-delivery/1\"}",
        )
        .unwrap();
        std::fs::write(stale.join(DELIVERY_SWAP_MARKER), "pid=999999\n").unwrap();
        let foreign_patterned = dir.path().join(".delivery.delivery-old-888888-2");
        std::fs::create_dir_all(foreign_patterned.join("app")).unwrap();
        std::fs::write(
            foreign_patterned.join("delivery-manifest.json"),
            "{\"schema\":\"generated-ui-delivery/1\"}",
        )
        .unwrap();
        std::fs::write(
            foreign_patterned.join("app").join("mine.txt"),
            b"no transaction marker",
        )
        .unwrap();
        generate_rust_ui_delivery(&project, &delivery).expect("GC 轮次失败");
        assert!(!stale.exists(), "带事务标记的陈残被 GC 回收");
        assert!(
            foreign_patterned.join("app").join("mine.txt").is_file(),
            "无事务标记的模式名外来目录不被删除"
        );

        // ④ 正常换装恢复（本轮唯一名 backup 均已随成功清理；固定名上的
        // 他应用交付与模式名外来目录持续存活，不属本生成器清理范围）
        generate_rust_ui_delivery(&project, &delivery).expect("正常再交付失败");
        let backups: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains(".delivery-old-"))
            .collect();
        assert!(
            backups
                .iter()
                .all(|e| e.path().join("app").join("mine.txt").is_file()),
            "模式名位仅存外来数据（无事务标记不被清理），本轮 backup 已清"
        );
    }

    /// R4-F01 永久用例：根缺失/唯一备份 + 生成失败保旧——旧根换出并写
    /// 换出标记后、staging 落位前中断留下的待恢复态（交付根缺失，唯一旧
    /// 包在 backup），重试生成失败时绝不可删该备份；成功重试后才可回收
    /// （此刻已被新包替代）。换出标记 ≠ 已提交可回收。
    #[test]
    fn delivery_gc_preserves_pending_recovery_backup() {
        let dir = tempfile::tempdir().unwrap();
        let producer = test_producer_root();
        let project = dir.path().join("proj");
        let front = project.join("src").join("front");
        std::fs::create_dir_all(&front).unwrap();
        let write_pac = |modules: &str| {
            std::fs::write(
                project.join("pac.at"),
                format!(
                    "{{ name: \"uiminimal\", scene: \"ui\", render: \"rust\", window: \"fit\" }}\n{modules}"
                ),
            )
            .unwrap();
        };
        write_pac("");
        std::fs::write(
            front.join("app.at"),
            "widget App {\n    view {\n        col {\n            text `hi`\n        }\n    }\n}\n",
        )
        .unwrap();

        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(v) => std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", v),
                    None => std::env::remove_var("AUTO_DELIVERY_SOURCE_ROOT"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("AUTO_DELIVERY_SOURCE_ROOT"));
        std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", &producer);

        let delivery = dir.path().join("recovery-target");
        generate_rust_ui_delivery(&project, &delivery).expect("首交失败");
        let last_good = delivery.join("app").join("last-good-user-data.txt");
        std::fs::write(&last_good, b"last good artifact must survive failed retry").unwrap();
        let manifest_before = std::fs::read(delivery.join("delivery-manifest.json")).unwrap();

        // 模拟 delivery_swap_in 中断态：旧根已换出+事务标记已写，新包未落位。
        let backup = dir.path().join(".recovery-target.delivery-old-999999-3");
        std::fs::rename(&delivery, &backup).unwrap();
        std::fs::write(backup.join(DELIVERY_SWAP_MARKER), "pid=999999\n").unwrap();
        assert!(!delivery.exists(), "待恢复态：交付根缺失");

        // 失败重试（sidecar 模块缺失——apply 阶段在 GC 之后判红）。
        write_pac("\nrust_sidecar {\n    modules: [\"glue:missing.rs\"]\n}\n");
        let err = generate_rust_ui_delivery(&project, &delivery)
            .expect_err("缺 sidecar 模块的重试必须显式 Err");
        assert!(
            err.to_string().contains("module 'glue'") || err.to_string().contains("not found"),
            "失败诊断指向缺失模块（实际：{err}）"
        );
        assert!(!delivery.exists(), "失败后不落新包");
        assert!(backup.is_dir(), "待恢复唯一旧包存活");
        assert!(
            backup.join("app").join("last-good-user-data.txt").is_file(),
            "last-good 载荷存活"
        );
        assert_eq!(
            std::fs::read(backup.join("delivery-manifest.json")).unwrap(),
            manifest_before,
            "备份清单字节不变"
        );
        assert!(
            backup.join(DELIVERY_SWAP_MARKER).is_file(),
            "换出标记保留（事务未完成）"
        );

        // 成功重试：新包落位后才允许回收已被替代的旧备份。
        write_pac("");
        generate_rust_ui_delivery(&project, &delivery).expect("成功重试失败");
        assert!(
            delivery.join("delivery-manifest.json").is_file(),
            "成功重试后新交付在位"
        );
        assert!(!backup.exists(), "成功发布后旧备份才可回收");
        assert!(!delivery.join(DELIVERY_STAGING_MARKER).exists());
    }

    /// R1-F02 永久用例：再交付准备失败保留旧交付——成功交付后，producer
    /// 破损（闭包 crate 缺 manifest）的再交付显式 Err，旧包字节/标记/
    /// sentinel 完整；producer 恢复后再交付成功（换装）。
    #[test]
    fn delivery_redelivery_failure_preserves_old() {
        let dir = tempfile::tempdir().unwrap();
        let producer = test_producer_root();
        let project = dir.path().join("proj");
        let front = project.join("src").join("front");
        std::fs::create_dir_all(&front).unwrap();
        std::fs::write(
            project.join("pac.at"),
            "{ name: \"uiminimal\", scene: \"ui\", render: \"rust\", window: \"fit\" }",
        )
        .unwrap();
        std::fs::write(
            front.join("app.at"),
            "widget App {\n    model {\n        var count int = 0\n    }\n    view {\n        col {\n            text `Count: ${.count}`\n            button \"+1\" { onclick: () => {.count += 1} }\n        }\n    }\n}\n",
        )
        .unwrap();

        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(v) => std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", v),
                    None => std::env::remove_var("AUTO_DELIVERY_SOURCE_ROOT"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("AUTO_DELIVERY_SOURCE_ROOT"));
        std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", &producer);

        let delivery = dir.path().join("delivery");
        generate_rust_ui_delivery(&project, &delivery).expect("首交失败");
        let sentinel = delivery.join("app").join("previous-build.txt");
        std::fs::write(&sentinel, b"old delivery must survive failed preparation").unwrap();
        let manifest_before =
            std::fs::read_to_string(delivery.join("delivery-manifest.json")).unwrap();

        // 破损 producer：crates/auto-lang 目录在但缺 manifest（复放 R1 反例）
        let broken = dir.path().join("broken-producer");
        std::fs::create_dir_all(broken.join("crates").join("auto-lang")).unwrap();
        std::fs::write(broken.join("Cargo.toml"), "[workspace]\n").unwrap();
        std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", &broken);
        let err = generate_rust_ui_delivery(&project, &delivery)
            .expect_err("破损 producer 的再交付必须显式 Err");
        assert!(
            err.to_string().contains("预检失败"),
            "预检判红（实际：{err}）"
        );
        assert!(sentinel.is_file(), "旧交付 sentinel 存活");
        assert_eq!(
            std::fs::read_to_string(delivery.join("delivery-manifest.json")).unwrap(),
            manifest_before,
            "旧交付清单字节不变"
        );
        assert!(
            !delivery_sibling_path(&delivery, "delivery-staging").exists()
                || std::fs::read_dir(delivery_sibling_path(&delivery, "delivery-staging"))
                    .map(|mut it| it.next().is_none())
                    .unwrap_or(true),
            "失败后无残留 staging（或仅空壳）"
        );

        // producer 恢复 → 再交付成功（新包换入，旧 sentinel 随旧包换出）
        std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", &producer);
        generate_rust_ui_delivery(&project, &delivery).expect("恢复后再交付失败");
        assert!(delivery.join("delivery-manifest.json").is_file());
    }

    /// R1-F03 永久用例：sidecar 外部 path 显式拒绝——绝对路径（含与
    /// producer 不同根）与越出交付根的相对 `..` 均判红且不产出；registry
    /// 版本依赖照常入清单；包内相对 path（../crates/…）合法。
    #[test]
    fn delivery_sidecar_external_path_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let producer = test_producer_root();
        let project = dir.path().join("proj");
        let front = project.join("src").join("front");
        std::fs::create_dir_all(&front).unwrap();
        let base_pac = "{ name: \"uiminimal\", scene: \"ui\", render: \"rust\", window: \"fit\" }";

        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(v) => std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", v),
                    None => std::env::remove_var("AUTO_DELIVERY_SOURCE_ROOT"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("AUTO_DELIVERY_SOURCE_ROOT"));
        std::env::set_var("AUTO_DELIVERY_SOURCE_ROOT", &producer);

        let mk_project = |sidecar_block: &str| {
            std::fs::create_dir_all(&front).unwrap();
            std::fs::write(
                project.join("pac.at"),
                format!("{base_pac}\n{sidecar_block}"),
            )
            .unwrap();
            std::fs::write(
                front.join("app.at"),
                "widget App {\n    view {\n        col {\n            text `hi`\n        }\n    }\n}\n",
            )
            .unwrap();
        };

        // 绝对路径（与 producer 不同根——复放 R1 反例形态）
        mk_project(
            "\nrust_sidecar {\n    deps: [\"auto-val:path:D:/some-other-root/crates/auto-val\"]\n}\n",
        );
        let delivery = dir.path().join("d-abs");
        let err = generate_rust_ui_delivery(&project, &delivery)
            .expect_err("外部绝对 path sidecar 必须拒绝");
        assert!(
            err.to_string().contains("外部绝对/根相对"),
            "绝对路径诊断（实际：{err}）"
        );
        assert!(!delivery.join("app/Cargo.toml").is_file(), "拒绝时不产出");

        // Windows 根相对路径（R2 反例复放：is_absolute 为假但 path::join
        // 语义丢弃基目录、解析落当前盘根——实际指向主检出）
        mk_project(
            "\nrust_sidecar {\n    deps: [\"auto-val:path:/autostack/auto-lang/crates/auto-val\"]\n}\n",
        );
        let delivery = dir.path().join("d-rootrel");
        let err = generate_rust_ui_delivery(&project, &delivery)
            .expect_err("Windows 根相对 path sidecar 必须拒绝");
        assert!(
            err.to_string().contains("根相对"),
            "根相对诊断（实际：{err}）"
        );
        assert!(!delivery.join("app/Cargo.toml").is_file(), "拒绝时不产出");

        // R3 反例复放：raw TOML 单引号内联表——sidecar 原样写入
        // `name = {path='...'}`（合法 TOML，旧文本匹配绕过；TOML 语义校验同拒）
        let ext = dir.path().join("external-crate");
        std::fs::create_dir_all(&ext).unwrap();
        std::fs::write(
            ext.join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n",
        )
        .unwrap();
        let spec = format!(
            "raw-ext:{{path='{}'}}",
            ext.to_string_lossy().replace('\\', "/")
        );
        mk_project(&format!(
            "\nrust_sidecar {{\n    deps: [\"{}\"]\n}}\n",
            spec
        ));
        let delivery = dir.path().join("d-rawtoml");
        let err = generate_rust_ui_delivery(&project, &delivery)
            .expect_err("单引号内联表 path sidecar 必须拒绝");
        assert!(
            err.to_string().contains("外部绝对/根相对"),
            "TOML 语义诊断（实际：{err}）"
        );
        assert!(!delivery.join("app/Cargo.toml").is_file(), "拒绝时不产出");

        // 相对 `..` 越出交付根
        mk_project("\nrust_sidecar {\n    deps: [\"ext:path:../../../outside\"]\n}\n");
        let delivery = dir.path().join("d-esc");
        let err = generate_rust_ui_delivery(&project, &delivery)
            .expect_err("越出交付根的相对 path sidecar 必须拒绝");
        assert!(
            err.to_string().contains("越出交付根"),
            "越界诊断（实际：{err}）"
        );

        // 合法控制：registry 版本依赖照常；包内相对 path 合法
        mk_project(
            "\nrust_sidecar {\n    deps: [\"libloading:0.8\", \"auto-val:path:../crates/auto-val\"]\n}\n",
        );
        let delivery = dir.path().join("d-ok");
        generate_rust_ui_delivery(&project, &delivery).expect("合法 sidecar 交付失败");
        let app_manifest = std::fs::read_to_string(delivery.join("app/Cargo.toml")).unwrap();
        assert!(
            app_manifest.contains("libloading = \"0.8\""),
            "registry 版本依赖入清单"
        );
        assert!(
            app_manifest.contains("auto-val = { path = \"../crates/auto-val\" }"),
            "包内相对 path 合法"
        );
        assert!(!app_manifest.contains("D:/"), "无绝对路径残留");
    }

    /// PLAN-792：同包 source 重复显式拒绝——bundle 内两份同名 crate 即身份
    /// 冲突红（构造：把 auto-val 目录复制为第二个成员位）。
    #[test]
    fn delivery_duplicate_crate_identity_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let crates = dir.path().join("crates");
        for name in ["auto-lang", "auto-val", "dupe-as-val"] {
            let c = crates.join(name);
            std::fs::create_dir_all(&c).unwrap();
            let pkg = if name == "auto-lang" {
                "auto-lang"
            } else {
                "auto-val"
            };
            std::fs::write(
                c.join("Cargo.toml"),
                format!("[package]\nname = \"{pkg}\"\nversion = \"0.1.0\"\n"),
            )
            .unwrap();
        }
        let err = delivery_assert_unique_crates(dir.path(), "app");
        assert!(err.is_err(), "同包 source 重复必须显式拒绝");
        assert!(
            delivery_assert_unique_crates(dir.path(), "auto-lang").is_err(),
            "应用撞名拒绝"
        );
    }
    #[test]
    fn generated_ui_cargo_uses_canonical_double_dependency() {
        // PLAN-814 T-05（union 迁移自 auto-man api_gen 测试——rust_ui 归属
        // auto-ui-tools 后断言随迁）：UI 工程 manifest canonical 双直依赖。
        // PLAN-815 T-07 子批2 补强：SDK 包（auto-lang-core）与交付 app 清单
        // 同钉——792 driver V01 曾以缺两行红（fresh/locked 真实构建 E0433）。
        let ui_cargo = crate::rust_ui::generate_cargo_toml(
            "image-ui",
            std::path::Path::new("."),
            &crate::rust_ui::PacSource::legacy(),
        );
        assert!(ui_cargo.contains("auto-ui-runtime"));
        assert!(
            ui_cargo.contains("auto-ui-runtime.workspace = true"),
            "canonical 双直依赖须 workspace 继承形态: {ui_cargo}"
        );
        assert!(
            ui_cargo.contains("auto-lang-core.workspace = true"),
            "SDK 直依赖（auto_lang_core:: import）: {ui_cargo}"
        );
        let delivery_cargo = generate_delivery_app_manifest(
            "image-ui",
            std::path::Path::new("."),
            &[],
            &PacSource::legacy(),
        );
        assert!(
            delivery_cargo.contains("auto-ui-runtime = { path = \"../crates/auto-ui-runtime\" }"),
            "交付 app 直依赖 canonical runtime: {delivery_cargo}"
        );
        assert!(
            delivery_cargo.contains("auto-lang-core = { path = \"../crates/auto-lang-core\" }"),
            "交付 app 直依赖 canonical SDK: {delivery_cargo}"
        );
    }
}
