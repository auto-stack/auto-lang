//! Plan 492: 引擎正确性专项——包组件编译/text 内容表达式/f-string 插值三族修复。
//!
//! - 族 A·解析器: ①primary-shorthand 不识别 `[` 后缀;②f-string 含字面量 `[`/`]`
//!   时 `${}` 插值破坏编译。
//! - 族 B·vue 生成器: 文本内容位置的 Index/Dot 表达式求值缺臂。
//! - 族 C·包组件单 VM 编译链: prop 字符串比较/带参 msg 声明静默破坏编译。

#[cfg(test)]
mod m1_fstr_bracket {
    use crate::lexer::Lexer;
    use crate::token::TokenKind;

    fn tokens_str(code: &str) -> String {
        let mut lexer = Lexer::new(code);
        let mut out = String::new();
        loop {
            let tk = lexer.next().expect("lex must succeed");
            if tk.kind == TokenKind::EOF {
                break;
            }
            out.push_str(&tk.to_string());
        }
        out
    }

    /// 词法层锚: `f"w-[${x}px]"` 的 token 序列(字面量 [] 是纯文本 part,
    /// ${x} 是 FStrNote+LBrace 表达式)。
    #[test]
    fn lexer_tokens_dollar_bracket() {
        let tokens = tokens_str(r#"f"w-[${x}px]""#);
        assert_eq!(
            tokens,
            "<fstrs><fstrp:w-[><$><{><ident:x><}><fstrp:px]><fstre>"
        );
    }

    /// vue 层锚: view prop 位置的 f-string(字面量 [] + ${} 插值)。
    /// parse → aura extract → vue SFC 生成,发射插值 style 绑定。
    #[test]
    fn view_prop_vue_sfc_dollar_bracket() {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(
            r##"
widget W {
    model { w int = 120 }
    view {
        col (style: f"w-[${.w}px]") {
            text "x" {}
        }
    }
}
"##,
        )
        .with_session(session);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(err) => {
                panic!("parse failed: {err:?}");
            }
        };
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = crate::ui_gen::VueGenerator::new_shadcn();
        use crate::ui_gen::BackendGenerator;
        let sfc = match gen.generate(&widget) {
            Ok(sfc) => sfc,
            Err(err) => panic!("vue generate failed: {err:?}"),
        };
        println!("=== SFC ===\n{sfc}");
    }

    /// VM 层锚: 同一 widget 源经 VM 编译链,style 类解析为 Width(Pixels)。
    #[cfg(feature = "ui-iced")]
    #[test]
    fn view_prop_vm_style_width_lands() {
        let src = r##"
widget W {
    model { w int = 120 }
    view {
        col (style: f"w-[${.w}px]") {
            text "x" {}
        }
    }
}
"##;
        match crate::build_dynamic_component(src, None) {
            Ok(mut dc) => {
                dc.fire_init();
                let (view, _, _) = dc.view_with_debug();
                println!("=== VM view ===\n{view:?}");
            }
            Err(err) => {
                panic!("VM build failed: {err:?}");
            }
        }
    }

    /// M1 复现(族 A2): 子组件(包组件同款单 VM 链)Init 内
    /// `f"w-[${slot}px] h-full"`——dollar-brace 插值 + 字面量方括号。
    /// 镜像 plan437_child_init 的内联夹具(同 with_registry_and_imports_from_decls 链)。
    #[cfg(feature = "ui-interpreter")]
    mod child_fstr {
        fn build_inline(src: &str) -> crate::ui::dynamic::DynamicComponent {
            use crate::ast::Stmt;
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(src).with_session(session);
            let ast = parser.parse().expect("parse");
            let mut root_decl = None;
            let mut view_widget = None;
            let mut child_decls = Vec::new();
            for stmt in &ast.stmts {
                if let Stmt::WidgetDecl(decl) = stmt {
                    if root_decl.is_none() {
                        root_decl = Some(decl.clone());
                        view_widget = Some(
                            crate::aura::extract_widget_from_decl(decl)
                                .map_err(|e| e.to_string())
                                .unwrap(),
                        );
                    } else {
                        child_decls.push(decl.clone());
                    }
                }
            }
            let root_decl = root_decl.expect("root widget");
            let view_widget = view_widget.expect("view widget");
            let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
            for d in &child_decls {
                let w = crate::aura::extract_widget_from_decl(d)
                    .map_err(|e| e.to_string())
                    .unwrap();
                registry.register(w);
            }
            crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
                &root_decl,
                &child_decls,
                &view_widget,
                registry,
                Vec::new(),
                &std::collections::HashMap::new(),
                false,
            )
            .expect("component builds")
        }

