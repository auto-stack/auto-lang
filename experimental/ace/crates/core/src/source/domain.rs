//! PLAN-979 T-02：**通用领域包接面**（domain package seam）。
//!
//! 一个领域包 = 既有 grammar profile + 一个未受信 CST 投影 + 兼容配置 + 隐藏求值
//! 前缀。共享驱动 `compile_domain` 负责：领域前门解析 → 包投影 → 兼容闸 → 隐藏名
//! 冲突 → 行为脱糖为普通函数 → **唯一 verify**（text-record 域）→ 从 Checked HIR
//! 读回结论 → 通用行描述。
//!
//! 领域 owner（auto-ui 或本 crate）**只**提供 `project`；不新建类型检查器、表达式
//! 求值器或第二套解析器。投影结果是未受信候选：结构 + 行为 + source map + 资源声明；
//! 结构通过不豁免行为检查。

use crate::atom_text::{Diagnostic, Span};
use crate::hir::{
    BuiltinType, DefKind, ExprKind as HirExprKind, Stmt as HirStmt, TypeKind, TypeUse,
};
use crate::source::age_h0::{domain_profile_refusal, infra_diag, text_diag, TextKind};
use crate::source::ast::{
    Block, Expr, ExprKind, FnItem, Ident, Item, Program, Receiver, Stmt, TypeAnn, TypeRef, TypeWord,
};
use crate::source::cst::{parse_domain_whole_module, FrontBudget, FrontOk, FrontOutcome};
use crate::source::SourceProfile;

/// 领域包版本（与 grammar/ABI 版本一起进入交接件身份）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainVersion {
    pub major: u16,
    pub minor: u16,
}

/// 兼容声明规则：某拼写**只在**显式启用兼容配置时准入（如旧 `widget`）。
#[derive(Debug, Clone, Copy)]
pub struct CompatRule {
    /// 需要兼容配置才可用的声明拼写（声明头实际拼写）。
    pub spelling: &'static str,
    /// 关闭时的具名拒绝码。
    pub disabled_code: &'static str,
    /// 关闭时的拒绝消息。
    pub disabled_message: &'static str,
}

/// 通用领域包（静态注册；投影为普通函数指针——无 dynamic dispatch/分配）。
pub struct DomainPackage {
    /// 稳定领域 id（`ace.<area>.<name>/v<major>` 形式，写入描述）。
    pub id: &'static str,
    pub version: DomainVersion,
    /// 领域前门 profile（grammar 裁剪面 + 集中参数）。
    pub profile: SourceProfile,
    /// 隐藏求值函数前缀（用户顶层名含此前缀 = 具名冲突）。
    pub hidden_prefix: &'static str,
    /// 兼容声明规则（无兼容面时为 None）。
    pub compat: Option<CompatRule>,
    /// 领域 owner 提供的未受信投影：解析结果 → 候选。
    pub project: fn(&FrontOk, &str) -> Result<DomainCandidate, Vec<Diagnostic>>,
}

/// 未受信领域候选（结构 + 行为 + 资源声明）。
pub struct DomainCandidate {
    pub structure: Vec<DomainItem>,
    /// 行为（表达式）；驱动按声明顺序脱糖为隐藏求值函数。
    pub behaviors: Vec<DomainBehavior>,
    /// 能力要求（如 `ac.runtime.text.v1`）。
    pub requirements: Vec<String>,
}

/// 结构行：role/名字/原 span + 次级 span + 已消费拼写 + 子行 + 行为引用。
pub struct DomainItem {
    pub role: String,
    pub name: Option<String>,
    /// 主 span（声明/槽位/叶的原始字节范围）。
    pub span: Span,
    /// 次级 span（如组件体的花括号范围；无则 None）。
    pub aux_span: Option<Span>,
    /// 该行实际消费的声明拼写（如 `node`/`widget`；槽位行可为 None）。
    pub spelling: Option<String>,
    /// 行为下标（此行的求值表达式）。
    pub bindings: Vec<usize>,
    pub children: Vec<DomainItem>,
}

/// 行为：一个待共享检查/求值的表达式及其原 span。
pub struct DomainBehavior {
    pub expr: Expr,
    pub span: Span,
}

/// 行为结论：隐藏函数名 + 共享 HIR 的 def id + 文本种类。
#[derive(Debug, Clone)]
pub struct HiddenConclusion {
    pub hidden_fn: String,
    pub def_id: String,
    pub kind: TextKind,
}

/// 已解析的行为绑定（结构行走查后填充）。
#[derive(Debug, Clone)]
pub struct ResolvedBinding {
    pub hidden_fn: String,
    pub def_id: String,
    pub kind: TextKind,
}

