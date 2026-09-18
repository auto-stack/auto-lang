// PLAN-031 —— rqhost：共享合成器原生窗运行时（第四运行形态）的核心。
//
// `auto run -q` 客户端把 RenderQueue 帧交给一个共享的后台合成器进程
// （`auto rqhost`，iced daemon），宿主 OS 原生窗形态渲染——虚拟桌面
// 同型的 compositor 架构（多 app 共享单实例）。
//
// 三件核心（§5.1 定案）：
// - **rendezvous 采纳**（D1）：well-known 管道 `autodesk-rqhost`——客户端
//   `adopt␟<app_name>` 记录（DesktopBus 管道串约定族，零 wire 变体）→
//   分配 per-app 管道（先行 listen）→ 回名 → 客户端直连。探测 ping
//   （连上即关）吞掉——broker serve_once 同型。
// - **客户端权威采纳**（D1）：RqClient 不设 resolver——ResolveAndAttach
//   直接以 Hello 凭据（title/width/height）activate，宿主零装载。区别于
//   桌面 broker_apply_actions 的"宿主内容权威"（resolver MISS 弃连）。
// - **单实例仲裁**（D2）：锁管道 `<wellknown>-lock` 以 FILE_FLAG_FIRST_
//   PIPE_INSTANCE 声明——第二实例创建即 PermissionDenied，OS 级原子
//   零窗口（候选 A"探测自杀"的非原子窗口由此根除）。
//
// per-app `wait_connect` 线程化（D2 新事实）：rust 轨 cargo build 分钟级
// 延迟不得阻塞 adopt 环路（broker serve_once 的阻塞第二等连不适用）。
// daemon 装配（多窗/view/输入路由/末窗退出）见本文件 `run_daemon` 段。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::endpoint::{HostAction, HostEndpoint, HostState};
use super::host::SurfaceStore;
use super::message::{
    ControlMsg, DrawList, FrameMode, FrameMsg, HandshakeMsg, ProtocolMsg,
};
use super::shm::SharedFrameBuffer;
use super::stage3::BrokerClient;
use super::transport::{self, Transport, TransportError};
use crate::ui::session::Wid;

/// 生产 well-known 管道名（`autodesk-broker` broker.rs 同族）。
pub const RQHOST_PIPE: &str = "autodesk-rqhost";

/// 测试缝（P489 `adjudicate_on` 同型）：env 覆盖 well-known（pid 后缀防
/// 并行测试/本机常驻实例串扰）；生产行为零变化。
pub const RQHOST_WELLKNOWN_ENV: &str = "AUTO_RQHOST_WELLKNOWN";

/// 生效 well-known 管道名（env 覆盖 > 生产常量）。
pub fn wellknown_pipe() -> String {
    std::env::var(RQHOST_WELLKNOWN_ENV).unwrap_or_else(|_| RQHOST_PIPE.to_string())
}

// ---------------------------------------------------------------------------
// rendezvous：客户端侧
// ---------------------------------------------------------------------------

/// 客户端采纳：连 well-known → 发 `adopt␟<app_name>` → 收
/// `adopt␟<per-app pipe>` → 转连。返回 (per-app 管道名, 协议端点)。
/// 管道串约定族零 wire 变体（DesktopBus 载荷，I1）。
pub fn adopt(
    wellknown: &str,
    app_name: &str,
    timeout_ms: u32,
) -> Result<(String, Box<dyn Transport + Send>), TransportError> {
    let mut end = transport::connect(wellknown, timeout_ms)?;
    let ask = ProtocolMsg::Control(ControlMsg::DesktopBus {
        wid: 0,
        record: format!("adopt\u{1f}{app_name}"),
    });
    end.send(&ask)?;
    let reply = end
        .recv_wait(timeout_ms)
        .ok_or(TransportError::Eof)?
        .map_err(TransportError::Codec)?;
    let pipe_name = match reply {
        ProtocolMsg::Control(ControlMsg::DesktopBus { record, .. }) => record
            .split_once('\u{1f}')
            .filter(|(verb, _)| *verb == "adopt")
            .map(|(_, name)| name.to_string())
            .ok_or_else(|| TransportError::Io("bad adopt reply".into()))?,
        _ => return Err(TransportError::Io("bad adopt reply".into())),
    };
    let app_end = transport::connect(&pipe_name, timeout_ms)?;
    Ok((pipe_name, app_end))
}

