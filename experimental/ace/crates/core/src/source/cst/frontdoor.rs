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
// PLAN-941 r2: condition/body braces are distinct from record constructors.
pub const FRONT_CAPS_VERSION: &str = "ac-source-front/4";

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
        // PLAN-961：数值面 slug。
        SourceProfile::Numeric => "numeric",
        // PLAN-962：聚合面 slug。
        SourceProfile::Aggregates => "aggregates",
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
        // PLAN-961：数值面 slug 反向映射。
        "numeric" => SourceProfile::Numeric,
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

static GRAMMARS: OnceLock<[OnceLock<CachedGrammar>; 11]> = OnceLock::new();

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

/// PLAN-963（T-02）：`record-instances` 裁剪程序（record 全表面 + 有限模板
/// 面）。独立缓存槽：plain record 程序逐字节不变，grammar-id 自然隔离。
static RECORD_INSTANCES_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();

/// record-instances 裁剪编译并缓存（进程内一次）。
pub fn record_instances_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&RECORD_INSTANCES_GRAMMAR, "record-instances")
}

/// PLAN-963（T-02）：`text-record-instances`（文本+记录+模板面）独立缓存槽。
static TEXT_RECORD_INSTANCES_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();

/// text-record-instances 裁剪编译并缓存（进程内一次）。
pub fn text_record_instances_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&TEXT_RECORD_INSTANCES_GRAMMAR, "text-record-instances")
}

static MODULES_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();
pub(crate) fn module_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&MODULES_GRAMMAR, "scalar-modules")
}

static PACKAGES_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();
/// PLAN-960（T-02）：完整模块包面裁剪编译并缓存（进程内一次）。旧
/// `scalar-modules` 裁剪与 GrammarId 不受影响。
pub fn packages_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&PACKAGES_GRAMMAR, "packages-modules")
}

/// PLAN-1043（T-03）：`packages-ct` 裁剪程序（packages-modules 全表面 +
/// `#in`/`pub spec fn`/单行 use 符号表）。独立缓存槽：plain
/// packages-modules 程序逐字节不变，grammar-id 自然隔离。
static PACKAGES_CT_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();

/// packages-ct 裁剪编译并缓存（进程内一次）。
pub fn packages_ct_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&PACKAGES_CT_GRAMMAR, "packages-ct")
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

/// PLAN-960（T-02）：packages-module 前门（与 950 模块前门同一严格投影；
/// items 以 TextRecord 域投影，模块元数据从 CST 字段读取）。
pub(crate) fn parse_packages_front(
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    parse_packages_with(false, src, budget)
}

/// PLAN-1043（T-03）：packages-ct 前门（`ac.source.cte.in/v1` 能力线；
/// `#in`/`pub spec fn` 面的同一严格投影）。
pub(crate) fn parse_packages_front_ct(
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    parse_packages_with(true, src, budget)
}

fn infra_err(e: String) -> Vec<Diagnostic> {
    vec![Diagnostic::new(
        crate::atom_text::Stage::Text,
        "source.infra",
        e,
        crate::atom_text::Span { start: 0, end: 0 },
    )]
}

