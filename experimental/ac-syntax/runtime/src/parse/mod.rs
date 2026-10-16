//! 规则程序执行器：显式工作栈（无原生深递归）+ 配置驱动 Pratt + 声明级恢复。
//!
//! - ordered-choice 回溯：alt 失败恢复光标试下一 alt；规则失败按 `recover`
//!   标志走同步点恢复（吃进 error 叶到 sync ∪ first(rule)，产出 missing）；
//! - List 步（ListPlain/ListNewline）：项间分隔 token 跳过；干净失败（0 消费）
//!   时若当前 token 既不能开始下一项也不能开始后续步骤 → 垃圾跳过
//!   （error 叶）后继续——"坏声明后合法声明仍可查询"；
//! - 换行策略（NewlinePolicy，generic 旋钮）：语句分隔/运算符同行/锚定
//!   primary/callee 同行/括号内自由——全部由语法包配置驱动，无语言分支；
//! - 左递归/零前进递归运行期防御（编译期已拒，此处兜底）。
//!
//! 节点构建期间一律使用**绝对**跨度（decl=0），声明完成时统一相对化
//! （origin = 声明首字节）并回写 decl 索引——增量复用免改后缀。

use crate::cst::{
    CstBlock, CstNode, CstTree, DeclUnit, SynDiag, CST_BLOCK_NODES, FLAG_ERROR, FLAG_MISSING,
    FLAG_TRIVIA, NO_FIELD, UNGRAFTED_DECL,
};
use crate::lexer::{CompiledScanner, LexDiag, Token};
use crate::program::*;
use crate::snapshot::Snapshot;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct ParseOutput {
    pub tree: CstTree,
    pub tokens: Vec<Token>,
    pub lex_diags: Vec<LexDiag>,
    /// 实际发生的声明嫁接次数（共享采纳；≠候选数——候选命中才算）。
    pub grafted_decls: usize,
}

pub fn parse_snapshot(
    p: &GrammarProgram,
    cs: &CompiledScanner,
    snap: &Snapshot,
    budget: &Budget,
) -> Result<ParseOutput, RtError> {
    let lex = crate::lexer::lex_all(p, cs, snap)?;
    parse_tokens(p, lex.tokens, lex.diags, budget)
}

/// 词法已就绪（增量路径复用）时的解析入口。
pub fn parse_tokens(
    p: &GrammarProgram,
    toks: Vec<Token>,
    lex_diags: Vec<LexDiag>,
    budget: &Budget,
) -> Result<ParseOutput, RtError> {
    parse_tokens_with_graft(p, toks, lex_diags, budget, None)
}

/// 嫁接候选：旧声明子树（**整块共享采纳**——不复制节点；相对跨度零改写）。
pub struct GraftCandidate {
    pub decl: crate::cst::DeclUnit,
    /// 子树覆盖的旧 token 范围 [tok_lo, tok_hi)（含 leading trivia）。
    pub tok_lo: usize,
    pub tok_hi: usize,
}

/// 嫁接输入：对齐区 token 映射 + 候选声明。
pub struct GraftInput<'a> {
    pub old_tree: &'a CstTree,
    /// 新 token ≥ new_base 与旧 token ≥ old_base 一一对齐。
    pub new_base: usize,
    pub old_base: usize,
    /// 未变前缀（checkpoint 前）：新/旧 token 下标一一相同（F787-R1-04——
    /// 前缀声明同样整块共享，不静默重解析）。
    pub prefix_len: usize,
    /// 旧 token 下标（单元首 token，含 trivia）→ 候选。
    pub candidates: std::collections::HashMap<usize, GraftCandidate>,
    /// 位移表：[(edit.end, delta)]；origin >= edit.end 的候选平移 delta。
    pub shifts: Vec<(u64, i64)>,
}

impl<'a> GraftInput<'a> {
    pub fn shift_for(&self, origin: u64) -> i64 {
        self.shifts
            .iter()
            .filter(|(end, _)| origin >= *end)
            .map(|(_, d)| *d)
            .sum()
    }
}

impl<'a> GraftInput<'a> {
    fn old_index_of(&self, new_idx: usize) -> Option<usize> {
        if new_idx >= self.new_base {
            Some(self.old_base + (new_idx - self.new_base))
        } else if new_idx < self.prefix_len {
            Some(new_idx)
        } else {
            None
        }
    }
}

/// 带嫁接的解析：顶层 List 尝试声明项时，若当前位置对齐一个未受影响旧声明，
/// 直接拷贝其子树并跳过其 token（真实复用；不重扫/不重解析）。
pub fn parse_tokens_with_graft(
    p: &GrammarProgram,
    toks: Vec<Token>,
    lex_diags: Vec<LexDiag>,
    budget: &Budget,
    graft: Option<GraftInput<'_>>,
) -> Result<ParseOutput, RtError> {
    let first = FirstSets::compute(p)?;
    let mut m = Machine::new(p, toks, lex_diags, first, budget, graft)?;
    m.run()?;
    let grafted = m.graft_count;
    let tree = m.finish();
    // token 流收紧容量（F787-R4-04：倍增松弛是真实驻留——流式构建的
    // 每块瞬态按实际字节计）。
    m.toks.shrink_to_fit();
    Ok(ParseOutput {
        tree,
        tokens: m.toks,
        lex_diags: m.lex_diags,
        grafted_decls: grafted,
    })
}

/// 规则 first 集与可空性（运行期一次计算；编译期校验复用同算法）。
pub struct FirstSets {
    pub first: Vec<Vec<bool>>, // rule -> token id -> bool
    pub nullable: Vec<bool>,
}

impl FirstSets {
    pub fn compute(p: &GrammarProgram) -> Result<FirstSets, RtError> {
        let n = p.rules.len();
        let nt = p.tokens.len();
        let mut first = vec![vec![false; nt]; n];
        let mut nullable = vec![false; n];
        let mut changed = true;
        let mut guard = 0;
        while changed {
            guard += 1;
            if guard > n * n + n + 8 {
                return Err(RtError::InvalidProgram(
                    "first-set fixpoint did not converge".into(),
                ));
            }
            changed = false;
            for ri in 0..n {
                for alt in &p.rules[ri].alts {
                    let mut all_nullable = true;
                    for s in &alt.steps {
                        match s.atom {
                            Atom::Tok(t) => {
                                if (t as usize) < nt {
                                    if !first[ri][t as usize] {
                                        first[ri][t as usize] = true;
                                        changed = true;
                                    }
                                } else {
                                    return Err(RtError::InvalidProgram(format!(
                                        "rule `{}` references unknown token #{t}",
                                        p.rules[ri].name
                                    )));
                                }
                                all_nullable = false;
                            }
                            Atom::Rule(r) => {
                                if (r as usize) >= n {
                                    return Err(RtError::InvalidProgram(format!(
                                        "rule `{}` references unknown rule #{r}",
                                        p.rules[ri].name
                                    )));
                                }
                                for t in 0..nt {
                                    if first[r as usize][t] && !first[ri][t] {
                                        first[ri][t] = true;
                                        changed = true;
                                    }
                                }
                                if !nullable[r as usize] {
                                    all_nullable = false;
                                }
                            }
                            Atom::Pratt(x) => {
                                if (x as usize) >= p.pratt.len() {
                                    return Err(RtError::InvalidProgram(format!(
                                        "rule `{}` references unknown pratt #{x}",
                                        p.rules[ri].name
                                    )));
                                }
                                let prim = p.pratt[x as usize].primary as usize;
                                for t in 0..nt {
                                    if first[prim][t] && !first[ri][t] {
                                        first[ri][t] = true;
                                        changed = true;
                                    }
                                }
                                if !nullable[prim] {
                                    all_nullable = false;
                                }
                            }
                        }
                        if !all_nullable {
                            break;
                        }
                    }
                    if all_nullable && !nullable[ri] {
                        nullable[ri] = true;
                        changed = true;
                    }
                }
            }
        }
        Ok(FirstSets { first, nullable })
    }

    pub fn starts(&self, rule: RuleId, tok: TokenId) -> bool {
        self.first
            .get(rule as usize)
            .and_then(|f| f.get(tok as usize))
            .copied()
            .unwrap_or(false)
    }
}

