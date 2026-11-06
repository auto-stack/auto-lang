//! 统一前门：typed 请求/结果、语法程序缓存、显式预算与兼容桥。

use super::kinds::SourceKinds;
use super::project;
use crate::atom_text::Diagnostic;
use crate::source::ast::{FnItem, Program, Receiver};
use crate::source::SourceProfile;
use ace_syntax_runtime::cst::CstTree;
use ace_syntax_runtime::lexer::CompiledScanner;
use ace_syntax_runtime::parse::{parse_snapshot, ParseOutput};
use ace_syntax_runtime::program::{Budget, GrammarProgram};
use ace_syntax_runtime::snapshot::{PageStore, Snapshot};
use std::sync::OnceLock;

/// 共享语法源（单文件单规则源；身份 = 裁剪后 canonical bytes 的 GrammarId）。
pub const AUTO_SOURCE_GRAMMAR: &str = include_str!("../../../../../grammars/auto-source.at");

/// profile 能力清单版本：语法语义/能力/映射面变更必须换键（计划 §5.2）。
/// 与 `auto-source.at` 的 revision、grammar-id 一同进入前端结果身份。
/// PLAN-936（T-04）：`ac-source-front/3` —— 新增 `scalar-ct` 裁剪面与
/// `ac.source.cte.scalar/v1` 显式能力（`#{ expr }` CT 需求原子）。
/// PLAN-935（T-02）：`ac-source-front/2` —— 新增 `scalar-instances` 裁剪面
/// 与 `ac.source.instances.scalar/v1` 显式能力（fn `<T>`/类型位裸 ident）。
pub const FRONT_CAPS_VERSION: &str = "ac-source-front/3";

/// 前端显式预算（V-09：不得沿 `Budget::default()` 的 256MiB work_bytes 冒
/// 64MiB 档授权；各消费门按原适用域显式传入）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontBudget {
    /// 解析工作集上限（syntax 流式档 ≤64MiB 的调用方须显式给 ≤64MiB）。
    pub work_bytes: usize,
    /// 最大解析深度（可配置资源额度，不是语言 unsupported）。
    pub max_parse_depth: usize,
}

impl FrontBudget {
    /// 64MiB work-owned 档的显式上限（原 787 流式固定负载授权）。
    pub const WORK_OWNED_64MIB: FrontBudget = FrontBudget {
        work_bytes: 64 << 20,
        max_parse_depth: 1 << 20,
    };
}

impl From<FrontBudget> for Budget {
    fn from(b: FrontBudget) -> Budget {
        Budget {
            max_parse_depth: b.max_parse_depth,
            work_bytes: b.work_bytes,
        }
    }
}

/// 入口种类（§5.4：完整模块、free fn、方法等 kind 不是四套语法实现）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontKind {
    /// 整个源码模块（冷编译前门）。
    WholeModule,
    /// 授权区间内的单个**自由函数**片段（旧 `parse_function_fragment`
    /// 合同：free-only、完整消耗、绝对 span；区间须有序/在界/UTF-8 边界）。
    FreeFnFragment { range: std::ops::Range<usize> },
    /// 方法片段：owner/receiver 来自请求上下文（不拼假 free fn——receiver
    /// 与求值序由真实 owner 决定；旧调用方在 929 迁移到此入口）。
    MethodFragment {
        range: std::ops::Range<usize>,
        owner: String,
        receiver: Receiver,
    },
}

/// 逻辑结果（§5.4 typed 接缝：成功/源码诊断/资源耗尽/stale-foreign/基础设施）。
pub enum FrontOutcome {
    /// 严格投影成功：语法 AST + 语法事实（无损 CST/身份）。不铸 Checked——
    /// 名称/类型/资源仍由 resolve/typecheck 决定。
    Ok(FrontOk),
    /// 源码诊断（stage/code/span/message 保真，顺序 = 旧首个事实序）。
    Diag(Vec<Diagnostic>),
    /// 资源耗尽（`RtError::ResourceExhausted` 穿过投影仍是资源态，不是
    /// `source.*` 语法拒绝；预算不足是资源态，不是语言不支持）。
    ResourceExhausted { what: &'static str, limit: usize },
    /// 请求与当前语法/快照身份不符（片段授权路径；T-07 接实际 producer）。
    StaleForeign(String),
    /// 基础设施失败（语法装载/裁剪失败、快照读写失败——不是源码问题）。
    Infra(String),
}

impl std::fmt::Debug for FrontOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrontOutcome::Ok(ok) => write!(
                f,
                "Ok(grammar={}, caps={}, items={})",
                ok.grammar_id,
                ok.caps_version,
                ok.program.items.len()
            ),
            FrontOutcome::Diag(ds) => write!(f, "Diag({ds:?})"),
            FrontOutcome::ResourceExhausted { what, limit } => {
                write!(f, "ResourceExhausted({what}, {limit})")
            }
            FrontOutcome::StaleForeign(m) => write!(f, "StaleForeign({m})"),
            FrontOutcome::Infra(m) => write!(f, "Infra({m})"),
        }
    }
}

