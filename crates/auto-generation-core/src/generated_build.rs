//! Generated-cargo build context, member identity, receipts, leases and
//! artifact publication (PLAN-910). Single source shared by UI `rust_ui`,
//! Web `api_gen` and Man `serve_rust`; production generation algorithms stay
//! with their owners.
//!
//! Three objects (Design 53 / Plan 910 §2):
//! - the generated source workspace (`<ws>`, per project or shared in-framework);
//! - the member identity inside `<ws>` (basename collisions are resolved by a
//!   content-stable project path id, never by mtime or claim order);
//! - one managed Cargo cache root per producer context:
//!   `<producer_root>/target/generated-consumers/<ctx8>/` (development mode).
//!
//! Flags are pinned through env (`CARGO_TARGET_DIR`, `CARGO_ENCODED_RUSTFLAGS`)
//! on every managed spawn so effective config no longer depends on the cwd a
//! caller happens to sit in (905 root cause). User-set RUSTFLAGS /
//! CARGO_ENCODED_RUSTFLAGS / CARGO_TARGET_DIR are respected and name-isolated
//! into the context id instead of being silently overwritten.
//!
//! This module never parses Cargo semantics beyond the pinned keys; the
//! effective identity it records (context.json + receipts) is verifiable
//! evidence, not a reimplementation of cargo.

use crate::rust_workspace::WorkspaceRequirements;
use crate::source_owner::is_managed_sdk;
use auto_val::AutoResult;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub const CONTEXT_SCHEMA: &str = "generated-build-context/v1";
pub const MEMBER_SCHEMA: &str = "generated-member/v1";
pub const FRONT_RECEIPT_SCHEMA: &str = "generated-front-receipt/v1";
/// Subdirectory of the producer's `target/` holding all generated-consumer
/// cache roots (never mixed with tool `target/debug|release` units).
pub const GENERATED_CONSUMERS_DIR: &str = "generated-consumers";
/// Explicit override for the managed cache root (absolute path; owner-checked).
pub const TARGET_ROOT_ENV: &str = "AUTO_GENERATED_TARGET_ROOT";
pub const CONTEXT_FILE: &str = "context.json";
pub const MEMBER_FILE: &str = "member.json";
pub const FRONT_RECEIPT_FILE: &str = "generation.json";
pub const LEASE_DIR: &str = ".lease";
/// MSVC deep-view stack reserve kept for generated Windows UI apps (PLAN-039
/// T-04; the language repo's own 32MiB applies to tooling only).
const MANAGED_MSVC_STACK_FLAG: &str = "-C\u{1f}link-arg=/STACK:268435456";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberRole {
    Front,
    Back,
}

impl MemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            MemberRole::Front => "front",
            MemberRole::Back => "back",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextMode {
    /// Producer is a source checkout (or any non-SDK root): the cache lives
    /// under that tree at `target/generated-consumers/<ctx8>/`.
    Development { producer_root: PathBuf },
    /// Producer is a read-only installed SDK: never write into the producer;
    /// isolate at the project's own workspace `<ws>/target`.
    InstalledSdk { install_root: PathBuf },
    /// Explicit user override (AUTO_GENERATED_TARGET_ROOT / CARGO_TARGET_DIR):
    /// validated absolute root with the owner recorded.
    Explicit { root: PathBuf, owner: String },
}

/// A resolved, reproducible generated-cargo build context.
#[derive(Clone, Debug)]
pub struct GeneratedBuildContext {
    pub mode: ContextMode,
    /// Absolute root of the tree owning `crates/auto-lang` (dev) or the SDK
    /// install (sdk). Explicit mode still records the discovered producer.
    pub producer_root: PathBuf,
    /// Managed Cargo cache root shared by compatible consumers.
    pub target_root: PathBuf,
    /// 8-hex context identity (schema + producer + flags policy + overrides).
    pub context_id: String,
    /// rustflags value we pin, or None when a user override is respected.
    pub managed_rustflags: Option<String>,
    /// Name of the user env var whose flags we did not override.
    pub user_flags_override: Option<String>,
    /// `rustc -vV` summary (host/release/commit-hash), recorded not keyed.
    pub toolchain: String,
    /// How the producer was selected (env var name, "安装 SDK", discovery
    /// arm, "编译期语言源"): audit record, not part of the context key.
    pub producer_origin: String,
}

fn sha256_hex(parts: &[&str]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

/// Canonical comparison key for a path: resolved when possible, `\\?\`
/// stripped, lowercased, forward slashes — NTFS is case-insensitive and this
/// key is only used for identity/hashing, never for display.
pub fn canonical_key(path: &Path) -> String {
    let abs = fs::canonicalize(path).unwrap_or_else(|_| {
        let mut p = path.to_path_buf();
        if p.is_relative() {
            p = std::env::current_dir().unwrap_or_default().join(p);
        }
        p
    });
    let text = abs
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .trim_start_matches(r"\\?\UNC\")
        .replace('\\', "/");
    let mut text = text.replace("//?/", "");
    if text.starts_with("UNC/") {
        text = format!("//{}", &text[4..]);
    }
    text.trim_end_matches('/').to_lowercase()
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn toolchain_summary() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let Ok(output) = std::process::Command::new("rustc").arg("-vV").output() else {
                return "rustc-unavailable".to_string();
            };
            let text = String::from_utf8_lossy(&output.stdout);
            let mut host = "";
            let mut release = "";
            let mut commit = "";
            for line in text.lines() {
                if let Some(v) = line.strip_prefix("host: ") {
                    host = v;
                } else if let Some(v) = line.strip_prefix("release: ") {
                    release = v;
                } else if let Some(v) = line.strip_prefix("commit-hash: ") {
                    commit = v;
                }
            }
            format!("host={host};release={release};commit={commit}")
        })
        .clone()
}

fn user_flags_override() -> Option<(&'static str, String)> {
    for name in ["CARGO_ENCODED_RUSTFLAGS", "RUSTFLAGS"] {
        if let Some(value) = std::env::var_os(name) {
            let value = value.to_string_lossy().to_string();
            if !value.trim().is_empty() {
                return Some((name, value));
            }
        }
    }
    None
}

