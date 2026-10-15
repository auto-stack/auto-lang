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

use crate::cst::{CstNode, CstTree, DeclUnit, SynDiag, FLAG_ERROR, FLAG_MISSING, FLAG_TRIVIA, NO_FIELD};
use crate::lexer::{CompiledScanner, LexDiag, Token};
use crate::program::*;
use crate::snapshot::Snapshot;

#[derive(Clone, Debug)]
pub struct ParseOutput {
    pub tree: CstTree,
    pub tokens: Vec<Token>,
    pub lex_diags: Vec<LexDiag>,
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

/// 嫁接候选：旧声明子树（POD 区间平移拷贝，相对跨度零改写）。
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
    /// 旧 token 下标（单元首 token，含 trivia）→ 候选。
    pub candidates: std::collections::HashMap<usize, GraftCandidate>,
    /// 位移表：[(edit.end, delta)]；origin >= edit.end 的候选平移 delta。
    pub shifts: Vec<(u64, i64)>,
}

impl<'a> GraftInput<'a> {
    pub fn shift_for(&self, origin: u64) -> i64 {
        self.shifts.iter().filter(|(end, _)| origin >= *end).map(|(_, d)| *d).sum()
    }
}

impl<'a> GraftInput<'a> {
    fn old_index_of(&self, new_idx: usize) -> Option<usize> {
        if new_idx >= self.new_base {
            Some(self.old_base + (new_idx - self.new_base))
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
    let tree = m.finish();
    Ok(ParseOutput { tree, tokens: m.toks, lex_diags: m.lex_diags })
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
                return Err(RtError::InvalidProgram("first-set fixpoint did not converge".into()));
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
    alt: usize,     // usize::MAX = 左递归哨兵（立即失败）
    step: usize,
    children: Vec<(u32, u32)>,
    save: Save,        // alt 入口（回溯）
    frame_save: Save,  // frame 入口
    consumed_any: bool,
    wait: Wait,
    list_started: bool,
    /// RepItem 迭代起点完整状态：item 失败回滚到迭代起点（非 alt 入口！）；
    /// 成功后零前进 → 结束 List。
    rep_save: Option<Save>,
    pushed_active: bool,
    is_decl_frame: bool,
    // pratt 状态
    pratt_vals: Vec<u32>,
    pratt_ops: Vec<(u8, Assoc, u32, u32)>, // (binding, assoc, op leaf node, op token id)
    pratt_stage: PrattStage,
    pratt_field: u32,
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
    nodes: Vec<CstNode>,
    children: Vec<u32>,
    fields: Vec<u32>,
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
                        p.token_name(op.op), op.binding
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
            nodes: Vec::new(),
            children: Vec::new(),
            fields: Vec::new(),
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

    /// 嫁接：拷贝旧声明子树（POD 平移）+ 跳过其 token。
    fn perform_graft(&mut self, decl: crate::cst::DeclUnit, tok_lo: usize, tok_hi: usize, shift: i64, field: u32) {
        let old_tree: &CstTree = match self.graft.as_ref() {
            Some(g) => g.old_tree,
            None => return,
        };
        let node_lo = decl.node_lo as usize;
        let node_hi = decl.node as usize;
        let count = node_hi + 1 - node_lo;
        let base = self.nodes.len() as u32;
        let decl_idx_new = (self.decls.len() + 1) as u32;
        for k in 0..count {
            let src = old_tree.nodes[node_lo + k];
            let new_first_child = self.children.len() as u32;
            for ci in 0..src.child_count as usize {
                let oc = old_tree.children[(src.first_child as usize) + ci];
                self.children.push(oc - node_lo as u32 + base);
                self.fields.push(old_tree.child_fields[(src.first_child as usize) + ci]);
            }
            self.nodes.push(CstNode {
                kind: src.kind,
                start: src.start,
                len: src.len,
                first_child: new_first_child,
                child_count: src.child_count,
                expected: src.expected,
                flags: src.flags,
                decl: decl_idx_new,
            });
        }
        let new_node = base + count as u32 - 1;
        let new_origin = (decl.origin as i64 + shift) as u64;
        self.decls.push(crate::cst::DeclUnit { origin: new_origin, node: new_node, node_lo: base });
        // 跳过该单元的 token（对齐区数量不变）；pending（leading trivia）随子树丢弃。
        let cur_new = if self.pending.is_empty() { self.i } else { self.pending[0] as usize };
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
            || self.active_rules.iter().any(|&(r, i)| r == rule && i == self.i);
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
                pushed_active: false,
                is_decl_frame: false,
                pratt_vals: Vec::new(),
                pratt_ops: Vec::new(),
                pratt_stage: PrattStage::WantPrimary { anchor: None },
                pratt_field: field,
                node_start: self.nodes.len(),
            });
            return;
        }
        self.active_rules.push((rule, self.i));
        let save = self.save();
        let is_decl = self.stack.len() == 1;
        if is_decl {
            let cur_new = if self.pending.is_empty() { self.i } else { self.pending[0] as usize };
            let hit = self.graft.as_ref().and_then(|gi| {
                let old_idx = gi.old_index_of(cur_new)?;
                gi.candidates.get(&old_idx).map(|c| (c.decl, c.tok_lo, c.tok_hi, gi.shift_for(c.decl.origin)))
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
            pushed_active: true,
            is_decl_frame: is_decl,
            pratt_vals: Vec::new(),
            pratt_ops: Vec::new(),
            pratt_stage: PrattStage::WantPrimary { anchor: None },
            pratt_field: field,
            node_start: self.nodes.len(),
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
            pushed_active: false,
            is_decl_frame: false,
            pratt_vals: Vec::new(),
            pratt_ops: Vec::new(),
            pratt_stage: PrattStage::WantPrimary { anchor: None },
            pratt_field: field,
            node_start: self.nodes.len(),
        });
    }