pub struct FrontOk {
    pub program: Program,
    pub grammar_id: String,
    pub caps_version: &'static str,
    pub snapshot_len: usize,
    /// 无损 CST（后续会话/持久化消费；恢复不携带 trusted 标志）。
    pub cst: CstTree,
}

/// 统一入口逻辑请求（§5.4：绑定 profile/能力版本/入口 kind/预算；profile
/// 是来源合同，不从后端名/文件名/源码内容猜）。
pub struct FrontRequest<'s> {
    pub src: &'s str,
    pub profile: SourceProfile,
    pub kind: FrontKind,
    pub budget: FrontBudget,
}

pub fn profile_slug(profile: SourceProfile) -> &'static str {
    match profile {
        SourceProfile::Core => "core",
        SourceProfile::Scalar => "scalar",
        SourceProfile::Text => "text",
        SourceProfile::Record => "record",
        SourceProfile::TextRecord => "text-record",
        SourceProfile::EntryUnit => "entry-unit",
        SourceProfile::HostConsole => "host-console",
        // PLAN-928 T-02/T-03：闭包/集合两面经统一前门（差分全集与冷入口
        // 切换由 T-04/T-07 冻结）。
        SourceProfile::Closures => "closures",
        SourceProfile::Collections => "collections",
    }
}

/// PLAN-928（T-09）：slug → profile 反向映射（清单解码侧的 profile 合法
/// 性校验——未知 slug 在消费前拒绝，V-08）。
pub fn profile_from_slug(slug: &str) -> Option<SourceProfile> {
    Some(match slug {
        "core" => SourceProfile::Core,
        "scalar" => SourceProfile::Scalar,
        "text" => SourceProfile::Text,
        "record" => SourceProfile::Record,
        "text-record" => SourceProfile::TextRecord,
        "entry-unit" => SourceProfile::EntryUnit,
        "host-console" => SourceProfile::HostConsole,
        "closures" => SourceProfile::Closures,
        "collections" => SourceProfile::Collections,
        _ => return None,
    })
}

pub struct CachedGrammar {
    program: GrammarProgram,
    scanner: CompiledScanner,
    kinds: SourceKinds,
}

impl CachedGrammar {
    /// PLAN-929（T-03）：共享语法程序访问器——常驻/暖路径消费同一份
    /// profile 裁剪编译产物（不另建第二语法栈）。经 `profile_grammar`
    /// 取得的 `&'static CachedGrammar` 上借用即进程级存活。
    pub fn program(&self) -> &GrammarProgram {
        &self.program
    }
    /// PLAN-929（T-03）：共享扫描器访问器（同上）。
    pub fn scanner(&self) -> &CompiledScanner {
        &self.scanner
    }
}

static GRAMMARS: OnceLock<[OnceLock<CachedGrammar>; 9]> = OnceLock::new();

/// PLAN-935（T-02）：`scalar-instances` 裁剪程序（scalar 全表面 + 有限模板
/// 面）。独立缓存槽：plain profile 程序逐字节不变，grammar-id 自然隔离。
static INSTANCES_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();

/// PLAN-936（T-04）：`scalar-ct` 裁剪程序（scalar-instances 全表面 +
/// `#{ expr }` CT 需求原子）。独立缓存槽，同上隔离。
static CT_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();

/// PLAN-936：启用有限 CT 表达式面的显式能力。调用方携带本能力时统一前门
/// 以 `scalar-ct` 裁剪编译（该面包含模板表面——CT 可作用于模板实例）；
/// 未携带 → `#` 不是词面 → 原语法拒绝面不变。
pub const CT_CAPABILITY: &str = "ac.source.cte.scalar/v1";

/// scalar-ct 裁剪编译并缓存（进程内一次）。
pub fn ct_grammar() -> Result<&'static CachedGrammar, String> {
    let slot = CT_GRAMMAR.get_or_init(|| Default::default());
    if let Some(g) = slot.get() {
        return Ok(g);
    }
    let program =
        ace_syntax_grammar::compile_profile(AUTO_SOURCE_GRAMMAR, "scalar-ct").map_err(|diags| {
            format!(
                "auto-source grammar failed to compile for `scalar-ct`: {:?}",
                diags
            )
        })?;
    let scanner =
        CompiledScanner::compile(&program).map_err(|e| format!("scanner compile: {e}"))?;
    let kinds = SourceKinds::from_program(&program).map_err(|missing| {
        format!("auto-source grammar for `scalar-ct` lacks required kinds: {missing:?}")
    })?;
    let cached = CachedGrammar {
        program,
        scanner,
        kinds,
    };
    let _ = slot.set(cached);
    Ok(slot.get().expect("just set"))
}

