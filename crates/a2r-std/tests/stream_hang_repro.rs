//! E-STREAM-HANG 复现矩阵（PLAN-099 引擎根修前置定谳）。
//!
//! daemon 流三种收尾形态下的 `post_stream_with_headers` 消费循环终结性：
//!   1. `silent_hold`  —— 发若干 SSE 块后**静默持连**（不关不终结）——
//!      preview-live-02/099-e2e 实机悬挂的候选形态 A；
//!   2. `abrupt_close` —— 发块后直接断 TCP（无 chunked 终结符）——候选形态 B；
//!   3. `proper_end`   —— 标准 chunked 终结（基线，应恒过）。
//!
//! 每形状带独立看门狗线程：消费循环超时未终结 = 复现悬挂（FAIL）。
//! 短 idle timeout 经 env 在内核首用前注入（本文件独立测试二进制，
//! `--test-threads=1` 串行）。
//!
//! 运行：`cargo test -p a2r-std --test stream_hang_repro -- --test-threads=1`

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// 内核全局单例首用前生效（独立测试二进制保证先于任何 http 调用）。
fn init_short_idle_timeout() {
    std::env::set_var("AUTO_A2R_HTTP_STREAM_IDLE_TIMEOUT_MS", "2000");
    std::env::set_var("AUTO_A2R_HTTP_STREAM_OPEN_TIMEOUT_MS", "5000");
}

/// SSE 流 stub：发 N 个 thinking 块后按 shape 收尾。
fn spawn_stream_stub(shape: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let Ok((stream, _)) = listener.accept() else {
            return;
        };
        serve_stream(stream, shape);
    });
    port
}

fn serve_stream(mut stream: TcpStream, shape: &'static str) {
    // 读掉请求头（到空行），再按 Content-Length 精确读 body。
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });
    let mut content_length = 0usize;
    let mut line = String::new();
    loop {
        match reader.read_line(&mut line) {
            Ok(0) => return,
            Ok(_) if line.trim().is_empty() => break,
            Ok(_) => {
                if let Some(v) = line
                    .to_ascii_lowercase()
                    .strip_prefix("content-length:")
                {
                    content_length = v.trim().parse().unwrap_or(0);
                }
            }
            Err(_) => return,
        }
        line.clear();
    }
    if content_length > 0 {
        let mut body = vec![0u8; content_length];
        use std::io::Read;
        let _ = reader.read_exact(&mut body);
    }
    let _ = stream.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n",
    );
    let _ = stream.flush();
    // 5 个 SSE thinking 块（模拟 daemon 正常流出）。
    for i in 0..5 {
        let ev = format!("data: {{\"type\":\"thinking\",\"thinking\":\"t{i}\"}}\n\n");
        let payload = ev.as_bytes();
        let _ = stream.write_all(format!("{:x}\r\n", payload.len()).as_bytes());
        let _ = stream.write_all(payload);
        let _ = stream.write_all(b"\r\n");
        let _ = stream.flush();
        std::thread::sleep(Duration::from_millis(30));
    }
    match shape {
        // A：静默持连——不发终结符不关（模拟 daemon 响应体不终止）。
        "silent_hold" => {
            std::thread::sleep(Duration::from_secs(30));
        }
        // B：直接断 TCP（无 chunked 终结符）。
        "abrupt_close" => {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        // 基线：标准 chunked 终结 + 关闭。
        _ => {
            let _ = stream.write_all(b"0\r\n\r\n");
            let _ = stream.flush();
        }
    }
}

/// 消费循环（复刻 auto-ai-client complete_stream 的 while is_done 形态，
/// 走 facade `post_stream_with_headers` 同一表面）。返回 (chunks, terminal_kind,
/// elapsed)。独立看门狗：超时 panic 定罪。
fn consume_like_client(url: String) -> (usize, String, Duration) {
    let t0 = Instant::now();
    let chunks = Arc::new(AtomicUsize::new(0));
    let terminal = Arc::new(std::sync::Mutex::<Option<String>>::new(None));
    let stream = a2r_std::http::post_stream_with_headers(
        &url,
        "{}",
        "Content-Type: application/json",
    );
    let watchdog = {
        let chunks = chunks.clone();
        let terminal = terminal.clone();
        std::thread::spawn(move || {
            for i in 0..120 {
                std::thread::sleep(Duration::from_secs(1));
                if terminal.lock().unwrap().is_some() {
                    return;
                }
                if i == 10 {
                    panic!(
                        "E-STREAM-HANG 复现：消费循环 10s 未终结（chunks={}）——内核流悬挂实锤",
                        chunks.load(Ordering::SeqCst)
                    );
                }
            }
        })
    };
    let mut got = 0usize;
    let mut kind = "none".to_string();
    while stream.is_done() == 0 {
        let chunk = stream.next();
        if !chunk.is_empty() {
            got += 1;
            chunks.store(got, Ordering::SeqCst);
        }
    }
    kind = match stream.terminal_error() {
        Some(None) => "eof".to_string(),
        Some(Some(e)) => format!("failed:{e}"),
        None => "none".to_string(),
    };
    terminal.lock().unwrap().replace(kind.clone());
    let _ = watchdog.join();
    (got, kind, t0.elapsed())
}

#[test]
fn hang_repro_silent_hold_recovers_by_idle_timeout() {
    init_short_idle_timeout();
    let port = spawn_stream_stub("silent_hold");
    let (chunks, kind, elapsed) = consume_like_client(format!("http://127.0.0.1:{port}/v1/chat"));
    println!(
        "[repro-silent-hold] chunks={chunks} kind={kind} elapsed={elapsed:?}"
    );
    assert_eq!(chunks, 5, "5 个数据块应全部送达");
    assert!(
        kind.starts_with("failed:stream idle timeout"),
        "静默持连应由 idle timeout 终结（实测 kind={kind}）"
    );
    assert!(
        elapsed < Duration::from_secs(15),
        "应在 idle 窗附近终结（实测 {elapsed:?}）"
    );
}

#[test]
fn hang_repro_abrupt_close_recovers() {
    init_short_idle_timeout();
    let port = spawn_stream_stub("abrupt_close");
    let (chunks, kind, elapsed) = consume_like_client(format!("http://127.0.0.1:{port}/v1/chat"));
    println!("[repro-abrupt-close] chunks={chunks} kind={kind} elapsed={elapsed:?}");
    assert!(chunks >= 5, "数据块应送达（实测 {chunks}）");
    assert!(
        elapsed < Duration::from_secs(10),
        "断连应快速终结（实测 {elapsed:?}）"
    );
}

#[test]
fn hang_repro_proper_end_baseline() {
    init_short_idle_timeout();
    let port = spawn_stream_stub("proper_end");
    let (chunks, kind, elapsed) = consume_like_client(format!("http://127.0.0.1:{port}/v1/chat"));
    println!("[repro-proper-end] chunks={chunks} kind={kind} elapsed={elapsed:?}");
    assert_eq!(chunks, 5);
    assert_eq!(kind, "eof");
    assert!(elapsed < Duration::from_secs(10));
}
