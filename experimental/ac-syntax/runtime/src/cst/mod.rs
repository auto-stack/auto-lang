//! 无损 CST：不可变共享内容块 + 长度目录 + 声明单元切分 + 相对跨度。
//!
//! - 内容块（`CstBlock`）= 节点/边 POD 数组，seal 后只读、跨树以 `Rc` **真实
//!   共享**（F787-R1-04：增量编辑对未变声明不做逐节点复制——新树直接引用
//!   旧块，长度目录 `bases` 给出块起始全局 id，全局 id = `bases[b] + 块内下标`）；
//! - 节点 = POD；块数组派生 Drop 不递归（V-12）；
//! - 边（`edges`/`fields`）是**块内局部下标**——块可在任意树中重定位；
//!   根的孩子跨块，放树级 `root_edges` 目录（与持久化 manifest 同构）；
//! - 顶层声明为首批增量单位：`DeclUnit { origin, node }`，节点 span 相对
//!   声明 origin——开头插入只改 origin 目录，未变声明子树原样共享；
//!   块内节点以 `decl`（块内声明序号+1）标记归属，树级 `block_decl` 把
//!   （块， 序号）映射回 `decls`——共享块的节点归属不经重写；
//! - 叶 = token/trivia/error 字节；missing 零宽、携带期望 kind；
//! - 按文档顺序流式重建必须逐字节等于 snapshot（`assert_lossless`）；
//! - 所有遍历/查询显式栈迭代（深链不爆栈）。

use crate::program::{GrammarProgram, KIND_ERROR, KIND_MISSING};
use crate::snapshot::Snapshot;
use std::rc::Rc;

pub const NO_FIELD: u32 = u32::MAX;

/// 内容块内节点数上限（seal 阈值；仅在声明边界封块——声明子树永不跨块）。
pub const CST_BLOCK_NODES: usize = 4096;

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
    /// 所属**块内**声明序号+1；0 = 根级（root/根级 trivia/error，绝对跨度）。
    pub decl: u32,
}

pub const FLAG_MISSING: u8 = 1 << 0;
pub const FLAG_ERROR: u8 = 1 << 1;
pub const FLAG_TRIVIA: u8 = 1 << 2;

/// 不可变内容块：节点 + 边（块内局部下标）+ 字段。seal 后只读、可跨树共享。
#[derive(Debug)]
pub struct CstBlock {
    pub nodes: Vec<CstNode>,
    /// 孩子（块内局部下标），与 `fields` 平行。
    pub edges: Vec<u32>,
    pub fields: Vec<u32>,
}

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
    /// 声明根节点全局 id（树局部坐标）。
    pub node: u32,
    /// 子树节点区间下界 [node_lo..=node]（含回溯孤儿；全局 id）。
    pub node_lo: u32,
}

/// 树 = 内容块目录 + 长度目录 + 根孩子目录 + 声明目录。
#[derive(Clone, Debug)]
pub struct CstTree {
    pub blocks: Vec<Rc<CstBlock>>,
    /// 长度目录：块 b 的全局 id 基址（升序）；全局 id = bases[b] + 块内下标。
    pub bases: Vec<u32>,
    /// 根节点全局 id（根自身边置零，孩子走 `root_edges` 目录）。
    pub root: u32,
    /// 根孩子目录：(全局 id, field)，文档序。
    pub root_edges: Vec<(u32, u32)>,
    pub decls: Vec<DeclUnit>,
    /// 块 → 块内声明序号 → `decls` 下标（u32::MAX = 该声明未被本树嫁接）。
    pub block_decl: Vec<Vec<u32>>,
    pub diags: Vec<SynDiag>,
    pub total_len: u64,
}

/// 未嫁接的块内声明槽位标记。
pub const UNGRAFTED_DECL: u32 = u32::MAX;

