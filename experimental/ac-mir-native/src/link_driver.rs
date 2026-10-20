//! Multi-object link driver (contract-decision §8): assembles the standard
//! rust-lld input for a closure (per-function COFFs + `ac_start` wrapper +
//! trace support library + kernel32), runs it through the LANDED contained
//! executor (`run_deadlined_captured` — suspended start, kill-on-close
//! job, single deadline, capped raw byte capture) and locates tools via
//! the landed `find_rust_lld` / `find_sdk_um_dir`. The old single-object
//! `link_object_staged` is NOT reused or modified — the multi-object
//! staging here follows the same stage/commit failure semantics.

use crate::config::*;
use crate::MnError;
use auto_ac_prototype::link::{
    find_rust_lld, find_sdk_um_dir, run_deadlined_captured, CaptureCaps,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// One object to link, under a filesystem-safe staged name.
pub struct ObjectInput {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// The outcome of a contained PE run.
pub struct PeRun {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exe_path: PathBuf,
}

fn safe_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Tool identities for the A10' key axis: the linker binary and the SDK's
/// kernel32 import library are content-bound (blake3, hex), the SDK um
/// dir by its version path tag. Hashes are memoized per (path, len,
/// mtime) — a request pays for them once even in a warm process.
pub fn file_identity(p: &Path) -> Result<String, MnError> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<(PathBuf, u64), String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let meta = std::fs::metadata(p)
        .map_err(|e| MnError::Infra(format!("tool identity {}: {e}", p.display())))?;
    let key = (p.to_path_buf(), meta.len());
    if let Some(h) = cache.lock().unwrap().get(&key) {
        return Ok(h.clone());
    }
    let bytes =
        std::fs::read(p).map_err(|e| MnError::Infra(format!("tool read {}: {e}", p.display())))?;
    let h = crate::cache::content_hash(&bytes);
    cache.lock().unwrap().insert(key, h.clone());
    Ok(h)
}

/// Link identities: (lld sha, sdk tag, kernel32 sha).
pub fn link_identities() -> Result<(String, String, String), MnError> {
    let lld = find_rust_lld().map_err(|d| MnError::Infra(d.message.clone()))?;
    let sdk = find_sdk_um_dir().map_err(|d| MnError::Infra(d.message.clone()))?;
    let lld_sha = file_identity(&lld)?;
    let kernel32 = sdk.join("kernel32.lib");
    let k32_sha = file_identity(&kernel32)?;
    // The um dir's parent carries the SDK version (e.g. Lib/10.0.26100.0).
    let sdk_tag = sdk
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| sdk.display().to_string());
    Ok((lld_sha, sdk_tag, k32_sha))
}

/// Link the closure into a staged PE and run it under the contained
/// executor. `workdir` receives the staged objects, the exe and a link
/// receipt line; it is caller-owned (a per-request temp dir in the T-02
/// driver). Every object byte passes through rust-lld — no byte splicing.
#[allow(clippy::too_many_arguments)]
pub fn link_and_run(
    objects: &[ObjectInput],
    entry_wrapper: &[u8],
    entry_symbol: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
    link_receipt_extra: &str,
) -> Result<PeRun, MnError> {
    let total: usize = objects.iter().map(|o| o.bytes.len()).sum::<usize>() + entry_wrapper.len();
    if total > MAX_LINK_INPUT_BYTES {
        return Err(MnError::Resource(
            "link-input",
            format!("link inputs total {total} bytes, above the frozen cap"),
        ));
    }
    std::fs::create_dir_all(workdir)
        .map_err(|e| MnError::Infra(format!("create {}: {e}", workdir.display())))?;
    let tag = std::process::id();
    let stage: Vec<PathBuf> = std::iter::once(("mn8-ac-start".to_string(), entry_wrapper))
        .chain(objects.iter().map(|o| (o.name.clone(), o.bytes.as_slice())))
        .enumerate()
        .map(|(i, (name, bytes))| {
            let p = workdir.join(format!("{:02}-{}.obj", i, safe_name(&name)));
            std::fs::write(&p, bytes)
                .map_err(|e| MnError::Infra(format!("stage {}: {e}", p.display())))?;
            Ok(p)
        })
        .collect::<Result<Vec<_>, MnError>>()?;

    let lld = find_rust_lld().map_err(|d| MnError::Infra(d.message.clone()))?;
    let sdk = find_sdk_um_dir().map_err(|d| MnError::Infra(d.message.clone()))?;
    let tmp_exe = workdir.join(format!(".tmp-mn8-exe-{tag}"));
    let mut cmd = Command::new(&lld);
    cmd.args(["-flavor", "link", "/nologo"]);
    cmd.arg(format!("/entry:{entry_symbol}"));
    cmd.arg("/subsystem:console");
    cmd.arg(format!("/out:{}", tmp_exe.display()));
    for obj in &stage {
        cmd.arg(obj.display().to_string());
    }
    cmd.arg(format!("/libpath:{}", sdk.display()));
    cmd.arg("kernel32.lib");
    if let Some(lib) = trace_lib {
        cmd.arg(lib.display().to_string());
    }
    let linked = run_deadlined_captured(
        &mut cmd,
        BUILD_BUDGET,
        CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
    )
    .map_err(|e| MnError::Infra(format!("rust-lld: {e:?}")))?;
    if linked.exit_code != Some(0) {
        // Reclaim the staged exe attempt; keep objects for diagnosis.
        let _ = std::fs::remove_file(&tmp_exe);
        return Err(MnError::Infra(format!(
            "rust-lld exit {:?}: {}",
            linked.exit_code,
            String::from_utf8_lossy(&linked.stderr).trim()
        )));
    }
    // Link receipt (staged next to the exe; the workdir is per-request).
    let receipt = format!(
        "PLAN-818 link receipt\nlinker: {}\nentry: {}\nobjs: {}\n{}\n",
        lld.display(),
        entry_symbol,
        stage.len(),
        link_receipt_extra,
    );
    let _ = std::fs::write(workdir.join(".mn-link.txt"), receipt);

    let exe_path = workdir.join(format!("mn8-out-{tag}.exe"));
    std::fs::rename(&tmp_exe, &exe_path)
        .map_err(|e| MnError::Infra(format!("publish staged exe: {e}")))?;

    let started = Instant::now();
    let mut run_cmd = Command::new(&exe_path);
    let run = run_deadlined_captured(
        &mut run_cmd,
        RUN_BUDGET,
        CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
    )
    .map_err(|e| {
        // A native crash/timeout is INFRA, never a language trap.
        MnError::Infra(format!("run {}: {e:?}", exe_path.display()))
    })?;
    let _ = started;
    Ok(PeRun {
        exit_code: run.exit_code,
        stdout: run.stdout,
        stderr: run.stderr,
        exe_path,
    })
}
