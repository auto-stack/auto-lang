//! `UiParseContext` —— 完整 UI 文法的解析端口（PLAN-810 T-01 冻结）。
//!
//! trait 单一定义在 **auto-parser-api**（`auto_parser_api::UiParseContext`，
//! PLAN-814 R3-03b 迁入——语言核心 SDK 经函数表派发消费本 crate 文法，
//! 清单不再引用本 crate；孤儿规则下端口 trait 须落在双侧共同依赖的轻
//! 端口 crate）。本模块保留 [`UiParseContext`] re-export——crate 内文法
//! 子模块与外部旧路径（`auto_ui_dialect::grammar::UiParseContext`）均零
//! 改动，类型身份不变。
//!
//! [`ParseContext`]（PLAN-801 轻端口）只覆盖游标/checkpoint/expr/type
//! 委托，不足以表达完整 UI 文法所需的：prev 位置、精确 `expect_ident`
//! 语义、全 token 原始前看、双 token 回推、**原始游标投机**（无旗标/
//! 无诊断回滚，区别于 checkpoint）、raw CSS 块捕获、skip_check/
//! in_on_body 旗标、局部诊断事务（computed object→body 回退）、
//! scope/绑定事务、body/stmt/closure/object/fstr/dot_item 核心委托、
//! docs/warning/样式配方注册副作用。完整事务合同逐项冻结（probe/raw
//! 游标/诊断事务/capture/register）见 trait 定义处文档与
//! `docs/plans/attachments/810/contract-baseline.md` §3。
//!
//! 由语言核心 `ParserBridge` 实现；完整 UI 文法（widget/store/scene/
//! view/component/fragment/on/actions/style…）只经本端口消费这些能力，
//! **不获得 Parser/lexer/TypeStore/InferenceContext 引用**。
//!
//! PLAN-814 R3-03b：本模块同时提供 [`ui_grammar()`]——SDK 函数表端口
//! [`auto_parser_api::UiGrammar`] 的**单一份构造器**（各字段即本 crate
//! 文法函数的直接指针）；上层组合点（旧 auto-lang 兼容包 /
//! auto-ui-compiler）经 inventory/显式装配注入 SDK。

pub use auto_parser_api::UiParseContext;

pub mod actions;
pub mod helpers;
pub mod scene;
pub mod store;
pub mod view;
pub mod widget;

use auto_parser_api::UiGrammar;

/// SDK UI 文法函数表的单一份构造器（PLAN-814 R3-03b）。
///
/// 表项与本 crate 文法函数签名字面一致；语言核心 SDK（auto-lang-core）
/// 的内置 UiDialect 派发与 Parser 公开 UI 门面经该表间接调用。上层
/// 组合点装配：`auto_lang_core::set_ui_grammar(auto_ui_dialect::ui_grammar())`
/// 或经 SDK 的 `UiGrammarSource` inventory 自注册。
pub fn ui_grammar() -> UiGrammar {
    UiGrammar {
        parse_widget_decl: widget::parse_widget_decl,
        parse_msg_decl: crate::declarations::parse_msg_decl,
        parse_model_block: crate::declarations::parse_model_block,
        parse_component_fn_decl: view::parse_component_fn_decl,
        parse_actions_decl_stmt: actions::parse_actions_decl_stmt,
        parse_style_recipe_decl_stmt: actions::parse_style_recipe_decl_stmt,
        parse_view_block: view::parse_view_block,
        parse_view_block_or_fragment: view::parse_view_block_or_fragment,
        parse_store_decl: store::parse_store_decl,
        parse_scene_decl: scene::parse_scene_decl,
        looks_like_scene_decl: scene::looks_like_scene_decl,
    }
}
