//! PLAN-801：新接口外部方言 → 生产派发表的协议转换 adapter。
//!
//! [`ExternalDialectAdapter`] 把 `auto_parser_api::ExternalDialect`（新接口，
//! `&mut dyn ParseContext` 端口）适配为旧 [`crate::dialect::Dialect`]
//!（`&mut ParserBridge`），使新方言进入**同一张**三段有序派发表
//!（pre → builtin → post，PLAN-778 冻结语义）——不维护另一套行为不同的
//! 派发表。恢复不变量（Arc 注册永不出表 / None 强制回滚 / Err 边界恢复
//! context 旗标）由既有派发器兜底，adapter 只做协议转换，不重复语法。

use crate::ast::Stmt;
use crate::dialect::Dialect;
use crate::error::AutoResult;
use crate::parser_bridge::ParserBridge;
use crate::session::CompilerSession;
use crate::token::TokenKind;

/// 新接口方言的生产注册包装（`Parser::register_external_dialect` 内部使用）。
pub(crate) struct ExternalDialectAdapter {
    pub(crate) inner: std::sync::Arc<dyn crate::dialect::ExternalDialect>,
}

impl Dialect for ExternalDialectAdapter {
    fn matches(&self, session: &CompilerSession) -> bool {
        self.inner.matches(session)
    }

    fn keywords(&self) -> &'static [&'static str] {
        self.inner.keywords()
    }

    fn try_parse_stmt(
        &self,
        bridge: &mut ParserBridge<'_, '_>,
        keyword: &str,
    ) -> AutoResult<Option<Stmt>> {
        // &mut ParserBridge → &mut dyn ParseContext：ParserBridge 在上层
        // 实现了对象安全端口（parser_bridge.rs 的 ParseContext impl）。
        self.inner.try_parse_stmt(bridge, keyword)
    }

    fn try_parse_token_stmt(
        &self,
        bridge: &mut ParserBridge<'_, '_>,
        kind: TokenKind,
    ) -> AutoResult<Option<Stmt>> {
        self.inner.try_parse_token_stmt(bridge, kind)
    }
}
