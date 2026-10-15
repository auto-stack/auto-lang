//! 增量编辑：可信 checkpoint 重扫 + token splice + 声明级树复用/嫁接。
//!
//! - 词法：从编辑区前最近 at-rest checkpoint 重扫，与旧流在换算位移后对齐
//!   （K=8 连续 token 同 kind+字节）→ 拼接尾部（行号按 Δ 行平移）；
//!   未闭合状态/超窗（64 KiB）不收敛 → 记录原因的全文回退；
//! - 解析：以顶层声明为增量单位——旧树声明不与「编辑区 ∪ 重扫区」相交者
//!   原样嫁接（节点区间 POD 平移拷贝，相对跨度零改写）；相交声明重新解析；
//! - 计量绑定真实读/parse 路径（rescan_bytes / reparsed_decls / reused_decls）；
//! - 迟到结果：`VersionedPublish` 携带目标 revision，旧任务完成只入 late 表，
//!   不覆盖当前根/诊断（V-08 事件屏障复现，不靠 sleep）。

use crate::cst::CstTree;
use crate::lexer::{lex_from, CompiledScanner, LexDiag, ScanCheckpoint, Token};
use crate::parse::parse_tokens;
use crate::program::*;
use crate::snapshot::{Edit, Snapshot};

/// 重扫窗上限（V-06 必验负载的验收界）。
pub const RESCAN_WINDOW: u64 = 64 * 1024;
/// 收敛判定连续匹配 token 数。
pub const CONVERGE_K: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub enum FallbackReason {
    /// checkpoint 前是未闭合（错误）token：at-rest 语义不可用。
    UnterminatedAtCheckpoint,
    /// 重扫超出 64 KiB 窗仍未与旧流对齐。
    NoConvergence,
    /// 多编辑跨声明同步点等复杂形态（保守回退）。
    ComplexEdits,
}

#[derive(Clone, Debug)]
pub struct IncrementalResult {
    pub tree: CstTree,
    pub tokens: Vec<Token>,
    pub lex_diags: Vec<LexDiag>,
    pub rescan_bytes: u64,
    pub reparsed_decls: usize,
    pub reused_decls: usize,
    pub fallback: Option<FallbackReason>,
}

pub struct Splice {
    pub tokens: Vec<Token>,
    /// 对齐区：新 token ≥ new_base 对应旧 token（old_base 起，逐一对齐）。
    pub new_base: usize,
    pub old_base: usize,
    pub rescan_bytes: u64,
}