fn parse_packages_with(
    ct_face: bool,
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    use crate::atom_text::{Span, Stage};
    let err = |code, message: String, span| vec![Diagnostic::new(Stage::Text, code, message, span)];
    let zero = Span { start: 0, end: 0 };
    let cached = if ct_face {
        packages_ct_grammar()
    } else {
        packages_grammar()
    }
    .map_err(infra_err)?;
    crate::source::parser::note_tokenize(Span {
        start: 0,
        end: src.len(),
    });
    crate::body_work::record_whole(crate::body_work::Stage::WholeParse);
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
    project_packages_front(src, vec![parsed.tree], Some(parsed.tokens), ct_face)
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

/// PLAN-960（T-02）：packages-module 元数据投影——items 以 TextRecord 域
/// 走统一投影器（type/fn/const/var 项 + str/ctor/方法表达式），导入/重导出/
/// 可见性事实从 CST 字段读取，绝不靠剥 token 或拼源重建。
/// PLAN-1043（T-03）：`ct_face` = packages-ct 裁剪（`ac.source.cte.in/v1`
/// 能力线）——额外投影 `#in` 编译期命名空间上下文（结构保持到绑定期按
/// 事实求值）与 `pub spec fn` 需求声明；plain 面 `ct_face=false` 时这些
/// 规则不在裁剪内，行为逐字节不变。
pub(crate) fn project_packages_front(
    src: &str,
    trees: Vec<CstTree>,
    tokens: Option<Vec<ace_syntax_runtime::lexer::Token>>,
    ct_face: bool,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    use crate::atom_text::{Span, Stage};
    use crate::source::packages::{PackageImport, PackageSyntax};
    let err = |code, message: String, span| vec![Diagnostic::new(Stage::Text, code, message, span)];
    let cached = if ct_face {
        packages_ct_grammar()
    } else {
        packages_grammar()
    }
    .map_err(|e| err("source.infra", e, Span { start: 0, end: 0 }))?;
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
    let use_item_p = rule("use-item-p");
    let use_tail_p = rule("use-tail-p");
    let use_alias = rule("use-alias");
    let public_item = rule("public-item");
    let const_item = rule("const-item");
    let var_item = rule("var-item");
    // PLAN-1043（T-03）：#in/需求规则 id（仅 packages-ct 裁剪存在；plain 面
    // 全 None → 分支不可达，行为不变）。
    let fn_require_p = rule("fn-require-p");
    let in_item = rule("in-item");
    let in_use_rule = rule("in-use");
    // public-item 的 `body` 字段指向 public-body 分派节点；实体是其首个非 trivia 孩子。
    let public_body = |tree: &CstTree, node: u32| -> Option<u32> {
        field(tree, node, "body").and_then(|b| {
            tree.children(b)
                .into_iter()
                .find(|&c| tree.node(c).flags & ace_syntax_runtime::cst::FLAG_TRIVIA == 0)
        })
    };
    // 收集 use-path 节点内按源序排列的全部 ident 叶（head + 各 seg）。
    let path_segments = |tree: &CstTree, path_node: u32| -> Vec<String> {
        let mut idents: Vec<(u64, String)> = Vec::new();
        let mut stack = vec![path_node];
        while let Some(n) = stack.pop() {
            if tree.kind(n) == cached.kinds.tok_ident {
                let (a, _) = tree.abs_range(n);
                idents.push((a, text(tree, n)));
            }
            for c in tree.children(n).into_iter().rev() {
                stack.push(c);
            }
        }
        idents.sort_by(|a, b| a.0.cmp(&b.0));
        idents.into_iter().map(|(_, s)| s).collect()
    };
    let mut declarations = std::collections::BTreeMap::new();
    let mut exports = std::collections::BTreeSet::new();
    let mut type_exports = std::collections::BTreeSet::new();
    let mut const_exports = std::collections::BTreeSet::new();
    let mut imports = Vec::new();
    let mut requires = Vec::new();
    let mut ct_ins = Vec::new();
    let mut saw_decl = false;
    let mut items = Vec::new();
    let node_span = |tree: &CstTree, n| {
        let (start, end) = tree.abs_range(n);
        Span {
            start: start as usize,
            end: end as usize,
        }
    };
    for tree in &trees {
        // PLAN-963（V-G06）：模块级记录模板推迟到扁平程序后统单态化——跨模块
        // 使用点必须命中同一实例符号（包/模块身份已进符号）。
        let program = project::project_program_deferring_templates(
            &cached.program,
            &cached.kinds,
            tree,
            &snap,
            SourceProfile::TextRecord,
        )?;
        items.extend(program.items);
        // PLAN-1043（T-03）：#in 结构投影器（共享同一声明子树投影引擎）。
        let mut in_face = ct_face.then(|| InFace {
            cached,
            snap: &snap,
            tree,
            proj: project::in_decl_projector(
                &cached.program,
                &cached.kinds,
                tree,
                &snap,
                SourceProfile::TextRecord,
            ),
        });
        for d in &tree.decls {
            let node = if tree.kind(d.node) == cached.kinds.item {
                tree.children(d.node)
                    .into_iter()
                    .find(|&c| tree.node(c).flags & ace_syntax_runtime::cst::FLAG_TRIVIA == 0)
                    .unwrap_or(d.node)
            } else {
                d.node
            };
            let kind = tree.kind(node);
            // PLAN-1043（T-03）：`#in` 项——结构保持进 ct_ins；含任何 use 的
            // #in 属导入位（须在声明区前），只含声明的 #in 视作声明项。
            if Some(kind) == in_item {
                let span = node_span(tree, node);
                let has_use = subtree_has_kind(tree, node, in_use_rule);
                if has_use && saw_decl {
                    return Err(err(
                        "source.module-import-order",
                        "imports must precede declarations".into(),
                        span,
                    ));
                }
                if !has_use {
                    saw_decl = true;
                }
                let Some(face) = in_face.as_mut() else {
                    return Err(err(
                        "source.infra",
                        "`#in` requires the packages-ct face".into(),
                        span,
                    ));
                };
                let (namespace, tail) = face.in_item(node).map_err(|d| vec![d])?;
                ct_ins.push(crate::source::packages::PackageCtIn {
                    namespace,
                    span,
                    tail,
                    decl_slot: items.len(),
                    import_slot: imports.len(),
                });
                continue;
            }
            // use 项（含 public-item 内的 use 尾）——路径/别名字段读取。
            let use_node = if Some(kind) == use_item_p {
                Some(node)
            } else if Some(kind) == public_item {
                public_body(tree, node).filter(|&b| Some(tree.kind(b)) == use_tail_p)
            } else {
                None
            };
            if let Some(use_node) = use_node {
                let span = node_span(tree, node);
                if saw_decl {
                    return Err(err(
                        "source.module-import-order",
                        "imports must precede declarations".into(),
                        span,
                    ));
                }
                let path_node = field(tree, use_node, "path").expect("use path");
                let path = path_segments(tree, path_node);
                // `as x` 住在 use-alias 子节点内（非直接字段）。
                let alias = tree
                    .children(use_node)
                    .into_iter()
                    .find(|&c| Some(tree.kind(c)) == use_alias)
                    .and_then(|a| field(tree, a, "alias"))
                    .map(|a| text(tree, a));
                let reexport = Some(kind) == public_item;
                let bound = alias
                    .clone()
                    .unwrap_or_else(|| path.last().cloned().unwrap_or_default());
                if path.len() < 2 {
                    return Err(err(
                        "source.module-path",
                        "module paths need at least `module.item`".into(),
                        span,
                    ));
                }
                if imports
                    .iter()
                    .any(|i: &PackageImport| i.bound_name() == bound)
                {
                    return Err(err(
                        "source.module-duplicate-import",
                        format!("duplicate imported name `{bound}`"),
                        span,
                    ));
                }
                if reexport {
                    // 重导出别名进入本模块公共面（下游 `use mod.alias` 可见）。
                    exports.insert(bound.clone());
                }
                imports.push(PackageImport {
                    path,
                    alias,
                    reexport,
                    span,
                });
            } else if Some(kind) == public_item {
                saw_decl = true;
                let body = public_body(tree, node).expect("public body");
                let bkind = tree.kind(body);
                // PLAN-1043（T-02）：`pub spec fn` 需求声明（bodyless；只记
                // requires 清单，不进 items——body 由所选 provider 实现填充）。
                if Some(bkind) == fn_require_p {
                    let Some(face) = in_face.as_mut() else {
                        return Err(err(
                            "source.infra",
                            "`pub spec fn` requires the packages-ct face".into(),
                            node_span(tree, node),
                        ));
                    };
                    let require = face.require(body).map_err(|d| vec![d])?;
                    requires.push(require);
                    continue;
                }
                let (name, name_field) = if bkind == cached.kinds.fn_item {
                    let tail = tree
                        .children(body)
                        .into_iter()
                        .find(|&n| tree.kind(n) == cached.kinds.fn_tail)
                        .expect("fn tail");
                    (
                        text(tree, field(tree, tail, "name").expect("fn name")),
                        None,
                    )
                } else if bkind == cached.kinds.type_item.unwrap_or(u32::MAX) {
                    (
                        text(tree, field(tree, body, "name").expect("type name")),
                        Some("type"),
                    )
                } else {
                    (
                        text(tree, field(tree, body, "name").expect("const name")),
                        Some("const"),
                    )
                };
                declarations.insert(name.clone(), node_span(tree, node));
                exports.insert(name.clone());
                match name_field {
                    Some("type") => {
                        type_exports.insert(name);
                    }
                    Some("const") => {
                        const_exports.insert(name);
                    }
                    _ => {}
                }
            } else if Some(kind) == const_item || Some(kind) == var_item {
                saw_decl = true;
                let name = text(tree, field(tree, node, "name").expect("global name"));
                declarations.insert(name, node_span(tree, node));
            } else if kind == cached.kinds.fn_item {
                saw_decl = true;
                let tail = tree
                    .children(node)
                    .into_iter()
                    .find(|&n| tree.kind(n) == cached.kinds.fn_tail)
                    .expect("fn tail");
                let name = text(tree, field(tree, tail, "name").expect("fn name"));
                declarations.insert(name, node_span(tree, node));
            } else if kind == cached.kinds.type_item.unwrap_or(u32::MAX) {
                saw_decl = true;
                let name = text(tree, field(tree, node, "name").expect("type name"));
                declarations.insert(name, node_span(tree, node));
            }
        }
    }
    Ok(PackageSyntax {
        program: crate::source::ast::Program { items },
        exports,
        type_exports,
        const_exports,
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
                        crate::source::modules::DeclarationPlacement {
                            index,
                            mint: span,
                            current: span,
                        }
                    })
                    .collect();
                crate::source::modules::SyntaxPiece {
                    source: std::rc::Rc::from(src),
                    tree: Some(tree),
                    tokens: tokens.clone(),
                    placements,
                    stored_cst: None,
                }
            })
            .collect(),
        declarations,
        requires,
        ct_ins,
    })
}

