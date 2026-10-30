//! Multi-object link driver (contract-decision §8/§10): assembles the
//! standard rust-lld input for a closure (per-function COFFs + `ac_start`
//! wrapper + trace support library + kernel32), runs it through the LANDED
//! contained executor (`run_deadlined_captured` — suspended start,
//! kill-on-close job, single deadline, capped raw byte capture) and locates
//! tools via the landed `find_rust_lld` / `find_sdk_um_dir`. The old
//! single-object `link_object_staged` is NOT reused or modified — the
//! multi-object staging here follows the same stage/commit failure
//! semantics:
//!
//! - every failure exit (link error, deadline, publish IO failure) reclaims
//!   EXACTLY the staged files this call created (staged objects, tmp exe,
//!   tmp receipt, backup) and reports blocked reclaims — never swallows
//!   them;
//! - with an existing output present, a failed publish RESTORES the old
//!   exe byte-identically (backup → place → rollback on any failure);
//! - the committed products of a successful call are the exe + the link
//!   receipt; staged object files are reclaimed on success too (the
//!   canonical objects live in the A09' store).

use crate::config::*;
use crate::MnError;
use ace_core::link::{find_rust_lld, find_sdk_um_dir, run_deadlined_captured, CaptureCaps};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

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

/// Commit a fully linked (and, for requests, already observed) collections
/// candidate. Both previous products survive every failed placement. Backups
/// are ordinary files under the explicitly supplied output directory.
pub(crate) fn publish_collections(candidate: &PeLink, output: &Path) -> Result<PeLink, MnError> {
    let exe_path = output.join("collections.exe");
    let receipt_path = output.join(".mn-collections-link.txt");
    publish_products(candidate, &exe_path, &receipt_path)
}

/// Same-volume transaction for a caller-selected pair of native products.
/// Candidate paths must be fresh files owned by this request.
pub fn publish_products(
    candidate: &PeLink,
    exe_path: &Path,
    receipt_path: &Path,
) -> Result<PeLink, MnError> {
    publish_products_checked(candidate, exe_path, receipt_path, |_| Ok(()))
}
fn publish_products_checked(
    candidate: &PeLink,
    exe_path: &Path,
    receipt_path: &Path,
    mut before_place: impl FnMut(usize) -> std::io::Result<()>,
) -> Result<PeLink, MnError> {
    let destinations = [exe_path, receipt_path];
    let sources = [
        candidate.exe_path.as_path(),
        candidate.receipt_path.as_path(),
    ];
    let backups = [
        candidate.exe_path.with_extension("old-exe"),
        candidate.receipt_path.with_extension("old-receipt"),
    ];
    // Refuse directories/reparse objects rather than treating them as outputs.
    for path in destinations.iter().chain(sources.iter()) {
        match std::fs::symlink_metadata(path) {
            Ok(meta) if !meta.file_type().is_file() => {
                return Err(MnError::Infra(format!(
                    "publish path is not a regular file: {}",
                    path.display()
                )))
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(MnError::Infra(format!("publish metadata: {e}")))
            }
            _ => {}
        }
    }
    let mut saved = [false; 2];
    let mut placed = [false; 2];
    let commit = (|| {
        for i in 0..2 {
            if destinations[i].is_file() {
                std::fs::rename(destinations[i], &backups[i])
                    .map_err(|e| format!("backup {}: {e}", destinations[i].display()))?;
                saved[i] = true;
            }
        }
        for i in 0..2 {
            before_place(i).map_err(|e| format!("place product {i}: {e}"))?;
            std::fs::rename(sources[i], destinations[i])
                .map_err(|e| format!("place {}: {e}", destinations[i].display()))?;
            placed[i] = true;
        }
        Ok::<_, String>(())
    })();
    if let Err(error) = commit {
        let mut failures = vec![];
        for i in (0..2).rev() {
            if placed[i] {
                if let Err(e) = std::fs::remove_file(destinations[i]) {
                    failures.push(format!(
                        "remove candidate {}: {e}",
                        destinations[i].display()
                    ));
                }
            }
            if saved[i] {
                if let Err(e) = std::fs::rename(&backups[i], destinations[i]) {
                    failures.push(format!("backup survives at {}: {e}", backups[i].display()));
                }
            }
        }
        return Err(MnError::Infra(format!(
            "collections publish: {error}; rollback: {}",
            if failures.is_empty() {
                "complete".into()
            } else {
                failures.join("; ")
            }
        )));
    }
    for i in 0..2 {
        if saved[i] {
            if let Some(error) = reclaim(&backups[i]) {
                eprintln!("collections committed; backup retained: {error}");
            }
        }
    }
    Ok(PeLink {
        exe_path: exe_path.to_path_buf(),
        receipt_path: receipt_path.to_path_buf(),
    })
}

