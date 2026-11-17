//! PLAN-982（B-CONFIG-I，E1/E2）：**配置文档 source 前门**。
//!
//! 职责边界（设计 `docs/design/ace-config-document-evaluation.md` §2/§3.2）：
//! - 配置文档不是模块：顶层是按源顺序的**文档行**（`key: value`、节点语句、
//!   `var`／`if`／`for`）。解析只走共享 CST 前门（`cfg-*` 规则），行结构与
//!   span 由 CST 承载，**不做字符串扫描**；行的值仍是普通 Auto 表达式（共享
//!   表达式规则），由共享 resolve/typecheck/emit/bind 检查。
//! - 文档行物化为**一个**顺序入口函数体（`script` 入口），而不是一组互相无状态
//!   的隐藏字段函数——跨字段变量、`if`/`for` 累积作用域都要求单入口顺序语义。
//! - 仅本前门消费 [`SourceProfile::ConfigDocument`]；通用解析/编译入口对该
//!   profile 具名拒绝（文档行不会被通用投影静默丢弃）。
//!
//! 台账进度（§5 E1..E8）：
//! - E1 已交：profile 身份/名字映射 + 单顺序入口的 `CompiledSource`；
//! - E2 本模块：文档行的 grammar 与 CST→候选投影（`ConfigDocumentSyntax`，
//!   三形 `dep` 头／顶层 prop／嵌套体／`if`／`for`／节点值），坏邻例给原 span
//!   的 `source.*` 诊断；
//! - 未交：文档**值语义**（`cfg.*` 原语、owned 导出、host 桥）＝E4/E5/E6。
//!   因此本前门当前只能**执行**语句形态的文档行（`var`／`if`）；`key: value`、
//!   节点行与运行期 `for` 在解析/投影层完整成立，但编译执行返回具名
//!   `source.config-document/pending`（不静默丢弃、不拿 CST 绿冒充能力）。

use super::adapter::CompiledSource;
use super::ast::{
    Block, Expr, FnItem, Ident, Item, Program, Receiver, Stmt, TypeAnn, TypeRef, TypeWord,
};
use super::SourceProfile;
use super::CONFIG_DOCUMENT_SOURCE_PROFILE;
use crate::atom_text::{Diagnostic, Span, Stage};
use crate::verify::CheckedModule;

/// 配置文档领域 id（设计 §3.2；结果身份的一部分，未来进正式来源）。
pub const CONFIG_DOCUMENT_DOMAIN: &str = "ace.config.document/v1";

/// 配置文档 owned 结果协议（设计 §3.2；E5 导出落地后进入交接件身份）。
pub const CONFIG_DOCUMENT_VALUE_SCHEMA: &str = "auto.config.value/1";

/// 顺序入口的源级名字（文档行序列的唯一入口；不是用户可见的 `main`）。
pub const CONFIG_DOCUMENT_ENTRY_NAME: &str = "__cfg_document";

/// 编译器为配置文档生成的辅助名字前缀（E3 起的 source map/原语 helper 共用；
/// 用户顶层名包含该前缀 = 具名冲突，不静默遮蔽）。
pub const CONFIG_DOCUMENT_HIDDEN_PREFIX: &str = "__cfg_";

// ---------------------------------------------------------------------------
// 文档候选（E2：结构 + 原 span + 共享表达式）
// ---------------------------------------------------------------------------

/// 一个文档体（顶层或节点/if/for 体）：按源顺序的行 + 体 span。
#[derive(Clone, Debug)]
pub struct ConfigBody {
    pub lines: Vec<ConfigLine>,
    pub span: Span,
}

/// 一行文档。
#[derive(Clone, Debug)]
pub enum ConfigLine {
    Pair(ConfigPair),
    Node(ConfigNode),
    Var(ConfigVar),
    If(ConfigIf),
    For(ConfigFor),
}

impl ConfigLine {
    pub fn span(&self) -> Span {
        match self {
            ConfigLine::Pair(p) => p.span,
            ConfigLine::Node(n) => n.span,
            ConfigLine::Var(v) => v.span,
            ConfigLine::If(i) => i.span,
            ConfigLine::For(f) => f.span,
        }
    }
}

/// `key: value`。
#[derive(Clone, Debug)]
pub struct ConfigPair {
    pub key: Ident,
    pub value: ConfigPairValue,
    pub span: Span,
}

/// pair 的值：普通表达式，或**带体**的节点值（`icf: file("linker/x.icf") { … }`）。
#[derive(Clone, Debug)]
pub enum ConfigPairValue {
    Expr(Expr),
    Node(ConfigNode),
}

