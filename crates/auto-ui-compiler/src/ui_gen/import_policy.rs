//! PLAN-814 T-05：生成代码 import 路径策略单源（§5.3）。
//!
//! **canonical 规则**：Rust 生成物（`RustGenerator`）只 import 两个新包——
//! UI 面 `auto_ui_runtime::ui::*`、语言/VM 面 `auto_lang_core::*`。旧
//! `auto_lang::*` 门面继续可用（上层兼容包再导出，V-02 旧路径），但生成
//! 器不再发射旧前缀（A 期"指向当期真实实现"的过渡形态就此收口）。
//!
//! **新旧路径显式映射**（golden 变更须对照本表，不得以更新 golden 掩盖）：
//!
//! | 旧（A 期发射） | canonical（T-05 起） | owner |
//! |---|---|---|
//! | `auto_lang::ui::view::*` | `auto_ui_runtime::ui::view::*` | runtime |
//! | `auto_lang::ui::{Component, View}` 等类型 | `auto_ui_runtime::ui::{..}` | runtime |
//! | `auto_lang::ui::style::*` | `auto_ui_runtime::ui::style::*`（runtime 再导出 compiler 单源） | compiler→runtime 面 |
//! | `auto_lang::ui::code_editor/scroll/action_config/dialog/frame_bench/iced/auto_val::*` | `auto_ui_runtime::ui::*` | runtime |
//! | `auto_lang::vm::{ffi,native,ui_console}::*` | `auto_lang_core::vm::*` | SDK |
//! | `auto_lang::a2r_std::*` | `auto_lang_core::a2r_std::*` | SDK |
//!
//! **生成工程 manifest 约束**：canonical import 要求生成工程 manifest 直接
//! 依赖 `auto-ui-runtime`（features 含 `ui-iced`）与 `auto-lang-core`（extern
//! 名可见性）；模板发射方（auto-man rust_ui / 本包 e2e harness）随本策略同步。
//!
//! vue/jet/ark 三生成器不发射 Rust import，不在本策略射面内。

/// canonical runtime crate 名（UI 运行域）。
pub const RUNTIME_CRATE: &str = "auto_ui_runtime";
/// canonical runtime UI 前缀（生成物 UI 面唯一入口）。
pub const RUNTIME_UI: &str = "auto_ui_runtime::ui";
/// canonical SDK crate 名（语言/VM 面）。
pub const CORE_CRATE: &str = "auto_lang_core";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::{AuraMessage, AuraStateDef, AuraWidget};
    use crate::ui_gen::rust::RustGenerator;
    use auto_lang_core::ast::Type;
    use std::collections::HashMap;

    /// PLAN-814 T-05 路径策略 pin：生成物必须走 canonical 新前缀，且零
    /// `auto_lang::` 残留（旧包门面仍可用，但生成器不再指向——回归到旧
    /// 前缀=本测试红，golden 差异须按模块头映射表解释）。
    #[test]
    fn plan814_t05_generated_imports_are_canonical_only() {
        let widget = AuraWidget {
            name: "PolicyPin".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: auto_lang_core::ast::Expr::Int(0),
                decorators: vec![],
            }],
            computed: vec![],
            messages: vec![AuraMessage { variants: vec![] }],
            view_tree: crate::aura::AuraNode::element("button")
                .with_prop("label", auto_lang_core::ast::Expr::Str("ok".into())),
            named_views: Vec::new(),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
            actions: None,
            timers: Vec::new(),
        };
        let mut gen = RustGenerator::new();
        let code = gen.generate_rust(&widget).expect("rust generation");
        assert!(
            code.contains(&format!("use {RUNTIME_UI}::{{Component, View}};")),
            "missing canonical UI import header:\n{code}"
        );
        assert!(
            !code.contains("auto_lang::"),
            "generated code must not reference the legacy auto_lang facade:\n{code}"
        );
        // 常量与发射前缀一致性（策略单源自检）。
        assert_eq!(RUNTIME_UI, "auto_ui_runtime::ui");
        assert_eq!(CORE_CRATE, "auto_lang_core");
    }
}
