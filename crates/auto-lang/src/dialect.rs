//! 方言（Dialect）体系 —— 轴 A：语法子集扩展
//!
//! 详见 `docs/design/dialect-extension-diagnosis.md` §4（三轴分析）/ §6.1（方案）。
//!
//! 一个方言代表"在某个场景下生效的一组关键字与语句解析"。
//! 方言只管解析时允许什么语法（轴 A），不管执行/转译（轴 B）或程序形态（轴 C）。
//! 方言解析出的节点仍是基础 `Stmt` 的合法变体，下游消费者类型签名不变。

use crate::ast::Stmt;
use crate::error::AutoResult;
use crate::parser::Parser;
use crate::session::CompilerSession;

/// 一个方言：在某个场景下生效的一组关键字与语句解析器。
///
/// 实现方在自己的模块/crate 里定义此 trait，然后在
/// `Parser::build_dialects` 里按 session 注册即可启用，核心 parser 无需改动。
pub trait Dialect: Send + Sync {
    /// 该方言是否在当前 session 下生效。
    fn matches(&self, session: &CompilerSession) -> bool;

    /// 该方言接管的语句起始关键字（仅作为语句起始、且在语句位置时被查询）。
    /// 返回的关键字在所属场景下应被视为"上下文关键字"而非普通标识符。
    fn keywords(&self) -> &'static [&'static str];

    /// 命中某个关键字时调用。
    /// - 返回 `Ok(Some(stmt))`：本方言已处理，产出 stmt。
    /// - 返回 `Ok(None)`：关键字虽在列表里但本次不归我管（让下一个方言/默认路径处理）。
    /// - 返回 `Err(_)`：报错。
    fn try_parse_stmt(&self, parser: &mut Parser, keyword: &str)
        -> AutoResult<Option<Stmt>>;
}
