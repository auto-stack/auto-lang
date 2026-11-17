//! PLAN-982（B-CONFIG-I，E1）：**配置文档 source 前门**。
//!
//! 职责边界（设计 `docs/design/ace-config-document-evaluation.md` §2/§3.2）：
//! - 配置文档不是模块：顶层是一段**按源顺序**执行的文档语句序列。本前门把该
//!   序列物化为**一个**顺序入口函数体（`script` 入口），而不是一组互相无状态
//!   的隐藏字段函数——跨字段变量、`if`/`for` 累积作用域都要求单入口顺序语义。
//! - 解析/检查只走共享链：本模块调用共享 CST 前门（`parse_program_with`）与
//!   共享 `compile_program_parts`/`verify_text_record`，不新建 parser、类型检查器
//!   或表达式求值器；文档语句的表达式仍是普通 Auto 表达式。
//! - 仅本前门消费 [`SourceProfile::ConfigDocument`]；通用入口对该 profile 的
//!   语义面在 E2 落地（当前该 profile 的裁剪 = text-record 表面 ∪ 脚本语句序列，
//!   因此通用入口也能解析它——文档语义只能经本模块取得，E2 起通用入口具名拒绝）。
//!
//! E1 范围（§5 台账）：profile 身份/名字映射 + 单顺序入口的 `CompiledSource`；
//! 配置文档专属语法（顶层 `key: value`、`dep`/节点三形）与 `cfg.*` 原语分别由
//! E2/E4 增量。本模块当前对"非文档形状"（声明项、多入口）具名拒绝，不静默丢。

use super::adapter::CompiledSource;
use super::ast::{Expr, ExprKind, FnItem, Item, Program, Stmt};
use super::SourceProfile;
use super::CONFIG_DOCUMENT_SOURCE_PROFILE;
use crate::atom_text::{Diagnostic, Span, Stage};
use crate::verify::CheckedModule;

/// 配置文档领域 id（设计 §3.2；结果身份的一部分，未来进正式来源）。
pub const CONFIG_DOCUMENT_DOMAIN: &str = "ace.config.document/v1";

/// 配置文档 owned 结果协议（设计 §3.2；E5 导出落地后进入交接件身份）。
pub const CONFIG_DOCUMENT_VALUE_SCHEMA: &str = "auto.config.value/1";

/// 顺序入口的源级名字（文档语句序列的唯一入口；不是用户可见的 `main`）。
pub const CONFIG_DOCUMENT_ENTRY_NAME: &str = "__cfg_document";

/// 编译器为配置文档生成的辅助名字前缀（E3 起的 source map/原语 helper 共用；
/// 用户顶层名包含该前缀 = 具名冲突，不静默遮蔽）。
pub const CONFIG_DOCUMENT_HIDDEN_PREFIX: &str = "__cfg_";

/// 一次配置文档前门编译：单顺序入口 + 共享链结果。
#[derive(Clone, Debug)]
pub struct ConfigDocumentFront {
    /// 共享链编译结果（profile 身份 = [`CONFIG_DOCUMENT_SOURCE_PROFILE`]）。
    pub compiled: CompiledSource,
    /// 顺序入口的源级名字（恒 [`CONFIG_DOCUMENT_ENTRY_NAME`]）。
    pub entry: String,
    /// 顶层文档语句条数（源顺序）。
    pub statements: usize,
    /// 本次裁剪的语法身份（进入交接件；换 grammar = 换身份）。
    pub grammar_id: String,
}

/// 取配置文档的**唯一**顺序入口：文档必须恰好物化为一个 `script` 入口，
/// 其余任何顶层项都是具名拒绝（不是静默丢弃）。返回入口函数（其名字尚未
/// 改写——[`compile_config_document`] 在同一位置写回稳定入口名）。
pub fn config_document_entry(program: &Program) -> Result<&FnItem, Vec<Diagnostic>> {
    let mut entries: Vec<&FnItem> = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(f) if f.script => entries.push(f),
            Item::Fn(f) => {
                return Err(vec![document_diag(
                    "source.config-document/declaration",
                    format!(
                        "a configuration document is a statement sequence, not a module: `fn {}` \
                         cannot appear in it (declare helpers in a module and import them)",
                        f.name.name
                    ),
                    f.span,
                )]);
            }
            Item::Type(t) => {
                return Err(vec![document_diag(
                    "source.config-document/declaration",
                    format!(
                        "a configuration document is a statement sequence, not a module: \
                         `type {}` cannot appear in it",
                        t.name.name
                    ),
                    t.span,
                )]);
            }
            Item::Enum(e) => {
                return Err(vec![document_diag(
                    "source.config-document/declaration",
                    format!(
                        "a configuration document is a statement sequence, not a module: \
                         `enum {}` cannot appear in it",
                        e.name.name
                    ),
                    e.span,
                )]);
            }
            Item::Const(c) => {
                return Err(vec![document_diag(
                    "source.config-document/declaration",
                    format!(
                        "a configuration document is a statement sequence, not a module: \
                         `const {}` cannot appear in it",
                        c.name.name
                    ),
                    c.span,
                )]);
            }
            Item::Store(s) => {
                return Err(vec![document_diag(
                    "source.config-document/declaration",
                    format!(
                        "a configuration document is a statement sequence, not a module: \
                         `store {}` cannot appear in it",
                        s.name.name
                    ),
                    s.span,
                )]);
            }
            Item::UseRs(_) => {
                return Err(vec![document_diag(
                    "source.config-document/declaration",
                    "a configuration document is a statement sequence, not a module: \
                     `use.rs` imports cannot appear in it"
                        .to_string(),
                    Span { start: 0, end: 0 },
                )]);
            }
            Item::Task(t) => {
                return Err(vec![document_diag(
                    "source.config-document/declaration",
                    format!(
                        "a configuration document is a statement sequence, not a module: \
                         `task {}` cannot appear in it",
                        t.name.name
                    ),
                    t.span,
                )]);
            }
        }
    }
    match entries.as_slice() {
        [entry] => Ok(entry),
        [] => Err(vec![document_diag(
            "source.config-document/empty",
            "a configuration document must contain at least one document statement; \
             an empty document is not a successful evaluation",
            Span { start: 0, end: 0 },
        )]),
        _ => Err(vec![document_diag(
            "source.config-document/entry",
            format!(
                "a configuration document materializes exactly one sequential entry, found {}",
                entries.len()
            ),
            entries[1].span,
        )]),
    }
}

