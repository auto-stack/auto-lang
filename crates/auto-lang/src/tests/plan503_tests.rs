//! Plan 503：桌面视觉体系刷新——引擎侧能力回归。
//!
//! M4（launcher 品牌色图标底块）依赖的 style 串循环成员插值
//! `${r.field}`（VM + vue 双端）此前双端皆缺：VM 的
//! resolve_literal_interpolation_with 与 vue 的 interpolated_class_parts
//! 均只认 `${.state}` 前导点形态。本文件钉死该能力 + 品牌色 8 位 hex
//! arbitrary 类（bg-[#rrggbbaa]）解析。

#[cfg(all(test, feature = "ui-iced"))]
mod loop_member_style_interp {
    /// 循环成员 style 插值（VM 轨）：`for r in .rows` 循环体内
    /// `style: "h-10 w-10 ${r.chip}"` 应解析为行对象字段值。
    #[test]
    fn vm_loop_member_style_interpolates() {
        let src = r#"widget App {
    model {
        var rows = []
    }
    view {
        col {
            style: "w-full h-full"
            for r in .rows {
                div { style: "h-10 w-10 ${r.chip}" }
            }
        }
    }
}
"#;
        let mut comp = crate::build_dynamic_component(src, None).expect("compile");
        let entries = vec![auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
            (
                "chip",
                // rounded-[10px] VM 侧无 arbitrary 半径档,块级用 rounded-xl 双端等价。
                auto_val::Value::Str("bg-[#7c9a6d21] rounded-xl".into()),
            ),
        ])))];
        let _ = comp.write_state_vec("rows", entries);
        let (view, _, _) = comp.view_with_debug_gated(false);
        let rendered = format!("{view:?}");
        assert!(
            rendered.contains("BackgroundColor(Hex(2090495265))") && rendered.contains("RoundedXl"),
            "循环成员 style 插值应展开为 8 位 hex 品牌色类(0x7C9A6D21)，实际: {rendered}"
        );
        assert!(
            !rendered.contains("${r.chip}"),
            "未解析的插值模板不应残留在视图: {rendered}"
        );
    }

    /// 状态字段插值回归（既有 `${.field}` 形态不回归）。
    #[test]
    fn vm_state_field_style_interpolation_no_regress() {
        let src = r#"widget App {
    model {
        var chip str = "bg-[#7c9a6d21]"
    }
    view {
        col {
            style: "w-full h-full ${.chip}"
            text "ok"
        }
    }
}
"#;
        let mut comp = crate::build_dynamic_component(src, None).expect("compile");
        let _ = comp.write_state("chip", auto_val::Value::str("bg-[#7c9a6d21]"));
        let (view, _, _) = comp.view_with_debug_gated(false);
        let rendered = format!("{view:?}");
        assert!(
            rendered.contains("BackgroundColor(Hex(2090495265))"),
            "状态字段插值应保持解析: {rendered}"
        );
    }
}

#[cfg(test)]
mod vue_loop_member_class {
    /// 循环成员 class 插值（vue 轨）：interpolated_class_parts 应把
    /// `${r.chip}` 拆为静态段 + JS 表达式（v-for 作用域内求值）。
    #[test]
    fn vue_loop_member_class_parts_split() {
        let s = "h-10 w-10 rounded-[10px] ${r.chip}";
        // 静态段原样保留;表达式段 = v-for 成员点路径。
        let (statics, expr) = crate::ui_gen::vue::VueGenerator::interpolated_class_parts(s)
            .expect("loop-member interpolation must split");
        assert_eq!(statics, vec!["h-10 w-10 rounded-[10px]".to_string()]);
        assert_eq!(expr, "'h-10 w-10 rounded-[10px]' + r.chip");
    }

    /// 既有 `${.field}` 形态不回归。
    #[test]
    fn vue_state_field_class_parts_no_regress() {
        let (_, expr) =
            crate::ui_gen::vue::VueGenerator::interpolated_class_parts("w-full ${.chip}")
                .expect("state interpolation must split");
        assert_eq!(expr, "'w-full' + chip");
    }
}