/// PLAN-935：启用有限模板面的显式能力（T-02 冻结字面量）。调用方在其
/// capabilities 中携带本能力时，统一前门以 `scalar-instances` 裁剪编译；
/// 未携带 → plain scalar 裁剪，`<T>`/裸 ident 维持原语法拒绝面。
pub const INSTANCES_CAPABILITY: &str = "ac.source.instances.scalar/v1";

/// scalar-instances 裁剪编译并缓存（进程内一次）。
pub fn instances_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&INSTANCES_GRAMMAR, "scalar-instances")
}

static MODULES_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();
pub(crate) fn module_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&MODULES_GRAMMAR, "scalar-modules")
}

fn capability_grammar(
    cell: &'static OnceLock<OnceLock<CachedGrammar>>,
    slug: &str,
) -> Result<&'static CachedGrammar, String> {
    let slot = cell.get_or_init(OnceLock::new);
    if let Some(g) = slot.get() {
        return Ok(g);
    }
    let program = ace_syntax_grammar::compile_profile(AUTO_SOURCE_GRAMMAR, slug)
        .map_err(|d| format!("auto-source grammar `{slug}`: {d:?}"))?;
    let scanner = CompiledScanner::compile(&program).map_err(|e| format!("scanner: {e}"))?;
    let kinds = SourceKinds::from_program(&program)
        .map_err(|e| format!("auto-source grammar `{slug}` missing kinds: {e:?}"))?;
    let _ = slot.set(CachedGrammar {
        program,
        scanner,
        kinds,
    });
    Ok(slot.get().expect("just set"))
}

/// PLAN-950: the module grammar uses the same scanner, parser and strict
/// projection as every source profile. Metadata is read from CST fields,
/// never reconstructed by token stripping or source concatenation.
pub(crate) fn parse_module_front(
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::modules::ModuleSyntax, Vec<Diagnostic>> {
    parse_module_syntax_range(src, budget, true)
}
pub(crate) fn parse_module_fragment(
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::modules::ModuleSyntax, Vec<Diagnostic>> {
    parse_module_syntax_range(src, budget, false)
}
fn parse_module_syntax_range(
    src: &str,
    budget: FrontBudget,
    whole: bool,
) -> Result<crate::source::modules::ModuleSyntax, Vec<Diagnostic>> {
    use crate::atom_text::{Span, Stage};
    let err = |code, message: String, span| vec![Diagnostic::new(Stage::Text, code, message, span)];
    let zero = Span { start: 0, end: 0 };
    let cached = capability_grammar(&MODULES_GRAMMAR, "scalar-modules")
        .map_err(|e| err("source.infra", e, zero))?;
    crate::source::parser::note_tokenize(Span {
        start: 0,
        end: src.len(),
    });
    if whole {
        crate::body_work::record_whole(crate::body_work::Stage::WholeParse);
    }
    let pages = std::rc::Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&pages, src);
    let parsed = parse_snapshot(
        &cached.program,
        &cached.scanner,
        &snap,
        &Budget::from(budget),
    )
    .map_err(|e| match e {
        ace_syntax_runtime::program::RtError::ResourceExhausted { what, limit } => err(
            "source.resource",
            format!("resource exhausted: {what} (limit {limit})"),
            zero,
        ),
        other => err("source.infra", format!("parse: {other}"), zero),
    })?;
    let result = project_module_front(src, vec![parsed.tree], Some(parsed.tokens))?;
    for item in &result.program.items {
        if let crate::source::ast::Item::Fn(f) = item {
            crate::source::parser::note_parse(&f.name.name);
        }
    }
    Ok(result)
}

pub(crate) fn project_module_front(
    src: &str,
    trees: Vec<CstTree>,
    tokens: Option<Vec<ace_syntax_runtime::lexer::Token>>,
) -> Result<crate::source::modules::ModuleSyntax, Vec<Diagnostic>> {
    use crate::atom_text::{Span, Stage};
    use crate::source::modules::{DeclarationPlacement, Import, ModuleSyntax, SyntaxPiece};
    let err = |code, message: String, span| vec![Diagnostic::new(Stage::Text, code, message, span)];
    let cached = module_grammar().map_err(|e| err("source.infra", e, Span { start: 0, end: 0 }))?;
    let pages = std::rc::Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&pages, src);
    let rule = |name: &str| {
        cached
            .program
            .rules
            .iter()
            .position(|r| r.name == name)
            .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
    };
    let field = |tree: &CstTree, n, name: &str| {
        cached
            .program
            .field_id(name)
            .and_then(|f| tree.field_child(n, f))
    };
    let text = |tree: &CstTree, n| {
        let (a, b) = tree.abs_range(n);
        snap.read_str(a, b).expect("CST source range")
    };
    let mut declarations = std::collections::BTreeMap::new();
    let mut exports = std::collections::BTreeSet::new();
    let mut imports = Vec::new();
    let mut saw_fn = false;
    let mut items = Vec::new();
    for tree in &trees {
        let program = project::project_program(
            &cached.program,
            &cached.kinds,
            tree,
            &snap,
            SourceProfile::Scalar,
        )?;
        items.extend(program.items);
        for d in &tree.decls {
            let node = if tree.kind(d.node) == cached.kinds.item {
                tree.children(d.node)
                    .into_iter()
                    .find(|&c| tree.node(c).flags & ace_syntax_runtime::cst::FLAG_TRIVIA == 0)
                    .unwrap_or(d.node)
            } else {
                d.node
            };
            if Some(tree.kind(node)) == rule("use-item") {
                let (a, b) = tree.abs_range(node);
                let span = Span {
                    start: a as usize,
                    end: b as usize,
                };
                if saw_fn {
                    return Err(err(
                        "source.module-import-order",
                        "imports must precede functions".into(),
                        span,
                    ));
                }
                let module = text(tree, field(tree, node, "module").expect("module field"));
                let name = text(tree, field(tree, node, "import").expect("import field"));
                if imports.iter().any(|i: &Import| i.name == name) {
                    return Err(err(
                        "source.module-duplicate-import",
                        format!("duplicate imported name `{name}`"),
                        span,
                    ));
                }
                imports.push(Import { module, name, span });
            } else if Some(tree.kind(node)) == rule("public-fn") {
                saw_fn = true;
                let function = field(tree, node, "function").expect("public function");
                let tail = tree
                    .children(function)
                    .into_iter()
                    .find(|&n| tree.kind(n) == cached.kinds.fn_tail)
                    .expect("fn tail");
                let name = text(tree, field(tree, tail, "name").expect("fn name"));
                let (start, end) = tree.abs_range(node);
                declarations.insert(
                    name.clone(),
                    Span {
                        start: start as usize,
                        end: end as usize,
                    },
                );
                exports.insert(name);
            } else if tree.kind(node) == cached.kinds.fn_item {
                saw_fn = true;
                let tail = tree
                    .children(node)
                    .into_iter()
                    .find(|&n| tree.kind(n) == cached.kinds.fn_tail)
                    .expect("fn tail");
                let name = text(tree, field(tree, tail, "name").expect("fn name"));
                let (start, end) = tree.abs_range(node);
                declarations.insert(
                    name,
                    Span {
                        start: start as usize,
                        end: end as usize,
                    },
                );
            }
        }
    }
    Ok(ModuleSyntax {
        program: crate::source::ast::Program { items },
        exports,
        imports,
        grammar_id: format!("{}", cached.program.identity.grammar_id.0),
        source_hash: crate::runtime_contract::sha256_hex(src.as_bytes()),
        pieces: trees
            .into_iter()
            .map(|tree| {
                let placements = tree
                    .decls
                    .iter()
                    .enumerate()
                    .map(|(index, d)| {
                        let (start, end) = tree.abs_range(d.node);
                        let span = Span {
                            start: start as usize,
                            end: end as usize,
                        };
                        DeclarationPlacement {
                            index,
                            mint: span,
                            current: span,
                        }
                    })
                    .collect();
                SyntaxPiece {
                    source: std::rc::Rc::from(src),
                    tree: Some(tree),
                    tokens: tokens.clone(),
                    placements,
                    stored_cst: None,
                }
            })
            .collect(),
        declarations,
    })
}

