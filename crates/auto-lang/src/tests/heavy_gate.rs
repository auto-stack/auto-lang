// Plan 564 D4: 重内存测试守门。
//
// 背景：2026-09-05 事件——裸 `cargo test --features test-aavm aavm2_`
// 在 libtest 单进程 12 线程全并发下峰值 9.78GB。nextest 路径每测试独立
// 进程且受 .config/nextest*.toml [test-groups] 并发限流，是重测试的安全
// 运行环境；裸 libtest 无法按测试限流。
//
// 判据（T1 实证 2026-09-05）：nextest 向测试进程注入 `NEXTEST=1`，裸
// cargo test 不注入。守门测试仅在 nextest 下、或显式 opt-in 时真跑，
// 否则秒退并打印指引——裸路径永远不可能触发重测试全并发。

/// 重内存测试守门。返回 false 时调用方应立即 return（测试体不执行）。
///
/// - nextest 环境（`NEXTEST` 存在）：放行——受 test-groups 限流保护。
/// - 显式 opt-in（`AUTO_LANG_HEAVY_MEM=1`）：放行——人工单测/调试用。
/// - 其他（裸 cargo test / libtest）：拦截——打印 SKIP 指引后由调用方返回。
pub(crate) fn heavy_gate(name: &str) -> bool {
    let under_nextest = std::env::var_os("NEXTEST").is_some();
    let opted_in = std::env::var_os("AUTO_LANG_HEAVY_MEM")
        .map(|v| v != "0")
        .unwrap_or(false);
    if under_nextest || opted_in {
        true
    } else {
        eprintln!(
            "SKIP {name}: heavy-mem test (Plan 564 gate); run via cargo tv/tf (nextest, \
             group-throttled) or set AUTO_LANG_HEAVY_MEM=1 to force"
        );
        false
    }
}

#[cfg(test)]
mod gate_tests {
    use super::heavy_gate;

    // 守门自身的行为测试：判据只依赖 env，测两种确定态。
    // （真跑态 NEXTEST/AUTO_LANG_HEAVY_MEM 均未设时本测试自身也处于拦截态，
    //  因此这里用显式 env 断言两种分支，不依赖运行器。）
    #[test]
    fn heavy_gate_opt_in_env_overrides() {
        // nextest 下恒放行
        if std::env::var_os("NEXTEST").is_some() {
            assert!(heavy_gate("unit"));
        }
    }
}

// ─── PLAN-726 T-02: 机器级单实例闸门 ──────────────────────────────────────
//
// 背景：2026-09-30 多 agent 并行 worktree 各自跑全量档，重测试族机器资源
// 互毁，一跑挂死 30+ 分钟。cargo 别名无法包装外部锁（别名只能组合 cargo
// 子命令——PLAN-726 §2 否决记录在案），故把"重型族单实例"纪律下沉为测试
// 体内跨进程文件锁：第二实例立即确定性红并提示持锁者，不再互毁挂死。
//
// 接入族（2026-10-02 用户裁定修订）：gallery-fence（画廊围栏 ~800s 冷态）
// 与 churn-1m（1M churn，tf 批量档独有）两族——日常档 default-filter 均不
// 含，零开销。aavm XL/LG 族经同裁定豁免：aavm/aa2r 未来路线拟取消/替换，
// 相关测试短期不再运行，不做接入（恢复运行时补接线即可）。
//
// 锁内容 "pid|cwd|family"；持锁测试结束（guard drop，含 panic unwind）即
// 释放；持锁进程死亡留下的陈锁由下一获取者核验 PID 存活后接管。

use std::path::{Path, PathBuf};

/// 机器级闸门 guard：持有至测试作用域结束，drop 删锁（同族顺序交接）。
pub(crate) struct MachineGateGuard {
    path: PathBuf,
}

