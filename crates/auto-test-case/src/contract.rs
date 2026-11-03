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
    /// `.mnf` (v2 only, PLAN-891): a file-ized native typed fixture
    /// (mn-typed-fixture/1 JSON bytes) consumed by the typed recipe only.
    TypedFixture,
}

impl Syntax {
    pub fn extension(&self) -> &'static str {
        match self {
            Syntax::Auto => "at",
            Syntax::AutoScript => "as",
            Syntax::HirAtom => "atom",
            Syntax::AtomData => "atomdata",
            // PLAN-891: the file-ized typed fixture input.
            Syntax::TypedFixture => "mnf",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Syntax::Auto => "auto",
            Syntax::AutoScript => "auto-script",
            Syntax::HirAtom => "hir-atom",
            Syntax::TypedFixture => "typed-fixture",
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
    /// probe only (PLAN-817): cte/scalar-query-v1 binding.
    pub probe_cte: Option<ProbeCteSpec>,
    /// probe only (PLAN-818): ac/mir-native-scalar-v1 binding.
    pub probe_mn: Option<ProbeMnSpec>,
    /// probe only (PLAN-892): ac-a2r/hir-scalar-emit-v1 binding.
    pub probe_a2x_hir: Option<ProbeA2xHirSpec>,
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
            probe_cte: None,
            probe_mn: None,
            probe_a2x_hir: None,
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
            // PLAN-909: "verify" completes the probe diagnostic-stage
            // vocabulary (the HIR verifier's rejections, e.g.
            // verify.immutable-write, were already documented upstream).
            Adapter::Probe => &["parse", "resolve", "typecheck", "verify", "run"],
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
    /// PLAN-862: the AV RESULT CHANNEL the run step must report —
    /// `"value"` (an `i32` payload) or `"unit"` (the Unit payload, which has
    /// no i32 form). Declaring it is how a Unit entry is distinguished from
    /// an int entry returning 0 (T-02 entry-contract §4/§6: Unit ≠ 0, and a
    /// consumer that prints only a number cannot tell them apart).
    pub av_channel: Option<String>,
    /// PLAN-891: the typed-entry VALUE oracle (probe, typed recipe only) —
    /// the machine result's kind and canonical raw bits. Judgement is
    /// parent-side; the worker never sees this.
    pub typed: Option<TypedExpect>,
    /// PLAN-914: the AV run-step MARK TRACE oracle (probe, AV recipes
    /// only) — the exact execution-order event sequence the worker
    /// reported (`step.trace`, e.g. "ba"). The worker's own report is
    /// observation, never an oracle; the parent compares these bytes.
    pub trace: Option<StreamExpect>,
}

/// PLAN-891 typed-entry expectation: kind label + canonical hex bits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExpect {
    pub kind: String,
    pub bits: String,
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
        // PLAN-891: v2 typed fixtures (mn-typed-fixture/1 JSON bytes).
        "typed-fixture" if version == ContractVersion::V2 => Syntax::TypedFixture,
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

    let console_recipe = options.probe_recipe.as_deref() == Some("ac-av/console-run-v1");
    // PLAN-892/902: the a2x emit recipes judge the four-file ACTUAL SET as
    // byte goldens — the only probe recipes with an artifacts channel.
    let a2x_recipe = matches!(
        options.probe_recipe.as_deref(),
        Some("ac-a2r/hir-scalar-emit-v1")
            | Some("ac-a2r/hir-record-text-emit-v1")
            | Some("ac-a2r/hir-collections-emit-v1")
            | Some("ac-a2c/hir-scalar-emit-v1")
            | Some("ac-a2c/hir-record-text-emit-v1")
    );
    // PLAN-914: the AV recipes are the only probe recipes whose run steps
    // report `step.trace`; the mark-trace oracle is admitted there only.
    let trace_recipe = options
        .probe_recipe
        .as_deref()
        .map(|r| r.starts_with("ac-av/"))
        .unwrap_or(false);
    // PLAN-914: the MIR-native recipes run a real PE and capture its
    // stdout/stderr (`stdout_hex`/`stderr_hex`); stderr is admitted there
    // for the executed mark bytes.
    let mn_recipe = options
        .probe_recipe
        .as_deref()
        .map(|r| r.starts_with("ac/mir-native-"))
        .unwrap_or(false);
    let expect = decode_expect(
        f.req("expect")?,
        &f.path_of("expect"),
        adapter,
        action,
        console_recipe,
        a2x_recipe,
        trace_recipe,
    )?;
    if adapter == Adapter::Probe {
        // PLAN-914 (I04): the declaration oracle and the declaration round
        // are two halves of one observation — never execute an invalidation
        // nobody judges, never declare an oracle no round produces.
        if let (Some(mn), Some(steps)) = (options.probe_mn.as_ref(), options.probe_steps.as_deref()) {
            let has_decl_step = steps
                .iter()
                .any(|s| matches!(s, ProbeStep::DeclarationInvalidate { .. }));
            if has_decl_step && mn.declaration.is_none() {
                return Err(err(
                    &f.path_of("options"),
                    "a declaration-invalidate step requires the mn.declaration oracle (the invalidation is judged, never merely executed)",
                ));
            }
            if mn.declaration.is_some() && !has_decl_step {
                return Err(err(
                    &f.path_of("options"),
                    "mn.declaration is declared without a declaration-invalidate step",
                ));
            }
        }
        // PLAN-914 (I04): the same two-way rule for the AV data-incremental
        // recipe — an edit is judged through the invalidation oracle, and the
        // oracle never dangles.
        if let Some(av) = options.probe_av.as_ref().filter(|_| {
            options.probe_recipe.as_deref() == Some("ac-av/data-incremental-v1")
        }) {
            let has_edit = options
                .probe_steps
                .as_deref()
                .map(|s| s.iter().any(|st| matches!(st, ProbeStep::EditSource { .. })))
                .unwrap_or(false);
            if has_edit && av.invalidation.is_none() {
                return Err(err(
                    &f.path_of("options"),
                    "an edit step on ac-av/data-incremental-v1 requires the av.invalidation oracle (the rebuild set is judged, never merely executed)",
                ));
            }
            if av.invalidation.is_some() && !has_edit {
                return Err(err(
                    &f.path_of("options"),
                    "av.invalidation is declared without an edit-source step",
                ));
            }
        }
        if console_recipe {
            if expect.probe.is_some() {
                return Err(err(
                    &f.path_of("expect"),
                    "console recipe exposes byte streams and result/error, not a probe-step oracle",
                ));
            }
            validate_console_oracle(
                expect.stdout.as_ref(),
                expect.stderr.as_ref(),
                expect.exit,
                expect.av_channel.as_deref(),
                &expect.completion,
            )
            .map_err(|m| err(&f.path_of("expect"), m))?;
            let av = options.probe_av.as_ref().ok_or_else(|| {
                err(
                    &f.path_of("options"),
                    "console recipe requires an av binding",
                )
            })?;
            if av.entry != "d_main" || av.session.is_some() || av.profile.is_some() {
                return Err(err(
                    &f.path_of("options"),
                    "console-run-v1 requires d_main and no session/profile override",
                ));
            }
            if !matches!(options.probe_steps.as_deref(), Some([ProbeStep::Run])) {
                return Err(err(
                    &f.path_of("options"),
                    "console-run-v1 requires exactly one run step",
                ));
            }
        } else if expect.stderr.is_some() && !mn_recipe {
            // PLAN-914: the MIR-native recipes capture the produced PE's
            // real streams; `stderr` is admitted there so a scenario can
            // observe the executed mark bytes. stdout stays console-only
            // until a consumer actually needs it (no silent widening).
            return Err(err(
                &f.path_of("expect"),
                "probe stderr is observable only with ac-av/console-run-v1 or an ac/mir-native-* recipe",
            ));
        } else if expect.stdout.is_some() {
            return Err(err(
                &f.path_of("expect"),
                "probe streams are observable only with ac-av/console-run-v1",
            ));
        }
    }

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
        Adapter::Probe => &[
            "recipe", "steps", "sema", "abi", "av", "cte", "mn", "a2x_hir",
        ],
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

