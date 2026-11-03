//! 严格投影：auto-source CST → 现有 `ast::Program`（PLAN-927）。
//!
//! - 结构驱动：从 CST 节点/字段/叶读取并构造 AST——不是第二份 token 流
//!   parser；字面量解码与 i32 域检查在投影层（语法身份承载于 CST/grammar）。
//! - 能力/形状拒绝按旧 parser 的逐构造顺序产生诊断；全部收集后取**首个**
//!   （span 起点序 = 旧首错事实）。
//! - 单一显式帧机：块/语句/表达式/实参/构造字段全迭代——深链（嵌套组/
//!   链式调用/深块/长一元链）不依赖线程栈；访问预算受控（资源态，不是
//!   语法 unsupported）。帧协议：需要 N 个子值的帧先压"汇总帧"，再逆序
//!   压子任务——子值按文档序留在 Val 栈上，汇总帧弹出组装。

use super::kinds::SourceKinds;
use super::lex;
use crate::atom_text::{Diagnostic, Span, Stage};
use crate::source::ast::*;
use crate::source::SourceProfile;
use ace_syntax_runtime::cst::{CstTree, FLAG_TRIVIA};
use ace_syntax_runtime::program::{GrammarProgram, KIND_CALL};
use ace_syntax_runtime::snapshot::Snapshot;

const I32_MIN_MAG: u128 = 2147483648;
const I32_MAX: u128 = 2147483647;
/// 投影工作预算（访问帧数；超出 = 资源态受控拒绝）。
const PROJECTION_NODE_BUDGET: usize = 1 << 22;

pub(crate) fn project_program(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
) -> Result<Program, Vec<Diagnostic>> {
    // 词法层先行（旧 tokenize 全文件先行的首事实序）。
    if let Some(d) = lex::first_lex_fact(tree, snap, k, profile) {
        return Err(vec![d]);
    }
    let mut eng = Engine::new(p, k, tree, snap, profile);
    let tree_has_errors = tree.has_error_nodes();
    let mut items = Vec::new();
    let mut saw_script = false;
    for d in &tree.decls {
        // decl 单元 = `item` 分派节点；真实声明是其首个非 trivia 孩子。
        let node = {
            let dn = d.node;
            if tree.kind(dn) == k.item {
                tree.children(dn)
                    .into_iter()
                    .find(|&c| tree.node(c).flags & FLAG_TRIVIA == 0)
                    .unwrap_or(dn)
            } else {
                dn
            }
        };
        let kind = tree.kind(node);
        // 错误子树跳过乐观投影（结构层映射负责其诊断；干净子树照常投影
        // 以保真能力拒绝的位置序）。
        if tree_has_errors && subtree_has_errors(tree, node) {
            continue;
        }
        if kind == k.fn_item || kind == k.type_item.unwrap_or(u32::MAX) {
            if saw_script {
                let span = first_leaf_span(&eng.cx, node);
                eng.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.entry-order",
                        "declarations must precede the script statement sequence; a script runs to end of file",
                        span,
                    )
                });
                continue;
            }
            if kind == k.fn_item {
                items.push(Item::Fn(eng.run_fn_item(node, None, Receiver::Free)));
            } else {
                items.push(Item::Type(eng.run_type_item(node)));
            }
        } else if kind == k.script_item.unwrap_or(u32::MAX) {
            saw_script = true;
            items.push(Item::Fn(eng.run_script_item(node)));
        }
    }
    if tree_has_errors {
        // 结构层首错映射（与干净子树的能力拒绝按位置竞争，最早者胜）。
        if let Some(d) = structural_first_diag(&eng.cx, tree) {
            eng.cx.diags.push((d.span.start as u64, d));
        }
        if let Some(first) = eng.cx.first_diag() {
            return Err(vec![first]);
        }
        let end = snap.len() as usize;
        return Err(vec![Diagnostic::new(
            Stage::Text,
            "source.syntax",
            "expected statement or `}`, found end of file",
            Span { start: end, end },
        )]);
    }
    if let Some(first) = eng.cx.first_diag() {
        return Err(vec![first]);
    }
    // 空程序区分（旧合同）：无任何非 trivia token → "expected `fn`, found end
    // of file"（span=EOF）；只有分隔符 token → Ok(empty)。
    if items.is_empty() {
        let leaves = lex::collect_leaves(tree);
        if leaves.is_empty() {
            let end = snap.len() as usize;
            return Err(vec![Diagnostic::new(
                Stage::Text,
                "source.syntax",
                "expected `fn`, found end of file",
                Span { start: end, end },
            )]);
        }
        return Ok(Program { items });
    }
    Ok(Program { items })
}

/// 子树是否含 error/missing 节点（迭代下降）。
fn subtree_has_errors(tree: &CstTree, root: u32) -> bool {
    use ace_syntax_runtime::cst::{FLAG_ERROR, FLAG_MISSING};
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        if nd.flags & (FLAG_ERROR | FLAG_MISSING) != 0 {
            return true;
        }
        for c in tree.children(n) {
            stack.push(c);
        }
    }
    false
}

/// 顶层不被支持的纯 ident 单词（旧 TOPLEVEL_UNSUPPORTED_WORDS 的 ident 部分）。
const TOPLEVEL_IDENT_WORDS: &[&str] = &[
    "struct", "trait", "impl", "pub", "checked", "unsafe", "macro", "import", "package",
];

/// 结构层首错映射：文档序最早的 error/missing 位置 → 旧诊断（首切上下文
/// 分类：item 位 / stmt 位 / expr 位；profile 特定消息由能力事实驱动）。
fn structural_first_diag(cx: &Cx, tree: &CstTree) -> Option<Diagnostic> {
    use ace_syntax_runtime::cst::{FLAG_ERROR, FLAG_MISSING};
    // 候选：带内容（len>0）的 ERROR 节点优先（恢复路径在区域起点插入的
    // 零宽 MISSING 标记不携带位置事实，仅作无 ERROR 候选时的后备）。
    let mut best_err: Option<u32> = None;
    let mut best_missing: Option<u32> = None;
    let mut stack = vec![tree.root];
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        let flags = nd.flags & (FLAG_ERROR | FLAG_MISSING);
        if flags != 0 {
            let (s, e) = tree.abs_range(n);
            if flags & FLAG_ERROR != 0 && e > s {
                let better = match best_err {
                    None => true,
                    Some(b) => (s, e) < tree.abs_range(b),
                };
                if better {
                    best_err = Some(n);
                }
            } else {
                let better = match best_missing {
                    None => true,
                    Some(b) => (s, e) < tree.abs_range(b),
                };
                if better {
                    best_missing = Some(n);
                }
            }
            continue;
        }
        for c in tree.children(n) {
            stack.push(c);
        }
    }
    let node = best_err.or(best_missing)?;
    // 期望序列游走（首切）：对 item/stmt 位的错误区域按旧 parser 的期望
    // 序列走叶，报告**分歧点**（旧 parser 在第一个不满足期望的 token 报错）。
    if let Some(d) = walk_expectations(cx, tree, node) {
        return Some(d);
    }
    // 消息定位用区域**首个叶**（旧 parser 在 token 上报错）；missing 节点
    // （零宽）用自身位置。
    let leaf = first_meaningful_leaf(cx, tree, node);
    let span = match leaf {
        Some(l) => cx.span_of(l),
        None => {
            let (s, e) = tree.abs_range(node);
            Span { start: s as usize, end: e as usize }
        }
    };
    let mut path: Vec<u32> = Vec::new();
    if !path_to(tree, tree.root, node, &mut path) {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.syntax",
            "source contains a syntax error",
            span,
        ));
    }
    let leaf = first_meaningful_leaf(cx, tree, node);
    let text = leaf.map(|l| cx.spelling(l)).unwrap_or_default();
    let kind = leaf.map(|l| tree.kind(l));
    let k = cx.k;
    let is_kw = kind.is_some_and(|g| {
        [
            k.tok_let, k.tok_var, k.tok_if, k.tok_else, k.tok_while, k.tok_loop, k.tok_break,
            k.tok_continue, k.tok_return, k.tok_true, k.tok_false, k.tok_for, k.tok_int_kw,
            k.tok_i32, k.tok_bool, k.tok_type_kw, k.tok_static, k.tok_mut, k.tok_enum_kw,
            k.tok_is, k.tok_in,
        ]
        .contains(&g)
    });
    let word_msg = |w: &str| {
        Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            format!("`{}` is not part of this source profile", w),
            span,
        )
    };
    let at_root = path.iter().any(|&g| g == cx.k.item)
        && !path.iter().any(|&g| g == cx.k.script_item.unwrap_or(u32::MAX));
    let in_stmt = path.iter().any(|&g| {
        g == cx.k.stmt || g == cx.k.block || g == cx.k.script_item.unwrap_or(u32::MAX)
    });
    let in_expr = path.iter().any(|&g| g == cx.k.expr || g == cx.k.unary_expr);
    if let Some(g) = kind {
        if g == cx.k.tok_while && cx.profile.surface_is_scalar_or_wider() && in_stmt {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.unsupported",
                "`while` is not part of this source profile; conditional loops use `for <condition> { ... }`",
                span,
            ));
        }
        if g == cx.k.tok_for && !cx.profile.surface_is_scalar_or_wider() && in_stmt {
            return Some(word_msg("for"));
        }
        if g == cx.k.tok_static && at_root {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.unsupported",
                "`static` is only legal before `fn` inside a `type` block of the record profile",
                span,
            ));
        }
    }
    if kind == Some(cx.k.tok_minus) && in_expr && !cx.profile.surface_is_scalar_or_wider() {
        let end = leaf_end(cx, tree, node).max(span.end);
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            "unary minus is only supported directly before an integer literal in this source profile",
            Span { start: span.start, end },
        ));
    }
    if is_kw || text == "fn" {
        return Some(word_msg(&text));
    }
    if kind == Some(cx.k.tok_r_move) || kind == Some(cx.k.tok_r_to) {
        if !in_expr {
            return Some(word_msg(&text));
        }
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            format!("keyword `{}` cannot be used as a name", text),
            span,
        ));
    }
    if let Some(g) = kind {
        let describe = if g == cx.k.tok_affix_num {
            Some("suffixed/radix/float numeric literal")
        } else if g == cx.k.tok_str_lit {
            Some("string literal")
        } else if g == cx.k.tok_char_lit {
            Some("char literal")
        } else if g == cx.k.tok_block_open {
            Some("block comment")
        } else {
            None
        };
        if let Some(desc) = describe {
            let quoted = if g == cx.k.tok_affix_num || g == cx.k.tok_block_open {
                format!("{} is not part of this source profile", desc)
            } else {
                format!("{} `{}` is not part of this source profile", desc, text)
            };
            return Some(Diagnostic::new(Stage::Text, "source.unsupported", quoted, span));
        }
    }
    if kind == Some(cx.k.tok_ident) && at_root {
        if TOPLEVEL_IDENT_WORDS.contains(&text.as_str()) {
            return Some(word_msg(&text));
        }
        return Some(Diagnostic::new(
            Stage::Text,
            "source.syntax",
            format!("expected `fn`, found `{}`", text),
            span,
        ));
    }
    let found = describe_leaf_text(cx, tree, leaf);
    if in_stmt || in_expr {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.syntax",
            format!("expected statement, found {}", found),
            span,
        ));
    }
    Some(Diagnostic::new(
        Stage::Text,
        "source.syntax",
        format!("expected `fn`, found {}", found),
        span,
    ))
}

/// 错误区域之后首个 token 的终点（旧 `span.join(end)` 形态的近似）。
fn leaf_end(cx: &Cx, tree: &CstTree, node: u32) -> usize {
    let leaves = lex::collect_leaves(tree);
    let (s, _) = tree.abs_range(node);
    let mut best: Option<usize> = None;
    for l in &leaves {
        if l.start >= s + 1 {
            best = Some(l.end as usize);
            break;
        }
    }
    best.unwrap_or(s as usize)
}

// ---------------------------------------------------------------- 期望序列游走
//
// 错误区域的叶序列按旧 parser 的期望顺序游走：第一个不满足期望的叶 =
// 旧首错位置。这是**诊断映射**（CST 叶事实 → 旧消息），不是第二份语法
// 实现——接受/拒绝决定仍由规则程序与投影能力层做。

/// 错误区域内的非 trivia 叶（文档序，含 error 叶）。
fn region_leaves(cx: &Cx, tree: &CstTree, node: u32) -> Vec<(u32, u32, String)> {
    let mut out = Vec::new();
    let mut stack = vec![node];
    // 先收集再排序（迭代下降）。
    let mut all: Vec<u32> = Vec::new();
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        if nd.flags & FLAG_TRIVIA != 0 {
            continue;
        }
        if nd.child_count == 0 {
            // 恢复标记（零宽 MISSING 叶）不携带源码字节——跳过。
            let (s, e) = tree.abs_range(n);
            if e > s {
                all.push(n);
            }
            continue;
        }
        let kids = tree.children(n);
        for c in kids.into_iter().rev() {
            stack.push(c);
        }
    }
    all.sort_by_key(|&n| tree.abs_range(n).0);
    for n in all {
        out.push((n, tree.kind(n), cx.spelling(n)));
    }
    out
}

