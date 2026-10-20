//! `UiParseContext` —— 完整 UI 文法的解析端口（PLAN-810 T-01 冻结；
//! PLAN-814 R3-03b 自 auto-ui-dialect 迁入本 crate）。
//!
//! [`ParseContext`](crate::ParseContext)（PLAN-801 轻端口）只覆盖游标/
//! checkpoint/expr/type 委托，不足以表达完整 UI 文法所需的：prev 位置、
//! 精确 `expect_ident` 语义、全 token 原始前看、双 token 回推、**原始游标
//! 投机**（无旗标/无诊断回滚，区别于 checkpoint）、raw CSS 块捕获、
//! skip_check/in_on_body 旗标、局部诊断事务（computed object→body 回退）、
//! scope/绑定事务、body/stmt/closure/object/fstr/dot_item 核心委托、
//! docs/warning/样式配方注册副作用。本 trait 是其**UI 伴随扩展**
//! （子 trait）。
//!
//! **位置沿革**：PLAN-810 §2 曾裁定本 trait 留在 auto-ui-dialect（避免向
//! 既有外部 consumer 强加实现面）。PLAN-814 R3-03b 起语言核心 SDK 的
//! 清单（含 optional path）不得再引用 auto-ui-dialect——纯 SDK 源码快照
//! 消费（无 UI checkout）要求消除该边；而 SDK 的内置 UI 方言派发与
//! `ParserBridge` 端口 impl 必须能命名本 trait（孤儿规则：impl 只能落在
//! trait 所属 crate 或类型所属 crate）。故迁至本 crate——两个既有消费者
//! （auto-ui-dialect 文法、auto-lang-core 桥）本就依赖本 crate，类型身份
//! 不变（auto-ui-dialect 经 re-export 保旧路径）。PLAN-810 裁定的**理据**
//! 保持：本 trait 仍是独立 opt-in 子 trait，`ParseContext` 实现方不被强加
//! 任何实现面。
//!
//! 由语言核心 `ParserBridge` 实现（PLAN-810 起经端口薄委托）；完整 UI
//! 文法（widget/store/scene/view/component/fragment/on/actions/style…）
//! 只经本端口消费这些能力，**不获得 Parser/lexer/TypeStore/
//! InferenceContext 引用**。事务合同逐项冻结见
//! `docs/plans/attachments/810/contract-baseline.md` §3：
//!
//! - `probe_next_token`：成功路径零副作用即回推（与 806 `probe_next_kind`
//!   同协议的全 token 形态）；失败路径遵循旧回推前瞻失败消费协议，词法
//!   错误**原样上抛**（memo/paren `?` 位点）；if-let-Ok 位点用 `.ok()`。
//! - `raw_cursor_save/restore`：只存/回 lexer 全状态 + cur/prev——**不含**
//!   五旗标、不回滚 errors/warnings/pending_docs，与旧臂投机位点
//!   （widget 近失/computed/style recipe）逐字等价；中间 `advance` 的
//!   docs 收集与词法错误记录保留。不得用 checkpoint 替代（checkpoint
//!   额外恢复旗标），也不得反向替代方言 None 回滚（派发器专用）。
//! - `diagnostics_mark/truncate`：**仅限** raw 投机回退位点的局部诊断
//!   事务（computed）；checkpoint 路径诊断恒单调。
//! - `capture_raw_block`：调用前置条件=cur 为 LBrace 且 lexer 无缓冲
//!   token（旧臂同位）；返回 owned 原始字节（嵌套括号/字符串/注释/
//!   换行/Unicode 由 Lexer 保持），闭 `}` 由捕获消费。
//! - `register_style_recipe`：解析期全局副作用（Core design_tokens
//!   注册表），仅成功构造 decl 后调用一次。

use crate::ast::{Body, Expr, ModelBlock, MsgDecl, OnEvents, Pair, Stmt, StyleRecipeDecl, Type};
use crate::context::ParseCheckpoint;
use crate::context::ParseContext;
use crate::error::AutoResult;
use crate::token::{Pos, Token, TokenKind};

/// 完整 UI 文法可见的最小解析端口（PLAN-810；对象安全，无泛型方法）。
pub trait UiParseContext: ParseContext {
    // ---- 游标（超出 801 端口的部分） ----

    /// 前一个已消费 token 的源位置（span 定位/起点算术）。
    fn prev_pos(&mut self) -> Pos;

    /// 期望当前 token 文本等于 `expected` 并消费；否则返回语法错误。
    /// 语义与旧 `Parser::expect_ident`（私有）逐字一致：纯文本比较、
    /// 无 kind 检查、错误文本 "Expected 'x', got 'y'"。
    fn expect_ident(&mut self, expected: &str) -> AutoResult<()>;

    /// 当前 token 是否软标识符（旧 `Parser::cur_is_soft_ident` 同语义）。
    fn cur_is_soft_ident(&mut self) -> bool;