impl CstTree {
    pub fn empty() -> CstTree {
        CstTree {
            blocks: vec![Rc::new(CstBlock {
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
                edges: Vec::new(),
                fields: Vec::new(),
            })],
            bases: vec![0],
            root: 0,
            root_edges: Vec::new(),
            decls: Vec::new(),
            block_decl: vec![Vec::new()],
            diags: Vec::new(),
            total_len: 0,
        }
    }

    /// 全局 id → (块下标, 块内下标)。
    pub fn block_and_local(&self, g: u32) -> (u32, u32) {
        let b = self.bases.partition_point(|&base| base <= g) - 1;
        (b as u32, g - self.bases[b])
    }

    /// 声明根所在块下标（树局部坐标）。
    pub fn block_of(&self, decl_node: u32) -> u32 {
        self.block_and_local(decl_node).0
    }

    pub fn node(&self, g: u32) -> &CstNode {
        let (b, l) = self.block_and_local(g);
        &self.blocks[b as usize].nodes[l as usize]
    }

    pub fn node_total(&self) -> u32 {
        self.bases.last().map(|&b| b).unwrap_or(0)
            + self
                .blocks
                .last()
                .map(|blk| blk.nodes.len() as u32)
                .unwrap_or(0)
    }

    #[inline]
    pub fn kind(&self, n: u32) -> u32 {
        self.node(n).kind
    }
    #[inline]
    pub fn is_missing(&self, n: u32) -> bool {
        self.node(n).flags & FLAG_MISSING != 0
    }
    #[inline]
    pub fn is_error(&self, n: u32) -> bool {
        self.node(n).flags & FLAG_ERROR != 0
    }

    pub fn child_count(&self, n: u32) -> u32 {
        if n == self.root && !self.root_edges.is_empty() {
            self.root_edges.len() as u32
        } else {
            self.node(n).child_count
        }
    }

    /// 孩子遍历（零分配）：f(全局 id, field)，文档序。
    pub fn for_each_child<F: FnMut(u32, u32)>(&self, n: u32, mut f: F) {
        if n == self.root && !self.root_edges.is_empty() {
            for &(c, field) in &self.root_edges {
                f(c, field);
            }
            return;
        }
        let (b, l) = self.block_and_local(n);
        let blk = &self.blocks[b as usize];
        let nd = &blk.nodes[l as usize];
        let fc = nd.first_child as usize;
        let base = self.bases[b as usize];
        for k in 0..nd.child_count as usize {
            // 边存块内局部下标——对外统一换算为全局 id（块可在任意树重定位）。
            f(base + blk.edges[fc + k], blk.fields[fc + k]);
        }
    }

    /// 孩子列表（全局 id；分配版本——外部消费者便利面）。
    pub fn children(&self, n: u32) -> Vec<u32> {
        let mut out = Vec::new();
        self.for_each_child(n, |c, _| out.push(c));
        out
    }

    pub fn child_fields(&self, n: u32) -> Vec<u32> {
        let mut out = Vec::new();
        self.for_each_child(n, |_, f| out.push(f));
        out
    }

    pub fn field_child(&self, n: u32, field: u32) -> Option<u32> {
        let mut hit = None;
        self.for_each_child(n, |c, f| {
            if f == field && hit.is_none() {
                hit = Some(c);
            }
        });
        hit
    }

    fn origin_of(&self, n: u32) -> u64 {
        let (b, l) = self.block_and_local(n);
        let nd = &self.blocks[b as usize].nodes[l as usize];
        if nd.decl == 0 {
            return 0;
        }
        let didx = self.block_decl[b as usize][(nd.decl - 1) as usize];
        if didx == UNGRAFTED_DECL {
            // 未嫁接声明的节点：本树目录不可达（防御——不应被遍历到）。
            return 0;
        }
        self.decls[didx as usize].origin
    }

    /// 绝对字节范围 [start, end)。
    pub fn abs_range(&self, n: u32) -> (u64, u64) {
        let nd = self.node(n);
        let base = self.origin_of(n);
        (
            base + nd.start as u64,
            base + nd.start as u64 + nd.len as u64,
        )
    }

