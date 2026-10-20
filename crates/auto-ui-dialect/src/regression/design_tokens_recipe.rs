//! PLAN-814 R3-03b：样式配方（design_tokens::recipe）回归——原
//! auto-lang-core `design_tokens/recipe.rs` 的 `ui-dialect` 门控测试随迁
//! （输入为 UI 语句，依赖文法表装配）。测试体逐字保持，仅路径改写。

use auto_lang_core::ast::*;
use auto_lang_core::design_tokens::recipe::*;
use auto_lang_core::parser::Parser;
use auto_lang_core::session::CompilerSession;

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAN-814 R3-03b：装配 UI 文法函数表（迁移前由 SDK `ui-dialect`
    /// feature 编译期提供；此处进程级注入单源构造器，语义等价）。
    fn setup() {
        let _ = auto_lang_core::set_ui_grammar(crate::ui_grammar());
    }

    fn plan635_source(base: &std::path::Path, file: &str) -> String {
        std::fs::read_to_string(base.join(file)).unwrap()
    }

    fn plan635_load(base: &std::path::Path) {
        let code = plan635_source(base, "app.at");
        let imports = prepare_style_recipe_imports(base, &code).unwrap();
        let mut p = Parser::from(code.as_str()).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        load_and_validate_style_recipes_with_imports(&imports, &ast.stmts).unwrap();
    }

    fn plan635_load_err(base: &std::path::Path) -> StyleRecipeError {
        let code = plan635_source(base, "app.at");
        match prepare_style_recipe_imports(base, &code) {
            Err(e) => e,
            Ok(imports) => {
                let mut p = Parser::from(code.as_str()).with_session(CompilerSession::ui());
                let ast = p.parse().unwrap();
                load_and_validate_style_recipes_with_imports(&imports, &ast.stmts).unwrap_err()
            }
        }
    }

    #[test]
    fn test_style_recipe_semantic_validation() {
        setup();
        clear_style_recipes();

        // 1. Direct cycle: a -> b -> a
        let code_cycle = r#"
            style a = b
            style b = a
        "#;
        let mut p = Parser::from(code_cycle).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let err = load_and_validate_style_recipes(&ast.stmts);
        assert!(err.is_err(), "Expected error for circular references");
        let err_str = err.err().unwrap().to_string();
        assert!(
            err_str.contains("Circular reference"),
            "Expected cycle error, got: {}",
            err_str
        );

        // 2. Unknown parameter in call
        let code_param = r#"
            style pill(bg: str = "bg-primary") = "{bg} rounded-full"
            style bad = pill(foo: "bar")
        "#;
        let mut p = Parser::from(code_param).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let err = load_and_validate_style_recipes(&ast.stmts);
        assert!(err.is_err(), "Expected error for unknown parameter");
        let err_str = err.err().unwrap().to_string();
        assert!(
            err_str.contains("Unknown parameter 'foo'"),
            "Expected unknown param error, got: {}",
            err_str
        );

        // 3. Missing required parameter
        let code_req = r#"
            style pill(bg: str, pad: str = "p-2") = "{bg} {pad}"
            style bad = pill(pad: "p-4")
        "#;
        let mut p = Parser::from(code_req).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let err = load_and_validate_style_recipes(&ast.stmts);
        assert!(
            err.is_err(),
            "Expected error for missing required parameter"
        );
        let err_str = err.err().unwrap().to_string();
        assert!(
            err_str.contains("Missing required parameter 'bg'"),
            "Expected missing param error, got: {}",
            err_str
        );

        // 4. Undefined recipe reference
        let code_undef = r#"
            style good = missing_recipe
        "#;
        let mut p = Parser::from(code_undef).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let err = load_and_validate_style_recipes(&ast.stmts);
        assert!(err.is_err(), "Expected error for undefined recipe");
        let err_str = err.err().unwrap().to_string();
        assert!(
            err_str.contains("is not defined"),
            "Expected undefined recipe error, got: {}",
            err_str
        );
    }

    #[test]
    fn test_style_recipe_desugar_engine() {
        setup();
        clear_style_recipes();

        let code = r#"
            style card_base = "bg-card rounded-xl shadow-sm"
            style pill(bg: str = "bg-primary", fg: str = "text-primary-foreground", pad: str = "px-4 py-2") =
                "{pad} {bg} {fg} rounded-full"
            style pill_danger = pill(bg: "bg-destructive", fg: "text-destructive-foreground")
            style composite = "{card_base} mt-4"
        "#;
        let mut p = Parser::from(code).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let res = load_and_validate_style_recipes(&ast.stmts);
        assert!(res.is_ok(), "Validation failed: {:?}", res);

        // Test bare ident expansion
        let ident_expr = Expr::Ident("card_base".into());
        let desugared = desugar_style_expr(&ident_expr).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(s.as_str(), "bg-card rounded-xl shadow-sm");
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test call with defaults
        let call_defaults = Parser::parse_expr_fragment("pill()").unwrap();
        let desugared = desugar_style_expr(&call_defaults).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(
                s.as_str(),
                "px-4 py-2 bg-primary text-primary-foreground rounded-full"
            );
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test call with overrides
        let call_override = Parser::parse_expr_fragment("pill(bg: \"bg-red-500\")").unwrap();
        let desugared = desugar_style_expr(&call_override).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(
                s.as_str(),
                "px-4 py-2 bg-red-500 text-primary-foreground rounded-full"
            );
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test composite recipe expansion
        let comp_ident = Expr::Ident("pill_danger".into());
        let desugared = desugar_style_expr(&comp_ident).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(
                s.as_str(),
                "px-4 py-2 bg-destructive text-destructive-foreground rounded-full"
            );
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test interpolation in string
        let interp = Expr::Str("{card_base} p-2".into());
        let desugared = desugar_style_expr(&interp).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(s.as_str(), "bg-card rounded-xl shadow-sm p-2");
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test array mixing
        let arr = Expr::Array(vec![
            Expr::Ident("card_base".into()),
            Expr::Str("border-2".into()),
        ]);
        let desugared = desugar_style_expr(&arr).unwrap();
        if let Expr::Array(elems) = desugared {
            assert_eq!(elems.len(), 2);
            if let Expr::Str(s0) = &elems[0] {
                assert_eq!(s0.as_str(), "bg-card rounded-xl shadow-sm");
            } else {
                panic!("Expected Expr::Str for elem 0");
            }
            if let Expr::Str(s1) = &elems[1] {
                assert_eq!(s1.as_str(), "border-2");
            } else {
                panic!("Expected Expr::Str for elem 1");
            }
        } else {
            panic!("Expected Expr::Array, got: {:?}", desugared);
        }
    }

    #[test]
    fn test_style_recipe_lint_palette_warnings() {
        setup();
        clear_style_recipes();

        let code = r#"
            style good_recipe = "bg-primary text-primary-foreground border-border"
            style bad_recipe = "bg-blue-500 text-zinc-400 hover:bg-red-600/80"
        "#;
        let mut p = Parser::from(code).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let warnings = load_and_validate_style_recipes(&ast.stmts).unwrap();

        // bad_recipe should generate warnings for bg-blue-500, text-zinc-400, hover:bg-red-600/80
        assert_eq!(
            warnings.len(),
            3,
            "Expected 3 warnings, got: {:?}",
            warnings
        );
        assert!(
            warnings.iter().any(|w| w.contains("bg-blue-500")),
            "Expected bg-blue-500 warning"
        );
        assert!(
            warnings.iter().any(|w| w.contains("text-zinc-400")),
            "Expected text-zinc-400 warning"
        );
        assert!(
            warnings.iter().any(|w| w.contains("hover:bg-red-600/80")),
            "Expected hover:bg-red-600/80 warning"
        );
    }

    #[test]
    fn test_plan635_named_import_registers_and_desugars() {
        setup();
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        std::fs::write(
            base.join("styles.at"),
            r#"
            pub style pill = "rounded-full px-4 py-2"
        "#,
        )
        .unwrap();
        std::fs::write(
            base.join("app.at"),
            r#"
            use styles: pill
            style local_btn = pill
        "#,
        )
        .unwrap();
        plan635_load(base);

        let pill = get_style_recipe("pill").expect("imported pub recipe registered");
        assert_eq!(pill.source_module.as_deref(), Some("styles"));
        assert_eq!(pill.is_pub, true);

        // Desugar: string interpolation of the imported recipe expands inline.
        let expanded =
            desugar_style_expr(&auto_lang_core::ast::Expr::Str("{pill} mt-2".into())).unwrap();
        match expanded {
            auto_lang_core::ast::Expr::Str(s) => {
                assert!(
                    s.as_str().contains("rounded-full"),
                    "imported recipe expanded: {}",
                    s.as_str()
                )
            }
            other => panic!("expected folded string, got {:?}", other),
        }
    }

    #[test]
    fn test_plan635_named_import_non_pub_is_error() {
        setup();
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        std::fs::write(
            base.join("styles.at"),
            r#"
            style internal_only = "hidden"
        "#,
        )
        .unwrap();
        std::fs::write(base.join("app.at"), "use styles: internal_only").unwrap();
        let err = plan635_load_err(base);
        assert!(err.to_string().contains("not pub"), "got: {}", err);
    }

    #[test]
    fn test_plan635_name_collision_is_error() {
        setup();
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        std::fs::write(
            base.join("styles.at"),
            r#"
            pub style pill = "rounded-full"
        "#,
        )
        .unwrap();
        std::fs::write(
            base.join("app.at"),
            r#"
            use styles: pill
            style pill = "local-pill"
        "#,
        )
        .unwrap();
        let err = plan635_load_err(base);
        assert!(err.to_string().contains("defined in both"), "got: {}", err);
    }

    #[test]
    fn test_plan635_wildcard_imports_pub_only() {
        setup();
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        std::fs::write(
            base.join("styles.at"),
            r#"
            pub style pub_chip = "rounded-full"
            style internal_only = "hidden"
        "#,
        )
        .unwrap();
        std::fs::write(base.join("app.at"), "use styles: *").unwrap();
        plan635_load(base);
        assert!(
            has_style_recipe("pub_chip"),
            "pub recipe imported by wildcard"
        );
        assert!(
            !has_style_recipe("internal_only"),
            "non-pub recipe stays private"
        );
    }

    #[test]
    fn test_plan635_transitive_import() {
        setup();
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        std::fs::write(
            base.join("tokens.at"),
            r#"
            pub style base_border = "border border-border"
        "#,
        )
        .unwrap();
        std::fs::write(
            base.join("styles.at"),
            r#"
            use tokens: base_border
            pub style card = "{base_border} bg-card rounded-xl"
        "#,
        )
        .unwrap();
        std::fs::write(base.join("app.at"), "use styles: card").unwrap();
        plan635_load(base);
        assert!(
            has_style_recipe("base_border"),
            "transitive import registered"
        );

        let expanded =
            desugar_style_expr(&auto_lang_core::ast::Expr::Str("{card}".into())).unwrap();
        match expanded {
            auto_lang_core::ast::Expr::Str(s) => {
                assert!(
                    s.as_str().contains("border-border"),
                    "transitive expansion: {}",
                    s.as_str()
                );
                assert!(
                    s.as_str().contains("bg-card"),
                    "own body expansion: {}",
                    s.as_str()
                );
            }
            other => panic!("expected folded string, got {:?}", other),
        }
    }

    #[test]
    fn test_plan635_cross_package_deps_layout() {
        setup();
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        let dep_front = base.join("deps").join("common").join("src").join("front");
        std::fs::create_dir_all(&dep_front).unwrap();
        std::fs::write(
            dep_front.join("styles.at"),
            r#"
            pub style pill = "rounded-full shadow-sm"
        "#,
        )
        .unwrap();
        // PLAN-635 D4: deps/ probing is declaration-gated.
        std::fs::write(
            base.join("pac.at"),
            "name: \"t\"
scene: \"ui\"
dep common {
    path: \"../common\"
}
",
        )
        .unwrap();
        std::fs::write(base.join("app.at"), "use common.styles: pill").unwrap();
        plan635_load(base);
        let pill = get_style_recipe("pill").expect("deps/<name> layout resolves");
        assert_eq!(pill.source_module.as_deref(), Some("common.styles"));
    }
}
