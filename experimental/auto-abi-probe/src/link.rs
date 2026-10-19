//! PE linking over the merged ac-core staged-link entry (plan §3, T-03).
//! Nothing here invents a second linker: rust-lld via
//! `link_object_staged`, SDK import libs via `find_sdk_um_dir`.

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ProbeError(pub String);

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "abi.probe: {}", self.0)
    }
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
    let lld = auto_ac_prototype::link::find_rust_lld()
        .map_err(|e| ProbeError(format!("find_rust_lld: {e:?}")))?;
    let sdk = auto_ac_prototype::link::find_sdk_um_dir()
        .map_err(|e| ProbeError(format!("find_sdk_um_dir: {e:?}")))?;
    let exe = dir.join("probe.exe");
    let staged =
        auto_ac_prototype::link::link_object_staged(&lld, &sdk, &objs[0], &exe, entry, &objs[1..])
            .map_err(|e| ProbeError(format!("staged link: {e:?}")))?;
    Ok(staged.tmp_exe)
}