fn walk_expectations(cx: &Cx, tree: &CstTree, node: u32) -> Option<Diagnostic> {
    let leaves = region_leaves(cx, tree, node);
    if leaves.is_empty() {
        return None;
    }
    let first = &leaves[0];
    // item 形态：`fn ...` / `type ...`。
    if first.2 == "fn" {
        return walk_fn_item(cx, &leaves);
    }
    if first.2 == "type" && cx.profile.is_record() {
        return walk_type_item(cx, &leaves);
    }
    // stmt 形态（块内错误区域）。
    if leaves.len() >= 1 {
        if let Some(d) = walk_stmt(cx, &leaves) {
            return Some(d);
        }
    }
    None
}

fn leaf_span(cx: &Cx, l: u32) -> Span {
    cx.span_of(l)
}

/// 旧 fn_item_tail 期望序列（fn 关键字已匹配）。
fn walk_fn_item(cx: &Cx, leaves: &[(u32, u32, String)]) -> Option<Diagnostic> {
    let k = cx.k;
    let mut i = 1usize; // 0 = `fn`
    let at = |idx: usize| -> (Span, u32, &str) {
        let (n, g, s) = &leaves[idx.min(leaves.len() - 1)];
        (leaf_span(cx, *n), *g, s.as_str())
    };
    // 1) 函数名。
    if i >= leaves.len() {
        return None;
    }
    {
        let (sp, g, text) = at(i);
        if g != k.tok_ident {
            if g == k.tok_affix_num || g == k.tok_str_lit || g == k.tok_char_lit || g == k.tok_block_open {
                return Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("{} cannot be used as a name in this source profile", raw_describe(g, text, &k)),
                    sp,
                ));
            }
            if g == k.tok_r_move || g == k.tok_r_to || is_kw_leaf(g, &k) || text == "fn" {
                return Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("keyword `{}` cannot be used as a name", text),
                    sp,
                ));
            }
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected function name, found {}", describe_g(g, text, &k)),
                sp,
            ));
        }
        i += 1;
    }
    // 2) `(`。
    if i >= leaves.len() {
        return None;
    }
    {
        let (sp, g, text) = at(i);
        if text != "(" {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected `(`, found {}", describe_g(g, text, &k)),
                sp,
            ));
        }
        i += 1;
    }
    // 3) 参数序列。
    if i < leaves.len() && leaves[i].2 != ")" {
        loop {
            // mode 词（可选）。
            if i < leaves.len() {
                let (_, g, text) = at(i);
                let is_move = g == k.tok_r_move;
                let is_mut = text == "mut";
                let admits_move = cx.profile.is_text();
                let admits_mut = cx.profile.is_text() || cx.profile.is_record();
                if (is_move && admits_move) || (is_mut && admits_mut) {
                    i += 1;
                }
            }
            // 参数名。
            if i >= leaves.len() {
                return None;
            }
            {
                let (sp, g, text) = at(i);
                if g != k.tok_ident {
                    if g == k.tok_r_move || g == k.tok_r_to || is_kw_leaf(g, &k) || text == "fn" {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("keyword `{}` cannot be used as a name", text),
                            sp,
                        ));
                    }
                    if g == k.tok_affix_num || g == k.tok_str_lit || g == k.tok_char_lit || g == k.tok_block_open {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("{} cannot be used as a name in this source profile", raw_describe(g, text, &k)),
                            sp,
                        ));
                    }
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        format!("expected parameter name, found {}", describe_g(g, text, &k)),
                        sp,
                    ));
                }
                i += 1;
            }
            // 参数类型词。
            if let Some(d) = walk_type_word(cx, leaves, &mut i, "parameter type annotation (`int`/`i32`/`bool`) is required") {
                return Some(d);
            }
            // `,` 或 `)`。
            if i >= leaves.len() {
                return None;
            }
            let (sp, g, text) = at(i);
            if text == "," {
                i += 1;
                if i < leaves.len() && leaves[i].2 == ")" {
                    break;
                }
                continue;
            }
            if text == ")" {
                break;
            }
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected `,` or `)`, found {}", describe_g(g, text, &k)),
                sp,
            ));
        }
    }
    // 4) `)`。
    if i >= leaves.len() {
        return None;
    }
    {
        let (sp, _g, text) = at(i);
        if text != ")" {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected `)`, found {}", text),
                sp,
            ));
        }
        i += 1;
    }
    let name_span = leaf_span(cx, leaves[1].0);
    let rparen_span = leaf_span(cx, leaves[i - 1].0);
    // 5) 结果类型。
    if i >= leaves.len() {
        return None;
    }
    {
        let (sp, g, text) = at(i);
        // 旧 fn_item_tail 结果位分类：int/i32/bool 全域；text 面 ident 仅
        // str/Str/char；entry 面 ident 仅 void；record 面任意 ident（名义）。
        let builtin = text == "int" || text == "i32" || text == "bool";
        let text_word =
            g == k.tok_ident && cx.profile.is_text() && matches!(text, "str" | "Str" | "char");
        let entry_void = g == k.tok_ident && cx.profile.is_entry_unit() && text == "void";
        let record_name = g == k.tok_ident && cx.profile.is_record();
        if !builtin && !text_word && !entry_void && !record_name {
            if text == "{" {
                return Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "function return type annotation (`int`/`i32`/`bool` or a record type) is required",
                    name_span.join(rparen_span),
                ));
            }
            if g == k.tok_ident || g == k.tok_r_move || g == k.tok_r_to {
                return Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("type `{}` is not part of this source profile", text),
                    sp,
                ));
            }
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected return type, found {}", describe_g(g, text, &k)),
                sp,
            ));
        }
        i += 1;
    }
    // 6) `{`（体）→ 语句序列游走。
    if i >= leaves.len() {
        return None;
    }
    {
        let (sp, g, text) = at(i);
        if text != "{" {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected `{{`, found {}", describe_g(g, text, &k)),
                sp,
            ));
        }
        i += 1;
    }
    let mut w = Walker::new(cx, leaves);
    w.i = i;
    w.walk_block_body(false)
}

/// 旧 type_item 期望序列（type 关键字已匹配；record 族）。
fn walk_type_item(cx: &Cx, leaves: &[(u32, u32, String)]) -> Option<Diagnostic> {
    let k = cx.k;
    let mut i = 1usize;
    let at = |idx: usize| -> (Span, u32, &str) {
        let (n, g, s) = &leaves[idx.min(leaves.len() - 1)];
        (leaf_span(cx, *n), *g, s.as_str())
    };
    if leaves.len() < 2 {
        return None;
    }
    {
        let (sp, g, text) = at(1);
        if g != k.tok_ident {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected type name, found {}", describe_g(g, text, &k)),
                sp,
            ));
        }
        i = 2;
    }
    if leaves.len() <= i {
        return None;
    }
    {
        let (sp, _g, text) = at(i);
        if text != "{" {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected `{{`, found {}", text),
                sp,
            ));
        }
    }
    None
}

/// 旧 parse_stmt_head 期望序列（首切：声明缺 `=` / 类型词拒绝）。
fn walk_stmt(cx: &Cx, leaves: &[(u32, u32, String)]) -> Option<Diagnostic> {
    let k = cx.k;
    if leaves.is_empty() {
        return None;
    }
    let (sp0, g0, t0) = (leaf_span(cx, leaves[0].0), leaves[0].1, leaves[0].2.as_str());
    if t0 == "let" || t0 == "var" {
        if leaves.len() < 2 {
            return None;
        }
        let mut i = 1usize;
        let (_, g, text) = (leaf_span(cx, leaves[i].0), leaves[i].1, leaves[i].2.as_str());
        let _ = g;
        if !is_ident_text(text) {
            return None; // 名字位消息由叶层兜底
        }
        i += 1;
        // 可选注解（类型词）。
        if i < leaves.len() {
            let (_, g2, text2) = (leaf_span(cx, leaves[i].0), leaves[i].1, leaves[i].2.as_str());
            let starts_type = text2 == "int" || text2 == "i32" || text2 == "bool"
                || (g2 == k.tok_ident && (cx.profile.is_text() || cx.profile.is_record() || cx.profile.is_entry_unit()));
            if starts_type {
                if let Some(d) = walk_type_word(cx, leaves, &mut i, "locals must be initialized") {
                    return Some(d);
                }
            } else if g2 == k.tok_ident && !cx.profile.is_text() && !cx.profile.is_record() && !cx.profile.is_entry_unit() && matches!(text2, "str" | "Str" | "char") {
                return Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("type `{}` is not part of this source profile", text2),
                    leaf_span(cx, leaves[i].0),
                ));
            }
        }
        // `=`。
        if i < leaves.len() {
            let (sp, _g, text) = (leaf_span(cx, leaves[i].0), leaves[i].1, leaves[i].2.as_str());
            if text != "=" {
                // 旧：locals must be initialized（span = kw..最后已消费 token）。
                let kw = sp0;
                let last_end = leaf_span(cx, leaves[i - 1].0).end;
                return Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "locals must be initialized at declaration in this source profile",
                    Span { start: kw.start, end: last_end },
                ));
            }
        }
        return None;
    }
    None
}

fn is_ident_text(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => chars.all(|c| c.is_ascii_alphanumeric() || c == '_'),
        _ => false,
    }
}

// ---------------------------------------------------------------- 叶序列游走器
//
// 镜像旧 parser 的语句/表达式失败点（parse_stmt_head / expr 的帧机）：
// 只用于定位**首个**违反期望的叶并给出旧消息；不构造 AST。

struct Walker<'c, 'a> {
    cx: &'c Cx<'a>,
    leaves: &'c [(u32, u32, String)],
    i: usize,
    lines: Vec<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Br {
    Paren,
    Call,
    Ctor,
}

impl<'c, 'a> Walker<'c, 'a> {
    fn new(cx: &'c Cx<'a>, leaves: &'c [(u32, u32, String)]) -> Self {
        // 行号：按快照前缀换行计数（仅错误路径使用）。
        let total = cx.snap.len();
        let text = cx.snap.read_str(0, total).unwrap_or_default();
        let bytes = text.as_bytes();
        let mut lines = Vec::with_capacity(leaves.len());
        for (n, _, _) in leaves {
            let (s, e) = cx.tree.abs_range(*n);
            // 旧 lexer：多行 token 记结束行。
            let upto = (e as usize).min(bytes.len());
            let mut line = 1u32;
            for &b in &bytes[..upto] {
                if b == b'\n' {
                    line += 1;
                }
            }
            let _ = s;
            lines.push(line);
        }
        Walker { cx, leaves, i: 0, lines }
    }

    fn k(&self) -> &'a super::kinds::SourceKinds {
        self.cx.k
    }
    fn at(&self, idx: usize) -> Option<(Span, u32, &str, u32)> {
        let (n, g, s) = self.leaves.get(idx)?;
        Some((self.cx.span_of(*n), *g, s.as_str(), self.lines[idx]))
    }
    fn cur(&self) -> Option<(Span, u32, &str, u32)> {
        self.at(self.i)
    }
    fn eof_span(&self) -> Span {
        let end = self.cx.snap.len() as usize;
        Span { start: end, end }
    }
    fn syntax(&self, msg: String) -> Diagnostic {
        let span = self.cur().map(|c| c.0).unwrap_or_else(|| self.eof_span());
        Diagnostic::new(Stage::Text, "source.syntax", msg, span)
    }
    fn unsupported(&self, msg: String, span: Span) -> Diagnostic {
        Diagnostic::new(Stage::Text, "source.unsupported", msg, span)
    }
    fn describe_cur(&self) -> String {
        match self.cur() {
            Some((_, g, text, _)) => describe_g(g, text, self.k()),
            None => "end of file".to_string(),
        }
    }
    fn prev_line(&self) -> u32 {
        if self.i == 0 {
            1
        } else {
            self.lines[self.i - 1]
        }
    }
    fn is_kw(&self, g: u32, text: &str) -> bool {
        is_kw_leaf(g, self.k()) || text == "fn"
    }
    fn is_reserved(&self, g: u32) -> bool {
        g == self.k().tok_r_move || g == self.k().tok_r_to
    }
    fn is_raw(&self, g: u32) -> bool {
        let k = self.k();
        g == k.tok_affix_num || g == k.tok_str_lit || g == k.tok_char_lit || g == k.tok_block_open
    }