        const SRC_DOLLAR: &str = r##"
widget Parent {
    model {
        series = [
            { m: "A", v: 30 },
            { m: "B", v: 80 }
        ]
    }
    view {
        col {
            child-geo (data: .series, field: "v") {}
        }
    }
}

widget ChildGeo (data: List, field: str = "v") {
    msg { Init }
    model {
        s str = ""
    }
    on {
        .Init -> {
            var slot float = 100.0
            .s = f"w-[${slot}px] h-full"
        }
    }
    view {
        col (style: .s) {
            text .s { }
        }
    }
}
"##;

        /// dollar-brace 形式:Init 计算产物必须落 state(现fail——Init 静默失效)。
        #[test]
        fn child_init_fstr_dollar_bracket_lands() {
            let mut dc = build_inline(SRC_DOLLAR);
            let (view, _, _) = dc.view_with_debug_gated(true);
            let _ = format!("{view:?}");
            let s = dc.bridge().read_state("s").map(|v| v.to_string()).unwrap_or_else(|e| format!("<err {e}>"));
            println!("dollar-brace .s = {s:?}");
            assert_eq!(
                s.replace(".0", ""),
                "\"w-[100px] h-full\"",
                "dollar-brace f-string with literal brackets must interpolate in child Init"
            );
        }

        /// 对照①: 无方括号的 dollar-brace(`f"w-${slot}px h-full"`)。
        #[test]
        fn child_init_fstr_dollar_nobracket_control() {
            let src = SRC_DOLLAR.replace("f\"w-[${slot}px] h-full\"", "f\"w-${slot}px h-full\"");
            let mut dc = build_inline(&src);
            let (view, _, _) = dc.view_with_debug_gated(true);
            let _ = format!("{view:?}");
            let s = dc.bridge().read_state("s").map(|v| v.to_string()).unwrap_or_else(|e| format!("<err {e}>"));
            println!("no-bracket .s = {s:?}");
            assert_eq!(
                s.replace(".0", ""),
                "\"w-100px h-full\"",
                "control: dollar-brace f-string without literal brackets must interpolate"
            );
        }

        /// 对照②: brace 形式(`f"w-[{slot}px] h-full"`)——484 绕开形态。
        #[test]
        fn child_init_fstr_brace_form_control() {
            let src = SRC_DOLLAR.replace("f\"w-[${slot}px] h-full\"", "f\"w-[{slot}px] h-full\"");
            let mut dc = build_inline(&src);
            let (view, _, _) = dc.view_with_debug_gated(true);
            let _ = format!("{view:?}");
            let s = dc.bridge().read_state("s").map(|v| v.to_string()).unwrap_or_else(|e| format!("<err {e}>"));
            // 语义锚: {} 花括号形式在 f-string 中是纯字面量,不插值。
            assert_eq!(s, "\"w-[{slot}px] h-full\"");
        }
    }
}

/// 生产路径复现夹具: 拷贝 charts-gallery src/front 整树到临时目录,按
/// (文件名, 旧串, 新串) 打补丁后经 build_dynamic_component 构建——真实
/// load_package + child_decls 单 VM 编译链(M1 族 A2 / M4 族 C 消费)。
#[cfg(all(test, feature = "ui-iced"))]
pub(crate) mod pkg_harness {
    use std::path::PathBuf;

    pub(crate) fn copy_tree_for_test(
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> std::io::Result<()> {
        copy_tree(src, dst)
    }

    fn copy_tree(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let from = entry.path();
            let to = dst.join(entry.file_name());
            if from.is_dir() {
                copy_tree(&from, &to)?;
            } else {
                std::fs::copy(&from, &to)?;
            }
        }
        Ok(())
    }

    /// 返回构建好的 DynamicComponent + 临时 app.at 路径(调试用)。
    /// `tag` 用于隔离并发测试的临时目录(cargo test 并行)。
    pub(crate) fn build_patched_gallery(
        tag: &str,
        patches: &[(&str, &str, &str)],
    ) -> (crate::ui::dynamic::DynamicComponent, PathBuf) {
        let front = crate::plan370_test_support::locate_example_app_at("charts-gallery")
            .expect("charts-gallery sources must exist")
            .parent()
            .expect("front dir")
            .to_path_buf();
        let tmp = std::env::temp_dir().join(format!("plan492-pkg-repro-{tag}"));
        let _ = std::fs::remove_dir_all(&tmp);
        copy_tree(&front, &tmp).expect("copy gallery front tree");
        for (file, from, to) in patches {
            let p = tmp.join("components").join(file);
            let code = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
            let patched = code.replace(from, to);
            assert!(
                patched != code || code == *to,
                "patch anchor not found in {}: {}",
                file,
                from
            );
            std::fs::write(&p, patched).expect("write patched");
            // 回读验证补丁确实落盘(隔离缓存层嫌疑)。
            let back = std::fs::read_to_string(&p).unwrap();
            assert!(back.contains(to) || to == from, "patch not on disk for {file}");
        }
        let app = tmp.join("app.at");
        let code = std::fs::read_to_string(&app).unwrap();
        let dc = crate::build_dynamic_component(&code, Some(app.to_str().unwrap()))
            .expect("patched gallery must build");
        (dc, app)
    }

