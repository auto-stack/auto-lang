//! pac.at 展示面声明提取轻规则（PLAN-831 T-05 C2 切点）。
//!
//! `theme{}` 块（PLAN-601 T-03 形态）与 `back`（Plan 061 external backend）
//! 的 root 提取规则自 auto-man-core/src/pac.rs 逐字单源迁入；Man 侧
//! `pac_from_decl` 对应字段改为委派本模块（零复制、行为恒等——Pac::new
//! 对这两个字段本就是 decl 字段直拷）。UI 工具/API 生成经本模块直取，
//! 不再依赖 Man 仓实现。

use crate::generation_request::ThemeDeclLite;
use auto_val::{AutoStr, Node, Value};

/// `theme{}` 块声明提取（Obj 形态；标量与块同现时块胜；缺席 → None）。
pub fn theme_decl_from_root(root: &Node) -> Option<ThemeDeclLite> {
    match root.get_prop("theme") {
        Value::Obj(obj) => {
            let extends = obj
                .get("extends")
                .map(|v| v.to_astr().trim().to_string())
                .filter(|s| !s.is_empty());
            let mode = obj
                .get("mode")
                .map(|v| v.to_astr().trim().to_lowercase())
                .filter(|s| !s.is_empty());
            let name = obj
                .get("name")
                .map(|v| v.to_astr().trim().to_string())
                .filter(|s| !s.is_empty());
            let colors = match obj.get("colors") {
                Some(Value::Obj(co)) => co
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_astr().trim().to_string()))
                    .filter(|(_, v)| !v.is_empty())
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            };
            Some(ThemeDeclLite {
                name,
                extends,
                mode,
                colors,
            })
        }
        _ => None,
    }
}

/// `back` 声明提取（external backend 工程：Obj 形态取 `project` 键，
/// 标量形态原值；空值 → None）。
pub fn external_backend_from_root(root: &Node) -> Option<AutoStr> {
    match root.get_prop("back") {
        Value::Obj(obj) => obj
            .get("project")
            .map(|v| v.to_astr().trim().to_string())
            .filter(|s| !s.is_empty())
            .map(AutoStr::from),
        v => {
            let s = v.to_astr().trim().to_string();
            (!s.is_empty()).then(|| AutoStr::from(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root_with(prop: &str, value: Value) -> Node {
        let mut root = Node::new("root");
        root.set_prop(prop, value);
        root
    }

    /// C2 正反控（PLAN-831 T-05）：块形态全字段、覆盖/缺省、Obj.project
    /// 与标量两形态、缺席——与 auto-man-core pac.rs 既有 oracle 同值
    /// （pac_from_decl 委派本模块后两径同源）。
    #[test]
    fn theme_decl_block_full_and_partial() {
        let mut colors = auto_val::Obj::new();
        colors.set("primary", Value::Str(AutoStr::from(" #123456 ")));
        colors.set("empty", Value::Str(AutoStr::from("  ")));
        let mut theme = auto_val::Obj::new();
        theme.set("name", Value::Str(AutoStr::from("brand")));
        theme.set("extends", Value::Str(AutoStr::from(" dark ")));
        theme.set("mode", Value::Str(AutoStr::from("Dark")));
        theme.set("colors", Value::obj(colors));
        let d = theme_decl_from_root(&root_with("theme", Value::obj(theme))).unwrap();
        assert_eq!(d.name.as_deref(), Some("brand"));
        assert_eq!(d.extends.as_deref(), Some("dark"));
        assert_eq!(d.mode.as_deref(), Some("dark"), "mode 归一小写");
        assert_eq!(d.colors, vec![("primary".to_string(), "#123456".to_string())]);

        let mut bare = auto_val::Obj::new();
        bare.set("name", Value::Str(AutoStr::from("x")));
        let d2 = theme_decl_from_root(&root_with("theme", Value::obj(bare))).unwrap();
        assert_eq!(d2.extends, None);
        assert!(d2.colors.is_empty());
    }

    #[test]
    fn theme_decl_absent_and_scalar_form() {
        let root = Node::new("root");
        assert!(theme_decl_from_root(&root).is_none(), "缺席 → None");
        // 标量 theme（Plan 458 激活名）不属块声明——块面 None（标量面归
        // theme 字段，见 pac_from_decl），块胜规则不受影响。
        let scalar = root_with("theme", Value::Str(AutoStr::from("dark")));
        assert!(theme_decl_from_root(&scalar).is_none());
    }

    #[test]
    fn external_backend_obj_scalar_absent() {
        let mut back = auto_val::Obj::new();
        back.set("project", Value::Str(AutoStr::from(" ../svc ")));
        let d = external_backend_from_root(&root_with("back", Value::obj(back)));
        assert_eq!(d.map(|s| s.to_string()), Some("../svc".to_string()));
        // Obj 无 project 键 → None（不落空串）
        let d2 = external_backend_from_root(&root_with("back", Value::obj(auto_val::Obj::new())));
        assert!(d2.is_none());
        // 标量形态
        let d3 = external_backend_from_root(&root_with("back", Value::Str(AutoStr::from("svc2"))));
        assert_eq!(d3.map(|s| s.to_string()), Some("svc2".to_string()));
        // 空标量/缺席
        let d4 = external_backend_from_root(&root_with("back", Value::Str(AutoStr::from(" "))));
        assert!(d4.is_none());
        assert!(external_backend_from_root(&Node::new("root")).is_none());
    }
}