    /// 块体：语句序列到 `}`（或 EOF 错误）。
    fn walk_block_body(&mut self, script_top: bool) -> Option<Diagnostic> {
        loop {
            // skip semis
            while matches!(self.cur(), Some((_, _, ";", _))) {
                self.i += 1;
            }
            match self.cur() {
                None => {
                    if script_top {
                        return None;
                    }
                    return Some(self.syntax("expected statement or `}`, found end of file".into()));
                }
                Some((_, _, "}", _)) => {
                    self.i += 1;
                    return None;
                }
                Some(_) => {
                    if let Some(d) = self.walk_stmt_head(script_top) {
                        return Some(d);
                    }
                }
            }
        }
    }

    /// 语句分隔检查（`;`/`}`/换行）。
    fn sep_check(&self) -> Option<Diagnostic> {
        match self.cur() {
            None => None,
            Some((_, _, ";", _)) | Some((_, _, "}", _)) => None,
            Some((_, _, _, line)) if line > self.prev_line() => None,
            Some(_) => Some(self.syntax(format!(
                "expected `;`, newline or `}}`, found {}",
                self.describe_cur()
            ))),
        }
    }

    fn walk_stmt_head(&mut self, script_top: bool) -> Option<Diagnostic> {
        let k = self.k();
        let (sp, g, text, _) = self.cur()?;
        let wide = self.cx.profile.surface_is_scalar_or_wider();
        match text {
            "let" | "var" => {
                let kw = sp;
                self.i += 1;
                // name
                let (nsp, ng, ntext, _) = match self.cur() {
                    Some(c) => c,
                    None => return Some(self.syntax("expected local name, found end of file".into())),
                };
                if ng != k.tok_ident {
                    return Some(self.name_error(ng, ntext, nsp, "local name"));
                }
                self.i += 1;
                // 注解
                if let Some((asp, ag, atext, _)) = self.cur() {
                    let is_type = atext == "int" || atext == "i32" || atext == "bool";
                    let is_text = self.cx.profile.is_text();
                    if ag == k.tok_ident && !is_text && matches!(atext, "str" | "Str" | "char") {
                        return Some(self.unsupported(
                            format!("type `{}` is not part of this source profile", atext),
                            asp,
                        ));
                    }
                    let ident_type = ag == k.tok_ident
                        && (is_text && matches!(atext, "str" | "Str" | "char")
                            || self.cx.profile.is_entry_unit() && atext == "void"
                            || self.cx.profile.is_record());
                    if is_type || ident_type {
                        self.i += 1;
                    } else if ag == k.tok_ident || self.is_reserved(ag) {
                        return Some(self.unsupported(
                            format!("type `{}` is not part of this source profile", atext),
                            asp,
                        ));
                    }
                }
                // `=`
                match self.cur() {
                    Some((_, _, "=", _)) => {
                        self.i += 1;
                    }
                    _ => {
                        let last_end = self.at(self.i - 1).map(|c| c.0.end).unwrap_or(kw.end);
                        return Some(self.unsupported(
                            "locals must be initialized at declaration in this source profile".into(),
                            Span { start: kw.start, end: last_end },
                        ));
                    }
                }
                if let Some(d) = self.walk_expr(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "return" => {
                self.i += 1;
                if script_top {
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "source.entry-top-return",
                        "a script's result is its trailing expression; `return` is not part of the script form (use a declared `fn main` for an explicit exit)",
                        sp,
                    ));
                }
                if let Some(d) = self.walk_expr(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "if" => {
                self.i += 1;
                if let Some(d) = self.walk_expr(false) {
                    return Some(d);
                }
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                if let Some(d) = self.walk_block_body(false) {
                    return Some(d);
                }
                if matches!(self.cur(), Some((_, _, "else", _))) {
                    self.i += 1;
                    if let Some(d) = self.expect_text("{") {
                        return Some(d);
                    }
                    if let Some(d) = self.walk_block_body(false) {
                        return Some(d);
                    }
                }
                self.sep_check()
            }
            "while" => {
                if wide {
                    return Some(self.unsupported(
                        "`while` is not part of this source profile; conditional loops use `for <condition> { ... }`".into(),
                        sp,
                    ));
                }
                self.i += 1;
                if let Some(d) = self.walk_expr(false) {
                    return Some(d);
                }
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                if let Some(d) = self.walk_block_body(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "for" => {
                if !wide {
                    return Some(self.unsupported("`for` is not part of this source profile".into(), sp));
                }
                self.i += 1;
                if let Some(d) = self.walk_expr(false) {
                    return Some(d);
                }
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                if let Some(d) = self.walk_block_body(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "loop" => {
                self.i += 1;
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                if let Some(d) = self.walk_block_body(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "break" | "continue" => {
                self.i += 1;
                self.sep_check()
            }
            _ => {
                if self.is_reserved(g) {
                    return Some(self.unsupported(
                        format!("`{}` is not part of this source profile", text),
                        sp,
                    ));
                }
                if g == k.tok_ident {
                    // 赋值/路径/表达式语句（旧投机臂）。
                    if let Some((_, _, t2, _)) = self.at(self.i + 1) {
                        if t2 == "=" || t2 == "." {
                            // 路径：ident (. ident)* 然后 `=`
                            let mark = self.i;
                            self.i += 1;
                            let mut is_assign = true;
                            while matches!(self.cur(), Some((_, _, ".", _))) {
                                if !self.cx.profile.is_record() {
                                    let dsp = self.cur().unwrap().0;
                                    return Some(self.unsupported(
                                        "field access is not part of this source profile".into(),
                                        dsp,
                                    ));
                                }
                                self.i += 1;
                                match self.cur() {
                                    Some((_, fg, ftext, _)) if fg == k.tok_ident => {
                                        let _ = ftext;
                                        self.i += 1;
                                    }
                                    Some((fsp, fg, ftext, _)) => {
                                        return Some(self.name_error(fg, ftext, fsp, "field name after `.`"));
                                    }
                                    None => {
                                        return Some(self.syntax(
                                            "expected field name after `.`, found end of file".into(),
                                        ))
                                    }
                                }
                            }
                            if !matches!(self.cur(), Some((_, _, "=", _))) {
                                is_assign = false;
                            }
                            if !is_assign {
                                self.i = mark;
                                let start = self.cur().map(|c| c.0).unwrap_or(sp);
                                if let Some(d) = self.walk_expr(false) {
                                    return Some(d);
                                }
                                let end = self.at(self.i - 1).map(|c| c.0.end).unwrap_or(start.end);
                                if !self.cx.profile.is_entry_unit() {
                                    return Some(self.unsupported(
                                        "expression statements are not part of this source profile; calls are only allowed in initializers, assignments, returns and conditions".into(),
                                        Span { start: start.start, end },
                                    ));
                                }
                                return self.sep_check();
                            }
                            self.i += 1; // `=`
                            if let Some(d) = self.walk_expr(false) {
                                return Some(d);
                            }
                            return self.sep_check();
                        }
                    }
                }
                if g == k.tok_ident || text == "true" || text == "false" || g == k.tok_int_lit {
                    let start = sp;
                    if let Some(d) = self.walk_expr(false) {
                        return Some(d);
                    }
                    let end = self.at(self.i - 1).map(|c| c.0.end).unwrap_or(start.end);
                    if !self.cx.profile.is_entry_unit() {
                        return Some(self.unsupported(
                            "expression statements are not part of this source profile; calls are only allowed in initializers, assignments, returns and conditions".into(),
                            Span { start: start.start, end },
                        ));
                    }
                    return self.sep_check();
                }
                Some(self.syntax(format!("expected statement, found {}", self.describe_cur())))
            }
        }
    }

    fn name_error(&self, g: u32, text: &str, sp: Span, what: &str) -> Diagnostic {
        if self.is_raw(g) {
            return self.unsupported(
                format!("{} cannot be used as a name in this source profile", raw_describe(g, text, self.k())),
                sp,
            );
        }
        if self.is_kw(g, text) || self.is_reserved(g) {
            return self.unsupported(format!("keyword `{}` cannot be used as a name", text), sp);
        }
        self.syntax(format!("expected {}, found {}", what, describe_g(g, text, self.k())))
    }

    fn expect_text(&mut self, t: &str) -> Option<Diagnostic> {
        match self.cur() {
            Some((_, _, x, _)) if x == t => {
                self.i += 1;
                None
            }
            _ => Some(self.syntax(format!("expected `{}`, found {}", t, self.describe_cur()))),
        }
    }

    /// 二元算子表（按 profile）：Some(prec) = 本面算子；None = 非算子。
    fn prec_of(&self, text: &str) -> Option<u8> {
        if self.cx.profile.surface_is_scalar_or_wider() {
            match text {
                "||" => Some(1),
                "&&" => Some(2),
                "==" | "!=" => Some(3),
                "<" | ">" | "<=" | ">=" => Some(4),
                "+" | "-" => Some(5),
                "*" | "/" | "%" => Some(6),
                _ => None,
            }
        } else {
            match text {
                "<" => Some(1),
                "+" => Some(2),
                "*" => Some(3),
                _ => None,
            }
        }
    }
    fn op_of_other_surface(&self, text: &str) -> bool {
        if self.cx.profile.surface_is_scalar_or_wider() {
            return false;
        }
        matches!(text, "||" | "&&" | "==" | "!=" | ">" | "<=" | ">=" | "-" | "/" | "%")
    }

    /// 表达式游走：primary + 算子/关闭循环（括号帧跟踪，行纪律）。
    fn walk_expr(&mut self, _bracketed: bool) -> Option<Diagnostic> {
        let k = self.k();
        let mut frames: Vec<Br> = Vec::new();
        let mut anchored = false; // 算子后需同行 primary
        let wide = self.cx.profile.surface_is_scalar_or_wider();
        loop {
            // ---- primary ----
            let in_br = !frames.is_empty();
            if anchored && !in_br {
                if let Some((psp, _, _, line)) = self.cur() {
                    if line != self.prev_line() {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            "unbracketed expression cannot continue across a newline",
                            psp,
                        ));
                    }
                }
            }
            let Some((sp, g, text, line)) = self.cur() else {
                return match frames.last() {
                    Some(Br::Call) => Some(self.syntax("expected argument expression".into())),
                    Some(Br::Paren) => Some(self.syntax("expected `)`".into())),
                    Some(Br::Ctor) => Some(self.syntax("unterminated expression".into())),
                    None => Some(self.syntax(format!("expected expression, found {}", self.describe_cur()))),
                };
            };
            if g == k.tok_int_lit {
                self.i += 1;
            } else if text == "-" {
                match self.at(self.i + 1) {
                    Some((dsp, dg, _, dline)) if dg == k.tok_int_lit => {
                        if !in_br && dline != line {
                            return Some(Diagnostic::new(
                                Stage::Text,
                                "source.syntax",
                                "unbracketed expression cannot continue across a newline",
                                dsp,
                            ));
                        }
                        self.i += 2;
                    }
                    _ if wide => {
                        self.i += 1;
                        anchored = true;
                        continue;
                    }
                    Some((esp, eg, etext, _)) if eg == k.tok_ident || etext == "true" || etext == "false" || etext == "(" => {
                        return Some(self.unsupported(
                            "unary minus is only supported directly before an integer literal in this source profile".into(),
                            sp.join(esp),
                        ));
                    }
                    _ => return Some(self.syntax("expected expression after `-`".into())),
                }
            } else if g == k.tok_str_lit || g == k.tok_char_lit {
                if self.cx.profile.admits_text_literals() {
                    self.i += 1;
                } else {
                    return Some(self.unsupported(
                        format!("{} `{}` is not part of this source profile", raw_describe(g, text, k), text),
                        sp,
                    ));
                }
            } else if text == "true" || text == "false" {
                self.i += 1;
            } else if text == "!" {
                if wide {
                    self.i += 1;
                    anchored = true;
                    continue;
                }
                return Some(self.unsupported("unary `!` is not part of this source profile".into(), sp));
            } else if text == "+" && wide {
                return Some(self.unsupported("unary `+` is not part of this source profile".into(), sp));
            } else if text == "type" || text == "static" || text == "mut" {
                return Some(self.unsupported(format!("`{}` is not a value in this source profile", text), sp));
            } else if self.is_reserved(g) {
                return Some(self.unsupported(format!("keyword `{}` is not part of this source profile", text), sp));
            } else if self.is_raw(g) {
                let msg = if g == k.tok_affix_num || g == k.tok_block_open {
                    format!("{} `{}` is not part of this source profile", raw_describe(g, text, k), text)
                } else {
                    format!("{} `{}` is not part of this source profile", raw_describe(g, text, k), text)
                };
                return Some(self.unsupported(msg, sp));
            } else if text == "(" {
                self.i += 1;
                frames.push(Br::Paren);
                anchored = false;
                continue;
            } else if text == "." && self.cx.profile.is_record() {
                // leading-dot self
                self.i += 1;
                match self.cur() {
                    Some((_, mg, _, _)) if mg == k.tok_ident => {
                        self.i += 1;
                        if matches!(self.cur(), Some((_, _, "(", l2)) if in_br || l2 == self.prev_line()) {
                            self.i += 1;
                            if matches!(self.cur(), Some((_, _, ")", _))) {
                                self.i += 1;
                            } else {
                                frames.push(Br::Call);
                                anchored = false;
                                continue;
                            }
                        }
                    }
                    _ => {
                        return Some(self.syntax(format!(
                            "expected field or method name after `.`, found {}",
                            self.describe_cur()
                        )))
                    }
                }
            } else if g == k.tok_ident {
                // ctor（record 面，`{` 同行或括号内）
                if self.cx.profile.is_record() {
                    if let Some((_, _, "{", l2)) = self.at(self.i + 1) {
                        if in_br || l2 == line {
                            self.i += 2;
                            if matches!(self.cur(), Some((_, _, "}", _))) {
                                self.i += 1;
                            } else {
                                frames.push(Br::Ctor);
                                // label?
                                self.take_label();
                                anchored = false;
                                continue;
                            }
                            // value complete
                            if let Some(d) = self.after_value(&mut frames, &mut anchored) {
                                return d;
                            }
                            continue;
                        }
                    }
                }
                // call（`(` 同行或括号内）
                if let Some((_, _, "(", l2)) = self.at(self.i + 1) {
                    if in_br || l2 == line {
                        self.i += 2;
                        if matches!(self.cur(), Some((_, _, ")", _))) {
                            self.i += 1;
                        } else {
                            frames.push(Br::Call);
                            self.take_label();
                            anchored = false;
                            continue;
                        }
                    } else {
                        self.i += 1;
                    }
                } else {
                    self.i += 1;
                }
            } else if self.is_kw(g, text) || text == ")" || text == "}" || text == "," || text == ";" || text == "{" || text == "=" || text == ":" || text == "]" || text == "[" {
                return Some(self.syntax(format!(
                    "expected expression, found {}",
                    if g == k.tok_ident { "token".to_string() } else { format!("`{}`", text) }
                )));
            } else {
                return Some(self.syntax(format!("expected expression, found `{}`", text)));
            }
            // ---- operator / close loop ----
            anchored = false;
            if let Some(d) = self.after_value(&mut frames, &mut anchored) {
                return d;
            }
            if frames.is_empty() && !anchored {
                return None;
            }
            if anchored {
                continue;
            }
            // 括号内等待下一 primary（刚 push 了帧）
        }
    }

    fn take_label(&mut self) {
        if let (Some((_, g, _, _)), Some((_, _, ":", _))) = (self.cur(), self.at(self.i + 1)) {
            if g == self.k().tok_ident {
                self.i += 2;
            }
        }
    }

    /// 值完成后：算子/点后缀/关闭括号/逗号。返回 Some(Some(diag)) = 错误；
    /// Some(None) = 表达式结束；None = 继续（已消费算子，anchored 置位）。
    fn after_value(&mut self, frames: &mut Vec<Br>, anchored: &mut bool) -> Option<Option<Diagnostic>> {
        let k = self.k();
        loop {
            let in_br = !frames.is_empty();
            // 点后缀
            if let Some((dsp, _, ".", dline)) = self.cur() {
                let is_record = self.cx.profile.is_record();
                let is_text = self.cx.profile.is_text();
                let next = self.at(self.i + 1);
                let next_is_to = matches!(next, Some((_, ng, _, _)) if ng == k.tok_r_to);
                if is_record && !(next_is_to && is_text) {
                    if !in_br && dline != self.prev_line() {
                        return Some(Some(Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            "unbracketed expression cannot continue across a newline",
                            dsp,
                        )));
                    }
                    self.i += 1;
                    match self.cur() {
                        Some((_, mg, _, _)) if mg == k.tok_r_move && (is_text) => {
                            self.i += 1;
                            continue;
                        }
                        Some((_, _, "mut", _)) => {
                            self.i += 1;
                            continue;
                        }
                        Some((_, mg, _, mline)) if mg == k.tok_ident => {
                            self.i += 1;
                            if matches!(self.cur(), Some((_, _, "(", l2)) if in_br || l2 == mline) {
                                self.i += 1;
                                if matches!(self.cur(), Some((_, _, ")", _))) {
                                    self.i += 1;
                                    continue;
                                }
                                frames.push(Br::Call);
                                self.take_label();
                                *anchored = false;
                                return None;
                            }
                            continue;
                        }
                        _ => {
                            return Some(Some(self.syntax(format!(
                                "expected field or method name after `.`, found {}",
                                self.describe_cur()
                            ))))
                        }
                    }
                }
                if is_text {
                    match next {
                        Some((msp, mg, _, mline)) if mg == k.tok_r_move => {
                            if !in_br && mline != self.prev_line() {
                                return Some(Some(Diagnostic::new(
                                    Stage::Text,
                                    "source.syntax",
                                    "unbracketed expression cannot continue across a newline",
                                    msp,
                                )));
                            }
                            self.i += 2;
                            continue;
                        }
                        Some((_, mg, _, _)) if mg == k.tok_r_to => {
                            self.i += 2;
                            if let Some(d) = self.expect_text("(") {
                                return Some(Some(d));
                            }
                            match self.cur() {
                                Some((_, wg, w, _)) if wg == k.tok_ident && (w == "str" || w == "Str") => {
                                    self.i += 1;
                                }
                                _ => return Some(Some(self.syntax("`.to` expects `str` or `Str`".into()))),
                            }
                            if let Some(d) = self.expect_text(")") {
                                return Some(Some(d));
                            }
                            continue;
                        }
                        Some((_, mg, mname, mline)) if mg == k.tok_ident => {
                            let call = matches!(self.at(self.i + 2), Some((_, _, "(", l3)) if in_br || l3 == mline);
                            if !call {
                                return Some(Some(self.syntax(format!("method `.{}` requires parentheses", mname))));
                            }
                            self.i += 3;
                            if matches!(self.cur(), Some((_, _, ")", _))) {
                                self.i += 1;
                                continue;
                            }
                            frames.push(Br::Call);
                            self.take_label();
                            *anchored = false;
                            return None;
                        }
                        _ => return Some(Some(self.syntax("expected a method name after `.`".into()))),
                    }
                }
                // 非 record/text：`.` 不是算子 → unsupported_op
                return Some(Some(self.unsupported("operator `.` is not part of this source profile".into(), dsp)));
            }
            let Some((sp, _g, text, line)) = self.cur() else {
                return Some(match frames.last() {
                    Some(Br::Call) => Some(self.syntax("expected `,` or `)`".into())),
                    Some(Br::Paren) => Some(self.syntax("expected `)`".into())),
                    Some(Br::Ctor) => Some(self.syntax("unterminated expression".into())),
                    None => None,
                });
            };
            if let Some(_) = self.prec_of(text) {
                if !in_br && line != self.prev_line() {
                    // 算子换行：表达式结束
                    return Some(match frames.last() {
                        Some(Br::Call) => Some(self.syntax("expected `,` or `)`".into())),
                        Some(Br::Paren) => Some(self.syntax("expected `)`".into())),
                        _ => None,
                    });
                }
                self.i += 1;
                *anchored = true;
                return None;
            }
            if self.op_of_other_surface(text) {
                return Some(Some(self.unsupported(format!("operator `{}` is not part of this source profile", text), sp)));
            }
            match text {
                "(" => return Some(Some(self.syntax(format!("expected operator, found {}", self.describe_cur())))),
                ")" => match frames.last() {
                    Some(Br::Paren) | Some(Br::Call) => {
                        frames.pop();
                        self.i += 1;
                        continue;
                    }
                    _ => return Some(None),
                },
                "," => match frames.last() {
                    Some(Br::Call) => {
                        self.i += 1;
                        self.take_label();
                        *anchored = false;
                        return None;
                    }
                    Some(Br::Ctor) => {
                        self.i += 1;
                        self.take_label();
                        *anchored = false;
                        return None;
                    }
                    _ => return Some(Some(self.syntax("expected operator, found `,`".into()))),
                },
                "}" => match frames.last() {
                    Some(Br::Ctor) => {
                        frames.pop();
                        self.i += 1;
                        continue;
                    }
                    Some(Br::Call) => return Some(Some(self.syntax("expected `,` or `)`".into()))),
                    Some(Br::Paren) => return Some(Some(self.syntax("expected `)`".into()))),
                    None => return Some(None),
                },
                _ => {
                    if matches!(frames.last(), Some(Br::Ctor)) && line > self.prev_line() {
                        self.take_label();
                        *anchored = false;
                        return None;
                    }
                    return Some(match frames.last() {
                        Some(Br::Call) => Some(self.syntax("expected `,` or `)`".into())),
                        Some(Br::Paren) => Some(self.syntax("expected `)`".into())),
                        Some(Br::Ctor) => Some(self.syntax("expected `,` or `)`".into())),
                        None => None,
                    });
                }
            }
        }
    }
}

/// 类型词游走（int/i32/bool/门控 ident；失败给 required 消息或类型拒绝）。
fn walk_type_word(
    cx: &Cx,
    leaves: &[(u32, u32, String)],
    i: &mut usize,
    required_msg: &str,
) -> Option<Diagnostic> {
    let k = cx.k;
    if *i >= leaves.len() {
        return None;
    }
    let (sp, g, text) = (leaf_span(cx, leaves[*i].0), leaves[*i].1, leaves[*i].2.as_str());
    let is_builtin = text == "int" || text == "i32" || text == "bool";
    let ident_ok = g == k.tok_ident
        && (cx.profile.is_text() || cx.profile.is_record() || cx.profile.is_entry_unit());
    if is_builtin || ident_ok {
        // text 族拼写分类 + 非 text 的 str/Str/char 拒绝由投影/叶层处理；
        // 这里只推进。
        *i += 1;
        return None;
    }
    if g == k.tok_ident && !cx.profile.is_text() && matches!(text, "str" | "Str" | "char") {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            format!("type `{}` is not part of this source profile", text),
            sp,
        ));
    }
    if g == k.tok_ident || g == k.tok_r_move || g == k.tok_r_to {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            format!("type `{}` is not part of this source profile", text),
            sp,
        ));
    }
    let _ = required_msg;
    None
}

fn is_kw_leaf(g: u32, k: &super::kinds::SourceKinds) -> bool {
    [
        k.tok_let, k.tok_var, k.tok_if, k.tok_else, k.tok_while, k.tok_loop, k.tok_break,
        k.tok_continue, k.tok_return, k.tok_true, k.tok_false, k.tok_for, k.tok_int_kw,
        k.tok_i32, k.tok_bool, k.tok_type_kw, k.tok_static, k.tok_mut, k.tok_enum_kw,
        k.tok_is, k.tok_in,
    ]
    .contains(&g)
}

fn raw_describe(g: u32, _text: &str, k: &super::kinds::SourceKinds) -> &'static str {
    if g == k.tok_affix_num {
        "suffixed/radix/float numeric literal"
    } else if g == k.tok_str_lit {
        "string literal"
    } else if g == k.tok_char_lit {
        "char literal"
    } else {
        "block comment"
    }
}