    /// **原始单 token 前看（probe 全 token 形态）**：词法成功则取回
    /// owned token 并立即回推（零副作用、幂等），失败则词法错误原样
    /// 上抛（失败推进由 Lexer 失败模式决定——见模块文档）。
    fn probe_next_token(&mut self) -> AutoResult<Token>;

    /// 回推一个 owned token 到流前部（供双 token 回推再重同步的旧臂
    /// lookahead 模式：scene 门/typed subresource/on stream 前看）。
    fn push_back_token(&mut self, tok: Token);

    /// **注释跳过前看**（旧臂 `save cur → next() → 观察 → push cur 回缓冲 +
    /// 手动恢复 cur` 模式的具名化——**on stream 位点形态**）：执行一次完整
    /// advance（含词法错误记录、EOF 兜底、docs 收集、注释跳过），返回结果
    /// token 的 kind，然后把该 token 回推缓冲并将 cur 恢复到 advance 前。
    /// prev 保持 advance 后的值（旧臂泄漏形态：prev=被前看的 token）。
    fn advance_peek_kind(&mut self) -> TokenKind;

    /// **注释跳过前看·双 push 重同步**（旧臂 `save cur → next() → 观察 →
    /// push observed → push saved → next() 重同步` 模式的具名化——
    /// **scene 门/typed subresource 位点形态**，F810-R1-04 区分）：与
    /// [`Self::advance_peek_kind`] 的差异仅在 **prev** 的终值——重同步的
    /// 第二次 next 使 prev=被观察 token（手动恢复形态则是 prev=advance
    /// 前 的 cur）。cur/缓冲终态两者一致。
    fn advance_peek_kind_resync(&mut self) -> TokenKind;

    // ---- 原始游标投机（无旗标/无诊断回滚；与 checkpoint 的区别见模块文档） ----

    /// 快照 lexer 全状态 + cur/prev（凭据复用 801 句柄表后端：实例 id +
    /// 槽位 + 单调 gen + 方言调用作用域失效，一次性）。
    fn raw_cursor_save(&mut self) -> ParseCheckpoint;

    /// 回滚到 raw 快照：仅 lexer + cur/prev（**不**恢复五旗标、**不**回滚
    /// 诊断/docs）。凭据校验与 `ParseContext::restore` 同规则（跨上下文/
    /// 过期/重复使用受控拒绝）。
    fn raw_cursor_restore(&mut self, cp: ParseCheckpoint) -> AutoResult<()>;

    /// **释放** raw 快照凭据（F810-R1-01）：投机成功（不再需要回滚）时
    /// 结束快照生命周期——仅失效句柄表槽位（tombstone），**不**恢复游标
    /// /旗标、**不**回滚诊断。与 `restore` 同为一次性：已消费/过期/跨
    /// 上下文的凭据为无操作（Drop 语义）。旧臂局部 `save_state` 在返回
    /// 时即释放；本方法把同一"出作用域即释放"带到端口形态，覆盖公开
    /// 门面、Core 直连与 builtin 派发全部入口（不依赖派发器退出清理）。
    /// 只失效给定凭据（实例 id + 槽位 + 代次校验），不影响外层存活快照。
    fn raw_cursor_release(&mut self, cp: ParseCheckpoint);

    // ---- raw source ----

    /// 捕获 LBrace 到配对 RBrace 的原始源字节（raw CSS 块）。前置条件与
    /// 后继位置见模块文档；闭 `}` 由捕获消费，后继首 token 由调用方推进。
    fn capture_raw_block(&mut self) -> AutoResult<String>;

    // ---- 语句级上下文旗标（保存-置位-恢复三段式由文法侧复刻） ----

    /// 置 skip_check 并返回旧值（投机表达式解析禁用符号检查）。
    fn set_skip_check(&mut self, value: bool) -> bool;

    /// 置 in_on_body 并返回旧值（on 处理器体内护栏）。
    fn set_in_on_body(&mut self, value: bool) -> bool;

    // ---- 局部诊断事务（仅 raw 投机回退位点） ----

    /// 当前 (errors, warnings) 长度快照。
    fn diagnostics_mark(&self) -> (usize, usize);

    /// 截断 errors/warnings 到 mark 位（computed object→body 回退专用；
    /// 除此之外诊断单调追加）。
    fn diagnostics_truncate(&mut self, mark: (usize, usize));

    // ---- scope/绑定事务（infer_ctx 符号面；不返回存储引用） ----

    fn enter_scope(&mut self);
    fn exit_scope(&mut self);

    /// 绑定变量（fragment/component 参数、for 循环变量/索引）。
    fn bind_var(&mut self, name: &str, ty: Type);

    /// 定义 on 处理器参数符号（桥内构造与旧臂一致的 Meta::Store(Var)）。
    fn define_handler_param(&mut self, name: &str);