    let (
        probe_recipe,
        probe_steps,
        probe_sema,
        probe_abi,
        probe_av,
        probe_cte,
        probe_mn,
        probe_a2x_hir,
    ) = if adapter == Adapter::Probe {
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
                    let sf = Fields::new(&ipath, pairs, &["op", "from", "to", "slices", "defs"])?;
                    let op = req_str(&sf, "op")?;
                    // PLAN-798: `run` joins the known op vocabulary
                    // (ac-av's only op); recipes never borrow each other's
                    // ops, and genuinely unknown ops keep the original
                    // unknown-op error.
                    let known_op = matches!(
                        op.as_str(),
                        "query"
                            | "edit-source"
                            | "declaration-invalidate"
                            | "partial-rebuild"
                            | "restart"
                            | "run"
                            | "resident-slice"
                            | "resident-apply"
                            | "resident-finish"
                    );
                    if !known_op {
                        return Err(err(
                            &sf.path_of("op"),
                            format!("unknown probe op {:?} (supported: query, edit-source, declaration-invalidate, restart, run, resident-slice, resident-apply, resident-finish)", op),
                        ));
                    }
                    let allowed = match recipe.as_str() {
                        // T-07 edit-session: runs + discriminating edits.
                        "ac-av/core-run-v1" => matches!(op.as_str(), "run" | "edit-source"),
                        "ac-av/console-run-v1" => op == "run",
                        // PLAN-820: the data recipe shares the core recipe's
                        // step vocabulary (runs + discriminating edits).
                        "ac-av/data-run-v1" => matches!(op.as_str(), "run" | "edit-source"),
                        // PLAN-914: the data INCREMENTAL session recipe shares the same
                        // run/edit vocabulary (the parent judges the invalidation facts).
                        "ac-av/data-incremental-v1" => matches!(
                            op.as_str(),
                            "run" | "edit-source" | "partial-rebuild"
                        ),
                        // PLAN-821: the owned recipe shares the av step
                        // vocabulary (runs + discriminating edits).
                        "ac-av/owned-run-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        // PLAN-919: the lifecycle recipes share the run/edit
                        // vocabulary (the judged step is the run).
                        "ac-av/resource-events-v1" | "ac/mir-native-resource-events-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        // PLAN-822: the collections recipe shares the av
                        // step vocabulary (runs + discriminating edits).
                        "ac-av/collections-run-v1" | "ac-av/loaded-run-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        // PLAN-826: the compact-wire recipe shares the
                        // av step vocabulary.
                        "ac-av/compact-run-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        // PLAN-864: the collections-source recipe shares
                        // the av step vocabulary.
                        "ac-av/collections-source-run-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        "ac-av/closures-run-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        // PLAN-865: source closures run through the same
                        // run/edit session vocabulary as the frozen fixture.
                        "ac-av/closures-source-run-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        // PLAN-912: the FUNCTION-session closures source
                        // recipe — same run/edit vocabulary.
                        "ac-av/closures-fn-run-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        // PLAN-913: the source-bound recovery recipe — the
                        // same run/edit session vocabulary (publish/restore
                        // are `run` steps; the post-restore edit is
                        // `edit-source` + `run`).
                        "ac-av/closures-recovery-run-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        // PLAN-824: the dynamic recipe shares the same
                        // step vocabulary.
                        "ac-av/dynamic-run-v1" => {
                            matches!(op.as_str(), "run" | "edit-source")
                        }
                        // PLAN-866: the resident-online recipe — slice /
                        // apply-at-safety-point / finish plus the shared
                        // discriminating edit vocabulary.
                        "ac-av/resident-run-v1" => matches!(
                            op.as_str(),
                            "edit-source" | "resident-slice" | "resident-apply" | "resident-finish"
                        ),
                        // PLAN-818 T-06: runs + discriminating edits +
                        // restart session boundaries (`query` stays a
                        // sema/abi op — recipes never borrow ops).
                        "ac/mir-native-scalar-v1" => {
                            matches!(op.as_str(), "run" | "edit-source" | "restart")
                        }
                        // PLAN-914 (I04): only the collections incremental
                        // session implements the explicit DECLARATION/layout
                        // invalidation round; the scalar profile has no
                        // declaration surface (types are not part of it).
                        "ac/mir-native-collections-v1" => matches!(
                            op.as_str(),
                            "run" | "edit-source" | "declaration-invalidate" | "restart"
                        ),
                        // PLAN-891: the typed recipe consumes exactly a
                        // single run step in the formal case form.
                        "ac/mir-native-typed-v1" => matches!(op.as_str(), "run"),
                        // PLAN-915: the resource recipe takes exactly the
                        // judged run step (the sweep/sequence is the op).
                        "ac-av/resource-run-v1" => matches!(op.as_str(), "run"),
                        // PLAN-892: the daily emit is a single run step
                        // (edit/restart sessions arrive with their own
                        // contract when the session face lands). The
                        // PLAN-902 record-text recipe shares the shape.
                        "ac-a2r/hir-scalar-emit-v1"
                        | "ac-a2r/hir-record-text-emit-v1"
                        | "ac-a2r/hir-collections-emit-v1" => matches!(op.as_str(), "run"),
                        // PLAN-921: the C scalar emit recipe takes exactly
                        // one judged run step (the daily `daily-emit-c`);
                        // certify/edit faces arrive with their own contract.
                        "ac-a2c/hir-scalar-emit-v1" => matches!(op.as_str(), "run"),
                        // PLAN-925: the C record-text emit recipe takes
                        // exactly one judged run step (the daily emit).
                        "ac-a2c/hir-record-text-emit-v1" => matches!(op.as_str(), "run"),
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
                        "resident-apply" => ProbeStep::ResidentApply,
                        "resident-finish" => ProbeStep::ResidentFinish,
                        "resident-slice" => {
                            let raw = req_str(&sf, "slices")?;
                            let slices = match raw.parse::<u32>() {
                                Ok(n) if (1..=65_536).contains(&n) => n,
                                _ => {
                                    return Err(err(
                                        &sf.path_of("slices"),
                                        "resident-slice needs an integer 1..=65536",
                                    ))
                                }
                            };
                            ProbeStep::ResidentSlice { slices }
                        }
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
                        "partial-rebuild" => {
                            let from = req_str(&sf, "from")?;
                            let to = req_str(&sf, "to")?;
                            if from.is_empty() || from == to {
                                return Err(err(
                                    &sf.path_of("from"),
                                    "partial-rebuild needs a non-empty `from` distinct from `to`",
                                ));
                            }
                            let defs = match sf.sub("defs") {
                                None => {
                                    return Err(err(
                                        &sf.path_of("defs"),
                                        "partial-rebuild needs the non-empty `defs` list (the controlled rebuild set)",
                                    ))
                                }
                                Some(lv) => {
                                    let list = as_list(lv, &sf.path_of("defs"))?;
                                    let mut out = Vec::new();
                                    for (i, item) in list.iter().enumerate() {
                                        let ipath = format!("{}.defs[{}]", sf.path_of("defs"), i);
                                        out.push(as_str(item, &ipath)?.to_string());
                                    }
                                    out
                                }
                            };
                            if defs.is_empty() {
                                return Err(err(
                                    &sf.path_of("defs"),
                                    "partial-rebuild defs must not be empty",
                                ));
                            }
                            ProbeStep::PartialRebuild { from, to, defs }
                        }
                        "declaration-invalidate" => {
                            // PLAN-914 (I04): an explicit declaration/layout
                            // invalidation round — the same exactly-once
                            // literal replacement shape as edit-source, but the
                            // session routes it through the cold declaration
                            // frontend + dependency-precise rebuild instead of
                            // the single-body fast path.
                            let from = req_str(&sf, "from")?;
                            let to = req_str(&sf, "to")?;
                            if from.is_empty() || from == to {
                                return Err(err(
                                    &sf.path_of("from"),
                                    "declaration-invalidate needs a non-empty `from` distinct from `to` (a discriminating declaration oracle)",
                                ));
                            }
                            if from.len() > 64 * 1024 || to.len() > 64 * 1024 {
                                return Err(err(
                                    &sf.path_of("to"),
                                    "declaration-invalidate literals exceed the 64KiB step cap",
                                ));
                            }
                            ProbeStep::DeclarationInvalidate { from, to }
                        }
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
                if recipe == "ac-av/core-run-v1"
                    || recipe == "ac-av/console-run-v1"
                    || recipe == "ac-av/data-run-v1"
                    || recipe == "ac-av/data-incremental-v1"
                    || recipe == "ac-av/owned-run-v1"
                    || recipe == "ac-av/resource-events-v1"
                    || recipe == "ac-av/collections-run-v1"
                    || recipe == "ac-av/collections-source-run-v1"
                    || recipe == "ac-av/loaded-run-v1"
                    || recipe == "ac-av/closures-run-v1"
                    || recipe == "ac-av/closures-source-run-v1"
                    || recipe == "ac-av/closures-fn-run-v1"
                    || recipe == "ac-av/closures-recovery-run-v1"
                    || recipe == "ac-av/dynamic-run-v1"
                    || recipe == "ac-av/compact-run-v1"
                {
                    // PLAN-798 T-07 edit-session shape (F798-R3-07): runs
                    // plus discriminating edit-source steps; a sequence
                    // may LEAD with edits (the measured formal window —
                    // the request materializes the source, applies leading
                    // edits, then runs) but must END with the judged run,
                    // and leading edits require an explicit session only
                    // for cross-call warmth (single-call shapes are
                    // unchanged). PLAN-820: the data recipe shares the
                    // same shape (its judged step is also a run).
                    if !matches!(out.last(), Some(ProbeStep::Run)) {
                        return Err(err(
                            &f.path_of("steps"),
                            "ac-av step sequences must end with a run step (the judged step)",
                        ));
                    }
                    if out.is_empty() {
                        return Err(err(&f.path_of("steps"), "steps must not be empty"));
                    }
                }
                // PLAN-866: the resident recipe's judged step is the
                // resident-finish (the chain's terminal outcome), never
                // a bare run.
                if recipe == "ac-av/resident-run-v1"
                    && !matches!(out.last(), Some(ProbeStep::ResidentFinish))
                {
                    return Err(err(
                            &f.path_of("steps"),
                            "ac-av/resident-run-v1 step sequences must end with a resident-finish step (the judged step)",
                        ));
                }
                if recipe == "cte/scalar-query-v1" && !matches!(out.last(), Some(ProbeStep::Query))
                {
                    return Err(err(
                            &f.path_of("steps"),
                            "cte/scalar-query-v1 step sequences must end with a query step (the judged step)",
                        ));
                }
                if matches!(
                    recipe.as_str(),
                    "ac/mir-native-scalar-v1" | "ac/mir-native-collections-v1"
                ) {
                    // PLAN-818 T-06 session shape: leading edits
                    // materialize parent-side, `restart` splits worker
                    // sessions (a real new process each), and the judged
                    // step is the sequence's LAST run.
                    if !matches!(out.last(), Some(ProbeStep::Run)) {
                        return Err(err(
                                &f.path_of("steps"),
                                "ac/mir-native-scalar-v1 step sequences must end with a run step (the judged step)",
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
                if recipe == "ac-av/core-run-v1"
                        || recipe == "ac-av/console-run-v1"
                        || recipe == "ac-av/data-run-v1"
                    || recipe == "ac-av/data-incremental-v1"
                        || recipe == "ac-av/owned-run-v1"
                        || recipe == "ac-av/resource-events-v1"
                        || recipe == "ac-av/collections-run-v1"
                        || recipe == "ac-av/collections-source-run-v1"
                        || recipe == "ac-av/loaded-run-v1"
                        || recipe == "ac-av/closures-run-v1"
                        || recipe == "ac-av/closures-source-run-v1"
                        || recipe == "ac-av/closures-fn-run-v1"
                        || recipe == "ac-av/closures-recovery-run-v1"
                        || recipe == "ac-av/dynamic-run-v1"
                        || recipe == "ac-av/compact-run-v1"
                        // PLAN-866: the resident recipe requires the av
                        // binding (entry) like its siblings.
                        || recipe == "ac-av/resident-run-v1"
                {
                    return Err(err(
                        &f.path_of("av"),
                        format!("{recipe} requires an av binding"),
                    ));
                }
                None
            }
            Some(v) => {
                if recipe != "ac-av/core-run-v1"
                        && recipe != "ac-av/console-run-v1"
                        && recipe != "ac-av/data-run-v1"
                        && recipe != "ac-av/owned-run-v1"
                        && recipe != "ac-av/resource-events-v1"
                        && recipe != "ac-av/collections-run-v1"
                        && recipe != "ac-av/collections-source-run-v1"
                        && recipe != "ac-av/loaded-run-v1"
                        && recipe != "ac-av/closures-run-v1"
                        && recipe != "ac-av/closures-source-run-v1"
                        && recipe != "ac-av/closures-fn-run-v1"
                        && recipe != "ac-av/closures-recovery-run-v1"
                        && recipe != "ac-av/dynamic-run-v1"
                        && recipe != "ac-av/compact-run-v1"
                        // PLAN-866: the resident recipe takes the same av
                        // binding shape as its siblings.
                        && recipe != "ac-av/resident-run-v1"
                        // PLAN-914: the data INCREMENTAL recipe takes the
                        // same av binding shape (entry/caps/session).
                        && recipe != "ac-av/data-incremental-v1"
                        // PLAN-915: the resource scenario recipe selects its
                        // face through av.entry (source:<id>|place:<mutation>).
                        && recipe != "ac-av/resource-run-v1"
                {
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
                let sf = Fields::new(
                    &apath,
                    pairs,
                    &["entry", "caps", "session", "profile", "invalidation", "resource"],
                )?;
                let entry = req_str(&sf, "entry")?;
                // PLAN-862: the SOURCE-profile selector (append-only). A
                // check that omits it keeps the historical scalar path
                // byte-for-byte (`auto.source.scalar-i32.draft`); the
                // entry-unit profile is the ONLY other value, and it is
                // served by the core recipe (it is the same AV worker /
                // pipeline, a different front door and wire family). Any
                // other combination is an admission error, never a
                // silent fallback to scalar.
                let profile = match sf.sub("profile") {
                    None => None,
                    Some(pv) => {
                        let p = as_str(pv, &sf.path_of("profile"))?.to_string();
                        if p != "entry-unit" {
                            return Err(err(
                                    &sf.path_of("profile"),
                                    format!(
                                        "unknown av profile {p:?} (supported: entry-unit; omit for the default scalar profile)"
                                    ),
                                ));
                        }
                        if recipe != "ac-av/core-run-v1" {
                            return Err(err(
                                    &sf.path_of("profile"),
                                    format!(
                                        "the entry-unit source profile belongs to ac-av/core-run-v1, not {recipe:?}"
                                    ),
                                ));
                        }
                        Some(p)
                    }
                };
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
                let invalidation = match sf.sub("invalidation") {
                    None => None,
                    Some(v) => {
                        if recipe != "ac-av/data-incremental-v1" {
                            return Err(err(
                                &sf.path_of("invalidation"),
                                "the invalidation oracle belongs to ac-av/data-incremental-v1 only",
                            ));
                        }
                        let ipath = sf.path_of("invalidation");
                        let ipairs = as_obj(v, &ipath)?;
                        let inf = Fields::new(
                            &ipath,
                            ipairs,
                            &["stable_defs", "rebuilt_defs"],
                        )?;
                        let read_defs = |key: &str| -> Result<Vec<String>, ContractError> {
                            match inf.sub(key) {
                                None => Ok(Vec::new()),
                                Some(lv) => {
                                    let list = as_list(lv, &inf.path_of(key))?;
                                    let mut out = Vec::new();
                                    for (i, item) in list.iter().enumerate() {
                                        let dpath = format!("{}.{}[{}]", ipath, key, i);
                                        let name = as_str(item, &dpath)?.to_string();
                                        if name.is_empty() {
                                            return Err(err(
                                                &dpath,
                                                "empty def id identifies nothing",
                                            ));
                                        }
                                        out.push(name);
                                    }
                                    Ok(out)
                                }
                            }
                        };
                        let stable_defs = read_defs("stable_defs")?;
                        let rebuilt_defs = read_defs("rebuilt_defs")?;
                        if stable_defs.is_empty() && rebuilt_defs.is_empty() {
                            return Err(err(
                                &ipath,
                                "the invalidation oracle needs stable_defs and/or rebuilt_defs (an empty oracle judges nothing)",
                            ));
                        }
                        for d in &stable_defs {
                            if rebuilt_defs.contains(d) {
                                return Err(err(
                                    &ipath,
                                    format!(
                                        "def {:?} is declared both stable and rebuilt (contradictory oracle)",
                                        d
                                    ),
                                ));
                            }
                        }
                        Some(ProbeAvInvalidationSpec {
                            stable_defs,
                            rebuilt_defs,
                        })
                    }
                };
                // PLAN-919: the lifecycle recipe's binding — the source
                // PROFILE (one of the two admitted production profiles)
                // and the R06 exit knobs. Operations and budgets only.
                let resource = match sf.sub("resource") {
                    None => {
                        if recipe == "ac-av/resource-events-v1" {
                            return Err(err(
                                &sf.path_of("resource"),
                                "ac-av/resource-events-v1 requires av.resource { profile, .. }",
                            ));
                        }
                        None
                    }
                    Some(rv) => {
                        if recipe != "ac-av/resource-events-v1" {
                            return Err(err(
                                &sf.path_of("resource"),
                                format!(
                                    "av.resource belongs to ac-av/resource-events-v1, not {recipe:?}"
                                ),
                            ));
                        }
                        let rpath = sf.path_of("resource");
                        let rpairs = as_obj(rv, &rpath)?;
                        let rf = Fields::new(
                            &rpath,
                            rpairs,
                            &["profile", "max_steps", "wall_ms", "cancel_after_ms"],
                        )?;
                        let rprofile = req_str(&rf, "profile")?;
                        if rprofile != "owned" && rprofile != "collections" {
                            return Err(err(
                                &rf.path_of("profile"),
                                format!(
                                    "unknown resource profile {rprofile:?} (owned | collections)"
                                ),
                            ));
                        }
                        let knob = |k: &str| -> Result<Option<u64>, ContractError> {
                            match rf.sub(k) {
                                None => Ok(None),
                                Some(kv) => match kv {
                                    CVal::Int(n) if *n >= 1 => Ok(Some(*n as u64)),
                                    _ => Err(err(
                                        &rf.path_of(k),
                                        format!("{k} must be an integer >= 1 (a zero budget judges nothing)"),
                                    )),
                                },
                            }
                        };
                        Some(ProbeAvResourceSpec {
                            profile: rprofile,
                            max_steps: knob("max_steps")?,
                            wall_ms: knob("wall_ms")?,
                            cancel_after_ms: knob("cancel_after_ms")?,
                        })
                    }
                };
                Some(ProbeAvSpec {
                    entry,
                    caps,
                    session,
                    profile,
                    invalidation,
                    resource,
                })
            }
        };
        // PLAN-817: the cte binding (entry/caps/session — the query and
        // binding tables live in the atom-data deck the worker owns).
        let cte = match f.sub("cte") {
            None => {
                if recipe == "cte/scalar-query-v1" {
                    return Err(err(
                        &f.path_of("cte"),
                        "cte/scalar-query-v1 requires a cte binding",
                    ));
                }
                None
            }
            Some(v) => {
                if recipe != "cte/scalar-query-v1" {
                    return Err(err(
                            &f.path_of("cte"),
                            format!(
                                "the cte binding belongs to cte/scalar-query-v1; it is not valid for recipe {:?}",
                                recipe
                            ),
                        ));
                }
                let cpath = f.path_of("cte");
                let pairs = as_obj(v, &cpath)?;
                let sf = Fields::new(&cpath, pairs, &["entry", "caps", "session"])?;
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
                                "cte session id must be 1-64 chars of [A-Za-z0-9_-]",
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
                            let cpath = format!("{}.caps[{}]", cpath, i);
                            out.push(as_str(item, &cpath)?.to_string());
                        }
                        out
                    }
                };
                Some(ProbeCteSpec {
                    entry,
                    caps,
                    session,
                })
            }
        };
        let mn = match f.sub("mn") {
            None => {
                if matches!(
                    recipe.as_str(),
                    "ac/mir-native-scalar-v1"
                        | "ac/mir-native-collections-v1"
                        | "ac/mir-native-typed-v1"
                        | "ac/mir-native-resource-events-v1"
                ) {
                    return Err(err(
                        &f.path_of("mn"),
                        "ac/mir-native recipes require an mn binding",
                    ));
                }
                None
            }
            Some(v) => {
                if !matches!(
                    recipe.as_str(),
                    "ac/mir-native-scalar-v1"
                        | "ac/mir-native-collections-v1"
                        | "ac/mir-native-typed-v1"
                        | "ac/mir-native-resource-events-v1"
                ) {
                    return Err(err(
                            &f.path_of("mn"),
                            format!(
                                "the mn binding belongs to ac/mir-native-scalar-v1; it is not valid for recipe {:?}",
                                recipe
                            ),
                        ));
                }
                let mpath = f.path_of("mn");
                let pairs = as_obj(v, &mpath)?;
                let mf = Fields::new(
                    &mpath,
                    pairs,
                    &[
                        "entry",
                        "caps",
                        "session",
                        "typed",
                        "diagnostics",
                        "declaration",
                        "resource",
                    ],
                )?;
                let entry = req_str(&mf, "entry")?;
                let caps = match mf.sub("caps") {
                    None => Vec::new(),
                    Some(v) => {
                        let list = as_list(v, &mf.path_of("caps"))?;
                        let mut out = Vec::new();
                        for (i, item) in list.iter().enumerate() {
                            let cpath = format!("{}.caps[{}]", mpath, i);
                            out.push(as_str(item, &cpath)?.to_string());
                        }
                        out
                    }
                };
                let session = match mf.sub("session") {
                    None => None,
                    Some(v) => Some(as_str(v, &mf.path_of("session"))?.to_string()),
                };
                let typed = match mf.sub("typed") {
                    None => {
                        if recipe == "ac/mir-native-typed-v1" {
                            return Err(err(
                                &mf.path_of("typed"),
                                "ac/mir-native-typed-v1 requires a typed binding",
                            ));
                        }
                        None
                    }
                    Some(v) => {
                        if recipe != "ac/mir-native-typed-v1" {
                            return Err(err(
                                &mf.path_of("typed"),
                                "the typed binding belongs to ac/mir-native-typed-v1",
                            ));
                        }
                        let tpath = mf.path_of("typed");
                        let tpairs = as_obj(v, &tpath)?;
                        let tf = Fields::new(&tpath, tpairs, &["fixture", "args"])?;
                        let fixture = req_str(&tf, "fixture")?;
                        let args = match tf.sub("args") {
                            None => Vec::new(),
                            Some(av) => {
                                let list = as_list(av, &tf.path_of("args"))?;
                                let mut out = Vec::new();
                                for (i, item) in list.iter().enumerate() {
                                    let ipath = format!("{tpath}.args[{i}]");
                                    let pair = as_list(item, &ipath)?;
                                    if pair.len() != 2 {
                                        return Err(err(
                                            &ipath,
                                            "a typed argument is a [kind, bits] pair",
                                        ));
                                    }
                                    out.push((
                                        as_str(&pair[0], &format!("{ipath}[0]"))?.to_string(),
                                        as_str(&pair[1], &format!("{ipath}[1]"))?.to_string(),
                                    ));
                                }
                                out
                            }
                        };
                        Some(ProbeMnTypedSpec { fixture, args })
                    }
                };
                let diagnostics = match mf.sub("diagnostics") {
                    None => None,
                    Some(v) => {
                        if recipe != "ac/mir-native-scalar-v1" {
                            return Err(err(
                                &mf.path_of("diagnostics"),
                                "mn.diagnostics belongs to ac/mir-native-scalar-v1 only",
                            ));
                        }
                        let dv = as_str(v, &mf.path_of("diagnostics"))?;
                        if dv != "source-v1" {
                            return Err(err(
                                &mf.path_of("diagnostics"),
                                "mn.diagnostics accepts only \"source-v1\" (the mn.source-diagnostic/1 observation mode)",
                            ));
                        }
                        Some(dv.to_string())
                    }
                };
                let declaration = match mf.sub("declaration") {
                    None => None,
                    Some(v) => {
                        if recipe != "ac/mir-native-collections-v1" {
                            return Err(err(
                                &mf.path_of("declaration"),
                                "the declaration oracle belongs to ac/mir-native-collections-v1 only (the scalar profile has no declaration surface)",
                            ));
                        }
                        let dpath = mf.path_of("declaration");
                        let dpairs = as_obj(v, &dpath)?;
                        let df = Fields::new(
                            &dpath,
                            dpairs,
                            &[
                                "min_frontend_invalidations",
                                "max_codegen_units",
                                "stable_defs",
                            ],
                        )?;
                        let min_frontend_invalidations =
                            match df.sub("min_frontend_invalidations") {
                                None => 1,
                                Some(mv) => match mv {
                                    CVal::Int(n) if *n >= 1 && *n <= 64 => *n as u64,
                                    _ => {
                                        return Err(err(
                                            &df.path_of("min_frontend_invalidations"),
                                            "min_frontend_invalidations must be an integer 1..=64",
                                        ))
                                    }
                                },
                            };
                        let max_codegen_units = match df.sub("max_codegen_units") {
                            None => None,
                            Some(mv) => match mv {
                                CVal::Int(n) if *n >= 0 && *n <= 1024 => Some(*n as u64),
                                _ => {
                                    return Err(err(
                                        &df.path_of("max_codegen_units"),
                                        "max_codegen_units must be an integer 0..=1024",
                                    ))
                                }
                            },
                        };
                        let stable_defs = match df.sub("stable_defs") {
                            None => Vec::new(),
                            Some(lv) => {
                                let list = as_list(lv, &df.path_of("stable_defs"))?;
                                let mut out = Vec::new();
                                for (i, item) in list.iter().enumerate() {
                                    let ipath = format!("{}.stable_defs[{}]", dpath, i);
                                    let name = as_str(item, &ipath)?.to_string();
                                    if name.is_empty() {
                                        return Err(err(
                                            &ipath,
                                            "empty stable_defs entry identifies nothing",
                                        ));
                                    }
                                    out.push(name);
                                }
                                out
                            }
                        };
                        Some(ProbeMnDeclarationSpec {
                            min_frontend_invalidations,
                            max_codegen_units,
                            stable_defs,
                        })
                    }
                };
                // PLAN-919: the native lifecycle recipe's exit knobs
                // (guest byte budget, cancellation after N work units,
                // the Nth-allocation fault) — driven by the generated
                // entry wrapper, never by a parent-side kill.
                let resource = match mf.sub("resource") {
                    None => {
                        if recipe == "ac/mir-native-resource-events-v1" {
                            return Err(err(
                                &mf.path_of("resource"),
                                "ac/mir-native-resource-events-v1 requires mn.resource { .. }",
                            ));
                        }
                        None
                    }
                    Some(rv) => {
                        if recipe != "ac/mir-native-resource-events-v1" {
                            return Err(err(
                                &mf.path_of("resource"),
                                format!(
                                    "mn.resource belongs to ac/mir-native-resource-events-v1, not {recipe:?}"
                                ),
                            ));
                        }
                        let rpath = mf.path_of("resource");
                        let rpairs = as_obj(rv, &rpath)?;
                        let rf = Fields::new(
                            &rpath,
                            rpairs,
                            &["budget_bytes", "cancel_after_work", "fail_nth"],
                        )?;
                        let knob = |k: &str| -> Result<Option<u64>, ContractError> {
                            match rf.sub(k) {
                                None => Ok(None),
                                Some(kv) => match kv {
                                    CVal::Int(n) if *n >= 1 => Ok(Some(*n as u64)),
                                    _ => Err(err(
                                        &rf.path_of(k),
                                        format!("{k} must be an integer >= 1 (a zero budget judges nothing)"),
                                    )),
                                },
                            }
                        };
                        Some(ProbeMnResourceSpec {
                            budget_bytes: knob("budget_bytes")?,
                            cancel_after_work: knob("cancel_after_work")?,
                            fail_nth: knob("fail_nth")?,
                        })
                    }
                };
                Some(ProbeMnSpec {
                    entry,
                    caps,
                    session,
                    typed,
                    diagnostics,
                    declaration,
                    resource,
                })
            }
        };

        // PLAN-892/902: the a2x_hir binding (same seam contract as mn —
        // required for its recipe, invalid elsewhere). The scalar recipe
        // keeps its core/scalar profiles; the record-text recipe admits
        // record/text/text-record (strictly disjoint). PLAN-921: the C
        // scalar recipe keeps its own core/scalar profile set.
        let is_a2x_recipe = matches!(
            recipe.as_str(),
            "ac-a2r/hir-scalar-emit-v1"
                | "ac-a2r/hir-record-text-emit-v1"
                | "ac-a2r/hir-collections-emit-v1"
                | "ac-a2c/hir-scalar-emit-v1"
                | "ac-a2c/hir-record-text-emit-v1"
        );
        let a2x_hir = match f.sub("a2x_hir") {
            None => {
                if is_a2x_recipe {
                    return Err(err(
                        &f.path_of("a2x_hir"),
                        format!("{} requires an a2x_hir binding", recipe),
                    ));
                }
                None
            }
            Some(v) => {
                if !is_a2x_recipe {
                    return Err(err(
                            &f.path_of("a2x_hir"),
                            format!(
                                "the a2x_hir binding belongs to the a2x emit recipes; it is not valid for recipe {:?}",
                                recipe
                            ),
                        ));
                }
                let hpath = f.path_of("a2x_hir");
                let pairs = as_obj(v, &hpath)?;
                let hf = Fields::new(&hpath, pairs, &["profile", "entry", "caps"])?;
                let profile = req_str(&hf, "profile")?;
                let profile_ok = match recipe.as_str() {
                    "ac-a2r/hir-scalar-emit-v1" => {
                        matches!(profile.as_str(), "core" | "scalar")
                    }
                    "ac-a2r/hir-collections-emit-v1" => profile == "collections",
                    // PLAN-921: the C scalar channel mirrors the a2r
                    // scalar profile set (core/scalar only).
                    "ac-a2c/hir-scalar-emit-v1" => {
                        matches!(profile.as_str(), "core" | "scalar")
                    }
                    // PLAN-925: the C record-text channel mirrors the a2r
                    // record-text profile set.
                    "ac-a2c/hir-record-text-emit-v1" => {
                        matches!(profile.as_str(), "record" | "text" | "text-record")
                    }
                    _ => matches!(profile.as_str(), "record" | "text" | "text-record"),
                };
                if !profile_ok {
                    return Err(err(
                        &hf.path_of("profile"),
                        format!(
                            "a2x_hir profile {:?} is not valid for recipe {:?}",
                            profile, recipe
                        ),
                    ));
                }
                let entry = req_str(&hf, "entry")?;
                let caps = match hf.sub("caps") {
                    None => Vec::new(),
                    Some(cv) => {
                        let list = as_list(cv, &hf.path_of("caps"))?;
                        let mut out = Vec::new();
                        for item in list {
                            out.push(as_str(item, &hf.path_of("caps"))?.to_string());
                        }
                        out
                    }
                };
                Some(ProbeA2xHirSpec {
                    profile,
                    entry,
                    caps,
                })
            }
        };

        (Some(recipe), Some(steps), sema, abi, av, cte, mn, a2x_hir)
    } else {
        (None, None, None, None, None, None, None, None)
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
        probe_cte,
        probe_mn,
        probe_a2x_hir,
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
/// recipe here under the same contract (single writer per window); 817
/// appends the CTE recipe and 818 the MIR-native recipe likewise.
pub const REGISTERED_PROBE_RECIPES: &[&str] = &[
    "sema/query-v1",
    "abi/interop-v1",
    "ac-av/core-run-v1",
    // PLAN-817: AV-driven CTE query + frozen cache. Appended in the
    // post-797/798 single-writer window; same seam contract.
    "cte/scalar-query-v1",
    // PLAN-818: the MIR-native experimental recipe, same seam contract.
    "ac/mir-native-scalar-v1",
    "ac/mir-native-collections-v1",
    // PLAN-892: the A2X Checked-HIR -> Rust emit recipe (prebuilt worker,
    // daily spawn-zero emit + parent-side golden judging).
    "ac-a2r/hir-scalar-emit-v1",
    // PLAN-902: the record/text/text-record A2X emit recipe — the same
    // prebuilt worker and seam contract, riding the record-text fileset.
    "ac-a2r/hir-record-text-emit-v1",
    // PLAN-904: the collections (List/Variant) A2X emit recipe — the same
    // prebuilt worker and seam contract, riding the collections fileset
    // (strictly disjoint recipe/profile/fileset/cert identities).
    "ac-a2r/hir-collections-emit-v1",
    // PLAN-921: the C scalar A2X emit recipe (HIR→C11) — the same prebuilt
    // worker (its `daily-emit-c` op) and seam contract, riding the C
    // fileset (main.c/a2x_c_runtime.h/generated.c + manifest.json sidecar;
    // strictly disjoint recipe/fileset/cert identity from every a2r one).
    "ac-a2c/hir-scalar-emit-v1",
    // PLAN-925: the C record-text A2X emit recipe - the same prebuilt
    // worker/seam contract, riding the record-text C fileset
    // (main.c/a2x_c_rt_runtime.h/generated.c with the seed-derived record
    // table + manifest.json sidecar; strictly disjoint from every other
    // recipe/fileset/cert identity).
    "ac-a2c/hir-record-text-emit-v1",
    // PLAN-820: the av.data/1 data recipe, same seam contract.
    "ac-av/data-run-v1",
    // PLAN-821: the av.owned/1 owned recipe, same seam contract.
    "ac-av/owned-run-v1",
    // PLAN-822: the av.collections/1 collections recipe, same seam contract.
    "ac-av/collections-run-v1",
    // PLAN-825: scalar LoadedProgram execution with parent-side counters.
    "ac-av/loaded-run-v1",
    // PLAN-823: the typed closures fixture uses the AV protocol.
    "ac-av/closures-run-v1",
    // PLAN-913: the source-bound recovery recipe (publish / restore /
    // restore-edit) — same seam contract, one closures profile identity.
    "ac-av/closures-recovery-run-v1",
    // PLAN-866: the resident-online recipe — one continuously-running
    // chain, edits at call-boundary safety points. Same seam contract.
    "ac-av/resident-run-v1",
    // PLAN-824: the av.dynamic/1 dynamic recipe, same seam contract.
    "ac-av/dynamic-run-v1",
    // PLAN-826: the compact AVAC wire recipe (scalar profile over the new
    // wire generation), same seam contract.
    "ac-av/compact-run-v1",
    // PLAN-864: the collections SOURCE recipe — real .at sources through
    // the collections frontend onto the frozen 822 execution, same seam
    // contract (append-only; the 822 fixture case files stay untouched).
    "ac-av/collections-source-run-v1",
    // PLAN-865: the closures SOURCE recipe — real .at sources through the
    // closures frontend onto the frozen 823 execution, same seam contract.
    "ac-av/closures-source-run-v1",
    // PLAN-912: the closures FUNCTION source recipe — same corpus through
    // the function session; the parent judges zero-production axes.
    "ac-av/closures-fn-run-v1",
    // PLAN-863: exact byte-stream observations, separate host-console profile.
    "ac-av/console-run-v1",
    // PLAN-914 (I04/I09): the DATA-profile INCREMENTAL recipe — a retained
    // DataIncrementalSession driven by run/edit-source steps; each edit
    // reports its dependency-precise rebuild set and the per-def resident
    // code identities.
    "ac-av/data-incremental-v1",
    // PLAN-891: the native typed-entry recipe — file-ized typed fixtures
    // through the real PE bitwise frame channel, parent-side judgement.
    "ac/mir-native-typed-v1",
    // PLAN-915: the R02 allocation-fault sweep / R03 stale-Place scenario
    // recipe — frozen corpus sources and typed place witnesses through the
    // real AV executor; the parent judges the frozen attempt counts.
    "ac-av/resource-run-v1",
    // PLAN-919: the per-owner lifecycle recipes (scenario R01/R04/R06 av/ac
    // cells) — real sources on the admitted production profiles with the
    // bounded observation channel on and the budget/cancel exit knobs;
    // parent-side judgement against frozen invariants and per-case tables.
    "ac-av/resource-events-v1",
    "ac/mir-native-resource-events-v1",
];
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
    /// PLAN-914 (I09): a CONTROLLED partial rebuild — rebuild exactly the
    /// named defs against the edited source while the other residents stay
    /// in place; the link/env-contract gate must REFUSE the incompatible
    /// mix and the previous snapshot must stay executable.
    PartialRebuild {
        from: String,
        to: String,
        defs: Vec<String>,
    },
    /// PLAN-914 (I04): an explicit DECLARATION/layout invalidation round
    /// (collections incremental session only). The same exactly-once literal
    /// replacement shape as `EditSource`, but the session runs the cold
    /// declaration frontend and re-produces only the dependency-affected
    /// units; the parent judges the declaration counters and the identity
    /// carry-over of the unaffected units.
    DeclarationInvalidate {
        from: String,
        to: String,
    },
    Restart,
    /// PLAN-798 ac-av/core-run-v1: run the case source through the full
    /// AV pipeline once and report the observed outcome (value / trap /
    /// diagnostic + counters) for parent-side judging.
    Run,
    /// PLAN-866 ac-av/resident-run-v1: advance the resident chain by
    /// `slices` fuel slices (each a fixed 64-step segment) — the chain is
    /// NEVER restarted by slicing; report the cumulative ledger.
    ResidentSlice {
        slices: u32,
    },
    /// PLAN-866 ac-av/resident-run-v1: apply every queued edit candidate
    /// at its call-boundary safety point (prepare → armed park → commit →
    /// swap-resolution resume); report the install/refusal faces.
    ResidentApply,
    /// PLAN-866 ac-av/resident-run-v1: drive the resident chain to
    /// completion and report the terminal outcome for parent-side
    /// judging (the judged step of the recipe).
    ResidentFinish,
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
    /// PLAN-862: optional SOURCE-profile selector. `None` = the historical
    /// scalar profile (`auto.source.scalar-i32.draft`, byte-identical for
    /// every existing case); `Some("entry-unit")` = the entry-unit profile
    /// (`auto.source.entry-unit.draft` → `av.entry-unit/1`, the `AVAU` wire
    /// family). Append-only: the field carries no default that could
    /// reinterpret an old check.
    pub profile: Option<String>,
    /// PLAN-914 (I04): the data-incremental invalidation oracle — the defs
    /// whose resident code identity must be CARRIED across an edit and the
    /// defs the dependency-precise rebuild must actually contain.
    pub invalidation: Option<ProbeAvInvalidationSpec>,
    /// PLAN-919: the lifecycle recipe's source profile and exit knobs.
    pub resource: Option<ProbeAvResourceSpec>,
}

/// PLAN-919 ac-av/resource-events-v1 binding: the admitted production
/// source profile the program runs on (`owned` | `collections`) and the
/// optional R06 exit knobs (instruction budget, wall budget, host
/// cancellation delay). Expectations never ride here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeAvResourceSpec {
    pub profile: String,
    pub max_steps: Option<u64>,
    pub wall_ms: Option<u64>,
    pub cancel_after_ms: Option<u64>,
}

/// PLAN-914 ac-av/data-incremental-v1 invalidation oracle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeAvInvalidationSpec {
    /// Def ids whose resident code identity must be unchanged (the
    /// UNAFFECTED definitions: not rebuilt, not re-lowered).
    pub stable_defs: Vec<String>,
    /// Def ids the rebuild set must contain (the affected definitions).
    pub rebuilt_defs: Vec<String>,
}