/// 孵化器注入点（测试替身用；生产 = current_exe re-exec）。
pub type RqhostSpawner = Arc<dyn Fn(&str) -> std::io::Result<()> + Send + Sync>;

/// 生产孵化器：spawn `auto rqhost --pipe <wellknown>`（current_exe 寻址，
/// `outproc_auto_binary` 同型三级向上探测）。Child 句柄即弃——daemon 寿命
/// 独立于客户端（末窗自退）；NEXTEST_* 剥除防测试上下文串染。
fn spawn_rqhost_default(wellknown: &str) -> std::io::Result<()> {
    let exe = rqhost_auto_binary()?;
    let mut cmd = std::process::Command::new(&exe);
    cmd.args(["rqhost", "--pipe", wellknown]);
    for (key, _) in std::env::vars() {
        if key.starts_with("NEXTEST_") {
            cmd.env_remove(&key);
        }
    }
    cmd.spawn().map(|_| ())
}

/// `auto` 本体寻址（session.rs `outproc_auto_binary` 同型）：current_exe
/// 自命中（auto/auto.exe）或同目录向上至多三级。
fn rqhost_auto_binary() -> std::io::Result<std::path::PathBuf> {
    let exe = std::env::current_exe()?;
    let is_auto = exe.file_name().is_some_and(|n| n == "auto" || n == "auto.exe");
    if is_auto {
        return Ok(exe);
    }
    let name = if cfg!(windows) { "auto.exe" } else { "auto" };
    let mut dir = exe.parent();
    for _ in 0..3 {
        let Some(d) = dir else { break };
        let candidate = d.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
        dir = d.parent();
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("rqhost binary `auto` not found beside {}", exe.display()),
    ))
}

/// 发现与孵化序（G6/D2）：直试 adopt（在线即成功——连接即探测）→ 失败
/// spawn `auto rqhost` → 就绪退避重试（100ms 起倍增，预算 15s）。返回
/// per-app 管道名（`ClientTarget::Rqhost` 直连同管）。
pub fn ensure_rqhost(app_name: &str) -> Result<String, String> {
    let spawner: RqhostSpawner = Arc::new(spawn_rqhost_default);
    ensure_rqhost_with(&spawner, app_name)
}