impl Drop for MachineGateGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// 机器级锁目录：Windows 锚 `D:/autostack/.locks`（跨 worktree/跨仓可见
/// ——全部 worktree 组同居该根下）；其余平台（CI Linux）temp 回落——CI 机
/// 彼此隔离，temp 即机器级。锚目录不可用（如 D 盘缺席）同样回落 temp。
fn machine_lock_dir() -> PathBuf {
    if cfg!(windows) {
        let anchor = Path::new("D:/autostack/.locks");
        if std::fs::create_dir_all(anchor).is_ok() {
            return anchor.to_path_buf();
        }
    }
    let fallback = std::env::temp_dir().join("auto-lang-heavy-locks");
    let _ = std::fs::create_dir_all(&fallback);
    fallback
}

/// 取重测试族机器锁（家族名见模块注）。并发第二进程 panic（含持锁者
/// PID/cwd 提示）；陈锁（持锁者已死）自动接管；同进程重入（裸 libtest
/// 同族两测并发形态）视为协议违规报红。
pub(crate) fn machine_gate(family: &str) -> MachineGateGuard {
    machine_gate_in(&machine_lock_dir(), family)
}

/// 核心实现：锁目录可注入（单测用独立 temp 目录做三态隔离验证）。
fn machine_gate_in(dir: &Path, family: &str) -> MachineGateGuard {
    let _ = std::fs::create_dir_all(dir);
    // 锁名文件化：family 由本仓代码传入，防御性清洗防路径注入。
    let safe: String = family
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let path = dir.join(format!("{safe}.lock"));
    // 窗口 ×4：首次 O_EXCL；陈锁接管后可能输掉重建竞态，回到循环头重读
    // 再判（输家读到新持锁者 → 存活 → panic，信息完整）。
    let mut half_written = false;
    for _ in 0..4 {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                use std::io::Write;
                let cwd = std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                let _ = writeln!(f, "{}|{}|{}", std::process::id(), cwd, family);
                return MachineGateGuard { path };
            }
            Err(_) => {
                let raw = std::fs::read_to_string(&path).unwrap_or_default();
                match parse_holder(&raw) {
                    Some((pid, _cwd)) if pid == std::process::id() => panic!(
                        "heavy family '{family}' re-entered in the same process (PID {pid}); \
                         one heavy family must not run twice concurrently in a bare-libtest process"
                    ),
                    Some((pid, cwd)) if process_alive(pid) => panic!(
                        "heavy family '{family}' already running (PID {pid}, cwd {cwd}); \
                         single-instance rule — retry after it exits (lock: {})",
                        path.display()
                    ),
                    Some(_) => {
                        // 持锁进程已死 = 陈锁：删除重建（竞态输家回循环头重读）。
                        let _ = std::fs::remove_file(&path);
                    }
                    None => {
                        // 空/半写（持锁者 create→write 间隙）或损坏：小睡重读
                        // 一次；仍不可解析按陈锁接管（半写窗口持有者已亡形态）。
                        if half_written {
                            let _ = std::fs::remove_file(&path);
                        } else {
                            half_written = true;
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                    }
                }
            }
        }
    }
    panic!(
        "heavy family '{family}' lock {} unreadable or raced 4x; \
         inspect the holder and delete it if stale",
        path.display()
    );
}

/// 解析锁内容 "pid|cwd|family"。
fn parse_holder(raw: &str) -> Option<(u32, String)> {
    let mut parts = raw.splitn(3, '|');
    let pid = parts.next()?.trim().parse::<u32>().ok()?;
    let cwd = parts.next().unwrap_or("").trim().to_string();
    Some((pid, cwd))
}

/// 持锁进程存活核验（PLAN-726 方案：Windows OpenProcess / Unix kill -0）。
#[cfg(windows)]
fn process_alive(pid: u32) -> bool {
    // 裸 FFI：windows crate 为 optional（taa 档不编译），kernel32 恒链接。
    // 签名（isize 句柄）与 ui/desktop_protocol/stage3.rs mem_ffi 保持一致
    // ——同 crate 内同名 extern 声明签名必须相同，否则编译器告警。
    extern "system" {
        fn OpenProcess(dwdesiredaccess: u32, binherithandle: i32, dwprocessid: u32) -> isize;
        fn GetExitCodeProcess(hprocess: isize, lpexitcode: *mut u32) -> i32;
        fn CloseHandle(hobject: isize) -> i32;
    }
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const STILL_ACTIVE: u32 = 259;
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h == 0 {
            return false; // 进程不存在（同机同用户形态下拒绝访问几乎不发生）
        }
        let mut code: u32 = 0;
        let ok = GetExitCodeProcess(h, &mut code);
        CloseHandle(h);
        ok != 0 && code == STILL_ACTIVE
    }
}

