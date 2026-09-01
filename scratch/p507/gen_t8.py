# Plan 507 T8 —— splice parity matrix tests into client_runtime.rs tests module.
# Uses raw strings: patterns must match file bytes exactly (Rust multi-line
# string literals contain REAL newlines in parity_001).
import io
import sys

p = 'crates/auto-lang/src/ui/desktop_protocol/client_runtime.rs'
s = io.open(p, encoding='utf-8').read()

old_001 = r'''        let component = crate::build_dynamic_component(&src, None).expect("build");
        let mut p = AppProjector::new(component, 480.0, 900.0);
        let frame = p.render_frame();
        // 确定性文本形态（clear + 算子序列；坐标/颜色全精度锁）。
        let mut out = String::new();
        match frame.clear {
            Some(c) => out.push_str(&format!("clear {},{},{},{}
", c.r, c.g, c.b, c.a)),
            None => out.push_str("clear -
"),
        }
        for op in &frame.ops {
            match op {
                DrawOp::Quad { rect, color } => out.push_str(&format!(
                    "quad {:.1},{:.1} {:.1}x{:.1} {},{},{},{}
",
                    rect.x, rect.y, rect.w, rect.h, color.r, color.g, color.b, color.a
                )),
                DrawOp::Text { x, y, size, line_height, color, text } => out.push_str(&format!(
                    "text {:.1},{:.1} size={:.1} lh={:.1} {},{},{},{} {:?}
",
                    x, y, size, line_height, color.r, color.g, color.b, color.a, text
                )),
            }
        }
        let exp_path = dir.join("001_queue.expected.txt");'''

new_001 = r'''        let component = crate::build_dynamic_component(&src, None).expect("build");
        let mut p = AppProjector::new(component, 480.0, 900.0);
        let frame = p.render_frame();
        let out = drawlist_to_text(&frame);
        let exp_path = dir.join("001_queue.expected.txt");'''

if old_001 not in s:
    print('FAIL: parity_001 body not matched', file=sys.stderr)
    sys.exit(1)
s = s.replace(old_001, new_001)

fixture_t1_display = (
    r'"widget M1 {\n    model {\n        var pct double = 0.6\n        var who str = \"Jane Cooper\"\n    }\n'
    r'    view {\n        col {\n'
    r'            image (src: \"a.png\") { style: \"w-10 h-10\" }\n'
    r'            img (src: \"b.png\") { style: \"w-8 h-8\" }\n'
    r'            icon (name: \"star\", size: 20.0)\n'
    r'            badge \"New\"\n'
    r'            avatar (fallback: .who)\n'
    r'            progress (value: .pct) { style: \"w-40\" }\n'
    r'            divider { style: \"w-full\" }\n'
    r'            separator { style: \"w-full\" }\n'
    r'            spacer { style: \"h-2\" }\n'
    r'            a \"docs\"\n'
    r'            text \"plain\"\n'
    r'            span \"spanned\"\n'
    r'            button \"Go\" { onclick: .Go }\n'
    r'        }\n    }\n}\n"'
)
fixture_t1_form = (
    r'"widget M2 {\n    model {\n'
    r'        var ok bool = false\n        var on bool = true\n        var pick bool = false\n'
    r'        var note str = \"\"\n        var name str = \"\"\n'
    r'    }\n    view {\n        col {\n'
    r'            input (value: .name, placeholder: \"name\") { oninput: .N }\n'
    r'            checkbox (checked: .ok) { onclick: .T }\n'
    r'            switch (checked: .on) { onchange: .S }\n'
    r'            radio (checked: .pick) { onclick: .R }\n'
    r'            textarea (value: .note, placeholder: \"note\", rows: 2.0) {}\n'
    r'            button \"Submit\" { onclick: .T }\n'
    r'        }\n    }\n}\n"'
)
fixture_t1_layout = (
    r'"widget M3 {\n    view {\n        center {\n            card {\n'
    r'                cardheader { cardtitle { text \"T\" } }\n'
    r'                cardcontent {\n'
    r'                    grid (cols: 3.0, gap: 4.0) {\n'
    r'                        grid-item { text \"1\" }\n'
    r'                        grid-item { text \"2\" }\n'
    r'                        grid-item { text \"3\" }\n'
    r'                    }\n                }\n'
    r'                cardfooter { text \"F\" }\n'
    r'            }\n            row {\n'
    r'                container { text \"c\" }\n'
    r'                scroll { text \"s\" }\n'
    r'            }\n        }\n    }\n}\n"'
)
fixture_t2_typo = (
    r'"widget M4 {\n    view {\n        col {\n'
    r'            h1 \"h1\"\n            p \"para\"\n            label \"lbl\"\n'
    r'            b \"bold\"\n            strong \"strong\"\n            em \"em\"\n'
    r'            i \"it\"\n            small \"sm\"\n            code \"c = 1\"\n'
    r'            pre \"l1\\nl2\"\n'
    r'            blockquote \"q\"\n            heading \"H\"\n            figcaption \"cap\"\n'
    r'        }\n    }\n}\n"'
)
fixture_t2_sem = (
    r'"widget M5 {\n    view {\n        main {\n'
    r'            header { text \"hd\" }\n            nav { text \"nv\" }\n'
    r'            article {\n                section { text \"sec\" }\n'
    r'                figure { text \"fig\" }\n                aside { text \"as\" }\n            }\n'
    r'            ul { li { text \"i1\" } }\n            ol { li { text \"i2\" } }\n'
    r'            dl { dt { text \"t\" } dd { text \"d\" } }\n'
    r'            details { summary { text \"sum\" } }\n'
    r'            footer { text \"ft\" }\n'
    r'        }\n    }\n}\n"'
)