    // ---- 核心文法委托（单一份算法留 Core） ----

    /// 解析语句块体（旧 `Parser::body`）。
    fn parse_body(&mut self) -> AutoResult<Body>;

    /// 解析单条语句（setup 块体）。
    fn parse_stmt(&mut self) -> AutoResult<Stmt>;

    /// 解析闭包字面量（内联事件 handler）。
    fn parse_closure(&mut self) -> AutoResult<Expr>;

    /// 解析对象字面量对列表（computed 投机 object 臂；旧 `Parser::object`）。
    fn parse_object_pairs(&mut self) -> AutoResult<Vec<Pair>>;

    /// 解析 f-string（视图文本插值；旧 `Parser::fstr`）。
    fn parse_fstr(&mut self) -> AutoResult<Expr>;

    /// 解析 `.field` 项（视图属性值；旧 `Parser::dot_item`）。
    fn parse_dot_item(&mut self) -> AutoResult<Expr>;

    /// 解析顶层 on-events（Core 共享文法，非 UI 回退同源）。
    fn parse_on_events(&mut self) -> AutoResult<OnEvents>;

    // ---- docs / warning / 注册副作用 ----

    /// 取走累计 doc 注释（join "\\n"；旧 `Parser::take_docs`）。
    fn take_docs(&mut self) -> Option<String>;

    /// 近失块关键字告警（span 取当前 token 位置——桥面与旧臂一致）。
    fn warn_suspicious_block_keyword(&mut self, found: String, suggestion: String);

    /// 注册样式配方（Core design_tokens 注册表的解析期副作用）。
    fn register_style_recipe(&mut self, decl: &StyleRecipeDecl);
}

// ---- PLAN-814 R3-03b：UI 文法函数表端口 ----
//
// 语言核心 SDK（auto-lang-core）不再清单依赖 auto-ui-dialect 后，其内置
// UiDialect 派发与 Parser 公开 UI 门面经本函数表间接调用文法单源实现：
// 表项即 auto-ui-dialect 文法函数的直接函数指针（签名字面一致——文法
// 函数本就取 `&mut dyn UiParseContext`），由上层组合点（旧 auto-lang
// 兼容包 / auto-ui-compiler）装配注入。表缺席（纯 SDK）语义与原
// `ui-dialect` feature off 逐位一致：方言表为空、公开门面显式拒绝。

/// UI 文法函数表——SDK 侧派发端口（PLAN-814 R3-03b）。
///
/// 各字段与 auto-ui-dialect 对应文法函数签名字面一致；装配方
/// （auto-ui-dialect::ui_grammar()）为单一份构造器。
#[derive(Clone, Copy)]
pub struct UiGrammar {
    /// `widget` 声明（grammar::widget::parse_widget_decl）。
    pub parse_widget_decl: fn(&mut dyn UiParseContext) -> AutoResult<Stmt>,
    /// `msg` 声明（declarations::parse_msg_decl——PLAN-806 轻端口面）。
    pub parse_msg_decl: fn(&mut dyn ParseContext) -> AutoResult<MsgDecl>,
    /// `model` 块（declarations::parse_model_block——PLAN-806 轻端口面）。
    pub parse_model_block: fn(&mut dyn ParseContext) -> AutoResult<ModelBlock>,
    /// `component fn` 声明（grammar::view::parse_component_fn_decl）。
    pub parse_component_fn_decl: fn(&mut dyn UiParseContext) -> AutoResult<Stmt>,
    /// 顶层 `actions` 声明（grammar::actions::parse_actions_decl_stmt）。
    pub parse_actions_decl_stmt: fn(&mut dyn UiParseContext) -> AutoResult<Option<Stmt>>,
    /// 顶层 `style` 配方声明（grammar::actions::parse_style_recipe_decl_stmt）。
    pub parse_style_recipe_decl_stmt: fn(&mut dyn UiParseContext) -> AutoResult<Option<Stmt>>,
    /// `view` 块（grammar::view::parse_view_block）。
    pub parse_view_block: fn(&mut dyn UiParseContext) -> AutoResult<Stmt>,
    /// `view` 块或 fragment（grammar::view::parse_view_block_or_fragment）。
    pub parse_view_block_or_fragment: fn(&mut dyn UiParseContext) -> AutoResult<Stmt>,
    /// `store` 声明（grammar::store::parse_store_decl）。
    pub parse_store_decl: fn(&mut dyn UiParseContext) -> AutoResult<Stmt>,
    /// `scene` 声明（grammar::scene::parse_scene_decl）。
    pub parse_scene_decl: fn(&mut dyn UiParseContext) -> AutoResult<Stmt>,
    /// scene 门前瞻（grammar::scene::looks_like_scene_decl）。
    pub looks_like_scene_decl: fn(&mut dyn UiParseContext) -> bool,
}
