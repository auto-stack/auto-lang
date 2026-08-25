//! Widget Registry for the Auto UI Interpreter
//!
//! Stores child widget definitions loaded from `use` imports,
//! enabling the interpreter to render custom component tags.

use std::collections::HashMap;
use crate::aura::AuraWidget;

/// Registry mapping widget names to their AuraWidget definitions.
#[derive(Debug, Default)]
pub struct WidgetRegistry {
    widgets: HashMap<String, AuraWidget>,
    /// Plan 408: route module → widget name alias map.
    /// Maps e.g. "button" → "ButtonPage" so render_outlet can find the page
    /// widget by route.module. Stored separately from `widgets` to avoid
    /// name collisions with built-in UI elements (e.g. a route module named
    /// "button" must not shadow the `<button>` element).
    route_aliases: HashMap<String, String>,
}

impl WidgetRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            route_aliases: HashMap::new(),
        }
    }

    /// Register a widget definition.
    pub fn register(&mut self, widget: AuraWidget) {
        self.widgets.insert(widget.name.clone(), widget);
    }

    /// Plan 408: Register a route module alias (module name → widget name).
    /// Used by render_outlet to look up page widgets by route.module.
    pub fn register_route_alias(&mut self, module: &str, widget_name: &str) {
        self.route_aliases.insert(module.to_string(), widget_name.to_string());
    }

    /// Plan 408: Look up a widget by route module name via the alias map.
    /// Returns the widget definition if the module alias exists and the
    /// target widget is registered.
    pub fn get_by_route_module(&self, module: &str) -> Option<&AuraWidget> {
        let widget_name = self.route_aliases.get(module)?;
        self.widgets.get(widget_name)
    }

    /// Look up a widget by name.
    ///
    /// Plan 435 P8-6(D13):精确未命中时按折叠键兜底(剥 `-`/`_` + 小写,
    /// 与 vue.rs map_tag 的折叠桥接同语义)—— kebab tag(copy-button)在
    /// iced 端也能命中 CopyButton,不再落 `<copy-button />` 文本占位。
    /// 内置优先不受影响:调用方(aura_view_builder 派发)先查内置臂,
    /// 折叠兜底只在未知 tag 分支生效。
    pub fn get(&self, name: &str) -> Option<&AuraWidget> {
        if let Some(w) = self.widgets.get(name) {
            return Some(w);
        }
        let fold = |s: &str| -> String {
            s.chars()
                .filter(|c| *c != '-' && *c != '_')
                .collect::<String>()
                .to_lowercase()
        };
        // 组件形态守卫(与 vue.rs tag_has_component_shape 同规):无分隔符
        // 且无大写的缩合小写词不是组件形态,不做折叠兜底(避免误聚 + O(n) 全扫)。
        if !name.contains('-')
            && !name.contains('_')
            && !name.chars().any(|c| c.is_uppercase())
        {
            return None;
        }
        let want = fold(name);
        self.widgets
            .values()
            .find(|w| fold(&w.name) == want)
    }

    /// Check if a widget with the given name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.widgets.contains_key(name)
    }

    /// Number of registered widgets.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    /// Whether the registry is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    /// Plan 320: iterate all registered widgets (for single-VM compilation).
    pub fn all(&self) -> impl Iterator<Item = AuraWidget> + '_ {
        self.widgets.values().cloned()
    }
}
