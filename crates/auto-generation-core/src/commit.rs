//! Neutral staged-commit primitives (PLAN-1006 T-02, SD-03).
//!
//! Single owner: `auto-generation-core`. A consumer stages a whole payload
//! directory through [`commit_managed_directory`]: a uniquely named staging
//! sibling is created with a pid marker, the caller populates and strictly
//! verifies it, then a previous root — only when the caller verifies it as a
//! managed payload of the same kind — is preserved in a backup sibling and
//! the staging directory is swapped in. Any failure restores the previous
//! root byte-for-byte (renames only, no markers written into preserved
//! trees), keeps the failed new payload for diagnosis once it had been
//! swapped in, and never deletes a staging directory owned by another
//! process (fail-closed: only same-pid leftovers are swept).
//!
//! Boundaries (agreed in PLAN-1006 §2): no Web/Man/UI domain knowledge, no
//! installer scripting, no process-environment mutation. Trees containing a
//! junction or symlink anywhere are rejected — a reparse point inside a
//! copied tree can redirect writes outside the payload.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use auto_val::AutoResult;
use sha2::{Digest, Sha256};

/// Marker file placed at the staging root while it is being populated.
pub const STAGING_MARKER_FILE: &str = ".auto-staged-commit.marker";
/// Schema tag inside the staging marker document.
pub const STAGING_MARKER_SCHEMA: &str = "auto-staged-commit/1";
/// Directory names never copied or hashed into a payload tree.
const CLEAN_SKIP_DIRS: [&str; 4] = [".git", "target", "node_modules", "__pycache__"];

/// Content identity of one file inside a payload tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRecord {
    pub bytes: u64,
    pub sha256: String,
}

impl FileRecord {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({"bytes": self.bytes, "sha256": self.sha256})
    }
}

