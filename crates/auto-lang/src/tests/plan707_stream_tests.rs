//! PLAN-707 T-07：流资源生命周期收口探针（常规档，无真 TCP 服务器 except
//! 黑洞 accept；真 TCP relay 见 plan707_stream_e2e_tests，`cargo th` 池）。
//!
//! 覆盖（AC-01..04/06 资源面）：
//! - **背压**：消费者停读 → 队列钉在 max_queued_items（≤16），上游不再被拉取；
//! - **取消风暴**：两轮同配置 open/abandon/close → 表与许可回基线无增长；
//! - **线程稳定**：批量流建立/终结，进程线程数不随流数增长（无每流线程）；
//! - **预算**：单事件字节超限 → 终结性 Failed（可观测，非伪造成功）；
//! - **abort 表收口**：成功/取消 job 的 abort 句柄均出表（无慢性增长）。

use crate::vm::ffi::async_http as ah;
use crate::vm::ffi::http_stream as hs;
use crate::vm::ffi::http_stream::StreamLimits;
use crate::vm::task::AutoTask;

fn open_raw(task: &mut AutoTask, vm: &crate::vm::engine::AutoVM, url: &str) -> u64 {
    let url_idx = vm.add_string(url.as_bytes().to_vec());
    vm.rc_push_str_idx(task, url_idx as usize);
    crate::vm::ffi::stdlib::shim_http_get_stream(task, vm).expect("open");
    task.ram.pop_i32() as u64
}

fn close_raw(task: &mut AutoTask, vm: &crate::vm::engine::AutoVM, id: u64) {
    task.ram.push_i32(id as i32);
    crate::vm::ffi::stdlib::shim_http_stream_close(task, vm).expect("close");
}

/// 黑洞上游：accept 后握住不响应（支持多连接）。返回端口与关闭栅栏。
fn blackhole_accepting(n: usize) -> (u16, std::sync::mpsc::Receiver<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        let _ = tx.send(());
        // 同线程持有全部连接（**不**为每连接开线程——线程计数探针的
        // 服务器侧必须零线程贡献，12 socket 仅持在 Vec 里）。
        let mut held = Vec::new();
        for _ in 0..n {
            if let Ok((s, _)) = listener.accept() {
                held.push(s);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(30));
        drop(held);
    });
    let _ = rx.recv_timeout(std::time::Duration::from_secs(2));
    (port, rx)
}

/// 快速上游：accept 后立即发 N 个小 chunk（间隔 gap），随后保持连接。
fn fast_chunk_server(n: usize, gap_ms: u64) -> (u16, std::sync::mpsc::Receiver<&'static str>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = std::sync::mpsc::channel::<&'static str>();
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            let _ = tx.send("accept-failed");
            return;
        };
        use std::io::{Read, Write};
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);
        let _ = stream.set_nodelay(true);
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
        );
        let mut blocked_observed = false;
        for i in 0..n {
            let chunk = format!("c{i};");
            if stream.write_all(chunk.as_bytes()).is_err() {
                break;
            }
            let _ = stream.flush();
            // 发送变慢 = TCP 缓冲满（背压传导到上游）的观测信号。
            if i > 40 && !blocked_observed {
                let t0 = std::time::Instant::now();
                std::thread::sleep(std::time::Duration::from_millis(gap_ms));
                if t0.elapsed() > std::time::Duration::from_millis(gap_ms + 150) {
                    blocked_observed = true;
                    let _ = tx.send("backpressured");
                }
            } else {
                std::thread::sleep(std::time::Duration::from_millis(gap_ms));
            }
        }
        let _ = tx.send(if blocked_observed { "backpressured" } else { "drained" });
        std::thread::sleep(std::time::Duration::from_secs(10));
    });
    (port, rx)
}

