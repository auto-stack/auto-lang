// PLAN-814 T-04：UI 动态构建管线（原 auto-lang lib.rs Plan 015 UI glue 区整体随迁）——
// collect_module_imports 模块装载链 → build_dynamic_component → run_file_dynamic_ui；
// crate 根经 lib.rs 再导出保持旧 crate::* 路径（ui/ 内 98 处 build_dynamic_component 引用零改写）。
use crate::{AutoResult, Parser, UseModuleResolution};
use auto_lang_core::resolve_use_module;
use auto_lang_core::{resolve_module_path, store_name_snake};
/// Used by CLI to decide whether to run on main thread for iced.
#[cfg(feature = "ui-iced")]
pub fn has_ui_keywords(code: &str) -> bool {
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("widget ") || trimmed.starts_with("widget{") {
            return true;
        }
        if trimmed.starts_with("app ") || trimmed.starts_with("app{") {
            return true;
        }
    }
    false
}

/// Return the declared name of an importable top-level statement
/// (`fn`/`type`/`enum`/`ext`), used to filter `use module: items` imports.
// Plan 330: un-gated — these helpers (stmt_symbol_name, resolve_module_path,
// collect_module_imports) are pure path/parsing utilities with no iced dep, so
// they compile in any feature config and tests can call them without ui-iced.
pub(crate) fn stmt_symbol_name(stmt: &auto_lang_core::ast::Stmt) -> Option<String> {
    use auto_lang_core::ast::Stmt;
    match stmt {
        Stmt::Fn(f) => Some(f.name.to_string()),
        Stmt::TypeDecl(t) => Some(t.name.to_string()),
        Stmt::EnumDecl(e) => Some(e.name.to_string()),
        Stmt::Ext(e) => Some(e.target.to_string()),
        _ => None,
    }
}