fn describe_g(g: u32, text: &str, k: &super::kinds::SourceKinds) -> String {
    if g == k.tok_int_lit {
        "integer literal".to_string()
    } else if g == k.tok_str_lit {
        "string literal".to_string()
    } else if g == k.tok_char_lit {
        "char literal".to_string()
    } else if g == k.tok_affix_num {
        "suffixed/radix/float numeric literal".to_string()
    } else if g == k.tok_block_open {
        "block comment".to_string()
    } else {
        format!("`{}`", text)
    }
}

fn describe_leaf_text(cx: &Cx, tree: &CstTree, leaf: Option<u32>) -> String {
    let Some(l) = leaf else {
        return "end of file".to_string();
    };
    let g = tree.kind(l);
    let text = cx.spelling(l);
    if g == cx.k.tok_ident {
        format!("`{}`", text)
    } else if g == cx.k.tok_int_lit {
        "integer literal".to_string()
    } else if g == cx.k.tok_str_lit {
        "string literal".to_string()
    } else if g == cx.k.tok_char_lit {
        "char literal".to_string()
    } else if g == cx.k.tok_affix_num {
        "suffixed/radix/float numeric literal".to_string()
    } else if g == cx.k.tok_block_open {
        "block comment".to_string()
    } else {
        format!("`{}`", text)
    }
}

fn first_meaningful_leaf(cx: &Cx, tree: &CstTree, node: u32) -> Option<u32> {
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        if nd.flags & FLAG_TRIVIA != 0 {
            continue;
        }
        if nd.child_count == 0 {
            let (s, e) = tree.abs_range(n);
            if e > s {
                return Some(n);
            }
            continue;
        }
        let kids = tree.children(n);
        for c in kids.into_iter().rev() {
            stack.push(c);
        }
    }
    None
}

fn path_to(tree: &CstTree, from: u32, target: u32, path: &mut Vec<u32>) -> bool {
    if from == target {
        return true;
    }
    path.push(tree.kind(from));
    for c in tree.children(from) {
        if path_to(tree, c, target, path) {
            return true;
        }
    }
    path.pop();
    false
}