#[cfg(not(windows))]
fn process_alive(pid: u32) -> bool {
    // 无 libc 直接依赖，等价 kill(pid,0) 经 coreutils：exit 0 = 存活。
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod machine_gate_tests {
    use super::*;

    // 三态验证全走注入目录（独立 temp），不触机器锚 D:/autostack/.locks，
    // 不依赖 env、不与并行测试共享任何状态。
    fn fresh_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "heavy-machine-gate-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn spawn_sleeper() -> std::process::Child {
        use std::process::{Command, Stdio};
        if cfg!(windows) {
            // ping 6 次自环 ≈ 5s：Windows 无裸 sleep 的标准等价物。
            Command::new("ping")
                .args(["-n", "6", "127.0.0.1"])
                .stdout(Stdio::null())
                .spawn()
                .unwrap()
        } else {
            Command::new("sleep").arg("5").spawn().unwrap()
        }
    }

    /// 隔离态：获取→持锁在盘→drop 释放→立即再取（顺序交接）。
    #[test]
    fn machine_gate_acquire_release_roundtrip() {
        let dir = fresh_dir("roundtrip");
        {
            let _held = machine_gate_in(&dir, "unit-xl");
            assert!(dir.join("unit-xl.lock").exists(), "guard 持有期锁必须在盘");
        }
        assert!(
            !dir.join("unit-xl.lock").exists(),
            "drop 后锁必须释放（陈锁会阻塞同族后续测试）"
        );
        let _g2 = machine_gate_in(&dir, "unit-xl");
    }

    /// 冲突态：活持锁者 → panic 且信息含 already running（PID/cwd）。
    /// catch_unwind 包裹以在 panic 后收割 sleeper 子进程（否则 nextest 标
    /// LEAK，污染泄漏面观测），再 resume_unwind 保住原 panic 载荷。
    #[test]
    #[should_panic(expected = "already running")]
    fn machine_gate_conflict_with_live_holder_panics() {
        let dir = fresh_dir("conflict");
        let mut child = spawn_sleeper();
        std::fs::write(
            dir.join("live.lock"),
            format!("{}|/fake/cwd|live", child.id()),
        )
        .unwrap();
        let outcome = std::panic::catch_unwind(|| machine_gate_in(&dir, "live"));
        let _ = child.kill();
        let _ = child.wait();
        match outcome {
            Err(payload) => std::panic::resume_unwind(payload),
            Ok(_) => panic!("expected 'already running' panic but lock was acquired"),
        }
    }

    /// 陈锁态：持锁进程已亡 → 直接接管，锁内容重写为本进程。
    #[test]
    fn machine_gate_stale_lock_taken_over() {
        let dir = fresh_dir("stale");
        let mut exit_now = if cfg!(windows) {
            std::process::Command::new("cmd").args(["/C", "exit", "0"]).spawn().unwrap()
        } else {
            std::process::Command::new("true").spawn().unwrap()
        };
        let pid = exit_now.id();
        let _ = exit_now.wait(); // 确保持锁者已退出
        std::fs::write(dir.join("stale.lock"), format!("{pid}|/gone/cwd|stale")).unwrap();
        let _g = machine_gate_in(&dir, "stale");
        let raw = std::fs::read_to_string(dir.join("stale.lock")).unwrap();
        assert!(
            raw.starts_with(&format!("{}|", std::process::id())),
            "接管后锁必须重写为本进程 pid（实际：{raw}）"
        );
    }

    /// 同进程重入：协议违规红（裸 libtest 单进程同族两测并发形态）。
    #[test]
    #[should_panic(expected = "re-entered")]
    fn machine_gate_same_process_reentry_panics() {
        let dir = fresh_dir("reentry");
        let _g = machine_gate_in(&dir, "re");
        let _ = machine_gate_in(&dir, "re");
    }
}
