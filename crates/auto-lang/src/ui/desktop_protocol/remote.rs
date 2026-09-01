// Plan 508 G4 —— 远程会话接纳：宿主 WS 监听（回环 + token）+ broker
// 会话镜像泵。
//
// 拓扑（架构图）：远程浏览器经 `packages/drawlist-renderer` 连入
// `:17800`（token query），镜像**已落地的 outproc queue 臂**——
// - 下行：宿主 composed DrawList 变化 → `FrameReady`（payload 内联，
//   v1.0 纯管道帧形态，无 shm）；订阅兑现时一次性 `Welcome` +
//   `HitTable`（D3 交互区表过线形态）；
// - 上行：`Hello`（订阅 = app 名）+ `InputMsg`（坐标直传——权威命中
//   判定在 app 侧，镜像端命中表只作本地寻址/光标 UX）。
//
// 会话泵复用：输入路由与 `broker_pointer_down` 同一收尾（直投目标
// client 连接）；帧源 = `pump_broker_clients` 的合成产物——五通道
// （握手/帧/输入/控制/观测）多路语义原样，`PROTOCOL_VERSION` 不变
// （HitTable 为追加式变体，旧链路不产不出）。
//
// 安全边界（v1，计划待澄清③）：回环 + 静态 token（storage
// `shell.remote.token`，缺省不监听 = 拒绝一切远程）；跨网/TLS 另立。

use crate::ui::desktop_protocol::message::{
    FrameMsg, HandshakeMsg, HitRegion, ProtocolMsg, HIT_KIND_BUTTON, HIT_KIND_INPUT,
};
use crate::ui::desktop_protocol::transport::ws::WsListener;
use crate::ui::session::{DesktopSession, Wid};

/// 宿主 WS 监听缺省端口（协议文档偏差表登记；Plan 508 G4）。
pub const REMOTE_WS_PORT: u16 = 17800;

/// 一条远程镜像会话（一个已过 token 校验的 WS 对端）。
pub struct RemoteMirror {
    pub(crate) end: Box<dyn crate::ui::desktop_protocol::transport::Transport + Send>,
    /// 订阅目标（Hello.app_name；目标 broker client 尚未落地时每拍重试）。
    app: Option<String>,
    /// 已兑现订阅的窗口（None = 待兑现/目标不在册）。
    wid: Option<Wid>,
    /// 上次已推送帧（变化检测——DrawList 全量相等比较，v1 帧小）。
    last_frame: Option<crate::ui::desktop_protocol::message::DrawList>,
    /// 推送序（frame_id/revision 同源递增）。
    seq: u64,
}

impl DesktopSession {
    /// Plan 508 G4：开远程 WS 监听（回环 + token）。绑定失败不炸桌面
    ///（ui_console + stderr 双落，降级 = 无远程面）。
    pub fn enable_remote_ws(&mut self, token: &str, port: u16) {
        match WsListener::bind(port, token) {
            Ok(listener) => {
                self.desktop.remote_listener = Some(listener);
                let line = format!(
                    "[remote] ws listening on 127.0.0.1:{} (token required)",
                    self.desktop.remote_listener.as_ref().map(|l| l.port()).unwrap_or(port)
                );
                crate::vm::ui_console::ui_console_push(&line);
                eprintln!("{line}");
            }
            Err(e) => {
                let line = format!("[remote] ws listen failed (remote disabled): {e:?}");
                crate::vm::ui_console::ui_console_push(&line);
                eprintln!("{line}");
            }
        }
    }

    /// 远程监听实际端口（bind(0) 测试形态取系统分配值；未监听 None）。
    pub fn remote_ws_port(&self) -> Option<u16> {
        self.desktop.remote_listener.as_ref().map(|l| l.port())
    }

    /// 待接纳远程连接数（ServiceTick 节流位）。
    pub fn pending_remote_connections(&mut self) -> usize {
        match self.desktop.remote_listener.as_mut() {
            Some(listener) => listener.accepted_len(),
            None => 0,
        }
    }

