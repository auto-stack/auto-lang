//! High-level Transpiler API for AutoUI
//!
//! Provides simple API for transpiling .at files to backend-specific code
//! using the AURA pipeline (Plan 096).
//!
//! # Active API
//! - `transpile_file` — Transpile a .at file to Rust code
//! - `transpile_aura` — Transpile UI source string to Rust
//! - `transpile_vue_aura` — Transpile UI source string to Vue3 SFC

use std::path::Path;

use crate::aura::extract_widget_from_decl;
use crate::parser::Parser;
use crate::session::CompilerSession;
use crate::ui_gen::{BackendGenerator, RustGenerator, VueGenerator};

/// Transpile Auto UI file to Rust code using AURA pipeline
///
/// # Arguments
/// * `input_path` - Path to .at file
/// * `output_path` - Optional path to write .rs file
///
/// # Returns
/// Generated Rust code as string
pub fn transpile_file(
    input_path: impl AsRef<Path>,
    output_path: Option<&str>,
) -> Result<String, String> {
    let input_path = input_path.as_ref();
    let source = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read file {}: {}", input_path.display(), e))?;

    transpile_aura(&source, output_path)
}

/// Transpile UI source code to Rust using AURA pipeline (Plan 096)
///
/// This is the preferred method for transpiling UI components.
/// It uses the new AURA-based architecture without DSL preprocessing.
pub fn transpile_aura(source: &str, output_path: Option<&str>) -> Result<String, String> {
    // Parse with UI scenario
    let session = CompilerSession::ui();
    let mut parser = Parser::from(source).with_session(session);
    let ast = parser.parse().map_err(|e| format!("Failed to parse: {:?}", e))?;

    // Extract and generate
    let mut code = String::new();
    let mut generator = RustGenerator::new();

    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| format!("Failed to extract widget: {}", e))?;
            let widget_code = generator.generate(&aura_widget)
                .map_err(|e| format!("Failed to generate: {}", e))?;
            code.push_str(&widget_code);
            code.push('\n');
        }
    }

    if let Some(output) = output_path {
        std::fs::write(output, &code)
            .map_err(|e| format!("Failed to write file {}: {}", output, e))?;
    }

    Ok(code)
}

/// Transpile UI source code to Vue3 SFC using AURA pipeline (Plan 096)
pub fn transpile_vue_aura(source: &str, output_path: Option<&str>) -> Result<String, String> {
    // Parse with UI scenario
    let session = CompilerSession::ui();
    let mut parser = Parser::from(source).with_session(session);
    let ast = parser.parse().map_err(|e| format!("Failed to parse: {:?}", e))?;

    // Extract and generate
    let mut code = String::new();
    let mut generator = VueGenerator::new();

    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| format!("Failed to extract widget: {}", e))?;
            let widget_code = generator.generate(&aura_widget)
                .map_err(|e| format!("Failed to generate: {}", e))?;
            code.push_str(&widget_code);
            code.push('\n');
        }
    }

    if let Some(output) = output_path {
        std::fs::write(output, &code)
            .map_err(|e| format!("Failed to write file {}: {}", output, e))?;
    }

    Ok(code)
}

// ============================================================================
// Plan 043 stream phase: resolve streaming endpoints from back/api.at
// ============================================================================

/// Resolve streaming API endpoints (`#[api] fn` returning `~Stream<T>`) from the
/// project's `back/api.at`, so the store composable can wire type-driven SSE.
/// (Plan 043 stream phase.) Uses a targeted regex scan (robust against
/// `use types:`-style module references that defeat full AST parsing). Returns
/// an empty vec if api.at is absent or has no stream endpoints.
///
/// Plan musk-022 Phase 1: for each endpoint, also resolve the discriminator
/// field name and variant→action map from the inner type `T`'s definition.
/// `T` is expected to be a `#[serde(tag = "X", rename_all = "snake_case")] pub tag`
/// (externally-tagged enum) declared in the same file. When found, each variant
/// name is converted to its snake_case wire form (matching serde's rename) and
/// paired with its PascalCase action name. When `T` is not resolvable here,
/// `variants` stays empty and the codegen falls back to the legacy
/// `command_output`/`command_result` dispatch (backward compatible).
pub fn resolve_stream_endpoints_for_project(root_dir: &str) -> Vec<crate::aura::StreamEndpoint> {
    let root = std::path::Path::new(root_dir);
    // Plan 061:统一契约定位(本地 back/ 或 pac.at back.project 外部后端)
    let api_file = match crate::config::resolve_back_api(root) {
        Some(f) => f,
        None => return Vec::new(),
    };
    let content = match std::fs::read_to_string(&api_file) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Match: #[api(method = "M", path = "/p")] pub fn name(...) ~Stream<T> {
    // The path attribute and the `~Stream<T>` return type are both required.
    // Robust against multi-line annotations and extra whitespace.
    let Ok(re) = regex::Regex::new(
        r#"(?s)#\[api\([^]]*path\s*=\s*"([^"]+)"[^]]*\)\]\s*pub\s+fn\s+(\w+)\s*\([^)]*\)\s*~?Stream<([^>]+)>"#,
    ) else { return Vec::new(); };

    re.captures_iter(&content)
        .filter_map(|cap| {
            let path = cap.get(1)?.as_str().to_string();
            let fn_name = cap.get(2)?.as_str().to_string();
            let item_type = cap.get(3)?.as_str().trim().to_string();
            let (discriminator, variants) = resolve_stream_variants(&content, &item_type);
            Some(crate::aura::StreamEndpoint {
                path,
                fn_name,
                item_type,
                discriminator,
                variants,
            })
        })
        .collect()
}