/// 增量重词法：返回 splice 结果或回退原因。
pub fn incremental_lex(
    p: &GrammarProgram,
    cs: &CompiledScanner,
    old_tokens: &[Token],
    old_snap: &Snapshot,
    new_snap: &Snapshot,
    edits: &[Edit],
) -> Result<Splice, FallbackReason> {
    if edits.is_empty() {
        return Ok(Splice { tokens: old_tokens.to_vec(), new_base: old_tokens.len(), old_base: old_tokens.len(), rescan_bytes: 0 });
    }
    if edits.len() > 1 {
        // 多编辑：保守回退（单编辑负载是本切片验收面；批量编辑走 fresh）。
        return Err(FallbackReason::ComplexEdits);
    }
    let e = &edits[0];
    let delta = e.insert.len() as i64 - (e.end - e.start) as i64;

    // 受影响的首个旧 token：覆盖 [e.start, e.end) 的最早 token；纯插入时
    // 取 start 之后的首 token（前 token 尾部不受影响）。
    let span_edit = e.end > e.start;
    let affected_first = old_tokens
        .iter()
        .position(|t| t.end > e.start && (if span_edit { t.start < e.end } else { t.start <= e.start }))
        .unwrap_or(old_tokens.len());
    // at-rest checkpoint：affected_first 前一 token 边界（非错误 token 之后）。
    let cp_idx = affected_first; // checkpoint 在 token[cp_idx] 起点
    if cp_idx == 0 {
        // 文档起点即 at-rest。
    } else if old_tokens[cp_idx - 1].error {
        return Err(FallbackReason::UnterminatedAtCheckpoint);
    }
    let cp = ScanCheckpoint {
        offset: old_tokens.get(cp_idx).map(|t| t.start).unwrap_or(old_snap.len()),
        mode: 0,
        mid: None,
        line: old_tokens.get(cp_idx).map(|t| t.line).unwrap_or(1),
    };

    // 重扫：新 snapshot 从 cp 起扫描，直至越过编辑新区并连续 K 个对齐。
    let shifted_end = (e.end as i64 + delta) as u64;
    let lx = lex_from(p, cs, new_snap, cp).map_err(|_| FallbackReason::NoConvergence)?;
    let new_head = lx.tokens;
    let rescan_start = cp.offset;
    // 在 new_head 中找对齐点：idx 使得 new_head[idx..idx+K] 与 old_tokens[cp_idx + (idx - offset_adjust) ...]
    // 逐个 kind+spell 相等且位置差恒为 delta。
    // new_head[j] 对应的旧候选：old_tokens 中 start == new_head[j].start - delta 的 token。
    let mut align_new: Option<usize> = None;
    let mut align_old: Option<usize> = None;
    let mut j = 0usize;
    while j < new_head.len() {
        let nt = &new_head[j];
        if nt.start + 1 > shifted_end {
            // 进入编辑区之后：尝试对齐。
            let old_pos = (nt.start as i64 - delta) as u64;
            let oi = old_tokens[cp_idx..]
                .iter()
                .position(|t| t.start == old_pos && t.token == nt.token && t.end - t.start == nt.end - nt.start);
            if let Some(rel) = oi {
                let o = cp_idx + rel;
                // 连续 K 对齐验证。
                let mut k = 0;
                while k < CONVERGE_K {
                    let a = new_head.get(j + k);
                    let b = old_tokens.get(o + k);
                    match (a, b) {
                        (Some(a), Some(b))
                            if a.token == b.token
                                && a.start as i64 - b.start as i64 == delta
                                && a.end - a.start == b.end - b.start =>
                        {
                            k += 1;
                        }
                        _ => break,
                    }
                }
                if k == CONVERGE_K {
                    align_new = Some(j + CONVERGE_K);
                    align_old = Some(o + CONVERGE_K);
                    break;
                }
            }
        }
        // 重扫窗超限。
        if nt.start - rescan_start > RESCAN_WINDOW {
            return Err(FallbackReason::NoConvergence);
        }
        j += 1;
    }
    let (an, ao) = match (align_new, align_old) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            // 到 EOF 都未对齐（删除至尾部等）——整流即新流。
            let nh_len = new_head.len();
            return Ok(Splice {
                rescan_bytes: new_snap.len().saturating_sub(rescan_start),
                tokens: new_head,
                new_base: nh_len,
                old_base: old_tokens.len(),
            });
        }
    };
    // 拼接：head = new_head[..an]；tail = old_tokens[ao..] 平移。
    let head_newlines = {
        // Δ 行 = 重扫区首 token 行 - checkpoint 行，与 tail 平移所需一致地计算：
        // 尾部 token 新行 = 旧行 + (new_head[an-1].line + 旧行差) —— 直接用
        // 对齐点行差：new_head[an-1] 对齐 old_tokens[ao-1]。
        let a = &new_head[an - 1];
        let b = &old_tokens[ao - 1];
        a.line as i64 - b.line as i64
    };
    let mut tokens = Vec::with_capacity(an + old_tokens.len() - ao);
    tokens.extend_from_slice(&new_head[..an]);
    for t in &old_tokens[ao..] {
        tokens.push(Token {
            token: t.token,
            start: (t.start as i64 + delta) as u64,
            end: (t.end as i64 + delta) as u64,
            line: (t.line as i64 + head_newlines) as u32,
            error: t.error,
        });
    }
    let rescan_bytes = new_head.get(an).map(|t| t.start).unwrap_or(new_snap.len()) - rescan_start;
    Ok(Splice { tokens, new_base: an, old_base: ao, rescan_bytes })
}

