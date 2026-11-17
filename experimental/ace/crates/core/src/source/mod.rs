//! Auto source-subset frontends for the `auto.source.core-i32.draft`
//! (PLAN-745) and `auto.source.scalar-i32.draft` (PLAN-767) profiles.
//!
//! Pipeline position (plan §2): `.at` source → S0 spanned tokens/AST
//! (this module: `lexer`/`ast`/`parser`) → signature pre-registration,
//! lexical name resolution and exact type checking (`resolve`, `typecheck`)
//! → Atom text emission with a generated-token → source-span map
//! (`adapter`) → the existing Schema binder (`crate::descriptor`) and
//! semantic verifier (`crate::verify`).
//!
//! The frontend is purely syntactic here: it accepts the statement-oriented
//! grammar of the plan §5.2 with byte spans into the original source and
//! rejects excluded-surface constructs with `source.*` diagnostics at stage
//! `text`. Which operator/loop surface a parse accepts is selected by the
//! explicit [`SourceProfile`] — the core profile's accept/reject sets are
//! unchanged by the scalar profile's wider surface.

pub mod adapter;
pub mod age_component;
pub mod age_h0;
/// PLAN-981（IF-981-MODULE）：AGE 多模块包前门。
pub mod age_package;
/// PLAN-981（IF-981-ORIGINAL）：AGE 资源服务的源侧 typed 声明。
pub mod age_resources;
pub mod ast;
pub(crate) mod closure_capture;
pub mod closures_context;
pub mod closures_fn;
/// PLAN-982（B-CONFIG-I）：配置文档面 —— 顶层文档语句序列物化为**一个**顺序
/// 入口的 source 前门（`auto.source.config-document.v1`）。
pub mod config_document;
pub mod console;
pub mod cst;
pub mod decl_env;
pub mod domain;
// PLAN-980（IF-980-FIELDS）：字段级投影完整性清单的通用机制（接既有
// mapping_complete/component_coverage 的字段粒度层）。
pub mod field_manifest;
pub mod fstr;
#[cfg(any(test, feature = "legacy-syntax-oracle"))]
pub mod lexer;
pub mod modules;
pub mod packages;
pub mod parser;
pub mod platform_io;
pub mod resolve;
pub mod sched_desugar;
pub mod typecheck;

pub use adapter::{
    compile_aggregates_source, compile_closures_source, compile_closures_source_incremental,
    compile_collections_source, compile_entry_unit_source, compile_host_console_source,
    compile_numeric_source, compile_parts, compile_parts_with, compile_program_parts,
    compile_scheduler_source,
    compile_record_source, compile_scalar_source, compile_source, compile_text_record_source,
    compile_text_source, emit_collections_function_batch, emit_function_body,
    emit_resolved_module_parts, emit_resolved_packages_parts, emit_resolved_packages_parts_profile,
    emit_resolved_parts, prepare_candidate_document, prepare_function_candidate,
    prepare_module_candidate, BodyEmitReceipt, ClosuresFrontendLedger, CompiledSource, FnCandidate,
    SourceParts, RUNTIME_TEXT_CAPABILITY,
};
pub use parser::{parse_program, parse_program_with};
/// PLAN-982（B-CONFIG-I）：配置文档前门（单一顺序入口）。
pub use config_document::{
    compile_config_document, config_document_entry, ConfigDocumentFront, CONFIG_DOCUMENT_DOMAIN,
    CONFIG_DOCUMENT_ENTRY_NAME, CONFIG_DOCUMENT_HIDDEN_PREFIX, CONFIG_DOCUMENT_VALUE_SCHEMA,
};
// PLAN-929（T-10）：旧手写 parser 面（整源/三片段/词法）退役为差分 oracle——
// 仅测试/`legacy-syntax-oracle` feature 可见（编译期不可达=退役门）。
#[cfg(any(test, feature = "legacy-syntax-oracle"))]
pub use parser::{
    parse_closure_fragment, parse_function_fragment, parse_method_fragment,
    parse_program_with_handwritten, parser_is_oracle_gated,
};
pub use resolve::{
    env_fn_body_identity, env_fn_count, env_fn_names, env_fn_regions, env_free_fn_index,
    env_method_index, env_note_cold_walks, recheck_function, resolve_closures_cached, resolve_env,
    resolve_env_with_supply, resolve_with, CachedOwner, CheckedEnv, ClosuresOwnerCache, ResolvedFn,
    ResolvedProgram, VT,
};
pub use typecheck::{check_body_in, typecheck_with, typecheck_with_skip};

