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
        // The annotated function must exist in the source with a frozen
        // F0 signature: metadata may never invent or widen a provider.
        let ret = require_pub_fn_signature(source, &a.rust_name)?;
        let form = match (a.form.as_str(), a.ret.as_deref(), ret.as_str()) {
            ("pure", Some("i32"), "i32") => RustFnForm::PureI32,
            ("pure", Some("bool"), "bool") => RustFnForm::PureBool,
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
        });
    }
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
    for part in parts {
        if let Some(v) = part.strip_prefix("symbol=") {
            symbol = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("form=") {
            form = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("ret=") {
            ret = Some(v.to_string());
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
    if !matches!(form.as_str(), "pure" | "fallible") {
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
    })
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

/// The provider source must really declare `pub fn <name>(<p>: &str) -> T`
/// with exactly one `&str` parameter and no generics; returns the declared
/// Rust return type (`i32`/`bool`).
fn require_pub_fn_signature(source: &str, name: &str) -> Result<String, ImportError> {
    let needle = format!("pub fn {name}(");
    let Some(pos) = source.find(&needle) else {
        return Err(err(
            "rust-import.domain",
            format!("annotated fn `{name}` has no `pub fn {name}(` in the provider source"),
        ));
    };
    let after = &source[pos + needle.len()..];
    let Some(close) = after.find(')') else {
        return Err(err(
            "rust-import.domain",
            format!("cannot find the parameter list end for fn `{name}`"),
        ));
    };
    let params = after[..close].trim();
    if params.is_empty() {
        return Err(err(
            "rust-import.domain",
            format!("fn `{name}` must take exactly one `&str` parameter"),
        ));
    }
    // Exactly one parameter, spelled `<ident>: &str` (a lifetime prefix
    // on the reference is allowed; nothing else is).
    let param = params.split(',').collect::<Vec<_>>();
    if param.len() != 1 {
        return Err(err(
            "rust-import.domain",
            format!(
                "fn `{name}` has {} parameters; the F0 domain takes exactly one `&str`",
                param.len()
            ),
        ));
    }
    let param = param[0].trim();
    // Accept `<ident>: &str` and `<ident>: &'lifetime str`; the type part
    // must be exactly a borrowed str — nothing wider, nothing owned.
    let ty = param.rsplit(':').next().unwrap_or("").trim();
    let borrowed = ty.strip_prefix('&').unwrap_or(ty).trim_start_matches('\'');
    if !borrowed.ends_with("str") || borrowed != "str" {
        return Err(err(
            "rust-import.domain",
            format!("fn `{name}` parameter `{param}` is not a borrowed `&str`"),
        ));
    }
    let rest = &after[close + 1..];
    let rest_trim = rest.trim_start();
    if rest_trim.starts_with('<') {
        return Err(err(
            "rust-import.domain",
            format!("fn `{name}` is generic; generics stay outside F0 (PLAN-963 owns them)"),
        ));
    }
    let Some(arrow) = rest_trim.find("->") else {
        return Err(err(
            "rust-import.domain",
            format!("fn `{name}` has no return type"),
        ));
    };
    let after_arrow = rest_trim[arrow + 2..].trim_start();
    let ret = after_arrow
        .split(|c: char| c == '{' || c == ';')
        .next()
        .unwrap_or("")
        .trim();
    if ret != "i32" && ret != "bool" && ret != "Option<i32>" {
        return Err(err(
            "rust-import.domain",
            format!("fn `{name}` returns `{ret}`; F0 fns return i32, bool or Option<i32>"),
        ));
    }
    Ok(ret.to_string())
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
    for f in &manifest.functions {
        let call = format!("{}(__f0_view(data, byte_len))", f.rust_name);
        match f.form {
            RustFnForm::PureI32 => {
                out.push_str(&format!(
                    "#[no_mangle]\npub extern \"C\" fn {export}(data: *const u8, byte_len: u64) -> i32 {{\n    match std::panic::catch_unwind(|| {{ {call} }}) {{\n        Ok(v) => v,\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); 0 }}\n    }}\n}}\n\n",
                    export = f.export,
                ));
            }
            RustFnForm::FallibleI32 { ref errors } => {
                // `None` maps to the SMALLEST named negative status (the
                // same canonical order `to_manual_manifest` freezes for
                // the error-model table — manifest and physical behavior
                // can never disagree; audit finding #1).
                let mut codes: Vec<i32> = errors.iter().map(|(_, c)| *c).collect();
                codes.sort_unstable();
                let none_code = codes[0];
                out.push_str(&format!(
                    "#[no_mangle]\npub extern \"C\" fn {export}(data: *const u8, byte_len: u64, out: *mut i32) -> i32 {{\n    match std::panic::catch_unwind(|| {{ {call} }}) {{\n        Ok(Some(v)) => {{ unsafe {{ *out = v; }} 0 }}\n        Ok(None) => {none_code},\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); {none_code} }}\n    }}\n}}\n\n",
                    export = f.export,
                ));
            }
            RustFnForm::PureBool => {
                out.push_str(&format!(
                    "#[no_mangle]\npub extern \"C\" fn {export}(data: *const u8, byte_len: u64) -> u8 {{\n    match std::panic::catch_unwind(|| {{ {call} }}) {{\n        Ok(v) => v as u8,\n        Err(_) => {{ __F0_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst); 0 }}\n    }}\n}}\n\n",
                    export = f.export,
                ));
            }
        }
    }
    out
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
        functions.push(FnFact {
            symbol: f.export.clone(),
            form,
            error_model,
            // The manual lane carries no semantic-service refs (the neutral
            // manifest keeps the versioned dotted symbol identity instead).
            semantic_ref: None,
            resource: Some(ResourceSig {
                format: RESOURCE_SIG_FORMAT.into(),
                params: vec![ResKind::ViewText],
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
        assert_eq!(import(generic).unwrap_err().code, "rust-import.domain");
        // owned String param
        let owned = "//! auto-ffi provider p/1 target=x86_64-pc-windows-msvc profile=p.v1\n// auto-ffi fn o symbol=p.o.v1 form=pure ret=i32\npub fn o(s: String) -> i32 { 0 }\n";
        assert_eq!(import(owned).unwrap_err().code, "rust-import.domain");
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