/// cte/scalar-query-v1 binding (PLAN-817 T-02): the consumer entry
/// function, optional capability list and cross-request session id.
/// The query/binding tables live in the atom-data deck (worker-owned).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeCteSpec {
    pub entry: String,
    pub caps: Vec<String>,
    pub session: Option<String>,
}

/// ac/mir-native-scalar-v1 binding (PLAN-818 T-02): the MIR-native entry
/// function and optional capability list — declarative Atom parameters
/// only. The worker performs the full `Checked HIR → MIR → per-function
/// COFF → multi-object PE` request under the parent's admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeMnSpec {
    pub entry: String,
    pub caps: Vec<String>,
    /// Optional cross-call session id (PLAN-818 T-07 turnaround shape):
    /// calls of one session share a warm store across worker processes;
    /// the PARENT owns the scratch root's cleanup.
    pub session: Option<String>,
    /// PLAN-891 typed-entry binding: the fixture FILE (mn-typed-fixture/1
    /// bytes ride the request; the worker re-verifies them) and the run
    /// arguments as canonical (kind, bits) pairs. Expected values never
    /// enter the request.
    pub typed: Option<ProbeMnTypedSpec>,
    /// PLAN-911: explicit source-diagnostic observation mode. The only
    /// legal value is "source-v1" (mn.source-diagnostic/1); only the
    /// scalar recipe accepts it; omitted keeps the pre-911 behavior.
    pub diagnostics: Option<String>,
    /// PLAN-914 (I04): the declaration/layout invalidation oracle of the
    /// collections recipe — the minimum cold-frontend invalidations, the
    /// maximum object-producer units and the defs whose identity must be
    /// CARRIED (the unaffected units). Declared expectations never reach
    /// the worker.
    pub declaration: Option<ProbeMnDeclarationSpec>,
    /// PLAN-919: the native lifecycle recipe's exit knobs.
    pub resource: Option<ProbeMnResourceSpec>,
}

