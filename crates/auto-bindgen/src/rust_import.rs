//! PLAN-967 F0 (T-F0-I): Rust public metadata importer.
//!
//! An annotated Rust provider source is the single import input: the
//! importer derives (a) a neutral provider manifest, (b) a generated
//! C-compatible shim source, and (c) the shared `ace.foreign.manual/2`
//! manifest used by the ACE checked frontdoor. No per-function manifest
//! is hand-written; anything outside the frozen F0 domain is rejected
//! with a named code instead of silently narrowed.
//!
//! Frozen F0 domain: non-generic public Rust free functions taking one
//! borrowed `&str` (physical `data + byte_len`), returning `i32`/`bool`
//! (pure) or an `i32` status (fallible, negative named codes). Sync,
//! same-thread, noescape. Full C/Rust importers remain PLAN-967 T-09..11.

use ace_foreign_contract::{
    manual::{ManualDecl, ManualManifest, MANUAL_SCHEMA_V2, SOURCE_CAP},
    resource::{ResKind, ResOut, ResourceForm, ResourceSig, RESOURCE_CAP, RESOURCE_SIG_FORMAT},
    ErrorModel, FactsSource, FnFact, FnForm, InterfaceFacts, LaneKind, FACTS_FORMAT,
};
use serde::{Deserialize, Serialize};

/// Manifest format identity for the neutral (pre-manifest) provider form.
pub const IMPORT_FORMAT: &str = "auto.bindgen.rust-provider/1";
/// Versioned source annotation markers (the metadata IS the provider file).
pub const ANNOTATION_PROVIDER: &str = "auto-ffi provider";
pub const ANNOTATION_FN: &str = "auto-ffi fn";
/// F0 physical target (frozen). MUST equal
/// `ace_foreign_contract::plan::TARGET` — the projection asserts it and
/// falls back to the shared constant, so a silent divergence cannot ship.
pub const F0_TARGET: &str = "x86_64-pc-windows-msvc";

/// Named import rejection. Codes are stable contract spellings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportError {
    pub code: &'static str,
    pub message: String,
}

fn err(code: &'static str, message: impl Into<String>) -> ImportError {
    ImportError {
        code,
        message: message.into(),
    }
}

/// Neutral provider manifest derived from the annotated Rust source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustProviderManifest {
    pub format: String,
    pub provider: String,
    pub interface_version: String,
    pub target: String,
    pub profile: String,
    pub functions: Vec<RustFnEntry>,
}

/// One imported provider function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustFnEntry {
    /// The Rust source's own `pub fn` name (the Auto source calls this).
    pub rust_name: String,
    /// Versioned logical symbol, e.g. `f0.text.byte_len.v1`.
    pub symbol: String,
    /// The `#[no_mangle]` C export name (the sanitized symbol).
    pub export: String,
    pub form: RustFnForm,
    /// T-11: the ordered parameter shapes (`view_utf8 | i32 | bool`).
    /// Exactly one `view_utf8` maximum; scalars ride as C lanes.
    pub params: Vec<RustParam>,
    /// T-05 F1 (AC-INT-1026 / IF-CALL): the checked TAG PARAMETER FRAME
    /// version (`p967.frame/1`) — receiver + positionals + kwargs ride
    /// the view wire in a versioned canonical encoding; the AV admission
    /// validates the frame BEFORE any dispatch and the generated shim
    /// decodes it into typed values for the provider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    /// Provider-owned opaque resource operation. Ordinary F0 manifests
    /// omit this field and keep their original content identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned: Option<RustOwnedEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion: Option<RustCompletion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RustCompletion {
    Sync,
    Async,
}