/// Resolve the producer root exactly the way workspace dependency anchors do
/// (same order as `compute_auto_lang_rel_path_for`): explicit envs, adjacent
/// installed SDK, project walk-up/sibling discovery, compiled-in fallback.
fn resolve_producer_root(
    project_dir: &Path,
    requirements: WorkspaceRequirements,
) -> AutoResult<(PathBuf, String)> {
    let selection = crate::rust_workspace::select_language_producer(project_dir, requirements)?;
    Ok((selection.producer_root(), selection.origin))
}

/// A checkout root marker: any ancestor carrying `.git` owns this tree.
fn owning_checkout(path: &Path) -> Option<PathBuf> {
    let mut dir = Some(path);
    while let Some(current) = dir {
        if current.join(".git").exists() {
            return Some(current.to_path_buf());
        }
        dir = current.parent();
    }
    None
}

fn strip_verbatim(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy().to_string();
    PathBuf::from(text.trim_start_matches(r"\\?\").to_string())
}

fn resolve_absolute(path: &Path) -> PathBuf {
    if let Ok(resolved) = fs::canonicalize(path) {
        return strip_verbatim(resolved);
    }
    // The tail may not exist yet (a cache root created on first use):
    // canonicalize the deepest existing ancestor and re-attach the tail so
    // the result is still the real absolute on-disk spelling.
    let mut tail = Vec::new();
    let mut current = path.to_path_buf();
    while let Some(name) = current.file_name() {
        tail.push(name.to_os_string());
        current.pop();
        if current.as_os_str().is_empty() {
            break;
        }
        if let Ok(resolved) = fs::canonicalize(&current) {
            let mut full = strip_verbatim(resolved);
            for part in tail.iter().rev() {
                full.push(part);
            }
            return full;
        }
    }
    let mut abs = path.to_path_buf();
    if abs.is_relative() {
        abs = std::env::current_dir().unwrap_or_default().join(abs);
    }
    abs
}

fn validate_explicit_root(
    root: &Path,
    producer_root: &Path,
    project_dir: &Path,
    owner: &str,
) -> AutoResult<PathBuf> {
    let mut abs = root.to_path_buf();
    if abs.is_relative() {
        abs = std::env::current_dir().unwrap_or_default().join(&abs);
    }
    let abs = resolve_absolute(&abs);
    if !abs.exists() {
        return Err(format!(
            "显式生成缓存根 {owner}={} 不存在（须先创建后声明）",
            abs.display()
        )
        .into());
    }
    let key = canonical_key(&abs);
    let producer_key = canonical_key(producer_root);
    // Never the producer's tool target itself (nor a sibling tool profile dir).
    let producer_target = format!("{producer_key}/target");
    if key == producer_target
        || key == format!("{producer_key}/target/debug")
        || key == format!("{producer_key}/target/release")
    {
        return Err(format!(
            "显式生成缓存根 {owner}={} 指向 producer 工具 target（须用 {}/ 下的受管子根）",
            abs.display(),
            producer_target
        )
        .into());
    }
    if key != producer_key
        && key.starts_with(&format!("{producer_key}/target/{GENERATED_CONSUMERS_DIR}"))
    {
        // A context root explicitly pinned under the producer is fine.
        return Ok(abs);
    }
    // Refuse roots inside another checkout/worktree that is not the producer
    // or the project's own tree.
    if let Some(owner_tree) = owning_checkout(&abs) {
        let owner_key = canonical_key(&owner_tree);
        if owner_key != producer_key {
            let project_owner = owning_checkout(project_dir).map(|p| canonical_key(&p));
            if project_owner.as_deref() != Some(owner_key.as_str()) {
                return Err(format!(
                    "显式生成缓存根 {owner}={} 位于另一检出 {} 内（拒绝借用他树可写 target）",
                    abs.display(),
                    owner_tree.display()
                )
                .into());
            }
        }
    }
    Ok(abs)
}

impl GeneratedBuildContext {
    /// Resolve the single managed context for a project's generated cargo
    /// consumers. Deterministic for (project env, producer, flags policy).
    pub fn resolve(
        project_dir: &Path,
        ws_dir: &Path,
        requirements: WorkspaceRequirements,
    ) -> AutoResult<Self> {
        let (producer_root, origin) = resolve_producer_root(project_dir, requirements)?;
        let user_flags = user_flags_override();
        let managed_rustflags = if cfg!(all(windows, target_env = "msvc")) && user_flags.is_none() {
            Some(MANAGED_MSVC_STACK_FLAG.to_string())
        } else {
            None
        };
        let sdk = is_managed_sdk(&producer_root) || origin == "安装 SDK";
        let flags_key = match (&user_flags, &managed_rustflags) {
            (Some((name, value)), _) => format!("user:{name}:{}", sha256_hex(&[value])),
            (None, Some(flags)) => format!("managed:{}", sha256_hex(&[flags])),
            (None, None) => "managed:none".to_string(),
        };
        let producer_key = canonical_key(&producer_root);
        let context_id = sha256_hex(&[CONTEXT_SCHEMA, &producer_key, &flags_key])[..8].to_string();
        let mode = if let Some(raw) = std::env::var_os(TARGET_ROOT_ENV) {
            let root = validate_explicit_root(
                Path::new(&raw),
                &producer_root,
                project_dir,
                TARGET_ROOT_ENV,
            )?;
            ContextMode::Explicit {
                root,
                owner: TARGET_ROOT_ENV.to_string(),
            }
        } else if let Some(raw) = std::env::var_os("CARGO_TARGET_DIR") {
            let text = raw.to_string_lossy().to_string();
            if !text.trim().is_empty() {
                let root = validate_explicit_root(
                    Path::new(&raw),
                    &producer_root,
                    project_dir,
                    "CARGO_TARGET_DIR",
                )?;
                ContextMode::Explicit {
                    root,
                    owner: "CARGO_TARGET_DIR".to_string(),
                }
            } else {
                default_mode(&producer_root, ws_dir, &context_id, sdk)
            }
        } else {
            default_mode(&producer_root, ws_dir, &context_id, sdk)
        };
        let target_root = match &mode {
            ContextMode::Development { producer_root } => producer_root
                .join("target")
                .join(GENERATED_CONSUMERS_DIR)
                .join(&context_id),
            ContextMode::InstalledSdk { install_root } => {
                let _ = install_root;
                ws_dir.join("target")
            }
            ContextMode::Explicit { root, .. } => root.clone(),
        };
        Ok(GeneratedBuildContext {
            mode,
            producer_root,
            target_root,
            context_id,
            managed_rustflags,
            user_flags_override: user_flags.map(|(name, _)| name.to_string()),
            toolchain: toolchain_summary(),
            producer_origin: origin,
        })
    }

    /// Env pins every managed cargo spawn must carry (target root + managed
    /// rustflags). A user flag override is respected, never overwritten.
    pub fn cargo_env(&self) -> Vec<(String, String)> {
        let mut env = vec![(
            "CARGO_TARGET_DIR".to_string(),
            self.target_root.to_string_lossy().to_string(),
        )];
        if let Some(flags) = &self.managed_rustflags {
            env.push(("CARGO_ENCODED_RUSTFLAGS".to_string(), flags.clone()));
        }
        env
    }

    pub fn profile_dir(&self, release: bool) -> PathBuf {
        self.target_root
            .join(if release { "release" } else { "debug" })
    }

    /// Persist the verifiable context identity (write-on-change). Recorded,
    /// never a gate: the ctx id itself is computed, this file proves what
    /// produced the root for cleanup and audits.
    pub fn write_context_file(&self) -> io::Result<()> {
        let path = self.target_root.join(CONTEXT_FILE);
        fs::create_dir_all(&self.target_root)?;
        let record = serde_json::json!({
            "schema": CONTEXT_SCHEMA,
            "context_id": self.context_id,
            "mode": match &self.mode {
                ContextMode::Development { .. } => "development",
                ContextMode::InstalledSdk { .. } => "installed-sdk",
                ContextMode::Explicit { owner, .. } => owner,
            },
            "producer_root": self.producer_root.to_string_lossy(),
            "target_root": self.target_root.to_string_lossy(),
            "rustflags": self.managed_rustflags,
            "user_flags_override": self.user_flags_override,
            "toolchain": self.toolchain,
            "producer_origin": self.producer_origin,
        });
        write_if_changed(&path, format!("{record}\n").as_bytes()).map(|_| ())
    }
}

fn default_mode(producer_root: &Path, ws_dir: &Path, context_id: &str, sdk: bool) -> ContextMode {
    if sdk {
        ContextMode::InstalledSdk {
            install_root: producer_root.to_path_buf(),
        }
    } else {
        // Development: the producer tree owns the cache. (ws_dir/context_id
        // are unused here but kept in the signature for symmetry.)
        let _ = (ws_dir, context_id);
        ContextMode::Development {
            producer_root: producer_root.to_path_buf(),
        }
    }
}

/// Write `bytes` to `path` only when the current content differs. Returns
/// whether the file was (re)written. PLAN-910: unchanged inputs must not
/// rewrite manifests/configs/receipts (mtime stability is observable).
pub fn write_if_changed(path: &Path, bytes: &[u8]) -> io::Result<bool> {
    if let Ok(existing) = fs::read(path) {
        if existing == bytes {
            return Ok(false);
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp-p910");
    fs::write(&tmp, bytes)?;
    // Best-effort mtime touch so fresh-vs-written distinction stays visible
    // even when content round-trips through external editors.
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(true),
        Err(_) => {
            fs::copy(&tmp, path)?;
            let _ = fs::remove_file(&tmp);
            Ok(true)
        }
    }
}

/// Content-stable member id for a project path (first 8 hex of sha256 over
/// the member schema + canonical path). Independent of mtime and claim order.
pub fn member_id8(project_dir: &Path) -> String {
    sha256_hex(&[MEMBER_SCHEMA, &canonical_key(project_dir)])[..8].to_string()
}

/// Sanitize a directory basename into a Cargo-package-safe fragment.
fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "myapp".to_string()
    } else {
        trimmed
    }
}

/// Member directory name: `<basename>-<id8>` (front) / `<basename>-<id8>-back`
/// (back; `-back` suffix preserved for existing tooling). Collision-free for
/// same-basename projects at different paths and for front/back roles.
pub fn member_dir_name(project_dir: &Path, role: MemberRole) -> String {
    let base = sanitize(
        project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("myapp"),
    );
    let id = member_id8(project_dir);
    match role {
        MemberRole::Front => format!("{base}-{id}"),
        MemberRole::Back => format!("{base}-{id}-back"),
    }
}

/// Cargo package name for a member: `<display>-<id8>` where display is the
/// pac name (front, snake-cased by the caller) or the member dir (back),
/// keeping the historical digit-leading `app-` prefix rule.
pub fn package_name(display: &str, project_dir: &Path, _role: MemberRole) -> String {
    let id = member_id8(project_dir);
    let mut name = format!("{}-{}", sanitize(display), id);
    if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        name = format!("app-{name}");
    }
    name
}