/// 裁剪编译并缓存 profile 专属语法程序（每 profile 一次；进程内复用，
/// 跨进程恢复按 grammar-id 从 store 装载，不重复编译）。
pub fn profile_grammar(profile: SourceProfile) -> Result<&'static CachedGrammar, String> {
    let idx = match profile {
        SourceProfile::Core => 0,
        SourceProfile::Scalar => 1,
        SourceProfile::Text => 2,
        SourceProfile::Record => 3,
        SourceProfile::TextRecord => 4,
        SourceProfile::EntryUnit => 5,
        SourceProfile::HostConsole => 6,
        SourceProfile::Closures => 7,
        SourceProfile::Collections => 8,
    };
    let slots = GRAMMARS.get_or_init(|| Default::default());
    // 先试读（快路径无锁竞争）；未命中再写入。
    if let Some(g) = slots[idx].get() {
        return Ok(g);
    }
    let program = ace_syntax_grammar::compile_profile(AUTO_SOURCE_GRAMMAR, profile_slug(profile))
        .map_err(|diags| {
        format!(
            "auto-source grammar failed to compile for `{}`: {:?}",
            profile_slug(profile),
            diags
        )
    })?;
    let scanner =
        CompiledScanner::compile(&program).map_err(|e| format!("scanner compile: {e}"))?;
    let kinds = SourceKinds::from_program(&program).map_err(|missing| {
        format!(
            "auto-source grammar for `{}` lacks required kinds: {missing:?}",
            profile_slug(profile)
        )
    })?;
    let cached = CachedGrammar {
        program,
        scanner,
        kinds,
    };
    let slot = &slots[idx];
    let _ = slot.set(cached);
    Ok(slot.get().expect("just set"))
}