    fn peek(&mut self) -> Option<Token> {
        while self.i < self.toks.len() {
            let t = &self.toks[self.i];
            let is_trivia = t.token != u32::MAX
                && self.trivia_tok.get(t.token as usize).copied().unwrap_or(false);
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
        if self.open_toks.get(t.token as usize).copied().unwrap_or(false) {
            self.bracket += 1;
        }
        if self.close_toks.get(t.token as usize).copied().unwrap_or(false) && self.bracket > 0 {
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
        let kind = if t.token == u32::MAX { KIND_ERROR } else { TOKEN_KIND_BASE + t.token };
        let flags = if t.error { FLAG_ERROR } else { 0 } | extra_flags;
        let id = self.nodes.len() as u32;
        self.nodes.push(CstNode {
            kind,
            start: t.start as u32,
            len: (t.end - t.start) as u32,
            first_child: 0,
            child_count: 0,
            expected: 0,
            flags,
            decl: 0,
        });
        id
    }

    /// 组装节点（构建期跨度为绝对；min/max 由孩子推导；空集 → 零宽）。
    fn alloc_node(&mut self, kind: u32, children: Vec<(u32, u32)>, missing: Option<u32>, err: bool) -> u32 {
        let first_child = self.children.len() as u32;
        let mut min_start = u64::MAX;
        let mut max_end = 0u64;
        for (c, _) in &children {
            let nd = self.nodes[*c as usize];
            if nd.len > 0 || nd.child_count > 0 || nd.start > 0 {
                let s = nd.start as u64;
                let e = s + nd.len as u64;
                min_start = min_start.min(s);
                max_end = max_end.max(e);
            }
        }
        for (c, f) in children {
            self.children.push(c);
            self.fields.push(f);
        }
        let (start, len) = if min_start == u64::MAX { (0, 0) } else { ((min_start) as u32, (max_end - min_start) as u32) };
        let id = self.nodes.len() as u32;
        self.nodes.push(CstNode {
            kind,
            start,
            len,
            first_child,
            child_count: self.children.len() as u32 - first_child,
            expected: missing.unwrap_or(0),
            flags: if err { FLAG_ERROR } else { 0 } | if missing.is_some() { FLAG_MISSING } else { 0 },
            decl: 0,
        });
        id
    }

    fn diag(&mut self, msg: String, start: u64, end: u64) {
        self.diags.push(SynDiag { message: msg, start, end });
    }

    fn run(&mut self) -> Result<(), RtError> {
        let mut steps: u64 = 0;
        let trace = std::env::var("AC787_TRACE").is_ok();
        let cap: u64 = std::env::var("AC787_TRACE_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(u64::MAX);
        loop {
            steps += 1;
            if trace && steps <= cap {
                let f = self.stack.last().unwrap();
                let st = if f.is_pratt() { format!("pratt {:?}", f.pratt_stage_variant()) } else { format!("r{} a{} s{}", f.rule, f.alt, f.step) };
                let pk = self.toks.get(self.i).map(|t| (t.token, t.start)).unwrap_or((u32::MAX, u64::MAX));
                eprintln!("#{steps} {} i={} pk={:?} stack={} nodes={} diags={}", st, self.i, pk, self.stack.len(), self.nodes.len(), self.diags.len());
            }
            if trace && steps > cap {
                return Err(RtError::InvalidProgram("trace cap".into()));
            }
            if steps & 0xFFFF == 0 && self.mem_estimate() > self.budget.work_bytes {
                return Err(RtError::ResourceExhausted { what: "parse work bytes", limit: self.budget.work_bytes });
            }
            if self.stack.is_empty() {
                return Ok(());
            }
            if self.stack.last().unwrap().is_pratt() {
                self.step_pratt()?;
            } else {
                self.step_rule()?;
            }
        }
    }

    fn mem_estimate(&self) -> usize {
        self.nodes.len() * std::mem::size_of::<CstNode>()
            + self.children.len() * 8
            + self.toks.len() * std::mem::size_of::<Token>()
    }

    fn step_rule(&mut self) -> Result<(), RtError> {
        if self.stack.len() > self.budget.max_parse_depth {
            return Err(RtError::ResourceExhausted { what: "parse depth", limit: self.budget.max_parse_depth });
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
        let step = self.p.rules[rule as usize].alts[self.stack.last().unwrap().alt].steps[step_idx].clone();
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
                    self.stack.last_mut().unwrap().wait = Wait::Optional;
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
                    self.stack.last_mut().unwrap().wait = Wait::Optional;
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
                    // 分隔 token 消费（`;` 等）。
                    loop {
                        match self.peek() {
                            Some(pk) if self.p.newline.stmt_end.contains(&pk.token) => {
                                if self.consume().is_err() {
                                    self.fail_alt();
                                    return Ok(());
                                }
                            }
                            _ => break,
                        }
                    }
                    // ListNewline：第二项起需 `;`/`}`/换行 分隔。
                    if kind == 2 && self.stack.last().unwrap().list_started
                        && self.p.newline.significant
                        && self.bracket == 0
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
                if self.stack.len() == 1 && !matches!(step.repeat, Repeat::ListNewline if self.stack.last().unwrap().list_started && false) {
                if let Some(pk) = self.peek() {
                    let can_start_item = match step.atom {
                        Atom::Tok(t) => pk.token == t,
                        Atom::Rule(r) => self.first.starts(r, pk.token),
                        Atom::Pratt(x) => {
                            let prim = self.p.pratt[x as usize].primary;
                            self.first.starts(prim, pk.token)
                        }
                    };
                    if !can_start_item && !self.continuation_can_start(rule, step_idx, pk.token) {
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
                                    if starts || self.continuation_can_start(rule, step_idx, pk2.token) {
                                        break;
                                    }
                                    if self.consume().is_err() {
                                        break;
                                    }
                                    let leaf = self.stack.last_mut().unwrap().children.pop().unwrap().0;
                                    let (ls, le) = { let nd = self.nodes[leaf as usize]; (nd.start as u64, nd.start as u64 + nd.len as u64) };
                                    let node = self.alloc_node(KIND_ERROR, vec![(leaf, NO_FIELD)], None, true);
                                    self.stack.last_mut().unwrap().children.push((node, NO_FIELD));
                                    self.diag("skipped unexpected token at top level".into(), ls, le);
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
            if !nullable_step && !matches!(s.repeat, Repeat::Optional | Repeat::ZeroOrMore | Repeat::ListPlain | Repeat::ListNewline) {
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
            let s = self.stack.last().unwrap().save;
            self.restore(s);
            let now = self.save();
            let f = self.stack.last_mut().unwrap();
            f.alt += 1;
            f.step = 0;
            f.consumed_any = false;
            f.save = now;
        } else {
            self.handle_frame_failure();
        }
    }

    /// 子帧失败信号分派到父帧（wait 语义）。
    fn dispatch_child_failure(&mut self) {
        if self.stack.last().unwrap().is_pratt() {
            self.fail_pratt();
            return;
        }
        let w = self.stack.last().unwrap().wait;
        self.stack.last_mut().unwrap().wait = Wait::None;
        match w {
            Wait::None | Wait::OnceStep => self.fail_alt(),
            Wait::Optional => {
                let s = self.stack.last().unwrap().save;
                self.restore(s);
                self.stack.last_mut().unwrap().step += 1;
            }
            Wait::RepItem { .. } => {
                // 回滚到迭代起点（不是 alt 入口：已成功的前序迭代不回滚）。
                let s = self.stack.last().unwrap().rep_save.unwrap_or(self.stack.last().unwrap().save);
                self.restore(s);
                self.end_list_step();
            }
        }
    }

    fn handle_frame_failure(&mut self) {
        let (rule, frame_save, consumed, recover, field, pushed_active, is_decl) = {
            let f = self.stack.last().unwrap();
            (
                f.rule,
                f.frame_save,
                f.consumed_any,
                self.p.rules[f.rule as usize].recover,
                f.pratt_field,
                f.pushed_active,
                f.is_decl_frame,
            )
        };
        self.restore(frame_save);
        if is_decl {
            // 声明尝试失败：尚未注册（restore 已截断 decls）。
        }
        if recover && consumed {
            let mut err_leaves: Vec<(u32, u32)> = Vec::new();
            // 恢复节点必须覆盖规则起点起的全部字节：先收 pending trivia
            // （restore 后 = 本规则前导/中途 trivia），消费时连同 flush 的
            // trivia 叶一起回收（丢弃任何字节都会破坏无损重建）。
            {
                let trivia: Vec<u32> = self.pending.drain(..).collect();
                for ti in trivia {
                    let tok = self.toks[ti as usize];
                    err_leaves.push((self.alloc_leaf(&tok, FLAG_TRIVIA), NO_FIELD));
                }
            }
            loop {
                let pk = match self.peek() {
                    Some(pk) => pk,
                    None => break,
                };
                if self.p.recovery.sync.contains(&pk.token) {
                    break;
                }
                // first(rule) 停点必须先消费 ≥1 token：当前 token 即规则起点时
                // 立即停会零前进（失败的声明首 token 往往就是规则起始符）。
                if !err_leaves.is_empty() && self.first.starts(rule, pk.token) {
                    break;
                }
                let before = self.stack.last().unwrap().children.len();
                if self.consume().is_err() {
                    break;
                }
                let appended: Vec<(u32, u32)> =
                    self.stack.last_mut().unwrap().children.drain(before..).collect();
                err_leaves.extend(appended);
            }
            let (start_abs, end_abs) = if err_leaves.is_empty() {
                let p = self.peek().map(|t| t.start).unwrap_or(0);
                (p, p)
            } else {
                let l = err_leaves[0].0;
                let s = self.nodes[l as usize].start as u64;
                let last = *err_leaves.last().unwrap();
                let nd = self.nodes[last.0 as usize];
                (s, nd.start as u64 + nd.len as u64)
            };
            let missing = self.alloc_node(KIND_MISSING, Vec::new(), Some(self.p.rule_kind(rule)), false);
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
            Some(_) => self.dispatch_child_failure(),
            None => self.root_failure_wrap(),
        }
    }

    fn handle_frame_success(&mut self) {
        let (rule, children, field, node_start, is_decl) = {
            let f = self.stack.last_mut().unwrap();
            (f.rule, std::mem::take(&mut f.children), f.pratt_field, f.node_start, f.is_decl_frame)
        };
        let kind = self.p.rule_kind(rule);
        let node = self.alloc_node(kind, children, None, false);
        if is_decl {
            // 声明单元登记：相对化 [node_start..=node]，写 decl 索引。
            let origin = self.nodes[node as usize].start as u64;
            for n in &mut self.nodes[node_start..] {
                if n.start as u64 >= origin {
                    n.start = (n.start as u64 - origin) as u32;
                } else {
                    n.start = 0;
                }
            }
            let idx = (self.decls.len() + 1) as u32;
            for n in &mut self.nodes[node_start..] {
                n.decl = idx;
            }
            self.nodes[node as usize].start = 0;
            self.decls.push(DeclUnit { origin, node, node_lo: node_start as u32 });
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
            let is_trivia =
                t.token != u32::MAX && self.trivia_tok.get(t.token as usize).copied().unwrap_or(false);
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
                self.expr_anchor = if self.bracket == 0 && self.p.newline.significant { anchor } else { None };
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
                // bootstrap：primary 交付的孩子 → vals。
                {
                    let f = self.stack.last_mut().unwrap();
                    if f.pratt_vals.is_empty() {
                        let kids = std::mem::take(&mut f.children);
                        for (n, _) in kids {
                            f.pratt_vals.push(n);
                        }
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
                        let ok_line = !cp.callee_same_line
                            || self.bracket > 0
                            || pk.line == self.last_line;
                        if ok_line {
                            let callee = match self.stack.last().unwrap().pratt_vals.last().copied() {
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
                            let open_leaf = {
                                let f = self.stack.last_mut().unwrap();
                                let n = f.children.len();
                                let (l, _) = f.children.remove(n - 1);
                                f.children.clear();
                                l
                            };
                            if let Some(args_rule) = table.call_args {
                                let f = self.stack.last_mut().unwrap();
                                f.pratt_stage = PrattStage::WantArgs { callee, open_leaf };
                                f.wait = Wait::OnceStep;
                                self.push_rule(args_rule, NO_FIELD, false);
                            } else {
                                let f = self.stack.last_mut().unwrap();
                                f.pratt_stage = PrattStage::WantClose { callee, open_leaf };
                            }
                            return Ok(());
                        }
                        // callee 不同行：表达式结束（不取调用后缀）。
                        self.finish_pratt();
                        return Ok(());
                    }
                }
                // 运算符。
                let b = if pk.token == u32::MAX { -1 } else { self.op_binding[pk.token as usize] };
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
                    let top = self.stack.last().unwrap().pratt_ops.last().copied();
                    match top {
                        Some((tb, tassoc, _, _)) => {
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
                let op_leaf = {
                    let f = self.stack.last_mut().unwrap();
                    let n = f.children.len();
                    let (l, _) = f.children.remove(n - 1);
                    f.children.clear();
                    l
                };
                let f = self.stack.last_mut().unwrap();
                f.pratt_ops.push((b as u8, assoc, op_leaf, op_tok));
                f.pratt_stage = PrattStage::WantPrimary { anchor: Some(self.last_line) };
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
        let close_leaf = {
            let f = self.stack.last_mut().unwrap();
            let n = f.children.len();
            let (l, _) = f.children.remove(n - 1);
            l
        };
        let mut kids = vec![(callee, NO_FIELD), (open_leaf, NO_FIELD)];
        if let Some(a) = args_node {
            kids.push((a, NO_FIELD));
        }
        kids.push((close_leaf, NO_FIELD));
        let node = self.alloc_node(KIND_CALL, kids, None, false);
        let f = self.stack.last_mut().unwrap();
        f.pratt_vals.clear();
        f.pratt_vals.push(node);
        f.children.clear();
        f.pratt_stage = PrattStage::WantOp;
    }

    fn reduce_one(&mut self) {
        let popped = {
            let f = self.stack.last_mut().unwrap();
            if f.pratt_vals.len() < 2 || f.pratt_ops.is_empty() {
                return;
            }
            let op = f.pratt_ops.pop().unwrap();
            let rhs = f.pratt_vals.pop().unwrap();
            let lhs = f.pratt_vals.pop().unwrap();
            (op, lhs, rhs)
        };
        let ((_, _, op_leaf, op_tok), lhs, rhs) = popped;
        let kids = vec![(lhs, NO_FIELD), (op_leaf, NO_FIELD), (rhs, NO_FIELD)];
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

    fn fail_pratt(&mut self) {
        let frame_save = self.stack.last().unwrap().frame_save;
        self.restore(frame_save);
        self.stack.pop();
        match self.stack.last_mut() {
            Some(_) => self.dispatch_child_failure(),
            None => self.root_failure_wrap(),
        }
    }

    fn finish(&mut self) -> CstTree {
        if self.root_node.is_none() {
            self.root_failure_wrap();
        }
        let root = self.root_node.unwrap();
        // 防御：根完成后仍有未消费 token（不应发生）→ 附加 error 叶。
        if self.i < self.toks.len() {
            let mut extras: Vec<(u32, u32)> = Vec::new();
            for k in self.i..self.toks.len() {
                let t = self.toks[k];
                let n = self.alloc_leaf(&t, 0);
                extras.push((n, NO_FIELD));
            }
            if !extras.is_empty() {
                let first_child = self.children.len() as u32;
                for (c, f) in extras {
                    self.children.push(c);
                    self.fields.push(f);
                }
                let nd = &mut self.nodes[root as usize];
                nd.child_count += self.children.len() as u32 - first_child;
                self.diag("trailing unparsed tokens".into(), 0, 0);
            }
            self.i = self.toks.len();
        }
        let total_len = self.toks.last().map(|t| t.end).unwrap_or(0);
        self.nodes[root as usize].decl = 0;
        CstTree {
            nodes: std::mem::take(&mut self.nodes),
            children: std::mem::take(&mut self.children),
            child_fields: std::mem::take(&mut self.fields),
            root,
            decls: std::mem::take(&mut self.decls),
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
        assert_eq!(out.tree.decls.len(), 2, "second valid decl must still parse");
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