/// Plan musk-022 Phase 1: resolve the SSE discriminator field name and the
/// variant→action map for a stream's inner type `T`. Scans `content` for a
/// `#[serde(tag = "X", ...)] pub tag T { V1, V2 {..}, ... }` declaration
/// (externally-tagged enum). Returns `("event", [])` (legacy defaults) when `T`
/// is not found or is not a `pub tag` with serde tag annotation.
fn resolve_stream_variants(content: &str, type_name: &str) -> (String, Vec<(String, String)>) {
    // Step 1: locate a `pub tag <type_name> {` or `pub enum <type_name> {`
    // declaration and capture any preceding `#[serde(...)]` attribute. We only
    // match up to the opening brace here (not the body), because variant bodies
    // like `Delta { text str }` contain nested braces that defeat a naive
    // `[^}]*` regex. Step 2 braces-balances to extract the full body.
    let head_re = match regex::Regex::new(&format!(
        r#"(?s)(#\[serde\(([^)]*)\)\]\s*)?(?:pub\s+)?(?:tag|enum)\s+{}\s*\{{"#,
        regex_escape(type_name)
    )) {
        Ok(r) => r,
        Err(_) => return ("event".to_string(), Vec::new()),
    };
    let Some(head_match) = head_re.captures(content) else {
        return ("event".to_string(), Vec::new());
    };
    let serde_args = head_match.get(2).map(|m| m.as_str()).unwrap_or("");

    // Step 2: from the opening `{` (end of the head match), braces-balance to
    // find the matching closing `}`. Nested variant bodies are handled because
    // we count every `{` and `}`.
    let body_start = head_match.get(0).unwrap().end();
    let body = match balance_braces(&content[body_start..]) {
        Some(b) => b,
        None => return ("event".to_string(), Vec::new()),
    };

    // Discriminator: `tag = "X"` in #[serde(...)]. Default "event" when absent.
    let discriminator = regex::Regex::new(r#"tag\s*=\s*"([^"]+)""#)
        .ok()
        .and_then(|re| re.captures(serde_args))
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| "event".to_string());

    // rename_all: when "snake_case", variant names get snake_cased on the wire.
    let snake = serde_args.contains("snake_case");

    // Variants: each line in the body looks like `VariantName { ... }` or
    // `VariantName` or `VariantName Type`. Take the leading PascalCase ident.
    let variant_re = match regex::Regex::new(r"([A-Z][A-Za-z0-9_]*)") {
        Ok(r) => r,
        Err(_) => return (discriminator, Vec::new()),
    };
    let mut variants = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if let Some(m) = variant_re.captures(line) {
            let name = m.get(1).unwrap().as_str().to_string();
            let wire = if snake { to_snake_case(&name) } else { name.clone() };
            variants.push((wire, name));
        }
    }
    (discriminator, variants)
}

/// Plan musk-022 Phase 1: braces-balance helper. Given a slice starting just
/// after an opening `{`, return the substring up to (not including) the matching
/// closing `}`, counting nested braces. Returns None if unbalanced.
fn balance_braces(s: &str) -> Option<&str> {
    let mut depth: i32 = 1;
    let bytes = s.as_bytes();
    let mut end = 0;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(&s[..end]);
            }
        }
        end += 1;
        i += 1;
    }
    None
}

