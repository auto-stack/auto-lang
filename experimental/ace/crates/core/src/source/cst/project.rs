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
use ace_syntax_runtime::program::{GrammarProgram, KIND_CALL, KIND_POSTFIX};
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
    project_program_with_caps(p, k, tree, snap, profile, false)
}

/// PLAN-935（T-02）：能力感知投影。`instances = true`（统一前门以
/// `scalar-instances` 裁剪解析时）类型位裸 ident 按声明类型变量投影为
/// `TypeRef::TyVar`；fn-tail 的 `<T>` 投影为 `FnItem::type_params`。plain
/// 面与旧路径逐字节同形。
pub(crate) fn project_program_with_caps(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
    instances: bool,
) -> Result<Program, Vec<Diagnostic>> {
    // 词法层先行（旧 tokenize 全文件先行的首事实序）。
    if let Some(d) = lex::first_lex_fact(tree, snap, k, profile) {
        return Err(vec![d]);
    }
    let mut eng = Engine::new(p, k, tree, snap, profile, instances);
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
        // PLAN-950: public visibility is retained in ModuleSyntax, while
        // the existing function projector consumes the nested fn item.
        let node = if p
            .rules
            .iter()
            .position(|r| r.name == "public-fn")
            .is_some_and(|i| {
                tree.kind(node) == ace_syntax_runtime::program::RULE_KIND_BASE + i as u32
            }) {
            p.field_id("function")
                .and_then(|f| tree.field_child(node, f))
                .unwrap_or(node)
        } else {
            node
        };
        let kind = tree.kind(node);
        // 错误子树跳过乐观投影（结构层映射负责其诊断；干净子树照常投影
        // 以保真能力拒绝的位置序）。PLAN-928（T-04）：脚本区首 token 的
        // 位置序拒绝先于跳过（R04 类：恢复区内部能力拒绝不得抢位）。
        if kind == k.script_item.unwrap_or(u32::MAX) {
            let mut n = node;
            loop {
                if eng.cx.tree.node(n).child_count == 0 {
                    break;
                }
                match eng
                    .cx
                    .children(n)
                    .into_iter()
                    .find(|&c| eng.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
                {
                    Some(c) => n = c,
                    None => break,
                }
            }
            let head = eng.cx.spelling(n);
            if matches!(head.as_str(), "type" | "enum" | "static") {
                let span = eng.cx.span_of(n);
                let h = head.clone();
                eng.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        format!("expected statement, found `{h}`"),
                        span,
                    )
                });
            }
        }
        if tree_has_errors && subtree_has_errors(tree, node) {
            continue;
        }
        if kind == k.fn_item
            || kind == k.type_item.unwrap_or(u32::MAX)
            || kind == k.enum_item.unwrap_or(u32::MAX)
        {
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
            } else if kind == k.enum_item.unwrap_or(u32::MAX) {
                items.push(Item::Enum(eng.run_enum_item(node)));
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
/// 旧 expect_ident 的 host-console 保留名拒绝（print/eprint）。
fn console_name_check(cx: &mut Cx, name: &Ident) {
    if cx.profile == SourceProfile::HostConsole && matches!(name.name.as_str(), "print" | "eprint")
    {
        let span = name.span;
        cx.reject(span, move || {
            Diagnostic::new(
                Stage::Text,
                "source.console-reserved",
                "console names cannot be bound in the host-console profile",
                span,
            )
        });
    }
}

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
            Span {
                start: s as usize,
                end: e as usize,
            }
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
            k.tok_let,
            k.tok_var,
            k.tok_if,
            k.tok_else,
            k.tok_while,
            k.tok_loop,
            k.tok_break,
            k.tok_continue,
            k.tok_return,
            k.tok_true,
            k.tok_false,
            k.tok_for,
            k.tok_int_kw,
            k.tok_i32,
            k.tok_bool,
            k.tok_type_kw,
            k.tok_static,
            k.tok_mut,
            k.tok_enum_kw,
            k.tok_is,
            k.tok_in,
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
    let at_root = (path.iter().any(|&g| g == cx.k.item)
        && !path
            .iter()
            .any(|&g| g == cx.k.script_item.unwrap_or(u32::MAX)))
        || path.len() <= 1; // 根直挂尾随 ERROR 叶（路径仅含 program kind）
    let mut in_stmt = path
        .iter()
        .any(|&g| g == cx.k.stmt || g == cx.k.block || g == cx.k.script_item.unwrap_or(u32::MAX));
    let in_expr = path.iter().any(|&g| g == cx.k.expr || g == cx.k.unary_expr);
    if cx.profile.is_entry_unit() && !in_expr {
        // entry 面：items() 把一切非 `fn` 顶层 token 路由进脚本语句序列
        // （旧 script_item → parse_stmt_head）——顶层错误即语句位。
        in_stmt = true;
    }
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
    // 关键字：item 位 = "`X` is not part..."；stmt/script 位 = "expected
    // statement, found X"（旧 parse_stmt_head 只有 Reserved 走 unsupported）。
    if is_kw || text == "fn" {
        if in_stmt {
            let found = describe_leaf_text(cx, tree, leaf);
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected statement, found {}", found),
                span,
            ));
        }
        if !in_expr {
            return Some(word_msg(&text));
        }
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
    // 其余保留 token（use/enum/view/...）：item 位 = "`X` is not part..."
    //（旧 items() 对 Reserved 的拒绝面）；stmt/expr 位由上方 keyword 分支
    // 与表达式游走按各自消息处理。
    if kind.is_some_and(|g| cx.k.reserved_toks.contains(&g)) && !in_stmt && !in_expr {
        return Some(word_msg(&text));
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
            return Some(Diagnostic::new(
                Stage::Text,
                "source.unsupported",
                quoted,
                span,
            ));
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
fn leaf_end(_cx: &Cx, tree: &CstTree, node: u32) -> usize {
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

/// 全树非 trivia 叶（根的孩子经树级 root_edges 目录起种）。
fn tree_leaves(cx: &Cx, tree: &CstTree) -> Vec<(u32, u32, String)> {
    let mut out = Vec::new();
    let mut stack = tree.children(tree.root);
    let mut all: Vec<u32> = Vec::new();
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        if nd.flags & FLAG_TRIVIA != 0 {
            continue;
        }
        if nd.child_count == 0 {
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
    let region = region_leaves(cx, tree, node);
    if region.is_empty() {
        return None;
    }
    // 旧 parser 无"区域"概念——线性走到文件尾。恢复截断（sync 停在 `}`）
    // 会把区域外的 token 丢出叶列，导致多级嵌套块在区域边界误判 EOF。
    // 因此游走叶列 = 从区域首叶起的全树后缀（文档序），跨区域边界继续。
    // PLAN-928（T-04）：区域含于 fn 声明内时自**该声明头**重根（旧解析
    // 从声明头线性走——签名/前体能力拒绝先于体内错误按位置胜出）。
    let mut first_start = tree.abs_range(region[0].0).0;
    {
        // path_to 收集的是 kind——这里需要**节点**路径找最近 fn-item 祖先。
        let mut node_path: Vec<u32> = Vec::new();
        if node_path_to(tree, tree.root, node, &mut node_path) {
            let mut anc: Option<u32> = None;
            for &a in node_path.iter().rev() {
                if tree.kind(a) == cx.k.fn_item {
                    anc = Some(a);
                    break;
                }
            }
            if let Some(a) = anc {
                let ar = tree.abs_range(a);
                if let Some(&(l, _, _)) = tree_leaves(cx, tree).iter().find(|&&(l, _, _)| {
                    let r = tree.abs_range(l);
                    r.0 >= ar.0 && r.1 <= ar.1
                }) {
                    first_start = first_start.min(tree.abs_range(l).0);
                }
            }
        }
    }
    let mut leaves = tree_leaves(cx, tree);
    assert!(
        !leaves.is_empty(),
        "tree leaves empty but region had leaves"
    );
    leaves.retain(|(n, _, _)| tree.abs_range(*n).0 >= first_start);
    // 区域截断（恢复停在 sync）时，其后的首 token 供 fn-tail 期望游走使用
    //（旧 parser 在区域后第一个 token 上报"expected return type"等）。
    let region_end = tree.abs_range(node).1;
    let beyond: Option<(u32, u64, u64, String)> = lex::collect_leaves(tree)
        .into_iter()
        .filter(|l| {
            l.start >= region_end && !cx.spelling_by_range(l.start, l.end).trim().is_empty()
        })
        .min_by_key(|l| l.start)
        .map(|l| (l.kind, l.start, l.end, cx.spelling_by_range(l.start, l.end)));
    let first = &leaves[0];
    // item 形态：`fn ...` / `type ...`。
    if first.2 == "fn" {
        return walk_fn_item(
            cx,
            &leaves,
            beyond.as_ref().map(|b| {
                (
                    b.0,
                    b.3.as_str(),
                    Span {
                        start: b.1 as usize,
                        end: b.2 as usize,
                    },
                )
            }),
        );
    }
    if first.2 == "type" && cx.profile.is_record() && !cx.profile.is_closures() {
        return walk_type_item(cx, &leaves);
    }
    // stmt 形态（块内错误区域）：完整语句期望游走（与旧 parse_stmt_head
    // 同序——return/if/for/赋值/表达式语句各自镜像）。仅语句位区域——
    // item 位（块/脚本外）由叶层分类器按旧 items() 消息处理。
    let mut path_kinds: Vec<u32> = Vec::new();
    let _ = path_to(tree, tree.root, node, &mut path_kinds);
    let in_stmt_ctx = path_kinds
        .iter()
        .any(|&g| g == cx.k.stmt || g == cx.k.block || g == cx.k.script_item.unwrap_or(u32::MAX))
        || cx.profile.is_entry_unit();
    if in_stmt_ctx && leaves.len() >= 1 {
        let mut w = Walker::new(cx, &leaves);
        if let Some(d) = w.walk_stmt_head(false) {
            return Some(d);
        }
    }
    None
}

fn leaf_span(cx: &Cx, l: u32) -> Span {
    cx.span_of(l)
}

/// PLAN-928：叶级 fn 型跳跃（closures 面 ret/ann 位；`fn ( 词, ... ) [->] 词|嵌套fn型`）。
/// 入参 `i` 指向 `fn` 词；返回越过整个 fn 型后的下标。
fn skip_fn_type_leaves(leaves: &[(u32, u32, String)], mut i: usize) -> usize {
    i += 1; // `fn`
    if i < leaves.len() && leaves[i].2 == "(" {
        i += 1;
        let mut depth = 1usize;
        while i < leaves.len() && depth > 0 {
            if leaves[i].2 == "(" {
                depth += 1;
            } else if leaves[i].2 == ")" {
                depth -= 1;
            }
            i += 1;
        }
    }
    if i < leaves.len() && leaves[i].2 == "->" {
        i += 1;
    }
    if i < leaves.len() && leaves[i].2 == "fn" {
        return skip_fn_type_leaves(leaves, i);
    }
    i + 1
}

/// 旧 fn_item_tail 期望序列（fn 关键字已匹配）。`beyond` = 区域后首 token
/// (kind, spelling)——恢复截断在 `)` 时，结果位错误在该 token 上。
fn walk_fn_item(
    cx: &Cx,
    leaves: &[(u32, u32, String)],
    beyond: Option<(u32, &str, Span)>,
) -> Option<Diagnostic> {
    walk_fn_item_ex(cx, leaves, beyond).1
}

/// 同 [`walk_fn_item`]，但成功走完时返回结束游标（`}` 后一位；相对
/// `leaves` 下标）——type-item 成员序列游走续位用。
fn walk_fn_item_ex(
    cx: &Cx,
    leaves: &[(u32, u32, String)],
    beyond: Option<(u32, &str, Span)>,
) -> (usize, Option<Diagnostic>) {
    let k = cx.k;
    let mut i = 1usize; // 0 = `fn`
    let at = |idx: usize| -> (Span, u32, &str) {
        let (n, g, s) = &leaves[idx.min(leaves.len() - 1)];
        (leaf_span(cx, *n), *g, s.as_str())
    };
    // 1) 函数名。
    if i >= leaves.len() {
        return (i, Some(eof_expected(cx, "function name")));
    }
    {
        let (sp, g, text) = at(i);
        if g != k.tok_ident {
            if g == k.tok_affix_num
                || g == k.tok_str_lit
                || g == k.tok_char_lit
                || g == k.tok_block_open
            {
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!(
                            "{} cannot be used as a name in this source profile",
                            raw_describe(g, text, &k)
                        ),
                        sp,
                    )),
                );
            }
            if g == k.tok_r_move || g == k.tok_r_to || is_kw_leaf(g, &k) || text == "fn" {
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!("keyword `{}` cannot be used as a name", text),
                        sp,
                    )),
                );
            }
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected function name, found {}", describe_g(g, text, &k)),
                    sp,
                )),
            );
        }
        i += 1;
    }
    // 2) `(`。
    if i >= leaves.len() {
        return (i, Some(eof_expected(cx, "`(`")));
    }
    {
        let (sp, g, text) = at(i);
        if text != "(" {
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected `(`, found {}", describe_g(g, text, &k)),
                    sp,
                )),
            );
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
                // PLAN-928：closures 面的 move 参数（`fn make(move s Str)`）。
                let admits_move = cx.profile.is_text() || cx.profile.is_closures();
                let admits_mut = cx.profile.is_text() || cx.profile.is_record();
                if (is_move && admits_move) || (is_mut && admits_mut) {
                    i += 1;
                }
            }
            // 参数名。
            if i >= leaves.len() {
                return (i, Some(eof_expected(cx, "parameter name")));
            }
            {
                let (sp, g, text) = at(i);
                if g != k.tok_ident {
                    if g == k.tok_r_move || g == k.tok_r_to || is_kw_leaf(g, &k) || text == "fn" {
                        return (
                            i,
                            Some(Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                format!("keyword `{}` cannot be used as a name", text),
                                sp,
                            )),
                        );
                    }
                    if g == k.tok_affix_num
                        || g == k.tok_str_lit
                        || g == k.tok_char_lit
                        || g == k.tok_block_open
                    {
                        return (
                            i,
                            Some(Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                format!(
                                    "{} cannot be used as a name in this source profile",
                                    raw_describe(g, text, &k)
                                ),
                                sp,
                            )),
                        );
                    }
                    return (
                        i,
                        Some(Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            format!("expected parameter name, found {}", describe_g(g, text, &k)),
                            sp,
                        )),
                    );
                }
                i += 1;
            }
            // PLAN-928（T-03）：collections/collections 参数注解的可选糖 `?T`。
            if cx.profile.is_collections() {
                if i < leaves.len() && leaves[i].2 == "?" {
                    i += 1;
                }
            }
            // 参数类型词。
            if let Some(d) = walk_type_word(
                cx,
                leaves,
                &mut i,
                "parameter type annotation (`int`/`i32`/`bool`) is required",
            ) {
                return (i, Some(d));
            }
            // PLAN-928（T-03）：collections/closures 参数注解的参数化类型
            // `List<T>` / `Result<T, E>`（平衡 `<...>` 跳跃；`>` 后无逗号
            // 嵌套——type-arg 内嵌套泛型由同一深度计数覆盖）。
            if cx.profile.is_collections() && i < leaves.len() && leaves[i].2 == "<" {
                let mut depth = 0usize;
                while i < leaves.len() {
                    if leaves[i].2 == "<" {
                        depth += 1;
                    } else if leaves[i].2 == ">" {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    i += 1;
                }
            }
            // `,` 或 `)`。
            if i >= leaves.len() {
                return (i, Some(eof_expected(cx, "`,` or `)`")));
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
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected `,` or `)`, found {}", describe_g(g, text, &k)),
                    sp,
                )),
            );
        }
    }
    // 4) `)`。
    if i >= leaves.len() {
        return (i, Some(eof_expected(cx, "`)`")));
    }
    {
        let (sp, _g, text) = at(i);
        if text != ")" {
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected `)`, found {}", text),
                    sp,
                )),
            );
        }
        i += 1;
    }
    let name_span = leaf_span(cx, leaves[1].0);
    let rparen_span = leaf_span(cx, leaves[i - 1].0);
    // 5) 结果类型。
    if i >= leaves.len() {
        let Some((bg, btext, bspan)) = beyond else {
            return (i, Some(eof_expected(cx, "return type")));
        };
        let sp = bspan;
        let builtin = btext == "int" || btext == "i32" || btext == "bool";
        let text_word =
            bg == cx.k.tok_ident && cx.profile.is_text() && matches!(btext, "str" | "Str" | "char");
        // PLAN-928（T-04）：collections 词域——`Str` 收（char 仅 closures），
        // 小写 `str`/`char` 具名拒绝（镜像旧 collections_type）。
        let coll_word = bg == cx.k.tok_ident
            && cx.profile.is_collections()
            && (btext == "Str" || (btext == "char" && cx.profile.is_closures()));
        let coll_reject = bg == cx.k.tok_ident
            && cx.profile.is_collections()
            && (btext == "str" || (btext == "char" && !cx.profile.is_closures()));
        let entry_void = bg == cx.k.tok_ident && cx.profile.is_entry_unit() && btext == "void";
        let record_name = bg == cx.k.tok_ident && cx.profile.is_record();
        // collections 词域拒绝先于名义接受门（str/char 命名拒绝不得被
        // record_name 的 ident 接受吞掉）。
        if coll_reject {
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("type `{}` is not part of this source profile", btext),
                    sp,
                )),
            );
        }
        if !builtin && !text_word && !entry_void && !record_name && !coll_word {
            if btext == "{" {
                return (i, Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "function return type annotation (`int`/`i32`/`bool` or a record type) is required",
                    name_span.join(rparen_span),
                )));
            }
            if bg == cx.k.tok_ident || bg == cx.k.tok_r_move || bg == cx.k.tok_r_to {
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!("type `{}` is not part of this source profile", btext),
                        sp,
                    )),
                );
            }
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected return type, found `{}`", btext),
                    sp,
                )),
            );
        }
        return (i, Some(eof_expected(cx, "return type")));
    }
    {
        let (sp, g, text) = at(i);
        // PLAN-928：closures 面 ret 位 fn 型（`fn(T0, ...) [->] R`，可嵌套）。
        if cx.profile.is_closures() && text == "fn" {
            i = skip_fn_type_leaves(leaves, i);
        } else {
            // 旧 fn_item_tail 结果位分类：int/i32/bool 全域；text 面 ident 仅
            // str/Str/char；entry 面 ident 仅 void；record 面任意 ident（名义）。
            let builtin = text == "int" || text == "i32" || text == "bool";
            let text_word =
                g == k.tok_ident && cx.profile.is_text() && matches!(text, "str" | "Str" | "char");
            // PLAN-928（T-04）：collections 词域（同上）。
            let coll_word = g == k.tok_ident
                && cx.profile.is_collections()
                && (text == "Str" || (text == "char" && cx.profile.is_closures()));
            let coll_reject = g == k.tok_ident
                && cx.profile.is_collections()
                && (text == "str" || (text == "char" && !cx.profile.is_closures()));
            let entry_void = g == k.tok_ident && cx.profile.is_entry_unit() && text == "void";
            let record_name = g == k.tok_ident && cx.profile.is_record();
            if coll_reject {
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!("type `{}` is not part of this source profile", text),
                        sp,
                    )),
                );
            }
            if !builtin && !text_word && !entry_void && !record_name && !coll_word {
                if text == "{" {
                    return (i, Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "function return type annotation (`int`/`i32`/`bool` or a record type) is required",
                    name_span.join(rparen_span),
                )));
                }
                if g == k.tok_ident || g == k.tok_r_move || g == k.tok_r_to {
                    return (
                        i,
                        Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("type `{}` is not part of this source profile", text),
                            sp,
                        )),
                    );
                }
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        format!("expected return type, found {}", describe_g(g, text, &k)),
                        sp,
                    )),
                );
            }
            i += 1;
        }
    }
    // 6) `{`（体）→ 语句序列游走。
    if i >= leaves.len() {
        return (i, Some(eof_expected(cx, "`{`")));
    }
    {
        let (sp, g, text) = at(i);
        if text != "{" {
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected `{{`, found {}", describe_g(g, text, &k)),
                    sp,
                )),
            );
        }
        i += 1;
    }
    let mut w = Walker::new(cx, leaves);
    w.i = i;
    match w.walk_block_body(false) {
        Some(d) => (w.i, Some(d)),
        None => (w.i, None),
    }
}

