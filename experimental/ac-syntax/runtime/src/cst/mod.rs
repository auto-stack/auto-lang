//! 无损 CST：arena 节点 + 声明单元切分 + 相对跨度。
//!
//! - 节点 = POD（kind/start/len/边索引/所属声明），arena `Vec` 派生 Drop 不递归（V-12）；
//! - 顶层声明为首批增量单位：`DeclUnit { origin, node }`，节点 span **相对
//!   声明 origin**——开头插入只改 origin 目录，未变声明子树原样复用；
//! - 叶 = token/trivia/error 字节；missing 零宽、携带期望 kind；
//! - 按文档顺序流式重建必须逐字节等于 snapshot（`assert_lossless`）；
//! - 所有遍历/查询显式栈迭代（深链不爆栈）。

use crate::program::{GrammarProgram, KIND_ERROR, KIND_MISSING};
use crate::snapshot::Snapshot;

pub const NO_FIELD: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CstNode {
    /// token/rule kind 或 KIND_MISSING（expected 存于 `expected`）/KIND_ERROR。
    pub kind: u32,
    /// 相对所属声明 origin 的字节起点（声明节点自身恒为 0；root 亦为 0）。
    pub start: u32,
    pub len: u32,
    pub first_child: u32,
    pub child_count: u32,
    /// MISSING 时的期望 kind；0 = 无。
    pub expected: u32,
    pub flags: u8,
    /// 所属声明索引+1；0 = 根级（root/根级 trivia/error）。
    pub decl: u32,
}

pub const FLAG_MISSING: u8 = 1 << 0;
pub const FLAG_ERROR: u8 = 1 << 1;
pub const FLAG_TRIVIA: u8 = 1 << 2;

#[derive(Clone, Debug, PartialEq)]
pub struct SynDiag {
    pub message: String,
    /// 绝对字节范围。
    pub start: u64,
    pub end: u64,
}

/// 顶层声明增量单位。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeclUnit {
    pub origin: u64,
    pub node: u32,
    /// 子树节点区间下界 [node_lo..=node]（含回溯孤儿；嫁接拷贝用）。
    pub node_lo: u32,
}

#[derive(Clone, Debug)]
pub struct CstTree {
    pub nodes: Vec<CstNode>,
    pub children: Vec<u32>,
    pub child_fields: Vec<u32>,
    pub root: u32,
    pub decls: Vec<DeclUnit>,
    pub diags: Vec<SynDiag>,
    pub total_len: u64,
}

impl CstTree {
    pub fn empty() -> CstTree {
        CstTree {
            nodes: vec![CstNode {
                kind: 0,
                start: 0,
                len: 0,
                first_child: 0,
                child_count: 0,
                expected: 0,
                flags: 0,
                decl: 0,
            }],
            children: Vec::new(),
            child_fields: Vec::new(),
            root: 0,
            decls: Vec::new(),
            diags: Vec::new(),
            total_len: 0,
        }
    }

    #[inline]
    pub fn kind(&self, n: u32) -> u32 {
        self.nodes[n as usize].kind
    }
    #[inline]
    pub fn is_missing(&self, n: u32) -> bool {
        self.nodes[n as usize].flags & FLAG_MISSING != 0
    }
    #[inline]
    pub fn is_error(&self, n: u32) -> bool {
        self.nodes[n as usize].flags & FLAG_ERROR != 0
    }
    #[inline]
    pub fn children(&self, n: u32) -> &[u32] {
        let nd = &self.nodes[n as usize];
        &self.children[nd.first_child as usize..(nd.first_child + nd.child_count) as usize]
    }
    #[inline]
    pub fn child_fields(&self, n: u32) -> &[u32] {
        let nd = &self.nodes[n as usize];
        &self.child_fields[nd.first_child as usize..(nd.first_child + nd.child_count) as usize]
    }
    pub fn field_child(&self, n: u32, field: u32) -> Option<u32> {
        self.child_fields(n)
            .iter()
            .position(|&f| f == field)
            .map(|i| self.children(n)[i])
    }

    fn origin_of(&self, n: u32) -> u64 {
        let d = self.nodes[n as usize].decl;
        if d == 0 {
            0
        } else {
            self.decls[(d - 1) as usize].origin
        }
    }

    /// 绝对字节范围 [start, end)。
    pub fn abs_range(&self, n: u32) -> (u64, u64) {
        let nd = self.nodes[n as usize];
        let base = self.origin_of(n);
        (base + nd.start as u64, base + nd.start as u64 + nd.len as u64)
    }