    pub fn has_errors(&self) -> bool {
        !self.diags.is_empty()
    }

    /// 树内是否含 error/missing 节点（strict 投影拒绝面）。
    pub fn has_error_nodes(&self) -> bool {
        self.blocks.iter().any(|b| {
            b.nodes
                .iter()
                .any(|n| n.flags & (FLAG_ERROR | FLAG_MISSING) != 0)
        })
    }

    /// 叶序（文档顺序）迭代：显式栈，无递归；构造保证孩子按位置有序。
    pub fn for_each_leaf<F: FnMut(u64, u64, u32, bool)>(&self, mut f: F) {
        let mut stack = vec![self.root];
        while let Some(n) = stack.pop() {
            let nd = self.node(n);
            if self.child_count(n) == 0 {
                let base = self.origin_of(n);
                f(
                    base + nd.start as u64,
                    base + nd.start as u64 + nd.len as u64,
                    nd.kind,
                    nd.flags & FLAG_ERROR != 0,
                );
            } else {
                let mut kids = Vec::new();
                self.for_each_child(n, |c, _| kids.push(c));
                for c in kids.into_iter().rev() {
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
                    bad = Some(format!(
                        "leaf gap/overlap at {s}: expected continuation at {expect}"
                    ));
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
            return Err(format!(
                "coverage ends at {expect} != snapshot len {}",
                snap.len()
            ));
        }
        Ok(())
    }

    /// 规范形（增量 vs fresh 等价比较：忽略分配 ID，比较 kind/结构/span/field/标记）。
    /// 迭代实现（V-12 深树）。
    pub fn canonical(&self) -> String {
        let mut out = String::with_capacity(self.node_total() as usize * 12);
        let mut stack: Vec<(u32, usize)> = vec![(self.root, 0)];
        while let Some((n, d)) = stack.pop() {
            let nd = self.node(n);
            let (s, e) = self.abs_range(n);
            let missing = if nd.flags & FLAG_MISSING != 0 {
                "!"
            } else {
                ""
            };
            let err = if nd.flags & FLAG_ERROR != 0 { "^" } else { "" };
            let triv = if nd.flags & FLAG_TRIVIA != 0 { "~" } else { "" };
            out.push_str(&format!(
                "{:indent$}n{}{missing}{err}{triv}:[{s},{e}) exp={}\n",
                "",
                nd.kind,
                nd.expected,
                indent = d
            ));
            let mut kids = Vec::new();
            self.for_each_child(n, |c, _| kids.push(c));
            for c in kids.into_iter().rev() {
                stack.push((c, d + 1));
            }
        }
        out
    }

    /// 与他树按块指针共享的节点数（F787-R1-04 复用计量：内容块身份）。
    pub fn shared_nodes_with(&self, old: &CstTree) -> u64 {
        let mut shared: u64 = 0;
        for blk in &self.blocks {
            if old.blocks.iter().any(|o| Rc::ptr_eq(blk, o)) {
                shared += blk.nodes.len() as u64;
            }
        }
        shared
    }

    /// 本树新分配的节点数（非共享块节点总和 = 实际复制/新建量）。
    pub fn fresh_nodes_vs(&self, old: &CstTree) -> u64 {
        self.node_total() as u64 - self.shared_nodes_with(old)
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
        // POD 数组 → 块 Drop 非递归；无 Box/String 字段。
        assert_eq!(std::mem::size_of::<CstNode>(), 4 * 7 + 1 + 3); // 7×u32 + u8 + pad
        assert_eq!(std::mem::align_of::<CstNode>(), 4);
    }

    #[test]
    fn empty_tree_identity() {
        let t = CstTree::empty();
        assert_eq!(t.node_total(), 1);
        assert_eq!(t.root, 0);
        assert!(t.root_edges.is_empty());
        assert_eq!(t.child_count(0), 0);
    }
}