/// 旧 EOF 期望：`expected X, found end of file` @ snap.len()（旧 eof_span）。
fn eof_expected(cx: &Cx, what: &str) -> Diagnostic {
    let e = cx.snap.len() as usize;
    Diagnostic::new(
        Stage::Text,
        "source.syntax",
        format!("expected {what}, found end of file"),
        Span { start: e, end: e },
    )
}

/// 旧 type_item 期望序列（type 关键字已匹配；record 族）——镜像旧
/// type_item() 成员循环：分隔纪律、field（self/可选 `:`/member_type
/// 分类/duplicate）、static fn / fn 方法（fn_item_tail 镜像）、`mut fn`
/// 与其他关键字拒绝、空 record 拒绝。
fn walk_type_item(cx: &Cx, leaves: &[(u32, u32, String)]) -> Option<Diagnostic> {
    let k = cx.k;
    let w = Walker::new(cx, leaves);
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
    }
    if leaves.len() <= 2 {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.syntax",
            "expected `{`, found end of file".to_string(),
            at(1).0,
        ));
    }
    {
        let (sp, _g, text) = at(2);
        if text != "{" {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected `{{`, found {}", text),
                sp,
            ));
        }
    }
    let type_kw_span = at(0).0;
    let mut i = 3usize;
    let mut seen_fields: Vec<String> = Vec::new();
    let mut any_member = false;
    let mut any_field = false;
    let mut close_span = None;
    while i < leaves.len() {
        let (_, _g, text) = at(i);
        if text == ";" {
            i += 1;
            continue;
        }
        if text == "}" {
            close_span = Some(at(i).0);
            break;
        }
        // 分隔纪律（旧 733-764）：首成员可同行；此后须 `;` 后或换行后。
        if any_member {
            let prev_semi = at(i - 1).2 == ";";
            let cur_line = w.at(i).map(|c| c.3).unwrap_or(0);
            let prev_line = w.at(i - 1).map(|c| c.3).unwrap_or(0);
            if !prev_semi && cur_line <= prev_line {
                let (sp, gg, tt) = at(i);
                return Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!(
                        "expected `;`, newline or `}}`, found {}",
                        describe_g(gg, tt, &k)
                    ),
                    sp,
                ));
            }
        }
        any_member = true;
        match text {
            "static" => {
                if i + 1 >= leaves.len() || at(i + 1).2 != "fn" {
                    let sp = at((i + 1).min(leaves.len() - 1)).0;
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        "expected `fn` after `static`".to_string(),
                        sp,
                    ));
                }
                let (end, d) = walk_fn_item_ex(cx, &leaves[i + 1..], None);
                if let Some(d) = d {
                    return Some(d);
                }
                i += 1 + end;
            }
            "fn" => {
                let (end, d) = walk_fn_item_ex(cx, &leaves[i..], None);
                if let Some(d) = d {
                    return Some(d);
                }
                i += end;
            }
            "mut" => {
                return Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "`mut fn` (mutable instance methods) is not part of this profile; methods are readonly views".to_string(),
                    at(i).0,
                ));
            }
            _ => {
                let (sp, gg, tt) = at(i);
                if gg == k.tok_ident {
                    if tt == "self" {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`self` cannot be a field name".to_string(),
                            sp,
                        ));
                    }
                    let mut j = i + 1;
                    if j < leaves.len() && at(j).2 == ":" {
                        j += 1;
                    }
                    if j >= leaves.len() {
                        return None;
                    }
                    let (tsp, tg, ttext) = at(j);
                    let is_text = cx.profile.is_text();
                    if tg == k.tok_ident && !is_text && matches!(ttext, "str" | "Str" | "char") {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("type `{}` is not part of this source profile", ttext),
                            tsp,
                        ));
                    }
                    let builtin = tg == k.tok_int_kw || tg == k.tok_i32 || tg == k.tok_bool;
                    if !(builtin || (tg == k.tok_ident && cx.profile.is_record())) {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "field type annotation (`int`/`i32`/`bool` or a record type) is required".to_string(),
                            sp,
                        ));
                    }
                    if seen_fields.contains(&tt.to_string()) {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.duplicate",
                            format!("field `{}` is declared more than once", tt),
                            sp,
                        ));
                    }
                    seen_fields.push(tt.to_string());
                    any_field = true;
                    i = j + 1;
                } else if is_kw_leaf(gg, &k) {
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!("`{}` is not a valid type member here", tt),
                        sp,
                    ));
                } else {
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        format!("expected field or method, found {}", describe_g(gg, tt, &k)),
                        sp,
                    ));
                }
            }
        }
    }
    if !any_field {
        // 空 record：kw..`}`（区域截断时以区域末叶代 `}`）。
        let end = close_span
            .or_else(|| leaves.last().map(|(n, _, _)| leaf_span(cx, *n)))
            .unwrap_or(type_kw_span);
        let span = type_kw_span.join(end);
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            "empty records are not part of this profile; a record must declare at least one field"
                .to_string(),
            span,
        ));
    }
    None
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
        // 行号：换行位置一次扫描 + 二分（后缀叶列可达全树——逐叶前缀计数
        // 是 O(n²)，567 语料差分会到分钟级）。
        let total = cx.snap.len();
        let text = cx.snap.read_str(0, total).unwrap_or_default();
        let bytes = text.as_bytes();
        let nl: Vec<u64> = bytes
            .iter()
            .enumerate()
            .filter(|(_, &b)| b == b'\n')
            .map(|(i, _)| i as u64)
            .collect();
        let mut lines = Vec::with_capacity(leaves.len());
        for (n, _, _) in leaves {
            let (s, e) = cx.tree.abs_range(*n);
            // 旧 lexer：多行 token 记结束行。
            let _ = s;
            lines.push(1 + nl.partition_point(|&x| x < e.min(total)) as u32);
        }
        Walker {
            cx,
            leaves,
            i: 0,
            lines,
        }
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
                    // 区域截断（恢复 sync 停在 `}`）：区域后首 token 代 cur——
                    // 旧语义是分隔纪律/块闭合检查，不是文件 EOF。
                    let region_end = self
                        .leaves
                        .last()
                        .map(|(n, _, _)| self.cx.tree.abs_range(*n).1)
                        .unwrap_or(0);
                    if let Some((g, text, span)) = self.cx.leaf_after(region_end) {
                        let k = self.k();
                        if text == "}" {
                            return None;
                        }
                        let cur_line = self.cx.line_of_pos(span.start as u64);
                        let prev_line = self
                            .leaves
                            .last()
                            .map(|(n, _, _)| self.cx.line_of_pos(self.cx.tree.abs_range(*n).0))
                            .unwrap_or(0);
                        if text != ";" && cur_line <= prev_line {
                            return Some(Diagnostic::new(
                                Stage::Text,
                                "source.syntax",
                                format!(
                                    "expected `;`, newline or `}}`, found {}",
                                    describe_g(g, &text, k)
                                ),
                                span,
                            ));
                        }
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            format!(
                                "expected statement or `}}`, found {}",
                                describe_g(g, &text, k)
                            ),
                            span,
                        ));
                    }
                    return Some(
                        self.syntax("expected statement or `}`, found end of file".into()),
                    );
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
                    None => {
                        return Some(self.syntax("expected local name, found end of file".into()))
                    }
                };
                if ng != k.tok_ident {
                    return Some(self.name_error(ng, ntext, nsp, "local name"));
                }
                self.i += 1;
                // 注解
                if let Some((asp, ag, atext, _)) = self.cur() {
                    let is_type = atext == "int" || atext == "i32" || atext == "bool";
                    // PLAN-928（T-04）：collections 词域——Str/char(closures) 收，
                    // 小写 str/char 拒；text 面三词全收。
                    let coll_ok = self.cx.profile.is_collections()
                        && (atext == "Str" || (atext == "char" && self.cx.profile.is_closures()));
                    let admits_text = self.cx.profile.is_text();
                    if ag == k.tok_ident
                        && !admits_text
                        && !coll_ok
                        && matches!(atext, "str" | "Str" | "char")
                    {
                        return Some(self.unsupported(
                            format!("type `{}` is not part of this source profile", atext),
                            asp,
                        ));
                    }
                    let ident_type = ag == k.tok_ident
                        && ((admits_text && matches!(atext, "str" | "Str" | "char"))
                            || coll_ok
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
                        return Some(
                            self.unsupported(
                                "locals must be initialized at declaration in this source profile"
                                    .into(),
                                Span {
                                    start: kw.start,
                                    end: last_end,
                                },
                            ),
                        );
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
                // PLAN-928（T-03）：collections/closures 面 `for x in <list>`。
                if self.cx.profile.is_collections() {
                    self.i += 1;
                    match self.cur() {
                        Some((_, ng, _, _)) if ng == k.tok_ident => {
                            self.i += 1;
                        }
                        _ => {
                            let (spn, gn, tn, _) = match self.cur() {
                                Some(c) => c,
                                None => {
                                    return Some(self.syntax(
                                        "expected iteration binding, found end of file".into(),
                                    ))
                                }
                            };
                            return Some(self.name_error(gn, tn, spn, "iteration binding"));
                        }
                    }
                    match self.cur() {
                        Some((_, _, "in", _)) => {
                            self.i += 1;
                        }
                        _ => {
                            return Some(self.syntax(format!(
                                "expected `in` after the `for` binding, found {}",
                                self.describe_cur()
                            )));
                        }
                    }
                    // 合同 C7：迭代器是裸 List 局部名（非表达式）。
                    match self.cur() {
                        Some((_, ig, _, _)) if ig == k.tok_ident => {
                            self.i += 1;
                        }
                        _ => {
                            let (spn, gn, tn, _) = match self.cur() {
                                Some(c) => c,
                                None => return Some(self.syntax("expected for-in iterator (a List local), found end of file".into())),
                            };
                            return Some(self.name_error(
                                gn,
                                tn,
                                spn,
                                "for-in iterator (a List local)",
                            ));
                        }
                    }
                    if let Some(d) = self.expect_text("{") {
                        return Some(d);
                    }
                    if let Some(d) = self.walk_block_body(false) {
                        return Some(d);
                    }
                    return self.sep_check();
                }
                if !wide {
                    return Some(
                        self.unsupported("`for` is not part of this source profile".into(), sp),
                    );
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
            "is" if self.cx.profile.is_collections() => {
                // PLAN-928（T-03）：`is <local> { Arm [(v)]? -> { .. } }`。
                self.i += 1;
                match self.cur() {
                    Some((_, sg, _, _)) if sg == k.tok_ident => {
                        self.i += 1;
                    }
                    _ => {
                        let (spn, gn, tn, _) = match self.cur() {
                            Some(c) => c,
                            None => {
                                return Some(
                                    self.syntax("expected scrutinee, found end of file".into()),
                                )
                            }
                        };
                        return Some(self.name_error(gn, tn, spn, "scrutinee"));
                    }
                }
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                loop {
                    while matches!(self.cur(), Some((_, _, ";", _))) {
                        self.i += 1;
                    }
                    match self.cur() {
                        None => return Some(self.syntax("expected `}` before end of file".into())),
                        Some((_, _, "}", _)) => {
                            self.i += 1;
                            break;
                        }
                        Some((_, ag, _, _)) if ag == k.tok_ident => {
                            // 限定形式 `Enum.Name`
                            self.i += 1;
                            if let Some((_, _, ".", _)) = self.cur() {
                                self.i += 1;
                                match self.cur() {
                                    Some((_, pg, _, _)) if pg == k.tok_ident => {
                                        self.i += 1;
                                    }
                                    _ => {
                                        let (spn, gn, tn, _) =
                                            match self.cur() {
                                                Some(c) => c,
                                                None => return Some(
                                                    self.syntax(
                                                        "expected arm pattern, found end of file"
                                                            .into(),
                                                    ),
                                                ),
                                            };
                                        return Some(self.name_error(gn, tn, spn, "arm pattern"));
                                    }
                                }
                            }
                            // 载荷绑定 `(v)`
                            if let Some((_, _, "(", _)) = self.cur() {
                                self.i += 1;
                                match self.cur() {
                                    Some((_, bg, _, _)) if bg == k.tok_ident => {
                                        self.i += 1;
                                    }
                                    _ => {
                                        let (spn, gn, tn, _) = match self.cur() {
                                            Some(c) => c,
                                            None => return Some(self.syntax("expected arm payload binding, found end of file".into())),
                                        };
                                        return Some(self.name_error(
                                            gn,
                                            tn,
                                            spn,
                                            "arm payload binding",
                                        ));
                                    }
                                }
                                match self.cur() {
                                    Some((_, _, ")", _)) => {
                                        self.i += 1;
                                    }
                                    _ => {
                                        return Some(self.syntax(
                                            "expected `)` after the arm payload binding".into(),
                                        ))
                                    }
                                }
                            }
                            match self.cur() {
                                Some((_, _, "->", _)) => {
                                    self.i += 1;
                                }
                                _ => {
                                    return Some(
                                        self.syntax("expected `->` after the arm pattern".into()),
                                    )
                                }
                            }
                            if let Some(d) = self.expect_text("{") {
                                return Some(d);
                            }
                            if let Some(d) = self.walk_block_body(false) {
                                return Some(d);
                            }
                        }
                        Some((_, t2, _, _)) => {
                            return Some(
                                self.syntax(format!("expected an arm pattern, found `{}`", t2)),
                            );
                        }
                    }
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
                                        return Some(self.name_error(
                                            fg,
                                            ftext,
                                            fsp,
                                            "field name after `.`",
                                        ));
                                    }
                                    None => {
                                        return Some(
                                            self.syntax(
                                                "expected field name after `.`, found end of file"
                                                    .into(),
                                            ),
                                        )
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
                                if !(self.cx.profile.is_entry_unit()
                                    || self.cx.profile.is_collections())
                                {
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
                    if !(self.cx.profile.is_entry_unit() || self.cx.profile.is_collections()) {
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
                format!(
                    "{} cannot be used as a name in this source profile",
                    raw_describe(g, text, self.k())
                ),
                sp,
            );
        }
        if self.is_kw(g, text) || self.is_reserved(g) {
            return self.unsupported(format!("keyword `{}` cannot be used as a name", text), sp);
        }
        self.syntax(format!(
            "expected {}, found {}",
            what,
            describe_g(g, text, self.k())
        ))
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
        matches!(
            text,
            "||" | "&&" | "==" | "!=" | ">" | "<=" | ">=" | "-" | "/" | "%"
        )
    }

    /// PLAN-928（T-07）：`( 参数? ) =>` 闭包头前瞻（token 级；镜像旧
    /// closure_header_ahead 的匹配 `)` 后随 `=>` 判定）。
    fn closure_header_ahead_tokens(&self, open: usize) -> bool {
        let mut depth = 0usize;
        let mut j = open;
        while let Some((_, _, t, _)) = self.at(j) {
            if t == "(" {
                depth += 1;
            } else if t == ")" {
                depth -= 1;
                if depth == 0 {
                    return matches!(self.at(j + 1), Some((_, _, "=>", _)));
                }
            }
            j += 1;
        }
        false
    }

    /// PLAN-928（T-03）：`ident <` 位是否为泛型接收——平衡 `<...>` 后随
    /// `.` 才提交（镜像旧 parser 的 generic_scan_commits）。
    fn generic_scan_commits(&self, open_lt: usize) -> bool {
        let k = self.k();
        let mut depth = 0usize;
        let mut j = open_lt;
        while let Some((_, g, text, _)) = self.at(j) {
            if text == "<" {
                depth += 1;
            } else if text == ">" {
                depth -= 1;
                if depth == 0 {
                    return matches!(self.at(j + 1), Some((_, _, ".", _)));
                }
            } else if !(g == k.tok_ident
                || g == k.tok_int_kw
                || g == k.tok_i32
                || g == k.tok_bool
                || text == ","
                || text == "?")
            {
                return false;
            }
            j += 1;
        }
        false
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
                    None => Some(self.syntax(format!(
                        "expected expression, found {}",
                        self.describe_cur()
                    ))),
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
                    Some((esp, eg, etext, _))
                        if eg == k.tok_ident
                            || etext == "true"
                            || etext == "false"
                            || etext == "(" =>
                    {
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
                        format!(
                            "{} `{}` is not part of this source profile",
                            raw_describe(g, text, k),
                            text
                        ),
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
                return Some(
                    self.unsupported("unary `!` is not part of this source profile".into(), sp),
                );
            } else if text == "+" && wide {
                return Some(
                    self.unsupported("unary `+` is not part of this source profile".into(), sp),
                );
            } else if text == "type" || text == "static" || text == "mut" {
                return Some(self.unsupported(
                    format!("`{}` is not a value in this source profile", text),
                    sp,
                ));
            } else if self.is_reserved(g) {
                return Some(self.unsupported(
                    format!("keyword `{}` is not part of this source profile", text),
                    sp,
                ));
            } else if self.is_raw(g) {
                let msg = if g == k.tok_affix_num || g == k.tok_block_open {
                    format!(
                        "{} `{}` is not part of this source profile",
                        raw_describe(g, text, k),
                        text
                    )
                } else {
                    format!(
                        "{} `{}` is not part of this source profile",
                        raw_describe(g, text, k),
                        text
                    )
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
                        if matches!(self.cur(), Some((_, _, "(", l2)) if in_br || l2 == self.prev_line())
                        {
                            self.i += 1;
                            if matches!(self.cur(), Some((_, _, ")", _))) {
                                self.i += 1;
                                // 旧怪癖保真：空实参 leading-dot 调用后旧 parser
                                // continue-'outer 重入 primary（不落回算子循环）——
                                // 下一 token 须能起始 primary；anchored 保持，未
                                // 括号待归约时锚定调用行（L04/L05/FIN864B 差分）。
                                continue;
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
            } else if g == k.tok_ident
                && self.cx.profile.is_closures()
                && text == "async"
                && self.closure_header_ahead_tokens(self.i + 1)
            {
                // PLAN-928（T-07）：`async (参数) =>` —— 闭包面前缀保留名
                // （镜像旧 parser 的 async 闭包拒绝；专用码）。
                return Some(Diagnostic::new(
                    crate::atom_text::Stage::Text,
                    "source.closure-unsupported",
                    "async closures are not supported by the closures source profile".to_string(),
                    sp,
                ));
            } else if g == k.tok_ident {
                // PLAN-928（T-03）：`List<T>` 接收位——平衡 `<...>` 后随 `.`
                // 才提交（镜像旧 generic_scan_commits；普通 `a < b` 比较不受
                // 影响——扫描失败即落回普通 ident + 二元 `<`）。
                if self.cx.profile.is_collections() {
                    if let Some((_, _, "<", _)) = self.at(self.i + 1) {
                        if self.generic_scan_commits(self.i + 1) {
                            self.i += 1;
                            let mut depth = 0usize;
                            loop {
                                match self.cur() {
                                    Some((_, _, "<", _)) => depth += 1,
                                    Some((_, _, ">", _)) => depth -= 1,
                                    Some(_) => {}
                                    None => break,
                                }
                                self.i += 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            anchored = false;
                            if let Some(d) = self.after_value(&mut frames, &mut anchored) {
                                return d;
                            }
                            continue;
                        }
                    }
                }
                // ctor（record 面，`{` 同行或括号内）
                if self.cx.profile.is_record() {
                    if let Some((_, _, "{", l2)) = self.at(self.i + 1) {
                        if in_br || l2 == line {
                            self.i += 2;
                            if matches!(self.cur(), Some((_, _, "}", _))) {
                                self.i += 1;
                                // 旧怪癖保真：空 ctor `Ty { }` 同样走 continue-
                                // 'outer 重入 primary（edge#47 差分）。
                                continue;
                            } else {
                                frames.push(Br::Ctor);
                                // label?
                                self.take_label();
                                anchored = false;
                                continue;
                            }
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
            } else if self.is_kw(g, text)
                || text == ")"
                || text == "}"
                || text == ","
                || text == ";"
                || text == "{"
                || text == "="
                || text == ":"
                || text == "]"
                || text == "["
            {
                return Some(self.syntax(format!(
                    "expected expression, found {}",
                    if g == k.tok_ident {
                        "token".to_string()
                    } else {
                        format!("`{}`", text)
                    }
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
    fn after_value(
        &mut self,
        frames: &mut Vec<Br>,
        anchored: &mut bool,
    ) -> Option<Option<Diagnostic>> {
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
                        Some((_, mg, _, _))
                            if mg == k.tok_r_move
                                && (is_text || self.cx.profile.is_collections()) =>
                        {
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
                                Some((_, wg, w, _))
                                    if wg == k.tok_ident && (w == "str" || w == "Str") =>
                                {
                                    self.i += 1;
                                }
                                _ => {
                                    return Some(Some(
                                        self.syntax("`.to` expects `str` or `Str`".into()),
                                    ))
                                }
                            }
                            if let Some(d) = self.expect_text(")") {
                                return Some(Some(d));
                            }
                            continue;
                        }
                        Some((_, mg, mname, mline)) if mg == k.tok_ident => {
                            let call = matches!(self.at(self.i + 2), Some((_, _, "(", l3)) if in_br || l3 == mline);
                            if !call {
                                return Some(Some(
                                    self.syntax(format!(
                                        "method `.{}` requires parentheses",
                                        mname
                                    )),
                                ));
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
                        _ => {
                            return Some(Some(
                                self.syntax("expected a method name after `.`".into()),
                            ))
                        }
                    }
                }
                // 非 record/text：`.` 不是算子 → unsupported_op
                return Some(Some(self.unsupported(
                    "operator `.` is not part of this source profile".into(),
                    dsp,
                )));
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
                return Some(Some(self.unsupported(
                    format!("operator `{}` is not part of this source profile", text),
                    sp,
                )));
            }
            match text {
                "(" => {
                    return Some(Some(self.syntax(format!(
                        "expected operator, found {}",
                        self.describe_cur()
                    ))))
                }
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
    let (sp, g, text) = (
        leaf_span(cx, leaves[*i].0),
        leaves[*i].1,
        leaves[*i].2.as_str(),
    );
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
        k.tok_let,
        k.tok_var,
        k.tok_if,
        k.tok_else,
        k.tok_while,
        k.tok_loop,
        k.tok_break,
        k.tok_continue,
        k.tok_return,
        k.tok_true,
        k.tok_false,
        k.tok_for,
        k.tok_int_kw,
        k.tok_i32,
        k.tok_bool,
        k.tok_type_kw,
        k.tok_static,
        k.tok_mut,
        k.tok_enum_kw,
        k.tok_is,
        k.tok_in,
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

fn first_meaningful_leaf(_cx: &Cx, tree: &CstTree, node: u32) -> Option<u32> {
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

/// 节点 id 路径（path_to 收集 kind；结构层重根需要真实祖先节点）。
fn node_path_to(tree: &CstTree, from: u32, target: u32, path: &mut Vec<u32>) -> bool {
    if from == target {
        return true;
    }
    path.push(from);
    for c in tree.children(from) {
        if node_path_to(tree, c, target, path) {
            return true;
        }
    }
    path.pop();
    false
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
    /// PLAN-936（T-04）：`#{ expr }` 原子——inner 表达式入场后包裹。
    HashExpr {
        span: Span,
    },
    LetFinish {
        node: u32,
        kw: Span,
        name: Ident,
        ann: Option<TypeAnn>,
        mutable: bool,
    },
    AssignFinish {
        name: Ident,
        path: Vec<Ident>,
    },
    ReturnFinish {
        kw: Span,
    },
    IfCond {
        then_n: u32,
        els_blk: Option<u32>,
        node: u32,
    },
    IfThen {
        cond: Expr,
        els_blk: Option<u32>,
        node: u32,
    },
    IfElse {
        cond: Expr,
        then: Block,
        node: u32,
    },
    WhileCond {
        body_n: u32,
        node: u32,
    },
    WhileBody {
        cond: Expr,
        node: u32,
    },
    LoopBody {
        node: u32,
    },
    ExprStmtFinish,
    FnBodyFinish {
        node: u32,
        name: Ident,
        type_params: Vec<Ident>,
        params: Vec<Param>,
        ret: TypeAnn,
        receiver: Receiver,
        owner: Option<Ident>,
    },
    ScriptFinish {
        start_span: Span,
    },
    /// PLAN-928（T-03）：`for x in <List 局部> { .. }` 体收尾帧。
    ForInBody {
        var: Ident,
        iter: Expr,
        node: u32,
    },
    /// PLAN-928（T-03）：`is <局部> { 臂 }` 逐臂帧 + 汇总帧。
    IsArms(IsMatchCtx),
    IsArmBlock {
        ctx: Box<IsMatchCtx>,
        qual: Option<Ident>,
        pattern: Ident,
        binding: Option<Ident>,
        kw: Span,
    },
    // -- 表达式 --
    ExprEnter(u32),
    /// PLAN-928：lambda 体收尾（块体/表达式体两形；参数与 move 词已冻结）。
    LambdaBlockBody {
        move_capture: bool,
        params: Vec<Param>,
        start: Span,
    },
    LambdaExprBody {
        move_capture: bool,
        params: Vec<Param>,
        start: Span,
    },
    OpPost {
        op: BinOp,
    },
    UnaryPost {
        op: UnaryOp,
    },
    PostfixFoldAll {
        steps: Vec<u32>,
    },
    /// 调用收尾：callee 语义（纯名字 / 方法链尾步）+ 一元链 + 实参数 +
    /// 调用节点终点。接收者值（方法路径）在实参之下。
    CallAssemble {
        callee: CalleePlan,
        argc: usize,
        close_end: usize,
    },
    ArgBuild {
        named: Option<Ident>,
    },
    CtorFieldBuild {
        label: Ident,
    },
    CtorFinish {
        node: u32,
        ty: Ident,
        count: usize,
    },
}

/// 调用 callee 的静态分析结果（enter_call 时求出；组装时消费）。
enum CalleePlan {
    /// 纯名字调用（ident 叶）+ 外→内一元链。
    Plain {
        ident: Ident,
        unary_ops: Vec<UnaryOp>,
    },
    /// PLAN-928（T-07）：表达式被调（括号组/λ 等值位 callee）→ 间接调用
    /// `CallValue`（镜像旧 CallFrame callee_expr；callee 名 "<closure>"）。
    Value {
        callee_n: u32,
        unary_ops: Vec<UnaryOp>,
    },
    /// 方法链：base（接收者求值任务节点）+ 中间步 + 链尾成员叶。
    Method {
        base: u32,
        member: u32,
        mids: Vec<u32>,
        unary_ops: Vec<UnaryOp>,
    },
}

/// PLAN-928（T-03）：is 分派的逐臂装配上下文。
struct IsMatchCtx {
    scrutinee: Ident,
    arms: Vec<MatchArm>,
    remaining: Vec<u32>,
    node: u32,
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
    /// PLAN-935：instances 能力面激活（scalar-instances 裁剪）。
    instances: bool,
    /// PLAN-935：当前投影中函数声明的类型参数名（fn 投影期间有效——
    /// 引擎栈在 run_fn_item 内驱完，无跨函数交错）。
    ty_params: Vec<String>,
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
        Span {
            start: s as usize,
            end: e as usize,
        }
    }
    fn spelling(&self, n: u32) -> String {
        let (s, e) = self.tree.abs_range(n);
        self.snap.read_str(s, e).unwrap_or_default()
    }
    fn spelling_by_range(&self, s: u64, e: u64) -> String {
        self.snap.read_str(s, e).unwrap_or_default()
    }
    /// `pos` 之后的首个非 trivia 叶（错误叶也算——保真旧 describe_cur）。
    fn leaf_after(&self, pos: u64) -> Option<(u32, String, Span)> {
        lex::collect_leaves(&self.tree)
            .into_iter()
            .filter(|l| l.start >= pos)
            .min_by_key(|l| l.start)
            .map(|l| {
                (
                    l.kind,
                    self.spelling_by_range(l.start, l.end),
                    Span {
                        start: l.start as usize,
                        end: l.end as usize,
                    },
                )
            })
    }
    /// 叶起始行（快照换行计数）。
    fn line_of_pos(&self, pos: u64) -> u32 {
        let text = self
            .snap
            .read_str(0, pos.min(self.snap.len() as u64))
            .unwrap_or_default();
        1 + text.bytes().filter(|&b| b == b'\n').count() as u32
    }
    fn children(&self, n: u32) -> Vec<u32> {
        self.tree.children(n)
    }
    fn field_child(&self, n: u32, field: &str) -> Option<u32> {
        let f = self.p.field_id(field)?;
        self.tree.field_child(n, f)
    }
    fn child_of_kind(&self, n: u32, kind: u32) -> Option<u32> {
        self.children(n)
            .into_iter()
            .find(|&c| self.tree.kind(c) == kind)
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
        match kids
            .iter()
            .find(|&&c| cx.tree.node(c).flags & FLAG_TRIVIA == 0)
        {
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
    fn new(
        p: &'a GrammarProgram,
        k: &'a SourceKinds,
        tree: &'a CstTree,
        snap: &'a Snapshot,
        profile: SourceProfile,
        instances: bool,
    ) -> Self {
        Engine {
            cx: Cx {
                p,
                k,
                tree,
                snap,
                profile,
                instances,
                ty_params: Vec::new(),
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
            _ => Expr {
                kind: ExprKind::IntLit(0),
                span: Span { start: 0, end: 0 },
            },
        }
    }
    fn pop_b(&mut self) -> Block {
        match self.vals.pop() {
            Some(Val::B(b)) => b,
            _ => Block {
                stmts: Vec::new(),
                span: Span { start: 0, end: 0 },
            },
        }
    }
    fn pop_s(&mut self) -> Stmt {
        match self.vals.pop() {
            Some(Val::S(s)) => s,
            _ => Stmt::Break {
                span: Span { start: 0, end: 0 },
            },
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
        // PLAN-935（T-02）：`<T>` 类型参数投影（scalar-instances 面才有
        // 该字段；plain 面无此步 = 无字段）。有限形态门：至多一个类型参
        // 数、不重名、模板方法与模板入口具名拒（覆盖边界，不当 token 跳过）。
        let type_params = self.project_type_params(self.cx.field_child(tail, "type_params"));
        if !type_params.is_empty() {
            let tp_span = type_params[0].span;
            if receiver != Receiver::Free || owner.is_some() {
                self.cx.reject(tp_span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        "template methods are not part of the finite instances surface",
                        tp_span,
                    )
                });
            } else if type_params.len() > 1 {
                let last = type_params[type_params.len() - 1].span;
                self.cx.reject(last, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        "at most one type parameter is supported (`<T>`)",
                        last,
                    )
                });
            } else if name.name == "main" {
                self.cx.reject(tp_span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        "the entry function cannot be a template",
                        tp_span,
                    )
                });
            }
        }
        // fn 投影期间类型变量可见（参数/返回/体内注解位）。
        self.cx.ty_params = type_params.iter().map(|t| t.name.clone()).collect();
        let params = self.project_params(self.cx.field_child(tail, "params"));
        if receiver == Receiver::View {
            for prm in &params {
                if prm.name.name == "self" {
                    let span = prm.name.span;
                    self.cx.reject(span, || self_param(span));
                }
            }
        }
        let ret = self.classify_word(self.cx.field_child(tail, "ty").expect("fn ret"), true);
        let body_n = self.cx.field_child(tail, "body").expect("fn body");
        let owner_for_frame = owner.clone();
        self.stack.push(Frame::FnBodyFinish {
            node,
            name,
            type_params,
            params,
            ret,
            receiver,
            owner: owner_for_frame,
        });
        self.enter_block(body_n);
        self.drive();
        self.cx.ty_params.clear();
        match self.vals.pop() {
            Some(Val::F(f)) => f,
            _ => FnItem {
                name: Ident {
                    name: String::new(),
                    span: Span { start: 0, end: 0 },
                },
                type_params: Vec::new(),
                params: Vec::new(),
                ret: TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::Int),
                    span: Span { start: 0, end: 0 },
                },
                body: Block {
                    stmts: Vec::new(),
                    span: Span { start: 0, end: 0 },
                },
                span: Span { start: 0, end: 0 },
                receiver,
                owner: owner.clone(),
                script: false,
            },
        }
    }

    /// PLAN-935（T-02）：fn-tail `<T>` → `Vec<Ident>`（声明序；重名具名拒）。
    fn project_type_params(&mut self, node: Option<u32>) -> Vec<Ident> {
        let mut out = Vec::new();
        let Some(n) = node else { return out };
        let Some(tp_kind) = self.cx.k.type_param else {
            return out;
        };
        let direct = collect_seq(&self.cx, n, tp_kind, "type-param-tail");
        for tp in direct {
            let leaf = self.cx.field_child(tp, "name").expect("type param name");
            let id = ident_of(&mut self.cx, leaf);
            // PLAN-935（T-03）：约束/const 形状（`<T has Bound>` / `<const N>`
            // ——r-has/r-const 保留词形状）语法可识别、此处具名拒（覆盖边
            // 界，不当 token 跳过）。
            let boundary = self
                .cx
                .field_child(tp, "constraint")
                .or_else(|| self.cx.field_child(tp, "const_kw"));
            if let Some(cst_n) = boundary {
                let span = self.cx.span_of(cst_n);
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        "type parameter constraints (and const parameters) are not part of \
                         the finite instances surface",
                        span,
                    )
                });
            }
            if out.iter().any(|p: &Ident| p.name == id.name) {
                let span = id.span;
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        format!("duplicate type parameter `{}`", id.name),
                        span,
                    )
                });
            }
            out.push(id);
        }
        out
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
                Some(mk) if mk == self.cx.k.tok_r_move => {
                    // PLAN-928：closures 面的 move 参数（`fn make(move s Str)`）
                    // 与 text 面同为合法 mode 词。
                    if !(self.cx.profile.is_text() || self.cx.profile.is_closures()) {
                        let mspan = self.cx.span_of(mode_leaf.unwrap());
                        self.cx.reject(mspan, || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                "keyword `move` cannot be used as a name",
                                mspan,
                            )
                        });
                    }
                    ParamMode::Move
                }
                Some(mk) if mk == self.cx.k.tok_mut => {
                    if !self.cx.profile.is_text() && !self.cx.profile.is_record() {
                        let mspan = self.cx.span_of(mode_leaf.unwrap());
                        self.cx.reject(mspan, || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                "keyword `mut` cannot be used as a name",
                                mspan,
                            )
                        });
                    }
                    ParamMode::Mut
                }
                _ => ParamMode::Default,
            };
            let ty =
                self.classify_word(self.cx.field_child(param_n, "ty").expect("param ty"), false);
            let span = match mode_leaf {
                Some(m) => self.cx.span_of(m).join(ty.span),
                None => name.span.join(ty.span),
            };
            out.push(Param {
                span,
                name,
                ty,
                mode,
            });
        }
        out
    }

    /// ret-ty/ann-ty → TypeAnn（词法分类 + profile 拒绝；非递归）。
    /// `pos`：Ret 位在 record 面收**任意** ident（含 str/Str/char——旧
    /// fn_item_tail 的名义分支）；param/field/let 注解位先做文本词拒绝。
    fn classify_word(&mut self, node: u32, ret_pos: bool) -> TypeAnn {
        // PLAN-928：closures 面函数类型（ret/ann 位的 fn-type 子节点承载）。
        if let Some(ft) = self.cx.field_child(node, "fn") {
            return self.project_fn_type(ft);
        }
        // PLAN-928（T-03）：collections/closures 类型位参数化/可选糖。
        if let Some(g) = self.cx.field_child(node, "generic") {
            return self.project_generic_ty(g);
        }
        if let Some(o) = self.cx.field_child(node, "optional") {
            return self.project_optional_ty(o);
        }
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
        let int = |tw: TypeWord| TypeAnn {
            kind: TypeRef::Builtin(tw),
            span,
        };
        match self.cx.tree.kind(word_leaf) {
            x if x == self.cx.k.tok_int_kw => return int(TypeWord::Int),
            x if x == self.cx.k.tok_i32 => return int(TypeWord::I32),
            x if x == self.cx.k.tok_bool => return int(TypeWord::Bool),
            _ => {}
        }
        let word = self.cx.spelling(word_leaf);
        // PLAN-935（T-02）：instances 面类型位裸 ident = 声明类型变量（仅
        // 当拼写匹配当前函数声明的 `<T>`；同名 record/局部不误绑——scalar
        // 面无 record，未匹配的 ident 走下方具名拒绝，不猜默认类型）。
        if self.cx.instances && self.cx.ty_params.iter().any(|t| t == &word) {
            return TypeAnn {
                kind: TypeRef::TyVar(Ident { name: word, span }),
                span,
            };
        }
        let is_text = self.cx.profile.is_text();
        // PLAN-928（T-04，镜像旧 collections_type）：collections 面 `Str`
        // → StrOwned；`char` 仅 closures（909 共享 cell 值域）→ Char；小写
        // `str`/`char` 具名拒绝。text 面维持 str/Str/char 三词。
        if self.cx.profile.is_collections() {
            match word.as_str() {
                "Str" => return int(TypeWord::StrOwned),
                "char" if self.cx.profile.is_closures() => return int(TypeWord::Char),
                "str" | "char" => {
                    let w = word.clone();
                    self.cx.reject(span, move || unsupported_ty(&w, span));
                    return int(TypeWord::Int);
                }
                _ => {}
            }
        }
        let admits_str = is_text;
        if admits_str && matches!(word.as_str(), "str" | "Str" | "char") {
            let tw = match word.as_str() {
                "str" => TypeWord::Str,
                "Str" => TypeWord::StrOwned,
                _ => TypeWord::Char,
            };
            return int(tw);
        }
        if !ret_pos
            && !admits_str
            && !self.cx.profile.is_collections()
            && matches!(word.as_str(), "str" | "Str" | "char")
        {
            self.cx.reject(span, move || unsupported_ty(&word, span));
            return int(TypeWord::Int);
        }
        if self.cx.profile.is_entry_unit() && word == "void" {
            return int(TypeWord::Void);
        }
        if self.cx.profile.is_record() {
            return TypeAnn {
                kind: TypeRef::Record(Ident { name: word, span }),
                span,
            };
        }
        self.cx.reject(span, move || unsupported_ty(&word, span));
        int(TypeWord::Int)
    }

    /// PLAN-928（T-03）：PLAN-864 有限 enum 投影（臂声明序 = 判别序）。
    fn run_enum_item(&mut self, node: u32) -> EnumItem {
        let name_leaf = self.cx.field_child(node, "name").expect("enum name");
        let name = ident_of(&mut self.cx, name_leaf);
        let arm_kind = self.cx.k.enum_arm.unwrap_or(u32::MAX);
        let mut arm_nodes: Vec<u32> = self
            .cx
            .children(node)
            .into_iter()
            .filter(|&c| self.cx.tree.kind(c) == arm_kind)
            .collect();
        arm_nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
        let mut arms = Vec::new();
        for a in arm_nodes {
            let aname_leaf = self.cx.field_child(a, "name").expect("arm name");
            let aname = ident_of(&mut self.cx, aname_leaf);
            let payload = self
                .cx
                .field_child(a, "payload")
                .map(|t| self.classify_word(t, false));
            let span = match &payload {
                Some(t) => aname.span.join(t.span),
                None => aname.span,
            };
            arms.push(EnumArm {
                name: aname,
                payload,
                span,
            });
        }
        EnumItem {
            name,
            arms,
            span: self.cx.span_of(node),
        }
    }

    /// PLAN-928（T-03）：类型位的参数化名义 `List<T>` / `Result<T, E>`。
    fn project_generic_ty(&mut self, node: u32) -> TypeAnn {
        let name_leaf = self
            .cx
            .field_child(node, "name")
            .expect("generic type name");
        let name = ident_of(&mut self.cx, name_leaf);
        let mut args = Vec::new();
        if let Some(an) = self.cx.field_child(node, "args") {
            for a in collect_seq(
                &self.cx,
                an,
                self.cx.k.type_arg.unwrap_or(u32::MAX),
                "type-arg-tail",
            ) {
                args.push(self.classify_ty_arg(a));
            }
        }
        TypeAnn {
            span: self.cx.span_of(node),
            kind: TypeRef::Generic { name, args },
        }
    }

    /// PLAN-928（T-03）：`?T` 可选糖 → `TypeRef::Optional`。
    fn project_optional_ty(&mut self, node: u32) -> TypeAnn {
        let inner = match self.cx.field_child(node, "inner") {
            Some(i) => self.classify_ty_arg(i),
            None => self.classify_word(node, false),
        };
        TypeAnn {
            span: self.cx.span_of(node),
            kind: TypeRef::Optional(Box::new(inner)),
        }
    }

    /// PLAN-928（T-03）：类型实参（词/嵌套泛型/可选糖）。
    fn classify_ty_arg(&mut self, n: u32) -> TypeAnn {
        if let Some(g) = self.cx.field_child(n, "generic") {
            return self.project_generic_ty(g);
        }
        if let Some(o) = self.cx.field_child(n, "optional") {
            return self.project_optional_ty(o);
        }
        self.classify_word(n, false)
    }

    /// PLAN-928：函数类型 `fn(T0, ...) [->] R`（嵌套允许；`->` 缺席 = K 语料
    /// 相邻结果拼写兼容形）。参数/结果经 classify_word（其 fn 分支承载嵌套）。
    fn project_fn_type(&mut self, node: u32) -> TypeAnn {
        let span = self.cx.span_of(node);
        let mut params = Vec::new();
        if let Some(pn) = self.cx.field_child(node, "params") {
            let plist: Vec<u32> = collect_seq(
                &self.cx,
                pn,
                self.cx.k.fn_type_param.unwrap_or(u32::MAX),
                "fn-type-param-tail",
            );
            for p in plist {
                params.push(self.classify_word(p, false));
            }
        }
        // 结果：fn-type-result 节点（词或嵌套 fn 型——classify_word 的 fn
        // 分支处理嵌套；词分支按拼写+profile 分类）。
        let result = match self.cx.field_child(node, "result") {
            Some(res) => self.classify_word(res, true),
            None => self.classify_word(node, true),
        };
        TypeAnn {
            span,
            kind: TypeRef::Function {
                params,
                result: Box::new(result),
            },
        }
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
                let ty = self.classify_word(self.cx.field_child(c, "ty").expect("field ty"), false);
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
                fields.push(FieldDecl {
                    span,
                    name: fname,
                    ty,
                });
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
        TypeItem {
            span: self.cx.span_of(node),
            name,
            fields,
            methods,
        }
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
                name: Ident {
                    name: "main".to_string(),
                    span: start_span,
                },
                type_params: Vec::new(),
                params: Vec::new(),
                ret: TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::Void),
                    span: start_span,
                },
                body: Block {
                    stmts: Vec::new(),
                    span: start_span,
                },
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
            self.vals.push(Val::B(Block {
                stmts: frame.stmts,
                span,
            }));
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
            let kw_kind = if kind == self.cx.k.let_stmt {
                self.cx.k.tok_let
            } else {
                self.cx.k.tok_var
            };
            let kw = self
                .cx
                .child_of_kind(inner, kw_kind)
                .map(|l| self.cx.span_of(l))
                .unwrap_or(Span { start: 0, end: 0 });
            let name_leaf = self.cx.field_child(inner, "name").expect("let name");
            let name = ident_of(&mut self.cx, name_leaf);
            console_name_check(&mut self.cx, &name);
            let ann = self
                .cx
                .field_child(inner, "ann")
                .map(|n| self.classify_word(n, false));
            let mutable = kind == self.cx.k.var_stmt;
            let value_n = self.cx.field_child(inner, "value").expect("let value");
            self.stack.push(Frame::LetFinish {
                node: inner,
                kw,
                name,
                ann,
                mutable,
            });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if kind == self.cx.k.assign_stmt
            || kind == self.cx.k.path_assign_stmt.unwrap_or(u32::MAX)
        {
            let name_leaf = self.cx.field_child(inner, "name").expect("assign name");
            let name = ident_of(&mut self.cx, name_leaf);
            let mut path = Vec::new();
            if kind == self.cx.k.path_assign_stmt.unwrap_or(u32::MAX) {
                // 路径段标识符住在 `path-seg` 子节点内（`many { path-seg }`
                // 直接平铺），不在语句节点的直接 ident 孩子里。
                let seg_kind = self.cx.k.path_seg.unwrap_or(u32::MAX);
                let mut seg_nodes: Vec<u32> = self
                    .cx
                    .children(inner)
                    .into_iter()
                    .filter(|&c| self.cx.tree.kind(c) == seg_kind)
                    .collect();
                seg_nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
                for seg in seg_nodes {
                    if let Some(pl) = self.cx.field_child(seg, "path") {
                        path.push(ident_of(&mut self.cx, pl));
                    }
                }
            }
            let value_n = self.cx.field_child(inner, "value").expect("assign value");
            self.stack.push(Frame::AssignFinish { name, path });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if kind == self.cx.k.return_stmt {
            let kw = self
                .cx
                .child_of_kind(inner, self.cx.k.tok_return)
                .map(|l| self.cx.span_of(l))
                .unwrap_or(Span { start: 0, end: 0 });
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
            let kw = self
                .cx
                .child_of_kind(inner, self.cx.k.tok_if)
                .map(|l| self.cx.span_of(l))
                .unwrap_or(Span { start: 0, end: 0 });
            let _ = kw;
            let cond_n = self.cx.field_child(inner, "cond").expect("if cond");
            let then_n = self.cx.field_child(inner, "then").expect("if then");
            let els_blk = self
                .cx
                .child_of_kind(inner, self.cx.k.else_block)
                .and_then(|eb| self.cx.child_of_kind(eb, self.cx.k.block));
            self.stack.push(Frame::IfCond {
                then_n,
                els_blk,
                node: inner,
            });
            self.stack.push(Frame::ExprEnter(cond_n));
        } else if kind == self.cx.k.while_stmt.unwrap_or(u32::MAX)
            || kind == self.cx.k.for_cond_stmt.unwrap_or(u32::MAX)
        {
            let kw_kind = if kind == self.cx.k.while_stmt.unwrap_or(u32::MAX) {
                self.cx.k.tok_while
            } else {
                self.cx.k.tok_for
            };
            let _kw_leaf = self.cx.child_of_kind(inner, kw_kind);
            let cond_n = self.cx.field_child(inner, "cond").expect("loop cond");
            let body_n = self.cx.field_child(inner, "body").expect("loop body");
            self.stack.push(Frame::WhileCond {
                body_n,
                node: inner,
            });
            self.stack.push(Frame::ExprEnter(cond_n));
        } else if kind == self.cx.k.for_in_stmt.unwrap_or(u32::MAX) {
            let var_leaf = self.cx.field_child(inner, "var").expect("for-in var");
            let var = ident_of(&mut self.cx, var_leaf);
            // 合同 C7：迭代器是裸 List 局部名（ident，非表达式——体块 `{`
            // 不得被误读为具名 ctor）。
            let iter_leaf = self.cx.field_child(inner, "iter").expect("for-in iter");
            let iter_ident = ident_of(&mut self.cx, iter_leaf);
            let iter_span = iter_ident.span;
            let iter = Expr {
                span: iter_span,
                kind: ExprKind::Var(iter_ident),
            };
            let body_n = self.cx.field_child(inner, "body").expect("for-in body");
            self.stack.push(Frame::ForInBody {
                var,
                iter,
                node: inner,
            });
            self.enter_block(body_n);
        } else if kind == self.cx.k.is_stmt.unwrap_or(u32::MAX) {
            let scrutinee_leaf = self
                .cx
                .field_child(inner, "scrutinee")
                .expect("is scrutinee");
            let scrutinee = ident_of(&mut self.cx, scrutinee_leaf);
            let arm_kind = self.cx.k.match_arm.unwrap_or(u32::MAX);
            let mut arm_nodes: Vec<u32> = self
                .cx
                .children(inner)
                .into_iter()
                .filter(|&c| self.cx.tree.kind(c) == arm_kind)
                .collect();
            arm_nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
            self.stack.push(Frame::IsArms(IsMatchCtx {
                scrutinee,
                arms: Vec::new(),
                remaining: arm_nodes,
                node: inner,
            }));
        } else if kind == self.cx.k.loop_stmt {
            let body_n = self.cx.field_child(inner, "body").expect("loop body");
            self.stack.push(Frame::LoopBody { node: inner });
            self.enter_block(body_n);
        } else if kind == self.cx.k.expr_stmt {
            let value_n = self
                .cx
                .field_child(inner, "value")
                .expect("expr stmt value");
            // 旧门 = surface.is_entry_unit()（含 HostConsole——控制台脚本面）。
            // 旧 stmt head 的表达式候选只从 ident/true/false/int 起始——其余
            // 起始（如 `(`）是"expected statement, found X"（c03 差分）。
            if !self.cx.profile.is_entry_unit() {
                let first_leaf = lex::collect_leaves(&self.cx.tree)
                    .into_iter()
                    .filter(|l| l.start >= self.cx.tree.abs_range(value_n).0)
                    .min_by_key(|l| l.start);
                if let Some(l) = first_leaf {
                    let k = &self.cx.k;
                    let cand = l.kind == k.tok_ident
                        || l.kind == k.tok_true
                        || l.kind == k.tok_false
                        || l.kind == k.tok_int_lit;
                    if !cand {
                        let span = Span {
                            start: l.start as usize,
                            end: l.end as usize,
                        };
                        let text = self.cx.spelling_by_range(l.start, l.end);
                        let found = describe_g(l.kind, &text, k);
                        self.cx.reject(span, move || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.syntax",
                                format!("expected statement, found {found}"),
                                span,
                            )
                        });
                    }
                }
            }
            if !(self.cx.profile.is_entry_unit() || self.cx.profile.is_collections()) {
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
            let inner = self
                .cx
                .children(node)
                .into_iter()
                .find(|&c| self.cx.tree.child_count(c) > 0);
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
                    self.vals.push(Val::E(Expr {
                        kind: ExprKind::IntLit(0),
                        span,
                    }));
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
                        self.cx
                            .reject(span, || syntax_diag("expected expression", span));
                        self.vals.push(Val::E(Expr {
                            kind: ExprKind::IntLit(0),
                            span,
                        }));
                    }
                }
                return;
            }
            if let Some(post) = self.cx.child_of_kind(node, self.cx.k.postfix_expr) {
                self.stack.push(Frame::ExprEnter(post));
                return;
            }
            let span = first_leaf_span(&self.cx, node);
            self.cx
                .reject(span, || syntax_diag("expected expression", span));
            self.vals.push(Val::E(Expr {
                kind: ExprKind::IntLit(0),
                span,
            }));
            return;
        }
        if kind == self.cx.k.postfix_expr || kind == KIND_POSTFIX {
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
                self.cx
                    .reject(span, || syntax_diag("binary operands", span));
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::IntLit(0),
                    span,
                }));
            }
            return;
        }
        self.enter_atom(node);
    }

    fn enter_atom(&mut self, node: u32) {
        for c in self.cx.children(node) {
            let ck = self.cx.tree.kind(c);
            // PLAN-950: preserve the qualified source identity for module binding.
            if self
                .cx
                .p
                .rules
                .iter()
                .position(|r| r.name == "qualified-name")
                .is_some_and(|r| ck == ace_syntax_runtime::program::RULE_KIND_BASE + r as u32)
            {
                if let Some(id) = self.first_ident_opt(c) {
                    self.vals.push(Val::E(Expr {
                        span: id.span,
                        kind: ExprKind::Var(id),
                    }));
                    return;
                }
            }
            // PLAN-928（T-03）：`List<T>.member` 接收原子。
            if ck == self.cx.k.generic_atom.unwrap_or(u32::MAX) {
                let name_leaf = self.cx.field_child(c, "name").expect("generic atom name");
                let name = ident_of(&mut self.cx, name_leaf);
                let mut args = Vec::new();
                if let Some(an) = self.cx.field_child(c, "args") {
                    for a in collect_seq(
                        &self.cx,
                        an,
                        self.cx.k.type_arg.unwrap_or(u32::MAX),
                        "type-arg-tail",
                    ) {
                        args.push(self.classify_ty_arg(a));
                    }
                }
                let gt_leaf = self
                    .cx
                    .children(c)
                    .into_iter()
                    .find(|&l| self.cx.tree.kind(l) == self.cx.k.op_gt)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or_else(|| self.cx.span_of(c));
                let g_span = name.span.join(gt_leaf);
                let g = Expr {
                    span: g_span,
                    kind: ExprKind::GenericType { name, args },
                };
                let member_leaf = self
                    .cx
                    .field_child(c, "member")
                    .expect("generic atom member");
                let mspan = self.cx.span_of(member_leaf);
                let mname = self.cx.spelling(member_leaf);
                self.vals.push(Val::E(Expr {
                    span: g_span.join(mspan),
                    kind: ExprKind::Field {
                        base: Box::new(g),
                        name: Ident {
                            name: mname,
                            span: mspan,
                        },
                    },
                }));
                return;
            }
            // PLAN-936（T-04）：`#{ expr }` CT 需求原子（scalar-ct 面）。
            if ck == self.cx.k.hash_expr.unwrap_or(u32::MAX) {
                let span = self.cx.span_of(c);
                match self.cx.field_child(c, "expr") {
                    Some(inner) => {
                        self.stack.push(Frame::HashExpr { span });
                        self.stack.push(Frame::ExprEnter(inner));
                    }
                    None => {
                        self.cx.reject(span, || {
                            syntax_diag("expected an expression inside `#{ ... }`", span)
                        });
                        self.vals.push(Val::E(Expr {
                            kind: ExprKind::IntLit(0),
                            span,
                        }));
                    }
                }
                return;
            }
            // PLAN-928：lambda 原子（closures 面）——先于其余原子形状。
            if ck == self.cx.k.lambda_expr.unwrap_or(u32::MAX) {
                self.enter_lambda(c);
                return;
            }
            if ck == self.cx.k.leading_dot.unwrap_or(u32::MAX) {
                let dot_leaf = self
                    .cx
                    .children(c)
                    .into_iter()
                    .find(|&l| self.cx.tree.kind(l) == self.cx.k.tok_dot);
                let dspan = dot_leaf
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or_else(|| self.cx.span_of(c));
                let member = self
                    .cx
                    .children(c)
                    .into_iter()
                    .find(|&l| self.cx.tree.kind(l) == self.cx.k.tok_ident)
                    .expect("leading-dot member");
                let mspan = self.cx.span_of(member);
                let mname = self.cx.spelling(member);
                let recv = Expr {
                    span: dspan,
                    kind: ExprKind::Var(Ident {
                        name: "self".to_string(),
                        span: dspan,
                    }),
                };
                let span = dspan.join(mspan);
                // `.name(args)`：args 由 KIND_CALL 收尾（callee 含此后缀链）。
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Field {
                        base: Box::new(recv),
                        name: Ident {
                            name: mname,
                            span: mspan,
                        },
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
                    .map(|fl| {
                        collect_seq(
                            &self.cx,
                            fl,
                            self.cx.k.ctor_field.unwrap_or(u32::MAX),
                            "ctor-tail",
                        )
                    })
                    .unwrap_or_default();
                let count = field_nodes.len();
                if count == 0 {
                    // 旧怪癖：空 ctor `Ty { }` 重入 primary（同 leading-dot 空调用）。
                    self.empty_value_quirk(self.cx.span_of(c).end);
                }
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
                // PLAN-928：collections/closures 面拥有 Str 字面量（元素域）。
                if !self.cx.profile.admits_text_literals() {
                    // 旧合同：非 text 面 lexer 产 Raw(StrLit)、primary 拒绝。
                    self.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("string literal `{raw}` is not part of this source profile"),
                            span,
                        )
                    });
                    self.vals.push(Val::E(Expr {
                        kind: ExprKind::StrLit(String::new()),
                        span,
                    }));
                    return;
                }
                let inner = &raw.as_bytes()[1..raw.len() - 1];
                let value = lex::decode_str_value(inner);
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::StrLit(value),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.tok_char_lit {
                let span = self.cx.span_of(c);
                let raw = self.cx.spelling(c);
                // PLAN-928：collections/closures 面拥有 char 字面量。
                if !self.cx.profile.admits_text_literals() {
                    self.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("char literal `{raw}` is not part of this source profile"),
                            span,
                        )
                    });
                    self.vals.push(Val::E(Expr {
                        kind: ExprKind::CharLit(0),
                        span,
                    }));
                    return;
                }
                let inner = &raw.as_bytes()[1..raw.len() - 1];
                let scalar = lex::decode_char_value(inner);
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::CharLit(scalar),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.tok_affix_num {
                // 带缀/进制/浮点数字面量：所有面都是 Raw 排除面（旧 Raw 臂）。
                let span = self.cx.span_of(c);
                let raw = self.cx.spelling(c);
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!(
                            "suffixed/radix/float numeric literal `{raw}` is not part of this source profile"
                        ),
                        span,
                    )
                });
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::IntLit(0),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.tok_true || ck == self.cx.k.tok_false {
                let span = self.cx.span_of(c);
                let v = ck == self.cx.k.tok_true;
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::BoolLit(v),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.tok_ident {
                let span = self.cx.span_of(c);
                let name = self.cx.spelling(c);
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::Var(Ident { name, span }),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.expr {
                // 括号组：内层表达式（span 保持内层自身——旧合同）。
                let inner = self
                    .cx
                    .children(c)
                    .into_iter()
                    .find(|&g| self.cx.tree.child_count(g) > 0);
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
                        self.vals.push(Val::E(Expr {
                            kind: ExprKind::IntLit(0),
                            span,
                        }));
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
        self.vals.push(Val::E(Expr {
            kind: ExprKind::IntLit(0),
            span,
        }));
    }

    /// PLAN-928 T-02：lambda 投影（closures 面）。`move? ( 参数? ) => 块|表达式`
    /// ——参数 = ident + 类型（无 mode 词，旧 closure_expr 合同）；块体走既有
    /// 语句帧机；span = move/`(` 起点join体终点。
    fn enter_lambda(&mut self, node: u32) {
        let mode_leaf = self.cx.child_of_kind(node, self.cx.k.tok_r_move);
        let move_capture = mode_leaf.is_some();
        let start = match mode_leaf {
            Some(m) => self.cx.span_of(m),
            None => self
                .cx
                .child_of_kind(node, self.cx.k.tok_lparen)
                .map(|l| self.cx.span_of(l))
                .unwrap_or_else(|| self.cx.span_of(node)),
        };
        let mut params = Vec::new();
        if let Some(pn) = self.cx.field_child(node, "params") {
            let plist: Vec<u32> = collect_seq(
                &self.cx,
                pn,
                self.cx.k.lambda_param.unwrap_or(u32::MAX),
                "lambda-param-tail",
            );
            for p in plist {
                let name_leaf = self.cx.field_child(p, "name").expect("lambda param name");
                let name = ident_of(&mut self.cx, name_leaf);
                let ty = self
                    .cx
                    .field_child(p, "ty")
                    .map(|t| self.classify_word(t, false))
                    .unwrap_or(TypeAnn {
                        kind: TypeRef::Builtin(TypeWord::Int),
                        span: name.span,
                    });
                let span = name.span.join(ty.span);
                params.push(Param {
                    span,
                    name,
                    ty,
                    mode: ParamMode::Default,
                });
            }
        }
        if let Some(body_n) = self.cx.field_child(node, "body") {
            if let Some(bb) = self.cx.field_child(body_n, "block-body") {
                self.stack.push(Frame::LambdaBlockBody {
                    move_capture,
                    params,
                    start,
                });
                self.enter_block(bb);
            } else if let Some(eb) = self.cx.field_child(body_n, "expr-body") {
                self.stack.push(Frame::LambdaExprBody {
                    move_capture,
                    params,
                    start,
                });
                self.stack.push(Frame::ExprEnter(eb));
            } else {
                let span = self.cx.span_of(node);
                self.cx.reject(span, || {
                    syntax_diag("expected lambda body after `=>`", span)
                });
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::IntLit(0),
                    span,
                }));
            }
        } else {
            let span = self.cx.span_of(node);
            self.cx.reject(span, || {
                syntax_diag("expected lambda body after `=>`", span)
            });
            self.vals.push(Val::E(Expr {
                kind: ExprKind::IntLit(0),
                span,
            }));
        }
    }

    fn project_int_literal(&mut self, node: u32) -> Expr {
        let int_leaf = self
            .cx
            .child_of_kind(node, self.cx.k.tok_int_lit)
            .expect("int digits");
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
            Expr {
                kind: ExprKind::IntLit((mag as i128 * -1) as i32),
                span,
            }
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
            Expr {
                kind: ExprKind::IntLit(mag as i32),
                span,
            }
        }
    }

    fn split_postfix(&self, node: u32) -> (u32, Vec<u32>) {
        // KIND_POSTFIX（调用收尾续读）：孩子 = [操作数, postfix-tail 规则节点]。
        if self.cx.tree.kind(node) == KIND_POSTFIX {
            let kids = self.cx.children(node);
            let base = *kids.first().expect("postfix wrap operand");
            let mut steps: Vec<u32> = Vec::new();
            if let Some(&tail) = kids.get(1) {
                let step_kind = self.cx.k.postfix_step.unwrap_or(u32::MAX);
                for c in self.cx.children(tail) {
                    if self.cx.tree.kind(c) == step_kind {
                        steps.push(c);
                    }
                }
            }
            return (base, steps);
        }
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
        let has_to = self
            .cx
            .children(step)
            .into_iter()
            .any(|c| self.cx.tree.kind(c) == self.cx.k.tok_r_to);
        if has_to {
            let word_leaf = self
                .cx
                .children(step)
                .into_iter()
                .find(|&c| self.cx.tree.kind(c) == self.cx.k.tok_ident)
                .expect(".to word");
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
                            name: Ident {
                                name: "to".to_string(),
                                span: word_span,
                            },
                            args: vec![Arg::Pos(Expr {
                                span: word_span,
                                kind: ExprKind::TypeWordArg(tw),
                            })],
                        },
                    }
                }
                None => {
                    self.cx.reject(word_span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            "`.to` expects `str` or `Str`",
                            word_span,
                        )
                    });
                    let span = recv.span.join(word_span);
                    Expr {
                        span,
                        kind: ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident {
                                name: "to".to_string(),
                                span: word_span,
                            },
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
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`.mut` applies to a local variable",
                            span,
                        )
                    });
                    Ident {
                        name: String::new(),
                        span: mspan,
                    }
                }
            };
            let span = name.span.join(mspan);
            return Expr {
                kind: ExprKind::MutArg { name },
                span,
            };
        }
        if mk == self.cx.k.tok_r_move {
            let mspan = self.cx.span_of(member);
            let span = recv.span.join(mspan);
            return Expr {
                span,
                kind: ExprKind::Method {
                    recv: Box::new(recv),
                    name: Ident {
                        name: "move".to_string(),
                        span: mspan,
                    },
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
                    name: Ident {
                        name: mname,
                        span: mspan,
                    },
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
                    name: Ident {
                        name: mname,
                        span: mspan,
                    },
                },
            }
        }
    }

    // ------------------------------------------------------------ calls

    fn enter_call(&mut self, node: u32) {
        let kids = self.cx.children(node);
        let callee_n = kids
            .iter()
            .copied()
            .find(|&c| self.cx.tree.kind(c) != self.cx.k.args)
            .expect("call callee");
        let args_n = kids
            .iter()
            .copied()
            .find(|&c| self.cx.tree.kind(c) == self.cx.k.args);
        let close_end = self.cx.span_of(node).end;
        let arg_nodes: Vec<u32> = args_n
            .map(|an| collect_seq(&self.cx, an, self.cx.k.arg, "arg-tail"))
            .unwrap_or_default();
        let argc = arg_nodes.len();
        // 旧 parser 怪癖保真（L04/L05/LSTR864/FIN864B 差分）：leading-dot
        // 空实参调用 `.m()` 走 continue-'outer 重入 primary——下一 token
        // 必须能起始 primary；且存在未括号待归约运算时锚定调用行（跨行
        // → unbracketed-newline；同行非 primary → expected expression）。
        if argc == 0 && self.callee_base_leading_dot(callee_n) {
            self.empty_value_quirk(close_end);
        }
        // callee 静态分析：expr 分派 → 一元链 → postfix/atom。callee 可能自身
        // 就是 unary_expr（pratt 把 primary 的 unary 节点作 callee——`!f(x)`
        // 的 Not/Neg 在此先收，is-digit 差分）；随后与原路径一致地下钻。
        let plan = {
            let mut unary_pre: Vec<UnaryOp> = Vec::new();
            let mut wrap = callee_n;
            while self.cx.tree.kind(wrap) == self.cx.k.unary_expr {
                let hm = self.cx.child_of_kind(wrap, self.cx.k.tok_minus).is_some();
                let hb = self.cx.child_of_kind(wrap, self.cx.k.tok_bang).is_some();
                if !hm && !hb {
                    break;
                }
                unary_pre.push(if hb { UnaryOp::Not } else { UnaryOp::Neg });
                match self
                    .cx
                    .child_of_kind(wrap, self.cx.k.unary_expr)
                    .or_else(|| self.cx.child_of_kind(wrap, self.cx.k.postfix_expr))
                {
                    Some(o) => wrap = o,
                    None => break,
                }
            }
            // PLAN-928（T-07）：括号组被调（paren-group callee）恒为间接调用
            // CallValue（镜像旧 CallFrame callee_expr——含 (f)(3) 形）。
            let mut paren_callee: Option<u32> = None;
            if self.cx.tree.kind(wrap) == self.cx.k.expr {
                paren_callee = Some(wrap);
            }
            if paren_callee.is_none()
                && self.cx.tree.kind(wrap) != KIND_POSTFIX
                && self.cx.tree.kind(wrap) != self.cx.k.postfix_expr
            {
                wrap = self
                    .cx
                    .children(wrap)
                    .into_iter()
                    .find(|&c| self.cx.tree.child_count(c) > 0)
                    .unwrap_or(wrap);
            }
            // 括号组 callee 在 atom 层显式探测（沿结构链降到 atom-expr，
            // 其 k.expr 孩子即括号组值位）。
            // PLAN-928（T-10 修正）：括号组 callee 在 atom 层显式探测（沿
            // 结构链降到 atom-expr，其 k.expr 孩子即括号组值位）。下降途中
            // 若穿过 postfix-step（如 `(a + "z").len()` 的 `.len`），括号组
            // 是**接收者基座**而非被调——不得强制 CallValue；仅当括号组是
            // 整个 callee（`(f)(3)` 形，无点步）才是间接调用。T-07 首版漏
            // 此区分，Text profile 的 `(组).方法()` 被误投 CallValue
            // （a2c_rt ledger 面实测暴露）。
            {
                let mut a = wrap;
                let mut through_postfix_step = false;
                loop {
                    let ka = self.cx.tree.kind(a);
                    if ka == self.cx.k.atom_expr {
                        break;
                    }
                    if ka == self.cx.k.unary_expr
                        || ka == self.cx.k.postfix_expr
                        || ka == KIND_POSTFIX
                    {
                        if let Some(step_kind) = self.cx.k.postfix_step {
                            if self
                                .cx
                                .children(a)
                                .into_iter()
                                .any(|c| self.cx.tree.kind(c) == step_kind)
                            {
                                through_postfix_step = true;
                            }
                        }
                        match self
                            .cx
                            .children(a)
                            .into_iter()
                            .find(|&c| self.cx.tree.child_count(c) > 0)
                        {
                            Some(c) => a = c,
                            None => break,
                        }
                    } else {
                        break;
                    }
                }
                if !through_postfix_step && self.cx.tree.kind(a) == self.cx.k.atom_expr {
                    if let Some(e) = self
                        .cx
                        .children(a)
                        .into_iter()
                        .find(|&c| self.cx.tree.kind(c) == self.cx.k.expr)
                    {
                        paren_callee = Some(e);
                    }
                }
            }
            let mut unary_ops: Vec<UnaryOp> = unary_pre;
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
            if self.cx.tree.kind(wrap) == self.cx.k.postfix_expr
                || self.cx.tree.kind(wrap) == KIND_POSTFIX
            {
                let (base, steps) = self.split_postfix(wrap);
                // PLAN-928（T-07）：括号组 callee 优先于名字/方法分析。
                if let Some(e) = paren_callee {
                    CalleePlan::Value {
                        callee_n: e,
                        unary_ops,
                    }
                } else {
                    match steps.split_last() {
                        Some((last, mids)) => CalleePlan::Method {
                            base,
                            member: self.step_member(*last).unwrap_or(*last),
                            mids: mids.to_vec(),
                            unary_ops,
                        },
                        None => {
                            // PLAN-928（T-03）：`List<T>.m(...)`——泛型接收原子自带
                            // 尾随 dot 成员（generic-atom）。
                            let ga = self.cx.children(base).into_iter().find(|&c| {
                                self.cx.tree.kind(c) == self.cx.k.generic_atom.unwrap_or(u32::MAX)
                            });
                            match ga.and_then(|g| self.cx.field_child(g, "member")) {
                                Some(member) => CalleePlan::Method {
                                    base,
                                    member,
                                    mids: Vec::new(),
                                    unary_ops,
                                },
                                None => match self.first_ident_opt(base) {
                                    Some(ident) => CalleePlan::Plain { ident, unary_ops },
                                    None => CalleePlan::Value {
                                        callee_n: base,
                                        unary_ops,
                                    },
                                },
                            }
                        }
                    }
                }
            } else if let Some(e) = paren_callee {
                CalleePlan::Value {
                    callee_n: e,
                    unary_ops,
                }
            } else {
                match self.first_ident_opt(wrap) {
                    Some(ident) => CalleePlan::Plain { ident, unary_ops },
                    None => CalleePlan::Value {
                        callee_n: wrap,
                        unary_ops,
                    },
                }
            }
        };
        // 压帧序（后压先弹）：接收者 base … 接收者折叠 … 实参任务 … 汇总帧。
        // 值栈布局：接收者值在实参值**之下**（assemble_call 先弹实参再弹接收者）。
        let push_args = |this: &mut Self, arg_nodes: Vec<u32>| {
            for a in arg_nodes.into_iter().rev() {
                if let Some(named) = this.cx.child_of_kind(a, this.cx.k.named_arg) {
                    let label_leaf = this.cx.field_child(named, "label").expect("arg label");
                    let label = ident_of(&mut this.cx, label_leaf);
                    let value_n = this.cx.field_child(named, "value").expect("arg value");
                    this.stack.push(Frame::ArgBuild { named: Some(label) });
                    this.stack.push(Frame::ExprEnter(value_n));
                } else if let Some(value_n) = this.cx.child_of_kind(a, this.cx.k.expr) {
                    this.stack.push(Frame::ArgBuild { named: None });
                    this.stack.push(Frame::ExprEnter(value_n));
                }
            }
        };
        match plan {
            CalleePlan::Value {
                callee_n,
                unary_ops,
            } => {
                self.stack.push(Frame::CallAssemble {
                    callee: CalleePlan::Value {
                        callee_n,
                        unary_ops,
                    },
                    argc,
                    close_end,
                });
                push_args(self, arg_nodes);
                self.stack.push(Frame::ExprEnter(callee_n));
            }
            CalleePlan::Method {
                base,
                member,
                mids,
                unary_ops,
            } => {
                let fold_mids = mids.clone();
                self.stack.push(Frame::CallAssemble {
                    callee: CalleePlan::Method {
                        base,
                        member,
                        mids,
                        unary_ops,
                    },
                    argc,
                    close_end,
                });
                push_args(self, arg_nodes);
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
                push_args(self, arg_nodes);
            }
        }
    }

    /// 旧 parser continue-'outer 怪癖（空实参 leading-dot 调用 / 空 ctor）：
    /// 值完成后重入 primary——下一 token 须能起始 primary；存在未括号待归约
    /// 运算时锚定值行（跨行 → unbracketed-newline；非 primary → expected
    /// expression）。L04/L05/LSTR864/FIN864B/edge47 差分。
    fn empty_value_quirk(&mut self, close_end: usize) {
        let close_end_u64 = close_end as u64;
        if let Some((g, text, span)) = self.cx.leaf_after(close_end_u64) {
            let pending_unbracketed = self.pending_unbracketed_op();
            let call_line = self.cx.line_of_pos(close_end_u64.saturating_sub(1));
            let next_line = self.cx.line_of_pos(span.start as u64);
            if pending_unbracketed && next_line > call_line {
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        "unbracketed expression cannot continue across a newline",
                        span,
                    )
                });
            } else {
                let k = &self.cx.k;
                let primary_start = matches!(
                    g,
                    x if x == k.tok_ident
                        || x == k.tok_int_lit
                        || x == k.tok_str_lit
                        || x == k.tok_char_lit
                        || x == k.tok_true
                        || x == k.tok_false
                        || x == k.tok_lparen
                        || x == k.tok_minus
                        || x == k.tok_bang
                        || (x == k.tok_dot && self.cx.profile.is_record())
                );
                if !primary_start {
                    let found = describe_g(g, &text, k);
                    self.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            format!("expected expression, found {found}"),
                            span,
                        )
                    });
                }
            }
        }
    }

    /// callee 链的 base atom 是否**裸 leading-dot**（`.m()` 的调用本体）。
    /// 有后缀步（`.text.len()`——调用挂在最后一步成员上）= false：
    /// 旧怪癖只覆盖 leading-dot 空 call 本体（p01 差分）。
    fn callee_base_leading_dot(&self, callee_n: u32) -> bool {
        let mut n = callee_n;
        for _ in 0..10 {
            let kind = self.cx.tree.kind(n);
            if kind == self.cx.k.postfix_expr || kind == KIND_POSTFIX {
                let (base, steps) = self.split_postfix(n);
                if !steps.is_empty() {
                    return false;
                }
                n = base;
                continue;
            }
            if kind == KIND_CALL {
                match self
                    .cx
                    .children(n)
                    .into_iter()
                    .find(|&c| self.cx.tree.kind(c) != self.cx.k.args)
                {
                    Some(c) => {
                        n = c;
                        continue;
                    }
                    None => return false,
                }
            }
            if kind == self.cx.k.atom_expr {
                return self
                    .cx
                    .child_of_kind(n, self.cx.k.leading_dot.unwrap_or(u32::MAX))
                    .is_some();
            }
            match self
                .cx
                .children(n)
                .into_iter()
                .find(|&c| self.cx.tree.child_count(c) > 0)
            {
                Some(c) => n = c,
                None => return false,
            }
        }
        false
    }

    /// 栈顶向下首个标记帧：OpPost/UnaryPost = 未括号待归约；ArgBuild/
    /// CtorFieldBuild = 括号内（实参/构造字段——旷新行为空白）。
    fn pending_unbracketed_op(&self) -> bool {
        for f in self.stack.iter().rev() {
            match f {
                Frame::OpPost { .. } | Frame::UnaryPost { .. } => return true,
                Frame::ArgBuild { .. } | Frame::CtorFieldBuild { .. } => return false,
                _ => {}
            }
        }
        false
    }

    /// PLAN-928（T-07）：callee 无 ident 叶（括号组/λ 值位被调）→ None。
    fn first_ident_opt(&self, node: u32) -> Option<Ident> {
        // 复用 first_ident 的下降逻辑但返回 Option。
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            if self
                .cx
                .p
                .rules
                .iter()
                .position(|r| r.name == "qualified-name")
                .is_some_and(|r| {
                    self.cx.tree.kind(n) == ace_syntax_runtime::program::RULE_KIND_BASE + r as u32
                })
            {
                let a = self.cx.field_child(n, "module")?;
                let b = self.cx.field_child(n, "member")?;
                return Some(Ident {
                    name: format!("{}.{}", self.cx.spelling(a), self.cx.spelling(b)),
                    span: self.cx.span_of(a).join(self.cx.span_of(b)),
                });
            }
            if self.cx.tree.kind(n) == self.cx.k.tok_ident {
                let span = self.cx.span_of(n);
                let name = self.cx.spelling(n);
                return Some(Ident { name, span });
            }
            let kids = self.cx.children(n);
            for c in kids.into_iter().rev() {
                stack.push(c);
            }
        }
        None
    }

    fn describe_node(&self, node: u32) -> String {
        let mut n = node;
        loop {
            if self.cx.tree.node(n).child_count == 0 {
                break;
            }
            let kids = self.cx.children(n);
            match kids
                .iter()
                .find(|&&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
            {
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
            self.cx.k.op_plus,
            self.cx.k.op_minus,
            self.cx.k.op_star,
            self.cx.k.op_slash,
            self.cx.k.op_percent,
            self.cx.k.op_lt,
            self.cx.k.op_gt,
            self.cx.k.op_le,
            self.cx.k.op_ge,
            self.cx.k.op_eq,
            self.cx.k.op_ne,
            self.cx.k.op_andand,
            self.cx.k.op_oror,
        ]
        .contains(&kind)
    }

    fn op_of_token(&self, kind: u32) -> BinOp {
        if kind == self.cx.k.op_plus {
            BinOp::Add
        } else if kind == self.cx.k.op_minus {
            BinOp::Sub
        } else if kind == self.cx.k.op_star {
            BinOp::Mul
        } else if kind == self.cx.k.op_slash {
            BinOp::Div
        } else if kind == self.cx.k.op_percent {
            BinOp::Rem
        } else if kind == self.cx.k.op_lt {
            BinOp::Lt
        } else if kind == self.cx.k.op_gt {
            BinOp::Gt
        } else if kind == self.cx.k.op_le {
            BinOp::Le
        } else if kind == self.cx.k.op_ge {
            BinOp::Ge
        } else if kind == self.cx.k.op_eq {
            BinOp::Eq
        } else if kind == self.cx.k.op_ne {
            BinOp::Ne
        } else if kind == self.cx.k.op_andand {
            BinOp::And
        } else {
            BinOp::Or
        }
    }

    // ------------------------------------------------------------ step

    fn step(&mut self, frame: Frame) {
        match frame {
            Frame::Block(bf) => self.step_block(bf),
            Frame::ForInBody { var, iter, node } => {
                let body = self.pop_b();
                let kw = self
                    .cx
                    .child_of_kind(node, self.cx.k.tok_for)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                self.vals.push(Val::S(Stmt::ForIn {
                    var,
                    iter,
                    body,
                    span,
                }));
            }
            Frame::IsArms(ctx) => {
                if let Some(&arm_n) = ctx.remaining.first() {
                    let mut rest = ctx.remaining.clone();
                    rest.remove(0);
                    let qual = self
                        .cx
                        .field_child(arm_n, "qual")
                        .and_then(|q| self.cx.field_child(q, "qual"))
                        .map(|q| ident_of(&mut self.cx, q));
                    let pattern_leaf = self.cx.field_child(arm_n, "pattern").expect("arm pattern");
                    let pattern = ident_of(&mut self.cx, pattern_leaf);
                    let binding = self
                        .cx
                        .field_child(arm_n, "payload")
                        .and_then(|b| self.cx.field_child(b, "binding"))
                        .map(|b| ident_of(&mut self.cx, b));
                    let body_n = self.cx.field_child(arm_n, "body").expect("arm body");
                    // 旧合同：臂 span 恒从模式词起（限定 `Enum.Name` 亦然）。
                    let start = pattern.span;
                    self.stack.push(Frame::IsArmBlock {
                        ctx: Box::new(IsMatchCtx {
                            remaining: rest,
                            ..ctx
                        }),
                        qual,
                        pattern,
                        binding,
                        kw: start,
                    });
                    self.enter_block(body_n);
                } else {
                    let span = self.cx.span_of(ctx.node);
                    self.vals.push(Val::S(Stmt::IsMatch {
                        scrutinee: ctx.scrutinee,
                        arms: ctx.arms,
                        span,
                    }));
                }
            }
            Frame::IsArmBlock {
                ctx,
                qual,
                pattern,
                binding,
                kw,
            } => {
                let body = self.pop_b();
                let span = kw.join(body.span);
                let mut ctx = *ctx;
                ctx.arms.push(MatchArm {
                    qual,
                    pattern,
                    binding,
                    body,
                    span,
                });
                self.stack.push(Frame::IsArms(ctx));
            }
            Frame::LambdaBlockBody {
                move_capture,
                params,
                start,
            } => {
                let body = self.pop_b();
                let span = start.join(body.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Closure {
                        move_capture,
                        params,
                        body: Box::new(ClosureBody::Block(body)),
                    },
                }));
            }
            Frame::LambdaExprBody {
                move_capture,
                params,
                start,
            } => {
                let value = self.pop_e();
                let span = start.join(value.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Closure {
                        move_capture,
                        params,
                        body: Box::new(ClosureBody::Expr(value)),
                    },
                }));
            }
            Frame::LetFinish {
                node,
                kw,
                name,
                ann,
                mutable,
            } => {
                let value = self.pop_e();
                let span = Span {
                    start: self.cx.span_of(node).start,
                    end: value.span.end,
                };
                self.vals.push(Val::S(Stmt::Let {
                    kw,
                    name,
                    ann,
                    mutable,
                    value,
                    span,
                }));
            }
            Frame::AssignFinish { name, path } => {
                let value = self.pop_e();
                let span = Span {
                    start: name.span.start,
                    end: value.span.end,
                };
                self.vals.push(Val::S(Stmt::Assign {
                    target: AssignTarget { base: name, path },
                    value,
                    span,
                }));
            }
            Frame::ReturnFinish { kw } => {
                let value = self.pop_e();
                let span = kw.join(value.span);
                self.vals.push(Val::S(Stmt::Return { value, span }));
            }
            Frame::IfCond {
                then_n,
                els_blk,
                node,
            } => {
                let cond = self.pop_e();
                self.stack.push(Frame::IfThen {
                    cond,
                    els_blk,
                    node,
                });
                self.enter_block(then_n);
            }
            Frame::IfThen {
                cond,
                els_blk,
                node,
            } => {
                let then = self.pop_b();
                match els_blk {
                    Some(eb) => {
                        self.stack.push(Frame::IfElse { cond, then, node });
                        self.enter_block(eb);
                    }
                    None => {
                        let kw = self
                            .cx
                            .child_of_kind(node, self.cx.k.tok_if)
                            .map(|l| self.cx.span_of(l))
                            .unwrap_or(Span { start: 0, end: 0 });
                        let span = kw.join(then.span);
                        self.vals.push(Val::S(Stmt::If {
                            cond,
                            then,
                            els: None,
                            span,
                        }));
                    }
                }
            }
            Frame::IfElse { cond, then, node } => {
                let els = self.pop_b();
                let kw = self
                    .cx
                    .child_of_kind(node, self.cx.k.tok_if)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(then.span).join(els.span);
                self.vals.push(Val::S(Stmt::If {
                    cond,
                    then,
                    els: Some(els),
                    span,
                }));
            }
            Frame::WhileCond { body_n, node } => {
                let cond = self.pop_e();
                self.stack.push(Frame::WhileBody { cond, node });
                self.enter_block(body_n);
            }
            Frame::WhileBody { cond, node } => {
                let body = self.pop_b();
                let kw_kind = if self.cx.tree.kind(node) == self.cx.k.while_stmt.unwrap_or(u32::MAX)
                {
                    self.cx.k.tok_while
                } else {
                    self.cx.k.tok_for
                };
                let kw = self
                    .cx
                    .child_of_kind(node, kw_kind)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                // 条件 for 复用 While AST 形状（旧合同）。
                self.vals.push(Val::S(Stmt::While { cond, body, span }));
            }
            Frame::LoopBody { node } => {
                let body = self.pop_b();
                let kw = self
                    .cx
                    .child_of_kind(node, self.cx.k.tok_loop)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                self.vals.push(Val::S(Stmt::Loop { body, span }));
            }
            Frame::ExprStmtFinish => {
                let value = self.pop_e();
                let span = value.span;
                self.vals.push(Val::S(Stmt::ExprStmt { value, span }));
            }
            Frame::FnBodyFinish {
                node,
                name,
                type_params,
                params,
                ret,
                receiver,
                owner,
            } => {
                let body = self.pop_b();
                let span = self.cx.span_of(node);
                self.vals.push(Val::F(FnItem {
                    span,
                    name,
                    type_params,
                    params,
                    ret,
                    body,
                    receiver,
                    owner,
                    script: false,
                }));
            }
            Frame::ScriptFinish { start_span } => {
                let mut body = self.pop_b();
                let span = start_span.join(body.span);
                body.span = span;
                self.vals.push(Val::F(FnItem {
                    name: Ident {
                        name: "main".to_string(),
                        span: start_span,
                    },
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret: TypeAnn {
                        kind: TypeRef::Builtin(TypeWord::Void),
                        span: start_span,
                    },
                    body,
                    span,
                    receiver: Receiver::Free,
                    owner: None,
                    script: true,
                }));
            }
            Frame::ExprEnter(n) => self.enter_expr(n),
            Frame::HashExpr { span } => {
                let inner = self.pop_e();
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::HashExpr {
                        expr: Box::new(inner),
                    },
                }));
            }
            Frame::OpPost { op } => {
                let rhs = self.pop_e();
                let lhs = self.pop_e();
                let span = lhs.span.join(rhs.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                }));
            }
            Frame::UnaryPost { op } => {
                let operand = self.pop_e();
                let span = operand.span;
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Unary {
                        op,
                        operand: Box::new(operand),
                    },
                }));
            }
            Frame::PostfixFoldAll { steps } => {
                let recv = self.pop_e();
                let e = self.fold_postfix(recv, steps);
                self.vals.push(Val::E(e));
            }
            Frame::CallAssemble {
                callee,
                argc,
                close_end,
            } => {
                let e = self.assemble_call(callee, argc, close_end);
                self.vals.push(Val::E(e));
            }
            Frame::ArgBuild { named } => {
                let value = self.pop_e();
                let arg = match named {
                    Some(name) => Arg::Named {
                        span: name.span.join(value.span),
                        name,
                        value,
                    },
                    None => Arg::Pos(value),
                };
                self.vals.push(Val::A(arg));
            }
            Frame::CtorFieldBuild { label } => {
                let value = self.pop_e();
                let span = label.span.join(value.span);
                self.vals.push(Val::CF(NamedCtorField {
                    name: label,
                    value,
                    span,
                }));
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
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::CtorNamed { ty, fields },
                }));
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
                    span: Span {
                        start: ident.span.start,
                        end: close_end,
                    },
                    kind: ExprKind::Call {
                        callee: ident,
                        args,
                    },
                };
                for op in unary_ops.iter().rev() {
                    let sp = e.span;
                    e = Expr {
                        span: sp,
                        kind: ExprKind::Unary {
                            op: *op,
                            operand: Box::new(e),
                        },
                    };
                }
                e
            }
            CalleePlan::Value { unary_ops, .. } => {
                let callee = self.pop_e();
                let mut e = Expr {
                    span: Span {
                        start: callee.span.start,
                        end: close_end,
                    },
                    kind: ExprKind::CallValue {
                        callee: Box::new(callee),
                        args,
                    },
                };
                for op in unary_ops.iter().rev() {
                    let sp = e.span;
                    e = Expr {
                        span: sp,
                        kind: ExprKind::Unary {
                            op: *op,
                            operand: Box::new(e),
                        },
                    };
                }
                e
            }
            CalleePlan::Method {
                member,
                mids: _,
                unary_ops,
                ..
            } => {
                let mut recv = self.pop_e();
                // PLAN-928（T-03）：`List<T>.m(...)`——泛型接收原子投影为
                // Field{GenericType}，此处解包为直接 GenericType 接收者
                // （镜像旧 postfix 折叠的 MethodCall 形状）。
                {
                    let want = self.cx.spelling(member).to_string();
                    let unwrapped = match &recv.kind {
                        ExprKind::Field { base, name }
                            if matches!(base.kind, ExprKind::GenericType { .. })
                                && name.name == want =>
                        {
                            Some(*base.clone())
                        }
                        _ => None,
                    };
                    if let Some(r) = unwrapped {
                        recv = r;
                    }
                }
                let mk = self.cx.tree.kind(member);
                let mspan = self.cx.span_of(member);
                let mname = self.cx.spelling(member);
                // type-level 接收者：char.from_codepoint（text 面）。
                let recv = if is_text && mname == "from_codepoint" {
                    let mut r = recv;
                    if let ExprKind::Var(id) = &r.kind {
                        if id.name == "char" {
                            r = Expr {
                                span: id.span,
                                kind: ExprKind::TypeWordArg(TypeWord::Char),
                            };
                        }
                    }
                    r
                } else {
                    recv
                };
                let full = Span {
                    start: recv.span.start,
                    end: close_end,
                };
                let mut e = if mk == self.cx.k.tok_mut {
                    self.cx.reject(mspan, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`.mut` applies to a local variable",
                            mspan,
                        )
                    });
                    Expr {
                        span: full,
                        kind: ExprKind::MutArg {
                            name: Ident {
                                name: mname,
                                span: mspan,
                            },
                        },
                    }
                } else if is_record {
                    Expr {
                        span: full,
                        kind: ExprKind::MethodCall {
                            recv: Box::new(recv),
                            name: Ident {
                                name: mname,
                                span: mspan,
                            },
                            args,
                        },
                    }
                } else {
                    Expr {
                        span: full,
                        kind: ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident {
                                name: mname,
                                span: mspan,
                            },
                            args,
                        },
                    }
                };
                for op in unary_ops.iter().rev() {
                    let sp = e.span;
                    e = Expr {
                        span: sp,
                        kind: ExprKind::Unary {
                            op: *op,
                            operand: Box::new(e),
                        },
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
    let tail_kind =
        cx.p.rules
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