/// 节点语句：头拼写 + 可选 id（裸名或字符串）+ 可选参数表 + 可选体。
#[derive(Clone, Debug)]
pub struct ConfigNode {
    /// 头拼写（`dep`／`pac`／`link`／普通名）。头是**名字**，语义由值/消费侧定。
    pub name: String,
    pub name_span: Span,
    pub id: Option<ConfigNodeId>,
    pub args: Vec<ConfigArg>,
    pub body: Option<ConfigBody>,
    pub span: Span,
}

/// 节点 id 两形（与旧实现一致：裸名或字符串字面量；都保原 span）。
#[derive(Clone, Debug)]
pub enum ConfigNodeId {
    Bare(Ident),
    Text { text: String, span: Span },
}

/// 节点参数：位置实参或具名实参（`dir(id: d)`）。
#[derive(Clone, Debug)]
pub enum ConfigArg {
    Pos(Expr),
    Named { name: Ident, value: Expr },
}

impl ConfigArg {
    pub fn span(&self) -> Span {
        match self {
            ConfigArg::Pos(e) => e.span,
            ConfigArg::Named { name, value } => Span {
                start: name.span.start,
                end: value.span.end,
            },
        }
    }
}

/// `var <name> = <expr>` 文档局部。
#[derive(Clone, Debug)]
pub struct ConfigVar {
    pub kw_span: Span,
    pub name: Ident,
    pub value: Expr,
    pub span: Span,
}

/// `if <cond> { … } [else { … }]` 文档分支（体是文档行）。
#[derive(Clone, Debug)]
pub struct ConfigIf {
    pub cond: Expr,
    pub then: ConfigBody,
    pub els: Option<ConfigBody>,
    pub span: Span,
}

/// `for <name> in <expr> { … }` 文档循环（体是文档行）。
#[derive(Clone, Debug)]
pub struct ConfigFor {
    pub var: Ident,
    pub iter: Expr,
    pub body: ConfigBody,
    pub span: Span,
}

/// 一份配置文档的未受信候选：行序列 + 源/语法身份。
#[derive(Clone, Debug)]
pub struct ConfigDocumentSyntax {
    pub lines: Vec<ConfigLine>,
    pub grammar_id: String,
    pub input_hash: String,
}

impl ConfigDocumentSyntax {
    /// 行总数（递归含嵌套体）。
    pub fn total_lines(&self) -> usize {
        fn count(body: &ConfigBody) -> usize {
            body.lines
                .iter()
                .map(|l| {
                    1 + match l {
                        ConfigLine::Node(n) => n.body.as_ref().map(count).unwrap_or(0),
                        ConfigLine::If(i) => {
                            count(&i.then) + i.els.as_ref().map(count).unwrap_or(0)
                        }
                        ConfigLine::For(f) => count(&f.body),
                        ConfigLine::Pair(p) => match &p.value {
                            ConfigPairValue::Node(n) => n.body.as_ref().map(count).unwrap_or(0),
                            ConfigPairValue::Expr(_) => 0,
                        },
                        ConfigLine::Var(_) => 0,
                    }
                })
                .sum()
        }
        count(&ConfigBody {
            lines: self.lines.clone(),
            span: Span { start: 0, end: 0 },
        })
    }
}

/// 一次配置文档前门编译：单顺序入口 + 共享链结果。
#[derive(Clone, Debug)]
pub struct ConfigDocumentFront {
    /// 共享链编译结果（profile 身份 = [`CONFIG_DOCUMENT_SOURCE_PROFILE`]）。
    pub compiled: CompiledSource,
    /// 顺序入口的源级名字（恒 [`CONFIG_DOCUMENT_ENTRY_NAME`]）。
    pub entry: String,
    /// 进入入口的语句条数（源顺序；合成尾值不计）。
    pub statements: usize,
    /// 本次裁剪的语法身份（进入交接件；换 grammar = 换身份）。
    pub grammar_id: String,
}

/// 通用入口对配置文档 profile 的具名拒绝（文档行不得被普通投影静默丢弃）。
pub fn config_profile_refusal(profile: SourceProfile, src_len: usize) -> Option<Diagnostic> {
    if profile != SourceProfile::ConfigDocument {
        return None;
    }
    Some(document_diag(
        "source.config-document/front-door",
        "the config-document profile is consumed only by the configuration document front door \
         (ace_core::source::config_document::parse_config_document / compile_config_document); \
         generic parsing/compilation cannot represent document lines",
        Span {
            start: 0,
            end: src_len,
        },
    ))
}

// ---------------------------------------------------------------------------
// E2：文档解析（共享 CST 前门 + cfg-* 投影）
// ---------------------------------------------------------------------------

/// 解析一份配置文档为未受信候选（结构 + 原 span + 共享表达式）。
///
/// 空文档不是成功求值（设计 §4.1）；坏邻例由投影层给原 span 的 `source.*`。
pub fn parse_config_document(src: &str) -> Result<ConfigDocumentSyntax, Vec<Diagnostic>> {
    if src.trim().is_empty() {
        return Err(vec![document_diag(
            "source.config-document/empty",
            "a configuration document must contain at least one document line; \
             an empty document is not a successful evaluation",
            Span {
                start: 0,
                end: src.len(),
            },
        )]);
    }
    super::cst::parse_config_document_front(src, super::cst::FrontBudget::WORK_OWNED_64MIB)
}