/// 通用行描述（领域包无关；领域 owner 用自己的类型映射/再包装）。
#[derive(Debug, Clone)]
pub struct DomainRow {
    pub role: String,
    pub name: Option<String>,
    pub span: Span,
    pub aux_span: Option<Span>,
    pub spelling: Option<String>,
    /// 是否用了兼容拼写（如 `widget`）。
    pub compat: bool,
    pub bindings: Vec<ResolvedBinding>,
    pub children: Vec<DomainRow>,
}

/// 领域编译结果：身份 + 通用描述 + 唯一 verify 的产物。
#[derive(Debug)]
pub struct DomainCompilation {
    pub id: &'static str,
    pub version: DomainVersion,
    pub grammar_id: String,
    /// 兼容配置是否启用（进入描述；本切片无投影缓存，故不在 (grammar,input) 对中）。
    pub compat: bool,
    pub description: Vec<DomainRow>,
    pub checked: crate::verify::CheckedModule,
    pub input_hash: String,
    pub atom_text: String,
}

/// 包注册冲突（V-02）：同 id 的版本/profile 不一致时**拒注册**，不静默覆盖。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainPackageConflict {
    pub id: String,
    pub existing_version: DomainVersion,
    pub incoming_version: DomainVersion,
    pub existing_profile: SourceProfile,
    pub incoming_profile: SourceProfile,
}

/// 领域包注册表（最小加载面）：同 id 同版本同 profile = 幂等；
/// 其余同 id 变体 = 具名冲突，旧注册保留（拒绝加载不破坏已装程序）。
#[derive(Default)]
pub struct DomainRegistry {
    packages: Vec<&'static DomainPackage>,
}