// ---------------------------------------------------------------- engine

enum Val {
    E(Expr),
    A(Arg),
    CF(NamedCtorField),
    S(Stmt),
    B(Block),
    F(FnItem),
}

enum Frame {
    // -- 块/语句 --
    Block(BlockFrame),
    LetFinish { node: u32, kw: Span, name: Ident, ann: Option<TypeAnn>, mutable: bool },
    AssignFinish { name: Ident, path: Vec<Ident> },
    ReturnFinish { kw: Span },
    IfCond { then_n: u32, els_blk: Option<u32>, node: u32 },
    IfThen { cond: Expr, els_blk: Option<u32>, node: u32 },
    IfElse { cond: Expr, then: Block, node: u32 },
    WhileCond { body_n: u32, node: u32 },
    WhileBody { cond: Expr, node: u32 },
    LoopBody { node: u32 },
    ExprStmtFinish,
    FnBodyFinish { node: u32, name: Ident, params: Vec<Param>, ret: TypeAnn, receiver: Receiver, owner: Option<Ident> },
    ScriptFinish { start_span: Span },
    // -- 表达式 --
    ExprEnter(u32),
    OpPost { op: BinOp },
    UnaryPost { op: UnaryOp },
    PostfixFoldAll { steps: Vec<u32> },
    /// 调用收尾：callee 语义（纯名字 / 方法链尾步）+ 一元链 + 实参数 +
    /// 调用节点终点。接收者值（方法路径）在实参之下。
    CallAssemble {
        callee: CalleePlan,
        argc: usize,
        close_end: usize,
    },
    ArgBuild { named: Option<Ident> },
    CtorFieldBuild { label: Ident },
    CtorFinish { node: u32, ty: Ident, count: usize },
}

/// 调用 callee 的静态分析结果（enter_call 时求出；组装时消费）。
enum CalleePlan {
    /// 纯名字调用（ident 叶）+ 外→内一元链。
    Plain { ident: Ident, unary_ops: Vec<UnaryOp> },
    /// 方法链：base（接收者求值任务节点）+ 中间步 + 链尾成员叶。
    Method { base: u32, member: u32, mids: Vec<u32>, unary_ops: Vec<UnaryOp> },
}

/// 块帧：语句节点序列 + 组装 span。
struct BlockFrame {
    nodes: Vec<u32>,
    cursor: usize,
    stmts: Vec<Stmt>,
    script_top: bool,
    span: Span,
}

pub(crate) struct Cx<'a> {
    p: &'a GrammarProgram,
    k: &'a SourceKinds,
    tree: &'a CstTree,
    snap: &'a Snapshot,
    profile: SourceProfile,
    visited: usize,
    diags: Vec<(u64, Diagnostic)>,
    over_budget: bool,
}

impl<'a> Cx<'a> {
    fn reject(&mut self, span: Span, make: impl FnOnce() -> Diagnostic) {
        let d = make();
        self.diags.push((span.start as u64, d));
    }
    fn first_diag(&self) -> Option<Diagnostic> {
        self.diags
            .iter()
            .min_by_key(|(pos, _)| *pos)
            .map(|(_, d)| d.clone())
    }
    fn span_of(&self, n: u32) -> Span {
        let (s, e) = self.tree.abs_range(n);
        Span { start: s as usize, end: e as usize }
    }
    fn spelling(&self, n: u32) -> String {
        let (s, e) = self.tree.abs_range(n);
        self.snap.read_str(s, e).unwrap_or_default()
    }
    fn children(&self, n: u32) -> Vec<u32> {
        self.tree.children(n)
    }
    fn field_child(&self, n: u32, field: &str) -> Option<u32> {
        let f = self.p.field_id(field)?;
        self.tree.field_child(n, f)
    }
    fn child_of_kind(&self, n: u32, kind: u32) -> Option<u32> {
        self.children(n).into_iter().find(|&c| self.tree.kind(c) == kind)
    }
    fn check_name(&mut self, id: &Ident) {
        if self.profile == SourceProfile::HostConsole
            && matches!(id.name.as_str(), "print" | "eprint")
        {
            let span = id.span;
            self.reject(span, || {
                Diagnostic::new(
                    Stage::Text,
                    "source.console-reserved",
                    "console names cannot be bound in the host-console profile",
                    span,
                )
            });
        }
    }
    fn bump(&mut self) -> bool {
        self.visited += 1;
        if self.visited > PROJECTION_NODE_BUDGET {
            if !self.over_budget {
                self.over_budget = true;
                let z = Span { start: 0, end: 0 };
                self.reject(z, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.resource",
                        "projection work budget exceeded",
                        z,
                    )
                });
            }
            return false;
        }
        true
    }
}

struct Engine<'a> {
    cx: Cx<'a>,
    stack: Vec<Frame>,
    vals: Vec<Val>,
}

fn first_leaf_span(cx: &Cx, node: u32) -> Span {
    let mut n = node;
    loop {
        let nd = cx.tree.node(n);
        if nd.flags & FLAG_TRIVIA != 0 || nd.child_count == 0 {
            break;
        }
        let kids = cx.tree.children(n);
        match kids.iter().find(|&&c| cx.tree.node(c).flags & FLAG_TRIVIA == 0) {
            Some(&c) => n = c,
            None => break,
        }
    }
    cx.span_of(n)
}

fn ident_of(cx: &mut Cx, leaf: u32) -> Ident {
    let id = Ident {
        name: cx.spelling(leaf),
        span: cx.span_of(leaf),
    };
    cx.check_name(&id);
    id
}

impl<'a> Engine<'a> {
    fn new(p: &'a GrammarProgram, k: &'a SourceKinds, tree: &'a CstTree, snap: &'a Snapshot, profile: SourceProfile) -> Self {
        Engine {
            cx: Cx {
                p,
                k,
                tree,
                snap,
                profile,
                visited: 0,
                diags: Vec::new(),
                over_budget: false,
            },
            stack: Vec::new(),
            vals: Vec::new(),
        }
    }

    fn pop_e(&mut self) -> Expr {
        match self.vals.pop() {
            Some(Val::E(e)) => e,
            _ => Expr { kind: ExprKind::IntLit(0), span: Span { start: 0, end: 0 } },
        }
    }
    fn pop_b(&mut self) -> Block {
        match self.vals.pop() {
            Some(Val::B(b)) => b,
            _ => Block { stmts: Vec::new(), span: Span { start: 0, end: 0 } },
        }
    }
    fn pop_s(&mut self) -> Stmt {
        match self.vals.pop() {
            Some(Val::S(s)) => s,
            _ => Stmt::Break { span: Span { start: 0, end: 0 } },
        }
    }

    fn drive(&mut self) {
        while let Some(frame) = self.stack.pop() {
            if !self.cx.bump() {
                // 预算耗尽：保留诊断，清栈提前结束（值由调用方兜底）。
                self.stack.clear();
                return;
            }
            self.step(frame);
        }
    }

    // ------------------------------------------------------------ items

    fn run_fn_item(&mut self, node: u32, owner: Option<Ident>, receiver: Receiver) -> FnItem {
        let tail = self
            .cx
            .children(node)
            .into_iter()
            .find(|&c| self.cx.tree.kind(c) == self.cx.k.fn_tail)
            .expect("fn-item carries fn-tail");
        let name_leaf = self.cx.field_child(tail, "name").expect("fn name");
        let name = ident_of(&mut self.cx, name_leaf);
        if receiver == Receiver::View && name.name == "self" {
            let span = name.span;
            self.cx.reject(span, || self_method_name(span));
        }
        let params = self.project_params(self.cx.field_child(tail, "params"));
        if receiver == Receiver::View {
            for prm in &params {
                if prm.name.name == "self" {
                    let span = prm.name.span;
                    self.cx.reject(span, || self_param(span));
                }
            }
        }
        let ret = self.classify_word(
            self.cx.field_child(tail, "ty").expect("fn ret"),
        );
        let body_n = self.cx.field_child(tail, "body").expect("fn body");
        let owner_for_frame = owner.clone();
        self.stack.push(Frame::FnBodyFinish {
            node,
            name,
            params,
            ret,
            receiver,
            owner: owner_for_frame,
        });
        self.enter_block(body_n);
        self.drive();
        match self.vals.pop() {
            Some(Val::F(f)) => f,
            _ => FnItem {
                name: Ident { name: String::new(), span: Span { start: 0, end: 0 } },
                params: Vec::new(),
                ret: TypeAnn { kind: TypeRef::Builtin(TypeWord::Int), span: Span { start: 0, end: 0 } },
                body: Block { stmts: Vec::new(), span: Span { start: 0, end: 0 } },
                span: Span { start: 0, end: 0 },
                receiver,
                owner: owner.clone(),
                script: false,
            },
        }
    }

    fn project_params(&mut self, params_node: Option<u32>) -> Vec<Param> {
        let mut out = Vec::new();
        let Some(pn) = params_node else { return out };
        let direct = collect_seq(&self.cx, pn, self.cx.k.param, "param-tail");
        for param_n in direct {
            let name_leaf = self.cx.field_child(param_n, "name").expect("param name");
            let name = ident_of(&mut self.cx, name_leaf);
            let mode_leaf = self.cx.field_child(param_n, "mode");
            let mode = match mode_leaf.map(|m| self.cx.tree.kind(m)) {
                Some(mk) if mk == self.cx.k.tok_r_move => ParamMode::Move,
                Some(mk) if mk == self.cx.k.tok_mut => ParamMode::Mut,
                _ => ParamMode::Default,
            };
            let ty = self.classify_word(self.cx.field_child(param_n, "ty").expect("param ty"));
            let span = match mode_leaf {
                Some(m) => self.cx.span_of(m).join(ty.span),
                None => name.span.join(ty.span),
            };
            out.push(Param { span, name, ty, mode });
        }
        out
    }

    /// ret-ty/ann-ty → TypeAnn（词法分类 + profile 拒绝；非递归）。
    fn classify_word(&mut self, node: u32) -> TypeAnn {
        let word_leaf = self
            .cx
            .children(node)
            .into_iter()
            .find(|&c| {
                let k = self.cx.tree.kind(c);
                k == self.cx.k.tok_int_kw
                    || k == self.cx.k.tok_i32
                    || k == self.cx.k.tok_bool
                    || k == self.cx.k.tok_ident
            })
            .expect("type word leaf");
        let span = self.cx.span_of(word_leaf);
        let int = |tw: TypeWord| TypeAnn { kind: TypeRef::Builtin(tw), span };
        match self.cx.tree.kind(word_leaf) {
            x if x == self.cx.k.tok_int_kw => return int(TypeWord::Int),
            x if x == self.cx.k.tok_i32 => return int(TypeWord::I32),
            x if x == self.cx.k.tok_bool => return int(TypeWord::Bool),
            _ => {}
        }
        let word = self.cx.spelling(word_leaf);
        let is_text = self.cx.profile.is_text();
        if is_text && matches!(word.as_str(), "str" | "Str" | "char") {
            let tw = match word.as_str() {
                "str" => TypeWord::Str,
                "Str" => TypeWord::StrOwned,
                _ => TypeWord::Char,
            };
            return int(tw);
        }
        if !is_text && matches!(word.as_str(), "str" | "Str" | "char") {
            self.cx.reject(span, move || unsupported_ty(&word, span));
            return int(TypeWord::Int);
        }
        if self.cx.profile.is_entry_unit() && word == "void" {
            return int(TypeWord::Void);
        }
        if self.cx.profile.is_record() {
            return TypeAnn { kind: TypeRef::Record(Ident { name: word, span }), span };
        }
        self.cx.reject(span, move || unsupported_ty(&word, span));
        int(TypeWord::Int)
    }