/// 可测性缝形态：孵化器注入（生产 [`ensure_rqhost`]；测试以进程内
/// `RqServe` 替身覆盖 spawn+就绪全序，零真实进程）。
pub fn ensure_rqhost_with(
    spawner: &RqhostSpawner,
    app_name: &str,
) -> Result<String, String> {
    let wellknown = wellknown_pipe();
    // 直试（短超时：daemon 在 = 秒成；不在 = ~500ms 内失败）。
    if let Ok((pipe, _)) = adopt(&wellknown, app_name, 500) {
        return Ok(pipe);
    }
    spawner(&wellknown).map_err(|e| format!("spawn rqhost 失败: {e}"))?;
    // 就绪退避：adopt 内建 FILE_NOT_FOUND 重试，间隔即单次超时预算。
    let started = std::time::Instant::now();
    let mut backoff_ms: u32 = 100;
    loop {
        match adopt(&wellknown, app_name, backoff_ms) {
            Ok((pipe, _)) => return Ok(pipe),
            Err(_) => {
                if started.elapsed() >= std::time::Duration::from_secs(15) {
                    return Err("rqhost 孵化后 15s 未就绪（adopt 重试预算耗尽）".to_string());
                }
                std::thread::sleep(std::time::Duration::from_millis(backoff_ms as u64 / 4));
                backoff_ms = (backoff_ms * 2).min(1600);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// rendezvous：宿主侧（serve 环 + 锁管道 + per-adoption 线程）
// ---------------------------------------------------------------------------

/// 一次采纳的落成材料（per-app 管道已先行 listen，等客户端转连）。
pub struct PendingAdoption {
    pub app_name: String,
    pub pipe_name: String,
    listener: transport::PendingServer,
}

/// 解析 `adopt␟<app_name>`（探测 ping/异记录 = Ok(None) 吞掉重听）。
fn parse_adopt_record(msg: &ProtocolMsg) -> Option<String> {
    let ProtocolMsg::Control(ControlMsg::DesktopBus { record, .. }) = msg else {
        return None;
    };
    let mut parts = record.split('\u{1f}');
    match (parts.next(), parts.next()) {
        (Some("adopt"), Some(name)) if !name.is_empty() => Some(name.to_string()),
        _ => None,
    }
}

/// 受理一连接：listen well-known → accept → 读记录（100ms 无请求 = 探测
/// ping 吞掉）→ 分配 per-app 管道先行 listen → 回名。**不等转连**——
/// `wait_connect` 由调用方线程化（rust 轨 build 延迟不阻塞 adopt 环路）。
fn serve_adopt_once(
    pipe_name: &str,
    next_id: &AtomicU64,
) -> Result<Option<PendingAdoption>, TransportError> {
    let listener = transport::listen(pipe_name)?;
    let mut client = listener.wait_connect()?;
    let request = match client.recv_wait(100) {
        Some(Ok(msg)) => msg,
        _ => return Ok(None),
    };
    let Some(app_name) = parse_adopt_record(&request) else {
        return Ok(None);
    };
    let n = next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let app_pipe = format!("{pipe_name}-app-{n}");
    let app_listener = transport::listen(&app_pipe)?;
    let reply = ProtocolMsg::Control(ControlMsg::DesktopBus {
        wid: 0,
        record: format!("adopt\u{1f}{app_pipe}"),
    });
    client.send(&reply)?;
    eprintln!("[rqhost] adopt {app_name} -> {app_pipe}");
    Ok(Some(PendingAdoption { app_name, pipe_name: app_pipe, listener: app_listener }))
}

/// rqhost 启动失败形态（锁管道被占 = 单实例仲裁出局，调用方干净退出）。
#[derive(Debug)]
pub enum RqServeError {
    Transport(TransportError),
    /// 已有实例持有锁管道（FILE_FLAG_FIRST_PIPE_INSTANCE PermissionDenied）。
    AlreadyRunning,
}

impl From<TransportError> for RqServeError {
    fn from(e: TransportError) -> Self {
        Self::Transport(e)
    }
}

/// 宿主侧服务柄：serve 线程 + 待认领连接队列（daemon tick 消费）。
pub struct RqServe {
    /// 已转连落成的客户端端点（pipe 名丢失——RqClient 以自增 wid 簿记；
    /// 队列元素 = (app_name, end)）。
    pub pending: Arc<Mutex<Vec<(String, Box<dyn Transport + Send>)>>>,
    stop: Arc<AtomicBool>,
}

impl RqServe {
    /// 启动：锁管道声明（单实例仲裁）→ serve 线程。返回 (柄, 锁守卫)——
    /// 守卫须与 daemon 同寿命（Drop = 让出名字，下一实例可孵化）。
    pub fn start(wellknown: &str) -> Result<(Arc<Self>, transport::PipeClaim), RqServeError> {
        let claim = match transport::try_claim_pipe(&format!("{wellknown}-lock")) {
            Ok(claim) => claim,
            Err(TransportError::Io(e)) if e == transport::CLAIM_DENIED_MARKER => {
                return Err(RqServeError::AlreadyRunning)
            }
            Err(e) => return Err(e.into()),
        };
        let serve = Arc::new(Self {
            pending: Arc::new(Mutex::new(Vec::new())),
            stop: Arc::new(AtomicBool::new(false)),
        });
        let pipe_name = wellknown.to_string();
        let stop = Arc::clone(&serve.stop);
        let pending = Arc::clone(&serve.pending);
        let next_id = Arc::new(AtomicU64::new(0));
        std::thread::Builder::new()
            .name("rqhost-serve".into())
            .spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    match serve_adopt_once(&pipe_name, &next_id) {
                        Ok(Some(adoption)) => {
                            // per-adoption 线程等转连（不阻塞 adopt 环路）。
                            let pending = Arc::clone(&pending);
                            std::thread::Builder::new()
                                .name("rqhost-adopt-wait".into())
                                .spawn(move || {
                                    match adoption.listener.wait_connect() {
                                        Ok(end) => pending
                                            .lock()
                                            .unwrap()
                                            .push((adoption.app_name, end)),
                                        Err(e) => eprintln!(
                                            "[rqhost] adoption {} 未转连: {e:?}",
                                            adoption.pipe_name
                                        ),
                                    }
                                })
                                .expect("spawn adopt-wait thread");
                        }
                        Ok(None) => {} // 探测 ping：吞掉重听
                        Err(_) => {
                            std::thread::sleep(std::time::Duration::from_millis(20));
                        }
                    }
                }
            })
            .expect("spawn rqhost serve thread");
        Ok((serve, claim))
    }

    /// 停机：置位 + 一记探测连接唤醒阻塞在 accept 的 serve 环
    /// （`shutdown_broker` session.rs 同款停机序）。
    pub fn stop(&self, wellknown: &str) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = transport::connect(wellknown, 500);
    }
}

