//! 外部方言实现面（PLAN-801 依赖倒置后的新接口）。
//!
//! [`ExternalDialect`] 是 crate 边界外的方言实现 trait：实现方只依赖本
//! crate（轻端口），经 auto-lang 的 `Parser::register_external_dialect`
//! 注册进**与旧 [`auto_lang::dialect::Dialect`] 同一张**三段有序派发表
//! （pre → builtin → post，PLAN-778 冻结语义），由 auto-lang 的
//! `ExternalDialectAdapter` 做协议转换——不维护另一套行为不同的派发表。
//!
//! 恢复不变量由派发器兜底保证（与 PLAN-778 相同，非方言自律）：
//! - `Ok(Some(stmt))`：本方言已处理，产出 stmt；
//! - `Ok(None)`：本次不归我管——派发器强制 `restore` 回滚游标+语句上下文
//!   旗标，不吞 token，继续问下一个方言；
//! - `Err(_)`：报错（方言注册永不丢失；派发器恢复五旗标、游标留失败位置，
//!   由核心错误收集/恢复机制收纳）。
//!
//! 旧 `Dialect` trait 在 auto-lang 保持不变（PLAN-778 公开面零漂移）；本
//! trait 的差异只在接收者形态：`&mut dyn ParseContext`（对象安全端口）
//! 取代 `&mut ParserBridge<'_, '_>`。

use auto_ast::ast::Stmt;
use auto_diagnostics::error::AutoResult;
use auto_lex::token::TokenKind;

use crate::context::ParseContext;
use crate::session::CompilerSession;

/// 一个外部方言：在某个场景下生效的一组关键字与语句解析器。
///
/// 实现方在自己的模块/crate 里定义此 trait（只依赖本 crate 及其轻闭包），
/// 然后在上层 driver 中经 `Parser::register_external_dialect` /
/// `with_external_dialect`（PLAN-801）挂载即可启用，核心 parser 无需改动。
pub trait ExternalDialect: Send + Sync {
    /// 该方言是否在当前 session 下生效。
    fn matches(&self, session: &CompilerSession) -> bool;

    /// 该方言接管的语句起始关键字（仅作为语句起始、且在语句位置时被查询）。
    /// 返回的关键字在所属场景下应被视为"上下文关键字"而非普通标识符。
    ///
    /// 注意：这里只列 `TokenKind::Ident` 路径的关键字（如 widget/msg/model）。
    /// 若方言需接管真实 TokenKind（如 view/on），实现 `try_parse_token_stmt`。
    fn keywords(&self) -> &'static [&'static str];

    /// 命中某个 Ident 关键字时调用。
    /// - 返回 `Ok(Some(stmt))`：本方言已处理，产出 stmt。
    /// - 返回 `Ok(None)`：关键字虽在列表里但本次不归我管（派发器保证 None
    ///   后游标回滚，不吞 token）。
    /// - 返回 `Err(_)`：报错（方言注册不丢失，错误由核心收集/恢复机制收纳）。
    fn try_parse_stmt(&self, ctx: &mut dyn ParseContext, keyword: &str)
        -> AutoResult<Option<Stmt>>;

    /// 命中真实 TokenKind 时调用（view/on 等非 Ident 的 UI token）。
    /// 默认不接管（返回 `Ok(None)`）；需要接管的方言覆写此方法。
    /// `kind` 为当前 token 的 TokenKind，供方言判断是否归自己管。
    fn try_parse_token_stmt(
        &self,
        _ctx: &mut dyn ParseContext,
        _kind: TokenKind,
    ) -> AutoResult<Option<Stmt>> {
        Ok(None)
    }
}

/// 外部方言注册优先级（PLAN-778 冻结；PLAN-801 迁入本 crate 单一定义，
/// `auto_lang::dialect::DialectPriority` 为薄 re-export，类型身份不变）：
///
/// - `Override`：前插（pre 段）——先于内建方言查询。外部方言显式覆盖内建
///   同名关键字处理；内建方言退为 `Ok(None)` 后备（外部 None 时接管）。
/// - `Append`：后接（post 段）——仅当内建方言全部 `Ok(None)` 后可见。
///
/// 派发序恒为 pre → builtin → post；首个 keywords 命中且返回 `Some` 者胜。
/// 注册跨 `with_session` 保留（`with_session` 只重建内建段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialectPriority {
    Override,
    Append,
}
