//! Contract schemas (`autolang.case/1`) and strict decoding from [`CVal`].
//!
//! Every decode failure carries a field path (e.g. `checks[2].expect`) and a
//! concrete reason; unknown fields, wrong types, invalid id/path forms,
//! unsupported fields (this pilot: variants/relations/recipes/budgets/
//! platform/native adapters) are all errors — nothing is parsed then ignored.

use crate::admission::{check_limit, validate_virtual_path, Limits};
use crate::atom::CVal;

pub const CASE_SCHEMA: &str = "autolang.case/1";

// ------------------------------------------------------------------ types --

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Adapter {
    Avm,
    A2r,
}

impl Adapter {
    pub fn as_str(&self) -> &'static str {
        match self {
            Adapter::Avm => "avm",
            Adapter::A2r => "a2r",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Run,
    Emit,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Run => "run",
            Action::Emit => "emit",
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
}

impl Syntax {
    pub fn extension(&self) -> &'static str {
        match self {
            Syntax::Auto => "at",
            Syntax::AutoScript => "as",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Syntax::Auto => "auto",
            Syntax::AutoScript => "auto-script",
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

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// a2r/emit only: module name passed to the emitter (legacy-compat
    /// metadata; does not affect emitted bytes).
    pub module_name: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options { module_name: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Parse,
    Type,
    Run,
}

impl Stage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Stage::Parse => "parse",
            Stage::Type => "type",
            Stage::Run => "run",
        }
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

#[derive(Debug, Clone, PartialEq)]
pub struct ExpectSpec {
    pub completion: Completion,
    pub result: Option<ResultExpect>,
    pub stdout: Option<StreamExpect>,
    pub stderr: Option<StreamExpect>,
    pub artifacts: Vec<ArtifactExpect>,
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
    if schema != CASE_SCHEMA {
        return Err(err(
            "case.schema",
            format!(
                "unsupported schema {:?} (this pilot implements {})",
                schema, CASE_SCHEMA
            ),
        ));
    }

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

    let source = decode_source(f.req("source")?, "case.source")?;
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
        checks.push(decode_check(cv, &format!("case.checks[{}]", i))?);
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

fn decode_source(v: &CVal, path: &str) -> Result<SourceSpec, ContractError> {
    let pairs = as_obj(v, path)?;
    let f = Fields::new(path, pairs, &["block", "file", "text", "syntax", "path"])?;
    let payload = decode_payload_from(&f)?;
    let syntax = match req_str(&f, "syntax")?.as_str() {
        "auto" => Syntax::Auto,
        "auto-script" => Syntax::AutoScript,
        other => {
            return Err(err(
                &f.path_of("syntax"),
                format!("unknown syntax {:?} (supported: auto, auto-script)", other),
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

fn decode_check(v: &CVal, path: &str) -> Result<CheckSpec, ContractError> {
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
        other => {
            return Err(err(&f.path_of("adapter"), format!("adapter {:?} is not available in this pilot (supported: avm, a2r; native/ac/acc and others belong to Design 35 phase B)", other)))
        }
    };
    let action = match req_str(&f, "action")?.as_str() {
        "run" => Action::Run,
        "emit" => Action::Emit,
        other => {
            return Err(err(
                &f.path_of("action"),
                format!(
                    "action {:?} is not available in this pilot (supported: run, emit)",
                    other
                ),
            ))
        }
    };
    match (adapter, action) {
        (Adapter::Avm, Action::Run) | (Adapter::A2r, Action::Emit) => {}
        (a, act) => {
            return Err(err(
                &f.path_of("action"),
                format!(
                    "illegal combination {}/{} (this pilot: avm/run, a2r/emit)",
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

    let expect = decode_expect(f.req("expect")?, &f.path_of("expect"), adapter)?;

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
    _action: Action,
    path: &str,
) -> Result<Options, ContractError> {
    let pairs = as_obj(v, path)?;
    let known: &[&str] = match adapter {
        Adapter::Avm => &[], // avm/run has no options in v1
        Adapter::A2r => &["module_name"],
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
    Ok(Options { module_name })
}

/// Per-adapter observable-channel support table (Plan 773 F-02, frozen for
/// v1). A channel declared for an adapter that cannot observe it is an
/// ADMISSION error — parsed-then-ignored expectations are forbidden.
pub fn channel_supported(adapter: Adapter, channel: &str) -> bool {
    match (adapter, channel) {
        (Adapter::Avm, "result") => true,
        (Adapter::Avm, "stdout") => true,
        (Adapter::Avm, "stderr") => true,
        (Adapter::A2r, "artifacts") => true,
        _ => false,
    }
}

fn decode_expect(v: &CVal, path: &str, adapter: Adapter) -> Result<ExpectSpec, ContractError> {
    let pairs = as_obj(v, path)?;
    let f = Fields::new(
        path,
        pairs,
        &["completion", "result", "stdout", "stderr", "artifacts"],
    )?;

    // Frozen support table: every declared channel must be observable by
    // this adapter, else the check never loads (F-02: unsupported fields
    // may not be skipped after decode).
    for channel in ["result", "stdout", "stderr", "artifacts"] {
        if f.sub(channel).is_some() && !channel_supported(adapter, channel) {
            return Err(err(
                &f.path_of(channel),
                format!(
                    "`{}` is not an observable channel for {}/{} in this pilot (supported by {}: {})",
                    channel,
                    adapter.as_str(),
                    match adapter {
                        Adapter::Avm => "run",
                        Adapter::A2r => "emit",
                    },
                    channel,
                    match adapter {
                        Adapter::Avm => "result, stdout, stderr",
                        Adapter::A2r => "artifacts",
                    }
                ),
            ));
        }
    }

    let completion = match f.req("completion")? {
        CVal::Str(s) if s == "success" => Completion::Success,
        CVal::Obj(cp) => {
            let cf = Fields::new(&f.path_of("completion"), cp, &["stage", "code", "message"])?;
            let stage = match req_str(&cf, "stage")?.as_str() {
                "parse" => Stage::Parse,
                "type" => Stage::Type,
                "run" => Stage::Run,
                other => {
                    return Err(err(
                        &format!("{}.stage", f.path_of("completion")),
                        format!("unknown stage {:?} (supported: parse, type, run)", other),
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

    Ok(ExpectSpec {
        completion,
        result,
        stdout,
        stderr,
        artifacts,
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