/// The core source profile (PLAN-745): `+ * <`, negative literals only as
/// `-` before digits, `while` conditional loops.
pub const SOURCE_PROFILE: &str = "auto.source.core-i32.draft";

/// The scalar source profile (PLAN-767): the core surface plus
/// `- / % > <= >= == !=` (binary), unary `- !`, `&& ||` short-circuit, and
/// conditional `for <bool-expr> { ... }` loops (`while` is rejected here).
pub const SCALAR_SOURCE_PROFILE: &str = "auto.source.scalar-i32.draft";

/// The text source profile (PLAN-770): the scalar surface plus str/Str/char
/// types, string/char literals (with the 774 escape whitelist and cross-line
/// string literals whose line breaks enter the value verbatim), the text
/// method set (`len/len_bytes/char_at/slice/to/clone/to_codepoint`,
/// `char.from_codepoint`), text `+` concatenation, text/char `==`/`!=`,
/// parameter mode words (`mut`/`move`) and call-site `.move`.
pub const TEXT_SOURCE_PROFILE: &str = "auto.source.text.draft";
/// The record source profile (PLAN-771): the scalar surface plus nominal
/// records — `type` declarations with fields and `static fn`/`fn` methods,
/// positional/named construction, nested field paths, readonly instance
/// receivers, and explicit `mut` free-function params (774 D-04).
pub const RECORD_SOURCE_PROFILE: &str = "auto.source.record.draft";
/// The text-record source profile (PLAN-782): the union of the text and
/// record surfaces — nominal records owning `Str` (and carrying `char`)
/// fields, aggregate explicit `.move`/`.clone()`, field reads/replacement,
/// plus the whole scalar/text operator and method surface.
pub const TEXT_RECORD_SOURCE_PROFILE: &str = "auto.source.text-record.draft";
/// PLAN-982（B-CONFIG-I）：配置文档 profile —— 文档语义面。E1 裁剪 = text-record
/// 全表面 ∪ 顶层脚本语句序列（语句按源顺序进入**一个**顺序入口）；配置文档
/// 专属语法（顶层 `key: value`、`dep`/节点三形）由 E2 在本面增量。消费者只有
/// [`config_document::compile_config_document`]。
pub const CONFIG_DOCUMENT_SOURCE_PROFILE: &str = "auto.source.config-document.v1";
/// The entry-unit source profile (PLAN-862, R2-E/SEM-05): the scalar
/// operator surface plus the ENTRY surface — `fn main() int`,
/// `fn main() void` (Unit result), a single-file top-level statement
/// sequence (script) whose trailing i32 expression statement is the
/// result, and the entry uniqueness/diagnostic matrix frozen by the T-02
/// `entry-contract.md`. Unit is a RESULT type only; no value-level Unit.
pub const ENTRY_UNIT_SOURCE_PROFILE: &str = "auto.source.entry-unit.draft";
/// The collections source profile (PLAN-864): the source-domain twin of
/// `av.collections/1` — typed `List<T>` locals/methods, `?T`/`Result<T,E>`
/// and finite `enum` variants with `is` dispatch, `for x in <list>`
/// iteration, `Str` literals + `.len()`, owned records (non-container
/// fields), record construction and `.move`/`.clone()`. The MIR lowering
/// reuses the frozen 822 instruction set; profile 4 semantics are
/// consumed unchanged.
pub const COLLECTIONS_SOURCE_PROFILE: &str = "auto.source.collections.draft";
/// PLAN-865: the entry-unit and collections surface with source closures.
pub const CLOSURES_SOURCE_PROFILE: &str = "auto.source.closures.draft";
/// PLAN-961: the numeric surface (closures superset + wide/narrow integers,
/// floats, typed literals, bit ops, shifts as methods, conversions).
pub const NUMERIC_SOURCE_PROFILE: &str = "auto.source.numeric.draft";
/// PLAN-962: the aggregates surface (numeric superset + composite data
/// structures — nested containers, tuple/array literals, named ctors,
/// destructuring, index places, ranges, wider patterns; the T-08 face
/// adds union/tag/grid/Hold/Box).
pub const AGGREGATES_SOURCE_PROFILE: &str = "auto.source.aggregates.draft";
/// PLAN-979（T-H0-I）：AGE 最小 Node 领域面（`node <name> { view { text <expr> {} } }`）。
/// 该面 = text-record 全表面 + 领域节点声明；领域结构名只由 `<profile>` 的
/// `contextual` 槽位声明赋予（词法面不被扩成保留字）。**领域候选只经
/// [`age_h0::compile_age_h0`] 消费**——通用解析/编译入口显式拒绝本 profile，
/// 避免领域声明被静默丢弃。
pub const AGE_H0_SOURCE_PROFILE: &str = "auto.source.age-h0.draft";

