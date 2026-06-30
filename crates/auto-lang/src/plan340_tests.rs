//! Plan 340: VM+VM split mode (HTTP) — JSON natives + codegen API rewriting tests.
//!
//! These verify the three layers added in Plan 340:
//!   1. `auto.json.to_value` / `auto.json.from_value` round-trip.
//!   2. codegen rewrites a bare `#[api]` call into HTTP native calls when
//!      `api_over_http=true` (and leaves it as a CALL reloc when false).
//!   3. Regression: merge mode (api_over_http=false) still links for 015-notes.

#[cfg(test)]
mod plan340_tests {
    use crate::run_with_capture;

    // ──────────────────────────────────────────────────────────────────────
    // Layer 1: JSON ↔ VM value native round-trip
    // ──────────────────────────────────────────────────────────────────────

    /// `auto.json.from_value` should serialize a string into a JSON string
    /// (quoted). This is the simplest smoke test that the native is registered
    /// and callable from VM code.
    #[test]
    fn test_json_from_value_string() {
        // Note: auto.json.from_value takes a NanoValue. For a string literal,
        // we pass it directly; the native wraps it in quotes.
        let code = r#"
let s = "hello"
let js = Json.from_value(s)
print(js)
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok(), "from_value should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        // A JSON-encoded "hello" is "\"hello\""
        eprintln!("plan340 from_value(string) = [{}]", stdout);
        assert!(
            stdout.contains("\"hello\""),
            "expected quoted JSON string, got: [{}]",
            stdout
        );
    }

    /// `auto.json.from_value` on an int yields the bare number.
    #[test]
    fn test_json_from_value_int() {
        let code = r#"
let n = 42
print(Json.from_value(n))
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok(), "from_value(int) should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        eprintln!("plan340 from_value(int) = [{}]", stdout);
        assert!(stdout.contains("42"), "expected 42, got: [{}]", stdout);
    }

    /// `auto.json.from_value` on a bool yields "true"/"false".
    #[test]
    fn test_json_from_value_bool() {
        let code = r#"
let b = true
print(Json.from_value(b))
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok(), "from_value(bool) should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        eprintln!("plan340 from_value(bool) = [{}]", stdout);
        assert!(stdout.contains("true"), "expected true, got: [{}]", stdout);
    }

    /// `auto.json.to_value` parses a JSON array string, then `from_value`
    /// re-serializes it — a round-trip that should yield the same array.
    /// This avoids depending on `.len()` interop on the resulting heap object.
    #[test]
    fn test_json_to_value_array_roundtrip() {
        let code = r#"
let js = "[1,2,3,4,5]"
let arr = Json.to_value(js)
let back = Json.from_value(arr)
print(back)
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok(), "to_value roundtrip should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        eprintln!("plan340 to_value(array) roundtrip = [{}]", stdout);
        assert!(
            stdout.contains("[1,2,3,4,5]") || stdout.contains("[1, 2, 3, 4, 5]"),
            "expected JSON array [1,2,3,4,5], got: [{}]",
            stdout
        );
    }

    /// `auto.json.to_value` on an object: build a struct-like object, then
    /// access a field by name (GET_FIELD via field_names).
    #[test]
    fn test_json_to_value_object_field() {
        // The object built by to_value uses GenericInstanceData with field_names
        // = JSON keys, so GET_FIELD by name should work.
        let code = r#"
let js = "{\"id\":7,\"title\":\"hello\"}"
let obj = Json.to_value(js)
print(obj.title)
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok(), "to_value(object) should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        eprintln!("plan340 to_value(object).title = [{}]", stdout);
        assert!(
            stdout.contains("hello"),
            "expected field title=hello, got: [{}]",
            stdout
        );
    }

    // ──────────────────────────────────────────────────────────────────────
    // Layer 2 & 3: codegen API rewriting + merge-mode regression
    // (These need the UI feature for synthesize_widget_module + VmBridge.)
    // ──────────────────────────────────────────────────────────────────────

    #[cfg(feature = "ui")]
    #[test]
    fn test_codegen_rewrites_api_call_to_http_when_split() {
        use crate::compile::CompileSession;
        use crate::use_scanner::scan_use_statements;
        use std::collections::{HashMap, HashSet};
        use std::path::PathBuf;

        // Build a tiny two-module project in a temp dir: back/api.at (with an
        // #[api] fn) + front/app.at (a widget calling that fn).
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let back = root.join("back");
        std::fs::create_dir_all(&back).unwrap();
        std::fs::write(
            back.join("api.at"),
            r#"
pub type Note = { id int, title str }

#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note {
    return []
}
"#,
        )
        .unwrap();
        let front = root.join("front");
        std::fs::create_dir_all(&front).unwrap();
        std::fs::write(
            front.join("app.at"),
            r#"
use back.api: list_notes

widget App {
    msg Msg { Load }

    model {
        var notes = []
    }

    view {
        text "hello"
    }

    on {
        .Load -> {
            .notes = list_notes()
        }
    }
}
"#,
        )
        .unwrap();