#[cfg(test)]
mod collections_publication_tests {
    use super::*;
    #[test]
    fn failed_second_placement_restores_both_previous_products() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../docs/plans/attachments/889")
            .join(format!(
                "publish-rollback-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&root).unwrap();
        for had_previous in [false, true] {
            let output = root.join(if had_previous { "old" } else { "first" });
            std::fs::create_dir(&output).unwrap();
            let exe = output.join("output.exe");
            let receipt = output.join("output.txt");
            if had_previous {
                std::fs::write(&exe, b"old PE").unwrap();
                std::fs::write(&receipt, b"old receipt").unwrap();
            }
            let candidate = PeLink {
                exe_path: output.join("new.exe"),
                receipt_path: output.join("new.txt"),
            };
            std::fs::write(&candidate.exe_path, b"new PE").unwrap();
            std::fs::write(&candidate.receipt_path, b"new receipt").unwrap();
            let mut actual_second = false;
            let result = publish_products_checked(&candidate, &exe, &receipt, |i| {
                if i == 1 {
                    actual_second = true;
                    assert_eq!(std::fs::read(&exe).unwrap(), b"new PE");
                    Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "second placement fault",
                    ))
                } else {
                    Ok(())
                }
            });
            assert!(actual_second && result.is_err());
            if had_previous {
                assert_eq!(std::fs::read(&exe).unwrap(), b"old PE");
                assert_eq!(std::fs::read(&receipt).unwrap(), b"old receipt");
            } else {
                assert!(!exe.exists() && !receipt.exists());
            }
        }
    }
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

/// The link-stage wall. Frozen at BUILD_BUDGET; the
/// `MN818_TEST_LINK_WALL_MS` seam may only SHORTEN it (deadline-kill
/// counter-controls run in seconds, never weakening the production
/// ceiling). Never set outside tests.
fn link_wall() -> Duration {
    let frozen = BUILD_BUDGET;
    match std::env::var("MN818_TEST_LINK_WALL_MS") {
        Ok(v) => v
            .parse::<u64>()
            .ok()
            .map(|ms| Duration::from_millis(ms))
            .filter(|d| *d < frozen)
            .unwrap_or(frozen),
        Err(_) => frozen,
    }
}