/// PLAN-968: the scheduler surface (aggregates superset + the task actor
/// face — `task` declarations with `on` message arms, `Task.spawn`,
/// handle `send`/`ask`/`stop`, `reply`, `await`, `go`, `sleep` and
/// console `print`). Consumed by `compile_scheduler_source`.
pub const SCHEDULER_SOURCE_PROFILE: &str = "auto.source.scheduler.draft";

/// PLAN-979 T-03：AGE 组件面 profile（`node { model … on … view … }`）。与 `age-h0`
/// 同源不同裁剪：H0 合同不变，组件语义只在本面出现。
pub const AGE_COMPONENT_SOURCE_PROFILE: &str = "auto.source.age-component.draft";

/// PLAN-981（SOURCE 批，T-01）：AGE **多模块包面** —— 包模块图（`pub fn` /
/// `use <mod>.<item>` / 模块级 const、var / 多段路径）与 AGE 组件的**同一次**
/// 解析面。只在 [`AgePackage`](SourceProfile::AgePackage) 上启用；H0 与组件
/// 面的裁剪字节不变（`age-package` 只是多出来的一个未启用名字）。
pub const AGE_PACKAGE_SOURCE_PROFILE: &str = "auto.source.age-package.draft";

/// The explicitly selected source profile of a compilation. Never inferred
/// from file extensions or content.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceProfile {
    Core,
    Scalar,
    Text,
    Record,
    TextRecord,
    EntryUnit,
    HostConsole,
    Collections,
    Closures,
    /// PLAN-961: the numeric profile (closures surface + numerics).
    Numeric,
    /// PLAN-962: the aggregates profile (numeric surface + composites).
    Aggregates,
    /// PLAN-968: the scheduler profile (aggregates superset + the task
    /// actor surface; see [`SCHEDULER_SOURCE_PROFILE`]).
    Scheduler,
    /// PLAN-979: the AGE H0 domain profile (text-record surface + AGE node
    /// declarations). Only the AGE domain front door consumes it.
    AgeH0,
    /// PLAN-979 T-03: the AGE component profile (`node { model … on … view … }`).
    /// Same source grammar, different trim; generic entries refuse it.
    AgeComponent,
    /// PLAN-981: the AGE package profile (the package module graph ∪ the AGE
    /// component face, parsed once). Only
    /// [`age_package::compile_age_package`](crate::source::age_package) consumes
    /// it; every generic entry refuses it.
    AgePackage,
    /// PLAN-982: the configuration-document profile (E1 trim = the
    /// text-record surface ∪ the top-level script statement sequence, which
    /// the projection materializes as ONE sequential entry function). Only
    /// [`config_document::compile_config_document`] consumes it.
    ConfigDocument,
}

