//! 句柄与代际：tree/grammar/snapshot 身份绑定 + lease 生命周期。
//!
//! - `NodeHandle { tree, node }`：tree = 会话内树实例 id（绑定 snapshot+grammar+revision）；
//! - 跨树使用受控拒绝（RtError::BadHandle）；旧树在显式 lease 内可读；
//! - lease 释放后旧树可淘汰；当前根永可达；
//! - "旧 handle 不能用于新代查询"：非当前且未 lease 的树 → 拒绝，不回落。

use crate::cst::CstTree;
use crate::identity::GrammarId;
use crate::identity::SnapshotId;
use crate::program::RtError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

/// 树实例 id 全局发生器（F787-R2-07：跨会话不可混淆——每会话从 1 起号会
/// 让 A 会话的 handle 在 B 会话解析成 B 的树）。
static TREE_ID_GEN: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NodeHandle {
    pub tree: u64,
    pub node: u32,
}

#[derive(Clone, Debug)]
pub struct TreeMeta {
    pub snapshot_id: SnapshotId,
    pub grammar_id: GrammarId,
    pub revision: u32,
}

pub struct TreeEntry {
    pub tree: CstTree,
    pub meta: TreeMeta,
}

struct SessionInner {
    entries: HashMap<u64, Rc<TreeEntry>>,
    lease_counts: HashMap<u64, u32>,
    current: Option<u64>,
}

/// 编辑会话的树注册表：当前根 + lease 的旧树。
pub struct TreeSession {
    inner: RefCell<SessionInner>,
}

impl Default for TreeSession {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeSession {
    pub fn new() -> Self {
        TreeSession {
            inner: RefCell::new(SessionInner {
                entries: HashMap::new(),
                lease_counts: HashMap::new(),
                current: None,
            }),
        }
    }

    /// 发布新当前根；未 lease 的旧树淘汰（可淘汰 ≠ 立即释放：Rc 语义）。
    /// id 取自全局发生器——同一进程内任意两会话的树 id 互不相同。
    pub fn publish(&self, entry: TreeEntry) -> u64 {
        let mut s = self.inner.borrow_mut();
        let id = TREE_ID_GEN.fetch_add(1, Ordering::SeqCst);
        let keep: Vec<u64> = s
            .lease_counts
            .iter()
            .filter(|(_, &c)| c > 0)
            .map(|(k, _)| *k)
            .collect();
        s.entries.retain(|k, _| *k == id || keep.contains(k));
        s.entries.insert(id, Rc::new(entry));
        s.current = Some(id);
        id
    }

    pub fn current_tree_id(&self) -> Option<u64> {
        self.inner.borrow().current
    }

    /// 旧树显式 lease（当前树也可 lease）。
    pub fn lease(&self, tree: u64) -> Result<TreeLease<'_>, RtError> {
        let mut s = self.inner.borrow_mut();
        if !s.entries.contains_key(&tree) {
            return Err(RtError::BadHandle(format!(
                "tree {tree} not retained (evicted or unknown)"
            )));
        }
        *s.lease_counts.entry(tree).or_insert(0) += 1;
        Ok(TreeLease {
            session: self,
            tree,
        })
    }

    pub fn is_live(&self, tree: u64) -> bool {
        let s = self.inner.borrow();
        s.current == Some(tree)
            || (s.entries.contains_key(&tree)
                && s.lease_counts.get(&tree).copied().unwrap_or(0) > 0)
    }

    /// 句柄解析：当前或 lease 内可读；否则受控拒绝。
    pub fn resolve(&self, h: NodeHandle) -> Result<Rc<TreeEntry>, RtError> {
        let s = self.inner.borrow();
        if !s.entries.contains_key(&h.tree) {
            return Err(RtError::BadHandle(format!(
                "handle references tree {} of an evicted generation",
                h.tree
            )));
        }
        let is_current = s.current == Some(h.tree);
        let leased = s.lease_counts.get(&h.tree).copied().unwrap_or(0) > 0;
        if !is_current && !leased {
            return Err(RtError::BadHandle(format!(
                "handle references old generation tree {} without an active lease",
                h.tree
            )));
        }
        let entry = s.entries[&h.tree].clone();
        let node_count = entry.tree.node_total();
        if h.node >= node_count {
            return Err(RtError::BadHandle(format!(
                "node index {} out of range (tree has {node_count})",
                h.node
            )));
        }
        Ok(entry)
    }
}

/// RAII lease：drop 时计数归零，树可被下一次 publish 淘汰。
pub struct TreeLease<'s> {
    session: &'s TreeSession,
    tree: u64,
}

impl<'s> Drop for TreeLease<'s> {
    fn drop(&mut self) {
        let mut s = self.session.inner.borrow_mut();
        if let Some(c) = s.lease_counts.get_mut(&self.tree) {
            *c = c.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> TreeEntry {
        TreeEntry {
            tree: CstTree::empty(),
            meta: TreeMeta {
                snapshot_id: crate::identity::SnapshotId("s".into()),
                grammar_id: crate::identity::GrammarId("g".into()),
                revision: 0,
            },
        }
    }

    #[test]
    fn generation_cross_tree_rejected_without_lease() {
        let s = TreeSession::new();
        let t1 = s.publish(entry());
        let h = NodeHandle { tree: t1, node: 0 };
        assert!(s.resolve(h).is_ok(), "current tree readable");
        let t2 = s.publish(entry());
        assert!(
            matches!(s.resolve(h), Err(RtError::BadHandle(_))),
            "old handle must be rejected"
        );
        let _ = t2;
    }

    #[test]
    fn foreign_session_handle_is_rejected_both_ways() {
        // F787-R2-07 反控：两会话各自 publish 后互用对方 handle 必须拒绝
        // （树 id 全局唯一，跨会话 id 在本会话不存在）。
        let a = TreeSession::new();
        let b = TreeSession::new();
        let ida = a.publish(entry());
        let idb = b.publish(entry());
        assert!(
            b.resolve(NodeHandle { tree: ida, node: 0 }).is_err(),
            "session A's handle must not resolve in session B"
        );
        assert!(
            a.resolve(NodeHandle { tree: idb, node: 0 }).is_err(),
            "session B's handle must not resolve in session A"
        );
        // 同内容不同文档：两会话各发布同 identity 树，仍互不可解析。
        assert_ne!(ida, idb);
        assert!(
            a.resolve(NodeHandle { tree: ida, node: 0 }).is_ok(),
            "own handle resolves"
        );
    }

    #[test]
    fn lease_keeps_old_tree_readable_until_release() {
        let s = TreeSession::new();
        let t1 = s.publish(entry());
        let lease = s.lease(t1).unwrap();
        s.publish(entry());
        let h = NodeHandle { tree: t1, node: 0 };
        assert!(s.resolve(h).is_ok(), "leased old tree stays readable");
        drop(lease);
        assert!(
            matches!(s.resolve(h), Err(RtError::BadHandle(_))),
            "after release, old tree evicted on next publish"
        );
    }
}