#[derive(Clone, Copy)]
struct Save {
    i: usize,
    pending: usize,
    last_line: u32,
    bracket: u32,
    decls: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum Wait {
    None,
    OnceStep,
    Optional,
    RepItem { kind: u8 }, // 0=ZeroOrMore 1=ListPlain 2=ListNewline
}

#[derive(Clone, Copy, PartialEq)]
enum FrameKind {
    Rule,
    Pratt { table: PrattId },
}

enum PrattStage {
    WantPrimary { anchor: Option<u32> },
    WantOp,
    WantArgs { callee: u32, open_leaf: u32 },
    WantClose { callee: u32, open_leaf: u32 },
    Done { node: u32 },
}

struct Frame {
    kind: FrameKind,
    rule: RuleId,
    alt: usize, // usize::MAX = 左递归哨兵（立即失败）
    step: usize,
    children: Vec<(u32, u32)>,
    save: Save,       // alt 入口（回溯）
    frame_save: Save, // frame 入口
    consumed_any: bool,
    wait: Wait,
    list_started: bool,
    /// RepItem 迭代起点完整状态：item 失败回滚到迭代起点（非 alt 入口！）；
    /// 成功后零前进 → 结束 List。
    rep_save: Option<Save>,
    /// Optional 步起点状态：item 失败回滚到步起点（非 alt 入口）。
    opt_save: Option<Save>,
    /// 帧进入时的覆盖起点（含 pending 前导 trivia）——声明单元 origin 用。
    cover_from: u64,
    pushed_active: bool,
    is_decl_frame: bool,
    // pratt 状态
    pratt_vals: Vec<u32>,
    pratt_ops: Vec<(u8, Assoc, u32, u32, Vec<u32>)>, // (binding, assoc, op leaf, op token, op 前导 trivia 叶)
    pratt_stage: PrattStage,
    pratt_field: u32,
    /// 调用后缀 open 括号前导 trivia（close 时并入 Call 孩子）。
    pratt_extra: Vec<u32>,
    node_start: usize,
}

impl Frame {
    fn is_pratt(&self) -> bool {
        matches!(self.kind, FrameKind::Pratt { .. })
    }
    fn pratt_stage_variant(&self) -> &'static str {
        match self.pratt_stage {
            PrattStage::WantPrimary { .. } => "WantPrimary",
            PrattStage::WantOp => "WantOp",
            PrattStage::WantArgs { .. } => "WantArgs",
            PrattStage::WantClose { .. } => "WantClose",
            PrattStage::Done { .. } => "Done",
        }
    }
}

/// 开放块构建器（封块前可变；声明边界才封块——声明子树永不跨块）。
struct OpenBlock {
    nodes: Vec<CstNode>,
    edges: Vec<u32>,
    fields: Vec<u32>,
    /// 本块内已完成声明的树级下标（封块时并入 block_decl）。
    decls: Vec<u32>,
}

struct Machine<'p, 'a> {
    p: &'p GrammarProgram,
    graft: Option<GraftInput<'a>>,
    toks: Vec<Token>,
    lex_diags: Vec<LexDiag>,
    i: usize,
    pending: Vec<u32>,
    last_line: u32,
    bracket: u32,
    expr_anchor: Option<u32>,
    // ---- 分块 arena（F787-R1-04：不可变共享内容块 + 长度目录）----
    /// 已封块（新建或从旧树采纳共享的 `Rc`）。
    blocks: Vec<Rc<CstBlock>>,
    /// 长度目录：与 blocks 平行，块 b 全局 id 基址（升序追加）。
    bases: Vec<u32>,
    /// 与 blocks 平行：块内声明序号 → 树级 decls 下标（UNGRAFTED_DECL=洞）。
    block_decl: Vec<Vec<u32>>,
    /// 当前开放块（尚未入 blocks）。
    open: OpenBlock,
    /// 开放块全局 id 基址。
    open_base: u32,
    /// 全局 id 前沿 = 已封节点总数。
    frontier: u32,
    /// 旧树块下标 → 本机 blocks 槽位（采纳去重）。
    adopted: std::collections::HashMap<u32, usize>,
    /// 根孩子目录（根节点跨块时；finish 组装进 root_edges）。
    pending_root_edges: Vec<(u32, u32)>,
    /// 实际嫁接（共享采纳）次数。
    graft_count: usize,
    diags: Vec<SynDiag>,
    decls: Vec<DeclUnit>,
    root_node: Option<u32>,
    trivia_tok: Vec<bool>,
    open_toks: Vec<bool>,
    close_toks: Vec<bool>,
    op_binding: Vec<i8>,
    first: FirstSets,
    budget: Budget,
    stack: Vec<Frame>,
    active_rules: Vec<(RuleId, usize)>,
}

impl<'p, 'a> Machine<'p, 'a> {
    fn new(
        p: &'p GrammarProgram,
        toks: Vec<Token>,
        lex_diags: Vec<LexDiag>,
        first: FirstSets,
        budget: &Budget,
        graft: Option<GraftInput<'a>>,
    ) -> Result<Self, RtError> {
        let mut trivia_tok = vec![false; p.tokens.len()];
        for r in &p.scanner.rules {
            if (r.token as usize) < p.tokens.len() {
                trivia_tok[r.token as usize] = r.trivia;
            }
        }
        let mut open_toks = vec![false; p.tokens.len()];
        let mut close_toks = vec![false; p.tokens.len()];
        let mut op_binding = vec![-1i8; p.tokens.len()];
        for t in &p.pratt {
            if let Some(o) = t.group_open {
                open_toks[o as usize] = true;
            }
            if let Some(c) = t.group_close {
                close_toks[c as usize] = true;
            }
            if let Some(cp) = &t.call_paren {
                open_toks[cp.open as usize] = true;
                close_toks[cp.close as usize] = true;
            }
            for op in &t.ops {
                if op.binding == 0 || op.binding > 100 {
                    return Err(RtError::InvalidProgram(format!(
                        "pratt op `{}` binding {} out of (0,100]",
                        p.token_name(op.op),
                        op.binding
                    )));
                }
                op_binding[op.op as usize] = op.binding as i8;
            }
        }
        let mut m = Machine {
            p,
            graft,
            toks,
            lex_diags,
            i: 0,
            pending: Vec::new(),
            last_line: 1,
            bracket: 0,
            expr_anchor: None,
            blocks: Vec::new(),
            bases: Vec::new(),
            block_decl: Vec::new(),
            open: OpenBlock {
                nodes: Vec::new(),
                edges: Vec::new(),
                fields: Vec::new(),
                decls: Vec::new(),
            },
            open_base: 0,
            frontier: 0,
            adopted: std::collections::HashMap::new(),
            pending_root_edges: Vec::new(),
            graft_count: 0,
            diags: Vec::new(),
            decls: Vec::new(),
            root_node: None,
            trivia_tok,
            open_toks,
            close_toks,
            op_binding,
            first,
            budget: *budget,
            stack: Vec::new(),
            active_rules: Vec::new(),
        };
        m.push_rule(p.start, NO_FIELD, false);
        Ok(m)
    }

    // ---- 分块 arena 原语 ----

    /// 全局 id → 节点（构建期：开放块 + 已封块统一寻址）。
    fn m_node(&self, g: u32) -> &CstNode {
        let off = g as i64 - self.open_base as i64;
        if off >= 0 && (off as usize) < self.open.nodes.len() {
            &self.open.nodes[off as usize]
        } else {
            let b = self.bases.partition_point(|&base| base <= g) - 1;
            let blk = &self.blocks[b];
            &blk.nodes[(g - self.bases[b]) as usize]
        }
    }

    /// 封当前开放块（仅在声明边界调用）；返回新开放块基址。封块即
    /// shrink_to_fit（F787-R4-04：倍增容量松弛是真实驻留——流式构建的
    /// 每块瞬态按实际字节计，不按分配上界）。
    fn seal_open(&mut self) {
        if !self.open.nodes.is_empty() {
            let mut ob = std::mem::replace(
                &mut self.open,
                OpenBlock {
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    fields: Vec::new(),
                    decls: Vec::new(),
                },
            );
            ob.nodes.shrink_to_fit();
            ob.edges.shrink_to_fit();
            ob.fields.shrink_to_fit();
            self.blocks.push(Rc::new(CstBlock {
                nodes: ob.nodes,
                edges: ob.edges,
                fields: ob.fields,
            }));
            self.bases.push(self.open_base);
            self.block_decl.push(ob.decls);
            self.frontier = self.open_base + self.blocks.last().unwrap().nodes.len() as u32;
            self.open_base = self.frontier;
        }
    }