/// 子树内是否存在指定 rule kind 的节点（迭代）。
fn subtree_has_kind(tree: &CstTree, node: u32, kind: Option<u32>) -> bool {
    let Some(kind) = kind else {
        return false;
    };
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if tree.kind(n) == kind {
            return true;
        }
        for c in tree.children(n).into_iter().rev() {
            stack.push(c);
        }
    }
    false
}

/// PLAN-1043（T-03）：`#in` 结构投影面（packages-ct）——按 CST 字段读取
/// namespace/is/if/for/use 与声明子树（声明复用统一投影引擎），结构保持
/// 进 [`crate::source::packages::PackageCtIn`]；求值归绑定。
struct InFace<'a> {
    cached: &'a CachedGrammar,
    snap: &'a Snapshot,
    tree: &'a CstTree,
    proj: project::InDeclProjector<'a>,
}

impl<'a> InFace<'a> {
    fn rule(&self, name: &str) -> Option<u32> {
        self.cached
            .program
            .rules
            .iter()
            .position(|r| r.name == name)
            .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
    }
    fn field(&self, n: u32, name: &str) -> Option<u32> {
        self.cached
            .program
            .field_id(name)
            .and_then(|f| self.tree.field_child(n, f))
    }
    fn text(&self, n: u32) -> String {
        let (a, b) = self.tree.abs_range(n);
        self.snap.read_str(a, b).expect("CST source range")
    }
    fn span_of(&self, n: u32) -> crate::atom_text::Span {
        let (start, end) = self.tree.abs_range(n);
        crate::atom_text::Span {
            start: start as usize,
            end: end as usize,
        }
    }
    fn nontrivia(&self, n: u32) -> Vec<u32> {
        self.tree
            .children(n)
            .into_iter()
            .filter(|&c| self.tree.node(c).flags & ace_syntax_runtime::cst::FLAG_TRIVIA == 0)
            .collect()
    }