/// sha256 of a file's bytes, hex encoded.
pub fn sha256_file(path: &Path) -> AutoResult<String> {
    let bytes = fs::read(path).map_err(|e| format!("hash 读取失败 {}: {e}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

/// sha256 of in-memory bytes, hex encoded.
pub fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn reject_link(path: &Path) -> AutoResult<()> {
    let meta =
        fs::symlink_metadata(path).map_err(|e| format!("stat 失败 {}: {e}", path.display()))?;
    if meta.file_type().is_symlink() {
        return Err(format!("载荷树禁止 junction/symlink: {}", path.display()).into());
    }
    Ok(())
}

/// Copy `source` into a fresh `destination` directory, skipping build/VCS
/// junk and rejecting any junction/symlink in the source tree. Returns the
/// number of files copied. `destination` must not exist.
pub fn copy_tree_clean(source: &Path, destination: &Path) -> AutoResult<u64> {
    if destination.exists() {
        return Err(format!("目标已存在，拒绝覆盖: {}", destination.display()).into());
    }
    copy_dir(source, destination)
}

fn copy_dir(source: &Path, destination: &Path) -> AutoResult<u64> {
    reject_link(source)?;
    fs::create_dir_all(destination)
        .map_err(|e| format!("创建目录失败 {}: {e}", destination.display()))?;
    let mut copied = 0u64;
    let mut entries: Vec<_> = fs::read_dir(source)
        .map_err(|e| format!("读取目录失败 {}: {e}", source.display()))?
        .flatten()
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        let path = entry.path();
        let target = destination.join(&name);
        if CLEAN_SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        reject_link(&path)?;
        if path.is_dir() {
            copied += copy_dir(&path, &target)?;
        } else {
            fs::copy(&path, &target).map_err(|e| {
                format!(
                    "复制文件失败 {} -> {}: {e}",
                    path.display(),
                    target.display()
                )
            })?;
            copied += 1;
        }
    }
    Ok(copied)
}

/// Deterministic file map of a payload tree: relative posix path → content
/// identity. Skips build/VCS junk and the given root-level files (e.g. the
/// payload's own manifest). Rejects junctions/symlinks anywhere.
pub fn file_map(root: &Path, skip_files: &[&str]) -> AutoResult<BTreeMap<String, FileRecord>> {
    let mut map = BTreeMap::new();
    walk_map(root, root, skip_files, &mut map)?;
    Ok(map)
}

fn walk_map(
    root: &Path,
    dir: &Path,
    skip_files: &[&str],
    map: &mut BTreeMap<String, FileRecord>,
) -> AutoResult<()> {
    reject_link(dir)?;
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("读取目录失败 {}: {e}", dir.display()))?
        .flatten()
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        if CLEAN_SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        let path = entry.path();
        reject_link(&path)?;
        if path.is_dir() {
            walk_map(root, &path, skip_files, map)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| format!("载荷路径越界: {}", path.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            if dir == root && skip_files.contains(&name.as_str()) {
                continue;
            }
            let bytes = entry
                .metadata()
                .map_err(|e| format!("stat 失败 {}: {e}", path.display()))?
                .len();
            map.insert(
                relative,
                FileRecord {
                    bytes,
                    sha256: sha256_file(&path)?,
                },
            );
        }
    }
    Ok(())
}

fn unique_sibling(root: &Path, kind: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    root.with_file_name(format!(
        ".{}.auto-{kind}-{}-{nanos}",
        root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("payload"),
        std::process::id()
    ))
}

/// Staging sibling path for a payload root (unique per call).
pub fn staging_sibling(root: &Path) -> PathBuf {
    unique_sibling(root, "staging")
}

fn marker_document(stage: &Path) -> Option<serde_json::Value> {
    let text = fs::read_to_string(stage.join(STAGING_MARKER_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Remove leftover staging siblings of `root` owned by the current process;
/// preserve and report every foreign staging directory (unknown liveness is
/// treated as alive — fail-closed, no platform process API).
pub fn sweep_own_staging(root: &Path) -> Vec<PathBuf> {
    let prefix = format!(
        ".{}.auto-staging-",
        root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("payload")
    );
    let mut preserved = Vec::new();
    let Ok(parent) = fs::read_dir(root.parent().unwrap_or(Path::new("."))) else {
        return preserved;
    };
    for entry in parent.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with(&prefix) || !entry.path().is_dir() {
            continue;
        }
        let Some(marker) = marker_document(&entry.path()) else {
            preserved.push(entry.path());
            continue;
        };
        let ours = marker["schema"].as_str() == Some(STAGING_MARKER_SCHEMA)
            && marker["pid"].as_u64() == Some(u64::from(std::process::id()));
        if ours {
            if fs::remove_dir_all(entry.path()).is_err() {
                preserved.push(entry.path());
            }
        } else {
            preserved.push(entry.path());
        }
    }
    preserved
}

/// Outcome of a successful staged commit.
#[derive(Debug)]
pub struct CommitOutcome {
    pub root: PathBuf,
    pub backup_root: Option<PathBuf>,
    /// Foreign staging siblings preserved (reported, never deleted).
    pub preserved_foreign_staging: Vec<PathBuf>,
}

/// Stage, verify and atomically swap a whole payload directory.
///
/// * `root` — final payload location. When it exists and is non-empty,
///   `managed` must accept it as a valid prior payload of the same kind;
///   an unknown non-empty directory is a named error and stays untouched.
/// * `payload_kind` — marker identity (e.g. "web-member-delivery").
/// * `prepare` — populate the freshly created staging directory (marker
///   already written).
/// * `verify_payload` — strict content verification of the staged payload
///   before the swap and of the committed root afterwards.
/// * `fail_point` — optional injected failure (`"prepare"`, `"verify"`,
///   `"swap"`) for transaction tests.
///
/// On failure the previous root is restored byte-for-byte; a payload that
/// had already been swapped in is preserved in a `.auto-failed-*` sibling
/// for diagnosis. The error message names the phase, the restored root and
/// every preserved artifact.
pub fn commit_managed_directory(
    root: &Path,
    payload_kind: &str,
    managed: &dyn Fn(&Path) -> AutoResult<()>,
    prepare: &dyn Fn(&Path) -> AutoResult<()>,
    verify_payload: &dyn Fn(&Path) -> AutoResult<()>,
    fail_point: Option<&str>,
) -> AutoResult<CommitOutcome> {
    let root = std::path::absolute(root)
        .map_err(|e| format!("payload 根路径无效 {}: {e}", root.display()))?;
    let parent = root
        .parent()
        .ok_or_else(|| format!("payload 根无父目录: {}", root.display()))?
        .to_path_buf();
    fs::create_dir_all(&parent)
        .map_err(|e| format!("payload 父目录不可创建 {}: {e}", parent.display()))?;
    let preserved_foreign_staging = sweep_own_staging(&root);
    if root.exists() {
        if !root.is_dir() {
            return Err(format!("payload 根不是目录: {}", root.display()).into());
        }
        let non_empty = fs::read_dir(&root)
            .map_err(|e| format!("payload 根不可读 {}: {e}", root.display()))?
            .next()
            .is_some();
        if non_empty {
            managed(&root).map_err(|e| {
                format!(
                    "拒绝替换未知归属的非空目录 {}: {e}（目标未改动）",
                    root.display()
                )
            })?;
        }
    }
    let stage = staging_sibling(&root);
    fs::create_dir(&stage).map_err(|e| format!("创建 staging 失败 {}: {e}", stage.display()))?;
    fs::write(
        stage.join(STAGING_MARKER_FILE),
        format!(
            "{{\"schema\":\"{STAGING_MARKER_SCHEMA}\",\"pid\":{},\"kind\":\"{payload_kind}\"}}\n",
            std::process::id()
        ),
    )
    .map_err(|e| format!("写入 staging 标记失败: {e}"))?;

    let mut backup: Option<PathBuf> = None;
    let mut swapped = false;
    let inner = (|| -> AutoResult<()> {
        if fail_point == Some("prepare") {
            return Err("注入 prepare 失败".into());
        }
        prepare(&stage)?;
        if fail_point == Some("verify") {
            return Err("注入 verify 失败".into());
        }
        verify_payload(&stage)?;
        // The marker is staging metadata only — it must not ship inside the
        // committed payload. Remove it while the staging tree is still ours
        // and exclusive (before any root mutation).
        fs::remove_file(stage.join(STAGING_MARKER_FILE))
            .map_err(|e| format!("staging 标记无法移除: {e}"))?;
        if root.exists() {
            let target = unique_sibling(&root, "backup");
            fs::rename(&root, &target)
                .map_err(|e| format!("备份旧 payload 失败 {}: {e}", target.display()))?;
            backup = Some(target);
        }
        if fail_point == Some("swap") {
            return Err("注入 swap 失败".into());
        }
        fs::rename(&stage, &root)
            .map_err(|e| format!("提交 staging 失败 -> {}: {e}", root.display()))?;
        swapped = true;
        verify_payload(&root)?;
        Ok(())
    })();

    match inner {
        Ok(()) => Ok(CommitOutcome {
            root: root.clone(),
            backup_root: backup,
            preserved_foreign_staging,
        }),
        Err(error) => {
            let mut facts = vec![error.to_string()];
            if swapped && root.exists() {
                let failed = unique_sibling(&root, "failed");
                match fs::rename(&root, &failed) {
                    Ok(()) => facts.push(format!("失败载荷保留于 {}", failed.display())),
                    Err(e) => facts.push(format!(
                        "失败载荷无法移出 {}: {e}（root 仍为新载荷）",
                        root.display()
                    )),
                }
            }
            if let Some(target) = &backup {
                if !root.exists() {
                    match fs::rename(target, &root) {
                        Ok(()) => facts.push(format!("旧 payload 已还原 {}", root.display())),
                        Err(e) => facts.push(format!(
                            "旧 payload 保留于 {}，还原失败: {e}",
                            target.display()
                        )),
                    }
                } else {
                    facts.push(format!("旧 payload 保留于 {}", target.display()));
                }
            }
            if stage.exists() {
                match fs::remove_dir_all(&stage) {
                    Ok(()) => facts.push("staging 已清理".to_string()),
                    Err(e) => facts.push(format!("staging 残留于 {}: {e}", stage.display())),
                }
            }
            Err(format!("staged commit 失败: {}", facts.join("; ")).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn verify_payload(dir: &Path) -> AutoResult<()> {
        let doc = std::fs::read_to_string(dir.join("payload.txt"))
            .map_err(|e| format!("payload 缺失: {e}"))?;
        if doc.trim().is_empty() {
            return Err("payload 内容无效".into());
        }
        Ok(())
    }

    #[test]
    fn fresh_commit_swaps_staging_in_and_creates_no_backup() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("bundle");
        let outcome = commit_managed_directory(
            &root,
            "test",
            &|_| Err("不应调用 managed".into()),
            &|stage| {
                write(&stage.join("payload.txt"), "v1");
                Ok(())
            },
            &verify_payload,
            None,
        )
        .unwrap();
        assert!(outcome.backup_root.is_none());
        assert_eq!(
            std::fs::read_to_string(root.join("payload.txt")).unwrap(),
            "v1"
        );
        assert!(!root.join(STAGING_MARKER_FILE).exists());
    }

    #[test]
    fn prior_managed_payload_backed_up_and_replaced() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("bundle");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("payload.txt"), "old").unwrap();
        let old_map = file_map(&root, &[]).unwrap();
        let outcome = commit_managed_directory(
            &root,
            "test",
            &|dir| {
                if std::fs::read_to_string(dir.join("payload.txt")).unwrap() == "old" {
                    Ok(())
                } else {
                    Err("不是本 kind 的旧包".into())
                }
            },
            &|stage| {
                write(&stage.join("payload.txt"), "new");
                Ok(())
            },
            &verify_payload,
            None,
        )
        .unwrap();
        let backup = outcome.backup_root.expect("backup exists");
        assert_eq!(file_map(&backup, &[]).unwrap(), old_map);
        assert_eq!(
            std::fs::read_to_string(root.join("payload.txt")).unwrap(),
            "new"
        );
    }

    #[test]
    fn unknown_nonempty_root_refused_and_untouched() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("bundle");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("foreign.txt"), "user data").unwrap();
        let before = file_map(&root, &[]).unwrap();
        let error = commit_managed_directory(
            &root,
            "test",
            &|_| Err("未知归属".into()),
            &|_| Ok(()),
            &verify_payload,
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("拒绝替换未知归属"));
        assert_eq!(file_map(&root, &[]).unwrap(), before);
    }

    #[test]
    fn fail_points_keep_prior_payload_bytes_identical() {
        for point in ["prepare", "verify", "swap"] {
            let base = tempfile::tempdir().unwrap();
            let root = base.path().join("bundle");
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(root.join("payload.txt"), "old").unwrap();
            let before = file_map(&root, &[]).unwrap();
            let error = commit_managed_directory(
                &root,
                "test",
                &|_| Ok(()),
                &|stage| {
                    write(&stage.join("payload.txt"), "new");
                    Ok(())
                },
                &verify_payload,
                Some(point),
            )
            .unwrap_err();
            assert!(error.to_string().contains(point), "{point}: {error}");
            assert_eq!(file_map(&root, &[]).unwrap(), before, "point {point}");
            let leftovers: Vec<_> = std::fs::read_dir(base.path())
                .unwrap()
                .flatten()
                .filter(|entry| entry.file_name().to_string_lossy().starts_with(".bundle."))
                .collect();
            assert!(
                leftovers.is_empty(),
                "point {point}: staging cleaned, got {} leftovers",
                leftovers.len()
            );
        }
    }

    #[test]
    fn post_swap_verify_failure_restores_old_and_preserves_failed() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("bundle");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("payload.txt"), "old").unwrap();
        let before = file_map(&root, &[]).unwrap();
        // The staged payload verifies fine, the committed root is corrupted
        // before the post-swap verify via a fail-time check hook: simulate by
        // verifying "new" content everywhere — pass a verifier that accepts
        // only "new", then corrupt the root inside prepare is impossible, so
        // use verify that fails on second call.
        let calls = std::cell::Cell::new(0u32);
        let error = commit_managed_directory(
            &root,
            "test",
            &|_| Ok(()),
            &|stage| {
                write(&stage.join("payload.txt"), "new");
                Ok(())
            },
            &|dir| {
                calls.set(calls.get() + 1);
                if calls.get() == 1 {
                    verify_payload(dir)
                } else {
                    Err("事后校验注入失败".into())
                }
            },
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("旧 payload 已还原"));
        assert!(error.to_string().contains("失败载荷保留于"));
        assert_eq!(file_map(&root, &[]).unwrap(), before);
        assert!(std::fs::read_dir(base.path())
            .unwrap()
            .flatten()
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with(".bundle.auto-failed-")));
    }

    #[test]
    fn sweep_removes_own_staging_and_preserves_foreign() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("bundle");
        let own = root.with_file_name(format!(".bundle.auto-staging-{}-1", std::process::id()));
        let foreign = root.with_file_name(".bundle.auto-staging-111111-2");
        for dir in [&own, &foreign] {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(
                dir.join(STAGING_MARKER_FILE),
                format!(
                    "{{\"schema\":\"{STAGING_MARKER_SCHEMA}\",\"pid\":{},\"kind\":\"test\"}}\n",
                    if dir == &own {
                        std::process::id()
                    } else {
                        111111
                    }
                ),
            )
            .unwrap();
        }
        let preserved = sweep_own_staging(&root);
        assert!(!own.exists(), "own staging swept");
        assert_eq!(preserved, vec![foreign.clone()]);
        assert!(foreign.exists(), "foreign staging preserved");
    }

    #[test]
    fn copy_tree_clean_skips_junk_and_is_hashable() {
        let base = tempfile::tempdir().unwrap();
        let source = base.path().join("crate");
        write(&source.join("Cargo.toml"), "[package]\n");
        write(&source.join("src/lib.rs"), "pub fn hi() {}\n");
        write(&source.join("target/debug/junk.bin"), "x");
        write(&source.join(".git/config"), "x");
        let destination = base.path().join("copy");
        let copied = copy_tree_clean(&source, &destination).unwrap();
        assert_eq!(copied, 2);
        let map = file_map(&destination, &[]).unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("Cargo.toml"));
        assert!(map.contains_key("src/lib.rs"));
    }

    #[test]
    #[cfg(windows)]
    fn copy_tree_clean_rejects_symlink() {
        let base = tempfile::tempdir().unwrap();
        let source = base.path().join("crate");
        write(&source.join("real.txt"), "x");
        let link = source.join("link.txt");
        if std::os::windows::fs::symlink_file(source.join("real.txt"), &link).is_err() {
            eprintln!("symlink 创建需开发者模式，跳过 reparse 断言");
            return;
        }
        let error = copy_tree_clean(&source, &base.path().join("copy")).unwrap_err();
        assert!(error.to_string().contains("symlink"));
    }
}