addition = r'''    /// 确定性文本形态（clear + 算子序列；坐标/颜色全精度锁）——parity
    /// 金样共用序列化（001 + 507 矩阵）。
    fn drawlist_to_text(frame: &DrawList) -> String {
        let mut out = String::new();
        match frame.clear {
            Some(c) => out.push_str(&format!("clear {},{},{},{}NL", c.r, c.g, c.b, c.a)),
            None => out.push_str("clear -NL"),
        }
        for op in &frame.ops {
            match op {
                DrawOp::Quad { rect, color } => out.push_str(&format!(
                    "quad {:.1},{:.1} {:.1}x{:.1} {},{},{},{}NL",
                    rect.x, rect.y, rect.w, rect.h, color.r, color.g, color.b, color.a
                )),
                DrawOp::Text { x, y, size, line_height, color, text } => out.push_str(&format!(
                    "text {:.1},{:.1} size={:.1} lh={:.1} {},{},{},{} {:?}NL",
                    x, y, size, line_height, color.r, color.g, color.b, color.a, text
                )),
            }
        }
        out
    }

    /// Plan 507 T8 —— parity 金样矩阵（覆盖表驱动抽样）：Tier1 每家族
    /// ≥1 条 + Tier2 每语义组 1 条；两阶段（初帧 → 家族规范交互 → 复帧）
    /// 全精度锁。`AUTO_WRITE_GOLDEN=1` 重写期望文件。
    #[test]
    fn parity_matrix_queue_golden() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test/parity/matrix");
        std::fs::create_dir_all(&dir).expect("mkdir matrix");
        for (name, src) in parity_matrix_fixtures() {
            let component =
                crate::build_dynamic_component(src, None).unwrap_or_else(|e| panic!("{name}: {e}"));
            let mut p = AppProjector::new(component, 480.0, 640.0);
            let mut out = String::new();
            out.push_str("---- frame 1 ----NL");
            out.push_str(&drawlist_to_text(&p.render_frame()));
            // 家族规范交互：首个命中区点击 + 输入框字符写入（交互闭环
            // 差分入样）。
            out.push_str("---- after input ----NL");
            if let Some((r, kind)) = p.hit_regions().first().cloned() {
                p.on_input(&InputMsg::PointerPressed {
                    wid: 1,
                    button: MouseButton::Left,
                    x: r.x + 2.0,
                    y: r.y + 2.0,
                    modifiers: 0,
                });
                if kind.starts_with("input:") {
                    p.on_input(&InputMsg::CharTyped { wid: 1, ch: 'x' });
                }
            }
            out.push_str(&drawlist_to_text(&p.render_frame()));
            let exp_path = dir.join(format!("{name}.expected.txt"));
            if std::env::var("AUTO_WRITE_GOLDEN").is_ok() || !exp_path.is_file() {
                std::fs::write(&exp_path, &out).expect("write golden");
            }
            let expected = std::fs::read_to_string(&exp_path)
                .unwrap_or_else(|e| panic!("read {name} golden: {e}"));
            if out != expected {
                let _ = std::fs::write(dir.join(format!("{name}.wrong.txt")), &out);
                panic!(
                    "{name} queue 金样不匹配（见 test/parity/matrix/{name}.wrong.txt）EXP---{expected}ACT---{out}"
                );
            }
        }
    }

    /// Plan 507 T8 防漏钉：矩阵夹具的扫描标签并集 ⊇ target_set 可投影集
    ///（kinds + layouts，视图构造 if 除外）——target_set 扩容必带矩阵
    /// 夹具，金样矩阵与覆盖表不脱钩。
    #[test]
    fn parity_matrix_covers_target_set() {
        use crate::ui::desktop_protocol::coverage::{judge, scan_view, Coverage, Verdict};
        let coverage = Coverage::target_set();
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for (_, src) in parity_matrix_fixtures() {
            let component = crate::build_dynamic_component(src, None).expect("build");
            let scan = scan_view(component.view_template());
            // 夹具本身必须 Covered（矩阵只收可上 queue 的形态）。
            assert!(
                matches!(judge(&scan, &coverage), Verdict::Covered),
                "矩阵夹具应 Covered: {:?}",
                scan.tags
            );
            seen.extend(scan.tags.iter().cloned());
        }
        for kind in &coverage.kinds {
            assert!(seen.contains(kind), "矩阵缺 {kind} 夹具（金样防漏）");
        }
        for layout in &coverage.layouts {
            if layout == "if" {
                continue; // 视图构造（001–005 if 已有行为金样）
            }
            assert!(seen.contains(layout), "矩阵缺 {layout} 夹具（金样防漏）");
        }
    }

    /// 矩阵夹具清单（name, 源）——Tier1 三族 + Tier2 两组；交互行为
    /// （首个命中区点击/输入）内建于 parity_matrix_queue_golden。
    fn parity_matrix_fixtures() -> &'static [(&'static str, &'static str)] {
        [
            ("t1_display", T1_DISPLAY),
            ("t1_form", T1_FORM),
            ("t1_layout_grid_card", T1_LAYOUT),
            ("t2_typography", T2_TYPO),
            ("t2_semantic", T2_SEM),
        ]
        .leak()
    }

'''

# NL marker -> Rust \n escape (kept as marker to survive raw strings above)
addition = addition.replace('NL', '\\n').replace('\\n---', '\\n---')
addition = addition.replace('T1_DISPLAY', fixture_t1_display)
addition = addition.replace('T1_FORM', fixture_t1_form)
addition = addition.replace('T1_LAYOUT', fixture_t1_layout)
addition = addition.replace('T2_TYPO', fixture_t2_typo)
addition = addition.replace('T2_SEM', fixture_t2_sem)

anchor = '''    /// Plan 500 步骤 9（T4）：queue 臂投影金样（001 三臂对拍基线的
    /// queue 臂；vue 臂挂 a2vue 同族，iced 像素臂留实机档——497 已证
    /// headless 栅格化不可行）。`AUTO_WRITE_GOLDEN=1` 重写期望文件。'''
assert anchor in s, 'anchor missing'
s = s.replace(anchor, addition + anchor)

with io.open(p, 'w', encoding='utf-8', newline='\n') as f:
    f.write(s)
print('T8 spliced ok')
