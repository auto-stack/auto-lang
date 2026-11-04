//! PLAN-928 T-02：语法单元目录最小机制（closures 面最小整链）。
//!
//! 机制决策（T-02 冻结，计划 §2/§5.2/§5.4）：
//! - **密封 body 句柄**（不可变 `Rc` 载荷 + 内容指纹）承载跨修订复用——
//!   不改 AST 本体（resolve 仍可变写回），复用发生在目录载荷层；
//! - **生产计量**：物化（投影/复制成一密封体）一次计一；`Rc` 复用计 0；
//!   失败尝试的废弃物化单列（discarded），不得计入提交面；
//! - **语法复用 ≠ 语义复用**：同文 lambda 在 owner 内容变化时语法体可
//!   共享，但目录对上层输出 `semantic_recheck_required` 信号——当前
//!   capture/creator 事实仍由 closures_fn/resolve 的 oracle 唯一裁决
//!   （计划 §5.4：不能用"语法同文"替代捕获重验）；
//! - **逻辑身份**：键 = module 指纹 + 逻辑 owner + kind + 创建序（同文
//!   异 owner 绝不合并；同 owner 同文重复以 ordinal 区分；插入/删除使
//!   序漂移且文同 => 具名非快路径，如实整 owner 重产，不得假 0）。
//!
//! T-05/T-06 在此之上扩为正式单元目录/变更事件/codec；本文件是最小可
//! 证明机制（whole-path 探针的载体，也是后续正式目录的基座）。

use crate::atom_text::Span;
use crate::source::ast::{ClosureBody, Expr, ExprKind, FnItem, Item, Program, Stmt};
use crate::source::SourceProfile;
use std::collections::HashMap;
use std::rc::Rc;

use super::frontdoor::{parse_front, FrontBudget, FrontKind, FrontOutcome, FrontRequest};

/// 内容指纹（FNV-1a 64；身份机制用，非密码学）。
pub fn content_sha(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Fn,
    Lambda,
}

/// 逻辑单元键（§5.4：module/owner/kind/ordinal；非 CST 节点号/绝对 offset）。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnitKey {
    pub module: u64,
    pub owner: String,
    pub kind: UnitKind,
    pub ordinal: u32,
}

/// 密封 lambda 语法体。`span` 是**当前**源坐标（可漂移；身份由键+content_sha
/// 承载，坐标漂移不换身份——§5.4）。
#[derive(Debug)]
pub struct SealedLambda {
    pub content_sha: u64,
    pub body: Rc<ClosureBody>,
    /// lambda 表达式整体（含参数表）的当前绝对跨度。
    pub span: Span,
}