/// The per-member ownership record. Secondary consistency check: a member dir
/// belongs to exactly one (canonical project path, role) pair; a mismatch is
/// a hard refusal, never a silent reclaim.
#[derive(Debug, Clone)]
pub struct MemberRecord {
    pub schema: String,
    pub role: MemberRole,
    pub project_path: String,
    pub created_at_ms: u128,
}

impl MemberRecord {
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "schema": self.schema,
            "role": self.role.as_str(),
            "project_path": self.project_path,
            "created_at_ms": self.created_at_ms as u64,
        })
        .to_string()
    }

    fn from_text(text: &str, project_dir: &Path, role: MemberRole) -> AutoResult<Self> {
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("{MEMBER_FILE} 解析失败: {e}"))?;
        if value.get("schema").and_then(|s| s.as_str()) != Some(MEMBER_SCHEMA) {
            return Err(format!("{MEMBER_FILE} schema 不符（非本布局成员）").into());
        }
        let recorded_role = value.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let recorded_path = value
            .get("project_path")
            .and_then(|p| p.as_str())
            .unwrap_or("");
        if recorded_role != role.as_str() || recorded_path != canonical_key(project_dir) {
            return Err(format!(
                "成员目录已被其他工程/角色占用: 记录={recorded_path}#{recorded_role}, \
                 请求={}#{}（旧 basename 成员不认领，请检查来源路径）",
                canonical_key(project_dir),
                role.as_str()
            )
            .into());
        }
        Ok(MemberRecord {
            schema: MEMBER_SCHEMA.to_string(),
            role,
            project_path: recorded_path.to_string(),
            created_at_ms: value
                .get("created_at_ms")
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as u128,
        })
    }
}

