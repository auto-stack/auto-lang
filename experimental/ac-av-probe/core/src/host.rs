//! PLAN-863: portable host declarations and per-run authority.
//! Trusted native providers have no guest reentry API. This is not memory isolation.
use crate::av::CancelFlag;
use crate::mir::{MirHostImport, MirModule, MirType};
pub const CATEGORY: &str = "console/1";
pub const STDOUT: &str = "console.stdout";
pub const STDERR: &str = "console.stderr";
pub const MAX_CALL_BYTES: usize = 65_536;
pub const MAX_RUN_BYTES: usize = 1_048_576;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Stdout,
    Stderr,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionError {
    pub code: &'static str,
    pub message: String,
}
impl std::fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for AdmissionError {}
fn err(code: &'static str, message: &str) -> AdmissionError {
    AdmissionError {
        code,
        message: message.into(),
    }
}
/// Stable symbol -> exact readonly input type and stream. No dynamic resolver.
pub fn signature(symbol: &str) -> Option<(MirType, Stream)> {
    let (rest, stream) = if let Some(s) = symbol.strip_prefix("auto.console.print.") {
        (s, Stream::Stdout)
    } else if let Some(s) = symbol.strip_prefix("auto.console.eprint.") {
        (s, Stream::Stderr)
    } else {
        return None;
    };
    Some((
        match rest {
            "i32.v1" => MirType::I32,
            "bool.v1" => MirType::Bool,
            "str.v1" => MirType::StrView,
            "char.v1" => MirType::Char,
            _ => return None,
        },
        stream,
    ))
}
pub fn capability(stream: Stream) -> &'static str {
    match stream {
        Stream::Stdout => STDOUT,
        Stream::Stderr => STDERR,
    }
}
pub fn declaration(symbol: &str, sig: u32) -> Result<MirHostImport, AdmissionError> {
    let (_, stream) =
        signature(symbol).ok_or_else(|| err("host.unknown-symbol", "unknown console symbol"))?;
    Ok(MirHostImport {
        category: CATEGORY.into(),
        symbol: symbol.into(),
        sig,
        capability: capability(stream).into(),
        max_call_bytes: MAX_CALL_BYTES as u32,
    })
}
/// Structural validation covers every declaration, even without a reachable call.
pub fn validate(module: &MirModule) -> Result<(), AdmissionError> {
    if !module.is_host_console_profile() && !module.host_imports.is_empty() {
        return Err(err(
            "host.profile",
            "host imports require the host-console profile",
        ));
    }
    if module.host_imports.len() > 8 {
        return Err(err(
            "host.duplicate",
            "at most eight console overloads can be declared",
        ));
    }
    if module
        .imports
        .iter()
        .any(|i| i.symbol.starts_with("auto.console."))
    {
        return Err(err(
            "host.category",
            "a host symbol cannot be a guest import",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for h in &module.host_imports {
        if !seen.insert(&h.symbol) {
            return Err(err("host.duplicate", "duplicate host declaration"));
        }
        let (input, stream) = signature(&h.symbol)
            .ok_or_else(|| err("host.unknown-symbol", "unknown console symbol"))?;
        let sig = module
            .sig_of(h.sig)
            .ok_or_else(|| err("host.signature", "host signature is not a function type"))?;
        if h.category != CATEGORY
            || h.capability != capability(stream)
            || h.max_call_bytes != MAX_CALL_BYTES as u32
            || sig.params.len() != 1
            || module.types.get(sig.params[0] as usize) != Some(&input)
            || module.types.get(sig.result as usize) != Some(&MirType::Unit)
        {
            return Err(err(
                "host.signature",
                "host category/symbol/signature/capability/budget tuple mismatch",
            ));
        }
        if !module.capabilities.contains(&h.capability) {
            return Err(err(
                "host.unauthorized",
                "host capability requirement is missing",
            ));
        }
    }
    Ok(())
}
/// A loaded structural credential never preserves grants; check current authority for every run.
pub fn authorize(module: &MirModule, grants: &[String]) -> Result<(), AdmissionError> {
    if !module.is_host_console_profile() {
        return Err(err(
            "host.profile",
            "console execution requires the host-console profile",
        ));
    }
    validate(module)?;
    for h in &module.host_imports {
        if !grants.contains(&h.capability) {
            return Err(err(
                "host.unauthorized",
                "host capability is not currently granted",
            ));
        }
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostError {
    Resource,
    Cancelled,
    Io(String),
}
/// One already-budgeted LF-terminated call. Providers poll cancel immediately before commit.
/// Successful writes are atomic for the in-memory console provider. No guest callback is exposed.
pub trait ConsoleHost {
    /// Already retained capture bytes are charged if a provider is reused.
    fn retained_bytes(&self) -> usize;
    fn write(&mut self, stream: Stream, bytes: &[u8], cancel: &CancelFlag)
        -> Result<(), HostError>;
}
