//! PLAN-814 T-02：recipe↔aura/ui_gen 的提取/生成集成测试——原内联于
//! design_tokens/recipe.rs（迁移后归 auto-lang-core），因横跨上层 aura/ui_gen
//! 域上移至本 crate；被测公共面不变（load_and_validate_style_recipes 等 pub）。
use crate::ast::Expr;
use crate::design_tokens::recipe::{
    clear_style_recipes, load_and_validate_style_recipes,
    load_and_validate_style_recipes_with_imports, prepare_style_recipe_imports, StyleRecipeError,
};
use crate::parser::Parser;

fn test_style_recipe_in_widget_extraction() {
    clear_style_recipes();

    let code = r#"
        style card_base = "bg-card rounded-xl shadow-sm"
        style btn(bg: str = "bg-primary") = "{bg} px-4 py-2 rounded-md"

        widget TestCard {
            view {
                col {
                    style: card_base
                    button "Click" {
                        style: btn(bg: "bg-destructive")
                    }
                }
            }
        }
    "#;
    let mut p = Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
    let ast = p.parse().unwrap();
    load_and_validate_style_recipes(&ast.stmts).unwrap();

    let widget_decl = ast
        .stmts
        .iter()
        .find_map(|s| {
            if let crate::ast::Stmt::WidgetDecl(d) = s {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();

    let aura_widget = crate::aura::extract_widget_from_decl(widget_decl).unwrap();
    if let crate::aura::AuraNode::Element {
        props, children, ..
    } = &aura_widget.view_tree
    {
        // The root col should have desugared style
        let root_style = props.get("style").unwrap();
        if let crate::aura::AuraPropValue::Expr(Expr::Str(s)) = root_style {
            assert_eq!(s.as_str(), "bg-card rounded-xl shadow-sm");
        } else {
            panic!("Expected Expr::Str for col style, got: {:?}", root_style);
        }

        // The button child should have desugared style
        if let crate::aura::AuraNode::Element {
            props: btn_props, ..
        } = &children[0]
        {
            let btn_style = btn_props.get("style").unwrap();
            if let crate::aura::AuraPropValue::Expr(Expr::Str(s)) = btn_style {
                assert_eq!(s.as_str(), "bg-destructive px-4 py-2 rounded-md");
            } else {
                panic!("Expected Expr::Str for button style, got: {:?}", btn_style);
            }
        } else {
            panic!(
                "Expected AuraNode::Element for child, got: {:?}",
                children[0]
            );
        }
    } else {
        panic!(
            "Expected AuraNode::Element for root view_tree, got: {:?}",
            aura_widget.view_tree
        );
    }
}

#[test]
fn test_style_recipe_in_vue_gen() {
    clear_style_recipes();

    let code = r#"
        style card_base = "bg-card rounded-xl shadow-sm border border-border"
        style pill(bg: str = "bg-primary") = "{bg} px-4 py-2 rounded-full"

        widget RecipeCard {
            view {
                col {
                    style: card_base
                    button "Click" {
                        style: pill(bg: "bg-destructive")
                    }
                }
            }
        }
    "#;
    let mut p = Parser::from(code).with_session(auto_lang_core::session::CompilerSession::ui());
    let ast = p.parse().unwrap();
    load_and_validate_style_recipes(&ast.stmts).unwrap();

    let widget_decl = ast
        .stmts
        .iter()
        .find_map(|s| {
            if let crate::ast::Stmt::WidgetDecl(d) = s {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();

    let aura_widget = crate::aura::extract_widget_from_decl(widget_decl).unwrap();
    let mut gen = crate::ui_gen::VueGenerator::new();
    let sfc = gen.generate_sfc(&aura_widget).unwrap();

    // Check that Vue template contains the desugared class strings
    assert!(
        sfc.contains("bg-card rounded-xl shadow-sm border border-border"),
        "Vue output should contain desugared card_base: {}",
        sfc
    );
    assert!(
        sfc.contains("bg-destructive px-4 py-2 rounded-full"),
        "Vue output should contain desugared pill: {}",
        sfc
    );
    // And should NOT contain the raw recipe identifier
    assert!(
        !sfc.contains("card_base"),
        "Vue output should not contain raw recipe symbol 'card_base': {}",
        sfc
    );
}
// ========================================================================
// PLAN-635: cross-package style recipe imports (use symbol form)
// ========================================================================

fn plan635_source(base: &std::path::Path, file: &str) -> String {
    std::fs::read_to_string(base.join(file)).unwrap()
}

fn plan635_load(base: &std::path::Path) {
    let code = plan635_source(base, "app.at");
    let imports = prepare_style_recipe_imports(base, &code).unwrap();
    let mut p = Parser::from(code.as_str()).with_session(auto_lang_core::session::CompilerSession::ui());
    let ast = p.parse().unwrap();
    load_and_validate_style_recipes_with_imports(&imports, &ast.stmts).unwrap();
}

fn plan635_load_err(base: &std::path::Path) -> StyleRecipeError {
    let code = plan635_source(base, "app.at");
    match prepare_style_recipe_imports(base, &code) {
        Err(e) => e,
        Ok(imports) => {
            let mut p = Parser::from(code.as_str()).with_session(auto_lang_core::session::CompilerSession::ui());
            let ast = p.parse().unwrap();
            load_and_validate_style_recipes_with_imports(&imports, &ast.stmts).unwrap_err()
        }
    }
}

#[test]


fn test_plan635_vue_chain_expands_imported_recipe() {
    // PLAN-635 T-04: production Vue chain (ui_build_shadcn -> generate_
    // component_from_file) must expand imported recipes — same
    // registration path as the VM track.
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path();
    std::fs::write(
        base.join("styles.at"),
        r#"
        pub style pill = "rounded-full px-4 py-2 shadow-sm"
    "#,
    )
    .unwrap();
    std::fs::write(
        base.join("app.at"),
        r#"
        use styles: pill
        widget App {
            view {
                col { style: pill
                    text "demo"
                }
            }
        }
    "#,
    )
    .unwrap();
    let sfc = crate::ui_build_shadcn(base.join("app.at").to_str().unwrap(), None).unwrap();
    assert!(
        sfc.contains("rounded-full px-4 py-2 shadow-sm"),
        "SFC should contain imported recipe classes: {}",
        sfc
    );
}

#[test]
fn test_plan635_example_fixture_vue_chain() {
    // PLAN-635 T-07: the tracked two-package example — pac.at declares
    // `dep stylekit { path: "../../ui/stylekit" }` (demo migrated to
    // capability-tests 2026-09-20), app.at uses
    // `use stylekit.styles: card_base, pill`. Compile through the real
    // production chain and assert the imported + derived recipes expand.
    let manifest = env!("CARGO_MANIFEST_DIR");
    let app = std::path::Path::new(manifest)
        .join("../../examples/capability-tests/style-import/src/front/app.at");
    if !app.exists() {
        panic!("example fixture missing: {}", app.display());
    }
    let sfc = crate::ui_build_shadcn(app.to_str().unwrap(), None).unwrap();
    assert!(
        sfc.contains("bg-card text-card-foreground rounded-xl shadow-sm border border-border"),
        "imported card_base expands in SFC: {}",
        sfc
    );
    assert!(
        sfc.contains("px-4 py-2 bg-primary text-primary-foreground rounded-full"),
        "imported pill() expands in SFC: {}",
        sfc
    );
    assert!(
        sfc.contains("bg-destructive"),
        "host pill_danger deriving imported pill expands: {}",
        sfc
    );
    assert!(
        !sfc.lines()
            .any(|l| l.contains("class=\"") && l.contains("card_base")),
        "raw recipe symbol must not leak into class bindings: {}",
        sfc
    );
}