    /// 采纳旧树块（Rc 共享，零节点复制）；去重。返回本机 blocks 槽位。
    fn ensure_adopted(&mut self, old_tree: &CstTree, old_block: u32) -> usize {
        if let Some(&slot) = self.adopted.get(&old_block) {
            return slot;
        }
        self.seal_open();
        let slot = self.blocks.len();
        self.blocks
            .push(old_tree.blocks[old_block as usize].clone());
        self.bases.push(self.frontier);
        let n_decl_slots = old_tree.block_decl[old_block as usize].len();
        self.block_decl.push(vec![UNGRAFTED_DECL; n_decl_slots]);
        self.frontier += self.blocks[slot].nodes.len() as u32;
        self.open_base = self.frontier;
        self.adopted.insert(old_block, slot);
        slot
    }

    /// 嫁接：**共享采纳**旧声明子树所在块（F787-R1-04：不做逐节点复制）+
    /// 跳过其 token。origin 按编辑位移平移（相对跨度零改写）。
    fn perform_graft(
        &mut self,
        decl: crate::cst::DeclUnit,
        tok_lo: usize,
        tok_hi: usize,
        shift: i64,
        field: u32,
    ) {
        let old_tree: &CstTree = match self.graft.as_ref() {
            Some(g) => g.old_tree,
            None => return,
        };
        let (ob, root_local) = old_tree.block_and_local(decl.node);
        let (_, lo_local) = old_tree.block_and_local(decl.node_lo);
        let slot = self.ensure_adopted(old_tree, ob);
        let new_base = self.bases[slot];
        let new_node = new_base + root_local;
        // 块内声明序号是块内在恰（共享块不重写）：旧树根节点 decl 字段直接沿用。
        let ordinal = old_tree.blocks[ob as usize].nodes[root_local as usize].decl;
        let bd = &mut self.block_decl[slot];
        if ordinal as usize > bd.len() {
            bd.resize(ordinal as usize, UNGRAFTED_DECL);
        }
        bd[(ordinal - 1) as usize] = self.decls.len() as u32;
        let new_origin = (decl.origin as i64 + shift) as u64;
        self.decls.push(crate::cst::DeclUnit {
            origin: new_origin,
            node: new_node,
            node_lo: new_base + lo_local,
        });
        // 跳过该单元的 token（对齐区数量不变）；pending（leading trivia）随子树丢弃。
        let cur_new = if self.pending.is_empty() {
            self.i
        } else {
            self.pending[0] as usize
        };
        let skip = tok_hi - tok_lo;
        self.i = (cur_new + skip).min(self.toks.len());
        self.pending.clear();
        if self.i > 0 {
            self.last_line = self.toks[self.i - 1].line;
        }
        let top = self.stack.last_mut().unwrap();
        top.children.push((new_node, field));
        top.list_started = true;
        top.consumed_any = true;
        self.graft_count += 1;
    }

    fn save(&self) -> Save {
        Save {
            i: self.i,
            pending: self.pending.len(),
            last_line: self.last_line,
            bracket: self.bracket,
            decls: self.decls.len(),
        }
    }

    fn restore(&mut self, s: Save) {
        self.i = s.i;
        self.pending.truncate(s.pending);
        self.last_line = s.last_line;
        self.bracket = s.bracket;
        self.decls.truncate(s.decls);
        self.expr_anchor = None;
    }

    fn push_rule(&mut self, rule: RuleId, field: u32, force_sentinel: bool) {
        let left_rec = force_sentinel
            || self
                .active_rules
                .iter()
                .any(|&(r, i)| r == rule && i == self.i);
        if left_rec {
            if !force_sentinel {
                self.diag(
                    format!(
                        "rule `{}` recurses without consuming input (left recursion)",
                        self.p.rule_name(rule)
                    ),
                    0,
                    0,
                );
            }
            let save = self.save();
            self.stack.push(Frame {
                kind: FrameKind::Rule,
                rule,
                alt: usize::MAX,
                step: 0,
                children: Vec::new(),
                save,
                frame_save: save,
                consumed_any: false,
                wait: Wait::None,
                list_started: false,
                rep_save: None,
                opt_save: None,
                cover_from: 0,
                pushed_active: false,
                is_decl_frame: false,
                pratt_vals: Vec::new(),
                pratt_ops: Vec::new(),
                pratt_stage: PrattStage::WantPrimary { anchor: None },
                pratt_field: field,
                pratt_extra: Vec::new(),
                node_start: self.open.nodes.len(),
            });
            return;
        }
        self.active_rules.push((rule, self.i));
        let save = self.save();
        let is_decl = self.stack.len() == 1;
        // 覆盖起点：pending 首个 trivia（前导）或当前 token。
        let cover_from = if let Some(&p0) = self.pending.first() {
            self.toks[p0 as usize].start
        } else {
            self.toks.get(self.i).map(|t| t.start).unwrap_or(0)
        };
        if is_decl {
            let cur_new = if self.pending.is_empty() {
                self.i
            } else {
                self.pending[0] as usize
            };
            let hit = self.graft.as_ref().and_then(|gi| {
                let old_idx = gi.old_index_of(cur_new)?;
                gi.candidates
                    .get(&old_idx)
                    .map(|c| (c.decl, c.tok_lo, c.tok_hi, gi.shift_for(c.decl.origin)))
            });
            if let Some((decl, tok_lo, tok_hi, shift)) = hit {
                self.perform_graft(decl, tok_lo, tok_hi, shift, field);
                return;
            }
        }
        self.stack.push(Frame {
            kind: FrameKind::Rule,
            rule,
            alt: 0,
            step: 0,
            children: Vec::new(),
            save,
            frame_save: save,
            consumed_any: false,
            wait: Wait::None,
            list_started: false,
            rep_save: None,
            opt_save: None,
            cover_from,
            pushed_active: true,
            is_decl_frame: is_decl,
            pratt_vals: Vec::new(),
            pratt_ops: Vec::new(),
            pratt_stage: PrattStage::WantPrimary { anchor: None },
            pratt_field: field,
            pratt_extra: Vec::new(),
            node_start: self.open.nodes.len(),
        });
    }

    fn push_pratt(&mut self, table: PrattId, field: u32) {
        let save = self.save();
        self.stack.push(Frame {
            kind: FrameKind::Pratt { table },
            rule: u32::MAX,
            alt: 0,
            step: 0,
            children: Vec::new(),
            save,
            frame_save: save,
            consumed_any: false,
            wait: Wait::None,
            list_started: false,
            rep_save: None,
            opt_save: None,
            cover_from: 0,
            pushed_active: false,
            is_decl_frame: false,
            pratt_vals: Vec::new(),
            pratt_ops: Vec::new(),
            pratt_stage: PrattStage::WantPrimary { anchor: None },
            pratt_field: field,
            pratt_extra: Vec::new(),
            node_start: self.open.nodes.len(),
        });
    }

    fn peek(&mut self) -> Option<Token> {
        while self.i < self.toks.len() {
            let t = &self.toks[self.i];
            let is_trivia = t.token != u32::MAX
                && self
                    .trivia_tok
                    .get(t.token as usize)
                    .copied()
                    .unwrap_or(false);
            if is_trivia {
                self.pending.push(self.i as u32);
                self.i += 1;
            } else {
                return Some(*t);
            }
        }
        None
    }

    /// 消费当前非 trivia token；pending trivia flush 到栈顶 frame。
    fn consume(&mut self) -> Result<Token, ()> {
        let t = match self.peek() {
            Some(t) => t,
            None => return Err(()),
        };
        if let Some(anchor) = self.expr_anchor {
            if self.bracket == 0 && t.line != anchor {
                return Err(());
            }
            self.expr_anchor = None;
        }
        if !self.pending.is_empty() {
            let trivia: Vec<u32> = self.pending.drain(..).collect();
            let mut leaves = Vec::with_capacity(trivia.len());
            for ti in trivia {
                let tok = self.toks[ti as usize];
                leaves.push(self.alloc_leaf(&tok, FLAG_TRIVIA));
            }
            let top = self.stack.last_mut().unwrap();
            for node in leaves {
                top.children.push((node, NO_FIELD));
            }
        }
        if self
            .open_toks
            .get(t.token as usize)
            .copied()
            .unwrap_or(false)
        {
            self.bracket += 1;
        }
        if self
            .close_toks
            .get(t.token as usize)
            .copied()
            .unwrap_or(false)
            && self.bracket > 0
        {
            self.bracket -= 1;
        }
        self.last_line = t.line;
        self.i += 1;
        self.stack.last_mut().unwrap().consumed_any = true;
        let node = self.alloc_leaf(&t, 0);
        let top = self.stack.last_mut().unwrap();
        top.children.push((node, NO_FIELD));
        Ok(t)
    }

