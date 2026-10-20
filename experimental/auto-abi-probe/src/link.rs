//! PE linking over the merged ac-core staged-link entry (plan §3, T-03).
//! Nothing here invents a second linker: rust-lld via
//! `link_object_staged`, SDK import libs via `find_sdk_um_dir`.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug)]
pub struct ProbeError(pub String);

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "abi.probe: {}", self.0)
    }
}

/// F-797-R4-09 (R8): the ac-core discovery entries spawn `rustc --print
/// sysroot` / `reg query` on EVERY call — two subprocesses per link that
/// dominated the per-request link stage. The toolchain identity cannot
/// change mid-process, so the probe caches the resolution once per
/// process (ac-core itself stays untouched).
fn cached_lld() -> Result<&'static PathBuf, ProbeError> {
    static CACHE: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CACHE
        .get_or_init(|| auto_ac_prototype::link::find_rust_lld().map_err(|e| format!("{e:?}")))
        .as_ref()
        .map_err(|e| ProbeError(e.clone()))
}

fn cached_sdk_um() -> Result<&'static PathBuf, ProbeError> {
    static CACHE: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CACHE
        .get_or_init(|| auto_ac_prototype::link::find_sdk_um_dir().map_err(|e| format!("{e:?}")))
        .as_ref()
        .map_err(|e| ProbeError(e.clone()))
}

/// Link object files into a staged PE (not published) with the REAL
/// ac-core staged-link entry. `objs[0]` is the primary object (it must
/// contain the entry symbol); the rest ride along as additional link
/// inputs — exactly how mixed-producer PEs are formed. Single-object
/// links are legal (e.g. the C layout oracle measures by itself); the
/// INTEROP requirement of two/three independent producers is a property
/// the leg tests assert on their own inputs, not a linker constraint.
pub fn link_pe(dir: &Path, objs: &[PathBuf], entry: &str) -> Result<PathBuf, ProbeError> {
    assert!(!objs.is_empty(), "a link needs at least one object");
    let lld = cached_lld()?;
    let sdk = cached_sdk_um()?;
    let exe = dir.join("probe.exe");
    let staged =
        auto_ac_prototype::link::link_object_staged(lld, sdk, &objs[0], &exe, entry, &objs[1..])
            .map_err(|e| ProbeError(format!("staged link: {e:?}")))?;
    Ok(staged.tmp_exe)
}
