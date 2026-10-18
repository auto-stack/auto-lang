//! `ParseContext` —— 方言端口的对象安全解析桥面（PLAN-801）。
//!
//! 本 trait 是 PLAN-778 `ParserBridge` 公开面的 dyn 形态：外部
//! [`crate::ExternalDialect`] 实现经 `&mut dyn ParseContext` 看到与旧桥面
//! 等价的最小能力（游标 / checkpoint / core expr/type 委托 / 场景与语句
//! 上下文查询 / 语法错误构造），不接触 `Parser` 本体、lexer、scope/符号表
//! 或 db/type_store——这些不在外部契约中。
//!
//! 由 auto-lang 在上层实现（`ParserBridge<'p, 'a>` 实现本 trait）并经
//! `ExternalDialectAdapter` 接入生产派发。泛型便利方法（如
//! `syntax_error(impl Into<String>)`）留在上层具体门面上；本端口使用
//! 对象安全参数形态（`syntax_error(String)`）。
//!
//! ## checkpoint：有作用域句柄表（T-01 D3）
//!
//! [`ParseContext::checkpoint`] 返回**不透明凭据** [`ParseCheckpoint`]，
//! 而非携带源码生命周期的快照值。凭据合法性由实现侧校验：
//!
//! - **跨上下文**：凭据绑定发起方 parse 上下文（Parser 实例）；在别的
//!   上下文上 `restore` 得到受控拒绝（`Err`，语法错误形态）。
//! - **过期/重复使用**：凭据一次性（restore 后即失效）；方言调用退出后
//!   由派发器按调用水位清表——跨调用持有的凭据再 restore 同样受控拒绝。
//!   代次号全局单调不重复，杜绝槽位复用的误接受。
//! - **嵌套/递归**：每次 checkpoint 是独立快照槽；内层恢复不覆盖外层快照，
//!   外层凭据在内层调用期间保持可用（水位截断只发生在本调用边界）。
//!
//! 禁止 unsafe 延寿、源码 'static 要求、全局快照池（T-01 冻结）。

use auto_ast::ast::{Expr, Type};
use auto_diagnostics::error::{AutoError, AutoResult};
use auto_lex::token::{Pos, Token, TokenKind};

use crate::session::Scenario;

/// `ExternalDialect` 实现方可见的最小解析端口（对象安全，PLAN-778 桥面
/// 的 dyn 形态）。实现方经 `&mut dyn ParseContext` 消费，不感知 Parser。
pub trait ParseContext {
    // ---- 游标 ----

    /// 当前 token 的 TokenKind。
    fn cur_kind(&mut self) -> TokenKind;

    /// 当前 token 文本（owned，脱离借用）。
    fn cur_text(&mut self) -> String;

    /// 当前 token 源位置。
    fn cur_pos(&mut self) -> Pos;

    /// 当前 token 是否为 `kind`。
    fn is_kind(&mut self, kind: TokenKind) -> bool;

    /// 前进一个 token，返回被消费的 token。
    fn advance(&mut self) -> Token;

    /// 期望当前 token 为 `kind` 并消费；否则返回语法错误。
    fn expect(&mut self, kind: TokenKind) -> AutoResult<()>;

    /// 期望当前 token 为文本等于 `word` 的标识符并消费；否则返回语法错误。
    fn expect_keyword(&mut self, word: &str) -> AutoResult<()>;

    /// 跳过空行（含注释性空行），返回跳过行数。进入块体解析循环时的标准动作。
    fn skip_empty_lines(&mut self) -> usize;

    /// **原始单 token 探测（probe，非纯 peek）**：返回当前 token 之后的
    /// 下一个**原始** token 的 `TokenKind`（不经注释跳过 / pending docs
    /// 收集 / 词法错误记录的 token 面上看）。
    ///
    /// - **成功路径（词法成功）**：token 立即回推流中，游标与诊断**零
    ///   副作用**，连续调用幂等；随后的 [`ParseContext::advance`] /
    ///   [`ParseContext::expect`] 重新读到同一 token 并正常执行其全部
    ///   副作用（注释跳过 / docs 收集 / 错误记录）。
    /// - **失败路径（词法错误）**：**遵循旧回推前瞻的失败消费协议**
    ///   （F806-R3-01 冻结）——词法失败是否推进输入由 Lexer 各失败
    ///   模式自身决定（如字符字面量非法转义会消费失败前缀、未知单
    ///   字符不消费）；该错误**不记录**（返回 `None`）。因此失败路径
    ///   **不承诺**游标不变或连续调用幂等——重复调用可能得到不同
    ///   结果，后续消费从失败模式留下的位置继续、该错误已丢弃。
    ///   这与旧 `lexer.next()`+`push_token` 前瞻的错误语义逐字一致
    ///   （错误丢弃 + 失败推进），声明文法依赖它保持旧臂行为，不得
    ///   在此偷偷恢复状态或补记错误。
    ///
    /// Core 含义（PLAN-806 具名通用能力，F806-R2-01 引入、R3-01 定形）：
    /// 消费前的一次性投机探测面（如 msg named payload 的 `Ident :` 判定）。
    /// 实现方承诺：成功路径零副作用/幂等/与首次 `advance` 消费的 token
    /// 一致；失败路径如实暴露其失败消费结果（含非幂等性）。
    fn probe_next_kind(&mut self) -> Option<TokenKind>;