        // Replicate collect_module_imports + alias build + synthesize, with
        // api_over_http=true. Then check the module's relocs do NOT contain
        // a CALL reloc for "list_notes" (it was rewritten to HTTP natives).
        let code = std::fs::read_to_string(front.join("app.at")).unwrap();
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut widget = None;
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                widget = Some(
                    crate::aura::extract_widget_from_decl(decl)
                        .map_err(|e| e.to_string())
                        .expect("extract"),
                );
                break;
            }
        }
        let widget = widget.expect("widget");

        // collect imports
        let mut visited = HashSet::new();
        let mut import_stmts: Vec<crate::ast::Stmt> = Vec::new();
        let mut seen = HashSet::new();
        let mut import_session = CompileSession::new();
        let mut aliases: HashMap<String, String> = HashMap::new();
        let use_stmts = scan_use_statements(&code);
        for us in &use_stmts {
            if us.is_c_import || us.is_rust_import {
                continue;
            }
            let Some(mp) = crate::resolve_module_path(&front, &us.module) else { continue };
            collect_imports_test(&mp, &mut visited, &mut import_stmts, &mut seen, &mut import_session);
            let qualifier = us.module.split('.').last().unwrap_or(&us.module);
            for item in &us.items {
                aliases.insert(item.clone(), format!("{}.{}", qualifier, item));
            }
        }

        // SPLIT mode (api_over_http=true): rewrite to HTTP.
        let (module_split, _) = crate::ui::handler_codegen::synthesize_widget_module(
            &widget, &[], import_stmts.clone(), &aliases, true,
        )
        .expect("synthesize split");

        // The list_notes call must NOT appear as a FuncCall reloc in split mode.
        let has_list_notes_reloc = module_split.relocs.iter().any(|r| {
            r.reloc_type == crate::vm::loader::RelocType::FuncCall
                && (r.symbol_name == "list_notes" || r.symbol_name.ends_with(".list_notes"))
        });
        eprintln!(
            "plan340 split-mode relocs: {:?}",
            module_split.relocs.iter().map(|r| &r.symbol_name).collect::<Vec<_>>()
        );
        assert!(
            !has_list_notes_reloc,
            "split mode: list_notes should be rewritten to HTTP, not a CALL reloc"
        );

        // MERGE mode (api_over_http=false): list_notes stays a CALL reloc.
        let (module_merge, _) = crate::ui::handler_codegen::synthesize_widget_module(
            &widget, &[], import_stmts, &aliases, false,
        )
        .expect("synthesize merge");
        let has_list_notes_reloc_merge = module_merge.relocs.iter().any(|r| {
            r.symbol_name == "list_notes"
                || r.symbol_name.ends_with(".list_notes")
        });
        eprintln!(
            "plan340 merge-mode relocs: {:?}",
            module_merge.relocs.iter().map(|r| &r.symbol_name).collect::<Vec<_>>()
        );
        assert!(
            has_list_notes_reloc_merge,
            "merge mode: list_notes should remain a CALL reloc"
        );
    }

    /// Local copy of collect_module_imports (private in lib.rs) for the test.
    fn collect_imports_test(
        module_path: &std::path::Path,
        visited: &mut std::collections::HashSet<std::path::PathBuf>,
        out: &mut Vec<crate::ast::Stmt>,
        seen: &mut std::collections::HashSet<String>,
        session: &mut crate::compile::CompileSession,
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
        if let Some(dir) = module_path.parent() {
            let abs = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
            session.add_source_dir(abs);
            if let Some(parent) = dir.parent() {
                let abs_p = std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
                session.add_source_dir(abs_p);
            }
        }
        let _ = session.resolve_uses(&code);
        let parser_session = if module_path.components().any(|c| c.as_os_str() == "back") {
            crate::session::CompilerSession::core()
        } else {
            crate::session::CompilerSession::ui()
        };
        let mut parser =
            crate::Parser::new_with_type_store(code.as_str(), session.type_store())
                .with_session(parser_session);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(_) => return,
        };
        let module_name = module_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        for stmt in &ast.stmts {
            match stmt {
                crate::ast::Stmt::Fn(_) => {
                    if let Some(name) = crate::stmt_symbol_name(stmt) {
                        let qualified = format!("{}.{}", module_name, name);
                        if seen.insert(qualified.clone()) {
                            let mut s = stmt.clone();
                            if let crate::ast::Stmt::Fn(ref mut f) = s {
                                f.name = crate::ast::Name::from(qualified.as_str());
                            }
                            out.push(s);
                        }
                    }
                }
                crate::ast::Stmt::TypeDecl(_) | crate::ast::Stmt::EnumDecl(_) | crate::ast::Stmt::Ext(_) => {
                    if let Some(name) = crate::stmt_symbol_name(stmt) {
                        if seen.insert(name.clone()) {
                            out.push(stmt.clone());
                        }
                    }
                }
                crate::ast::Stmt::Use(_) => {
                    out.push(stmt.clone());
                }
                crate::ast::Stmt::Store(s) => {
                    let key = format!("__store:{}", s.name);
                    if seen.insert(key) {
                        out.push(stmt.clone());
                    }
                }
                _ => {}
            }
        }
        let module_dir = module_path.parent().unwrap_or(std::path::Path::new("."));
        for dep in crate::use_scanner::scan_use_statements(&code) {
            if dep.is_c_import || dep.is_rust_import {
                continue;
            }
            if let Some(dep_path) = crate::resolve_module_path(module_dir, &dep.module) {
                collect_imports_test(&dep_path, visited, out, seen, session);
            }
        }
    }
}