/// 预热某 profile 的语法程序缓存（进程级一次编译——调用方在**请求时钟
/// 之外**先行预热，首请求不为编译付预算；幂等）。
pub fn warm_profile(profile: SourceProfile) {
    let _ = profile_grammar(profile);
}

/// 统一入口：共享 scanner/规则程序产无损 CST → 严格投影 → typed 结果。
/// 片段 kind 走 [`parse_fragment_front`]（同语法/同投影/同预算，区间授权
/// 与绝对 span 平移）。
pub fn parse_front(req: FrontRequest) -> FrontOutcome {
    let FrontRequest {
        src,
        profile,
        kind,
        budget,
    } = req;
    match kind {
        FrontKind::WholeModule => parse_whole_module(src, profile, budget),
        FrontKind::FreeFnFragment { range } => {
            parse_fragment_front(src, profile, budget, &range, None, Receiver::Free)
        }
        FrontKind::MethodFragment {
            range,
            owner,
            receiver,
        } => parse_fragment_front(
            src,
            profile,
            budget,
            &range,
            Some((owner, receiver)),
            receiver,
        ),
    }
}

/// PLAN-935（T-02）：能力感知统一入口。capabilities 携带
/// [`INSTANCES_CAPABILITY`] 且 profile 为 Scalar 时以 `scalar-instances`
/// 裁剪解析（fn `<T>` + 类型位裸 ident 合法，投影产出
/// `FnItem::type_params`/`TypeRef::TyVar`）；其余组合与 [`parse_front`]
/// 逐字节同面（无能力仍原拒绝）。片段 kind 在 instances 面不支持（增量
/// 片段合同仍走 plain 面——模板函数编辑的增量路径由 T-07 冻结）。
pub fn parse_front_with_caps(req: FrontRequest, capabilities: &[String]) -> FrontOutcome {
    // PLAN-936（T-04）：CT 能力优先——scalar-ct 面包含 scalar-instances 全
    // 表面（CT 可作用于模板实例），不与 instances 面叠加选择。
    let ct =
        capabilities.iter().any(|c| c == CT_CAPABILITY) && req.profile == SourceProfile::Scalar;
    let instances = !ct
        && capabilities.iter().any(|c| c == INSTANCES_CAPABILITY)
        && req.profile == SourceProfile::Scalar;
    if !instances && !ct {
        return parse_front(req);
    }
    if !matches!(req.kind, FrontKind::WholeModule) {
        return parse_front(req);
    }
    let FrontRequest {
        src,
        budget,
        profile,
        ..
    } = req;
    let cached = match if ct {
        ct_grammar()
    } else {
        instances_grammar()
    } {
        Ok(c) => c,
        Err(e) => return FrontOutcome::Infra(e),
    };
    let store = std::rc::Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, src);
    let ParseOutput {
        tree,
        tokens: _,
        lex_diags: _,
        grafted_decls: _,
    } = match parse_snapshot(
        &cached.program,
        &cached.scanner,
        &snap,
        &Budget::from(budget),
    ) {
        Ok(o) => o,
        Err(ace_syntax_runtime::program::RtError::ResourceExhausted { what, limit }) => {
            return FrontOutcome::ResourceExhausted { what, limit };
        }
        Err(e) => return FrontOutcome::Infra(format!("parse: {e}")),
    };
    match project::project_program_with_caps(
        &cached.program,
        &cached.kinds,
        &tree,
        &snap,
        profile,
        true,
        ct,
    ) {
        Ok(program) => FrontOutcome::Ok(FrontOk {
            program,
            grammar_id: format!("{}", cached.program.identity.grammar_id.0),
            caps_version: FRONT_CAPS_VERSION,
            snapshot_len: src.len(),
            cst: tree,
        }),
        Err(diags) => FrontOutcome::Diag(diags),
    }
}

fn parse_whole_module(src: &str, profile: SourceProfile, budget: FrontBudget) -> FrontOutcome {
    let cached = match profile_grammar(profile) {
        Ok(c) => c,
        Err(e) => return FrontOutcome::Infra(e),
    };
    let store = std::rc::Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, src);
    let ParseOutput {
        tree,
        tokens: _,
        lex_diags: _,
        grafted_decls: _,
    } = match parse_snapshot(
        &cached.program,
        &cached.scanner,
        &snap,
        &Budget::from(budget),
    ) {
        Ok(o) => o,
        Err(ace_syntax_runtime::program::RtError::ResourceExhausted { what, limit }) => {
            return FrontOutcome::ResourceExhausted { what, limit };
        }
        Err(e) => return FrontOutcome::Infra(format!("parse: {e}")),
    };
    match project::project_program(&cached.program, &cached.kinds, &tree, &snap, profile) {
        Ok(program) => FrontOutcome::Ok(FrontOk {
            program,
            grammar_id: format!("{}", cached.program.identity.grammar_id.0),
            caps_version: FRONT_CAPS_VERSION,
            snapshot_len: src.len(),
            cst: tree,
        }),
        Err(diags) => FrontOutcome::Diag(diags),
    }
}