    // ---- checkpoint（投机解析 / None 回滚） ----

    /// 快照当前游标（lexer 状态 + cur/prev token）与语句级上下文旗标，
    /// 返回不透明凭据。凭据在本次方言调用内有效；跨上下文/过期使用由
    /// [`ParseContext::restore`] 受控拒绝（见模块文档）。
    fn checkpoint(&mut self) -> ParseCheckpoint;

    /// 回滚到 checkpoint 状态：游标（lexer + cur/prev）与语句级 context
    /// 旗标一并恢复。诊断单调追加（不回滚 errors/warnings）。凭据一次性；
    /// 非本上下文或已过期/已消费的凭据返回语法错误（受控拒绝）。
    fn restore(&mut self, cp: ParseCheckpoint) -> AutoResult<()>;

    // ---- core expr/type（委托核心解析器） ----

    /// 用核心类型文法解析一个类型（`int` / `str` / `NavPathStack` / 泛型等）。
    fn parse_type(&mut self) -> AutoResult<Type>;

    /// 用核心表达式文法解析一个表达式（Pratt 优先级完整）。
    fn parse_expr(&mut self) -> AutoResult<Expr>;

    // ---- context ----

    /// 当前编译场景。
    fn scenario(&self) -> Scenario;

    /// 是否在 UI 场景。
    fn is_ui_scenario(&self) -> bool;

    /// 是否在 on 处理器体内（内建方言在此护栏下让行，外部方言应镜像）。
    fn in_on_body(&self) -> bool;

    /// 是否在 fn 体内。
    fn in_fn_body(&self) -> bool;

    // ---- diagnostics ----

    /// 以当前 token 位置构造一个通用语法错误。外部方言的报错出口：
    /// 返回 Err 由核心错误收集机制收纳（`errors` + `error_limit` +
    /// synchronize）。对象安全参数形态（`String`）；泛型便利方法
    /// （`impl Into<String>`）留在上层具体门面。
    ///
    /// （端口不做通用 warning 出口：`Warning` 枚举无通用变体，不为此扩
    /// 枚举；外部诊断统一走 Err 路径——与 PLAN-778 桥面裁定一致。）
    fn syntax_error(&self, message: String) -> AutoError;
}

/// checkpoint 的不透明凭据（有作用域句柄，T-01 D3）。
///
/// 字段私有；外部只作不透明令牌在 `checkpoint()`/`restore()` 间传递。
/// 值语义为 move-only（不 Clone/Copy）：跨上下文（别的 Parser/别的方言
/// 调用）或过期（调用退出清表、已 restore、槽位代次更替）使用在
/// `restore` 处受控拒绝并返回语法错误。无 unsafe、无源码 'static 要求、
/// 无全局快照池；快照数据与生命周期由实现侧（auto-lang Parser）持有。
#[derive(Debug)]
pub struct ParseCheckpoint {
    /// 发起方 parse 上下文身份（Parser 实例 id；跨上下文拒绝）。
    parser_id: u64,
    /// 实现侧句柄表槽位下标。
    slot: u32,
    /// 本次签发的全局单调代次（过期/复用拒绝；不重复故无误接受）。
    gen: u64,
}

impl ParseCheckpoint {
    /// 端口实现侧构造（auto-lang adapter；`#[doc(hidden)]`——凭据的效力
    /// 完全来自 `restore` 侧校验，构造本身不授予任何恢复能力）。
    #[doc(hidden)]
    pub fn new(parser_id: u64, slot: u32, gen: u64) -> Self {
        Self {
            parser_id,
            slot,
            gen,
        }
    }

    /// 端口实现侧读取（restore 校验用；与构造同属 `#[doc(hidden)]` 内部面）。
    #[doc(hidden)]
    pub fn parts(&self) -> (u64, u32, u64) {
        (self.parser_id, self.slot, self.gen)
    }
}