/// 配置文档前门：共享 CST 解析 → 唯一顺序入口物化 → 共享 resolve/typecheck/emit/
/// bind → `text-record` 域唯一 verify。
///
/// 入口命名：文档语句序列由共享投影产出 `script` 入口（名为 `main`）；本前门
/// 在同一 `FnItem` 上写回稳定入口名 [`CONFIG_DOCUMENT_ENTRY_NAME`]，各语句及其
/// 表达式的 span 原样保留（精确 source map）。改名不新造语句、不改任何 span。
pub fn compile_config_document(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<ConfigDocumentFront, Vec<Diagnostic>> {
    // 空文档不是成功求值（设计 §4.1：空文档返回空 root 不能充作无执行的假成功；
    // 本前门在解析前就把它具名拒绝，避免共享前门的 `expected fn` 消息掩盖
    // "文档为空"这一真实原因）。
    if src.trim().is_empty() {
        return Err(vec![document_diag(
            "source.config-document/empty",
            "a configuration document must contain at least one document statement; \
             an empty document is not a successful evaluation",
            Span {
                start: 0,
                end: src.len(),
            },
        )]);
    }
    let mut program =
        super::parse_program_with(src, SourceProfile::ConfigDocument).map_err(|d| vec![d])?;
    let statements = {
        let entry = config_document_entry(&program)?;
        entry.body.stmts.len()
    };
    {
        // 物化：唯一顺序入口的源级名字（span 保原位置 = 文档起点）＋入口的
        // 顺序合同尾值。
        //
        // **尾值不是文档**：配置文档求值的结果是 cfg 构建器导出的 owned 快照
        // （设计 §3.1/§3.2，E4/E5），不是入口返回值。共享脚本入口合同要求一个
        // i32 尾值，这里物化常量 `0`：它是**编译期合成**的物化事实（零宽 span
        // = 文档末尾），不执行任何用户代码、不能遮蔽用户语句；文档语义不读它。
        for item in &mut program.items {
            if let Item::Fn(f) = item {
                if f.script {
                    f.name.name = CONFIG_DOCUMENT_ENTRY_NAME.to_string();
                    let end = Span {
                        start: src.len(),
                        end: src.len(),
                    };
                    f.body.stmts.push(Stmt::ExprStmt {
                        value: Expr {
                            span: end,
                            kind: ExprKind::IntLit(0),
                        },
                        span: end,
                    });
                }
            }
        }
    }
    let parts = super::adapter::compile_program_parts(
        &program,
        src,
        module_path,
        capabilities,
        SourceProfile::ConfigDocument,
    )?;
    let checked: CheckedModule = crate::verify::verify_text_record(parts.bundle)?;
    let grammar_id = super::cst::profile_grammar(SourceProfile::ConfigDocument)
        .map(|g| format!("{}", g.program().identity.grammar_id.0))
        .map_err(|e| {
            vec![Diagnostic::new(
                Stage::Text,
                "source.infra",
                e,
                Span {
                    start: 0,
                    end: src.len(),
                },
            )]
        })?;
    Ok(ConfigDocumentFront {
        compiled: CompiledSource {
            profile: CONFIG_DOCUMENT_SOURCE_PROFILE,
            input_hash: parts.input_hash,
            atom_text: parts.atom_text,
            checked,
        },
        entry: CONFIG_DOCUMENT_ENTRY_NAME.to_string(),
        statements,
        grammar_id,
    })
}

fn document_diag(code: &str, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Text, code, message, span)
}