    fn diag(&self, code: &'static str, msg: String, span: crate::atom_text::Span) -> Diagnostic {
        Diagnostic::new(crate::atom_text::Stage::Text, code, msg, span)
    }

    /// `pub spec fn` 需求签名（fn-require-p 节点）。
    fn require(
        &mut self,
        node: u32,
    ) -> Result<crate::source::packages::PackageRequire, Diagnostic> {
        let (name, params, ret) = self.proj.require(node);
        if let Some(d) = self.proj.first_diag() {
            return Err(d);
        }
        Ok(crate::source::packages::PackageRequire {
            name: name.name.clone(),
            params,
            ret,
            span: name.span,
        })
    }

    /// 一个 `#in` 项（in-item 节点）→ 结构（槽位由调用方回填）。
    fn in_item(
        &mut self,
        node: u32,
    ) -> Result<(String, crate::source::packages::CtInTail), Diagnostic> {
        let namespace = self.text(self.field(node, "namespace").expect("#in namespace"));
        let tail_node = self.field(node, "tail").expect("#in tail");
        let inner = self
            .nontrivia(tail_node)
            .into_iter()
            .find(|&c| self.tree.child_count(c) > 0)
            .expect("in-tail dispatch");
        let in_is = self.rule("in-is").expect("packages-ct carries in-is");
        let tail = if self.tree.kind(inner) == in_is {
            let scrutinee = self.ct_expr_field(inner, "scrutinee")?;
            let in_arm = self.rule("in-arm").expect("in-arm");
            let mut arms = Vec::new();
            for a in self
                .nontrivia(inner)
                .into_iter()
                .filter(|&c| self.tree.kind(c) == in_arm)
            {
                arms.push(self.arm(a)?);
            }
            crate::source::packages::CtInTail::Is { scrutinee, arms }
        } else {
            crate::source::packages::CtInTail::Block(self.block(inner)?)
        };
        Ok((namespace, tail))
    }

