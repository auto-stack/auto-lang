//! PLAN-1062 T-01: production syn AST closure for file-helper corpora that
//! legacy-inventory's wrapper sources do not cover (trans/python.rs a2p
//! helpers and conformance_tests.rs). Discovery only: a binding here is a
//! declared producer/consumer edge, never runtime registration or execution.
//! The registration/ignored split is joined from the native test listing
//! elsewhere; unresolved nonliteral inputs are surfaced, not guessed.
use std::path::Path;

use auto_test_case::sha256::{hex, sha256};
use serde_json::{json, Value};
use syn::visit::Visit;

struct HelperCalls {
    manifest: std::path::PathBuf,
    corpus: &'static str,
    children: Vec<Value>,
    unresolved: Vec<String>,
    /// Literal string arrays bound by `let NAME = [...]` in the visited fn
    /// (the conformance_bootstrap shape feeds its loop from such a binding).
    let_arrays: Vec<(String, Vec<String>)>,
    /// Loop variables currently in scope whose iterable resolved to a let
    /// array: `(loop_var, cases)`. Calls to the generator helper with the
    /// loop variable close over exactly these cases instead of dangling as
    /// nonliteral.
    active_loops: Vec<(String, Vec<String>)>,
}

fn literal(expression: &syn::Expr) -> Option<String> {
    match expression {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(value),
            ..
        }) => Some(value.value()),
        _ => None,
    }
}

fn literal_array(expression: &syn::Expr) -> Option<Vec<String>> {
    if let syn::Expr::Array(array) = expression {
        let cases: Vec<String> = array.elems.iter().filter_map(literal).collect();
        if cases.len() == array.elems.len() && !cases.is_empty() {
            return Some(cases);
        }
    }
    None
}

/// Exact name derivation of trans/python.rs `test_a2p`: strip the numeric
/// prefix of the last path segment ("000_hello" -> "hello",
/// "09_001_option" stays "001_option" only as a directory-bearing case whose
/// last segment itself carries the prefix).
fn python_input_name(case: &str) -> String {
    let last_segment = case.rsplit('/').next().unwrap_or(case);
    let parts: Vec<&str> = last_segment.split('_').collect();
    parts[1..].join("_")
}

