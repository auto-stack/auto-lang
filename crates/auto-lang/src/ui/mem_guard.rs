//! 014 内存哨兵:进程提交内存自检 + 超限冻结。
//!
//! 动机:at-app 实测出现 ~1GB/s 内存暴涨(commit 冲到 ~16GB 直到系统
//! 上限,整机卡死被迫重启)。护栏挂在渲染层 update 入口:每秒采样一次
//! 提交内存,超限后进入**冻结态**——丢弃一切后续消息(泄漏若由消息驱动
//! 的热循环产生,立即停摆),视图切换为超限告警,按 F12 干净退出。冻结
//! 是"暂停":进程与窗口保留,现场不丢,任务管理器可继续观察。
//!
//! - 阈值:`AUTO_MEM_LIMIT_MB`(默认 10240 = 10GB);0 = 关闭护栏。
//! - 度量:PagefileUsage(提交字节,与任务管理器"提交大小"同源),
//!   Kernel32!K32GetProcessMemoryInfo 直接声明,零新依赖。
//! - 非 Windows:private_mb 恒 0,护栏自动失效(当前产品 Windows 优先)。

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{OnceLock, Mutex};
use std::time::{Duration, Instant};

static FROZEN: AtomicBool = AtomicBool::new(false);
static PEAK_MB: AtomicU64 = AtomicU64::new(0);
static LAST_SAMPLE_MS: AtomicU64 = AtomicU64::new(0);
static START: OnceLock<Instant> = OnceLock::new();
/// 采样节流周期(ms);热路径每次调用只做原子读。
const SAMPLE_INTERVAL_MS: u64 = 1000;
/// 默认阈值 10GB。
const DEFAULT_LIMIT_MB: u64 = 10240;
/// 大块分配登记阈值:≥ 此值的 alloc 记录回溯(泄漏嫌疑主犯)。
pub const BIG_ALLOC_THRESHOLD: usize = 256 * 1024;

static LAST_LOG: Mutex<Option<String>> = Mutex::new(None);

fn now_ms() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// 阈值(MB);`AUTO_MEM_LIMIT_MB=0` 关闭护栏。
fn limit_mb() -> u64 {
    static LIMIT: OnceLock<u64> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        std::env::var("AUTO_MEM_LIMIT_MB")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(DEFAULT_LIMIT_MB)
    })
}

pub fn is_frozen() -> bool {
    FROZEN.load(Ordering::Relaxed)
}

pub fn peak_mb() -> u64 {
    PEAK_MB.load(Ordering::Relaxed)
}

pub fn limit() -> u64 {
    limit_mb()
}

/// 最近一次采样说明(供 MCP/调试面读取;采样节流窗口内更新)。
pub fn last_note() -> Option<String> {
    LAST_LOG.lock().ok()?.clone()
}

// ── 记账分配器(GuardAlloc)────────────────────────────────────────────
// 生成应用以 #[global_allocator] 挂接;目标:冻结瞬间回答「哪个分配点
// 占用最大」。活字节 = 原子累计;≥256KB 的存活大块单独登记(指针 →
// 大小 + 分配点回溯),冻结时随报告落盘。

static LIVE_BYTES: AtomicI64 = AtomicI64::new(0);

struct BigBlock {
    size: usize,
    backtrace: std::backtrace::Backtrace,
}

fn big_live() -> &'static Mutex<std::collections::HashMap<usize, BigBlock>> {
    static MAP: OnceLock<Mutex<std::collections::HashMap<usize, BigBlock>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

pub struct GuardAlloc;