    fn arm(&mut self, node: u32) -> Result<crate::source::packages::CtInArm, Diagnostic> {
        let pattern = self.text(self.field(node, "pattern").expect("arm pattern"));
        let qual = self
            .nontrivia(node)
            .into_iter()
            .find(|&c| Some(self.tree.kind(c)) == self.rule("in-arm-qual"))
            .and_then(|q| self.field(q, "qual"))
            .map(|q| self.text(q));
        let body_node = self.field(node, "body").expect("arm body");
        let inner = self
            .nontrivia(body_node)
            .into_iter()
            .find(|&c| self.tree.child_count(c) > 0)
            .expect("arm body dispatch");
        let body = if Some(self.tree.kind(inner)) == self.rule("in-use") {
            crate::source::packages::CtArmBody::Use(self.ct_use(inner, false)?)
        } else {
            crate::source::packages::CtArmBody::Block(self.block(inner)?)
        };
        Ok(crate::source::packages::CtInArm {
            qual,
            pattern,
            span: self.span_of(node),
            body,
        })
    }

    /// in-block 节点 → 语句列表（list-nl 源序）。
    fn block(&mut self, node: u32) -> Result<Vec<crate::source::packages::CtInStmt>, Diagnostic> {
        let in_stmt = self.rule("in-stmt").expect("in-stmt");
        let mut out = Vec::new();
        for c in self.nontrivia(node) {
            if self.tree.kind(c) == in_stmt {
                // in-stmt 的分派实体是其首个非 trivia 孩子。
                let inner = self
                    .nontrivia(c)
                    .into_iter()
                    .find(|&g| self.tree.child_count(g) > 0)
                    .unwrap_or(c);
                out.push(self.stmt(inner)?);
            }
        }
        Ok(out)
    }

    fn ct_expr_field(
        &mut self,
        node: u32,
        field: &str,
    ) -> Result<crate::source::packages::CtExpr, Diagnostic> {
        let n = self
            .field(node, field)
            .unwrap_or_else(|| panic!("field `{field}` on node kind {}", node));
        self.ct_expr(n)
    }

    /// in-ct-expr 规则节点 → pratt 结果 → CtExpr。
    fn ct_expr(&mut self, node: u32) -> Result<crate::source::packages::CtExpr, Diagnostic> {
        let inner = self
            .nontrivia(node)
            .into_iter()
            .find(|&c| self.tree.child_count(c) > 0)
            .unwrap_or(node);
        self.ct_expr_node(inner)
    }

