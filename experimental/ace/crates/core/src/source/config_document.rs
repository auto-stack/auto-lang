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

/// 文档是否**没有任何内容**（只有空白/`//` 行注释/`#` 行注释）：设计 §4.1 的
/// "空文档 = 空 root"。字符串里的 `#`/`//` 不算注释（按字节扫描、跳过字符串）。
fn is_effectively_empty(src: &str) -> bool {
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut in_str = false;
    let mut in_char = false;
    let mut escaped = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_str {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_char {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }
        match c {
            b' ' | b'\t' | b'\r' | b'\n' => {
                i += 1;
            }
            b'"' => {
                in_str = true;
                i += 1;
            }
            b'\'' => {
                in_char = true;
                i += 1;
            }
            b'#' => {
                // `#` 行注释（config 面 trivia）；跳到行尾。
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            _ => return false,
        }
    }
    true
}
/// 解析一份配置文档为未受信候选（结构 + 原 span + 共享表达式）。
///
/// 原始前门对**空输入**给具名 `source.config-document/empty`（它描述的是"没有
/// 文档行"这一事实）；而**求值入口** [`compile_config_document`] 按设计 §4.1 把空
/// 输入当作"空 root 文档"成功返回。坏邻例由投影层给原 span 的 `source.*`。
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
// E3：文档行 → 单顺序入口（cfg 原语调用） + 共享编译链
// ---------------------------------------------------------------------------

/// 配置文档前门（无注入）：等价于 [`compile_config_document_with_injection`]
/// 的空注入形式。
pub fn compile_config_document(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<ConfigDocumentFront, Vec<Diagnostic>> {
    compile_config_document_with_injection(src, module_path, capabilities, &[])
}

/// 与 [`compile_config_document`] 相同，但带**显式注入的上下文值**（legacy
/// 语义：请求 args 在文档里按名可见；`on`/`off` 内建由桥补齐）。
///
/// 注入点语义：注入名先绑成不可变局部，文档行按源顺序在其后执行；文档自己用
/// `var` 声明的同名局部**遮蔽**注入值（注入不覆盖用户声明）；非法名/重复名
/// 直接具名拒绝，而不是假装注入成功。
pub fn compile_config_document_with_injection(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    injected: &[(String, InjectValue)],
) -> Result<ConfigDocumentFront, Vec<Diagnostic>> {
    // V-CFG-TEMPLATE：普通字符串里的 `${expr}` 与节点头的裸 `${expr}` 先按
    // f-string 同一生成物重写成 `__fstr(...)`（legacy config 也在这里插值）。
    // 重写后的文本是本次编译的**源**（诊断/末尾 span 以它为准）。
    let src_text = match super::fstr::desugar_config_templates(src) {
        Ok(s) => s,
        Err(message) => {
            return Err(vec![document_diag(
                "source.fstr-syntax",
                message,
                Span {
                    start: 0,
                    end: src.len(),
                },
            )])
        }
    };
    let src: &str = &src_text;
    // 设计 §4.1：**空文档返回空 root**（不是"求值失败"）。只有真正含内容却
    // 无法成行的文档才是 `source.config-document/structure` 的拒绝面。
    let syntax = if is_effectively_empty(src) {
        ConfigDocumentSyntax {
            lines: Vec::new(),
            grammar_id: CONFIG_DOCUMENT_SOURCE_PROFILE.to_string(),
            input_hash: String::new(),
        }
    } else {
        parse_config_document(src)?
    };
    let doc_span = Span {
        start: 0,
        end: src.len(),
    };
    let mut ctx = EmitCtx::default();
    let mut stmts: Vec<Stmt> = Vec::new();
    // 1) root 句柄（一份文档一个 root，顺序语义的起点）。
    let root = emit_call(&mut stmts, &mut ctx, "cfg_root", Vec::new(), doc_span)?;
    // 1b) 注入的上下文变量（用户 `var` 同名者不注入：文档声明遮蔽注入值）。
    emit_injected(injected, &mut stmts, &mut ctx, doc_span)?;
    // 2) 文档行（用户 `var` 名与隐藏前缀冲突 = 具名拒绝）。
    let body = ConfigBody {
        lines: syntax.lines.clone(),
        span: doc_span,
    };
    emit_body(&body, &root, &mut stmts, &mut ctx)?;
    // 3) 收尾：交出构建器状态（导出/限额核在 E5/E6）。
    let _ = emit_call(&mut stmts, &mut ctx, "cfg_finish", Vec::new(), doc_span)?;
    let statements = stmts.len();
    // 共享脚本入口合同要求 i32 尾值；文档的值**不是**这个返回值，故物化常量
    // `0`（零宽 span = 文档末尾，不执行用户代码、不能遮蔽用户行）。
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
    let checked: CheckedModule = crate::verify::verify_host_console(parts.bundle)?;
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

/// 物化上下文：隐藏名序号 + `var` 局部的**静态种类**表。
///
/// 节点/对象/列表句柄在源面与 i32 值同型，所以"这个变量装的是哪种容器"必须由
/// 物化侧按源结构记账（`var x = { … }` → 对象）；这不是运行期类型猜测，而是
/// 文档字面量语法的确定事实。
#[derive(Default)]
struct EmitCtx {
    next: usize,
    locals: std::collections::BTreeMap<String, LocalClass>,
    /// 已声明过的文档局部（`var` 重声明 = 赋值，不是二次 `let`）。
    declared: std::collections::BTreeSet<String>,
    /// 绑定到**字面数组**的文档 `var`（`for` 的字面展开用；运行期列表不在其中）。
    list_literals: std::collections::BTreeMap<String, Vec<Expr>>,
    /// 宿主注入的键：**根级**同名 pair 的默认值不发射（被注入值覆盖）；嵌套体
    /// 里的同名键是节点自己的属性，必须保留。
    injected_keys: std::collections::BTreeSet<String>,
    /// 当前文档嵌套深度（根级 = 0）：注入覆盖只作用于根级。
    depth: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LocalClass {
    Scalar,
    Obj,
    List,
}

fn emit_body(
    body: &ConfigBody,
    cur: &str,
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
) -> Result<(), Vec<Diagnostic>> {
    for line in &body.lines {
        emit_line(line, cur, out, ctx)?;
    }
    Ok(())
}

/// `Index { base: Var(x), index: StrLit(k) }` 且 `x` 是**对象**局部 ⇒
/// `(x, k)`。其它形态返回 `None`（走普通值物化）。
fn object_index_read(e: &Expr, ctx: &EmitCtx) -> Option<(String, String)> {
    let super::ast::ExprKind::Index { base, index } = &e.kind else {
        return None;
    };
    let super::ast::ExprKind::Var(id) = &base.kind else {
        return None;
    };
    if ctx.locals.get(&id.name) != Some(&LocalClass::Obj) {
        return None;
    }
    let super::ast::ExprKind::StrLit(text) = &index.kind else {
        return None;
    };
    Some((id.name.clone(), text.clone()))
}
fn emit_line(
    line: &ConfigLine,
    cur: &str,
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
) -> Result<(), Vec<Diagnostic>> {
    match line {
        ConfigLine::Var(v) => {
            if v.name.name.contains(CONFIG_DOCUMENT_HIDDEN_PREFIX) {
                return Err(vec![document_diag(
                    "source.config-document/hidden-name",
                    format!(
                        "`{}` contains `{}`, which is reserved for compiler-generated \
                         configuration document locals",
                        v.name.name, CONFIG_DOCUMENT_HIDDEN_PREFIX
                    ),
                    v.name.span,
                )]);
            }
            let (value, class) = emit_value(&v.value, out, ctx)?;
            // 记忆字面数组绑定：`for d in <var>` 因此可以按源顺序展开。
            if let super::ast::ExprKind::ArrayLit(items) = &v.value.kind {
                ctx.list_literals.insert(v.name.name.clone(), items.clone());
            }
            // legacy 口径：`var x` 在同一块内**可重声明**，后者生效
            // （V-CFG-FLOW 的 `test_config_var_redeclared_takes_latest`）。
            // 物化侧把重声明落成一次赋值，避免 `let` 重复绑定被 resolve 拒。
            if ctx.declared.contains(&v.name.name) {
                out.push(Stmt::Assign {
                    target: super::ast::AssignTarget {
                        base: v.name.clone(),
                        path: Vec::new(),
                    },
                    value,
                    span: v.span,
                });
            } else {
                out.push(Stmt::Let {
                    kw: v.kw_span,
                    name: v.name.clone(),
                    ann: None,
                    mutable: true,
                    value,
                    span: v.span,
                });
                ctx.declared.insert(v.name.name.clone());
            }
            if class != LocalClass::Scalar {
                ctx.locals.insert(v.name.name.clone(), class);
            }
            Ok(())
        }
        ConfigLine::If(i) => {
            // 分支体是**独立块作用域**：各自可以 `var x`（不互相成重声明），
            // 分支里声明的名字也不泄漏到外层（块作用域语义）。
            let outer_declared = ctx.declared.clone();
            let outer_locals = ctx.locals.clone();
            let mut then = Vec::new();
            ctx.declared = outer_declared.clone();
            ctx.locals = outer_locals.clone();
            let then_result = emit_body(&i.then, cur, &mut then, ctx);
            then_result?;
            let els = match &i.els {
                Some(e) => {
                    ctx.declared = outer_declared.clone();
                    ctx.locals = outer_locals.clone();
                    let mut stmts = Vec::new();
                    let else_result = emit_body(e, cur, &mut stmts, ctx);
                    else_result?;
                    Some(Block {
                        stmts,
                        span: e.span,
                    })
                }
                None => None,
            };
            ctx.declared = outer_declared;
            ctx.locals = outer_locals;
            out.push(Stmt::If {
                cond: i.cond.clone(),
                then: Block {
                    stmts: then,
                    span: i.then.span,
                },
                els,
                span: i.span,
            });
            Ok(())
        }
        ConfigLine::For(f) => emit_for(f, cur, out, ctx),
        ConfigLine::Pair(p) => {
            // 设计 §4.3：宿主注入的键**赢**，其文档默认值不执行（否则一个
            // 本会被覆盖的默认 RHS 仍可能失败/计费）。注入名在物化侧已知，
            // 因此这里直接不发射该 pair。
            if ctx.depth == 0 && ctx.injected_keys.contains(&p.key.name) {
                return Ok(());
            }
            let key = str_lit(&p.key.name, p.key.span);
            match &p.value {
                ConfigPairValue::Node(n) => {
                    let child = build_node(n, out, ctx)?;
                    let _ = emit_call(
                        out,
                        ctx,
                        "cfg_put_node",
                        vec![var_expr(cur, p.span), key, var_expr(&child, p.span)],
                        p.span,
                    )?;
                }
                ConfigPairValue::Expr(e) => {
                    // PLAN-982：`heap: k["heap"]` —— 对象字段**搬运**进文档，
                    // 不经过 Str 返回面（`cfg_super_put(dst, src_obj, key)`）。
                    if let Some((obj_local, field)) = object_index_read(e, ctx) {
                        // 链式读：中间步用 `cfg_obj_get_obj` 取对象句柄（i32-only），
                        // 末端 `cfg_super_put` 把字段搬到 pair 键上。
                        let obj = var_expr(&obj_local, e.span);
                        let _ = emit_call(
                            out,
                            ctx,
                            "cfg_super_put",
                            vec![
                                var_expr(cur, p.span),
                                obj,
                                str_lit(&field, e.span),
                                str_lit(&p.key.name, p.span),
                            ],
                            p.span,
                        )?;
                        return Ok(());
                    }
                    let (value, class) = emit_value(e, out, ctx)?;
                    write_value(out, ctx, cur, "cfg_put", key, value, class, p.span)?;
                }
            }
            Ok(())
        }
        ConfigLine::Node(n) => {
            let child = build_node(n, out, ctx)?;
            let _ = emit_call(
                out,
                ctx,
                "cfg_append",
                vec![var_expr(cur, n.span), var_expr(&child, n.span)],
                n.span,
            )?;
            Ok(())
        }
    }
}

/// 建节点（不追加）：`cfg_node(name)` → id 两形 → 实参 → 体。返回节点句柄局部名。
fn build_node(
    node: &ConfigNode,
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
) -> Result<String, Vec<Diagnostic>> {
    let handle = emit_call(
        out,
        ctx,
        "cfg_node",
        vec![str_lit(&node.name, node.name_span)],
        node.span,
    )?;
    if let Some(id) = &node.id {
        match id {
            ConfigNodeId::Bare(ident) => {
                let _ = emit_call(
                    out,
                    ctx,
                    "cfg_id_name",
                    vec![
                        var_expr(&handle, ident.span),
                        str_lit(&ident.name, ident.span),
                    ],
                    ident.span,
                )?;
            }
            ConfigNodeId::Text { text, span } => {
                let _ = emit_call(
                    out,
                    ctx,
                    "cfg_id_text",
                    vec![var_expr(&handle, *span), str_lit(text, *span)],
                    *span,
                )?;
            }
        }
    }
    for arg in &node.args {
        match arg {
            ConfigArg::Pos(e) => {
                let (value, class) = emit_value(e, out, ctx)?;
                let callee = container_callee("cfg_arg", class);
                let _ = emit_call(
                    out,
                    ctx,
                    &callee,
                    vec![var_expr(&handle, e.span), value],
                    e.span,
                )?;
            }
            ConfigArg::Named { name, value } => {
                let v = emit_value(value, out, ctx)?;
                named_arg_value(out, ctx, &handle, name, v, name.span)?;
            }
        }
    }
    if let Some(body) = &node.body {
        // 节点体同样是独立块作用域（体内 `var` 不与外层互撞、不泄漏），深度 +1
        // （根级注入覆盖不作用于节点自己的同名属性）。
        let outer_declared = ctx.declared.clone();
        let outer_locals = ctx.locals.clone();
        ctx.depth += 1;
        let body_result = emit_body(body, &handle, out, ctx);
        ctx.depth -= 1;
        body_result?;
        ctx.declared = outer_declared;
        ctx.locals = outer_locals;
    }
    Ok(handle)
}

/// 物化一个值表达式：对象/数组字面量先产出容器句柄语句；其它表达式原样返回
/// （标量/文本由 resolve 的重载选择），并带回**静态种类**。
fn emit_value(
    expr: &Expr,
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
) -> Result<(Expr, LocalClass), Vec<Diagnostic>> {
    match &expr.kind {
        super::ast::ExprKind::ObjectLit { fields } => {
            let obj = emit_call(out, ctx, "cfg_obj_new", Vec::new(), expr.span)?;
            for f in fields {
                let v = emit_value(&f.value, out, ctx)?;
                write_value(
                    out,
                    ctx,
                    &obj,
                    "cfg_obj_put",
                    str_lit(&f.key, f.span),
                    v.0,
                    v.1,
                    f.span,
                )?;
            }
            Ok((var_expr(&obj, expr.span), LocalClass::Obj))
        }
        super::ast::ExprKind::ArrayLit(elems) => {
            let list = emit_call(out, ctx, "cfg_list_new", Vec::new(), expr.span)?;
            for e in elems {
                let v = emit_value(e, out, ctx)?;
                push_list_value(out, ctx, &list, v.0, v.1, e.span)?;
            }
            Ok((var_expr(&list, expr.span), LocalClass::List))
        }
        super::ast::ExprKind::Var(id) => {
            let class = ctx
                .locals
                .get(&id.name)
                .copied()
                .unwrap_or(LocalClass::Scalar);
            Ok((expr.clone(), class))
        }
        _ => Ok((expr.clone(), LocalClass::Scalar)),
    }
}

fn write_value(
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
    target: &str,
    base: &str,
    key: Expr,
    value: Expr,
    class: LocalClass,
    span: Span,
) -> Result<String, Vec<Diagnostic>> {
    let callee = container_callee(base, class);
    emit_call(
        out,
        ctx,
        &callee,
        vec![var_expr(target, span), key, value],
        span,
    )
}

fn push_list_value(
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
    list: &str,
    value: Expr,
    class: LocalClass,
    span: Span,
) -> Result<(), Vec<Diagnostic>> {
    let callee = container_callee("cfg_list_push", class);
    let _ = emit_call(out, ctx, &callee, vec![var_expr(list, span), value], span)?;
    Ok(())
}

fn named_arg_value(
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
    node: &str,
    name: &Ident,
    value: (Expr, LocalClass),
    span: Span,
) -> Result<(), Vec<Diagnostic>> {
    let (v, class) = value;
    let callee = container_callee("cfg_arg_named", class);
    let _ = emit_call(
        out,
        ctx,
        &callee,
        vec![var_expr(node, span), str_lit(&name.name, name.span), v],
        span,
    )?;
    Ok(())
}

fn container_callee(base: &str, class: LocalClass) -> String {
    match class {
        LocalClass::Scalar => base.to_string(),
        LocalClass::Obj => format!("{base}_obj"),
        LocalClass::List => format!("{base}_list"),
    }
}

/// 一条 `let __cfg_s<N> = <callee>(args...)`；返回隐藏局部名。
fn emit_call(
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
    callee: &str,
    args: Vec<Expr>,
    span: Span,
) -> Result<String, Vec<Diagnostic>> {
    let name = format!("{CONFIG_DOCUMENT_HIDDEN_PREFIX}s{}", ctx.next);
    ctx.next += 1;
    let value = Expr {
        span,
        kind: super::ast::ExprKind::Call {
            callee: Ident {
                name: callee.to_string(),
                span,
            },
            args: args.into_iter().map(super::ast::Arg::Pos).collect(),
        },
    };
    out.push(Stmt::Let {
        kw: span,
        name: Ident {
            name: name.clone(),
            span,
        },
        ann: None,
        mutable: false,
        value,
        span,
    });
    Ok(name)
}

fn var_expr(name: &str, span: Span) -> Expr {
    Expr {
        span,
        kind: super::ast::ExprKind::Var(Ident {
            name: name.to_string(),
            span,
        }),
    }
}

fn str_lit(text: &str, span: Span) -> Expr {
    Expr {
        span,
        kind: super::ast::ExprKind::StrLit(text.to_string()),
    }
}

// ---------------------------------------------------------------------------
// E7（context 注入）：注入值面与小工具
// ---------------------------------------------------------------------------

/// 上下文注入值（legacy 语义：请求 args 在文档里**按名可见**）。
///
/// 只收宿主车道当前能精确承载的标量/文本；对象/数组/浮点/空值的注入由桥具名
/// 拒绝（见 `auto-lang-ace-host::config_document`），不静默降级。
#[derive(Clone, Debug, PartialEq)]
pub enum InjectValue {
    I32(i32),
    Bool(bool),
    Char(char),
    Text(String),
}

impl InjectValue {
    fn to_expr(&self, span: Span) -> Expr {
        Expr {
            span,
            kind: match self {
                InjectValue::I32(n) => super::ast::ExprKind::IntLit(*n),
                InjectValue::Bool(b) => super::ast::ExprKind::BoolLit(*b),
                InjectValue::Char(c) => super::ast::ExprKind::CharLit(*c as u32),
                InjectValue::Text(t) => super::ast::ExprKind::StrLit(t.clone()),
            },
        }
    }
}

/// `for <var> in <list> { … }`：
///
/// **字面列表展开**（legacy `test_config_for_unrolls_literal_array` 语义）——迭代源
/// 是字面数组（`for d in ["a","b"]`）或绑定到字面数组的文档 `var` 时，物化侧按源
/// 顺序展开体，逐项把循环变量绑成该元素值；**运行期列表**（元素来自注入/计算）
/// 仍需列表计数/取值原语，这里具名 `pending`（不静默丢弃、不假装迭代）。
///
/// 每个元素都用**同一个**可变局部（首项 `let`，其后赋值），因此体内不要重复
/// `var` 同名局部（块作用域限制，已记录）。
fn emit_for(
    f: &ConfigFor,
    cur: &str,
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
) -> Result<(), Vec<Diagnostic>> {
    let elements: Option<Vec<Expr>> = match &f.iter.kind {
        super::ast::ExprKind::ArrayLit(items) => Some(items.clone()),
        super::ast::ExprKind::Var(id) => ctx.list_literals.get(&id.name).cloned(),
        _ => None,
    };
    let Some(elements) = elements else {
        return Err(vec![document_diag(
            "source.config-document/pending",
            "a `for` over a runtime list needs the list-count/element primitives \
             (`cfg.list.len/at`), which are not implemented yet; only literal lists \
             (`for d in [\"a\", \"b\"]` or a `var` bound to one) unroll today",
            f.span,
        )]);
    };
    let saved_declared = ctx.declared.clone();
    let saved_locals = ctx.locals.clone();
    for (index, element) in elements.into_iter().enumerate() {
        let (value, _class) = emit_value(&element, out, ctx)?;
        if index == 0 {
            out.push(Stmt::Let {
                kw: f.span,
                name: f.var.clone(),
                ann: None,
                mutable: true,
                value,
                span: f.span,
            });
            ctx.declared.insert(f.var.name.clone());
        } else {
            out.push(Stmt::Assign {
                target: super::ast::AssignTarget {
                    base: f.var.clone(),
                    path: Vec::new(),
                },
                value,
                span: f.span,
            });
        }
        // 体内声明不跨迭代泄漏（每次迭代从展开前的声明面开始）。
        ctx.declared = saved_declared.clone();
        ctx.locals = saved_locals.clone();
        ctx.declared.insert(f.var.name.clone());
        let body_result = emit_body(&f.body, cur, out, ctx);
        body_result?;
    }
    ctx.declared = saved_declared;
    ctx.locals = saved_locals;
    Ok(())
}
fn emit_injected(
    injected: &[(String, InjectValue)],
    out: &mut Vec<Stmt>,
    ctx: &mut EmitCtx,
    span: Span,
) -> Result<(), Vec<Diagnostic>> {
    if injected.is_empty() {
        return Ok(());
    }
    let mut seen = std::collections::BTreeSet::new();
    for (name, value) in injected {
        ctx.injected_keys.insert(name.clone());
        if !is_document_ident(name) {
            return Err(vec![document_diag(
                "source.config-document/inject-name",
                format!(
                    "injected context name `{name}` is not a legal document identifier; \
                     context values are visible by name in the document"
                ),
                span,
            )]);
        }
        if !seen.insert(name.clone()) {
            return Err(vec![document_diag(
                "source.config-document/inject-duplicate",
                format!("injected context name `{name}` is supplied more than once"),
                span,
            )]);
        }
        // 注入值按 legacy 语义是**可变全局**：文档后面的 `var <same>` 就是一次
        // 重新赋值（不是二次声明），之前的引用仍看到注入值。
        out.push(Stmt::Let {
            kw: span,
            name: Ident {
                name: name.clone(),
                span,
            },
            ann: None,
            mutable: true,
            value: value.to_expr(span),
            span,
        });
        ctx.declared.insert(name.clone());
    }
    Ok(())
}

/// 文档里所有 `var` 名（递归含嵌套体）。保留给需要枚举声明面的调用方。

fn is_document_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn document_diag(code: &str, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Text, code, message, span)
}
