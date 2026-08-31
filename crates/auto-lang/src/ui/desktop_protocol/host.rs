// Plan 386 Stage 1 —— 宿主侧适配层：协议端点 → 真实 462 会话对象。
//
// [`ProtocolHost`] 把 [`HostAction`] 落到 `DesktopSession`/`WmState`
// （孵化 = `allocate_app` + `wm_add_win`；回收 = `wm_remove_win` + App
// 移除，462 Close 语义；输入 = `WmState::hit_test` → (Wid, event)），
// 并持有 [`SurfaceStore`]——双缓冲表面即 Stage 1 的"共享纹理模拟"，
// `front()` 就是虚拟窗合成面（live-iced 消费留 Stage 2，零删除替换）。

use std::collections::BTreeMap;

use iced::{Point, Rectangle, Size};

use super::endpoint::{HostAction, HostEndpoint, ProtocolError};
use super::message::{ControlMsg, DrawList, FrameMsg, InputMsg, MouseButton, ObserveMsg, ProtocolMsg, WRect};
use crate::ui::dynamic::DynamicComponent;
use crate::ui::session::{AppId, DesktopSession, Wid};

/// widget 本地坐标 ↔ 宿主坐标互转（协议侧永远携带 widget 本地坐标）。
pub fn rect_to_wire(r: &Rectangle) -> WRect {
    WRect::new(r.x, r.y, r.width, r.height)
}

// ---------------------------------------------------------------------------
// SurfaceStore：双缓冲表面（共享纹理模拟）
// ---------------------------------------------------------------------------

/// 一个虚拟窗的表面：2 槽双缓冲 + 翻面记录。
pub struct Surface {
    slots: [Option<DrawList>; 2],
    /// 当前合成面（最近一次 compose 的槽）。
    pub front: u8,
    pub width: f32,
    pub height: f32,
}

/// 表面注册表（surface 句柄分配即 Welcome 回传的句柄）。
#[derive(Default)]
pub struct SurfaceStore {
    next_surface: u64,
    surfaces: BTreeMap<u64, Surface>,
}

impl SurfaceStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// 分配双缓冲表面（v1 固定 2 槽），返回句柄。
    ///
    /// 句柄**进程级唯一**（全局原子计数——Plan 500 T3）：多 client 宿主
    /// 各持一份 `SurfaceStore`，per-instance 自增会跨 client 重复 → shm
    /// 段名 `autodesk-shm-<pid>-<surface>` 撞名（Windows 同名 = 打开既有
    /// 段；480 压测五 child 同源 App 掩蔽了此缺陷，异源 App 直接串段）。
    pub fn alloc(&mut self, width: f32, height: f32) -> u64 {
        static GLOBAL_SURFACE: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let id = GLOBAL_SURFACE.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        self.next_surface = id;
        self.surfaces.insert(
            id,
            Surface { slots: [None, None], front: 0, width, height },
        );
        id
    }

    /// 合成一帧：payload 入槽并翻面；返回让给 app 的空槽（原 front）。
    pub fn compose(&mut self, surface: u64, slot: u8, payload: DrawList) -> Option<u8> {
        let s = self.surfaces.get_mut(&surface)?;
        if slot as usize >= s.slots.len() {
            return None;
        }
        s.slots[slot as usize] = Some(payload);
        let freed = s.front;
        s.front = slot;
        Some(freed)
    }

    /// 虚拟窗当前合成面（live-iced 渲染器 Stage 2 的取用点）。
    pub fn front(&self, surface: u64) -> Option<&DrawList> {
        self.surfaces.get(&surface)?.slots[self.surfaces[&surface].front as usize].as_ref()
    }

    pub fn release(&mut self, surface: u64) -> bool {
        self.surfaces.remove(&surface).is_some()
    }

    pub fn len(&self) -> usize {
        self.surfaces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }
}

// ---------------------------------------------------------------------------
// ProtocolHost
// ---------------------------------------------------------------------------