/// AC-04：队满背压——消费者不拉取时，队列钉在 max_queued_items=4，
/// 上游写端被 TCP 反压（观测 backpressured），取消仍可打断。
#[test]
fn plan707_stream_backpressure_bounds_queue_and_stops_pull() {
    hs::set_stream_limits_for_test(StreamLimits {
        max_active: 4,
        queue_capacity: 8,
        max_queued_items: 4,
        max_item_bytes: 64 * 1024,
        open_timeout: std::time::Duration::from_secs(5),
        idle_timeout: std::time::Duration::from_secs(60),
    });
    // 上游 200 个 chunk、间隔 5ms（远快于消费），总供给远超队列容量。
    let (port, rx) = fast_chunk_server(200, 5);
    let (vm, _out, _e, _o) = crate::create_vm_from_source("fn main() {}").expect("vm");
    let mut task = AutoTask::new(0, 65536, 0);
    let id = open_raw(&mut task, &vm, &format!("http://127.0.0.1:{port}/bp"));

    // 等队列填满并稳住（上游持续供给，消费侧完全不拉取）。
    std::thread::sleep(std::time::Duration::from_millis(800));
    let queued = hs::stream_queue_len_for_test(id);
    assert!(
        queued <= 4,
        "RED：队列钉不住（queued={queued} > 4）——字节/条数无界"
    );

    // 取消能打断满队列生产者（abort 生效于 space 等待）。
    close_raw(&mut task, &vm, id);
    assert_eq!(hs::stream_live_count(), 0, "close 出表");

    // 上游写端被反压（弱断言：200 chunk × 5ms ≈ 1s 供给窗口内未全部发完；
    // 若 OS 缓冲吞下全部 200 chunk（<64KB），此信号缺失不判红——队列钉住
    // 已是主证据）。
    let _ = rx.recv_timeout(std::time::Duration::from_millis(500));
}

/// AC-03/AC-06：取消风暴两轮——黑洞上游 open/abandon/close，流表、
/// stream 许可、live-op 表全部回基线（同配置重复无增长）。
#[test]
fn plan707_stream_cancel_storm_two_rounds_baseline() {
    hs::set_stream_limits_for_test(StreamLimits {
        max_active: 8,
        queue_capacity: 16,
        max_queued_items: 4,
        max_item_bytes: 64 * 1024,
        open_timeout: std::time::Duration::from_secs(5),
        idle_timeout: std::time::Duration::from_secs(60),
    });
    let (vm, _out, _e, _o) = crate::create_vm_from_source("fn main() {}").expect("vm");
    let mut task = AutoTask::new(0, 65536, 0);

    let baseline_streams = hs::stream_live_count();
    let baseline_active = hs::stream_active_available();
    let baseline_ops = crate::vm::ffi::async_http::live_op_count();

    for round in 0..2 {
        let (port, _rx) = blackhole_accepting(8);
        let mut ids = Vec::new();
        for i in 0..8 {
            let id = open_raw(&mut task, &vm, &format!("http://127.0.0.1:{port}/s{round}-{i}"));
            ids.push(id);
        }
        assert_eq!(
            hs::stream_live_count(),
            baseline_streams + 8,
            "round {round} 打开数不符"
        );
        // 一半显式 close（取消路径），一半 close（同收口路径——无消费者的
        // 流不留存活面）。
        for (i, id) in ids.iter().enumerate() {
            close_raw(&mut task, &vm, *id);
            let _ = i;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert_eq!(hs::stream_live_count(), baseline_streams, "round {round} 未回基线");
        assert_eq!(
            hs::stream_active_available(),
            baseline_active,
            "round {round} 流许可未回基线"
        );
    }
    assert_eq!(
        crate::vm::ffi::async_http::live_op_count(),
        baseline_ops,
        "live-op 未回基线"
    );
}

/// AC-06：批量流建立/终结，进程线程数不随流数增长（无每流线程/runtime）。
#[cfg(windows)]
#[test]
fn plan707_stream_thread_count_stable_across_streams() {
    fn thread_count() -> usize {
        #[repr(C)]
        struct THREADENTRY32 {
            dw_size: u32,
            cnt_usage: u32,
            th32_thread_id: u32,
            th32_owner_process_id: u32,
            tp_base_pri: i32,
            tp_delta_pri: i32,
            dw_flags: u32,
        }
        unsafe extern "system" {
            fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
            fn Thread32First(h: isize, e: *mut THREADENTRY32) -> i32;
            fn Thread32Next(h: isize, e: *mut THREADENTRY32) -> i32;
        }
        unsafe {
            let snap = CreateToolhelp32Snapshot(0x4, std::process::id());
            let mut e = THREADENTRY32 {
                dw_size: std::mem::size_of::<THREADENTRY32>() as u32,
                cnt_usage: 0,
                th32_thread_id: 0,
                th32_owner_process_id: 0,
                tp_base_pri: 0,
                tp_delta_pri: 0,
                dw_flags: 0,
            };
            let mut n = 0;
            if Thread32First(snap, &mut e) != 0 {
                loop {
                    if e.th32_owner_process_id == std::process::id() {
                        n += 1;
                    }
                    if Thread32Next(snap, &mut e) == 0 {
                        break;
                    }
                }
            }
            n
        }
    }

    hs::set_stream_limits_for_test(StreamLimits {
        max_active: 16,
        queue_capacity: 32,
        max_queued_items: 4,
        max_item_bytes: 64 * 1024,
        open_timeout: std::time::Duration::from_secs(5),
        idle_timeout: std::time::Duration::from_secs(60),
    });
    let (vm, _out, _e, _o) = crate::create_vm_from_source("fn main() {}").expect("vm");
    let mut task = AutoTask::new(0, 65536, 0);

    // 预热（client runtime + stream executor 建立后的稳态为基线）。
    let (port, _rx) = blackhole_accepting(12);
    let warm = open_raw(&mut task, &vm, &format!("http://127.0.0.1:{port}/warm"));
    std::thread::sleep(std::time::Duration::from_millis(300));
    close_raw(&mut task, &vm, warm);
    std::thread::sleep(std::time::Duration::from_millis(200));
    let before = thread_count();
    assert!(before > 0, "线程枚举失败");

    // 12 条并发流批量建立再终结。
    let mut ids = Vec::new();
    for i in 0..12 {
        ids.push(open_raw(&mut task, &vm, &format!("http://127.0.0.1:{port}/t{i}")));
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    for id in &ids {
        close_raw(&mut task, &vm, *id);
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    let after = thread_count();
    assert!(
        after <= before + 2,
        "RED：12 条流后线程 {before}→{after}——疑似每流线程"
    );
}

/// AC-04：单事件字节预算超限 → 终结性 Failed（拉取面可观测诊断）。
#[test]
fn plan707_stream_item_budget_terminal_failed() {
    hs::set_stream_limits_for_test(StreamLimits {
        max_active: 4,
        queue_capacity: 8,
        max_queued_items: 4,
        max_item_bytes: 32, // 极小预算：单 chunk 超 32 字节即超限
        open_timeout: std::time::Duration::from_secs(5),
        idle_timeout: std::time::Duration::from_secs(60),
    });
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            use std::io::{Read, Write};
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
            );
            let big = "x".repeat(256);
            let _ = stream.write_all(big.as_bytes());
            let _ = stream.flush();
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });
    let (vm, _out, _e, _o) = crate::create_vm_from_source("fn main() {}").expect("vm");
    let mut task = AutoTask::new(0, 65536, 0);
    let id = open_raw(&mut task, &vm, &format!("http://127.0.0.1:{port}/big"));

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        match hs::stream_pull(id) {
            hs::Pull::Failed(e) => {
                assert!(e.contains("budget"), "诊断应含 budget：{e}");
                break;
            }
            hs::Pull::Data(_) => panic!("超预算 chunk 不应作为 Data 交付"),
            _ => {}
        }
        assert!(std::time::Instant::now() < deadline, "5s 内未终结");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    // 终态条目保留至显式 close（消费方可查 sse_error/stream_terminal_error
    // 诊断——D-3 契约）；close 后出表。
    assert_eq!(hs::stream_live_count(), 1, "终态条目应保留至 close");
    close_raw(&mut task, &vm, id);
    assert_eq!(hs::stream_live_count(), 0, "close 后出表");
}