/// Compute (and verify ownership of) the member directory for a project.
/// Creates the directory with its `member.json` on first use; an existing dir
/// with a different owner is a named refusal.
pub fn claim_member_dir(
    ws_dir: &Path,
    project_dir: &Path,
    role: MemberRole,
) -> AutoResult<PathBuf> {
    let dir = ws_dir.join(member_dir_name(project_dir, role));
    let record_path = dir.join(MEMBER_FILE);
    if record_path.exists() {
        let text = fs::read_to_string(&record_path)
            .map_err(|e| format!("读取 {} 失败: {e}", record_path.display()))?;
        MemberRecord::from_text(&text, project_dir, role)?;
    } else if dir.exists() {
        // Pre-existing directory without a member record: a legacy basename
        // member. It is never claimed, overwritten or deleted by us.
        if dir.join("Cargo.toml").exists()
            || fs::read_dir(&dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(true)
        {
            return Err(format!(
                "成员目录 {} 已存在且无 {} 归属记录（旧布局成员不认领/不覆写；请人工核属后迁移）",
                dir.display(),
                MEMBER_FILE
            )
            .into());
        }
    }
    fs::create_dir_all(&dir)?;
    // Keep the original creation time on re-claim so the record bytes (and
    // therefore its mtime) stay stable for unchanged ownership.
    let existing_created = fs::read_to_string(&record_path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|value| value.get("created_at_ms").and_then(|t| t.as_u64()))
        .unwrap_or(0) as u128;
    let record = MemberRecord {
        schema: MEMBER_SCHEMA.to_string(),
        role,
        project_path: canonical_key(project_dir),
        created_at_ms: if existing_created > 0 {
            existing_created
        } else {
            now_millis()
        },
    };
    write_if_changed(&record_path, format!("{}\n", record.to_json()).as_bytes())?;
    Ok(dir)
}

/// Read the recorded owner of a member dir without claiming it.
pub fn member_record(ws_dir: &Path, project_dir: &Path, role: MemberRole) -> Option<MemberRecord> {
    let text = fs::read_to_string(
        ws_dir
            .join(member_dir_name(project_dir, role))
            .join(MEMBER_FILE),
    )
    .ok()?;
    MemberRecord::from_text(&text, project_dir, role).ok()
}

// ---------------------------------------------------------------------------
// Workspace generation lock (reentrant per thread, exclusive across processes)
// ---------------------------------------------------------------------------

thread_local! {
    static LOCK_DEPTH: std::cell::RefCell<BTreeMap<String, usize>> =
        const { std::cell::RefCell::new(BTreeMap::new()) };
}

/// RAII workspace generation lock: an exclusive OS file lock on
/// `<ws>/.generation.lock`, reentrant within the owning thread (generation
/// entry points nest ensure/generate/receipt calls), cross-process safe.
pub struct WorkspaceGenerationGuard {
    key: String,
    file: Option<fs::File>,
}

impl WorkspaceGenerationGuard {
    pub fn acquire(ws_dir: &Path) -> io::Result<Self> {
        let key = canonical_key(ws_dir);
        let nested = LOCK_DEPTH.with(|m| *m.borrow().get(&key).unwrap_or(&0) > 0);
        let file = if nested {
            None
        } else {
            fs::create_dir_all(ws_dir)?;
            let file = fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(ws_dir.join(".generation.lock"))?;
            file.lock()?;
            Some(file)
        };
        LOCK_DEPTH.with(|m| {
            let mut m = m.borrow_mut();
            *m.entry(key.clone()).or_insert(0) += 1;
        });
        Ok(WorkspaceGenerationGuard { key, file })
    }
}

impl Drop for WorkspaceGenerationGuard {
    fn drop(&mut self) {
        LOCK_DEPTH.with(|m| {
            let mut m = m.borrow_mut();
            if let Some(depth) = m.get_mut(&self.key) {
                *depth = depth.saturating_sub(1);
            }
        });
        if let Some(file) = self.file.take() {
            let _ = file.unlock();
        }
    }
}

// ---------------------------------------------------------------------------
// Target-root leases (build/run liveness for cleanup refusal)
// ---------------------------------------------------------------------------

/// A live lease file under `<target_root>/.lease/`. Cleanup refuses while any
/// lease whose pid is still alive exists; stale leases are reported, ignored.
pub struct TargetLease {
    path: PathBuf,
}

impl TargetLease {
    pub fn acquire(target_root: &Path, purpose: &str) -> io::Result<Self> {
        let dir = target_root.join(LEASE_DIR);
        fs::create_dir_all(&dir)?;
        // Sweep leases left by force-killed holders (dead pid = no holder);
        // live leases are never touched here.
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let dead = fs::read_to_string(&path)
                    .ok()
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
                    .and_then(|value| value.get("pid").and_then(|p| p.as_u64()))
                    .is_some_and(|pid| !process_alive(pid as u32));
                if dead {
                    let _ = fs::remove_file(&path);
                }
            }
        }
        let path = dir.join(format!("{}-{}.json", std::process::id(), now_millis()));
        let record = serde_json::json!({
            "pid": std::process::id(),
            "purpose": purpose,
            "started_at_ms": now_millis() as u64,
            "cwd": std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
        });
        fs::write(&path, format!("{record}\n"))?;
        Ok(TargetLease { path })
    }
}

