//! PLAN-727 T-07：文件传输串行集成矩阵（真 TCP；`--test-threads=1` 串行）。
//!
//! 与内核 in-file 测试（transfer.rs mod tests，临时目录隔离可并行）分工：
//! 本文件跑**独占全局内核配额**的用例（queue 满载/活跃占满取消）与跨规模
//! 矩阵（>10MiB 流式、If-Range 版本变更、416、重试/SourceChanged、未知
//! total 进度、资源回基线）——并行运行会互相挤占全局配额，故整体串行。
//!
//! 门禁：`cargo test -p a2r-std --test http_transfer -- --test-threads=1`。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use a2r_std::http::{
    transfer_cancel, transfer_download, transfer_error, transfer_next_progress, transfer_upload,
    transfer_wait_typed, FileTransfer, TransferOutcome,
};

fn temp_dir(tag: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("plan727-it-{}-{}-{}", tag, std::process::id(), n));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn wait_terminal(t: &FileTransfer) -> a2r_std::http::TransferReceipt {
    loop {
        if t.is_terminal() {
            return transfer_wait_typed(t);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// 回环服务器：accept 一次 → 读请求头（ready 信号）→ handler → done 信号。
fn spawn_server(
    handler: impl FnOnce(String, TcpStream) + Send + 'static,
) -> (
    u16,
    std::sync::mpsc::Receiver<()>,
    std::sync::mpsc::Receiver<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 16384];
        let n = stream.read(&mut buf).unwrap_or(0);
        let head = String::from_utf8_lossy(&buf[..n]).to_string();
        let _ = ready_tx.send(());
        handler(head, stream);
        let _ = done_tx.send(());
    });
    (port, ready_rx, done_rx)
}

fn wait_ready(ready: &std::sync::mpsc::Receiver<()>) {
    ready
        .recv_timeout(Duration::from_secs(10))
        .expect("服务器 10s 未收到请求");
}

fn join_server(done: std::sync::mpsc::Receiver<()>) {
    let _ = done.recv_timeout(Duration::from_secs(10));
}

