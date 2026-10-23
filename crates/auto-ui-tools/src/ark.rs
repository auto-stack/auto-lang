//! ArkTS (HarmonyOS) project generation utilities
//!
//! This module provides the complete ArkTS/HarmonyOS project workflow:
//! 1. Generate ArkTS code from .at files
//! 2. Generate full project structure (optional)
//! 3. Copy to output directory

use std::fs;
use std::path::Path;

use auto_lang_core::Parser;
use auto_ui_compiler::ui_gen::ark::{ArkGenerator, ArkProjectGenerator};
use auto_ui_compiler::ui_gen::BackendGenerator;
use colored::Colorize;

use auto_val::AutoResult;

/// Parse project name from pac.at content
fn parse_pac_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Check if ark is in the render list (supports both "render:" and legacy "backend:" fields)
fn has_ark_backend(content: &str) -> bool {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("render:") || line.starts_with("backend:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                // Check if it's an array format
                if value.starts_with('[') {
                    // Extract all backends from array
                    let backends: Vec<&str> = value
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .split(',')
                        .filter_map(|s| {
                            let s = s.trim().trim_matches('"').trim_matches('\'');
                            if !s.is_empty() {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .collect();
                    // Check for both "ark" and "arkts" variants
                    return backends.iter().any(|&b| b == "ark" || b == "arkts");
                } else {
                    let value = value.trim_end_matches(',');
                    let value = value.trim_matches('"').trim_matches('\'');
                    return value == "ark" || value == "arkts";
                }
            }
        }
    }
    false
}

/// ArkTS project generation context
pub struct ArkProject {
    /// Project root directory (where pac.at is)
    pub root_dir: std::path::PathBuf,
    /// Output directory
    pub output_dir: std::path::PathBuf,
    /// Project name
    pub name: String,
    /// Front source directory
    pub front_dir: std::path::PathBuf,
    /// Generated ArkTS files (relative_path, content)
    pub arkts_files: Vec<(String, String)>,
    /// Widget names discovered from .at files
    pub widget_names: Vec<String>,
}

/// ark 项目读取缝（PLAN-809 T-06c / I-PROJECT）：同 JetReads 模式。
pub(crate) struct ArkReads {
    pub name: String,
    pub gate_ok: bool,
}

impl ArkReads {
    fn scan(root_dir: &Path) -> AutoResult<Self> {
        let pac_path = root_dir.join("pac.at");
        if !pac_path.exists() {
            return Err("pac.at not found in workspace".into());
        }
        let pac_content =
            fs::read_to_string(&pac_path).map_err(|e| format!("Failed to read pac.at: {}", e))?;
        Ok(Self {
            name: parse_pac_name(&pac_content).unwrap_or_else(|| "MyApp".to_string()),
            gate_ok: has_ark_backend(&pac_content),
        })
    }

    fn from_request(req: &auto_project_model::generation_request::GenerationRequest) -> Self {
        let gate_ok = req
            .frontend_names()
            .iter()
            .any(|b| b == "ark" || b == "arkts");
        Self {
            name: req
                .identity
                .name
                .clone()
                .unwrap_or_else(|| "MyApp".to_string()),
            gate_ok,
        }
    }
}

impl ArkProject {
    /// Create a new Ark project context from a workspace directory
    pub fn from_workspace(root_dir: &Path) -> AutoResult<Self> {
        let reads = ArkReads::scan(root_dir)?;
        Self::build(root_dir, reads)
    }

    /// 请求路径（PLAN-809 T-06c 生产入口）；前端目录消费请求声明
    /// （T-06 R1/F-815-T06-R1-01——节点形式优先，无声明回退探测链）。
    pub fn from_workspace_with_request(
        root_dir: &Path,
        req: &auto_project_model::generation_request::GenerationRequest,
    ) -> AutoResult<Self> {
        let front_override = crate::util::request_front_dir(root_dir, req);
        Self::build_front(
            root_dir,
            ArkReads::from_request(req),
            front_override.as_deref(),
        )
    }

    fn build(root_dir: &Path, reads: ArkReads) -> AutoResult<Self> {
        Self::build_front(root_dir, reads, None)
    }

    fn build_front(
        root_dir: &Path,
        reads: ArkReads,
        front_override: Option<&Path>,
    ) -> AutoResult<Self> {
        if !reads.gate_ok {
            return Err("Backend 'ark' not found in pac.at. Add 'ark' to backend list.".into());
        }

        let name = reads.name;

        // Determine front/source directory: the request's declaration
        // wins; legacy keeps the multi-structure probe chain:
        // 1. source/front/ (standard)  2. front/ (alternative)
        // 3. pages/ or widgets/ in root (quickstart)  4. aura/
        // 5. app.at directly in root (simple single-file projects)
        let front_dir = match front_override {
            Some(declared) => declared.to_path_buf(),
            None if root_dir.join("src").join("front").exists() => {
                root_dir.join("src").join("front")
            }
            None if root_dir.join("source").join("front").exists() => {
                root_dir.join("source").join("front")
            }
            None if root_dir.join("front").exists() => root_dir.join("front"),
            None if root_dir.join("pages").exists() || root_dir.join("widgets").exists() => {
                // Quickstart structure: pages/ and widgets/ directly in root
                root_dir.to_path_buf()
            }
            None if root_dir.join("aura").exists() => root_dir.join("aura"),
            None if root_dir.join("app.at").exists() => {
                // Simple single-file project: app.at directly in root
                root_dir.to_path_buf()
            }
            None => root_dir.join("src").join("front"),
        };

        // Output directory
        let output_dir = root_dir.join("gen").join("front").join("ark");

        // Compile .at files to ArkTS
        let mut arkts_files: Vec<(String, String)> = Vec::new();
        let mut widget_names: Vec<String> = Vec::new();

        // Process app.at if exists (check both front_dir and root_dir)
        let app_at = front_dir.join("app.at");
        let root_app_at = root_dir.join("app.at");
        let app_at_to_use = if app_at.exists() {
            app_at
        } else if root_app_at.exists() {
            root_app_at
        } else {
            app_at // default to front_dir join for error messages
        };

        if app_at_to_use.exists() {
            match Self::compile_at_file(&app_at_to_use, &name, true) {
                Ok((files, names)) => {
                    arkts_files.extend(files);
                    widget_names.extend(names);
                }
                Err(e) => {
                    println!(
                        "{} {}",
                        "Warning: Failed to compile app.at:".bright_yellow(),
                        e
                    );
                }
            }
        }

        // Process widgets/ directory
        let widgets_dir = front_dir.join("widgets");
        if widgets_dir.exists() {
            for entry in fs::read_dir(&widgets_dir)
                .map_err(|e| format!("Failed to read widgets directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    match Self::compile_at_file(&path, &name, true) {
                        Ok((files, names)) => {
                            arkts_files.extend(files);
                            widget_names.extend(names);
                        }
                        Err(e) => {
                            println!(
                                "{} Failed to compile {}: {}",
                                "Warning:".bright_yellow(),
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        // Process pages/ directory
        let pages_dir = front_dir.join("pages");
        if pages_dir.exists() {
            for entry in fs::read_dir(&pages_dir)
                .map_err(|e| format!("Failed to read pages directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    match Self::compile_at_file(&path, &name, true) {
                        Ok((files, names)) => {
                            arkts_files.extend(files);
                            widget_names.extend(names);
                        }
                        Err(e) => {
                            println!(
                                "{} Failed to compile {}: {}",
                                "Warning:".bright_yellow(),
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        // Process components/ directory
        let components_dir = front_dir.join("components");
        if components_dir.exists() {
            for entry in fs::read_dir(&components_dir)
                .map_err(|e| format!("Failed to read components directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    match Self::compile_at_file(&path, &name, true) {
                        Ok((files, names)) => {
                            arkts_files.extend(files);
                            widget_names.extend(names);
                        }
                        Err(e) => {
                            println!(
                                "{} Failed to compile {}: {}",
                                "Warning:".bright_yellow(),
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        // Process .at files directly in front_dir (quickstart structure)
        // Skip app.at (already processed) and pac.at (project config)
        for entry in fs::read_dir(&front_dir)
            .map_err(|e| format!("Failed to read front directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().map(|e| e == "at").unwrap_or(false) {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                // Skip app.at and pac.at
                if file_name == "app.at" || file_name == "pac.at" {
                    continue;
                }

                match Self::compile_at_file(&path, &name, true) {
                    Ok((files, names)) => {
                        arkts_files.extend(files);
                        widget_names.extend(names);
                    }
                    Err(e) => {
                        println!(
                            "{} Failed to compile {}: {}",
                            "Warning:".bright_yellow(),
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(Self {
            root_dir: root_dir.to_path_buf(),
            output_dir,
            name,
            front_dir,
            arkts_files,
            widget_names,
        })
    }

    /// Compile a single .at file to ArkTS code
    /// Returns (arkts_files, widget_names)
    fn compile_at_file(
        at_path: &Path,
        _project_name: &str,
        verbose: bool,
    ) -> Result<(Vec<(String, String)>, Vec<String>), String> {
        if verbose {
            let file_name = at_path.file_name().unwrap_or_default().to_string_lossy();
            println!("  Parsing {}", file_name);
        }

        let code = fs::read_to_string(at_path)
            .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

        // Parse with UI scenario
        use auto_lang_core::session::CompilerSession;
        let session = CompilerSession::ui().with_backend("ark");
        let mut parser = Parser::from(code.as_str());
        parser = parser.with_session(session);
        let ast = parser
            .parse()
            .map_err(|e| format!("Parse error: {:?}", e))?;

        let mut files = Vec::new();
        let mut names = Vec::new();

        // First, generate model files for type definitions
        for stmt in &ast.stmts {
            if let auto_lang_core::ast::Stmt::TypeDecl(type_decl) = stmt {
                let model_code = Self::generate_model_file(type_decl);
                if let Some((model_name, model_content)) = model_code {
                    let relative_path = format!("entry/src/main/ets/model/{}.ets", model_name);
                    files.push((relative_path, model_content));
                }
            }
        }

        // Extract AURA widgets from AST
        let mut widgets = Vec::new();
        for stmt in &ast.stmts {
            if let auto_lang_core::ast::Stmt::WidgetDecl(widget_decl) = stmt {
                let aura_widget = auto_ui_compiler::aura::extract_widget_from_decl(widget_decl)
                    .map_err(|e| e.to_string())?;
                widgets.push(aura_widget);
            }
        }

        if widgets.is_empty() && files.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        // Generate ArkTS code for each widget
        let mut generator = ArkGenerator::new();

        for widget in &widgets {
            let arkts_code = generator.generate(widget).map_err(|e| e.to_string())?;

            // Collect widget name
            names.push(widget.name.clone());

            // Generate file path: entry/src/main/ets/pages/{WidgetName}.ets
            let widget_name = &widget.name;
            let file_name = format!("{}.ets", widget_name);
            let relative_path = format!("entry/src/main/ets/pages/{}", file_name);
            files.push((relative_path, arkts_code));
        }

        Ok((files, names))
    }

    /// Generate model file for a type definition
    fn generate_model_file(type_decl: &auto_lang_core::ast::TypeDecl) -> Option<(String, String)> {
        let name = type_decl.name.as_str();

        // Skip if it looks like a built-in type
        if matches!(
            name,
            "NavPathStack" | "string" | "number" | "boolean" | "Object"
        ) {
            return None;
        }

        // Skip if no members (just a type reference)
        if type_decl.members.is_empty() {
            return None;
        }

        let mut lines = Vec::new();

        // Generate export class with properties
        lines.push(format!("export class {} {{", name));
        for member in &type_decl.members {
            let field_name = member.name.as_str();
            let field_type = Self::type_to_arkts(&member.ty);
            // Use appropriate default value based on type
            let default_value = if field_type.ends_with("[]") {
                "[]".to_string()
            } else {
                "''".to_string()
            };
            lines.push(format!(
                "  {}: {} = {}",
                field_name, field_type, default_value
            ));
        }
        lines.push("}".to_string());

        Some((name.to_string(), lines.join("\n")))
    }

    /// Convert Type to ArkTS type string
    fn type_to_arkts(ty: &auto_lang_core::ast::Type) -> String {
        use auto_lang_core::ast::Type;
        match ty {
            Type::Int | Type::Uint | Type::I64 | Type::U64 | Type::Float | Type::Double => {
                "number".to_string()
            }
            Type::Bool => "boolean".to_string(),
            Type::StrFixed(_) | Type::CStrLit | Type::StrSlice => "string".to_string(),
            Type::User(type_decl) => {
                let type_name = type_decl.name.as_str();
                // Special handling for List type - treat as Object[]
                if type_name == "List" {
                    "Object[]".to_string()
                } else {
                    type_decl.name.to_string()
                }
            }
            Type::Option(inner) => format!("{} | null", Self::type_to_arkts(inner)),
            Type::List(inner) => {
                let elem_type = Self::type_to_arkts(inner);
                if elem_type == "Object" {
                    "Object[]".to_string()
                } else {
                    format!("{}[]", elem_type)
                }
            }
            Type::Map(k, v) => {
                format!(
                    "HashMap<{}, {}>",
                    Self::type_to_arkts(k),
                    Self::type_to_arkts(v)
                )
            }
            Type::Slice(slice) => {
                let elem_type = Self::type_to_arkts(&slice.elem);
                if elem_type == "Object" {
                    "Object[]".to_string()
                } else {
                    format!("{}[]", elem_type)
                }
            }
            Type::Array(arr) => {
                let elem_type = Self::type_to_arkts(&arr.elem);
                if elem_type == "Object" {
                    "Object[]".to_string()
                } else {
                    format!("{}[]", elem_type)
                }
            }
            // Unknown type (e.g., unresolved List) - treat as Object[] for collection-like fields
            Type::Unknown => "Object[]".to_string(),
            _ => "Object".to_string(),
        }
    }

    /// Check if the project structure already exists
    pub fn exists(&self) -> bool {
        self.output_dir.exists() && self.output_dir.join("build-profile.json5").exists()
    }

    /// Generate the ArkTS project structure
    pub fn generate(&self) -> AutoResult<()> {
        println!(
            "{}",
            "─────────────────────────────────".bright_yellow().bold()
        );
        println!(
            "{}",
            "  AURA Workspace → ArkTS/HarmonyOS".bright_yellow().bold()
        );
        println!(
            "{}",
            "─────────────────────────────────".bright_yellow().bold()
        );
        println!();

        println!("{} {}", "Output:".bright_cyan(), self.output_dir.display());
        println!("{} {}", "Name:".bright_cyan(), self.name);
        println!(
            "{} {}",
            "Widgets:".bright_cyan(),
            self.widget_names.join(", ")
        );
        println!();

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Generate full project structure using ArkProjectGenerator
        let project_gen = ArkProjectGenerator::new(&self.name);
        let project_files = project_gen.generate();

        // Write all project files
        for (file_path, content) in project_files {
            let full_path = self.output_dir.join(&file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
            // Handle binary files (PNG) which are base64 encoded
            if file_path.ends_with(".png") {
                use base64::{engine::general_purpose, Engine as _};
                let bytes = general_purpose::STANDARD
                    .decode(&content)
                    .map_err(|e| format!("Failed to decode base64 for {}: {}", file_path, e))?;
                fs::write(&full_path, bytes)
                    .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;
            } else {
                fs::write(&full_path, content)
                    .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;
            }
        }

        println!("{}", "✓ Created project structure".bright_green());

        // Write generated ArkTS widget files
        for (relative_path, content) in &self.arkts_files {
            let full_path = self.output_dir.join(relative_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
            fs::write(&full_path, content)
                .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;
            println!("{} {}", "  Generated".bright_green(), relative_path);
        }

        // main_pages.json should only contain App - other pages are NavDestinations
        // (handled by Navigation component, not as separate entry pages)
        let main_pages_path = self
            .output_dir
            .join("entry/src/main/resources/base/profile/main_pages.json");
        let main_pages_content = serde_json::json!({
            "src": ["pages/App"]
        })
        .to_string();
        fs::write(&main_pages_path, main_pages_content)
            .map_err(|e| format!("Failed to write main_pages.json: {}", e))?;

        println!();
        println!("═════════════════════════════════");
        println!(
            "{}",
            "  ArkTS/HarmonyOS project generated!".bright_green().bold()
        );
        println!("═════════════════════════════════");
        println!();
        println!("{} {}", "Next steps:".bright_cyan(), "");
        println!("  cd {}", self.output_dir.display());
        println!("  # Open in DevEco Studio or run hvigorw assembleHap");

        Ok(())
    }

    /// Generate only ArkTS files (no full project structure)
    pub fn generate_arkts_only(&self, output_dir: &Path) -> AutoResult<()> {
        println!(
            "{}",
            "─────────────────────────────────".bright_yellow().bold()
        );
        println!(
            "{}",
            "  AURA → ArkTS Code Generation".bright_yellow().bold()
        );
        println!(
            "{}",
            "─────────────────────────────────".bright_yellow().bold()
        );
        println!();

        println!("{} {}", "Output:".bright_cyan(), output_dir.display());
        println!("{} {}", "Files:".bright_cyan(), self.arkts_files.len());
        println!();

        // Create output directory
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Write generated ArkTS files
        for (relative_path, content) in &self.arkts_files {
            let full_path = output_dir.join(relative_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
            fs::write(&full_path, content)
                .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;
            println!("{} {}", "  Generated".bright_green(), relative_path);
        }

        println!();
        println!("{} ArkTS files generated.", "✓".bright_green());

        Ok(())
    }
}

/// Generate ArkTS code from .at files (auto gen command for ark backend)
pub fn generate_ark_project(
    root_dir: &Path,
    output_dir: Option<&Path>,
    full_project: bool,
) -> AutoResult<()> {
    println!("{}", "Generating ArkTS/HarmonyOS project".bright_cyan());
    let project = ArkProject::from_workspace(root_dir)?;
    generate_ark_project_with_project(output_dir, full_project, project)
}

/// 请求路径（PLAN-809 T-06c 生产入口）。
pub fn generate_ark_project_with_request(
    root_dir: &Path,
    output_dir: Option<&Path>,
    full_project: bool,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    println!("{}", "Generating ArkTS/HarmonyOS project".bright_cyan());
    let project = ArkProject::from_workspace_with_request(root_dir, req)?;
    generate_ark_project_with_project(output_dir, full_project, project)
}

fn generate_ark_project_with_project(
    output_dir: Option<&Path>,
    full_project: bool,
    project: ArkProject,
) -> AutoResult<()> {
    // Determine output directory
    // PLAN-816 T-05 R2（F-R2-02/04）：显式 output 对全工程生成同样生效
    //（覆盖工程上下文的默认落点——镜像 vscode；此前 full_project 分支
    // 静默忽略 output_dir，选项被丢而产物落在默认布局）。
    let mut project = project;
    if let Some(out) = output_dir {
        project.output_dir = out.to_path_buf();
    }
    let output = output_dir.unwrap_or(&project.output_dir);

    if full_project || !project.exists() {
        // Generate full project structure
        project.generate()?;
    } else {
        // Generate only ArkTS files
        project.generate_arkts_only(output)?;
    }

    Ok(())
}

/// Build ArkTS project (auto build command for ark backend)
pub fn build_ark_project(root_dir: &Path) -> AutoResult<()> {
    build_ark_project_inner(root_dir, None)
}

/// 请求路径（PLAN-809 R2-F02：build 分派同样经合同）。
pub fn build_ark_project_with_request(
    root_dir: &Path,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    build_ark_project_inner(root_dir, Some(req))
}

fn build_ark_project_inner(
    root_dir: &Path,
    req: Option<&auto_project_model::generation_request::GenerationRequest>,
) -> AutoResult<()> {
    println!(
        "{}",
        "Building ArkTS/HarmonyOS project (backend: ark)".bright_cyan()
    );

    // PLAN-815 R5（F-815-T05-R5-01）：协议严格失败语义只在**托管形态**
    // （工具宿主 RunIo 在装）生效；旧公开门面（CLI 直跑/Man 转发）保持
    // 基线返回语义——缺 wrapper/构建失败打印提示后 Ok，不静默升级为 Err。
    let strict = crate::proc_host::current().is_some();

    // Step 1: Generate code
    println!();
    println!("{}", "▶ Step 1: Generating ArkTS code...".bright_cyan());
    match req {
        Some(r) => generate_ark_project_with_request(root_dir, None, false, r)?,
        None => generate_ark_project(root_dir, None, false)?,
    }

    let ark_dir = root_dir.join("gen").join("front").join("ark");
    let _hap = assemble_hap(&ark_dir, strict)?;
    Ok(())
}

/// Shared HAP assembly (wrapper check → owned `hvigorw assembleHap` →
/// artifact oracle). PLAN-815 T-06 R1（F-815-T06-R1-02 抽取共享）：
/// `strict`（托管/协议形态）下缺 wrapper/构建失败/退出 0 但产物缺失均
/// 具名 `Err`；legacy 基线=打印提示后 `Ok(None)`（未真正构建）。
fn assemble_hap(ark_dir: &Path, strict: bool) -> AutoResult<Option<std::path::PathBuf>> {
    // Step 2: Check for hvigorw
    println!();
    println!("{}", "▶ Step 2: Checking Hvigor wrapper...".bright_cyan());

    let hvigorw = if cfg!(windows) {
        ark_dir.join("hvigorw.bat")
    } else {
        ark_dir.join("hvigorw")
    };

    if !hvigorw.exists() {
        // PLAN-815 R4-06/R5-01：托管/协议形态缺构建 wrapper=具名失败
        // （未真正执行过构建就不能回报成功）；legacy 基线=提示后 Ok。
        if strict {
            return Err(format!(
                "Hvigor wrapper not found at {} — open the project in DevEco Studio to build (no build was executed)",
                hvigorw.display()
            )
            .into());
        }
        println!("  ⚠ Hvigor wrapper not found.");
        println!("  Please open the project in DevEco Studio to build.");
        println!("  Project location: {}", ark_dir.display());
        return Ok(None);
    }

    println!("  ✓ Hvigor wrapper found");

    // Step 3: Build the project
    println!();
    println!("{}", "▶ Step 3: Building HAP...".bright_cyan());

    // PLAN-815 R4-05（F-815-T05-R4-05）：hvigorw 经 owned 树设施——取消/
    // EOF 真终止整树（不再只置标志等 status 返回）。
    let build_result =
        crate::proc_host::run_external_status(&hvigorw, &["assembleHap"], ark_dir, "ark-build")
            .map_err(|e| format!("Failed to build: {}", e))?;

    let hap = ark_dir.join("entry/build/default/outputs/default/entry-default-signed.hap");
    if build_result.success() {
        // PLAN-815 R4-06/R5-01：产物 oracle 仅托管/协议形态（legacy 基线
        // 保持成功打印语义）。
        if strict && !hap.is_file() {
            return Err(format!(
                "hvigorw assembleHap exited 0 but the HAP artifact is missing at {} — the build did not really run",
                hap.display()
            )
            .into());
        }
        println!();
        println!("{}", "✓ Build successful!".bright_green());
        println!();
        println!("HAP location:");
        println!("  {}", hap.display().to_string().bright_cyan());
        Ok(Some(hap))
    } else {
        // PLAN-815 R4-06/R5-01：托管/协议形态真实构建失败=具名 Err；
        // legacy 基线=打印提示后 Ok（旧公开入口返回语义不静默改）。
        if strict {
            return Err(format!(
                "ark build failed (hvigorw assembleHap exited with {build_result}); try manually: cd {} && ./hvigorw assembleHap",
                ark_dir.display()
            )
            .into());
        }
        println!();
        println!("  ⚠ Build failed. Try running manually:");
        println!("    cd {} && ./hvigorw assembleHap", ark_dir.display());
        Ok(None)
    }
}

/// Run ArkTS project (auto run command for ark backend)
pub fn run_ark_project(root_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    run_ark_project_inner(root_dir, args, None)
}

/// 请求路径（PLAN-809 R1-F02）。
pub fn run_ark_project_with_request(
    root_dir: &Path,
    args: Vec<String>,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    run_ark_project_inner(root_dir, args, Some(req))
}

fn run_ark_project_inner(
    root_dir: &Path,
    _args: Vec<String>,
    req: Option<&auto_project_model::generation_request::GenerationRequest>,
) -> AutoResult<()> {
    println!(
        "{}",
        "Running ArkTS/HarmonyOS project (backend: ark)".bright_cyan()
    );

    // PLAN-815 T-06 R1（F-815-T06-R1-02）：托管/协议形态的 run 是**真实
    // 执行链**——生成 → 构建 HAP → hdc 安装 → aa start 启动；任一前置
    // 缺失（wrapper/hdc/设备）都是具名失败，手工准备不冒充运行完成。
    // legacy CLI 直跑保持手工指导语义（基线返回 Ok，兼容边界）。
    let strict = crate::proc_host::current().is_some();

    // Step 1: Generate code
    println!();
    println!("{}", "▶ Step 1: Generating ArkTS code...".bright_cyan());
    match req {
        Some(r) => generate_ark_project_with_request(root_dir, None, false, r)?,
        None => generate_ark_project(root_dir, None, false)?,
    }

    let ark_dir = root_dir.join("gen").join("front").join("ark");

    if !strict {
        // Legacy manual guidance (frozen baseline).
        println!();
        println!("{}", "═══════════════════════════════════".bright_green());
        println!("{}", "  ArkTS/HarmonyOS project ready!".bright_green());
        println!("{}", "═══════════════════════════════════".bright_green());
        println!();
        println!("To run the app:");
        println!("  1. Open DevEco Studio");
        println!(
            "  2. Open the project at: {}",
            ark_dir.display().to_string().bright_cyan()
        );
        println!("  3. Connect a HarmonyOS device or start an emulator");
        println!("  4. Click Run or use: ./hvigorw installHap");
        return Ok(());
    }

    // Step 2: Assemble the HAP (strict: named failure on any missing
    // prerequisite — wrapper/toolchain included).
    let Some(hap) = assemble_hap(&ark_dir, true)? else {
        // Unreachable in strict mode (assemble_hap errors instead).
        return Err("ark run did not produce a HAP artifact".into());
    };

    // Step 3: hdc toolchain gate.
    println!();
    println!("{}", "▶ Step 3: Checking hdc toolchain...".bright_cyan());
    if !crate::pkg::command_exists("hdc") {
        return Err(
            "hdc not found — install DevEco Studio / the HarmonyOS SDK and put hdc on PATH; manual fallback: open the project in DevEco Studio and Run"
                .into(),
        );
    }

    // Step 4: device probe (owned capture; incomplete probes are named
    // failures per the R6 capture contract).
    println!();
    println!(
        "{}",
        "▶ Step 4: Checking HarmonyOS devices...".bright_cyan()
    );
    let targets = crate::proc_host::run_capture_probe(
        std::path::Path::new("hdc"),
        &["list", "targets"],
        &ark_dir,
        "ark-hdc",
    )?;
    let device = targets
        .as_deref()
        .unwrap_or("")
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.eq_ignore_ascii_case("[empty]"));
    let Some(device) = device else {
        return Err(
            "no HarmonyOS device or emulator connected (hdc list targets is empty) — connect a device or start an emulator; manual fallback: DevEco Studio Run"
                .into(),
        );
    };
    println!("  ✓ device: {device}");

    // Step 5: install the HAP (owned tree).
    println!();
    println!("{}", "▶ Step 5: Installing HAP...".bright_cyan());
    let install = crate::proc_host::run_external_status(
        std::path::Path::new("hdc"),
        &["-t", device, "install", "-r", &hap.to_string_lossy()],
        &ark_dir,
        "ark-install",
    )
    .map_err(|e| format!("hdc install failed to start: {e}"))?;
    if !install.success() {
        return Err(format!(
            "hdc install exited with {install} — HAP at {}",
            hap.display()
        )
        .into());
    }

    // Step 6: launch (bundle from the generated AppScope/app.json5 — the
    // generated output is the single source; ability defaults to the
    // scaffold's EntryAbility unless module.json5 names another).
    let bundle = read_generated_bundle_name(&ark_dir)?;
    let ability = read_generated_main_ability(&ark_dir)?;
    println!();
    println!("{}", "▶ Step 6: Launching app...".bright_cyan());
    let start = crate::proc_host::run_external_status(
        std::path::Path::new("hdc"),
        &[
            "-t", device, "shell", "aa", "start", "-b", &bundle, "-a", &ability,
        ],
        &ark_dir,
        "ark-launch",
    )
    .map_err(|e| format!("hdc shell aa start failed to start: {e}"))?;
    if !start.success() {
        return Err(format!(
            "hdc shell aa start exited with {start} (bundle {bundle}, ability {ability})"
        )
        .into());
    }
    println!();
    println!(
        "{}",
        format!("✓ App launched on {device} (bundle {bundle})").bright_green()
    );
    Ok(())
}

/// One `"key": "value",` line's string value with JSON5 punctuation
/// handled by construction (PLAN-815 T-06 R2 / F-815-T06-R1-02：行尾逗号
/// 与收尾引号必须剥净——`trim_matches('"')` 对 `"...",` 留下 `...",`）。
fn json5_string_value(line: &str, key: &str) -> Option<String> {
    let prefix = format!("\"{key}\":");
    let rest = line.trim().strip_prefix(&prefix)?;
    let rest = rest.trim().trim_end_matches(',').trim();
    let rest = rest.strip_prefix('"')?.strip_suffix('"')?;
    let value = rest.trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// The generated AppScope/app.json5 `bundleName` (single source = the
/// generated output; missing/unparsable = NAMED failure — no default-ID
/// guess may silently launch the wrong app).
fn read_generated_bundle_name(ark_dir: &Path) -> AutoResult<String> {
    let path = ark_dir.join("AppScope").join("app.json5");
    let text = fs::read_to_string(&path).map_err(|e| {
        format!(
            "generated AppScope/app.json5 unreadable ({e}) — cannot determine the launch bundle"
        )
    })?;
    for line in text.lines() {
        if let Some(value) = json5_string_value(line, "bundleName") {
            return Ok(value);
        }
    }
    Err("generated AppScope/app.json5 carries no bundleName — cannot launch".into())
}

/// The generated entry module's `mainElement` ability name (missing/
/// unparsable = named failure, same no-guess rule).
fn read_generated_main_ability(ark_dir: &Path) -> AutoResult<String> {
    let path = ark_dir
        .join("entry")
        .join("src")
        .join("main")
        .join("module.json5");
    let text = fs::read_to_string(&path).map_err(|e| {
        format!(
            "generated entry/src/main/module.json5 unreadable ({e}) — cannot determine the launch ability"
        )
    })?;
    for line in text.lines() {
        if let Some(value) = json5_string_value(line, "mainElement") {
            return Ok(value);
        }
    }
    Err("generated module.json5 carries no mainElement — cannot launch".into())
}

#[cfg(test)]
mod tests {
    use super::json5_string_value;

    /// PLAN-815 T-06 R2（F-815-T06-R1-02）：生成器真实输出形状的 ID 解析
    /// ——行尾逗号+收尾引号剥净；缺引号/空值/其他键不误取。
    #[test]
    fn json5_string_value_strips_trailing_comma_and_quotes() {
        // The exact generated shapes.
        assert_eq!(
            json5_string_value("  \"bundleName\": \"com.example.t05probe\",", "bundleName"),
            Some("com.example.t05probe".to_string())
        );
        assert_eq!(
            json5_string_value("      \"mainElement\": \"EntryAbility\",", "mainElement"),
            Some("EntryAbility".to_string())
        );
        // Last entry of an object (no trailing comma).
        assert_eq!(
            json5_string_value("\"bundleName\": \"x.y.z\"", "bundleName"),
            Some("x.y.z".to_string())
        );
        // Not the key we asked for / non-string / empty → None (no guesses).
        assert_eq!(
            json5_string_value("\"versionCode\": 100,", "bundleName"),
            None
        );
        assert_eq!(
            json5_string_value("\"bundleName\": \"\",", "bundleName"),
            None
        );
        assert_eq!(
            json5_string_value("\"bundleName\": 42,", "bundleName"),
            None
        );
    }
}
