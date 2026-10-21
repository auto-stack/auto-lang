//! PLAN-712 T-17（r3，端到端红→绿锁）：routes 路由页（pages/*.at）的
//! 模块读点必须过前缀化 overlay。
//!
//! PLAN-814 T-02 迁移上移：本测试消费上层 run 族/动态组件面
//! （crate::build_dynamic_component / DynamicComponent 渲染驱动），原居
//! auto-lang-core back_prefix.rs 的 `ui-iced` 门在 SDK 无定义——迁移后
//! 死门、选择丢失（V-05）；按真实所有者上移本包恢复选择
//! （back_prefix::set_guard/PrefixSpec 经 SDK 再导出）。

use crate::back_prefix::{set_guard, PrefixSpec};

/// PLAN-712 T-17（r3，端到端红→绿锁）：**routes 路由页（pages/*.at）的
/// 模块读点必须过前缀化 overlay**。此前 lib.rs 的路由页装载臂裸读文件
/// （唯一未包 `apply` 的模块读点）——018-book-reader 实机实证：顶层
/// `book_store.at` 前缀生效（书架取数 2 REQ 到 proxy），`pages/
/// book_detail.at` 留相对路径 → VM 解析到 app 自身 back origin（死端口）
/// → `Http.get_json` 吞错返空 → 详情页 0 entries。
///
/// 本测用真实 TCP 监听充当 proxy 根：前缀化生效 ⇒ 页 Init 的 fetch 打到
/// 监听口（路径带 `/apps/{key}/api/` 前缀）⇒ 数据回灌状态；未生效 ⇒
/// 相对路径解析不到本监听口（红相形态：零请求 + 状态滞留初值）。
/// PLAN-814 T-02 迁移勘误：本测试仅依赖 SDK 面（真 TCP + back_prefix），
/// 原 ui-iced 门为旧 crate feature 残留——拆除恢复选择。
#[test]
fn route_page_fetch_reaches_proxied_root_when_guard_active() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    // ① 真 TCP stub：accept 循环，逐连接读请求行、回固定 JSON。
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let seen_paths: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let served = Arc::new(AtomicUsize::new(0));
    {
        let seen_paths = Arc::clone(&seen_paths);
        let served = Arc::clone(&served);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut s = stream;
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                // 请求行形如 `GET /apps/<key>/api/books/1 HTTP/1.1`。
                let head = String::from_utf8_lossy(&buf).to_string();
                if let Some(first) = head.lines().next() {
                    seen_paths.lock().unwrap().push(first.to_string());
                }
                let body = br#"{"title": "The Source"}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = s.write_all(resp.as_bytes());
                let _ = s.write_all(body);
                served.fetch_add(1, Ordering::Relaxed);
            }
        });
    }

    // ② fixture：routes 页 + 页内相对 `/api/` 取数（018 book_detail 同构）。
    let dir = tempfile::tempdir().expect("tempdir");
    let pages = dir.path().join("pages");
    std::fs::create_dir_all(&pages).expect("pages dir");
    std::fs::write(
        dir.path().join("app.at"),
        r#"
widget App {
routes {
    "/" -> use detail
}
view {
    outlet
}
}
"#,
    )
    .expect("write app.at");
    std::fs::write(
        pages.join("detail.at"),
        r#"
widget detail {
msg { Init }

model {
    var title str = "none"
    var ok bool = false
}

view {
    text .title {}
}

on {
    .Init -> {
        try {
            let d = Http.get_json("/api/books/1")
            .title = d.title
            .ok = true
        } catch {
            .ok = false
        }
    }
}
}
"#,
    )
    .expect("write pages/detail.at");

    // ③ guard 激活 + 生产构建路径（路由页装载臂在其内）。
    let app_path = dir.path().join("app.at");
    let code = std::fs::read_to_string(&app_path).expect("read app.at");
    let mut dc = {
        let _g = set_guard(PrefixSpec {
            front_dir: dir.path().to_path_buf(),
            root: format!("http://127.0.0.1:{port}"),
            app_key: "018-book-reader".to_string(),
        });
        crate::build_dynamic_component(&code, app_path.to_str()).expect("routes app 必须能编译")
    };

    // ④ 驱动 Init（页 Init 是 demand 制——**先渲染**：outlet 挂载路由页
    //    时才登记页 Init demand（437 同款），再泵驱动；parked fetch 由
    //    async HTTP 执行器就绪，外层轮询等待）。
    dc.fire_init();
    let _ = dc.view_with_debug_gated(true);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut ok = false;
    while std::time::Instant::now() < deadline {
        dc.drive_scheduler_to_quiescence(10_000);
        let _ = dc.view_with_debug_gated(true);
        if matches!(dc.read_state("ok"), Ok(auto_val::Value::Bool(true))) {
            ok = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    // ⑤ 断言：fetch 到达 stub 且带前缀；数据回灌状态。
    let paths = seen_paths.lock().unwrap().clone();
    eprintln!(
        "T17-DBG ok={ok} title={:?} parked_io={} init_work={} cpu={} served={} paths={:?}",
        dc.read_state("title").ok(),
        dc.has_parked_io_tasks(),
        dc.has_pending_init_work(),
        dc.has_cpu_continuations(),
        served.load(Ordering::Relaxed),
        paths
    );
    assert!(
        ok,
        "页 Init 的取数应成功回灌（前缀化生效 ⇒ 打到本监听口）；\
         stub 收到的请求行：{paths:?}"
    );
    assert!(
        !paths.is_empty(),
        "stub 未收到任何请求——路由页取数未到达 proxy 根（前缀化未覆盖 pages/ 装载臂）"
    );
    assert!(
        paths
            .iter()
            .any(|p| p.contains(&format!("/apps/018-book-reader/api/books/1"))),
        "请求行应带 /apps/<key>/api 前缀，实际：{paths:?}"
    );
    assert_eq!(
        dc.read_state("title").ok(),
        Some(auto_val::Value::Str("The Source".into())),
        "响应体应经 get_json 回灌到页状态"
    );
}