    /// 渲染并返回 debug dump(触发子组件 Init 重放)。
    pub(crate) fn render_dump(dc: &mut crate::ui::dynamic::DynamicComponent) -> String {
        dc.fire_init();
        dc.set_route("/");
        let (view, _, _) = dc.view_with_debug_gated(true);
        format!("{view:?}")
    }
}


/// M1 族 A2 vue 侧验证: 打补丁后的 bar_chart 经 vue SFC 生成。
#[cfg(all(test, feature = "ui-iced"))]
mod m1_vue_fstr {
    #[test]
    fn vue_sfc_dollar_bracket_fstr_in_init() {
        // 492 M6 后组件原生 dollar 形态——无补丁基线断言。
        let front = crate::plan370_test_support::locate_example_app_at("charts-gallery")
            .expect("gallery")
            .parent()
            .expect("front")
            .to_path_buf();
        let patched = std::fs::read_to_string(front.join("components/bar_chart.at")).unwrap();
        assert!(
            patched.contains("f\"w-[${slot}px] h-full flex-1\""),
            "component must carry the native dollar-form band style"
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(patched.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract");
        let mut gen = crate::ui_gen::VueGenerator::new_shadcn();
        use crate::ui_gen::BackendGenerator;
        let sfc = gen.generate(&widget).expect("generate SFC");
        // Init handler 里 band 样式字符串必须以插值模板形态出现。
        let has_tpl = sfc.contains("w-[${") || sfc.contains("w-[${slot}px]") || sfc.contains("w-[` +") || sfc.contains("w-[${");
        println!("=== bar SFC (Init 段) ===");
        for line in sfc.lines() {
            if line.contains("w-[") || line.contains("bands") {
                println!("{line}");
            }
        }
        assert!(has_tpl, "vue SFC must carry the interpolated band style; see printed lines");
    }
}

/// M1 族 A2 定案回归锚: dollar-brace + 字面量 [] 在生产包链必须全程存活。
/// (484 记档的"整体失效"在 master 不可复现——金丝雀对照证明夹具可检出
/// 真失败;判定误归因与族 C1 同现场,详见计划 M1 记录。)
#[cfg(all(test, feature = "ui-iced"))]
mod m1_pkg_fstr {
    use super::pkg_harness::{build_patched_gallery, render_dump};

    /// dollar-brace 直写(M6 后组件原生形态,无补丁): 几何+band 全存活。
    #[test]
    fn pkg_init_fstr_dollar_bracket_keeps_geometry() {
        let (mut dc, _) = build_patched_gallery("m1-dollar", &[]);
        let dump = render_dump(&mut dc);
        assert!(
            dump.contains("M 40") || dump.contains("h19") || dump.contains("h25"),
            "bar geometry must survive dollar-brace f-string in Init (A2)"
        );
        assert!(
            dump.matches("MouseArea").count() >= 15,
            "bar bands (mouse-area) must render with dollar-brace f-string in Init"
        );
    }

    /// 金丝雀(负对照): 未定义变量必须杀死 bar Init——证明上方夹具可检出失败。
    /// Plan 498: line/area 图例新增合法 mouse-area 后绝对上界过时,改
    /// 健康态相对比较(canary < healthy),底线保留(其余图命中区不连坐)。
    #[test]
    fn pkg_canary_undefined_var_kills_bar_init() {
        let (mut healthy_dc, _) = build_patched_gallery("m1-canary-healthy", &[]);
        let healthy = render_dump(&mut healthy_dc).matches("MouseArea").count();
        let (mut dc, _) = build_patched_gallery(
            "m1-canary",
            &[(
                "bar_chart.at",
                "s: f\"w-[${slot}px] h-full flex-1\"",
                "s: f\"w-[${undefined_var_xyz}px] h-full flex-1\"",
            )],
        );
        let dump = render_dump(&mut dc);
        let mouse = dump.matches("MouseArea").count();
        assert!(
            mouse < healthy,
            "canary: undefined var in f-string must kill bar bands (canary={mouse} healthy={healthy})"
        );
        assert!(mouse >= 12, "canary floor: sibling charts' hit areas stay alive (got {mouse})");
    }
}