// ---------------------------------------------------------------------------
// RqClient：每客户端装配（HostEndpoint + 表面 + shm + OS 窗槽位）
// ---------------------------------------------------------------------------

/// 客户端权威采纳的宿主侧材料（桌面 broker_apply_actions 的"零装载"
/// 同构臂——D1：无 resolver，Hello 凭据即开窗凭据）。
pub struct RqClient {
    /// 协议装配（stage3::BrokerClient 复用：端点/表面/shm/wid 映射）。
    pub inner: BrokerClient,
    pub app_name: String,
    /// Hello 凭据（title 空名回退 app_name——broker_apply_actions 同则）。
    pub title: String,
    pub width: f32,
    pub height: f32,
    /// OS 窗（daemon 装配回填；core 层仅簿记槽位）。
    pub window: Option<iced::window::Id>,
    /// 已合成帧数（观测/e2e 断言面）。
    pub frames: u64,
}

/// rqhost 域 id 分配器（app_id/wid 与桌面会话无关的自增序）。
#[derive(Default)]
pub struct RqIds {
    next: u64,
}

impl RqIds {
    fn next(&mut self) -> u64 {
        self.next += 1;
        self.next
    }
}

/// 泵侧事件（daemon 消费：开窗/关窗）。
#[derive(Debug, Clone, PartialEq)]
pub enum RqEvent {
    /// Hello 已采纳——开窗凭据齐备（title/尺寸）。
    Adopted { app_name: String, title: String, width: f32, height: f32 },
    /// 窗应回收（ExitRequest→ReclaimWindow 握手完成，BufferRelease 已回发）。
    Reclaimed { wid: u64 },
}