/// Lexically collapse `..`/`.` so emitted identities stay repo-relative
/// (`crates/auto-lang-core/../auto-lang/test/...` -> `crates/auto-lang/...`)
/// without touching the filesystem or resolving links.
fn normalize(path: &Path) -> std::path::PathBuf {
    use std::path::Component;
    let mut out = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

impl<'ast> Visit<'ast> for HelperCalls {
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {} // uncalled nested definitions are not executions

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(syn::LocalInit { expr, .. }) = &local.init {
            let ident = match &local.pat {
                syn::Pat::Ident(pi) => Some(pi.ident.to_string()),
                syn::Pat::Type(typed) => match &*typed.pat {
                    syn::Pat::Ident(pi) => Some(pi.ident.to_string()),
                    _ => None,
                },
                _ => None,
            };
            if let (Some(ident), Some(cases)) = (ident, literal_array(expr)) {
                self.let_arrays.push((ident, cases));
            }
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        let iterable = match &*expression.expr {
            syn::Expr::Reference(reference) => &*reference.expr,
            other => other,
        };
        let cases = match iterable {
            syn::Expr::Path(path) => path
                .path
                .segments
                .last()
                .and_then(|s| {
                    self.let_arrays
                        .iter()
                        .find(|(name, _)| *name == s.ident.to_string())
                })
                .map(|(_, cases)| cases.clone()),
            other => literal_array(other),
        };
        let loop_var = match expression.pat.as_ref() {
            syn::Pat::Ident(pi) => Some(pi.ident.to_string()),
            syn::Pat::Reference(reference) => match &*reference.pat {
                syn::Pat::Ident(pi) => Some(pi.ident.to_string()),
                _ => None,
            },
            _ => None,
        };
        if let (Some(cases), Some(loop_var)) = (cases, loop_var) {
            self.active_loops.push((loop_var, cases));
            syn::visit::visit_expr_for_loop(self, expression);
            self.active_loops.pop();
        } else {
            syn::visit::visit_expr_for_loop(self, expression);
        }
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        let syn::Expr::Path(path) = &*expression.func else {
            syn::visit::visit_expr_call(self, expression);
            return;
        };
        let Some(name) = path.path.segments.last().map(|s| s.ident.to_string()) else {
            syn::visit::visit_expr_call(self, expression);
            return;
        };
        let known = match self.corpus {
            "python" => name == "test_a2p",
            "conformance" => matches!(name.as_str(), "run_conformance_test" | "generate_expected"),
            _ => false,
        };
        if known {
            let args: Option<Vec<_>> = expression.args.iter().map(literal).collect();
            let Some(args) = args else {
                // A generator call over an in-scope loop variable closes the
                // loop's literal case array; anything else is unresolved.
                if name == "generate_expected" && expression.args.len() == 1 {
                    if let syn::Expr::Path(arg) = &expression.args[0] {
                        let arg_name = arg.path.segments.last().map(|s| s.ident.to_string());
                        if let Some(arg_name) = arg_name {
                            if let Some((_, cases)) =
                                self.active_loops.iter().find(|(var, _)| *var == arg_name)
                            {
                                self.children.push(json!({
                                    "kind":"generator-loop","family":"a2r-conformance",
                                    "id":arg_name,"cases":cases,
                                    "oracle":"expected_output.txt",
                                    "comparison":"generator-write",
                                }));
                                syn::visit::visit_expr_call(self, expression);
                                return;
                            }
                        }
                    }
                }
                self.unresolved.push(format!(
                    "{name}: nonliteral input in helper corpus test body"
                ));
                syn::visit::visit_expr_call(self, expression);
                return;
            };
            if args.len() != 1 {
                self.unresolved
                    .push(format!("{name}: unexpected helper arity {}", args.len()));
                syn::visit::visit_expr_call(self, expression);
                return;
            }
            let case = &args[0];
            if self.corpus == "python" {
                let input_name = python_input_name(case);
                let directory = normalize(&self.manifest.join("../auto-lang/test/a2p")).join(case);
                self.children.push(json!({
                    "kind":"oracle","family":"a2p","id":case,
                    "source":super::super::execution_trace::file_identity(
                        &directory.join(format!("{input_name}.at"))),
                    "oracle":super::super::execution_trace::file_identity(
                        &directory.join(format!("{input_name}.expected.py"))),
                    "comparison":"bytes",
                }));
            } else {
                let directory = normalize(&self.manifest.join("test/a2r/conformance")).join(case);
                let comparison = if name == "run_conformance_test" {
                    "unicode-trim"
                } else {
                    "generator-write"
                };
                self.children.push(json!({
                    "kind":if name == "run_conformance_test" {"oracle"} else {"generator-call"},
                    "family":"a2r-conformance","id":case,
                    "source":super::super::execution_trace::file_identity(&directory.join("input.at")),
                    "oracle":super::super::execution_trace::file_identity(
                        &directory.join("expected_output.txt")),
                    "comparison":comparison,
                }));
            }
        }
        syn::visit::visit_expr_call(self, expression);
    }
}

fn scan(
    items: Vec<syn::Item>,
    manifest: &Path,
    corpus: &'static str,
    namespace: &str,
    bindings: &mut Vec<Value>,
    unresolved: &mut Vec<Value>,
) {
    for item in items {
        if let syn::Item::Mod(nested) = &item {
            if let Some((_, children)) = &nested.content {
                scan(
                    children.clone(),
                    manifest,
                    corpus,
                    &format!("{namespace}::{}", nested.ident),
                    bindings,
                    unresolved,
                );
            }
            continue;
        }
        let syn::Item::Fn(function) = item else {
            continue;
        };
        if !function.attrs.iter().any(|a| a.path().is_ident("test")) {
            continue;
        }
        let name = format!("{namespace}::{}", function.sig.ident);
        let mut calls = HelperCalls {
            manifest: manifest.to_path_buf(),
            corpus,
            children: Vec::new(),
            unresolved: Vec::new(),
            let_arrays: Vec::new(),
            active_loops: Vec::new(),
        };
        calls.visit_block(&function.block);
        if !calls.children.is_empty() {
            bindings.push(json!({"test":name,"children":calls.children}));
        }
        for reason in calls.unresolved {
            unresolved.push(json!({"test":name,"reason":reason}));
        }
    }
}

/// AST-declared file-helper bindings for the python/conformance corpora.
/// `workspace_root` is the repository root (the two manifest directories are
/// derived from the crate layout, never from ambient state).
pub fn helper_inventory(workspace_root: &Path) -> Result<Value, String> {
    let sources: &[(&str, &str, std::path::PathBuf, &'static str)] = &[
        (
            "crates/auto-lang-core/src/trans/python.rs",
            "trans::python",
            workspace_root.join("crates/auto-lang-core"),
            "python",
        ),
        (
            "crates/auto-lang/src/tests/conformance_tests.rs",
            "tests::conformance_tests",
            workspace_root.join("crates/auto-lang"),
            "conformance",
        ),
    ];
    let mut source_rows = Vec::new();
    let mut bindings = Vec::new();
    let mut unresolved = Vec::new();
    for (relative, namespace, manifest, corpus) in sources {
        let path = workspace_root.join(relative);
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        let syntax = syn::parse_file(text).map_err(|e| format!("{}: {e}", path.display()))?;
        source_rows.push(json!({"path":path,"sha256":hex(&sha256(&bytes))}));
        scan(
            syntax.items,
            manifest,
            corpus,
            namespace,
            &mut bindings,
            &mut unresolved,
        );
    }
    if bindings.is_empty() {
        return Err("helper inventory selected zero declared bindings".into());
    }
    Ok(json!({
        "schema":"test-helper-inventory/1","scope":"discovery","executed":0,
        "sources":source_rows,"bindings":bindings,"unresolved":unresolved,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus(root: &Path, corpus: &str, source: &str) {
        // helper_inventory always reads both sources; the untested one is an
        // empty stub (zero bindings), the tested one carries `source`.
        let (python, conformance) = if corpus == "python" {
            (source, "")
        } else {
            ("", source)
        };
        let trans = root.join("crates/auto-lang-core/src/trans");
        std::fs::create_dir_all(&trans).unwrap();
        std::fs::write(trans.join("python.rs"), python).unwrap();
        let tests = root.join("crates/auto-lang/src/tests");
        std::fs::create_dir_all(&tests).unwrap();
        std::fs::write(tests.join("conformance_tests.rs"), conformance).unwrap();
    }

    #[test]
    fn python_literals_bind_named_inputs_with_byte_comparison() {
        let directory = tempfile::tempdir().unwrap();
        corpus(
            directory.path(),
            "python",
            r###"
            fn test_a2p(case: &str) {}
            mod tests {
                fn test_a2p(case: &str) {}
                #[test] fn hello(){super::super::test_a2p("000_hello");}
                #[test] fn option(){test_a2p("09_option_result/001_option");}
                fn uncalled(){test_a2p("000_uncalled");}
            }
        "###,
        );
        // The helper lives in the same file the CLI tool parses; emulate the
        // real layout by putting the call module under `mod tests`.
        let value = helper_inventory(directory.path()).unwrap();
        let bindings = value["bindings"].as_array().unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0]["test"], "trans::python::tests::hello");
        assert_eq!(bindings[0]["children"][0]["id"], "000_hello");
        assert_eq!(
            bindings[0]["children"][0]["source"]["path"]
                .as_str()
                .unwrap()
                .replace('\\', "/")
                .ends_with("test/a2p/000_hello/hello.at"),
            true
        );
        assert_eq!(bindings[1]["children"][0]["comparison"], "bytes");
        assert!(value["unresolved"].as_array().unwrap().is_empty());
        assert_eq!(value["executed"], 0);
    }

    #[test]
    fn conformance_generator_loop_and_oracle_calls_close_the_write_surface() {
        let directory = tempfile::tempdir().unwrap();
        corpus(
            directory.path(),
            "conformance",
            r###"
            fn run_conformance_test(case: &str) {}
            fn generate_expected(case: &str) {}
            #[test] fn bootstrap(){
                let cases = ["001_int_add", "002_str_concat"];
                for case in &cases { let _ = generate_expected(case); }
            }
            #[test] fn one(){run_conformance_test("001_int_add");}
            #[test] fn dynamic(){let c="002_str_concat";run_conformance_test(c);}
        "###,
        );
        let value = helper_inventory(directory.path()).unwrap();
        let bindings = value["bindings"].as_array().unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0]["children"][0]["kind"], json!("generator-loop"));
        assert_eq!(
            bindings[0]["children"][0]["cases"],
            json!(["001_int_add", "002_str_concat"])
        );
        assert_eq!(bindings[1]["children"][0]["comparison"], "unicode-trim");
        let unresolved = value["unresolved"].as_array().unwrap();
        assert_eq!(unresolved.len(), 1);
        assert!(unresolved[0]["reason"]
            .as_str()
            .unwrap()
            .contains("nonliteral"));
    }
}