    fn run_type_item(&mut self, node: u32) -> TypeItem {
        let name_leaf = self.cx.field_child(node, "name").expect("type name");
        let name = ident_of(&mut self.cx, name_leaf);
        let mut fields: Vec<FieldDecl> = Vec::new();
        let mut methods: Vec<FnItem> = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for c_raw in self.cx.children(node) {
            // member 是分派节点：取首个非 trivia 孩子（field/static/view）。
            let c = self
                .cx
                .children(c_raw)
                .into_iter()
                .find(|&x| self.cx.tree.node(x).flags & FLAG_TRIVIA == 0)
                .unwrap_or(c_raw);
            let kind = self.cx.tree.kind(c);
            if kind == self.cx.k.field_member {
                let fname_leaf = self.cx.field_child(c, "name").expect("field name");
                let fname = ident_of(&mut self.cx, fname_leaf);
                if fname.name == "self" {
                    let span = fname.span;
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`self` cannot be a field name",
                            span,
                        )
                    });
                }
                let ty = self.classify_word(self.cx.field_child(c, "ty").expect("field ty"));
                let span = fname.span.join(ty.span);
                if seen.contains(&fname.name) {
                    let (f, t) = (fname.name.clone(), name.name.clone());
                    let sp = fname.span;
                    self.cx.reject(sp, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.duplicate",
                            format!("field `{}` is declared more than once in type `{}`", f, t),
                            sp,
                        )
                    });
                }
                seen.push(fname.name.clone());
                fields.push(FieldDecl { span, name: fname, ty });
            } else if kind == self.cx.k.static_method || kind == self.cx.k.view_method {
                let receiver = if kind == self.cx.k.static_method {
                    Receiver::Static
                } else {
                    Receiver::View
                };
                methods.push(self.run_fn_item(c, Some(name.clone()), receiver));
            }
        }
        if fields.is_empty() {
            let span = self.cx.span_of(node);
            self.cx.reject(span, || {
                Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "empty records are not part of this profile; a record must declare at least one field",
                    span,
                )
            });
        }
        TypeItem { span: self.cx.span_of(node), name, fields, methods }
    }

    fn run_script_item(&mut self, node: u32) -> FnItem {
        let start_span = first_leaf_span(&self.cx, node);
        self.stack.push(Frame::ScriptFinish { start_span });
        let stmts: Vec<u32> = self
            .cx
            .children(node)
            .into_iter()
            .filter(|&c| self.cx.tree.kind(c) == self.cx.k.stmt)
            .collect();
        let end = self.cx.span_of(node).end;
        self.stack.push(Frame::Block(BlockFrame {
            nodes: stmts,
            cursor: 0,
            stmts: Vec::new(),
            script_top: true,
            span: start_span.join(Span { start: end, end }),
        }));
        self.drive();
        match self.vals.pop() {
            Some(Val::F(f)) => f,
            _ => FnItem {
                name: Ident { name: "main".to_string(), span: start_span },
                params: Vec::new(),
                ret: TypeAnn { kind: TypeRef::Builtin(TypeWord::Void), span: start_span },
                body: Block { stmts: Vec::new(), span: start_span },
                span: start_span,
                receiver: Receiver::Free,
                owner: None,
                script: true,
            },
        }
    }

    // ------------------------------------------------------------ blocks

    fn enter_block(&mut self, node: u32) {
        let stmts: Vec<u32> = self
            .cx
            .children(node)
            .into_iter()
            .filter(|&c| self.cx.tree.kind(c) == self.cx.k.stmt)
            .collect();
        let span = self.cx.span_of(node);
        self.stack.push(Frame::Block(BlockFrame {
            nodes: stmts,
            cursor: 0,
            stmts: Vec::new(),
            script_top: false,
            span,
        }));
    }

    fn step_block(&mut self, mut frame: BlockFrame) {
        if frame.cursor > 0 {
            let s = self.pop_s();
            frame.stmts.push(s);
        }
        if frame.cursor >= frame.nodes.len() {
            let span = frame.span;
            self.vals.push(Val::B(Block { stmts: frame.stmts, span }));
            return;
        }
        let stmt_n = frame.nodes[frame.cursor];
        let script_top = frame.script_top;
        frame.cursor += 1;
        self.stack.push(Frame::Block(frame));
        self.enter_stmt(stmt_n, script_top);
    }

    fn enter_stmt(&mut self, stmt_dispatch: u32, script_top: bool) {
        let inner = self
            .cx
            .children(stmt_dispatch)
            .into_iter()
            .find(|&c| {
                self.cx.tree.node(c).flags & FLAG_TRIVIA == 0
                    && self.cx.tree.node(c).flags & ace_syntax_runtime::cst::FLAG_ERROR == 0
            })
            .unwrap_or(stmt_dispatch);
        let kind = self.cx.tree.kind(inner);
        if kind == self.cx.k.let_stmt || kind == self.cx.k.var_stmt {
            let kw_kind = if kind == self.cx.k.let_stmt { self.cx.k.tok_let } else { self.cx.k.tok_var };
            let kw = self.cx.child_of_kind(inner, kw_kind).map(|l| self.cx.span_of(l)).unwrap_or(Span { start: 0, end: 0 });
            let name_leaf = self.cx.field_child(inner, "name").expect("let name");
            let name = ident_of(&mut self.cx, name_leaf);
            let ann = self.cx.field_child(inner, "ann").map(|n| self.classify_word(n));
            let mutable = kind == self.cx.k.var_stmt;
            let value_n = self.cx.field_child(inner, "value").expect("let value");
            self.stack.push(Frame::LetFinish { node: inner, kw, name, ann, mutable });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if kind == self.cx.k.assign_stmt || kind == self.cx.k.path_assign_stmt.unwrap_or(u32::MAX) {
            let name_leaf = self.cx.field_child(inner, "name").expect("assign name");
            let name = ident_of(&mut self.cx, name_leaf);
            let assign_leaf = self.cx.child_of_kind(inner, self.cx.k.tok_assign);
            let mut path = Vec::new();
            if kind == self.cx.k.path_assign_stmt.unwrap_or(u32::MAX) {
                let name_start = self.cx.tree.abs_range(name_leaf).0;
                let assign_start = assign_leaf.map(|a| self.cx.tree.abs_range(a).0).unwrap_or(u64::MAX);
                let mut path_leaves: Vec<u32> = self
                    .cx
                    .children(inner)
                    .into_iter()
                    .filter(|&c| {
                        self.cx.tree.kind(c) == self.cx.k.tok_ident
                            && self.cx.tree.abs_range(c).0 > name_start
                            && self.cx.tree.abs_range(c).0 < assign_start
                    })
                    .collect();
                path_leaves.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
                for pl in path_leaves {
                    path.push(ident_of(&mut self.cx, pl));
                }
            }
            let value_n = self.cx.field_child(inner, "value").expect("assign value");
            self.stack.push(Frame::AssignFinish { name, path });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if kind == self.cx.k.return_stmt {
            let kw = self.cx.child_of_kind(inner, self.cx.k.tok_return).map(|l| self.cx.span_of(l)).unwrap_or(Span { start: 0, end: 0 });
            if script_top {
                self.cx.reject(kw, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.entry-top-return",
                        "a script's result is its trailing expression; `return` is not part of the script form (use a declared `fn main` for an explicit exit)",
                        kw,
                    )
                });
            }
            let value_n = self.cx.field_child(inner, "value").expect("return value");
            self.stack.push(Frame::ReturnFinish { kw });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if kind == self.cx.k.if_stmt {
            let kw = self.cx.child_of_kind(inner, self.cx.k.tok_if).map(|l| self.cx.span_of(l)).unwrap_or(Span { start: 0, end: 0 });
            let _ = kw;
            let cond_n = self.cx.field_child(inner, "cond").expect("if cond");
            let then_n = self.cx.field_child(inner, "then").expect("if then");
            let els_blk = self
                .cx
                .child_of_kind(inner, self.cx.k.else_block)
                .and_then(|eb| self.cx.child_of_kind(eb, self.cx.k.block));
            self.stack.push(Frame::IfCond { then_n, els_blk, node: inner });
            self.stack.push(Frame::ExprEnter(cond_n));
        } else if kind == self.cx.k.while_stmt.unwrap_or(u32::MAX) || kind == self.cx.k.for_cond_stmt.unwrap_or(u32::MAX) {
            let kw_kind = if kind == self.cx.k.while_stmt.unwrap_or(u32::MAX) { self.cx.k.tok_while } else { self.cx.k.tok_for };
            let _kw_leaf = self.cx.child_of_kind(inner, kw_kind);
            let cond_n = self.cx.field_child(inner, "cond").expect("loop cond");
            let body_n = self.cx.field_child(inner, "body").expect("loop body");
            self.stack.push(Frame::WhileCond { body_n, node: inner });
            self.stack.push(Frame::ExprEnter(cond_n));
        } else if kind == self.cx.k.loop_stmt {
            let body_n = self.cx.field_child(inner, "body").expect("loop body");
            self.stack.push(Frame::LoopBody { node: inner });
            self.enter_block(body_n);
        } else if kind == self.cx.k.expr_stmt {
            let value_n = self.cx.field_child(inner, "value").expect("expr stmt value");
            if self.cx.profile != SourceProfile::EntryUnit {
                let span = self.cx.span_of(value_n);
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        "expression statements are not part of this source profile; calls are only allowed in initializers, assignments, returns and conditions",
                        span,
                    )
                });
            }
            self.stack.push(Frame::ExprStmtFinish);
            self.stack.push(Frame::ExprEnter(value_n));
        } else {
            // break/continue 叶。
            let text = self.cx.spelling(inner);
            let span = self.cx.span_of(inner);
            let stmt = if text == "break" {
                Stmt::Break { span }
            } else if text == "continue" {
                Stmt::Continue { span }
            } else {
                let found = self.describe_node(inner);
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        format!("expected statement, found {}", found),
                        span,
                    )
                });
                Stmt::Break { span }
            };
            self.vals.push(Val::S(stmt));
        }
    }

    // ------------------------------------------------------------ exprs

    fn enter_expr(&mut self, node: u32) {
        let kind = self.cx.tree.kind(node);
        if kind == self.cx.k.expr {
            // 分派节点 → 内层产物。
            let inner = self.cx.children(node).into_iter().find(|&c| self.cx.tree.child_count(c) > 0);
            match inner {
                Some(c) => self.stack.push(Frame::ExprEnter(c)),
                None => {
                    let span = first_leaf_span(&self.cx, node);
                    let found = self.describe_node(node);
                    self.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            format!("expected expression, found {}", found),
                            span,
                        )
                    });
                    self.vals.push(Val::E(Expr { kind: ExprKind::IntLit(0), span }));
                }
            }
            return;
        }
        if kind == self.cx.k.unary_expr {
            if let Some(lit) = self.cx.child_of_kind(node, self.cx.k.int_literal) {
                self.stack.push(Frame::ExprEnter(lit));
                return;
            }
            let has_minus = self.cx.child_of_kind(node, self.cx.k.tok_minus).is_some();
            let has_bang = self.cx.child_of_kind(node, self.cx.k.tok_bang).is_some();
            if has_minus || has_bang {
                let op = if has_bang { UnaryOp::Not } else { UnaryOp::Neg };
                let operand = self
                    .cx
                    .child_of_kind(node, self.cx.k.unary_expr)
                    .or_else(|| self.cx.child_of_kind(node, self.cx.k.postfix_expr));
                match operand {
                    Some(o) => {
                        // 内→外包裹：UnaryPost 先压（后弹）。
                        self.stack.push(Frame::UnaryPost { op });
                        self.stack.push(Frame::ExprEnter(o));
                    }
                    None => {
                        let span = first_leaf_span(&self.cx, node);
                        self.cx.reject(span, || syntax_diag("expected expression", span));
                        self.vals.push(Val::E(Expr { kind: ExprKind::IntLit(0), span }));
                    }
                }
                return;
            }
            if let Some(post) = self.cx.child_of_kind(node, self.cx.k.postfix_expr) {
                self.stack.push(Frame::ExprEnter(post));
                return;
            }
            let span = first_leaf_span(&self.cx, node);
            self.cx.reject(span, || syntax_diag("expected expression", span));
            self.vals.push(Val::E(Expr { kind: ExprKind::IntLit(0), span }));
            return;
        }
        if kind == self.cx.k.postfix_expr {
            let (base, steps) = self.split_postfix(node);
            if steps.is_empty() {
                self.stack.push(Frame::ExprEnter(base));
            } else {
                self.stack.push(Frame::PostfixFoldAll { steps });
                self.stack.push(Frame::ExprEnter(base));
            }
            return;
        }
        if kind == self.cx.k.int_literal {
            let e = self.project_int_literal(node);
            self.vals.push(Val::E(e));
            return;
        }
        if kind == KIND_CALL {
            self.enter_call(node);
            return;
        }
        if self.is_op_token(kind) {
            let kids = self.cx.children(node);
            if kids.len() >= 2 {
                let op = self.op_of_token(kind);
                self.stack.push(Frame::OpPost { op });
                self.stack.push(Frame::ExprEnter(kids[kids.len() - 1]));
                self.stack.push(Frame::ExprEnter(kids[0]));
            } else {
                let span = first_leaf_span(&self.cx, node);
                self.cx.reject(span, || syntax_diag("binary operands", span));
                self.vals.push(Val::E(Expr { kind: ExprKind::IntLit(0), span }));
            }
            return;
        }
        self.enter_atom(node);
    }

    fn enter_atom(&mut self, node: u32) {
        for c in self.cx.children(node) {
            let ck = self.cx.tree.kind(c);
            if ck == self.cx.k.leading_dot.unwrap_or(u32::MAX) {
                let dot_leaf = self.cx.children(c).into_iter().find(|&l| self.cx.tree.kind(l) == self.cx.k.tok_dot);
                let dspan = dot_leaf.map(|l| self.cx.span_of(l)).unwrap_or_else(|| self.cx.span_of(c));
                let member = self.cx.children(c).into_iter().find(|&l| self.cx.tree.kind(l) == self.cx.k.tok_ident).expect("leading-dot member");
                let mspan = self.cx.span_of(member);
                let mname = self.cx.spelling(member);
                let recv = Expr {
                    span: dspan,
                    kind: ExprKind::Var(Ident { name: "self".to_string(), span: dspan }),
                };
                let span = dspan.join(mspan);
                // `.name(args)`：args 由 KIND_CALL 收尾（callee 含此后缀链）。
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Field {
                        base: Box::new(recv),
                        name: Ident { name: mname, span: mspan },
                    },
                }));
                return;
            }
            if ck == self.cx.k.ctor_expr.unwrap_or(u32::MAX) {
                let ty_leaf = self.cx.field_child(c, "ty").expect("ctor type");
                let ty = ident_of(&mut self.cx, ty_leaf);
                let field_nodes: Vec<u32> = self
                    .cx
                    .child_of_kind(c, self.cx.k.ctor_fields.unwrap_or(u32::MAX))
                    .map(|fl| collect_seq(&self.cx, fl, self.cx.k.ctor_field.unwrap_or(u32::MAX), "ctor-tail"))
                    .unwrap_or_default();
                let count = field_nodes.len();
                self.stack.push(Frame::CtorFinish { node: c, ty, count });
                for f in field_nodes.into_iter().rev() {
                    let label_leaf = self.cx.field_child(f, "label").expect("ctor label");
                    let label = ident_of(&mut self.cx, label_leaf);
                    let value_n = self.cx.field_child(f, "value").expect("ctor value");
                    self.stack.push(Frame::CtorFieldBuild { label });
                    self.stack.push(Frame::ExprEnter(value_n));
                }
                return;
            }
            if ck == self.cx.k.tok_str_lit {
                let span = self.cx.span_of(c);
                let raw = self.cx.spelling(c);
                let inner = &raw.as_bytes()[1..raw.len() - 1];
                let value = lex::decode_str_value(inner);
                self.vals.push(Val::E(Expr { kind: ExprKind::StrLit(value), span }));
                return;
            }
            if ck == self.cx.k.tok_char_lit {
                let span = self.cx.span_of(c);
                let raw = self.cx.spelling(c);
                let inner = &raw.as_bytes()[1..raw.len() - 1];
                let scalar = lex::decode_char_value(inner);
                self.vals.push(Val::E(Expr { kind: ExprKind::CharLit(scalar), span }));
                return;
            }
            if ck == self.cx.k.tok_true || ck == self.cx.k.tok_false {
                let span = self.cx.span_of(c);
                let v = ck == self.cx.k.tok_true;
                self.vals.push(Val::E(Expr { kind: ExprKind::BoolLit(v), span }));
                return;
            }
            if ck == self.cx.k.tok_ident {
                let span = self.cx.span_of(c);
                let name = self.cx.spelling(c);
                self.vals.push(Val::E(Expr { kind: ExprKind::Var(Ident { name, span }), span }));
                return;
            }
            if ck == self.cx.k.expr {
                // 括号组：内层表达式（span 保持内层自身——旧合同）。
                let inner = self.cx.children(c).into_iter().find(|&g| self.cx.tree.child_count(g) > 0);
                match inner {
                    Some(g) => self.stack.push(Frame::ExprEnter(g)),
                    None => {
                        let span = first_leaf_span(&self.cx, node);
                        let found = self.describe_node(node);
                        self.cx.reject(span, move || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.syntax",
                                format!("expected expression, found {}", found),
                                span,
                            )
                        });
                        self.vals.push(Val::E(Expr { kind: ExprKind::IntLit(0), span }));
                    }
                }
                return;
            }
        }
        let span = first_leaf_span(&self.cx, node);
        let found = self.describe_node(node);
        self.cx.reject(span, move || {
            Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected expression, found {}", found),
                span,
            )
        });
        self.vals.push(Val::E(Expr { kind: ExprKind::IntLit(0), span }));
    }

    fn project_int_literal(&mut self, node: u32) -> Expr {
        let int_leaf = self.cx.child_of_kind(node, self.cx.k.tok_int_lit).expect("int digits");
        let neg = self.cx.child_of_kind(node, self.cx.k.tok_minus).is_some();
        let digits = self.cx.spelling(int_leaf);
        let mag: u128 = digits.parse().unwrap_or(u128::MAX);
        let span = self.cx.span_of(node);
        if neg {
            if mag > I32_MIN_MAG {
                let digits2 = digits.clone();
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.literal-range",
                        format!(
                            "integer literal `-{}` is outside the signed 32-bit range of this source profile",
                            digits2
                        ),
                        span,
                    )
                });
            }
            Expr { kind: ExprKind::IntLit((mag as i128 * -1) as i32), span }
        } else {
            if mag > I32_MAX {
                let digits2 = digits.clone();
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.literal-range",
                        format!(
                            "integer literal `{}` is outside the signed 32-bit range of this source profile",
                            digits2
                        ),
                        span,
                    )
                });
            }
            Expr { kind: ExprKind::IntLit(mag as i32), span }
        }
    }

    fn split_postfix(&self, node: u32) -> (u32, Vec<u32>) {
        let mut base = None;
        let mut steps: Vec<u32> = Vec::new();
        for c in self.cx.children(node) {
            let k = self.cx.tree.kind(c);
            if k == self.cx.k.atom_expr && base.is_none() {
                base = Some(c);
            } else if k == self.cx.k.postfix_step.unwrap_or(u32::MAX) {
                steps.push(c);
            }
        }
        (base.expect("postfix base"), steps)
    }

    /// 折叠全部后缀步（迭代；不递归）。
    fn fold_postfix(&mut self, recv: Expr, steps: Vec<u32>) -> Expr {
        let mut value = recv;
        for step in steps {
            let member = self.step_member(step);
            value = self.fold_one_postfix(value, step, member);
        }
        value
    }

    fn step_member(&self, step: u32) -> Option<u32> {
        self.cx.children(step).into_iter().find(|&m| {
            let mk = self.cx.tree.kind(m);
            mk == self.cx.k.tok_ident || mk == self.cx.k.tok_mut || mk == self.cx.k.tok_r_move
        })
    }

    fn fold_one_postfix(&mut self, recv: Expr, step: u32, member: Option<u32>) -> Expr {
        // `.to(...)`：步自带括号与词。
        let has_to = self.cx.children(step).into_iter().any(|c| self.cx.tree.kind(c) == self.cx.k.tok_r_to);
        if has_to {
            let word_leaf = self.cx.children(step).into_iter().find(|&c| self.cx.tree.kind(c) == self.cx.k.tok_ident).expect(".to word");
            let word_span = self.cx.span_of(word_leaf);
            let word = self.cx.spelling(word_leaf);
            let tw = match word.as_str() {
                "str" => Some(TypeWord::Str),
                "Str" => Some(TypeWord::StrOwned),
                _ => None,
            };
            return match tw {
                Some(tw) => {
                    let span = recv.span.join(word_span);
                    Expr {
                        span,
                        kind: ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident { name: "to".to_string(), span: word_span },
                            args: vec![Arg::Pos(Expr { span: word_span, kind: ExprKind::TypeWordArg(tw) })],
                        },
                    }
                }
                None => {
                    self.cx.reject(word_span, || {
                        Diagnostic::new(Stage::Text, "source.syntax", "`.to` expects `str` or `Str`", word_span)
                    });
                    let span = recv.span.join(word_span);
                    Expr {
                        span,
                        kind: ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident { name: "to".to_string(), span: word_span },
                            args: Vec::new(),
                        },
                    }
                }
            };
        }
        let Some(member) = member else {
            return recv;
        };
        let mk = self.cx.tree.kind(member);
        if mk == self.cx.k.tok_mut {
            let mspan = self.cx.span_of(member);
            let name = match &recv.kind {
                ExprKind::Var(id) => id.clone(),
                _ => {
                    let span = recv.span.join(mspan);
                    self.cx.reject(span, || {
                        Diagnostic::new(Stage::Text, "source.unsupported", "`.mut` applies to a local variable", span)
                    });
                    Ident { name: String::new(), span: mspan }
                }
            };
            let span = name.span.join(mspan);
            return Expr { kind: ExprKind::MutArg { name }, span };
        }
        if mk == self.cx.k.tok_r_move {
            let mspan = self.cx.span_of(member);
            let span = recv.span.join(mspan);
            return Expr {
                span,
                kind: ExprKind::Method {
                    recv: Box::new(recv),
                    name: Ident { name: "move".to_string(), span: mspan },
                    args: Vec::new(),
                },
            };
        }
        // `.name`：record 族 = 字段读；text-only 面 = 无括号文本方法（旧拒绝）。
        let mspan = self.cx.span_of(member);
        let mname = self.cx.spelling(member);
        if self.cx.profile.is_record() {
            let span = recv.span.join(mspan);
            Expr {
                span,
                kind: ExprKind::Field {
                    base: Box::new(recv),
                    name: Ident { name: mname, span: mspan },
                },
            }
        } else {
            let dspan = self
                .cx
                .children(step)
                .into_iter()
                .find(|&l| self.cx.tree.kind(l) == self.cx.k.tok_dot)
                .map(|l| self.cx.span_of(l))
                .unwrap_or(mspan);
            let mname2 = mname.clone();
            self.cx.reject(dspan, move || {
                Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("method `.{}` requires parentheses", mname2),
                    dspan,
                )
            });
            let span = recv.span.join(mspan);
            Expr {
                span,
                kind: ExprKind::Field {
                    base: Box::new(recv),
                    name: Ident { name: mname, span: mspan },
                },
            }
        }
    }

    // ------------------------------------------------------------ calls

    fn enter_call(&mut self, node: u32) {
        let kids = self.cx.children(node);
        let callee_n = kids.iter().copied().find(|&c| self.cx.tree.kind(c) != self.cx.k.args).expect("call callee");
        let args_n = kids.iter().copied().find(|&c| self.cx.tree.kind(c) == self.cx.k.args);
        let close_end = self.cx.span_of(node).end;
        let arg_nodes: Vec<u32> = args_n
            .map(|an| collect_seq(&self.cx, an, self.cx.k.arg, "arg-tail"))
            .unwrap_or_default();
        let argc = arg_nodes.len();
        // callee 静态分析：expr 分派 → 一元链 → postfix/atom。
        let plan = {
            let inner = self.cx.children(callee_n).into_iter().find(|&c| self.cx.tree.child_count(c) > 0);
            let mut wrap = inner.unwrap_or(callee_n);
            let mut unary_ops: Vec<UnaryOp> = Vec::new();
            loop {
                if self.cx.tree.kind(wrap) == self.cx.k.unary_expr {
                    let has_minus = self.cx.child_of_kind(wrap, self.cx.k.tok_minus).is_some();
                    let has_bang = self.cx.child_of_kind(wrap, self.cx.k.tok_bang).is_some();
                    if !has_minus && !has_bang {
                        break;
                    }
                    unary_ops.push(if has_bang { UnaryOp::Not } else { UnaryOp::Neg });
                    let inner2 = self
                        .cx
                        .child_of_kind(wrap, self.cx.k.unary_expr)
                        .or_else(|| self.cx.child_of_kind(wrap, self.cx.k.postfix_expr));
                    match inner2 {
                        Some(i) => wrap = i,
                        None => break,
                    }
                } else {
                    break;
                }
            }
            if self.cx.tree.kind(wrap) == self.cx.k.postfix_expr {
                let (base, steps) = self.split_postfix(wrap);
                match steps.split_last() {
                    Some((last, mids)) => CalleePlan::Method {
                        base,
                        member: self.step_member(*last).unwrap_or(*last),
                        mids: mids.to_vec(),
                        unary_ops,
                    },
                    None => CalleePlan::Plain { ident: self.first_ident(base), unary_ops },
                }
            } else {
                CalleePlan::Plain { ident: self.first_ident(wrap), unary_ops }
            }
        };
        // 压帧序（后压先弹）：实参任务 … 接收者折叠 … 接收者 base … 汇总帧。
        match plan {
            CalleePlan::Method { base, member, mids, unary_ops } => {
                let fold_mids = mids.clone();
                self.stack.push(Frame::CallAssemble {
                    callee: CalleePlan::Method { base, member, mids, unary_ops },
                    argc,
                    close_end,
                });
                if !fold_mids.is_empty() {
                    self.stack.push(Frame::PostfixFoldAll { steps: fold_mids });
                }
                self.stack.push(Frame::ExprEnter(base));
            }
            plan @ CalleePlan::Plain { .. } => {
                self.stack.push(Frame::CallAssemble {
                    callee: plan,
                    argc,
                    close_end,
                });
            }
        }
        for a in arg_nodes.into_iter().rev() {
            if let Some(named) = self.cx.child_of_kind(a, self.cx.k.named_arg) {
                let label_leaf = self.cx.field_child(named, "label").expect("arg label");
                let label = ident_of(&mut self.cx, label_leaf);
                let value_n = self.cx.field_child(named, "value").expect("arg value");
                self.stack.push(Frame::ArgBuild { named: Some(label) });
                self.stack.push(Frame::ExprEnter(value_n));
            } else if let Some(value_n) = self.cx.child_of_kind(a, self.cx.k.expr) {
                self.stack.push(Frame::ArgBuild { named: None });
                self.stack.push(Frame::ExprEnter(value_n));
            }
        }
    }

    fn first_ident(&self, node: u32) -> Ident {
        let mut work: Vec<(u32, u8)> = vec![(node, 0)];
        let mut found: Option<u32> = None;
        while let Some((n, depth)) = work.pop() {
            if depth > 3 {
                continue;
            }
            for c in self.cx.children(n) {
                if self.cx.tree.kind(c) == self.cx.k.tok_ident {
                    found = Some(c);
                } else {
                    work.push((c, depth + 1));
                }
            }
            if found.is_some() {
                break;
            }
        }
        let leaf = found.expect("callee ident");
        Ident { name: self.cx.spelling(leaf), span: self.cx.span_of(leaf) }
    }

    // ------------------------------------------------------------ describe

    fn describe_node(&self, node: u32) -> String {
        let mut n = node;
        loop {
            if self.cx.tree.node(n).child_count == 0 {
                break;
            }
            let kids = self.cx.children(n);
            match kids.iter().find(|&&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0) {
                Some(&c) => n = c,
                None => break,
            }
        }
        let kind = self.cx.tree.kind(n);
        let text = self.cx.spelling(n);
        if kind == self.cx.k.tok_ident {
            format!("`{}`", text)
        } else if kind == self.cx.k.tok_int_lit {
            "integer literal".to_string()
        } else if kind == self.cx.k.tok_str_lit {
            "string literal".to_string()
        } else if kind == self.cx.k.tok_char_lit {
            "char literal".to_string()
        } else if kind == self.cx.k.tok_affix_num {
            "suffixed/radix/float numeric literal".to_string()
        } else if kind == self.cx.k.tok_block_open {
            "block comment".to_string()
        } else {
            format!("`{}`", text)
        }
    }

    fn is_op_token(&self, kind: u32) -> bool {
        [
            self.cx.k.op_plus, self.cx.k.op_minus, self.cx.k.op_star, self.cx.k.op_slash,
            self.cx.k.op_percent, self.cx.k.op_lt, self.cx.k.op_gt, self.cx.k.op_le,
            self.cx.k.op_ge, self.cx.k.op_eq, self.cx.k.op_ne, self.cx.k.op_andand,
            self.cx.k.op_oror,
        ]
        .contains(&kind)
    }

    fn op_of_token(&self, kind: u32) -> BinOp {
        if kind == self.cx.k.op_plus { BinOp::Add }
        else if kind == self.cx.k.op_minus { BinOp::Sub }
        else if kind == self.cx.k.op_star { BinOp::Mul }
        else if kind == self.cx.k.op_slash { BinOp::Div }
        else if kind == self.cx.k.op_percent { BinOp::Rem }
        else if kind == self.cx.k.op_lt { BinOp::Lt }
        else if kind == self.cx.k.op_gt { BinOp::Gt }
        else if kind == self.cx.k.op_le { BinOp::Le }
        else if kind == self.cx.k.op_ge { BinOp::Ge }
        else if kind == self.cx.k.op_eq { BinOp::Eq }
        else if kind == self.cx.k.op_ne { BinOp::Ne }
        else if kind == self.cx.k.op_andand { BinOp::And }
        else { BinOp::Or }
    }

    // ------------------------------------------------------------ step

    fn step(&mut self, frame: Frame) {
        match frame {
            Frame::Block(bf) => self.step_block(bf),
            Frame::LetFinish { node, kw, name, ann, mutable } => {
                let value = self.pop_e();
                let span = Span { start: self.cx.span_of(node).start, end: value.span.end };
                self.vals.push(Val::S(Stmt::Let { kw, name, ann, mutable, value, span }));
            }
            Frame::AssignFinish { name, path } => {
                let value = self.pop_e();
                let span = Span { start: name.span.start, end: value.span.end };
                self.vals.push(Val::S(Stmt::Assign { target: AssignTarget { base: name, path }, value, span }));
            }
            Frame::ReturnFinish { kw } => {
                let value = self.pop_e();
                let span = kw.join(value.span);
                self.vals.push(Val::S(Stmt::Return { value, span }));
            }
            Frame::IfCond { then_n, els_blk, node } => {
                let cond = self.pop_e();
                self.stack.push(Frame::IfThen { cond, els_blk, node });
                self.enter_block(then_n);
            }
            Frame::IfThen { cond, els_blk, node } => {
                let then = self.pop_b();
                match els_blk {
                    Some(eb) => {
                        self.stack.push(Frame::IfElse { cond, then, node });
                        self.enter_block(eb);
                    }
                    None => {
                        let kw = self.cx.child_of_kind(node, self.cx.k.tok_if).map(|l| self.cx.span_of(l)).unwrap_or(Span { start: 0, end: 0 });
                        let span = kw.join(then.span);
                        self.vals.push(Val::S(Stmt::If { cond, then, els: None, span }));
                    }
                }
            }
            Frame::IfElse { cond, then, node } => {
                let els = self.pop_b();
                let kw = self.cx.child_of_kind(node, self.cx.k.tok_if).map(|l| self.cx.span_of(l)).unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(then.span).join(els.span);
                self.vals.push(Val::S(Stmt::If { cond, then, els: Some(els), span }));
            }
            Frame::WhileCond { body_n, node } => {
                let cond = self.pop_e();
                self.stack.push(Frame::WhileBody { cond, node });
                self.enter_block(body_n);
            }
            Frame::WhileBody { cond, node } => {
                let body = self.pop_b();
                let kw_kind = if self.cx.tree.kind(node) == self.cx.k.while_stmt.unwrap_or(u32::MAX) { self.cx.k.tok_while } else { self.cx.k.tok_for };
                let kw = self.cx.child_of_kind(node, kw_kind).map(|l| self.cx.span_of(l)).unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                // 条件 for 复用 While AST 形状（旧合同）。
                self.vals.push(Val::S(Stmt::While { cond, body, span }));
            }
            Frame::LoopBody { node } => {
                let body = self.pop_b();
                let kw = self.cx.child_of_kind(node, self.cx.k.tok_loop).map(|l| self.cx.span_of(l)).unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                self.vals.push(Val::S(Stmt::Loop { body, span }));
            }
            Frame::ExprStmtFinish => {
                let value = self.pop_e();
                let span = value.span;
                self.vals.push(Val::S(Stmt::ExprStmt { value, span }));
            }
            Frame::FnBodyFinish { node, name, params, ret, receiver, owner } => {
                let body = self.pop_b();
                let span = self.cx.span_of(node);
                self.vals.push(Val::F(FnItem {
                    span, name, params, ret, body, receiver, owner, script: false,
                }));
            }
            Frame::ScriptFinish { start_span } => {
                let mut body = self.pop_b();
                let span = start_span.join(body.span);
                body.span = span;
                self.vals.push(Val::F(FnItem {
                    name: Ident { name: "main".to_string(), span: start_span },
                    params: Vec::new(),
                    ret: TypeAnn { kind: TypeRef::Builtin(TypeWord::Void), span: start_span },
                    body,
                    span,
                    receiver: Receiver::Free,
                    owner: None,
                    script: true,
                }));
            }
            Frame::ExprEnter(n) => self.enter_expr(n),
            Frame::OpPost { op } => {
                let rhs = self.pop_e();
                let lhs = self.pop_e();
                let span = lhs.span.join(rhs.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                }));
            }
            Frame::UnaryPost { op } => {
                let operand = self.pop_e();
                let span = operand.span;
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Unary { op, operand: Box::new(operand) },
                }));
            }
            Frame::PostfixFoldAll { steps } => {
                let recv = self.pop_e();
                let e = self.fold_postfix(recv, steps);
                self.vals.push(Val::E(e));
            }
            Frame::CallAssemble { callee, argc, close_end } => {
                let e = self.assemble_call(callee, argc, close_end);
                self.vals.push(Val::E(e));
            }
            Frame::ArgBuild { named } => {
                let value = self.pop_e();
                let arg = match named {
                    Some(name) => Arg::Named { span: name.span.join(value.span), name, value },
                    None => Arg::Pos(value),
                };
                self.vals.push(Val::A(arg));
            }
            Frame::CtorFieldBuild { label } => {
                let value = self.pop_e();
                let span = label.span.join(value.span);
                self.vals.push(Val::CF(NamedCtorField { name: label, value, span }));
            }
            Frame::CtorFinish { node, ty, count } => {
                let mut fields = Vec::with_capacity(count);
                for _ in 0..count {
                    match self.vals.pop() {
                        Some(Val::CF(f)) => fields.push(f),
                        _ => {}
                    }
                }
                fields.reverse();
                let span = self.cx.span_of(node);
                self.vals.push(Val::E(Expr { span, kind: ExprKind::CtorNamed { ty, fields } }));
            }
        }
    }

    /// 调用组装（CalleePlan 已含静态分析；接收者值在实参之下）。
    fn assemble_call(&mut self, callee: CalleePlan, argc: usize, close_end: usize) -> Expr {
        let mut args = Vec::with_capacity(argc);
        for _ in 0..argc {
            match self.vals.pop() {
                Some(Val::A(a)) => args.push(a),
                _ => {}
            }
        }
        args.reverse();
        let is_record = self.cx.profile.is_record();
        let is_text = self.cx.profile.is_text();
        let mut e = match callee {
            CalleePlan::Plain { ident, unary_ops } => {
                let mut e = Expr {
                    span: Span { start: ident.span.start, end: close_end },
                    kind: ExprKind::Call { callee: ident, args },
                };
                for op in unary_ops.iter().rev() {
                    let sp = e.span;
                    e = Expr {
                        span: sp,
                        kind: ExprKind::Unary { op: *op, operand: Box::new(e) },
                    };
                }
                e
            }
            CalleePlan::Method { member, mids: _, unary_ops, .. } => {
                let recv = self.pop_e();
                let mk = self.cx.tree.kind(member);
                let mspan = self.cx.span_of(member);
                let mname = self.cx.spelling(member);
                // type-level 接收者：char.from_codepoint（text 面）。
                let recv = if is_text && mname == "from_codepoint" {
                    let mut r = recv;
                    if let ExprKind::Var(id) = &r.kind {
                        if id.name == "char" {
                            r = Expr { span: id.span, kind: ExprKind::TypeWordArg(TypeWord::Char) };
                        }
                    }
                    r
                } else {
                    recv
                };
                let full = Span { start: recv.span.start, end: close_end };
                let mut e = if mk == self.cx.k.tok_mut {
                    self.cx.reject(mspan, || {
                        Diagnostic::new(Stage::Text, "source.unsupported", "`.mut` applies to a local variable", mspan)
                    });
                    Expr { span: full, kind: ExprKind::MutArg { name: Ident { name: mname, span: mspan } } }
                } else if is_record {
                    Expr {
                        span: full,
                        kind: ExprKind::MethodCall {
                            recv: Box::new(recv),
                            name: Ident { name: mname, span: mspan },
                            args,
                        },
                    }
                } else {
                    Expr {
                        span: full,
                        kind: ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident { name: mname, span: mspan },
                            args,
                        },
                    }
                };
                for op in unary_ops.iter().rev() {
                    let sp = e.span;
                    e = Expr {
                        span: sp,
                        kind: ExprKind::Unary { op: *op, operand: Box::new(e) },
                    };
                }
                e
            }
        };
        let _ = &mut e;
        e
    }
}


