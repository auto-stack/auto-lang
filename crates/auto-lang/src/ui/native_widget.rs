//! PLAN-066: 原生组件外部注册 SPI（NativeWidgetRegistry）。
//!
//! VM/iced 渲染栈的原生富组件此前只能"编译期闭集"接入——aura_view_builder
//! 的硬编码字符串臂 + View 专属变体（提案 066 §2.1 证据表）。本模块给它们
//! 立**运行时注册通道**：
//!
//! - [`NativeWidgetEntry::View`]——派发期工厂（案 a）：组件可由既有 View 词
//!   汇表达，factory 拿 [`NativeCtx`] 直接产 `View<DynamicMessage>`；
//! - [`NativeWidgetEntry::Element`]——后端 lowering 期通道：派发期产
//!   `View::Custom { name, .. }`，iced renderer 查表取 Element factory（首个
//!   后端原生件落地时消费；本计划只落通道与防御臂）。
//!
//! 语义模板 = Plan 435 ui_gen `ComponentRegistry`（Builtin 优先 / 同名注册拒
//! 绝记 violation / 注册面可 dump）；名字折叠查找语义 = `WidgetRegistry::get`
//! （P435 P8-6：剥 `-`/`_` + 小写，折叠兜底只在直接 miss 时参与）。
//!
//! "外部"语义 = Cargo 可选依赖的外部 crate + 运行时注册（autodown-core path
//! 依赖先例），不做 dylib。内置硬编码臂优先级不变：注册表在全部内置臂穷尽后
//! 才被查询（提案 §3.3 派发序）。

use std::collections::HashMap;
use std::sync::OnceLock;

use auto_val::Value;

use super::aura_view_builder::AuraViewBuilder;
use crate::aura::{AuraEvent, AuraNode, AuraPropValue};
use crate::ui::interpreter::DynamicMessage;
use crate::ui::view::View;

/// 派发期上下文：builder 侧解析助手 + 当前节点的原始面。
///
/// factory 内部经 `ctx.builder` 调用 builder 的 pub(crate) 转换助手
/// （extract/resolve/event 面），与内置臂同源取数——语义保形的根基。
pub struct NativeCtx<'s, 'a> {
    pub builder: &'s AuraViewBuilder<'a>,
    pub tag: &'s str,
    pub props: &'s HashMap<String, AuraPropValue>,
    pub events: &'s HashMap<String, AuraEvent>,
    pub children: &'s [AuraNode],
    pub bindings: &'s HashMap<String, Value>,
}

/// 派发期 View 工厂（[`NativeWidgetEntry::View`] 载荷）。
///
/// 返回 `None` = 本工厂声明放弃该节点（回落后续派发：.at AuraWidget → 折叠
/// 兜底 → 未知 tag 兜底），供"有条件接手"的组件用。
pub type NativeViewFactory = fn(&NativeCtx) -> Option<View<DynamicMessage>>;

/// 原生组件注册入口。
#[derive(Debug, Clone)]
pub enum NativeWidgetEntry {
    /// 派发期 View 工厂（案 a：既有 View 词汇表达的组件）。
    View(NativeViewFactory),
    /// 后端 lowering 期通道：派发产 `View::Custom`，iced 查 Element factory。
    Element,
}

impl NativeWidgetEntry {
    /// 注册面 kind 名（dump/门检用）。
    pub fn kind(&self) -> &'static str {
        match self {
            NativeWidgetEntry::View(_) => "view",
            NativeWidgetEntry::Element => "element",
        }
    }
}

/// 原生组件注册表。
///
/// 同名重注册**拒绝**并记 violation（435 口径：注册冲突不静默）；查找折叠语
/// 义与 `WidgetRegistry::get` 同规。
#[derive(Debug, Default)]
pub struct NativeWidgetRegistry {
    entries: HashMap<String, NativeWidgetEntry>,
    violations: Vec<String>,
}

impl NativeWidgetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册派发期 View 工厂。同名已注册（直接键命中）时拒绝并记 violation，
    /// 返回 false。
    pub fn register_view(&mut self, name: &str, factory: NativeViewFactory) -> bool {
        self.register_entry(name, NativeWidgetEntry::View(factory))
    }

    /// 注册后端 lowering 期 Element 通道入口。同名拒绝语义同上。
    pub fn register_element(&mut self, name: &str) -> bool {
        self.register_entry(name, NativeWidgetEntry::Element)
    }

    fn register_entry(&mut self, name: &str, entry: NativeWidgetEntry) -> bool {
        if self.entries.contains_key(name) {
            self.violations.push(format!(
                "native widget `{}` already registered ({})",
                name,
                self.entries[name].kind()
            ));
            return false;
        }
        self.entries.insert(name.to_string(), entry);
        true
    }

    /// 查找：直接键优先；miss 时折叠兜底（剥 `-`/`_` + 小写，P8-6 同规——
    /// 无分隔符且无大写的缩合小写词不做折叠全扫）。
    pub fn lookup(&self, name: &str) -> Option<&NativeWidgetEntry> {
        if let Some(e) = self.entries.get(name) {
            return Some(e);
        }
        if !name.contains('-') && !name.contains('_') && !name.chars().any(|c| c.is_uppercase()) {
            return None;
        }
        let want = fold_name(name);
        self.entries
            .iter()
            .find(|(k, _)| fold_name(k) == want)
            .map(|(_, e)| e)
    }

    /// 直接键命中判定（折叠语义不含）。
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// 注册冲突台账（violation 不静默，435 口径）。
    pub fn violations(&self) -> &[String] {
        &self.violations
    }

    /// 注册面 dump（name, kind），按名字排序——门检/AUTOUI 清单消费。
    pub fn dump(&self) -> Vec<(String, &'static str)> {
        let mut out: Vec<(String, &'static str)> = self
            .entries
            .iter()
            .map(|(k, e)| (k.clone(), e.kind()))
            .collect();
        out.sort();
        out
    }
}