impl RustCompletion {
    pub fn is_async(self) -> bool {
        self == Self::Async
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustOwnedEntry {
    pub type_name: String,
    pub role: RustOwnedRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RustOwnedRole {
    OpenText,
    OpenI32,
    OpenUnit,
    Borrow,
    Close,
}

impl RustProviderManifest {
    /// The original close export identifies a resource kind within this provider.
    pub fn owned_close(&self, type_name: &str) -> Option<(usize, &str)> {
        self.functions.iter().enumerate().find_map(|(index, f)| {
            f.owned
                .as_ref()
                .filter(|entry| entry.role == RustOwnedRole::Close && entry.type_name == type_name)
                .map(|_| (index, f.export.as_str()))
        })
    }
}

/// The frozen tag-frame encoding identity (IF-CALL physical projection).
pub const FRAME_FORMAT: &str = "p967.frame/1";

/// One parameter shape of an imported Rust function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RustParam {
    ViewUtf8,
    OutBytes,
    I32,
    Bool,
    OwnedHandle,
    Callback,
    /// `frame: &F0Frame` — the whole call rides the tag frame (the only
    /// parameter a frame-declared fn may take).
    FrameCall,
}

/// The frozen F0 calling shapes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RustFnForm {
    /// `(&str) -> i32`.
    PureI32,
    /// `(&str) -> bool` (physical 0/1 byte lane).
    PureBool,
    /// `(&str) -> Option<i32>`: `None` maps to the SMALLEST named
    /// negative status (the error-model table's canonical first code),
    /// `Some(v)` to status `0` + the `out` value. Physical frame
    /// `fn(data, byte_len, out: *mut i32) -> i32` (status).
    FallibleI32 { errors: Vec<(String, i32)> },
}

/// blake3 hex digest of the provider source bytes (provider identity).
pub fn provider_digest(source: &str) -> String {
    ace_foreign_contract::blake3_hex(source.as_bytes())
}

/// blake3 hex digest of the generated shim source bytes.
pub fn shim_digest(shim_source: &str) -> String {
    ace_foreign_contract::blake3_hex(shim_source.as_bytes())
}

/// Import an annotated Rust provider source. The header line
/// `//! auto-ffi provider <id>/<version> target=<t> profile=<p>` must
/// appear exactly once; every `// auto-ffi fn ...` line must match a
/// real `pub fn <name>(<one &str param>) -> i32|bool` signature in the
/// same source (the metadata cannot describe a function the provider
/// does not actually export in the frozen domain).
pub fn import(source: &str) -> Result<RustProviderManifest, ImportError> {
    let mut header: Option<(String, String, String, String)> = None;
    let mut annotations: Vec<FnAnnotation> = Vec::new();
    for (idx, raw) in source.lines().enumerate() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("//! ") {
            if let Some(body) = rest.trim().strip_prefix(ANNOTATION_PROVIDER) {
                if header.is_some() {
                    return Err(err(
                        "rust-import.header",
                        format!("duplicate provider annotation (line {})", idx + 1),
                    ));
                }
                header = Some(parse_provider_header(body.trim(), idx)?);
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("// ") {
            if let Some(body) = rest.trim().strip_prefix(ANNOTATION_FN) {
                annotations.push(parse_fn_annotation(body.trim(), idx)?);
            }
        }
    }
    audit_rust_shapes(source)?;
    let (provider, interface_version, target, profile) = header.ok_or_else(|| {
        err(
            "rust-import.header",
            "missing `//! auto-ffi provider <id>/<version> target=... profile=...` header",
        )
    })?;
    if annotations.is_empty() {
        return Err(err(
            "rust-import.annotation",
            "no `// auto-ffi fn ...` annotations: an F0 provider exports at least one function",
        ));
    }
    if !is_provider_id(&provider) {
        return Err(err(
            "rust-import.domain",
            format!("provider id `{provider}` is outside the F0 spelling [a-z0-9_]+"),
        ));
    }
    let mut functions = Vec::with_capacity(annotations.len());
    for a in annotations {
        // The annotated function must exist in the source with an
        // importable signature: metadata may never invent or widen a
        // provider (params + ret cross-checked against the annotation).
        let (params, ret_kind, owned) = if let Some(type_name) = a.owned_type.as_deref() {
            let (params, owned) = require_owned_signature(source, &a, type_name)?;
            (params, "i32".to_owned(), Some(owned))
        } else {
            let (params, ret) = require_pub_fn_signature(source, &a.rust_name)?;
            (params, ret, None)
        };
        // T-05 F1 (IF-CALL): a frame-declared fn takes EXACTLY one
        // `frame: &F0Frame` parameter — the whole call rides the tag
        // frame over the view wire (a frame annotation on an ordinary
        // shape, or the frame type without the axis, are named rejects).
        if a.frame.is_some() {
            if params.len() != 1 || !matches!(params[0], RustParam::FrameCall) {
                return Err(err(
                    "rust-import.frame",
                    format!(
                        "fn `{}` declares frame={} but must take exactly one `frame: &F0Frame` parameter",
                        a.rust_name, FRAME_FORMAT
                    ),
                ));
            }
        } else if params.len() == 1 && matches!(params[0], RustParam::FrameCall) {
            return Err(err(
                "rust-import.frame",
                format!(
                    "fn `{}` takes `&F0Frame` without the frame={} annotation",
                    a.rust_name, FRAME_FORMAT
                ),
            ));
        }
        if params.contains(&RustParam::Callback)
            && (params.as_slice() != [RustParam::Callback, RustParam::I32, RustParam::I32]
                || a.form != "pure"
                || a.ret.as_deref() != Some("i32"))
        {
            return Err(err(
                "rust-import.callback",
                format!("fn `{}` needs (&F0Callback, i32, i32) -> i32", a.rust_name),
            ));
        }
        let completion = match a.form.as_str() {
            "complete_sync" => Some(RustCompletion::Sync),
            "complete_async" => Some(RustCompletion::Async),
            _ => None,
        };
        if completion.is_some() && (params != [RustParam::I32] || ret_kind != "i32") {
            return Err(err(
                "rust-import.completion",
                format!(
                    "fn `{}` needs (i32) -> i32 for a completion port",
                    a.rust_name
                ),
            ));
        }
        let form = match (a.form.as_str(), a.ret.as_deref(), ret_kind.as_str()) {
            ("owned_open" | "owned_borrow" | "owned_close", None, _) if owned.is_some() => {
                RustFnForm::PureI32
            }
            ("pure", Some("i32"), "i32") => RustFnForm::PureI32,
            ("pure", Some("bool"), "bool") => RustFnForm::PureBool,
            ("complete_sync" | "complete_async", None, "i32") => RustFnForm::PureI32,
            ("fallible", None, "Option<i32>") => RustFnForm::FallibleI32 { errors: a.errors },
            (form, want, got) => {
                return Err(err(
                    "rust-import.domain",
                    format!(
                        "fn `{}`: annotation form `{}` (ret {:?}) does not match the Rust signature `-> {}` in the frozen F0 domain",
                        a.rust_name, form, want, got
                    ),
                ))
            }
        };
        let export = sanitize_symbol(&a.symbol)?;
        if functions
            .iter()
            .any(|f: &RustFnEntry| f.symbol == a.symbol || f.rust_name == a.rust_name)
        {
            return Err(err(
                "rust-import.duplicate",
                format!("duplicate fn annotation `{}`", a.rust_name),
            ));
        }
        functions.push(RustFnEntry {
            rust_name: a.rust_name,
            symbol: a.symbol,
            export,
            form,
            params,
            frame: a.frame.clone(),
            owned,
            completion,
        });
    }
    validate_owned_entries(&functions)?;
    Ok(RustProviderManifest {
        format: IMPORT_FORMAT.into(),
        provider,
        interface_version,
        target,
        profile,
        functions,
    })
}

struct FnAnnotation {
    rust_name: String,
    symbol: String,
    form: String,
    ret: Option<String>,
    errors: Vec<(String, i32)>,
    frame: Option<String>,
    owned_type: Option<String>,
}

fn parse_provider_header(
    body: &str,
    idx: usize,
) -> Result<(String, String, String, String), ImportError> {
    // `<id>/<version> target=<t> profile=<p>`
    let mut parts = body.split_whitespace();
    let id_slash_ver = parts.next().ok_or_else(|| {
        err(
            "rust-import.header",
            format!(
                "provider annotation needs `<id>/<version>` (line {})",
                idx + 1
            ),
        )
    })?;
    let (id, ver) = id_slash_ver.split_once('/').ok_or_else(|| {
        err(
            "rust-import.header",
            format!(
                "provider identity must be `<id>/<version>` (line {})",
                idx + 1
            ),
        )
    })?;
    let mut target = None;
    let mut profile = None;
    for part in parts {
        if let Some(v) = part.strip_prefix("target=") {
            target = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("profile=") {
            profile = Some(v.to_string());
        } else {
            return Err(err(
                "rust-import.header",
                format!(
                    "unknown provider annotation field `{part}` (line {})",
                    idx + 1
                ),
            ));
        }
    }
    let target = target.ok_or_else(|| {
        err(
            "rust-import.header",
            format!("provider annotation needs target= (line {})", idx + 1),
        )
    })?;
    let profile = profile.ok_or_else(|| {
        err(
            "rust-import.header",
            format!("provider annotation needs profile= (line {})", idx + 1),
        )
    })?;
    Ok((id.to_string(), ver.to_string(), target, profile))
}

fn parse_fn_annotation(body: &str, idx: usize) -> Result<FnAnnotation, ImportError> {
    // `<rust_name> symbol=<sym> form=pure ret=i32|bool`
    // `<rust_name> symbol=<sym> form=fallible errors=name:-code[,name:-code...]`
    let mut parts = body.split_whitespace();
    let rust_name = parts.next().unwrap_or_default().to_string();
    if rust_name.is_empty()
        || !rust_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(err(
            "rust-import.annotation",
            format!("fn annotation needs a Rust name (line {})", idx + 1),
        ));
    }
    let mut symbol = None;
    let mut form = None;
    let mut ret = None;
    let mut errors = Vec::new();
    let mut frame = None;
    let mut owned_type = None;
    for part in parts {
        if let Some(v) = part.strip_prefix("symbol=") {
            symbol = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("form=") {
            form = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("ret=") {
            ret = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("owned=") {
            let valid = v
                .bytes()
                .next()
                .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
                && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
            if !valid || owned_type.replace(v.to_owned()).is_some() {
                return Err(err(
                    "rust-import.owned",
                    format!("invalid or duplicate owned type (line {})", idx + 1),
                ));
            }
        } else if let Some(v) = part.strip_prefix("frame=") {
            if v != FRAME_FORMAT {
                return Err(err(
                    "rust-import.frame",
                    format!(
                        "frame `{v}` is not the frozen tag-frame encoding `{FRAME_FORMAT}` (line {})",
                        idx + 1
                    ),
                ));
            }
            frame = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("errors=") {
            for pair in v.split(',') {
                let (name, code) = pair.split_once(':').ok_or_else(|| {
                    err(
                        "rust-import.annotation",
                        format!("error codes must be `name:-code` pairs (line {})", idx + 1),
                    )
                })?;
                let code: i32 = code.parse().map_err(|_| {
                    err(
                        "rust-import.annotation",
                        format!("error code `{code}` is not an i32 (line {})", idx + 1),
                    )
                })?;
                if code >= 0 {
                    return Err(err(
                        "rust-import.annotation",
                        format!("status codes must be negative (line {})", idx + 1),
                    ));
                }
                errors.push((name.to_string(), code));
            }
        } else {
            return Err(err(
                "rust-import.annotation",
                format!("unknown fn annotation field `{part}` (line {})", idx + 1),
            ));
        }
    }
    let symbol = symbol.ok_or_else(|| {
        err(
            "rust-import.annotation",
            format!("fn annotation needs symbol= (line {})", idx + 1),
        )
    })?;
    let form = form.ok_or_else(|| {
        err(
            "rust-import.annotation",
            format!("fn annotation needs form= (line {})", idx + 1),
        )
    })?;
    let owned_form = matches!(form.as_str(), "owned_open" | "owned_borrow" | "owned_close");
    if owned_form {
        if owned_type.is_none() || frame.is_some() || ret.is_some() || !errors.is_empty() {
            return Err(err(
                "rust-import.owned",
                format!(
                    "{form} needs owned=Type and a matching Rust signature (line {})",
                    idx + 1
                ),
            ));
        }
    } else if matches!(form.as_str(), "complete_sync" | "complete_async") {
        if owned_type.is_some() || frame.is_some() || ret.is_some() || !errors.is_empty() {
            return Err(err(
                "rust-import.completion",
                format!(
                    "{form} needs a standalone (i32) -> i32 signature (line {})",
                    idx + 1
                ),
            ));
        }
    } else if owned_type.is_some() || !matches!(form.as_str(), "pure" | "fallible") {
        return Err(err(
            "rust-import.domain",
            format!("form `{form}` is outside the F0 domain (line {})", idx + 1),
        ));
    }
    if form == "fallible" && errors.is_empty() {
        return Err(err(
            "rust-import.annotation",
            format!("fallible fns need named error codes (line {})", idx + 1),
        ));
    }
    if form == "pure" && ret.is_none() {
        return Err(err(
            "rust-import.annotation",
            format!("pure fns need ret=i32|bool (line {})", idx + 1),
        ));
    }
    Ok(FnAnnotation {
        rust_name,
        symbol,
        form,
        ret,
        errors,
        frame,
        owned_type,
    })
}

fn require_owned_signature(
    source: &str,
    a: &FnAnnotation,
    type_name: &str,
) -> Result<(Vec<RustParam>, RustOwnedEntry), ImportError> {
    let (params, ret) = public_fn_shape(source, &a.rust_name)?;
    let params = params.trim();
    let param_type = if params.is_empty() {
        String::new()
    } else {
        params
            .split_once(':')
            .map(|(_, ty)| ty.split_whitespace().collect::<String>())
            .unwrap_or_else(|| "?".into())
    };
    let ret: String = ret.split_whitespace().collect();
    let boxed = format!("Box<{type_name}>");
    let borrowed = format!("&{type_name}");
    let (params, role) = match (a.form.as_str(), param_type.as_str(), ret.as_str()) {
        ("owned_open", "&str", result) if result == boxed => {
            (vec![RustParam::ViewUtf8], RustOwnedRole::OpenText)
        }
        ("owned_open", "i32", result) if result == boxed => {
            (vec![RustParam::I32], RustOwnedRole::OpenI32)
        }
        ("owned_open", "", result) if result == boxed => (vec![], RustOwnedRole::OpenUnit),
        ("owned_borrow", param, "i32") if param == borrowed => {
            (vec![RustParam::OwnedHandle], RustOwnedRole::Borrow)
        }
        ("owned_close", param, "i32") if param == boxed => {
            (vec![RustParam::OwnedHandle], RustOwnedRole::Close)
        }
        _ => {
            return Err(err(
                "rust-import.owned",
                format!(
                    "fn `{}` signature is incompatible with {} owned={type_name}",
                    a.rust_name, a.form
                ),
            ))
        }
    };
    Ok((
        params,
        RustOwnedEntry {
            type_name: type_name.into(),
            role,
        },
    ))
}

fn validate_owned_entries(functions: &[RustFnEntry]) -> Result<(), ImportError> {
    let mut resources = std::collections::BTreeMap::<&str, (bool, usize)>::new();
    for owned in functions.iter().filter_map(|f| f.owned.as_ref()) {
        let (has_open, closes) = resources.entry(&owned.type_name).or_default();
        match owned.role {
            RustOwnedRole::OpenText | RustOwnedRole::OpenI32 | RustOwnedRole::OpenUnit => {
                *has_open = true
            }
            RustOwnedRole::Close => *closes += 1,
            RustOwnedRole::Borrow => {}
        }
    }
    for (type_name, (has_open, closes)) in resources {
        if !has_open || closes != 1 {
            return Err(err(
                "rust-import.owned-lifecycle",
                format!("owned {type_name} needs an opener and exactly one consuming close"),
            ));
        }
    }
    Ok(())
}

fn is_provider_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Symbols are versioned dotted spellings (`f0.text.byte_len.v1`); the
/// C export name is the sanitized form (`f0_text_byte_len_v1`).
fn sanitize_symbol(symbol: &str) -> Result<String, ImportError> {
    let valid = !symbol.is_empty()
        && symbol
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_')
        && symbol.contains('.');
    if !valid {
        return Err(err(
            "rust-import.symbol",
            format!("symbol `{symbol}` must be a dotted lowercase versioned spelling"),
        ));
    }
    Ok(symbol.replace('.', "_"))
}

/// T-11 shape audit over the WHOLE provider source: shapes outside the
/// importable domain are NAMED per-row rejections (owned text/lists,
/// generics on any fn, async fns, trait objects, mutable borrows) —
/// closing the String/Vec/Result/future boundary rows as named facts.
pub fn audit_rust_shapes(source: &str) -> Result<(), ImportError> {
    for (idx, line) in source.lines().enumerate() {
        let t = line.trim();
        if t.starts_with("async fn") || t.contains(" async fn ") {
            return Err(err(
                "rust-import.future",
                format!("async fn at line {}: futures are outside the import domain (T-13 ports own completion)", idx + 1),
            ));
        }
        if t.starts_with("pub fn ")
            && t.contains('<')
            && t.contains('>')
            && t.split('(')
                .next()
                .map(|h| h.contains("fn "))
                .unwrap_or(false)
            && t.contains(" -> ")
        {
            // a generic fn head: `pub fn name<T...>(...`
            let head = t.split('(').next().unwrap_or("");
            if head.contains('<') {
                return Err(err(
                    "rust-import.generics",
                    format!("generic fn at line {}: import the CONCRETE instances (annotation on the monomorphic fn)", idx + 1),
                ));
            }
        }
        if t.contains("dyn ") {
            return Err(err(
                "rust-import.trait-object",
                format!(
                    "trait object at line {}: trait objects never cross the C boundary",
                    idx + 1
                ),
            ));
        }
    }
    Ok(())
}

/// The provider source must really declare `pub fn <name>(<params>) -> T`
/// with params drawn from the frozen T-11 domain — at most ONE borrowed
/// `&str` plus i32/bool scalars (any order); returns `i32`/`bool` (pure)
/// or `Option<i32>` (fallible). Every shape outside the domain is a
/// NAMED rejection, never a narrowing. Returns (param shapes, ret kind).
fn require_pub_fn_signature(
    source: &str,
    name: &str,
) -> Result<(Vec<RustParam>, String), ImportError> {
    let (params_text, ret) = public_fn_shape(source, name)?;
    let mut params = Vec::new();
    let mut views = 0usize;
    let mut out_bufs = 0usize;
    if !params_text.is_empty() {
        for raw in params_text.split(',') {
            let param = raw.trim();
            let ty = param.rsplit(':').next().unwrap_or("").trim();
            let borrowed = ty.strip_prefix('&').unwrap_or(ty);
            let borrowed = borrowed.trim_start_matches('\'').trim();
            if borrowed == "str" {
                if views == 1 {
                    return Err(err(
                        "rust-import.domain",
                        format!("fn `{name}` has two view parameters; at most one borrowed UTF-8 view is importable"),
                    ));
                }
                views += 1;
                params.push(RustParam::ViewUtf8);
                continue;
            }
            if borrowed == "mut [u8]" || ty.replace(' ', "") == "&mut[u8]" {
                if out_bufs == 1 {
                    return Err(err(
                        "rust-import.domain",
                        format!("fn `{name}` has two writable byte-slice parameters; at most one `&mut [u8]` is importable"),
                    ));
                }
                out_bufs += 1;
                params.push(RustParam::OutBytes);
                continue;
            }
            if borrowed == "F0Frame" {
                // the tag-frame parameter (IF-CALL): only legal as the
                // SINGLE parameter of a frame-declared fn (import()
                // cross-checks the annotation axis)
                params.push(RustParam::FrameCall);
                continue;
            }
            match ty {
                "i32" => params.push(RustParam::I32),
                "bool" => params.push(RustParam::Bool),
                "&F0Callback" => params.push(RustParam::Callback),
                "String" | "&String" => {
                    return Err(err(
                        "rust-import.owned-text",
                        format!("fn `{name}` takes an owned String; owned text crossing needs the allocator lane (T-13/T-15)"),
                    ))
                }
                "Vec" | _ if ty.starts_with("Vec<") => {
                    return Err(err(
                        "rust-import.owned-list",
                        format!("fn `{name}` takes a Vec; owned lists need their lane (T-13/T-15)"),
                    ))
                }
                _ if ty.starts_with("&mut ") || borrowed.starts_with("mut ") => {
                    return Err(err(
                        "rust-import.mut-borrow",
                        format!("fn `{name}` takes a mutable borrow; mut crossing rides the record-lane roles, not raw &mut"),
                    ))
                }
                _ => {
                    return Err(err(
                        "rust-import.domain",
                        format!("fn `{name}` parameter `{param}` is outside the importable domain (view_utf8 | i32 | bool)"),
                    ))
                }
            }
        }
    }
    if params.len() > 4 {
        return Err(err(
            "rust-import.domain",
            format!(
                "fn `{name}` has {} parameters; the frozen frame caps at 4",
                params.len()
            ),
        ));
    }
    match ret {
        "i32" | "bool" | "Option<i32>" => Ok((params, ret.to_string())),
        "String" => Err(err(
            "rust-import.owned-text",
            format!("fn `{name}` returns an owned String; owned text crossing needs the allocator lane (T-13/T-15)"),
        )),
        other if other.starts_with("Vec<") => Err(err(
            "rust-import.owned-list",
            format!("fn `{name}` returns an owned Vec; owned lists need their lane (T-13/T-15)"),
        )),
        other if other.starts_with("Result<") => Err(err(
            "rust-import.result-type",
            format!("fn `{name}` returns {other}; map the error set to the fallible status contract (Option<i32> + named codes)"),
        )),
        other => Err(err(
            "rust-import.domain",
            format!("fn `{name}` returns `{other}`; importable returns are i32, bool, Option<i32>"),
        )),
    }
}

fn public_fn_shape<'a>(source: &'a str, name: &str) -> Result<(&'a str, &'a str), ImportError> {
    let needle = format!("pub fn {name}(");
    let Some(pos) = source.find(&needle) else {
        return Err(err(
            "rust-import.domain",
            format!("annotated fn `{name}` has no `pub fn {name}(` in the provider source"),
        ));
    };
    let after = &source[pos + needle.len()..];
    let Some((params, rest)) = after.split_once(')') else {
        return Err(err(
            "rust-import.domain",
            format!("cannot find the parameter list end for fn `{name}`"),
        ));
    };
    let Some(ret) = rest.trim_start().strip_prefix("->") else {
        return Err(err(
            "rust-import.domain",
            format!("fn `{name}` has no return type"),
        ));
    };
    Ok((
        params.trim(),
        ret.split(['{', ';']).next().unwrap_or_default().trim(),
    ))
}

/// Generate the C-compatible shim source for the whole provider. The
/// shim file is compiled INTO the provider crate (one build closure,
/// PLAN-967 B-01 seed): `#[no_mangle] extern "C"` exports with the
/// frozen F0 physical frame `fn(data: *const u8, byte_len: u64)`.
///
/// The UTF-8 guarantee is the caller's contract (AV rejects invalid
/// UTF-8 before dispatch); the shim's `from_utf8` panic is the honest
/// defensive boundary, carried out as a host failure — never a silent
/// reinterpretation.
pub fn generate_shim(manifest: &RustProviderManifest) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "// Generated by auto-bindgen rust_import ({IMPORT_FORMAT}) for provider {}/{}.\n",
        manifest.provider, manifest.interface_version
    ));
    out.push_str(
        "// PLAN-967 F0 physical frame: extern \"C\" fn(data: *const u8, byte_len: u64).\n",
    );
    out.push_str(
        "// Panic channel: in-shim catch + exported flag; no unwind crosses the boundary.\n",
    );
    out.push_str("// DO NOT EDIT: regenerate from the annotated provider source.\n\n");
    out.push_str(
        "fn __f0_view(data: *const u8, byte_len: u64) -> &'static str {\n\
         \x20   let bytes = unsafe { std::slice::from_raw_parts(data, byte_len as usize) };\n\
         \x20   match std::str::from_utf8(bytes) {\n\
         \x20       Ok(s) => s,\n\
         \x20       Err(_) => panic!(\"auto-ffi f0: caller must guarantee UTF-8 (foreign.utf8)\"),\n\
         \x20   }\n\
         }\n\n",
    );
    // PLAN-967 F0 panic channel (frozen): NO unwind ever crosses the C
    // boundary (B-03). The shim catches a provider panic IN-UNIT (the
    // same std instance — always catchable) and raises the exported
    // flag; the host observes the flag after each call and converts it
    // to an independent HOST failure. A set flag invalidates the call's
    // return values.
    out.push_str(
        "static __F0_PANIC_FLAG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);\n\
         \x20#[no_mangle]\n\
         \x20pub extern \"C\" fn f0_panic_flag() -> *mut u32 {\n\
         \x20   &__F0_PANIC_FLAG as *const _ as *mut u32\n\
         }\n\n",
    );
    // PLAN-967 T-03: the native fallible failure transport. The caller
    // COFF imports `ace_foreign_fail` for fallible declarations; the F0
    // build unit exports the SAME semantics as the C-side runtime
    // (stderr line + controlled exit 70), so a native link needs no
    // external runtime object (B-06 runtime-dependency seed).
    out.push_str(
        "#[no_mangle]\npub extern \"C\" fn ace_foreign_fail(status: i32) -> ! {\n\
         \x20   eprintln!(\"foreign.status={}\", status);\n\
         \x20   std::process::exit(70)\n\
         }\n\n",
    );
    // T-05 F1 (IF-CALL): the tag-frame types + wire decoder ride the
    // shim ONCE when any fn declares frame=p967.frame/1. The types are
    // ordinary pub items of the SAME crate (provider + shim are one flat
    // build closure), so the provider's `frame: &F0Frame` parameter
    // resolves without any dependency.
    if manifest.functions.iter().any(|f| f.frame.is_some()) {
        out.push_str(FRAME_SHIM_PRELUDE);
    }
    if manifest
        .functions
        .iter()
        .any(|f| f.params.contains(&RustParam::Callback))
    {
        out.push_str(CALLBACK_SHIM_PRELUDE);
    }
    for f in &manifest.functions {
        if f.completion.is_some() {
            emit_completion_fn(&mut out, f);
            continue;
        }
        if f.params.contains(&RustParam::Callback) {
            emit_callback_fn(&mut out, f);
            continue;
        }
        if let Some(owned) = &f.owned {
            emit_owned_fn(&mut out, f, owned);
            continue;
        }
        if f.frame.is_some() {
            emit_frame_fn(&mut out, f);
            continue;
        }
        // The T-11 general frame: the view (when present) opens as
        // (data, byte_len); every scalar rides as its C lane; the
        // fallible form appends the out pointer.
        let mut params_c = String::new();
        let mut args_rust = String::new();
        if f.params.iter().any(|p| matches!(p, RustParam::ViewUtf8)) {
            params_c.push_str("data: *const u8, byte_len: u64");
            args_rust.push_str("__f0_view(data, byte_len)");
        }
        let mut scalar_idx = 0;
        for p in &f.params {
            match p {
                RustParam::ViewUtf8 => {}
                RustParam::OutBytes => {
                    if !params_c.is_empty() {
                        params_c.push_str(", ");
                        args_rust.push_str(", ");
                    }
                    params_c.push_str("dst: *mut u8, cap: u64");
                    args_rust.push_str(
                        "if cap == 0 || dst.is_null() { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(dst, cap as usize) } }",
                    );
                }
                // frame fns are emitted by emit_frame_fn (they never
                // reach the general frame loop; import() enforces the
                // single-parameter shape)
                RustParam::FrameCall | RustParam::OwnedHandle | RustParam::Callback => {}
                RustParam::I32 => {
                    if !params_c.is_empty() {
                        params_c.push_str(", ");
                        args_rust.push_str(", ");
                    }
                    params_c.push_str(&format!("p{scalar_idx}: i32"));
                    args_rust.push_str(&format!("p{scalar_idx}"));
                    scalar_idx += 1;
                }
                RustParam::Bool => {
                    if !params_c.is_empty() {
                        params_c.push_str(", ");
                        args_rust.push_str(", ");
                    }
                    params_c.push_str(&format!("p{scalar_idx}: bool"));
                    args_rust.push_str(&format!("p{scalar_idx}"));
                    scalar_idx += 1;
                }
            }
        }
        let none_code = match &f.form {
            RustFnForm::FallibleI32 { errors } => {
                let mut codes: Vec<i32> = errors.iter().map(|(_, c)| *c).collect();
                codes.sort_unstable();
                codes[0]
            }
            _ => 0,
        };
        match &f.form {
            RustFnForm::PureI32 => {
                out.push_str(&format!(
                    "#[no_mangle]\npub extern \"C\" fn {export}({params_c}) -> i32 {{\n    match std::panic::catch_unwind(|| {{ {name}({args_rust}) }}) {{\n        Ok(v) => v,\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); 0 }}\n    }}\n}}\n\n",
                    export = f.export,
                    name = f.rust_name,
                ));
            }
            RustFnForm::FallibleI32 { .. } => {
                let out_param = if params_c.is_empty() {
                    "out: *mut i32".to_string()
                } else {
                    format!(", out: *mut i32")
                };
                out.push_str(&format!(
                    "#[no_mangle]\npub extern \"C\" fn {export}({params_c}{out_param}) -> i32 {{\n    match std::panic::catch_unwind(|| {{ {name}({args_rust}) }}) {{\n        Ok(Some(v)) => {{ unsafe {{ *out = v; }} 0 }}\n        Ok(None) => {none_code},\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); {none_code} }}\n    }}\n}}\n\n",
                    export = f.export,
                    name = f.rust_name,
                ));
            }
            RustFnForm::PureBool => {
                out.push_str(&format!(
                    "#[no_mangle]\npub extern \"C\" fn {export}({params_c}) -> u8 {{\n    match std::panic::catch_unwind(|| {{ {name}({args_rust}) }}) {{\n        Ok(v) => v as u8,\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); 0 }}\n    }}\n}}\n\n",
                    export = f.export,
                    name = f.rust_name,
                ));
            }
        }
    }
    out
}

const CALLBACK_SHIM_PRELUDE: &str = r#"
mod __f0_callback {
    pub struct F0Callback {
        target: extern "C" fn(i64, i64) -> i64,
        thread: std::marker::PhantomData<std::rc::Rc<()>>,
    }
    impl F0Callback {
        pub(super) fn new(target: extern "C" fn(i64, i64) -> i64) -> Self {
            Self { target, thread: std::marker::PhantomData }
        }
        pub fn call(&self, ctx: i32, item: i32) -> i32 {
            i32::try_from((self.target)(i64::from(ctx), i64::from(item))).expect("checked callback result")
        }
    }
}
pub use __f0_callback::F0Callback;
"#;

fn emit_callback_fn(out: &mut String, f: &RustFnEntry) {
    out.push_str(&format!(
        "#[no_mangle]\npub extern \"C\" fn {export}(target: extern \"C\" fn(i64, i64) -> i64, ctx: i64, item: i32) -> i32 {{\n    let callback = F0Callback::new(target);\n    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {name}(&callback, i32::try_from(ctx).expect(\"checked callback context\"), item))) {{\n        Ok(value) => value,\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); 0 }}\n    }}\n}}\n\n",
        export = f.export,
        name = f.rust_name,
    ));
}

fn emit_completion_fn(out: &mut String, f: &RustFnEntry) {
    // Completion status is per invocation. A late background panic must
    // never set the DLL-wide panic flag of a different foreground call.
    out.push_str(&format!(
        "#[no_mangle]\npub extern \"C\" fn {export}(input: i32, out: *mut i32) -> i32 {{\n    match std::panic::catch_unwind(|| {name}(input)) {{\n        Ok(value) => {{ unsafe {{ *out = value; }} 0 }}\n        Err(_) => 1,\n    }}\n}}\n\n",
        export = f.export,
        name = f.rust_name,
    ));
}

fn emit_owned_fn(out: &mut String, f: &RustFnEntry, owned: &RustOwnedEntry) {
    let ty = &owned.type_name;
    let name = &f.rust_name;
    let (params, ret, body) = match owned.role {
        RustOwnedRole::OpenText => (
            "data: *const u8, byte_len: u64",
            "i64",
            format!("Box::into_raw({name}(__f0_view(data, byte_len))) as i64"),
        ),
        RustOwnedRole::OpenI32 => (
            "seed: i32",
            "i64",
            format!("Box::into_raw({name}(seed)) as i64"),
        ),
        RustOwnedRole::OpenUnit => ("", "i64", format!("Box::into_raw({name}()) as i64")),
        RustOwnedRole::Borrow => (
            "handle: i64",
            "i32",
            format!(
                "{name}(unsafe {{ (handle as *const {ty}).as_ref().expect(\"owned handle\") }})"
            ),
        ),
        RustOwnedRole::Close => (
            "handle: i64",
            "i32",
            format!(
                "assert_ne!(handle, 0); {name}(unsafe {{ Box::from_raw(handle as *mut {ty}) }})"
            ),
        ),
    };
    out.push_str(&format!(
        "#[no_mangle]\npub extern \"C\" fn {export}({params}) -> {ret} {{\n    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {{ {body} }})) {{\n        Ok(value) => value,\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); 0 }}\n    }}\n}}\n\n",
        export = f.export
    ));
}

/// The p967.frame/1 tag-frame shim prelude (generated; DO NOT EDIT).
const FRAME_SHIM_PRELUDE: &str = r####"
#[derive(Debug, Clone)]
pub enum F0Val { None, I32(i32), Bool(bool), Text(String), Opaque(u64) }
#[derive(Debug, Clone)]
pub struct F0Kwarg { pub name: String, pub value: F0Val }
#[derive(Debug, Clone)]
pub struct F0Frame { pub receiver: F0Val, pub positionals: Vec<F0Val>, pub kwargs: Vec<F0Kwarg> }
fn __f0_take_line<'a>(cur: &mut &'a [u8]) -> Result<&'a [u8], &'static str> {
    let n = cur.iter().position(|&b| b == b'\n').ok_or("unterminated header line")?;
    let (l, tail) = cur.split_at(n);
    if l.is_empty() { return Err("empty header line"); }
    *cur = &tail[1..];
    Ok(l)
}
fn __f0_decimal(l: &[u8]) -> Option<(u64, &[u8])> {
    let mut n: u64 = 0; let mut i = 0;
    if l.first() == Some(&b'-') { i = 1; }
    let d = i;
    while i < l.len() && l[i].is_ascii_digit() {
        n = n.checked_mul(10)?.checked_add((l[i] - b'0') as u64)?;
        if n > (i64::MAX as u64) { return None; }
        i += 1;
    }
    if i == d { return None; }
    Some((n, &l[i..]))
}
fn __f0_take_value<'a>(cur: &mut &'a [u8]) -> Result<(u8, &'a [u8]), &'static str> {
    let header = __f0_take_line(cur)?;
    let tag = *header.first().ok_or("missing tag")?;
    if !matches!(tag, b'n' | b'i' | b'b' | b't' | b'o') { return Err("unknown tag"); }
    let tail = header[1..].strip_prefix(b" ").ok_or("tag not followed by a length")?;
    let (len, tail) = __f0_decimal(tail).ok_or("bad length")?;
    if !tail.is_empty() { return Err("length not the header end"); }
    if len > 4096 { return Err("payload over the frame text cap"); }
    let len = len as usize;
    if cur.len() < len { return Err("payload truncated"); }
    let (payload, tail) = cur.split_at(len);
    let tail = tail.strip_prefix(b"\n").ok_or("payload not LF-terminated")?;
    *cur = tail;
    Ok((tag, payload))
}
fn __f0_take_name(cur: &mut &[u8]) -> Result<String, &'static str> {
    let header = __f0_take_line(cur)?;
    let (nl, t) = __f0_decimal(header).ok_or("bad name length")?;
    if !t.is_empty() { return Err("name length not the header end"); }
    if nl < 1 || nl > 64 { return Err("kwarg name length out of range"); }
    let nl = nl as usize;
    if cur.len() < nl { return Err("kwarg name truncated"); }
    let (nb, t2) = cur.split_at(nl);
    *cur = t2;
    match std::str::from_utf8(nb).ok() {
        Some(s) => Ok(s.to_string()),
        None => Err("kwarg name is not UTF-8"),
    }
}
fn __f0_val(tag: u8, payload: &[u8]) -> Result<F0Val, &'static str> {
    match tag {
        b'n' if payload.is_empty() => Ok(F0Val::None),
        b'i' => std::str::from_utf8(payload).ok().and_then(|t| t.parse::<i32>().ok()).map(F0Val::I32).ok_or("tag i payload is not an i32"),
        b'o' => std::str::from_utf8(payload).ok().and_then(|t| t.parse::<u64>().ok()).map(F0Val::Opaque).ok_or("tag o payload is not a u64"),
        b'b' if payload == b"0" => Ok(F0Val::Bool(false)),
        b'b' if payload == b"1" => Ok(F0Val::Bool(true)),
        b't' => std::str::from_utf8(payload).ok().map(|t| F0Val::Text(t.to_string())).ok_or("tag t payload is not UTF-8"),
        _ => Err("bad tagged value"),
    }
}
/// Decode a p967.frame/1 wire frame (the AV admission validated the
/// structure pre-dispatch; this decode is the defensive second gate).
fn __f0_frame_decode(bytes: &[u8]) -> Result<F0Frame, &'static str> {
    let mut cur = bytes;
    let magic = b"p967.frame/1\n";
    if cur.len() < magic.len() || &cur[..magic.len()] != magic { return Err("bad magic/version header"); }
    cur = &cur[magic.len()..];
    let rh = __f0_take_line(&mut cur)?;
    let rh = rh.strip_prefix(b"r ").ok_or("expected the receiver value header")?;
    let rtag = *rh.first().ok_or("missing tag")?;
    let rtail = rh[1..].strip_prefix(b" ").ok_or("tag not followed by a length")?;
    let (rlen, rtail) = __f0_decimal(rtail).ok_or("bad length")?;
    if !rtail.is_empty() || rlen > 4096 { return Err("bad receiver header"); }
    let rlen = rlen as usize;
    if cur.len() < rlen { return Err("payload truncated"); }
    let (rpayload, t2) = cur.split_at(rlen);
    let t2 = t2.strip_prefix(b"\n").ok_or("payload not LF-terminated")?;
    cur = t2;
    let receiver = __f0_val(rtag, rpayload)?;
    let header = __f0_take_line(&mut cur)?;
    if header[0] != b'p' { return Err("expected the positional count line"); }
    let (n, tail) = __f0_decimal(header[1..].strip_prefix(b" ").ok_or("count not delimited")?).ok_or("bad count")?;
    if !tail.is_empty() || n > 8 { return Err("count over the frame cap"); }
    let mut positionals = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let (tag, payload) = __f0_take_value(&mut cur)?;
        positionals.push(__f0_val(tag, payload)?);
    }
    let header = __f0_take_line(&mut cur)?;
    if header[0] != b'k' { return Err("expected the kwarg count line"); }
    let (n, tail) = __f0_decimal(header[1..].strip_prefix(b" ").ok_or("count not delimited")?).ok_or("bad count")?;
    if !tail.is_empty() || n > 8 { return Err("count over the frame cap"); }
    let mut kwargs = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let name = __f0_take_name(&mut cur)?;
        let (tag, payload) = __f0_take_value(&mut cur)?;
        kwargs.push(F0Kwarg { name, value: __f0_val(tag, payload)? });
    }
    if !cur.is_empty() { return Err("trailing bytes after the frame"); }
    Ok(F0Frame { receiver, positionals, kwargs })
}
"####;