    /// 排空受理队列 → 镜像会话在册（ServiceTick 周期调用）。
    pub fn attach_pending_remotes(&mut self) {
        let Some(listener) = self.desktop.remote_listener.as_mut() else { return };
        while let Some(end) = listener.try_accept() {
            self.desktop.remote_mirrors.push(RemoteMirror {
                end,
                app: None,
                wid: None,
                last_frame: None,
                seq: 0,
            });
        }
    }

    /// 远程镜像泵（ServiceTick 周期；在 `pump_broker_clients` 之后调用
    /// ——帧源 = 合成产物）。断连镜像摘除。
    pub fn pump_remote_mirrors(&mut self) {
        if self.desktop.remote_mirrors.is_empty() {
            return;
        }
        let mut mirrors = std::mem::take(&mut self.desktop.remote_mirrors);
        let mut dead = Vec::new();
        for (idx, mirror) in mirrors.iter_mut().enumerate() {
            // 入站排空（坏载荷丢弃该条，不断链——远程端容错）。
            loop {
                match mirror.end.try_recv() {
                    Some(Ok(msg)) => self.remote_mirror_on_msg(mirror, msg),
                    Some(Err(_)) => continue,
                    None => break,
                }
            }
            if mirror.end.is_eof() {
                dead.push(idx);
                continue;
            }
            // 订阅兑现（目标 client 落地后一次性 Welcome + HitTable）。
            if mirror.app.is_some() && mirror.wid.is_none() {
                Self::remote_mirror_subscribe(self, mirror);
            }
            // 帧推送：composed 变化 → FrameReady（payload 内联）。
            if let Some(wid) = mirror.wid {
                let composed = self.broker_composed(wid).cloned();
                if let Some(frame) = composed {
                    if mirror.last_frame.as_ref() != Some(&frame) {
                        mirror.seq += 1;
                        let msg = ProtocolMsg::Frame(FrameMsg::FrameReady {
                            wid: wid.0,
                            frame_id: mirror.seq,
                            slot: 0,
                            damage: None,
                            revision: mirror.seq,
                            payload: frame.clone(),
                        });
                        if mirror.end.send(&msg).is_err() {
                            dead.push(idx);
                            continue;
                        }
                        mirror.last_frame = Some(frame);
                    }
                }
            }
        }
        for idx in dead.into_iter().rev() {
            mirrors.remove(idx);
        }
        self.desktop.remote_mirrors = mirrors;
    }

