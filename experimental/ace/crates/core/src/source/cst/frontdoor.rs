//! 统一前门：typed 请求/结果、语法程序缓存、显式预算与兼容桥。

use super::kinds::SourceKinds;
use super::project;
use crate::atom_text::Diagnostic;
use crate::source::ast::{FnItem, Program, Receiver};
use crate::source::SourceProfile;
use ace_syntax_runtime::cst::CstTree;
use ace_syntax_runtime::lexer::CompiledScanner;
use ace_syntax_runtime::parse::{parse_snapshot, parse_snapshot_with_ctx, ParseOutput};
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
    /// PLAN-979：领域候选（未受信；只有 AGE 领域 profile 有内容）。通用
    /// profile 恒为空——领域声明不进入普通 AST，必须经领域前门消费。
    pub age_nodes: Vec<crate::source::age_h0::AgeNodeItem>,
    /// PLAN-979 T-03：组件面候选（未受信；只有 age-component profile 有内容）。
    pub component_nodes: Vec<crate::source::age_component::ComponentNode>,
}

/// PLAN-979：槽位限定拼写的事实源（真实快照字节；本 crate 不复制源）。
pub(crate) struct SnapshotTokenText<'s> {
    snap: &'s Snapshot,
}

impl<'s> ace_syntax_runtime::parse::CtxTokenText for SnapshotTokenText<'s> {
    fn token_equals(&self, start: u64, end: u64, want: &str) -> bool {
        self.snap
            .read_bytes(start, end)
            .map(|b| b == want.as_bytes())
            .unwrap_or(false)
    }
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
        // PLAN-979 T-03：组件面（age-h0 的同源异裁剪）。
        SourceProfile::AgeComponent => "age-component",
        // PLAN-981: the AGE package trim (package modules ∪ AGE components).
        SourceProfile::AgePackage => "age-package",
        SourceProfile::HostConsole => "host-console",
        // PLAN-928 T-02/T-03：闭包/集合两面经统一前门（差分全集与冷入口
        // 切换由 T-04/T-07 冻结）。
        SourceProfile::Closures => "closures",
        SourceProfile::Collections => "collections",
        // PLAN-961：数值面 slug。
        SourceProfile::Numeric => "numeric",
        // PLAN-962：聚合面 slug。
        SourceProfile::Aggregates => "aggregates",
        // PLAN-968：调度器面 slug（聚合超集 + task actor 面）。
        SourceProfile::Scheduler => "scheduler",
        // PLAN-1078：Auto 面 slug（aggregates 面身份）。
        SourceProfile::Auto => "auto",
        // PLAN-979：AGE 领域面 slug（Schema 数据键，非产品名）。
        SourceProfile::AgeH0 => "age-h0",
        // PLAN-982（B-CONFIG-I）：配置文档面 slug（grammar 裁剪名；不与产品名混用）。
        SourceProfile::ConfigDocument => "config-document",
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
        "aggregates" => SourceProfile::Aggregates,
        // PLAN-1078: the Auto slug round-trip.
        "auto" => SourceProfile::Auto,
        "age-h0" => SourceProfile::AgeH0,
        // PLAN-981: AGE package slug round-trip.
        "age-package" => SourceProfile::AgePackage,
        // PLAN-982（B-CONFIG-I）：配置文档面 slug 反向映射。
        "config-document" => SourceProfile::ConfigDocument,
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

static GRAMMARS: OnceLock<[OnceLock<CachedGrammar>; 17]> = OnceLock::new();

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

/// PLAN-946（T-02）：`packages-aggregates` 裁剪程序（packages-modules 全表面
/// + 集合族最小语句面）。独立缓存槽：plain packages-modules/packages-ct
/// 程序逐字节不变，grammar-id 自然隔离。
static PACKAGES_AGGREGATES_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();

/// packages-aggregates 裁剪编译并缓存（进程内一次）。
pub fn packages_aggregates_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&PACKAGES_AGGREGATES_GRAMMAR, "packages-aggregates")
}

/// PLAN-981（IF-981-MODULE）：`age-package` 裁剪程序（packages-modules 全表面
/// ∪ AGE 组件面）。独立缓存槽：其它三个包面与三个 AGE 面的裁剪字节不变，
/// grammar-id 自然隔离。
static PACKAGES_AGE_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();

/// age-package 裁剪编译并缓存（进程内一次）。
pub fn packages_age_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&PACKAGES_AGE_GRAMMAR, "age-package")
}

/// PLAN-972（T-02）：`packages-aggregates-ct` 裁剪程序（packages-aggregates
/// 全表面 + packages-ct 装配项 + `fn(T..) -> R` 闭包类型注解/闭包字面量）。
/// 独立缓存槽：plain packages-modules/packages-ct/packages-aggregates 程序
/// 逐字节不变，grammar-id 自然隔离。
static PACKAGES_AGGREGATES_CT_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();

/// packages-aggregates-ct 裁剪编译并缓存（进程内一次）。
pub fn packages_aggregates_ct_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&PACKAGES_AGGREGATES_CT_GRAMMAR, "packages-aggregates-ct")
}

static AUTO_PACKAGES_GRAMMAR: OnceLock<OnceLock<CachedGrammar>> = OnceLock::new();

pub(crate) fn auto_packages_grammar() -> Result<&'static CachedGrammar, String> {
    capability_grammar(&AUTO_PACKAGES_GRAMMAR, "auto-packages")
}

