//! PLAN-884 T-04：storage 后备（STORAGE_MAP/DIRTY + 文件装载/合并写 + host 发布面）
//! 自 stdlib 壳迁入。
use super::*;

// ── Plan 401: localStorage-style key/value store (Storage module) ────────
// The VM/iced path has no browser, so back storage.get/set by an in-process
// HashMap (session-scoped persistence). vue implements these against the real
// browser localStorage; this is the VM-side mirror so the same .at source runs
// in both render modes.
//
// Plan 438 M2: the map is now file-backed — localStorage is per-origin AND
// durable, so the mirror is per-project (cwd-hashed JSON under the temp dir)
// and survives process restarts, making panel-config persistence work in vm
// mode (025-dashboard). `AUTO_VM_STORAGE_FILE` overrides the location (e.g.
// a durable per-user path; the desktop_mcp suite uses it for isolation).
// All file IO is best-effort: a missing/unreadable file starts empty, a
// failed write stays in-memory (config persistence is a convenience, never
// a correctness gate).
lazy_static::lazy_static! {
    pub(crate) static ref STORAGE_MAP: std::sync::Mutex<std::collections::HashMap<String, String>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
    /// PLAN-044 T-02：本进程写过的键（写路径统一登记）。持久化合并语义
    /// 「盘 ∪ 脏键」的脏键集——未触键以盘上新值为准，脏键本进程胜（含
    /// 删除胜）。跨进程不共享（各进程各记各的），恰是合并写需要的语义。
    static ref STORAGE_DIRTY: std::sync::Mutex<std::collections::HashSet<String>> =
        std::sync::Mutex::new(std::collections::HashSet::new());
}

/// Location of the backing JSON file. Deterministic per cwd (DefaultHasher
/// with fixed keys), so a project keeps its store across runs.
fn storage_file() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("AUTO_VM_STORAGE_FILE") {
        if !p.is_empty() {
            return Some(std::path::PathBuf::from(p));
        }
    }
    let cwd = std::env::current_dir().ok()?;
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    cwd.hash(&mut h);
    Some(
        std::env::temp_dir()
            .join("auto-vm-storage")
            .join(format!("{:016x}.json", h.finish())),
    )
}

/// Merge the backing file into the map. entry().or_insert() keeps values
/// already written in this process, so in-process writes always win over
/// the on-disk snapshot.
pub(crate) fn storage_load() {
    let Some(path) = storage_file() else { return };
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return;
    };
    let Ok(parsed) = serde_json::from_str::<std::collections::HashMap<String, String>>(&raw) else {
        return;
    };
    let mut map = STORAGE_MAP.lock().unwrap();
    for (k, v) in parsed {
        map.entry(k).or_insert(v);
    }
}

/// Write the whole map through to the backing file (best-effort). Config
/// writes are low-frequency, so write-through needs no flush hook.
///
/// PLAN-044 T-02：改为**键级合并写**——`<file>.lock` 互斥下重读磁盘 →
/// 盘 ∪ 脏键 → [`crate::state_file::atomic_write`] 原子替换。多进程
/// （宿主 × outproc 子进程）交错写不同键两键均存活（修复前全量覆盖必丢，
/// tests/storage_cross_process.rs 红→绿对拍）；锁超时 best-effort 降级时
/// 仍走合并（降级只跳过互斥）；盘文件坏 JSON → `.corrupt.bak` 留证后
/// 以内存重建（撕裂自愈）。
pub(crate) fn storage_persist() {
    let Some(path) = storage_file() else { return };
    let dirty: Vec<String> = {
        let d = STORAGE_DIRTY.lock().unwrap();
        if d.is_empty() {
            return;
        }
        d.iter().cloned().collect()
    };
    let wrote = crate::state_file::with_lock(&path, crate::state_file::LOCK_TIMEOUT, || {
        let mut disk = match storage_read_disk(&path) {
            Ok(m) => m,
            Err(raw) => {
                let _ = std::fs::write(path.with_extension("json.corrupt.bak"), &raw);
                std::collections::HashMap::new()
            }
        };
        let map = STORAGE_MAP.lock().unwrap();
        for k in &dirty {
            match map.get(k) {
                Some(v) => {
                    disk.insert(k.clone(), v.clone());
                }
                None => {
                    disk.remove(k);
                }
            }
        }
        match serde_json::to_string_pretty(&disk) {
            Ok(json) => crate::state_file::atomic_write(&path, json.as_bytes()).is_ok(),
            Err(_) => false,
        }
    })
    .into_inner();
    if wrote {
        // 只清除本次已落盘的脏键（并发线程新标的键保留待下轮）
        let mut d = STORAGE_DIRTY.lock().unwrap();
        for k in &dirty {
            d.remove(k);
        }
    }
}