    fn consume_same_line(&mut self, same_line: bool) -> Result<Token, ()> {
        if same_line && self.p.newline.significant && self.bracket == 0 {
            if let Some(t) = self.peek() {
                if t.line != self.last_line {
                    return Err(());
                }
            }
        }
        self.consume()
    }

    fn alloc_leaf(&mut self, t: &Token, extra_flags: u8) -> u32 {
        let kind = if t.token == u32::MAX {
            KIND_ERROR
        } else {
            TOKEN_KIND_BASE + t.token
        };
        let flags = if t.error { FLAG_ERROR } else { 0 } | extra_flags;
        let local = self.open.nodes.len() as u32;
        self.open.nodes.push(CstNode {
            kind,
            start: t.start as u32,
            len: (t.end - t.start) as u32,
            first_child: 0,
            child_count: 0,
            expected: 0,
            flags,
            decl: 0,
        });
        self.open_base + local
    }

    /// 组装节点（构建期跨度为绝对；min/max 由孩子推导；空集 → 零宽）。
    /// 孩子跨块（只有根：嫁接采纳块/已封块的孩子）→ 树级 `root_edges`
    /// 目录，节点块内置零；否则边为开放块局部下标。
    fn alloc_node(
        &mut self,
        kind: u32,
        children: Vec<(u32, u32)>,
        missing: Option<u32>,
        err: bool,
    ) -> u32 {
        let cross = children.iter().any(|&(c, _)| c < self.open_base);
        let mut min_start = u64::MAX;
        let mut max_end = 0u64;
        for &(c, _) in &children {
            let nd = *self.m_node(c);
            if nd.flags & FLAG_TRIVIA != 0 {
                continue; // 语义 span 不含 trivia（无损覆盖由孩子序列承担）
            }
            if nd.len > 0 || nd.child_count > 0 || nd.start > 0 {
                let s = nd.start as u64;
                let e = s + nd.len as u64;
                min_start = min_start.min(s);
                max_end = max_end.max(e);
            }
        }
        let (start, len) = if min_start == u64::MAX {
            (0, 0)
        } else {
            ((min_start) as u32, (max_end - min_start) as u32)
        };
        let flags =
            if err { FLAG_ERROR } else { 0 } | if missing.is_some() { FLAG_MISSING } else { 0 };
        let local = self.open.nodes.len() as u32;
        let id = self.open_base + local;
        if cross {
            self.pending_root_edges.extend(children.iter().cloned());
            self.open.nodes.push(CstNode {
                kind,
                start,
                len,
                first_child: 0,
                child_count: 0,
                expected: missing.unwrap_or(0),
                flags,
                decl: 0,
            });
        } else {
            let first_child = self.open.edges.len() as u32;
            for (c, f) in children {
                self.open.edges.push(c - self.open_base);
                self.open.fields.push(f);
            }
            self.open.nodes.push(CstNode {
                kind,
                start,
                len,
                first_child,
                child_count: self.open.edges.len() as u32 - first_child,
                expected: missing.unwrap_or(0),
                flags,
                decl: 0,
            });
        }
        id
    }

    fn diag(&mut self, msg: String, start: u64, end: u64) {
        self.diags.push(SynDiag {
            message: msg,
            start,
            end,
        });
    }

    fn run(&mut self) -> Result<(), RtError> {
        let mut steps: u64 = 0;
        let trace = std::env::var("AC787_TRACE").is_ok();
        let cap: u64 = std::env::var("AC787_TRACE_CAP")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(u64::MAX);
        loop {
            steps += 1;
            if self.stack.is_empty() {
                return Ok(());
            }
            if trace && steps <= cap {
                let f = self.stack.last().unwrap();
                let st = if f.is_pratt() {
                    format!(
                        "pratt {:?} vals={:?} ops={:?}",
                        f.pratt_stage_variant(),
                        f.pratt_vals,
                        f.pratt_ops.iter().map(|o| (o.0, o.3)).collect::<Vec<_>>()
                    )
                } else {
                    format!("r{} a{} s{}", f.rule, f.alt, f.step)
                };
                let pk = self
                    .toks
                    .get(self.i)
                    .map(|t| (t.token, t.start, t.line))
                    .unwrap_or((u32::MAX, u64::MAX, 0));
                eprintln!(
                    "#{steps} {} i={} pk={:?} last_line={} bracket={} stack={} nodes={}",
                    st,
                    self.i,
                    pk,
                    self.last_line,
                    self.bracket,
                    self.stack.len(),
                    self.frontier
                );
            }
            if trace && steps > cap {
                return Err(RtError::InvalidProgram("trace cap".into()));
            }
            if steps & 0xFFFF == 0 && self.mem_estimate() > self.budget.work_bytes {
                return Err(RtError::ResourceExhausted {
                    what: "parse work bytes",
                    limit: self.budget.work_bytes,
                });
            }
            if self.stack.last().unwrap().is_pratt() {
                self.step_pratt()?;
            } else {
                self.step_rule()?;
            }
        }
    }

    fn mem_estimate(&self) -> usize {
        self.frontier as usize * std::mem::size_of::<CstNode>()
            + self.open.edges.len() * 8
            + self.toks.len() * std::mem::size_of::<Token>()
    }

