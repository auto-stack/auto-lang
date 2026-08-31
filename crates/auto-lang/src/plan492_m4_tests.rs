//! Plan 492 M4 (族 C): 包组件单 VM 编译链分叉定位与修复。
//!
//! 定案(2026-08-30): 不存在 codegen 分叉——包链与 use-widget 链同 Parser
//! 同合成器。484 记档的"prop 比较/带参 msg 破坏编译"实为:
//! ①裸 prop 名在赋值 RHS 位触发 undefined variable 解析错;
//! ②包文件被 `parse_package_widgets` per-file try-parse 静默整文件丢弃
//! (诊断面缺陷,M5 已修)。
//! 点前缀 `.curve` 直接形态与带参 msg 声明在两链均正常。
//! M6 后组件回归直接写法——本模块以无补丁基线钉住三副本直接形态。

#[cfg(all(test, feature = "ui-iced"))]
mod m4_pkg_compile_chain {
    use crate::plan492_tests::pkg_harness::{build_patched_gallery, render_dump};

    /// 三副本组件目录(cargo 测试 cwd 在 crate 下,经 CARGO_MANIFEST_DIR 定位)。
    fn copies() -> Vec<std::path::PathBuf> {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        [
            "../../examples/charts-gallery/src/front/components",
            "../../examples/ui/024-charts/src/front/components",
            "../../examples/widgets-gallery/src/front/components",
        ]
        .iter()
        .map(|rel| {
            let p = std::path::Path::new(rel).to_path_buf();
            if p.exists() {
                p
            } else {
                std::path::Path::new(&manifest).join(rel)
            }
        })
        .collect()
    }

    /// C1 锚(无补丁基线): 三副本 Init 内原生 prop 字符串比较直接形态
    /// (`if .curve == "monotone"` / `if .type == "stacked"`),双算字段
    /// (segsM/segsS)清零,渲染产物几何存活且 monotone 分支实际执行。
    #[test]
    fn c1_prop_compare_in_init_alive() {
        for rel in copies() {
            let line_src = std::fs::read_to_string(rel.join("line_chart.at")).unwrap();
            let bar_src = std::fs::read_to_string(rel.join("bar_chart.at")).unwrap();
            assert!(
                line_src.contains("if .curve == \"monotone\" {"),
                "{}/line_chart.at must carry direct-form prop compare in Init",
                rel.display()
            );
            assert!(
                bar_src.contains("if .type == \"stacked\" {"),
                "{}/bar_chart.at must carry direct-form prop compare in Init",
                rel.display()
            );
            assert!(
                !line_src.contains("segsM List") && !bar_src.contains("segsS List"),
                "{}: dual-store fields must be gone",
                rel.display()
            );
        }
        let (mut dc, _) = build_patched_gallery("m4-c1", &[]);
        let dump = render_dump(&mut dc);
        let alive = dump.contains("M 40");
        let c_segs = dump.matches(" C ").count();
        println!("C1 anchor: alive={alive} line_C_segs={c_segs}");
        assert!(alive, "geometry must survive prop compare in Init");
        assert!(c_segs > 0, "monotone branch must execute (C segments): {c_segs}");
    }

    /// C2 锚(无补丁基线): 三副本带参 msg 声明原生恢复(`msg { Init,
    /// Hover(int) }`),VM 渲染存活 + vue SFC 生成含 emit 派发。
    /// Plan 498: 声明行随交互扩展增长(HoverSeries/SeriesOut 追加),
    /// 锚改前缀匹配(意图 = 带参形态存在,非全串冻结)。
    #[test]
    fn c2_param_msg_declaration_both_tracks_alive() {
        for rel in copies() {
            for file in ["line_chart.at", "bar_chart.at", "area_chart.at", "donut_chart.at"] {
                let src = std::fs::read_to_string(rel.join(file)).unwrap();
                assert!(
                    src.contains("msg { Init, Hover(int)"),
                    "{}/{} must declare the param msg form",
                    rel.display(),
                    file
                );
            }
        }
        // VM 轨: 整包渲染存活(同族几何 + sibling 图不连坐)。
        let (mut dc, _) = build_patched_gallery("m4-c2", &[]);
        let dump = render_dump(&mut dc);
        let donut_alive = dump.contains("A100 100 0");
        let bar_alive = dump.contains("h19") || dump.contains("h25");
        let line_c_segs = dump.matches(" C ").count();
        println!(
            "C2 anchor: donut={donut_alive} bar={bar_alive} line_C_segs={line_c_segs}"
        );
        assert!(donut_alive && bar_alive, "siblings must stay alive");
        assert!(line_c_segs > 0, "line geometry must render with param msg decl");

        // vue 轨: SFC 生成含 emit('Hover', i)。
        let front = crate::plan370_test_support::locate_example_app_at("charts-gallery")
            .expect("gallery")
            .parent()
            .expect("front")
            .to_path_buf();
        let code = std::fs::read_to_string(front.join("components/line_chart.at")).unwrap();
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let w = crate::aura::extract_widget_from_decl(decl).expect("extract");
        let mut gen = crate::ui_gen::VueGenerator::new_shadcn();
        use crate::ui_gen::BackendGenerator;
        let sfc = gen.generate(&w).expect("vue SFC generate");
        assert!(
            sfc.contains("emit('Hover', i)"),
            "vue SFC must emit the param Hover event"
        );
    }

    /// M6 摘绕开验收 grep 锚: 全部 chart 组件源内绕开痕迹清零
    /// (双算字段/双域 yTickS 组/裸挂带参 handler 的 msg 形态)。
    #[test]
    fn m6_workaround_anchors_gone() {
        for rel in copies() {
            for file in ["line_chart.at", "bar_chart.at", "area_chart.at", "donut_chart.at"] {
                let src = std::fs::read_to_string(rel.join(file)).unwrap();
                let name = format!("{}/{}", rel.display(), file);
                assert!(!src.contains("segsM List"), "{name}: segsM gone");
                assert!(!src.contains("segsS List"), "{name}: segsS gone");
                assert!(!src.contains("yTickS4"), "{name}: dual tick set gone");
                assert!(
                    !src.contains("msg { Init }\n"),
                    "{name}: bare `msg {{ Init }}` workaround gone"
                );
                assert!(
                    !src.contains("s: \"h-full flex-1\""),
                    "{name}: static band style workaround gone"
                );
            }
        }
    }

}