/// Escape regex metacharacters in a type name for safe interpolation.
fn regex_escape(s: &str) -> String {
    s.chars()
        .map(|c| {
            if r"\.+*?()|[]{}^$".contains(c) {
                format!("\\{}", c)
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// Convert a PascalCase identifier to snake_case (e.g. `ToolCall` → `tool_call`,
/// `HTTPError` → `http_error`). Matches serde's `rename_all = "snake_case"`.
fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                // Insert underscore before an uppercase letter when preceded by a
                // lowercase letter or digit, OR before the last uppercase in a
                // run followed by a lowercase (e.g. "HTTPError" → "http_error").
                let prev = chars[i - 1];
                let next_lower = chars.get(i + 1).map(|n| n.is_ascii_lowercase()).unwrap_or(false);
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

// ============================================================================
// Plan 361 §3: generate_component_from_file — 统一生成入口
// ============================================================================

use std::collections::HashSet;

/// Options for `generate_component_from_file`.
///
/// All fields are optional overrides. When `None`, the function auto-detects
/// api_imports / store_deps / sub_widgets from the .at source.
#[derive(Debug, Default, Clone)]
pub struct ComponentGenOptions {
    /// Known sub-widget names (avoids shadcn-vue name collisions).
    pub sub_widgets: Option<Vec<String>>,
    /// PLAN-037 T5: cross-file sub-widget model-var map (name -> channels),
    /// collected by the workspace prescan (auto-man from_workspace). Merged
    /// with the same-file widgets' state vars inside generate_component_from_file.
    pub sub_widget_models: Option<std::collections::HashMap<String, Vec<String>>>,
    /// Override API imports. When `None`, auto-detected from `use back.api:`.
    pub api_imports_override: Option<Vec<String>>,
    /// Override store dependencies. When `None`, auto-detected from `use store:`.
    pub store_deps_override: Option<Vec<String>>,
    /// Root directory for API import validation (auto-man uses this).
    pub root_dir_for_validation: Option<std::path::PathBuf>,
    /// Streaming API endpoints (`#[api] fn` returning `~Stream<T>`) discovered
    /// from the project's `back/api.at`. Populated by the build driver; stamped
    /// onto each store so its composable can wire type-driven SSE. (Plan 043
    /// stream phase.) When `None`, no SSE wiring is generated.
    pub stream_endpoints: Option<Vec<crate::aura::StreamEndpoint>>,
    /// shadcn-vue widget mapping toggle (pac.at `shadcn:` field). `None` /
    /// `Some(true)` = Shadcn mode (current default, unchanged); `Some(false)`
    /// = Plain mode: widgets render as native HTML elements (`button` stays
    /// `<button>`) and no `@/components/ui/*` imports are emitted.
    pub shadcn: Option<bool>,
    /// Default Tailwind class injection toggle (pac.at `default_classes:`
    /// field). `None` / `Some(true)` = on (current default, unchanged):
    /// `extract_classes` injects the doc-theme defaults (`text` →
    /// `text-muted-foreground leading-7`, `h1` → `text-3xl ...`, `button` →
    /// `px-4 py-2 rounded`, ...). `Some(false)` = off: skip every default
    /// class EXCEPT the structural layout primitives (`row`/`col`/`column`/
    /// `grid`/`scroll`/`center`/`container`/`square`) — those stay so the
    /// layout doesn't collapse. For pixel-exact replica projects that bring
    /// their own styling.
    pub default_classes: Option<bool>,
}

/// Result of generating a component from an .at file.
#[derive(Debug, Default, Clone)]
pub struct GeneratedComponent {
    /// Vue SFC code for the first widget (used as App.vue).
    pub vue_code: String,
    /// All widget SFC codes (one per widget declaration).
    pub all_widget_codes: Vec<(String, String)>, // (widget_name, code)
    /// Store composable files (filename, code).
    pub store_composables: Vec<(String, String)>,
    /// Detected API imports from `use back.api:`.
    pub detected_api_imports: Vec<String>,
    /// Detected store dependencies from `use store:`.
    pub detected_store_deps: Vec<String>,
    /// All extracted AURA widgets.
    pub widgets: Vec<crate::aura::AuraWidget>,
    /// Validation warnings from the post-generation check.
    pub validation_warnings: Vec<crate::ui_gen::validators::ValidationWarning>,
}

/// Unified single-entry-point for "parse .at → extract imports/stores →
/// register view fragments → extract widgets → generate SFC → validate".
///
/// This replaces the duplicated logic in:
/// - `ui_build_shadcn_with_widgets` (lib.rs)
/// - `ui_build_shadcn_with_sub_widgets` (lib.rs)
/// - `compile_at_to_vue` (auto-man/vue.rs)
/// - `compile_at_to_vue_with_sub_widgets` (auto-man/vue.rs)
///
/// All four callers should migrate to this function (Plan 361 §3).
pub fn generate_component_from_file(
    at_path: &std::path::Path,
    opts: ComponentGenOptions,
) -> Result<GeneratedComponent, String> {
    use crate::session::CompilerSession;
    use crate::ui_gen::{BackendGenerator, VueGenerator, VueMode};
    use crate::aura::extract_widget_from_decl;
    use crate::aura::extract_store_from_decl;

    let code = std::fs::read_to_string(at_path)
        .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

    // Parse with UI scenario
    let session = CompilerSession::ui().with_backend("vue");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);
    let ast = parser.parse()
        .map_err(|e| format!("Parse error in {}: {:?}", at_path.display(), e))?;

    // Auto-detect or use overrides
    let api_imports = opts.api_imports_override.unwrap_or_else(|| {
        extract_api_imports_from_ast(&ast)
    });
    let store_deps = opts.store_deps_override.unwrap_or_else(|| {
        extract_store_imports_from_ast(&ast)
    });
    let sub_widgets = opts.sub_widgets.unwrap_or_default();

    // Extract store declarations → generate composables
    // Clear thread-local first to avoid cross-test contamination
    crate::STORE_EXTRA_FILES.with(|cell| {
        cell.borrow_mut().clear();
    });
    let mut store_composables: Vec<(String, String)> = Vec::new();
    let mut store_warnings: Vec<crate::ui_gen::validators::ValidationWarning> = Vec::new();
    // Plan 367 P2-4: module-level plain functions in the store file (siblings of
    // the `store { ... }` block, e.g. `fn format_git_label` helpers). The vue
    // codegen emits them into the composable so handlers can call them by name.
    let module_fns: Vec<crate::aura::AuraModuleFn> = ast.stmts.iter()
        .filter_map(|stmt| {
            if let crate::ast::Stmt::Fn(fn_decl) = stmt {
                crate::aura::extract_module_fn(fn_decl)
            } else {
                None
            }
        })
        .collect();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::StoreDecl(store_decl) = stmt {
            let mut store = extract_store_from_decl(store_decl)
                .map_err(|e| e.to_string())?;
            store.api_imports = api_imports.clone();
            store.stream_endpoints = opts.stream_endpoints.clone().unwrap_or_default();
            store.module_fns = module_fns.clone();
            let (composable, warnings) = VueGenerator::generate_store_composable_full(&store);
            store_warnings.extend(warnings);
            let filename = format!("stores/use{}Store.ts", store.name);
            store_composables.push((filename, composable));
            // Also stash via thread-local for callers that use STORE_EXTRA_FILES
            crate::STORE_EXTRA_FILES.with(|cell| {
                cell.borrow_mut().push((
                    format!("stores/use{}Store.ts", store.name),
                    VueGenerator::generate_store_composable(&store),
                ));
            });
        }
    }

    // Register view fn fragments before widget extraction (Plan 367 P2-3)
    crate::aura::extract::clear_view_fragments();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::ViewFragmentDecl(frag) = stmt {
            crate::aura::extract::register_view_fragment(frag);
        }
    }

    // Extract widgets
    let mut widgets: Vec<crate::aura::AuraWidget> = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let mut aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            aura_widget.api_imports = api_imports.clone();
            widgets.push(aura_widget);
        }
    }

    // Plan 408 collected `component fn` declarations here (fragment → SFC
    // synthesis + sub-widget registration). Plan 425: `component fn` sugars
    // to a WidgetDecl at parse time, so the main WidgetDecl loop above
    // already extracts them and the same-file merge below registers their
    // names — this second track is deleted.

    // PLAN-037 T6: file-level `use.web` statements attach their entries to
    // EVERY widget declared in the file (the import lands in each generated
    // SFC whose source references the symbol).
    let mut web_ext_imports: Vec<crate::ast::ui::ExtImport> = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::UseWeb(entries) = stmt {
            web_ext_imports.extend(entries.iter().cloned());
        }
    }
    if !web_ext_imports.is_empty() {
        for w in widgets.iter_mut() {
            w.ext_imports.extend(web_ext_imports.iter().cloned());
        }
    }

    if widgets.is_empty() && store_composables.is_empty() {
        return Err("No widget or store declarations found in input file".into());
    }

    // Plan 408: merge synthesized component fn names into sub_widgets so every
    // widget/component sees them as referenceable components.
    // Plan 425: same-file WidgetDecl names enter the list too — `component fn`
    // now sugars to WidgetDecl, and same-file widget references take the same
    // component path (`<Name/>` + `@/components/Name.vue` import) the fragment
    // track used to provide.
    let mut all_sub_widgets: Vec<String> = sub_widgets.clone();
    for w in &widgets {
        if !all_sub_widgets.contains(&w.name) {
            all_sub_widgets.push(w.name.clone());
        }
    }

    // PLAN-037 T5: sub-widget model-var map = cross-file (opts) + same-file
    // widgets' state vars. Powers call-site model addressing (v-model).
    let mut sub_widget_models: std::collections::HashMap<String, Vec<String>> =
        opts.sub_widget_models.clone().unwrap_or_default();
    for w in &widgets {
        let entry = sub_widget_models.entry(w.name.clone()).or_default();
        for sv in &w.state_vars {
            if !entry.contains(&sv.name) {
                entry.push(sv.name.clone());
            }
        }
    }

    // Generate SFC for each widget
    let mut all_widget_codes: Vec<(String, String)> = Vec::new();
    let mut all_validation_warnings: Vec<crate::ui_gen::validators::ValidationWarning> = store_warnings;

    // pac.at `shadcn: off` (Plan 013): Plain mode keeps native HTML elements
    // and emits no `@/components/ui/*` imports; default stays Shadcn.
    let vue_mode = if opts.shadcn.unwrap_or(true) {
        VueMode::Shadcn
    } else {
        VueMode::Plain
    };
    // pac.at `default_classes: off` (Plan 014): skip the doc-theme default
    // Tailwind classes for non-layout-primitive tags; default stays on.
    let default_classes = opts.default_classes.unwrap_or(true);

    for widget in &widgets {
        let mut gen = VueGenerator::new()
            .with_mode(vue_mode)
            .with_default_classes(default_classes)
            .with_store_deps(store_deps.clone())
            .with_sub_widgets(all_sub_widgets.clone())
            .with_sub_widget_models(sub_widget_models.clone());
        if !api_imports.is_empty() {
            gen = gen.with_project_api_functions(api_imports.clone());
        }

        let widget_code = gen.generate(widget)
            .map_err(|e| format!("Failed to generate {}: {}", widget.name, e))?;
        all_widget_codes.push((widget.name.clone(), widget_code));

        // Collect validation warnings from this widget's generation
        for w in &gen.last_validation_warnings {
            all_validation_warnings.push(w.clone());
        }
    }

    let vue_code = all_widget_codes
        .first()
        .map(|(_, code)| code.clone())
        .unwrap_or_default();

    // Plan 012 Batch A: --strict (`auto build --strict`) escalates any
    // Warning/Error-severity validation warning to a hard build failure.
    // Info-severity warnings stay advisory.
    if crate::ui_gen::validators::strict_enabled()
        && crate::ui_gen::validators::has_blocking_warnings(&all_validation_warnings)
    {
        return Err(format!(
            "codegen validation failed (strict mode) for {}:\n{}",
            at_path.display(),
            crate::ui_gen::validators::format_warnings(&all_validation_warnings)
        ));
    }

    Ok(GeneratedComponent {
        vue_code,
        all_widget_codes,
        store_composables,
        detected_api_imports: api_imports,
        detected_store_deps: store_deps,
        widgets,
        validation_warnings: all_validation_warnings,
    })
}