// ---------------------------------------------------------------------------
// E1：单顺序入口物化 + 共享编译链
// ---------------------------------------------------------------------------

/// 配置文档前门：文档候选 → 唯一顺序入口物化 → 共享 resolve/typecheck/emit/bind
/// → `text-record` 域唯一 verify。
///
/// 当前只物化**语句形态**的文档行（`var`／`if`）；`key: value`、节点与运行期
/// `for` 的物化属 E3，值语义属 E4（见模块头）。
pub fn compile_config_document(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<ConfigDocumentFront, Vec<Diagnostic>> {
    let syntax = parse_config_document(src)?;
    let mut stmts: Vec<Stmt> = Vec::new();
    materialize_body(
        &ConfigBody {
            lines: syntax.lines.clone(),
            span: Span {
                start: 0,
                end: src.len(),
            },
        },
        &mut stmts,
    )?;
    let statements = stmts.len();
    let doc_span = Span {
        start: 0,
        end: src.len(),
    };
    // 共享脚本入口合同要求 i32 尾值；文档的值**不是**这个返回值（配置文档的
    // 结果来自 cfg 构建器导出的 owned 快照，E4/E5），故物化常量 `0`（零宽 span
    // = 文档末尾，不执行用户代码、不能遮蔽用户行）。
    let tail = Span {
        start: src.len(),
        end: src.len(),
    };
    stmts.push(Stmt::ExprStmt {
        value: Expr {
            span: tail,
            kind: super::ast::ExprKind::IntLit(0),
        },
        span: tail,
    });
    let entry = FnItem {
        name: Ident {
            name: CONFIG_DOCUMENT_ENTRY_NAME.to_string(),
            span: doc_span,
        },
        type_params: Vec::new(),
        params: Vec::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(TypeWord::Void),
            span: doc_span,
        },
        body: Block {
            stmts,
            span: doc_span,
        },
        span: doc_span,
        receiver: Receiver::Free,
        owner: None,
        script: true,
    };
    let program = Program {
        items: vec![Item::Fn(entry)],
    };
    let parts = super::adapter::compile_materialized_program_parts(
        &program,
        src,
        module_path,
        capabilities,
        SourceProfile::ConfigDocument,
    )?;
    let checked: CheckedModule = crate::verify::verify_text_record(parts.bundle)?;
    Ok(ConfigDocumentFront {
        compiled: CompiledSource {
            profile: CONFIG_DOCUMENT_SOURCE_PROFILE,
            input_hash: parts.input_hash,
            atom_text: parts.atom_text,
            checked,
        },
        entry: CONFIG_DOCUMENT_ENTRY_NAME.to_string(),
        statements,
        grammar_id: syntax.grammar_id,
    })
}

/// 文档体 → 语句序列（当前只含语句形态行；其余行给具名 pending）。
fn materialize_body(body: &ConfigBody, out: &mut Vec<Stmt>) -> Result<(), Vec<Diagnostic>> {
    for line in &body.lines {
        match line {
            ConfigLine::Var(v) => out.push(Stmt::Let {
                kw: v.kw_span,
                name: v.name.clone(),
                ann: None,
                mutable: true,
                value: v.value.clone(),
                span: v.span,
            }),
            ConfigLine::If(i) => {
                let mut then = Vec::new();
                materialize_body(&i.then, &mut then)?;
                let els = match &i.els {
                    Some(e) => {
                        let mut stmts = Vec::new();
                        materialize_body(e, &mut stmts)?;
                        Some(Block {
                            stmts,
                            span: e.span,
                        })
                    }
                    None => None,
                };
                out.push(Stmt::If {
                    cond: i.cond.clone(),
                    then: Block {
                        stmts: then,
                        span: i.then.span,
                    },
                    els,
                    span: i.span,
                });
            }
            ConfigLine::Pair(p) => {
                return Err(vec![pending_line("a `key: value` document pair", p.span)])
            }
            ConfigLine::Node(n) => return Err(vec![pending_line("a document node", n.span)]),
            ConfigLine::For(f) => return Err(vec![pending_line("a `for` document line", f.span)]),
        }
    }
    Ok(())
}

fn pending_line(what: &str, span: Span) -> Diagnostic {
    document_diag(
        "source.config-document/pending",
        format!(
            "{what} parses into the document candidate (E2) but materialization/value \
             semantics land with E3/E4; it is not silently dropped"
        ),
        span,
    )
}

fn document_diag(code: &str, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Text, code, message, span)
}