    fn ct_expr_node(&mut self, node: u32) -> Result<crate::source::packages::CtExpr, Diagnostic> {
        use crate::source::packages::{CtBinOp, CtExpr};
        let kind = self.tree.kind(node);
        let k = &self.cached.kinds;
        let op = if kind == k.op_eq {
            Some(CtBinOp::Eq)
        } else if kind == k.op_ne {
            Some(CtBinOp::Ne)
        } else if kind == k.op_lt {
            Some(CtBinOp::Lt)
        } else if kind == k.op_le {
            Some(CtBinOp::Le)
        } else if kind == k.op_gt {
            Some(CtBinOp::Gt)
        } else if kind == k.op_ge {
            Some(CtBinOp::Ge)
        } else if kind == k.op_andand {
            Some(CtBinOp::And)
        } else if kind == k.op_oror {
            Some(CtBinOp::Or)
        } else {
            None
        };
        if let Some(op) = op {
            let kids = self.nontrivia(node);
            if kids.len() < 2 {
                return Err(self.diag(
                    "source.cte.in-expr",
                    "a compile-time comparison needs exactly two operands".into(),
                    self.span_of(node),
                ));
            }
            // 操作数取首/末孩子（算子叶可居中——与通用表达式投影同形）。
            let lhs = self.ct_expr_node(kids[0])?;
            let rhs = self.ct_expr_node(kids[kids.len() - 1])?;
            let span = lhs.span().join(rhs.span());
            return Ok(CtExpr::Bin {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            });
        }
        // primary：in-ct-unary 包 int-literal/true/false/ns-ref。
        let inner = if Some(kind) == self.rule("in-ct-unary") {
            self.nontrivia(node)
                .into_iter()
                .find(|&c| self.tree.child_count(c) > 0)
                .ok_or_else(|| {
                    self.diag(
                        "source.cte.in-expr",
                        "expected a compile-time expression".into(),
                        self.span_of(node),
                    )
                })?
        } else {
            node
        };
        let ik = self.tree.kind(inner);
        if ik == self.cached.kinds.int_literal {
            let mut value = String::new();
            let mut stack = vec![inner];
            while let Some(c) = stack.pop() {
                if self.tree.kind(c) == self.cached.kinds.tok_minus {
                    value.push('-');
                }
                if self.tree.kind(c) == self.cached.kinds.tok_int_lit {
                    value.push_str(&self.text(c));
                }
                for g in self.nontrivia(c).into_iter().rev() {
                    stack.push(g);
                }
            }
            let v: i32 = value.parse().map_err(|_| {
                self.diag(
                    "source.cte.in-domain",
                    format!("`{value}` is outside the i32 compile-time domain"),
                    self.span_of(inner),
                )
            })?;
            return Ok(CtExpr::Int(v));
        }
        if ik == self.cached.kinds.tok_true {
            return Ok(CtExpr::Bool(true));
        }
        if ik == self.cached.kinds.tok_false {
            return Ok(CtExpr::Bool(false));
        }
        if Some(ik) == self.rule("ns-ref") {
            let head = self.field(inner, "head").expect("ns head");
            let member = self.field(inner, "member");
            return Ok(CtExpr::Ref {
                head: self.text(head),
                member: member.map(|m| self.text(m)),
                span: self.span_of(inner),
            });
        }
        Err(self.diag(
            "source.cte.in-expr",
            "compile-time expressions accept int/bool literals and namespace names only".into(),
            self.span_of(inner),
        ))
    }

    /// in-range 界限：分解形（start/end 字段）或融合单 token（`0..3` 被
    /// affix-num 连续字符集吞成一个词——按 `..` 拆分；负界与复杂表达式用
    /// 分解形书写）。
    fn range_bounds(
        &mut self,
        iter: u32,
    ) -> Result<
        (
            crate::source::packages::CtExpr,
            crate::source::packages::CtExpr,
        ),
        Diagnostic,
    > {
        use crate::source::packages::CtExpr;
        if let Some(both) = self.field(iter, "both") {
            let lit = self.field(both, "lit").expect("range lit");
            let text = self.text(lit);
            let span = self.span_of(lit);
            let (a, b) = text.split_once("..").ok_or_else(|| {
                self.diag(
                    "source.cte.in-expr",
                    format!("`{text}` is not a compile-time range"),
                    span,
                )
            })?;
            if a.is_empty() || b.is_empty() || b.contains("..") {
                return Err(self.diag(
                    "source.cte.in-expr",
                    format!("`{text}` is not a compile-time range"),
                    span,
                ));
            }
            let atom = |t: &str, off: usize| -> CtExpr {
                match t.parse::<i32>() {
                    Ok(v) => CtExpr::Int(v),
                    Err(_) => CtExpr::Ref {
                        head: t.to_string(),
                        member: None,
                        span: crate::atom_text::Span {
                            start: span.start + off,
                            end: span.start + off + t.len(),
                        },
                    },
                }
            };
            return Ok((atom(a, 0), atom(b, a.len() + 2)));
        }
        let start = self.ct_expr_field(iter, "start")?;
        let end = self.ct_expr_field(iter, "end")?;
        Ok((start, end))
    }