// Re-export helper functions from super (used by generate_component_from_file)
fn extract_api_imports_from_ast(ast: &crate::ast::Code) -> Vec<String> {
    let mut imports = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::Use(ref use_stmt) = stmt {
            if is_api_use_stmt(use_stmt) {
                imports.extend(use_stmt.items.iter().map(|s| s.as_str().to_string()));
            }
        }
    }
    imports
}

/// Check if a `use` statement targets `back.api` (same logic as lib.rs::is_api_use_stmt).
fn is_api_use_stmt(use_stmt: &crate::ast::Use) -> bool {
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

fn extract_store_imports_from_ast(ast: &crate::ast::Code) -> Vec<String> {
    let mut deps = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::Use(use_stmt) = stmt {
            let is_store = use_stmt.paths.len() == 1
                && (use_stmt.paths[0].as_str() == "store"
                    || use_stmt.paths[0].as_str().contains("store"))
                || use_stmt.module_path.as_ref().map_or(false, |mp| {
                    mp.display() == "store" || mp.display().contains("store")
                });
            if is_store {
                deps.extend(use_stmt.items.iter().map(|s| s.as_str().to_string()));
            }
        }
    }
    deps
}

#[cfg(test)]
mod tests {
    use super::transpile_vue_aura;

    #[test]
    fn test_transpile_placeholder() {
        // Active API smoke test — real tests require .at fixture files
        assert!(true);
    }