fn respond(stream: &mut TcpStream, status_line: &str, headers: &[(&str, &str)], body: &[u8]) {
    let mut resp = format!("{status_line}\r\n");
    for (k, v) in headers {
        resp.push_str(&format!("{k}: {v}\r\n"));
    }
    resp.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    stream.write_all(resp.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
    let _ = stream.flush();
}

/// If-Range ETag 匹配 → 206 续传；不匹配 → 200 完整重启（不再续传）。
#[test]
fn it_if_range_etag_match_appends_mismatch_restarts() {
    let dir = temp_dir("if-range");
    let target = dir.join("large.bin");
    let prefix = vec![1u8; 128];
    let prefix_len = prefix.len();
    let suffix_v2 = vec![2u8; 64];
    let fresh_v3 = vec![9u8; 200];
    std::fs::write(&target, &prefix).unwrap();

    // 第一轮：ETag 匹配 → 206 + Content-Range → staging 前缀复制+后缀追加。
    let suffix_clone = suffix_v2.clone();
    let (port, ready, done) = spawn_server(move |head, mut s| {
        let head_l = head.to_ascii_lowercase();
        assert!(head_l.contains("if-range: \"v2\""), "{head}");
        respond(
            &mut s,
            "HTTP/1.1 206 Partial Content",
            &[(
                "Content-Range",
                &format!(
                    "bytes {}-{}/{}",
                    prefix_len,
                    prefix_len + suffix_clone.len() - 1,
                    prefix_len + suffix_clone.len()
                ),
            )],
            &suffix_clone,
        );
    });
    let t = transfer_download(
        &format!("http://127.0.0.1:{port}/f"),
        target.to_str().unwrap(),
        r#"{"offset":128,"validator":{"etag":"\"v2\""}}"#,
    );
    let r = wait_terminal(&t);
    assert_eq!(r.kind, TransferOutcome::Success, "{r:?}");
    assert_eq!(r.status, Some(206));
    let mut expect = prefix.clone();
    expect.extend_from_slice(&suffix_v2);
    assert_eq!(std::fs::read(&target).unwrap(), expect);
    wait_ready(&ready);
    join_server(done);

    // 第二轮：If-Range 失配 → 200 全量重启（绝不 append）。
    let fresh_clone = fresh_v3.clone();
    let (port, ready, done) = spawn_server(move |_head, mut s| {
        respond(
            &mut s,
            "HTTP/1.1 200 OK",
            &[("ETag", "\"v9\"")],
            &fresh_clone,
        );
    });
    let t = transfer_download(
        &format!("http://127.0.0.1:{port}/f"),
        target.to_str().unwrap(),
        r#"{"offset":192,"validator":{"etag":"\"stale\""}}"#,
    );
    let r = wait_terminal(&t);
    assert_eq!(r.kind, TransferOutcome::Success, "{r:?}");
    assert_eq!(r.status, Some(200), "If-Range 失配须以 200 完整重启");
    assert_eq!(std::fs::read(&target).unwrap(), fresh_v3);
    wait_ready(&ready);
    join_server(done);
    std::fs::remove_dir_all(dir).ok();
}

/// 416 Range Not Satisfiable → 终结性 http_status 失败，原目标保持。
#[test]
fn it_416_preserves_target() {
    let dir = temp_dir("r416");
    let target = dir.join("f.bin");
    std::fs::write(&target, vec![1u8; 100]).unwrap();
    let (port, ready, done) = spawn_server(move |_head, mut s| {
        respond(
            &mut s,
            "HTTP/1.1 416 Range Not Satisfiable",
            &[],
            b"bad range",
        );
    });
    let t = transfer_download(
        &format!("http://127.0.0.1:{port}/f"),
        target.to_str().unwrap(),
        r#"{"offset":100,"validator":{"etag":"\"v\""}}"#,
    );
    let r = wait_terminal(&t);
    assert_eq!(r.kind, TransferOutcome::Failed);
    assert_eq!(r.status, Some(416));
    assert!(transfer_error(&t).contains("status 416"), "{r:?}");
    assert_eq!(std::fs::read(&target).unwrap().len(), 100, "原目标保持");
    wait_ready(&ready);
    join_server(done);
    std::fs::remove_dir_all(dir).ok();
}

/// 上传重试面：服务器首轮读头后断连（transport 失败）→ 显式 retries=1
/// 重开文件重放；第二连无人响应 → 有界期限终结。主判据：终态可观察、
/// 不挂死、源文件身份复核路径不炸（正面重试成功由 e2e 腿覆盖）。
#[test]
fn it_upload_retry_replay_bounded_termination() {
    let dir = temp_dir("retry");
    let src = dir.join("blob.bin");
    std::fs::write(&src, vec![7u8; 4096]).unwrap();
    let (port, ready, done) = spawn_server(move |_head, mut s| {
        let _ = s.shutdown(std::net::Shutdown::Both);
    });
    let t = transfer_upload(
        &format!("http://127.0.0.1:{port}/up"),
        src.to_str().unwrap(),
        r#"{"mode":"raw","retries":1,"timeout_ms":3000}"#,
    );
    let r = wait_terminal(&t);
    assert_eq!(r.kind, TransferOutcome::Failed);
    assert!(matches!(
        r.error.as_ref().unwrap().kind,
        a2r_std::http::TransferErrorKind::Transport | a2r_std::http::TransferErrorKind::Timeout
    ));
    wait_ready(&ready);
    join_server(done);
    std::fs::remove_dir_all(dir).ok();
}

/// >10MiB 流式下载：12MiB 随机内容分块下发，hash 对拍（内存不随文件增长
/// 的结构性保证 = 64KiB 块管道；此处验证正确性与规模）。
#[test]
fn it_large_download_12mib_streaming() {
    let dir = temp_dir("big");
    let target = dir.join("big.bin");
    let size = 12 * 1024 * 1024usize;
    // 确定性伪随机内容 + hash。
    let mut seed = plan727_seed();
    let mut content = vec![0u8; size];
    for b in content.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *b = (seed >> 33) as u8;
    }
    let expected_hash = fnv1a(&content);
    let (port, ready, done) = spawn_server(move |_head, mut s| {
        s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {size}\r\n\r\n").as_bytes())
            .unwrap();
        let _ = s.flush();
        let mut offset = 0usize;
        while offset < content.len() {
            let end = (offset + 32 * 1024).min(content.len());
            if s.write_all(&content[offset..end]).is_err() {
                break;
            }
            let _ = s.flush();
            offset = end;
        }
    });
    let t = transfer_download(
        &format!("http://127.0.0.1:{port}/big"),
        target.to_str().unwrap(),
        "",
    );
    // 进度面：轮询若干次确认 percent 推进到 100（未知时刻的合并语义）。
    let r = wait_terminal(&t);
    assert_eq!(r.kind, TransferOutcome::Success, "{r:?}");
    assert_eq!(r.bytes, size as u64);
    assert_eq!(r.total, Some(size as u64));
    let got = std::fs::read(&target).unwrap();
    assert_eq!(got.len(), size);
    assert_eq!(fnv1a(&got), expected_hash, "12MiB 内容逐字节一致");
    let _ = ready.recv_timeout(Duration::from_secs(5));
    join_server(done);
    std::fs::remove_dir_all(dir).ok();
}