/// Plan 503 M4：launcher 重写回归——真 028 app.at 经 build_dynamic_component
/// 管线编译（include_str 只内嵌 pack,examples 的 .at 无编译门禁,语法回归
/// 此前只能实机才发现）+ ApplyFilter/PickCat 行为断言。
/// PLAN-590（Stage B P-5）：028-launcher 迁 auto-os/apps/——语料经解析序
/// 定位（os_paths 家族）；solo 检出 SKIP 不炸。
#[cfg(all(test, feature = "ui-iced"))]
mod launcher_rewrite {
    fn build_launcher() -> Option<crate::ui::dynamic::DynamicComponent> {
        let sibling_base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let path = crate::os_paths::resolve_os_top_dir(&sibling_base, "apps")
            .map(|apps| apps.join("auto-launcher/src/front/app.at"));
        let code = match path.map(|p| std::fs::read_to_string(&p)) {
            Some(Ok(code)) => code,
            _ => {
                eprintln!(
                    "plan503 launcher_rewrite: SKIPPED — auto-os/apps/auto-launcher\
                     未解析(solo 检出;设 AUTO_OS_ROOT 或并置 auto-os 兄弟检出可启用)"
                );
                return None;
            }
        };
        Some(crate::build_dynamic_component(&code, None).expect("launcher app.at 编译"))
    }

    #[test]
    fn launcher_compiles_and_renders_brand_chips() {
        let Some(mut comp) = build_launcher() else {
            return;
        };
        let _ = comp.write_state("visible", auto_val::Value::str("1"));
        comp.bridge_mut()
            .call_handler("ApplyFilter", &[])
            .expect("ApplyFilter");
        let (view, _, _) = comp.view_with_debug_gated(false);
        let rendered = format!("{view:?}");
        // 首枚 mock(011-calculator #7c9a6d)品牌底块进视图(13% alpha 8 位 hex)。
        assert!(
            rendered.contains("BackgroundColor(Hex(2090495265))"),
            "品牌色图标底块应进入视图,实际(截断): {:.800}",
            rendered
        );
        // 硬编码暗色(gray-800/900)清除后,选中行应为语义 accent-light(Rgba a=38)。
        assert!(
            !rendered.contains("bg-gray-800") && !rendered.contains("bg-gray-900"),
            "不应残留硬编码 gray 底类: {:.800}",
            rendered
        );
    }


    /// PLAN-867 T-12：oracle 对齐 launcher app.at 现行形态。空查询分支的
    /// quicklinks（provider="quicklinks"，category="link"）不做分类过滤
    /// （模型默认 ql_admit="1"），PickCat("editor") 忠实执行 = 1 个 editor
    /// app + 3 个 quicklink（nres=4）。钉定提交 464e2316b 对照探针逐行
    /// 相同（同为 4 行，apps_names/apps_cats 状态一致）——该差异是外部
    /// launcher app.at 输入态漂移 + 早期 VM 执行不忠实的组合，非拆分回归。
    /// 断言维持原意图（editor 分类下 app 仅 041-auto-edit 一枚），对
    /// quicklink 行不敏感。
    fn ranked_str_field(fields: &auto_val::Obj, key: &str) -> String {
        match fields.get(key) {
            Some(auto_val::Value::Str(s)) => s.to_string(),
            Some(auto_val::Value::String(s)) => s.to_string(),
            other => format!("{other:?}"),
        }
    }

    #[test]
    fn launcher_pickcat_filters_results() {
        let Some(mut comp) = build_launcher() else {
            return;
        };
        let _ = comp.write_state("visible", auto_val::Value::str("1"));
        comp.bridge_mut()
            .call_handler("ApplyFilter", &[])
            .expect("ApplyFilter");
        comp.bridge_mut()
            .call_handler("PickCat", &[auto_val::Value::str("editor")])
            .expect("PickCat");
        let ranked = comp
            .bridge()
            .read_all_state_materialized()
            .get("ranked")
            .cloned()
            .expect("ranked 可读");
        let auto_val::Value::Array(rows) = ranked else {
            panic!("ranked 应为数组");
        };
        let editor_apps: Vec<String> = rows
            .iter()
            .filter_map(|row| {
                let auto_val::Value::Obj(fields) = comp.materialize_obj_value(row) else {
                    return None;
                };
                if ranked_str_field(&fields, "provider") == "apps"
                    && ranked_str_field(&fields, "category") == "editor"
                {
                    Some(ranked_str_field(&fields, "name"))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            editor_apps,
            vec!["041-auto-edit".to_string()],
            "editor 分类下 app 仅 041-auto-edit 一枚（quicklink 行不计）"
        );
    }
}
