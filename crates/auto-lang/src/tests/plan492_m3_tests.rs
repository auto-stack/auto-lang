//! Plan 492 M3 (族 B): vue 生成器文本内容位置的 Index/Dot 表达式求值臂。
//!
//! 探针枚举: 哪些表达式形式在 `text (text: …)` 位置失灵(B1 现场是循环内
//! `text (text: li["name"])` 渲染空/dump)。

#[cfg(test)]
mod m3_vue_text_content_arms {
    fn gen_widget(view_body: &str) -> crate::aura::AuraWidget {
        let src = format!(
            r##"
widget W {{
    model {{ items = [{{ name: "a", label: "L" }}] }}
    view {{
        col {{
{view_body}
        }}
    }}
}}
"##
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        crate::aura::extract_widget_from_decl(decl).expect("extract")
    }

    fn gen_sfc_with_warnings(view_body: &str) -> (String, Vec<String>) {
        let widget = gen_widget(view_body);
        let mut gen = crate::ui_gen::VueGenerator::new_shadcn();
        use crate::ui_gen::BackendGenerator;
        let sfc = gen.generate(&widget).expect("generate SFC");
        let warnings = gen
            .last_validation_warnings
            .iter()
            .map(|w| format!("{} {}", w.rule, w.message))
            .collect();
        (sfc, warnings)
    }

    fn gen_sfc(view_body: &str) -> String {
        let src = format!(
            r##"
widget W {{
    model {{ items = [{{ name: "a", label: "L" }}] }}
    view {{
        col {{
{view_body}
        }}
    }}
}}
"##
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract");
        let mut gen = crate::ui_gen::VueGenerator::new_shadcn();
        use crate::ui_gen::BackendGenerator;
        gen.generate(&widget).expect("generate SFC")
    }

    fn template_of(sfc: &str) -> String {
        let start = sfc.find("<template>").map(|i| i + 10).unwrap_or(0);
        let end = sfc.find("</template>").unwrap_or(sfc.len());
        sfc[start..end].trim().to_string()
    }

    /// B1 复现(红): 循环内 `text (text: li["name"])` 必须发射带引号的
    /// 字符串索引 `li['name']`(现状剥引号 → `li[name]` 裸标识符 → 渲染空)。
    #[test]
    fn text_index_string_key_keeps_quotes() {
        let sfc = gen_sfc("for li in .items { text (text: li[\"name\"]) { } }");
        let tpl = template_of(&sfc);
        assert!(
            tpl.contains("li['name']"),
            "string index must keep quotes in text content; got:
{tpl}"
        );
        assert!(
            !tpl.contains("li[name]"),
            "bare-identifier index (undefined) must be gone; got:
{tpl}"
        );
    }

    /// 简写形态经 M2 修复后同链路也必须带引号。
    #[test]
    fn text_index_shorthand_keeps_quotes() {
        let sfc = gen_sfc("for li in .items { text li[\"name\"] { } }");
        let tpl = template_of(&sfc);
        assert!(
            tpl.contains("li['name']"),
            "shorthand index must keep quotes; got:
{tpl}"
        );
    }

    /// 数值/标识符索引不回归: li[0] / li[k] 保持裸形态。
    #[test]
    fn text_index_int_and_ident_unchanged() {
        let sfc = gen_sfc("for li in .items { text (text: li[0]) { } }");
        let tpl = template_of(&sfc);
        assert!(tpl.contains("li[0]"), "int index stays bare; got:
{tpl}");
    }

    /// 不支持形式在文本位置必须发 R046 告警(替换静默 dump)。
    #[test]
    fn unsupported_text_form_warns_r046() {
        // 数组字面量是 text_raw 无臂的形式 → 落 fallback。
        let (sfc, warnings) = gen_sfc_with_warnings("text (text: [1, 2]) { }");
        let r046: Vec<&String> = warnings.iter().filter(|w| w.starts_with("R046")).collect();
        assert!(
            !r046.is_empty(),
            "unsupported text-content form must raise R046; warnings: {warnings:?}
sfc:
{sfc}"
        );
    }

    /// 探针: 各表达式形式在 text 内容位置的发射现状。
    #[test]
    fn probe_text_content_forms() {
        let cases: Vec<(&str, &str)> = vec![
            (
                "index-paren-prop",
                "for li in .items { text (text: li[\"name\"]) { } }",
            ),
            (
                "dot-paren-prop",
                "for li in .items { text (text: li.name) { } }",
            ),
            (
                "index-shorthand",
                "for li in .items { text li[\"name\"] { } }",
            ),
            (
                "dot-shorthand",
                "for li in .items { text li.name { } }",
            ),
            (
                "binary-paren-prop",
                "for li in .items { text (text: li[\"name\"] + \"!\") { } }",
            ),
            (
                "index-state",
                "text (text: .items[0][\"name\"]) { }",
            ),
        ];
        for (name, body) in cases {
            let sfc = gen_sfc(body);
            let tpl = template_of(&sfc);
            println!("--- {name} ---\n{tpl}\n");
        }
    }
}