/// Emit one frame-typed export: the wire rides the view physical lane
/// (data, byte_len[, out]) exactly like every view fn — only the
/// payload is a p967.frame/1 frame the shim decodes into typed values.
fn emit_frame_fn(out: &mut String, f: &RustFnEntry) {
    let none_code = match &f.form {
        RustFnForm::FallibleI32 { errors } => {
            let mut codes: Vec<i32> = errors.iter().map(|(_, c)| *c).collect();
            codes.sort_unstable();
            codes[0]
        }
        _ => 0,
    };
    match &f.form {
        RustFnForm::FallibleI32 { .. } => out.push_str(&format!(
            "#[no_mangle]\npub extern \"C\" fn {export}(data: *const u8, byte_len: u64, out: *mut i32) -> i32 {{\n    let bytes = unsafe {{ std::slice::from_raw_parts(data, byte_len as usize) }};\n    let frame = match __f0_frame_decode(bytes) {{ Ok(f) => f, Err(_) => return {none_code} }};\n    match std::panic::catch_unwind(|| {{ {name}(&frame) }}) {{\n        Ok(Some(v)) => {{ unsafe {{ *out = v; }} 0 }}\n        Ok(None) => {none_code},\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); {none_code} }}\n    }}\n}}\n\n",
            export = f.export,
            name = f.rust_name,
            none_code = none_code,
        )),
        RustFnForm::PureI32 => out.push_str(&format!(
            "#[no_mangle]\npub extern \"C\" fn {export}(data: *const u8, byte_len: u64) -> i32 {{\n    let bytes = unsafe {{ std::slice::from_raw_parts(data, byte_len as usize) }};\n    let frame = match __f0_frame_decode(bytes) {{ Ok(f) => f, Err(_) => return 0 }};\n    match std::panic::catch_unwind(|| {{ {name}(&frame) }}) {{\n        Ok(v) => v,\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); 0 }}\n    }}\n}}\n\n",
            export = f.export,
            name = f.rust_name,
        )),
        RustFnForm::PureBool => out.push_str(&format!(
            "#[no_mangle]\npub extern \"C\" fn {export}(data: *const u8, byte_len: u64) -> u8 {{\n    let bytes = unsafe {{ std::slice::from_raw_parts(data, byte_len as usize) }};\n    let frame = match __f0_frame_decode(bytes) {{ Ok(f) => f, Err(_) => return 0 }};\n    match std::panic::catch_unwind(|| {{ {name}(&frame) }}) {{\n        Ok(v) => v as u8,\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); 0 }}\n    }}\n}}\n\n",
            export = f.export,
            name = f.rust_name,
        )),
    }
}