/// 宿主侧协议运行时：端点 + 表面 + 真实会话绑定。
///
/// `resolver` 是孵化材料源（app_name → 编译装载）——与
/// `DesktopSession.desktop.app_resolver` 同形；demo 内联注入，生产侧 =
/// 注册表（Stage 2 = spawn exe）。
pub struct ProtocolHost<'a> {
    pub session: &'a mut DesktopSession,
    pub endpoint: HostEndpoint,
    pub surfaces: SurfaceStore,
    /// 待回发 app 的消息队列（`handle` 累积，调用方经 loopback 转交）。
    pub to_app: Vec<ProtocolMsg>,
    /// 控制上行收件箱（TitleChanged/Notify/DesktopBus——chrome/通知中心/
    /// DesktopBus 执行体的 Stage 2 落点，v1 记录供断言）。
    pub control_inbox: Vec<ControlMsg>,
    /// 观测上行收件箱（MCP 代理 Stage 2 落点）。
    pub observe_inbox: Vec<ObserveMsg>,
    /// wid → surface 句柄。
    wid_surface: BTreeMap<u64, u64>,
    /// surface → 共享内存帧缓冲（S9：FrameReadyShared 载荷源）。
    shm_buffers: BTreeMap<u64, super::shm::SharedFrameBuffer>,
    /// 孵化材料解析器。
    resolver: Box<dyn FnMut(&str) -> Result<DynamicComponent, String> + 'a>,
}

impl<'a> ProtocolHost<'a> {
    pub fn new(
        session: &'a mut DesktopSession,
        resolver: impl FnMut(&str) -> Result<DynamicComponent, String> + 'a,
    ) -> Self {
        Self {
            session,
            endpoint: HostEndpoint::listen(),
            surfaces: SurfaceStore::new(),
            to_app: Vec::new(),
            control_inbox: Vec::new(),
            observe_inbox: Vec::new(),
            wid_surface: BTreeMap::new(),
            shm_buffers: BTreeMap::new(),
            resolver: Box::new(resolver),
        }
    }

