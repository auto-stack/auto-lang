//! PLAN-662 T-02: 画廊行扫描磁盘缓存。
//!
//! 痛点：画廊启动（VM 臂 registry/proxy 编排 + Vue 臂 host 生成共用扫描面）
//! 逐 demo 跑 `VueProject::from_workspace`（pac resolve + SFC 编译 ~2s/个，
//! 36 demo 串行 ~77s），且既有 `GALLERY_ROWS_CACHE` 为进程内线程局部——
//! 二次启动同样慢。本模块把 per-demo 分析产物（`GalleryDemoRow`）按
//! **内容哈希**落盘，命中即免 `from_workspace`。
//!
//! 契约（SD-01，ui/overview.md §ui-gallery）：
//! - 键 = demo 目录（relpath+size+mtime 走查）+ pac.at 声明的路径依赖目录
//!   （stylekit 等共享依赖在一次调用内 memo）的 FNV-1a 64 摘要；
//! - 存储 = `AUTO_GALLERY_CACHE_DIR` env 覆写，缺省
//!   `<home>/.auto/auto-man/gallery-cache/<apps_dir 摘要>.json`；
//! - fail-open：读不出/版本不符/字段缺失 → 整体丢弃按未命中处理，写失败
//!   仅告警，绝不阻断画廊；
//! - 确定性：缓存只重放分析结果，不改变发射输入——命中/冷跑产出的
//!   registry.at 必须字节一致（测试钉住）；
//! - `CACHE_FORMAT_VERSION` 在行字段或 loadable/fullstack 判定逻辑变化时
//!   手动 bump，旧缓存整体失效。

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::vue::GalleryDemoRow;

const CACHE_FORMAT_VERSION: u32 = 1;

/// 一条缓存记录：demo id → 内容哈希 + 行。
pub type DiskEntries = HashMap<String, (String, GalleryDemoRow)>;

#[derive(Serialize, Deserialize)]
struct CacheFile {
    version: u32,
    entries: DiskEntries,
}

/// 缓存根目录：env 权威，缺省 `<home>/.auto/auto-man/gallery-cache`。
fn cache_root() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("AUTO_GALLERY_CACHE_DIR") {
        return Some(PathBuf::from(d));
    }
    dirs::home_dir().map(|h| h.join(".auto").join("auto-man").join("gallery-cache"))
}

/// apps_dir → 缓存文件路径（路径串 FNV 摘要做文件名，避免跨检出互撞）。
fn cache_file(apps_dir: &Path) -> Option<PathBuf> {
    Some(cache_root()?.join(format!(
        "rows-{:016x}.json",
        fnv1a64(apps_dir.to_string_lossy().to_lowercase().as_bytes())
    )))
}

/// FNV-1a 64——缓存键/文件名用途（失效检测，非安全哈希），零新依赖。
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// 读缓存：任何失败（缺文件/损坏/版本不符）→ 空 map（fail-open）。
pub fn load(apps_dir: &Path) -> DiskEntries {
    let Some(path) = cache_file(apps_dir) else {
        return HashMap::new();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return HashMap::new();
    };
    match serde_json::from_str::<CacheFile>(&text) {
        Ok(f) if f.version == CACHE_FORMAT_VERSION => f.entries,
        _ => HashMap::new(),
    }
}

/// 写缓存：temp+rename 原子落盘；内容与既有文件相同则跳过（全命中时零写）。
pub fn store(apps_dir: &Path, entries: &DiskEntries) {
    let Some(path) = cache_file(apps_dir) else {
        return;
    };
    let file = CacheFile {
        version: CACHE_FORMAT_VERSION,
        entries: entries.clone(),
    };
    let Ok(text) = serde_json::to_string(&file) else {
        return;
    };
    if let Ok(prev) = std::fs::read_to_string(&path) {
        if prev == text {
            return;
        }
    }
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    let tmp = path.with_extension("json.tmp");
    if let Ok(mut f) = std::fs::File::create(&tmp) {
        if f.write_all(text.as_bytes()).is_ok() {
            let _ = f.flush();
            if std::fs::rename(&tmp, &path).is_ok() {
                return;
            }
        }
    }
    eprintln!("  {} gallery rows cache write failed: {}", "⚠".bright_yellow(), path.display());
}

/// 目录内容摘要：walk（不跟符号链接）收集 `rel|size|mtime` 排序后 FNV。
/// 目录不存在/不可读 → None（调用方按未命中处理，不缓存）。
fn dir_content_hash(root: &Path, memo: &mut HashMap<PathBuf, Option<String>>) -> Option<String> {
    let canon = std::fs::canonicalize(root).ok()?;
    if let Some(hit) = memo.get(&canon) {
        return hit.clone();
    }
    let mut lines: Vec<String> = Vec::new();
    let mut stack = vec![canon.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else {
            memo.insert(canon.clone(), None);
            return None;
        };
        for e in read.flatten() {
            let Ok(meta) = e.metadata() else {
                continue;
            };
            let p = e.path();
            let rel = match p.strip_prefix(&canon) {
                Ok(r) => r.to_string_lossy().replace('\\', "/"),
                Err(_) => p.to_string_lossy().replace('\\', "/"),
            };
            // 仅文件入哈希行：目录 mtime 在创建后的最初时刻可能延迟落定
            // （NTFS 实测漂移→并行测试下假失效），且目录元数据变化不携带
            // 行计算关心的信息——增删文件必然改变文件行集合。
            if meta.is_dir() {
                stack.push(p);
                continue;
            }
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis())
                .unwrap_or(0);
            lines.push(format!("{}|{}|{}", rel, meta.len(), mtime));
        }
    }
    lines.sort();
    let mut acc = String::new();
    for l in &lines {
        acc.push_str(l);
        acc.push('\n');
    }
    let h = format!("{:016x}", fnv1a64(acc.as_bytes()));
    memo.insert(canon, Some(h.clone()));
    Some(h)
}

/// demo 内容哈希：demo 目录 + pac.at 声明的路径依赖目录（dep 块的 path）。
/// 依赖目录（如共享 stylekit）经 memo 每调用一次只摘要一遍。
pub fn demo_content_hash(app_root: &Path, memo: &mut HashMap<PathBuf, Option<String>>) -> Option<String> {
    let mut acc = dir_content_hash(app_root, memo)?;
    for dep in pac_dep_roots(app_root) {
        acc.push('\n');
        acc.push_str(&dir_content_hash(&dep, memo)?);
    }
    Some(format!("{:016x}", fnv1a64(acc.as_bytes())))
}

/// pac.at `dep <name> { path: "..." }` 块的 path 值解析（行级扫描，镜像
/// vue.rs stylekit_pub_recipes 的块约定；解析失败的块静默跳过）。
pub fn pac_dep_roots(app_root: &Path) -> Vec<PathBuf> {
    let Ok(pac) = std::fs::read_to_string(app_root.join("pac.at")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut in_dep = false;
    let mut dep_path: Option<String> = None;
    for l in pac.lines() {
        let t = l.trim_start();
        if t.starts_with("dep ") {
            in_dep = true;
            dep_path = None;
            continue;
        }
        if in_dep {
            if t.starts_with("path:") {
                dep_path = t["path:".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim_matches(',')
                    .to_string()
                    .into();
            }
            if t.starts_with('}') {
                if let Some(rel) = dep_path.take() {
                    let base = app_root.join(rel.trim_end_matches(['/', '\\']));
                    if base.is_dir() {
                        out.push(base);
                    }
                }
                in_dep = false;
            }
        }
    }
    out
}