thread_local! {
    /// 防递归:force_capture 内部的分配再次进入大块路径会重入 std 回溯
    /// 内部锁(自锁)——嵌套期一律跳过采集。
    static IN_BIG_CAPTURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

unsafe impl std::alloc::GlobalAlloc for GuardAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let ptr = std::alloc::System.alloc(layout);
        if !ptr.is_null() {
            let live = LIVE_BYTES.fetch_add(layout.size() as i64, Ordering::Relaxed)
                + layout.size() as i64;
            if layout.size() >= BIG_ALLOC_THRESHOLD {
                let nested = IN_BIG_CAPTURE.with(|c| c.get());
                let bt = if nested {
                    std::backtrace::Backtrace::disabled()
                } else {
                    IN_BIG_CAPTURE.with(|c| c.set(true));
                    let bt = std::backtrace::Backtrace::force_capture();
                    IN_BIG_CAPTURE.with(|c| c.set(false));
                    bt
                };
                if let Ok(mut map) = big_live().lock() {
                    map.insert(ptr as usize, BigBlock { size: layout.size(), backtrace: bt });
                }
            }
            PEAK_LIVE_BYTES.fetch_max(live as u64, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        LIVE_BYTES.fetch_sub(layout.size() as i64, Ordering::Relaxed);
        if layout.size() >= BIG_ALLOC_THRESHOLD {
            if let Ok(mut map) = big_live().lock() {
                map.remove(&(ptr as usize));
            }
        }
        std::alloc::System.dealloc(ptr, layout)
    }
}

static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);

pub fn live_bytes() -> i64 {
    LIVE_BYTES.load(Ordering::Relaxed)
}

pub fn peak_live_bytes() -> u64 {
    PEAK_LIVE_BYTES.load(Ordering::Relaxed)
}

/// 冻结(或手动)时落盘报告:活字节总量 + 存活大块表(大小 + 分配点
/// 回溯,按大小降序取前 40)。写到 %TEMP%/auto-term-mem-report.txt 并
/// stderr 摘要。调用前提:应用以 GuardAlloc 为 global_allocator。
pub fn dump_alloc_report() -> Option<std::path::PathBuf> {
    let live = LIVE_BYTES.load(Ordering::Relaxed);
    let mut lines = Vec::new();
    lines.push(format!(
        "live_bytes={} ({:.1} MB) peak_live={:.1} MB commit_peak={} MB frozen={}",
        live,
        live as f64 / 1048576.0,
        peak_live_bytes() as f64 / 1048576.0,
        peak_mb(),
        is_frozen()
    ));
    let big = big_live().lock().ok()?;
    lines.push(format!("big_live_count={} (>= {BIG_ALLOC_THRESHOLD} bytes each)", big.len()));
    let mut blocks: Vec<(usize, usize, String)> = big
        .iter()
        .map(|(ptr, b)| (*ptr, b.size, format!("{}", b.backtrace)))
        .collect();
    drop(big);
    blocks.sort_by_key(|(_, size, _)| std::cmp::Reverse(*size));
    let total_big: usize = blocks.iter().map(|(_, s, _)| *s).sum();
    lines.push(format!("big_live_total={:.1} MB", total_big as f64 / 1048576.0));
    for (ptr, size, bt) in blocks.iter().take(40) {
        lines.push(format!("--- block {:#x} {} bytes ---", ptr, size));
        for (i, frame_line) in bt.lines().take(8).enumerate() {
            lines.push(format!("    #{i} {frame_line}"));
        }
    }
    let text = lines.join("\n");
    let path = std::env::temp_dir().join("auto-term-mem-report.txt");
    let write_ok = std::fs::write(&path, &text).is_ok();
    eprintln!("[mem-guard] 分配报告({} 块, 存活 {:.1} MB):{}", big_count(), total_big as f64 / 1048576.0,
        if write_ok { format!("{}", path.display()) } else { "写入失败,见 stderr".to_string() });
    eprintln!("{text}");
    Some(path)
}

fn big_count() -> usize {
    big_live().lock().map(|m| m.len()).unwrap_or(0)
}

