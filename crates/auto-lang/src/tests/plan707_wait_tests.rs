//! PLAN-707 T-05：generator/段等待凭据化（D-6）。
//!
//! AC-02 判据：gate 关闭期间驱动次数不随等待时长增长（旧形态 yield_now
//! 自旋 → spawn id 每 200ms 窗口数千增长；凭据化等待后静态）；链式
//! future await 在 generator 体内停步恢复。

use crate::vm::ffi::http_stream as hs;
use crate::vm::task::AutoTask;

/// 起 SSE 上游：headers + 首事件立即发送，随后**握住**直到放行栅栏。
/// 返回 (port, open_tx, done_rx)——open_tx 发送后补发第二事件。
fn serve_gated_sse() -> (
    u16,
    std::sync::mpsc::Sender<()>,
    std::sync::mpsc::Receiver<()>,
) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let (open_tx, open_rx) = std::sync::mpsc::channel::<()>();
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        use std::io::{Read, Write};
        let _ = stream.set_nodelay(true);
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
        );
        let _ = stream.write_all(b"data: first\n\n");
        let _ = stream.flush();
        // gate：收到放行信号后补发第二事件（OS channel 阻塞读）。
        let _ = open_rx.recv();
        let _ = stream.write_all(b"data: second\n\n");
        let _ = stream.flush();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = done_tx.send(());
        std::thread::sleep(std::time::Duration::from_millis(100));
    });
    (port, open_tx, done_rx)
}

/// 编译 generator relay 并推进到首个产出（驱动一次 pull）。
/// 返回 (vm, iterator_id)。
fn make_relay_generator(port: u16) -> (std::rc::Rc<crate::vm::engine::AutoVM>, u32) {
    let code = format!(
        r#"
fn relay() ~Iter<str> {{
    for e in http.sse_get_stream("http://127.0.0.1:{port}/sse") {{
        yield e
    }}
}}
"#
    );
    let (vm, _out, _entry, _obj) = crate::create_vm_from_source(&code).expect("compile");
    let vm = std::rc::Rc::new(vm);
    let mut task = AutoTask::new(0, 65536, 0);
    match vm.call_fn_by_name_segment(&mut task, "relay", 0) {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("generator dispatch 应 Completed(Ok)，得到 {other:?}"),
    }
    let iter_id = auto_val::decode_i32(task.ram.pop_nv()) as u32;
    (vm, iter_id)
}

/// AC-02：gate 关闭期间驱动次数不随等待时长增长（凭据化等待 vs 自旋）。
#[tokio::test(flavor = "current_thread")]
async fn plan707_wait_generator_park_drive_count_static() {
    let (port, open_tx, _done_rx) = serve_gated_sse();
    let (vm, iter_id) = make_relay_generator(port);

    // 首个值：数据已在途（headers+首事件已发送）→ 快速返回。
    let first = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        crate::vm::ffi::http_server::next_sse_generator_value_for_test(&vm, iter_id),
    )
    .await
    .expect("首值 5s 内应产出");
    let first = first.expect("首值存在");
    assert!(
        crate::vm::ffi::http_server::sse_value_is(&vm, first, "first"),
        "首值应为 first"
    );

    // 第二次 pull：gate 关闭 → 凭据化 park。采样两个 200ms 窗口的
    // spawn 计数（vm.id_gen）——自旋形态每窗口数千，凭据化 ≈ 0。
    let wait_fut = crate::vm::ffi::http_server::next_sse_generator_value_for_test(&vm, iter_id);
    tokio::pin!(wait_fut);
    let mut spawns_total = 0u64;
    for _ in 0..3 {
        let before = vm.id_gen.load(std::sync::atomic::Ordering::Relaxed);
        let r = tokio::time::timeout(std::time::Duration::from_millis(200), &mut wait_fut).await;
        assert!(r.is_err(), "gate 关闭期间 pull 不应返回");
        let after = vm.id_gen.load(std::sync::atomic::Ordering::Relaxed);
        spawns_total += after - before;
    }
    assert!(
        spawns_total < 30,
        "RED：gate 关闭 600ms 内驱动 {spawns_total} 次——等待未凭据化（自旋；         自旋形态每 200ms 窗口数千次）。timeout 轮询本身每次驱动一轮，         3 窗口基线个位数，负载抖动容忍到 30"
    );

    // 放行：第二事件在途 → park 的 pull 被就绪通知唤醒并取值。
    let _ = open_tx.send(());
    let second = tokio::time::timeout(std::time::Duration::from_secs(5), &mut wait_fut)
        .await
        .expect("放行后 5s 内应产出");
    let second = second.expect("第二值存在");
    assert!(
        crate::vm::ffi::http_server::sse_value_is(&vm, second, "second"),
        "第二值应为 second"
    );

    // 收尾：连接关闭 → 流终结（资源观测由 T-07 收口）。
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        crate::vm::ffi::http_server::next_sse_generator_value_for_test(&vm, iter_id),
    )
    .await;
    let _ = hs::stream_live_count();
}