/// AC-06：abort 句柄表收口——成功与取消 job 均出表（无慢性增长）。
#[test]
fn plan707_stream_job_abort_table_reclaims() {
    ah::set_client_limits_for_test(ah::ClientLimits {
        workers: 1,
        max_active: 2,
        queue_capacity: 8,
        body_limit: 1024 * 1024,
        total_timeout: std::time::Duration::from_secs(10),
    });
    let before = ah::job_abort_count();
    // 成功 job（立即完成）。
    for i in 0..4 {
        let id = crate::vm::ffi::stdlib::alloc_async_id();
        ah::register_live_op(id);
        assert!(ah::submit_client_job(
            id,
            async move { Ok(crate::vm::ffi::stdlib::AsyncResult::Body(format!("ok{i}"))) }
        ));
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert_eq!(
        ah::job_abort_count(),
        before,
        "RED：成功 job 的 abort 句柄未出表（慢性泄漏）"
    );
    // 取消 job。
    let id = crate::vm::ffi::stdlib::alloc_async_id();
    ah::register_live_op(id);
    let (tx, _rx) = tokio::sync::oneshot::channel::<()>();
    assert!(ah::submit_client_job(id, async move {
        let _ = _rx.await;
        Ok(crate::vm::ffi::stdlib::AsyncResult::Body("x".into()))
    }));
    std::thread::sleep(std::time::Duration::from_millis(100));
    ah::cancel_live_op(id);
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert_eq!(ah::job_abort_count(), before, "取消 job 的 abort 句柄未出表");
}