/// 序列收集：列表节点直接孩子 + tail 包裹（`X many { X-tail { , X } }`）内
/// 的项目节点，按文档序（参数/实参/构造字段共用）。
fn collect_seq(cx: &Cx, list_node: u32, item_kind: u32, tail_name: &str) -> Vec<u32> {
    let tail_kind = cx
        .p
        .rules
        .iter()
        .position(|r| r.name == tail_name)
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    let mut out: Vec<u32> = Vec::new();
    for c in cx.children(list_node) {
        let ck = cx.tree.kind(c);
        if ck == item_kind {
            out.push(c);
        } else if Some(ck) == tail_kind {
            for g in cx.children(c) {
                if cx.tree.kind(g) == item_kind {
                    out.push(g);
                }
            }
        }
    }
    out.sort_by_key(|&n| cx.tree.abs_range(n).0);
    out
}

fn syntax_diag(msg: &str, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Text, "source.syntax", msg.to_string(), span)
}

fn unsupported_ty(word: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.unsupported",
        format!("type `{}` is not part of this source profile", word),
        span,
    )
}

fn self_method_name(span: Span) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.unsupported",
        "`self` is the implicit receiver of an instance method and cannot be a method name",
        span,
    )
}

fn self_param(span: Span) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.unsupported",
        "`self` is the implicit receiver of an instance method and cannot be declared as a parameter",
        span,
    )
}
