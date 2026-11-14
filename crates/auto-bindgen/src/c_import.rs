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

/// C importer annotation markers (the metadata IS the header).
pub const C_ANNOTATION_PROVIDER: &str = "auto-ffi-c provider";
pub const C_ANNOTATION_FN: &str = "auto-ffi-c fn";
pub const C_IMPORT_FORMAT: &str = "auto.bindgen.c-header/1";
/// The frozen physical target (pinned to the shared plan constant).
pub const C_TARGET: &str = "x86_64-pc-windows-msvc";

/// The provider header line:
/// `// auto-ffi-c provider <name>/<version> target=<t>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CProviderHeader {
    pub name: String,
    pub version: String,
    pub target: String,
}

/// Parse the provider header annotation (exactly one, required first
/// annotation). The TARGET is an admission axis: a header for another
/// target is a named rejection, never a reinterpretation.
pub fn parse_provider_header(header: &str) -> Result<CProviderHeader, ImportError> {
    let mut found: Option<CProviderHeader> = None;
    for (idx, line) in header.lines().enumerate() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("// ") else {
            continue;
        };
        let Some(body) = rest.trim().strip_prefix(C_ANNOTATION_PROVIDER) else {
            continue;
        };
        if found.is_some() {
            return Err(err(
                "c-import.header",
                format!("duplicate provider annotation (line {})", idx + 1),
            ));
        }
        let mut parts = body.trim().split_whitespace();
        let id_slash = parts.next().ok_or_else(|| {
            err(
                "c-import.header",
                format!(
                    "provider annotation needs `<name>/<version>` (line {})",
                    idx + 1
                ),
            )
        })?;
        let (name, version) = id_slash.split_once('/').ok_or_else(|| {
            err(
                "c-import.header",
                format!(
                    "provider identity must be `<name>/<version>` (line {})",
                    idx + 1
                ),
            )
        })?;
        let mut target = None;
        for part in parts {
            if let Some(v) = part.strip_prefix("target=") {
                target = Some(v.to_string());
            } else {
                return Err(err(
                    "c-import.header",
                    format!(
                        "unknown provider annotation field `{part}` (line {})",
                        idx + 1
                    ),
                ));
            }
        }
        let target = target.ok_or_else(|| {
            err(
                "c-import.header",
                format!("provider annotation needs target= (line {})", idx + 1),
            )
        })?;
        if target != C_TARGET {
            return Err(err(
                "c-import.target",
                format!("header target `{target}` is not the frozen import target `{C_TARGET}`"),
            ));
        }
        found = Some(CProviderHeader {
            name: name.to_string(),
            version: version.to_string(),
            target,
        });
    }
    found.ok_or_else(|| {
        err(
            "c-import.header",
            "missing `// auto-ffi-c provider <name>/<version> target=...` annotation",
        )
    })
}

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