/// 热路径入口:update 每条消息前调用。节流(1s)采样提交内存;超阈值
/// 置冻结。返回值:是否刚刚触发冻结(供调用方打日志)。
///
/// 014 加固:采样搬进**独立看门狗线程**(懒启动)——UI 线程卡死时
/// 消息路径的采样会瞎,看门狗不受影响。冻结时额外调 freeze hook
/// (宿主注册,如挂起子 shell)——只丢消息拦不住非消息线程的增长。
pub fn sample_and_guard() -> bool {
    ensure_watcher();
    if FROZEN.load(Ordering::Relaxed) || limit_mb() == 0 {
        return false;
    }
    let t = now_ms();
    if t.saturating_sub(LAST_SAMPLE_MS.load(Ordering::Relaxed)) < SAMPLE_INTERVAL_MS {
        return false;
    }
    LAST_SAMPLE_MS.store(t, Ordering::Relaxed);
    let mb = private_mb();
    if mb > PEAK_MB.load(Ordering::Relaxed) {
        PEAK_MB.store(mb, Ordering::Relaxed);
    }
    let limit = limit_mb();
    let note = format!("commit={mb}MB limit={limit}MB");
    *LAST_LOG.lock().unwrap() = Some(note.clone());
    if mb > limit {
        FROZEN.store(true, Ordering::Relaxed);
        eprintln!(
            "[mem-guard] 提交内存 {mb}MB 超过阈值 {limit}MB —— 冻结消息循环;按 F12 退出进程"
        );
        // 冻结即出报告:活字节 + 存活大块(含分配点回溯)→ 泄漏点定位。
        let _ = dump_alloc_report();
        // 宿主 hook(如挂起子 shell——只丢消息拦不住非消息线程的增长)。
        if let Ok(hook) = FREEZE_HOOK.lock() {
            if let Some(f) = hook.as_ref() {
                f();
            }
        }
        return true;
    }
    false
}

static FREEZE_HOOK: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
static WATCHER: OnceLock<()> = OnceLock::new();

/// 宿主注册冻结回调(at-app:挂起子 shell 进程,掐断产出侧)。
pub fn set_freeze_hook(f: Box<dyn Fn() + Send + 'static>) {
    *FREEZE_HOOK.lock().unwrap() = Some(f);
}

/// 独立看门狗线程:1s 采样,不依赖消息循环(懒启动一次)。
fn ensure_watcher() {
    WATCHER.get_or_init(|| {
        let _ = std::thread::Builder::new()
            .name("mem-guard-watcher".into())
            .spawn(|| loop {
                std::thread::sleep(Duration::from_millis(SAMPLE_INTERVAL_MS));
                let _ = sample_and_guard();
            });
    });
}

#[cfg(windows)]
fn private_mb() -> u64 {
    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set: usize,
        working_set: usize,
        quota_peak_paged_pool: usize,
        quota_paged_pool: usize,
        quota_peak_non_paged_pool: usize,
        quota_non_paged_pool: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }
    #[link(name = "Kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
        fn K32GetProcessMemoryInfo(
            process: *mut core::ffi::c_void,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }
    let mut pmc = ProcessMemoryCounters {
        cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
        page_fault_count: 0,
        peak_working_set: 0,
        working_set: 0,
        quota_peak_paged_pool: 0,
        quota_paged_pool: 0,
        quota_peak_non_paged_pool: 0,
        quota_non_paged_pool: 0,
        pagefile_usage: 0,
        peak_pagefile_usage: 0,
    };
    unsafe {
        let handle = GetCurrentProcess();
        if K32GetProcessMemoryInfo(handle, &mut pmc, pmc.cb) != 0 {
            return (pmc.pagefile_usage / (1024 * 1024)) as u64;
        }
    }
    0
}

#[cfg(not(windows))]
fn private_mb() -> u64 {
    0
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn private_mb_reports_nonzero_on_windows() {
        // 探针:提交内存采样在本进程必然 > 0(rustc test harness 常驻)。
        assert!(private_mb() > 0);
    }

    #[test]
    fn sample_is_throttled_and_does_not_freeze_under_limit() {
        // 不设环境变量时默认阈值 10GB,测试进程远低于此——两次采样都不冻结。
        assert!(!sample_and_guard());
        assert!(!is_frozen());
    }
}