impl DomainRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个静态包（编译期常量身份；本切片无动态装载）。
    pub fn register(&mut self, pkg: &'static DomainPackage) -> Result<(), DomainPackageConflict> {
        if let Some(existing) = self.packages.iter().find(|p| p.id == pkg.id) {
            if existing.version == pkg.version && existing.profile == pkg.profile {
                return Ok(());
            }
            return Err(DomainPackageConflict {
                id: pkg.id.to_string(),
                existing_version: existing.version,
                incoming_version: pkg.version,
                existing_profile: existing.profile,
                incoming_profile: pkg.profile,
            });
        }
        self.packages.push(pkg);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&'static DomainPackage> {
        self.packages.iter().copied().find(|p| p.id == id)
    }

    pub fn len(&self) -> usize {
        self.packages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }
}
/// 共享驱动：领域前门 → 包投影 → 兼容闸 → 隐藏名 → 脱糖 → 唯一 verify → 描述。
pub fn compile_domain(
    pkg: &'static DomainPackage,
    src: &str,
    module_path: &str,
    compat: bool,
    capabilities: &[String],
) -> Result<DomainCompilation, Vec<Diagnostic>> {
    // 0) 包必须拥有领域 profile；普通 profile 不进入领域驱动（对称于通用入口
    //    对领域 profile 的具名拒绝——两个方向都不静默）。
    if crate::source::age_h0::domain_profile_refusal(pkg.profile, 0).is_none() {
        return Err(vec![text_diag(
            "source.domain-package",
            format!(
                "a domain package must own a domain profile; `{}` is a general profile",
                crate::source::cst::profile_slug(pkg.profile)
            ),
            Span {
                start: 0,
                end: src.len(),
            },
        )]);
    }
    // 1) 领域前门（公共入口对领域 profile 具名拒绝；此处为 crate 内部路径）。
    let ok = match parse_domain_whole_module(src, pkg.profile, FrontBudget::WORK_OWNED_64MIB) {
        FrontOutcome::Ok(ok) => ok,
        FrontOutcome::Diag(ds) => return Err(ds),
        FrontOutcome::ResourceExhausted { what, limit } => {
            return Err(vec![text_diag(
                "source.resource",
                format!("resource exhausted: {what} (limit {limit})"),
                Span {
                    start: 0,
                    end: src.len(),
                },
            )])
        }
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => {
            return Err(vec![infra_diag(
                format!("domain front door: {m}"),
                src.len(),
            )])
        }
    };

    // 2) 领域 owner 投影（未受信）。
    let candidate = (pkg.project)(&ok, src)?;
    // 2b) 候选自检：未受信投影必须可推进且不丢字段——越界/重复绑定、空角色、
    //     过深/过大结构一律具名拒绝（T-08 V-08：坏包不死循环、不静默丢）。
    validate_candidate(&candidate)?;

    // 3) 兼容闸：兼容拼写必须显式启用，否则原声明头 span 具名拒绝（不回落旧引擎）。
    if let Some(rule) = pkg.compat {
        if !compat {
            if let Some(item) = find_compat_item(&candidate.structure, rule.spelling) {
                return Err(vec![text_diag(
                    rule.disabled_code,
                    rule.disabled_message,
                    item.aux_span.unwrap_or(item.span),
                )]);
            }
        }
    }

    // 4) 隐藏名冲突：任何用户顶层名**包含**前缀即具名冲突（不静默遮蔽）。
    for item in &ok.program.items {
        let user = match item {
            Item::Fn(f) => Some(&f.name),
            Item::Type(t) => Some(&t.name),
            Item::Enum(e) => Some(&e.name),
            Item::Const(c) => Some(&c.name),
            Item::Store(s) => Some(&s.name),
            Item::UseRs(_) => None,
            // PLAN-968: task names join the hidden-name conflict check.
            Item::UsePy(_) => None,
            // PLAN-968: task names join the hidden-name conflict check.
            Item::Task(t) => Some(&t.name),
        };
        if let Some(u) = user {
            if u.name.contains(pkg.hidden_prefix) {
                return Err(vec![text_diag(
                    "source.age.hidden-name",
                    format!(
                        "`{}` contains `{}` which is reserved for domain value evaluation",
                        u.name, pkg.hidden_prefix
                    ),
                    u.span,
                )]);
            }
        }
    }

    // 5) 脱糖：每个行为一个隐藏求值函数（普通 Auto 源，交给唯一 verify）。
    let mut items = ok.program.items.clone();
    let mut hidden: Vec<(String, HiddenConclusion)> = Vec::new();
    let mut hidden_names: Vec<String> = Vec::new();
    for (idx, b) in candidate.behaviors.iter().enumerate() {
        let name = format!("{}{idx}", pkg.hidden_prefix);
        hidden_names.push(name.clone());
        items.push(Item::Fn(hidden_fn(&name, &b.expr, b.span)));
    }
    if items.len() == ok.program.items.len() {
        hidden_names.clear();
    }
    let desugared = Program { items };

    // 6) 唯一 verify：整模块（普通项 + 隐藏求值函数）走 text-record 域共享链。
    let parts = match crate::source::adapter::compile_program_parts_age(
        &desugared,
        src,
        module_path,
        capabilities,
        SourceProfile::TextRecord,
    ) {
        Ok(p) => p,
        Err(ds) => return Err(relabel_behavior_diags(ds, &candidate)),
    };
    let input_hash = parts.input_hash.clone();
    let checked = match crate::verify::verify_text_record_age(parts.bundle) {
        Ok(c) => c,
        Err(ds) => return Err(relabel_behavior_diags(ds, &candidate)),
    };

    // 7) 结论：从 Checked HIR 精确读回每个行为的文本种类/def id。
    for name in &hidden_names {
        let Some((def_id, kind)) = hidden_conclusion(&checked, name) else {
            let behavior = candidate
                .behaviors
                .get(hidden_names.iter().position(|h| h == name).unwrap_or(0));
            let span = behavior
                .map(|b| b.span)
                .unwrap_or(Span { start: 0, end: 0 });
            return Err(vec![text_diag(
                "source.age.text-type",
                "a domain value expression must be text (`str` or `Str`); the shared type \
                 checker rejected this expression",
                span,
            )]);
        };
        hidden.push((
            name.clone(),
            HiddenConclusion {
                hidden_fn: name.clone(),
                def_id,
                kind,
            },
        ));
    }

    // 8) 描述：通用行树（行为下标 → 已解析绑定）。
    let description = resolve_rows(&candidate.structure, &hidden, pkg.compat, compat);

    Ok(DomainCompilation {
        id: pkg.id,
        version: pkg.version,
        grammar_id: ok.grammar_id.clone(),
        compat,
        description,
        checked,
        input_hash,
        atom_text: parts.atom_text,
    })
}

/// 候选自检：保证驱动可推进且不静默丢字段（domain owner 的投影是不可信输入）。
/// 深度/条数上界只防病态投影；不是语言语义限制（超限=资源/候选拒绝，非语言非法）。
fn validate_candidate(cand: &DomainCandidate) -> Result<(), Vec<Diagnostic>> {
    const MAX_ITEMS: usize = 100_000;
    let mut count = 0usize;
    let mut seen: Vec<usize> = Vec::new();
    for item in &cand.structure {
        validate_item(item, cand.behaviors.len(), 1, &mut count, &mut seen)?;
    }
    if count > MAX_ITEMS {
        return Err(vec![text_diag(
            "source.domain-candidate",
            format!("domain candidate has {count} structural rows (limit {MAX_ITEMS})"),
            Span { start: 0, end: 0 },
        )]);
    }
    Ok(())
}