impl SourceProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceProfile::Core => SOURCE_PROFILE,
            SourceProfile::Scalar => SCALAR_SOURCE_PROFILE,
            SourceProfile::Text => TEXT_SOURCE_PROFILE,
            SourceProfile::Record => RECORD_SOURCE_PROFILE,
            SourceProfile::TextRecord => TEXT_RECORD_SOURCE_PROFILE,
            SourceProfile::EntryUnit => ENTRY_UNIT_SOURCE_PROFILE,
            SourceProfile::HostConsole => HOST_CONSOLE_SOURCE_PROFILE,
            SourceProfile::Collections => COLLECTIONS_SOURCE_PROFILE,
            SourceProfile::Closures => CLOSURES_SOURCE_PROFILE,
            SourceProfile::Numeric => NUMERIC_SOURCE_PROFILE,
            SourceProfile::Aggregates => AGGREGATES_SOURCE_PROFILE,
            SourceProfile::Scheduler => SCHEDULER_SOURCE_PROFILE,
            SourceProfile::AgeH0 => AGE_H0_SOURCE_PROFILE,
            SourceProfile::AgeComponent => AGE_COMPONENT_SOURCE_PROFILE,
            SourceProfile::AgePackage => AGE_PACKAGE_SOURCE_PROFILE,
            SourceProfile::ConfigDocument => CONFIG_DOCUMENT_SOURCE_PROFILE,
        }
    }

    /// The HIR identity domain documents of this profile compile into.
    pub fn hir_domain(&self) -> crate::hir::HirDomain {
        match self {
            SourceProfile::Core => crate::hir::HirDomain::Core,
            SourceProfile::Scalar => crate::hir::HirDomain::Scalar,
            SourceProfile::Text => crate::hir::HirDomain::Text,
            SourceProfile::Record => crate::hir::HirDomain::Record,
            SourceProfile::TextRecord => crate::hir::HirDomain::TextRecord,
            SourceProfile::EntryUnit => crate::hir::HirDomain::EntryUnit,
            SourceProfile::HostConsole => crate::hir::HirDomain::HostConsole,
            SourceProfile::Collections => crate::hir::HirDomain::Collections,
            SourceProfile::Closures => crate::hir::HirDomain::Closures,
            // PLAN-961: numeric documents verify under the closures-superset
            // domain (numeric typing rules live in the shared walks).
            SourceProfile::Numeric => crate::hir::HirDomain::Closures,
            // PLAN-962: aggregates documents verify under the same
            // collections-family domain (aggregate admission lives in the
            // resolver and the MIR verifier, keyed by profile).
            SourceProfile::Aggregates => crate::hir::HirDomain::Closures,
            // PLAN-968: scheduler documents verify under the same
            // collections-family domain (task admission lives in the
            // resolver and the MIR verifier, keyed by profile).
            SourceProfile::Scheduler => crate::hir::HirDomain::Closures,
            // PLAN-979: the AGE domain's ordinary surface is text-record; the
            // domain description is verified inside that same domain, never a
            // second checker.
            SourceProfile::AgeH0 => crate::hir::HirDomain::TextRecord,
            SourceProfile::AgeComponent => crate::hir::HirDomain::TextRecord,
            // PLAN-981: the AGE package face's ordinary surface is the
            // package/text-record one; its domain rows are verified inside the
            // same shared domain, never a second checker.
            SourceProfile::AgePackage => crate::hir::HirDomain::TextRecord,
            // PLAN-982：配置文档的普通表达式面 = text-record 域；文档语义只经
            // [`config_document::compile_config_document`]，不新开第二检查器。
            SourceProfile::ConfigDocument => crate::hir::HirDomain::TextRecord,
        }
    }

    /// Whether the scalar-only operator surface (new binary operators,
    /// unary ops, logical short-circuit) and the conditional `for` loop
    /// are accepted.
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            SourceProfile::Scalar
                | SourceProfile::Text
                | SourceProfile::TextRecord
                | SourceProfile::HostConsole
                | SourceProfile::EntryUnit
                | SourceProfile::Collections
                | SourceProfile::Closures
                | SourceProfile::Numeric
                | SourceProfile::Aggregates
                // PLAN-968: the scheduler face keeps the scalar operator surface.
                | SourceProfile::Scheduler
                // PLAN-979: the AGE domain face keeps the text-record surface.
                | SourceProfile::AgeH0
                | SourceProfile::AgeComponent
                | SourceProfile::AgePackage
                // PLAN-982: the configuration-document face keeps the
                // text-record operator surface.
                | SourceProfile::ConfigDocument
        )
    }

    /// PLAN-864: whether the collections surface is accepted — List<T>/
    /// Option/Result/enum types, `is` dispatch, `for-in` iteration, record
    /// construction with owned fields, Str literals and the restricted
    /// method surface.
    pub fn is_collections(&self) -> bool {
        matches!(
            self,
            SourceProfile::Collections
                | SourceProfile::Closures
                | SourceProfile::Numeric
                | SourceProfile::Aggregates
                // PLAN-968: the scheduler face keeps the collections surface.
                | SourceProfile::Scheduler
        )
    }

    /// PLAN-865: whether lambda expressions and closure indirect calls are
    /// accepted on this otherwise collection-compatible source surface.
    pub fn is_closures(&self) -> bool {
        matches!(
            self,
            SourceProfile::Closures
                | SourceProfile::Numeric
                | SourceProfile::Aggregates
                // PLAN-968: the scheduler face keeps the closures surface.
                | SourceProfile::Scheduler
        )
    }

    /// PLAN-961: whether the numeric surface is accepted — typed literals,
    /// numeric type words, bit ops, shift methods and conversions.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            SourceProfile::Numeric | SourceProfile::Aggregates | SourceProfile::Scheduler
        )
    }

    /// PLAN-962: whether the aggregate surface is accepted — nested
    /// containers, composite record/variant fields, tuples, index
    /// places, ranges and the wider pattern set.
    pub fn is_aggregates(&self) -> bool {
        matches!(self, SourceProfile::Aggregates | SourceProfile::Scheduler)
    }

    /// PLAN-968: whether the task actor surface is accepted — `task`
    /// declarations, `Task.spawn`, handle `send`/`ask`, `reply`, `await`,
    /// `go`, `sleep` and scheduler-profile `print`.
    pub fn is_scheduler(&self) -> bool {
        matches!(self, SourceProfile::Scheduler)
    }

    /// PLAN-864: whether string literals lex on this surface (the text
    /// profiles and the collections profile, which owns `Str` values).
    pub fn admits_text_literals(&self) -> bool {
        self.is_text() || self.is_collections()
    }

    /// Whether the text surface (str/Str/char, literals, text methods,
    /// mode words) is accepted (PLAN-770).
    pub fn is_text(&self) -> bool {
        matches!(
            self,
            SourceProfile::Text
                | SourceProfile::TextRecord
                | SourceProfile::HostConsole
                | SourceProfile::AgeH0
                | SourceProfile::AgeComponent
                | SourceProfile::AgePackage
                // PLAN-982: configuration documents are text/object values.
                | SourceProfile::ConfigDocument
        )
    }

    /// Whether the record surface (type declarations, construction, field
    /// paths, methods, `mut` params — PLAN-771) is accepted.
    pub fn is_record(&self) -> bool {
        matches!(
            self,
            SourceProfile::Record
                | SourceProfile::TextRecord
                | SourceProfile::Collections
                | SourceProfile::Closures
                | SourceProfile::Aggregates
                // PLAN-968: the scheduler face keeps the record surface.
                | SourceProfile::Scheduler
                | SourceProfile::AgeH0
                | SourceProfile::AgeComponent
                | SourceProfile::AgePackage
                // PLAN-982: configuration documents construct objects.
                | SourceProfile::ConfigDocument
        )
    }

    /// PLAN-862: whether the ENTRY surface is accepted — `void` results,
    /// the single-file script form, leading/trailing expression
    /// statements and the entry uniqueness matrix.
    pub fn is_entry_unit(&self) -> bool {
        matches!(
            self,
            SourceProfile::EntryUnit
                | SourceProfile::Closures
                | SourceProfile::HostConsole
                // PLAN-982：配置文档是**一个**顺序入口（文档语句序列），入口
                // 形状沿用脚本入口合同；文档的值不走返回值（见
                // `config_document::compile_config_document`）。
                | SourceProfile::ConfigDocument
        )
    }

    /// The widest operator surface this profile accepts: record builds on
    /// the scalar surface.
    pub fn surface_is_scalar_or_wider(&self) -> bool {
        // every NON-CORE profile builds on the scalar operator surface:
        // scalar itself, text (PLAN-770), record (PLAN-771) and the
        // entry-unit surface (PLAN-862)
        matches!(
            self,
            SourceProfile::Scalar
                | SourceProfile::Text
                | SourceProfile::Record
                | SourceProfile::TextRecord
                | SourceProfile::HostConsole
                | SourceProfile::EntryUnit
                | SourceProfile::Collections
                | SourceProfile::Closures
                // PLAN-961: the numeric surface is a closures superset.
                | SourceProfile::Numeric
                // PLAN-962: the aggregates surface is a numeric superset.
                | SourceProfile::Aggregates
                // PLAN-979: the AGE domain face is a text-record superset.
                | SourceProfile::AgeH0
                | SourceProfile::AgeComponent
                | SourceProfile::AgePackage
                // PLAN-982: the configuration-document face is a text-record
                // superset.
                | SourceProfile::ConfigDocument
        )
    }

    pub fn of(s: &str) -> Option<SourceProfile> {
        match s {
            SOURCE_PROFILE => Some(SourceProfile::Core),
            SCALAR_SOURCE_PROFILE => Some(SourceProfile::Scalar),
            TEXT_SOURCE_PROFILE => Some(SourceProfile::Text),
            RECORD_SOURCE_PROFILE => Some(SourceProfile::Record),
            TEXT_RECORD_SOURCE_PROFILE => Some(SourceProfile::TextRecord),
            ENTRY_UNIT_SOURCE_PROFILE => Some(SourceProfile::EntryUnit),
            HOST_CONSOLE_SOURCE_PROFILE => Some(SourceProfile::HostConsole),
            COLLECTIONS_SOURCE_PROFILE => Some(SourceProfile::Collections),
            CLOSURES_SOURCE_PROFILE => Some(SourceProfile::Closures),
            NUMERIC_SOURCE_PROFILE => Some(SourceProfile::Numeric),
            AGGREGATES_SOURCE_PROFILE => Some(SourceProfile::Aggregates),
            AGE_H0_SOURCE_PROFILE => Some(SourceProfile::AgeH0),
            AGE_COMPONENT_SOURCE_PROFILE => Some(SourceProfile::AgeComponent),
            AGE_PACKAGE_SOURCE_PROFILE => Some(SourceProfile::AgePackage),
            CONFIG_DOCUMENT_SOURCE_PROFILE => Some(SourceProfile::ConfigDocument),
            _ => None,
        }
    }
}