/// 密封 owner fn 语法体（含整个 body；lambda 子体单列密封）。
#[derive(Debug)]
pub struct SealedFn {
    pub content_sha: u64,
    pub item: Rc<FnItem>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct ProductionMeter {
    /// 提交的物化次数（首次密封或重投影后入目录）。
    pub produced: usize,
    /// Rc 复用次数（0 生产命中）。
    pub reused: usize,
    /// 失败尝试产生的废弃物化（未提交目录；单列，不冒 0）。
    pub discarded: usize,
}

/// 一次单元编辑的提交结果（诊断/资源失败不产生本结构——目录不变）。
#[derive(Debug)]
pub struct EditReceipt {
    pub new_src: String,
    /// 本编辑真实物化的单元键（生产 >0 的单元）。
    pub produced: Vec<UnitKey>,
    /// Rc 复用命中的单元键（生产 0）。
    pub reused: Vec<UnitKey>,
    /// owner 内容变化时同文 lambda 语法共享成立——语义重验信号（§5.2b）。
    pub semantic_recheck_required: bool,
    /// 本次编辑是否走了具名非快路径（如序漂移歧义 → 整 owner 重产）。
    pub non_fast_path: Option<&'static str>,
}

/// lambda 单元重解析失败（源码诊断；目录与源保持旧态）。
#[derive(Debug)]
pub enum EditError {
    Diag(Vec<crate::atom_text::Diagnostic>),
    ResourceExhausted { what: &'static str, limit: usize },
    Infra(String),
}

#[derive(Debug)]
pub struct UnitCatalog {
    pub module: u64,
    pub profile: SourceProfile,
    pub fns: HashMap<String, SealedFn>,
    /// (owner, ordinal) —— ordinal = owner 内 lambda 的源序创建序。
    pub lambdas: HashMap<(String, u32), SealedLambda>,
    pub meter: ProductionMeter,
    /// 目录内当前源文本（编辑的基；提交后更新）。
    src: String,
}

/// 表达式直接子表达式（值位；Ident 型 callee/名字叶不构成子值）。
fn expr_children(e: &Expr) -> Vec<&Expr> {
    fn arg_exprs(args: &[crate::source::ast::Arg]) -> Vec<&Expr> {
        args.iter()
            .filter_map(|a| match a {
                crate::source::ast::Arg::Pos(x) => Some(x),
                crate::source::ast::Arg::MovePos(x) => Some(x),
                crate::source::ast::Arg::Named { value, .. } => Some(value),
            })
            .collect()
    }
    match &e.kind {
        ExprKind::Binary { lhs, rhs, .. } => vec![lhs, rhs],
        ExprKind::Unary { operand, .. } => vec![operand],
        ExprKind::Call { args, .. } => arg_exprs(args),
        ExprKind::CallValue { callee, args } => {
            let mut v = vec![&**callee];
            v.extend(arg_exprs(args));
            v
        }
        ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
            let mut v = vec![&**recv];
            v.extend(arg_exprs(args));
            v
        }
        ExprKind::Field { base, .. } => vec![base],
        ExprKind::Closure { body, .. } => match &**body {
            ClosureBody::Expr(x) => vec![x],
            ClosureBody::Block(_) => vec![],
        },
        _ => vec![],
    }
}

/// 源序遍历 owner body，收集 lambda（单遍 DFS：外层 lambda 先于其体内嵌套
/// lambda；同层按源码序）。深度随 AST 结构（解析预算已限深；T-08 的有界
/// 工作栈由正式目录承载，本最小面沿用访问递归）。
fn collect_lambdas(stmts: &[Stmt], out: &mut Vec<(Span, u64)>, src: &str) {
    fn visit_expr(e: &Expr, out: &mut Vec<(Span, u64)>, src: &str) {
        if let ExprKind::Closure { .. } = &e.kind {
            let slice = src.get(e.span.start..e.span.end).unwrap_or("");
            out.push((e.span, content_sha(slice.as_bytes())));
        }
        for c in expr_children(e) {
            visit_expr(c, out, src);
        }
        // 块体 lambda：体内语句的嵌套 lambda 在外层之后（DFS 先序）。
        if let ExprKind::Closure { body: b, .. } = &e.kind {
            if let ClosureBody::Block(blk) = &**b {
                for st in &blk.stmts {
                    visit_stmt(st, out, src);
                }
            }
        }
    }
    fn visit_stmt(s: &Stmt, out: &mut Vec<(Span, u64)>, src: &str) {
        match s {
            Stmt::Let { value, .. } | Stmt::Assign { value, .. } | Stmt::Return { value, .. } | Stmt::ExprStmt { value, .. } => visit_expr(value, out, src),
            Stmt::If { cond, then, els, .. } => {
                visit_expr(cond, out, src);
                for st in &then.stmts {
                    visit_stmt(st, out, src);
                }
                if let Some(e) = els {
                    for st in &e.stmts {
                        visit_stmt(st, out, src);
                    }
                }
            }
            Stmt::While { cond, body, .. } => {
                visit_expr(cond, out, src);
                for st in &body.stmts {
                    visit_stmt(st, out, src);
                }
            }
            Stmt::Loop { body, .. } => {
                for st in &body.stmts {
                    visit_stmt(st, out, src);
                }
            }
            Stmt::ForIn { iter, body, .. } => {
                visit_expr(iter, out, src);
                for st in &body.stmts {
                    visit_stmt(st, out, src);
                }
            }
            Stmt::IsMatch { arms, .. } => {
                for arm in arms {
                    for st in &arm.body.stmts {
                        visit_stmt(st, out, src);
                    }
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }
    for s in stmts {
        visit_stmt(s, out, src);
    }
}

impl UnitCatalog {
    /// 冷构建：整源解析 + 全单元首次密封（各计生产 1）。
    pub fn build(src: &str, profile: SourceProfile, budget: FrontBudget) -> Result<UnitCatalog, EditError> {
        let program = parse_whole(src, profile, budget)?;
        let module = content_sha(src.as_bytes());
        let mut cat = UnitCatalog {
            module,
            profile,
            fns: HashMap::new(),
            lambdas: HashMap::new(),
            meter: ProductionMeter::default(),
            src: src.to_string(),
        };
        for item in &program.items {
            if let Item::Fn(f) = item {
                if f.script {
                    continue;
                }
                let owner = f.name.name.clone();
                let sha = content_sha(cat.src.get(f.span.start as usize..f.span.end as usize).unwrap_or("").as_bytes());
                cat.fns.insert(owner.clone(), SealedFn { content_sha: sha, item: Rc::new(f.clone()) });
                cat.meter.produced += 1;
                let mut lambdas = Vec::new();
                collect_lambdas(&f.body.stmts, &mut lambdas, &cat.src);
                for (ordinal, (span, lsha)) in lambdas.into_iter().enumerate() {
                    // 从已密封 owner 体中取对应 lambda 语法体（同一次遍历序）。
                    let body = extract_lambda_body(&cat.fns[&owner].item, ordinal)
                        .expect("lambda ordinal matches collection order");
                    cat.lambdas.insert(
                        (owner.clone(), ordinal as u32),
                        SealedLambda { content_sha: lsha, body: Rc::new(body), span },
                    );
                    cat.meter.produced += 1;
                }
            }
        }
        Ok(cat)
    }

    /// §5.2(a)：单 lambda 编辑——只重解析/重投影目标单元；兄弟与 owner 的
    /// 语法体保持 Rc 复用（生产 0）。
    pub fn edit_lambda(
        &mut self,
        owner: &str,
        ordinal: u32,
        new_slice: &str,
        budget: FrontBudget,
    ) -> Result<EditReceipt, EditError> {
        let key = (owner.to_string(), ordinal);
        let Some(old) = self.lambdas.get(&key) else {
            return Err(EditError::Infra(format!("unknown lambda unit {key:?}")));
        };
        let new_src = replace_range(&self.src, old.span, new_slice);
        // 单元域重解析（共享链：同 grammar/同投影；包装为 return 表达式位）。
        match parse_lambda_unit(new_slice, self.profile, budget) {
            Ok((expr, consumed_len)) => {
                if consumed_len != new_slice.len() {
                    // 片段必须完整消耗（§5.1 前门合同）。
                    self.meter.discarded += 1;
                    return Err(EditError::Diag(vec![crate::atom_text::Diagnostic::new(
                        crate::atom_text::Stage::Text,
                        "source.incremental-fragment",
                        "a lambda fragment must be consumed completely",
                        crate::atom_text::Span { start: 0, end: 0 },
                    )]));
                }
                let body = match &expr.kind {
                    ExprKind::Closure { body, .. } => (**body).clone(),
                    _ => {
                        self.meter.discarded += 1;
                        return Err(EditError::Diag(vec![crate::atom_text::Diagnostic::new(
                            crate::atom_text::Stage::Text,
                            "source.incremental-fragment",
                            "a lambda fragment must be a lambda expression",
                            crate::atom_text::Span { start: 0, end: 0 },
                        )]));
                    }
                };
                let sha = content_sha(new_slice.as_bytes());
                // 提交：仅目标单元物化；坐标 = 新源中该片的绝对位置。
                let new_start = old.span.start;
                let sealed = SealedLambda {
                    content_sha: sha,
                    body: Rc::new(body),
                    span: Span { start: new_start, end: new_start + new_slice.len() },
                };
                self.lambdas.insert(key.clone(), sealed);
                self.src = new_src.clone();
                self.meter.produced += 1;
                // owner 内容已变（lambda 文本是 owner 切片的一部分）。
                self.reseal_owner_span(owner);
                Ok(EditReceipt {
                    new_src,
                    produced: vec![UnitKey { module: self.module, owner: owner.to_string(), kind: UnitKind::Lambda, ordinal }],
                    reused: self
                        .lambdas
                        .keys()
                        .filter(|k| k.0 == owner && k.1 != ordinal)
                        .map(|k| UnitKey { module: self.module, owner: k.0.clone(), kind: UnitKind::Lambda, ordinal: k.1 })
                        .collect(),
                    semantic_recheck_required: true,
                    non_fast_path: None,
                })
            }
            Err(e) => {
                self.meter.discarded += 1;
                Err(e)
            }
        }
    }

    /// owner fn 编辑（§5.2(b)：同文 lambda 语法体共享；语义重验信号置位）。
    pub fn edit_owner(&mut self, owner: &str, new_fn_src: &str, budget: FrontBudget) -> Result<EditReceipt, EditError> {
        let old = self.fns.get(owner).ok_or_else(|| EditError::Infra(format!("unknown owner {owner}")))?;
        let old_span = old.item.span;
        let new_src = replace_range(&self.src, old_span, new_fn_src);
        let outcome = parse_front(FrontRequest {
            src: &new_src,
            profile: self.profile,
            kind: FrontKind::FreeFnFragment {
                range: old_span.start..(old_span.start + new_fn_src.len()),
            },
            budget,
        });
        match outcome {
            FrontOutcome::Ok(ok) => {
                let item = match ok.program.items.into_iter().next() {
                    Some(Item::Fn(f)) => f,
                    _ => {
                        self.meter.discarded += 1;
                        return Err(EditError::Infra("owner fragment carried no fn".into()));
                    }
                };
                let owner_sha = content_sha(new_fn_src.as_bytes());
                let owner_changed = self.fns.get(owner).map(|o| o.content_sha) != Some(owner_sha);
                self.fns.insert(owner.to_string(), SealedFn { content_sha: owner_sha, item: Rc::new(item.clone()) });
                self.meter.produced += 1;
                // 遍历新 owner 的 lambda：同文（content sha 相等）=> 复用旧 Rc
                // （生产 0）；文变 => 新密封（生产 1）。序漂移且文同 => 具名
                // 非快路径（整 owner 重产，如实计数——§5.4 不得假 0）。
                let mut lambdas = Vec::new();
                collect_lambdas(&item.body.stmts, &mut lambdas, &new_src);
                let old_count = self.lambdas.keys().filter(|k| k.0 == owner).count();
                let mut produced = vec![UnitKey { module: self.module, owner: owner.to_string(), kind: UnitKind::Fn, ordinal: 0 }];
                let mut reused = Vec::new();
                let mut non_fast: Option<&'static str> = None;
                if lambdas.len() == old_count {
                    for (ordinal, (span, lsha)) in lambdas.into_iter().enumerate() {
                        let key = (owner.to_string(), ordinal as u32);
                        let body = extract_lambda_body(&item, ordinal).expect("ordinal order");
                        match self.lambdas.get(&key) {
                            Some(prev) if prev.content_sha == lsha => {
                                // 语法复用：共享 Rc，仅坐标漂移（身份不变）。
                                let mut updated = SealedLambda { content_sha: lsha, body: Rc::clone(&prev.body), span };
                                updated.span = span;
                                self.lambdas.insert(key.clone(), updated);
                                self.meter.reused += 1;
                                reused.push(UnitKey { module: self.module, owner: owner.to_string(), kind: UnitKind::Lambda, ordinal: ordinal as u32 });
                            }
                            _ => {
                                self.lambdas.insert(key.clone(), SealedLambda { content_sha: lsha, body: Rc::new(body), span });
                                self.meter.produced += 1;
                                produced.push(UnitKey { module: self.module, owner: owner.to_string(), kind: UnitKind::Lambda, ordinal: ordinal as u32 });
                            }
                        }
                    }
                } else {
                    // 拓扑变化（数量不等）→ 具名非快路径：整 owner lambda 重产。
                    non_fast = Some("owner-lambda-topology-changed");
                    self.lambdas.retain(|k, _| k.0 != owner);
                    for (ordinal, (span, lsha)) in lambdas.into_iter().enumerate() {
                        let body = extract_lambda_body(&item, ordinal).expect("ordinal order");
                        self.lambdas.insert((owner.to_string(), ordinal as u32), SealedLambda { content_sha: lsha, body: Rc::new(body), span });
                        self.meter.produced += 1;
                        produced.push(UnitKey { module: self.module, owner: owner.to_string(), kind: UnitKind::Lambda, ordinal: ordinal as u32 });
                    }
                }
                self.src = new_src.clone();
                Ok(EditReceipt {
                    new_src,
                    produced,
                    reused,
                    semantic_recheck_required: owner_changed,
                    non_fast_path: non_fast,
                })
            }
            FrontOutcome::Diag(ds) => {
                self.meter.discarded += 1;
                Err(EditError::Diag(ds))
            }
            FrontOutcome::ResourceExhausted { what, limit } => {
                self.meter.discarded += 1;
                Err(EditError::ResourceExhausted { what, limit })
            }
            FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => {
                self.meter.discarded += 1;
                Err(EditError::Infra(m))
            }
        }
    }

    fn reseal_owner_span(&mut self, owner: &str) {
        // 单 lambda 编辑后 owner 切片随之变化：重算 owner content_sha（生产
        // 不计——owner 语法体本身未重投影；T-05 冻结更精细的 owner 增量）。
        if let Some(sealed) = self.fns.get_mut(owner) {
            let span = sealed.item.span;
            let sha = content_sha(self.src.get(span.start as usize..span.end as usize).unwrap_or("").as_bytes());
            sealed.content_sha = sha;
        }
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn lambda(&self, owner: &str, ordinal: u32) -> Option<&SealedLambda> {
        self.lambdas.get(&(owner.to_string(), ordinal))
    }

    // ------------------------------------------------ T-05：候选事务与 pin

    /// 暂存一次 owner 编辑为**候选**：解析/投影/复用决策全部执行，但目录
    /// 与源不提交（V-05/V-10：失败/取消/多候选共存不改变 current）。
    pub fn stage_owner_edit(
        &self,
        owner: &str,
        new_fn_src: &str,
        budget: FrontBudget,
    ) -> Result<StagedCandidate, EditError> {
        let Some(sealed) = self.fns.get(owner) else {
            return Err(EditError::Infra(format!("unknown owner {owner}")));
        };
        let old_span = sealed.item.span;
        let new_src = replace_range(&self.src, old_span, new_fn_src);
        let outcome = parse_front(FrontRequest {
            src: &new_src,
            profile: self.profile,
            kind: FrontKind::FreeFnFragment {
                range: old_span.start..(old_span.start + new_fn_src.len()),
            },
            budget,
        });
        match outcome {
            FrontOutcome::Ok(ok) => {
                let Some(Item::Fn(item)) = ok.program.items.into_iter().next() else {
                    return Err(EditError::Infra("owner fragment carried no fn".into()));
                };
                let mut lambdas = Vec::new();
                collect_lambdas(&item.body.stmts, &mut lambdas, &new_src);
                let mut staged_lambdas: Vec<(u32, SealedLambda)> = Vec::new();
                let mut produced = 1usize;
                let mut reused = 0usize;
                let mut semantic_recheck: Vec<u32> = Vec::new();
                for (ordinal, (span, lsha)) in lambdas.iter().enumerate() {
                    let key = (owner.to_string(), ordinal as u32);
                    match self.lambdas.get(&key) {
                        Some(prev) if prev.content_sha == *lsha => {
                            staged_lambdas.push((
                                ordinal as u32,
                                SealedLambda { content_sha: *lsha, body: Rc::clone(&prev.body), span: *span },
                            ));
                            reused += 1;
                            semantic_recheck.push(ordinal as u32);
                        }
                        _ => {
                            let body = extract_lambda_body(&item, ordinal).expect("ordinal order");
                            staged_lambdas.push((
                                ordinal as u32,
                                SealedLambda { content_sha: *lsha, body: Rc::new(body), span: *span },
                            ));
                            produced += 1;
                            semantic_recheck.push(ordinal as u32);
                        }
                    }
                }
                let owner_sha = content_sha(new_fn_src.as_bytes());
                Ok(StagedCandidate {
                    owner: owner.to_string(),
                    new_src,
                    owner_sha,
                    owner_item: Rc::new(item),
                    lambdas: staged_lambdas,
                    produced,
                    reused,
                    semantic_recheck,
                })
            }
            FrontOutcome::Diag(ds) => Err(EditError::Diag(ds)),
            FrontOutcome::ResourceExhausted { what, limit } => Err(EditError::ResourceExhausted { what, limit }),
            FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => Err(EditError::Infra(m)),
        }
    }

    /// 提交暂存候选（当前修订推进；计量按暂存面如实入账）。
    pub fn commit_staged(&mut self, cand: StagedCandidate) -> EditReceipt {
        let is_reuse = |cat: &Self, owner: &str, ordinal: u32, sl: &SealedLambda| {
            cat.lambdas
                .get(&(owner.to_string(), ordinal))
                .map(|p| p.content_sha == sl.content_sha && Rc::ptr_eq(&p.body, &sl.body))
                .unwrap_or(false)
        };
        let produced_keys: Vec<UnitKey> = std::iter::once(UnitKey {
            module: self.module,
            owner: cand.owner.clone(),
            kind: UnitKind::Fn,
            ordinal: 0,
        })
        .chain(cand.lambdas.iter().filter(|(o, sl)| !is_reuse(self, &cand.owner, *o, sl)).map(|(o, _)| UnitKey {
            module: self.module,
            owner: cand.owner.clone(),
            kind: UnitKind::Lambda,
            ordinal: *o,
        }))
        .collect();
        let reused_keys: Vec<UnitKey> = cand
            .lambdas
            .iter()
            .filter(|(o, sl)| is_reuse(self, &cand.owner, *o, sl))
            .map(|(o, _)| UnitKey {
                module: self.module,
                owner: cand.owner.clone(),
                kind: UnitKind::Lambda,
                ordinal: *o,
            })
            .collect();
        self.fns.insert(
            cand.owner.clone(),
            SealedFn { content_sha: cand.owner_sha, item: cand.owner_item },
        );
        self.lambdas.retain(|k, _| k.0 != cand.owner);
        for (ordinal, sl) in cand.lambdas {
            self.lambdas.insert((cand.owner.clone(), ordinal), sl);
        }
        self.meter.produced += cand.produced;
        self.meter.reused += cand.reused;
        self.src = cand.new_src.clone();
        EditReceipt {
            new_src: cand.new_src,
            produced: produced_keys,
            reused: reused_keys,
            semantic_recheck_required: true,
            non_fast_path: None,
        }
    }

    /// 显式 pin：目录载荷的修订局部快照。旧 pin 只在其原修订视图读取；
    /// 跨修订消费按内容指纹由消费方校验（V-10：current/history/pin 同时
    /// 存活、互不串用）。
    pub fn pin(&self) -> CatalogPin {
        CatalogPin {
            revision_src: self.src.clone(),
            fns: self.fns.iter().map(|(k, v)| (k.clone(), Rc::clone(&v.item))).collect(),
            lambdas: self.lambdas.iter().map(|(k, v)| (k.clone(), Rc::clone(&v.body))).collect(),
        }
    }
}

/// 暂存候选（未提交；目录/源不变）。
pub struct StagedCandidate {
    owner: String,
    new_src: String,
    owner_sha: u64,
    owner_item: Rc<FnItem>,
    lambdas: Vec<(u32, SealedLambda)>,
    produced: usize,
    reused: usize,
    semantic_recheck: Vec<u32>,
}

impl StagedCandidate {
    pub fn owner(&self) -> &str {
        &self.owner
    }
    pub fn produced(&self) -> usize {
        self.produced
    }
    pub fn reused(&self) -> usize {
        self.reused
    }
    /// 需语义重验的 lambda 序号（语法共享不免除 capture 重验——§5.2b）。
    pub fn semantic_recheck(&self) -> &[u32] {
        &self.semantic_recheck
    }
    /// 候选源（诊断/比对用；不落目录）。
    pub fn new_src(&self) -> &str {
        &self.new_src
    }
}

/// 目录 pin：修订局部视图（旧 pin 句柄不跨修订冒用——`revision_sha` 由
/// 消费方校验；跨树句柄按 §5.4 拒绝）。
#[derive(Debug)]
pub struct CatalogPin {
    pub revision_src: String,
    fns: std::collections::HashMap<String, Rc<FnItem>>,
    #[allow(dead_code)]
    lambdas: std::collections::HashMap<(String, u32), Rc<ClosureBody>>,
}

impl CatalogPin {
    pub fn owner_item(&self, owner: &str) -> Option<&FnItem> {
        self.fns.get(owner).map(|r| r.as_ref())
    }
    pub fn revision_sha(&self) -> u64 {
        content_sha(self.revision_src.as_bytes())
    }
}

fn replace_range(src: &str, span: Span, new: &str) -> String {
    let mut out = String::with_capacity(src.len());
    out.push_str(&src[..span.start as usize]);
    out.push_str(new);
    out.push_str(&src[span.end as usize..]);
    out
}

/// 包装前缀长度（lambda 单元域重解析的合成宿主）。
const WRAP_PREFIX_LEN: usize = "fn __lambda_unit() int {\n    return ".len();

fn parse_whole(src: &str, profile: SourceProfile, budget: FrontBudget) -> Result<Program, EditError> {
    match parse_front(FrontRequest { src, profile, kind: FrontKind::WholeModule, budget }) {
        FrontOutcome::Ok(ok) => Ok(ok.program),
        FrontOutcome::Diag(ds) => Err(EditError::Diag(ds)),
        FrontOutcome::ResourceExhausted { what, limit } => Err(EditError::ResourceExhausted { what, limit }),
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => Err(EditError::Infra(m)),
    }
}

/// 单 lambda 片段经共享链重解析：合成 `fn __lambda_unit() int { return <slice> }`
/// 宿主走统一前门（同 grammar/同投影/同预算），取出 return 位表达式并平移
/// 回片内坐标。返回 (expr, 消费长度)。
fn parse_lambda_unit(slice: &str, profile: SourceProfile, budget: FrontBudget) -> Result<(Expr, usize), EditError> {
    let wrapped = format!("fn __lambda_unit() int {{\n    return {slice}\n}}\n");
    let program = parse_whole(&wrapped, profile, budget)?;
    let mut items = program.items;
    let Some(Item::Fn(mut f)) = items.pop() else {
        return Err(EditError::Infra("lambda wrapper carried no fn".into()));
    };
    let mut stmts = std::mem::take(&mut f.body.stmts);
    let Some(Stmt::Return { mut value, .. }) = stmts.pop() else {
        return Err(EditError::Infra("lambda wrapper carried no return".into()));
    };
    shift_expr(&mut value, WRAP_PREFIX_LEN, 0);
    Ok((value, slice.len()))
}

/// 表达式全跨度平移（片内相对坐标；T-05 由正式 codec 承载）。
fn shift_expr(e: &mut Expr, back: usize, _fwd: usize) {
    let mut work = vec![e];
    while let Some(cur) = work.pop() {
        cur.span = Span { start: cur.span.start.saturating_sub(back), end: cur.span.end.saturating_sub(back) };
        let mut children: Vec<&mut Expr> = Vec::new();
        fn push_args<'a>(args: &'a mut [crate::source::ast::Arg], children: &mut Vec<&'a mut Expr>) {
            for a in args.iter_mut() {
                match a {
                    crate::source::ast::Arg::Pos(x) => children.push(x),
                    crate::source::ast::Arg::MovePos(x) => children.push(x),
                    crate::source::ast::Arg::Named { value, .. } => children.push(value),
                }
            }
        }
        match &mut cur.kind {
            ExprKind::Binary { lhs, rhs, .. } => {
                children.push(lhs.as_mut());
                children.push(rhs.as_mut());
            }
            ExprKind::Unary { operand, .. } => children.push(operand.as_mut()),
            ExprKind::Call { args, .. } => push_args(args, &mut children),
            ExprKind::CallValue { callee, args } => {
                children.push(callee.as_mut());
                push_args(args, &mut children);
            }
            ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
                children.push(recv.as_mut());
                push_args(args, &mut children);
            }
            ExprKind::Field { base, .. } => children.push(base.as_mut()),
            ExprKind::Closure { body, .. } => {
                if let ClosureBody::Expr(x) = body.as_mut() {
                    children.push(x);
                }
            }
            _ => {}
        }
        for c in children {
            work.push(c);
        }
    }
}

/// 从密封 owner 体按序取 lambda 语法体（与 collect_lambdas 同一 DFS 先序：
/// 外层先、块内嵌套随后）。
fn extract_lambda_body(f: &FnItem, ordinal: usize) -> Option<ClosureBody> {
    let mut found = ordinal;
    let mut hit: Option<ClosureBody> = None;
    fn visit_expr(e: &Expr, ordinal: &mut usize, hit: &mut Option<ClosureBody>) -> bool {
        if let ExprKind::Closure { body, .. } = &e.kind {
            if *ordinal == 0 {
                hit.replace((**body).clone());
                return true;
            }
            *ordinal -= 1;
        }
        for c in expr_children(e) {
            if visit_expr(c, ordinal, hit) {
                return true;
            }
        }
        // 块体嵌套（DFS 先序与 collect_lambdas 一致）。
        if let ExprKind::Closure { body, .. } = &e.kind {
            if let ClosureBody::Block(blk) = &**body {
                for st in &blk.stmts {
                    if visit_stmt(st, ordinal, hit) {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn visit_stmt(s: &Stmt, ordinal: &mut usize, hit: &mut Option<ClosureBody>) -> bool {
        let exprs: Vec<&Expr> = match s {
            Stmt::Let { value, .. } | Stmt::Assign { value, .. } | Stmt::Return { value, .. } | Stmt::ExprStmt { value, .. } => vec![value],
            _ => vec![],
        };
        for e in exprs {
            if visit_expr(e, ordinal, hit) {
                return true;
            }
        }
        match s {
            Stmt::If { cond, then, els, .. } => {
                if visit_expr(cond, ordinal, hit) {
                    return true;
                }
                for st in &then.stmts {
                    if visit_stmt(st, ordinal, hit) {
                        return true;
                    }
                }
                if let Some(e) = els {
                    for st in &e.stmts {
                        if visit_stmt(st, ordinal, hit) {
                            return true;
                        }
                    }
                }
                false
            }
            Stmt::While { cond, body, .. } => {
                if visit_expr(cond, ordinal, hit) {
                    return true;
                }
                for st in &body.stmts {
                    if visit_stmt(st, ordinal, hit) {
                        return true;
                    }
                }
                false
            }
            Stmt::Loop { body, .. } => {
                for st in &body.stmts {
                    if visit_stmt(st, ordinal, hit) {
                        return true;
                    }
                }
                false
            }
            Stmt::ForIn { iter, body, .. } => {
                if visit_expr(iter, ordinal, hit) {
                    return true;
                }
                for st in &body.stmts {
                    if visit_stmt(st, ordinal, hit) {
                        return true;
                    }
                }
                false
            }
            Stmt::IsMatch { arms, .. } => {
                for arm in arms {
                    for st in &arm.body.stmts {
                        if visit_stmt(st, ordinal, hit) {
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }
    for s in &f.body.stmts {
        if visit_stmt(s, &mut found, &mut hit) {
            return hit;
        }
    }
    hit
}
