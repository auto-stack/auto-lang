//! PLAN-677 T-03: menubar 族 tag → vue 组件解析契约。
#[cfg(test)]
mod tests {
    use crate::ui_gen::widget::registry::WidgetRegistry;

    #[test]
    fn menubar_family_tags_resolve_vue_components() {
        let registry = WidgetRegistry::with_defaults();
        let expect = [
            ("menubar", Some("Menubar")),
            ("menubar-menu", Some("MenubarMenu")),
            ("menubar-trigger", Some("MenubarTrigger")),
            ("menubar-content", Some("MenubarContent")),
            ("menubar-item", Some("MenubarItem")),
            ("menubar-separator", Some("MenubarSeparator")),
            ("menubar-checkbox-item", Some("MenubarCheckboxItem")),
            ("menubar_menu", Some("MenubarMenu")),
            ("menubar_trigger", Some("MenubarTrigger")),
            ("menubar_content", Some("MenubarContent")),
            ("menubar_item", Some("MenubarItem")),
            ("menubar_separator", Some("MenubarSeparator")),
            ("menubar_checkbox_item", Some("MenubarCheckboxItem")),
        ];
        for (tag, want) in expect {
            let got = registry.get_primary_component("vue", tag);
            assert_eq!(got.as_deref(), want, "tag `{}` -> {:?}, got {:?}", tag, want, got);
        }
    }
}