    /// 镜像入站消息：Hello = 订阅；Input = 路由到目标 client（与
    /// `broker_pointer_down` 同一收尾——直投连接，权威命中在 app 侧）。
    fn remote_mirror_on_msg(&mut self, mirror: &mut RemoteMirror, msg: ProtocolMsg) {
        match msg {
            ProtocolMsg::Handshake(HandshakeMsg::Hello { app_name, .. }) => {
                mirror.app = Some(app_name);
            }
            ProtocolMsg::Input(input) => {
                let wid = match mirror.wid {
                    Some(w) => w,
                    None => return,
                };
                let wire = ProtocolMsg::Input(input);
                for client in self.broker_clients.values_mut() {
                    if client.wid == Some(wid) {
                        let _ = client.end.send(&wire);
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    /// 订阅兑现：目标在册 → Welcome（复用握手通道语义；surface=0 无
    /// shm——帧内联）+ HitTable（宿主孪生投影器同源布局，T3 孪生确定性
    /// 同理）+ 首帧。
    fn remote_mirror_subscribe(&mut self, mirror: &mut RemoteMirror) {
        let app_name = mirror.app.clone().unwrap_or_default();
        let Some((app_id, wid, rect, width, height)) = (|| {
            let client = self
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some(app_name.as_str()))?;
            let wid = client.wid?;
            let app_id = client.app_id?;
            let host = self.host.as_ref()?;
            let v = host.wm.wins.get(&wid)?;
            let r = *v.rect.borrow();
            Some((app_id, wid, r, r.width.max(1.0), r.height.max(1.0)))
        })() else {
            return; // 目标未落地：留待下拍重试
        };
        mirror.wid = Some(wid);
        // Welcome（帧模式恒 Commands——镜像只走内联命令帧）。
        let welcome = ProtocolMsg::Handshake(HandshakeMsg::Welcome {
            app_id: app_id.0,
            wid: wid.0,
            surface: 0,
            rect: crate::ui::desktop_protocol::host::rect_to_wire(&rect),
            frame_mode: crate::ui::desktop_protocol::message::FrameMode::Commands,
        });
        let _ = mirror.end.send(&welcome);
        // HitTable：宿主孪生（同源 resolver → 同布局）。
        if let Some(hits) = self.remote_twin_hits(&app_name, width, height) {
            let table = ProtocolMsg::Frame(FrameMsg::HitTable { wid: wid.0, hits });
            let _ = mirror.end.send(&table);
        }
    }

    /// 宿主孪生命中表：resolver 同源编译 + AppProjector（T3 孪生确定性
    /// ——child 同引擎同尺寸 → 同命中布局）。
    fn remote_twin_hits(
        &self,
        app_name: &str,
        width: f32,
        height: f32,
    ) -> Option<Vec<HitRegion>> {
        use crate::ui::desktop_protocol::endpoint::FrameSource;
        let spec = self.desktop.app_resolver.as_ref()?(app_name)?;
        let comp = crate::build_dynamic_component(&spec.code, spec.source_path.as_deref()).ok()?;
        let mut twin =
            crate::ui::desktop_protocol::client_runtime::AppProjector::new(comp, width, height);
        twin.render_frame();
        Some(
            twin.hit_regions()
                .into_iter()
                .map(|(rect, kind)| {
                    let (kind, action) = if let Some(handler) = kind.strip_prefix("button:") {
                        (HIT_KIND_BUTTON, handler.to_string())
                    } else if let Some(field) = kind.strip_prefix("input:") {
                        (HIT_KIND_INPUT, field.to_string())
                    } else {
                        (0u8, String::new())
                    };
                    HitRegion { rect, kind, action }
                })
                .filter(|h| h.kind != 0)
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::broker;
    use crate::ui::desktop_protocol::message::{DrawOp, InputMsg, MouseButton};
    use crate::ui::desktop_protocol::transport::{self, ws};
    use crate::ui::session::{DesktopSession, LaunchSpec, ProcessModel};
    use std::sync::Arc;

    /// 002-counter 源（真示例经 `example_source` 读取——T3 同源）。
    fn example_source(dir: &str) -> String {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/ui/P/src/front/app.at"
        )
        .replace('P', dir);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
    }

    /// Plan 508 T2：远程会话孵化 → 帧到达 → 输入回发 → handler 闭环
    /// （Rust 侧 headless，不经浏览器）。子进程 = re-exec 测试体
    /// （t3_child_body queue 档——WS 镜像与 pipe 会话同代码路径的实证）。
    #[test]
    fn t2_remote_ws_session_end_to_end() {
        let app = "002-counter";
        let src = example_source(app);
        let broker_pipe = format!("autodesk-broker-t2-508-{}", std::process::id());

        // 宿主会话：outproc 002 + broker + 远程监听（port 0 = 系统分配）。
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let src_for_resolver = src.clone();
        session.desktop.app_resolver = Some(Arc::new(move |name: &str| {
            (name == app).then(|| LaunchSpec {
                code: src_for_resolver.clone(),
                source_path: None,
                title: Some(app.to_string()),
                name: None,
                fit: false,
                daemon: None,
                back_root: None,
            })
        }));
        session.desktop.process_model = ProcessModel::Outproc;
        let pipe_for_spawn = broker_pipe.clone();
        session.desktop.outproc_spawner = Some(Arc::new(move |_name| {
            let exe = std::env::current_exe().expect("current_exe");
            let mut cmd = std::process::Command::new(&exe);
            cmd.args(["t3_child_body", "--test-threads", "1", "--nocapture"])
                .env("AUTO_500_BROKER", &pipe_for_spawn)
                .env("AUTO_500_APP", app)
                .env("AUTO_500_MODE", "queue")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::inherit());
            for (k, _) in std::env::vars() {
                if k.starts_with("NEXTEST_") {
                    cmd.env_remove(&k);
                }
            }
            cmd.spawn()
        }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));
        session.enable_remote_ws("t2-token", 0);
        let port = session.remote_ws_port().expect("ws port");

        // outproc 孵化落地（G1 臂）。
        session.launch_app(app).expect("outproc 落地");

        // 远程端连入 + 订阅。
        let url = format!("ws://127.0.0.1:{port}/?token=t2-token");
        let mut remote = ws::connect(&url, 3000).expect("ws connect");
        let mut remote_hello = |remote: &mut Box<dyn crate::ui::desktop_protocol::transport::Transport + Send>| {
            use crate::ui::desktop_protocol::message::HandshakeMsg;
            let _ = remote.send(&ProtocolMsg::Handshake(HandshakeMsg::Hello {
                version: crate::ui::desktop_protocol::PROTOCOL_VERSION,
                app_name: app.to_string(),
                title: app.to_string(),
                icon: None,
                width: 480.0,
                height: 320.0,
                fonts: Vec::new(),
            }));
        };
        remote_hello(&mut remote);

        // 泊泵：Welcome + HitTable + 首帧（"Counter: 0"）。
        let mut saw_welcome = false;
        let mut saw_hits = false;
        let mut saw_first_frame = false;
        let mut wid = 0u64;
        let mut plus_center = None;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !(saw_welcome && saw_hits && saw_first_frame) {
            assert!(std::time::Instant::now() < deadline, "T2 握手三件套超时");
            session.attach_pending_remotes();
            session.pump_broker_clients();
            session.pump_remote_mirrors();
            while let Some(loaded) = remote.try_recv() {
                match loaded.expect("解码") {
                    ProtocolMsg::Handshake(HandshakeMsg::Welcome { wid: w, .. }) => {
                        saw_welcome = true;
                        wid = w;
                    }
                    ProtocolMsg::Frame(FrameMsg::HitTable { hits, .. }) => {
                        saw_hits = true;
                        // "+" = 第三个按钮命中区（T3 行序同源）。
                        if let Some(r) = hits
                            .iter()
                            .filter(|h| h.kind == HIT_KIND_BUTTON)
                            .nth(2)
                        {
                            plus_center = Some((r.rect.x + r.rect.w / 2.0, r.rect.y + r.rect.h / 2.0));
                        }
                    }
                    ProtocolMsg::Frame(FrameMsg::FrameReady { payload, .. }) => {
                        if payload.ops.iter().any(|op| matches!(op,
                            DrawOp::Text { text, .. } if text == "Counter: 0"))
                        {
                            saw_first_frame = true;
                        }
                    }
                    _ => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(saw_welcome && saw_hits && saw_first_frame, "三件套");
        let (hx, hy) = plus_center.expect("'+' 命中区随表下发");

        // 输入回发：点击 "+" → 新帧 "Counter: 1"（handler 闭环）。
        let _ = remote.send(&ProtocolMsg::Input(InputMsg::PointerPressed {
            wid,
            button: MouseButton::Left,
            x: hx,
            y: hy,
            modifiers: 0,
        }));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut clicked = false;
        while !clicked {
            assert!(std::time::Instant::now() < deadline, "点击帧超时");
            session.attach_pending_remotes();
            session.pump_broker_clients();
            session.pump_remote_mirrors();
            while let Some(loaded) = remote.try_recv() {
                if let ProtocolMsg::Frame(FrameMsg::FrameReady { payload, .. }) =
                    loaded.expect("解码")
                {
                    if payload.ops.iter().any(|op| matches!(op,
                        DrawOp::Text { text, .. } if text == "Counter: 1"))
                    {
                        clicked = true;
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        // 收尾：kill 子进程 + 停 broker。
        for mut child in session.desktop.outproc_children.drain(..) {
            let _ = child.kill();
            let _ = child.wait();
        }
        drop(remote);
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }
}

#[cfg(test)]
mod ts_fixtures {
    use super::*;
    use crate::ui::desktop_protocol::message::{DrawOp, InputMsg, MouseButton, Rgba8, WRect};

    /// Plan 508 T3 对拍锚点：Rust ↔ TS（packages/drawlist-renderer
    /// test/fixtures.golden.ts）双侧钉同一批字节——Hello（握手编码）/
    /// PointerPressed（InputMsg 编码）/ FrameReady（DrawList 解码）。
    /// 任一侧 codec 漂移即红（计划②：手工镜像 + 对拍兜底）。
    #[test]
    fn p508_ts_crosscheck_golden_bytes() {
        let hello = ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: 1,
            app_name: "002-counter".into(),
            title: "002-counter".into(),
            icon: None,
            width: 480.0,
            height: 320.0,
            fonts: Vec::new(),
        });
        let expect_hello: Vec<u8> = vec![
            0x41, 0x50, 0x44, 0x4c, 0x01, 0x00, 0x01, 0x00, 0x2e, 0x00, 0x00, 0x00, 0x01, 0x01,
            0x00, 0x0b, 0x00, 0x00, 0x00, 0x30, 0x30, 0x32, 0x2d, 0x63, 0x6f, 0x75, 0x6e, 0x74,
            0x65, 0x72, 0x0b, 0x00, 0x00, 0x00, 0x30, 0x30, 0x32, 0x2d, 0x63, 0x6f, 0x75, 0x6e,
            0x74, 0x65, 0x72, 0x00, 0x00, 0x00, 0xf0, 0x43, 0x00, 0x00, 0xa0, 0x43, 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(hello.encode(), expect_hello, "TS 对拍锚点 Hello");

        let press = ProtocolMsg::Input(InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: 100.5,
            y: 50.25,
            modifiers: 0,
        });
        let expect_press: Vec<u8> = vec![
            0x41, 0x50, 0x44, 0x4c, 0x01, 0x00, 0x03, 0x00, 0x13, 0x00, 0x00, 0x00, 0x02, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0xc9, 0x42, 0x00, 0x00,
            0x49, 0x42, 0x00,
        ];
        assert_eq!(press.encode(), expect_press, "TS 对拍锚点 PointerPressed");

        let frame = ProtocolMsg::Frame(FrameMsg::FrameReady {
            wid: 1,
            frame_id: 2,
            slot: 0,
            damage: None,
            revision: 2,
            payload: crate::ui::desktop_protocol::message::DrawList {
                clear: Some(Rgba8 { r: 9, g: 14, b: 26, a: 255 }),
                ops: vec![
                    DrawOp::Quad {
                        rect: WRect { x: 8.0, y: 8.0, w: 100.0, h: 40.0 },
                        color: Rgba8 { r: 59, g: 130, b: 246, a: 255 },
                    },
                    DrawOp::Text {
                        x: 12.0,
                        y: 16.0,
                        size: 14.0,
                        line_height: 20.0,
                        color: Rgba8 { r: 255, g: 255, b: 255, a: 255 },
                        text: "Counter: 0".into(),
                    },
                ],
            },
        });
        let expect_frame: Vec<u8> = vec![
            0x41, 0x50, 0x44, 0x4c, 0x01, 0x00, 0x02, 0x00, 0x5d, 0x00, 0x00, 0x00, 0x04, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x09,
            0x0e, 0x1a, 0xff, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x41, 0x00, 0x00,
            0x00, 0x41, 0x00, 0x00, 0xc8, 0x42, 0x00, 0x00, 0x20, 0x42, 0x3b, 0x82, 0xf6, 0xff,
            0x02, 0x00, 0x00, 0x40, 0x41, 0x00, 0x00, 0x80, 0x41, 0x00, 0x00, 0x60, 0x41, 0x00,
            0x00, 0xa0, 0x41, 0xff, 0xff, 0xff, 0xff, 0x0a, 0x00, 0x00, 0x00, 0x43, 0x6f, 0x75,
            0x6e, 0x74, 0x65, 0x72, 0x3a, 0x20, 0x30,
        ];
        assert_eq!(frame.encode(), expect_frame, "TS 对拍锚点 FrameReady");
    }
}

#[cfg(test)]
mod host_body {
    use super::*;
    use crate::ui::desktop_protocol::transport::{self, ws};
    use crate::ui::session::{DesktopSession, LaunchSpec, ProcessModel};
    use std::sync::Arc;

    /// demo/T4 宿主 harness 识别 env（缺省直接跑套件时跳过）。
    const ENV_TOKEN: &str = "P508_HOST_TOKEN";
    const ENV_PORT: &str = "P508_HOST_PORT";
    const ENV_READY: &str = "P508_HOST_READY";
    const ENV_APPS: &str = "P508_HOST_APPS";

    /// 真 `auto` 二进制定位（stage3 同款：debug > release，缺则增量构建）。
    fn auto_exe() -> std::path::PathBuf {
        let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
        for profile in ["debug", "release"] {
            let p = target.join(profile).join("auto.exe");
            if p.exists() {
                return p;
            }
        }
        let status = std::process::Command::new("cargo")
            .args(["build", "-p", "auto", "--bin", "auto"])
            .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .status()
            .expect("spawn cargo build -p auto");
        assert!(status.success(), "cargo build -p auto 失败");
        target.join("debug").join("auto.exe")
    }

    /// Plan 508 T4/T6 宿主 harness：outproc 孵化示例批（真 auto.exe
    /// 生产链）+ WS 远程监听 + 常驻泵，直至驱动脚本 kill。ready 文件写
    /// 实际端口（bind(0) 支撑）。直接跑套件（无 env）时跳过。
    #[test]
    fn p508_remote_host_body() {
        let Ok(token) = std::env::var(ENV_TOKEN) else {
            return;
        };
        let port: u16 = std::env::var(ENV_PORT)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let apps: Vec<String> = std::env::var(ENV_APPS)
            .unwrap_or_else(|_| "002-counter".into())
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let broker_pipe = format!("autodesk-broker-508host-{}", std::process::id());

        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        // 注册表 = examples 主根（真磁盘源；T3 example_source 同源装载）。
        let mut entries: Vec<(String, LaunchSpec)> = Vec::new();
        for name in &apps {
            let path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../examples/ui/P/src/front/app.at"
            )
            .replace('P', name);
            let code = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {path}: {e}"));
            entries.push((
                name.clone(),
                LaunchSpec {
                    code,
                    source_path: Some(path),
                    title: Some(name.clone()),
                    ..Default::default()
                },
            ));
        }
        session.desktop.app_resolver = Some(Arc::new(move |name: &str| {
            entries.iter().find(|(n, _)| n == name).map(|(_, s)| LaunchSpec {
                code: s.code.clone(),
                source_path: s.source_path.clone(),
                title: s.title.clone(),
                ..Default::default()
            })
        }));
        // outproc 生产 spawner：真 auto.exe + 测试隔离 broker 管道名。
        let exe = auto_exe();
        let app_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui");
        let pipe_for_spawn = broker_pipe.clone();
        session.desktop.outproc_spawner = Some(Arc::new(move |child_name| {
            let mut cmd = std::process::Command::new(&exe);
            cmd.args([
                "run",
                "--autodesk-incubate",
                &format!("--app386={child_name}"),
                &format!("--autodesk-broker={pipe_for_spawn}"),
            ])
            .env("AUTO_386_APP_ROOT", &app_root)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit());
            for (k, _) in std::env::vars() {
                if k.starts_with("NEXTEST_") {
                    cmd.env_remove(&k);
                }
            }
            cmd.spawn()
        }));
        session.desktop.process_model = ProcessModel::Outproc;
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));
        session.enable_remote_ws(&token, port);
        let actual_port = session.remote_ws_port().expect("ws port");
        eprintln!(
            "[p508-host] ws=ws://127.0.0.1:{actual_port}/?token={token} apps={apps:?}"
        );
        // ready 文件 = 驱动脚本同步点（端口握手）。
        if let Ok(ready) = std::env::var(ENV_READY) {
            std::fs::write(&ready, actual_port.to_string()).expect("write ready");
        }
        for app in &apps {
            match session.launch_app(app) {
                Ok(_) => eprintln!("[p508-host] outproc landed: {app}"),
                Err(e) => eprintln!("[p508-host] launch {app} failed: {e}"),
            }
        }
        // 常驻泵：帧/输入/远程镜像（15ms 节拍——快于人眼帧预算，慢于忙转）。
        loop {
            if session.pending_remote_connections() > 0 {
                session.attach_pending_remotes();
            }
            session.pump_broker_clients();
            session.pump_remote_mirrors();
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    }
}
