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
use crate::lexer::{CompiledScanner, LexDiag, ScanCheckpoint, Token};
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
    /// 未变前缀长度（checkpoint 之前新/旧 token 下标一一相同）。
    pub prefix_len: usize,
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
        return Ok(Splice {
            tokens: old_tokens.to_vec(),
            new_base: old_tokens.len(),
            old_base: old_tokens.len(),
            prefix_len: old_tokens.len(),
            rescan_bytes: 0,
        });
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
        .position(|t| {
            t.end > e.start
                && (if span_edit {
                    t.start < e.end
                } else {
                    t.start <= e.start
                })
        })
        .unwrap_or(old_tokens.len());
    // at-rest checkpoint：回退跳过紧邻前导 trivia（splice 在 token 边界切分会
    // 把连续 trivia 拆成两个叶，fresh 全量扫是一个 lexeme——从上一语义 token
    // 起重扫保证编辑区 trivia 完整重建）。前一 token（语义）非错误 = at-rest。
    let mut cp_idx = affected_first; // checkpoint 在 token[cp_idx] 起点
    while cp_idx > 0
        && old_tokens[cp_idx - 1].token != u32::MAX
        && p.scanner
            .rules
            .iter()
            .any(|r| r.token == old_tokens[cp_idx - 1].token && r.trivia)
    {
        cp_idx -= 1;
    }
    // 边界安全判据：前一 token 是 trivia（重扫将完整重建 trivia 链）或与当前
    // token 之间存在字节间隙；两侧都是语义 token 且直接相邻（粘连 lexeme 被
    // 切开）的边界不安全（`api add` → `apixadd` 类重组分叉）。BOF 安全。
    let is_trivia_tok = |t: &Token| {
        t.token != u32::MAX
            && p.scanner
                .rules
                .iter()
                .any(|r| r.token == t.token && r.trivia)
    };
    while cp_idx > 0
        && !is_trivia_tok(&old_tokens[cp_idx - 1])
        && old_tokens[cp_idx - 1].end == old_tokens[cp_idx].start
    {
        cp_idx -= 1;
    }
    if cp_idx == 0 {
        // 文档起点即 at-rest。
    } else if old_tokens[cp_idx - 1].error {
        return Err(FallbackReason::UnterminatedAtCheckpoint);
    }
    let cp = ScanCheckpoint {
        offset: old_tokens
            .get(cp_idx)
            .map(|t| t.start)
            .unwrap_or(old_snap.len()),
        mode: 0,
        mid: None,
        line: old_tokens.get(cp_idx).map(|t| t.line).unwrap_or(1),
    };

    // 重扫：新 snapshot 从 cp 起扫描，越过编辑新区后连续 K 个 token 对齐即停
    // （F787-R1-03：scanner 真正停止——不是扫到 EOF 再挑对齐点；扫描器带
    // 硬限，未闭合 token 的内部推进同样有界）。
    let shifted_end = (e.end as i64 + delta) as u64;
    let rescan_start = cp.offset;
    let scan_limit = rescan_start
        .saturating_add(RESCAN_WINDOW.saturating_mul(2))
        .min(new_snap.len());
    let mut sc = crate::lexer::Scanner::start_bounded(p, cs, new_snap, &cp, Some(scan_limit))
        .map_err(|_| FallbackReason::NoConvergence)?;
    let mut new_head: Vec<Token> = Vec::new();
    // 流式 K 连续对齐（原实现先全量扫再回头验证——停不下来）：
    // token j 匹配旧流 start==j.start-delta 的同 kind/同长 token 且连续 → run+1。
    let mut run_new: usize = 0; // run 首个 token 的 new_head 下标
    let mut run_old: usize = 0; // 对应旧 token 下标
    let mut run_len: usize = 0;
    let (mut align_new, mut align_old) = (None, None);
    loop {
        if sc.checkpoint().offset.saturating_sub(rescan_start) > RESCAN_WINDOW {
            return Err(FallbackReason::NoConvergence);
        }
        let t = match sc.next_token().map_err(|_| FallbackReason::NoConvergence)? {
            Some(t) => t,
            None => break, // EOF（或硬限）：未达 K 对齐 → 回退
        };
        let j = new_head.len();
        new_head.push(t);
        let nt = &new_head[j];
        // 流式续跑：run 内任意 kind（含 trivia）与旧流 run_old+run_len 精确
        // delta 平移对齐（与原"回头验证 K 窗"同谓词）。
        if run_len > 0 {
            let cont = old_tokens.get(run_old + run_len).is_some_and(|b| {
                (nt.start as i64 - b.start as i64) == delta
                    && nt.token == b.token
                    && nt.end - nt.start == b.end - b.start
            });
            if cont {
                run_len += 1;
                if run_len == CONVERGE_K {
                    align_new = Some(run_new + CONVERGE_K);
                    align_old = Some(run_old + CONVERGE_K);
                    break; // 收敛即停：不扫对齐区之后的任何字节
                }
                continue;
            }
            run_len = 0;
        }
        // 起跑候选：只取非 trivia 且越过编辑区的 token——trivia 处对齐会把
        // 连续 trivia 在 head/tail 边界拆裂（与全量词法不一致 → 树分叉）。
        let nt_is_trivia = nt.token != u32::MAX
            && p.scanner
                .rules
                .iter()
                .any(|r| r.token == nt.token && r.trivia);
        if !nt_is_trivia && nt.start + 1 > shifted_end {
            let old_pos = (nt.start as i64 - delta) as u64;
            // 旧流按 start 有序 → 二分定位（线性 position 在大文档回退路径退化）。
            let base = &old_tokens[cp_idx..];
            let oi = base.partition_point(|t| t.start < old_pos);
            if base.get(oi).is_some_and(|t| {
                t.start == old_pos && t.token == nt.token && t.end - t.start == nt.end - nt.start
            }) {
                let o = cp_idx + oi;
                // 对齐点边界安全判据同 checkpoint：前一 token 为 trivia 或有间隙。
                let boundary_safe = o == 0
                    || is_trivia_tok(&old_tokens[o - 1])
                    || old_tokens[o - 1].end < old_tokens[o].start;
                if boundary_safe {
                    run_new = j;
                    run_old = o;
                    run_len = 1;
                }
            }
        }
    }
    let (an, ao) = match (align_new, align_old) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            // 到 EOF/硬限都未对齐（删除至尾部、未闭合吃满尾部等）——重扫无法
            // 与旧流对齐：记录原因，交给上层全文回退（结果 = fresh）。
            return Err(FallbackReason::NoConvergence);
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
    // 全量流 = checkpoint 前旧前缀（字节未变，原样保留）+ 重扫 head + 平移 tail。
    // 少前缀会把增量流截断（1MiB 负载实测丢 ~40 万 token）。
    let mut tokens = Vec::with_capacity(cp_idx + an + old_tokens.len() - ao);
    tokens.extend_from_slice(&old_tokens[..cp_idx]);
    let head = new_head[..an].to_vec();
    tokens.extend_from_slice(&head);
    for t in &old_tokens[ao..] {
        tokens.push(Token {
            token: t.token,
            start: (t.start as i64 + delta) as u64,
            end: (t.end as i64 + delta) as u64,
            line: (t.line as i64 + head_newlines) as u32,
            error: t.error,
        });
    }
    // 重扫计量 = scanner 实际停止位置（收敛点 K 个 token 末）- checkpoint。
    // 绑定真实扫描路径，不是"所需逻辑跨度"。
    let rescan_bytes = sc.checkpoint().offset.saturating_sub(rescan_start);
    Ok(Splice {
        tokens,
        new_base: cp_idx + an,
        old_base: ao,
        prefix_len: cp_idx,
        rescan_bytes,
    })
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
    for d in &old.tree.decls {
        let (s, e) = old.tree.abs_range(d.node);
        if e > affected_lo && s < affected_hi {
            continue; // 相交 → 重解析
        }
        // 含 error/missing 节点的声明也重解析：子树可拷贝，但诊断 Vec 不随
        // 嫁接转移——重验证使 fresh 与增量的诊断集一致（round5 差分实测）。
        let mut has_err = false;
        let mut stack = vec![d.node];
        while let Some(n) = stack.pop() {
            let nd = old.tree.node(n);
            if nd.flags & (crate::cst::FLAG_ERROR | crate::cst::FLAG_MISSING) != 0 {
                has_err = true;
                break;
            }
            for &c in old.tree.children(n).iter().rev() {
                stack.push(c);
            }
        }
        if has_err {
            continue;
        }
        // 声明 token 范围：起点用 DeclUnit.origin（帧 cover_from，**含前导
        // trivia**——嫁接检查点在 leading trivia 上，语义 span 起点会对不上键）；
        // 尾点用语义 span 尾（尾部 trivia 归下一单元）。
        let lo = old.tokens.partition_point(|t| t.start < d.origin);
        let hi = old.tokens.partition_point(|t| t.start < e);
        by_old_tok.insert(
            lo,
            GraftCandidate {
                decl: *d,
                tok_lo: lo,
                tok_hi: hi,
            },
        );
    }

    if by_old_tok.is_empty() {
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
    let shifts: Vec<(u64, i64)> = edits
        .iter()
        .map(|e| (e.end, e.insert.len() as i64 - (e.end - e.start) as i64))
        .collect();
    let graft = crate::parse::GraftInput {
        old_tree: &old.tree,
        new_base: sp.new_base,
        old_base: sp.old_base,
        prefix_len: sp.prefix_len,
        candidates: by_old_tok,
        shifts,
    };
    let out = crate::parse::parse_tokens_with_graft(
        p,
        sp.tokens.clone(),
        Vec::new(),
        budget,
        Some(graft),
    )?;
    // 复用计量 = 实际嫁接（共享采纳）次数（F787-R1-04：候选命中才算，
    // 未命中的声明被重解析——如实计数）。
    let reused_actual = out.grafted_decls;
    let reparsed_actual = old.tree.decls.len().saturating_sub(reused_actual);
    Ok(IncrementalResult {
        tree: out.tree,
        tokens: out.tokens,
        lex_diags: out.lex_diags,
        rescan_bytes: sp.rescan_bytes,
        reparsed_decls: reparsed_actual,
        reused_decls: reused_actual,
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
        VersionedPublish {
            current_revision,
            published: None,
            late: Vec::new(),
            cancelled: Vec::new(),
        }
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
        assert_eq!(
            vp.published().map(|(_, v)| *v),
            Some("current"),
            "first publish wins"
        );
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