/// HostAction 落 RqClient（session.rs broker_apply_actions 同构，减会话
/// 臂——不触 DesktopSession/WM/注册表）。返回回发 app 的消息 + 泵事件。
fn apply_actions(
    client: &mut RqClient,
    ids: &mut RqIds,
    actions: Vec<HostAction>,
) -> (Vec<ProtocolMsg>, Vec<RqEvent>) {
    let mut to_app = Vec::new();
    let mut events = Vec::new();
    for action in actions {
        match action {
            HostAction::ResolveAndAttach { app_name, title, width, height, .. } => {
                // 客户端权威：零装载。app_id/wid = rqhost 自有计数器；
                // rect = (0,0,w,h) 窗口本地坐标（表面铺满客户区，D4）。
                let app_id = ids.next();
                let wid = ids.next();
                let rect = super::message::WRect::new(0.0, 0.0, width, height);
                let surface = client.inner.surfaces.alloc(width, height);
                client.inner.wid_surface.insert(wid, surface);
                // shm 段名进程级唯一（host.rs 同则——pid+surface）。
                let shm_name = format!("autodesk-shm-{}-{surface}", std::process::id());
                let Ok(shm) = SharedFrameBuffer::create(&shm_name, 2, 16384) else {
                    continue;
                };
                client.inner.shm.insert(surface, shm);
                match client.inner.endpoint.activate(
                    app_id,
                    wid,
                    surface,
                    rect,
                    FrameMode::Commands,
                ) {
                    Ok(welcome) => {
                        to_app.push(welcome);
                        to_app.push(ProtocolMsg::Frame(FrameMsg::BufferAlloc {
                            surface,
                            slots: 2,
                            width,
                            height,
                            shm: Some(shm_name),
                        }));
                        client.inner.app_id = Some(crate::ui::session::AppId(app_id));
                        client.inner.wid = Some(Wid(wid));
                        client.inner.app_name = Some(app_name.clone());
                        let title = if title.is_empty() { app_name.clone() } else { title };
                        client.app_name = app_name.clone();
                        client.title = title.clone();
                        client.width = width;
                        client.height = height;
                        events.push(RqEvent::Adopted { app_name, title, width, height });
                    }
                    Err(_) => continue,
                }
            }
            HostAction::ComposeFrame { surface, wid, frame_id, slot, payload, .. } => {
                if let Some(freed) = client.inner.surfaces.compose(surface, slot, payload) {
                    client.frames += 1;
                    to_app.push(ProtocolMsg::Frame(FrameMsg::FrameAck {
                        wid,
                        frame_id,
                        slot: freed,
                    }));
                }
            }
            HostAction::ComposeFrameShared { surface, wid, frame_id, slot, .. } => {
                let ready = client
                    .inner
                    .shm
                    .get(&surface)
                    .and_then(|shm| shm.read_slot(slot).ok())
                    .and_then(|payload| {
                        super::shm::draw_list_from_slot_payload(&payload).ok()
                    });
                if let Some(payload) = ready {
                    if let Some(freed) = client.inner.surfaces.compose(surface, slot, payload) {
                        client.frames += 1;
                        to_app.push(ProtocolMsg::Frame(FrameMsg::FrameAck {
                            wid,
                            frame_id,
                            slot: freed,
                        }));
                    }
                }
            }
            // queue 唯一档（Welcome=Commands）——像素帧臂理论不到达；
            // 容错直通 BrokerClient::compose_pixels（不 ack 失败同族）。
            HostAction::ComposeFramePixels {
                surface,
                wid,
                frame_id,
                slot,
                revision,
                w,
                h,
                stride,
            } => {
                if let Some(ack) =
                    client.inner.compose_pixels(surface, wid, frame_id, slot, revision, w, h, stride)
                {
                    to_app.push(ProtocolMsg::Frame(ack));
                }
            }
            HostAction::ReclaimWindow { wid } => {
                if let Some(surface) = client.inner.wid_surface.remove(&wid) {
                    client.inner.shm.remove(&surface);
                    client.inner.surfaces.release(surface);
                    to_app.push(ProtocolMsg::Frame(FrameMsg::BufferRelease { surface }));
                }
                if client.inner.wid == Some(Wid(wid)) {
                    client.inner.wid = None;
                    client.inner.app_id = None;
                }
                events.push(RqEvent::Reclaimed { wid });
            }
            HostAction::ObserveUp { .. } => {
                // 观测上行：v1 不消费（桌面 MCP 代理线归桌面）。
            }
        }
    }
    (to_app, events)
}

/// 采纳单连接：泵到宿主 Active（Hello → Welcome/BufferAlloc 回发）。
/// 预算内未收敛返回 None（调用方弃置连接——broker_attach_one 同则）。
pub fn adopt_one(
    app_name: String,
    mut end: Box<dyn Transport + Send>,
    ids: &mut RqIds,
    budget_ms: u32,
) -> Option<(RqClient, Vec<RqEvent>)> {
    let mut client = RqClient {
        inner: BrokerClient::new(String::new(), end),
        app_name: app_name.clone(),
        title: app_name,
        width: 480.0,
        height: 320.0,
        window: None,
        frames: 0,
    };
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_millis(budget_ms as u64);
    let mut events = Vec::new();
    while std::time::Instant::now() < deadline {
        if client.inner.endpoint.state == HostState::Active {
            return Some((client, events));
        }
        match client.inner.end.try_recv() {
            Some(Ok(msg)) => {
                let actions = match client.inner.endpoint.on_message(msg) {
                    Ok(a) => a,
                    Err(_) => return None,
                };
                let (to_app, mut ev) = apply_actions(&mut client, ids, actions);
                events.append(&mut ev);
                for reply in to_app {
                    let _ = client.inner.end.send(&reply);
                }
            }
            Some(Err(_)) => return None,
            None => std::thread::sleep(std::time::Duration::from_millis(5)),
        }
    }
    None
}

