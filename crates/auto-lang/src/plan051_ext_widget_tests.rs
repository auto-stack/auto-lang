//! PLAN-051 C4 regression tests: `use.web` 点分 kind 文法 + adapter widget
//! 注册进视图 registry + 显式 fn 导出校验。
//!
//! ## 修复背景（KD-047 ② / PLAN-051 T5）
//!
//! VM 装载器 `load_ext_imports_for_vm` 此前 fn-only：`use.web component
//! Markdown from "…/ports/renderer.at"` 的 widget 形态在 VM 轨无落点——
//! adapter 链解析到的 renderer.vm.at 里的同名降级 widget 从不注册，消息
//! 正文 fallback 空白。修复：adapter 解析出的同名 widget 声明进视图
//! registry + child_decls；`use.web.fn` 显式声明做 .at 目标导出校验
//! （typo 编译期报错）；点分 kind（use.web.fn/component/composable）与
//! 旧空格/裸形式兼容等价。
//!
//! Corpus: `test/ui/plan051_ext_widget/`（mirrors musk ports 形态）。

#[cfg(all(test, feature = "ui-interpreter"))]
mod plan051_ext_widget_tests {
    /// Corpus: src/front/app.at（含 use.web component Markdown + use.web.fn）。
    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan051_ext_widget/src/front/app.at";
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    fn view_contains_text_deep<M: Clone + std::fmt::Debug>(
        view: &crate::ui::view::View<M>,
        needle: &str,
    ) -> bool {
        use crate::ui::view::View;
        match view {
            View::Text { content, .. } => content.contains(needle),
            View::Column { children, .. } | View::Row { children, .. } => {
                children.iter().any(|c| view_contains_text_deep(c, needle))
            }
            View::Container { child, .. } => view_contains_text_deep(child, needle),
            _ => false,
        }
    }

    /// REGRESSION (widget 臂): `use.web component Markdown` 经 adapter 链
    /// （renderer.at → renderer.vm.at）注册进视图 registry——view 中的
    /// Markdown 节点以 **VM adapter** 的纯文本降级渲染（修复前 fallback
    /// 空白 / `<Markdown />` 占位文本）。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan051_ext_widget_markdown_renders_from_vm_adapter() {
        let dc = match crate::plan370_test_support::build_component_from_app(
            &locate_corpus().expect("corpus app.at"),
        ) {
            Some(c) => c,
            None => {
                eprintln!("plan051: SKIPPED — corpus app.at not found");
                return;
            }
        };
        // Init 跑通：helperFn 经 adapter 链取 renderer.vm.at 实现。
        let body = match dc.read_state("body") {
            Ok(auto_val::Value::Str(s)) => s.as_str().to_string(),
            other => panic!("body 状态异常: {:?}", other),
        };
        assert_eq!(body, "vm-plain-degraded", "helperFn 必须来自 VM adapter（非 web 原文件）");

        // Markdown widget 渲染了 source（pre-wrap 降级形态）。
        let (view, _, _) = dc.view_with_debug_gated(false);
        assert!(
            view_contains_text_deep(&view, "vm-plain-degraded"),
            "Markdown 组件必须以 VM adapter 降级形态渲染 source; got: {:?}",
            view
        );
        assert!(
            !view_contains_text_deep(&view, "<Markdown />"),
            "未注册组件不得回落占位文本"
        );
    }

    /// 点分 kind 文法：`use.web.fn/component/composable` 解析为对应 kind，
    /// 与旧空格形式 / 裸形式兼容等价。
    #[test]
    fn plan051_useweb_dotted_kind_parses() {
        use crate::ast::ui::ExtImportKind;
        fn parse_first_kind(src: &str) -> ExtImportKind {
            let mut parser =
                crate::Parser::from(src).with_session(crate::session::CompilerSession::ui());
            let ast = parser.parse().expect("parse");
            for stmt in &ast.stmts {
                if let crate::ast::Stmt::UseWeb(entries) = stmt {
                    return entries[0].kind;
                }
            }
            panic!("no UseWeb stmt in: {}", src);
        }
        assert_eq!(
            parse_first_kind("use.web.fn a from \"x.at\""),
            ExtImportKind::ExplicitFn,
            "点分 fn → ExplicitFn（校验语义）"
        );
        assert_eq!(
            parse_first_kind("use.web.component Markdown from \"x.at\""),
            ExtImportKind::Component
        );
        assert_eq!(
            parse_first_kind("use.web.composable useT from \"x.ts\""),
            ExtImportKind::Composable
        );
        // 旧形式兼容：空格 component / 裸形式（无 kind = Fn，不校验）。
        assert_eq!(
            parse_first_kind("use.web component Markdown from \"x.at\""),
            ExtImportKind::Component
        );
        assert_eq!(
            parse_first_kind("use.web helperFn from \"x.at\""),
            ExtImportKind::Fn,
            "裸形式保持 Fn（可能是函数/对象/常量，不校验）"
        );
    }

    /// 显式 fn 导出校验：`use.web.fn` 指向 .at 目标但目标未导出该符号 →
    /// 编译期报错（替代现状 typo 静默落 stub）。
    #[test]
    fn plan051_useweb_explicit_fn_typo_fails() {
        let tmp = std::env::temp_dir().join(format!(
            "plan051_typo_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let front = tmp.join("src").join("front");
        std::fs::create_dir_all(&front).unwrap();
        std::fs::write(
            front.join("app.at"),
            concat!(
                "use.web.fn noSuchHelper from \"src/front/ports/helper.at\"\n",
                "widget App {\n",
                "    msg { Init }\n",
                "    view { col { text \"x\" } }\n",
                "}\n",
            ),
        )
        .unwrap();
        let ports = front.join("ports");
        std::fs::create_dir_all(&ports).unwrap();
        std::fs::write(
            ports.join("helper.at"),
            "fn realHelper() str { return \"ok\" }\n",
        )
        .unwrap();

        let code = std::fs::read_to_string(front.join("app.at")).unwrap();
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let root_decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .expect("root decl");
        let mut visited = std::collections::HashSet::new();
        let mut seen = std::collections::HashSet::new();
        let mut import_stmts = Vec::new();
        let mut import_session = crate::compile::CompileSession::new();
        let mut aliases = std::collections::HashMap::new();
        let mut ext_widgets = Vec::new();
        let base_dir = front.clone();
        let err = crate::load_ext_imports_for_vm(
            &base_dir,
            &ast,
            &root_decl,
            &[],
            &mut visited,
            &mut import_stmts,
            &mut seen,
            &mut import_session,
            None,
            &mut aliases,
            &mut ext_widgets,
        )
        .expect_err("typo 必须报错");
        assert!(
            err.contains("noSuchHelper"),
            "报错必须点名符号: {}",
            err
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