    fn step_rule(&mut self) -> Result<(), RtError> {
        if self.stack.len() > self.budget.max_parse_depth {
            return Err(RtError::ResourceExhausted {
                what: "parse depth",
                limit: self.budget.max_parse_depth,
            });
        }
        let sentinel = self.stack.last().unwrap().alt == usize::MAX;
        if sentinel {
            self.handle_frame_failure();
            return Ok(());
        }
        let (rule, step_len, step_idx) = {
            let f = self.stack.last().unwrap();
            let r = &self.p.rules[f.rule as usize];
            (f.rule, r.alts[f.alt].steps.len(), f.step)
        };
        if step_idx >= step_len {
            self.handle_frame_success();
            return Ok(());
        }
        let step = self.p.rules[rule as usize].alts[self.stack.last().unwrap().alt].steps[step_idx]
            .clone();
        let field = step.field.unwrap_or(NO_FIELD);

        match step.repeat {
            Repeat::Once => match step.atom {
                Atom::Tok(t) => {
                    let m = self.peek().map(|pk| pk.token == t).unwrap_or(false);
                    if !m {
                        self.fail_alt();
                        return Ok(());
                    }
                    match self.consume_same_line(step.same_line) {
                        Ok(_) => {
                            let f = self.stack.last_mut().unwrap();
                            let n = f.children.len();
                            f.children[n - 1].1 = field;
                            f.step += 1;
                        }
                        Err(()) => self.fail_alt(),
                    }
                }
                Atom::Rule(r) => {
                    if (r as usize) >= self.p.rules.len() {
                        self.diag(format!("unknown rule #{r}"), 0, 0);
                        self.fail_alt();
                        return Ok(());
                    }
                    self.stack.last_mut().unwrap().wait = Wait::OnceStep;
                    self.push_rule(r, field, false);
                }
                Atom::Pratt(x) => {
                    if (x as usize) >= self.p.pratt.len() {
                        self.fail_alt();
                        return Ok(());
                    }
                    self.push_pratt(x, field);
                }
            },
            Repeat::Optional => match step.atom {
                Atom::Rule(r) => {
                    let sv = self.save();
                    let f = self.stack.last_mut().unwrap();
                    f.wait = Wait::Optional;
                    f.opt_save = Some(sv);
                    self.push_rule(r, field, false);
                }
                Atom::Tok(t) => {
                    let m = self.peek().map(|pk| pk.token == t).unwrap_or(false);
                    if m {
                        match self.consume_same_line(step.same_line) {
                            Ok(_) => {
                                let f = self.stack.last_mut().unwrap();
                                let n = f.children.len();
                                f.children[n - 1].1 = field;
                                f.step += 1;
                            }
                            Err(()) => self.fail_alt(),
                        }
                    } else {
                        self.stack.last_mut().unwrap().step += 1;
                    }
                }
                Atom::Pratt(x) => {
                    let sv = self.save();
                    let f = self.stack.last_mut().unwrap();
                    f.wait = Wait::Optional;
                    f.opt_save = Some(sv);
                    self.push_pratt(x, field);
                }
            },
            Repeat::ZeroOrMore | Repeat::ListPlain | Repeat::ListNewline => {
                let kind: u8 = match step.repeat {
                    Repeat::ZeroOrMore => 0,
                    Repeat::ListPlain => 1,
                    _ => 2,
                };
                if kind >= 1 {
                    // 分隔 token 消费（仅 sep_consume=`;`；`}` 是边界不消费，
                    // 留给块规则的右括号步骤）。
                    let mut consumed_sep = false;
                    loop {
                        match self.peek() {
                            Some(pk) if self.p.newline.sep_consume.contains(&pk.token) => {
                                if self.consume().is_err() {
                                    self.fail_alt();
                                    return Ok(());
                                }
                                consumed_sep = true;
                            }
                            _ => break,
                        }
                    }
                    // ListNewline：第二项起需 `;`（已消费）/`}`/换行 分隔——
                    // 已消费分隔符即满足（`; x` 同行合法）。
                    if kind == 2
                        && self.stack.last().unwrap().list_started
                        && self.p.newline.significant
                        && self.bracket == 0
                        && !consumed_sep
                    {
                        if let Some(pk) = self.peek() {
                            let stmt_end = self.p.newline.stmt_end.contains(&pk.token);
                            if !(stmt_end || pk.line > self.last_line) {
                                self.end_list_step();
                                return Ok(());
                            }
                        }
                    }
                }
                // 干净失败前的"垃圾跳过"判定：仅顶层声明列表（root frame）启用；
                // 嵌套列表干净失败 = 列表结束，由外层规则/恢复机制接管——
                // 否则会把合法的列表终结符（如 `)`）误吃为垃圾。
                if self.stack.len() == 1
                    && !matches!(step.repeat, Repeat::ListNewline if self.stack.last().unwrap().list_started && false)
                {
                    if let Some(pk) = self.peek() {
                        let can_start_item = match step.atom {
                            Atom::Tok(t) => pk.token == t,
                            Atom::Rule(r) => self.first.starts(r, pk.token),
                            Atom::Pratt(x) => {
                                let prim = self.p.pratt[x as usize].primary;
                                self.first.starts(prim, pk.token)
                            }
                        };
                        if !can_start_item && !self.continuation_can_start(rule, step_idx, pk.token)
                        {
                            // 垃圾：作为 error 叶吃进（逐 token），直到能开始 item/后续/EOF。
                            let mut ate = 0u32;
                            loop {
                                match self.peek() {
                                    Some(pk2) => {
                                        let starts = match step.atom {
                                            Atom::Tok(t) => pk2.token == t,
                                            Atom::Rule(r) => self.first.starts(r, pk2.token),
                                            Atom::Pratt(x) => {
                                                let prim = self.p.pratt[x as usize].primary;
                                                self.first.starts(prim, pk2.token)
                                            }
                                        };
                                        if starts
                                            || self
                                                .continuation_can_start(rule, step_idx, pk2.token)
                                        {
                                            break;
                                        }
                                        if self.consume().is_err() {
                                            break;
                                        }
                                        let leaf = self
                                            .stack
                                            .last_mut()
                                            .unwrap()
                                            .children
                                            .pop()
                                            .unwrap()
                                            .0;
                                        let (ls, le) = {
                                            let nd = *self.m_node(leaf);
                                            (nd.start as u64, nd.start as u64 + nd.len as u64)
                                        };
                                        let node = self.alloc_node(
                                            KIND_ERROR,
                                            vec![(leaf, NO_FIELD)],
                                            None,
                                            true,
                                        );
                                        self.stack
                                            .last_mut()
                                            .unwrap()
                                            .children
                                            .push((node, NO_FIELD));
                                        self.diag(
                                            "skipped unexpected token at top level".into(),
                                            ls,
                                            le,
                                        );
                                        ate += 1;
                                    }
                                    None => break,
                                }
                            }
                            if ate > 0 {
                                let f = self.stack.last_mut().unwrap();
                                f.list_started = true;
                                return Ok(()); // 重新进入本步（现在可开始 item 或 EOF）
                            }
                            // 未吃到（peek 已可开始后续）→ List 结束。
                            self.end_list_step();
                            return Ok(());
                        }
                    }
                }
                // 尝试 item。
                match step.atom {
                    Atom::Rule(r) => {
                        let sv = self.save();
                        let f = self.stack.last_mut().unwrap();
                        f.wait = Wait::RepItem { kind };
                        f.rep_save = Some(sv);
                        self.push_rule(r, field, false);
                    }
                    Atom::Tok(t) => {
                        let m = self.peek().map(|pk| pk.token == t).unwrap_or(false);
                        if m {
                            match self.consume_same_line(step.same_line) {
                                Ok(_) => {
                                    let f = self.stack.last_mut().unwrap();
                                    let n = f.children.len();
                                    f.children[n - 1].1 = field;
                                    f.list_started = true;
                                }
                                Err(()) => self.fail_alt(),
                            }
                        } else {
                            self.end_list_step();
                        }
                    }
                    Atom::Pratt(x) => {
                        let sv = self.save();
                        let f = self.stack.last_mut().unwrap();
                        f.wait = Wait::RepItem { kind };
                        f.rep_save = Some(sv);
                        self.push_pratt(x, field);
                    }
                }
            }
        }
        Ok(())
    }

    /// 当前 alt 中 List 步之后是否还有能以 `tok` 开始的步骤（含 nullable 穿透）。
    fn continuation_can_start(&self, rule: RuleId, from_step: usize, tok: TokenId) -> bool {
        let steps = &self.p.rules[rule as usize].alts[self.stack.last().unwrap().alt].steps;
        let mut si = from_step + 1;
        while si < steps.len() {
            let s = &steps[si];
            let starts = match s.atom {
                Atom::Tok(t) => t == tok,
                Atom::Rule(r) => self.first.starts(r, tok),
                Atom::Pratt(x) => self.first.starts(self.p.pratt[x as usize].primary, tok),
            };
            if starts {
                return true;
            }
            // 该步可空 → 继续看后续。
            let nullable_step = match s.atom {
                Atom::Tok(_) => false,
                Atom::Rule(r) => self.first.nullable[r as usize],
                Atom::Pratt(_) => false,
            };
            if !nullable_step
                && !matches!(
                    s.repeat,
                    Repeat::Optional | Repeat::ZeroOrMore | Repeat::ListPlain | Repeat::ListNewline
                )
            {
                return false;
            }
            si += 1;
        }
        false // alt 末尾（EOF/外层接管）
    }

    fn end_list_step(&mut self) {
        if self.peek().is_none() {
            // EOF：flush 尾部 trivia 到当前 frame。
            let trivia: Vec<u32> = self.pending.drain(..).collect();
            if !trivia.is_empty() {
                let mut leaves = Vec::with_capacity(trivia.len());
                for ti in trivia {
                    let tok = self.toks[ti as usize];
                    leaves.push(self.alloc_leaf(&tok, FLAG_TRIVIA));
                }
                let top = self.stack.last_mut().unwrap();
                for node in leaves {
                    top.children.push((node, NO_FIELD));
                }
            }
        }
        self.stack.last_mut().unwrap().step += 1;
    }

    fn fail_alt(&mut self) {
        let has_next = {
            let f = self.stack.last().unwrap();
            f.alt + 1 < self.p.rules[f.rule as usize].alts.len()
        };
        if has_next {
            self.advance_alt();
        } else {
            self.handle_frame_failure();
        }
    }

    /// 子帧失败信号分派到当前顶帧（wait 语义）。
    /// `child_consumed`：失败子帧是否消费过 token——归入本帧（分派包装规则
    /// 自身不 consume；不传播则其恢复永不触发）。
    /// 返回 true = 本帧也失败（继续 unwind 上抛）；false = 已消化（停止）。
    /// Pratt 帧失败由统一迭代 unwind 处理（禁止递归级联：V-12）。
    fn dispatch_child_failure(&mut self, child_consumed: bool) -> bool {
        if self.stack.last().unwrap().is_pratt() {
            return true; // pratt 帧自身失败 → 上抛
        }
        if child_consumed {
            self.stack.last_mut().unwrap().consumed_any = true;
        }
        let w = self.stack.last().unwrap().wait;
        self.stack.last_mut().unwrap().wait = Wait::None;
        match w {
            Wait::None | Wait::OnceStep => {
                // 尝试下一 alt；耗尽 → 本帧失败上抛。
                let has_next = {
                    let f = self.stack.last().unwrap();
                    f.alt + 1 < self.p.rules[f.rule as usize].alts.len()
                };
                if has_next {
                    self.advance_alt();
                    false
                } else {
                    true
                }
            }
            Wait::Optional => {
                let s = self
                    .stack
                    .last()
                    .unwrap()
                    .opt_save
                    .unwrap_or_else(|| self.stack.last().unwrap().save);
                self.restore(s);
                self.stack.last_mut().unwrap().step += 1;
                false
            }
            Wait::RepItem { .. } => {
                // 回滚到迭代起点（不是 alt 入口：已成功的前序迭代不回滚）。
                let s = self
                    .stack
                    .last()
                    .unwrap()
                    .rep_save
                    .unwrap_or(self.stack.last().unwrap().save);
                self.restore(s);
                self.end_list_step();
                false
            }
        }
    }