/// PLAN-919 ac/mir-native-resource-events-v1 binding: the R06 exit knobs
/// the generated entry wrapper installs into the collections runtime
/// before the entry runs (`mn_coll_set_budget` / `mn_coll_cancel_after_work`
/// / `mn_coll_fail_nth`). Expectations never ride here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeMnResourceSpec {
    pub budget_bytes: Option<u64>,
    pub cancel_after_work: Option<u64>,
    pub fail_nth: Option<u64>,
}

/// PLAN-914 ac/mir-native-collections-v1 declaration-invalidation oracle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeMnDeclarationSpec {
    /// The recorded cold declaration-frontend invalidations (>= this).
    pub min_frontend_invalidations: u64,
    /// Upper bound on the object-producer units of the invalidation round
    /// (`codegen_emit_units`); None = only the carry-over is judged.
    pub max_codegen_units: Option<u64>,
    /// Unit keys (e.g. `d_unrelated`) whose object identity must be
    /// UNCHANGED across the invalidation (the unaffected units).
    pub stable_defs: Vec<String>,
}

/// PLAN-891 ac/mir-native-typed-v1 binding payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeMnTypedSpec {
    pub fixture: String,
    pub args: Vec<(String, String)>,
}

/// PLAN-892 ac-a2r/hir-scalar-emit-v1 binding: the explicit source
/// profile (core/scalar for the scalar recipe; record/text/text-record
/// for the record-text recipe), entry function name and declared
/// capabilities the prebuilt a2x worker compiles the case source under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeA2xHirSpec {
    pub profile: String,
    pub entry: String,
    pub caps: Vec<String>,
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
        // PLAN-892: the a2x emit recipe byte-compares its actual file set
        // (decode admits the artifacts field for this recipe only).
        (Adapter::Probe, "artifacts") => true,
        (Adapter::Probe, "stdout" | "stderr") => true,
        // PLAN-891: the native typed-entry recipe observes the machine
        // result's (kind, canonical bits) frame face.
        (Adapter::Probe, "typed") => true,
        // PLAN-914: the AV run-step mark trace (`step.trace`, the executed
        // `hir.test.trace.v1` events in execution order). Decode admits the
        // field for the AV recipes only (`trace_recipe`); the parent judges
        // the sequence the worker actually reported, never a worker verdict.
        (Adapter::Probe, "trace") => true,
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
    console_recipe: bool,
    a2x_recipe: bool,
    trace_recipe: bool,
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
    } else if probe_channels && a2x_recipe {
        // PLAN-892: ac-a2r/hir-scalar-emit-v1 judges the emitted actual
        // file set as byte goldens (the a2r face) — the only probe recipe
        // admitted to the artifacts channel.
        &[
            "completion",
            "probe",
            "exit",
            "av_channel",
            "stdout",
            "stderr",
            "artifacts",
        ]
    } else if probe_channels {
        // PLAN-798 T-07: + exit (value-reporting recipes).
        // PLAN-862: + av_channel (the Unit/value channel discriminant).
        // PLAN-891: + typed (the native typed-entry value oracle).
        &[
            "completion",
            "probe",
            "exit",
            "av_channel",
            "typed",
            "stdout",
            "stderr",
            "trace",
        ]
    } else {
        &["completion", "result", "stdout", "stderr", "artifacts"]
    };
    let f = Fields::new(path, pairs, known)?;

    // Frozen support table: every declared channel must be observable by
    // this adapter, else the check never loads (F-02: unsupported fields
    // may not be skipped after decode).
    let channels: &[&str] = if v2_channels {
        &["result", "stdout", "stderr", "artifacts", "exit", "perf"]
    } else if probe_channels && a2x_recipe {
        &["probe", "exit", "stdout", "stderr", "artifacts"]
    } else if probe_channels {
        &["probe", "exit", "stdout", "stderr", "trace"]
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
                if matches!(completion, Completion::Error { .. }) && !console_recipe {
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

    // PLAN-914: the AV run-step mark trace. Only the AV recipes report
    // `step.trace`, so the field is admitted by recipe (never parsed and
    // then ignored on a path that cannot observe it). The oracle is the
    // exact execution-order event sequence; a declared trace is a
    // success-completion observation like the other streams.
    let trace = match f.sub("trace") {
        None => None,
        Some(tv) => {
            let tp = f.path_of("trace");
            if !probe_channels || !trace_recipe {
                return Err(err(
                    &tp,
                    "`trace` is an AV run-step observation — only the ac-av/* probe recipes report a mark trace",
                ));
            }
            if matches!(completion, Completion::Error { .. }) {
                return Err(err(
                    &tp,
                    "`trace` expectations require completion \"success\" (a failed run carries no judged mark trace)",
                ));
            }
            let pairs = as_obj(tv, &tp)?;
            let tf = Fields::new(&tp, pairs, &["text", "compare"])?;
            let compare = decode_compare(tf.sub("compare"), &format!("{}.compare", tp))?;
            let text = req_str(&tf, "text")?;
            // The AV trace alphabet is the delivered mark set ("a"/"b");
            // anything else cannot be produced and would only ever be a
            // silently-never-matching oracle.
            if let Some(byte) = text.bytes().find(|b| *b != b'a' && *b != b'b') {
                return Err(err(
                    &format!("{}.text", tp),
                    format!(
                        "unknown trace event byte {:?} (the delivered mark alphabet is \"a\" and \"b\")",
                        byte as char
                    ),
                ));
            }
            Some(StreamExpect { text, compare })
        }
    };

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

    // PLAN-862: the AV result-channel discriminant (probe recipes only). A
    // Unit success has NO i32 value, so `exit` alone cannot tell it from an
    // int entry returning 0; the channel name is the observable that does
    // (entry-contract §4/§6). The consumer-side exit mapping (unit → 0) is a
    // SEPARATE contract and is never expressed as an AV value oracle, so any
    // `exit` declared beside `unit` — including 0 — is contradictory and
    // rejected here (an exit oracle would either be a type error or silently
    // re-collapse Unit into zero).
    let av_channel = match f.sub("av_channel") {
        None => None,
        Some(cv) => {
            if !probe_channels {
                return Err(err(
                    &f.path_of("av_channel"),
                    "`av_channel` is a probe observation (the AV result channel of a run step)",
                ));
            }
            let ch = as_str(cv, &f.path_of("av_channel"))?.to_string();
            if ch != "value" && ch != "unit" {
                return Err(err(
                    &f.path_of("av_channel"),
                    format!("unknown av_channel {ch:?} (supported: value, unit)"),
                ));
            }
            if matches!(completion, Completion::Error { .. }) {
                return Err(err(
                    &f.path_of("av_channel"),
                    "`av_channel` requires completion \"success\" (a trap/resource/infra exit is its own channel)",
                ));
            }
            if ch == "unit" && exit.is_some() {
                // The Unit channel has NO i32 payload, so an `exit` oracle
                // would either be a type error or silently re-collapse Unit
                // into 0. The consumer-side exit mapping (entry-contract §6,
                // unit → 0) belongs to the consumer that ALSO prints the
                // channel name; it is never expressed as an AV value oracle.
                return Err(err(
                    &f.path_of("exit"),
                    "a `unit` av_channel carries no i32 payload — declare the channel, not an exit oracle (the consumer's exit mapping is a separate contract)",
                ));
            }
            Some(ch)
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

    // PLAN-891: the typed value oracle (typed recipe only; success only).
    let typed = match f.sub("typed") {
        None => None,
        Some(v) => {
            if !probe_channels {
                return Err(err(
                    &f.path_of("typed"),
                    "`typed` is a probe observation (the typed-entry machine value)",
                ));
            }
            let tpath = f.path_of("typed");
            let tpairs = as_obj(v, &tpath)?;
            let tf = Fields::new(&tpath, tpairs, &["kind", "bits"])?;
            let kind = req_str(&tf, "kind")?.to_string();
            if !matches!(
                kind.as_str(),
                "i32" | "bool" | "i64" | "u64" | "f32" | "f64"
            ) {
                return Err(err(
                    &tf.path_of("kind"),
                    format!("unknown typed kind {kind:?}"),
                ));
            }
            let bits = req_str(&tf, "bits")?.to_string();
            if matches!(completion, Completion::Error { .. }) {
                return Err(err(
                    &f.path_of("typed"),
                    "`typed` requires completion \"success\" (a trap/resource/error frame is judged by its own face)",
                ));
            }
            Some(TypedExpect { kind, bits })
        }
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
        av_channel,
        typed,
        trace,
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

/// PLAN-863: both byte-stream oracles and the result/error axis are mandatory.
/// Parent callers also use this for programmatically constructed checks.
pub fn validate_console_oracle(
    stdout: Option<&StreamExpect>,
    stderr: Option<&StreamExpect>,
    exit: Option<i32>,
    channel: Option<&str>,
    completion: &Completion,
) -> Result<(), String> {
    let (Some(out), Some(err)) = (stdout, stderr) else {
        return Err("console recipe requires explicit stdout and stderr byte assertions".into());
    };
    if out.compare != Compare::Bytes || err.compare != Compare::Bytes {
        return Err("console streams require compare bytes".into());
    }
    if out.text.len().saturating_add(err.text.len()) > 1_048_576 {
        return Err("console expected streams exceed the combined 1 MiB cap".into());
    }
    match completion {
        Completion::Success if exit.is_none() && channel != Some("unit") => {
            Err("console success requires an exit oracle or explicit unit channel".into())
        }
        Completion::Success if channel == Some("unit") && exit.is_some() => {
            Err("unit has no exit payload".into())
        }
        Completion::Error { code: None, .. } => {
            Err("console errors require an explicit diagnostic or trap code".into())
        }
        _ => Ok(()),
    }
}

