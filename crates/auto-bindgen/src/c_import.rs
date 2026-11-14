//! PLAN-967 T-10 (AC-08): the C importer — REAL preprocessing (the
//! caller runs `cl /EP` via `ace-abi`'s ctool; the preprocessor owns
//! macros, includes and the target), then a BOUNDED declaration parser
//! over the preprocessed stream for the frozen C domain: scalar
//! functions, records (the shared FieldKind closure), enums (integer
//! constants), opaque handles, fn-pointer callbacks, and pointer+len
//! text views. Everything else (variadics, unions, bitfields, arrays,
//! wider pointer graphs) is a NAMED domain rejection — never silently
//! narrowed. The projection is the same `ace.foreign.manual/2` admission
//! every other lane consumes; no second FFI world.
//!
//! Declarations are SELECTED by annotation lines in the header
//! (`// auto-ffi-c fn <name> ...`) exactly like the Rust importer — the
//! metadata lives in the provider header itself, and every annotated
//! declaration must really exist in the preprocessed output with an
//! admitted shape.

use crate::rust_import::ImportError;
use ace_foreign_contract::manual::{ManualDecl, ManualManifest, MANUAL_SCHEMA_V2, SOURCE_CAP};
use ace_foreign_contract::resource::{
    FieldKind, ResKind, ResOut, ResourceForm, ResourceSig, ViewRole, RESOURCE_CAP,
    RESOURCE_SIG_FORMAT,
};
use ace_foreign_contract::{
    ErrorModel, FactsSource, FnFact, FnForm, InterfaceFacts, LaneKind, FACTS_FORMAT,
};

/// C importer annotation marker (the metadata IS the header).
pub const C_ANNOTATION_FN: &str = "auto-ffi-c fn";
pub const C_IMPORT_FORMAT: &str = "auto.bindgen.c-header/1";

fn err(code: &'static str, message: impl Into<String>) -> ImportError {
    ImportError {
        code,
        message: message.into(),
    }
}

/// The frozen annotation grammar for one C entry:
/// `// auto-ffi-c fn <name> form=pure ret=i32|bool | form=fallible errors=n:-c[,..]`
#[derive(Debug, Clone, serde::Serialize)]
pub struct CEntry {
    pub name: String,
    pub form: CForm,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum CForm {
    PureI32,
    PureBool,
    FallibleI32 { errors: Vec<(String, i32)> },
}

/// Parse the ANNOTATION lines out of the RAW header (before
/// preprocessing — comments vanish under /EP).
pub fn parse_annotations(header: &str) -> Result<Vec<CEntry>, ImportError> {
    let mut out = Vec::new();
    for (idx, line) in header.lines().enumerate() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("// ") else {
            continue;
        };
        let Some(body) = rest.trim().strip_prefix(C_ANNOTATION_FN) else {
            continue;
        };
        let mut parts = body.trim().split_whitespace();
        let name = parts.next().unwrap_or_default().to_string();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(err(
                "c-import.annotation",
                format!("fn annotation needs a C name (line {})", idx + 1),
            ));
        }
        let mut form = None;
        let mut ret = None;
        let mut errors: Vec<(String, i32)> = Vec::new();
        for part in parts {
            if let Some(v) = part.strip_prefix("form=") {
                form = Some(v.to_string());
            } else if let Some(v) = part.strip_prefix("ret=") {
                ret = Some(v.to_string());
            } else if let Some(v) = part.strip_prefix("errors=") {
                for pair in v.split(',') {
                    let (n, c) = pair.split_once(':').ok_or_else(|| {
                        err(
                            "c-import.annotation",
                            format!("error codes must be `name:-code` (line {})", idx + 1),
                        )
                    })?;
                    let c: i32 = c.parse().map_err(|_| {
                        err(
                            "c-import.annotation",
                            format!("error code `{c}` is not an i32 (line {})", idx + 1),
                        )
                    })?;
                    if c >= 0 {
                        return Err(err(
                            "c-import.annotation",
                            format!("status codes must be negative (line {})", idx + 1),
                        ));
                    }
                    errors.push((n.to_string(), c));
                }
            } else {
                return Err(err(
                    "c-import.annotation",
                    format!("unknown fn annotation field `{part}` (line {})", idx + 1),
                ));
            }
        }
        let form = match (form.as_deref(), ret.as_deref()) {
            (Some("pure"), Some("i32")) => CForm::PureI32,
            (Some("pure"), Some("bool")) => CForm::PureBool,
            (Some("fallible"), None) if !errors.is_empty() => CForm::FallibleI32 { errors },
            (f, r) => {
                return Err(err(
                    "c-import.domain",
                    format!("fn `{name}`: form `{f:?}` ret `{r:?}` is outside the frozen C domain (line {})", idx + 1),
                ))
            }
        };
        if out.iter().any(|e: &CEntry| e.name == name) {
            return Err(err(
                "c-import.duplicate",
                format!("duplicate fn annotation `{name}`"),
            ));
        }
        out.push(CEntry { name, form });
    }
    if out.is_empty() {
        return Err(err(
            "c-import.annotation",
            "no `// auto-ffi-c fn ...` annotations: an imported header exports at least one function",
        ));
    }
    Ok(out)
}