    /// alt 推进（原 fail_alt 的有下一 alt 分支）。
    fn advance_alt(&mut self) {
        let s = self.stack.last().unwrap().save;
        self.restore(s);
        let now = self.save();
        let f = self.stack.last_mut().unwrap();
        f.alt += 1;
        f.step = 0;
        // consumed_any 不重置：恢复触发语义 = 本规则任一次尝试曾消费（部分匹配）。
        f.save = now;
    }

    /// 帧失败：恢复 / 迭代上抛（一次一层，禁止递归级联——V-12 深嵌套失败
    /// 路径同样不允许原生深递归）。
    fn handle_frame_failure(&mut self) {
        loop {
            let (rule, frame_save, consumed, recover, field, pushed_active, _is_pratt) = {
                let f = self.stack.last().unwrap();
                (
                    f.rule,
                    f.frame_save,
                    f.consumed_any,
                    if f.is_pratt() {
                        false
                    } else {
                        self.p.rules[f.rule as usize].recover
                    },
                    f.pratt_field,
                    f.pushed_active,
                    f.is_pratt(),
                )
            };
            self.restore(frame_save);
            if recover && consumed {
                // 覆盖语义：restore 把光标放回规则入口，恢复循环从入口**重新消费**
                // 到停点。帧入口光标已越过 leading trivia（List peek 预跳）——从旧流
                // 回扫重建其叶子；mid trivia 由循环内 peek/consume 自然重建
                // （restore 只能截断 pending 长度、不能还原内容，禁止依赖过期缓冲）。
                self.pending.clear();
                let mut err_leaves: Vec<(u32, u32)> = Vec::new();
                {
                    let mut t = frame_save.i;
                    let mut lead: Vec<usize> = Vec::new();
                    while t > 0 {
                        let prev = &self.toks[t - 1];
                        let is_triv = prev.token != u32::MAX
                            && self
                                .trivia_tok
                                .get(prev.token as usize)
                                .copied()
                                .unwrap_or(false);
                        if is_triv {
                            lead.push(t - 1);
                            t -= 1;
                        } else {
                            break;
                        }
                    }
                    lead.reverse();
                    for ti in lead {
                        let tok = self.toks[ti];
                        err_leaves.push((self.alloc_leaf(&tok, FLAG_TRIVIA), NO_FIELD));
                    }
                }
                let mut loop_ate_token = 0usize;
                loop {
                    let pk = match self.peek() {
                        Some(pk) => pk,
                        None => break,
                    };
                    if self.p.recovery.sync.contains(&pk.token) {
                        break;
                    }
                    // first(rule) 停点必须在恢复循环内消费 ≥1 个 token（前导 trivia
                    // flush 不算）：失败的声明首 token 往往就是规则起始符。
                    if loop_ate_token > 0 && self.first.starts(rule, pk.token) {
                        break;
                    }
                    let before = self.stack.last().unwrap().children.len();
                    if self.consume().is_err() {
                        break;
                    }
                    let appended: Vec<(u32, u32)> = self
                        .stack
                        .last_mut()
                        .unwrap()
                        .children
                        .drain(before..)
                        .collect();
                    loop_ate_token += 1;
                    err_leaves.extend(appended);
                }
                let (start_abs, end_abs) = if err_leaves.is_empty() {
                    let p = self.peek().map(|t| t.start).unwrap_or(0);
                    (p, p)
                } else {
                    let l = err_leaves[0].0;
                    let s = self.m_node(l).start as u64;
                    let last = *err_leaves.last().unwrap();
                    let nd = *self.m_node(last.0);
                    (s, nd.start as u64 + nd.len as u64)
                };
                let missing = {
                    let m = CstNode {
                        kind: KIND_MISSING,
                        start: start_abs as u32,
                        len: 0,
                        first_child: 0,
                        child_count: 0,
                        expected: self.p.rule_kind(rule),
                        flags: FLAG_MISSING,
                        decl: 0,
                    };
                    self.open.nodes.push(m);
                    self.open_base + (self.open.nodes.len() - 1) as u32
                };
                let mut kids = vec![(missing, NO_FIELD)];
                kids.extend(err_leaves);
                let node = self.alloc_node(self.p.rule_kind(rule), kids, None, true);
                self.diag(
                    format!("recovered at malformed `{}`", self.p.rule_name(rule)),
                    start_abs,
                    end_abs,
                );
                self.complete_frame_success(node, field);
                return;
            }
            if pushed_active {
                self.active_rules.pop();
            }
            self.stack.pop();
            match self.stack.last_mut() {
                None => {
                    self.root_failure_wrap();
                    return;
                }
                Some(_) => {
                    if self.dispatch_child_failure(consumed) {
                        continue; // 父帧也失败 → 迭代上抛
                    }
                    return;
                }
            }
        }
    }

    fn handle_frame_success(&mut self) {
        let (rule, children, field, node_start, is_decl) = {
            let f = self.stack.last_mut().unwrap();
            (
                f.rule,
                std::mem::take(&mut f.children),
                f.pratt_field,
                f.node_start,
                f.is_decl_frame,
            )
        };
        let kind = self.p.rule_kind(rule);
        let node = self.alloc_node(kind, children, None, false);
        if is_decl {
            // 声明单元登记：相对化 [node_start..]（开放块局部区间——封块只在
            // 声明边界，声明子树必在同一块内），写块内声明序号。
            // origin 取 cover_from（**含前导 trivia**——语义 span 已排除
            // trivia，用节点自身 start 会把前导 trivia 截断错位）。
            let origin = self.stack.last().unwrap().cover_from;
            for n in &mut self.open.nodes[node_start..] {
                if n.start as u64 >= origin {
                    n.start = (n.start as u64 - origin) as u32;
                } else {
                    n.start = 0;
                }
            }
            let ordinal = self.open.decls.len() as u32 + 1;
            for n in &mut self.open.nodes[node_start..] {
                n.decl = ordinal;
            }
            self.open.decls.push(self.decls.len() as u32);
            // 声明节点保留语义相对 span（首个非 trivia 孩子 .. 尾）：
            // abs = origin + start = 语义 token 起点（如 `fn`），与 ac-core 对齐。
            self.decls.push(DeclUnit {
                origin,
                node,
                node_lo: self.open_base + node_start as u32,
            });
            // 封块阈值（仅声明边界；下一声明起换新块——增量编辑按块共享）。
            if self.open.nodes.len() >= CST_BLOCK_NODES {
                self.seal_open();
            }
        }
        self.complete_frame_success(node, field);
    }

    fn complete_frame_success(&mut self, node: u32, field: u32) {
        let pushed_active = self.stack.last().unwrap().pushed_active;
        let was_decl = self.stack.last().unwrap().is_decl_frame;
        let _ = was_decl;
        if pushed_active {
            self.active_rules.pop();
        }
        self.stack.pop();
        match self.stack.last_mut() {
            Some(parent) => {
                parent.children.push((node, field));
                let w = parent.wait;
                parent.wait = Wait::None;
                match w {
                    Wait::None | Wait::OnceStep | Wait::Optional => {
                        self.stack.last_mut().unwrap().step += 1;
                    }
                    Wait::RepItem { .. } => {
                        let f = self.stack.last_mut().unwrap();
                        let mark = f.rep_save;
                        let no_progress = mark.map(|m| m.i == self.i).unwrap_or(true);
                        if no_progress {
                            f.step += 1;
                        } else {
                            f.list_started = true;
                        }
                    }
                }
            }
            None => {
                self.root_node = Some(node);
            }
        }
    }