/// Audit the RAW HEADER for shapes OUTSIDE the frozen domain (the audit
/// is about the imported declarations, never system-header internals).
/// These are per-row named rejections (V-10's shape axes): varargs,
/// unions, bitfields, and non-natural record packing. Each is its own
/// code so an old-usage row closes against a NAMED fact, never a silent
/// narrowing.
pub fn audit_shapes(preprocessed: &str) -> Result<(), ImportError> {
    let mut logical = String::new();
    for raw in preprocessed.lines() {
        let line = raw.trim_end_matches('\r').trim();
        if line.is_empty() {
            continue;
        }
        if !logical.is_empty() {
            logical.push(' ');
        }
        logical.push_str(line);
    }
    if logical.contains(", ...") || logical.contains(",...") {
        // a variadic parameter list survived into a declaration
        let evidence = logical
            .lines()
            .find(|l| l.contains("..."))
            .unwrap_or("")
            .trim()
            .chars()
            .take(80)
            .collect::<String>();
        return Err(err(
            "c-import.varargs",
            format!("variadic parameters are outside the frozen C domain: {evidence}"),
        ));
    }
    if logical.contains("union ") {
        let evidence = logical
            .split_whitespace()
            .collect::<Vec<_>>()
            .windows(2)
            .find(|w| w[0] == "union")
            .map(|w| w.join(" "))
            .unwrap_or_default();
        return Err(err(
            "c-import.union",
            format!("union declarations are outside the frozen C domain: {evidence}"),
        ));
    }
    // bitfield: a record member with `:<bits>` — detect `name : n` inside
    // struct bodies (any `identifier : digits` followed by ; or ,)
    for line in preprocessed.lines() {
        let line = line.trim_end_matches('\r').trim();
        let cleaned: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        if cleaned.contains(':') {
            // skip labels/ternaries heuristically: only flag inside
            // struct-adjacent context by checking neighboring `struct`
            if line.contains("struct")
                && cleaned.contains(":")
                && cleaned
                    .split(':')
                    .nth(1)
                    .map(|r| {
                        r.chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            {
                return Err(err(
                    "c-import.bitfield",
                    "bitfield members are outside the frozen C domain",
                ));
            }
        }
    }
    // packing: any `#pragma pack` switches records off the frozen
    // natural-align layout
    for line in preprocessed.lines() {
        let line = line.trim_end_matches('\r').trim();
        if line.starts_with("#pragma") && line.contains("pack") {
            return Err(err(
                "c-import.packing",
                "#pragma pack changes record layout away from the frozen natural-align packing",
            ));
        }
    }
    Ok(())
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
    Function {
        name: String,
        ret: CRet,
        params: Vec<CParam>,
    },
    /// `typedef struct <tag> <name>;` — an opaque handle typedef (the
    /// incomplete-type object; the provider owns the lifecycle).
    Opaque { name: String },
    /// `static inline <scalar> name(params) { body }` — the header's own
    /// implementation; the importer re-emits the (macro-expanded) body as
    /// a plain extern definition in the generated provider shim.
    Inline {
        name: String,
        signature: String,
        body: String,
    },
}

/// A function return shape.
#[derive(Debug, Clone, PartialEq)]
pub enum CRet {
    I32,
    Bool,
    /// an opaque handle typedef name (the resource Handle out lane).
    Opaque(String),
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
    /// an opaque handle typedef parameter (create/get/destroy lifecycle;
    /// the header's opaque object, provider-owned).
    Handle {
        ty: String,
    },
    /// the i64 ctx seed following a callback lane (physical only).
    CtxSeed,
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
        if line.ends_with(';')
            || line == "}"
            || (line.ends_with('}') && current.contains("static inline"))
        {
            lines.push(std::mem::take(&mut current));
        }
    }
    // First pass: static-inline definitions (bodies carry braces; they
    // never terminate on ';' — collect them so the collapse cannot glue
    // them onto later declarations).
    {
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("static inline ") && line.contains('(') && line.contains('{') {
                let mut depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                let mut whole = line.to_string();
                let mut j = i;
                while depth > 0 && j + 1 < lines.len() {
                    j += 1;
                    whole.push(' ');
                    whole.push_str(lines[j].trim());
                    depth +=
                        lines[j].matches('{').count() as i32 - lines[j].matches('}').count() as i32;
                }
                if let Some(decl) = parse_inline(&whole) {
                    out.push(decl);
                }
                // remove the consumed lines from the second pass
                for k in i..=j {
                    lines[k] = String::new();
                }
                i = j + 1;
                continue;
            }
            i += 1;
        }
    }
    for line in &lines {
        if line.trim().is_empty() {
            continue;
        }
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
        // opaque handle: `typedef struct <tag> <name>;` (no body)
        if let Some(rest) = line.strip_prefix("typedef struct ") {
            if !rest.contains('{') {
                let rest = rest.trim().trim_end_matches(';').trim();
                if let Some((tag, name)) = rest.rsplit_once(' ') {
                    if is_c_ident(tag.trim()) && is_c_ident(name.trim()) {
                        out.push(CDecl::Opaque {
                            name: name.trim().to_string(),
                        });
                        continue;
                    }
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
    // the DELIVERED callback physical contract: long long (*)(long long, long long)
    if ret != "long" {
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
    if normalized == ["long", "long"] {
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
    let ret = if ret == "int32_t" || ret == "int" {
        CRet::I32
    } else if ret == "bool" {
        CRet::Bool
    } else {
        // an opaque handle typedef return, tolerating the pointer star
        let ret = ret.trim_end_matches('*');
        if is_c_ident(ret) {
            CRet::Opaque(ret.to_string())
        } else {
            return None;
        }
    };
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
            // a bare typedef'd parameter: callback OR opaque handle —
            // the projection resolves it against the parsed decl sets;
            // scalar keywords fall through to the scalar arm below.
            if tokens.len() == 2
                && is_c_ident(&tokens[0])
                && !matches!(
                    tokens[0].as_str(),
                    "int32_t" | "int" | "bool" | "uint64_t" | "uint8_t" | "int64_t" | "void"
                )
            {
                params.push(CParam::Callback {
                    ty: tokens[0].clone(),
                });
                i += 1;
                continue;
            }
            // `long long ctx` — the ctx seed that follows a callback
            // lane: a PHYSICAL frame argument, never a logical lane.
            if tokens.len() == 3 && tokens[0] == "long" && tokens[1] == "long" {
                params.push(CParam::CtxSeed);
                i += 1;
                continue;
            }
            // `<opaque>* h` — a bare handle-typedef pointer parameter
            if tokens.len() == 2 && tokens[0].ends_with('*') {
                let ty = tokens[0].trim_end_matches('*').to_string();
                if is_c_ident(&ty) {
                    params.push(CParam::Handle { ty });
                    i += 1;
                    continue;
                }
                return None;
            }
            // `struct <tag>* h` — a handle-shaped pointer parameter
            if tokens.len() == 3 && tokens[0] == "struct" && tokens[1].ends_with('*') {
                params.push(CParam::Handle {
                    ty: tokens[1].trim_end_matches('*').to_string(),
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
        ret,
        params,
    })
}

/// `static inline int32_t name(params) { body }` — split the signature
/// from the body at the FIRST brace.
fn parse_inline(whole: &str) -> Option<CDecl> {
    let open = whole.find('{')?;
    let close = whole.rfind('}')?;
    if close <= open {
        return None;
    }
    let signature = whole[..open].trim().to_string();
    let body = whole[open + 1..close].trim().to_string();
    // the name is the identifier right before the parameter '('
    let paren = signature.find('(')?;
    let head = signature[..paren].trim_end();
    let name = head.split_whitespace().last()?.to_string();
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    Some(CDecl::Inline {
        name,
        signature,
        body,
    })
}

/// Generate the provider shim for annotated static-inline entries: each
/// body re-emitted as a plain extern definition (the binding symbol =
/// the header's own name; macro expansion already happened).
pub fn generate_inline_shim(decls: &[CDecl], entries: &[CEntry]) -> Option<String> {
    let mut out = String::new();
    for entry in entries {
        let Some(CDecl::Inline {
            name,
            signature,
            body,
        }) = decls
            .iter()
            .find(|d| matches!(d, CDecl::Inline { name, .. } if *name == entry.name))
        else {
            continue;
        };
        // strip the `static inline ` prefix for the extern definition
        let plain = signature
            .strip_prefix("static inline ")
            .unwrap_or(signature);
        out.push_str(&format!(
            "{plain} {{ {body} }}
"
        ));
        let _ = name;
    }
    if out.is_empty() {
        None
    } else {
        Some(format!(
            "// Generated by auto-bindgen c_import ({C_IMPORT_FORMAT}) — static-inline shim.\n// DO NOT EDIT.\n#include <stdint.h>\n{out}"
        ))
    }
}

/// Extract the enum constants (standalone bindings for consumers).
pub fn extract_constants(decls: &[CDecl]) -> Vec<(String, i32)> {
    let mut out = Vec::new();
    for d in decls {
        if let CDecl::Enum { consts, .. } = d {
            out.extend(consts.iter().cloned());
        }
    }
    out
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
    let opaque_set: std::collections::BTreeSet<String> = decls
        .iter()
        .filter_map(|d| match d {
            CDecl::Opaque { name } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let mut functions = Vec::new();
    let mut declarations = Vec::new();
    let mut effects = Vec::new();
    for entry in entries {
        let decl = decls
            .iter()
            .find(|d| {
                matches!(d, CDecl::Function { name, .. } if *name == entry.name)
                    || matches!(d, CDecl::Inline { name, .. } if *name == entry.name)
            })
            .ok_or_else(|| {
                err(
                    "c-import.missing-decl",
                    format!(
                        "annotated fn `{}` has no matching admitted declaration in the preprocessed header",
                        entry.name
                    ),
                )
            })?;
        // an inline entry's lanes come from its signature (scalar-only for
        // the frozen domain); pointer/record shapes inside inlines are a
        // named domain rejection.
        let params: Vec<CParam> = match decl {
            CDecl::Function { params, .. } => params.clone(),
            CDecl::Inline { signature, .. } => {
                let open = signature.find('(').unwrap_or(0);
                let close = signature.rfind(')').unwrap_or(open);
                let text = signature[open + 1..close].trim();
                if text.is_empty() || text == "void" {
                    Vec::new()
                } else {
                    text.split(',')
                        .map(|p| {
                            let first = p.trim().split_whitespace().next().unwrap_or("");
                            match first {
                                "int32_t" | "int" => CParam::ScalarI32,
                                "bool" => CParam::ScalarBool,
                                _ => CParam::ScalarI32, // narrowed below by the lane check
                            }
                        })
                        .collect()
                }
            }
            _ => unreachable!("filtered above"),
        };
        if matches!(decl, CDecl::Inline { .. })
            && params
                .iter()
                .any(|p| !matches!(p, CParam::ScalarI32 | CParam::ScalarBool))
        {
            return Err(err(
                "c-import.domain",
                format!(
                    "inline fn `{}` carries a non-scalar parameter (outside the frozen inline domain)",
                    entry.name
                ),
            ));
        }
        // The scalar spine is IGNORED for resource entries (the 939
        // convention): the resource sig is the authoritative shape.
        let mut lanes = Vec::new();
        for p in &params {
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
                    // a `const <ty>* p` parameter: a RECORD view when the
                    // typedef is an admitted record, an OPAQUE HANDLE when
                    // it is an opaque typedef — resolved against the parsed
                    // sets, never guessed.
                    match records.get(record) {
                        Some(fields) => {
                            lanes.push(ResKind::Rec {
                                rec: record.clone(),
                                fields: fields.clone(),
                                role: ViewRole::Const,
                            });
                        }
                        None if opaque_set.contains(record) => {
                            lanes.push(ResKind::Handle {
                                provider: header_name.to_string(),
                            });
                        }
                        None => {
                            return Err(err(
                                "c-import.domain",
                                format!("fn `{}` views `{record}` which is neither an admitted record nor an opaque typedef in this header", entry.name),
                            ));
                        }
                    }
                }
                CParam::Handle { ty } => {
                    if !opaque_set.contains(ty) {
                        return Err(err(
                            "c-import.domain",
                            format!(
                                "fn `{}` takes handle `{ty}` which is not an opaque typedef in this header",
                                entry.name
                            ),
                        ));
                    }
                    lanes.push(ResKind::Handle {
                        provider: header_name.to_string(),
                    });
                }
                CParam::Callback { ty } => {
                    if !callbacks.contains(&ty.clone()) {
                        return Err(err(
                            "c-import.domain",
                            format!(
                                "fn `{}` takes callback `{ty}` which is not the admitted fn(i32,i32)->i32 typedef",
                                entry.name
                            ),
                        ));
                    }
                    lanes.push(ResKind::Callback { ctx: true });
                }
                CParam::CtxSeed => {
                    // the ctx seed IS a logical lane (939 convention: an
                    // i32 source argument physically widened to i64 after
                    // the callback lane).
                    lanes.push(ResKind::I32);
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
                    if Some(&p.clone()) != params.last() {
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
            CForm::PureI32 => {
                // an opaque C return turns the entry into a handle factory
                // (the resource Handle out lane); the SCALAR spine stays
                // the ignored empty convention.
                let ret_lane = match decl {
                    CDecl::Function {
                        ret: CRet::Opaque(ty),
                        ..
                    } => {
                        if !opaque_set.contains(ty) {
                            return Err(err(
                                "c-import.domain",
                                format!(
                                    "fn `{}` returns handle `{ty}` which is not an opaque typedef in this header",
                                    entry.name
                                ),
                            ));
                        }
                        ResOut::Handle {
                            provider: header_name.to_string(),
                        }
                    }
                    _ => ResOut::I32,
                };
                (
                    FnForm::Pure {
                        params: vec![],
                        ret: LaneKind::I32,
                    },
                    None,
                    ResourceForm::Pure { ret: ret_lane },
                    format!("pure:{}", entry.name),
                )
            }
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