    /// 单行/块内 use（in-use 节点；reexport = `pub use` 形）。
    fn ct_use(
        &mut self,
        node: u32,
        reexport: bool,
    ) -> Result<crate::source::packages::CtUse, Diagnostic> {
        let path_node = self.field(node, "path").expect("use path");
        let path = self.ident_seq(path_node);
        if path.len() < 2 {
            return Err(self.diag(
                "source.module-path",
                "module paths need at least `module.item`".into(),
                self.span_of(node),
            ));
        }
        let syms = self
            .nontrivia(node)
            .into_iter()
            .find(|&c| Some(self.tree.kind(c)) == self.rule("in-use-syms"))
            .map(|s| self.ident_seq(s));
        let alias = self
            .nontrivia(node)
            .into_iter()
            .find(|&c| Some(self.tree.kind(c)) == self.rule("use-alias"))
            .and_then(|a| self.field(a, "alias"))
            .map(|a| self.text(a));
        if syms.is_some() && alias.is_some() {
            return Err(self.diag(
                "source.module-import-form",
                "a symbol-list import cannot also carry an `as` alias".into(),
                self.span_of(node),
            ));
        }
        Ok(crate::source::packages::CtUse {
            path,
            syms,
            alias,
            reexport,
            span: self.span_of(node),
        })
    }