/// Remove one owned staged file; NotFound is success. Returns the blocked
/// report (path + OS error) for the error message — a reclaim failure is
/// real state the caller must see, never swallowed.
fn reclaim(path: &Path) -> Option<String> {
    match std::fs::remove_file(path) {
        Ok(()) => None,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => Some(format!("{} ({e})", path.display())),
    }
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
    // Tool binaries can be large. Hash the same complete byte stream without
    // allocating a file-sized temporary on every fresh worker process.
    use std::io::Read;
    let mut file = std::fs::File::open(p)
        .map_err(|e| MnError::Infra(format!("tool read {}: {e}", p.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = match file.read(&mut buffer) {
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(MnError::Infra(format!("tool read {}: {e}", p.display()))),
        };
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let h = hasher.finalize().to_hex().to_string();
    cache.lock().unwrap().insert(key, h.clone());
    Ok(h)
}

/// Link identities: (lld sha, sdk tag, kernel32 sha). Tool DISCOVERY
/// (rustc sysroot / registry walks — subprocess-priced) is memoized per
/// process for the non-override path; env overrides are re-read on every
/// call (test seams), and content binding still goes through
/// `file_identity`, which re-hashes on any (path, len) change.
pub fn link_identities() -> Result<(String, String, String), MnError> {
    let (lld, sdk) = located_tools()?;
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

/// New-profile restoration re-hashes actual tools even when their path/length
/// are unchanged. Keep the five-profile memo and identity behavior intact.
pub fn collections_link_identities() -> Result<(String, String, String), MnError> {
    let (lld, sdk) = located_tools()?;
    let lld_hash = crate::collections_runtime::file_hash(&lld)?;
    let kernel_hash = crate::collections_runtime::file_hash(&sdk.join("kernel32.lib"))?;
    let sdk_tag = sdk
        .parent()
        .and_then(|p| p.file_name())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| sdk.display().to_string());
    Ok((lld_hash, sdk_tag, kernel_hash))
}

/// Optional cross-process DISK memo for tool DISCOVERY (path hints only —
/// content binding still goes through `file_identity`, which re-hashes per
/// process). Set by short-lived worker processes that share a store root;
/// env overrides always win and are re-read per call.
static TOOL_DISK_MEMO: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

pub fn set_tool_disk_memo(path: std::path::PathBuf) {
    *TOOL_DISK_MEMO.lock().unwrap() = Some(path);
}

fn read_tool_disk_memo() -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    let guard = TOOL_DISK_MEMO.lock().unwrap();
    let path = guard.as_ref()?;
    let text = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let lld = std::path::PathBuf::from(v.get("lld")?.as_str()?);
    let sdk = std::path::PathBuf::from(v.get("sdk_um")?.as_str()?);
    if lld.is_file() && sdk.join("kernel32.lib").is_file() {
        Some((lld, sdk))
    } else {
        None
    }
}

fn write_tool_disk_memo(lld: &std::path::Path, sdk: &std::path::Path) {
    let guard = TOOL_DISK_MEMO.lock().unwrap();
    if let Some(path) = guard.as_ref() {
        let v = serde_json::json!({
            "lld": lld.display().to_string(),
            "sdk_um": sdk.display().to_string(),
        });
        let _ = std::fs::write(path, v.to_string());
    }
}

/// Resolve (rust-lld, SDK um dir): env overrides per call (the
/// sleeping-linker/timeout seams stay injectable), then the disk memo,
/// then the process-memoized default discovery (which seeds the disk memo
/// for the next short-lived process).
fn located_tools() -> Result<(std::path::PathBuf, std::path::PathBuf), MnError> {
    fn memoized_default() -> Result<(std::path::PathBuf, std::path::PathBuf), MnError> {
        use std::sync::OnceLock;
        static MEMO: OnceLock<Result<(std::path::PathBuf, std::path::PathBuf), String>> =
            OnceLock::new();
        MEMO.get_or_init(|| {
            let lld = find_rust_lld().map_err(|d| d.message.clone())?;
            let sdk = find_sdk_um_dir().map_err(|d| d.message.clone())?;
            Ok((lld, sdk))
        })
        .clone()
        .map_err(MnError::Infra)
    }
    let env_lld = std::env::var("AC741_RUST_LLD")
        .ok()
        .filter(|p| std::path::Path::new(p).is_file())
        .map(std::path::PathBuf::from);
    let env_sdk = std::env::var("AC741_SDK_UM_LIB")
        .ok()
        .filter(|d| std::path::Path::new(d).join("kernel32.lib").is_file())
        .map(std::path::PathBuf::from);
    if let (Some(lld), Some(sdk)) = (&env_lld, &env_sdk) {
        return Ok((lld.clone(), sdk.clone()));
    }
    let discovered = match read_tool_disk_memo() {
        Some(pair) => pair,
        None => {
            let pair = memoized_default()?;
            write_tool_disk_memo(&pair.0, &pair.1);
            pair
        }
    };
    Ok((
        env_lld.unwrap_or(discovered.0),
        env_sdk.unwrap_or(discovered.1),
    ))
}

pub fn link_and_run(
    objects: &[ObjectInput],
    entry_wrapper: &[u8],
    entry_symbol: &str,
    workdir: &Path,
    support_libs: &[PathBuf],
    link_receipt_extra: &str,
) -> Result<PeRun, MnError> {
    let linked = link_only(
        objects,
        entry_wrapper,
        entry_symbol,
        workdir,
        support_libs,
        link_receipt_extra,
    )?;
    run_pe(
        &linked.exe_path,
        RUN_BUDGET,
        CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
    )
}

/// The committed products of a successful [`link_only`] call: the published
/// exe and the link receipt next to it. Staged inputs are already
/// reclaimed — these are the only files this call leaves behind.
pub struct PeLink {
    pub exe_path: PathBuf,
    pub receipt_path: PathBuf,
}

/// Link WITHOUT running (PLAN-871 T-04): the same stage/commit/rollback
/// discipline as [`link_and_run`], stopping after the receipt is published.
/// Consumers that must stay in charge of execution (the ac-case worker's
/// budgeted/capped run leg, the `ac-native build` CLI) use this; a run
/// failure therefore can never look like a build failure.
pub fn link_only(
    objects: &[ObjectInput],
    entry_wrapper: &[u8],
    entry_symbol: &str,
    workdir: &Path,
    support_libs: &[PathBuf],
    link_receipt_extra: &str,
) -> Result<PeLink, MnError> {
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
    // ---- stage: owned files of THIS call ------------------------------
    // Content-stable reuse (PLAN-818 T-11/R1): an object file whose staged
    // bytes are already on disk (same length AND content hash) is NOT
    // rewritten — warm incremental rounds only restage the units that
    // changed plus the wrapper. Cold paths stage everything (no files
    // exist yet); the byte comparison is exact, not a heuristic.
    let mut owned: Vec<PathBuf> = Vec::new();
    let stage: Vec<PathBuf> = std::iter::once(("mn8-ac-start".to_string(), entry_wrapper))
        .chain(objects.iter().map(|o| (o.name.clone(), o.bytes.as_slice())))
        .enumerate()
        .map(|(i, (name, bytes))| {
            let p = workdir.join(format!("{:02}-{}.obj", i, safe_name(&name)));
            let unchanged = std::fs::metadata(&p)
                .map(|m| m.len() == bytes.len() as u64)
                .unwrap_or(false)
                && std::fs::read(&p).map(|prev| prev == bytes).unwrap_or(false);
            if !unchanged {
                std::fs::write(&p, bytes)
                    .map_err(|e| MnError::Infra(format!("stage {}: {e}", p.display())))?;
            }
            owned.push(p.clone());
            Ok(p)
        })
        .collect::<Result<Vec<_>, MnError>>()?;
    // Reclaim every owned staged file; return the composed failure with
    // the blocked-reclaim report appended.
    let fail = |mut err: MnError, mut owned: Vec<PathBuf>| -> MnError {
        let mut blocked = Vec::new();
        for p in owned.drain(..) {
            if let Some(b) = reclaim(&p) {
                blocked.push(b);
            }
        }
        if !blocked.is_empty() {
            let msg = format!(
                "{}; staged files NOT reclaimed (release the blocking handles, then remove): {}",
                err.message(),
                blocked.join(", ")
            );
            err = match err {
                MnError::Infra(_) => MnError::Infra(msg),
                other => other,
            };
        }
        err
    };

    let (lld, sdk) = located_tools()?;
    let tmp_exe = workdir.join(format!(".tmp-mn8-exe-{tag}"));
    owned.push(tmp_exe.clone());
    let tmp_receipt = workdir.join(format!(".tmp-mn8-link-{tag}.txt"));
    owned.push(tmp_receipt.clone());
    let exe_path = workdir.join(format!("mn8-out-{tag}.exe"));
    let receipt_path = workdir.join(".mn-link.txt");
    let backup = workdir.join(format!(".tmp-mn8-old-{tag}.exe"));

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
    // PLAN-870: the link line carries every support library (the trace
    // lib, the text runtime lib) in the given order.
    for lib in support_libs {
        cmd.arg(lib.display().to_string());
    }
    let linked = match run_deadlined_captured(
        &mut cmd,
        link_wall(),
        CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
    ) {
        Ok(r) => r,
        // Deadline/containment/spawn: the tree is already killed and
        // reaped by the executor; reclaim staging, report.
        Err(e) => return Err(fail(MnError::Infra(format!("rust-lld: {e:?}")), owned)),
    };
    if linked.exit_code != Some(0) {
        return Err(fail(
            MnError::Infra(format!(
                "rust-lld exit {:?}: {}",
                linked.exit_code,
                String::from_utf8_lossy(&linked.stderr).trim()
            )),
            owned,
        ));
    }

    // ---- publish: stage receipt → backup old → place → drop backup -----
    let receipt = format!(
        "PLAN-818 link receipt\nlinker: {}\nentry: {}\nobjs: {}\n{}\n",
        lld.display(),
        entry_symbol,
        stage.len(),
        link_receipt_extra,
    );
    if let Err(e) = std::fs::write(&tmp_receipt, receipt) {
        return Err(fail(MnError::Infra(format!("stage receipt: {e}")), owned));
    }
    // A final path occupied by a DIRECTORY is user property: never backed
    // up, never deleted — the place step fails on it (MoveFileEx cannot
    // replace a directory) and the failure exit reports it.
    let had_old = exe_path.is_file();
    if had_old {
        if let Err(e) = std::fs::rename(&exe_path, &backup) {
            return Err(fail(
                MnError::Infra(format!("backup old output: {e}")),
                owned,
            ));
        }
        owned.push(backup.clone());
    }
    // Place the new exe; on failure restore the old one byte-identically.
    if let Err(e) = std::fs::rename(&tmp_exe, &exe_path) {
        let mut restore_note = String::new();
        if had_old {
            match std::fs::rename(&backup, &exe_path) {
                Ok(()) => {
                    owned.retain(|p| p != &backup);
                }
                Err(re) => {
                    restore_note = format!(
                        "; old output backup survives at {} ({re})",
                        backup.display()
                    )
                }
            }
        }
        return Err(fail(
            MnError::Infra(format!("publish staged exe: {e}{restore_note}")),
            owned,
        ));
    }
    owned.retain(|p| p != &tmp_exe);
    // Place the receipt; on failure roll the exe back to the old output.
    if let Err(e) = std::fs::rename(&tmp_receipt, &receipt_path) {
        // Roll the just-placed exe back to the backup (or remove it when
        // there was no old output) so the previous artifacts survive.
        let rollback_note = if had_old {
            let _ = std::fs::rename(&backup, &exe_path);
            owned.retain(|p| p != &backup);
            "; old output restored"
        } else {
            let _ = std::fs::remove_file(&exe_path);
            "; new output removed (no previous output)"
        };
        return Err(fail(
            MnError::Infra(format!("publish receipt: {e}{rollback_note}")),
            owned,
        ));
    }
    owned.retain(|p| p != &tmp_receipt);
    if had_old {
        // Committed: drop the backup.
        if let Some(b) = reclaim(&backup) {
            // The commit is complete; a blocked backup removal is reported
            // but does not un-publish a correct output.
            eprintln!("mn8 link: backup not reclaimed after commit: {b}");
        }
        owned.retain(|p| p != &backup);
    }
    // Success: staged objects were inputs, not products — reclaim them.
    for p in &stage {
        let _ = reclaim(p);
    }
    owned.clear();

    Ok(PeLink {
        exe_path,
        receipt_path,
    })
}

/// Contained run of an already-published PE (PLAN-871 T-04): single
/// deadline, kill-on-close job, capped raw capture — the ONLY way this
/// crate executes a product binary. A crash/timeout is INFRA, never a
/// language trap, and the published evidence from the link stays put.
pub fn run_pe(exe_path: &Path, budget: Duration, caps: CaptureCaps) -> Result<PeRun, MnError> {
    let mut run_cmd = Command::new(exe_path);
    let run = match run_deadlined_captured(&mut run_cmd, budget, caps) {
        Ok(r) => r,
        // A native crash/timeout is INFRA, never a language trap.
        Err(e) => return Err(MnError::Infra(format!("run {}: {e:?}", exe_path.display()))),
    };
    Ok(PeRun {
        exit_code: run.exit_code,
        stdout: run.stdout,
        stderr: run.stderr,
        exe_path: exe_path.to_path_buf(),
    })
}