/// Capability that activates the closed set of test-only trace intrinsics.
pub const TRACE_CAPABILITY: &str = "hir.test.trace.v1";

/// PLAN-962: document-level capability every aggregates-profile document
/// requires. It is the single admission key downstream verifiers use to
/// widen the frozen 822 one-layer element domain (nested List, composite
/// record/variant payloads): documents without it keep the strict
/// collections/closures admission byte-for-byte.
pub const AGGREGATES_ADMISSION_CAP: &str = "av.aggregates/1";

/// Map a source-level callee name to its intrinsic symbol. These names
/// resolve only when the caller provided [`TRACE_CAPABILITY`], and user
/// functions and locals always win over them.
pub fn trace_intrinsic(callee: &str) -> Option<&'static str> {
    match callee {
        "mark_a" => Some("hir.test.mark_a"),
        "mark_b" => Some("hir.test.mark_b"),
        _ => None,
    }
}

/// PLAN-936 (T-04): capability that activates the closed set of CT meta
/// query intrinsics (`ct_field_count` / `ct_has_field`). Defaults to
/// CLOSED; these names are PROTECTED on the CT face — with the capability
/// granted, a user declaration of the same name is a named conflict (it
/// never silently shadows the query).
pub const META_CAPABILITY: &str = "cte.meta.fields/v1";

/// Map a source-level callee name to its meta-query intrinsic symbol
/// (PLAN-936 §5.2). The type/field operands are QUERY SELECTORS, never
/// runtime values: they are validated at bind and consumed by the CT
/// query layer; only i32/bool values ever reach the AV.
pub fn meta_intrinsic(callee: &str) -> Option<&'static str> {
    match callee {
        "ct_field_count" => Some("cte.meta.field_count"),
        "ct_has_field" => Some("cte.meta.has_field"),
        _ => None,
    }
}

/// PLAN-863: explicitly selected typed console source profile.
pub const HOST_CONSOLE_SOURCE_PROFILE: &str = "auto.source.host-console.draft";