/// 增量重解析（编辑后）。edits 基于旧 snapshot 坐标。
pub fn incremental_reparse(
    p: &GrammarProgram,
    cs: &CompiledScanner,
    old: &crate::parse::ParseOutput,
    old_snap: &Snapshot,
    new_snap: &Snapshot,
    edits: &[Edit],
    budget: &Budget,
) -> Result<IncrementalResult, RtError> {
    // 1) token splice（失败 → 记录原因的全文 fresh 回退，结果与 fresh 一致）。
    let sp = match incremental_lex(p, cs, &old.tokens, old_snap, new_snap, edits) {
        Ok(sp) => Some(sp),
        Err(reason) => {
            let lx = crate::lexer::lex_all(p, cs, new_snap)?;
            let out = parse_tokens(p, lx.tokens, lx.diags, budget)?;
            let decls = out.tree.decls.len();
            return Ok(IncrementalResult {
                tree: out.tree,
                tokens: out.tokens,
                lex_diags: out.lex_diags,
                rescan_bytes: new_snap.len(),
                reparsed_decls: decls,
                reused_decls: 0,
                fallback: Some(reason),
            });
        }
    };
    let Some(sp) = sp else { unreachable!() };

    // 2) 受影响字节区（旧坐标）：checkpoint..对齐点。
    let first_edit_start = edits.first().map(|e| e.start).unwrap_or(0);
    let cp_off = if sp.old_base < old.tokens.len() {
        old.tokens[sp.old_base].start
    } else {
        old_snap.len()
    };
    let converge_old = if sp.old_base < old.tokens.len() {
        old.tokens[sp.old_base].start
    } else {
        old_snap.len()
    };
    let affected_lo = first_edit_start.min(cp_off);
    let affected_hi = edits.last().map(|e| e.end).unwrap_or(0).max(converge_old);

    // 3) 嫁接候选：不与受影响区相交的旧声明。
    use crate::parse::GraftCandidate;
    use std::collections::HashMap;
    let mut by_old_tok: HashMap<usize, GraftCandidate> = HashMap::new();
    let mut reused = 0usize;
    for d in &old.tree.decls {
        let (s, e) = old.tree.abs_range(d.node);
        if e > affected_lo && s < affected_hi {
            continue; // 相交 → 重解析
        }
        // 声明 token 范围（旧流，含 leading trivia；span 起点即首 trivia）。
        let lo = old.tokens.partition_point(|t| t.start < s);
        let hi = old.tokens.partition_point(|t| t.start < e);
        by_old_tok.insert(lo, GraftCandidate { decl: *d, tok_lo: lo, tok_hi: hi });
        reused += 1;
    }
    let reparsed = old.tree.decls.len().saturating_sub(reused);

    if reused == 0 {
        // 无可复用 → 等价 fresh。
        let out = parse_tokens(p, sp.tokens.clone(), Vec::new(), budget)?;
        let decls = out.tree.decls.len();
        return Ok(IncrementalResult {
            tree: out.tree,
            tokens: out.tokens,
            lex_diags: out.lex_diags,
            rescan_bytes: sp.rescan_bytes,
            reparsed_decls: decls,
            reused_decls: 0,
            fallback: None,
        });
    }

    // 4) 带嫁接的解析：对齐区 token 直接换算旧 token 下标。
    let shifts: Vec<(u64, i64)> = edits.iter().map(|e| (e.end, e.insert.len() as i64 - (e.end - e.start) as i64)).collect();
    let graft = crate::parse::GraftInput {
        old_tree: &old.tree,
        new_base: sp.new_base,
        old_base: sp.old_base,
        candidates: by_old_tok,
        shifts,
    };
    let out = crate::parse::parse_tokens_with_graft(p, sp.tokens.clone(), Vec::new(), budget, Some(graft))?;
    Ok(IncrementalResult {
        tree: out.tree,
        tokens: out.tokens,
        lex_diags: out.lex_diags,
        rescan_bytes: sp.rescan_bytes,
        reparsed_decls: reparsed,
        reused_decls: reused,
        fallback: None,
    })
}

/// 迟到/取消结果的版本化发布（V-08）。
pub struct VersionedPublish<T> {
    pub current_revision: u32,
    published: Option<(u32, T)>,
    late: Vec<(u32, T)>,
    cancelled: Vec<(u32, T)>,
}

impl<T> VersionedPublish<T> {
    pub fn new(current_revision: u32) -> Self {
        VersionedPublish { current_revision, published: None, late: Vec::new(), cancelled: Vec::new() }
    }

    /// 完成一个任务：目标 revision == 当前 → 发布；否则入 late（不覆盖当前根）。
    pub fn complete(&mut self, target_revision: u32, value: T) {
        if target_revision == self.current_revision {
            if self.published.is_none() {
                self.published = Some((target_revision, value));
            } else {
                self.late.push((target_revision, value));
            }
        } else {
            self.late.push((target_revision, value));
        }
    }

    /// 取消：任务结果显式作废（进入 cancelled，不发布）。
    pub fn cancel(&mut self, target_revision: u32, value: T) {
        self.cancelled.push((target_revision, value));
    }

    pub fn published(&self) -> Option<&(u32, T)> {
        self.published.as_ref()
    }
    pub fn late(&self) -> &[(u32, T)] {
        &self.late
    }
    pub fn cancelled(&self) -> &[(u32, T)] {
        &self.cancelled
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn versioned_publish_never_overwrites_current() {
        let mut vp = VersionedPublish::new(3);
        vp.complete(2, "old-late");
        assert!(vp.published().is_none(), "late result must not publish");
        vp.complete(3, "current");
        assert_eq!(vp.published().map(|(r, _)| *r), Some(3));
        vp.complete(3, "dup-current");
        assert_eq!(vp.published().map(|(_, v)| *v), Some("current"), "first publish wins");
        assert_eq!(vp.late().len(), 2);
    }

    #[test]
    fn versioned_cancel_isolated() {
        let mut vp = VersionedPublish::new(1);
        vp.cancel(1, "x");
        assert!(vp.published().is_none());
        assert_eq!(vp.cancelled().len(), 1);
    }
}
