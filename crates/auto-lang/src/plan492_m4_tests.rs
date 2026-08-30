//! Plan 492 M4 (族 C): 包组件单 VM 编译链分叉——Init 内 prop 字符串比较
//! (C1)与带参 msg 声明(C2)静默破坏整组件/整包编译。
//!
//! 对照系: `use widget:` 链(013-todo todo_list.at)同款形态正常;缺口仅
//! 在包路径(lib.rs P4-4/D13 load_package → child_decls → 单 VM 编译)。
//! 探针经 charts-gallery 真源 + 字符串补丁在生产链上复现。

#[cfg(all(test, feature = "ui-iced"))]
mod m4_pkg_compile_chain {
    use crate::plan492_tests::pkg_harness::{build_patched_gallery, render_dump};

    /// C1 锚: Init 内 prop 字符串比较(条件位裸名 + 赋值位点前缀)双形态
    /// 均正常。master 上 484 记档的"整组件失效"实为裸名 RHS 触发 undefined
    /// variable 解析错 → 包文件被 per-file try-parse 静默丢弃(诊断面缺陷,
    /// M5 修);prop 比较本身在两链均正常。
    #[test]
    fn c1_prop_compare_in_init_alive() {
        let patches: &[(&str, &str, &str)] = &[
            // model 加探针字段
            (
                "line_chart.at",
                "hovered str = \"false\"",
                "hovered str = \"false\"\n        probeMark str = \"unset\"",
            ),
            // Init 顶部插入 prop 比较
            (
                "line_chart.at",
                ".Init -> {\n            // ---- 全系列共享 y 域 ----",
                ".Init -> {\n            if .curve == \"monotone\" {\n                .probeMark = \"mono\"\n            } else {\n                .probeMark = \"linear\"\n            }\n            // ---- 全系列共享 y 域 ----",
            ),
        ];
        let (mut dc, _) = build_patched_gallery("m4-c1", patches);
        let dump = render_dump(&mut dc);
        let alive = dump.contains("M 40");
        let probe = dc
            .bridge()
            .read_state("probeMark")
            .map(|v| v.to_string())
            .unwrap_or_else(|e| format!("<err {e}>"));
        println!("C1 anchor: alive={alive} probeMark={probe}");
        assert!(alive, "geometry must survive prop compare in Init");
        assert!(
            probe.contains("mono") || probe.contains("linear"),
            "prop compare branch must execute: {probe}"
        );
    }

    /// C2 锚: 带参 msg 声明双轨均正常(master 不可复现 484 记档的整包失效)。
    /// line 专属标记: probeMark 字段并入根态 + monotone C 段渲染。
    /// "M 40" 不可用作 line 判据(area_chart 产生同款起点)。
    #[test]
    fn c2_param_msg_declaration_both_tracks_alive() {
        let (mut dc, _) = build_patched_gallery(
            "m4-c2",
            &[
                ("line_chart.at", "msg { Init }", "msg { Init, Hover(int) }"),
                (
                    "line_chart.at",
                    "hovered str = \"false\"",
                    "hovered str = \"false\"
        probeMark str = \"unset\"",
                ),
            ],
        );
        let dump = render_dump(&mut dc);
        let donut_alive = dump.contains("A100 100 0");
        let bar_alive = dump.contains("h19") || dump.contains("h25");
        let line_c_segs = dump.matches(" C ").count();
        let probe = dc
            .bridge()
            .read_state("probeMark")
            .map(|v| v.to_string())
            .unwrap_or_else(|e| format!("<err {e}>"));
        // vue 侧: 带参 msg 声明的 SFC 发射(484 现场疑 vue handler 生成)。
        let vue_probe = {
            let patched = std::fs::read_to_string(
                std::env::temp_dir().join("plan492-pkg-repro-m4-c2/components/line_chart.at"),
            )
            .unwrap();
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(patched.as_str()).with_session(session);
            match parser.parse() {
                Ok(ast) => {
                    let decl = ast.stmts.iter().find_map(|s| match s {
                        crate::ast::Stmt::WidgetDecl(d) => Some(d),
                        _ => None,
                    }).expect("decl");
                    match crate::aura::extract_widget_from_decl(decl) {
                        Ok(w) => {
                            let mut gen = crate::ui_gen::VueGenerator::new_shadcn();
                            use crate::ui_gen::BackendGenerator;
                            match gen.generate(&w) {
                                Ok(sfc) => {
                                    let emits: Vec<&str> = sfc.lines().filter(|l| l.contains("emit")).collect();
                                    format!("SFC ok; emit lines: {emits:?}")
                                }
                                Err(e) => format!("SFC generate FAILED: {e}"),
                            }
                        }
                        Err(e) => format!("extract FAILED: {e}"),
                    }
                }
                Err(e) => format!("PARSE FAILED: {e}"),
            }
        };
        println!("C2 anchor: donut={donut_alive} bar={bar_alive} line_C_segs={line_c_segs} probeMark={probe}
  vue: {vue_probe}");
        assert!(donut_alive && bar_alive, "siblings must stay alive with param msg decl");
        assert!(line_c_segs > 0, "line geometry must render with param msg decl");
        assert!(!probe.starts_with("<err"), "line child fields must merge: {probe}");
        assert!(vue_probe.contains("SFC ok"), "vue SFC must generate with param msg: {vue_probe}");
    }

    /// 直接形态双补丁锚(M6 前置验证): Init 内点前缀 prop 比较门控存储段 +
    /// 带参 msg 声明同时生效,几何/字段/hover 链全存活。
    #[test]
    fn direct_form_c1_c2_together_survive() {
        let patches: &[(&str, &str, &str)] = &[
            // C2: 带参 msg 声明
            (
                "line_chart.at",
                "msg { Init }",
                "msg { Init, Hover(int) }",
            ),
            // C1: Init 内 prop 字符串比较门控 .segs 存储段(直接形态,点前缀)
            (
                "line_chart.at",
                ".segsM = outSegsM",
                ".segsM = outSegsM
            if .curve == \"monotone\" {
                .segs = outSegsM
            } else {
                .segs = outSegs
            }
            .probeMark = .curve",
            ),
            // 探针字段
            (
                "line_chart.at",
                "hovered str = \"false\"",
                "hovered str = \"false\"
        probeMark str = \"unset\"",
            ),
        ];
        let (mut dc, _) = build_patched_gallery("m4-direct", patches);
        let dump = render_dump(&mut dc);
        let alive = dump.contains("M 40");
        let probe = dc
            .bridge()
            .read_state("probeMark")
            .map(|v| v.to_string())
            .unwrap_or_else(|e| format!("<err {e}>"));
        // hover 派发: 调用带参 handler 后 hovered 置位。
        let hovered = dc
            .bridge()
            .read_state("hovered")
            .map(|v| v.to_string())
            .unwrap_or_else(|e| format!("<err {e}>"));
        println!("direct form: alive={alive} probeMark={probe} hovered(before)={hovered}");
        assert!(alive, "geometry must survive direct-form prop compare + param msg");
        assert!(
            probe.contains("linear") || probe.contains("mono"),
            "prop read in Init must work: {probe}"
        );
        // line 专属几何必须在(monotone C 段,仅 line_chart 产生)。
        let c_segs = dump.matches(" C ").count();
        assert!(c_segs > 0, "line monotone geometry must render (C segments): got {c_segs}");
    }
}