/// 日常泵一轮：drain 已到达消息（帧合成/Ack、回收、上行），应答直发。
/// 返回 (泵事件, 存活)——EOF/协议错 = false（调用方回收窗与资源，
/// pump_broker_clients 回收臂同构）。
pub fn pump_client(client: &mut RqClient, ids: &mut RqIds) -> (Vec<RqEvent>, bool) {
    let mut events = Vec::new();
    let mut alive = true;
    loop {
        match client.inner.end.try_recv() {
            Some(Ok(msg)) => {
                let actions = match client.inner.endpoint.on_message(msg) {
                    Ok(a) => a,
                    Err(_) => {
                        alive = false;
                        break;
                    }
                };
                let (to_app, mut ev) = apply_actions(client, ids, actions);
                events.append(&mut ev);
                for reply in to_app {
                    let _ = client.inner.end.send(&reply);
                }
            }
            Some(Err(_)) => {
                alive = false;
                break;
            }
            None => {
                alive = alive && !client.inner.end.is_eof();
                break;
            }
        }
    }
    (events, alive)
}

/// 该客户端当前合成面（view/断言口——DrawListPainter 取帧点）。
pub fn composed(client: &RqClient) -> Option<&DrawList> {
    client.inner.composed()
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn pid_pipe(tag: &str) -> String {
        format!("{RQHOST_PIPE}-t{tag}-{}", std::process::id())
    }

    /// 最小 Hello（凭据即断言面——title/icon/wh）。
    fn hello(app_name: &str, title: &str) -> ProtocolMsg {
        ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: super::super::PROTOCOL_VERSION,
            app_name: app_name.into(),
            title: title.into(),
            icon: None,
            width: 480.0,
            height: 320.0,
            fonts: vec![],
        })
    }

    /// serve 环 + 客户端采纳 + Hello 握手泵到 Active（多测试共用序）。
    fn start_serve(pipe: &str) -> (Arc<RqServe>, transport::PipeClaim) {
        RqServe::start(pipe).expect("serve start")
    }

    /// T-02：rendezvous 往返 + 探测 ping 吞掉 + per-app 管道先行 listen。
    #[test]
    fn rendezvous_roundtrip_and_probe_swallow() {
        let pipe = pid_pipe("rv");
        let (serve, _claim) = start_serve(&pipe);

        // 探测 ping：连上即关——不占采纳名额，serve 环继续。
        drop(transport::connect(&pipe, 2000).expect("probe connect"));
        std::thread::sleep(std::time::Duration::from_millis(50));

        // 真实采纳：per-app 管道名回带 + 转连成功。
        let (app_pipe, _end) = adopt(&pipe, "counter", 2000).expect("adopt");
        assert!(app_pipe.contains("-app-"), "per-app 管道名 {app_pipe}");

        // 再探测一轮后仍可继续采纳（吞掉不杀伤 serve）。
        drop(transport::connect(&pipe, 2000).expect("probe 2"));
        std::thread::sleep(std::time::Duration::from_millis(50));
        let (app_pipe2, _) = adopt(&pipe, "counter", 2000).expect("adopt 2");
        assert_ne!(app_pipe, app_pipe2, "递增分配不重名");

        serve.stop(&pipe);
    }

    /// T-02：锁管道单实例仲裁——transport 层第二声明失败；serve 级
    /// `RqServe::start` 同名再启出 AlreadyRunning（干净退出信号）。
    #[test]
    fn lock_pipe_second_claim_fails() {
        let lock = pid_pipe("lock");
        let _first = transport::try_claim_pipe(&lock).expect("首声明");
        assert!(transport::try_claim_pipe(&lock).is_err(), "transport 层第二声明失败");
        drop(_first);
        assert!(transport::try_claim_pipe(&lock).is_ok(), "Drop 后名字让出");

        // serve 级：同名第二实例出 AlreadyRunning（daemon 主据此退 0）。
        let wellknown = pid_pipe("lock-s");
        let (_serve, _claim) = start_serve(&wellknown);
        match RqServe::start(&wellknown) {
            Err(RqServeError::AlreadyRunning) => {}
            Ok(_) => panic!("第二实例应被锁管道拒绝"),
            Err(RqServeError::Transport(e)) => panic!("意外传输错误: {e:?}"),
        }
    }

    /// T-02：客户端权威——未知 app 名照样采纳到 Active（零装载断言：
    /// 桌面 broker_apply_actions resolver MISS 静默弃连的反面）。
    #[test]
    fn adopt_unknown_app_reaches_active_without_load() {
        let pipe = pid_pipe("auth");
        let (serve, _claim) = start_serve(&pipe);

        let (_, mut app_end) = adopt(&pipe, "no-such-app-anywhere", 2000).expect("adopt");
        app_end.send(&hello("no-such-app-anywhere", "幻影 App")).unwrap();

        // 宿主侧收端点 + 泵到 Active。
        let (app_name, end) = {
            let deadline =
                std::time::Instant::now() + std::time::Duration::from_secs(3);
            loop {
                let got = serve.pending.lock().unwrap().pop();
                if let Some(x) = got {
                    break x;
                }
                assert!(std::time::Instant::now() < deadline, "pending 3s 未落");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };
        assert_eq!(app_name, "no-such-app-anywhere");

        let mut ids = RqIds::default();
        let (client, events) =
            adopt_one(app_name, end, &mut ids, 3000).expect("泵到 Active");
        assert_eq!(client.inner.endpoint.state, HostState::Active);
        // Welcome + BufferAlloc 已回发；app 侧消费即 Active。
        let welcome = app_end.recv_wait(2000).unwrap().unwrap();
        match welcome {
            ProtocolMsg::Handshake(HandshakeMsg::Welcome { frame_mode, .. }) => {
                assert_eq!(frame_mode, FrameMode::Commands, "queue 唯一档");
            }
            other => panic!("期待 Welcome: {other:?}"),
        }
        assert!(matches!(
            app_end.recv_wait(2000),
            Some(Ok(ProtocolMsg::Frame(FrameMsg::BufferAlloc { .. })))
        ));
        // 事件面：开窗凭据（title 空名回退 app_name）。
        assert!(events
            .iter()
            .any(|e| matches!(e, RqEvent::Adopted { title, width, height, .. }
                if *title == "幻影 App" && *width == 480.0 && *height == 320.0)));

        serve.stop(&pipe);
    }

    /// T-02：多客户端并发接纳——N 端点 N 表面，shm 段名不撞（surface
    /// 全局唯一自增，host.rs Plan 500 T3 同则）。
    #[test]
    fn multi_client_concurrent_adoption() {
        let pipe = pid_pipe("multi");
        let (serve, _claim) = start_serve(&pipe);
        let mut ids = RqIds::default();
        let mut surfaces = Vec::new();

        for i in 1..=3 {
            let (_, mut app_end) = adopt(&pipe, &format!("app{i}"), 2000).expect("adopt");
            app_end.send(&hello(&format!("app{i}"), &format!("t{i}"))).unwrap();
            let (_, end) = {
                let deadline =
                    std::time::Instant::now() + std::time::Duration::from_secs(3);
                loop {
                    let got = serve.pending.lock().unwrap().pop();
                    if let Some(x) = got {
                        break x;
                    }
                    assert!(std::time::Instant::now() < deadline, "pending {i} 3s 未落");
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            };
            let (client, _) = adopt_one(format!("app{i}"), end, &mut ids, 3000)
                .expect("泵到 Active");
            assert_eq!(client.inner.endpoint.state, HostState::Active);
            surfaces.push(client.inner.endpoint.surface.expect("surface 在册"));
        }
        // surface 句柄互异（shm 名 = pid-surface → 不撞段）。
        let mut uniq = surfaces.clone();
        uniq.sort_unstable();
        uniq.dedup();
        assert_eq!(uniq.len(), surfaces.len(), "surface 句柄不重");

        serve.stop(&pipe);
    }

    /// T-02：泵循环——FrameReady 合成回 Ack + 帧计数；ExitRequest →
    /// Reclaim → BufferRelease；EOF 判死。
    #[test]
    fn pump_frames_reclaim_and_eof() {
        let pipe = pid_pipe("pump");
        let (serve, _claim) = start_serve(&pipe);

        let (_, mut app_end) = adopt(&pipe, "counter", 2000).expect("adopt");
        app_end.send(&hello("counter", "计数器")).unwrap();
        let (_, end) = {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            loop {
                let got = serve.pending.lock().unwrap().pop();
                if let Some(x) = got {
                    break x;
                }
                assert!(std::time::Instant::now() < deadline, "pending 3s 未落");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };
        let mut ids = RqIds::default();
        let (mut client, _) =
            adopt_one("counter".into(), end, &mut ids, 3000).expect("泵到 Active");
        // app 消费 Welcome/BufferAlloc（握手收尾）。
        let _ = app_end.recv_wait(500);
        let _ = app_end.recv_wait(500);

        // 一帧（输入路由方向 = host→app，app→host 的 Input 会被宿主状态
        // 机拒收——输入闭环断言归 T-04/daemon 腿）。
        let wid = client.inner.wid.expect("Active 有 wid").0;
        app_end
            .send(&ProtocolMsg::Frame(FrameMsg::FrameReady {
                wid,
                frame_id: 1,
                slot: 1,
                damage: None,
                revision: 2,
                payload: DrawList::default(),
            }))
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            let (events, alive) = pump_client(&mut client, &mut ids);
            assert!(alive);
            assert!(events.is_empty(), "帧/输入不产泵事件: {events:?}");
            if client.frames >= 1 {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "帧未合成");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // FrameAck 归还槽。
        assert!(matches!(
            app_end.recv_wait(2000),
            Some(Ok(ProtocolMsg::Frame(FrameMsg::FrameAck { slot: 0, .. })))
        ));
        // 合成面可取（view 消费口）。
        assert!(composed(&client).is_some());

        // ExitRequest → ReclaimWindow → BufferRelease + 事件。
        app_end
            .send(&ProtocolMsg::Control(ControlMsg::ExitRequest { wid }))
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut reclaimed = false;
        loop {
            let (events, alive) = pump_client(&mut client, &mut ids);
            if events.iter().any(|e| matches!(e, RqEvent::Reclaimed { wid: w } if *w == wid)) {
                assert!(alive, "端点回 Listening 仍活");
                reclaimed = true;
                break;
            }
            assert!(std::time::Instant::now() < deadline, "回收事件未到");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(reclaimed);
        assert!(matches!(
            app_end.recv_wait(2000),
            Some(Ok(ProtocolMsg::Frame(FrameMsg::BufferRelease { .. })))
        ));

        // EOF 判死：对端关闭 → pump 报死（窗回收归 daemon 层）。
        drop(app_end);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            let (_, alive) = pump_client(&mut client, &mut ids);
            if !alive {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "EOF 未检出");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        serve.stop(&pipe);
    }

    /// T-02：ensure_rqhost 全序（注入孵化器替身——进程内起 serve 模拟
    /// spawn 延迟）：不在 → 孵化 → 退避 → 成。
    #[test]
    fn ensure_rqhost_spawns_with_backoff() {
        let pipe = pid_pipe("ensure");
        // well-known env 缝注入（测试隔离）。
        std::env::set_var(RQHOST_WELLKNOWN_ENV, &pipe);
        // 孵化器替身：延迟 300ms 起进程内 serve（模拟 daemon 启动就绪窗）。
        let spawn_pipe = pipe.clone();
        let spawner: RqhostSpawner = Arc::new(move |_wellknown: &str| {
            let pipe = spawn_pipe.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(300));
                // serve 不停——本测试末尾统一停；句柄泄漏进线程无害
                // （进程随测试退出）。
                let _ = RqServe::start(&pipe);
            });
            Ok(())
        });
        let got = ensure_rqhost_with(&spawner, "counter").expect("ensure");
        assert!(got.contains("-app-"), "per-app 管道 {got}");
        std::env::remove_var(RQHOST_WELLKNOWN_ENV);
        // 清理：探测连接使 serve 环退出难（句柄在线程内）——留进程退出
        // 回收（单测进程隔离，管道名 pid 后缀不串扰）。
    }
}