    /// DOM escape hatch, end to end (plain Tailwind mode):
    /// `ref: "menuEl"` in the view → `ref="menuEl"` template attribute +
    /// `const menuEl = ref<HTMLElement | null>(null)` script declaration;
    /// `.menuEl.xxx` in `on` handlers → `menuEl.value!.xxx`;
    /// `document.*` / `window.*` pass through unchanged.
    #[test]
    fn test_dom_escape_hatch_plain_mode() {
        let src = r#"
widget App {
    msg Msg {
        Open
        Scrolled
    }
    model {
        var menu_left int = 0
    }
    view {
        col {
            button "open" {
                ref: "triggerEl",
                onclick: .Open
            }
            col {
                ref: "menuEl"
                text "menu"
            }
            col {
                ref: "scrollEl",
                onwheel: .Scrolled($event)
                text "content"
            }
        }
    }
    on {
        .Open -> {
            let r = .triggerEl.getBoundingClientRect()
            .menu_left = r.left
            let w = window.innerWidth
            let h = window.innerHeight
            let ae = document.activeElement
        }

        .Scrolled(e) -> {
            .scrollEl.scrollTop = .scrollEl.scrollTop + e.deltaY
            let sh = .scrollEl.scrollHeight
            let ch = .scrollEl.clientHeight
            let q = .menuEl.querySelector(".item")
        }
    }
}
"#;
        let out = transpile_vue_aura(src, None).expect("dom escape hatch must generate");
        // Template ref attributes
        assert!(out.contains("ref=\"triggerEl\""), "sfc:\n{out}");
        assert!(out.contains("ref=\"menuEl\""), "sfc:\n{out}");
        assert!(out.contains("ref=\"scrollEl\""), "sfc:\n{out}");
        // Script declarations
        assert!(out.contains("const triggerEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        assert!(out.contains("const menuEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        assert!(out.contains("const scrollEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        // Handler body: ref access maps onto the DOM element
        assert!(out.contains("triggerEl.value!.getBoundingClientRect()"), "sfc:\n{out}");
        assert!(out.contains("scrollEl.value!.scrollTop = scrollEl.value!.scrollTop + e.deltaY"), "sfc:\n{out}");
        assert!(out.contains("scrollEl.value!.scrollHeight"), "sfc:\n{out}");
        assert!(out.contains("scrollEl.value!.clientHeight"), "sfc:\n{out}");
        assert!(out.contains("menuEl.value!.querySelector('.item')"), "sfc:\n{out}");
        // document/window pass-through
        assert!(out.contains("window.innerWidth"), "sfc:\n{out}");
        assert!(out.contains("window.innerHeight"), "sfc:\n{out}");
        assert!(out.contains("document.activeElement"), "sfc:\n{out}");
    }

    /// Same escape hatch in shadcn mode: layout elements (col) and buttons
    /// are mapped through the shadcn branch, which whitelists props — the
    /// `ref` prop must survive there too.
    #[test]
    fn test_dom_escape_hatch_shadcn_mode() {
        let src = r#"
widget App {
    msg Msg {
        Open
    }
    model {
        var menu_left int = 0
    }
    view {
        col {
            button "open" {
                ref: "triggerEl",
                onclick: .Open
            }
            col {
                ref: "menuEl"
                text "menu"
            }
        }
    }
    on {
        .Open -> {
            let r = .triggerEl.getBoundingClientRect()
            .menu_left = r.left
        }
    }
}
"#;
        use crate::session::CompilerSession;
        use crate::ui_gen::{BackendGenerator, VueGenerator, VueMode};
        let session = CompilerSession::ui().with_backend("vue");
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut generated = None;
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(w) = stmt {
                let widget = crate::aura::extract_widget_from_decl(w).expect("extract");
                let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
                generated = Some(gen.generate(&widget).expect("generate"));
            }
        }
        let out = generated.expect("widget");
        assert!(out.contains("ref=\"triggerEl\""), "sfc:\n{out}");
        assert!(out.contains("ref=\"menuEl\""), "sfc:\n{out}");
        assert!(out.contains("const triggerEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        assert!(out.contains("const menuEl = ref<HTMLElement | null>(null)"), "sfc:\n{out}");
        assert!(out.contains("triggerEl.value!.getBoundingClientRect()"), "sfc:\n{out}");
    }

    /// Plan 356 follow-up #2: a reserved-keyword identifier (e.g. `tag`,
    /// lexed as TokenKind::Tag) used inside an `if` condition — specifically
    /// as the right-hand side of a comparison in a `style: if` attribute:
    ///   style: if .active_tag == tag { ... } else { ... }
    ///
    /// Previously the expression atom parser matched only TokenKind::Ident,
    /// so `tag` here produced "Expected term, got RBrace" (the real 015-notes
    /// sidebar's offset-8873 error). Must now parse + generate.
    #[test]
    fn test_soft_keyword_in_if_condition() {
        let src = r#"
widget W(active_tag: str) {
    view {
        button {
            style: if .active_tag == tag {
                "bg-blue-500"
            } else {
                "bg-gray-100"
            }
        }
    }
}
"#;
        let out = transpile_vue_aura(src, None).expect("soft-kw in if-condition must generate");
        assert!(out.contains(":class"), "expected :class binding in:\n{out}");
    }

    /// Plan 356 follow-up #2: a reserved-keyword identifier as a comparison
    /// operand in an `if` condition (the shape the real sidebar uses:
    /// `if .active_tag == tag`). Covers several soft keywords.
    ///
    /// NOTE: a bare soft keyword as the *whole* condition (`if tag {}`) is a
    /// separate, narrower pre-existing issue with `style:`-attribute parsing
    /// and is intentionally not covered here — the sidebar doesn't use it.
    #[test]
    fn test_soft_keyword_as_condition_value() {
        for kw in ["tag", "type", "move", "copy", "super"] {
            let src = format!(
                "widget W(x: str) {{ view {{ button {{ style: if .x == {kw} {{ \"a\" }} else {{ \"b\" }} }} }} }}",
            );
            transpile_vue_aura(&src, None)
                .unwrap_or_else(|e| panic!("compare-operand {kw}: {e}"));
        }
    }

    /// Plan 356 follow-up: a view `for`-loop over an `ident.field` iterable
    /// (e.g. `for tag in note.tags`) must parse + generate. Previously
    /// `parse_view_for_loop` only accepted `.field`, numeric ranges, or a bare
    /// ident, so `note.tags` was read as just `note`, leaving `.tags` to break
    /// the rest of the view ("Expected term, got RBrace").
    #[test]
    fn test_view_for_loop_ident_field_iterable() {
        let src = r#"
widget Tags {
    view {
        col {
            for note in .notes {
                for tag in note.tags {
                    button { text tag }
                }
            }
        }
    }
}
"#;
        let out = transpile_vue_aura(src, None).expect("ident.field iterable must generate");
        // Outer loop iterates .notes; inner loop iterates note.tags.
        assert!(out.matches("v-for").count() == 2, "expected 2 v-for loops in:\n{out}");
    }

    /// Plan 356 follow-up: `ident.field.sub` chains as an iterable must also
    /// work (symmetric with `.field.sub` chained access).
    #[test]
    fn test_view_for_loop_ident_field_chain_iterable() {
        let src = r#"
widget W {
    view {
        for x in store.items {
            button { text "x" }
        }
    }
}
"#;
        let out = transpile_vue_aura(src, None).expect("ident.field.chain iterable must generate");
        assert!(out.contains("v-for"), "missing v-for in:\n{out}");
    }

    /// Plan 356 end-to-end: the real 015-notes sidebar (commit 50307d51, 200
    /// lines) that originally OOM'd. This is the full integration guard — it
    /// exercises every trigger fixed across the three Plan 356 commits:
    ///   - `onclick: .SelectTag(tag)`        (reserved-kw loop var → OOM)
    ///   - `for tag in note.tags`            (ident.field iterable)
    ///   - `style: if .active_tag == tag {}` (reserved-kw in if-condition)
    /// It must parse AND generate end to end.
    #[test]
    fn test_plan356_real_sidebar_generates() {
        let src = include_str!("../../tests/fixtures/plan356_oom_sidebar.at");
        let out = transpile_vue_aura(src, None).expect("real sidebar must generate");
        assert!(out.len() < 100_000, "output too large: {} bytes", out.len());
        assert!(out.contains("v-for"), "expected v-for in sidebar output");
        assert!(out.contains("SelectTag"), "expected SelectTag handler in sidebar output");
    }

    /// Plan 356 regression: the minimal OOM trigger. A `for`-loop whose body
    /// has an event handler taking the loop variable as an argument, where the
    /// loop variable is a reserved-keyword identifier (`tag` → TokenKind::Tag).
    ///
    /// Before the fix this exhausted memory (parse_event_arg ignored the `Tag`
    /// token and the handler arg loop spun forever). It must now parse +
    /// generate a sane SFC inline (no thread, no timeout needed).
    #[test]
    fn test_plan356_oom_regression_loop_var_as_handler_arg() {
        let src = include_str!("../../tests/fixtures/plan356_minimal_oom.at");
        let out = transpile_vue_aura(src, None).expect("Plan 356 trigger must generate");
        assert!(out.contains("v-for"), "expected v-for in:\n{out}");
        assert!(out.contains("SelectTag"), "expected handler binding in:\n{out}");
    }

    /// Plan 356: the soft-keyword-as-identifier fix applies to any reserved
    /// keyword, not just `tag`. Verify a few others (`type`, `move`) used as a
    /// loop variable passed to a handler also generate cleanly.
    #[test]
    fn test_plan356_soft_keyword_loop_var() {
        for kw in ["tag", "type", "move", "copy", "super"] {
            let src = format!(
                r#"
widget W(active: bool) {{
    msg Msg {{ Go(str) }}
    view {{
        col {{
            for {kw} in .items {{
                button {{
                    text {kw}
                    onclick: .Go({kw})
                }}
            }}
        }}
    }}
    on {{ .Go(x) -> {{ }} }}
}}
"#,
            );
            let out =
                transpile_vue_aura(&src, None).unwrap_or_else(|e| panic!("{kw}: {e}"));
            assert!(out.contains("v-for"), "{kw}: missing v-for in:\n{out}");
            assert!(out.contains("Go"), "{kw}: missing handler in:\n{out}");
        }
    }

    /// Plan 356 control: a non-keyword loop variable name worked before and
    /// must still work (guards against the fix over-reaching).
    #[test]
    fn test_plan356_normal_loop_var_still_works() {
        let src = r#"
widget W(active: bool) {
    msg Msg { Go(str) }
    view {
        col {
            for item in .items {
                button {
                    text item
                    onclick: .Go(item)
                }
            }
        }
    }
    on { .Go(x) -> { } }
}
"#;
        let out = transpile_vue_aura(src, None).expect("normal loop var must generate");
        assert!(out.contains("v-for"));
    }

    /// Plan 358 D1 stress guard: the `for` + `style:if` + `msg`/`on`
    /// combination that OOM'd (1.7GB+) on the real 015-notes sidebar. The
    /// root cause was a `format!("{:?}", widget.view_tree)` dark-mode scan in
    /// `generate_sfc` whose Debug output exploded on large trees with
    /// `Expr::If` nodes (fixed in a86c183c by removing the Debug format).
    ///
    /// This test generates a widget with 10 for-loops x 60 `style:if`
    /// branches (600 if-nodes) plus `msg`/`on`, and asserts generation
    /// completes well under the Plan 358 budget (< 5s; measured ~0.1s) so a
    /// reintroduced tree-wide Debug format — or any other superlinear blowup
    /// on this pattern — fails loudly instead of OOMing a build.
    #[test]
    fn test_plan358_d1_for_style_if_msg_on_stress() {
        let mut src = String::from(
            "widget NavTree(active: bool) {\n    msg Msg { SelectTag(str) }\n    view {\n        col {\n",
        );
        for l in 0..10 {
            src.push_str(&format!(
                "            for tag{l} in .items {{\n                button {{\n                    text tag{l}\n                    onclick: .SelectTag(tag{l})\n"
            ));
            for i in 0..60 {
                src.push_str(&format!(
                    "                    style: if tag{l} == \"t{i}\" {{ \"l{l}c{i}a\" }} else if .active {{ \"l{l}c{i}b\" }} else {{ \"l{l}c{i}c\" }}\n"
                ));
            }
            src.push_str("                }\n            }\n");
        }
        src.push_str("        }\n    }\n    on { .SelectTag(t) -> { } }\n}\n");

        let start = std::time::Instant::now();
        let out = transpile_vue_aura(&src, None).expect("D1 trigger pattern must generate");
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 5,
            "generation took {:?}, over the 5s Plan 358 budget (possible D1 regression)",
            elapsed
        );
        assert_eq!(out.matches("v-for").count(), 10, "expected 10 v-for loops");
        assert!(out.contains("SelectTag"), "expected handler binding");
    }

    // --- Plan 012 Batch A: --strict escalation ----------------------------

    /// Resets the process-wide strict flag on drop, so a failing test can't
    /// leave strict mode on for parallel tests.
    struct StrictGuard;
    impl StrictGuard {
        fn on() -> Self {
            crate::ui_gen::validators::set_strict(true);
            StrictGuard
        }
    }
    impl Drop for StrictGuard {
        fn drop(&mut self) {
            crate::ui_gen::validators::set_strict(false);
        }
    }

    /// Strict mode is a process-wide flag and cargo runs tests in parallel,
    /// so both strict assertions live in ONE test to avoid toggling races.
    #[test]
    fn test_strict_mode_escalation() {
        use super::{generate_component_from_file, ComponentGenOptions};

        let comma_src = r#"
widget StrictCommaProbe {
    view {
        col {
            button "a"
            ,
            button "b",
        }
    }
}
"#;
        let comma_path = std::env::temp_dir().join("plan012_strict_comma_probe.at");
        std::fs::write(&comma_path, comma_src).expect("write probe .at");

        // Non-strict: R008 is advisory, generation succeeds.
        crate::ui_gen::validators::set_strict(false);
        let ok = generate_component_from_file(&comma_path, ComponentGenOptions::default());
        assert!(ok.is_ok(), "non-strict build must succeed: {:?}", ok.err());

        // Strict: the R008 warning becomes a hard build failure.
        let _guard = StrictGuard::on();
        let err = generate_component_from_file(&comma_path, ComponentGenOptions::default());
        let msg = err.expect_err("strict build must fail on blocking warnings");
        assert!(
            msg.contains("strict mode") && msg.contains("R008"),
            "error should name strict mode and the rule: {msg}"
        );

        // Strict: R013 (unsupported bound-position expr, `??` has no
        // bound-value arm) also escalates to a hard build failure.
        let r013_src = r#"
widget StrictR013Probe {
    model {
        var a str = "x"
        var b str = "y"
    }
    view {
        col {
            span {
                style: { "line-through": .a ?? .b }
                text "label"
            }
        }
    }
}
"#;
        let r013_path = std::env::temp_dir().join("plan012_strict_r013_probe.at");
        std::fs::write(&r013_path, r013_src).expect("write probe .at");
        let err = generate_component_from_file(&r013_path, ComponentGenOptions::default());
        let msg = err.expect_err("strict build must fail on R013");
        assert!(
            msg.contains("strict mode") && msg.contains("R013"),
            "error should name strict mode and R013: {msg}"
        );

        // Strict + Info only: `.remove` on an ext-composable facade passes
        // through with an R010 Info note — advisory, must NOT fail the build.
        // (A composable facade rather than `store.*`, to avoid tripping
        // R002's store-without-import Error.)
        let info_src = r#"
widget StrictInfoProbe {
    use { composable: useRecentFilesStore from "src/front/composables/useRecentFilesStore.ts" }
    msg Msg { ExtDel(int) }
    view {
        col {
            button "xdel" { onclick: .ExtDel(0) }
        }
    }
    on {
        .ExtDel(i) -> { .recentFilesStore.remove(i) }
    }
}
"#;
        let info_path = std::env::temp_dir().join("plan012_strict_info_probe.at");
        std::fs::write(&info_path, info_src).expect("write probe .at");
        let res = generate_component_from_file(&info_path, ComponentGenOptions::default());
        drop(_guard);
        assert!(
            res.is_ok(),
            "Info-severity notes must not block a strict build: {:?}",
            res.err()
        );

        let _ = std::fs::remove_file(&comma_path);
        let _ = std::fs::remove_file(&info_path);
    }

    // --- Plan 012 Batch B: store codegen correctness -------------------------

    /// Gap 9a: a single .at file declaring TWO stores must emit BOTH store
    /// composables through the real parse path (return value AND the
    /// STORE_EXTRA_FILES thread-local used by legacy callers).
    #[test]
    fn test_multi_store_single_file_emission() {
        use super::{generate_component_from_file, ComponentGenOptions};

        let src = r#"
store AlphaStore {
    model {
        var items []str = []
    }
    msg Msg { Touch }
    on {
        .Touch -> { }
    }
}

store BetaStore {
    model {
        var count int = 0
    }
    msg Msg { Bump }
    on {
        .Bump -> { .count = .count + 1 }
    }
}
"#;
        let path = std::env::temp_dir().join("plan012_multi_store_probe.at");
        std::fs::write(&path, src).expect("write probe .at");

        let result = generate_component_from_file(&path, ComponentGenOptions::default())
            .expect("two-store file must generate");

        let names: Vec<&str> = result
            .store_composables
            .iter()
            .map(|(f, _)| f.as_str())
            .collect();
        assert_eq!(
            names,
            vec!["stores/useAlphaStoreStore.ts", "stores/useBetaStoreStore.ts"],
            "both stores must be emitted, got: {:?}",
            names
        );

        // The thread-local (drained by auto-man's incremental path) must hold
        // the same two files.
        let drained = crate::drain_store_extra_files();
        let drained_names: Vec<&str> = drained.iter().map(|(f, _)| f.as_str()).collect();
        assert_eq!(drained_names, names, "thread-local must match return value");

        let _ = std::fs::remove_file(&path);
    }

    /// Gap 2: the 015-notes-specific `all_tags` getter auto-inject is gone.
    /// A store with a `notes` state var but NO declared `all_tags` computed
    /// must generate a composable with no `all_tags` anywhere (previously the
    /// hack injected a getter referencing `notes.value`).
    #[test]
    fn test_store_with_notes_var_no_all_tags_injection() {
        use super::{generate_component_from_file, ComponentGenOptions};

        let src = r#"
store NotesLikeStore {
    model {
        var notes []str = []
    }
    msg Msg { Touch }
    on {
        .Touch -> { }
    }
}
"#;
        let path = std::env::temp_dir().join("plan012_notes_no_all_tags_probe.at");
        std::fs::write(&path, src).expect("write probe .at");

        let result = generate_component_from_file(&path, ComponentGenOptions::default())
            .expect("store file must generate");
        crate::drain_store_extra_files(); // keep thread-local clean for other tests

        assert_eq!(result.store_composables.len(), 1);
        let (filename, code) = &result.store_composables[0];
        assert_eq!(filename, "stores/useNotesLikeStoreStore.ts");
        assert!(
            code.contains("export function useNotesLikeStoreStore()"),
            "composable function, got:\n{}",
            code
        );
        assert!(
            !code.contains("all_tags"),
            "no all_tags getter may be injected, got:\n{}",
            code
        );

        let _ = std::fs::remove_file(&path);
    }

    /// Gap 2 (compatibility): a store that DECLARES `all_tags` in its
    /// `computed {}` block — the workaround jade and 015-notes carry — must
    /// still compile, and the declared getter is emitted exactly once.
    #[test]
    fn test_store_declared_all_tags_placeholder_still_compiles() {
        use super::{generate_component_from_file, ComponentGenOptions};

        let src = r#"
store TagsStore {
    model {
        var notes []str = []
    }
    msg Msg { Touch }
    computed {
        all_tags => []
    }
    on {
        .Touch -> { }
    }
}
"#;
        let path = std::env::temp_dir().join("plan012_declared_all_tags_probe.at");
        std::fs::write(&path, src).expect("write probe .at");

        let result = generate_component_from_file(&path, ComponentGenOptions::default())
            .expect("declared all_tags placeholder must still compile");
        crate::drain_store_extra_files();

        let (_, code) = &result.store_composables[0];
        let occurrences = code.matches("get all_tags()").count();
        assert_eq!(
            occurrences, 1,
            "declared all_tags getter must appear exactly once, got:\n{}",
            code
        );

        let _ = std::fs::remove_file(&path);
    }
    // ====================================================================
    // Plan musk-022 Phase 1: resolve_stream_variants parses a
    // `#[serde(tag="type", rename_all="snake_case")] pub tag T { ... }`
    // declaration, extracting the discriminator field name and the
    // wire-value → action-name variant map. Nested variant bodies
    // (e.g. `Delta { text str }`) must NOT truncate parsing.
    // ====================================================================

    #[test]
    fn test_resolve_stream_variants_snake_case_tag() {
        let content = r#"
#[serde(tag = "type", rename_all = "snake_case")]
pub tag SseEventDto {
    Delta { text str }
    Thinking { thinking str }
    ToolCall { id Option<str>, name str, arguments Value }
    ToolResult { id Option<str>, name str, result str, status str }
    Done { output str, turns int }
    Error { message str }
}
"#;
        let (disc, variants) = super::resolve_stream_variants(content, "SseEventDto");
        assert_eq!(disc, "type");
        // All 6 variants captured, snake_cased.
        let wires: Vec<_> = variants.iter().map(|(w, _)| w.as_str()).collect();
        assert_eq!(wires, vec!["delta", "thinking", "tool_call", "tool_result", "done", "error"]);
        let actions: Vec<_> = variants.iter().map(|(_, a)| a.as_str()).collect();
        assert_eq!(
            actions,
            vec!["Delta", "Thinking", "ToolCall", "ToolResult", "Done", "Error"]
        );
    }

    #[test]
    fn test_resolve_stream_variants_defaults_when_absent() {
        // No pub tag declared → legacy defaults.
        let content = "#[api(...)] pub fn stream() ~Stream<Foo> { }";
        let (disc, variants) = super::resolve_stream_variants(content, "Foo");
        assert_eq!(disc, "event");
        assert!(variants.is_empty());
    }

    #[test]
    fn test_resolve_stream_variants_default_discriminator_without_tag_attr() {
        // A pub tag WITHOUT #[serde(tag=...)] → discriminator defaults to "event".
        // (Auto's pub tag convention: one variant per line.)
        let content = "pub tag Plain {\n    A\n    B\n    C\n}";
        let (disc, variants) = super::resolve_stream_variants(content, "Plain");
        assert_eq!(disc, "event");
        assert_eq!(
            variants.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>(),
            vec!["A", "B", "C"]
        );
    }

    #[test]
    fn test_to_snake_case_variants() {
        use super::to_snake_case;
        assert_eq!(to_snake_case("Delta"), "delta");
        assert_eq!(to_snake_case("ToolCall"), "tool_call");
        assert_eq!(to_snake_case("ToolResult"), "tool_result");
        assert_eq!(to_snake_case("HTTPError"), "http_error");
        assert_eq!(to_snake_case("Done"), "done");
    }

    // ====================================================================
    // Plan 013: pac.at `shadcn: off` — ComponentGenOptions.shadcn switch
    // ====================================================================

    /// Write `src` to a temp .at file and run `generate_component_from_file`.
    fn gen_with_shadcn(src: &str, shadcn: Option<bool>) -> super::GeneratedComponent {
        let tmp = std::env::temp_dir().join(format!(
            "plan013_shadcn_{:?}",
            shadcn
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let at_path = tmp.join("app.at");
        std::fs::write(&at_path, src).unwrap();
        let opts = super::ComponentGenOptions {
            shadcn,
            ..Default::default()
        };
        super::generate_component_from_file(&at_path, opts).expect("generate must succeed")
    }

    const SHADCN_SWITCH_SRC: &str = r#"
widget App {
    view {
        col {
            button "save" {
                class: "primary"
            }
        }
    }
}
"#;

    /// Default (None) keeps the current shadcn behavior: `button` maps to the
    /// shadcn-vue `<Button>` component with a `@/components/ui/button` import.
    #[test]
    fn test_shadcn_default_on_maps_button_component() {
        let result = gen_with_shadcn(SHADCN_SWITCH_SRC, None);
        assert!(
            result.vue_code.contains("@/components/ui/button"),
            "default must emit shadcn import:\n{}",
            result.vue_code
        );
        assert!(
            result.vue_code.contains("<Button"),
            "default must map button → <Button>:\n{}",
            result.vue_code
        );
    }

    /// `shadcn: Some(false)` (pac.at `shadcn: off`): `button` stays a native
    /// `<button>` element and no `@/components/ui/*` import is emitted.
    #[test]
    fn test_shadcn_off_keeps_native_button() {
        let result = gen_with_shadcn(SHADCN_SWITCH_SRC, Some(false));
        assert!(
            result.vue_code.contains("<button"),
            "shadcn off must keep native <button>:\n{}",
            result.vue_code
        );
        assert!(
            !result.vue_code.contains("@/components/ui"),
            "shadcn off must not emit shadcn imports:\n{}",
            result.vue_code
        );
        // The widget's own class still lands on the element.
        assert!(
            result.vue_code.contains("primary"),
            "shadcn off must preserve classes:\n{}",
            result.vue_code
        );
    }
}
