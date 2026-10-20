//! Contract schemas (`autolang.case/1`, and the strict v2 superset
//! `autolang.case/2` from PLAN-788) and strict decoding from [`CVal`].
//!
//! Every decode failure carries a field path (e.g. `checks[2].expect.stdout`) and a
//! concrete reason; unknown fields, wrong types, invalid id/path forms,
//! unsupported fields (this pilot: variants/relations/recipes/budgets/
//! platform/native adapters) are all errors — nothing is parsed then ignored.
//!
//! v2 is a strict superset (PLAN-788 T-01 contract-decision.md §3): the v1
//! accept/reject sets are byte-identical — a v1 document carrying any v2-only
//! field is still a located unknown-field error, and v2 avm/a2r checks behave
//! exactly like v1. v2 adds: source syntax `hir-atom` (.atom raw HIR input),
//! adapters `ac`/`syntax`, actions check/build (ac) and parse/edit (syntax),
//! typed v2 stages, ac options (profile/target/entry/cache/budget), the
//! native `exit` oracle and optional `perf` assertion face. `syntax` is
//! decodable from day one but its executor is bound only after PLAN-787
//! merges (D-787); until then executing it is an admission ERROR, never a
//! SKIP and never silently green.

use crate::admission::{check_limit, validate_virtual_path, Limits};
use crate::atom::CVal;

pub const CASE_SCHEMA: &str = "autolang.case/1";
pub const CASE_SCHEMA_V2: &str = "autolang.case/2";

/// Which contract version a document decoded under. New-in-v2 fields are
/// only reachable under [`ContractVersion::V2`]; v1 known-field sets are
/// untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractVersion {
    V1,
    V2,
}

// PLAN-788 frozen budget hard caps (contract-decision.md §5). Declared
// per-check budgets may only tighten these; raising them is a contract
// revision, not a case-author decision.
pub const BUDGET_HARD_CAP_BUILD_MS: u64 = 60_000;
pub const BUDGET_HARD_CAP_RUN_MS: u64 = 5_000;
pub const BUDGET_HARD_CAP_TOTAL_MS: u64 = 70_000;
/// The only target the first native batch imports (Windows x64 msvc).
pub const AC_TARGET: &str = "x86_64-pc-windows-msvc";
/// Implicit ac entry when the case declares none.
pub const AC_ENTRY_DEFAULT: &str = "main";

// ------------------------------------------------------------------ types --

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Adapter {
    Avm,
    A2r,
    /// PLAN-788 v2: native AC adapter (core/scalar via the ac-case worker).
    Ac,
    /// PLAN-788 v2: 787 grammar adapter. Decodable now; the executor binds
    /// only after PLAN-787 merges (D-787) — executing it before that is an
    /// admission ERROR.
    Syntax,
    /// PLAN-796 T-06: restricted probe/run seam — a REGISTERED declarative
    /// recipe executes against a trusted worker tool identity (never a
    /// case-carried executable). First recipe: sema/query-v1 (the 796
    /// semantic probe); 797 registers its ABI recipe under the same
    /// contract, never a second runner.
    Probe,
}

impl Adapter {
    pub fn as_str(&self) -> &'static str {
        match self {
            Adapter::Avm => "avm",
            Adapter::A2r => "a2r",
            Adapter::Ac => "ac",
            Adapter::Syntax => "syntax",
            Adapter::Probe => "probe",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Run,
    Emit,
    /// ac: frontend + verify only (no PE).
    Check,
    /// ac: Checked → real COFF/PE.
    Build,
    /// syntax: CST/diagnostics projection over one source.
    Parse,
    /// syntax: bounded edit sequence against one base (T-06).
    Edit,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Run => "run",
            Action::Emit => "emit",
            Action::Check => "check",
            Action::Build => "build",
            Action::Parse => "parse",
            Action::Edit => "edit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryMode {
    Script,
    Expression,
    Program,
    Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syntax {
    /// `.at`
    Auto,
    /// `.as` (lowered by the pipeline via the entry path suffix)
    AutoScript,
    /// `.atom` (v2 only): raw HIR Atom text input — checked/bound by the
    /// real descriptor binder, never derived-only negatives.
    HirAtom,
    /// `.atomdata` (v2 only, PLAN-796): Atom-format DATA payload consumed
    /// by probe recipes that declare it (data input, not executable
    /// source). Registered with the probe seam; sema/query-v1 consumes
    /// `.at` sources and rejects this kind at admission.
    AtomData,
}

impl Syntax {
    pub fn extension(&self) -> &'static str {
        match self {
            Syntax::Auto => "at",
            Syntax::AutoScript => "as",
            Syntax::HirAtom => "atom",
            Syntax::AtomData => "atomdata",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Syntax::Auto => "auto",
            Syntax::AutoScript => "auto-script",
            Syntax::HirAtom => "hir-atom",
            Syntax::AtomData => "atom-data",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PayloadSpec {
    /// Anchor of a heading section holding exactly one direct fence.
    Block(String),
    /// Sidecar file path, relative to the case file.
    File(String),
    /// Literal inline text (exact bytes of the UTF-8 string).
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSpec {
    pub payload: PayloadSpec,
    pub syntax: Syntax,
    /// Virtual project path; extension must match `syntax`.
    pub path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileEntry {
    pub path: String,
    pub payload: PayloadSpec,
}

/// AC source profile the first native batch imports (PLAN-788 §1: core and
/// scalar only; text/record/text-record are explicitly NOT imported and are
/// decode errors, never SKIPs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcProfile {
    Core,
    Scalar,
}

impl AcProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            AcProfile::Core => "core",
            AcProfile::Scalar => "scalar",
        }
    }
}

/// Cache participation mode for ac checks (contract-decision.md §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    /// No cache reads; real recompute; publishes under the safe contract.
    Fresh,
    /// get→compute→publish (HIR hits re-verify; object hits still link+run).
    Reuse,
    /// Existing store only (missing root is an infrastructure error); hits
    /// consumed, misses reported and really recomputed but never published.
    ReadOnly,
}

impl CacheMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheMode::Fresh => "fresh",
            CacheMode::Reuse => "reuse",
            CacheMode::ReadOnly => "read-only",
        }
    }
}

/// Per-check ac budget declaration. `None` fields mean "hard cap" — decode
/// fills nothing here; the executor clamps to the frozen constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BudgetSpec {
    pub build_ms: Option<u64>,
    pub run_ms: Option<u64>,
    pub total_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// a2r/emit only: module name passed to the emitter (legacy-compat
    /// metadata; does not affect emitted bytes).
    pub module_name: Option<String>,
    /// ac only (v2). Decoded to its effective value: absent → the frozen
    /// implicit default (Core / AC_TARGET / "main" / Reuse).
    pub ac_profile: Option<AcProfile>,
    pub ac_target: Option<String>,
    pub ac_entry: Option<String>,
    pub ac_cache: Option<CacheMode>,
    pub ac_budget: Option<BudgetSpec>,
    /// v2 ac only (R2 F01): declared frontend capabilities (e.g.
    /// "hir.test.trace.v1"); empty when absent.
    pub ac_capabilities: Option<Vec<String>>,
    /// v2 ac only (R2 F01): NAMED support library to link ("trace-marks").
    pub ac_support_lib: Option<String>,
    /// syntax only (T-06): grammar selector, default "auto-core".
    pub syntax_grammar: Option<String>,
    /// syntax only (T-06): ordered single-edit steps.
    pub syntax_edits: Option<Vec<SyntaxEdit>>,
    /// probe only (PLAN-796 T-06): the REGISTERED recipe id (admission
    /// rejects anything outside REGISTERED_PROBE_RECIPES — a case can
    /// never name an executable).
    pub probe_recipe: Option<String>,
    /// probe only: ordered declarative steps (1..=PROBE_MAX_STEPS).
    pub probe_steps: Option<Vec<ProbeStep>>,
    /// probe only, sema/query-v1: the semantic query binding.
    pub probe_sema: Option<ProbeSemaSpec>,
    /// probe only, abi/interop-v1 (PLAN-797 T-06): the ABI interop leg
    /// binding. The deck itself is the case's `atomdata` source (data,
    /// never an executable); the binding selects which interop leg the
    /// frozen pipeline drives.
    pub probe_abi: Option<ProbeAbiSpec>,
    /// probe only (PLAN-798): ac-av/core-run-v1 binding.
    pub probe_av: Option<ProbeAvSpec>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            module_name: None,
            ac_profile: None,
            ac_target: None,
            ac_entry: None,
            ac_cache: None,
            ac_budget: None,
            ac_capabilities: None,
            ac_support_lib: None,
            syntax_grammar: None,
            syntax_edits: None,
            probe_recipe: None,
            probe_steps: None,
            probe_sema: None,
            probe_abi: None,
            probe_av: None,
        }
    }
}