/// Resolve a (possibly dotted) module path to its source file under `base_dir`.
///
/// A dotted module path maps to a filesystem path segment by segment:
/// `back.api` → `back/api`, `calendar_util` → `calendar_util`. Per the module
/// rules a module lives at either `{path}.at` or `{path}/mod.at`; the first
/// existing candidate wins. Returns `None` if neither exists.
/// Plan 061:外部后端根(pac.at `back: { project }`)。设置后,`back.*`
/// 模块解析优先映射到该目录(`back.api` → `<root>/api.at`),前端项目
/// 无需本地 back/ 目录 —— 契约与桩由后端项目拥有,链接式引用(零复制)。
/// 进程级全局(`auto run` 单用途进程,生命周期即进程;unset 无需)。
/// pathological inputs). Single-element result for paths without underscore
/// segments — callers loop and behave exactly as before in that case.
#[allow(clippy::too_many_arguments)]
#[cfg(feature = "ui")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn load_ext_imports_for_vm(
    base_dir: &std::path::Path,
    ast: &auto_lang_core::ast::Code,
    root_decl: &auto_lang_core::ast::ui::WidgetDecl,
    all_child_decls: &[auto_lang_core::ast::ui::WidgetDecl],
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
    import_stmts: &mut Vec<auto_lang_core::ast::Stmt>,
    seen_symbols: &mut std::collections::HashSet<String>,
    import_session: &mut auto_lang_core::compile::CompileSession,
    override_scenario: Option<&auto_lang_core::session::CompilerSession>,
    import_aliases: &mut std::collections::HashMap<String, String>,
    // PLAN-051 C4: adapter 链解析到的 .at 目标内、与导入符号同名的 widget
    // 声明（ports renderer.vm.at 的 Markdown 纯文本降级等）——调用方注册进
    // 视图 registry + child_decls，激活 use.web component 的 VM widget 形态。
    ext_widget_decls: &mut Vec<auto_lang_core::ast::ui::WidgetDecl>,
    // PLAN-633: 解析到的适配器文件路径——调用方补传递子 widget 注册
    // （Demo013Todo 视图内的 TodoList 等 registry miss → Empty 实证）。
    ext_adapter_paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), String> {
    let mut ext_imports = crate::ui::ext_stubs::collect_useweb_imports(&ast.stmts);
    ext_imports.extend(crate::ui::ext_stubs::collect_widget_ext_imports(
        std::slice::from_ref(root_decl),
    ));
    ext_imports.extend(crate::ui::ext_stubs::collect_widget_ext_imports(
        all_child_decls,
    ));
    // Plan 442 B-support: child modules' own top-level `use.web` (e.g.
    // specs_view.at's `use.web spec_next_id from ".../specs_helpers.at"`) —
    // the root-AST walk above misses them; sweep every loaded module file.
    for path in visited.iter() {
        if let Ok(code) = std::fs::read_to_string(path) {
            // PLAN-037: 模块重读统一过前缀化 overlay（visited 集扫）。同下。
            let code = auto_lang_core::back_prefix::apply(path, code);
            // PLAN-668 A-02：逐模块 stylekit 预注册（同主收集路径；前缀化不触
            // stylekit 导入面，变换后扫等价）。
            let _ = auto_lang_core::design_tokens::recipe::prepare_style_recipe_imports(
                path.parent().unwrap_or(std::path::Path::new(".")),
                &code,
            );
            let session = auto_lang_core::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            if let Ok(mod_ast) = parser.parse() {
                ext_imports.extend(crate::ui::ext_stubs::collect_useweb_imports(&mod_ast.stmts));
            }
        }
    }
    if ext_imports.is_empty() {
        return Ok(());
    }
    let (loaded_adapters, nested) =
        crate::ui::ext_stubs::load_at_ext_imports(base_dir, &ext_imports, &mut |adapter_path| {
            collect_module_imports(
                adapter_path,
                visited,
                import_stmts,
                seen_symbols,
                import_session,
                override_scenario,
            );
        });
    // PLAN-633: 适配器路径上交调用方（传递子 widget 注册用）。
    for (adapter, _) in &loaded_adapters {
        ext_adapter_paths.push(adapter.clone());
    }
    // Alias each adapter-loaded symbol to ITS adapter's module-qualified
    // name (collect_module_imports renames adapter fns by file stem) so call
    // sites emit resolvable qualified relocs. Pairs come from the loader —
    // aliasing by file, not by "any .at import" (which would mis-attribute
    // symbols to the wrong adapter).
    for (adapter, symbols) in &loaded_adapters {
        let qualifier = adapter
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("ext");
        for sym in symbols {
            import_aliases
                .entry(sym.clone())
                .or_insert_with(|| format!("{}.{}", qualifier, sym));
        }
    }
    ext_imports.extend(nested);

    // PLAN-051 C4 widget 臂：逐 adapter 解析 widget 声明（同名符号命中）+
    // ExplicitFn 导出校验（typo 编译期报错，替代静默落 stub——裸形式不校验，
    // 沿"可能是函数/对象/常量"的含糊语义）。
    let mut adapter_fn_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (adapter, symbols) in &loaded_adapters {
        let Ok(code) = std::fs::read_to_string(adapter) else {
            continue;
        };
        let code = auto_lang_core::back_prefix::apply(adapter, code);
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let Ok(adapter_ast) = parser.parse() else {
            continue;
        };
        for stmt in &adapter_ast.stmts {
            match stmt {
                auto_lang_core::ast::Stmt::WidgetDecl(wd) => {
                    if symbols.iter().any(|s| s == wd.name.as_str()) {
                        ext_widget_decls.push(wd.clone());
                    }
                }
                auto_lang_core::ast::Stmt::Fn(f) => {
                    // 记录裸名供校验（collect_module_imports 重命名前形态）。
                    adapter_fn_names.insert(f.name.to_string());
                }
                _ => {}
            }
        }
    }
    for imp in &ext_imports {
        if imp.kind == auto_lang_core::ast::ExtImportKind::ExplicitFn && imp.path.ends_with(".at") {
            for sym in &imp.symbols {
                if !adapter_fn_names.contains(sym) {
                    return Err(format!(
                        "use.web.fn '{}': .at 目标 {} 未导出名为 '{}' 的顶层 fn（显式 fn 声明校验，PLAN-051 C4）",
                        sym, imp.path, sym
                    ));
                }
            }
        }
    }

    // PLAN-632 F3: 已装载模块自身 use 链上的 widget 注册。根 use 环的
    // register_transitive_widgets 只覆盖根文件;use.web demo 适配器
    // （AppViewport.vm.at → Demo*.at）自带的 `use <mod>: Component`
    // （006 的 SettingsPopover）此前从不进 registry/child_decls——
    // 组件实例在 VM 渲染目标渲染 Empty/缺席。扫 visited 全集（与上方
    // use.web 收集同集、同重解析模式），按各文件 use items 显式名单
    // 注册 widget 声明进 ext_widget_decls（调用方统一入
    // registry+child_decls，handler 随之编译进单 VM）。P545 语义：
    // 仅具名 items / 显式通配可见，bare use 不触发。
    {
        let mut swept: std::collections::HashSet<String> = ext_widget_decls
            .iter()
            .map(|wd| wd.name.to_string())
            .collect();
        for wd in all_child_decls {
            swept.insert(wd.name.to_string());
        }
        swept.insert(root_decl.name.to_string());
        for path in visited.iter() {
            if path.extension().and_then(|e| e.to_str()) != Some("at") {
                continue;
            }
            let Ok(code) = std::fs::read_to_string(path) else {
                continue;
            };
            let code = auto_lang_core::back_prefix::apply(path, code);
            let session = auto_lang_core::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            let Ok(mod_ast) = parser.parse() else {
                continue;
            };
            let mod_dir = path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            for use_stmt in auto_lang_core::use_scanner::scan_use_statements(&code) {
                if use_stmt.is_c_import || use_stmt.is_rust_import {
                    continue;
                }
                if !(use_stmt.is_wildcard || !use_stmt.items.is_empty()) {
                    continue;
                }
                // F2 增强后的解析（dep item 文件命中同路）。
                let module_path = match resolve_use_module(&mod_dir, &use_stmt) {
                    UseModuleResolution::Module(p) => p,
                    _ => continue,
                };
                // PLAN-632 F4: 已装载模块 use 链的符号别名——裸名 → 模块限定名
                // （与根 use 环 lib.rs 别名位同规则）。此前别名只对根文件 use
                // 填充：demo 作为 ext 适配器装载时其 `use calendar_util:
                // month_name` 无别名 → 视图 computed 体 `month_name(...)`
                // call_vm_fn 裸名查 exports miss（fn 实为
                // `calendar_util.month_name`）→ 求值失败回退 raw `${}`。
                // or_insert：根环别名（先填）优先。
                {
                    let qualifier = use_stmt
                        .module
                        .split('.')
                        .last()
                        .unwrap_or(&use_stmt.module);
                    for item in &use_stmt.items {
                        import_aliases
                            .entry(item.clone())
                            .or_insert_with(|| format!("{}.{}", qualifier, item));
                    }
                }
                let Ok(module_code) = std::fs::read_to_string(&module_path) else {
                    continue;
                };
                let module_code = auto_lang_core::back_prefix::apply(&module_path, module_code);
                let mod_session = auto_lang_core::session::CompilerSession::ui();
                let mut mod_parser =
                    crate::Parser::from(module_code.as_str()).with_session(mod_session);
                let Ok(target_ast) = mod_parser.parse() else {
                    continue;
                };
                for stmt in &target_ast.stmts {
                    if let auto_lang_core::ast::Stmt::WidgetDecl(decl) = stmt {
                        if swept.contains(decl.name.as_str()) {
                            continue;
                        }
                        if use_stmt.is_wildcard
                            || use_stmt.items.iter().any(|s| s == &decl.name.as_str())
                        {
                            swept.insert(decl.name.to_string());
                            ext_widget_decls.push(decl.clone());
                        }
                    }
                }
            }
        }
    }

    // PLAN-643: 已装载模块自身 widget 级 `use { package: x from "dir" }` 的
    // 装载。demo 适配器（AppViewport.vm.at → demos/<id>.at）内的包导入此前
    // 无任何装载分支——模块链只处理 `use <mod>`/`use.web`，package 仅根组件
    // 动态分支（build_dynamic_component_inner）处理——包组件（024-charts 的
    // LineChart/BarChart/AreaChart/DonutChart）不进 registry/child_decls，
    // 实例 tag 落 schema builtin 空壳桩 → 画布空（P642-D1 根因）。扫 visited
    // 全集（与上方 use.web/632-F3 收集同集），镜像动态分支同语义：
    // load_package → decls 入 ext_widget_decls（调用方统一注册
    // registry+child_decls，handler 随之编译进单 VM）+ 包文件 use 依赖
    // collect_module_imports 入编译单元 + 裸名别名。schema package_origin
    // 分类（PLAN-643 双态归属）保证同名 tag 不再被 builtin 臂截胡。
    {
        let mut swept_pkgs: std::collections::HashSet<String> = ext_widget_decls
            .iter()
            .map(|wd| wd.name.to_string())
            .collect();
        for wd in all_child_decls {
            swept_pkgs.insert(wd.name.to_string());
        }
        swept_pkgs.insert(root_decl.name.to_string());
        let mut seen_pkg_dirs: std::collections::HashSet<std::path::PathBuf> = Default::default();
        // 快照迭代:collect_module_imports 会向 visited 追加包文件自身 use 链,
        // 不能在 visited.iter() 借用下调用(新装载模块的包导入由后续调用方的
        // 下一轮装载自然覆盖——与根环一次性 sweep 语义一致)。
        let visited_snap: Vec<std::path::PathBuf> = visited.iter().cloned().collect();
        for path in visited_snap.iter() {
            if path.extension().and_then(|e| e.to_str()) != Some("at") {
                continue;
            }
            let Ok(code) = std::fs::read_to_string(path) else {
                continue;
            };
            let code = auto_lang_core::back_prefix::apply(path, code);
            let session = auto_lang_core::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            let Ok(mod_ast) = parser.parse() else {
                continue;
            };
            let mod_dir = path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            for wd in mod_ast.stmts.iter().filter_map(|s| match s {
                auto_lang_core::ast::Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            }) {
                for imp in &wd.ext_imports {
                    if !matches!(imp.kind, auto_lang_core::ast::ui::ExtImportKind::Package) {
                        continue;
                    }
                    let dir = std::path::PathBuf::from(imp.path.as_str());
                    if !seen_pkg_dirs.insert(dir.clone()) {
                        continue;
                    }
                    let candidates = [mod_dir.join(&dir), mod_dir.join("pages").join(&dir)];
                    let mut pkg_reg = auto_ui_compiler::ui_gen::widget::ComponentRegistry::new();
                    let first = pkg_reg.load_package(&candidates[0], &mod_dir);
                    let (loaded, loaded_dir, load_err) = match first {
                        Ok(p) => (Ok(p), Some(candidates[0].clone()), String::new()),
                        Err(_) => match pkg_reg.load_package(&candidates[1], &mod_dir) {
                            Ok(p) => (Ok(p), Some(candidates[1].clone()), String::new()),
                            Err(e) => (Err(()), None, e),
                        },
                    };
                    if loaded.is_err() {
                        log::warn!(
                            "package `{}` load failed (VM module chain): {} (tried {:?})",
                            imp.path,
                            load_err,
                            candidates
                        );
                        continue;
                    }
                    let pkg = loaded.unwrap();
                    for (d, _aw) in &pkg.full_widgets {
                        if swept_pkgs.insert(d.name.to_string()) {
                            ext_widget_decls.push(d.clone());
                        }
                    }
                    // 包组件文件自身的 use 依赖入编译单元 + 裸名别名
                    //（与动态分支 Plan 522 同规则）。
                    if let Some(pkg_dir) = loaded_dir {
                        if let Ok(entries) = std::fs::read_dir(&pkg_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.extension().map(|e| e != "at").unwrap_or(true) {
                                    continue;
                                }
                                crate::collect_module_imports(
                                    &p,
                                    visited,
                                    import_stmts,
                                    seen_symbols,
                                    import_session,
                                    None,
                                );
                                if let Ok(pc) = std::fs::read_to_string(&p) {
                                    for us in auto_lang_core::use_scanner::scan_use_statements(&pc)
                                    {
                                        let qualifier =
                                            us.module.split('.').last().unwrap_or(&us.module);
                                        for item in &us.items {
                                            import_aliases.entry(item.clone()).or_insert_with(
                                                || format!("{}.{}", qualifier, item),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !crate::ui::ext_stubs::ext_stubs_enabled() {
        return Ok(());
    }
    // Symbols already defined (adapter fns, root/imported fns — qualified or
    // plain) must not be stubbed.
    let mut defined: std::collections::HashSet<String> = std::collections::HashSet::new();
    for stmt in import_stmts.iter() {
        if let auto_lang_core::ast::Stmt::Fn(f) = stmt {
            defined.insert(f.name.to_string());
            if let Some(pos) = f.name.as_str().rfind('.') {
                defined.insert(f.name.as_str()[pos + 1..].to_string());
            }
        }
    }
    let mut targets = std::collections::HashSet::new();
    for imp in &ext_imports {
        for sym in &imp.symbols {
            if !defined.contains(sym) {
                targets.insert(sym.clone());
            }
        }
    }
    if targets.is_empty() {
        return Ok(());
    }
    let mut arities = std::collections::HashMap::new();
    crate::ui::ext_stubs::scan_call_arities(&ast.stmts, &targets, &mut arities);
    crate::ui::ext_stubs::scan_call_arities(import_stmts, &targets, &mut arities);
    crate::ui::ext_stubs::scan_decl_arities(all_child_decls, &targets, &mut arities);
    let mut names: Vec<&String> = targets.iter().collect();
    names.sort();
    for name in names {
        let arity = arities.get(name).copied().unwrap_or(0);
        log::warn!(
            "ext stub: `{}` has no VM-target implementation (web-ecosystem source) — no-op platform stub, arity {}",
            name,
            arity
        );
        import_stmts.push(crate::ui::ext_stubs::synthesize_stub_fn(name, arity));
    }
    Ok(())
}

/// PascalCase → snake_case (compact local variant of ui_gen::api's helper;
/// `AuthStore` → `auth_store`, `AgentConfigs` → `agent_configs`).
/// StoreDecl → view-less child WidgetDecl（store-as-child：store 与子件
/// 同构、仅缺视图——统一根态模型并入 + handler 编译进单 VM 都走 child
/// 通道）。PLAN-632 F1 抽出公共转换体，供 ext 装载后的补转换位使用。
fn store_decl_as_widget_decl(
    store_decl: &auto_lang_core::ast::ui::StoreDecl,
) -> auto_lang_core::ast::ui::WidgetDecl {
    auto_lang_core::ast::ui::WidgetDecl {
        name: store_decl.name.clone(),
        messages: store_decl.messages.clone(),
        model: store_decl.model.clone(),
        computed: store_decl.computed.clone(),
        view: None,
        named_views: Vec::new(),
        on: store_decl.on.clone(),
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
        timer: store_decl.timer.clone(),
    }
}

/// Plan 442 A2 (store facade): the legacy `use store: Name` form uses the
/// literal module name "store", which never maps to a `store.at` file — the
/// StoreDecl lives in a sibling file named after the store (musk convention:
/// `AuthStore` → `auth_store.at`, `AgentConfigs` → `agent_configs.at`).
/// Locate those files by naming convention first, then a bounded recursive
/// scan of the source dir (parsing each .at and matching StoreDecl names).
/// Returns store name → file path for every name that resolves.
/// Recursively collect ALL declarations (Fn/TypeDecl/EnumDecl/Ext/Store/Use)
/// from a module and its transitive `use` dependencies, for VM-render
/// compilation.
///
/// Why "all declarations" rather than just the `use`-named items: an imported
/// function's body may call sibling helpers in the same module that are NOT
/// listed in the `use` clause (e.g. `use calendar_util: build_month_grid` does
/// not name `weekday_of`/`days_in_month`/`format_date`, yet `build_month_grid`
/// calls them). A module is a compilation unit, so the whole module — plus the
/// modules it itself imports — must be fed to the single Codegen pass for
/// callees to resolve. Dedups by canonical path (cycle guard) and by symbol
/// name (first definition wins).
///
/// Plan 333: a shared `CompileSession` is threaded through the recursion so
/// that `resolve_uses` pre-registers imported types (e.g. `Note` from
/// `use api: Note` in back/db.at) into a `type_store` BEFORE each module is
/// parsed. Without this, db.at's `var notes List<Note> = List<Note>.new([...])`
/// fails to parse with "undefined variable: Note", dropping all of db.at's
/// functions and breaking `db.func()` linking. This mirrors the script path
/// (`execute_autovm_with_path`), which already does this.
/// Plan 446 批一 (C1-3): UI 模块 parse 失败的进程级登记表。收集侧
/// (`collect_module_imports`) 保持非致命(投机解析路径安全),`VmBridge` 构造时
/// 检查本表并升级为致命错误 —— 消除"WARN + 静默空渲染"(os-config C1 现场)。
pub fn ui_module_parse_failures() -> &'static std::sync::Mutex<Vec<String>> {
    static REG: std::sync::OnceLock<std::sync::Mutex<Vec<String>>> = std::sync::OnceLock::new();
    REG.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// Plan 446 批一 (C1-2): 把 AutoError 的 miette span 偏移对照源码换算为
/// `line:col` 定位。AutoError 的 Display 链不含位置;`Diagnostic::labels()`
/// 给出字节偏移。MultipleErrors 递归展开(上限 20 条,与 parser 恢复上限一致)。
fn positioned_parse_errors(e: &auto_lang_core::error::AutoError, source: &str) -> String {
    use miette::Diagnostic;
    fn walk(
        e: &auto_lang_core::error::AutoError,
        source: &str,
        out: &mut Vec<String>,
        depth: usize,
    ) {
        if out.len() >= 20 || depth > 4 {
            return;
        }
        if let auto_lang_core::error::AutoError::MultipleErrors { errors, .. } = e {
            for sub in errors {
                walk(sub, source, out, depth + 1);
            }
            return;
        }
        let msg = e.to_string();
        let mut line_col = None;
        if let Some(span) = e.labels().and_then(|mut ls| ls.next()) {
            let mut offset = span.inner().offset();
            // Plan 011(os-config):span offset 可能落在多字节字符内部(如中文
            // 注释/字符串里报错)——直接切片会 panic("not a char boundary"),
            // 把真实语法错误掩盖成崩溃。回退到最近的字符边界。
            while offset > 0 && !source.is_char_boundary(offset) {
                offset -= 1;
            }
            if offset <= source.len() {
                let until = &source[..offset];
                let line = until.matches('\n').count() + 1;
                let col = until
                    .rsplit('\n')
                    .next()
                    .map(|l| l.chars().count())
                    .unwrap_or(0)
                    + 1;
                line_col = Some((line, col));
            }
        }
        out.push(match line_col {
            Some((l, c)) => format!("  error at {l}:{c}: {msg}"),
            None => format!("  error (no span): {msg}"),
        });
    }
    let mut out = Vec::new();
    walk(e, source, &mut out, 0);
    if out.is_empty() {
        e.to_string()
    } else {
        out.join("\n")
    }
}

pub(crate) fn collect_module_imports(
    module_path: &std::path::Path,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
    out: &mut Vec<auto_lang_core::ast::Stmt>,
    seen: &mut std::collections::HashSet<String>,
    session: &mut auto_lang_core::compile::CompileSession,
    override_scenario: Option<&auto_lang_core::session::CompilerSession>,
) {
    let canon = module_path
        .canonicalize()
        .unwrap_or_else(|_| module_path.to_path_buf());
    if !visited.insert(canon) {
        return;
    }
    let code = match std::fs::read_to_string(module_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    // PLAN-037: launch 期作用域化模块源前缀化（未激活时零成本透传）。
    let code = auto_lang_core::back_prefix::apply(module_path, code);
    // Plan 333: pre-resolve this module's `use` deps into the shared type_store
    // so types like `Note` (from `use api: Note`) are registered before parse.
    // resolve_uses is itself recursive (it follows each module's own uses), so
    // transitive types land in the store. Seed source_dirs with this module's
    // directory AND its parent so `back.api` resolves from `src/front/` and
    // `use api` resolves from `src/back/`.
    if let Some(dir) = module_path.parent() {
        let abs = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        session.add_source_dir(abs);
        if let Some(parent) = dir.parent() {
            let abs_parent = std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
            session.add_source_dir(abs_parent);
        }
    }
    // Tolerate resolve_uses failures (e.g. a rust dep not declared); we still
    // attempt the parse below with whatever types did resolve.
    let _ = session.resolve_uses(&code);

    // Choose parser scenario. Priority (PR-6):
    //   1. override_scenario (from CLI --scene / pac.at scene field)
    //   2. .au extension → UI scenario (弱提示推断)
    //   3. path heuristic: back/ → Core, else UI
    //   4. default: Core
    let parser_session = if let Some(s) = override_scenario {
        s.clone()
    } else if module_path.extension().map_or(false, |e| e == "au") {
        // PR-6: .au files default to UI scenario (weak hint, overridable by --scene).
        auto_lang_core::session::CompilerSession::ui()
    } else if module_path.components().any(|c| c.as_os_str() == "back") {
        auto_lang_core::session::CompilerSession::core()
    } else {
        auto_lang_core::session::CompilerSession::ui()
    };
    let mut parser = Parser::new_with_type_store(code.as_str(), session.type_store())
        .with_session(parser_session);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            // Plan 398: surface parse failures instead of silently dropping the
            // module. Previously a parse error here caused ALL of the module's
            // symbols (e.g. every `api.*` fn) to vanish from the VM module,
            // producing misleading "Undefined symbol: api.X" link errors far
            // downstream. Log so the real .at parse issue is visible.
            //
            // Plan 446 批一 (C1-2/C1-3): (1) 诊断带 文件:行:列 —— Display 链不含
            // 位置,这里从 miette Diagnostic 的 labels 提取 span 偏移,对照手边的
            // 源码换算行列;(2) 记入全局注册表,VmBridge 构造时升级为致命错误
            // (此前 WARN + 静默空渲染,os-config C1 现场"模块丢弃极难定位")。
            // 本函数保持非致命返回(投机解析路径安全),致命化在桥接层统一执行。
            let detail = positioned_parse_errors(&e, &code);
            eprintln!(
                "\n════ module parse failed (fatal at boot, plan-446 C1) ════\n  file: {}\n{}\n════════════════════════════════════════════════════════",
                module_path.display(),
                detail
            );
            log::warn!(
                "collect_module_imports: parse failed for {}: {}",
                module_path.display(),
                e
            );
            ui_module_parse_failures().lock().unwrap().push(format!(
                "{}: {}",
                module_path.display(),
                detail.lines().next().unwrap_or("(no detail)").trim()
            ));
            return;
        }
    };
    // Plan 339: derive module name from file path.
    // back/db.at → "db", back/api.at → "api", front/editor.at → "editor"
    let module_name = module_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // 1. Take every declaration in this module (dedup by qualified name).
    //    Plan 339: prefix functions from back/ modules with the module name
    //    (e.g. db.at's create_note → db.create_note) so db.at and api.at's
    //    functions don't collide. This replaces the last-wins workaround
    //    (Plan 338) since qualified names are unique.
    for stmt in &ast.stmts {
        match stmt {
            auto_lang_core::ast::Stmt::Fn(_) => {
                if let Some(name) = stmt_symbol_name(stmt) {
                    // Plan 339 Phase 3: ALL Fn declarations from ALL modules get
                    // a module-qualified name (e.g. api.create_note, db.create_note).
                    // TypeDecl/EnumDecl/Ext are NOT qualified — types are shared.
                    let qualified = format!("{}.{}", module_name, name);
                    if seen.insert(qualified.clone()) {
                        let mut s = stmt.clone();
                        if let auto_lang_core::ast::Stmt::Fn(ref mut f) = s {
                            f.name = auto_lang_core::ast::Name::from(qualified.as_str());
                        }
                        out.push(s);
                    }
                }
            }
            auto_lang_core::ast::Stmt::TypeDecl(_)
            | auto_lang_core::ast::Stmt::EnumDecl(_)
            | auto_lang_core::ast::Stmt::Ext(_) => {
                if let Some(name) = stmt_symbol_name(stmt) {
                    if seen.insert(name.clone()) {
                        out.push(stmt.clone());
                    }
                }
            }
            // Use statements have no dedup key; emit them as-is. (codegen
            // re-dedups auto_modules registration by name.)
            auto_lang_core::ast::Stmt::Use(_) => {
                out.push(stmt.clone());
            }
            // Module-level stores (globals). Dedup by the variable name so a
            // transitive re-import doesn't double-declare.
            auto_lang_core::ast::Stmt::Store(s) => {
                let key = format!("__store:{}", s.name);
                if seen.insert(key) {
                    out.push(stmt.clone());
                }
            }
            // VM multi-store fix: collect StoreDecl (store Name { ... }) from
            // imported modules so stores used only by route pages (not the root
            // widget) are compiled into the VM module. Without this, the store's
            // handlers are undefined at link time. Dedup by store name.
            auto_lang_core::ast::Stmt::StoreDecl(sd) => {
                // PLAN-733: 登记 store 源文件模块（file stem）——合成该
                // store 的 handler 时作 own-module 裸名绑定域（同文件模块级
                // fn 裸调用不再因跨模块同名歧义悬空）。
                #[cfg(feature = "ui")]
                crate::ui::handler_codegen::register_store_source_module(
                    &sd.name.to_string(),
                    &module_name,
                );
                let key = format!("__storedecl:{}", sd.name);
                if seen.insert(key) {
                    out.push(stmt.clone());
                }
            }
            _ => {}
        }
    }
    // 2. Recurse into this module's own `use` dependencies, each resolved
    //    relative to THIS module's directory (so back/api.at's `use db` finds
    //    back/db.at).
    let module_dir = module_path.parent().unwrap_or(std::path::Path::new("."));
    for dep in auto_lang_core::use_scanner::scan_use_statements(&code) {
        if dep.is_c_import || dep.is_rust_import {
            continue;
        }
        match resolve_use_module(module_dir, &dep) {
            UseModuleResolution::Module(dep_path) => {
                collect_module_imports(&dep_path, visited, out, seen, session, override_scenario);
            }
            UseModuleResolution::StoreFiles(found) => {
                // Plan 442 A2: transitive `use store: X` (legacy facade form) —
                // module name never maps to store.at; locate the StoreDecl's file
                // by convention/scan so the store (and its own `use` deps)
                // compile into the VM module. Without this, a child widget's
                // store stays uncollected and its handlers fail at link time.
                for (_, path) in found {
                    collect_module_imports(&path, visited, out, seen, session, override_scenario);
                }
            }
            UseModuleResolution::None => {
                // PLAN-664 U-4（P-15）：传递性 use 同告警化（见根 use 环
                // 同名诊断注记）；review R1 F-1 同卫（非文件系统形态）。
                if !is_non_fs_module_use(&dep) {
                    auto_lang_core::use_diag(format!(
                        "[AUTO-USE-DIAG] P-15 传递 use 模块 `{}` 解析失败（items: {:?}）——基目录 {} 均无对应 .at/mod.at（PLAN-664）。该导入静默跳过。",
                        dep.module,
                        dep.items,
                        module_dir.display()
                    ));
                }
            }
        }
    }
}

/// Run a .at file as dynamic UI with iced backend.
/// MUST be called from the OS main thread (iced requirement).
#[cfg(feature = "ui-iced")]
pub fn run_file_dynamic_ui(code: &str, path: Option<&str>) -> AutoResult<String> {
    // PR-6: 默认 UI 场景（向后兼容）。调用方可通过 run_file_dynamic_ui_with_scenario 覆写。
    run_file_dynamic_ui_inner(code, path, None)
}

/// PR-6: 带场景覆写的 run_file_dynamic_ui。
/// override_scenario 为 None 时默认 UI（向后兼容）。
#[cfg(feature = "ui-iced")]
pub fn run_file_dynamic_ui_with_scenario(
    code: &str,
    path: Option<&str>,
    override_scenario: Option<&auto_lang_core::session::CompilerSession>,
) -> AutoResult<String> {
    run_file_dynamic_ui_inner(code, path, override_scenario)
}

/// EDGE-16 第四层:递归注册传递性子组件(孙组件)。
///
/// 直接子组件(如 block_list.at 的 BlockList)自己 `use` 的子组件(如
/// block_item.at 的 BlockItem)默认不进 registry(lib.rs 的 widget 收集
/// 注释明确说 "Transitive modules do not register widgets here"),导致
/// BlockList 渲染时 `<BlockItem ...>` 在 registry 找不到 → fallback Empty。
///
/// 此函数从 `module_path` 出发,扫它的 `use` 导入,把传递性 widget 注册进
/// `registry`。用 `visited` 防循环(与 collect_module_imports 共享 visited)。
/// 只注册 AuraWidget(孙组件),不重复处理 store(store 走 Plan 370 路径)。
#[cfg(feature = "ui-iced")]
fn register_transitive_widgets(
    module_path: &std::path::Path,
    registry: &mut crate::ui::widget_registry::WidgetRegistry,
    child_decls: &mut Vec<auto_lang_core::ast::WidgetDecl>,
) {
    // 独立的 visited(不与 collect_module_imports 共享,避免互相跳过模块)。
    register_transitive_widgets_inner(
        module_path,
        registry,
        child_decls,
        &mut std::collections::HashSet::new(),
    )
}

#[cfg(feature = "ui-iced")]
fn register_transitive_widgets_inner(
    module_path: &std::path::Path,
    registry: &mut crate::ui::widget_registry::WidgetRegistry,
    child_decls: &mut Vec<auto_lang_core::ast::WidgetDecl>,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    let module_code = match std::fs::read_to_string(module_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    // PLAN-037: 同上——孙组件模块读点过前缀化 overlay。
    let module_code = auto_lang_core::back_prefix::apply(module_path, module_code);
    let base_dir = module_path.parent().unwrap_or(std::path::Path::new("."));
    let use_stmts = auto_lang_core::use_scanner::scan_use_statements(&module_code);
    for use_stmt in &use_stmts {
        if use_stmt.is_c_import || use_stmt.is_rust_import {
            continue;
        }
        let sub_path = match auto_lang_core::resolve_module_path(base_dir, &use_stmt.module) {
            Some(p) => p,
            None => continue,
        };
        // 防循环:规范化路径后查 visited
        let canon = match sub_path.canonicalize() {
            Ok(c) => c,
            Err(_) => sub_path.clone(),
        };
        if !visited.insert(canon.clone()) {
            continue;
        }
        let sub_code = match std::fs::read_to_string(&sub_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // PLAN-037: 同上——孙组件子模块读点过前缀化 overlay。
        let sub_code = auto_lang_core::back_prefix::apply(&sub_path, sub_code);

        // PLAN-668 A-02：逐模块 stylekit 预注册（同 4244 主路径——模块级
        // `use stylekit.styles:` 缺预注册则整文件 parse 失败被静默吞）。
        let _ = auto_lang_core::design_tokens::recipe::prepare_style_recipe_imports(
            sub_path.parent().unwrap_or(std::path::Path::new(".")),
            &sub_code,
        );
        let mod_session = auto_lang_core::session::CompilerSession::ui();
        let mut mod_parser = crate::Parser::from(sub_code.as_str()).with_session(mod_session);
        if let Ok(sub_ast) = mod_parser.parse() {
            for stmt in &sub_ast.stmts {
                if let auto_lang_core::ast::Stmt::WidgetDecl(decl) = stmt {
                    if let Ok(child_widget) = auto_ui_compiler::aura::extract_widget_from_decl(decl)
                    {
                        // 只注册 use 子句明确要的(或通配/裸装载的),且 registry
                        // 还没有的。PLAN-545 回归修复（069 收口转介收回）：
                        // bare `use mod` 的组件发现语义恢复（items.is_empty()
                        // 臂）——545 的"bare=命名空间"针对 fn 符号冲突面；
                        // UI 子件码道收紧后裸 use 孙件 handler 不再合成
                        // （同 demo CustomScrollbar 断裂）。
                        // PLAN-093 G-16（095 T-07 根文件修复的传递装载对应
                        // 面）：同文件兄弟 widget 一并注册——items 过滤只决定
                        // "显式引用谁"，不应把同文件其他 widget 逐出 registry
                        // （musk canvas_panel.at 实证：use 只列
                        // CanvasPanel/CanvasStudioSlot，兄弟 CanvasCanvasColumn
                        // 被 skip → 组件臂 miss → 画布列空渲染）。registry
                        // 已有者仍跳过（先到优先，显式 use 同名覆写沿 095
                        // 口径由后装载覆盖）。
                        if registry.get(&child_widget.name).is_none() {
                            // Plan 049:孙组件 handler 也要编译进 VM 模块,
                            // 否则点击其按钮(如 BlockItem.ToggleCollapse)会
                            // HandlerNotFound 静默失败。
                            child_decls.push(decl.clone());
                            registry.register(child_widget);
                        }
                    }
                }
                // Plan 044 M3: 传递性模块的 view fn 也注册(同类 EDGE-16c,
                // 但 view fn 走 register_view_fragment 而非 widget registry)。
                if let auto_lang_core::ast::Stmt::ViewFragmentDecl(frag) = stmt {
                    auto_ui_compiler::aura::extract::register_view_fragment(frag);
                }
            }
        }
        // 继续递归(孙组件可能还 use 了更深的组件)
        register_transitive_widgets_inner(&sub_path, registry, child_decls, visited);
    }
}

#[cfg(feature = "ui-iced")]
fn build_dynamic_component_inner(
    code: &str,
    path: Option<&str>,
    override_scenario: Option<&auto_lang_core::session::CompilerSession>,
) -> AutoResult<crate::ui::dynamic::DynamicComponent> {
    use crate::ui::dynamic::DynamicComponent;
    use crate::ui::widget_registry::WidgetRegistry;
    use auto_lang_core::session::CompilerSession;

    // PLAN-635: pre-register use-imported recipes BEFORE the parse — the
    // parser's symbol checker accepts `style: <name>` idents only when the
    // recipe is in the live registry.
    let recipe_imports = match path {
        Some(p) => {
            let base_dir = std::path::Path::new(p)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            auto_lang_core::design_tokens::recipe::prepare_style_recipe_imports(&base_dir, code)
                .map_err(|e| e.to_string())?
        }
        None => Vec::new(),
    };

    // 1. Parse with UI scenario (or override if provided — PR-6)
    let session = override_scenario
        .cloned()
        .unwrap_or_else(CompilerSession::ui);
    let mut parser = Parser::from(code).with_session(session);
    let ast = parser.parse()?;

    // 2. Extract first AuraWidget (and keep its WidgetDecl — PR-3b Step 4).
    // Plan 367 P2-3: register the root module's `view fn` fragments before
    // extracting the root widget's view tree, so fragment calls inline-expand.
    auto_ui_compiler::aura::extract::clear_view_fragments();
    for stmt in &ast.stmts {
        if let auto_lang_core::ast::Stmt::ViewFragmentDecl(frag) = stmt {
            auto_ui_compiler::aura::extract::register_view_fragment(frag);
        }
    }
    // PLAN-607: register and validate style recipes before extracting widget.
    // PLAN-635: replay imported (source-tagged) + local recipes in order.
    auto_lang_core::design_tokens::recipe::load_and_validate_style_recipes_with_imports(
        &recipe_imports,
        &ast.stmts,
    )
    .map_err(|e| e.to_string())?;
    let mut root_decl: Option<auto_lang_core::ast::WidgetDecl> = None;
    let mut widget = None;
    // PLAN-095 T-07 (G-11): 同文件兄弟 widget 一并提取注册——此前 break 只取
    // 首个 WidgetDecl，同文件兄弟不进 WidgetRegistry，组件臂 miss 静默降级，
    // 子件整体零节点（093 §8.16 实证）。兄弟先于 use 导入注册（显式 use
    // 稍后装载时同名覆写——显式导入优先）。
    let mut sibling_decls: Vec<auto_lang_core::ast::WidgetDecl> = Vec::new();
    for stmt in &ast.stmts {
        if let auto_lang_core::ast::Stmt::WidgetDecl(decl) = stmt {
            if root_decl.is_none() {
                root_decl = Some(decl.clone());
                widget = Some(
                    auto_ui_compiler::aura::extract_widget_from_decl(decl)
                        .map_err(|e| e.to_string())?,
                );
            } else {
                sibling_decls.push(decl.clone());
            }
        }
    }

    let widget = widget.ok_or("No widget declaration found")?;
    let root_decl = root_decl.ok_or("No widget declaration found")?;

    // PLAN-050 T9 (C7): VM 轨 t() 插值查表——front 根目录 i18n/{lang}.json
    // 存在则装载（默认 zh, AUTO_LOCALE 可覆）；缺席 = 空表静默。
    if let Some(file_path) = path {
        if let Some(base_dir) = std::path::Path::new(file_path).parent() {
            crate::ui::i18n_lookup::load_from_dir(base_dir);
        }
    }

    // Plan 451: root widget 的 `actions {}` 块安装 DSL 动作配置（优先于
    // auto-atom 外挂文件；热重载经 action_config::reload_action_config
    // 重读本源文件）。消费点（键盘回退层/menubar·toolbar 合成/MCP 同源）
    // 全部经 action_config() 取数，零改动。
    // Plan 451 P3 合并优先级：宿主 widget 自带块 > 本文件顶层声明 >
    // use 引入模块的顶层声明（导入序首个）。顶层声明的 handler 引用此时
    // 才有宿主，不做解析期校验（由 from_actions_block 的引用校验兜底）。
    let mut dsl_actions_installed = false;
    if let Some(ref acts) = root_decl.actions {
        crate::ui::action_config::set_dsl_action_config_from_block(acts, path);
        dsl_actions_installed = true;
    }

    // 2b. Load child widgets + imported functions/types from `use` imports
    let mut registry = WidgetRegistry::new();
    // PR-3b Step 4: collect child WidgetDecls alongside child AuraWidgets so the
    // decl-based synthesis path can compile child handlers into the single VM.
    let mut child_decls: Vec<auto_lang_core::ast::WidgetDecl> = Vec::new();
    // PLAN-095 T-07 (G-11): 同文件兄弟 widget 注册进 registry + child_decls
    //（handler 编入单 VM，与 use 导入子件同面）。
    for decl in &sibling_decls {
        let aura =
            auto_ui_compiler::aura::extract_widget_from_decl(decl).map_err(|e| e.to_string())?;
        registry.register(aura);
        child_decls.push(decl.clone());
    }
    let mut import_stmts: Vec<auto_lang_core::ast::Stmt> = Vec::new();
    // Visited-module / seen-symbol sets span ALL top-level use stmts so shared
    // transitive deps load once.
    let mut visited = std::collections::HashSet::new();
    let mut seen_symbols = std::collections::HashSet::new();
    // Plan 333: a shared CompileSession carries the type_store across the whole
    // recursive import walk so imported types resolve before each module parse.
    let mut import_session = auto_lang_core::compile::CompileSession::new();
    // Plan 339 Phase 4: collect use alias mappings (bare name → module-qualified name)
    let mut import_aliases: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    if let Some(file_path) = path {
        let use_stmts = auto_lang_core::use_scanner::scan_use_statements(code);
        let base_dir = std::path::Path::new(file_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        // PLAN-012 F2：`use` 兄弟模块解析的**仓根回退基**——source path 为
        // 仓根相对形态（ui_desktop 直挂窗：`examples/ui/...`）且 CWD ≠ 仓根
        // 时（desktop.ps1 iced 轨 CWD=os 根），CWD 相对解析永不命中，
        // `use prog_util` 链接死（Undefined symbol——桌面 boot 直挂计算器
        // 实测）。回退基 = 语言仓根（编译期锚，CWD 无关；主检出/worktree
        // 构建各自自洽）。首基（CWD 相对）优先，回退基仅在前者 miss 时参与。
        let mut base_dirs: Vec<std::path::PathBuf> = vec![base_dir.to_path_buf()];
        // PLAN-041 T-02（T-01 决策件推荐臂：装载链补 back 符号注册）：
        // auto-os app 布局 = `src/front/` + `src/back/`，front 的
        // `use back.api` 按既有基目录只探 `front/back/api.at`——miss 后
        // 静默跳过（下方 `if let Ok(read_to_string)`），符号悬空至链接期
        // `Undefined symbol: api.kill_process` 死窗（025-sys-monitor/
        // kanban/017-chat 家族实录）。增补 entry 父目录（= `src/`）为次
        // 基：`back.api → src/back/api.at` 命中，back 模块随 import 束
        // 正常编译链接。首基序不变（front 兄弟优先，无回归面）。
        if let Some(src_dir) = base_dir.parent() {
            if src_dir.file_name().map_or(false, |n| n == "src") {
                base_dirs.push(src_dir.to_path_buf());
            }
        }
        let lang_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .map(|p| p.to_path_buf());
        if let Some(root) = &lang_root {
            let fb = root.join(base_dir);
            if !base_dirs.contains(&fb) {
                base_dirs.push(fb);
            }
        }

        for use_stmt in &use_stmts {
            // Skip non-module imports (c, rust, py)
            if use_stmt.is_c_import || use_stmt.is_rust_import {
                continue;
            }
            // PLAN-664 U-4（P-16）：use 导入符号与 widget model 字段撞名 =
            // link 期 "Undefined symbol: App_State.<field>" 死面（080 T-03
            // 实录：`use panels_graph_fns: graph_stats` × `var graph_stats
            // map`，MCP 不起、零编译期告警）。撞名时名字被状态通道抢占，
            // 此处编译期告警化；硬错化留待确认无合法撞名形态后另议。
            // review R1 F-1：非文件系统形态（auto.*/.web/use.py）跳过——
            // 其 items 是外域符号，撞名检查无意义且会误报。
            if !is_non_fs_module_use(use_stmt) {
                for item in &use_stmt.items {
                    let mut owner: Option<String> = None;
                    if let Some(model) = &root_decl.model {
                        if model.fields.iter().any(|f| f.name.as_str() == item) {
                            owner = Some(root_decl.name.to_string());
                        }
                    }
                    if owner.is_none() {
                        for d in &child_decls {
                            if d.model
                                .as_ref()
                                .map_or(false, |m| m.fields.iter().any(|f| f.name.as_str() == item))
                            {
                                owner = Some(d.name.to_string());
                                break;
                            }
                        }
                    }
                    if let Some(w) = owner {
                        auto_lang_core::use_diag(format!(
                        "[AUTO-USE-DIAG] P-16 use 导入符号 `{item}` 与 widget `{w}` 的 model 字段撞名——VM 轨该名字被状态通道占用，fn 导入将在 link 期落 `Undefined symbol` 类死面（PLAN-664）。fn 与字段改名错开其一。"
                    ));
                    }
                }
            }
            // Locate the module source file. A dotted module path maps to a
            // filesystem path: `back.api` → `back/api.at`, `calendar_util` →
            // `calendar_util.at`. Per the module rules (CLAUDE.md) a module is
            // either `{path}.at` or `{path}/mod.at`; try both.
            // PLAN-012 F2：逐基尝试（CWD 相对首基优先，仓根回退基兜底）。
            let mut module_path = UseModuleResolution::None;
            for base in &base_dirs {
                module_path = resolve_use_module(base, use_stmt);
                if !matches!(module_path, UseModuleResolution::None) {
                    break;
                }
            }
            let module_path = match module_path {
                UseModuleResolution::Module(p) => p,
                UseModuleResolution::StoreFiles(found) => {
                    // Plan 442 A2 (store facade): legacy `use store: Name` uses
                    // the literal module name "store" — no store.at exists; the
                    // StoreDecl lives in a sibling file named after the store.
                    // Feed each located file through collect_module_imports so
                    // the store's decl (→ view-less child WidgetDecl via the
                    // import conversion below) and its own `use` deps compile
                    // into the VM module. Without this the store context stays
                    // empty and handlers/views referencing `store.X` fail with
                    // "Undefined variable: store" (musk KNOWN-DEBT 028 ③).
                    for (name, path) in &found {
                        log::info!(
                            "store facade: `use store: {}` resolved to {}",
                            name,
                            path.display()
                        );
                        collect_module_imports(
                            path,
                            &mut visited,
                            &mut import_stmts,
                            &mut seen_symbols,
                            &mut import_session,
                            override_scenario,
                        );
                    }
                    for name in &use_stmt.items {
                        if !found.contains_key(name) {
                            log::warn!(
                                "store facade: `use store: {}` — no StoreDecl file found under {} (looked for {}.at)",
                                name,
                                base_dir.display(),
                                store_name_snake(name)
                            );
                        }
                    }
                    continue;
                }
                UseModuleResolution::None => {
                    // PLAN-664 U-4（P-15）：fn/组件 use 模块解析失败此前
                    // 静默跳过——消费侧表现 = 引用符号的视图/handler 落空、
                    // App 空视图零报错（jade 079 E-8 跨项目 fn 导入静默
                    // 死面实录）。编译期告警化：留下可 grep 诊断；硬错化
                    // 留待确认无合法未解析形态后另议。
                    // review R1 F-1：非文件系统形态（auto.*/.web/use.py）
                    // 解析失败是预期路由，不告警（jade use auto.http
                    // 实测假阳性）。
                    if !is_non_fs_module_use(use_stmt) {
                        auto_lang_core::use_diag(format!(
                            "[AUTO-USE-DIAG] P-15 use 模块 `{}` 解析失败（items: {:?}）——候选基目录均无对应 .at/mod.at（PLAN-664）。该导入静默跳过，引用其符号的面将落空。",
                            use_stmt.module, use_stmt.items
                        ));
                    }
                    continue;
                }
            };

            // Register child widgets declared directly in this top-level module
            // (filtered by the use clause's item list). Transitive modules do
            // not register widgets here.
            // PLAN-668 A-02：逐模块文件 stylekit 预注册——模块自身可携带
            // `use stylekit.styles:`（PLAN-637 起 editor.at 实案），host 级
            // 预注册只覆盖根 app.at；缺本步则模块 parse 落 UndefinedVariable
            // 被 `if let Ok` 静默吞掉 → 子 widget/handler 整件丢失（015
            // EditorPanel：handler_EditorPanel_EditTitle 不在 exports 实证）。
            if let Ok(module_code) = std::fs::read_to_string(&module_path) {
                // PLAN-037: 同上——子模块读点过前缀化 overlay。
                let module_code = auto_lang_core::back_prefix::apply(&module_path, module_code);
                // PLAN-668 A-02：逐模块文件 stylekit 预注册（同上，变换后扫等价）。
                let _ = auto_lang_core::design_tokens::recipe::prepare_style_recipe_imports(
                    module_path.parent().unwrap_or(std::path::Path::new(".")),
                    &module_code,
                );
                let mod_session = CompilerSession::ui();
                let mut mod_parser = Parser::from(module_code.as_str()).with_session(mod_session);
                if let Ok(mod_ast) = mod_parser.parse() {
                    // Plan 367 P2-3: register this module's `view fn` fragments
                    // BEFORE extracting any widget view trees below. A widget in
                    // the same module (e.g. NavTree in sidebar.at) calls a
                    // fragment (e.g. NoteItem); extract_widget_from_decl inlines
                    // those calls only if the fragment is in the VIEW_FRAGMENTS
                    // thread_local. Without this the call stays as an unexpanded
                    // component node and renders empty in VM mode.
                    for stmt in &mod_ast.stmts {
                        if let auto_lang_core::ast::Stmt::ViewFragmentDecl(frag) = stmt {
                            auto_ui_compiler::aura::extract::register_view_fragment(frag);
                        }
                    }
                    for stmt in &mod_ast.stmts {
                        if let auto_lang_core::ast::Stmt::WidgetDecl(decl) = stmt {
                            if let Ok(child_widget) =
                                auto_ui_compiler::aura::extract_widget_from_decl(decl)
                            {
                                // PLAN-545 回归修复（069 收口转介收回）：bare
                                // use 的**组件发现**语义恢复——`use custom_scrollbar`
                                //（无 items 无通配）装载该模块的子件。545 的
                                // "bare=命名空间"裁定针对 fn 符号冲突面
                                // （compile.rs/linker），此处是 UI 子件收集码道；
                                // 收紧时连带删掉 items.is_empty() 臂令裸 use 子件
                                // 的 handler 不再合成（handler_<Child>_* 导出
                                // 缺席→运行时派发全灭，demo CustomScrollbar
                                // 拖拽死——vm-smoke 组 4(d) 实证）。
                                // PLAN-093 G-16（095 T-07 的导入模块对应面）：
                                // 同文件兄弟 widget 一并注册——use items 过滤
                                // 只决定"显式引用谁"，不应把同文件其他 widget
                                // 逐出 registry（musk canvas_panel.at 实证：
                                // 兄弟 CanvasCanvasColumn 被 skip → 组件臂
                                // miss → 画布列空渲染）。显式 use 同名覆写
                                // 优先沿 095 根文件修复口径（后到覆盖）。
                                if std::env::var_os("AUTO_DEBUG_G16").is_some() {
                                    eprintln!(
                                        "[G16] load-site register: {} from {:?} (matched={})",
                                        child_widget.name,
                                        module_path.display(),
                                        use_stmt.items.iter().any(|s| s == &child_widget.name)
                                    );
                                }
                                child_decls.push(decl.clone());
                                registry.register(child_widget);
                            }
                        } else if let auto_lang_core::ast::Stmt::StoreDecl(store_decl) = stmt {
                            // Plan 370 D-GAP-4: stores imported via `use module: Store`
                            // (e.g. `use notes_store: NotesStore`) live in their own
                            // source file, NOT the root AST. Convert them to view-less
                            // child WidgetDecls here — mirroring the root-AST store
                            // conversion below — so their state fields get merged into
                            // the root state object and their handlers compile into the
                            // single VM module. Without this, `store.X` fields (notes,
                            // active_id, ...) never exist in `__state`, so handlers
                            // like `.notes = list_notes()` silently no-op and the view
                            // renders an empty list.
                            let name = store_decl.name.clone();
                            // 069 同款回归修复：bare use 亦装载 store（组件
                            // 发现语义，见上方 WidgetDecl 臂注）。
                            if use_stmt.is_wildcard
                                || use_stmt.items.is_empty()
                                || use_stmt.items.iter().any(|s| *s == name.as_str())
                            {
                                child_decls.push(auto_lang_core::ast::ui::WidgetDecl {
                                    name: name.clone(),
                                    messages: store_decl.messages.clone(),
                                    model: store_decl.model.clone(),
                                    computed: store_decl.computed.clone(),
                                    view: None,
                                    named_views: Vec::new(),
                                    on: store_decl.on.clone(),
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
                                    timer: store_decl.timer.clone(),
                                });
                            }
                        }
                    }
                }
            }

            // EDGE-16 第四层:递归收集孙组件(传递性 widget)。
            // 直接子组件(block_list.at 的 BlockList)自己 use 的子组件
            // (block_item.at 的 BlockItem)不会进 registry,导致 BlockList
            // 渲染时 BlockItem 找不到 → fallback Empty。这里递归扫当前模块的
            // use,把孙组件也注册进同一个 registry,并同步收集其 WidgetDecl,
            // 让孙组件的 handler(model var 读写)也编译进单一 VM 模块。
            // Plan 049:没有后者,BlockItem 的 .ToggleCollapse handler 不存在,
            // 点击折叠静默失败(HandlerNotFound)。
            register_transitive_widgets(&module_path, &mut registry, &mut child_decls);

            // Recursively collect ALL declarations from this module + its
            // transitive `use` deps (full module load so an imported fn's
            // intra-module callees — e.g. build_month_grid → weekday_of — and
            // cross-module deps — e.g. back.api → back.db — both resolve).
            collect_module_imports(
                &module_path,
                &mut visited,
                &mut import_stmts,
                &mut seen_symbols,
                &mut import_session,
                override_scenario,
            );

            // Plan 339 Phase 4: populate import_scope aliases directly from the
            // use_scanner data. This is more reliable than parsing use statements.
            // use back.api: create_note → import_aliases["create_note"] = "api.create_note"
            let module_qualifier = use_stmt
                .module
                .split('.')
                .last()
                .unwrap_or(&use_stmt.module);
            for item in &use_stmt.items {
                let qualified = format!("{}.{}", module_qualifier, item);
                import_aliases.insert(item.clone(), qualified);
            }
        }
    }

    // 2c. Plan 401/VM-routing: load route-target pages (pages/{module}.at).
    // The vue codegen discovers these via `routes { "/" -> use bookshelf }`
    // and emits a router; the VM/iced path instead renders the matched page
    // widget inside `outlet`. But the VM loader (above) only follows top-level
    // `use` lines — `-> use X` inside `routes {}` is invisible to use_scanner.
    // So explicitly load each route's page module + register its widget, the
    // same way top-level `use` modules are handled above. This makes the page
    // widgets available for outlet to render (see aura_view_builder.rs Outlet).
    if let Some(file_path) = path {
        if let Some(ref routes_block) = root_decl.routes {
            let base_dir = std::path::Path::new(file_path)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            for route in &routes_block.routes {
                // Convention: `-> use bookshelf` maps to pages/bookshelf.at
                // (mirrors vue's @/pages/{module}.vue — see ast/route.rs:45-46).
                let page_module = format!("pages/{}", route.module);
                let module_path = resolve_module_path(base_dir, &page_module);
                let Some(module_path) = module_path else {
                    continue;
                };
                if let Ok(module_code) = std::fs::read_to_string(&module_path) {
                    // PLAN-037: 路由页读点同过前缀化 overlay——PLAN-712 T-17：
                    // 此前此处**裸读**（全部模块读点中唯一未包 apply 的），
                    // routes 页（pages/*.at）的相对 `/api/` 字面量不被改写 →
                    // fetch 解析到 app 自身 back origin（死端口）→ get_json
                    // 吞错返空 = 018 详情页「0 entries」（实机实证：顶层
                    // book_store.at 前缀生效、书架 2 REQ 到 proxy；pages/ 页
                    // 零 REQ + 86ms 秒拒）。
                    let module_code = auto_lang_core::back_prefix::apply(&module_path, module_code);
                    let mod_session = CompilerSession::ui();
                    let mut mod_parser =
                        Parser::from(module_code.as_str()).with_session(mod_session);
                    if let Ok(mod_ast) = mod_parser.parse() {
                        // Register view-fn fragments from the page module too.
                        for stmt in &mod_ast.stmts {
                            if let auto_lang_core::ast::Stmt::ViewFragmentDecl(frag) = stmt {
                                auto_ui_compiler::aura::extract::register_view_fragment(frag);
                            }
                        }
                        for stmt in &mod_ast.stmts {
                            if let auto_lang_core::ast::Stmt::WidgetDecl(decl) = stmt {
                                if let Ok(child_widget) =
                                    auto_ui_compiler::aura::extract_widget_from_decl(decl)
                                {
                                    child_decls.push(decl.clone());
                                    // Plan 408: record route.module → widget.name
                                    // mapping so render_outlet can find the page.
                                    // Using a separate alias map (not a registry
                                    // entry) avoids shadowing built-in UI elements
                                    // like <button>, <input>, etc.
                                    registry
                                        .register_route_alias(&route.module, &child_widget.name);
                                    registry.register(child_widget);
                                }
                            }
                            // VM multi-store fix: also collect StoreDecls declared
                            // in the page's own source (e.g. home.at has
                            // `use article_store: ArticleStore` → article_store.at
                            // has the StoreDecl). Without this, stores only used by
                            // route pages never enter all_decls → their handlers
                            // are undefined at link time. This mirrors the root-AST
                            // store conversion (line ~2582).
                            if let auto_lang_core::ast::Stmt::StoreDecl(store_decl) = stmt {
                                let name = store_decl.name.clone();
                                child_decls.push(auto_lang_core::ast::ui::WidgetDecl {
                                    name: name.clone(),
                                    messages: store_decl.messages.clone(),
                                    model: store_decl.model.clone(),
                                    computed: store_decl.computed.clone(),
                                    view: None,
                                    named_views: Vec::new(),
                                    on: store_decl.on.clone(),
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
                                    timer: store_decl.timer.clone(),
                                });
                            }
                        }
                        // Pull in the page's own `use` deps (e.g. a page doing
                        // `use store: BooksStore` or `use back.api: list_books`)
                        // so its handlers compile.
                        collect_module_imports(
                            &module_path,
                            &mut visited,
                            &mut import_stmts,
                            &mut seen_symbols,
                            &mut import_session,
                            override_scenario,
                        );
                    }
                }
            }
        }
    }

    // 3. Create DynamicComponent with registry + imported symbols
    // Plan 451 P3: 顶层 actions 回退链（widget 块未安装时）：本文件顶层
    // 声明 → import_stmts 里首个模块声明（use 传递闭包已在此前收齐）。
    if !dsl_actions_installed {
        let fallback = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                auto_lang_core::ast::Stmt::ActionsDecl(b) => Some(b.clone()),
                _ => None,
            })
            .or_else(|| {
                import_stmts.iter().find_map(|s| match s {
                    auto_lang_core::ast::Stmt::ActionsDecl(b) => Some(b.clone()),
                    _ => None,
                })
            });
        if let Some(acts) = fallback {
            crate::ui::action_config::set_dsl_action_config_from_block(&acts, path);
        }
    }

    // Plan 370 D-GAP-4: collect StoreDecls and convert them to child WidgetDecls
    // so their state + handlers get compiled into the single VM module.
    // A store is isomorphic to a child widget minus a view — we give it an
    // empty view so the existing child-widget synthesis machinery handles it.
    let mut store_as_child_decls: Vec<auto_lang_core::ast::WidgetDecl> = Vec::new();
    for stmt in &ast.stmts {
        if let auto_lang_core::ast::Stmt::StoreDecl(store_decl) = stmt {
            // Convert StoreDecl → WidgetDecl with empty view
            let fake_widget = auto_lang_core::ast::ui::WidgetDecl {
                name: store_decl.name.clone(),
                messages: store_decl.messages.clone(),
                model: store_decl.model.clone(),
                computed: store_decl.computed.clone(),
                view: None, // stores have no view
                named_views: Vec::new(),
                on: store_decl.on.clone(),
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
                timer: store_decl.timer.clone(),
            };
            store_as_child_decls.push(fake_widget);
        }
    }
    // VM multi-store fix: also extract StoreDecls collected from route-page
    // imports (import_stmts now carries them via collect_module_imports).
    // Convert each to a WidgetDecl so its handlers compile into the VM module.
    // Dedup by store name against what root already collected.
    let root_store_names: std::collections::HashSet<String> = store_as_child_decls
        .iter()
        .map(|d| d.name.to_string())
        .collect();
    for stmt in &import_stmts {
        if let auto_lang_core::ast::Stmt::StoreDecl(store_decl) = stmt {
            let sname = store_decl.name.to_string();
            if !root_store_names.contains(&sname) {
                store_as_child_decls.push(auto_lang_core::ast::ui::WidgetDecl {
                    name: store_decl.name.clone(),
                    messages: store_decl.messages.clone(),
                    model: store_decl.model.clone(),
                    computed: store_decl.computed.clone(),
                    view: None,
                    named_views: Vec::new(),
                    on: store_decl.on.clone(),
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
                    timer: store_decl.timer.clone(),
                });
            }
        }
    }
    // Plan 435 P8-6(D13):包组件接入桌面端(iced)—— `use { package: x from
    // "dir" }` 的 widget:视图注册进 WidgetRegistry(fold 桥接见
    // WidgetRegistry::get),decl 并入 child_decls 使 handler 编入单 VM。
    // 与 vue 侧(api.rs SFC 生成)同源同机制;加载失败仅告警不阻塞。
    {
        let base_dir = path
            .map(|p| {
                std::path::Path::new(p)
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
            })
            .unwrap_or(std::path::Path::new("."));
        let mut pkg_reg = auto_ui_compiler::ui_gen::widget::ComponentRegistry::new();
        let mut seen_dirs: std::collections::HashSet<std::path::PathBuf> = Default::default();
        let mut sources: Vec<auto_ui_compiler::aura::AuraWidget> = vec![widget.clone()];
        sources.extend(registry.all());
        for w in &sources {
            for imp in &w.ext_imports {
                if !matches!(imp.kind, auto_lang_core::ast::ui::ExtImportKind::Package) {
                    continue;
                }
                let dir = std::path::PathBuf::from(imp.path.as_str());
                if !seen_dirs.insert(dir.clone()) {
                    continue;
                }
                // Plan 437 Phase 2:包路径基准与 vue 侧(api.rs)对齐 —— 页文件
                // 相对声明(`from "../components"`)在 pages/ 下解析;app 相对
                // 仍先行(根 widget 的声明形态)。两个候选逐一试,首个存在者
                // 生效,全部不存在才告警。
                let candidates = [base_dir.join(&dir), base_dir.join("pages").join(&dir)];
                let first = pkg_reg.load_package(&candidates[0], base_dir);
                let (loaded, loaded_dir) = match first {
                    Ok(p) => (Ok(p), Some(candidates[0].clone())),
                    Err(_) => match pkg_reg.load_package(&candidates[1], base_dir) {
                        Ok(p) => (Ok(p), Some(candidates[1].clone())),
                        Err(e) => (Err(e), None),
                    },
                };
                match loaded {
                    Ok(pkg) => {
                        for (d, aw) in &pkg.full_widgets {
                            child_decls.push(d.clone());
                            registry.register(aw.clone());
                        }
                        // Plan 522:包组件文件自身的 `use` 依赖也入编译单元。
                        // 此前仅根文件 use 链被收集——包组件 handler 引用的
                        // helper 模块(如 donut 的 chart_geom dc/ds)在 VM 侧
                        // 无定义;import_aliases 同步补齐(裸名 → 模块限定名,
                        // 与根文件 use 同规则)。
                        if let Some(pkg_dir) = loaded_dir {
                            if let Ok(entries) = std::fs::read_dir(&pkg_dir) {
                                for entry in entries.flatten() {
                                    let p = entry.path();
                                    if !p.extension().map(|e| e == "at").unwrap_or(false) {
                                        continue;
                                    }
                                    crate::collect_module_imports(
                                        &p,
                                        &mut visited,
                                        &mut import_stmts,
                                        &mut seen_symbols,
                                        &mut import_session,
                                        None,
                                    );
                                    if let Ok(code) = std::fs::read_to_string(&p) {
                                        for us in
                                            auto_lang_core::use_scanner::scan_use_statements(&code)
                                        {
                                            let qualifier =
                                                us.module.split('.').last().unwrap_or(&us.module);
                                            for item in &us.items {
                                                import_aliases.insert(
                                                    item.clone(),
                                                    format!("{}.{}", qualifier, item),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "package `{}` load failed (VM): {} (tried {:?})",
                            imp.path,
                            e,
                            candidates
                        )
                    }
                }
            }
        }
    }
    // Merge store-as-child decls with actual child widget decls
    let mut all_child_decls = child_decls.clone();
    // Plan 442 A2: also dedup against child decls — a store can reach
    // child_decls directly (unified-form module branch above) AND via
    // import_stmts (legacy store-facade fallback); compiling the same
    // StoreDecl twice would collide in the single VM module.
    let child_decl_names: std::collections::HashSet<String> =
        all_child_decls.iter().map(|d| d.name.to_string()).collect();
    store_as_child_decls.retain(|d| !child_decl_names.contains(&d.name.to_string()));
    all_child_decls.extend(store_as_child_decls);

    // Plan 403: collect the root module's own top-level fn/type/enum
    // declarations so they're compiled into the VM alongside imported ones.
    // Without this, a handler calling a top-level fn (e.g. fn eval_expr in
    // 011-calculator) fails with "Undefined symbol".
    for stmt in &ast.stmts {
        match stmt {
            auto_lang_core::ast::Stmt::Fn(_)
            | auto_lang_core::ast::Stmt::TypeDecl(_)
            | auto_lang_core::ast::Stmt::EnumDecl(_) => {
                import_stmts.push(stmt.clone());
            }
            _ => {}
        }
    }

    // Plan 442 A3 (ext link): `use.web` ext imports on the VM render target
    // (adapter-chain loading + platform stubs). See load_ext_imports_for_vm.
    if let Some(file_path) = path {
        let base_dir = std::path::Path::new(file_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let mut ext_widget_decls: Vec<auto_lang_core::ast::ui::WidgetDecl> = Vec::new();
        let mut ext_adapter_paths: Vec<std::path::PathBuf> = Vec::new();
        load_ext_imports_for_vm(
            base_dir,
            &ast,
            &root_decl,
            &all_child_decls,
            &mut visited,
            &mut import_stmts,
            &mut seen_symbols,
            &mut import_session,
            override_scenario,
            &mut import_aliases,
            &mut ext_widget_decls,
            &mut ext_adapter_paths,
        )
        .map_err(|e| format!("ext imports (use.web) failed: {}", e))?;
        // PLAN-051 C4: adapter widget 注册——use.web component 的 VM widget
        // 形态（renderer.vm.at Markdown 纯文本降级）进视图 registry +
        // child_decls（handlers 一并编译进单 VM 模块）。
        for wd in &ext_widget_decls {
            if let Ok(w) = auto_ui_compiler::aura::extract_widget_from_decl(wd) {
                // PLAN-642 R642-F3: `.Tick` handler + `interval` 模型变量的
                // 子件（027/012/025 惯用法——Init 轻量、重 FS/时钟活延后到
                // Tick）在合并 VM 无 tick 源：根件（画廊宿主）无 tick →
                // view.tick_interval 订阅不存在，子件 Tick 永不派发（027
                // 内嵌永久"正在加载..."实证；chart 的 timer{} 块另有 051 C7
                // 通路不受影响）。按 051 C7 timer 条目形态为该子件合成
                // `Tick (every_ms: interval)` 定时条目，走既有 fire_timer
                // 门控派发（handler_<Widget>_Tick 语义不变）。
                let mut wd_owned = wd.clone();
                if let Some(ms) = w.tick_interval {
                    let has_tick_timer = wd_owned
                        .timer
                        .as_ref()
                        .map(|tb| tb.entries.iter().any(|e| e.event.as_str() == "Tick"))
                        .unwrap_or(false);
                    if !has_tick_timer {
                        let mut tb = auto_lang_core::ast::ui::TimerBlock {
                            entries: Vec::new(),
                        };
                        tb.entries.push(auto_lang_core::ast::ui::TimerEntry {
                            event: auto_lang_core::ast::Name::from("Tick"),
                            every_ms: ms.max(16) as u64,
                            when: None,
                        });
                        wd_owned.timer = Some(tb);
                    }
                }
                all_child_decls.push(wd_owned);
                registry.register(w);
            }
        }
        // PLAN-633: demo 适配器的传递子 widget 注册——Demo013Todo 视图里的
        // TodoList 等经适配器自身的 `use` 引入，C4 只注册适配器本体 widget；
        // 不补传递注册则子件 registry miss → fallback Empty（013 内嵌列表区
        // 空白实证）。stores 由下方 632-F1 晚到补转换覆盖，此处只补 widget。
        {
            let mut tw_visited: std::collections::HashSet<std::path::PathBuf> =
                std::collections::HashSet::new();
            for p in &ext_adapter_paths {
                register_transitive_widgets_inner(
                    p,
                    &mut registry,
                    &mut all_child_decls,
                    &mut tw_visited,
                );
            }
        }
        // PLAN-642 R642-F2: use.web 适配器自带 `use { package: X from "dir" }`
        // 的包组件注册。上方的包装载走查（435 P8-6）只覆盖根件——彼时
        // registry 尚无适配器 widget，适配器的包导入永远错过装载且无任何
        // 告警 → 包内 widget 全部缺注册，实例落 builtin 桩（024 chart 内嵌
        // 空画布实证；独立臂根件自带包导入 → 同语料正常出图）。适配器注册
        // 后补一轮包装载：dir 优先相对适配器文件目录（ext_adapter_paths
        // 同源），回退根 base_dir；包组件 fn 模块链按 435 P8-6 同款收集。
        {
            let mut pkg_seen_dirs: std::collections::HashSet<std::path::PathBuf> =
                Default::default();
            let mut pkg_reg = auto_ui_compiler::ui_gen::widget::ComponentRegistry::new();
            for wd in &ext_widget_decls {
                for imp in &wd.ext_imports {
                    if !matches!(imp.kind, auto_lang_core::ast::ui::ExtImportKind::Package) {
                        continue;
                    }
                    let dir = std::path::PathBuf::from(imp.path.as_str());
                    if !pkg_seen_dirs.insert(dir.clone()) {
                        continue;
                    }
                    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
                    for adapter_path in &ext_adapter_paths {
                        if let Some(adapter_dir) = adapter_path.parent() {
                            candidates.push(adapter_dir.join(&dir));
                        }
                    }
                    candidates.push(base_dir.join(&dir));
                    let mut loaded: Option<(
                        Vec<(
                            auto_lang_core::ast::ui::WidgetDecl,
                            auto_ui_compiler::aura::AuraWidget,
                        )>,
                        std::path::PathBuf,
                    )> = None;
                    for cand in &candidates {
                        if cand.is_dir() {
                            if let Ok(pkg) = pkg_reg.load_package(cand, base_dir) {
                                // load_package 返回注册表内缓存引用（&mut self
                                // 借续）——立即抽走所需列表，避免借用跨迭代。
                                loaded = Some((pkg.full_widgets.clone(), cand.clone()));
                                break;
                            }
                        }
                    }
                    match loaded {
                        Some((pkg_full, loaded_dir)) => {
                            for (d, aw) in &pkg_full {
                                all_child_decls.push(d.clone());
                                registry.register(aw.clone());
                            }
                            if let Ok(entries) = std::fs::read_dir(&loaded_dir) {
                                for entry in entries.flatten() {
                                    let p = entry.path();
                                    if !p.extension().map(|e| e == "at").unwrap_or(false) {
                                        continue;
                                    }
                                    crate::collect_module_imports(
                                        &p,
                                        &mut visited,
                                        &mut import_stmts,
                                        &mut seen_symbols,
                                        &mut import_session,
                                        None,
                                    );
                                    if let Ok(code) = std::fs::read_to_string(&p) {
                                        for us in
                                            auto_lang_core::use_scanner::scan_use_statements(&code)
                                        {
                                            let qualifier =
                                                us.module.split('.').last().unwrap_or(&us.module);
                                            for item in &us.items {
                                                import_aliases.insert(
                                                    item.clone(),
                                                    format!("{}.{}", qualifier, item),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        None => log::warn!(
                            "package `{}` load failed (VM adapter) (tried {:?})",
                            imp.path,
                            candidates
                        ),
                    }
                }
            }
        }
        // PLAN-632 F1: 装载顺序缺陷收口——use.web demo 适配器链带来的
        // StoreDecl 在 load_ext_imports_for_vm 内才进 import_stmts，晚于
        // 上方 store→child 转换位（Plan 370 D-GAP-4 / 多 store fix：根
        // AST + 当时的 import_stmts）。晚到的 store（016 的 CalendarStore
        // 经 Demo*.at 适配器链）错过转换 → model 不并入统一根态、handler
        // 不编译（synthesize 的 all_decls 只含 decl+child_decls）→
        // `.store.*` 全落空 + `store.X()` 派发失联。此处按名去重补转换。
        {
            let known: std::collections::HashSet<String> =
                all_child_decls.iter().map(|d| d.name.to_string()).collect();
            for stmt in &import_stmts {
                if let auto_lang_core::ast::Stmt::StoreDecl(store_decl) = stmt {
                    let sname = store_decl.name.to_string();
                    if !known.contains(&sname) {
                        all_child_decls.push(store_decl_as_widget_decl(store_decl));
                    }
                }
            }
        }
    }

    // Plan 340: AUTO_VM_MERGE=0 (i.e. --no-merge) enables API-over-HTTP:
    // cross-module `#[api]` calls are rewritten at codegen time into HTTP
    // requests instead of in-process direct calls. Default (merge) keeps the
    // existing in-process behavior.
    // Plan 057 (ash-gui): AUTO_BACKEND (e.g. http://127.0.0.1:3000) also
    // enables the rewrite — the VM app then talks to a real backend over HTTP
    // (run/complete/history/prompt_context/jobs), not the in-process mock.
    // emit_api_http_call prefers AUTO_BACKEND as the base URL.
    let api_over_http = std::env::var("AUTO_VM_MERGE").as_deref() == Ok("0")
        || std::env::var("AUTO_BACKEND")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
    // PLAN-050 T7 (C5): use.web component 名单注册——builder 图标组件臂的
    // 生产数据源（import_stmts 不含 UseWeb,Plan 442 ext 流单独消费）。每次
    // 装载整体替换。all_child_decls 此处已定型（child 装载循环已收尾）。
    // 来源三路:①根 AST 顶层 use.web（musk app.at 形态）②widget 内嵌
    // ext_imports（decl 携带形态）③子模块文件顶层 use.web——PLAN-051 P2-①
    // 补：chats_view.at/nav_item.at 的模块级声明此前两路均不覆盖 → 图标
    // 组件 unknown fallback → Empty（icon 按钮空白现场）。沿 ext_stubs 的
    // visited 重解析模式（文件小、装载一次）。
    {
        let mut names: Vec<String> = Vec::new();
        for stmt in &ast.stmts {
            if let auto_lang_core::ast::Stmt::UseWeb(entries) = stmt {
                for e in entries {
                    if matches!(e.kind, auto_lang_core::ast::ui::ExtImportKind::Component) {
                        names.extend(e.symbols.iter().map(|n| n.to_string()));
                    }
                }
            }
        }
        for decl in std::iter::once(&root_decl).chain(all_child_decls.iter()) {
            for ext in &decl.ext_imports {
                if matches!(ext.kind, auto_lang_core::ast::ui::ExtImportKind::Component) {
                    names.extend(ext.symbols.iter().map(|n| n.to_string()));
                }
            }
        }
        // ③ 子模块顶层 use.web（visited 为 collect_module_imports 的已装载集）。
        for path in visited.iter() {
            if path.extension().and_then(|e| e.to_str()) != Some("at") {
                continue;
            }
            let Ok(code) = std::fs::read_to_string(path) else {
                continue;
            };
            let code = auto_lang_core::back_prefix::apply(path, code);
            let session = auto_lang_core::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            let Ok(mod_ast) = parser.parse() else {
                continue;
            };
            for stmt in &mod_ast.stmts {
                if let auto_lang_core::ast::Stmt::UseWeb(entries) = stmt {
                    for e in entries {
                        if matches!(e.kind, auto_lang_core::ast::ui::ExtImportKind::Component) {
                            names.extend(e.symbols.iter().map(|n| n.to_string()));
                        }
                    }
                }
            }
        }
        crate::ui::aura_view_builder::register_imported_components(names);
    }
    let mut comp = DynamicComponent::with_registry_and_imports_from_decls(
        &root_decl,
        &all_child_decls,
        &widget,
        registry,
        import_stmts,
        &import_aliases,
        api_over_http,
    )
    .map_err(|e| format!("DynamicComponent init failed: {}", e))?;

    // 3b. Set source path for hot-reload tracking
    if let Some(p) = path {
        comp.set_source_path(p);
    }

    // 3c. Fire .Init lifecycle event before starting Iced loop.
    //     Init is now a regular VM handler (handler_Init), dispatched
    //     unconditionally via call_handler.
    comp.fire_init();

    Ok(comp)
}

/// Plan 459：按 `auto run` 同一管线构造 DynamicComponent（不启动 iced 循环）。
/// 双窗口 demo（examples/ui_dual_app）等宿主复用本入口装配多个 App。
#[cfg(feature = "ui-iced")]
pub fn build_dynamic_component(
    code: &str,
    path: Option<&str>,
) -> AutoResult<crate::ui::dynamic::DynamicComponent> {
    build_dynamic_component_inner(code, path, None)
}

#[cfg(feature = "ui-iced")]
fn run_file_dynamic_ui_inner(
    code: &str,
    path: Option<&str>,
    override_scenario: Option<&auto_lang_core::session::CompilerSession>,
) -> AutoResult<String> {
    use crate::ui::iced::run_dynamic_iced;
    // PLAN-815 T-05（R3-02）：在组件构建**之前**开启退出请求窗口——
    // 初始化/构建期间到达的退出请求（如内部执行臂的 q）不被随后的
    // 快照吸收。
    crate::ui::iced::renderer::host_exit_window_begin();
    // PLAN-779: UI 宿主能力装配——`auto run` UI 入口在组件/VM 构造前注册
    // frame provider 进程默认（覆盖库消费者不经 CLI main 的形态；-q
    // rqhost 客户端在 build 之后分岔，同样受益于此处注册）。
    crate::ui::frame_provider::install_process_default();
    // PLAN-785: clipboard text adapter 同点并入（if-absent 兼容默认）。
    #[cfg(feature = "ui-clipboard")]
    crate::ui::clipboard_provider::install_process_default();
    // PLAN-790: clipboard media adapter 同点并入（if-absent 兼容默认；
    // windows+native-clipboard 双门控——非 Windows 不装配）。
    #[cfg(all(windows, feature = "native-clipboard"))]
    crate::ui::clipboard_media_provider::install_process_default();
    // PLAN-794: scroll host adapter 同点并入（if-absent 兼容默认；ui 单
    // 门控——controller 注册表接线恢复，9900..9905/9922/9923）。
    crate::ui::scroll_host_provider::install_process_default();
    // PLAN-808: terminal adapter 同点并入（if-absent 兼容默认；ui 单门控——
    // 34 native 全族的输入/投影/menu/scroll/window/palette 接线恢复）。
    crate::ui::terminal_provider::install_process_default();
    // PLAN-802: code editor host adapter 同点并入（if-absent 兼容默认；
    // code-editor 单门控——editor 注册表接线恢复，22 native 全族）。
    #[cfg(feature = "code-editor")]
    crate::ui::code_editor_provider::install_process_default();
    // PLAN-804: dialog adapter 同点并入（if-absent 兼容默认；ui-dialog
    // 单门控——host_adapters 落点，2927/2928 parented 策略接线恢复）。
    #[cfg(feature = "ui-dialog")]
    crate::host_adapters::dialog::install_process_default();
    // PLAN-804: dnd adapter 同点并入（if-absent 兼容默认；windows+
    // native-dnd 双门控，本函数在 ui-iced 门内=有效三分，2938 接线恢复）。
    #[cfg(all(windows, feature = "native-dnd"))]
    crate::ui::dnd_host_provider::install_process_default();
    let comp = build_dynamic_component_inner(code, path, override_scenario)?;
    // PLAN-031 T-05：-q vm 轨分岔（env 门）——装载链零改：CWD/主题/后端
    // 序由 run_vm_ui 上游照常执行（组件已按同链装载），仅"自开 OS 窗"
    // 换 rqhost 客户端（build_dynamic_component 后、run_dynamic_iced
    // 前——§5.1 D6 定案位）。env 值 = well-known 管道名（rendezvous
    // 采纳内建在 ClientTarget::Rqhost——不预连不烧一次性管道实例）。
    if let Ok(pipe) = std::env::var("AUTO_RQHOST_PIPE") {
        return crate::ui::desktop_protocol::rqhost::run_vm_rqhost_client(comp, &pipe)
            .map_err(auto_lang_core::error::AutoError::Msg);
    }
    // 4. Run iced (blocks until all windows close; plan-459 daemon semantics)
    run_dynamic_iced(comp).map_err(|e| auto_lang_core::error::AutoError::Msg(format!("{}", e)))
}

/// PLAN-664 U-4（review R1 F-1）：该 use 语句是否为非文件系统模块形态
/// ——P-15/P-16 诊断的假阳性卫。三族：
/// - `auto.*` 内建命名空间（jade `use auto.http` 实测——走 natives
///   通道非文件解析，曾被误报 P-15）；
/// - `.web` 表单（ext 适配器通道另行装载，scanner 把整段收进 module
///   字段——plan632 F2 stderr 实证）；
/// - `use.py`（`is_python_import`——Python 侧符号）。
/// 这些形态在本环解析失败是预期路由，不是静默死面，不得告警。
/// PLAN-814 T-04：唯一消费方为本管线，随 lib.rs 迁入。
fn is_non_fs_module_use(u: &auto_lang_core::use_scanner::UseStatement) -> bool {
    u.is_python_import || u.module.starts_with("auto.") || u.module.starts_with(".web")
}

// =============================================================================
// Plan 446 批一 (C1-2): parse 错误定位换算回归（PLAN-814 T-04 随
// positioned_parse_errors 实装自 auto-lang lib.rs 迁入）
// =============================================================================
#[cfg(test)]
mod plan446_batch1_tests {
    use super::*;

    #[test]
    fn positioned_parse_errors_reports_line_col() {
        // 第 2 行制造语法错误(`let` 缺名) —— 诊断必须带 2:N 行列,而非裸消息。
        let src = "fn ok() int {\n    let = 1\n    return 1\n}\n";
        let mut cs = auto_lang_core::compile::CompileSession::new();
        let mut parser = Parser::new_with_type_store(src, cs.type_store())
            .with_session(auto_lang_core::session::CompilerSession::ui());
        let err = parser.parse().expect_err("broken source must fail");
        let detail = positioned_parse_errors(&err, src);
        assert!(
            detail.contains("at 2:"),
            "expected line-2 position in: {detail}"
        );
    }

    #[test]
    fn positioned_parse_errors_multi_error_cap() {
        // 多错误(MultipleErrors)展开且不超过 20 条。
        let mut src = String::new();
        for i in 0..30 {
            src.push_str(&format!("let broken{i} =\n")); // 每行一个残缺 stmt
        }
        let mut cs = auto_lang_core::compile::CompileSession::new();
        let mut parser = Parser::new_with_type_store(src.as_str(), cs.type_store())
            .with_session(auto_lang_core::session::CompilerSession::ui());
        if let Err(err) = parser.parse() {
            let detail = positioned_parse_errors(&err, &src);
            let count = detail.lines().filter(|l| l.contains("error")).count();
            assert!(count >= 1 && count <= 20, "cap violated: {count}\n{detail}");
        }
    }
}