    /// 宿主侧处理一条来自 app 的消息；回发消息累积进 [`Self::to_app`]。
    pub fn handle(&mut self, msg: &ProtocolMsg) -> Result<(), ProtocolError> {
        let actions = self.endpoint.on_message(msg.clone())?;
        for action in actions {
            match action {
                HostAction::ResolveAndAttach { app_name, title, width, height, .. } => {
                    // ① 编译装载（独立模式下这一步 = spawn exe，Stage 2）。
                    let component = (self.resolver)(&app_name)
                        .map_err(ProtocolError::ResolveFailed)?;
                    // ② 真实会话登记：App + 虚拟窗（462 孵化语义）。
                    let app_id = self.session.allocate_app(component);
                    let title = if title.is_empty() { app_name.clone() } else { title };
                    let rect = Rectangle::new(Point::new(16.0, 16.0), Size::new(width, height));
                    let wid = self.session.wm_add_win(app_id, title, rect);
                    // ③ 表面（含共享内存段）分配 + Welcome/BufferAlloc 回发。
                    let surface = self.surfaces.alloc(width, height);
                    self.wid_surface.insert(wid.0, surface);
                    // 全局唯一：pid 前缀防跨进程同名段（Windows
                    // CreateFileMappingW 同名=打开既有段，Plan 480 压测暴露）。
                    let shm_name =
                        format!("autodesk-shm-{}-{surface}", std::process::id());
                    let shm = super::shm::SharedFrameBuffer::create(&shm_name, 2, 16384)
                        .map_err(ProtocolError::Shm)?;
                    self.shm_buffers.insert(surface, shm);
                    let welcome = self.endpoint.activate(
                        app_id.0,
                        wid.0,
                        surface,
                        rect_to_wire(&rect),
                        self.endpoint.frame_mode, // v1.3 缺省 Commands（三态开关在 adjudicate 链接入时改写）
                    )?;
                    self.to_app.push(welcome);
                    self.to_app.push(ProtocolMsg::Frame(FrameMsg::BufferAlloc {
                        surface,
                        slots: 2,
                        width,
                        height,
                        shm: Some(shm_name),
                    }));
                }
                HostAction::ComposeFrame { surface, wid, frame_id, slot, payload, .. } => {
                    if let Some(freed) = self.surfaces.compose(surface, slot, payload) {
                        self.to_app.push(ProtocolMsg::Frame(FrameMsg::FrameAck {
                            wid,
                            frame_id,
                            slot: freed,
                        }));
                    }
                }
                HostAction::ComposeFrameShared { surface, wid, frame_id, slot, .. } => {
                    // 从共享内存槽读载荷 → 解码 → 与管道帧同路合成。
                    let ready = self
                        .shm_buffers
                        .get(&surface)
                        .and_then(|shm| shm.read_slot(slot).ok())
                        .and_then(|payload| {
                            super::shm::draw_list_from_slot_payload(&payload).ok()
                        });
                    if let Some(payload) = ready {
                        if let Some(freed) = self.surfaces.compose(surface, slot, payload) {
                            self.to_app.push(ProtocolMsg::Frame(FrameMsg::FrameAck {
                                wid,
                                frame_id,
                                slot: freed,
                            }));
                        }
                    }
                }
                HostAction::ComposeFramePixels { surface, wid, frame_id, slot, .. } => {
                    // v1.3 像素臂（单 client 测试机件的最小处理）：shm 槽读
                    // RGBA 成功即翻面回 ack（像素前缓冲驻留归 stage3 多 client
                    // 宿主——BrokerClient::pixels）。
                    let ready = self
                        .shm_buffers
                        .get(&surface)
                        .and_then(|shm| shm.read_slot(slot).ok());
                    if ready.is_some() {
                        self.to_app.push(ProtocolMsg::Frame(FrameMsg::FrameAck {
                            wid,
                            frame_id,
                            slot,
                        }));
                    }
                }
                HostAction::ReclaimWindow { wid } => {
                    // 462 Close 语义：窗随 App 移除，表面释放，通知 app。
                    let app_id = self.session.wm_remove_win(Wid(wid));
                    if let Some(app_id) = app_id {
                        self.session.apps.remove(&app_id);
                    }
                    if let Some(surface) = self.wid_surface.remove(&wid) {
                        self.shm_buffers.remove(&surface);
                        self.surfaces.release(surface);
                        self.to_app.push(ProtocolMsg::Frame(FrameMsg::BufferRelease { surface }));
                    }
                }
                HostAction::ObserveUp { msg } => self.observe_inbox.push(msg),
            }
        }
        // 控制上行（端点只透传，这里落收件箱）。
        match msg {
            ProtocolMsg::Control(c @ (ControlMsg::TitleChanged { .. }
            | ControlMsg::Notify { .. }
            | ControlMsg::DesktopBus { .. })) => self.control_inbox.push(c.clone()),
            _ => {}
        }
        Ok(())
    }

    /// 桌面级指针按下：WM 命中 → 聚焦 → (Wid, event) 注入 app。
    /// 返回注入消息（调用方经 loopback 发往 app 侧）。
    /// chrome 命中（标题栏/把手）在 live 集成由 update 壳层先行；v1
    /// loopback 把窗内按下全量转 app（chrome 拦截是渲染器层职责）。
    pub fn pointer_down(&mut self, x: f32, y: f32, button: MouseButton) -> Option<ProtocolMsg> {
        let wid = {
            let host = self.session.host.as_ref()?;
            host.wm.hit_test(x, y)?
        };
        self.session.wm_focus(wid);
        let rect = {
            let host = self.session.host.as_ref()?;
            let vwin = host.wm.wins.get(&wid)?;
            *vwin.rect.borrow()
        };
        // 宿主坐标 → widget 本地坐标（v1：客户区 = 虚拟窗矩形，无 chrome 偏移）。
        let input = InputMsg::PointerPressed {
            wid: wid.0,
            button,
            x: x - rect.x,
            y: y - rect.y,
            modifiers: 0,
        };
        Some(ProtocolMsg::Input(input))
    }