fn fold_name(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_lowercase()
}

static GLOBAL: OnceLock<NativeWidgetRegistry> = OnceLock::new();

/// 进程级内置注册表：feature 门控的原生组件在此登记（builder 字段缺席时的
/// 兜底源；测试经 `AuraViewBuilder` 的 native_registry 字段注入自有实例）。
pub fn global() -> &'static NativeWidgetRegistry {
    GLOBAL.get_or_init(register_builtin_entries)
}

fn register_builtin_entries() -> NativeWidgetRegistry {
    let mut reg = NativeWidgetRegistry::new();
    #[cfg(all(feature = "autodown", feature = "code-editor"))]
    {
        // PLAN-066 T2：autodown_editor 自硬编码臂迁出——案 a sugar（factory
        // 内部仍产 View::AutodownEditor，快照 kind 与断言面零改动）。
        // 原臂双拼写（"autodown_editor" | "autodowneditor"）→ 双直接键：
        // 压缩小写别名不在 P8-6 折叠兜底范围（无分隔符无大写不折叠），必须
        // 显式注册才不回归。
        reg.register_view("autodown_editor", |ctx| {
            Some(ctx.builder.convert_autodown_editor_native(ctx.props, ctx.events, ctx.bindings))
        });
        reg.register_view("autodowneditor", |ctx| {
            Some(ctx.builder.convert_autodown_editor_native(ctx.props, ctx.events, ctx.bindings))
        });
    }
    #[cfg(not(all(feature = "autodown", feature = "code-editor")))]
    {
        // 无 feature 降级链保持：原臂的 `#[cfg(not(...))] textarea` 分支迁经
        // 注册表（D-GAP-3 textarea 降级语义不变）。双拼写同上。
        reg.register_view("autodown_editor", |ctx| {
            Some(ctx.builder.convert_textarea(ctx.props, ctx.events, ctx.bindings))
        });
        reg.register_view("autodowneditor", |ctx| {
            Some(ctx.builder.convert_textarea(ctx.props, ctx.events, ctx.bindings))
        });
    }
    #[cfg(test)]
    {
        // PLAN-066 T1 语料探针：Element 通道（仅测试构建注册；nextest 每测
        // 独立进程，无跨测泄漏面）。
        reg.register_element("plan066_element_probe");
    }
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_factory(_ctx: &NativeCtx) -> Option<View<DynamicMessage>> {
        None
    }

    #[test]
    fn register_and_lookup_view_entry() {
        let mut reg = NativeWidgetRegistry::new();
        assert!(reg.register_view("my_widget", stub_factory));
        assert!(matches!(reg.lookup("my_widget"), Some(NativeWidgetEntry::View(_))));
        assert!(reg.contains("my_widget"));
        assert!(reg.violations().is_empty());
    }

    #[test]
    fn register_element_entry() {
        let mut reg = NativeWidgetRegistry::new();
        assert!(reg.register_element("canvas_thing"));
        assert!(matches!(reg.lookup("canvas_thing"), Some(NativeWidgetEntry::Element)));
        assert_eq!(reg.lookup("canvas_thing").unwrap().kind(), "element");
    }

    #[test]
    fn duplicate_registration_rejected_with_violation() {
        let mut reg = NativeWidgetRegistry::new();
        assert!(reg.register_view("dup", stub_factory));
        assert!(!reg.register_view("dup", stub_factory));
        assert!(!reg.register_element("dup"));
        assert_eq!(reg.violations().len(), 2);
        assert!(reg.violations()[0].contains("already registered (view)"));
        assert!(reg.violations()[1].contains("already registered (view)"));
    }

    #[test]
    fn fold_lookup_kebab_and_camel_hit_canonical() {
        let mut reg = NativeWidgetRegistry::new();
        reg.register_view("MyWidget", stub_factory);
        assert!(reg.lookup("my-widget").is_some());
        assert!(reg.lookup("My_Widget").is_some());
        // 缩合小写词不做折叠兜底（P8-6 同规）。
        assert!(reg.lookup("mywidget").is_none());
    }

    #[test]
    fn unknown_name_misses() {
        let reg = NativeWidgetRegistry::new();
        assert!(reg.lookup("nope").is_none());
        assert!(reg.lookup("nope-thing").is_none());
    }

    #[test]
    fn dump_is_sorted_with_kinds() {
        let mut reg = NativeWidgetRegistry::new();
        reg.register_element("b_elem");
        reg.register_view("a_view", stub_factory);
        let dump = reg.dump();
        assert_eq!(
            dump,
            vec![
                ("a_view".to_string(), "view"),
                ("b_elem".to_string(), "element"),
            ]
        );
    }

    #[cfg(all(feature = "autodown", feature = "code-editor"))]
    #[test]
    fn global_registers_autodown_editor() {
        let g = global();
        assert!(matches!(g.lookup("autodown_editor"), Some(NativeWidgetEntry::View(_))));
        // 折叠别名命中同一入口。
        assert!(g.lookup("AutodownEditor").is_some());
        assert!(g.lookup("autodown-editor").is_some());
        assert!(g.violations().is_empty());
    }
}