    fn root_failure_wrap(&mut self) {
        let mut children: Vec<(u32, u32)> = Vec::new();
        let pending: Vec<u32> = self.pending.drain(..).collect();
        for ti in pending {
            let tok = self.toks[ti as usize];
            let n = self.alloc_leaf(&tok, FLAG_TRIVIA);
            children.push((n, NO_FIELD));
        }
        while self.i < self.toks.len() {
            let t = self.toks[self.i];
            let is_trivia = t.token != u32::MAX
                && self
                    .trivia_tok
                    .get(t.token as usize)
                    .copied()
                    .unwrap_or(false);
            let n = self.alloc_leaf(&t, if is_trivia { FLAG_TRIVIA } else { 0 });
            children.push((n, NO_FIELD));
            self.i += 1;
        }
        let end = self.toks.last().map(|t| t.end).unwrap_or(0);
        let root = self.alloc_node(self.p.rule_kind(self.p.start), children, None, true);
        self.diag("root parse failed; partial error tree".into(), 0, end);
        self.stack.clear();
        self.root_node = Some(root);
    }

    // ---- Pratt ----

    fn step_pratt(&mut self) -> Result<(), RtError> {
        let table_id = match &self.stack.last().unwrap().kind {
            FrameKind::Pratt { table } => *table,
            _ => unreachable!(),
        };
        let stage_taken;
        {
            let f = self.stack.last().unwrap();
            stage_taken = match &f.pratt_stage {
                PrattStage::WantPrimary { anchor } => PrattStage::WantPrimary { anchor: *anchor },
                PrattStage::WantOp => PrattStage::WantOp,
                PrattStage::WantArgs { callee, open_leaf } => PrattStage::WantArgs {
                    callee: *callee,
                    open_leaf: *open_leaf,
                },
                PrattStage::WantClose { callee, open_leaf } => PrattStage::WantClose {
                    callee: *callee,
                    open_leaf: *open_leaf,
                },
                PrattStage::Done { node } => PrattStage::Done { node: *node },
            };
        }
        let table = &self.p.pratt[table_id as usize];
        match stage_taken {
            PrattStage::WantPrimary { anchor } => {
                self.expr_anchor = if self.bracket == 0 && self.p.newline.significant {
                    anchor
                } else {
                    None
                };
                {
                    let f = self.stack.last_mut().unwrap();
                    f.wait = Wait::OnceStep;
                    f.pratt_stage = PrattStage::WantOp;
                }
                let prim = table.primary;
                self.push_rule(prim, NO_FIELD, false);
                Ok(())
            }
            PrattStage::WantOp => {
                // bootstrap：primary 交付的孩子 → vals（无条件搬移：
                // 每次 WantOp 进入时 children 应为空，滞留即丢失操作数）。
                {
                    let f = self.stack.last_mut().unwrap();
                    let kids = std::mem::take(&mut f.children);
                    for (n, _) in kids {
                        f.pratt_vals.push(n);
                    }
                }
                let pk = match self.peek() {
                    Some(pk) => pk,
                    None => {
                        self.finish_pratt();
                        return Ok(());
                    }
                };
                // 调用后缀。
                if let Some(cp) = &table.call_paren {
                    if pk.token == cp.open {
                        let ok_line =
                            !cp.callee_same_line || self.bracket > 0 || pk.line == self.last_line;
                        if ok_line {
                            let callee = match self.stack.last().unwrap().pratt_vals.last().copied()
                            {
                                Some(c) => c,
                                None => {
                                    self.fail_pratt();
                                    return Ok(());
                                }
                            };
                            if self.consume().is_err() {
                                self.fail_pratt();
                                return Ok(());
                            }
                            let (open_leaf, open_trivia) = {
                                let f = self.stack.last_mut().unwrap();
                                let mut taken = std::mem::take(&mut f.children);
                                let (l, _) = taken.pop().expect("consume pushed open leaf");
                                let triv: Vec<u32> = taken.into_iter().map(|(n, _)| n).collect();
                                (l, triv)
                            };
                            let f_open_trivia = open_trivia;
                            if let Some(args_rule) = table.call_args {
                                let f = self.stack.last_mut().unwrap();
                                f.pratt_stage = PrattStage::WantArgs { callee, open_leaf };
                                f.pratt_extra = f_open_trivia;
                                f.wait = Wait::OnceStep;
                                self.push_rule(args_rule, NO_FIELD, false);
                            } else {
                                let f = self.stack.last_mut().unwrap();
                                f.pratt_stage = PrattStage::WantClose { callee, open_leaf };
                                f.pratt_extra = f_open_trivia;
                            }
                            return Ok(());
                        }
                        // callee 不同行：表达式结束（不取调用后缀）。
                        self.finish_pratt();
                        return Ok(());
                    }
                }
                // 运算符。
                let b = if pk.token == u32::MAX {
                    -1
                } else {
                    self.op_binding[pk.token as usize]
                };
                if b < 0 {
                    self.finish_pratt();
                    return Ok(());
                }
                if self.p.newline.significant && self.bracket == 0 && pk.line > self.last_line {
                    // 后行运算符：表达式结束（语句层接管）。
                    self.finish_pratt();
                    return Ok(());
                }
                let assoc = table
                    .ops
                    .iter()
                    .find(|o| o.op == pk.token)
                    .map(|o| o.assoc)
                    .unwrap_or(Assoc::Left);
                loop {
                    let top = self
                        .stack
                        .last()
                        .unwrap()
                        .pratt_ops
                        .last()
                        .map(|(tb, ta, _, _, _)| (*tb, *ta));
                    match top {
                        Some((tb, tassoc)) => {
                            let reduce = match tassoc {
                                Assoc::Left => tb as i8 >= b,
                                Assoc::Right => tb as i8 > b,
                            };
                            if reduce {
                                self.reduce_one();
                            } else {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                let op_tok = pk.token;
                if self.consume().is_err() {
                    self.fail_pratt();
                    return Ok(());
                }
                // 取走全部孩子：最后一个 = 运算符叶；其余 = 前导 trivia 叶
                // （必须进入归约节点，否则丢字节破坏无损）。
                let (op_leaf, trivia) = {
                    let f = self.stack.last_mut().unwrap();
                    let mut taken = std::mem::take(&mut f.children);
                    let (l, _) = taken.pop().expect("consume pushed op leaf");
                    let triv: Vec<u32> = taken.into_iter().map(|(n, _)| n).collect();
                    (l, triv)
                };
                let f = self.stack.last_mut().unwrap();
                f.pratt_ops.push((b as u8, assoc, op_leaf, op_tok, trivia));
                f.pratt_stage = PrattStage::WantPrimary {
                    anchor: Some(self.last_line),
                };
                Ok(())
            }
            PrattStage::WantArgs { callee, open_leaf } => {
                let args_node = self.stack.last().unwrap().children.last().map(|(n, _)| *n);
                self.close_call(callee, open_leaf, args_node);
                Ok(())
            }
            PrattStage::WantClose { callee, open_leaf } => {
                self.close_call(callee, open_leaf, None);
                Ok(())
            }
            PrattStage::Done { node } => {
                let field = self.stack.last().unwrap().pratt_field;
                self.complete_frame_success(node, field);
                Ok(())
            }
        }
    }

    fn close_call(&mut self, callee: u32, open_leaf: u32, args_node: Option<u32>) {
        let close_ok = self
            .p
            .pratt
            .iter()
            .find_map(|t| t.call_paren.as_ref().map(|cp| cp))
            .and_then(|cp| self.peek().map(|pk| pk.token == cp.close));
        if close_ok != Some(true) {
            self.fail_pratt();
            return;
        }
        if self.consume().is_err() {
            self.fail_pratt();
            return;
        }
        // close 前导 trivia（args 末到 `)` 之间）+ open 前导 trivia 一并入树。
        // taken 里可能仍含 args 交付节点（kids 已按 args_node 参数拼装）——
        // 过滤首个重复引用，否则 args 子树在孩子里出现两次（字节重叠）。
        let (close_trivia, close_leaf) = {
            let f = self.stack.last_mut().unwrap();
            let mut taken = std::mem::take(&mut f.children);
            let (l, _) = taken.pop().expect("consume pushed close leaf");
            let mut rest: Vec<u32> = taken.into_iter().map(|(n, _)| n).collect();
            if let Some(a) = args_node {
                if let Some(pos) = rest.iter().position(|&n| n == a) {
                    rest.remove(pos);
                }
            }
            (rest, l)
        };
        let open_trivia = std::mem::take(&mut self.stack.last_mut().unwrap().pratt_extra);
        let mut kids = vec![(callee, NO_FIELD)];
        for t in open_trivia {
            kids.push((t, NO_FIELD));
        }
        kids.push((open_leaf, NO_FIELD));
        if let Some(a) = args_node {
            kids.push((a, NO_FIELD));
        }
        for t in close_trivia {
            kids.push((t, NO_FIELD));
        }
        kids.push((close_leaf, NO_FIELD));
        let node = self.alloc_node(KIND_CALL, kids, None, false);
        let f = self.stack.last_mut().unwrap();
        // 调用是**后缀**：仅弹出栈顶操作数（callee）替换为 call 节点；
        // clear 全部会丢掉已归约的 lhs（d07：`ac_start() + sf_ac_start()`）。
        f.pratt_vals.pop();
        f.pratt_vals.push(node);
        f.children.clear();
        f.pratt_stage = PrattStage::WantOp;
    }

    fn reduce_one(&mut self) {
        // 先弹 op——调用方 reduce 循环以 ops 递减为前进不变式；operand 不足
        // （非法中间态防御）以 missing 补位，绝不静默返回造成循环停滞。
        let popped = {
            let f = self.stack.last_mut().unwrap();
            let Some(op) = f.pratt_ops.pop() else { return };
            let rhs = f.pratt_vals.pop();
            let lhs = f.pratt_vals.pop();
            (op, lhs, rhs)
        };
        let ((_, _, op_leaf, op_tok, op_trivia), lhs, rhs) = popped;
        // missing 补位**惰性**分配：预分配会在正常归约中留下孤儿 MISSING 节点，
        // 污染 has_error_nodes（strict 投影误拒）。
        let (lhs, rhs) = match (lhs, rhs) {
            (Some(l), Some(r)) => (l, r),
            (l, r) => {
                let op_start = self.m_node(op_leaf).start;
                let m = CstNode {
                    kind: KIND_MISSING,
                    start: op_start,
                    len: 0,
                    first_child: 0,
                    child_count: 0,
                    expected: TOKEN_KIND_BASE + op_tok,
                    flags: FLAG_MISSING,
                    decl: 0,
                };
                self.open.nodes.push(m);
                let mid = self.open_base + (self.open.nodes.len() - 1) as u32;
                (l.unwrap_or(mid), r.unwrap_or(mid))
            }
        };
        let mut kids = vec![(lhs, NO_FIELD)];
        for t in op_trivia {
            kids.push((t, NO_FIELD));
        }
        kids.push((op_leaf, NO_FIELD));
        kids.push((rhs, NO_FIELD));
        let node = self.alloc_node(TOKEN_KIND_BASE + op_tok, kids, None, false);
        self.stack.last_mut().unwrap().pratt_vals.push(node);
    }

    fn finish_pratt(&mut self) {
        loop {
            let f = self.stack.last().unwrap();
            if f.pratt_ops.is_empty() || f.pratt_vals.len() < 2 {
                break;
            }
            self.reduce_one();
        }
        let node = match self.stack.last().unwrap().pratt_vals.last().copied() {
            Some(n) => n,
            None => {
                self.fail_pratt();
                return;
            }
        };
        self.stack.last_mut().unwrap().pratt_stage = PrattStage::Done { node };
    }

    /// pratt 帧失败：恢复光标、pop、并入统一迭代 unwind。
    fn fail_pratt(&mut self) {
        let frame_save = self.stack.last().unwrap().frame_save;
        self.restore(frame_save);
        self.stack.pop();
        loop {
            match self.stack.last_mut() {
                None => {
                    self.root_failure_wrap();
                    return;
                }
                Some(_) => {
                    if self.dispatch_child_failure(true) {
                        // 父帧也失败：pop 它并继续（与 handle_frame_failure 同一纪律）。
                        let pushed_active = self.stack.last().unwrap().pushed_active;
                        let fs = self.stack.last().unwrap().frame_save;
                        self.restore(fs);
                        if pushed_active {
                            self.active_rules.pop();
                        }
                        self.stack.pop();
                        continue;
                    }
                    return;
                }
            }
        }
    }

    fn finish(&mut self) -> CstTree {
        if self.root_node.is_none() {
            self.root_failure_wrap();
        }
        let root = self.root_node.unwrap();
        // 防御：根完成后仍有未消费 token（不应发生）→ 附加 error 叶（根目录）。
        let mut tail_edges: Vec<(u32, u32)> = Vec::new();
        if self.i < self.toks.len() {
            for k in self.i..self.toks.len() {
                let t = self.toks[k];
                let n = self.alloc_leaf(&t, 0);
                tail_edges.push((n, NO_FIELD));
            }
            if !tail_edges.is_empty() {
                self.diag("trailing unparsed tokens".into(), 0, 0);
            }
            self.i = self.toks.len();
        }
        let total_len = self.toks.last().map(|t| t.end).unwrap_or(0);
        // 根节点归一化（根必在当前开放块——它是最后分配的节点）：局部边
        // （单块小树场景）合并进树级目录，节点块内置零——与持久化 manifest
        // 的 root_edges 目录同构。
        let root_local = (root - self.open_base) as usize;
        debug_assert!(
            root_local < self.open.nodes.len(),
            "root must live in the open block"
        );
        let root_local = root_local.min(self.open.nodes.len().saturating_sub(1));
        let (mut root_edges, root_decl_zero) = {
            let nd = &mut self.open.nodes[root_local];
            let mut edges = Vec::with_capacity(
                nd.child_count as usize + self.pending_root_edges.len() + tail_edges.len(),
            );
            for k in 0..nd.child_count as usize {
                let c = self.open_base + self.open.edges[nd.first_child as usize + k];
                let f = self.open.fields[nd.first_child as usize + k];
                edges.push((c, f));
            }
            nd.first_child = 0;
            nd.child_count = 0;
            (edges, nd)
        };
        let _ = root_decl_zero;
        root_edges.append(&mut self.pending_root_edges);
        root_edges.append(&mut tail_edges);
        self.open.nodes[root_local].decl = 0;
        // 封最后一块并组装树。
        self.seal_open();
        CstTree {
            blocks: std::mem::take(&mut self.blocks),
            bases: std::mem::take(&mut self.bases),
            root,
            root_edges,
            decls: std::mem::take(&mut self.decls),
            block_decl: std::mem::take(&mut self.block_decl),
            diags: std::mem::take(&mut self.diags),
            total_len,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::CompiledScanner;
    use crate::snapshot::{PageStore, Snapshot};
    use crate::testkit::mini_program;
    use std::rc::Rc;

    fn parse_mini(text: &str) -> (ParseOutput, Snapshot) {
        let p = mini_program();
        let cs = CompiledScanner::compile(&p).unwrap();
        let snap = Snapshot::from_text(&Rc::new(PageStore::new()), text);
        let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
        (out, snap)
    }

    #[test]
    fn parse_mini_valid_two_decls() {
        let (out, snap) = parse_mini("api add(a: i32, b: i32) -> i32; note \"hi\";");
        assert!(out.tree.diags.is_empty(), "{:?}", out.tree.diags);
        assert_eq!(out.tree.decls.len(), 2);
        out.tree.assert_lossless(&snap).unwrap();
    }

    #[test]
    fn parse_mini_bad_decl_recovers_and_next_decl_parses() {
        let text = "api broken( -> i32;\napi ok(x: i32) -> i32;";
        let (out, snap) = parse_mini(text);
        assert!(!out.tree.diags.is_empty());
        assert_eq!(
            out.tree.decls.len(),
            2,
            "second valid decl must still parse"
        );
        out.tree.assert_lossless(&snap).unwrap();
    }

    #[test]
    fn parse_mini_garbage_between_decls_keeps_following_decls() {
        let text = "api a(x: i32) -> i32; $$$$ api b(y: i32) -> i32;";
        let (out, snap) = parse_mini(text);
        assert!(!out.tree.diags.is_empty());
        assert_eq!(out.tree.decls.len(), 2);
        out.tree.assert_lossless(&snap).unwrap();
    }

    #[test]
    fn parse_mini_empty_and_trivia_only() {
        let (out, snap) = parse_mini("// only a comment\n");
        assert_eq!(out.tree.decls.len(), 0);
        out.tree.assert_lossless(&snap).unwrap();
    }

    #[test]
    fn parse_deep_chain_no_stack_overflow() {
        // 8000 层嵌套括号表达式（平链+嵌套代表负载；≥4096 见 depth_ 档）。
        let mut text = String::from("api t(a: i32) -> i32;");
        let n = 8000;
        text.push_str(&format!("{}1{}", "(".repeat(n), ")".repeat(n)));
        let (out, snap) = parse_mini(&text);
        out.tree.assert_lossless(&snap).unwrap();
    }
}
