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
use crate::source::SourceProfile;
use crate::source::ast::{ClosureBody, Expr, ExprKind, FnItem, Item, ParamMode, Program, Stmt};
use std::collections::HashMap;
use std::rc::Rc;

use super::frontdoor::{FrontBudget, FrontKind, FrontOutcome, FrontRequest, parse_front};

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
    /// PLAN-928（T-06）：`type` 声明单元（collections 面）。
    TypeDecl,
    /// PLAN-928（T-06）：`enum` 声明单元（collections 面）。
    EnumDecl,
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
///
/// PLAN-929（T-02，§5.6 惰性 body）：`body` 在**恢复构造**的目录里为 `None`
/// （身份-only，未物化——恢复不做源码生产）；冷构建/编辑/暂存路径恒 `Some`。
/// 未改单元跨恢复后编辑保持 `None`（物化 0）；消费方需要语法体时按单元惰性
/// 物化，不得为补上下文物化全部未改函数。
#[derive(Debug)]
pub struct SealedLambda {
    pub content_sha: u64,
    pub body: Option<Rc<ClosureBody>>,
    /// lambda 表达式整体（含参数表）的当前绝对跨度。
    pub span: Span,
}

/// 密封 owner fn 语法体（含整个 body；lambda 子体单列密封）。
/// `mint_span` = 逻辑身份铸造时的坐标（跨编辑携带不变；§5.3 纪律的
/// 语法面对应——身份 = 键+content_sha+mint_span）。
///
/// PLAN-928（T-09，V-06）：**当前坐标**在包装字段 `span`（目录本地）；
/// `Rc<FnItem>` 载荷内的 span 是铸造时视图且与 pin 共享——绝不原地
/// mutate（§5.4：不能 mutate 旧共享节点 span 来发布新修订）。上方编辑
/// 导致的坐标漂移由 `shift_units_after` 只改包装字段承载。
#[derive(Debug)]
pub struct SealedFn {
    pub content_sha: u64,
    /// PLAN-929（T-02）：恢复构造为 `None`（惰性 body，§5.6）；冷/编辑路径 `Some`。
    pub item: Option<Rc<FnItem>>,
    pub mint_span: Span,
    /// 当前绝对跨度（漂移维护面；载荷内 span 不动）。
    pub span: Span,
}

/// PLAN-928（T-06）：密封声明单元（type/enum；载荷以文本指纹承载——
/// 声明的语法体以源切片为身份面）。
#[derive(Debug)]
pub struct SealedDecl {
    pub content_sha: u64,
    pub name: String,
    /// TypeDecl / EnumDecl。
    pub kind: UnitKind,
    /// 声明首词（`type`/`enum`）起整段 span（当前坐标）。
    pub span: Span,
    /// 逻辑身份铸造时的坐标（跨编辑携带）。
    pub mint_span: Span,
}

/// PLAN-928（T-06）：接口/声明集变更事件（§5.2：类型/enum/签名改变须
/// 输出明确事件，不能因调用点文本未改而静默）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogEvent {
    /// owner 签名（参数/返回/mode）变化——依赖该接口的单元须重验。
    OwnerSignatureChanged { owner: String },
    /// 声明内容变化（type/enum）——`dependents` 为目录内引用该声明名的
    /// owner 集合（名字引用由语法面提取；语义依赖归 resolve 裁决）。
    DeclChanged {
        name: String,
        dependents: Vec<String>,
    },
    /// 声明集拓扑变化（新增/删除声明）。
    DeclSetChanged,
    /// λ 拓扑变化（具名非快路径路径）。
    LambdaTopologyChanged { owner: String },
}

/// PLAN-929（T-02，V-05）：恢复构造的受控拒绝分类——全部**在使用前**
/// 拒绝，不产生半恢复态；不放宽 913/928 更严格的 miss/拒绝合同。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreError {
    SchemaMismatch {
        found: String,
    },
    CapsMismatch {
        manifest: String,
        expected: String,
    },
    ProfileMismatch {
        manifest: String,
        expected: String,
    },
    ModuleHashMismatch {
        manifest: u64,
        source: u64,
    },
    UnitHashMismatch {
        owner: String,
        kind: &'static str,
        ordinal: u32,
        manifest: u64,
        source: u64,
    },
    SpanOutOfBounds {
        owner: String,
        kind: &'static str,
        start: usize,
        end: usize,
        src_len: usize,
    },
    OverlappingUnits {
        a: String,
        b: String,
    },
    DuplicateUnit {
        owner: String,
        kind: &'static str,
        ordinal: u32,
    },
}

/// PLAN-928（T-06）：单元清单（929 消费的可恢复目录投影——版本化、
/// 无宿主指针、无 trusted 位；身份 = SourceProfile+caps/grammar 版本+
/// source/单元内容指纹）。
pub const MANIFEST_SCHEMA: &str = "ac-source-units/1";

