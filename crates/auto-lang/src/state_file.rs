//! 用户级状态文件写安全工具层（PLAN-044）。
//!
//! `~/.config/autoos` 下的配置/状态文件此前全部是「`fs::write` 全量覆盖 +
//! load-once 内存态」，多进程并发（桌面宿主 × outproc 子进程 × os-config
//! daemon）交错即产生丢更新；写盘中途被杀则留下撕裂文件（坏 JSON 静默
//! 空库）。本模块提供 OS 级先例的轻量等价物（对齐 Chromium
//! ImportantFileWriter / git index.lock 形态，单写者 daemon 化为远期债）：
//!
//! - L1 [`atomic_write`]：同目录 tmp 写 + flush + rename 原子替换
//!   （Windows 上 `std::fs::rename` = `MoveFileExW(REPLACE_EXISTING)`，
//!   同卷替换原子；tmp 与目标同目录保证同卷）。
//! - L2 [`with_lock`]：`<目标>.lock` 独占创建互斥（O_CREAT|O_EXCL 竞态
//!   安全），把 read-modify-write 圈在锁内。
//! - L3 键级合并写在消费方（`vm::ffi::stdlib::storage_persist`，盘 ∪ 脏键）。
//!
//! # 跨仓锁路径约定（单源）
//!
//! 互斥锁文件 = `<目标文件>.lock`（与目标同目录）；锁体两行文本：
//! `pid` 与 `epoch_ms`。auto-os-config daemon（独立 Rust 仓）的本地替身
//! 必须保持同一约定，两侧 doc 注释互指本模块。
//!
//! # v1 取舍
//!
//! - stale 判定取**锁龄** = max(mtime 龄, 锁体 epoch 龄)（> [`STALE_LOCK_AGE`]
//!   视为崩溃残留可接管）——std 无 pid 存活探测；锁内临界区都是 ms 级
//!   RMW，活锁超龄接管的概率可忽略，eprintln 留痕。
//! - 锁获取超时后 **best-effort 降级放行**（可用性优先；降级只跳过互斥，
//!   消费方的合并语义照常执行）——PLAN-044 §10.2 缺省取向。

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// 锁获取超时（生产缺省；超时后降级放行）。
pub const LOCK_TIMEOUT: Duration = Duration::from_secs(2);
/// 锁龄超过此值视为 stale（持锁进程崩溃残留），可接管。
pub const STALE_LOCK_AGE: Duration = Duration::from_secs(10);

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

fn epoch_ms_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// 锁获取结果。`Executed` = 持锁执行；`Degraded` = 超时/硬错误后放行执行
/// （互斥未保证，消费方语义仍完整）。两者都**必然**执行了闭包。
#[derive(Debug, PartialEq, Eq)]
pub enum LockOutcome<T> {
    Executed(T),
    Degraded(T),
}

impl<T> LockOutcome<T> {
    pub fn into_inner(self) -> T {
        match self {
            LockOutcome::Executed(v) | LockOutcome::Degraded(v) => v,
        }
    }

    pub fn degraded(&self) -> bool {
        matches!(self, LockOutcome::Degraded(_))
    }
}

