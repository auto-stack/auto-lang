//! 方言（Dialect）体系 —— 轴 A：语法子集扩展
//!
//! 详见 `docs/design/dialect-extension-diagnosis.md` §4（三轴分析）/ §6.1（方案）。
//!
//! 一个方言代表"在某个场景下生效的一组关键字与语句解析"。
//! 方言只管解析时允许什么语法（轴 A），不管执行/转译（轴 B）或程序形态（轴 C）。
//! 方言解析出的节点仍是基础 `Stmt` 的合法变体，下游消费者类型签名不变。

use crate::ast::Stmt;
use crate::error::AutoResult;
use crate::parser_bridge::ParserBridge;
use crate::session::CompilerSession;
use crate::token::TokenKind;

/// 一个方言：在某个场景下生效的一组关键字与语句解析器。
///
/// 实现方在自己的模块/crate 里定义此 trait，然后经
/// `Parser::register_dialect` / `Parser::with_dialect`（PLAN-778 显式注册，
/// Override 前插或 Append 后接）挂载即可启用，核心 parser 无需改动。
///
/// PLAN-778 起解析入口收窄为 [`ParserBridge`] 最小桥面：外部 crate 实现
/// 本 trait 时不接触 `Parser` 本体及其 pub(crate)/私有字段（AC-01）。
pub trait Dialect: Send + Sync {
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
    /// - 返回 `Ok(None)`：关键字虽在列表里但本次不归我管（让下一个方言/默认
    ///   路径处理；派发器保证 None 后游标回滚，不吞 token）。
    /// - 返回 `Err(_)`：报错（方言注册不丢失，错误由核心收集/恢复机制收纳）。
    fn try_parse_stmt(
        &self,
        bridge: &mut ParserBridge<'_, '_>,
        keyword: &str,
    ) -> AutoResult<Option<Stmt>>;

    /// 命中真实 TokenKind 时调用（view/on 等非 Ident 的 UI token）。
    /// 默认不接管（返回 `Ok(None)`）；需要接管的方言覆写此方法。
    /// `kind` 为当前 token 的 TokenKind，供方言判断是否归自己管。
    fn try_parse_token_stmt(
        &self,
        _bridge: &mut ParserBridge<'_, '_>,
        _kind: TokenKind,
    ) -> AutoResult<Option<Stmt>> {
        Ok(None)
    }
}

/// 外部方言注册优先级（PLAN-778 冻结）：
///
/// - `Override`：前插（pre 段）——先于内建方言查询。外部方言显式覆盖内建
///   同名关键字处理；内建方言退为 `Ok(None)` 后备（外部 None 时接管）。
/// - `Append`：后接（post 段）——仅当内建方言全部 `Ok(None)` 后可见。
///
/// 派发序恒为 pre → builtin → post；首个 keywords 命中且返回 `Some` 者胜。
/// 注册跨 `with_session` 保留（`with_session` 只重建内建段）。
///
/// PLAN-801：单一定义迁至 auto-parser-api（`auto_parser_api::DialectPriority`），
/// 此处薄 re-export——类型身份不变，旧调用点（含
/// `Parser::register_dialect`/`with_dialect`）零改动。
pub use auto_parser_api::DialectPriority;

/// PLAN-801 新接口外部方言 trait（crate 边界外实现，只依赖轻端口 crate）。
/// 与旧 [`Dialect`] 同表派发（经 `dialect::external::ExternalDialectAdapter`
/// 协议转换 + `Parser::register_external_dialect` 注册）；恢复不变量同一套
/// 派发器兜底。外部 crate 优先实现本 trait；旧 trait 为兼容面保持。
pub use auto_parser_api::ExternalDialect;

pub mod external;
// PLAN-814 R3-03b：UI 方言 adapter 无条件编译——文法调用经 UiGrammar
// 函数表端口（auto-parser-api 定义，上层组合点装配 auto-ui-dialect 单源
// 构造器）。表缺席（纯 SDK）时 build_dialects 不装配本方言，Scenario::UI
// 方言表为空，UI 语句回退非 UI 路径；外部方言体系（external）不受影响。
pub mod ui;
