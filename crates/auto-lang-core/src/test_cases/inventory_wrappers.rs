//! Literal calls in real Rust test ASTs bind external goldens to their wrapper.
//! This is discovery only; each binding still requires a real terminal receipt.
use std::path::Path;

use auto_test_case::sha256::{hex, sha256};
use serde_json::{json, Value};
use syn::parse::Parser;
use syn::visit::Visit;

struct Calls<'a> {
    manifest: &'a Path,
    module: &'a str,
    children: Vec<Value>,
    unresolved: Vec<String>,
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

impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {} // uncalled nested definitions are not executions

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if let Ok(arguments) =
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
                .parse2(mac.tokens.clone())
        {
            for argument in &arguments {
                self.visit_expr(argument);
            }
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
        let known = match self.module {
            "a2r_tests" => matches!(
                name.as_str(),
                "test_a2r"
                    | "test_a2r_deep"
                    | "test_cookbook"
                    | "test_cookbook_deep"
                    | "test_a2r_expect_error"
            ),
            "a2c_tests" => name == "test_a2c",
            "a2ts_tests" => name == "test_a2ts",
            "cookbook_vm_tests" => name == "test_cookbook",
            _ => false,
        };
        if known {
            let args: Option<Vec<_>> = expression.args.iter().map(literal).collect();
            let Some(args) = args else {
                self.unresolved
                    .push(format!("{name}: nonliteral input in registered test body"));
                return;
            };
            let expected = if self.module == "cookbook_vm_tests" && args.len() == 2 {
                let directory = self
                    .manifest
                    .join("test/cookbook")
                    .join(&args[0])
                    .join(&args[1]);
                // PLAN-1066 T-03[B]/T-08: the document is the entry's author
                // source and oracle carrier. The runtime wrapper records the
                // SAME derivation, so this static guard cannot drift from it.
                let has_document = std::fs::read_dir(&directory)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .any(|entry| {
                        entry
                            .path()
                            .file_name()
                            .is_some_and(|f| f.to_string_lossy().ends_with(".case.ad"))
                    });
                if !has_document {
                    self.unresolved.push(format!(
                        "{name}: original cookbook directory has no migration document"
                    ));
                    return;
                }
                super::super::execution_trace::cookbook_doc_identity(&directory, &args[0], &args[1])
            } else if name == "test_a2r_expect_error" && args.len() == 2 {
                super::super::execution_trace::legacy_error_identity(
                    self.manifest.to_str().unwrap(),
                    &args[0],
                    &args[1],
                )
            } else if args.len() == 1 {
                let (base, suffixes): (&str, &[&str]) = match self.module {
                    "a2c_tests" => (
                        "a2c",
                        &[".expected.c", ".expected.h", ".expected.error.log"],
                    ),
                    "a2ts_tests" => ("a2ts", &[".expected.ts"]),
                    _ => (
                        if name.starts_with("test_cookbook") {
                            "cookbook"
                        } else {
                            "a2r"
                        },
                        &[".expected.rs"],
                    ),
                };
                // PLAN-1066 T-08: one document identity for every legacy
                // family (the suffix list retired with the sidecars).
                let _ = suffixes;
                super::super::execution_trace::legacy_document_identity(
                    self.manifest.to_str().unwrap(),
                    base,
                    &args[0],
                )
            } else {
                self.unresolved
                    .push(format!("{name}: unexpected original helper arity"));
                return;
            };
            self.children.push(expected);
        }
        syn::visit::visit_expr_call(self, expression);
    }
}

pub(super) fn discover(test_root: &Path) -> Result<Value, String> {
    let manifest = test_root.parent().ok_or("legacy manifest root missing")?;
    let mut sources = Vec::new();
    let mut bindings = Vec::new();
    let mut unresolved = Vec::new();
    for module in ["a2r_tests", "a2c_tests", "a2ts_tests", "cookbook_vm_tests"] {
        let path = manifest.join("src/tests").join(format!("{module}.rs"));
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        let syntax = syn::parse_file(text).map_err(|e| e.to_string())?;
        sources.push(json!({"path":path,"sha256":hex(&sha256(&bytes))}));
        fn scan(
            items: Vec<syn::Item>,
            manifest: &Path,
            module: &str,
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
                            module,
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
                let mut calls = Calls {
                    manifest,
                    module,
                    children: Vec::new(),
                    unresolved: Vec::new(),
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
        scan(
            syntax.items,
            manifest,
            module,
            &format!("tests::{module}"),
            &mut bindings,
            &mut unresolved,
        );
    }
    Ok(json!({"sources":sources,"bindings":bindings,"unresolved":unresolved,"executed":0}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(source: &str) -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(directory.path().join("src/tests")).unwrap();
        std::fs::create_dir(directory.path().join("test")).unwrap();
        for module in ["a2r_tests", "a2c_tests", "a2ts_tests", "cookbook_vm_tests"] {
            std::fs::write(
                directory.path().join(format!("src/tests/{module}.rs")),
                if module == "a2r_tests" { source } else { "" },
            )
            .unwrap();
        }
        directory
    }

    #[test]
    fn ast_calls_include_nested_and_macro_expressions_but_not_guest_text_or_uncalled_definitions() {
        let directory = fixture(
            r###"
            fn helper(){test_a2r("uncalled").unwrap();}
            #[test] fn first(){
                let guest=r#"test_a2r("literal-only")"#;
                fn uncalled(){test_a2r("inner-definition").unwrap();}
                assert!(test_a2r("01/001_first").is_ok());
            }
            mod nested { #[test] fn second(){super::test_a2r_deep("01/002_second");} }
            #[cfg(any())] #[test] fn not_registered(){test_a2r("01/003_cfg").unwrap();}
        "###,
        );
        let value = discover(&directory.path().join("test")).unwrap();
        let bindings = value["bindings"].as_array().unwrap();
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings[0]["children"][0]["id"], "01/001_first");
        assert_eq!(bindings[1]["test"], "tests::a2r_tests::nested::second");
        assert_eq!(bindings[1]["children"][0]["id"], "01/002_second");
        assert!(value["unresolved"].as_array().unwrap().is_empty());
        assert_eq!(value["executed"], 0); // AST declaration is not runtime registration or execution.
    }

    #[test]
    fn nonliteral_external_input_is_explicitly_unresolved() {
        let directory =
            fixture(r#"#[test] fn dynamic(){let case="01/001_first";test_a2r(case).unwrap();}"#);
        let value = discover(&directory.path().join("test")).unwrap();
        assert!(value["bindings"].as_array().unwrap().is_empty());
        assert_eq!(value["unresolved"][0]["test"], "tests::a2r_tests::dynamic");
        assert!(value["unresolved"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("nonliteral"));
    }
}