fn validate_item(
    item: &DomainItem,
    behaviors: usize,
    depth: usize,
    count: &mut usize,
    seen: &mut Vec<usize>,
) -> Result<(), Vec<Diagnostic>> {
    const MAX_DEPTH: usize = 64;
    if depth > MAX_DEPTH {
        return Err(vec![text_diag(
            "source.domain-candidate",
            format!("domain candidate structure is deeper than {MAX_DEPTH}"),
            item.span,
        )]);
    }
    if item.role.is_empty() {
        return Err(vec![text_diag(
            "source.domain-candidate",
            "domain candidate row has an empty role",
            item.span,
        )]);
    }
    *count += 1;
    for idx in &item.bindings {
        if *idx >= behaviors {
            return Err(vec![text_diag(
                "source.domain-candidate",
                format!(
                    "domain candidate binding {idx} is out of range (the candidate declares \
                     {behaviors} behaviours)"
                ),
                item.span,
            )]);
        }
        if seen.contains(idx) {
            return Err(vec![text_diag(
                "source.domain-candidate",
                format!("domain candidate binds behaviour {idx} more than once"),
                item.span,
            )]);
        }
        seen.push(*idx);
    }
    for child in &item.children {
        validate_item(child, behaviors, depth + 1, count, seen)?;
    }
    Ok(())
}

/// 投影完整性交接检查：`required` 里的每个 role 都必须在描述里出现。
/// 删除一条必需映射（如投影不再产出 `view` 行）时返回缺失清单 → 调用方判红；
/// 合法近邻（多出其它 role）不受影响。这是 V-11 的通用机制，不含产品语义。
pub fn mapping_complete(description: &[DomainRow], required: &[&str]) -> Result<(), Vec<String>> {
    let mut present: Vec<&str> = Vec::new();
    fn walk<'a>(rows: &'a [DomainRow], out: &mut Vec<&'a str>) {
        for r in rows {
            out.push(r.role.as_str());
            walk(&r.children, out);
        }
    }
    walk(description, &mut present);
    let missing: Vec<String> = required
        .iter()
        .filter(|want| !present.iter().any(|p| *p == **want))
        .map(|w| w.to_string())
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}
/// 领域证据身份：包 id/版本 + grammar 身份 + 源哈希 + 兼容配置（F12 收口）。
/// 任一项变化即旧证据失效；接投影缓存时可直接以此元组为键。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainIdentity {
    pub id: &'static str,
    pub version: DomainVersion,
    pub grammar_id: String,
    /// 源内容哈希（同包不同源不得复用证据）。
    pub input_hash: String,
    /// 兼容配置（同 id/源、不同 `widget` 准入不得复用证据）。
    pub compat: bool,
}

impl DomainCompilation {
    /// 本次编译的证据身份（热换 schema/包/源/兼容配置后必须不同）。
    pub fn identity(&self) -> DomainIdentity {
        DomainIdentity {
            id: self.id,
            version: self.version,
            grammar_id: self.grammar_id.clone(),
            input_hash: self.input_hash.clone(),
            compat: self.compat,
        }
    }
}

/// 两份证据是否来自同一身份（不同 = 旧证据失效）。
pub fn same_evidence(a: &DomainIdentity, b: &DomainIdentity) -> bool {
    a == b
}
/// 隐藏求值函数：`fn <prefix><n>() Str { return (<expr>).clone() }`（普通 Auto 源）。
fn hidden_fn(name: &str, expr: &Expr, span: Span) -> FnItem {
    let clone = Expr {
        span,
        kind: ExprKind::Method {
            recv: Box::new(expr.clone()),
            name: Ident {
                name: "clone".to_string(),
                span,
            },
            args: Vec::new(),
        },
    };
    FnItem {
        name: Ident {
            name: name.to_string(),
            span,
        },
        type_params: Vec::new(),
        params: Vec::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(TypeWord::StrOwned),
            span,
        },
        body: Block {
            stmts: vec![Stmt::Return { value: clone, span }],
            span,
        },
        span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
        proto: false,
    }
}

/// 兼容拼写的首个使用点（声明头 span）。
fn find_compat_item<'a>(items: &'a [DomainItem], spelling: &str) -> Option<&'a DomainItem> {
    for item in items {
        if item.spelling.as_deref() == Some(spelling) {
            return Some(item);
        }
        if let Some(found) = find_compat_item(&item.children, spelling) {
            return Some(found);
        }
    }
    None
}