/// 旧签名兼容桥（§5.1：`parse_program_with` 保兼容返回类型，七 profile
/// 路由至通用 CST）。typed 分类由 [`parse_front`] 承载；本桥只作单诊断投影。
pub fn parse_program_via_cst(src: &str, profile: SourceProfile) -> Result<Program, Diagnostic> {
    match parse_front(FrontRequest {
        src,
        profile,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    }) {
        FrontOutcome::Ok(ok) => Ok(ok.program),
        FrontOutcome::Diag(diags) => Err(super::first_diag(&diags)),
        FrontOutcome::ResourceExhausted { what, limit } => Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.resource",
            format!("resource exhausted: {what} (limit {limit})"),
            crate::atom_text::Span { start: 0, end: 0 },
        )),
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.infra",
            m,
            crate::atom_text::Span { start: 0, end: 0 },
        )),
    }
}

/// 片段统一入口（T-07）：授权区间上的 free fn / 方法片段。同一 profile
/// 语法程序与严格投影（不是第二套语法）；区间校验/单函数/完整消耗/
/// receiver-owner 上下文按旧 `parse_function_fragment` 合同与 §5.1 方法
/// 扩展；全部 span 平移回原源绝对 UTF-8 byte 坐标。
pub fn parse_fragment_front(
    src: &str,
    profile: SourceProfile,
    budget: FrontBudget,
    range: &std::ops::Range<usize>,
    method_ctx: Option<(String, Receiver)>,
    receiver: Receiver,
) -> FrontOutcome {
    use crate::atom_text::Stage;
    let frag_src = match src.get(range.clone()) {
        Some(f) => f,
        None => {
            return FrontOutcome::Diag(vec![Diagnostic::new(
                Stage::Text,
                "source.incremental-fragment",
                "fragment range must be ordered, in bounds and on UTF-8 boundaries",
                crate::atom_text::Span { start: 0, end: 0 },
            )]);
        }
    };
    let cached = match profile_grammar(profile) {
        Ok(c) => c,
        Err(e) => return FrontOutcome::Infra(e),
    };
    let store = std::rc::Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, frag_src);
    let ParseOutput {
        tree,
        tokens: _,
        lex_diags: _,
        grafted_decls: _,
    } = match parse_snapshot(
        &cached.program,
        &cached.scanner,
        &snap,
        &Budget::from(budget),
    ) {
        Ok(o) => o,
        Err(ace_syntax_runtime::program::RtError::ResourceExhausted { what, limit }) => {
            return FrontOutcome::ResourceExhausted { what, limit };
        }
        Err(e) => return FrontOutcome::Infra(format!("parse: {e}")),
    };
    let mut program =
        match project::project_program(&cached.program, &cached.kinds, &tree, &snap, profile) {
            Ok(p) => p,
            Err(mut ds) => {
                for d in ds.iter_mut() {
                    super::shift::shift_diag(d, range.start as i64);
                }
                return FrontOutcome::Diag(ds);
            }
        };
    let span = crate::atom_text::Span {
        start: range.start,
        end: range.end,
    };
    if program.items.len() != 1 {
        return FrontOutcome::Diag(vec![Diagnostic::new(
            Stage::Text,
            "source.incremental-fragment",
            "a fragment must contain exactly one free function",
            span,
        )]);
    }
    let item = program.items.remove(0);
    let mut function = match item {
        crate::source::ast::Item::Fn(f) if !f.script => f,
        _ => {
            return FrontOutcome::Diag(vec![Diagnostic::new(
                Stage::Text,
                "source.incremental-fragment",
                "a fragment must contain one declared free function",
                span,
            )]);
        }
    };
    match method_ctx {
        None => {
            // free-fn 合同：不得是方法形状（receiver 由请求给出——free）。
            if function.receiver != Receiver::Free {
                return FrontOutcome::Diag(vec![Diagnostic::new(
                    Stage::Text,
                    "source.incremental-fragment",
                    "a fragment must contain one declared free function",
                    span,
                )]);
            }
        }
        Some((owner, _recv)) => {
            // 方法片段：owner/receiver 由请求上下文恢复（同一语法规则；
            // 区间内就是一个 `fn` 形状，receiver 不从内容猜）。
            function.receiver = receiver;
            function.owner = Some(crate::source::ast::Ident {
                name: owner,
                span: crate::atom_text::Span { start: 0, end: 0 },
            });
        }
    }
    super::shift::shift_fn_item(&mut function, range.start);
    FrontOutcome::Ok(FrontOk {
        program: Program {
            items: vec![crate::source::ast::Item::Fn(function)],
        },
        grammar_id: format!("{}", cached.program.identity.grammar_id.0),
        caps_version: FRONT_CAPS_VERSION,
        snapshot_len: src.len(),
        cst: tree,
    })
}

