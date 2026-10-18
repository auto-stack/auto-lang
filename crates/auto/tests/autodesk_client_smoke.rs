// Plan 480 S2 —— 双模 exe 两进程 smoke：spawn 真实 `auto` 二进制
// （`run --autodesk-client=<pipe> --app386=<name>`）+ 测试侧 host。
//
// 父测试 = 桌面侧：真实 462 会话 + ProtocolHost 泵；子进程 = `auto`
// 二进制走 cmd_autodesk 入口裁决 → 装载临时目录最小 App → client_runtime
// 主循环。断言：孵化落地（462 AppSession + 虚拟窗）、共享内存帧随协议
// 点击递增、L2Detach 后子进程以 exit=L2Detached 标记退出（stdout）。

use auto_lang::ui::desktop_protocol::host::ProtocolHost;
use auto_lang::ui::desktop_protocol::message::{DrawOp, MouseButton};
use auto_lang::ui::desktop_protocol::transport::{self, Transport};
use auto_lang::ui::session::{AppId, DesktopSession};

const APP_NAME: &str = "smoke-counter";
const COUNTER_SRC: &str = "widget SmokeCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n        text `count: ${.count}`\n    }\n}\n";

#[test]
fn autodesk_client_two_process_smoke() {
    // ---- App 材料（临时目录注入，避免依赖仓内 examples 编译面）----
    let app_root = std::env::temp_dir().join(format!("auto386-smoke-{}", std::process::id()));
    let app_dir = app_root.join(APP_NAME).join("src").join("front");
    std::fs::create_dir_all(&app_dir).expect("mkdir");
    std::fs::write(app_dir.join("app.at"), COUNTER_SRC).expect("write app.at");

    // ---- 桌面侧：per-app 管道先 listen（spawn 注入①直连）----
    let pipe = format!("autodesk-app-smoke-{}", std::process::id());
    let listener = transport::listen(&pipe).expect("listen");

    // ---- spawn 真实 auto 二进制（client 形态）----
    let exe = env!("CARGO_BIN_EXE_auto");
    let mut child = std::process::Command::new(exe)
        .args([
            "run",
            &format!("--autodesk-client={pipe}"),
            &format!("--app386={APP_NAME}"),
        ])
        .env("AUTO_386_APP_ROOT", &app_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("spawn auto --autodesk-client");
    let mut server_end = listener.wait_connect().expect("server connect");

    // ---- 测试侧 host：真实 462 会话 + ProtocolHost 泵 ----
    let mut session = DesktopSession::__test_session();
    session.__test_open_desktop();
    let src = COUNTER_SRC;
    let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
        if name == APP_NAME {
            auto_lang::build_dynamic_component(src, None).map_err(|e| format!("{e}"))
        } else {
            Err(format!("unknown app {name}"))
        }
    });

    fn pump(server_end: &mut Box<dyn Transport + Send>, ph: &mut ProtocolHost<'_>) {
        if server_end.is_eof() {
            eprintln!("[host-dbg] server_end EOF");
        }
        while let Some(loaded) = server_end.try_recv() {
            let msg = loaded.expect("解码");
            ph.handle(&msg).expect("host 状态机");
            for reply in std::mem::take(&mut ph.to_app) {
                let _ = server_end.send(&reply);
            }
        }
    }

    // ---- 泵到 Active + 首帧 ----
    let mut wid = None;
    for _ in 0..300 {
        pump(&mut server_end, &mut ph);
        if !ph.session.apps.is_empty() {
            wid = ph.active().1;
            if ph.composed(wid.expect("wid").0).is_some() {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let wid = wid.expect("真实 auto 二进制已孵化");
    assert!(ph.session.apps.contains_key(&AppId(1)), "462 AppSession 落地");

    // ---- 协议点击 ×2 → 帧递增 ----
    for expect in 1..=2i64 {
        let injected = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected).unwrap();
        let mut seen = false;
        for _ in 0..300 {
            pump(&mut server_end, &mut ph);
            if let Some(list) = ph.composed(wid.0) {
                if list.ops.iter().any(|op| matches!(op,
                    DrawOp::Text { text, .. } if text == &format!("count: {expect}")))
                {
                    seen = true;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(seen, "帧 count 递增到 {expect}");
    }

    // ---- L2Detach → 子进程 exit=L2Detached 收尾 ----
    let detach = ph.endpoint.l2_detach().expect("Active 才可 l2_detach");
    server_end.send(&detach).unwrap();
    let start = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            break status;
        }
        assert!(start.elapsed() < std::time::Duration::from_secs(30), "子进程 30s 未退出");
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    assert!(status.success(), "子进程退出码 {status}");
    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        use std::io::Read as _;
        let _ = out.read_to_string(&mut stdout);
    }
    assert!(
        stdout.contains("exit=L2Detached revision=3"),
        "client 出口标记（2 次点击 rev=1+2）: {stdout:?}"
    );

    // 宿主侧回收收敛。
    for _ in 0..200 {
        pump(&mut server_end, &mut ph);
        if ph.session.apps.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(ph.session.apps.is_empty(), "L2Detached 后宿主回收");

    let _ = std::fs::remove_dir_all(&app_root);
}