/// 原子替换写：`path` 的父目录下创建 `.名称.tmp-<pid>-<seq>`，write_all +
/// `sync_all` 后 rename 覆盖目标。失败清理 tmp 并返回错误——调用方拿到的
/// `Err` 意味着目标文件保持原值（成功则新值）。
pub fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let dir = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "atomic_write: no parent dir")
    })?;
    std::fs::create_dir_all(dir)?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("target");
    let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".{}.tmp-{}-{}", name, std::process::id(), seq));

    let result = (|| -> std::io::Result<()> {
        {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(bytes)?;
            f.sync_all()?;
        } // drop 句柄后再 rename（Windows 不允许删除/替换打开中的文件）
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// 目标文件的互斥锁路径（跨仓约定单源，见模块头）。
pub fn lock_path_for(target: &Path) -> std::path::PathBuf {
    let mut s = target.as_os_str().to_os_string();
    s.push(".lock");
    std::path::PathBuf::from(s)
}

/// 锁体内容：pid 与 epoch_ms 两行（stale 判定/诊断用）。
fn lock_body() -> String {
    format!("{}\n{}", std::process::id(), epoch_ms_now())
}

/// 解析锁体 `(pid, epoch_ms)`；格式坏视作 epoch 0（龄满即 stale）。
fn parse_lock_body(raw: &str) -> (u32, u128) {
    let mut it = raw.lines();
    let pid = it.next().and_then(|l| l.trim().parse().ok()).unwrap_or(0);
    let epoch = it.next().and_then(|l| l.trim().parse().ok()).unwrap_or(0);
    (pid, epoch)
}

fn mtime_age_ms(path: &Path) -> u128 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .map(|mtime| epoch_ms_now().saturating_sub(mtime))
        .unwrap_or(0) // stat 失败（如正在被删除的锁）不判 stale——
                     // 活锁误删比 stale 泄漏代价高（stale 有超时降级兜底）
}

/// 锁龄（ms）= max(mtime 龄, 锁体 epoch 龄)——两者任一超龄即可判定 stale，
/// 取较大者对"活锁"最保守（真实持锁者两龄都新鲜）。锁体空/不可解析（
/// create_new 成功到写入锁体之间的微秒级空窗，或损坏残留）时 epoch 臂
/// 缺席、只依 mtime——空体文件 mtime 是新鲜的，绝不误判活锁为 stale。
fn lock_age_ms(path: &Path) -> u128 {
    let epoch_age = std::fs::read_to_string(path)
        .ok()
        .filter(|raw| !raw.trim().is_empty())
        .map(|raw| {
            let (_, epoch) = parse_lock_body(&raw);
            epoch_ms_now().saturating_sub(epoch)
        })
        .unwrap_or(0);
    mtime_age_ms(path).max(epoch_age)
}

/// 在 `<target>.lock` 互斥保护下执行 `f`。锁被他人持有时重试至 `timeout`；
/// 期间发现 stale 锁（龄 > [`STALE_LOCK_AGE`]）则删除接管；超时或硬错误
/// （权限等）**降级放行**执行 `f` 并返回 [`LockOutcome::Degraded`]。
pub fn with_lock<T>(target: &Path, timeout: Duration, f: impl FnOnce() -> T) -> LockOutcome<T> {
    let lock = lock_path_for(target);
    if let Some(dir) = lock.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let deadline = Instant::now() + timeout;
    let mut acquired = false;
    while Instant::now() < deadline {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
        {
            Ok(mut f) => {
                use std::io::Write;
                let _ = f.write_all(lock_body().as_bytes());
                acquired = true;
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if lock_age_ms(&lock) > STALE_LOCK_AGE.as_millis() {
                    // 接管：删后重走 create_new（与并发接管者竞态仍安全）
                    let _ = std::fs::remove_file(&lock);
                    continue;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(_) => break, // 权限等硬错误 → 降级
        }
    }
    if acquired {
        let out = f();
        let _ = std::fs::remove_file(&lock);
        LockOutcome::Executed(out)
    } else {
        eprintln!(
            "[state_file] lock {} acquire timeout/hard-error, proceeding degraded",
            lock.display()
        );
        LockOutcome::Degraded(f())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Arc, Mutex};

    fn tmpdir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "auto-state-file-{}-{}-{}",
            name,
            std::process::id(),
            TMP_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn atomic_write_creates_and_overwrites() {
        let dir = tmpdir("basic");
        let p = dir.join("state.json");
        atomic_write(&p, b"{\"a\":1}").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"{\"a\":1}");
        atomic_write(&p, b"{\"a\":2}").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"{\"a\":2}");
        // tmp 无残留
        let residue: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(residue.is_empty(), "tmp residue: {:?}", residue);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_write_failure_keeps_target() {
        let dir = tmpdir("fail");
        let p = dir.join("state.json");
        atomic_write(&p, b"old").unwrap();
        // 目标位置被目录占据 → rename 必败，原内容不被破坏
        let blocker = dir.join("block");
        std::fs::create_dir(&blocker).unwrap();
        assert!(atomic_write(&blocker, b"x").is_err());
        assert_eq!(std::fs::read(&p).unwrap(), b"old");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn concurrent_atomic_writes_never_tear() {
        let dir = tmpdir("tear");
        let p = dir.join("state.json");
        let payloads: Vec<String> = (0..8).map(|i| format!("{{\"w\":{}}}", i)).collect();
        let mut handles = Vec::new();
        for pl in &payloads {
            let pl = pl.clone();
            let pp = p.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..50 {
                    atomic_write(&pp, pl.as_bytes()).unwrap();
                }
            }));
        }
        // 读侧线程：任意时刻文件内容要么完整要么尚不存在（绝不半截）
        let rp = p.clone();
        let reader = std::thread::spawn(move || {
            for _ in 0..200 {
                if let Ok(raw) = std::fs::read_to_string(&rp) {
                    assert!(raw.starts_with('{') && raw.ends_with('}'), "torn: {:?}", raw);
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        });
        for h in handles {
            h.join().unwrap();
        }
        reader.join().unwrap();
        let raw = std::fs::read_to_string(&p).unwrap();
        assert!(payloads.contains(&raw));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn lock_serializes_critical_sections() {
        let dir = tmpdir("serial");
        let target = dir.join("f.json");
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..4 {
            let counter = counter.clone();
            let target = target.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..25 {
                    with_lock(&target, LOCK_TIMEOUT, || {
                        // 非原子 read-modify-write：无锁下必丢更新
                        let cur = counter.load(Ordering::SeqCst);
                        std::thread::sleep(Duration::from_micros(50));
                        counter.store(cur + 1, Ordering::SeqCst);
                    });
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(counter.load(Ordering::SeqCst), 100, "lock lost updates");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn lock_timeout_degrades_but_executes() {
        let dir = tmpdir("degrade");
        let target = dir.join("f.json");
        // 他人持锁：新鲜锁体（epoch=now），不满足 stale
        let lock = lock_path_for(&target);
        std::fs::write(&lock, format!("999999\n{}", epoch_ms_now())).unwrap();
        let outcome = with_lock(&target, Duration::from_millis(80), || 42);
        assert!(outcome.degraded());
        assert_eq!(outcome.into_inner(), 42);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stale_lock_is_taken_over() {
        let dir = tmpdir("stale");
        let target = dir.join("f.json");
        // 伪造超龄锁：锁体 epoch 拨老 20s（龄判定取 max，无需动 mtime）
        let old_epoch = epoch_ms_now().saturating_sub(20_000);
        let lock = lock_path_for(&target);
        std::fs::write(&lock, format!("999999\n{}", old_epoch)).unwrap();
        let outcome = with_lock(&target, Duration::from_millis(500), || "ran");
        assert!(!outcome.degraded(), "stale lock should be taken over");
        assert_eq!(outcome.into_inner(), "ran");
        assert!(!lock.exists(), "lock released after execution");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