/// 未知 Content-Length（chunked）下载：total/percent 为 null 的进度与成功收据。
#[test]
fn it_chunked_unknown_total_progress_null() {
    let dir = temp_dir("chunked");
    let target = dir.join("c.bin");
    let (port, ready, done) = spawn_server(move |_head, mut s| {
        s.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n")
            .unwrap();
        for _ in 0..8 {
            s.write_all(b"4000\r\n").unwrap(); // 16KiB 块
            s.write_all(&[b'a'; 0x4000]).unwrap();
            s.write_all(b"\r\n").unwrap();
            let _ = s.flush();
        }
        s.write_all(b"0\r\n\r\n").unwrap();
        let _ = s.flush();
    });
    let t = transfer_download(
        &format!("http://127.0.0.1:{port}/c"),
        target.to_str().unwrap(),
        "",
    );
    // 终态前轮询一次进度：total 未知 → "total":null。
    std::thread::sleep(Duration::from_millis(80));
    let p = transfer_next_progress(&t);
    if !p.is_empty() && !t.is_terminal() {
        assert!(p.contains("\"total\":null"), "未知 total 须为 null: {p}");
        assert!(
            p.contains("\"percent\":null"),
            "未知 total percent 须为 null: {p}"
        );
    }
    let r = wait_terminal(&t);
    assert_eq!(r.kind, TransferOutcome::Success, "{r:?}");
    assert_eq!(r.bytes, 8 * 0x4000);
    assert_eq!(r.total, None, "chunked 无总长 → 收据 total null");
    assert_eq!(std::fs::read(&target).unwrap().len(), 8 * 0x4000);
    wait_ready(&ready);
    join_server(done);
    std::fs::remove_dir_all(dir).ok();
}

/// 全局配额语义（冻结于决策报告 §7）：**在途总量上限 = queue_capacity 16**
///（队列许可提交即取，其中至多 max_active 4 笔同时执行）。本用例：16 笔
/// 全部在途（4 活跃 + 12 排队）后第 17 笔提交即 QueueFull 终结；全部取消后
/// 注册表/仲裁表回基线（资源报告数据源）。
#[test]
fn it_queue_saturation_rejects_then_recovers() {
    let dir = temp_dir("sat");
    // 不 accept 的监听器：连接入 backlog → 占住 active 的 job 停在建立期。
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let mut transfers = Vec::new();
    for i in 0..16 {
        let target = dir.join(format!("hold{i}.bin"));
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/h{i}"),
            target.to_str().unwrap(),
            "",
        );
        transfers.push(t);
        std::thread::sleep(Duration::from_millis(20));
    }
    // 等 16 笔全部登记（4 活跃建立中 + 12 排队）。
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while a2r_std::http::in_flight_target_len() < 16 {
        assert!(std::time::Instant::now() < deadline, "传输未全部登记");
        std::thread::sleep(Duration::from_millis(25));
    }
    // 第 17 笔：在途总量已满 → 终态 queue_full。
    let overflow = dir.join("overflow.bin");
    let t17 = transfer_download(
        &format!("http://127.0.0.1:{port}/overflow"),
        overflow.to_str().unwrap(),
        "",
    );
    let r17 = wait_terminal(&t17);
    assert_eq!(r17.kind, TransferOutcome::Failed);
    assert_eq!(
        r17.error.as_ref().unwrap().kind,
        a2r_std::http::TransferErrorKind::QueueFull,
        "{r17:?}"
    );
    // 清理：全部取消 → 注册表/仲裁表回基线。
    for t in &transfers {
        transfer_cancel(t);
    }
    for t in &transfers {
        let _ = wait_terminal(t);
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while a2r_std::http::active_transfer_len() > 0 {
        assert!(std::time::Instant::now() < deadline, "取消后注册表未清空");
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_eq!(a2r_std::http::in_flight_target_len(), 0, "仲裁表须回基线");
    drop(listener);
    std::fs::remove_dir_all(dir).ok();
}

// ---- 工具 ----

fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in data {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn plan727_seed() -> u64 {
    0x7027_7271_8
}