    /// 虚拟窗当前合成面（`wid` 定位；demo/测试断言口）。
    pub fn composed(&self, wid: u64) -> Option<&DrawList> {
        let surface = *self.wid_surface.get(&wid)?;
        self.surfaces.front(surface)
    }

    /// 当前客户端的 (app_id, wid)。
    pub fn active(&self) -> (Option<AppId>, Option<Wid>) {
        (
            self.endpoint.app_id.map(AppId),
            self.endpoint.wid.map(Wid),
        )
    }

    /// 孵化连接泵到 Active（Plan 480 S3）：Hello → ResolveAndAttach →
    /// Welcome/BufferAlloc 回发。宿主侧 Active 即落地完成（app 的
    /// Ready 是 no-op 例行收尾，不等）；预算内未收敛返回 None，调用方
    /// 弃置该连接。回发经 `end` 送回 app 侧。
    pub fn pump_incubation(
        &mut self,
        end: &mut Box<dyn super::transport::Transport + Send>,
        budget_ms: u32,
    ) -> Option<Wid> {
        use super::endpoint::HostState;
        let deadline = std::time::Instant::now()
            + std::time::Duration::from_millis(budget_ms as u64);
        while std::time::Instant::now() < deadline {
            if self.endpoint.state == HostState::Active {
                return self.endpoint.wid.map(Wid);
            }
            match end.try_recv() {
                Some(Ok(msg)) => {
                    if self.handle(&msg).is_err() {
                        return None;
                    }
                    for reply in std::mem::take(&mut self.to_app) {
                        let _ = end.send(&reply);
                    }
                }
                Some(Err(_)) => return None,
                None => std::thread::sleep(std::time::Duration::from_millis(5)),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::demo::COUNTER_SRC;
    use crate::ui::desktop_protocol::endpoint::{AppEndpoint, FrameSource, AppState};
    use crate::ui::desktop_protocol::message::{HandshakeMsg, Rgba8};

    fn resolver() -> impl FnMut(&str) -> Result<DynamicComponent, String> {
        |name: &str| {
            if name == "counter" {
                crate::build_dynamic_component(COUNTER_SRC, None)
                    .map_err(|e| format!("build failed: {e}"))
            } else {
                Err(format!("unknown app: {name}"))
            }
        }
    }

    /// 计数器 FrameSource：按钮区域命中 → onclick；帧 = 按钮 + count 文本。
    struct CounterSource {
        component: DynamicComponent,
        button: WRect,
        rev: u64,
    }

    impl CounterSource {
        fn new(component: DynamicComponent) -> Self {
            Self { component, button: WRect::new(10.0, 10.0, 120.0, 36.0), rev: 1 }
        }
    }

    fn count_of(component: &DynamicComponent) -> i64 {
        match component.read_state("count") {
            Ok(auto_val::Value::Int(n)) => n as i64,
            other => panic!("count 读取失败: {other:?}"),
        }
    }

    impl FrameSource for CounterSource {
        fn revision(&self) -> u64 {
            self.rev
        }

        fn render_frame(&mut self) -> DrawList {
            let n = count_of(&self.component);
            DrawList {
                clear: Some(Rgba8::new(24, 24, 28, 255)),
                ops: vec![
                    super::super::message::DrawOp::Quad {
                        rect: self.button,
                        color: Rgba8::new(48, 96, 200, 255),
                    },
                    super::super::message::DrawOp::Text {
                        x: self.button.x + 50.0,
                        y: self.button.y + 10.0,
                        size: 14.0,
                        line_height: 18.0,
                        color: Rgba8::new(255, 255, 255, 255),
                        text: "+".into(),
                    },
                    super::super::message::DrawOp::Text {
                        x: 10.0,
                        y: 60.0,
                        size: 16.0,
                        line_height: 20.0,
                        color: Rgba8::new(220, 220, 220, 255),
                        text: format!("count: {n}"),
                    },
                ],
            }
        }

        fn on_input(&mut self, input: &InputMsg) {
            if let InputMsg::PointerPressed { x, y, button: MouseButton::Left, .. } = input {
                let b = self.button;
                if *x >= b.x && *x < b.x + b.w && *y >= b.y && *y < b.y + b.h {
                    // 与直挂完全同一调用：DynamicComponent::on_with_input。
                    self.component.on_with_input("__evt_onclick_1", None);
                    self.rev += 1;
                }
            }
        }

        fn on_control(&mut self, _control: &ControlMsg) {}
    }

    #[test]
    fn incubation_allocates_real_session_objects() {
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let mut host = ProtocolHost::new(&mut session, resolver());
        let mut app = AppEndpoint::new(
            CounterSource::new(crate::build_dynamic_component(COUNTER_SRC, None).unwrap()),
            "counter",
            "计数器",
            480.0,
            320.0,
        );

        let hello = app.connect().unwrap();
        host.handle(&hello).unwrap();
        // Welcome + BufferAlloc 都已排队。
        assert_eq!(host.to_app.len(), 2);
        assert!(matches!(host.to_app[0], ProtocolMsg::Handshake(HandshakeMsg::Welcome { .. })));
        assert!(matches!(host.to_app[1], ProtocolMsg::Frame(FrameMsg::BufferAlloc { .. })));

        // 真实 462 对象就位：AppSession + VWinState + 焦点。
        let (app_id, wid) = host.active();
        assert_eq!(app_id, Some(AppId(1)));
        assert_eq!(wid, Some(Wid(1)));
        let s = &host.session;
        assert!(s.apps.contains_key(&AppId(1)), "AppSession 已登记");
        let host_ctx = s.host.as_ref().unwrap();
        assert!(host_ctx.wm.wins.contains_key(&Wid(1)), "虚拟窗已登记");
        assert_eq!(host_ctx.wm.focused, Some(Wid(1)), "新窗即焦点");
        assert_eq!(host.surfaces.len(), 1, "表面已分配");

        // app 消费 Welcome → Active。
        let replies = app.on_message(host.to_app[0].clone()).unwrap();
        assert_eq!(app.state, AppState::Active);
        assert_eq!(replies.len(), 1);
        host.handle(&replies[0]).unwrap();
    }

    #[test]
    fn pointer_routes_via_wm_hit_test_to_local_coords() {
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let mut host = ProtocolHost::new(&mut session, resolver());
        let mut app = AppEndpoint::new(
            CounterSource::new(crate::build_dynamic_component(COUNTER_SRC, None).unwrap()),
            "counter",
            "c",
            480.0,
            320.0,
        );
        let hello = app.connect().unwrap();
        host.handle(&hello).unwrap();
        let welcome = host.to_app[0].clone();
        for r in app.on_message(welcome).unwrap() {
            host.handle(&r).unwrap();
        }

        // 宿主坐标 (60, 40) 落在窗 (16,16,480,320) 内 → 本地 (44, 24)。
        let injected = host.pointer_down(60.0, 40.0, MouseButton::Left).unwrap();
        let ProtocolMsg::Input(InputMsg::PointerPressed { wid, x, y, .. }) = &injected else {
            panic!("期待 Input");
        };
        assert_eq!(*wid, 1);
        assert!(((*x) - 44.0).abs() < 1e-4 && ((*y) - 24.0).abs() < 1e-4, "本地坐标 {x},{y}");

        // app 侧命中按钮区 (10,10,120,36)：本地 (44,24) 在内 → onclick。
        app.on_message(injected).unwrap();
        assert_eq!(count_of(&app.session.component), 1, "VM handler 已执行");

        // 窗外按下不注入。
        assert!(host.pointer_down(4000.0, 4000.0, MouseButton::Left).is_none());
    }

    #[test]
    fn reclaim_removes_win_app_surface_and_notifies() {
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let mut host = ProtocolHost::new(&mut session, resolver());
        let mut app = AppEndpoint::new(
            CounterSource::new(crate::build_dynamic_component(COUNTER_SRC, None).unwrap()),
            "counter",
            "c",
            480.0,
            320.0,
        );
        let hello = app.connect().unwrap();
        host.handle(&hello).unwrap();
        for r in app.on_message(host.to_app[0].clone()).unwrap() {
            host.handle(&r).unwrap();
        }

        // 宿主 Close → app Closing + ExitRequest → 回收。
        let close = host.endpoint.close().unwrap();
        let replies = app.on_message(close).unwrap();
        assert_eq!(app.state, AppState::Closing);
        assert!(matches!(replies[0], ProtocolMsg::Control(ControlMsg::ExitRequest { .. })));
        host.handle(&replies[0]).unwrap();

        // 回收落地：462 对象与表面同清，BufferRelease 排队。
        let s = &host.session;
        assert!(s.apps.is_empty(), "App 已移除");
        assert!(s.host.as_ref().unwrap().wm.wins.is_empty(), "虚拟窗已回收");
        assert!(host.surfaces.is_empty(), "表面已释放");
        assert!(matches!(
            host.to_app.last(),
            Some(ProtocolMsg::Frame(FrameMsg::BufferRelease { .. }))
        ));

        // app 消费 BufferRelease → Detached。
        let release = host.to_app.last().unwrap().clone();
        app.on_message(release).unwrap();
        assert_eq!(app.state, AppState::Detached);
    }

    #[test]
    fn control_uplink_and_observe_inboxes() {
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let mut host = ProtocolHost::new(&mut session, resolver());

        // 先孵化一个客户端（控制/观测上行只在 Active 后有意义）。
        let hello = ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: 1,
            app_name: "counter".into(),
            title: "c".into(),
            icon: None,
            width: 480.0,
            height: 320.0,
            fonts: vec![],
        });
        host.handle(&hello).unwrap();
        assert_eq!(host.endpoint.state, super::super::endpoint::HostState::Active);

        // 控制上行三形态落收件箱；DesktopBus 载荷与既有 DesktopCommand 解析互通。
        host.handle(&ProtocolMsg::Control(ControlMsg::TitleChanged { wid: 1, title: "新".into() })).unwrap();
        host.handle(&ProtocolMsg::Control(ControlMsg::Notify { wid: 1, summary: "s".into(), body: "b".into() })).unwrap();
        host.handle(&ProtocolMsg::Control(ControlMsg::DesktopBus { wid: 1, record: "launch\u{1f}counter".into() })).unwrap();
        assert_eq!(host.control_inbox.len(), 3);
        let records: Vec<String> = host
            .control_inbox
            .iter()
            .filter_map(|c| match c {
                ControlMsg::DesktopBus { record, .. } => Some(record.clone()),
                _ => None,
            })
            .collect();
        let parsed = crate::ui::session::DesktopCommand::parse_records(&records.join("\n"));
        assert_eq!(
            parsed,
            vec![crate::ui::session::DesktopCommand::LaunchApp("counter".into())],
            "DesktopBus 载荷 = DesktopCommand 同格式"
        );

        host.handle(&ProtocolMsg::Observe(ObserveMsg::Log {
            wid: 1,
            level: crate::ui::desktop_protocol::message::LogLevel::Info,
            message: "ok".into(),
        }))
        .unwrap();
        assert_eq!(host.observe_inbox.len(), 1);
    }
}
