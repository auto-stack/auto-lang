//! 结构 outline：由 grammar 声明的声明规则/字段驱动的候选抽取。
//!
//! 产物是**语法候选**（名字、参数文本、范围、syntax 状态）——类型含义/
//! 实际导出/ABI 全部 unknown，不作语义授信（Design39 §8 信任分层）。

use crate::cst::{CstTree, FLAG_ERROR, FLAG_MISSING};
use crate::program::{GrammarProgram, RtError};
use crate::snapshot::Snapshot;

#[derive(Clone, Debug, PartialEq)]
pub struct OutlineItem {
    pub name: String,
    /// 参数区原文（含括号与否由 grammar 字段范围决定）。
    pub params_text: Option<String>,
    /// 绝对字节范围。
    pub start: u64,
    pub end: u64,
    /// 声明子树内无 error/missing → true（仍不构成语义信任）。
    pub syntax_ok: bool,
}

/// 抽取 outline 候选（迭代遍历，无递归）。
pub fn outline(
    p: &GrammarProgram,
    tree: &CstTree,
    snap: &Snapshot,
) -> Result<Vec<OutlineItem>, RtError> {
    let Some(spec) = &p.outline else {
        return Ok(Vec::new());
    };
    let name_field = p.field_id(&spec.name_field);
    let params_field = spec.params_field.as_ref().and_then(|f| p.field_id(f));
    let decl_kind = p.rule_kind(
        p.rules
            .iter()
            .position(|r| r.name == spec.decl_rule)
            .ok_or_else(|| {
                RtError::InvalidProgram(format!("outline decl_rule `{}` unknown", spec.decl_rule))
            })? as u32,
    );
    let mut out = Vec::with_capacity(tree.decls.len());
    for d in &tree.decls {
        let node = d.node;
        if tree.kind(node) != decl_kind {
            continue;
        }
        // 字段可能在包裹层（如 decl := api-decl | note-decl 的分派节点）内部：
        // 子树内先序找第一个带该字段的孩子（迭代）。
        let find_field = |field: u32| -> Option<(u64, u64)> {
            let mut stack = vec![node];
            while let Some(n) = stack.pop() {
                if let Some(c) = tree.field_child(n, field) {
                    let (s, e) = tree.abs_range(c);
                    if e > s {
                        return Some((s, e));
                    }
                }
                let kids: Vec<u32> = tree.children(n).to_vec();
                for &c in kids.iter().rev() {
                    stack.push(c);
                }
            }
            None
        };
        let name = match name_field.and_then(|f| find_field(f)) {
            Some((s, e)) => snap.read_str(s, e)?,
            None => String::new(),
        };
        let params_text = match params_field.and_then(|f| find_field(f)) {
            Some((s, e)) => Some(snap.read_str(s, e)?),
            None => None,
        };
        // syntax 状态：子树 error/missing 扫描（显式栈）。
        let mut ok = true;
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            let nd = tree.node(n);
            if nd.flags & (FLAG_ERROR | FLAG_MISSING) != 0 {
                ok = false;
                break;
            }
            for &c in tree.children(n).iter().rev() {
                stack.push(c);
            }
        }
        let (s, e) = tree.abs_range(node);
        out.push(OutlineItem {
            name,
            params_text,
            start: s,
            end: e,
            syntax_ok: ok,
        });
    }
    Ok(out)
}