/// 旧签名兼容桥：free fn 片段（`parse_function_fragment` 的统一前门版）。
pub fn parse_function_fragment_via_cst(
    src: &str,
    range: std::ops::Range<usize>,
    profile: SourceProfile,
) -> Result<FnItem, Diagnostic> {
    match parse_front(FrontRequest {
        src,
        profile,
        kind: FrontKind::FreeFnFragment { range },
        budget: FrontBudget::WORK_OWNED_64MIB,
    }) {
        FrontOutcome::Ok(mut ok) => match ok.program.items.remove(0) {
            crate::source::ast::Item::Fn(f) => {
                // PLAN-929（T-04）：计量保真——旧 parser 的 fn 片段路径在
                // fn_item_tail 内 note_parse；统一前门投影后按真实函数名
                // 补记（结构门按此对账"改函恰 1 次解析"）。
                crate::source::parser::note_parse(&f.name.name);
                Ok(f)
            }
            _ => Err(Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.infra",
                "fragment outcome carried no function".to_string(),
                crate::atom_text::Span { start: 0, end: 0 },
            )),
        },
        FrontOutcome::Diag(ds) => Err(super::first_diag(&ds)),
        FrontOutcome::ResourceExhausted { what, limit } => Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.resource",
            format!("resource exhausted: {what} (limit {limit})"),
            crate::atom_text::Span { start: 0, end: 0 },
        )),
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.infra",
            m,
            crate::atom_text::Span { start: 0, end: 0 },
        )),
    }
}

/// 旧签名兼容桥：lambda 片段（`parse_closure_fragment` 的统一前门版；
/// PLAN-929 T-03——closures_fn 暖编辑的片段生产者迁移入口）。
///
/// 合同同旧入口：授权区间内恰一个 lambda 表达式、完整消耗、span 平移回
/// 原源绝对 UTF-8 byte 坐标；同一 profile 语法程序（合成宿主
/// `fn __closure_unit() int { return <fragment> }` 走整模块请求，同
/// grammar/同投影/同预算——不是第二套语法）。
pub fn parse_closure_fragment_via_cst(
    src: &str,
    range: std::ops::Range<usize>,
    profile: SourceProfile,
) -> Result<crate::source::ast::Expr, Diagnostic> {
    use crate::source::ast::{ExprKind, Item, Stmt};
    let fragment = match src.get(range.clone()) {
        Some(f) => f,
        None => {
            return Err(Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.incremental-fragment",
                "fragment range must be ordered, in bounds and on UTF-8 boundaries",
                crate::atom_text::Span { start: 0, end: 0 },
            ));
        }
    };
    let prefix = "fn __closure_unit() int {\n    return ".len();
    let wrapped = format!("fn __closure_unit() int {{\n    return {fragment}\n}}\n");
    let outcome = parse_front(FrontRequest {
        src: &wrapped,
        profile,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    });
    let mut program = match outcome {
        FrontOutcome::Ok(ok) => ok.program,
        FrontOutcome::Diag(mut ds) => {
            // 诊断坐标回原源（宿主前缀内 ⇒ 片内相对 0 起点）。
            if let Some(d) = ds.first_mut() {
                let s = d.span.start.saturating_sub(prefix) + range.start;
                let e = d.span.end.saturating_sub(prefix) + range.start;
                d.span = crate::atom_text::Span {
                    start: s.min(range.end),
                    end: e.min(range.end),
                };
            }
            return Err(super::first_diag(&ds));
        }
        FrontOutcome::ResourceExhausted { what, limit } => {
            return Err(Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.resource",
                format!("resource exhausted: {what} (limit {limit})"),
                crate::atom_text::Span { start: 0, end: 0 },
            ));
        }
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => {
            return Err(Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.infra",
                m,
                crate::atom_text::Span { start: 0, end: 0 },
            ));
        }
    };
    let whole = crate::atom_text::Span {
        start: range.start,
        end: range.end,
    };
    let Some(Item::Fn(mut f)) = program.items.pop() else {
        return Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.infra",
            "closure wrapper carried no fn".to_string(),
            crate::atom_text::Span { start: 0, end: 0 },
        ));
    };
    let mut stmts = std::mem::take(&mut f.body.stmts);
    if stmts.len() != 1 || !matches!(stmts[0], Stmt::Return { .. }) {
        return Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.incremental-fragment",
            "a lambda fragment must contain exactly one lambda expression",
            whole,
        ));
    }
    let Some(Stmt::Return { mut value, .. }) = stmts.pop() else {
        unreachable!("checked Return above");
    };
    let is_closure = matches!(value.kind, ExprKind::Closure { .. });
    // 宿主相对 → 片内相对（−prefix）→ 原源绝对（+range.start）。
    super::units::shift_expr(&mut value, range.start as i64 - prefix as i64);
    // 完整消耗合同：闭包表达式尾必须到达片段尾（return 语法保证单表达式，
    // 但片段尾随垃圾会让投影失败——由上面的整模块诊断面覆盖）。
    if !is_closure {
        return Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.incremental-fragment",
            "a lambda fragment must be a lambda expression",
            whole,
        ));
    }
    match value.kind {
        ExprKind::Closure { .. } => Ok(value),
        _ => unreachable!("checked above"),
    }
}