impl Options {
    /// Effective ac option values: the declared value, else the suite
    /// default already merged by `apply_defaults`, else the frozen implicit
    /// default. Executors must consume ac options ONLY through these.
    pub fn effective_profile(&self) -> AcProfile {
        self.ac_profile.unwrap_or(AcProfile::Core)
    }

    pub fn effective_target(&self) -> &str {
        self.ac_target.as_deref().unwrap_or(AC_TARGET)
    }

    pub fn effective_entry(&self) -> &str {
        self.ac_entry.as_deref().unwrap_or(AC_ENTRY_DEFAULT)
    }

    pub fn effective_cache(&self) -> CacheMode {
        self.ac_cache.unwrap_or(CacheMode::Reuse)
    }

    /// Effective budget: declared/merged value, `None` field = hard cap.
    pub fn effective_budget(&self) -> BudgetSpec {
        self.ac_budget.unwrap_or_default()
    }

    /// Effective declared capabilities (R2 F01): merged list, else empty.
    pub fn effective_capabilities(&self) -> &[String] {
        self.ac_capabilities.as_deref().unwrap_or(&[])
    }

    /// Effective NAMED support library (R2 F01), else none.
    pub fn effective_support_lib(&self) -> Option<&str> {
        self.ac_support_lib.as_deref()
    }

    /// Effective syntax grammar (T-06): declared/merged, else auto-core.
    pub fn effective_grammar(&self) -> &str {
        self.syntax_grammar.as_deref().unwrap_or("auto-core")
    }
}

/// Typed stage for error completions. v1/avm keeps `parse`/`type`/`run`;
/// v2 ac checks use the full pipeline boundary set (contract-decision.md §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Parse,
    Type,
    Run,
    Resolve,
    Typecheck,
    Bind,
    Verify,
    Capability,
    Backend,
    Link,
}