#[derive(Clone, Debug)]
pub struct CatalogManifest {
    pub schema: &'static str,
    pub module_sha: u64,
    /// `FRONT_CAPS_VERSION`（调用方注入——目录不持有 caps 常量的引用）。
    pub caps_version: String,
    /// profile 名（来源合同，不从内容猜）。
    pub profile: &'static str,
    pub units: Vec<ManifestUnit>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestUnit {
    pub kind: UnitKind,
    pub owner: String,
    pub ordinal: u32,
    pub content_sha: u64,
    pub mint_span: (usize, usize),
    pub current_span: (usize, usize),
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
    ResourceExhausted {
        what: &'static str,
        limit: usize,
    },
    Infra(String),
    /// PLAN-928（T-09，V-05）：stale/foreign 候选——候选不是针对当前修订
    /// （stale：staging 后目录已推进）或不是本目录的候选（foreign：另一
    /// 目录实例铸造）。两类提交一律拒绝且目录零变异（§5.4：候选失败/
    /// 取消/stale/foreign 不改变 current/原源/受检 registry）。
    StaleForeign(String),
}

#[derive(Debug)]
pub struct UnitCatalog {
    pub module: u64,
    /// PLAN-928（T-09）：进程内目录实例身份（跨目录 foreign 候选判别；
    /// 同源文本的两个目录 module 碰撞——instance 区分。仅进程内身份，
    /// **不入清单**——CatalogManifest 保持无宿主指针）。
    pub instance: u64,
    pub profile: SourceProfile,
    pub fns: HashMap<String, SealedFn>,
    /// (owner, ordinal) —— ordinal = owner 内 lambda 的源序创建序。
    pub lambdas: HashMap<(String, u32), SealedLambda>,
    /// PLAN-928（T-06）：type/enum 声明单元（名 → 密封体）。
    pub decls: HashMap<String, SealedDecl>,
    /// 逐次提交累积的变更事件（消费方读取；目录不越权裁决语义）。
    pub events: Vec<CatalogEvent>,
    pub meter: ProductionMeter,
    /// 目录内当前源文本（编辑的基；提交后更新）。
    src: String,
}

/// 目录实例计数器（进程内单调；从 1 起，0 保留为"无"）。
static INSTANCE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl UnitCatalog {
    /// PLAN-928（T-06）：接口签名摘要（名/参数表/返回/mode——签名变化
    /// 判定面；不含 body）。
    /// PLAN-929（T-02）：恢复目录未物化单元返回 `None`（签名面不可比——
    /// 消费方按保守重验处理，不得以 None 冒"签名未变"）。
    pub fn signature_snapshot(&self, owner: &str) -> Option<String> {
        self.fns
            .get(owner)
            .and_then(|s| s.item.as_deref())
            .map(signature_of)
    }

    /// PLAN-928（T-06）：声明变更事件（外部声明编辑路径提交后调用）——
    /// `dependents` = 目录内**名字引用**该声明的 owner（语法面；语义
    /// 依赖归 resolve）。刷新声明指纹。
    /// PLAN-929（T-02）：dependents 只在**已物化** owner 上提取（恢复目录
    /// 未物化单元不进入名字扫描——物化按需，不补扫全部未改函数，§5.6）。
    pub fn note_decl_changed(&mut self, name: &str, new_span: Span) {
        let dependents: Vec<String> = self
            .fns
            .iter()
            .filter(|(_, sf)| {
                sf.item
                    .as_deref()
                    .map(|i| references_name(i, name))
                    .unwrap_or(false)
            })
            .map(|(o, _)| o.clone())
            .collect();
        if let Some(d) = self.decls.get_mut(name) {
            d.span = new_span;
            d.content_sha = content_sha(
                self.src
                    .get(new_span.start..new_span.end)
                    .unwrap_or("")
                    .as_bytes(),
            );
        }
        self.events.push(CatalogEvent::DeclChanged {
            name: name.to_string(),
            dependents,
        });
        self.events.push(CatalogEvent::DeclSetChanged);
    }

    /// PLAN-928（T-06）：无指针清单（929 恢复消费；版本+指纹身份，不含
    /// 宿主指针/trusted 位；恢复深验归 929）。
    /// PLAN-929（T-02，修复）：`module_sha` 取**当前源**指纹——编辑后导出
    /// 不再携带冷建陈旧值（恢复侧 `from_restored` 以源指纹对账）。
    pub fn to_manifest(&self, caps_version: &str) -> CatalogManifest {
        let mut units: Vec<ManifestUnit> = self
            .fns
            .iter()
            .map(|(owner, sf)| ManifestUnit {
                kind: UnitKind::Fn,
                owner: owner.clone(),
                ordinal: 0,
                content_sha: sf.content_sha,
                mint_span: (sf.mint_span.start, sf.mint_span.end),
                current_span: (sf.span.start, sf.span.end),
            })
            .collect();
        units.extend(
            self.lambdas
                .iter()
                .map(|((owner, ordinal), sl)| ManifestUnit {
                    kind: UnitKind::Lambda,
                    owner: owner.clone(),
                    ordinal: *ordinal,
                    content_sha: sl.content_sha,
                    mint_span: (sl.span.start, sl.span.end),
                    current_span: (sl.span.start, sl.span.end),
                }),
        );
        units.extend(self.decls.iter().map(|(name, d)| ManifestUnit {
            kind: d.kind,
            owner: name.clone(),
            ordinal: 0,
            content_sha: d.content_sha,
            mint_span: (d.mint_span.start, d.mint_span.end),
            current_span: (d.span.start, d.span.end),
        }));
        units.sort_by(|a, b| {
            (&a.owner, a.ordinal, a.kind as u8)
                .partial_cmp(&(&b.owner, b.ordinal, b.kind as u8))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        CatalogManifest {
            schema: MANIFEST_SCHEMA,
            module_sha: content_sha(self.src.as_bytes()),
            caps_version: caps_version.to_string(),
            profile: super::frontdoor::profile_slug(self.profile),
            units,
        }
    }

    /// Candidate source-map metadata, without committing or cloning any AST.
    /// Mirrors commit_staged's span update and lambda directory replacement.
    pub fn staged_manifest(
        &self,
        cand: &StagedCandidate,
        caps_version: &str,
    ) -> Result<CatalogManifest, EditError> {
        self.validate_staged(cand)?;
        let mut manifest = self.to_manifest(caps_version);
        if !cand.changed {
            return Ok(manifest);
        }
        let old = self.fns[&cand.owner].span;
        let delta = cand.new_fn_len as i64 - (old.end - old.start) as i64;
        manifest.module_sha = content_sha(cand.new_src.as_bytes());
        manifest
            .units
            .retain(|u| !(u.owner == cand.owner && u.kind == UnitKind::Lambda));
        for unit in &mut manifest.units {
            if unit.kind == UnitKind::Fn && unit.owner == cand.owner {
                unit.content_sha = cand.owner_sha;
                unit.current_span = (old.start, old.start + cand.new_fn_len);
            } else if unit.current_span.0 >= old.end {
                let shift = |n: usize| -> Result<usize, EditError> {
                    usize::try_from(n as i128 + delta as i128)
                        .map_err(|_| EditError::Infra("candidate span overflow".into()))
                };
                unit.current_span = (shift(unit.current_span.0)?, shift(unit.current_span.1)?);
                if unit.kind == UnitKind::Lambda {
                    unit.mint_span = unit.current_span;
                }
            }
        }
        manifest
            .units
            .extend(cand.lambdas.iter().map(|(ordinal, sl)| ManifestUnit {
                kind: UnitKind::Lambda,
                owner: cand.owner.clone(),
                ordinal: *ordinal,
                content_sha: sl.content_sha,
                mint_span: (sl.span.start, sl.span.end),
                current_span: (sl.span.start, sl.span.end),
            }));
        manifest
            .units
            .sort_by_key(|u| (u.owner.clone(), u.ordinal, u.kind as u8));
        Ok(manifest)
    }
}

/// 签名摘要（声明序：名(参数, ...) -> 返回；mode 词计入）。
/// PLAN-999（T-05，F-941-13）：摘要为**无来源坐标**的结构化投影——名字、
/// 参数名/序/mode、名义类型与嵌套类型、返回类型全部保留；TypeRef 内
/// Ident 携带的 span 不再进入摘要（旧 `{:?}` 面把 span 一起编入，跨
/// owner 编辑的坐标漂移会误报 `OwnerSignatureChanged`）。
fn signature_of(f: &FnItem) -> String {
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| {
            let mode = match p.mode {
                ParamMode::Move => "move ",
                ParamMode::Mut => "mut ",
                ParamMode::Default => "",
            };
            format!("{}{}: {}", mode, p.name.name, type_digest(&p.ty.kind))
        })
        .collect();
    format!(
        "{}({}) -> {}",
        f.name.name,
        params.join(", "),
        type_digest(&f.ret.kind)
    )
}

/// PLAN-999（T-05）：类型注解的无坐标结构化投影。覆盖每一个携带
/// `Ident`（名字+span）或嵌套 `TypeAnn` 的变体；语义信息与旧 Debug 面
/// 一致，仅不编入 span。纯 int 例没有内层 Ident，区分不了本修复——
/// 反例必须含名义/嵌套注解（§5.4）。
fn type_digest(t: &crate::source::ast::TypeRef) -> String {
    use crate::source::ast::TypeRef;
    match t {
        TypeRef::Builtin(w) => format!("{w:?}"),
        TypeRef::TyVar(id) => format!("tv:{}", id.name),
        TypeRef::Record(id) => format!("rec:{}", id.name),
        TypeRef::Optional(inner) => format!("opt:{}", type_digest(&inner.kind)),
        TypeRef::Generic { name, args } => format!(
            "gen:{}<{}>",
            name.name,
            args.iter()
                .map(|a| type_digest(&a.kind))
                .collect::<Vec<_>>()
                .join(",")
        ),
        TypeRef::Function { params, result } => format!(
            "fn({}){}",
            params
                .iter()
                .map(|a| type_digest(&a.kind))
                .collect::<Vec<_>>()
                .join(","),
            type_digest(&result.kind)
        ),
    }
}

/// PLAN-929（T-07）：名字引用扫描扩为**全语法引用面**——注解（参数/返回/
/// let-var，含 Optional/Generic 嵌套）+ body 内全部名字叶（裸名、调用
/// callee、构造 `Type(...)`/`Type{...}`、方法名/静态基座）。语义依赖裁决
/// 仍归 resolve；本面只负责"谁提到这个名字"的语法事实（V-03 dependents）。
fn references_name(f: &FnItem, name: &str) -> bool {
    let ann = |t: &crate::source::ast::TypeAnn| -> bool {
        let mut stack = vec![t];
        while let Some(cur) = stack.pop() {
            if matches!(&cur.kind, crate::source::ast::TypeRef::Record(id) if id.name == name) {
                return true;
            }
            match &cur.kind {
                crate::source::ast::TypeRef::Optional(inner) => stack.push(inner),
                crate::source::ast::TypeRef::Generic { args, .. } => stack.extend(args.iter()),
                _ => {}
            }
        }
        false
    };
    if ann(&f.ret) {
        return true;
    }
    if f.params.iter().any(|p| ann(&p.ty)) {
        return true;
    }
    // 语句/表达式全走（显式栈——深链不爆栈；与 extract_lambda_body 同纪律）。
    let mut work: Vec<WalkItem> = f.body.stmts.iter().map(WalkItem::Stmt).collect();
    work.reverse();
    while let Some(item) = work.pop() {
        match item {
            WalkItem::Stmt(s) => {
                let mut sub: Vec<WalkItem> = Vec::new();
                match s {
                    crate::source::ast::Stmt::Let { ann: a, value, .. } => {
                        if let Some(ty) = a {
                            if ann(ty) {
                                return true;
                            }
                        }
                        sub.push(WalkItem::Expr(value));
                    }
                    crate::source::ast::Stmt::Assign { value, .. }
                    | crate::source::ast::Stmt::Return { value, .. }
                    | crate::source::ast::Stmt::ExprStmt { value, .. } => {
                        sub.push(WalkItem::Expr(value));
                    }
                    crate::source::ast::Stmt::If {
                        cond, then, els, ..
                    } => {
                        sub.push(WalkItem::Expr(cond));
                        sub.extend(then.stmts.iter().map(WalkItem::Stmt));
                        if let Some(e) = els {
                            sub.extend(e.stmts.iter().map(WalkItem::Stmt));
                        }
                    }
                    crate::source::ast::Stmt::While { cond, body, .. } => {
                        sub.push(WalkItem::Expr(cond));
                        sub.extend(body.stmts.iter().map(WalkItem::Stmt));
                    }
                    crate::source::ast::Stmt::Loop { body, .. }
                    | crate::source::ast::Stmt::ForIn { body, .. } => {
                        sub.extend(body.stmts.iter().map(WalkItem::Stmt));
                    }
                    crate::source::ast::Stmt::IsMatch { arms, .. } => {
                        for arm in arms {
                            sub.extend(arm.body.stmts.iter().map(WalkItem::Stmt));
                        }
                    }
                    crate::source::ast::Stmt::Break { .. }
                    | crate::source::ast::Stmt::Continue { .. } => {}
                }
                sub.reverse();
                work.extend(sub);
            }
            WalkItem::Expr(e) => {
                // 名字叶：裸名 / 调用 callee / 构造类型词 / 方法名。
                match &e.kind {
                    ExprKind::Var(i) => {
                        if i.name == name {
                            return true;
                        }
                    }
                    ExprKind::Call { callee, .. } => {
                        if callee.name == name {
                            return true;
                        }
                    }
                    ExprKind::Method { name: n, .. } => {
                        if n.name == name {
                            return true;
                        }
                    }
                    ExprKind::CtorNamed { ty, .. } => {
                        if ty.name == name {
                            return true;
                        }
                    }
                    _ => {}
                }
                let mut kids: Vec<&Expr> = expr_children(e);
                if let ExprKind::Closure { body: b, .. } = &e.kind {
                    if let ClosureBody::Block(blk) = &**b {
                        kids.extend(stmt_value_exprs_stmts(blk));
                    }
                }
                for k in kids.into_iter().rev() {
                    work.push(WalkItem::Expr(k));
                }
            }
        }
    }
    false
}

/// 块内语句的值表达式（闭包块走查用；复用 stmt_value_exprs 的判定面）。
fn stmt_value_exprs_stmts(b: &crate::source::ast::Block) -> Vec<&Expr> {
    let mut out = Vec::new();
    for s in &b.stmts {
        out.extend(stmt_value_exprs(s));
    }
    out
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
/// lambda；同层按源码序）。**显式工作栈**——访问不递归，深度只受解析预算
/// 限定的 AST 结构约束（深链不爆线程栈；计划 §5.3 深语法纪律）。
fn collect_lambdas(stmts: &[Stmt], out: &mut Vec<(Span, u64)>, src: &str) {
    let mut work: Vec<WalkItem<'_>> = stmts.iter().map(WalkItem::Stmt).collect();
    work.reverse();
    while let Some(item) = work.pop() {
        match item {
            WalkItem::Stmt(s) => {
                let mut sub: Vec<WalkItem<'_>> = Vec::new();
                match s {
                    Stmt::Let { value, .. }
                    | Stmt::Assign { value, .. }
                    | Stmt::Return { value, .. }
                    | Stmt::ExprStmt { value, .. } => work.push(WalkItem::Expr(value)),
                    Stmt::If {
                        cond, then, els, ..
                    } => {
                        work.push(WalkItem::Expr(cond));
                        sub.extend(then.stmts.iter().map(WalkItem::Stmt));
                        if let Some(e) = els {
                            sub.extend(e.stmts.iter().map(WalkItem::Stmt));
                        }
                    }
                    Stmt::While { cond, body, .. } => {
                        work.push(WalkItem::Expr(cond));
                        sub.extend(body.stmts.iter().map(WalkItem::Stmt));
                    }
                    Stmt::Loop { body, .. } | Stmt::ForIn { body, .. } => {
                        sub.extend(body.stmts.iter().map(WalkItem::Stmt));
                    }
                    Stmt::IsMatch { arms, .. } => {
                        for arm in arms {
                            sub.extend(arm.body.stmts.iter().map(WalkItem::Stmt));
                        }
                    }
                    Stmt::Break { .. } | Stmt::Continue { .. } => {}
                }
                sub.reverse();
                work.extend(sub);
            }
            WalkItem::Expr(e) => {
                if let ExprKind::Closure { .. } = &e.kind {
                    let slice = src.get(e.span.start..e.span.end).unwrap_or("");
                    out.push((e.span, content_sha(slice.as_bytes())));
                }
                let mut kids: Vec<&Expr> = expr_children(e);
                if let ExprKind::Closure { body: b, .. } = &e.kind {
                    if let ClosureBody::Block(blk) = &**b {
                        for st in &blk.stmts {
                            kids.extend(stmt_value_exprs(st));
                        }
                    }
                }
                for k in kids.into_iter().rev() {
                    work.push(WalkItem::Expr(k));
                }
            }
        }
    }
}

/// 走查工作项（语句 / 表达式）。
enum WalkItem<'a> {
    Stmt(&'a Stmt),
    Expr(&'a Expr),
}

/// 语句的直接值表达式。
fn stmt_value_exprs(s: &Stmt) -> Vec<&Expr> {
    match s {
        Stmt::Let { value, .. }
        | Stmt::Assign { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::ExprStmt { value, .. } => vec![value],
        _ => vec![],
    }
}

impl UnitCatalog {
    /// 冷构建：整源解析 + 全单元首次密封（各计生产 1）。
    pub fn build(
        src: &str,
        profile: SourceProfile,
        budget: FrontBudget,
    ) -> Result<UnitCatalog, EditError> {
        let program = parse_whole(src, profile, budget)?;
        let module = content_sha(src.as_bytes());
        let instance = INSTANCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut cat = UnitCatalog {
            module,
            instance,
            profile,
            fns: HashMap::new(),
            lambdas: HashMap::new(),
            decls: HashMap::new(),
            events: Vec::new(),
            meter: ProductionMeter::default(),
            src: src.to_string(),
        };
        for item in &program.items {
            // PLAN-928（T-06）：声明单元（type/enum）入目录（mint=冷坐标）。
            match item {
                Item::Type(t) => {
                    let sha = content_sha(
                        cat.src
                            .get(t.span.start..t.span.end)
                            .unwrap_or("")
                            .as_bytes(),
                    );
                    cat.decls.insert(
                        t.name.name.clone(),
                        SealedDecl {
                            content_sha: sha,
                            name: t.name.name.clone(),
                            kind: UnitKind::TypeDecl,
                            span: t.span,
                            mint_span: t.span,
                        },
                    );
                    cat.meter.produced += 1;
                }
                Item::Enum(e) => {
                    let sha = content_sha(
                        cat.src
                            .get(e.span.start..e.span.end)
                            .unwrap_or("")
                            .as_bytes(),
                    );
                    cat.decls.insert(
                        e.name.name.clone(),
                        SealedDecl {
                            content_sha: sha,
                            name: e.name.name.clone(),
                            kind: UnitKind::EnumDecl,
                            span: e.span,
                            mint_span: e.span,
                        },
                    );
                    cat.meter.produced += 1;
                }
                _ => {}
            }
            if let Item::Fn(f) = item {
                if f.script {
                    continue;
                }
                let owner = f.name.name.clone();
                let sha = content_sha(
                    cat.src
                        .get(f.span.start..f.span.end)
                        .unwrap_or("")
                        .as_bytes(),
                );
                cat.fns.insert(
                    owner.clone(),
                    SealedFn {
                        content_sha: sha,
                        item: Some(Rc::new(f.clone())),
                        mint_span: f.span,
                        span: f.span,
                    },
                );
                cat.meter.produced += 1;
                let mut lambdas = Vec::new();
                collect_lambdas(&f.body.stmts, &mut lambdas, &cat.src);
                for (ordinal, (span, lsha)) in lambdas.into_iter().enumerate() {
                    // 从已密封 owner 体中取对应 lambda 语法体（同一次遍历序）。
                    let body = extract_lambda_body(
                        cat.fns[&owner]
                            .item
                            .as_deref()
                            .expect("cold owner materialized"),
                        ordinal,
                    )
                    .expect("lambda ordinal matches collection order");
                    cat.lambdas.insert(
                        (owner.clone(), ordinal as u32),
                        SealedLambda {
                            content_sha: lsha,
                            body: Some(Rc::new(body)),
                            span,
                        },
                    );
                    cat.meter.produced += 1;
                }
            }
        }
        Ok(cat)
    }

    /// PLAN-929（T-02，§5.6）：**恢复构造**——从（调用方随恢复请求提供的）
    /// 完整源文本 + 解码清单重建目录，**零源码生产**：
    ///
    /// - 深重验全部为字节/坐标面（R 轴，如实非零但不解析）：schema/caps/
    ///   profile 合同、module 指纹、逐单元 `current_span` 切片指纹、span
    ///   边界/重叠/重复键；
    /// - 全部单元以**身份-only**（惰性 body，`None`）入目录——不物化任何
    ///   语法体，不为补上下文扫描未改函数；
    /// - 计量归零（本进程无生产）；`instance` 为新铸（跨目录 foreign 候选
    ///   判别沿 928 合同）；
    /// - 任何不匹配在使用前受控拒绝（`RestoreError`，V-05 分类面）；拒绝
    ///   不产生半恢复态。
    ///
    /// 恢复后的第一次编辑按单元走片段入口（`edit_lambda`/`edit_owner`/
    /// `stage_owner_edit`），恰目标物化；未改单元保持 `None`（生产 0）。
    pub fn from_restored(
        src: &str,
        manifest: &DecodedManifest,
        profile: SourceProfile,
        caps_version: &str,
    ) -> Result<UnitCatalog, RestoreError> {
        if manifest.schema != MANIFEST_SCHEMA {
            return Err(RestoreError::SchemaMismatch {
                found: manifest.schema.clone(),
            });
        }
        if manifest.caps_version != caps_version {
            return Err(RestoreError::CapsMismatch {
                manifest: manifest.caps_version.clone(),
                expected: caps_version.to_string(),
            });
        }
        let expected_slug = super::frontdoor::profile_slug(profile);
        if manifest.profile != expected_slug {
            return Err(RestoreError::ProfileMismatch {
                manifest: manifest.profile.clone(),
                expected: expected_slug.to_string(),
            });
        }
        let module = content_sha(src.as_bytes());
        if manifest.module_sha != module {
            return Err(RestoreError::ModuleHashMismatch {
                manifest: manifest.module_sha,
                source: module,
            });
        }
        let instance = INSTANCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut cat = UnitCatalog {
            module,
            instance,
            profile,
            fns: HashMap::new(),
            lambdas: HashMap::new(),
            decls: HashMap::new(),
            events: Vec::new(),
            meter: ProductionMeter::default(),
            src: src.to_string(),
        };
        // 坐标结构健全性收集面（排序后查重叠）。
        let mut spans: Vec<(usize, usize, String, UnitKind)> = Vec::new();
        for u in &manifest.units {
            let (cs, ce) = u.current_span;
            if ce > src.len() {
                return Err(RestoreError::SpanOutOfBounds {
                    owner: u.owner.clone(),
                    kind: kind_token(u.kind),
                    start: cs,
                    end: ce,
                    src_len: src.len(),
                });
            }
            let slice_sha = content_sha(src.get(cs..ce).unwrap_or("").as_bytes());
            if slice_sha != u.content_sha {
                return Err(RestoreError::UnitHashMismatch {
                    owner: u.owner.clone(),
                    kind: kind_token(u.kind),
                    ordinal: u.ordinal,
                    manifest: u.content_sha,
                    source: slice_sha,
                });
            }
            let span = Span { start: cs, end: ce };
            let mint = Span {
                start: u.mint_span.0,
                end: u.mint_span.1,
            };
            let dup = |kind: &'static str| RestoreError::DuplicateUnit {
                owner: u.owner.clone(),
                kind,
                ordinal: u.ordinal,
            };
            match u.kind {
                UnitKind::Fn => {
                    if cat
                        .fns
                        .insert(
                            u.owner.clone(),
                            SealedFn {
                                content_sha: u.content_sha,
                                item: None,
                                mint_span: mint,
                                span,
                            },
                        )
                        .is_some()
                    {
                        return Err(dup("fn"));
                    }
                }
                UnitKind::Lambda => {
                    if cat
                        .lambdas
                        .insert(
                            (u.owner.clone(), u.ordinal),
                            SealedLambda {
                                content_sha: u.content_sha,
                                body: None,
                                span,
                            },
                        )
                        .is_some()
                    {
                        return Err(dup("lambda"));
                    }
                }
                UnitKind::TypeDecl | UnitKind::EnumDecl => {
                    if cat
                        .decls
                        .insert(
                            u.owner.clone(),
                            SealedDecl {
                                content_sha: u.content_sha,
                                name: u.owner.clone(),
                                kind: u.kind,
                                span,
                                mint_span: mint,
                            },
                        )
                        .is_some()
                    {
                        return Err(dup(kind_token(u.kind)));
                    }
                }
            }
            spans.push((cs, ce, u.owner.clone(), u.kind));
        }
        // 坐标结构健全性（类型感知）：唯一合法嵌套 = Fn ⊇ 其 Lambda；
        // Fn/Decl 顶层互斥（Fn 套 Fn、Decl 套任何、λ 包含任何 = 伪造/交叉，
        // 一律拒）。栈式扫描（start 升序/end 降序）：弹出已闭合单元后，
        // 若当前单元起点仍在栈顶内，则栈顶必须是 Fn 且当前必须是 Lambda，
        // 且不得越过栈顶 end（部分交叉）。
        spans.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(b.1.cmp(&a.1))
                .then((a.3 as u8).cmp(&(b.3 as u8)))
        });
        let mut open: Vec<(usize, usize, String, UnitKind)> = Vec::new();
        for (start, end, owner, kind) in spans {
            while let Some(top) = open.last() {
                if top.1 <= start {
                    open.pop();
                } else {
                    break;
                }
            }
            if let Some(top) = open.last() {
                let nested = start < top.1 && end <= top.1;
                if !nested || top.3 != UnitKind::Fn || kind != UnitKind::Lambda {
                    return Err(RestoreError::OverlappingUnits {
                        a: top.2.clone(),
                        b: owner,
                    });
                }
            }
            open.push((start, end, owner, kind));
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
        // PLAN-928（T-09，V-03）：零编辑重放——新切片与当前字节全等时
        // 不重解析/不物化：全部单元复用（生产 0），身份不变，无语义重验
        // 义务（源未变 ⇒ 无新语义事实）。
        let old_slice = self
            .src
            .get(old.span.start as usize..old.span.end as usize)
            .unwrap_or("")
            .to_string();
        if old_slice.as_str() == new_slice {
            let keys: Vec<UnitKey> = self
                .lambdas
                .keys()
                .filter(|k| k.0 == owner)
                .map(|k| UnitKey {
                    module: self.module,
                    owner: k.0.clone(),
                    kind: UnitKind::Lambda,
                    ordinal: k.1,
                })
                .collect();
            self.meter.reused += keys.len();
            return Ok(EditReceipt {
                new_src: self.src.clone(),
                produced: Vec::new(),
                reused: keys,
                semantic_recheck_required: false,
                non_fast_path: None,
            });
        }
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
                // T-09：漂移维护——编辑点之后的单元当前坐标平移 delta；
                // owner 包装 span 的 end 同步延长（编辑发生在 owner 内）。
                let delta = new_slice.len() as i64 - (old.span.end - old.span.start) as i64;
                let boundary = old.span.end;
                self.shift_units_after(boundary, delta);
                if let Some(owner_sealed) = self.fns.get_mut(owner) {
                    if owner_sealed.span.start <= new_start && owner_sealed.span.end >= boundary {
                        owner_sealed.span.end = (owner_sealed.span.end as i64 + delta) as usize;
                    }
                }
                let sealed = SealedLambda {
                    content_sha: sha,
                    body: Some(Rc::new(body)),
                    span: Span {
                        start: new_start,
                        end: new_start + new_slice.len(),
                    },
                };
                self.lambdas.insert(key.clone(), sealed);
                self.src = new_src.clone();
                self.meter.produced += 1;
                // 未触兄弟 λ 保持 Rc 共享（0 生产）——如实计入复用。
                let siblings = self
                    .lambdas
                    .keys()
                    .filter(|k| k.0 == owner && k.1 != ordinal)
                    .count();
                self.meter.reused += siblings;
                // owner 内容已变（lambda 文本是 owner 切片的一部分）。
                self.reseal_owner_span(owner);
                Ok(EditReceipt {
                    new_src,
                    produced: vec![UnitKey {
                        module: self.module,
                        owner: owner.to_string(),
                        kind: UnitKind::Lambda,
                        ordinal,
                    }],
                    reused: self
                        .lambdas
                        .keys()
                        .filter(|k| k.0 == owner && k.1 != ordinal)
                        .map(|k| UnitKey {
                            module: self.module,
                            owner: k.0.clone(),
                            kind: UnitKind::Lambda,
                            ordinal: k.1,
                        })
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
    pub fn edit_owner(
        &mut self,
        owner: &str,
        new_fn_src: &str,
        budget: FrontBudget,
    ) -> Result<EditReceipt, EditError> {
        let old = self
            .fns
            .get(owner)
            .ok_or_else(|| EditError::Infra(format!("unknown owner {owner}")))?;
        let old_span = old.span;
        // PLAN-928（T-09，V-03）：零编辑重放——owner 切片字节全等 ⇒ 不重解
        // 析/不物化（owner 与全部子 λ 生产 0），身份不变，无语义重验义务。
        {
            let old_slice = self
                .src
                .get(old_span.start as usize..old_span.end as usize)
                .unwrap_or("");
            if old_slice == new_fn_src {
                let mut keys = vec![UnitKey {
                    module: self.module,
                    owner: owner.to_string(),
                    kind: UnitKind::Fn,
                    ordinal: 0,
                }];
                keys.extend(
                    self.lambdas
                        .keys()
                        .filter(|k| k.0 == owner)
                        .map(|k| UnitKey {
                            module: self.module,
                            owner: k.0.clone(),
                            kind: UnitKind::Lambda,
                            ordinal: k.1,
                        }),
                );
                self.meter.reused += keys.len();
                return Ok(EditReceipt {
                    new_src: self.src.clone(),
                    produced: Vec::new(),
                    reused: keys,
                    semantic_recheck_required: false,
                    non_fast_path: None,
                });
            }
        }
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
                // PLAN-928（T-06）：签名变化事件（参数/返回/mode 面变化——
                // 不能因调用点文本未改而静默；§5.2）。
                // PLAN-929（T-02）：恢复目录旧 owner 未物化（None）⇒ 签名面
                // 不可比——保守置 true（事件必达，依赖者重验；§5.2 不静默）。
                let signature_changed = owner_changed
                    && self
                        .fns
                        .get(owner)
                        .map(|old| match old.item.as_deref() {
                            Some(old_item) => signature_of(old_item) != signature_of(&item),
                            None => true,
                        })
                        .unwrap_or(true);
                if signature_changed {
                    self.events.push(CatalogEvent::OwnerSignatureChanged {
                        owner: owner.to_string(),
                    });
                }
                let prev_mint = self.fns.get(owner).map(|o| o.mint_span);
                // T-09：漂移维护——编辑点之后单元当前坐标平移。
                let delta = new_fn_src.len() as i64 - (old_span.end - old_span.start) as i64;
                self.shift_units_after(old_span.end, delta);
                self.fns.insert(
                    owner.to_string(),
                    SealedFn {
                        content_sha: owner_sha,
                        item: Some(Rc::new(item.clone())),
                        mint_span: prev_mint.unwrap_or(item.span),
                        span: Span {
                            start: old_span.start,
                            end: old_span.start + new_fn_src.len(),
                        },
                    },
                );
                self.meter.produced += 1;
                // 遍历新 owner 的 lambda：同文（content sha 相等）=> 复用旧 Rc
                // （生产 0）；文变 => 新密封（生产 1）。序漂移且文同 => 具名
                // 非快路径（整 owner 重产，如实计数——§5.4 不得假 0）。
                let mut lambdas = Vec::new();
                collect_lambdas(&item.body.stmts, &mut lambdas, &new_src);
                let old_count = self.lambdas.keys().filter(|k| k.0 == owner).count();
                let mut produced = vec![UnitKey {
                    module: self.module,
                    owner: owner.to_string(),
                    kind: UnitKind::Fn,
                    ordinal: 0,
                }];
                let mut reused = Vec::new();
                let mut non_fast: Option<&'static str> = None;
                if lambdas.len() == old_count {
                    for (ordinal, (span, lsha)) in lambdas.into_iter().enumerate() {
                        let key = (owner.to_string(), ordinal as u32);
                        let body = extract_lambda_body(&item, ordinal).expect("ordinal order");
                        match self.lambdas.get(&key) {
                            Some(prev) if prev.content_sha == lsha => {
                                // 语法复用：共享 Rc（恢复目录 None 保持 None——
                                // 未改单元物化 0，§5.6），仅坐标漂移（身份不变）。
                                let mut updated = SealedLambda {
                                    content_sha: lsha,
                                    body: prev.body.clone(),
                                    span,
                                };
                                updated.span = span;
                                self.lambdas.insert(key.clone(), updated);
                                self.meter.reused += 1;
                                reused.push(UnitKey {
                                    module: self.module,
                                    owner: owner.to_string(),
                                    kind: UnitKind::Lambda,
                                    ordinal: ordinal as u32,
                                });
                            }
                            _ => {
                                self.lambdas.insert(
                                    key.clone(),
                                    SealedLambda {
                                        content_sha: lsha,
                                        body: Some(Rc::new(body)),
                                        span,
                                    },
                                );
                                self.meter.produced += 1;
                                produced.push(UnitKey {
                                    module: self.module,
                                    owner: owner.to_string(),
                                    kind: UnitKind::Lambda,
                                    ordinal: ordinal as u32,
                                });
                            }
                        }
                    }
                } else {
                    // 拓扑变化（数量不等）→ 具名非快路径：整 owner lambda 重产。
                    non_fast = Some("owner-lambda-topology-changed");
                    self.events.push(CatalogEvent::LambdaTopologyChanged {
                        owner: owner.to_string(),
                    });
                    self.lambdas.retain(|k, _| k.0 != owner);
                    for (ordinal, (span, lsha)) in lambdas.into_iter().enumerate() {
                        let body = extract_lambda_body(&item, ordinal).expect("ordinal order");
                        self.lambdas.insert(
                            (owner.to_string(), ordinal as u32),
                            SealedLambda {
                                content_sha: lsha,
                                body: Some(Rc::new(body)),
                                span,
                            },
                        );
                        self.meter.produced += 1;
                        produced.push(UnitKey {
                            module: self.module,
                            owner: owner.to_string(),
                            kind: UnitKind::Lambda,
                            ordinal: ordinal as u32,
                        });
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
            let span = sealed.span;
            let sha = content_sha(self.src.get(span.start..span.end).unwrap_or("").as_bytes());
            sealed.content_sha = sha;
        }
    }

    /// PLAN-928（T-09，V-06）：坐标漂移维护——`boundary` 起的后续单元的
    /// **当前坐标**（包装字段）平移 `delta`。mint 身份、内容指纹与 Rc 载
    /// 载荷一律不动（pin 视图不受影响；§5.4 禁 mutate 共享节点 span）。
    fn shift_units_after(&mut self, boundary: usize, delta: i64) {
        if delta == 0 {
            return;
        }
        let shift = |s: &mut Span| {
            if s.start >= boundary {
                s.start = (s.start as i64 + delta) as usize;
                s.end = (s.end as i64 + delta) as usize;
            }
        };
        for sealed in self.fns.values_mut() {
            shift(&mut sealed.span);
        }
        for sealed in self.lambdas.values_mut() {
            shift(&mut sealed.span);
        }
        for sealed in self.decls.values_mut() {
            shift(&mut sealed.span);
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
        let old_span = sealed.span;
        // PLAN-928（T-09，V-03）：零编辑重放（暂存面）——owner 切片字节全等
        // ⇒ 候选 produced=0、全部复用、无语义重验义务。
        {
            let old_slice = self
                .src
                .get(old_span.start as usize..old_span.end as usize)
                .unwrap_or("");
            if old_slice == new_fn_src {
                let lambdas: Vec<(u32, SealedLambda)> = self
                    .lambdas
                    .iter()
                    .filter(|(k, _)| k.0 == owner)
                    .map(|(k, v)| {
                        (
                            k.1,
                            SealedLambda {
                                content_sha: v.content_sha,
                                body: v.body.clone(),
                                span: v.span,
                            },
                        )
                    })
                    .collect();
                let n = lambdas.len();
                return Ok(StagedCandidate {
                    owner: owner.to_string(),
                    new_src: self.src.clone(),
                    owner_sha: sealed.content_sha,
                    owner_item: sealed.item.clone(),
                    lambdas,
                    produced: 0,
                    reused: 1 + n,
                    semantic_recheck: Vec::new(),
                    changed: false,
                    base_revision: content_sha(self.src.as_bytes()),
                    instance: self.instance,
                    new_fn_len: new_fn_src.len(),
                });
            }
        }
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
                                SealedLambda {
                                    content_sha: *lsha,
                                    body: prev.body.clone(),
                                    span: *span,
                                },
                            ));
                            reused += 1;
                            semantic_recheck.push(ordinal as u32);
                        }
                        _ => {
                            let body = extract_lambda_body(&item, ordinal).expect("ordinal order");
                            staged_lambdas.push((
                                ordinal as u32,
                                SealedLambda {
                                    content_sha: *lsha,
                                    body: Some(Rc::new(body)),
                                    span: *span,
                                },
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
                    owner_item: Some(Rc::new(item)),
                    lambdas: staged_lambdas,
                    produced,
                    reused,
                    semantic_recheck,
                    changed: true,
                    base_revision: content_sha(self.src.as_bytes()),
                    instance: self.instance,
                    new_fn_len: new_fn_src.len(),
                })
            }
            FrontOutcome::Diag(ds) => Err(EditError::Diag(ds)),
            FrontOutcome::ResourceExhausted { what, limit } => {
                Err(EditError::ResourceExhausted { what, limit })
            }
            FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => Err(EditError::Infra(m)),
        }
    }

    /// Materialize only an actual declaration reader, including immediately
    /// after restore. Reuse a current-span sealed AST when available; otherwise
    /// use the same fragment parser/projector as an ordinary owner edit.
    pub fn owner_for_recheck(
        &self,
        owner: &str,
        budget: FrontBudget,
    ) -> Result<(Rc<FnItem>, bool), EditError> {
        self.owner_for_staged_recheck(owner, None, budget)
    }

    fn reader_span(&self, owner: &str, cand: Option<&StagedCandidate>) -> Result<Span, EditError> {
        let mut span = self
            .fns
            .get(owner)
            .ok_or_else(|| EditError::Infra(format!("unknown owner {owner}")))?
            .span;
        if let Some(cand) = cand {
            self.validate_staged(cand)?;
            let old = self.fns[&cand.owner].span;
            if owner == cand.owner {
                span.end = span.start + cand.new_fn_len;
            } else if span.start >= old.end {
                let delta = cand.new_fn_len as i64 - (old.end - old.start) as i64;
                let shift = |n: usize| {
                    usize::try_from(n as i128 + delta as i128)
                        .map_err(|_| EditError::Infra("candidate reader span overflow".into()))
                };
                span = Span {
                    start: shift(span.start)?,
                    end: shift(span.end)?,
                };
            }
        }
        Ok(span)
    }
    pub fn reader_projection_required(
        &self,
        owner: &str,
        cand: Option<&StagedCandidate>,
    ) -> Result<bool, EditError> {
        let span = self.reader_span(owner, cand)?;
        if let Some(cand) = cand.filter(|c| c.owner == owner) {
            return Ok(cand.owner_item.is_none());
        }
        Ok(self.fns[owner].item.as_ref().is_none_or(|f| f.span != span))
    }
    /// Project just this reader at its candidate source position. Prefix drift
    /// never causes unrelated ASTs to be cloned or materialized.
    pub fn owner_for_staged_recheck(
        &self,
        owner: &str,
        cand: Option<&StagedCandidate>,
        budget: FrontBudget,
    ) -> Result<(Rc<FnItem>, bool), EditError> {
        let span = self.reader_span(owner, cand)?;
        if let Some(item) = cand
            .filter(|c| c.owner == owner)
            .and_then(|c| c.owner_item.as_ref())
        {
            return Ok((item.clone(), false));
        }
        if let Some(item) = &self.fns[owner].item {
            if item.span == span {
                return Ok((item.clone(), false));
            }
        }
        let src = cand.map(|c| c.new_src.as_str()).unwrap_or(&self.src);
        match parse_front(FrontRequest {
            src,
            profile: self.profile,
            kind: FrontKind::FreeFnFragment {
                range: span.start..span.end,
            },
            budget,
        }) {
            FrontOutcome::Ok(ok) => match ok.program.items.into_iter().next() {
                Some(Item::Fn(item)) if item.name.name == owner => Ok((Rc::new(item), true)),
                _ => Err(EditError::Infra("reader fragment carried no owner".into())),
            },
            FrontOutcome::Diag(ds) => Err(EditError::Diag(ds)),
            FrontOutcome::ResourceExhausted { what, limit } => {
                Err(EditError::ResourceExhausted { what, limit })
            }
            FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => Err(EditError::Infra(m)),
        }
    }

    /// 提交暂存候选（当前修订推进；计量按暂存面如实入账）。
    ///
    /// PLAN-928（T-09，V-05/§5.4）：stale（候选基修订 ≠ 当前修订）与
    /// foreign（候选非本目录实例铸造）一律 typed 拒绝（`StaleForeign`），
    /// 目录/源/计量**零变异**后原样返回。
    pub fn validate_staged(&self, cand: &StagedCandidate) -> Result<(), EditError> {
        if cand.instance != self.instance {
            return Err(EditError::StaleForeign(format!(
                "foreign candidate: minted by catalog instance {}, this is {}",
                cand.instance, self.instance
            )));
        }
        let current_revision = content_sha(self.src.as_bytes());
        if cand.base_revision != current_revision {
            return Err(EditError::StaleForeign(format!(
                "stale candidate: staged against revision {:#x}, current is {:#x}",
                cand.base_revision, current_revision
            )));
        }
        Ok(())
    }

    pub fn commit_staged(&mut self, cand: StagedCandidate) -> Result<EditReceipt, EditError> {
        self.validate_staged(&cand)?;
        let is_reuse = |cat: &Self, owner: &str, ordinal: u32, sl: &SealedLambda| {
            cat.lambdas
                .get(&(owner.to_string(), ordinal))
                .map(|p| {
                    p.content_sha == sl.content_sha
                        && match (&p.body, &sl.body) {
                            // 冷建路径：Rc 同一（真实共享证明）。
                            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                            // 恢复路径：两侧都未物化（同 sha 且零物化 ⇒ 复用成立）。
                            (None, None) => true,
                            _ => false,
                        }
                })
                .unwrap_or(false)
        };
        let produced_keys: Vec<UnitKey> = std::iter::once(UnitKey {
            module: self.module,
            owner: cand.owner.clone(),
            kind: UnitKind::Fn,
            ordinal: 0,
        })
        .chain(
            cand.lambdas
                .iter()
                .filter(|(o, sl)| !is_reuse(self, &cand.owner, *o, sl))
                .map(|(o, _)| UnitKey {
                    module: self.module,
                    owner: cand.owner.clone(),
                    kind: UnitKind::Lambda,
                    ordinal: *o,
                }),
        )
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
        let prev_mint = self.fns.get(&cand.owner).map(|o| o.mint_span);
        // 零编辑候选（changed=false）：源/身份零变化——诚实回执（生产 0、
        // 全部复用、无事件、无重验义务），目录内容不动。
        if !cand.changed {
            self.meter.reused += cand.reused;
            let mut keys = vec![UnitKey {
                module: self.module,
                owner: cand.owner.clone(),
                kind: UnitKind::Fn,
                ordinal: 0,
            }];
            keys.extend(cand.lambdas.iter().map(|(o, _)| UnitKey {
                module: self.module,
                owner: cand.owner.clone(),
                kind: UnitKind::Lambda,
                ordinal: *o,
            }));
            return Ok(EditReceipt {
                new_src: cand.new_src,
                produced: Vec::new(),
                reused: keys,
                semantic_recheck_required: false,
                non_fast_path: None,
            });
        }
        // PLAN-928（T-06）：签名变化事件（提交面；与直接路径同判定）。
        // PLAN-929（T-02）：旧 owner 未物化（恢复目录 None）⇒ 签名面不可比，
        // 保守置"变"（事件必达；§5.2 不静默）。真实编辑候选恒携带 owner item。
        if let Some(new_item) = cand.owner_item.as_deref() {
            let changed = self
                .fns
                .get(&cand.owner)
                .map(|old| match old.item.as_deref() {
                    Some(old_item) => signature_of(old_item) != signature_of(new_item),
                    None => true,
                })
                .unwrap_or(true);
            if changed {
                self.events.push(CatalogEvent::OwnerSignatureChanged {
                    owner: cand.owner.clone(),
                });
            }
        }
        // T-09：提交面漂移维护 + λ 拓扑事件（与直路 edit_owner 同判定）。
        let old_span = self
            .fns
            .get(&cand.owner)
            .map(|o| o.span)
            .unwrap_or(Span { start: 0, end: 0 });
        let old_count = self.lambdas.keys().filter(|k| k.0 == cand.owner).count();
        if cand.lambdas.len() != old_count {
            self.events.push(CatalogEvent::LambdaTopologyChanged {
                owner: cand.owner.clone(),
            });
        }
        let delta = cand.new_fn_len as i64 - (old_span.end - old_span.start) as i64;
        self.shift_units_after(old_span.end, delta);
        self.fns.insert(
            cand.owner.clone(),
            SealedFn {
                content_sha: cand.owner_sha,
                item: cand.owner_item,
                mint_span: prev_mint.unwrap_or(Span { start: 0, end: 0 }),
                span: Span {
                    start: old_span.start,
                    end: old_span.start + cand.new_fn_len,
                },
            },
        );
        // 注：changed=true 候选恒 Some；上方零编辑分支已提前返回。
        self.lambdas.retain(|k, _| k.0 != cand.owner);
        for (ordinal, sl) in cand.lambdas {
            self.lambdas.insert((cand.owner.clone(), ordinal), sl);
        }
        self.meter.produced += cand.produced;
        self.meter.reused += cand.reused;
        self.src = cand.new_src.clone();
        Ok(EditReceipt {
            new_src: cand.new_src,
            produced: produced_keys,
            reused: reused_keys,
            semantic_recheck_required: true,
            non_fast_path: None,
        })
    }

    /// 显式 pin：目录载荷的修订局部快照。旧 pin 只在其原修订视图读取；
    /// 跨修订消费按内容指纹由消费方校验（V-10：current/history/pin 同时
    /// 存活、互不串用）。
    /// PLAN-929（T-02）：仅含**已物化**载荷——恢复目录的未物化单元不进
    /// pin（无语法体可共享；身份/坐标面由清单承载）。
    pub fn pin(&self) -> CatalogPin {
        CatalogPin {
            revision_src: self.src.clone(),
            fns: self
                .fns
                .iter()
                .filter_map(|(k, v)| v.item.as_ref().map(|r| (k.clone(), Rc::clone(r))))
                .collect(),
            lambdas: self
                .lambdas
                .iter()
                .filter_map(|(k, v)| v.body.as_ref().map(|r| (k.clone(), Rc::clone(r))))
                .collect(),
        }
    }
}

/// 暂存候选（未提交；目录/源不变）。
pub struct StagedCandidate {
    owner: String,
    new_src: String,
    owner_sha: u64,
    /// PLAN-929（T-02）：`None` = 零编辑候选在恢复目录上暂存（owner 未物化，
    /// 生产 0、无解析）；`Some` = 真实编辑候选（暂存面已解析/投影）。
    owner_item: Option<Rc<FnItem>>,
    lambdas: Vec<(u32, SealedLambda)>,
    produced: usize,
    reused: usize,
    semantic_recheck: Vec<u32>,
    /// PLAN-928（T-09）：候选携带铸造时的基修订指纹与目录实例身份——
    /// `commit_staged` 据此拒绝 stale/foreign（§5.4/§5.2d）。
    changed: bool,
    base_revision: u64,
    instance: u64,
    /// 新 owner 片段长度（提交面漂移维护与拓扑判定用）。
    new_fn_len: usize,
}

impl StagedCandidate {
    pub fn owner_item(&self) -> Option<&FnItem> {
        self.owner_item.as_deref()
    }
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

fn parse_whole(
    src: &str,
    profile: SourceProfile,
    budget: FrontBudget,
) -> Result<Program, EditError> {
    match parse_front(FrontRequest {
        src,
        profile,
        kind: FrontKind::WholeModule,
        budget,
    }) {
        FrontOutcome::Ok(ok) => Ok(ok.program),
        FrontOutcome::Diag(ds) => Err(EditError::Diag(ds)),
        FrontOutcome::ResourceExhausted { what, limit } => {
            Err(EditError::ResourceExhausted { what, limit })
        }
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => Err(EditError::Infra(m)),
    }
}

/// 单 lambda 片段经共享链重解析：合成 `fn __lambda_unit() int { return <slice> }`
/// 宿主走统一前门（同 grammar/同投影/同预算），取出 return 位表达式并平移
/// 回片内坐标。返回 (expr, 消费长度)。
fn parse_lambda_unit(
    slice: &str,
    profile: SourceProfile,
    budget: FrontBudget,
) -> Result<(Expr, usize), EditError> {
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
    shift_expr(&mut value, -(WRAP_PREFIX_LEN as i64));
    Ok((value, slice.len()))
}

/// 表达式全跨度平移（有符号 delta；负=回退到片段相对坐标，正=提升到
/// 原源绝对坐标；饱和到 0）。PLAN-929（T-03）：泛化为统一片段桥共用。
pub(crate) fn shift_expr(e: &mut Expr, delta: i64) {
    let sh = |v: usize| ((v as i64 + delta).max(0)) as usize;
    let mut work = vec![e];
    while let Some(cur) = work.pop() {
        cur.span = Span {
            start: sh(cur.span.start),
            end: sh(cur.span.end),
        };
        let mut children: Vec<&mut Expr> = Vec::new();
        fn push_args<'a>(
            args: &'a mut [crate::source::ast::Arg],
            children: &mut Vec<&'a mut Expr>,
        ) {
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
/// 外层先、块内嵌套随后；**显式工作栈**——同上深度纪律）。
fn extract_lambda_body(f: &FnItem, ordinal: usize) -> Option<ClosureBody> {
    let mut remaining = ordinal;
    let mut work: Vec<WalkItem<'_>> = f.body.stmts.iter().map(WalkItem::Stmt).collect();
    work.reverse();
    while let Some(item) = work.pop() {
        match item {
            WalkItem::Stmt(s) => {
                let mut sub: Vec<WalkItem<'_>> = Vec::new();
                match s {
                    Stmt::Let { value, .. }
                    | Stmt::Assign { value, .. }
                    | Stmt::Return { value, .. }
                    | Stmt::ExprStmt { value, .. } => work.push(WalkItem::Expr(value)),
                    Stmt::If {
                        cond, then, els, ..
                    } => {
                        work.push(WalkItem::Expr(cond));
                        sub.extend(then.stmts.iter().map(WalkItem::Stmt));
                        if let Some(e) = els {
                            sub.extend(e.stmts.iter().map(WalkItem::Stmt));
                        }
                    }
                    Stmt::While { cond, body, .. } => {
                        work.push(WalkItem::Expr(cond));
                        sub.extend(body.stmts.iter().map(WalkItem::Stmt));
                    }
                    Stmt::Loop { body, .. } | Stmt::ForIn { body, .. } => {
                        sub.extend(body.stmts.iter().map(WalkItem::Stmt));
                    }
                    Stmt::IsMatch { arms, .. } => {
                        for arm in arms {
                            sub.extend(arm.body.stmts.iter().map(WalkItem::Stmt));
                        }
                    }
                    Stmt::Break { .. } | Stmt::Continue { .. } => {}
                }
                sub.reverse();
                work.extend(sub);
            }
            WalkItem::Expr(e) => {
                if let ExprKind::Closure { body, .. } = &e.kind {
                    if remaining == 0 {
                        return Some((**body).clone());
                    }
                    remaining -= 1;
                }
                let mut kids: Vec<&Expr> = expr_children(e);
                if let ExprKind::Closure { body: b, .. } = &e.kind {
                    if let ClosureBody::Block(blk) = &**b {
                        for st in &blk.stmts {
                            kids.extend(stmt_value_exprs(st));
                        }
                    }
                }
                for k in kids.into_iter().rev() {
                    work.push(WalkItem::Expr(k));
                }
            }
        }
    }
    None
}
// ============================================================================
// PLAN-928（T-09，V-08）：清单 codec——确定性行式文本编码 + 消费前校验解码。
// 来源结构与版本在消费前拒绝：坏 schema/未知 profile/未知 kind/坏数字/坏
// span 一律 Err；解码结果是 owned 镜像（不信任任何来源），与 `to_manifest`
// 的当次输出全等才可作为目录身份面（恢复深验/全编译来源恢复 P0 归 929）。
// ============================================================================

/// 解码镜像（owned；`matches_manifest` 与当次导出全等判等）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedManifest {
    pub schema: String,
    pub module_sha: u64,
    pub caps_version: String,
    pub profile: String,
    pub units: Vec<ManifestUnit>,
}

impl DecodedManifest {
    /// 与目录当次导出的清单全等（身份面重验——伪造/近邻在此拒绝）。
    pub fn matches_manifest(&self, m: &CatalogManifest) -> bool {
        self.schema == m.schema
            && self.module_sha == m.module_sha
            && self.caps_version == m.caps_version
            && self.profile == m.profile
            && self.units == m.units
    }
}

fn kind_token(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::Fn => "fn",
        UnitKind::Lambda => "lambda",
        UnitKind::TypeDecl => "typedecl",
        UnitKind::EnumDecl => "enumdecl",
    }
}

fn kind_from_token(tok: &str) -> Option<UnitKind> {
    Some(match tok {
        "fn" => UnitKind::Fn,
        "lambda" => UnitKind::Lambda,
        "typedecl" => UnitKind::TypeDecl,
        "enumdecl" => UnitKind::EnumDecl,
        _ => return None,
    })
}

/// 编码：首行 schema，次行 module/caps/profile 头，随后每单元一行。
/// 全部字段为无空格 token + 十进制数（确定性：同清单同字节）。
pub fn encode_manifest(m: &CatalogManifest) -> String {
    let mut out = String::new();
    out.push_str(m.schema);
    out.push('\n');
    out.push_str(&format!("module {}\n", m.module_sha));
    out.push_str(&format!("caps {}\n", m.caps_version));
    out.push_str(&format!("profile {}\n", m.profile));
    for u in &m.units {
        out.push_str(&format!(
            "unit {} {} {} {} {} {} {} {}\n",
            kind_token(u.kind),
            u.owner,
            u.ordinal,
            u.content_sha,
            u.mint_span.0,
            u.mint_span.1,
            u.current_span.0,
            u.current_span.1
        ));
    }
    out
}

/// 解码 + 消费前校验（V-08：来源结构/版本拒绝；错行号报告）。
pub fn decode_manifest(text: &str) -> Result<DecodedManifest, String> {
    let bad = |line: usize, why: &str| format!("manifest line {line}: {why}");
    // 恰允许一个尾随换行；中段空行/多余尾随垃圾一律拒绝。
    let body = text.strip_suffix('\n').unwrap_or(text);
    if body.contains("\n\n") || body.ends_with('\n') || body.is_empty() {
        return Err("manifest: blank line or trailing garbage".into());
    }
    let mut lines = body.split('\n');
    let schema = lines.next().unwrap_or("");
    if schema != MANIFEST_SCHEMA {
        return Err(format!("manifest line 1: unsupported schema {schema:?}"));
    }
    let mut header = |key: &str| -> Result<String, String> {
        let ln = lines
            .next()
            .ok_or_else(|| format!("manifest: truncated at {key}"))?;
        let (k, v) = ln
            .split_once(' ')
            .ok_or_else(|| format!("manifest: malformed {key} header"))?;
        if k != key || v.is_empty() || v.contains(' ') {
            return Err(format!("manifest: malformed {key} header"));
        }
        Ok(v.to_string())
    };
    let module_sha: u64 = header("module")?
        .parse()
        .map_err(|_| "manifest: module sha not a u64".to_string())?;
    let caps_version = header("caps")?;
    let profile = header("profile")?;
    // 未知 profile 在消费前拒绝（来源合同不从内容猜）。
    if super::frontdoor::profile_from_slug(&profile).is_none() {
        return Err(format!("manifest: unknown profile slug {profile:?}"));
    }
    let mut units = Vec::new();
    let mut line_no = 4usize;
    for ln in lines {
        line_no += 1;
        if ln.is_empty() {
            return Err(bad(line_no, "empty unit line"));
        }
        let parts: Vec<&str> = ln.split(' ').collect();
        if parts.len() != 9 || parts[0] != "unit" {
            return Err(bad(line_no, "malformed unit record"));
        }
        let kind = kind_from_token(parts[1]).ok_or_else(|| bad(line_no, "unknown unit kind"))?;
        let owner = parts[2];
        if owner.is_empty() {
            return Err(bad(line_no, "empty owner"));
        }
        let ordinal: u32 = parts[3]
            .parse()
            .map_err(|_| bad(line_no, "ordinal not u32"))?;
        let content_sha: u64 = parts[4]
            .parse()
            .map_err(|_| bad(line_no, "content sha not u64"))?;
        let nums: Vec<usize> = parts[5..9]
            .iter()
            .map(|p| p.parse::<usize>())
            .collect::<Result<_, _>>()
            .map_err(|_| bad(line_no, "span not usize"))?;
        let (ms, me, cs, ce) = (nums[0], nums[1], nums[2], nums[3]);
        if ms > me || cs > ce {
            return Err(bad(line_no, "span start > end"));
        }
        units.push(ManifestUnit {
            kind,
            owner: owner.to_string(),
            ordinal,
            content_sha,
            mint_span: (ms, me),
            current_span: (cs, ce),
        });
    }
    Ok(DecodedManifest {
        schema: schema.to_string(),
        module_sha,
        caps_version,
        profile,
        units,
    })
}