/// 行为绑定 → 已解析结论（未绑定行为的行保持空）。
fn resolve_rows(
    items: &[DomainItem],
    hidden: &[(String, HiddenConclusion)],
    _rule: Option<CompatRule>,
    compat: bool,
) -> Vec<DomainRow> {
    items
        .iter()
        .map(|item| {
            let bindings = item
                .bindings
                .iter()
                .filter_map(|idx| hidden.get(*idx).map(|(_, c)| c))
                .map(|c| ResolvedBinding {
                    hidden_fn: c.hidden_fn.clone(),
                    def_id: c.def_id.clone(),
                    kind: c.kind,
                })
                .collect::<Vec<_>>();
            DomainRow {
                role: item.role.clone(),
                name: item.name.clone(),
                span: item.span,
                aux_span: item.aux_span,
                spelling: item.spelling.clone(),
                compat: compat && item.spelling.as_deref() == Some("widget"),
                bindings,
                children: resolve_rows(&item.children, hidden, _rule, compat),
            }
        })
        .collect()
}

/// 隐藏求值函数的 Checked 结论（HIR 名由共享 adapter 唯一决定：`sf_<source name>`）。
pub(crate) fn hidden_conclusion(
    checked: &crate::verify::CheckedModule,
    fn_name: &str,
) -> Option<(String, TextKind)> {
    let module = checked.bundle().modules.first()?;
    // 隐藏求值函数的 HIR 名由共享 adapter 唯一决定（`sf_<source name>`），
    // 按**完整名**精确匹配：后缀式用户函数不会被误选。
    let symbol = crate::source::adapter::free_fn_symbol(fn_name);
    let def = module.defs.iter().find(|d| match &d.kind {
        DefKind::Function { name, .. } => name == &symbol,
        _ => false,
    })?;
    conclusion_of_def(module, def)
}

/// 一个函数 def 的 (def id, 文本种类) 结论（两个查名入口共用同一判据）。
fn conclusion_of_def(
    module: &crate::hir::Module,
    def: &crate::hir::Def,
) -> Option<(String, TextKind)> {
    let body_id = match &def.kind {
        DefKind::Function { body, .. } => *body,
        _ => return None,
    };
    let body = module.bodies.get(body_id.0 as usize)?;
    let block = body.blocks.get(body.entry.0 as usize)?;
    let value = block.stmts.iter().find_map(|s| match s {
        HirStmt::Return { value } => Some(*value),
        _ => None,
    })?;
    let expr = body.exprs.get(value.0 as usize)?;
    let HirExprKind::TextClone { operand } = &expr.kind else {
        return None;
    };
    let operand_expr = body.exprs.get(operand.0 as usize)?;
    let kind = match operand_expr.ty {
        TypeUse::Ref(tid) => match module.types.get(tid.0 as usize).map(|t| &t.kind) {
            Some(TypeKind::Str) => TextKind::View,
            Some(TypeKind::StrOwned) => TextKind::Owned,
            _ => return None,
        },
        TypeUse::Builtin(BuiltinType::StrSlice) => TextKind::View,
        TypeUse::Builtin(BuiltinType::StrOwned) => TextKind::Owned,
        _ => return None,
    };
    Some((def.id_text.clone(), kind))
}

/// 行为表达式内的共享拒绝 → 领域代码标注（只标脱糖壳引起的文本类型拒绝；其余保留
/// 共享消息，改标中立 `source.age.leaf-expr`）。
fn relabel_behavior_diags(
    mut diags: Vec<Diagnostic>,
    candidate: &DomainCandidate,
) -> Vec<Diagnostic> {
    for d in &mut diags {
        let inside = candidate.behaviors.iter().any(|b| {
            d.span.start >= b.span.start && d.span.start < b.span.end.max(b.span.start + 1)
        });
        if inside {
            let text_ness = d.message.contains("clone");
            d.code = if text_ness {
                "source.age.text-type".to_string()
            } else {
                "source.age.leaf-expr".to_string()
            };
            d.message = if text_ness {
                format!(
                    "a `text` leaf expression must be text (`str` or `Str`); the shared checker \
                     rejected it: {}",
                    d.message
                )
            } else {
                format!("the `text` leaf expression was rejected: {}", d.message)
            };
        }
    }
    diags
}

/// 领域 profile 的通用入口拒绝（供各领域/前门复用）。
pub fn refusal(profile: SourceProfile, src_len: usize) -> Option<Diagnostic> {
    domain_profile_refusal(profile, src_len)
}