impl Stage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Stage::Parse => "parse",
            Stage::Type => "type",
            Stage::Run => "run",
            Stage::Resolve => "resolve",
            Stage::Typecheck => "typecheck",
            Stage::Bind => "bind",
            Stage::Verify => "verify",
            Stage::Capability => "capability",
            Stage::Backend => "backend",
            Stage::Link => "link",
        }
    }

    /// Stages legal in an error completion for this adapter. avm/a2r keep
    /// the v1 set byte-identically; ac gets the typed pipeline boundaries;
    /// syntax negatives are parse-stage diagnostics only (rest binds at
    /// T-06).
    pub fn supported_for(adapter: Adapter) -> &'static [&'static str] {
        match adapter {
            Adapter::Avm | Adapter::A2r => &["parse", "type", "run"],
            Adapter::Ac => &[
                "parse",
                "resolve",
                "typecheck",
                "bind",
                "verify",
                "capability",
                "backend",
                "link",
                "run",
            ],
            Adapter::Syntax => &["parse"],
            // PLAN-796: probe failure completions may declare frontend
            // stages only; worker/protocol failures are Infrastructure
            // errors at the runner level and never satisfy language
            // negatives (same boundary as ac).
            // PLAN-798 T-07: value-reporting probe recipes execute the
            // pipeline, so run-stage completions (language traps) are
            // declarable alongside the parse/resolve/typecheck faces.
            Adapter::Probe => &["parse", "resolve", "typecheck", "run"],
        }
    }

    fn decode(s: &str) -> Option<Stage> {
        Some(match s {
            "parse" => Stage::Parse,
            "type" => Stage::Type,
            "run" => Stage::Run,
            "resolve" => Stage::Resolve,
            "typecheck" => Stage::Typecheck,
            "bind" => Stage::Bind,
            "verify" => Stage::Verify,
            "capability" => Stage::Capability,
            "backend" => Stage::Backend,
            "link" => Stage::Link,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageMatcher {
    Exact(String),
    Contains(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Completion {
    Success,
    Error {
        stage: Stage,
        code: Option<String>,
        message: Option<MessageMatcher>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compare {
    Bytes,
    Lf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamExpect {
    pub text: String,
    pub compare: Compare,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResultExpect {
    /// Only `legacy-display` exists in v1.
    pub format: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactExpect {
    /// Logical output path (e.g. "main.rs").
    pub path: String,
    /// Sidecar file with the expected bytes, relative to the case file.
    pub file: String,
    pub compare: Compare,
}

/// Optional performance assertion face (v2 ac). Absent fields are
/// record-only; the reference-environment recipe (contract-decision §8)
/// decides enforcement — ordinary unit tests never carry wall-clock gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PerfExpect {
    pub total_ms_max: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpectSpec {
    pub completion: Completion,
    pub result: Option<ResultExpect>,
    pub stdout: Option<StreamExpect>,
    pub stderr: Option<StreamExpect>,
    pub artifacts: Vec<ArtifactExpect>,
    /// v2 ac/run only: native exit-code oracle. A trap (exit 70) is an
    /// ordinary declarable exit, not an error completion.
    pub exit: Option<i32>,
    /// v2 ac only: optional perf assertion face.
    pub perf: Option<PerfExpect>,
    /// v2 probe only (PLAN-796): per-query-step structured expectations.
    pub probe: Option<Vec<ProbeStepExpect>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkipSpec {
    pub reason: String,
    pub issue: String,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckSpec {
    pub id: String,
    pub adapter: Adapter,
    pub action: Action,
    pub tier: Option<String>,
    pub options: Options,
    pub expect: ExpectSpec,
    /// Validated reference to an earlier check in the same case (DAG edge).
    /// The pilot has no product consumers; this only proves the wiring.
    pub product_from: Option<String>,
    pub skip: Option<SkipSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseSpec {
    pub schema: String,
    pub id: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub source: SourceSpec,
    pub files: Vec<FileEntry>,
    pub entry: EntryMode,
    pub checks: Vec<CheckSpec>,
    /// Case-level explicit skip (`status: { skip: … }`), validated at decode.
    pub status_skip: Option<SkipSpec>,
}

// ----------------------------------------------------------------- errors --

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractError {
    /// Field path from the case root, e.g. `checks[1].expect.stdout`.
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ContractError {}

fn err(path: &str, message: impl Into<String>) -> ContractError {
    ContractError {
        path: path.to_string(),
        message: message.into(),
    }
}

// --------------------------------------------------------------- decoding --

/// Field set helper: strict unknown-field detection over ordered pairs.
struct Fields<'a> {
    path: String,
    pairs: &'a [(String, CVal)],
    #[allow(dead_code)] // kept for debuggability of decode errors
    known: Vec<&'a str>,
}

impl<'a> Fields<'a> {
    fn new(
        path: &str,
        pairs: &'a [(String, CVal)],
        known: &[&'a str],
    ) -> Result<Self, ContractError> {
        for (k, _) in pairs {
            if !known.contains(&k.as_str()) {
                return Err(err(
                    &format!("{}.{}", path, k),
                    format!(
                        "unknown field {:?} (known fields: {}; misspelled fields are errors, not silently ignored)",
                        k,
                        known.join(", ")
                    ),
                ));
            }
        }
        Ok(Fields {
            path: path.to_string(),
            pairs,
            known: known.to_vec(),
        })
    }

    fn sub(&self, key: &str) -> Option<&'a CVal> {
        self.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn req(&self, key: &str) -> Result<&'a CVal, ContractError> {
        self.sub(key).ok_or_else(|| {
            err(
                &format!("{}.{}", self.path, key),
                "required field is missing",
            )
        })
    }

    fn path_of(&self, key: &str) -> String {
        format!("{}.{}", self.path, key)
    }
}

fn as_str<'v>(v: &'v CVal, path: &str) -> Result<&'v str, ContractError> {
    match v {
        CVal::Str(s) => Ok(s),
        other => Err(err(
            path,
            format!("expected string, got {}", other.type_name()),
        )),
    }
}

fn as_obj<'v>(v: &'v CVal, path: &str) -> Result<&'v [(String, CVal)], ContractError> {
    match v {
        CVal::Obj(pairs) => Ok(pairs),
        other => Err(err(
            path,
            format!("expected object, got {}", other.type_name()),
        )),
    }
}

fn as_list<'v>(v: &'v CVal, path: &str) -> Result<&'v [CVal], ContractError> {
    match v {
        CVal::List(items) => Ok(items),
        other => Err(err(
            path,
            format!("expected array, got {}", other.type_name()),
        )),
    }
}

fn req_str(fields: &Fields, key: &str) -> Result<String, ContractError> {
    as_str(fields.req(key)?, &fields.path_of(key)).map(|s| s.to_string())
}

fn opt_str(fields: &Fields, key: &str) -> Result<Option<String>, ContractError> {
    match fields.sub(key) {
        Some(v) => as_str(v, &fields.path_of(key)).map(|s| Some(s.to_string())),
        None => Ok(None),
    }
}

fn opt_str_list(fields: &Fields, key: &str) -> Result<Vec<String>, ContractError> {
    match fields.sub(key) {
        None => Ok(Vec::new()),
        Some(v) => {
            let path = fields.path_of(key);
            as_list(v, &path)?
                .iter()
                .enumerate()
                .map(|(i, item)| as_str(item, &format!("{}[{}]", path, i)).map(|s| s.to_string()))
                .collect()
        }
    }
}

fn valid_case_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.chars().next().unwrap().is_ascii_lowercase()
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

fn valid_check_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.chars().next().unwrap().is_ascii_lowercase()
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

/// Decode one `case { … }` node into a [`CaseSpec`] under admission limits.
pub fn decode_case(v: &CVal, limits: &Limits) -> Result<CaseSpec, ContractError> {
    let (name, props) = v.as_node().ok_or_else(|| {
        err(
            "case",
            format!("expected `case {{ … }}` node, got {}", v.type_name()),
        )
    })?;
    if name != "case" {
        return Err(err("case", format!("expected node `case`, got {:?}", name)));
    }
    let f = Fields::new(
        "case",
        props,
        &[
            "schema", "id", "tags", "links", "source", "files", "entry", "checks", "status",
        ],
    )?;

    let schema = req_str(&f, "schema")?;
    let version = if schema == CASE_SCHEMA {
        ContractVersion::V1
    } else if schema == CASE_SCHEMA_V2 {
        ContractVersion::V2
    } else {
        return Err(err(
            "case.schema",
            format!(
                "unsupported schema {:?} (this pilot implements {}, {})",
                schema, CASE_SCHEMA, CASE_SCHEMA_V2
            ),
        ));
    };

    let id = req_str(&f, "id")?;
    if !valid_case_id(&id) {
        return Err(err("case.id", "invalid case id (lowercase alphanumerics, '.', '_', '-'; must start lowercase letter/digit)"));
    }

    let status_skip = if let Some(status) = f.sub("status") {
        // Only explicit skip is supported; xfail and anything else is an error.
        let pairs = as_obj(status, "case.status")?;
        let sf = Fields::new("case.status", pairs, &["skip"])?;
        let skip = sf.req("skip")?;
        let pairs = as_obj(skip, "case.status.skip")?;
        let kf = Fields::new("case.status.skip", pairs, &["reason", "issue", "owner"])?;
        Some(SkipSpec {
            reason: req_str(&kf, "reason")?,
            issue: req_str(&kf, "issue")?,
            owner: opt_str(&kf, "owner")?,
        })
    } else {
        None
    };

    let source = decode_source(f.req("source")?, "case.source", version)?;
    let files = decode_files(f.sub("files"), "case.files")?;
    check_limit(
        "max_files_per_case",
        files.len() + 1,
        limits.max_files_per_case,
    )
    .map_err(|e| err("case.files", e.to_string()))?;

    let entry_pairs = as_obj(f.req("entry")?, "case.entry")?;
    let ef = Fields::new("case.entry", entry_pairs, &["mode"])?;
    let mode = req_str(&ef, "mode")?;
    let entry = match mode.as_str() {
        "script" => EntryMode::Script,
        "expression" => EntryMode::Expression,
        "program" => EntryMode::Program,
        "module" => EntryMode::Module,
        other => {
            return Err(err(
                "case.entry.mode",
                format!(
                    "unknown entry mode {:?} (supported: script, expression, program, module)",
                    other
                ),
            ))
        }
    };
    if entry == EntryMode::Module && files.is_empty() {
        return Err(err(
            "case.entry.mode",
            "module entry requires `files` (multi-file project)",
        ));
    }

    let checks_v = f.req("checks")?;
    let checks_list = as_list(checks_v, "case.checks")?;
    if checks_list.is_empty() {
        return Err(err("case.checks", "must declare at least one check"));
    }
    check_limit(
        "max_checks_per_case",
        checks_list.len(),
        limits.max_checks_per_case,
    )
    .map_err(|e| err("case.checks", e.to_string()))?;
    let mut checks = Vec::new();
    for (i, cv) in checks_list.iter().enumerate() {
        checks.push(decode_check(cv, &format!("case.checks[{}]", i), version)?);
    }
    // PLAN-788 R1 (F08): a cache declaration that promises behavioral
    // participation (fresh/read-only) on a combination that never touches
    // the unified store is an admission error — ac/check observes the real
    // frontend itself, and raw-HIR inputs bypass the driver pipeline. A
    // silent no-op would misreport the requested mode.
    for (i, c) in checks.iter().enumerate() {
        let participates = c.adapter == Adapter::Ac
            && matches!(c.action, Action::Build | Action::Run)
            && source.syntax == Syntax::Auto;
        if !participates {
            if let Some(mode) = c.options.ac_cache {
                if matches!(mode, CacheMode::Fresh | CacheMode::ReadOnly) {
                    return Err(err(
                        &format!("case.checks[{}].options.cache", i),
                        format!(
                            "cache {:?} requires auto-source ac/build|run store participation — ac/check observes the real frontend and raw-HIR inputs bypass the unified store (omit cache, or the default reuse)",
                            mode.as_str()
                        ),
                    ));
                }
            }
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    for c in &checks {
        if !seen.insert(c.id.clone()) {
            return Err(err(
                "case.checks",
                format!("duplicate check id {:?} (unique within the case)", c.id),
            ));
        }
    }
    // product_from must reference an existing check and stay acyclic
    // (edges may only point to EARLIER checks by declaration order).
    for (i, c) in checks.iter().enumerate() {
        if let Some(up) = &c.product_from {
            match checks.iter().position(|x| &x.id == up) {
                Some(pos) if pos < i => {}
                Some(_) => {
                    return Err(err(
                        &format!("case.checks[{}].product_from", i),
                        format!("must reference an earlier check (cycle), got {:?}", up),
                    ))
                }
                None => {
                    return Err(err(
                        &format!("case.checks[{}].product_from", i),
                        format!("references unknown check {:?}", up),
                    ))
                }
            }
        }
    }

    Ok(CaseSpec {
        schema,
        id,
        tags: opt_str_list(&f, "tags")?,
        links: opt_str_list(&f, "links")?,
        source,
        files,
        entry,
        checks,
        status_skip,
    })
}

/// Extract the payload discriminator (exactly one of block/file/text) from
/// an ALREADY field-validated object.
fn decode_payload_from(f: &Fields) -> Result<PayloadSpec, ContractError> {
    let given: Vec<&str> = ["block", "file", "text"]
        .iter()
        .filter(|k| f.sub(k).is_some())
        .copied()
        .collect();
    if given.len() != 1 {
        return Err(err(
            &f.path,
            format!(
                "exactly one of block/file/text is required, got {:?}",
                given
            ),
        ));
    }
    if let Some(b) = f.sub("block") {
        let anchor = as_str(b, &f.path_of("block"))?.to_string();
        if anchor.is_empty()
            || !anchor
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(err(
                &f.path_of("block"),
                "invalid anchor (upstream ^anchor charset: [A-Za-z0-9_-]+)",
            ));
        }
        return Ok(PayloadSpec::Block(anchor));
    }
    if let Some(t) = f.sub("file") {
        let file = as_str(t, &f.path_of("file"))?.to_string();
        validate_virtual_path(&file).map_err(|e| err(&f.path_of("file"), e.to_string()))?;
        return Ok(PayloadSpec::File(file));
    }
    let text = match f.sub("text") {
        Some(v) => as_str(v, &f.path_of("text"))?.to_string(),
        None => return Err(err(&f.path, "text payload missing")),
    };
    Ok(PayloadSpec::Text(text))
}

fn decode_source(
    v: &CVal,
    path: &str,
    version: ContractVersion,
) -> Result<SourceSpec, ContractError> {
    let pairs = as_obj(v, path)?;
    let f = Fields::new(path, pairs, &["block", "file", "text", "syntax", "path"])?;
    let payload = decode_payload_from(&f)?;
    let syntax = match req_str(&f, "syntax")?.as_str() {
        "auto" => Syntax::Auto,
        "auto-script" => Syntax::AutoScript,
        "hir-atom" if version == ContractVersion::V2 => Syntax::HirAtom,
        "atom-data" if version == ContractVersion::V2 => Syntax::AtomData,
        other => {
            return Err(err(
                &f.path_of("syntax"),
                format!(
                    "unknown syntax {:?} (supported: {})",
                    other,
                    match version {
                        ContractVersion::V1 => "auto, auto-script",
                        ContractVersion::V2 => "auto, auto-script, hir-atom, atom-data",
                    }
                ),
            ))
        }
    };
    let path_v = req_str(&f, "path")?;
    validate_virtual_path(&path_v).map_err(|e| err(&f.path_of("path"), e.to_string()))?;
    let ext = path_v.rsplit('.').next().unwrap_or("");
    if ext != syntax.extension() {
        return Err(err(
            &f.path_of("path"),
            format!(
                "path extension {:?} does not match syntax {:?} (expected .{})",
                ext,
                syntax.as_str(),
                syntax.extension()
            ),
        ));
    }
    Ok(SourceSpec {
        payload,
        syntax,
        path: path_v,
    })
}

fn decode_files(v: Option<&CVal>, path: &str) -> Result<Vec<FileEntry>, ContractError> {
    let list = match v {
        None => return Ok(Vec::new()),
        Some(v) => as_list(v, path)?,
    };
    let mut out = Vec::new();
    for (i, item) in list.iter().enumerate() {
        let ipath = format!("{}[{}]", path, i);
        let pairs = as_obj(item, &ipath)?;
        let f = Fields::new(&ipath, pairs, &["block", "file", "text", "path"])?;
        let payload = decode_payload_from(&f)?;
        let fpath = req_str(&f, "path")?;
        validate_virtual_path(&fpath).map_err(|e| err(&f.path_of("path"), e.to_string()))?;
        out.push(FileEntry {
            path: fpath,
            payload,
        });
    }
    Ok(out)
}

fn decode_check(
    v: &CVal,
    path: &str,
    version: ContractVersion,
) -> Result<CheckSpec, ContractError> {
    let pairs = as_obj(v, path)?;
    let f = Fields::new(
        path,
        pairs,
        &[
            "id",
            "adapter",
            "action",
            "tier",
            "options",
            "expect",
            "product_from",
            "skip",
        ],
    )?;

    let id = req_str(&f, "id")?;
    if !valid_check_id(&id) {
        return Err(err(
            &f.path_of("id"),
            "invalid check id (lowercase alphanumerics, '.', '_', '-')",
        ));
    }

    let adapter = match req_str(&f, "adapter")?.as_str() {
        "avm" => Adapter::Avm,
        "a2r" => Adapter::A2r,
        "ac" if version == ContractVersion::V2 => Adapter::Ac,
        "syntax" if version == ContractVersion::V2 => Adapter::Syntax,
        "probe" if version == ContractVersion::V2 => Adapter::Probe,
        other => {
            return Err(err(&f.path_of("adapter"), format!("adapter {:?} is not available ({} implements avm, a2r{}; native/ac/acc and others belong to Design 35 phase B)", other, if version == ContractVersion::V2 { "v2" } else { "v1" }, if version == ContractVersion::V2 { ", ac, syntax" } else { "" })))
        }
    };
    let action = match req_str(&f, "action")?.as_str() {
        "run" => Action::Run,
        "emit" => Action::Emit,
        "check" if version == ContractVersion::V2 => Action::Check,
        "build" if version == ContractVersion::V2 => Action::Build,
        "parse" if version == ContractVersion::V2 => Action::Parse,
        "edit" if version == ContractVersion::V2 => Action::Edit,
        other => {
            return Err(err(
                &f.path_of("action"),
                format!(
                    "action {:?} is not available (supported: run, emit{})",
                    other,
                    if version == ContractVersion::V2 {
                        ", check, build, parse, edit"
                    } else {
                        ""
                    }
                ),
            ))
        }
    };
    match (adapter, action) {
        (Adapter::Avm, Action::Run)
        | (Adapter::A2r, Action::Emit)
        | (Adapter::Ac, Action::Check)
        | (Adapter::Ac, Action::Build)
        | (Adapter::Ac, Action::Run)
        | (Adapter::Syntax, Action::Parse)
        | (Adapter::Syntax, Action::Edit)
        | (Adapter::Probe, Action::Run) => {}
        (a, act) => {
            return Err(err(
                &f.path_of("action"),
                format!(
                    "illegal combination {}/{} (supported: avm/run, a2r/emit, ac/check, ac/build, ac/run, syntax/parse, syntax/edit, probe/run)",
                    a.as_str(),
                    act.as_str()
                ),
            ))
        }
    }

    let tier = opt_str(&f, "tier")?;

    let options = match f.sub("options") {
        None => Options::default(),
        Some(ov) => decode_options(ov, adapter, action, &f.path_of("options"))?,
    };

    let product_from = opt_str(&f, "product_from")?;

    let skip = match f.sub("skip") {
        None => None,
        Some(sv) => {
            let pairs = as_obj(sv, &f.path_of("skip"))?;
            let sf = Fields::new(&f.path_of("skip"), pairs, &["reason", "issue", "owner"])?;
            Some(SkipSpec {
                reason: req_str(&sf, "reason")?,
                issue: req_str(&sf, "issue")?,
                owner: opt_str(&sf, "owner")?,
            })
        }
    };

    let expect = decode_expect(f.req("expect")?, &f.path_of("expect"), adapter, action)?;

    Ok(CheckSpec {
        id,
        adapter,
        action,
        tier,
        options,
        expect,
        product_from,
        skip,
    })
}

fn decode_options(
    v: &CVal,
    adapter: Adapter,
    action: Action,
    path: &str,
) -> Result<Options, ContractError> {
    let pairs = as_obj(v, path)?;
    let known: &[&str] = match adapter {
        Adapter::Avm => &[], // avm/run has no options (v1 and v2)
        Adapter::A2r => &["module_name"],
        Adapter::Ac => &[
            "profile",
            "target",
            "entry",
            "cache",
            "budget",
            "capabilities",
            "support_lib",
        ],
        // T-06 frozen syntax options: the grammar selector and the ordered
        // single-edit sequence (coordinates relative to the CURRENT
        // revision at each step — sequential application).
        Adapter::Syntax => &["grammar", "edits"],
        // PLAN-796 T-06 frozen probe options: the registered recipe id,
        // the declarative step sequence, and the sema/query-v1 binding.
        Adapter::Probe => &["recipe", "steps", "sema", "abi", "av"],
    };
    let f = Fields::new(path, pairs, known)?;
    let module_name = opt_str(&f, "module_name")?;
    if let Some(name) = &module_name {
        // F-05: module_name becomes a product filename (`<name>.rs`) inside
        // the run scratch — it must be a single safe path component.
        let ok = !name.is_empty()
            && name.len() <= 64
            && !name.contains(['/', '\\', ':', '\0'])
            && name != "."
            && name != ".."
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'));
        if !ok {
            return Err(err(
                &f.path_of("module_name"),
                "module_name must be a single path component ([A-Za-z0-9_-], non-empty, <=64 chars) — it names an emitted artifact",
            ));
        }
    }

    let mut ac_profile = None;
    let mut ac_target = None;
    let mut ac_entry = None;
    let mut ac_cache = None;
    let mut ac_budget = None;
    let mut ac_capabilities = None;
    let mut ac_support_lib = None;
    if adapter == Adapter::Ac {
        // Only DECLARED values are stored — implicit defaults resolve at
        // execution via the `effective_*` accessors, so a suite default can
        // still fill an undeclared case field through apply_defaults.
        ac_profile = match f.sub("profile") {
            None => None,
            Some(CVal::Str(s)) if s == "core" => Some(AcProfile::Core),
            Some(CVal::Str(s)) if s == "scalar" => Some(AcProfile::Scalar),
            Some(CVal::Str(s)) => {
                return Err(err(
                    &f.path_of("profile"),
                    format!(
                        "profile {:?} is not imported in this batch (supported: core, scalar; text/record/text-record remain guarded by the legacy gates)",
                        s
                    ),
                ))
            }
            Some(other) => {
                return Err(err(
                    &f.path_of("profile"),
                    format!("expected string, got {}", other.type_name()),
                ))
            }
        };
        ac_target = match f.sub("target") {
            None => None,
            Some(CVal::Str(s)) if s == AC_TARGET => Some(s.clone()),
            Some(CVal::Str(s)) => {
                return Err(err(
                    &f.path_of("target"),
                    format!(
                        "target {:?} is not supported (this batch imports {} only)",
                        s, AC_TARGET
                    ),
                ))
            }
            Some(other) => {
                return Err(err(
                    &f.path_of("target"),
                    format!("expected string, got {}", other.type_name()),
                ))
            }
        };
        ac_entry = match f.sub("entry") {
            None => None,
            Some(CVal::Str(s)) => {
                let ok = !s.is_empty()
                    && s.len() <= 64
                    && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                if !ok {
                    return Err(err(
                        &f.path_of("entry"),
                        "entry must be a function-name identifier ([A-Za-z0-9_], non-empty, <=64 chars)",
                    ));
                }
                Some(s.clone())
            }
            Some(other) => {
                return Err(err(
                    &f.path_of("entry"),
                    format!("expected string, got {}", other.type_name()),
                ))
            }
        };
        ac_cache = match f.sub("cache") {
            None => None,
            Some(CVal::Str(s)) if s == "fresh" => Some(CacheMode::Fresh),
            Some(CVal::Str(s)) if s == "reuse" => Some(CacheMode::Reuse),
            Some(CVal::Str(s)) if s == "read-only" => Some(CacheMode::ReadOnly),
            Some(CVal::Str(s)) => {
                return Err(err(
                    &f.path_of("cache"),
                    format!(
                        "unknown cache mode {:?} (supported: fresh, reuse, read-only)",
                        s
                    ),
                ))
            }
            Some(other) => {
                return Err(err(
                    &f.path_of("cache"),
                    format!("expected string, got {}", other.type_name()),
                ))
            }
        };
        ac_capabilities = match f.sub("capabilities") {
            None => None,
            Some(v) => {
                let path = f.path_of("capabilities");
                let list = as_list(v, &path)?;
                if list.len() > 16 {
                    return Err(err(
                        &path,
                        format!("at most 16 capabilities, got {}", list.len()),
                    ));
                }
                let mut caps = Vec::new();
                for (i, item) in list.iter().enumerate() {
                    let s = as_str(item, &format!("{}[{}]", path, i))?;
                    let ok = !s.is_empty()
                        && s.len() <= 64
                        && s.chars().all(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-')
                        });
                    if !ok {
                        return Err(err(
                            &format!("{}[{}]", path, i),
                            "capability names are lowercase dotted ids (<=64 chars)",
                        ));
                    }
                    caps.push(s.to_string());
                }
                Some(caps)
            }
        };
        ac_support_lib = match f.sub("support_lib") {
            None => None,
            Some(CVal::Str(s)) if s == "trace-marks" => Some(s.clone()),
            Some(CVal::Str(s)) => {
                return Err(err(
                    &f.path_of("support_lib"),
                    format!(
                        "unknown support library {:?} (this batch names only \"trace-marks\" — the PLAN-741 trace-marks runtime)",
                        s
                    ),
                ))
            }
            Some(other) => {
                return Err(err(
                    &f.path_of("support_lib"),
                    format!("expected string, got {}", other.type_name()),
                ))
            }
        };
        ac_budget = match f.sub("budget") {
            None => None,
            Some(bv) => {
                let bp = f.path_of("budget");
                let bpairs = as_obj(bv, &bp)?;
                let bf = Fields::new(&bp, bpairs, &["build_ms", "run_ms", "total_ms"])?;
                let read_ms = |key: &str, cap: u64| -> Result<Option<u64>, ContractError> {
                    match bf.sub(key) {
                        None => Ok(None),
                        Some(CVal::Int(n)) if *n > 0 && (*n as u64) <= cap => Ok(Some(*n as u64)),
                        Some(CVal::Int(n)) => Err(err(
                            &bf.path_of(key),
                            format!(
                                "{} must be in (0, {}] ms — frozen hard cap (got {})",
                                key, cap, n
                            ),
                        )),
                        Some(other) => Err(err(
                            &bf.path_of(key),
                            format!("expected integer, got {}", other.type_name()),
                        )),
                    }
                };
                Some(BudgetSpec {
                    build_ms: read_ms("build_ms", BUDGET_HARD_CAP_BUILD_MS)?,
                    run_ms: read_ms("run_ms", BUDGET_HARD_CAP_RUN_MS)?,
                    total_ms: read_ms("total_ms", BUDGET_HARD_CAP_TOTAL_MS)?,
                })
            }
        };
    }

    let (syntax_grammar, syntax_edits) = if adapter == Adapter::Syntax {
        let g = match f.sub("grammar") {
            None => Some("auto-core".to_string()),
            Some(CVal::Str(s)) if s == "auto-core" || s == "interface-mini-v1" => Some(s.clone()),
            Some(CVal::Str(s)) => {
                return Err(err(
                    &f.path_of("grammar"),
                    format!(
                        "unknown grammar {:?} (supported: auto-core, interface-mini-v1)",
                        s
                    ),
                ))
            }
            Some(other) => {
                return Err(err(
                    &f.path_of("grammar"),
                    format!("expected string, got {}", other.type_name()),
                ))
            }
        };
        let e = match f.sub("edits") {
            None => None,
            Some(v) => {
                let list = as_list(v, &f.path_of("edits"))?;
                if list.is_empty() || list.len() > 64 {
                    return Err(err(
                        &f.path_of("edits"),
                        format!("edit sequence must be 1..=64 steps, got {}", list.len()),
                    ));
                }
                let mut out = Vec::new();
                for (i, item) in list.iter().enumerate() {
                    let ipath = format!("{}.edits[{}]", f.path, i);
                    let pairs = as_obj(item, &ipath)?;
                    let ef = Fields::new(&ipath, pairs, &["start", "end", "insert"])?;
                    let read_u64 = |key: &str| -> Result<u64, ContractError> {
                        match ef.sub(key) {
                            Some(CVal::Int(n)) if *n >= 0 => Ok(*n as u64),
                            Some(other) => Err(err(
                                &ef.path_of(key),
                                format!("expected non-negative integer, got {}", other.type_name()),
                            )),
                            None => Err(err(&ef.path_of(key), "required field is missing")),
                        }
                    };
                    let start = read_u64("start")?;
                    let end = read_u64("end")?;
                    if end < start {
                        return Err(err(
                            &ef.path_of("end"),
                            "edit span is half-open [start,end) — end must be >= start",
                        ));
                    }
                    let insert = req_str(&ef, "insert")?;
                    if insert.len() > 64 * 1024 {
                        return Err(err(
                            &ef.path_of("insert"),
                            format!(
                                "edit[{}] insert {} bytes exceeds the 64KiB step cap",
                                i,
                                insert.len()
                            ),
                        ));
                    }
                    out.push(SyntaxEdit { start, end, insert });
                }
                Some(out)
            }
        };
        if action == Action::Edit && e.is_none() {
            return Err(err(
                &f.path_of("edits"),
                "syntax/edit requires a non-empty edit sequence",
            ));
        }
        (g, e)
    } else {
        (None, None)
    };

    let (probe_recipe, probe_steps, probe_sema, probe_abi, probe_av) = if adapter == Adapter::Probe
    {
        let recipe = req_str(&f, "recipe")?;
        if !REGISTERED_PROBE_RECIPES.contains(&recipe.as_str()) {
            return Err(err(
                &f.path_of("recipe"),
                format!(
                    "recipe {:?} is not registered (registered: {}; a case can never name an executable)",
                    recipe,
                    REGISTERED_PROBE_RECIPES.join(", ")
                ),
            ));
        }
        let steps = match f.sub("steps") {
            None => {
                return Err(err(
                    &f.path_of("steps"),
                    "probe/run requires a non-empty step sequence",
                ))
            }
            Some(v) => {
                let list = as_list(v, &f.path_of("steps"))?;
                if list.is_empty() || list.len() > PROBE_MAX_STEPS {
                    return Err(err(
                        &f.path_of("steps"),
                        format!(
                            "probe step sequence must be 1..={} steps, got {}",
                            PROBE_MAX_STEPS,
                            list.len()
                        ),
                    ));
                }
                let mut out = Vec::new();
                for (i, item) in list.iter().enumerate() {
                    let ipath = format!("{}.steps[{}]", f.path, i);
                    let pairs = as_obj(item, &ipath)?;
                    let sf = Fields::new(&ipath, pairs, &["op", "from", "to"])?;
                    let op = req_str(&sf, "op")?;
                    // PLAN-798: `run` joins the known op vocabulary
                    // (ac-av's only op); recipes never borrow each other's
                    // ops, and genuinely unknown ops keep the original
                    // unknown-op error.
                    let known_op =
                        matches!(op.as_str(), "query" | "edit-source" | "restart" | "run");
                    if !known_op {
                        return Err(err(
                            &sf.path_of("op"),
                            format!("unknown probe op {:?} (supported: query, edit-source, restart, run)", op),
                        ));
                    }
                    let allowed = match recipe.as_str() {
                        // T-07 edit-session: runs + discriminating edits.
                        "ac-av/core-run-v1" => matches!(op.as_str(), "run" | "edit-source"),
                        _ => op != "run",
                    };
                    if !allowed {
                        return Err(err(
                            &sf.path_of("op"),
                            format!(
                                "probe op {:?} is not valid for recipe {:?} (recipe-specific step vocabulary)",
                                op, recipe
                            ),
                        ));
                    }
                    let step = match op.as_str() {
                        "query" => ProbeStep::Query,
                        "run" => ProbeStep::Run,
                        "edit-source" => {
                            let from = req_str(&sf, "from")?;
                            let to = req_str(&sf, "to")?;
                            if from.is_empty() || from == to {
                                return Err(err(
                                    &sf.path_of("from"),
                                    "edit-source needs a non-empty `from` distinct from `to` (a discriminating edit oracle)",
                                ));
                            }
                            if from.len() > 64 * 1024 || to.len() > 64 * 1024 {
                                return Err(err(
                                    &sf.path_of("to"),
                                    "edit-source literals exceed the 64KiB step cap",
                                ));
                            }
                            ProbeStep::EditSource { from, to }
                        }
                        "restart" => ProbeStep::Restart,
                        other => {
                            return Err(err(
                                &sf.path_of("op"),
                                format!(
                                "unknown probe op {:?} (supported: query, edit-source, restart)",
                                other
                            ),
                            ))
                        }
                    };
                    out.push(step);
                }
                if recipe == "ac-av/core-run-v1" {
                    // PLAN-798 T-07 edit-session shape (F798-R3-07): runs
                    // plus discriminating edit-source steps; a sequence
                    // may LEAD with edits (the measured formal window —
                    // the request materializes the source, applies leading
                    // edits, then runs) but must END with the judged run,
                    // and leading edits require an explicit session only
                    // for cross-call warmth (single-call shapes are
                    // unchanged).
                    if !matches!(out.last(), Some(ProbeStep::Run)) {
                        return Err(err(
                            &f.path_of("steps"),
                            "ac-av/core-run-v1 step sequences must end with a run step (the judged step)",
                        ));
                    }
                    if out.is_empty() {
                        return Err(err(&f.path_of("steps"), "steps must not be empty"));
                    }
                }
                out
            }
        };
        // The binding object is RECIPE-SPECIFIC: sema/query-v1 takes
        // `sema`, abi/interop-v1 takes `abi`; declaring the wrong one
        // (or none) is an admission error, never a parsed-then-ignored
        // field.
        let sema = match f.sub("sema") {
            None => {
                if recipe == "sema/query-v1" {
                    return Err(err(
                        &f.path_of("sema"),
                        "sema/query-v1 requires a sema binding",
                    ));
                }
                None
            }
            Some(v) => {
                if recipe != "sema/query-v1" {
                    return Err(err(
                        &f.path_of("sema"),
                        format!(
                            "the sema binding belongs to sema/query-v1; it is not valid for recipe {:?}",
                            recipe
                        ),
                    ));
                }
                let spath = f.path_of("sema");
                let pairs = as_obj(v, &spath)?;
                let sf = Fields::new(&spath, pairs, &["package", "module", "kind", "fn"])?;
                let kind = req_str(&sf, "kind")?;
                if kind != "infer-body" {
                    return Err(err(
                        &sf.path_of("kind"),
                        format!("unknown sema query kind {:?} (v1 freezes infer-body)", kind),
                    ));
                }
                Some(ProbeSemaSpec {
                    package: req_str(&sf, "package")?,
                    module: req_str(&sf, "module")?,
                    kind,
                    fn_name: req_str(&sf, "fn")?,
                })
            }
        };
        let abi = match f.sub("abi") {
            None => {
                if recipe == "abi/interop-v1" {
                    return Err(err(
                        &f.path_of("abi"),
                        "abi/interop-v1 requires an abi binding",
                    ));
                }
                None
            }
            Some(v) => {
                if recipe != "abi/interop-v1" {
                    return Err(err(
                        &f.path_of("abi"),
                        format!(
                            "the abi binding belongs to abi/interop-v1, not {:?}",
                            recipe
                        ),
                    ));
                }
                let apath = f.path_of("abi");
                let pairs = as_obj(v, &apath)?;
                let af = Fields::new(&apath, pairs, &["leg"])?;
                let leg = req_str(&af, "leg")?;
                if !matches!(leg.as_str(), "a" | "b" | "c") {
                    return Err(err(
                        &af.path_of("leg"),
                        format!("unknown ABI leg {:?} (v1 freezes a/b/c)", leg),
                    ));
                }
                Some(ProbeAbiSpec { leg })
            }
        };
        let av = match f.sub("av") {
            None => {
                if recipe == "ac-av/core-run-v1" {
                    return Err(err(
                        &f.path_of("av"),
                        "ac-av/core-run-v1 requires an av binding",
                    ));
                }
                None
            }
            Some(v) => {
                if recipe != "ac-av/core-run-v1" {
                    return Err(err(
                        &f.path_of("av"),
                        format!(
                            "av binding is not valid for recipe {:?} (recipe-specific bindings)",
                            recipe
                        ),
                    ));
                }
                let apath = f.path_of("av");
                let pairs = as_obj(v, &apath)?;
                let sf = Fields::new(&apath, pairs, &["entry", "caps", "session"])?;
                let entry = req_str(&sf, "entry")?;
                let session = match sf.sub("session") {
                    None => None,
                    Some(v) => {
                        let sid = as_str(v, &sf.path_of("session"))?.to_string();
                        if sid.is_empty()
                            || sid.len() > 64
                            || !sid
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                        {
                            return Err(err(
                                &sf.path_of("session"),
                                "av session id must be 1-64 chars of [A-Za-z0-9_-]",
                            ));
                        }
                        Some(sid)
                    }
                };
                let caps = match sf.sub("caps") {
                    None => Vec::new(),
                    Some(v) => {
                        let list = as_list(v, &sf.path_of("caps"))?;
                        let mut out = Vec::new();
                        for (i, item) in list.iter().enumerate() {
                            let cpath = format!("{}.caps[{}]", apath, i);
                            out.push(as_str(item, &cpath)?.to_string());
                        }
                        out
                    }
                };
                Some(ProbeAvSpec {
                    entry,
                    caps,
                    session,
                })
            }
        };
        (Some(recipe), Some(steps), sema, abi, av)
    } else {
        (None, None, None, None, None)
    };

    Ok(Options {
        module_name,
        ac_profile,
        ac_target,
        ac_entry,
        ac_cache,
        ac_budget,
        ac_capabilities,
        ac_support_lib,
        syntax_grammar,
        syntax_edits,
        probe_recipe,
        probe_steps,
        probe_sema,
        probe_abi,
        probe_av,
    })
}

/// One bounded syntax edit step (T-06): coordinates are relative to the
/// CURRENT revision at this step (sequential application).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxEdit {
    pub start: u64,
    pub end: u64,
    pub insert: String,
}

/// Registered probe recipes (PLAN-796 T-06 freeze). A case may only name
/// one of these — never an executable, path or shell. 797 appends its ABI
/// recipe here under the same contract (single writer per window).
pub const REGISTERED_PROBE_RECIPES: &[&str] =
    &["sema/query-v1", "abi/interop-v1", "ac-av/core-run-v1"];
/// Probe step-count cap (frozen with the seam; mirrors the syntax edit cap).
pub const PROBE_MAX_STEPS: usize = 64;
/// One declarative probe step. `EditSource` replaces a LITERAL substring
/// that must match exactly once in the current source (a discriminating
/// edit oracle — ambiguous or absent matches are worker errors). `Restart`
/// ends the worker session; the NEXT step runs in a fresh process against
/// the same store root (the real cross-process persistence path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeStep {
    /// Run the recipe's full query flow (lookup → miss? compute → publish)
    /// and report the structured result for parent-side judging.
    Query,
    EditSource {
        from: String,
        to: String,
    },
    Restart,
    /// PLAN-798 ac-av/core-run-v1: run the case source through the full
    /// AV pipeline once and report the observed outcome (value / trap /
    /// diagnostic + counters) for parent-side judging.
    Run,
}

/// ac-av/core-run-v1 binding (PLAN-798 T-07): the AV entry function and
/// optional capability list — declarative Atom parameters only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeAvSpec {
    pub entry: String,
    pub caps: Vec<String>,
    /// Optional cross-request SESSION id (F798-R3-07 formal window
    /// primitive): requests sharing a session id share the worker's
    /// persistent per-session store across calls; absent = a private
    /// per-request store (cleaned up by the worker).
    pub session: Option<String>,
}

/// sema/query-v1 binding: which semantic query the recipe runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeSemaSpec {
    pub package: String,
    pub module: String,
    /// Query kind; v1 freezes "infer-body" as the only kind.
    pub kind: String,
    pub fn_name: String,
}

/// abi/interop-v1 binding (PLAN-797 T-06): which interop leg the frozen
/// ABI pipeline drives. v1 freezes the three legs a/b/c.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeAbiSpec {
    /// "a" (AutoABI caller -> AutoABI callee), "b" (AutoABI caller ->
    /// C callee), "c" (C caller -> AutoABI callee).
    pub leg: String,
}

/// Per-query-step expectation (judged parent-side; order = the query steps).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProbeStepExpect {
    /// Required cache outcome: "hit" | "miss" | "stale" | "published".
    pub cache: Option<String>,
    /// Required compute-call count (0 on a clean hit — the no-compute
    /// control is a declarable oracle).
    pub compute_calls: Option<u32>,
    /// Sidecar JSON with the expected bindings (name/ty list).
    pub bindings_file: Option<String>,
    /// Stage names the step trace must include.
    pub stages_include: Vec<String>,
}

/// Logical artifact paths syntax checks may byte-compare against (T-06):
/// the lossless reconstruction, the outline projection and the canonical
/// CST form — all of the FINAL revision.
pub const SYNTAX_ARTIFACT_PATHS: &[&str] = &["reconstructed.at", "outline.json", "cst.canonical"];

/// Per-adapter observable-channel support table (Plan 773 F-02 for v1;
/// PLAN-788 extends it for `ac`). A channel declared for an adapter that
/// cannot observe it is an ADMISSION error — parsed-then-ignored
/// expectations are forbidden. Action-level narrowing (exit/stdout/stderr
/// are ac/run-only; ac artifacts are the canonical HIR only) is enforced in
/// [`decode_expect`]. `syntax` exposes no channels until T-06 freezes its
/// projection face.
pub fn channel_supported(adapter: Adapter, channel: &str) -> bool {
    match (adapter, channel) {
        (Adapter::Avm, "result") => true,
        (Adapter::Avm, "stdout") => true,
        (Adapter::Avm, "stderr") => true,
        (Adapter::A2r, "artifacts") => true,
        (Adapter::Ac, "exit") => true,
        (Adapter::Ac, "stdout") => true,
        (Adapter::Ac, "stderr") => true,
        (Adapter::Ac, "artifacts") => true,
        (Adapter::Ac, "perf") => true,
        (Adapter::Syntax, "artifacts") => true,
        (Adapter::Syntax, "perf") => true,
        // PLAN-796: the probe adapter observes its STRUCTURED step face
        // only (cache outcome / compute counter / bindings oracle /
        // stage trace). It never observes stdout/stderr/result channels.
        (Adapter::Probe, "probe") => true,
        // PLAN-798 T-07: value-reporting probe recipes (ac-av/core-run-v1
        // reports the AV Success value) observe the generic `exit` face.
        (Adapter::Probe, "exit") => true,
        _ => false,
    }
}

/// Logical artifact path ac checks may byte-compare against: the canonical
/// HIR emitted by the real frontend. obj/exe identity is manifest-bound
/// (product_from handoff / run observation), never a byte golden.
pub const AC_HIR_ARTIFACT_PATH: &str = "hir.atom";

fn decode_expect(
    v: &CVal,
    path: &str,
    adapter: Adapter,
    action: Action,
) -> Result<ExpectSpec, ContractError> {
    let v2_channels = adapter == Adapter::Ac;
    let probe_channels = adapter == Adapter::Probe;
    let pairs = as_obj(v, path)?;
    let known: &[&str] = if v2_channels {
        &[
            "completion",
            "result",
            "stdout",
            "stderr",
            "artifacts",
            "exit",
            "perf",
        ]
    } else if probe_channels {
        // PLAN-798 T-07: + exit (value-reporting recipes).
        &["completion", "probe", "exit"]
    } else {
        &["completion", "result", "stdout", "stderr", "artifacts"]
    };
    let f = Fields::new(path, pairs, known)?;

    // Frozen support table: every declared channel must be observable by
    // this adapter, else the check never loads (F-02: unsupported fields
    // may not be skipped after decode).
    let channels: &[&str] = if v2_channels {
        &["result", "stdout", "stderr", "artifacts", "exit", "perf"]
    } else if probe_channels {
        &["probe", "exit"]
    } else {
        &["result", "stdout", "stderr", "artifacts"]
    };
    for channel in channels {
        if f.sub(channel).is_some() && !channel_supported(adapter, channel) {
            let adapter_channels = match adapter {
                Adapter::Avm => "result, stdout, stderr",
                Adapter::A2r => "artifacts",
                Adapter::Ac => "exit, stdout, stderr, artifacts (hir.atom), perf",
                Adapter::Syntax => "none until T-06 (787 grammar projection face)",
                Adapter::Probe => "probe (structured step face)",
            };
            return Err(err(
                &f.path_of(channel),
                format!(
                    "`{}` is not an observable channel for {}/{} in this pilot (supported by {}: {})",
                    channel,
                    adapter.as_str(),
                    action.as_str(),
                    channel,
                    adapter_channels
                ),
            ));
        }
    }

    let completion = match f.req("completion")? {
        CVal::Str(s) if s == "success" => Completion::Success,
        CVal::Obj(cp) => {
            let cf = Fields::new(&f.path_of("completion"), cp, &["stage", "code", "message"])?;
            let stage_str = req_str(&cf, "stage")?;
            let stage = match Stage::decode(&stage_str) {
                Some(s) if Stage::supported_for(adapter).contains(&s.as_str()) => s,
                _ => {
                    return Err(err(
                        &format!("{}.stage", f.path_of("completion")),
                        format!(
                            "unknown or unsupported stage {:?} for {} (supported: {})",
                            stage_str,
                            adapter.as_str(),
                            Stage::supported_for(adapter).join(", ")
                        ),
                    ))
                }
            };
            let code = opt_str(&cf, "code")?;
            if code.as_deref() == Some("") {
                return Err(err(
                    &format!("{}.code", f.path_of("completion")),
                    "empty `code` identifies nothing — declare a real variant/diagnostic code or omit the field",
                ));
            }
            let message = match cf.sub("message") {
                None => None,
                Some(mv) => {
                    let mp = format!("{}.message", f.path_of("completion"));
                    let mpairs = as_obj(mv, &mp)?;
                    let mf = Fields::new(&mp, mpairs, &["exact", "contains"])?;
                    let has_exact = mf.sub("exact").is_some();
                    let has_contains = mf.sub("contains").is_some();
                    if has_exact == has_contains {
                        return Err(err(&mp, "exactly one of exact/contains is required"));
                    }
                    if has_exact {
                        let s = req_str(&mf, "exact")?;
                        if s.is_empty() {
                            return Err(err(&mp, "exact: \"\" matches every message — a strong oracle needs discriminating text"));
                        }
                        Some(MessageMatcher::Exact(s))
                    } else {
                        let s = req_str(&mf, "contains")?;
                        if s.is_empty() {
                            return Err(err(&mp, "contains: \"\" matches every message — a strong oracle needs discriminating text"));
                        }
                        Some(MessageMatcher::Contains(s))
                    }
                }
            };
            if code.is_none() && message.is_none() {
                return Err(err(
                    &f.path_of("completion"),
                    "error completion needs a stable identification: `code` and/or `message` (a bare stage is a weak oracle and is not accepted)",
                ));
            }
            Completion::Error {
                stage,
                code,
                message,
            }
        }
        other => {
            return Err(err(
                &f.path_of("completion"),
                format!(
                    "expected \"success\" or an error object, got {}",
                    other.type_name()
                ),
            ))
        }
    };

    let result = match f.sub("result") {
        None => None,
        Some(rv) => {
            let rp = f.path_of("result");
            let pairs = as_obj(rv, &rp)?;
            let rf = Fields::new(&rp, pairs, &["format", "text"])?;
            let format = req_str(&rf, "format")?;
            if format != "legacy-display" {
                return Err(err(
                    &format!("{}.format", rp),
                    format!("unknown result format {:?} (v1: legacy-display)", format),
                ));
            }
            if matches!(completion, Completion::Error { .. }) {
                return Err(err(
                    &rp,
                    "`result` expectations require completion \"success\"",
                ));
            }
            Some(ResultExpect {
                format,
                text: req_str(&rf, "text")?,
            })
        }
    };

    let decode_stream = |key: &str| -> Result<Option<StreamExpect>, ContractError> {
        match f.sub(key) {
            None => Ok(None),
            Some(sv) => {
                let sp = f.path_of(key);
                if adapter == Adapter::Ac && action != Action::Run {
                    return Err(err(
                        &sp,
                        format!(
                            "`{}` is an ac/run observation — ac/{} exposes no process streams",
                            key,
                            action.as_str()
                        ),
                    ));
                }
                let pairs = as_obj(sv, &sp)?;
                let sf = Fields::new(&sp, pairs, &["text", "compare"])?;
                // F-02: streams are success-completion observations; an
                // error completion exposes no streams to compare against,
                // so declaring one there is an admission error instead of a
                // silently-ignored assertion.
                if matches!(completion, Completion::Error { .. }) {
                    return Err(err(
                        &sp,
                        format!("`{}` expectations require completion \"success\" (error completions expose no streams in this pilot)", key),
                    ));
                }
                let compare = decode_compare(sf.sub("compare"), &format!("{}.compare", sp))?;
                Ok(Some(StreamExpect {
                    text: req_str(&sf, "text")?,
                    compare,
                }))
            }
        }
    };
    let stdout = decode_stream("stdout")?;
    let stderr = decode_stream("stderr")?;

    let exit = match f.sub("exit") {
        None => None,
        Some(CVal::Int(n)) if i32::try_from(*n).is_ok() => {
            // PLAN-798 T-07: value-reporting probe recipes (ac-av/core-run-v1
            // reports the AV Success value) also observe `exit`; the probe
            // adapter admits it alongside ac/run.
            let exit_observer =
                (adapter == Adapter::Ac && action == Action::Run) || adapter == Adapter::Probe;
            if !exit_observer {
                return Err(err(
                    &f.path_of("exit"),
                    "`exit` is an ac/run observation — only the native run exposes an exit code",
                ));
            }
            if matches!(completion, Completion::Error { .. }) {
                return Err(err(
                    &f.path_of("exit"),
                    "`exit` expectations require completion \"success\" — a native trap exit (e.g. 70) IS a success completion with an exit code",
                ));
            }
            Some(*n as i32)
        }
        Some(CVal::Int(n)) => {
            return Err(err(
                &f.path_of("exit"),
                format!("exit code {} is outside the i32 native exit-code range", n),
            ))
        }
        Some(other) => {
            return Err(err(
                &f.path_of("exit"),
                format!("expected integer, got {}", other.type_name()),
            ))
        }
    };

    let perf = match f.sub("perf") {
        None => None,
        Some(pv) => {
            let pp = f.path_of("perf");
            let ppairs = as_obj(pv, &pp)?;
            let pf = Fields::new(&pp, ppairs, &["total_ms_max"])?;
            let total_ms_max = match pf.sub("total_ms_max") {
                None => None,
                Some(CVal::Int(n)) if *n > 0 => Some(*n as u64),
                Some(CVal::Int(n)) => {
                    return Err(err(
                        &pf.path_of("total_ms_max"),
                        format!("total_ms_max must be > 0 (got {})", n),
                    ))
                }
                Some(other) => {
                    return Err(err(
                        &pf.path_of("total_ms_max"),
                        format!("expected integer, got {}", other.type_name()),
                    ))
                }
            };
            Some(PerfExpect { total_ms_max })
        }
    };

    let mut artifacts = Vec::new();
    if let Some(av) = f.sub("artifacts") {
        let ap = f.path_of("artifacts");
        let list = as_list(av, &ap)?;
        if matches!(completion, Completion::Error { .. }) {
            return Err(err(
                &ap,
                "`artifacts` expectations require completion \"success\"",
            ));
        }
        for (i, item) in list.iter().enumerate() {
            let ipath = format!("{}[{}]", ap, i);
            let pairs = as_obj(item, &ipath)?;
            let af = Fields::new(&ipath, pairs, &["path", "file", "compare"])?;
            let p = req_str(&af, "path")?;
            validate_virtual_path(&p).map_err(|e| err(&af.path_of("path"), e.to_string()))?;
            if adapter == Adapter::Syntax && !SYNTAX_ARTIFACT_PATHS.contains(&p.as_str()) {
                return Err(err(
                    &af.path_of("path"),
                    format!(
                        "syntax artifact byte-oracles support {:?} only (got {:?})",
                        SYNTAX_ARTIFACT_PATHS, p
                    ),
                ));
            }
            if adapter == Adapter::Ac && p != AC_HIR_ARTIFACT_PATH {
                return Err(err(
                    &af.path_of("path"),
                    format!(
                        "ac artifact byte-oracles support {:?} only (canonical HIR); obj/exe identity is manifest-bound — observe via ac/run or product_from, not byte goldens (got {:?})",
                        AC_HIR_ARTIFACT_PATH, p
                    ),
                ));
            }
            let file = req_str(&af, "file")?;
            validate_virtual_path(&file).map_err(|e| err(&af.path_of("file"), e.to_string()))?;
            let compare = decode_compare(af.sub("compare"), &format!("{}.compare", ipath))?;
            artifacts.push(ArtifactExpect {
                path: p,
                file,
                compare,
            });
        }
    }

    // PLAN-796: probe structured expectations — one entry per QUERY step
    // (edit-source/restart steps produce no observable of their own).
    let probe = if probe_channels {
        match f.sub("probe") {
            None => None,
            Some(v) => {
                let list = as_list(v, &f.path_of("probe"))?;
                if list.is_empty() || list.len() > PROBE_MAX_STEPS {
                    return Err(err(
                        &f.path_of("probe"),
                        format!(
                            "probe expectations must be 1..={} entries, got {}",
                            PROBE_MAX_STEPS,
                            list.len()
                        ),
                    ));
                }
                let mut out = Vec::new();
                for (i, item) in list.iter().enumerate() {
                    let ipath = format!("{}.probe[{}]", f.path, i);
                    let pairs = as_obj(item, &ipath)?;
                    let pf = Fields::new(
                        &ipath,
                        pairs,
                        &["cache", "compute_calls", "bindings_file", "stages_include"],
                    )?;
                    let cache = match pf.sub("cache") {
                        None => None,
                        // R4 vocabulary: hit/miss/stale/published (coarse
                        // outcomes), reprojected (content reused, positions
                        // refreshed) and diags (definitive diagnostics
                        // under an error completion)
                        Some(CVal::Str(c))
                            if ["hit", "miss", "stale", "published", "reprojected", "diags"]
                                .contains(&c.as_str()) =>
                        {
                            Some(c.clone())
                        }
                        Some(other) => {
                            return Err(err(
                                &pf.path_of("cache"),
                                format!(
                                    "cache must be one of hit/miss/stale/published, got {}",
                                    other.type_name()
                                ),
                            ))
                        }
                    };
                    let compute_calls = match pf.sub("compute_calls") {
                        None => None,
                        Some(CVal::Int(n)) if *n >= 0 && *n <= 64 => Some(*n as u32),
                        Some(other) => {
                            return Err(err(
                                &pf.path_of("compute_calls"),
                                format!(
                                    "compute_calls must be a small non-negative integer, got {}",
                                    other.type_name()
                                ),
                            ))
                        }
                    };
                    let bindings_file = match pf.sub("bindings_file") {
                        None => None,
                        Some(CVal::Str(fpath)) => {
                            if fpath.contains(['\\', ':', '\0']) || fpath.is_empty() {
                                return Err(err(
                                    &pf.path_of("bindings_file"),
                                    "bindings_file must be a case-relative sidecar path (no drive/absolute components)",
                                ));
                            }
                            Some(fpath.clone())
                        }
                        Some(other) => {
                            return Err(err(
                                &pf.path_of("bindings_file"),
                                format!("expected string, got {}", other.type_name()),
                            ))
                        }
                    };
                    let stages_include = match pf.sub("stages_include") {
                        None => Vec::new(),
                        Some(v) => {
                            let list = as_list(v, &pf.path_of("stages_include"))?;
                            let mut out = Vec::new();
                            for (j, sv) in list.iter().enumerate() {
                                match sv {
                                    CVal::Str(st) if !st.is_empty() => out.push(st.clone()),
                                    other => {
                                        return Err(err(
                                            &format!("{}.stages_include[{}]", pf.path, j),
                                            format!(
                                                "expected non-empty stage name, got {}",
                                                other.type_name()
                                            ),
                                        ))
                                    }
                                }
                            }
                            out
                        }
                    };
                    out.push(ProbeStepExpect {
                        cache,
                        compute_calls,
                        bindings_file,
                        stages_include,
                    });
                }
                Some(out)
            }
        }
    } else {
        None
    };

    Ok(ExpectSpec {
        completion,
        result,
        stdout,
        stderr,
        artifacts,
        exit,
        perf,
        probe,
    })
}

fn decode_compare(v: Option<&CVal>, path: &str) -> Result<Compare, ContractError> {
    match v {
        None => Ok(Compare::Bytes),
        Some(CVal::Str(s)) if s == "bytes" => Ok(Compare::Bytes),
        Some(CVal::Str(s)) if s == "lf" => Ok(Compare::Lf),
        Some(CVal::Str(other)) => Err(err(
            path,
            format!("unknown compare mode {:?} (supported: bytes, lf)", other),
        )),
        Some(other) => Err(err(
            path,
            format!("compare must be a string, got {}", other.type_name()),
        )),
    }
}