/// 旧签名兼容桥：方法片段（`parse_method_fragment` 的统一前门版；
/// PLAN-929 T-05——a2x-store 方法/ranged 暖编辑的片段生产者迁移入口）。
///
/// 合同同旧入口：区间内恰一个方法声明（`static fn`/`fn` 前缀——receiver
/// 由片段语法决定，owner 由调用方提供）、完整消耗、span 平移回原源绝对
/// 坐标。统一语法中 `static fn` 仅在 type 块内合法——片段经合成宿主
/// `type __method_unit { __host_field int <fragment> }` 走整模块请求（占位
/// 字段满足 record 面最少一字段约束；同 grammar/同投影/
/// 同预算），提取唯一方法成员后有符号平移回绝对坐标。
pub fn parse_method_fragment_via_cst(
    src: &str,
    range: std::ops::Range<usize>,
    profile: SourceProfile,
    owner: &str,
) -> Result<FnItem, Diagnostic> {
    let fragment = match src.get(range.clone()) {
        Some(f) => f,
        None => {
            return Err(Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.incremental-fragment",
                "fragment range must be ordered, in bounds and on UTF-8 boundaries",
                crate::atom_text::Span { start: 0, end: 0 },
            ));
        }
    };
    // receiver 形状来自片段语法（镜像类型块成员 parser：`static fn` 从
    // `static` 起、`fn` 从 `fn` 起——首词判定与 token peek 等价）。
    let mut words = fragment.split_whitespace();
    let receiver = match (words.next(), words.next()) {
        (Some("static"), Some("fn")) => Receiver::Static,
        (Some("fn"), _) => Receiver::View,
        _ => {
            return Err(Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.incremental-fragment",
                "a method fragment must start with `static fn` or `fn`",
                crate::atom_text::Span {
                    start: range.start,
                    end: range.end,
                },
            ));
        }
    };
    let prefix = "type __method_unit {
    __host_field int
    "
    .len();
    let wrapped = format!(
        "type __method_unit {{
    __host_field int
    {fragment}
}}
"
    );
    let outcome = parse_front(FrontRequest {
        src: &wrapped,
        profile,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    });
    let mut program = match outcome {
        FrontOutcome::Ok(ok) => ok.program,
        FrontOutcome::Diag(mut ds) => {
            let delta = range.start as i64 - prefix as i64;
            for d in ds.iter_mut() {
                super::shift::shift_diag(d, delta);
            }
            return Err(super::first_diag(&ds));
        }
        FrontOutcome::ResourceExhausted { what, limit } => {
            return Err(Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.resource",
                format!("resource exhausted: {what} (limit {limit})"),
                crate::atom_text::Span { start: 0, end: 0 },
            ));
        }
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => {
            return Err(Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.infra",
                m,
                crate::atom_text::Span { start: 0, end: 0 },
            ));
        }
    };
    let whole = crate::atom_text::Span {
        start: range.start,
        end: range.end,
    };
    let Some(crate::source::ast::Item::Type(ty)) = program.items.pop() else {
        return Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.infra",
            "method wrapper carried no type block".to_string(),
            crate::atom_text::Span { start: 0, end: 0 },
        ));
    };
    let mut methods = ty.methods;
    if methods.len() != 1 {
        return Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.incremental-fragment",
            "a method fragment must contain exactly one method declaration",
            whole,
        ));
    }
    let mut f = methods.remove(0);
    // 宿主相对 → 片内相对（−prefix）→ 原源绝对（+range.start）。
    super::shift::shift_fn_item_signed(&mut f, range.start as i64 - prefix as i64);
    // owner 由调用方提供（同旧合同）；receiver 校验：投影按语法给出。
    if f.receiver != receiver {
        return Err(Diagnostic::new(
            crate::atom_text::Stage::Text,
            "source.infra",
            format!(
                "receiver mismatch: prefix says {receiver:?}, projection says {:?}",
                f.receiver
            ),
            crate::atom_text::Span { start: 0, end: 0 },
        ));
    }
    f.owner = Some(crate::source::ast::Ident {
        name: owner.to_string(),
        span: crate::atom_text::Span {
            start: range.start,
            end: range.start,
        },
    });
    // PLAN-929（T-05）：计量保真——按真实函数名补记解析（结构门对账）。
    crate::source::parser::note_parse(&f.name.name);
    Ok(f)
}