    /// 子树内按源序收集全部 ident 叶。
    fn ident_seq(&self, node: u32) -> Vec<String> {
        let mut idents: Vec<(u64, String)> = Vec::new();
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            if self.tree.kind(n) == self.cached.kinds.tok_ident {
                let (a, _) = self.tree.abs_range(n);
                idents.push((a, self.text(n)));
            }
            for c in self.nontrivia(n).into_iter().rev() {
                stack.push(c);
            }
        }
        idents.sort_by(|a, b| a.0.cmp(&b.0));
        idents.into_iter().map(|(_, s)| s).collect()
    }

    /// 一条 in-stmt（分派实体节点）。
    fn stmt(&mut self, node: u32) -> Result<crate::source::packages::CtInStmt, Diagnostic> {
        use crate::source::packages::CtInStmt;
        let kind = self.tree.kind(node);
        if Some(kind) == self.rule("in-is") {
            let scrutinee = self.ct_expr_field(node, "scrutinee")?;
            let in_arm = self.rule("in-arm").expect("in-arm");
            let mut arms = Vec::new();
            for a in self
                .nontrivia(node)
                .into_iter()
                .filter(|&c| self.tree.kind(c) == in_arm)
            {
                arms.push(self.arm(a)?);
            }
            return Ok(CtInStmt::Is {
                scrutinee,
                arms,
                span: self.span_of(node),
            });
        }
        if Some(kind) == self.rule("in-if") {
            let cond = self.ct_expr_field(node, "cond")?;
            let then = self.block(self.field(node, "then").expect("then block"))?;
            let els = self
                .nontrivia(node)
                .into_iter()
                .find(|&c| Some(self.tree.kind(c)) == self.rule("in-else"))
                .and_then(|e| self.field(e, "body"))
                .map(|b| self.block(b))
                .transpose()?;
            return Ok(CtInStmt::If {
                cond,
                then,
                els,
                span: self.span_of(node),
            });
        }
        if Some(kind) == self.rule("in-for") {
            let var = self.text(self.field(node, "var").expect("for var"));
            let iter = self.field(node, "iter").expect("for iter");
            let (start, end) = self.range_bounds(iter)?;
            let body = self.block(self.field(node, "body").expect("for body"))?;
            return Ok(CtInStmt::For {
                var,
                start,
                end,
                body,
                span: self.span_of(node),
            });
        }
        if Some(kind) == self.rule("in-use") {
            return Ok(CtInStmt::Use(self.ct_use(node, false)?));
        }
        // 声明：fn/type/const/var 直书或 pub 壳。
        let (decl_node, public) = self.decl_target(node)?;
        let dk = self.tree.kind(decl_node);
        let item = if dk == self.cached.kinds.fn_item {
            crate::source::ast::Item::Fn(self.proj.fn_item(decl_node))
        } else if Some(dk) == self.cached.kinds.type_item {
            if self.proj.is_type_alias(decl_node) {
                return Err(self.diag(
                    "source.cte.in-decl",
                    "type aliases inside `#in` are not part of this surface".into(),
                    self.span_of(decl_node),
                ));
            }
            crate::source::ast::Item::Type(self.proj.type_item(decl_node))
        } else if Some(dk) == self.rule("const-item") {
            self.proj.global_item(decl_node, false)
        } else if Some(dk) == self.rule("var-item") {
            self.proj.global_item(decl_node, true)
        } else if Some(dk) == self.rule("fn-require-p") {
            return Ok(CtInStmt::Require(self.require(decl_node)?));
        } else if Some(dk) == self.rule("use-tail-p") {
            return Ok(CtInStmt::Use(self.ct_use(decl_node, true)?));
        } else {
            return Err(self.diag(
                "source.cte.in-stmt",
                "only `is`/`if`/`for`/`use`/declarations may appear inside `#in`".into(),
                self.span_of(node),
            ));
        };
        if let Some(d) = self.proj.first_diag() {
            return Err(d);
        }
        Ok(CtInStmt::Decl { item, public })
    }

    /// 声明分派：直书项 → 自身；public-item 壳 → 内层实体 + public=true。
    fn decl_target(&mut self, node: u32) -> Result<(u32, bool), Diagnostic> {
        let kind = self.tree.kind(node);
        if kind == self.cached.kinds.fn_item
            || Some(kind) == self.cached.kinds.type_item
            || Some(kind) == self.rule("const-item")
            || Some(kind) == self.rule("var-item")
        {
            return Ok((node, false));
        }
        if Some(kind) == self.rule("public-item") {
            let body = self.field(node, "body").expect("public body field");
            let inner = self
                .nontrivia(body)
                .into_iter()
                .find(|&c| self.tree.child_count(c) > 0)
                .ok_or_else(|| {
                    self.diag(
                        "source.cte.in-stmt",
                        "unsupported public item inside `#in`".into(),
                        self.span_of(node),
                    )
                })?;
            let ik = self.tree.kind(inner);
            if ik == self.cached.kinds.fn_item
                || Some(ik) == self.cached.kinds.type_item
                || Some(ik) == self.rule("const-item")
                || Some(ik) == self.rule("fn-require-p")
                || Some(ik) == self.rule("use-tail-p")
            {
                return Ok((inner, true));
            }
            return Err(self.diag(
                "source.cte.in-stmt",
                "unsupported public item inside `#in`".into(),
                self.span_of(node),
            ));
        }
        Err(self.diag(
            "source.cte.in-stmt",
            "only `is`/`if`/`for`/`use`/declarations may appear inside `#in`".into(),
            self.span_of(node),
        ))
    }
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
        // PLAN-961：数值面槽位。
        SourceProfile::Numeric => 9,
        // PLAN-962：聚合面槽位。
        SourceProfile::Aggregates => 10,
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
/// `FnItem::type_params`/`TypeRef::TyVar`）；PLAN-963（T-02）同一能力在
/// Record 面上以 `record-instances` 裁剪（record 全表面 + 模板面）；其余
/// 组合与 [`parse_front`] 逐字节同面（无能力仍原拒绝）。片段 kind 在
/// instances 面不支持（增量片段合同仍走 plain 面——模板函数编辑的增量
/// 路径由 T-07 冻结）。
pub fn parse_front_with_caps(req: FrontRequest, capabilities: &[String]) -> FrontOutcome {
    // PLAN-936（T-04）：CT 能力优先——scalar-ct 面包含 scalar-instances 全
    // 表面（CT 可作用于模板实例），不与 instances 面叠加选择。
    let ct =
        capabilities.iter().any(|c| c == CT_CAPABILITY) && req.profile == SourceProfile::Scalar;
    let instances = !ct
        && capabilities.iter().any(|c| c == INSTANCES_CAPABILITY)
        && req.profile == SourceProfile::Scalar;
    // PLAN-963（T-02）：同一实例能力在 Record 面上以 `record-instances`
    // 裁剪解析（record 全表面 + fn `<T>`）；其他 profile 组合维持原面。
    let record_instances = !ct
        && !instances
        && capabilities.iter().any(|c| c == INSTANCES_CAPABILITY)
        && req.profile == SourceProfile::Record;
    // PLAN-963（T-02）：文本+记录面上的同一实例能力（文本实参的泛型函数）。
    let text_record_instances = !ct
        && !instances
        && !record_instances
        && capabilities.iter().any(|c| c == INSTANCES_CAPABILITY)
        && req.profile == SourceProfile::TextRecord;
    if !instances && !ct && !record_instances && !text_record_instances {
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
    } else if record_instances {
        record_instances_grammar()
    } else if text_record_instances {
        text_record_instances_grammar()
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
        ct,
        // PLAN-963（T-04）：实例面（scalar/record/text-record + 能力）是
        // `<const N>` 与 `generic-call` 的唯一成立面。
        instances || record_instances || text_record_instances,
        // PLAN-963（V-G06）：记录面就地单态化（模块/包面走上面的推迟入口）。
        false,
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