    pub fn has_errors(&self) -> bool {
        !self.diags.is_empty()
    }

    /// 树内是否含 error/missing 节点（strict 投影拒绝面）。
    pub fn has_error_nodes(&self) -> bool {
        self.nodes.iter().any(|n| n.flags & (FLAG_ERROR | FLAG_MISSING) != 0)
    }

    /// 叶序（文档顺序）迭代：显式栈，无递归；构造保证孩子按位置有序。
    pub fn for_each_leaf<F: FnMut(u64, u64, u32, bool)>(&self, mut f: F) {
        let mut stack = vec![self.root];
        while let Some(n) = stack.pop() {
            let nd = self.nodes[n as usize];
            if nd.child_count == 0 {
                let base = self.origin_of(n);
                f(base + nd.start as u64, base + nd.start as u64 + nd.len as u64, nd.kind, nd.flags & FLAG_ERROR != 0);
            } else {
                for &c in self.children(n).iter().rev() {
                    stack.push(c);
                }
            }
        }
    }

    /// 无损校验：叶覆盖必须逐段连续且等于 snapshot 全文。
    pub fn assert_lossless(&self, snap: &Snapshot) -> Result<(), String> {
        let mut expect = 0u64;
        let mut bad: Option<String> = None;
        self.for_each_leaf(|s, e, _k, _err| {
            if bad.is_none() {
                if s != expect {
                    bad = Some(format!("leaf gap/overlap at {s}: expected continuation at {expect}"));
                } else if e < s {
                    bad = Some(format!("bad leaf span at {s}"));
                } else {
                    expect = e;
                }
            }
        });
        if let Some(b) = bad {
            return Err(b);
        }
        if expect != snap.len() {
            return Err(format!("coverage ends at {expect} != snapshot len {}", snap.len()));
        }
        Ok(())
    }

    /// 规范形（增量 vs fresh 等价比较：忽略分配 ID，比较 kind/结构/span/field/标记）。
    /// 迭代实现（V-12 深树）。
    pub fn canonical(&self) -> String {
        let mut out = String::with_capacity(self.nodes.len() * 12);
        let mut stack: Vec<(u32, usize)> = vec![(self.root, 0)];
        while let Some((n, d)) = stack.pop() {
            let nd = self.nodes[n as usize];
            let (s, e) = self.abs_range(n);
            let missing = if nd.flags & FLAG_MISSING != 0 { "!" } else { "" };
            let err = if nd.flags & FLAG_ERROR != 0 { "^" } else { "" };
            let triv = if nd.flags & FLAG_TRIVIA != 0 { "~" } else { "" };
            out.push_str(&format!(
                "{:indent$}n{}{missing}{err}{triv}:[{s},{e}) exp={}\n",
                "",
                nd.kind,
                nd.expected,
                indent = d
            ));
            for &c in self.children(n).iter().rev() {
                stack.push((c, d + 1));
            }
        }
        out
    }
}

/// 叶集合收集（token 叶 + trivia 叶 + error 叶，文档序）。
pub fn leaves_in_order(tree: &CstTree) -> Vec<(u64, u64, u32, bool)> {
    let mut v = Vec::new();
    tree.for_each_leaf(|s, e, k, err| v.push((s, e, k, err)));
    v
}

/// 流式重建原文（按文档顺序叶拼接；跨声明用 snapshot 读回）。
pub fn reconstruct(tree: &CstTree, snap: &Snapshot) -> Result<Vec<u8>, crate::program::RtError> {
    let mut out = Vec::with_capacity(snap.len() as usize);
    for (s, e, _k, _err) in leaves_in_order(tree) {
        out.extend_from_slice(&snap.read_bytes(s, e)?);
    }
    Ok(out)
}

pub fn kind_name(p: &GrammarProgram, kind: u32) -> String {
    if kind == KIND_ERROR {
        "ERROR".into()
    } else if kind == KIND_MISSING {
        "MISSING".into()
    } else if (crate::program::TOKEN_KIND_BASE..crate::program::RULE_KIND_BASE).contains(&kind) {
        p.token_name(kind - crate::program::TOKEN_KIND_BASE).into()
    } else {
        p.rule_name(kind - crate::program::RULE_KIND_BASE).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cst_node_is_pod() {
        // POD 数组 → Vec drop 非递归；无 Box/String 字段。
        assert_eq!(std::mem::size_of::<CstNode>(), 4 * 7 + 1 + 3); // 7×u32 + u8 + pad
        assert_eq!(std::mem::align_of::<CstNode>(), 4);
    }
}