pub(crate) fn parse_packages_front_auto(
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    parse_packages_with(
        PackagesFace::Auto,
        src,
        budget,
        &crate::source::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
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
    parse_packages_with(
        PackagesFace::Modules,
        src,
        budget,
        &crate::source::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
}

/// PLAN-1043（T-03）：packages-ct 前门（`ac.source.cte.in/v1` 能力线；
/// `#in`/`pub spec fn` 面的同一严格投影）。
pub(crate) fn parse_packages_front_ct(
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    parse_packages_with(
        PackagesFace::Ct,
        src,
        budget,
        &crate::source::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
}

/// PLAN-946（T-02）：packages-aggregates 前门（PLAN-962 `av.aggregates/1`
/// 准入的 aggregates packages 线；packages-modules 全表面 + 集合族最小
/// 语句面 for-in/is + `?T` 可选类型位 + `List<T>.member` 接收原子）。
pub(crate) fn parse_packages_front_aggregates(
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    parse_packages_with(
        PackagesFace::Aggregates,
        src,
        budget,
        &crate::source::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
}

/// PLAN-981（IF-981-MODULE）：`age-package` 前门（包模块图 ∪ AGE 组件）。
/// `node_base` = 本模块在**包级组件声明序**里的起始下标（多模块身份）。
pub(crate) fn parse_packages_front_age(
    src: &str,
    budget: FrontBudget,
    scenario: &crate::source::age_component::ComponentScenario,
    node_base: usize,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    parse_packages_with(PackagesFace::Age, src, budget, scenario, node_base)
}

/// PLAN-972（T-02）：packages-aggregates-ct 前门（装配 aggregates 线；
/// `ac.source.cte.in/v1` + `av.aggregates/1` 双准入，`#in`/`pub spec fn`/
/// 闭包类型注解面的同一严格投影）。
pub(crate) fn parse_packages_front_aggregates_ct(
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    parse_packages_with(
        PackagesFace::AggregatesCt,
        src,
        budget,
        &crate::source::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
}

/// 包面前门的三种裁剪面（旧布尔 `ct_face` 的结构化形态）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PackagesFace {
    /// packages-modules（旧 `false`）。
    Modules,
    Auto,
    /// packages-ct（PLAN-1043；旧 `true`）。
    Ct,
    /// packages-aggregates（PLAN-946；ct 专属规则在本面裁剪缺席→不可达）。
    Aggregates,
    /// PLAN-981（IF-981-MODULE）：`age-package` —— 包模块面 ∪ AGE 组件面。
    /// 与其它三面的差别只有两点：解析用该裁剪，且本面额外投影 AGE 候选。
    Age,
    /// packages-aggregates-ct（PLAN-972；aggregates 全表面 + ct 装配项 +
    /// 闭包类型注解/字面量——std 装配标准库的承载面）。
    AggregatesCt,
}

impl PackagesFace {
    fn grammar(&self) -> Result<&'static CachedGrammar, String> {
        match self {
            PackagesFace::Modules => packages_grammar(),
            PackagesFace::Auto => auto_packages_grammar(),
            PackagesFace::Ct => packages_ct_grammar(),
            PackagesFace::Aggregates => packages_aggregates_grammar(),
            PackagesFace::Age => packages_age_grammar(),
            PackagesFace::AggregatesCt => packages_aggregates_ct_grammar(),
        }
    }
    fn ct_face(&self) -> bool {
        matches!(self, PackagesFace::Ct | PackagesFace::AggregatesCt)
    }
    /// PLAN-972（T-02）：aggregates 域投影（集合族语句/`?T`/闭包类型形状按
    /// 962 域投影）；plain/ct 面保持 TextRecord。
    fn aggregates_proj(&self) -> bool {
        matches!(self, PackagesFace::Aggregates | PackagesFace::AggregatesCt)
    }
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
    face: PackagesFace,
    src: &str,
    budget: FrontBudget,
    scenario: &crate::source::age_component::ComponentScenario,
    node_base: usize,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    use crate::atom_text::{Span, Stage};
    let err = |code, message: String, span| vec![Diagnostic::new(Stage::Text, code, message, span)];
    let zero = Span { start: 0, end: 0 };
    let cached = face.grammar().map_err(infra_err)?;
    crate::source::parser::note_tokenize(Span {
        start: 0,
        end: src.len(),
    });
    crate::body_work::record_whole(crate::body_work::Stage::WholeParse);
    let pages = std::rc::Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&pages, src);
    // PLAN-981：`age-package` 裁剪带 `contextual` 槽位（`node`/`model`/`on`…），
    // 解析必须拿到**真实快照字节**才能识别限定拼写；其它包面无该表，走原
    // 无上下文入口（字节与诊断面不变）。
    let ctx_text = SnapshotTokenText { snap: &snap };
    let parsed = if matches!(face, PackagesFace::Age) {
        parse_snapshot_with_ctx(
            &cached.program,
            &cached.scanner,
            &snap,
            &Budget::from(budget),
            Some(&ctx_text),
        )
    } else {
        parse_snapshot(
            &cached.program,
            &cached.scanner,
            &snap,
            &Budget::from(budget),
        )
    }
    .map_err(|e| match e {
        ace_syntax_runtime::program::RtError::ResourceExhausted { what, limit } => err(
            "source.resource",
            format!("resource exhausted: {what} (limit {limit})"),
            zero,
        ),
        other => err("source.infra", format!("parse: {other}"), zero),
    })?;
    project_packages_front(
        src,
        vec![parsed.tree],
        Some(parsed.tokens),
        face,
        scenario,
        node_base,
    )
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
    face: PackagesFace,
    scenario: &crate::source::age_component::ComponentScenario,
    // PLAN-981（W1 跨模块实例身份）：**包级**组件基址——本模块的 node 序号从
    // 它开始编号（多模块时逐模块局部序号会让两模块的 node 0 撞号：发射的
    // `__age_state_n<k>_…` 与读回/AV 的包级序号对不上，UI058 T-05 实测）。
    node_base: usize,
) -> Result<crate::source::packages::PackageSyntax, Vec<Diagnostic>> {
    use crate::atom_text::{Span, Stage};
    use crate::source::packages::{PackageImport, PackageSyntax};
    let err = |code, message: String, span| vec![Diagnostic::new(Stage::Text, code, message, span)];
    let ct_face = face.ct_face();
    let cached = face
        .grammar()
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
    let auto_var_item = rule("auto-var-item");
    // PLAN-981（IF-981-REF）：`pub node Foo {..}` —— 公共 node 声明的导出面。
    let age_pub_node = rule("age-pub-node-item");
    // PLAN-981（IF-981-ORIGINAL）：模块级 `let` 与符号表导入。
    let age_let_item = rule("age-let-item");
    let age_use_sym = rule("age-use-sym-item");
    let age_use_sym_tail = rule("age-use-sym-tail");
    // PLAN-980 ④-A：`use <module>.<name>: a, b` 点分成员导入（包面此前零投影）。
    let age_use_module = rule("age-use-module-item");
    let age_use_import_tail = rule("age-use-import-tail");
    // PLAN-980 P3：`style NAME [(params)] = <text>`（仅 age-package；str const/fn 糖）。
    let style_item = rule("style-item");
    let style_param = rule("style-param");
    let style_param_ty_ann = rule("style-param-ty-ann");
    let style_param_default = rule("style-param-default");
    // PLAN-1043（T-03）/PLAN-966（T-02）：项位 CT 选择/需求规则 id（仅
    // packages-ct 裁剪存在；plain 面全 None → 分支不可达，行为不变）。
    let fn_require_p = rule("fn-require-p");
    let ct_item = rule("ct-item");
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
    // PLAN-980 ④-A：导入项自身的 span（引用集要把导入拼写排除在外）。
    let mut import_spans: Vec<(usize, usize)> = Vec::new();
    // PLAN-980 P3：style 字面量表（用法投影期替换用）与待解析的 ident 复制。
    let mut style_lits: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    let mut style_copies: Vec<(String, String, Span, bool)> = Vec::new();
    let mut requires = Vec::new();
    let mut ct_items = Vec::new();
    // PLAN-981：本模块的 AGE 领域候选（仅 age-package 面有内容）。
    let mut age_components = Vec::new();
    let mut saw_decl = false;
    let mut items = Vec::new();
    let node_span = |tree: &CstTree, n| {
        let (start, end) = tree.abs_range(n);
        crate::atom_text::Span {
            start: start as usize,
            end: end as usize,
        }
    };
    for tree in &trees {
        // PLAN-963（V-G06）：模块级记录模板推迟到扁平程序后统单态化——跨模块
        // 使用点必须命中同一实例符号（包/模块身份已进符号）。
        // PLAN-946（T-02）：aggregates 面按 Aggregates 域投影（集合族语句/
        // 表达式语句/`?T` 形状按 962 域投影）；plain/ct 面保持 TextRecord。
        // PLAN-972（T-02）：packages-aggregates-ct 同按 Aggregates 域投影
        //（其 #in 落位声明共享同一域）。
        let proj_profile = if face == PackagesFace::Auto {
            SourceProfile::Auto
        } else if face.aggregates_proj() {
            SourceProfile::Aggregates
        } else {
            SourceProfile::TextRecord
        };
        let program = project::project_program_deferring_templates(
            &cached.program,
            &cached.kinds,
            tree,
            &snap,
            proj_profile,
            // PLAN-981：AGE 作者面（age-package 裁剪）——浮点字面量等合法。
            matches!(face, PackagesFace::Age),
        )?;
        items.extend(program.items);
        // PLAN-981（IF-981-MODULE）：AGE 领域候选从**同一次** CST 里投影，
        // 与普通包项并排记录（结构名按 Schema 的 role 归属，不按拼写猜）。
        // 只有 `age-package` 裁剪存在 `age-node-c-item` 规则，其余面保持空集。
        //
        // 每条文本叶随即脱糖成**普通隐藏函数**并进本模块 items —— 这样共享
        // 绑定器会像对待普通函数一样给它 mangle 符号/平移 span/走唯一 verify，
        // 领域层不另建求值路径。
        if matches!(face, PackagesFace::Age) {
            // PLAN-980 P3（981 (d)）：`style: <name>(…)` 调用形在投影**之前**具名拒——
            // 组件投影会把文本属性脱糖成隐藏值函数，事后无法区分"用户调用"与普通调用。
            if let Some(sk) = style_item {
                let mut style_names: std::collections::BTreeSet<String> =
                    std::collections::BTreeSet::new();
                let mut stack = vec![tree.root];
                while let Some(n) = stack.pop() {
                    if tree.kind(n) == sk {
                        if let Some(nm) = field(tree, n, "name") {
                            style_names.insert(text(tree, nm));
                        }
                    }
                    for c in tree.children(n) {
                        stack.push(c);
                    }
                }
                if !style_names.is_empty() {
                    let entry_kind = rule("age-view-entry");
                    let mut stack = vec![tree.root];
                    while let Some(n) = stack.pop() {
                        if Some(tree.kind(n)) == entry_kind {
                            if let Some(k) = field(tree, n, "key") {
                                if text(tree, k) == "style" {
                                    if let Some(v) = field(tree, n, "value") {
                                        let spelled = text(tree, v);
                                        if let Some(open) = spelled.find('(') {
                                            let callee = spelled[..open].trim().to_string();
                                            if style_names.iter().any(|x| *x == callee) {
                                                return Err(err(
                                                    "source.age.style-call",
                                                    format!(
                                                        "style call `{callee}(…)` is a named \
                                                         pending surface in this slice \
                                                         (parameterised style calls are not folded)"
                                                    ),
                                                    node_span(tree, v),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        for c in tree.children(n) {
                            stack.push(c);
                        }
                    }
                }
            }
            let nodes = project::project_component_nodes(
                &cached.program,
                &cached.kinds,
                tree,
                &snap,
                SourceProfile::AgePackage,
            )?;
            // PLAN-981（W1 跨模块实例身份，2026-11-18 第六批）：场景的
            // `Send`/`SendArgs` **声明级**核对（arity/载荷域）移到包级一次核
            // ——`ForNode{node}` 的下标是**包级组件声明序**，在逐模块循环里用
            // 本模块的 nodes 索引会误拒跨模块定向（UI058 T-05 实测：两模块各
            // 一个 node 时 `ForNode{1}` 报 "package declares 1 component(s)"）。
            // 逐 node 的**执行**脱糖本来就用包级 `node_index`（`instance_stmts`
            // 按 `op.node_scope()` 过滤），故这里删掉局部预检不影响语义。
            for node in nodes {
                // R64-4：包内节点下标——多 node 同名字段的状态函数需要它消歧。
                let node_index = node_base + age_components.len();
                let mut text_fns = Vec::new();
                // PLAN-981 B-W1（IF-981-PROOF-AV）：场景属性值函数的登记事实
                //（AV 观察通道的入口；与发射的 `{prefix}scene_*` 函数同序）。
                let mut scene_fns: Vec<crate::source::age_package::AgeSceneValueFn> = Vec::new();
                // PLAN-981 B-W1（IF-981-PARAMS-B）：未派发 handler 的声明语义
                // 检查函数（check_value 闭包独占 items 期间先收集，最后统一入列）。
                let mut handler_check_items: Vec<crate::source::ast::Item> = Vec::new();
                // PLAN-981：把场景里的 `Tick` 归一到本 node 实际声明的生命周期
                // 消息拼写（`Life.Tick` 优先），再交给共享脱糖/检查。
                let node_scenario =
                    crate::source::age_package::resolve_tick_spelling(scenario, &node);
                // `model` 字段：按场景实例数脱糖成普通状态函数
                // `__age_state_<i>_<field>()`（init + 该实例的定向 ops + 返回）。
                // 静态面（`instances == 0`）仍生成**实例 0** 的状态，否则文本叶里的
                // 字段引用没有绑定——"静态"只表示不额外复制实例，不等于没有 model。
                let state_instances = if node_scenario.instances == 0 {
                    1
                } else {
                    node_scenario.instances
                };
                for inst in 0..state_instances {
                    for f in &node.fields {
                        let mut stmts = match crate::source::age_component::instance_stmts(
                            inst,
                            node_index,
                            &node,
                            &node_scenario,
                            &std::collections::HashMap::new(),
                        ) {
                            Ok(s) => s,
                            Err(ds) => return Err(ds),
                        };
                        stmts.push(crate::source::ast::Stmt::Return {
                            value: crate::source::ast::Expr {
                                span: f.span,
                                kind: crate::source::ast::ExprKind::Var(
                                    crate::source::ast::Ident {
                                        name: crate::source::age_component::local_name(
                                            inst, &f.name,
                                        ),
                                        span: f.span,
                                    },
                                ),
                            },
                            span: f.span,
                        });
                        items.push(crate::source::ast::Item::Fn(crate::source::ast::FnItem {
                            name: crate::source::ast::Ident {
                                name: crate::source::age_component::state_fn_name_at(
                                    node_index, inst, &f.name,
                                ),
                                span: f.span,
                            },
                            type_params: Vec::new(),
                            params: Vec::new(),
                            ret: crate::source::ast::TypeAnn {
                                // 状态函数返回**字段自己的类型**（int/i32/bool）——
                                // 原先硬编码 i32，于是 `flag bool = true` 被共享
                                // 检查判成 `expected i32, found bool`（轮 48 R47-3）。
                                // 不支持的类型已由上面的 `instance_stmts` 具名拒绝。
                                kind: crate::source::ast::TypeRef::Builtin(
                                    crate::source::age_component::field_type(f).expect(
                                        "instance_stmts rejected unsupported model field types",
                                    ),
                                ),
                                span: f.ty_span,
                            },
                            body: crate::source::ast::Block {
                                stmts,
                                span: node.span,
                            },
                            span: node.span,
                            receiver: crate::source::ast::Receiver::Free,
                            owner: None,
                            script: false,
                            proto: false,
                        }));
                    }
                }
                // 文本叶：每个实例一份求值函数（字段引用 → 该实例状态函数调用）。
                // 静态面同样按实例 0 生成（与上面的状态函数一致：有 model 就有状态，
                // "静态"只表示不复制实例，不表示叶可以无绑定地脱离 model）。
                let insts: Vec<usize> = (0..state_instances).collect();
                for inst in insts {
                    for leaf in &node.view.texts {
                        let name = format!(
                            "{}{}_{}",
                            crate::source::age_package::AGE_PACKAGE_HIDDEN_PREFIX,
                            age_components.len(),
                            text_fns.len()
                        );
                        let bound = match crate::source::age_component::rename_expr(
                            &leaf.expr,
                            crate::source::age_component::FieldBinding::StateFnAt {
                                node: node_index,
                                inst,
                            },
                            &node.fields,
                        ) {
                            Ok(b) => b,
                            Err(ds) => return Err(ds),
                        };
                        items.push(crate::source::ast::Item::Fn(
                            crate::source::age_package::hidden_fn(&name, &bound, leaf.expr.span),
                        ));
                        text_fns.push((name, leaf.span));
                    }
                }
                // B-LANGUAGE-I（R59-3）：`design` / 成员槽位的**值表达式**也必须进
                // 唯一 verify。此前只投影形状，于是 `design { theme: nosuch_ident }`、
                // `computed { double => count + "x" }` 之类的非法输入被静默接受；
                // 现在按文本叶同一机制生成检查函数（`let` 绑定 + 定值返回），
                // 名字解析/类型/可见性都由共享检查判定，诊断落回作者 span。
                {
                    let component_index = age_components.len();
                    let mut value_fns = 0usize;
                    let mut check_value = |expr: &crate::source::ast::Expr| {
                        let bound = match crate::source::age_component::rename_expr(
                            expr,
                            crate::source::age_component::FieldBinding::StateFnAt {
                                node: node_index,
                                inst: 0,
                            },
                            &node.fields,
                        ) {
                            Ok(b) => b,
                            Err(ds) => return Err(ds),
                        };
                        let name = format!(
                            "{}val_{}_{}",
                            crate::source::age_package::AGE_PACKAGE_HIDDEN_PREFIX,
                            component_index,
                            value_fns
                        );
                        value_fns += 1;
                        items.push(crate::source::ast::Item::Fn(
                            crate::source::age_package::hidden_value_fn(&name, &bound, expr.span),
                        ));
                        Ok(())
                    };
                    for d in &node.design {
                        if let Some(v) = &d.value {
                            check_value(v)?;
                        }
                    }
                    // R60-4：`if` 形态的条件/分支同样进唯一 verify。
                    for d in &node.design {
                        for part in [&d.then_value, &d.else_value] {
                            if let Some(v) = part {
                                check_value(v)?;
                            }
                        }
                        // R61-3：条件必须能当 bool 用——用 `!cond` 把它放进布尔
                        // 位置（`!` 只接受 bool），诊断落在条件表达式上。
                        if let Some(cond) = &d.cond {
                            let neg = crate::source::ast::Expr {
                                span: cond.span,
                                kind: crate::source::ast::ExprKind::Unary {
                                    op: crate::source::ast::UnaryOp::Not,
                                    operand: Box::new(cond.clone()),
                                },
                            };
                            check_value(&neg)?;
                        }
                    }
                    for m in &node.members {
                        if let Some(v) = &m.value {
                            check_value(v)?;
                        }
                    }
                    // R61-4：view 元素/属性行里的表达式（属性值、内联 lambda 赋值）
                    // 同样进唯一 verify——`class: nosuch_ident` 不能静默通过。
                    // PLAN-981（B-LANGUAGE-V，980/A 真实源）：`onclick: .Msg` 这类
                    // **事件绑定**是事实，不是值表达式——`.Msg` 不是 `self` 字段引用，
                    // 不能送进值 verify（否则整源报 `no local named self`）。
                    fn event_binding_msg<'a>(
                        en: &'a crate::source::age_component::ComponentViewEntry,
                        msgs: &[String],
                    ) -> Option<&'a str> {
                        if !en.is_event {
                            return None;
                        }
                        match &en.value {
                            crate::source::age_component::ComponentEntryValue::Expr(
                                crate::source::ast::Expr {
                                    kind: crate::source::ast::ExprKind::Field { base, name },
                                    ..
                                },
                            ) if matches!(
                                base.kind,
                                crate::source::ast::ExprKind::Var(ref v) if v.name == "self"
                            ) =>
                            {
                                msgs.iter()
                                    .find(|m| **m == name.name)
                                    .map(|_| name.name.as_str())
                            }
                            _ => None,
                        }
                    }

                    // PLAN-980 lambda-if 刀：lambda 体 if 条件表达式入绑定
                    // 验证（镜像 view 条件节点 Cond 的条件入列；赋值 RHS 已
                    // 由深平铺 assigns 覆盖）。
                    fn push_stmt_conds<'a>(
                        stmts: &'a [crate::source::age_component::ComponentStmt],
                        out: &mut Vec<&'a crate::source::ast::Expr>,
                    ) {
                        use crate::source::age_component::ComponentStmt;
                        for st in stmts {
                            if let ComponentStmt::If {
                                cond, then, els, ..
                            } = st
                            {
                                out.push(cond);
                                push_stmt_conds(then, out);
                                if let Some(e) = els {
                                    push_stmt_conds(e, out);
                                }
                            }
                        }
                    }
                    fn collect_item_exprs<'a>(
                        items: &'a [crate::source::age_component::ComponentViewItem],
                        msgs: &[String],
                        out: &mut Vec<&'a crate::source::ast::Expr>,
                    ) {
                        use crate::source::age_component::ComponentEntryValue;
                        use crate::source::age_component::ComponentViewItem;
                        for it in items {
                            match it {
                                ComponentViewItem::Text(_) => {}
                                ComponentViewItem::Element(e) => {
                                    if let Some(l) = &e.label {
                                        out.push(l);
                                    }
                                    collect_item_exprs(&e.items, msgs, out);
                                }
                                // PLAN-1072 刀B：迭代式不入绑定验证（视图数据
                                // 字段无绑定域——惰性），子树照走。
                                ComponentViewItem::For(f) => {
                                    collect_item_exprs(&f.items, msgs, out);
                                }
                                // PLAN-980 view-if：条件表达式入绑定验证 +
                                // 双子树照走。
                                ComponentViewItem::Cond(c) => {
                                    out.push(&c.cond);
                                    collect_item_exprs(&c.then_items, msgs, out);
                                    collect_item_exprs(&c.else_items, msgs, out);
                                }
                                ComponentViewItem::Entry(en) => {
                                    if event_binding_msg(en, msgs).is_some() {
                                        continue;
                                    }
                                    match &en.value {
                                        ComponentEntryValue::Expr(x) => out.push(x),
                                        ComponentEntryValue::Lambda { assigns, stmts, .. } => {
                                            for a in assigns {
                                                out.push(&a.rhs);
                                            }
                                            push_stmt_conds(stmts, out);
                                        }
                                        // PLAN-1072 刀B：条件样式 cond 入绑定
                                        // 验证（两臂为字面量/替换后字面量）。
                                        ComponentEntryValue::If { cond, .. } => out.push(cond),
                                    }
                                }
                            }
                        }
                    }
                    let declared_msgs: Vec<String> =
                        node.messages.iter().map(|m| m.name.clone()).collect();
                    let mut entry_exprs = Vec::new();
                    collect_item_exprs(&node.view.tree, &declared_msgs, &mut entry_exprs);
                    for (_, items) in &node.view.named_trees {
                        collect_item_exprs(items, &declared_msgs, &mut entry_exprs);
                    }
                    for e in entry_exprs {
                        check_value(e)?;
                    }
                    // R59-3 尾项：lifecycle/watch 体内语句——RHS 进唯一 verify，LHS 字段
                    // 必须是本 node 的 model 字段（否则具名拒绝，不静默写不存在的字段）。
                    // R65-3：widget 形 handler 必须指向本 node 声明的消息。
                    // PLAN-981 B-W1（IF-981-PARAMS-B）：带参 handler 与载荷槽的
                    // 绑定在**声明级**核对（形参数 = 载荷槽数、重复形参拒），
                    // 未派发也核对——取代 A 期“派发时具名拒”的 R65-8 守卫。
                    for h in &node.handlers {
                        if h.from_widget && !node.messages.iter().any(|m| m.name == h.msg) {
                            return Err(vec![crate::source::age_h0::text_diag(
                                "source.age.msg-unknown",
                                format!(
                                    "handler `{}` names a message that no `msg` block declares \
                                     in this node",
                                    h.msg
                                ),
                                h.span,
                            )]);
                        }
                    }
                    crate::source::age_component::check_handler_bindings(&node)?;
                    // PLAN-981 B-W1（IF-981-PARAMS-B）：每个 handler 铸一个隐藏
                    // 检查函数（形参 = 真函数参数，载荷类型来自声明拼写）——
                    // **未派发**的 handler 也按原声明语义进唯一 verify。名字在
                    // 包隐藏前缀下取**独立子族**（`hv_`），不与文本/值函数撞名。
                    for hi in 0..node.handlers.len() {
                        match crate::source::age_component::handler_check_fn(
                            &node,
                            node_index,
                            hi,
                            "__age_p_hidden_hv_",
                        ) {
                            Some(Ok(item)) => handler_check_items.push(item),
                            Some(Err(ds)) => return Err(ds),
                            None => {}
                        }
                    }
                    // R61-2：`watch` 的每个源必须是本 node 的 model 字段。
                    for m in &node.members {
                        // R62-2：诊断落在**源 token** 上；修饰形只检查头字段。
                        for (source, source_span) in m.sources.iter().zip(m.source_spans.iter()) {
                            let head = source.split('.').next().unwrap_or(source);
                            if !node.fields.iter().any(|f| f.name == head) {
                                return Err(vec![crate::source::age_h0::text_diag(
                                    "source.age.member-field",
                                    format!(
                                        "`{source}` is not a `model` field of this node; \
                                         the `{}` member watches it",
                                        m.role
                                    ),
                                    *source_span,
                                )]);
                            }
                        }
                    }
                    for m in &node.members {
                        for (field, rhs) in &m.stmts {
                            if !node.fields.iter().any(|f| f.name == *field) {
                                return Err(vec![crate::source::age_h0::text_diag(
                                    "source.age.member-field",
                                    format!(
                                        "`{field}` is not a `model` field of this node; \
                                         the `{}` member assigns it",
                                        m.role
                                    ),
                                    rhs.span,
                                )]);
                            }
                            check_value(rhs)?;
                        }
                    }
                    items.extend(handler_check_items);
                } // PLAN-981（IF-981-ORIGINAL）：场景属性值降级成普通求值函数，
                  // 与文本叶同一机制——共享 verify 检查它，领域行引用其 def id。
                  // 不另建求值器，也不把手写数值当常量。
                for (si, scene) in node.view.scene_components.iter().enumerate() {
                    // PLAN-981：属性名在**一行内必须唯一**。此前重复键被静默接受，
                    // 后值覆盖前值、零诊断——写两遍 `kind:` 的作者得不到任何反馈。
                    let mut seen_keys: std::collections::BTreeMap<&str, Span> =
                        std::collections::BTreeMap::new();
                    for attr in &scene.attrs {
                        if let Some(first) = seen_keys.insert(attr.key.as_str(), attr.span) {
                            let msg = format!(
                                "attribute `{}` is written twice on `{}` (first at byte {}, \
                                 again at byte {})",
                                attr.key, scene.name, first.start, attr.span.start
                            );
                            return Err(vec![crate::source::age_h0::text_diag(
                                "source.age.scene-attr-duplicate",
                                msg,
                                attr.span,
                            )]);
                        }
                    }
                    for (ai, attr) in scene.attrs.iter().enumerate() {
                        let name = format!(
                            "{}scene_{}_{}_{}",
                            crate::source::age_package::AGE_PACKAGE_HIDDEN_PREFIX,
                            age_components.len(),
                            si,
                            ai
                        );
                        // PLAN-981（IF-981-ORIGINAL）：按**属性类别**校验，而不是
                        // 把所有值塞进同一个值函数：
                        //  - `kind:` 是冻结几何符号（circle/quad）→ 查符号表；
                        //  - `material:`/`tint:` 是具名资源引用 → 查导入/声明事实；
                        //  - 其余是普通 Auto 值 → 降级成函数交共享 verify。
                        // 该场景行是不是 2D 叶（`shape2d`）——决定 `kind:` 是否适用。
                        let is_2d_leaf = scene.kind == "shape2d";
                        // 3D 叶种类必须属于冻结集合（`cylinder3d`/`model3d`/`group3d`）；
                        // 2D 场景只允许 `shape2d`（以及 `Disc` 这类组件实例名）。
                        if scene.kind != "shape2d"
                            && !scene
                                .kind
                                .chars()
                                .next()
                                .is_some_and(|c| c.is_ascii_uppercase())
                            && !crate::source::age_resources::is_3d_element(&scene.kind)
                        {
                            return Err(vec![crate::source::age_h0::text_diag(
                                "source.age.scene-element",
                                format!(
                                    "`{}` is not an AGE scene element (2D: `shape2d` or a \
                                     component name; 3D: cylinder3d/model3d/group3d)",
                                    scene.kind
                                ),
                                scene.span,
                            )]);
                        }
                        // 属性必须**适用于**该元素（`yaw:` 不属 shape2d 等）。
                        if !crate::source::age_resources::attr_applies_to(
                            scene.kind.as_str(),
                            attr.key.as_str(),
                        ) {
                            return Err(vec![crate::source::age_h0::text_diag(
                                "source.age.scene-attr",
                                format!("`{}` does not apply to `{}`", attr.key, scene.kind),
                                attr.span,
                            )]);
                        }
                        use crate::source::age_resources::AttrValueKind;
                        // 组件实例的属性名就是该 node 声明的**参数名**，作者自定，
                        // 无法在 `attr_value_kind` 的扁平表里穷举。故实例上的未知键
                        // 按普通值处理；`shape2d` / 3D 元素仍只接受已知属性。
                        let is_component_instance =
                            !crate::source::age_resources::is_3d_element(scene.kind.as_str())
                                && scene.kind != "shape2d";
                        let kind = match crate::source::age_resources::attr_value_kind(&attr.key) {
                            Some(k) => k,
                            None if is_component_instance => {
                                crate::source::age_resources::AttrValueKind::Value
                            }
                            None => {
                                return Err(vec![crate::source::age_h0::text_diag(
                                    "source.age.scene-attr",
                                    format!(
                                        "`{}` is not an AGE 2D scene attribute (expected \
                                         kind/position/size/material/visible, or x/y/diameter/tint)",
                                        attr.key
                                    ),
                                    attr.span,
                                )]);
                            }
                        };
                        match kind {
                            AttrValueKind::GeometryKind => {
                                // `kind:` 只属于 **2D** 叶（`shape2d`）——规范里
                                // `view-2d` 的叶是 `shape2d { kind: circle|quad }`，
                                // 而 `view-3d` 的叶种类是元素名本身（`cylinder3d`
                                // 等），没有 `kind` 属性。两套符号表不可互套。
                                if !is_2d_leaf {
                                    return Err(vec![crate::source::age_h0::text_diag(
                                        "source.age.scene-attr",
                                        format!(
                                            "`kind` is a `shape2d` (view-2d) attribute; \
                                             `{elem}` declares its kind by its element name",
                                            elem = scene.kind
                                        ),
                                        attr.span,
                                    )]);
                                }
                                let spelled = match &attr.value.kind {
                                    crate::source::ast::ExprKind::Var(id) => id.name.clone(),
                                    _ => String::new(),
                                };
                                if !crate::source::age_resources::is_geometry_kind(&spelled) {
                                    return Err(vec![crate::source::age_h0::text_diag(
                                        "source.age.scene-kind",
                                        format!(
                                            "`kind` must be one of the frozen AGE geometry \
                                             symbols (circle/quad), got `{spelled}`"
                                        ),
                                        attr.value.span,
                                    )]);
                                }
                            }
                            AttrValueKind::Layer => {
                                // PLAN-981（G1，2026-11-18 裁定）：`layer: render-only`
                                // ——冻结符号行（专用文法臂已保证值是单 token；
                                // 组件实例/3D 上的 `layer:` 也落本臂并按元素拒）。
                                // 语义：该行可见但**不进碰撞候选集**（typed 事实进
                                // `AgeSceneRow.layer`；`cpu` 面同步 Symbol）。
                                if !is_2d_leaf {
                                    return Err(vec![crate::source::age_h0::text_diag(
                                        "source.age.scene-attr",
                                        format!(
                                            "`layer` is a `shape2d` (view-2d) attribute; it does \
                                             not apply to `{}`",
                                            scene.kind
                                        ),
                                        attr.span,
                                    )]);
                                }
                                let spelled = match &attr.value.kind {
                                    crate::source::ast::ExprKind::Var(id) => id.name.clone(),
                                    _ => String::new(),
                                };
                                if spelled
                                    != crate::source::age_resources::AGE_ROW_LAYER_RENDER_ONLY
                                {
                                    return Err(vec![crate::source::age_h0::text_diag(
                                        "source.age.scene-layer",
                                        format!(
                                            "`layer` accepts one frozen symbol `render-only` \
                                             (visible, not a collision candidate), got `{spelled}`"
                                        ),
                                        attr.value.span,
                                    )]);
                                }
                            }
                            AttrValueKind::ResourceRef => {
                                // 具名资源引用：值必须是**引用形状**（不是字面量/任意
                                // 表达式）。名字是否存在、是否可见——本层**不能**担保
                                // （实测：不存在的 `tint: no_such_resource` 曾静默通过）。
                                // 故把引用**降级成求值函数**，由共享 resolver/binder 按
                                // 960 的导入/可见性规则真正解析符号；这里只拒绝非引用形状。
                                let literal = matches!(
                                    &attr.value.kind,
                                    crate::source::ast::ExprKind::IntLit(_)
                                        | crate::source::ast::ExprKind::BoolLit(_)
                                        | crate::source::ast::ExprKind::StrLit(_)
                                        | crate::source::ast::ExprKind::TupleLit(_)
                                        | crate::source::ast::ExprKind::Unary { .. }
                                        | crate::source::ast::ExprKind::Binary { .. }
                                );
                                if literal {
                                    return Err(vec![crate::source::age_h0::text_diag(
                                        "source.age.scene-resource",
                                        format!(
                                            "`{}` must name a resource (`resources.<symbol>`), \
                                             not a literal or computed value",
                                            attr.key
                                        ),
                                        attr.value.span,
                                    )]);
                                }
                                // 引用的**存在性/可见性/端口类型**由前门用 binder 事实
                                // 裁定（`age_package::check_scene_resource_refs`）；这里
                                // 不伪造值类型，也不降级成函数（目标声明决定其类型）。
                            }
                            AttrValueKind::Value => {
                                match crate::source::age_package::hidden_fns_scene_value(
                                    &name,
                                    &attr.value,
                                    attr.span,
                                    &node.params,
                                ) {
                                    Some(fs) => {
                                        // PLAN-981 B-W1：按发射序登记 (row, attr,
                                        // 分量, 函数名, 形参数)——元组属性逐分量。
                                        let is_tuple = matches!(
                                            &attr.value.kind,
                                            crate::source::ast::ExprKind::TupleLit(_)
                                        );
                                        for (ci, f) in fs.iter().enumerate() {
                                            scene_fns.push(
                                                crate::source::age_package::AgeSceneValueFn {
                                                    row: scene.name.clone(),
                                                    attr: attr.key.clone(),
                                                    component: is_tuple.then_some(ci),
                                                    fn_name: f.name.name.clone(),
                                                    params: f.params.len(),
                                                },
                                            );
                                        }
                                        items.extend(
                                            fs.into_iter().map(crate::source::ast::Item::Fn),
                                        )
                                    }
                                    None => {
                                        return Err(vec![crate::source::age_h0::text_diag(
                                            "source.age.scene-value",
                                            format!(
                                                "scene attribute `{}` has a value shape this \
                                                 slice cannot type yet",
                                                attr.key
                                            ),
                                            attr.span,
                                        )]);
                                    }
                                }
                            }
                        }
                    }
                }
                age_components.push((node, text_fns, scene_fns));
            }
        }
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
                proj_profile,
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
            // PLAN-980 P3（981 回签 (a)(c)(d)）：`style` 声明 = str const/fn 糖。
            // 裸 `style-item` 与其 `pub` 包装都归这里；默认值/插值/跨名引用具名拒。
            let style_node: Option<(u32, bool)> = if Some(kind) == style_item {
                Some((node, false))
            } else if Some(kind) == public_item {
                public_body(tree, node)
                    .and_then(|b| (Some(tree.kind(b)) == style_item).then_some((b, true)))
            } else {
                None
            };
            if let Some((st, is_pub)) = style_node {
                saw_decl = true;
                let name_leaf = field(tree, st, "name").expect("style name");
                let name = text(tree, name_leaf);
                let name_span = node_span(tree, name_leaf);
                let item_span = node_span(tree, node);
                let value_node = field(tree, st, "value").expect("style value");
                let value_span = node_span(tree, value_node);
                let value = project::project_expr_node(
                    &cached.program,
                    &cached.kinds,
                    tree,
                    &snap,
                    proj_profile,
                    value_node,
                );
                let mut params: Vec<crate::source::ast::Param> = Vec::new();
                if let Some(pnode) = field(tree, st, "params") {
                    let mut found: Vec<u32> = Vec::new();
                    let mut stack = vec![pnode];
                    while let Some(n) = stack.pop() {
                        if Some(tree.kind(n)) == style_param {
                            found.push(n);
                        }
                        for c in tree.children(n) {
                            stack.push(c);
                        }
                    }
                    found.sort_by_key(|&n| tree.abs_range(n).0);
                    for sp in found {
                        let pname = text(tree, field(tree, sp, "name").expect("style param name"));
                        let pspan = node_span(tree, field(tree, sp, "name").expect("style param"));
                        let default_node = tree
                            .children(sp)
                            .into_iter()
                            .find(|&c| Some(tree.kind(c)) == style_param_default)
                            .and_then(|d| field(tree, d, "default"));
                        if let Some(def) = default_node {
                            let dspan = node_span(tree, def);
                            return Err(err(
                                "source.age.style-defaults",
                                format!(
                                    "style recipe `{name}` declares a default value for parameter                                      `{pname}`; default parameters are a named pending surface in                                      this slice"
                                ),
                                dspan,
                            ));
                        }
                        let ty_word = tree
                            .children(sp)
                            .into_iter()
                            .find(|&c| Some(tree.kind(c)) == style_param_ty_ann)
                            .and_then(|t| field(tree, t, "ty"))
                            .and_then(|t| field(tree, t, "word"))
                            .map(|w| text(tree, w));
                        let ty = match ty_word.as_deref() {
                            None | Some("str") => crate::source::ast::TypeWord::Str,
                            Some(other) => {
                                return Err(err(
                                    "source.age.style-param",
                                    format!(
                                        "style parameter `{pname}` has type `{other}`; style                                          parameters are `str` in this slice"
                                    ),
                                    pspan,
                                ))
                            }
                        };
                        params.push(crate::source::ast::Param {
                            name: crate::source::ast::Ident {
                                name: pname,
                                span: pspan,
                            },
                            ty: crate::source::ast::TypeAnn {
                                kind: crate::source::ast::TypeRef::Builtin(ty),
                                span: pspan,
                            },
                            mode: crate::source::ast::ParamMode::Default,
                            default: None,
                            span: pspan,
                        });
                    }
                }
                // 零参 ident 复制（引用另一条本地 style 字面量）——延后解析，允许前向。
                if params.is_empty() {
                    if let crate::source::ast::ExprKind::Var(target) = &value.kind {
                        style_copies.push((name.clone(), target.name.clone(), value_span, is_pub));
                        continue;
                    }
                    if let crate::source::ast::ExprKind::Call { callee, .. } = &value.kind {
                        return Err(err(
                            "source.age.style-call",
                            format!(
                                "style `{name} = {}(…)` is a named pending surface in this slice \
                                 (parameterised style calls are not folded)",
                                callee.name
                            ),
                            value_span,
                        ));
                    }
                    if let crate::source::ast::ExprKind::StrLit(lit) = &value.kind {
                        style_lits.insert(name.clone(), lit.clone());
                    }
                    items.push(crate::source::ast::Item::Const(
                        crate::source::ast::ConstItem {
                            name: crate::source::ast::Ident {
                                name: name.clone(),
                                span: name_span,
                            },
                            ty: crate::source::ast::TypeAnn {
                                kind: crate::source::ast::TypeRef::Builtin(
                                    crate::source::ast::TypeWord::StrOwned,
                                ),
                                span: name_span,
                            },
                            value,
                            span: item_span,
                        },
                    ));
                } else {
                    if let crate::source::ast::ExprKind::StrLit(lit) = &value.kind {
                        if lit.contains('{') {
                            return Err(err(
                                "source.age.style-interp",
                                format!(
                                    "style recipe `{name}` interpolates parameters (`{{…}}`);                                      parameter interpolation is a named pending surface in this slice"
                                ),
                                value_span,
                            ));
                        }
                    }
                    items.push(crate::source::ast::Item::Fn(crate::source::ast::FnItem {
                        name: crate::source::ast::Ident {
                            name: name.clone(),
                            span: name_span,
                        },
                        type_params: Vec::new(),
                        params,
                        ret: crate::source::ast::TypeAnn {
                            kind: crate::source::ast::TypeRef::Builtin(
                                crate::source::ast::TypeWord::StrOwned,
                            ),
                            span: name_span,
                        },
                        body: crate::source::ast::Block {
                            stmts: vec![crate::source::ast::Stmt::Return {
                                value,
                                span: value_span,
                            }],
                            span: value_span,
                        },
                        span: item_span,
                        receiver: crate::source::ast::Receiver::Free,
                        owner: None,
                        script: false,
                        proto: false,
                    }));
                }
                declarations.insert(name.clone(), item_span);
                if is_pub {
                    exports.insert(name);
                }
                continue;
            }
            // PLAN-1043（T-03）/PLAN-966（T-02）：项位编译期选择（`#in` 与
            // 独立 `#if/#is/#for`）——结构保持进 ct_items；含任何 use 的项属
            // 导入位（须在声明区前），只含声明的项视作声明项。
            if Some(kind) == ct_item {
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
                        "compile-time items require the packages-ct face".into(),
                        span,
                    ));
                };
                // ct-item 是分派 wrapper：实体是 in-item/ct-if-item/ct-is-item/
                // ct-for-item 中命中的那个规则节点。
                let inner = tree
                    .children(node)
                    .into_iter()
                    .find(|&c| {
                        matches!(
                            face.rule_of(c),
                            "in-item" | "ct-if-item" | "ct-is-item" | "ct-for-item"
                        )
                    })
                    .unwrap_or(node);
                let kind = face.ct_item(inner).map_err(|d| vec![d])?;
                ct_items.push(crate::source::packages::PackageCtItem {
                    kind,
                    span,
                    decl_slot: items.len(),
                    import_slot: imports.len(),
                });
                continue;
            }
            // use 项（含 public-item 内的 use 尾）——路径/别名字段读取。
            // PLAN-981（IF-981-ORIGINAL）：`use mod: a, b` → 每个符号一条
            // 与 `use mod.a` **同义**的绑定（同一命名空间、同一可见性规则）。
            if Some(kind) == age_use_sym {
                if saw_decl {
                    return Err(err(
                        "source.module-import-order",
                        "imports must precede declarations".into(),
                        node_span(tree, node),
                    ));
                }
                let module = text(tree, field(tree, node, "module").expect("use module"));
                let mut syms: Vec<(String, u64)> = Vec::new();
                let head = field(tree, node, "head").expect("use sym head");
                let (ha, _) = tree.abs_range(head);
                syms.push((text(tree, head), ha));
                for c in tree.children(node) {
                    if Some(tree.kind(c)) != age_use_sym_tail {
                        continue;
                    }
                    let leaf = field(tree, c, "sym").expect("use sym");
                    let (a, _) = tree.abs_range(leaf);
                    syms.push((text(tree, leaf), a));
                }
                for (sym, at) in syms {
                    let span = Span {
                        start: at as usize,
                        end: at as usize + sym.len(),
                    };
                    if imports
                        .iter()
                        .any(|i: &PackageImport| i.bound_name() == sym)
                    {
                        return Err(err(
                            "source.module-duplicate-import",
                            format!("duplicate imported name `{sym}`"),
                            span,
                        ));
                    }
                    imports.push(PackageImport {
                        path: vec![module.clone(), sym],
                        alias: None,
                        reexport: false,
                        span,
                    });
                }
                import_spans.push({
                    let (a, b) = tree.abs_range(node);
                    (a as usize, b as usize)
                });
            }
            // PLAN-980 ④-A：点分成员导入 `use <module>.<name>: a, b` —— 逐符号
            // 落成 3 段路径 [module, name, sym]（与包面 `use pkg.mod: a, b` 同义）。
            // 此前包面**没有任何投影分支**（解析后零事实）。
            if Some(kind) == age_use_module {
                if saw_decl {
                    return Err(err(
                        "source.module-import-order",
                        "imports must precede declarations".into(),
                        node_span(tree, node),
                    ));
                }
                let module = text(tree, field(tree, node, "module").expect("use module"));
                let name = text(tree, field(tree, node, "name").expect("use member"));
                let mut syms: Vec<(String, u64)> = Vec::new();
                if let Some(head) = field(tree, node, "import") {
                    let (a, _) = tree.abs_range(head);
                    syms.push((text(tree, head), a));
                }
                for c in tree.children(node) {
                    if Some(tree.kind(c)) != age_use_import_tail {
                        continue;
                    }
                    if let Some(leaf) = field(tree, c, "import") {
                        let (a, _) = tree.abs_range(leaf);
                        syms.push((text(tree, leaf), a));
                    }
                }
                for (sym, at) in syms {
                    let span = Span {
                        start: at as usize,
                        end: at as usize + sym.len(),
                    };
                    if imports
                        .iter()
                        .any(|i: &PackageImport| i.bound_name() == sym)
                    {
                        return Err(err(
                            "source.module-duplicate-import",
                            format!("duplicate imported name `{sym}`"),
                            span,
                        ));
                    }
                    imports.push(PackageImport {
                        path: vec![module.clone(), name.clone(), sym],
                        alias: None,
                        reexport: false,
                        span,
                    });
                }
                import_spans.push({
                    let (a, b) = tree.abs_range(node);
                    (a as usize, b as usize)
                });
            }
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
                // PLAN-1064 T-MODULE-I r2: a ONE-segment path is the
                // module-scope import (`use demo` — declares the module
                // dependency edge; members stay qualified `demo.item`).
                // Re-exporting a whole module is a named rejection.
                if path.is_empty() {
                    return Err(err(
                        "source.module-path",
                        "module paths need at least `module` or `module.item`".into(),
                        span,
                    ));
                }
                if path.len() == 1 && reexport {
                    return Err(err(
                        "source.module-path",
                        "a whole module cannot be re-exported; re-export its items".into(),
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
                import_spans.push((span.start, span.end));
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
            } else if Some(kind) == age_pub_node {
                // PLAN-981（IF-981-REF）：`pub node Foo` 进本模块**公共面**，别的
                // 模块才能按 node 名引用其实例。可见性语义沿用 960 的 exports。
                saw_decl = true;
                let name = text(tree, field(tree, node, "name").expect("pub node name"));
                exports.insert(name.clone());
                declarations.insert(name, node_span(tree, node));
            } else if Some(kind) == age_let_item {
                // PLAN-981（IF-981-ORIGINAL）：`let name = expr` 是**不可变**模块
                // 绑定，语义等同 `const`（同初始化序、同跨模块 get）。资源模块靠
                // 这些名字对外供给，故进公共面；可见性仍由共享 binder 裁定。
                saw_decl = true;
                let name = text(tree, field(tree, node, "name").expect("let name"));
                exports.insert(name.clone());
                const_exports.insert(name.clone());
                declarations.insert(name, node_span(tree, node));
            } else if Some(kind) == const_item
                || Some(kind) == var_item
                || Some(kind) == auto_var_item
            {
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
    // PLAN-980 P3：零参 ident 复制（本地字面量）延后解析；跨模块/非字面量具名拒。
    for (name, target, span, is_pub) in style_copies {
        let Some(lit) = style_lits.get(&target).cloned() else {
            return Err(err(
                "source.age.style-call",
                format!(
                    "style `{name}` copies `{target}`, which is not a literal style declared in                      this module; imported/non-literal style references are a named pending                      surface in this slice"
                ),
                span,
            ));
        };
        items.push(crate::source::ast::Item::Const(
            crate::source::ast::ConstItem {
                name: crate::source::ast::Ident {
                    name: name.clone(),
                    span,
                },
                ty: crate::source::ast::TypeAnn {
                    kind: crate::source::ast::TypeRef::Builtin(
                        crate::source::ast::TypeWord::StrOwned,
                    ),
                    span,
                },
                value: crate::source::ast::Expr {
                    kind: crate::source::ast::ExprKind::StrLit(lit),
                    span,
                },
                span,
            },
        ));
        declarations.insert(name.clone(), span);
        if is_pub {
            exports.insert(name);
        }
    }
    // PLAN-980 P3：用法投影期替换（`style: NAME` → 字面量；981 回签 (c)3）。
    // 导入的 style 引用与带参调用一律 `style-call` 具名拒（不折叠、不留 global 读）。
    let import_names: std::collections::BTreeSet<String> =
        imports.iter().map(|i| i.bound_name().to_string()).collect();
    for (node, _texts, _scene) in &mut age_components {
        if let Err(d) =
            replace_style_in_items(&mut node.view.tree, &style_lits, &import_names, false)
        {
            return Err(vec![d]);
        }
    }
    // PLAN-980 ④-A：引用驱动（惰性）导入的**引用集**——本模块源里出现在导入项
    // 之外的标识符拼写（注释/字符串/导入项自身不计）。AGE 面用它跳过未被引用的
    // 导入（例如只声明不用的 `use themes.themes: Auto`），其它面保持 eager 语义。
    let mut referenced: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for tree in &trees {
        tree.for_each_leaf(|s, e, _k, is_error| {
            if is_error {
                return;
            }
            let (a, b) = (s as usize, e as usize);
            if import_spans.iter().any(|(x, y)| a >= *x && b <= *y) {
                return;
            }
            if let Ok(spelled) = snap.read_str(s, e) {
                let ident = !spelled.is_empty()
                    && spelled
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && spelled
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
                if ident {
                    referenced.insert(spelled);
                }
            }
        });
    }
    // PLAN-981（W1 跨模块实例身份，2026-11-18 第六批）：场景 `Send`/`SendArgs`
    // 的**声明级**核对（arity/载荷域）不在本函数做——本函数是**逐模块**的，
    // 而 `ForNode{node}` 的下标是**包级组件声明序**（在逐模块用本模块 nodes
    // 索引会误拒跨模块定向）。该核对移到绑定器的包级聚合点
    // （`bind_packages_face_with`：`age_out` 收齐后一次核）。
    // 逐 node 的**执行**脱糖本来就用包级 `node_index`（`instance_stmts` 按
    // `op.node_scope()` 过滤），故这里不再需要局部预检。
    Ok(PackageSyntax {
        program: crate::source::ast::Program { items },
        exports,
        type_exports,
        const_exports,
        imports,
        referenced,
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
        ct_items,
        age_components,
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

    /// 一个节点命中的规则名（`ct-item` 分派 wrapper 用；非规则节点返回空）。
    fn rule_of(&self, n: u32) -> &str {
        let k = self.tree.kind(n);
        let idx = k.wrapping_sub(ace_syntax_runtime::program::RULE_KIND_BASE) as usize;
        self.cached
            .program
            .rules
            .get(idx)
            .map(|r| r.name.as_str())
            .unwrap_or("")
    }

    /// PLAN-966（T-02）：一项位编译期选择（ct-item 的分派实体）→ 形态。
    fn ct_item(&mut self, node: u32) -> Result<crate::source::packages::CtItemKind, Diagnostic> {
        use crate::source::packages::CtItemKind;
        let kind = self.tree.kind(node);
        if Some(kind) == self.rule("in-item") {
            let (namespace, tail) = self.in_item(node)?;
            return Ok(CtItemKind::In { namespace, tail });
        }
        if Some(kind) == self.rule("ct-if-item") {
            let cond = self.ct_expr_field(node, "cond")?;
            let then = self.block(self.field(node, "then").expect("then block"))?;
            let els = self
                .nontrivia(node)
                .into_iter()
                .find(|&c| Some(self.tree.kind(c)) == self.rule("in-else"))
                .and_then(|e| self.field(e, "body"))
                .map(|b| self.block(b))
                .transpose()?;
            return Ok(CtItemKind::If { cond, then, els });
        }
        if Some(kind) == self.rule("ct-is-item") {
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
            return Ok(CtItemKind::Is { scrutinee, arms });
        }
        if Some(kind) == self.rule("ct-for-item") {
            let var = self.text(self.field(node, "var").expect("for var"));
            let iter = self.field(node, "iter").expect("for iter");
            let (start, end) = self.range_bounds(iter)?;
            let body = self.block(self.field(node, "body").expect("for body"))?;
            return Ok(CtItemKind::For {
                var,
                start,
                end,
                body,
            });
        }
        Err(self.diag(
            "source.cte.in-stmt",
            "unsupported compile-time item form".into(),
            self.span_of(node),
        ))
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
        // primary：in-ct-unary 包 int-literal/true/false/ns-ref。取首个非
        // trivia 孩子（bool 字面量是 token 叶，不能按"有孩子"过滤）。
        let inner = if Some(kind) == self.rule("in-ct-unary") {
            self.nontrivia(node).into_iter().next().ok_or_else(|| {
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
        // PLAN-979：AGE 领域面槽位。
        SourceProfile::AgeH0 => 11,
        // PLAN-979 T-03：组件面槽位。
        SourceProfile::AgeComponent => 12,
        // PLAN-981：AGE 包面槽位（包模块图 ∪ AGE 组件）。
        SourceProfile::AgePackage => 14,
        // PLAN-968：调度器面槽位。
        SourceProfile::Scheduler => 13,
        // PLAN-982（B-CONFIG-I）：配置文档面槽位（既有 15 个 profile 的裁剪
        // 字节与 GrammarId 不受本槽影响）。
        SourceProfile::ConfigDocument => 15,
        // PLAN-1078：Auto 面槽位（aggregates 面身份）。
        SourceProfile::Auto => 16,
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
    // PLAN-979 F2 / PLAN-981：AGE 领域 profile（H0、组件面、**包面**）只经各自的
    // 领域前门消费；通用入口（含片段入口，它们没有槽位拼写的事实源）一律具名拒绝。
    //
    // 复审 M5：`domain_profile_refusal` 早已包含 `AgePackage`，但**这里的 matches!
    // 没同步**——于是通用入口对 `age-package` 返回 `Ok`（零项目、零领域行），
    // 而 `age_package.rs` 里那个专门的 `age_package_refusal` **零调用**、是死代码。
    // 现在两者一致：入口拒绝，且拒绝理由指向正确的前门。
    if matches!(
        profile,
        SourceProfile::AgeH0 | SourceProfile::AgeComponent | SourceProfile::AgePackage
    ) {
        if let Some(d) = crate::source::age_h0::domain_profile_refusal(profile, src.len()) {
            return FrontOutcome::Diag(vec![d]);
        }
    }
    // PLAN-982（B-CONFIG-I）：配置文档行只经其文档前门消费（通用投影无法表示
    // 文档行——具名拒绝，而不是静默丢弃）。
    if let Some(d) = crate::source::config_document::config_profile_refusal(profile, src.len()) {
        return FrontOutcome::Diag(vec![d]);
    }
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
        // PLAN-981：整模块入口不属于 AGE 作者面。
        false,
    ) {
        Ok(program) => FrontOutcome::Ok(FrontOk {
            program,
            grammar_id: format!("{}", cached.program.identity.grammar_id.0),
            caps_version: FRONT_CAPS_VERSION,
            snapshot_len: src.len(),
            cst: tree,
            age_nodes: Vec::new(),
            component_nodes: Vec::new(),
        }),
        Err(diags) => FrontOutcome::Diag(diags),
    }
}

/// PLAN-980 P3 / PLAN-1072：对单个 view 条目表达式执行本地 style 字面量替换，
/// 并统一对导入引用、未声明名引用及带参调用形执行 `source.age.style-call` fail-closed 检查。
fn replace_style_in_entry_expr(
    e: &mut crate::source::ast::Expr,
    entry_key: &str,
    lits: &std::collections::BTreeMap<String, String>,
    imports: &std::collections::BTreeSet<String>,
    reject_unknown: bool,
) -> Result<(), crate::atom_text::Diagnostic> {
    match &e.kind {
        crate::source::ast::ExprKind::Var(id) => {
            if let Some(lit) = lits.get(&id.name) {
                e.kind = crate::source::ast::ExprKind::StrLit(lit.clone());
            } else if imports.contains(&id.name) {
                return Err(crate::atom_text::Diagnostic::new(
                    crate::atom_text::Stage::Text,
                    "source.age.style-call",
                    format!(
                        "style reference `{}` resolves to an imported style; \
                         imported style references are a named pending surface in \
                         this slice",
                        id.name
                    ),
                    e.span,
                ));
            } else if reject_unknown && entry_key == "style" {
                // PLAN-1072 刀A：组件面具名拒（fail-closed——
                // 条目值惰性，但不留静默未解析 ident；限定
                // `style` 键——其它键的裸 ident 值不属 style 面）。
                return Err(crate::atom_text::Diagnostic::new(
                    crate::atom_text::Stage::Text,
                    "source.age.style-call",
                    format!(
                        "style reference `{}` does not resolve to a style declared \
                         in this module; undeclared style references are a named \
                         pending surface in this slice",
                        id.name
                    ),
                    e.span,
                ));
            }
        }
        crate::source::ast::ExprKind::Call { callee, .. } => {
            if lits.contains_key(&callee.name)
                || imports.contains(&callee.name)
                || (reject_unknown && entry_key == "style")
            {
                return Err(crate::atom_text::Diagnostic::new(
                    crate::atom_text::Stage::Text,
                    "source.age.style-call",
                    format!(
                        "style call `{}(…)` is a named pending surface in this \
                         slice (parameterised style calls are not folded)",
                        callee.name
                    ),
                    e.span,
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

/// PLAN-980 P3：把 view 条目里的本地 style 名替换为其字面量（投影期解析；
/// 981 回签 (c)3——不引入 module-global 读，下游车道只见到普通字符串字面量）。
/// 导入的 style 引用与带参调用 = `style-call` 具名拒（本刀不折叠）。
fn replace_style_in_items(
    items: &mut Vec<crate::source::age_component::ComponentViewItem>,
    lits: &std::collections::BTreeMap<String, String>,
    imports: &std::collections::BTreeSet<String>,
    reject_unknown: bool,
) -> Result<(), crate::atom_text::Diagnostic> {
    use crate::source::age_component::{ComponentEntryValue, ComponentViewItem};
    for it in items.iter_mut() {
        match it {
            // PLAN-1072 刀B：style 字面量替换进循环子树。
            ComponentViewItem::For(f) => {
                replace_style_in_items(&mut f.items, lits, imports, reject_unknown)?
            }
            // PLAN-980 view-if：style 字面量替换进条件双子树。
            ComponentViewItem::Cond(c) => {
                replace_style_in_items(&mut c.then_items, lits, imports, reject_unknown)?;
                replace_style_in_items(&mut c.else_items, lits, imports, reject_unknown)?;
            }
            ComponentViewItem::Element(el) => {
                replace_style_in_items(&mut el.items, lits, imports, reject_unknown)?
            }
            ComponentViewItem::Entry(en) => match &mut en.value {
                ComponentEntryValue::If {
                    then_value,
                    else_value,
                    ..
                } => {
                    // PLAN-1072 刀B / T-04：条件样式两臂与 Expr 值共享同一 fail-closed 检查。
                    replace_style_in_entry_expr(
                        then_value,
                        &en.key,
                        lits,
                        imports,
                        reject_unknown,
                    )?;
                    replace_style_in_entry_expr(
                        else_value,
                        &en.key,
                        lits,
                        imports,
                        reject_unknown,
                    )?;
                }
                ComponentEntryValue::Expr(e) => {
                    replace_style_in_entry_expr(e, &en.key, lits, imports, reject_unknown)?;
                }
                ComponentEntryValue::Lambda { .. } => {}
            },
            ComponentViewItem::Text(_) => {}
        }
    }
    Ok(())
}

/// PLAN-1072 刀A：组件面 `style` 声明 = **惰性视图数据**（981 r18 会签 (c)3
/// 投影期字面量替换路径——非 const 面：不产 Item::Const/Fn、不进 resolve/
/// ModuleGlobalsPlan，981 会签三锁条件不触发）。收集本模块 style 字面量表
/// 并替换 view 内 `style: NAME` ident 引用；重名沿包面 (c)1 同码，导入引用/
/// 带参调用/不可折叠值沿包面 style-call 族具名拒；未声明名引用 fail-closed
/// 具名拒（条目值惰性，不留静默未解析 ident）。参数化/pub 声明是包面专属
///（组件面 style-params 步骤 profile 门控缺席=文法层边界）。
fn apply_component_face_styles(
    cached: &CachedGrammar,
    tree: &CstTree,
    snap: &Snapshot,
    nodes: &mut [crate::source::age_component::ComponentNode],
) -> Result<(), Vec<Diagnostic>> {
    let rule = |name: &str| {
        cached
            .program
            .rules
            .iter()
            .position(|r| r.name == name)
            .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
    };
    let style_item = match rule("style-item") {
        Some(k) => k,
        None => return Ok(()),
    };
    let node_span = |tree: &CstTree, n| {
        let (start, end) = tree.abs_range(n);
        crate::atom_text::Span {
            start: start as usize,
            end: end as usize,
        }
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
    // 源序收集 style 声明节点 + 组件面导入名（age-use-module-item 的成员）。
    let mut style_nodes: Vec<(u64, u32)> = Vec::new();
    let mut imports: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let age_use_module = rule("age-use-module-item");
    let age_use_import_tail = rule("age-use-import-tail");
    let mut stack = vec![tree.root];
    while let Some(n) = stack.pop() {
        if tree.kind(n) == style_item {
            let (a, _) = tree.abs_range(n);
            style_nodes.push((a, n));
        }
        if Some(tree.kind(n)) == age_use_module {
            if let Some(im) = field(tree, n, "import") {
                imports.insert(text(tree, im));
            }
            for c in tree.children(n) {
                if Some(tree.kind(c)) == age_use_import_tail {
                    if let Some(im) = field(tree, c, "import") {
                        imports.insert(text(tree, im));
                    }
                }
            }
        }
        for c in tree.children(n).into_iter().rev() {
            stack.push(c);
        }
    }
    if style_nodes.is_empty() {
        // 零 style 声明也要走替换/拒绝走查——`style: ghost`（未声明名）与
        // 导入名引用在无声明源里同样 fail-closed 具名拒。
        for node in nodes.iter_mut() {
            if let Err(d) = replace_style_in_items(
                &mut node.view.tree,
                &std::collections::BTreeMap::new(),
                &imports,
                true,
            ) {
                return Err(vec![d]);
            }
        }
        return Ok(());
    }
    style_nodes.sort_by_key(|(a, _)| *a);
    let mut lits: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut spans: std::collections::BTreeMap<String, crate::atom_text::Span> =
        std::collections::BTreeMap::new();
    let mut copies: Vec<(String, String, crate::atom_text::Span)> = Vec::new();
    for (_, st) in &style_nodes {
        let name_leaf = field(tree, *st, "name").expect("style name");
        let name = text(tree, name_leaf);
        let value_node = field(tree, *st, "value").expect("style value");
        let value_span = node_span(tree, value_node);
        let value = project::project_expr_node(
            &cached.program,
            &cached.kinds,
            tree,
            snap,
            SourceProfile::AgeComponent,
            value_node,
        );
        match &value.kind {
            crate::source::ast::ExprKind::StrLit(lit) => {
                if lits.insert(name.clone(), lit.clone()).is_some() {
                    return Err(vec![Diagnostic::new(
                        crate::atom_text::Stage::Text,
                        "source.module-duplicate-declaration",
                        format!(
                            "style `{name}` is declared more than once in this module \
                             (component-face style declarations share the module style table)"
                        ),
                        spans.get(&name).copied().unwrap_or(value_span),
                    )]);
                }
                spans.insert(name, node_span(tree, name_leaf));
            }
            // 零参 ident 复制（本地字面量）——延后解析，允许前向（包面同形）。
            crate::source::ast::ExprKind::Var(target) => {
                copies.push((name.clone(), target.name.clone(), value_span));
            }
            crate::source::ast::ExprKind::Call { callee, .. } => {
                return Err(vec![Diagnostic::new(
                    crate::atom_text::Stage::Text,
                    "source.age.style-call",
                    format!(
                        "style `{name} = {}(…)` is a named pending surface in this slice \
                         (parameterised style calls are not folded; parameterised styles are a \
                         package-face form)",
                        callee.name
                    ),
                    value_span,
                )]);
            }
            _other => {
                return Err(vec![Diagnostic::new(
                    crate::atom_text::Stage::Text,
                    "source.age.style-call",
                    format!(
                        "style `{name}` has a value that does not fold to a string literal on \
                         the component face in this slice"
                    ),
                    value_span,
                )]);
            }
        }
    }
    for (name, target, span) in copies {
        let Some(lit) = lits.get(&target).cloned() else {
            return Err(vec![Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.age.style-call",
                format!(
                    "style `{name}` copies `{target}`, which is not a literal style declared in \
                     this module; imported/non-literal style references are a named pending \
                     surface in this slice"
                ),
                span,
            )]);
        };
        if lits.insert(name.clone(), lit).is_some() {
            return Err(vec![Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.module-duplicate-declaration",
                format!(
                    "style `{name}` is declared more than once in this module \
                     (component-face style declarations share the module style table)"
                ),
                span,
            )]);
        }
    }
    for node in nodes.iter_mut() {
        if let Err(d) = replace_style_in_items(&mut node.view.tree, &lits, &imports, true) {
            return Err(vec![d]);
        }
    }
    Ok(())
}

/// PLAN-979：crate 内部领域入口（仅 AGE 领域前门使用）。公共 [`parse_front`]
/// 拒绝领域 profile——这样任何通用消费者都不可能把领域声明静默丢掉。
pub(crate) fn parse_domain_whole_module(
    src: &str,
    profile: SourceProfile,
    budget: FrontBudget,
) -> FrontOutcome {
    debug_assert!(matches!(
        profile,
        SourceProfile::AgeH0 | SourceProfile::AgeComponent
    ));
    parse_whole_module(src, profile, budget)
}

/// PLAN-982（B-CONFIG-I E2）：配置文档前门（crate 内部；公共 [`parse_front`]
/// 对 config-document profile 具名拒绝）。与其它前门共用同一 scanner/快照/预算
/// 管线；差别只在投影：`cfg-*` 规则 → 文档候选（不是通用 Program）。
pub(crate) fn parse_config_document_front(
    src: &str,
    budget: FrontBudget,
) -> Result<crate::source::config_document::ConfigDocumentSyntax, Vec<Diagnostic>> {
    let profile = SourceProfile::ConfigDocument;
    let cached = match profile_grammar(profile) {
        Ok(c) => c,
        Err(e) => return Err(infra_err(e)),
    };
    crate::source::parser::note_tokenize(crate::atom_text::Span {
        start: 0,
        end: src.len(),
    });
    crate::body_work::record_whole(crate::body_work::Stage::WholeParse);
    let store = std::rc::Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, src);
    let ctx_text = SnapshotTokenText { snap: &snap };
    let ParseOutput {
        tree,
        tokens: _,
        lex_diags: _,
        grafted_decls: _,
    } = match parse_snapshot_with_ctx(
        &cached.program,
        &cached.scanner,
        &snap,
        &Budget::from(budget),
        Some(&ctx_text),
    ) {
        Ok(o) => o,
        Err(ace_syntax_runtime::program::RtError::ResourceExhausted { what, limit }) => {
            return Err(vec![Diagnostic::new(
                crate::atom_text::Stage::Text,
                "source.resource",
                format!("resource exhausted: {what} (limit {limit})"),
                crate::atom_text::Span {
                    start: 0,
                    end: src.len(),
                },
            )])
        }
        Err(e) => return Err(infra_err(format!("parse: {e}"))),
    };
    let grammar_id = format!("{}", cached.program.identity.grammar_id.0);
    let input_hash = format!("{:016x}", super::units::content_sha(src.as_bytes()));
    project::project_config_document(
        &cached.program,
        &cached.kinds,
        &tree,
        &snap,
        profile,
        grammar_id,
        input_hash,
    )
}

fn parse_whole_module(src: &str, profile: SourceProfile, budget: FrontBudget) -> FrontOutcome {
    let cached = match profile_grammar(profile) {
        Ok(c) => c,
        Err(e) => return FrontOutcome::Infra(e),
    };
    let store = std::rc::Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, src);
    let ctx_text = SnapshotTokenText { snap: &snap };
    let ParseOutput {
        tree,
        tokens: _,
        lex_diags: _,
        grafted_decls: _,
    } = match parse_snapshot_with_ctx(
        &cached.program,
        &cached.scanner,
        &snap,
        &Budget::from(budget),
        Some(&ctx_text),
    ) {
        Ok(o) => o,
        Err(ace_syntax_runtime::program::RtError::ResourceExhausted { what, limit }) => {
            return FrontOutcome::ResourceExhausted { what, limit };
        }
        Err(e) => return FrontOutcome::Infra(format!("parse: {e}")),
    };
    let program =
        match project::project_program(&cached.program, &cached.kinds, &tree, &snap, profile) {
            Ok(program) => program,
            Err(diags) => {
                // PLAN-979：槽位/结构拒绝是比"首个 token 期望"更精确的结构
                // 事实（例如 `node` 已消费、`view` 槽位遇到 `panel`）——领域
                // profile 上优先报它，保原 span；但**不吞更早的错误**（F6：
                // 只有 hint 不晚于共享投影的最早诊断时才取代它）。普通 profile
                // 的既有首诊断序不变。
                if matches!(profile, SourceProfile::AgeH0 | SourceProfile::AgeComponent) {
                    let hint = tree
                        .diags
                        .iter()
                        .filter(|d| {
                            (d.message.starts_with("expected `") && d.message.contains("slot"))
                                || d.message.starts_with("unexpected token in ")
                        })
                        // tree.diags 里的 hint 已按 (深度, 构造级优先, 位置) 排好
                        // ——取第一条即最精确的领域诊断。
                        .next()
                        .map(|h| {
                            let code = if h.message.contains("slot") {
                                "source.age.slot"
                            } else {
                                "source.age.structure"
                            };
                            Diagnostic::new(
                                crate::atom_text::Stage::Text,
                                code,
                                h.message.clone(),
                                crate::atom_text::Span {
                                    start: h.start as usize,
                                    end: h.end as usize,
                                },
                            )
                        });
                    if let Some(hint) = hint {
                        // AGE 只信"声明的头之前"的共享诊断作为更早错误（F6：
                        // 不吞更早的真实错误）。头之后/之内的通用 token 游走
                        // 诊断（"expected fn"/"skipped unexpected token"）是
                        // 同一失败区的回滚产物，由领域 hint 取代。
                        let head_starts: Vec<&str> = cached
                            .program
                            .ctx_names
                            .iter()
                            .filter(|n| n.role == "node")
                            .map(|n| n.spelling.as_str())
                            .collect();
                        let mut node_head_start: Option<usize> = None;
                        // 叶序扫描：回滚后的声明头是**错误叶**——只认错误叶，
                        // 这样前面合法函数里叫 `node`/`widget` 的标识符不会被
                        // 误当成领域声明头（F9）。
                        tree.for_each_leaf(|s, e, _k, is_error| {
                            if node_head_start.is_none() && is_error {
                                if let Ok(text) = snap.read_str(s, e) {
                                    if head_starts.contains(&text.as_str()) {
                                        node_head_start = Some(s as usize);
                                    }
                                }
                            }
                        });
                        let competing = match node_head_start {
                            Some(head) => diags.iter().any(|d| d.span.start < head),
                            None => false,
                        };
                        if !competing {
                            return FrontOutcome::Diag(vec![hint]);
                        }
                    }
                }
                return FrontOutcome::Diag(diags);
            }
        };
    // PLAN-979：AGE 领域 profile 额外投影未受信领域候选（结构 + 原 span）。
    let mut age_nodes = Vec::new();
    let mut component_nodes = Vec::new();
    if profile == SourceProfile::AgeH0 {
        match project::project_age_nodes(&cached.program, &cached.kinds, &tree, &snap, profile) {
            Ok(nodes) => age_nodes = nodes,
            Err(diags) => return FrontOutcome::Diag(diags),
        }
    } else if profile == SourceProfile::AgeComponent {
        match project::project_component_nodes(
            &cached.program,
            &cached.kinds,
            &tree,
            &snap,
            profile,
        ) {
            Ok(mut nodes) => {
                // PLAN-1072 刀A：组件面 style 惰性视图数据 + 投影期字面量
                // 替换（style 声明在 program 投影层跳过——项目循环的领域前门
                // 单独投影注释同源）。
                if let Err(ds) = apply_component_face_styles(cached, &tree, &snap, &mut nodes) {
                    return FrontOutcome::Diag(ds);
                }
                component_nodes = nodes;
            }
            Err(diags) => return FrontOutcome::Diag(diags),
        }
    }
    FrontOutcome::Ok(FrontOk {
        program,
        grammar_id: format!("{}", cached.program.identity.grammar_id.0),
        caps_version: FRONT_CAPS_VERSION,
        snapshot_len: src.len(),
        cst: tree,
        age_nodes,
        component_nodes,
    })
}

/// 旧签名兼容桥（§5.1：`parse_program_with` 保兼容返回类型，七 profile
/// 路由至通用 CST）。typed 分类由 [`parse_front`] 承载；本桥只作单诊断投影。
pub fn parse_program_via_cst(src: &str, profile: SourceProfile) -> Result<Program, Diagnostic> {
    parse_whole_front_ok(src, profile).map(|ok| ok.program)
}

/// Find instance syntax in the admitted CST without traversing or cloning
/// the AST of an ordinary program with no templates.
pub(crate) fn parse_auto_program_for_instances(src: &str) -> Result<(Program, bool), Diagnostic> {
    let profile = SourceProfile::Auto;
    let ok = parse_whole_front_ok(src, profile)?;
    let template = ok.program.items.iter().any(|item| {
        matches!(item, crate::source::ast::Item::Fn(f) if !f.type_params.is_empty())
    });
    let explicit_call = !template && src.contains('<') && profile_grammar(profile)
        .ok()
        .and_then(|cached| cached.kinds.generic_call)
        .is_some_and(|kind| {
            ok.cst.blocks.iter().any(|block| block.nodes.iter().any(|node| node.kind == kind))
        });
    Ok((ok.program, template || explicit_call))
}

fn parse_whole_front_ok(src: &str, profile: SourceProfile) -> Result<FrontOk, Diagnostic> {
    match parse_front(FrontRequest {
        src,
        profile,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    }) {
        FrontOutcome::Ok(ok) => Ok(ok),
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
    // PLAN-979（评审 F2 观察）：片段入口同样拒绝领域 profile——片段不承载领域
    // 声明，且没有槽位拼写的字节事实源；只经领域前门消费整模块。
    //
    // 复审 M5 第二半：`parse_front` 的**整模块**守卫会先拦住片段请求，所以从那里
    // 永远到不了本函数；只测 `parse_front` 会让"本函数是否拒绝"成为盲区（实测：
    // 直接调用本函数在修复前返回 `Ok`）。故本函数必须自己拒绝，并有专测直接调它。
    if matches!(
        profile,
        SourceProfile::AgeH0 | SourceProfile::AgeComponent | SourceProfile::AgePackage
    ) {
        if let Some(d) = crate::source::age_h0::domain_profile_refusal(profile, src.len()) {
            return FrontOutcome::Diag(vec![d]);
        }
    }
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
        // 片段入口不承载领域声明（AGE 只走 WholeModule）。
        age_nodes: Vec::new(),
        component_nodes: Vec::new(),
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
        (Some("mut"), Some("fn")) if profile.is_auto_language() => Receiver::Mut,
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