/// 直读磁盘 storage 库（合并写锁内专用；缺文件/不可读 = 空库，坏 JSON =
/// Err(原文)——调用方留证重建）。
fn storage_read_disk(
    path: &std::path::Path,
) -> Result<std::collections::HashMap<String, String>, String> {
    match std::fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str(&raw).map_err(|_| raw),
        Err(_) => Ok(std::collections::HashMap::new()),
    }
}

/// 写路径统一登记脏键（PLAN-044 T-02；合并写语义见 [`storage_persist`]）。
pub(crate) fn storage_mark_dirty(key: &str) {
    STORAGE_DIRTY.lock().unwrap().insert(key.to_string());
}

// Plan 442 B-support: raw accessors behind the JS-shaped localStorage
// bridge (native.rs shims) — getItem returns None for a missing key (musk
// sources test `saved != None`), unlike storage.get's "" default.
// KD-048「KV 会话恢复断裂」补齐：raw 家族原先只在进程内存 MAP 打转
// （get 不 load、set/remove 不 persist），VM 轨 localStorage 写的键
// （musk jwt/登录凭据）跨重启即失。改为与 Storage.* 同一 load-once +
// write-through 契约；None 语义保持不变。
pub(crate) fn storage_raw_get(key: &str) -> Option<String> {
    storage_load();
    STORAGE_MAP.lock().unwrap().get(key).cloned()
}

pub fn storage_raw_set(key: String, value: String) {
    storage_load();
    storage_mark_dirty(&key);
    STORAGE_MAP.lock().unwrap().insert(key, value);
    storage_persist();
}

pub fn storage_raw_remove(key: &str) {
    storage_load();
    STORAGE_MAP.lock().unwrap().remove(key);
    storage_mark_dirty(key);
    storage_persist();
}

/// PLAN-046-B (auto-musk T7): host-side publish so non-.at runtime components
/// (the iced renderer's window lifecycle) feed the same Plan 401 session KV
/// that .at code reads via the localStorage/storage shim family. Same
/// load-once + write-through contract as Storage.set.
pub fn storage_host_publish(key: &str, value: String) {
    storage_load();
    STORAGE_MAP.lock().unwrap().insert(key.to_string(), value);
    storage_mark_dirty(key);
    storage_persist();
}

/// Plan 472 T4：host-side read（与 [`storage_host_publish`] 对偶）——
/// load-once 后读，非 .at 运行时组件（renderer boot 的 dock 配置边距）
/// 与 .at storage.get 同源同契约。
pub fn storage_host_read(key: &str) -> Option<String> {
    storage_load();
    STORAGE_MAP.lock().unwrap().get(key).cloned()
}

/// PLAN-028：host-side 新鲜读——绕开 load-once `or_insert` 的首读冻结
/// 语义（[`storage_host_read`] 对已加载键永远返回首读值，发布端后续
/// 写入不可见）。分体轨后端按拍拉取前端发布的窗口尺寸真值用：每次
/// 直读 backing 文件最新值；键不在盘上时回落进程内 map（含本进程
/// 直写）。只读不落缓存——不污染 load-once 语义。
pub fn storage_host_read_fresh(key: &str) -> Option<String> {
    if let Some(path) = storage_file() {
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(parsed) =
                serde_json::from_str::<std::collections::HashMap<String, String>>(&raw)
            {
                if let Some(v) = parsed.get(key) {
                    return Some(v.clone());
                }
            }
        }
    }
    STORAGE_MAP.lock().unwrap().get(key).cloned()
}
