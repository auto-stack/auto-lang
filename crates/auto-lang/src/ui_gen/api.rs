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

#[cfg(test)]
mod tests {
    #[test]
    fn test_transpile_placeholder() {
        // Active API smoke test — real tests require .at fixture files
        assert!(true);
    }
}