/// One declared shape extracted from the preprocessed stream.
#[derive(Debug, Clone, PartialEq)]
pub enum CDecl {
    /// `typedef struct { fields } name;` — the FieldKind closure.
    Record {
        name: String,
        fields: Vec<(String, FieldKind)>,
    },
    /// `typedef enum { A [= n], ... } name;` — i32 constants.
    Enum {
        name: String,
        consts: Vec<(String, i32)>,
    },
    /// `typedef int32_t (*name)(int32_t, int32_t);` — the source-domain
    /// callback type `fn(i32, i32) -> i32`.
    Callback { name: String },
    /// `extern <scalar> name(params);` — an annotated function.
    Function { name: String, params: Vec<CParam> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CParam {
    ScalarI32,
    ScalarBool,
    /// `const uint8_t* data, uint64_t len` — the ViewText pair.
    ViewText,
    /// `const <record>* p` / `<record>* p` — a record view (role by the
    /// annotation contract: importer emits Const; Mut/Out join with the
    /// ownership annotations at T-13, never by guesswork).
    RecordView {
        record: String,
    },
    /// a callback typedef parameter.
    Callback {
        ty: String,
    },
    /// a trailing `int32_t* out` — the fallible value-out lane (required
    /// by fallible annotations, rejected for pure ones).
    OutI32,
}

/// Parse the PREPROCESSED text for the frozen declaration shapes. The
/// parser is strictly bounded: whole declarations are matched on single
/// logical lines (the preprocessor has already folded macros, comments
/// and line splices); anything that is not one of the admitted shapes is
/// simply not an importable declaration, and annotated-but-unmatched
/// entries are named errors at projection time.
pub fn parse_declarations(preprocessed: &str) -> Vec<CDecl> {
    let mut out = Vec::new();
    let mut records: std::collections::BTreeMap<String, Vec<(String, FieldKind)>> =
        Default::default();
    let mut lines: Vec<String> = Vec::new();
    // collapse the stream into logical lines (multi-line braces for
    // records/enums)
    let mut current = String::new();
    for raw in preprocessed.lines() {
        // cl /EP emits CRLF; strip the carriage before any shape match.
        let line = raw.trim_end_matches('\r').trim();
        if line.is_empty() {
            continue;
        }
        // preprocessor residue (#pragma/#line markers survive /EP) is not
        // declaration content; dropping it keeps directive tails from
        // gluing onto the next declaration.
        if line.starts_with('#') {
            continue;
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(line);
        // Declarations END with ';' (a record's '} name;' may wrap
        // lines; pushing on ANY '}' would truncate the declarator). A
        // LONE '}' closes a non-declaration region (system-header
        // residue) and also breaks the accumulation so it cannot swallow
        // later declarations.
        if line.ends_with(';') || line == "}" {
            lines.push(std::mem::take(&mut current));
        }
    }
    for line in &lines {
        let line = line.trim();
        // record: typedef struct [tag] { f T; ... } name;
        if let Some(rest) = line.strip_prefix("typedef struct ") {
            if let Some((body, name)) = split_brace_init(rest) {
                if let Some(fields) = parse_record_fields(&body) {
                    records.insert(name.clone(), fields.clone());
                    out.push(CDecl::Record { name, fields });
                    continue;
                }
            }
        }
        // enum: typedef enum [tag] { A [= n], ... } name;
        if let Some(rest) = line.strip_prefix("typedef enum ") {
            if let Some((body, name)) = split_brace_init(rest) {
                if let Some(consts) = parse_enum_consts(&body) {
                    out.push(CDecl::Enum { name, consts });
                    continue;
                }
            }
        }
        // callback: typedef int32_t (*name)(int32_t a, int32_t b);
        if let Some(rest) = line.strip_prefix("typedef ") {
            if let Some(decl) = parse_callback(rest) {
                out.push(decl);
                continue;
            }
        }
        // function: [extern] <ret> name(params);
        if let Some(decl) = parse_function(line) {
            out.push(decl);
        }
    }
    out
}

/// `struct { body } name;` tail after `typedef struct [tag] `.
fn split_brace_init(rest: &str) -> Option<(String, String)> {
    let open = rest.find('{')?;
    let close = rest.rfind('}')?;
    if close < open {
        return None;
    }
    let tag_body = &rest[..open];
    let body = rest[open + 1..close].to_string();
    let name = rest[close + 1..]
        .trim()
        .trim_end_matches(';')
        .trim()
        .to_string();
    // optional tag before the brace
    let _tag = tag_body.trim();
    if name.is_empty() || !is_c_ident(&name) {
        return None;
    }
    Some((body, name))
}

fn is_c_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .next()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false)
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn parse_record_fields(body: &str) -> Option<Vec<(String, FieldKind)>> {
    let mut fields = Vec::new();
    for member in body.split(';') {
        let member = member.trim();
        if member.is_empty() {
            continue;
        }
        let (ty, name) = member.rsplit_once(' ')?;
        let kind = match ty.trim() {
            "int" | "int32_t" | "signed int" => FieldKind::I32,
            "unsigned int" | "uint32_t" => FieldKind::U32,
            "int64_t" | "long long" => FieldKind::I64,
            "uint64_t" | "unsigned long long" => FieldKind::U64,
            "double" => FieldKind::F64,
            "bool" => FieldKind::Bool,
            "char" => FieldKind::Char,
            _ => return None,
        };
        if !is_c_ident(name.trim()) {
            return None;
        }
        fields.push((name.trim().to_string(), kind));
    }
    if fields.is_empty() {
        None
    } else {
        Some(fields)
    }
}

fn parse_enum_consts(body: &str) -> Option<Vec<(String, i32)>> {
    let mut consts = Vec::new();
    let mut next: i32 = 0;
    for arm in body.split(',') {
        let arm = arm.trim();
        if arm.is_empty() {
            continue;
        }
        let (name, value) = match arm.split_once('=') {
            Some((n, v)) => {
                let v: i32 = v.trim().parse().ok()?;
                (n.trim().to_string(), v)
            }
            None => (arm.to_string(), next),
        };
        if !is_c_ident(&name) {
            return None;
        }
        consts.push((name, value));
        next = value.wrapping_add(1);
    }
    if consts.is_empty() {
        None
    } else {
        Some(consts)
    }
}

/// after `typedef `: `int32_t (*name)(int32_t a, int32_t b);`
fn parse_callback(rest: &str) -> Option<CDecl> {
    let rest = rest.trim().trim_end_matches(';').trim();
    let ret = rest.split_whitespace().next()?;
    if ret != "int32_t" && ret != "int" {
        return None;
    }
    // `int32_t (*name)(int32_t a, int32_t b)` — the FIRST paren pair
    // wraps `*name`; the SECOND carries the parameter list.
    let open = rest.find('(')?;
    let close = rest.find(')')?;
    if close <= open {
        return None;
    }
    let inner = &rest[open + 1..close]; // "*name"
    let name = inner.strip_prefix('*')?.trim();
    if !is_c_ident(name) {
        return None;
    }
    let tail = rest[close + 1..].trim();
    let params = tail.strip_prefix('(')?.strip_suffix(')')?.trim();
    let normalized: Vec<String> = params
        .split(',')
        .map(|p| p.trim().split_whitespace().next().unwrap_or("").to_string())
        .collect();
    if normalized == ["int32_t", "int32_t"] || normalized == ["int", "int"] {
        Some(CDecl::Callback {
            name: name.to_string(),
        })
    } else {
        None
    }
}

fn parse_function(line: &str) -> Option<CDecl> {
    let line = line.trim().trim_end_matches(';').trim();
    let line = line.strip_prefix("extern ").unwrap_or(line);
    let open = line.find('(')?;
    let close = line.rfind(')')?;
    if close <= open {
        return None;
    }
    let head = line[..open].trim();
    let (ret, name) = head.rsplit_once(' ')?;
    if !is_c_ident(name) {
        return None;
    }
    if ret != "int32_t" && ret != "int" && ret != "bool" {
        return None;
    }
    let params_text = line[open + 1..close].trim();
    let mut params = Vec::new();
    if !params_text.is_empty() && params_text != "void" {
        // normalize to the cl /EP spellings: `<type tokens...> [star] <name>`
        let raw: Vec<Vec<String>> = params_text
            .split(',')
            .map(|p| {
                p.split_whitespace()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut i = 0;
        while i < raw.len() {
            let tokens = &raw[i];
            let Some(first) = tokens.first() else {
                return None;
            };
            if first == "..." {
                return None; // variadic: named domain rejection upstream
            }
            // `const uint8_t* data, uint64_t len` -> the ViewText pair
            if first == "const"
                && tokens.len() == 3
                && (tokens[1] == "uint8_t*" || tokens[1] == "unsigned")
            {
                let len = raw.get(i + 1)?;
                if len.first().map(|t| t == "uint64_t").unwrap_or(false) && len.len() == 2 {
                    params.push(CParam::ViewText);
                    i += 2;
                    continue;
                }
                return None;
            }
            // `const <record>* p` -> a record view
            if first == "const" && tokens.len() == 3 && tokens[1].ends_with('*') {
                let record = tokens[1].trim_end_matches('*').to_string();
                if is_c_ident(&record) {
                    params.push(CParam::RecordView { record });
                    i += 1;
                    continue;
                }
                return None;
            }
            // `<ty>* out` -> the fallible out lane
            if tokens.len() == 2 && (tokens[0] == "int32_t*" || tokens[0] == "int*") {
                params.push(CParam::OutI32);
                i += 1;
                continue;
            }
            // a bare typedef'd callback parameter
            if tokens.len() == 2 && is_c_ident(&tokens[0]) {
                params.push(CParam::Callback {
                    ty: tokens[0].clone(),
                });
                i += 1;
                continue;
            }
            if tokens.len() == 2 {
                match tokens[0].as_str() {
                    "int32_t" | "int" => params.push(CParam::ScalarI32),
                    "bool" => params.push(CParam::ScalarBool),
                    _ => return None,
                }
                i += 1;
                continue;
            }
            return None;
        }
    }
    Some(CDecl::Function {
        name: name.to_string(),
        params,
    })
}

/// Project the parsed declarations + annotations onto the shared
/// `ace.foreign.manual/2` manifest (the same admission every lane uses).
pub fn to_manual_manifest(
    header_name: &str,
    entries: &[CEntry],
    decls: &[CDecl],
    review_id: &str,
    header_digest: &str,
) -> Result<ManualManifest, ImportError> {
    let records: std::collections::BTreeMap<String, Vec<(String, FieldKind)>> = decls
        .iter()
        .filter_map(|d| match d {
            CDecl::Record { name, fields } => Some((name.clone(), fields.clone())),
            _ => None,
        })
        .collect();
    let callbacks: std::collections::BTreeSet<String> = decls
        .iter()
        .filter_map(|d| match d {
            CDecl::Callback { name } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let mut functions = Vec::new();
    let mut declarations = Vec::new();
    let mut effects = Vec::new();
    for entry in entries {
        let decl = decls
            .iter()
            .find(|d| matches!(d, CDecl::Function { name, .. } if *name == entry.name))
            .ok_or_else(|| {
                err(
                    "c-import.missing-decl",
                    format!(
                        "annotated fn `{}` has no matching admitted declaration in the preprocessed header",
                        entry.name
                    ),
                )
            })?;
        let CDecl::Function { params, .. } = decl else {
            unreachable!("filtered above");
        };
        // The scalar spine is IGNORED for resource entries (the 939
        // convention): the resource sig is the authoritative shape.
        let mut lanes = Vec::new();
        for p in params {
            match p {
                CParam::ScalarI32 => {
                    lanes.push(ResKind::I32);
                }
                CParam::ScalarBool => {
                    lanes.push(ResKind::Bool);
                }
                CParam::ViewText => {
                    lanes.push(ResKind::ViewText);
                }
                CParam::RecordView { record } => {
                    let fields = records.get(record).ok_or_else(|| {
                        err(
                            "c-import.domain",
                            format!("fn `{}` views record `{record}` which is not an admitted record in this header", entry.name),
                        )
                    })?;
                    lanes.push(ResKind::Rec {
                        rec: record.clone(),
                        fields: fields.clone(),
                        role: ViewRole::Const,
                    });
                }
                CParam::Callback { ty } => {
                    if !callbacks.contains(ty) {
                        return Err(err(
                            "c-import.domain",
                            format!(
                                "fn `{}` takes callback `{ty}` which is not the admitted fn(i32,i32)->i32 typedef",
                                entry.name
                            ),
                        ));
                    }
                    lanes.push(ResKind::Callback { ctx: false });
                }
                CParam::OutI32 => {
                    // the fallible value-out lane: legal ONLY as the final
                    // parameter of a fallible annotation; the resource form
                    // carries it, never the lane list.
                    if !matches!(entry.form, CForm::FallibleI32 { .. }) {
                        return Err(err(
                            "c-import.domain",
                            format!(
                                "fn `{}` has an `int32_t* out` parameter but is not annotated fallible",
                                entry.name
                            ),
                        ));
                    }
                    if Some(p) != params.last() {
                        return Err(err(
                            "c-import.domain",
                            format!(
                                "fn `{}` must carry its `int32_t* out` as the final parameter",
                                entry.name
                            ),
                        ));
                    }
                }
            }
        }
        if matches!(entry.form, CForm::FallibleI32 { .. })
            && !matches!(params.last(), Some(CParam::OutI32))
        {
            return Err(err(
                "c-import.domain",
                format!(
                    "fallible fn `{}` must declare the trailing `int32_t* out` parameter",
                    entry.name
                ),
            ));
        }
        let (form, error_model, res_form, effect) = match &entry.form {
            CForm::PureI32 => (
                FnForm::Pure {
                    params: vec![],
                    ret: LaneKind::I32,
                },
                None,
                ResourceForm::Pure { ret: ResOut::I32 },
                format!("pure:{}", entry.name),
            ),
            CForm::PureBool => (
                FnForm::Pure {
                    params: vec![],
                    ret: LaneKind::Bool,
                },
                None,
                ResourceForm::Pure { ret: ResOut::Bool },
                format!("pure:{}", entry.name),
            ),
            CForm::FallibleI32 { errors } => {
                let mut codes: Vec<(i32, String)> =
                    errors.iter().map(|(n, c)| (*c, n.clone())).collect();
                codes.sort();
                codes.dedup();
                (
                    FnForm::Fallible {
                        ctx: None,
                        params: vec![],
                        outs: vec![LaneKind::I32],
                    },
                    Some(ErrorModel {
                        kind: "status-i32".into(),
                        codes,
                    }),
                    ResourceForm::Fallible {
                        ctx: None,
                        out: Some(ResOut::I32),
                    },
                    "checked".into(),
                )
            }
        };
        functions.push(FnFact {
            symbol: entry.name.clone(),
            form,
            error_model,
            semantic_ref: None,
            resource: Some(ResourceSig {
                format: RESOURCE_SIG_FORMAT.into(),
                params: lanes,
                form: res_form,
                atomic: false,
                noescape: true,
                typed_peer: false,
            }),
        });
        effects.push(effect);
        declarations.push(ManualDecl {
            context_symbol: None,
            source_name: entry.name.clone(),
            symbol: entry.name.clone(),
        });
    }
    let m = ManualManifest {
        schema: MANUAL_SCHEMA_V2.into(),
        review_id: review_id.into(),
        provider: header_name.to_string(),
        header_digest: header_digest.into(),
        // the C lane's physical table digest is the toolchain+header
        // identity; T-14 replaces this with the full BuildIdentity key.
        physical_table_digest: header_digest.into(),
        target: ace_foreign_contract::plan::TARGET.into(),
        profile: ace_foreign_contract::plan::PROFILE.into(),
        callconv: "windows-x64".into(),
        direction: "source-import".into(),
        facts: InterfaceFacts {
            format: FACTS_FORMAT.into(),
            logical: format!("c/{header_name}"),
            content_version: ace_foreign_contract::blake3_hex(
                format!(
                    "{}:{}",
                    header_digest,
                    serde_json::to_string(entries).unwrap_or_default()
                )
                .as_bytes(),
            ),
            source: FactsSource::Typed {
                id: format!("auto-bindgen/c-import/{header_name}"),
            },
            functions,
            effects,
            capabilities: vec![SOURCE_CAP.into(), RESOURCE_CAP.into()],
        },
        declarations,
    };
    m.validate()
        .map_err(|e| err("c-import.manifest", format!("manifest validation: {e}")))?;
    Ok(m)
}