impl Drop for TargetLease {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Keep a lease alive for the rest of this process (a spawned backend that
/// runs from the target root outlives the function that started it). Dropped
/// on normal exit; a killed process leaves a dead-pid lease the cleaner
/// reports as stale and ignores.
pub fn hold_lease_for_process(lease: TargetLease) {
    static HELD: OnceLock<std::sync::Mutex<Vec<TargetLease>>> = OnceLock::new();
    if let Ok(mut held) = HELD
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
    {
        held.push(lease);
    }
}

/// Live (pid still running) lease count under a target root — advisory only;
/// the authoritative cleanup gate lives in the cleaner.
pub fn live_leases(target_root: &Path) -> Vec<(PathBuf, u32)> {
    let dir = target_root.join(LEASE_DIR);
    let mut live = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(text) = fs::read_to_string(&path) {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(pid) = value.get("pid").and_then(|p| p.as_u64()) {
                            if process_alive(pid as u32) {
                                live.push((path, pid as u32));
                            }
                        }
                    }
                }
            }
        }
    }
    live
}

pub fn process_alive(pid: u32) -> bool {
    // Windows: OpenProcess existence probe without external crates.
    #[cfg(windows)]
    {
        const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
        let handle = unsafe { windows_open_process(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle == 0 {
            // ERROR_ACCESS_DENIED still proves existence.
            return unsafe { GetLastError() } == 5;
        }
        struct Handle(usize);
        impl Drop for Handle {
            fn drop(&mut self) {
                unsafe {
                    windows_close_handle(self.0);
                }
            }
        }
        let _guard = Handle(handle);
        let mut code = 0u32;
        let ok = unsafe { windows_exit_code(handle, &mut code) } != 0;
        let still_active = 259u32; // STILL_ACTIVE
        !ok || code == still_active
    }
    #[cfg(not(windows))]
    {
        Path::new(&format!("/proc/{pid}")).exists()
    }
}

#[cfg(windows)]
extern "system" {
    fn GetLastError() -> u32;
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> usize;
    fn CloseHandle(handle: usize) -> i32;
    fn GetExitCodeProcess(handle: usize, code: *mut u32) -> i32;
}

#[cfg(windows)]
use CloseHandle as windows_close_handle;
#[cfg(windows)]
use GetExitCodeProcess as windows_exit_code;
#[cfg(windows)]
use OpenProcess as windows_open_process;

// ---------------------------------------------------------------------------
// Artifact publication (collision-free discovery location)
// ---------------------------------------------------------------------------

/// Publish the uplifted cargo artifact (exe + optional pdb) into the member's
/// own `bin/<profile>/` directory. The exe is COPIED (a hard link would share
/// the inode with cargo's deps/uplift files, and a running published exe
/// would then lock cargo out of rebuilding — observed on Windows as
/// "failed to remove file ... os error 5"); the pdb is hard-linked (never
/// mapped by the running process). The published path is stable per member,
/// immune to same-basename uplift races in the shared target root, and is
/// what discovery receipts point at. A running published exe is renamed
/// away before the new copy lands (Windows allows renaming a mapped image).
pub fn publish_artifact(
    uplifted: &Path,
    member_dir: &Path,
    release: bool,
    published_file_name: &str,
) -> io::Result<PathBuf> {
    let profile = if release { "release" } else { "debug" };
    let dest_dir = member_dir.join("bin").join(profile);
    fs::create_dir_all(&dest_dir)?;
    // The cargo bin target carries the member id (Windows/MSVC cargo links
    // bins WITHOUT a metadata hash — `deps/<bin>.exe` and the uplifted
    // `<profile>/<bin>.exe` are shared by every same-named bin in a target
    // root, so uniqueness must live in the bin name); the user-visible exe
    // name is restored here, at publication.
    let dest = dest_dir.join(published_file_name);
    replace_durable(uplifted, &dest, true)?;
    let pdb = uplifted.with_extension("pdb");
    if pdb.is_file() {
        let pdb_dest = Path::new(published_file_name)
            .with_extension("pdb")
            .file_name()
            .map(|n| dest_dir.join(n))
            .unwrap_or_else(|| dest_dir.join("published.pdb"));
        if !same_file(&pdb, &pdb_dest) {
            replace_durable(&pdb, &pdb_dest, false)?;
        }
    }
    Ok(dest)
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => false,
    }
}

fn replace_durable(src: &Path, dest: &Path, copy: bool) -> io::Result<()> {
    if dest.exists() {
        // Windows: a running exe cannot be deleted but can be renamed away.
        let old = dest.with_extension(format!("old-{}", now_millis()));
        match fs::rename(dest, &old) {
            Ok(()) => {
                // Best effort: a still-running old image stays as *.old-* and
                // is swept on the next successful publication.
                let _ = fs::remove_file(&old);
            }
            Err(_) => {
                let _ = fs::remove_file(dest);
            }
        }
    }
    if copy {
        fs::copy(src, dest)?;
    } else if fs::hard_link(src, dest).is_err() {
        fs::copy(src, dest)?;
    }
    // Sweep stale renamed images from earlier publications (ignore locked).
    if let Some(parent) = dest.parent() {
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains(".old-") {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Front-member generation receipt (content identity + artifact)
// ---------------------------------------------------------------------------

/// Content-identity receipt for the generated front (UI) member — the front
/// counterpart of the Web back member's generation.json. Freshness is judged
/// against declared input contents, never mtime.
#[derive(Debug, Clone, Default)]
pub struct FrontReceipt {
    /// rel path (from project dir, '/'-separated) -> sha256 hex of content.
    pub inputs: BTreeMap<String, String>,
    /// Identity of the generator (crate version + key config).
    pub generator: String,
    /// workspace lock identity (absent / empty / hash — three states kept).
    pub workspace_lock: Option<String>,
    /// "absent" was materialized into a real lock id by a later check.
    pub lock_materialized: bool,
    pub context_id: String,
    pub target_root: String,
    /// published exe absolute path.
    pub exe: Option<String>,
    pub exe_sha256: Option<String>,
    /// user-visible published exe stem (pac exe_name else snake pac name).
    pub bin_name: Option<String>,
    /// cargo `[[bin]]` target name (`<bin_name>-<id8>`, unique in the root).
    pub cargo_bin: Option<String>,
    pub release: bool,
}

impl FrontReceipt {
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "schema": FRONT_RECEIPT_SCHEMA,
            "inputs": self.inputs.iter().map(|(k, v)| serde_json::json!([k, v])).collect::<Vec<_>>(),
            "generator": self.generator,
            "workspace_lock": self.workspace_lock,
            "context_id": self.context_id,
            "target_root": self.target_root,
            "artifact": {
                "exe": self.exe,
                "exe_sha256": self.exe_sha256,
                "bin_name": self.bin_name,
                "cargo_bin": self.cargo_bin,
                "profile": if self.release { "release" } else { "debug" },
            },
        })
        .to_string()
    }

    pub fn read(member_dir: &Path) -> Option<FrontReceipt> {
        let text = fs::read_to_string(member_dir.join(FRONT_RECEIPT_FILE)).ok()?;
        let value: serde_json::Value = serde_json::from_str(&text).ok()?;
        if value.get("schema").and_then(|s| s.as_str()) != Some(FRONT_RECEIPT_SCHEMA) {
            return None;
        }
        let mut inputs = BTreeMap::new();
        if let Some(list) = value.get("inputs").and_then(|i| i.as_array()) {
            for entry in list {
                if let (Some(k), Some(v)) = (
                    entry.get(0).and_then(|k| k.as_str()),
                    entry.get(1).and_then(|v| v.as_str()),
                ) {
                    inputs.insert(k.to_string(), v.to_string());
                }
            }
        }
        let artifact = value.get("artifact").cloned().unwrap_or_default();
        Some(FrontReceipt {
            inputs,
            generator: value
                .get("generator")
                .and_then(|g| g.as_str())
                .unwrap_or("")
                .to_string(),
            workspace_lock: value
                .get("workspace_lock")
                .and_then(|w| w.as_str())
                .map(str::to_string),
            lock_materialized: false,
            context_id: value
                .get("context_id")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
            target_root: value
                .get("target_root")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string(),
            exe: artifact
                .get("exe")
                .and_then(|e| e.as_str())
                .map(str::to_string),
            exe_sha256: artifact
                .get("exe_sha256")
                .and_then(|e| e.as_str())
                .map(str::to_string),
            bin_name: artifact
                .get("bin_name")
                .and_then(|e| e.as_str())
                .map(str::to_string),
            cargo_bin: artifact
                .get("cargo_bin")
                .and_then(|e| e.as_str())
                .map(str::to_string),
            release: artifact.get("profile").and_then(|p| p.as_str()) == Some("release"),
        })
    }

    pub fn write(&self, member_dir: &Path) -> io::Result<bool> {
        write_if_changed(
            &member_dir.join(FRONT_RECEIPT_FILE),
            format!("{}\n", self.to_json()).as_bytes(),
        )
    }

    /// Content freshness: the recorded input set equals the current computed
    /// set (same files, same hashes). Missing inputs / read errors surface as
    /// a differing current set, never as "no change".
    pub fn inputs_match(&self, _project_dir: &Path, current: &BTreeMap<String, String>) -> bool {
        !self.inputs.is_empty() && &self.inputs == current
    }
}

/// Discover the published front exe for a project (receipt-first; the caller
/// may keep convention probing as a legacy fallback). Returns None when no
/// fresh receipt or the published exe is gone.
pub fn discover_front_exe(project_dir: &Path) -> Option<PathBuf> {
    let ws_dir = crate::rust_workspace::resolve_rust_workspace_dir(project_dir);
    let member = ws_dir.join(member_dir_name(project_dir, MemberRole::Front));
    let receipt = FrontReceipt::read(&member)?;
    let exe = PathBuf::from(receipt.exe?);
    exe.is_file().then_some(exe)
}

pub fn file_sha256(path: &Path) -> io::Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

#[cfg(test)]
mod plan910_tests {
    use super::*;

    /// Env-mutating context tests serialize on this mutex: nextest runs test
    /// threads in one process and the process env is shared state.
    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn tmp() -> PathBuf {
        tempfile::tempdir().unwrap().keep()
    }

    #[test]
    fn member_dir_names_are_role_and_path_unique() {
        let a = tmp().join("projA").join("002-counter");
        let b = tmp().join("projB").join("002-counter");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        let a_front = member_dir_name(&a, MemberRole::Front);
        let a_back = member_dir_name(&a, MemberRole::Back);
        let b_front = member_dir_name(&b, MemberRole::Front);
        let b_back = member_dir_name(&b, MemberRole::Back);
        assert!(a_front.ends_with(&format!("-{}", member_id8(&a))));
        assert!(a_back.starts_with("002-counter-") && a_back.ends_with("-back"));
        assert_ne!(a_front, b_front);
        assert_ne!(a_back, b_back);
        assert_ne!(a_front, a_back);
        // A project whose basename ends with -back cannot collide with a back member.
        let tricky = tmp().join("x-back");
        fs::create_dir_all(&tricky).unwrap();
        let t_front = member_dir_name(&tricky, MemberRole::Front);
        let plain = tmp().join("x");
        fs::create_dir_all(&plain).unwrap();
        let p_back = member_dir_name(&plain, MemberRole::Back);
        assert_ne!(t_front, p_back);
    }

    #[test]
    fn claim_member_dir_refuses_foreign_owner_and_legacy_dirs() {
        let base = tmp();
        let ws = base.join("ws");
        let a = base.join("projA").join("app");
        let b = base.join("projB").join("app");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        let claimed = claim_member_dir(&ws, &a, MemberRole::Front).unwrap();
        assert!(claimed.join(MEMBER_FILE).is_file());
        // Same project re-claims fine (stable id).
        assert_eq!(
            claim_member_dir(&ws, &a, MemberRole::Front).unwrap(),
            claimed
        );
        // Same basename from another path lands in a different dir.
        let other = claim_member_dir(&ws, &b, MemberRole::Front).unwrap();
        assert_ne!(claimed, other);
        // Tampered record is refused.
        let record = other.join(MEMBER_FILE);
        fs::write(
            &record,
            serde_json::json!({"schema": MEMBER_SCHEMA, "role": "front",
                "project_path": canonical_key(&a), "created_at_ms": 1})
            .to_string(),
        )
        .unwrap();
        let err = claim_member_dir(&ws, &b, MemberRole::Front)
            .unwrap_err()
            .to_string();
        assert!(err.contains("占用"), "{err}");
        // Old basename members never collide with new identity dirs: the
        // legacy dir stays untouched (not claimed, not deleted) ...
        let legacy = ws.join("app");
        fs::create_dir_all(legacy.join("src")).unwrap();
        fs::write(legacy.join("src/main.rs"), "fn main(){}").unwrap();
        let legacy_project = base.join("projC").join("app");
        fs::create_dir_all(&legacy_project).unwrap();
        let claimed_c = claim_member_dir(&ws, &legacy_project, MemberRole::Front).unwrap();
        assert!(legacy.join("src/main.rs").is_file());
        assert_ne!(claimed_c, legacy);
        // ... and a populated directory at the NEW identity without member.json
        // is refused, never silently reclaimed.
        let stale_project = base.join("projD").join("app");
        fs::create_dir_all(&stale_project).unwrap();
        let stale = ws.join(member_dir_name(&stale_project, MemberRole::Front));
        fs::create_dir_all(&stale).unwrap();
        fs::write(stale.join("Cargo.toml"), "[package]").unwrap();
        let err = claim_member_dir(&ws, &stale_project, MemberRole::Front)
            .unwrap_err()
            .to_string();
        assert!(err.contains("不认领") || err.contains("归属"), "{err}");
    }

    #[test]
    fn write_if_changed_is_stable() {
        let dir = tmp();
        let path = dir.join("nested").join("file.toml");
        assert!(write_if_changed(&path, b"one").unwrap());
        let first = fs::metadata(&path).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert!(!write_if_changed(&path, b"one").unwrap());
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), first);
        assert!(write_if_changed(&path, b"two").unwrap());
        assert_eq!(fs::read(&path).unwrap(), b"two");
        assert!(!path.with_extension("tmp-p910").exists());
    }

    #[test]
    fn publish_artifact_hardlinks_and_replaces() {
        let dir = tmp();
        let src = dir.join("uplifted.exe");
        fs::write(&src, b"payload-a").unwrap();
        let member = dir.join("member");
        let dest = publish_artifact(&src, &member, false, "app.exe").unwrap();
        assert!(dest.is_file());
        assert_eq!(fs::read(&dest).unwrap(), b"payload-a");
        // Rebuild same member with new bytes replaces the publication.
        fs::write(&src, b"payload-b-longer").unwrap();
        let dest2 = publish_artifact(&src, &member, false, "app.exe").unwrap();
        assert_eq!(dest2, dest);
        assert_eq!(fs::read(&dest).unwrap(), b"payload-b-longer");
        // The published exe is an independent copy: rewriting the uplifted
        // source afterwards must not change the published bytes (a hard link
        // would), so cargo can relink while the published copy runs.
        fs::write(&src, b"payload-c").unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"payload-b-longer");
    }

    #[test]
    fn workspace_guard_is_reentrant_and_locks_the_file() {
        let dir = tmp();
        let ws = dir.join("ws");
        {
            let _outer = WorkspaceGenerationGuard::acquire(&ws).unwrap();
            {
                let _inner = WorkspaceGenerationGuard::acquire(&ws).unwrap();
            }
            assert!(ws.join(".generation.lock").is_file());
        }
        // Fresh acquisition after drop still works (lock released).
        let _again = WorkspaceGenerationGuard::acquire(&ws).unwrap();
    }

    #[test]
    fn lease_acquires_and_releases() {
        let dir = tmp();
        let root = dir.join("ctxroot");
        {
            let _lease = TargetLease::acquire(&root, "test").unwrap();
            let live = live_leases(&root);
            assert_eq!(live.len(), 1, "{live:?}");
        }
        assert!(live_leases(&root).is_empty());
    }

    #[test]
    fn context_uses_managed_stack_flag_and_respects_user_override() {
        let _env_guard = ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // Deterministic producer selection regardless of ambient env.
        let saved: Vec<(&'static str, Option<std::ffi::OsString>)> = [
            "RUSTFLAGS",
            "CARGO_ENCODED_RUSTFLAGS",
            "CARGO_TARGET_DIR",
            TARGET_ROOT_ENV,
            "AUTO_LANG_ROOT",
            "AUTO_DELIVERY_SOURCE_ROOT",
            "AUTO_LANG_CRATE",
            "AUTO_RUST_WORKSPACE",
        ]
        .iter()
        .map(|k| (*k, std::env::var_os(k)))
        .collect();
        for (key, _) in &saved {
            std::env::remove_var(key);
        }
        let producer = make_producer_fixture();
        std::env::set_var("AUTO_LANG_ROOT", producer.to_string_lossy().to_string());
        let project = producer.join("examples/ui/002-counter");
        let ws = project.join("rust-workspace");
        let ctx = GeneratedBuildContext::resolve(&project, &ws, WorkspaceRequirements::Ui).unwrap();
        assert!(matches!(ctx.mode, ContextMode::Development { .. }));
        let env_pairs = ctx.cargo_env();
        assert_eq!(
            env_pairs
                .iter()
                .find(|(k, _)| k == "CARGO_ENCODED_RUSTFLAGS")
                .map(|(_, v)| v.clone()),
            Some(MANAGED_MSVC_STACK_FLAG.to_string())
        );
        assert!(env_pairs.iter().any(|(k, v)| k == "CARGO_TARGET_DIR"
            && v.replace('\\', "/").ends_with(&format!(
                "target/{GENERATED_CONSUMERS_DIR}/{}",
                ctx.context_id
            ))));
        ctx.write_context_file().unwrap();
        let record: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(ctx.target_root.join(CONTEXT_FILE)).unwrap())
                .unwrap();
        assert_eq!(record["schema"], CONTEXT_SCHEMA);
        assert_eq!(record["context_id"], ctx.context_id.as_str());
        assert_eq!(record["producer_origin"], "AUTO_LANG_ROOT");
        // Second write with unchanged identity does not rewrite.
        let mtime = fs::metadata(ctx.target_root.join(CONTEXT_FILE))
            .unwrap()
            .modified()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        // write_context_file maps to (); verify mtime stability directly.
        ctx.write_context_file().unwrap();
        assert_eq!(
            fs::metadata(ctx.target_root.join(CONTEXT_FILE))
                .unwrap()
                .modified()
                .unwrap(),
            mtime
        );
        // User override: flag not injected, context id changes.
        std::env::set_var("RUSTFLAGS", "-C opt-level=9");
        let ctx_user =
            GeneratedBuildContext::resolve(&project, &ws, WorkspaceRequirements::Ui).unwrap();
        assert!(!ctx_user
            .cargo_env()
            .iter()
            .any(|(k, _)| k == "CARGO_ENCODED_RUSTFLAGS"));
        assert_ne!(ctx_user.context_id, ctx.context_id);
        for (key, value) in saved {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn explicit_root_rejects_tool_target_and_foreign_checkouts() {
        let _env_guard = ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let saved: Vec<(&'static str, Option<std::ffi::OsString>)> = [
            TARGET_ROOT_ENV,
            "CARGO_TARGET_DIR",
            "AUTO_LANG_ROOT",
            "AUTO_DELIVERY_SOURCE_ROOT",
            "AUTO_LANG_CRATE",
        ]
        .iter()
        .map(|k| (*k, std::env::var_os(k)))
        .collect();
        for (key, _) in &saved {
            std::env::remove_var(key);
        }
        let producer = make_producer_fixture();
        std::env::set_var("AUTO_LANG_ROOT", producer.to_string_lossy().to_string());
        let project = producer.join("examples/ui/002-counter");
        let ws = project.join("rust-workspace");
        fs::create_dir_all(producer.join("target")).unwrap();
        std::env::set_var(
            TARGET_ROOT_ENV,
            producer.join("target").to_string_lossy().to_string(),
        );
        let err = GeneratedBuildContext::resolve(&project, &ws, WorkspaceRequirements::Ui)
            .unwrap_err()
            .to_string();
        assert!(err.contains("工具 target"), "{err}");
        let foreign = tmp().join("other-repo");
        fs::create_dir_all(foreign.join(".git")).unwrap();
        fs::create_dir_all(&foreign).unwrap();
        fs::create_dir_all(foreign.join("cache")).unwrap();
        std::env::set_var(
            TARGET_ROOT_ENV,
            foreign.join("cache").to_string_lossy().to_string(),
        );
        let err = GeneratedBuildContext::resolve(&project, &ws, WorkspaceRequirements::Ui)
            .unwrap_err()
            .to_string();
        assert!(err.contains("另一检出"), "{err}");
        // A plain directory outside any checkout is accepted.
        let plain = tmp().join("shared-cache");
        fs::create_dir_all(&plain).unwrap();
        std::env::set_var(TARGET_ROOT_ENV, plain.to_string_lossy().to_string());
        let ctx = GeneratedBuildContext::resolve(&project, &ws, WorkspaceRequirements::Ui).unwrap();
        assert!(matches!(ctx.mode, ContextMode::Explicit { .. }));
        assert_eq!(ctx.target_root, plain);
        std::env::remove_var(TARGET_ROOT_ENV);
        for (key, value) in saved {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn front_receipt_roundtrip_and_input_freshness() {
        let dir = tmp();
        let member = dir.join("member");
        fs::create_dir_all(&member).unwrap();
        let mut receipt = FrontReceipt {
            generator: "test-gen".into(),
            context_id: "abcd1234".into(),
            target_root: dir.join("root").to_string_lossy().to_string(),
            ..Default::default()
        };
        receipt
            .inputs
            .insert("src/front/app.at".into(), "00ff".into());
        assert!(receipt.write(&member).unwrap());
        let read = FrontReceipt::read(&member).unwrap();
        assert_eq!(
            read.inputs.get("src/front/app.at").cloned(),
            Some("00ff".to_string())
        );
        let mut current = BTreeMap::new();
        current.insert("src/front/app.at".to_string(), "00ff".to_string());
        assert!(read.inputs_match(&dir, &current));
        current.insert("src/front/app.at".to_string(), "01ee".to_string());
        assert!(!read.inputs_match(&dir, &current));
    }

    /// Minimal producer fixture: a fake language source tree that satisfies
    /// `select_language_producer` (workspace root + required crates).
    fn make_producer_fixture() -> PathBuf {
        let root = tmp().join("auto-lang");
        fs::create_dir_all(root.join("crates")).unwrap();
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        for name in ["auto-lang", "auto-lang-core", "a2r-std", "auto-media-core"] {
            let package = root.join("crates").join(name);
            fs::create_dir_all(package.join("src")).unwrap();
            fs::write(
                package.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n"),
            )
            .unwrap();
            fs::write(package.join("src/lib.rs"), "").unwrap();
        }
        // auto-lang must have real deps resolved? development deps validator
        // requires no path deps pointing to missing crates for [dependencies]
        // entries; the fixture manifests have none.
        root
    }
}