/// Project the neutral provider manifest onto the shared
/// `ace.foreign.manual/2` manifest consumed by the ACE checked
/// frontdoor (the same admission path PLAN-938/939 delivered).
///
/// `header_digest` is the provider source digest; `physical_table_digest`
/// is the generated shim digest — both feed the host authority checks.
/// The physical profile/target are pinned to the shared plan constants
/// (the annotation's profile names the PROVIDER identity, not a new ABI).
pub fn to_manual_manifest(
    manifest: &RustProviderManifest,
    review_id: &str,
    header_digest: &str,
    physical_table_digest: &str,
) -> Result<ManualManifest, String> {
    if manifest.target != F0_TARGET {
        return Err(format!(
            "provider target `{}` is not the frozen F0 target `{F0_TARGET}`",
            manifest.target
        ));
    }
    let mut functions = Vec::with_capacity(manifest.functions.len());
    let mut effects = Vec::new();
    let mut declarations = Vec::new();
    for f in &manifest.functions {
        let lanes: Vec<ResKind> = f
            .params
            .iter()
            .map(|p| match p {
                RustParam::ViewUtf8 | RustParam::FrameCall => ResKind::ViewText,
                RustParam::OutBytes => ResKind::OutBytes,
                RustParam::I32 => ResKind::I32,
                RustParam::Bool => ResKind::Bool,
                RustParam::OwnedHandle => ResKind::Handle {
                    provider: manifest.provider.clone(),
                },
                RustParam::Callback => ResKind::Callback { ctx: true },
            })
            .collect();
        let (form, error_model, resource, effect) = match &f.form {
            RustFnForm::PureI32 => (
                FnForm::Pure {
                    params: vec![],
                    ret: LaneKind::I32,
                },
                None,
                ResourceForm::Pure { ret: ResOut::I32 },
                format!("pure:{}", f.export),
            ),
            RustFnForm::PureBool => (
                FnForm::Pure {
                    params: vec![],
                    ret: LaneKind::Bool,
                },
                None,
                ResourceForm::Pure { ret: ResOut::Bool },
                format!("pure:{}", f.export),
            ),
            RustFnForm::FallibleI32 { ref errors } => {
                let mut codes: Vec<(i32, String)> = errors
                    .iter()
                    .map(|(name, code)| (*code, name.clone()))
                    .collect();
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
        let resource = match f.owned.as_ref().map(|owned| owned.role) {
            Some(RustOwnedRole::OpenText | RustOwnedRole::OpenI32 | RustOwnedRole::OpenUnit) => {
                ResourceForm::Pure {
                    ret: ResOut::Handle {
                        provider: manifest.provider.clone(),
                    },
                }
            }
            _ => resource,
        };
        functions.push(FnFact {
            symbol: f.export.clone(),
            form,
            error_model,
            // The manual lane carries no semantic-service refs (the neutral
            // manifest keeps the versioned dotted symbol identity instead).
            semantic_ref: None,
            resource: Some(ResourceSig {
                format: RESOURCE_SIG_FORMAT.into(),
                params: lanes,
                form: resource,
                atomic: false,
                noescape: true,
                typed_peer: false,
            }),
        });
        effects.push(effect);
        declarations.push(ManualDecl {
            context_symbol: None,
            source_name: f.rust_name.clone(),
            symbol: f.export.clone(),
        });
    }
    let content = serde_json::to_vec(manifest).map_err(|e| e.to_string())?;
    let m = ManualManifest {
        schema: MANUAL_SCHEMA_V2.into(),
        review_id: review_id.into(),
        provider: manifest.provider.clone(),
        header_digest: header_digest.into(),
        physical_table_digest: physical_table_digest.into(),
        target: ace_foreign_contract::plan::TARGET.into(),
        profile: ace_foreign_contract::plan::PROFILE.into(),
        callconv: "windows-x64".into(),
        direction: "source-import".into(),
        facts: InterfaceFacts {
            format: FACTS_FORMAT.into(),
            logical: format!("rust/{}", manifest.provider),
            content_version: ace_foreign_contract::blake3_hex(&content),
            source: FactsSource::Typed {
                id: format!(
                    "auto-bindgen/rust-import/{}/{}",
                    manifest.provider, manifest.interface_version
                ),
            },
            functions,
            effects,
            capabilities: vec![SOURCE_CAP.into(), RESOURCE_CAP.into()],
        },
        declarations,
    };
    m.validate()
        .map_err(|e| format!("manifest validation: {e}"))?;
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROVIDER: &str = r#"//! auto-ffi provider f0_text/1 target=x86_64-pc-windows-msvc profile=f0-text.v1
// auto-ffi fn text_byte_len symbol=f0.text.byte_len.v1 form=pure ret=i32
pub fn text_byte_len(s: &str) -> i32 { s.as_bytes().len() as i32 }

// auto-ffi fn text_is_ascii symbol=f0.text.is_ascii.v1 form=pure ret=bool
pub fn text_is_ascii(s: &str) -> bool { s.is_ascii() }

// auto-ffi fn text_first_byte symbol=f0.text.first_byte.v1 form=fallible errors=empty-input:-1
pub fn text_first_byte(s: &str) -> Option<i32> { s.as_bytes().first().map(|b| *b as i32) }
"#;

    #[test]
    fn imports_frozen_provider_and_projects_shared_manifest() {
        let m = import(PROVIDER).expect("imports");
        assert_eq!(m.provider, "f0_text");
        assert_eq!(m.interface_version, "1");
        assert_eq!(m.functions.len(), 3);
        assert_eq!(m.functions[0].export, "f0_text_byte_len_v1");
        let shim = generate_shim(&m);
        assert!(shim.contains(r#"pub extern "C" fn f0_text_byte_len_v1("#));
        assert!(shim.contains("-> u8"));
        let manual = to_manual_manifest(
            &m,
            "f0-review/1",
            &provider_digest(PROVIDER),
            &shim_digest(&shim),
        )
        .expect("projects");
        assert_eq!(manual.schema, MANUAL_SCHEMA_V2);
        assert_eq!(manual.declarations.len(), 3);
        assert_eq!(manual.declarations[0].source_name, "text_byte_len");
        assert!(manual
            .facts
            .capabilities
            .contains(&RESOURCE_CAP.to_string()));
        // admission works end to end against a matching authority
        let host = ace_foreign_contract::manual::HostAuthority {
            review_id: manual.review_id.clone(),
            contract_digest: manual.contract_digest().unwrap(),
            provider: manual.provider.clone(),
            provider_digest: provider_digest(PROVIDER),
            header_digest: manual.header_digest.clone(),
            physical_table_digest: manual.physical_table_digest.clone(),
            target: manual.target.clone(),
            capabilities: manual.facts.capabilities.iter().cloned().collect(),
            available_symbols: manual
                .declarations
                .iter()
                .map(|d| d.symbol.clone())
                .collect(),
            binding_generation: 1,
        };
        let admitted = manual.admit(&host).expect("admits");
        assert_eq!(admitted.source_declarations_ref().len(), 3);
        assert!(admitted.source_declarations_ref()[0].resource().is_some());
    }

    #[test]
    fn rejects_domain_violations_with_named_codes() {
        // no header
        assert_eq!(
            import("pub fn a(s: &str) -> i32 { 0 }").unwrap_err().code,
            "rust-import.header"
        );
        // annotated fn missing from source
        let missing = "//! auto-ffi provider p/1 target=x86_64-pc-windows-msvc profile=p.v1\n// auto-ffi fn ghost symbol=p.ghost.v1 form=pure ret=i32\npub fn real(s: &str) -> i32 { 0 }\n";
        assert_eq!(import(missing).unwrap_err().code, "rust-import.domain");
        // generic signature
        let generic = "//! auto-ffi provider p/1 target=x86_64-pc-windows-msvc profile=p.v1\n// auto-ffi fn g symbol=p.g.v1 form=pure ret=i32\npub fn g<T>(s: &str) -> i32 { 0 }\n";
        assert_eq!(import(generic).unwrap_err().code, "rust-import.generics");
        // owned String param
        let owned = "//! auto-ffi provider p/1 target=x86_64-pc-windows-msvc profile=p.v1\n// auto-ffi fn o symbol=p.o.v1 form=pure ret=i32\npub fn o(s: String) -> i32 { 0 }\n";
        assert_eq!(import(owned).unwrap_err().code, "rust-import.owned-text");
        // ret mismatch with annotation
        let mismatch = "//! auto-ffi provider p/1 target=x86_64-pc-windows-msvc profile=p.v1\n// auto-ffi fn m symbol=p.m.v1 form=pure ret=bool\npub fn m(s: &str) -> i32 { 0 }\n";
        assert_eq!(import(mismatch).unwrap_err().code, "rust-import.domain");
        // non-symbolic symbol
        let bad = "//! auto-ffi provider p/1 target=x86_64-pc-windows-msvc profile=p.v1\n// auto-ffi fn b symbol=BadSym form=pure ret=i32\npub fn b(s: &str) -> i32 { 0 }\n";
        assert_eq!(import(bad).unwrap_err().code, "rust-import.symbol");
        // two params
        let two = "//! auto-ffi provider p/1 target=x86_64-pc-windows-msvc profile=p.v1\n// auto-ffi fn t symbol=p.t.v1 form=pure ret=i32\npub fn t(a: &str, b: &str) -> i32 { 0 }\n";
        assert_eq!(import(two).unwrap_err().code, "rust-import.domain");
        // duplicate annotation
        let dup = "//! auto-ffi provider p/1 target=x86_64-pc-windows-msvc profile=p.v1\n// auto-ffi fn d symbol=p.d.v1 form=pure ret=i32\n// auto-ffi fn d symbol=p.d.v1 form=pure ret=i32\npub fn d(s: &str) -> i32 { 0 }\n";
        assert_eq!(import(dup).unwrap_err().code, "rust-import.duplicate");
    }

    #[test]
    fn fallible_none_maps_to_the_smallest_named_code() {
        // Audit #1: the generated shim must carry the SAME smallest named
        // status the manifest's canonical error table freezes — manifest
        // and physical behavior can never disagree.
        let src = "//! auto-ffi provider p2/1 target=x86_64-pc-windows-msvc profile=p.v1
// auto-ffi fn f symbol=p.f.v1 form=fallible errors=b:-1,a:-2
pub fn f(s: &str) -> Option<i32> { None }
";
        let m = import(src).expect("imports");
        let shim = generate_shim(&m);
        assert!(shim.contains("Ok(None) => -2"), "{shim}");
        assert!(shim.contains(
            "Err(_) => { __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); -2 }"
        ));
        let manual = to_manual_manifest(&m, "r/1", &provider_digest(src), &shim_digest(&shim))
            .expect("projects");
        let first = &manual.facts.functions[0]
            .error_model
            .as_ref()
            .unwrap()
            .codes[0];
        assert_eq!(first.0, -2);
    }

    #[test]
    fn shim_is_deterministic_and_versioned() {
        let m = import(PROVIDER).unwrap();
        let a = generate_shim(&m);
        let b = generate_shim(&m);
        assert_eq!(a, b);
        assert!(a.contains(IMPORT_FORMAT));
        assert!(a.contains("text_byte_len(__f0_view(data, byte_len))"));
    }
}
