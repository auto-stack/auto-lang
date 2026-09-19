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
    ControlMsg, DrawList, FrameMode, FrameMsg, HandshakeMsg, InputMsg, MouseButton, ProtocolMsg,
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

/// 发现与孵化序（G6/D2；T-05 测试暴露的设计修正：**不在此采纳**——
/// per-app 管道实例一次性，ensure 若连后即弃会烧掉实例，真正客户端
/// 转连即死管。本函数只保证 daemon 在线）：直试探测（连上即关——
/// serve 环吞 ping）→ 不在 spawn `auto rqhost` → 就绪退避重试（100ms
/// 起倍增，预算 15s）。rendezvous 采纳由各轨客户端自办（vm 轨 = lib.rs
/// 分岔 `ClientTarget::Rqhost` 内建；rust 轨 = 子进程生成 gate）。
pub fn ensure_rqhost_ready() -> Result<(), String> {
    ensure_ready_with(&{
        let spawner: RqhostSpawner = Arc::new(spawn_rqhost_default);
        spawner
    })
}

/// [`ensure_rqhost_ready`] 的可测性缝（孵化器注入——测试以进程内
/// `RqServe` 替身覆盖 spawn+就绪全序，零真实进程）。
pub fn ensure_ready_with(spawner: &RqhostSpawner) -> Result<(), String> {
    let wellknown = wellknown_pipe();
    // 直试探测（短超时：daemon 在 = 秒成；不在 = ~500ms 内失败）。
    if transport::connect(&wellknown, 500).is_ok() {
        return Ok(());
    }
    spawner(&wellknown).map_err(|e| format!("spawn rqhost 失败: {e}"))?;
    // 就绪退避：connect 内建 FILE_NOT_FOUND 重试，间隔即单次超时预算。
    let started = std::time::Instant::now();
    let mut backoff_ms: u32 = 100;
    loop {
        if transport::connect(&wellknown, backoff_ms).is_ok() {
            return Ok(());
        }
        if started.elapsed() >= std::time::Duration::from_secs(15) {
            return Err("rqhost 孵化后 15s 未就绪（探测重试预算耗尽）".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(backoff_ms as u64 / 4));
        backoff_ms = (backoff_ms * 2).min(1600);
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

/// 解析采纳记录（探测 ping/异记录 = None 吞掉重听）。双动词兼容
/// （T-05 设计修正）：`adopt␟<name>`（rqhost 本尊——queue 唯一档）与
/// broker 族 `incubate␟<name>␟<mode>`（旧生成物 `--autodesk-incubate`
/// 直连 rqhost 零改接驳；mode 字段忽略——pixels 诉求记观测行，宿主
/// 仍按 Welcome=Commands 定档）。
fn parse_adopt_record(msg: &ProtocolMsg) -> Option<String> {
    let ProtocolMsg::Control(ControlMsg::DesktopBus { record, .. }) = msg else {
        return None;
    };
    let mut parts = record.split('\u{1f}');
    match (parts.next(), parts.next()) {
        (Some("adopt"), Some(name)) | (Some("incubate"), Some(name))
            if !name.is_empty() =>
        {
            Some(name.to_string())
        }
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

    /// 停机旗标已置位（P031-R3：末窗退出门的单测观测口——rq_update
    /// 经 `iced::exit` 收尾前先置本旗标）。
    pub fn stop_requested(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
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
    /// 最近帧 revision（前进时打观测行——输入驱动帧变的 e2e 断言锚点）。
    pub last_revision: u64,
    /// 首帧观测行已打（e2e 断言锚点——[rqhost] first frame）。
    first_frame_observed: bool,
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

/// 帧版本观测（P031-R2 e2e 断言锚点）：revision 前进 = 客户端内容
/// 变化（输入驱动帧变的宿主侧证据）；仅变化时打行——静态 app 零噪声。
fn note_revision(client: &mut RqClient, revision: u64) {
    if revision != client.last_revision {
        eprintln!("[rqhost] revision `{}` {revision}", client.title);
        client.last_revision = revision;
    }
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
            HostAction::ComposeFrame { surface, wid, frame_id, slot, revision, payload, .. } => {
                if let Some(freed) = client.inner.surfaces.compose(surface, slot, payload) {
                    client.frames += 1;
                    note_revision(client, revision);
                    to_app.push(ProtocolMsg::Frame(FrameMsg::FrameAck {
                        wid,
                        frame_id,
                        slot: freed,
                    }));
                }
            }
            HostAction::ComposeFrameShared { surface, wid, frame_id, slot, revision, .. } => {
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
                        note_revision(client, revision);
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
            // PLAN-030 增臂（master 调和补面）：desktop.* 命令上行——
            // 执行体是桌面宿主的 DesktopBus broker 臂；rqhost 客户端为
            // 普通 app（非特权 shell），v1 不执行仅观测行留痕。
            HostAction::DesktopBus { record, .. } => {
                eprintln!("[rqhost] desktop-bus 上行（普通 app 不执行）: {record}");
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
        last_revision: 0,
        first_frame_observed: false,
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
// -q vm 轨客户端分岔（T-05/D6）：装载链零改
// ---------------------------------------------------------------------------

/// vm 轨 -q 客户端（`auto run -r vm -q`）：run_vm_ui 的 CWD/主题/后端序
/// 上游照常执行后，本函数替代 `run_dynamic_iced` 的"自开 OS 窗"——组件
/// 原地转 rqhost 客户端（AppProjector 产帧，exit-on-EOF 档）。
/// rendezvous 采纳内建（`ClientTarget::Rqhost`——不预连不烧管道实例）；
/// Hello 凭据：title = `AUTO_VM_TITLE` env 缺省 widget 名；尺寸 =
/// `AUTO_VM_WINDOW` 解析（[`vm_window_size`]）。
pub fn run_vm_rqhost_client(
    component: crate::ui::dynamic::DynamicComponent,
    wellknown: &str,
) -> Result<String, String> {
    use super::client_entry::{self, ClientOpts, ClientTarget};
    use super::message::FrameMode;
    let app_name = component.widget_name().to_string();
    let title = std::env::var("AUTO_VM_TITLE").unwrap_or_else(|_| app_name.clone());
    let (width, height) = vm_window_size();
    let opts = ClientOpts {
        app_name: app_name.clone(),
        title,
        width,
        height,
        frame_mode: FrameMode::Commands,
        auto_downgraded: false,
    };
    let target = ClientTarget::Rqhost { wellknown: wellknown.to_string(), app_name };
    client_entry::run_dynamic_client(component, opts, target)
        .map(|_| "rqhost client exited".to_string())
}

/// `AUTO_VM_WINDOW` 解析（renderer.rs `startup_window_size` 同式边界
/// 校验；缺席/非法/"fit" = 480×320——rqhost 档 broker 子缺省，D6）。
pub fn vm_window_size() -> (f32, f32) {
    if let Ok(spec) = std::env::var("AUTO_VM_WINDOW") {
        if let Some((w, h)) = spec.trim().split_once(['x', 'X']) {
            if let (Ok(w), Ok(h)) = (w.trim().parse::<f32>(), h.trim().parse::<f32>()) {
                if w >= 200.0 && h >= 200.0 && w <= 7680.0 && h <= 4320.0 {
                    return (w, h);
                }
            }
        }
    }
    (480.0, 320.0)
}

// ---------------------------------------------------------------------------
// daemon 装配（T-03）：iced 多窗宿主 + 泵循环 + 末窗退出
// ---------------------------------------------------------------------------

/// daemon 消息面（listen_with 事件带发生窗——D4 路由键）。
#[derive(Debug, Clone)]
pub enum RqMessage {
    /// 帧泵节拍（15ms：pending 消费 + 日常泵；桌面 400ms ServiceTick 对
    /// 原生窗输入→帧响应太钝，D3 定案）。
    Tick,
    /// OS 窗 resize → `FrameMsg::Resize` 下发客户端重排（AC-06）。
    WindowResized { window: iced::window::Id, width: f32, height: f32 },
    /// OS 窗关闭：宿主 Close 下发（app ExitRequest → Reclaim 既有状态机，
    /// 退出码 0）；末窗 = daemon 退出（iced 空窗不自动退出的反面，D5）。
    WindowClosed { window: iced::window::Id },
    /// live 输入（键盘/滚轮/IME——029 映射族产物；发生窗随行）。
    Live { window: iced::window::Id, input: crate::ui::session::LiveInput },
    /// 光标移动（窗口本地坐标 = 表面坐标，D4；也是按下事件的坐标源）。
    CursorMoved { window: iced::window::Id, x: f32, y: f32 },
    /// 指针按下/释放（坐标取 `last_cursor` 簿记——Button 事件不带位）。
    PointerPressed { window: iced::window::Id, button: iced::mouse::Button },
    PointerReleased { window: iced::window::Id, button: iced::mouse::Button },
}

/// daemon 状态：客户端表（窗注册表即 `client.window`）+ serve 柄。
pub struct RqDaemon {
    pub serve: Arc<RqServe>,
    /// 锁管道守卫——与 daemon 同寿命（Drop = 让出单实例声明）。
    pub claim: Option<transport::PipeClaim>,
    pub wellknown: String,
    pub ids: RqIds,
    pub clients: Vec<RqClient>,
    /// 曾开过窗（末窗退出门——boot 零窗待命不退）。
    had_window: bool,
    /// 开窗级联序（多窗不叠死）。
    opened: u64,
    /// 窗 → 最近光标位（窗口本地坐标；按下事件的坐标源——Button 事件
    /// 不带位，WM `last_cursor` 簿记同型）。
    last_cursor: BTreeMap<iced::window::Id, (f32, f32)>,
}

impl RqDaemon {
    /// 按发生窗找客户端（输入/resize/关窗路由键——D4）。
    fn client_of(&mut self, window: iced::window::Id) -> Option<&mut RqClient> {
        self.clients.iter_mut().find(|c| c.window == Some(window))
    }

    /// 该窗标题（`.title` 装配面）。
    pub fn title_of(&self, window: iced::window::Id) -> String {
        self.clients
            .iter()
            .find(|c| c.window == Some(window))
            .map(|c| c.title.clone())
            .unwrap_or_else(|| "rqhost".to_string())
    }
}

/// 开窗（Adopted 事件消费）：Hello 凭据 → OS 窗（级联偏移防叠死）。
/// 登记即刻生效（`window::open` 同步返回 Id）；Task 随 update 返回派发。
fn open_window_for(client: &mut RqClient, cascade: u64) -> iced::Task<RqMessage> {
    let (win_id, task) = iced::window::open(iced::window::Settings {
        size: iced::Size::new(client.width.max(160.0), client.height.max(120.0)),
        position: iced::window::Position::Specific(iced::Point::new(
            80.0 + 36.0 * (cascade % 10) as f32,
            80.0 + 36.0 * (cascade % 10) as f32,
        )),
        ..Default::default()
    });
    client.window = Some(win_id);
    eprintln!(
        "[rqhost] window opened for `{}` ({:.0}x{:.0})",
        client.app_name, client.width, client.height
    );
    task.map(|_| RqMessage::Tick)
}

/// iced 鼠标按钮 → 协议按钮。
fn wire_button(b: iced::mouse::Button) -> MouseButton {
    match b {
        iced::mouse::Button::Left => MouseButton::Left,
        iced::mouse::Button::Right => MouseButton::Right,
        iced::mouse::Button::Middle => MouseButton::Middle,
        _ => MouseButton::Left,
    }
}

/// live 输入 → 协议 InputMsg（029 六型逐映射；`broker_*` 路由族语义的
/// 纯函数化——桌面按焦点窗/hit_test 选窗，rqhost 按发生窗，映射同源）。
fn live_input_msgs(wid: u64, input: &crate::ui::session::LiveInput) -> Vec<InputMsg> {
    use crate::ui::session::LiveInput;
    match input {
        LiveInput::KeyPressed { key, modifiers } => {
            vec![InputMsg::KeyPressed { wid, key: *key, modifiers: *modifiers }]
        }
        LiveInput::Chars { text } => text
            .chars()
            .map(|ch| InputMsg::CharTyped { wid, ch })
            .collect(),
        LiveInput::ImeCommit { text } => {
            vec![InputMsg::ImeCommit { wid, text: text.clone() }]
        }
        LiveInput::ImePreedit { text } => vec![InputMsg::ImePreedit {
            wid,
            text: text.clone(),
            cursor: super::message::WRect::new(0.0, 0.0, 0.0, 0.0),
        }],
        LiveInput::ImeCancelled => vec![InputMsg::ImeCancelled { wid }],
        LiveInput::Wheel { dx, dy } => vec![InputMsg::Scroll { wid, dx: *dx, dy: *dy }],
    }
}

/// daemon update：Tick（采纳+泵）/ resize / 关窗 / 输入路由（D4）。
fn rq_update(state: &mut RqDaemon, msg: RqMessage) -> iced::Task<RqMessage> {
    match msg {
        RqMessage::Tick => {
            let mut tasks = Vec::new();
            // ① 待定采纳消费（serve 线程生产——队列 drain）。
            let pending: Vec<_> = state.serve.pending.lock().unwrap().drain(..).collect();
            for (name, end) in pending {
                match adopt_one(name, end, &mut state.ids, 2000) {
                    Some((mut client, events)) => {
                        for ev in events {
                            if matches!(ev, RqEvent::Adopted { .. }) {
                                state.opened += 1;
                                state.had_window = true;
                                tasks.push(open_window_for(&mut client, state.opened));
                            }
                        }
                        state.clients.push(client);
                    }
                    None => eprintln!("[rqhost] adoption 未达 Active（预算耗尽弃置）"),
                }
            }
            // ② 日常泵（帧合成/回收；EOF 判死）。
            let mut dead: Vec<usize> = Vec::new();
            for idx in 0..state.clients.len() {
                let (events, alive) = pump_client(&mut state.clients[idx], &mut state.ids);
                for ev in events {
                    match ev {
                        RqEvent::Adopted { .. } => {
                            state.opened += 1;
                            state.had_window = true;
                            tasks.push(open_window_for(&mut state.clients[idx], state.opened));
                        }
                        RqEvent::Reclaimed { .. } => {
                            if let Some(w) = state.clients[idx].window.take() {
                                tasks.push(iced::window::close(w));
                            }
                        }
                    }
                }
                // 首帧观测行（e2e 断言锚点：帧已到宿主并合成）。
                let client = &mut state.clients[idx];
                if client.frames >= 1 && !client.first_frame_observed {
                    client.first_frame_observed = true;
                    eprintln!("[rqhost] first frame `{}`", client.app_name);
                }
                if !alive {
                    dead.push(idx);
                }
            }
            for idx in dead.into_iter().rev() {
                let mut client = state.clients.remove(idx);
                if let Some(w) = client.window.take() {
                    tasks.push(iced::window::close(w));
                }
                eprintln!("[rqhost] client `{}` 断连（EOF）——窗回收", client.app_name);
            }
            iced::Task::batch(tasks)
        }
        RqMessage::WindowResized { window, width, height } => {
            if let Some(client) = state.client_of(window) {
                client.width = width;
                client.height = height;
                if let Some(surface) = client.inner.endpoint.surface {
                    let _ = client.inner.end.send(&ProtocolMsg::Frame(FrameMsg::Resize {
                        surface,
                        width,
                        height,
                    }));
                    // 观测行（e2e resize 腿断言锚点——AC-06）。
                    eprintln!(
                        "[rqhost] window `{}` resized {width:.0}x{height:.0}",
                        client.app_name
                    );
                }
            }
            iced::Task::none()
        }
        RqMessage::WindowClosed { window } => {
            // 用户关窗 → 宿主 Close 下发（app ExitRequest → Reclaim →
            // 退出码 0——端点状态机既有，D5②）。
            if let Some(client) = state.client_of(window) {
                client.window = None;
                if let Ok(close) = client.inner.endpoint.close() {
                    let _ = client.inner.end.send(&close);
                }
            }
            // 末窗退出（D5①）：无在册窗 + 无待定采纳 + 曾开过窗。
            let no_windows = state.clients.iter().all(|c| c.window.is_none());
            let no_pending = state.serve.pending.lock().unwrap().is_empty();
            if no_windows && no_pending && state.had_window {
                eprintln!("[rqhost] 末窗关闭——daemon 退出");
                state.serve.stop(&state.wellknown);
                return iced::exit();
            }
            iced::Task::none()
        }
        RqMessage::Live { window, input } => {
            if let Some(client) = state.client_of(window) {
                if let Some(wid) = client.inner.wid.map(|w| w.0) {
                    for msg in live_input_msgs(wid, &input) {
                        let _ = client.inner.end.send(&ProtocolMsg::Input(msg));
                    }
                }
            }
            iced::Task::none()
        }
        RqMessage::CursorMoved { window, x, y } => {
            state.last_cursor.insert(window, (x, y));
            if let Some(client) = state.client_of(window) {
                if let Some(wid) = client.inner.wid.map(|w| w.0) {
                    let _ = client.inner.end.send(&ProtocolMsg::Input(
                        InputMsg::PointerMoved { wid, x, y },
                    ));
                }
            }
            iced::Task::none()
        }
        RqMessage::PointerPressed { window, button } => {
            let at = state.last_cursor.get(&window).copied().unwrap_or((0.0, 0.0));
            if let Some(client) = state.client_of(window) {
                if let Some(wid) = client.inner.wid.map(|w| w.0) {
                    let _ = client.inner.end.send(&ProtocolMsg::Input(
                        InputMsg::PointerPressed {
                            wid,
                            button: wire_button(button),
                            x: at.0,
                            y: at.1,
                            modifiers: 0,
                        },
                    ));
                }
            }
            iced::Task::none()
        }
        RqMessage::PointerReleased { window, button } => {
            let at = state.last_cursor.get(&window).copied().unwrap_or((0.0, 0.0));
            if let Some(client) = state.client_of(window) {
                if let Some(wid) = client.inner.wid.map(|w| w.0) {
                    let _ = client.inner.end.send(&ProtocolMsg::Input(
                        InputMsg::PointerReleased {
                            wid,
                            button: wire_button(button),
                            x: at.0,
                            y: at.1,
                            modifiers: 0,
                        },
                    ));
                }
            }
            iced::Task::none()
        }
    }
}

/// daemon view：按窗路由——注册表命中 = DrawListPainter 栅格化当前
/// 合成面；未登记窗 = 占位（renderer.rs view_desktop_fn 先例，D3）。
fn rq_view(state: &RqDaemon, window: iced::window::Id) -> iced::Element<'_, RqMessage> {
    let frame = state
        .clients
        .iter()
        .find(|c| c.window == Some(window))
        .and_then(composed);
    match frame {
        Some(list) => crate::ui::iced::broker_surface::drawlist_element(list),
        None => iced::widget::container(
            iced::widget::text("[rqhost] 窗口未登记").size(14),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center(iced::Length::Fill)
        .into(),
    }
}

/// daemon 订阅：15ms 帧泵 + 窗事件流 + 输入流（键盘/滚轮/IME Ignored
/// 门 + 指针全事件——desktop_window_events 029 族同源直调，D4）。
fn rq_subscription(_state: &RqDaemon) -> iced::Subscription<RqMessage> {
    use crate::ui::session::{
        live_input_from_input_method, live_input_from_wheel, live_inputs_from_keyboard,
    };
    iced::Subscription::batch(vec![
        iced::time::every(std::time::Duration::from_millis(15)).map(|_| RqMessage::Tick),
        iced::event::listen_with(|e, status, window_id| match e {
            iced::Event::Window(iced::window::Event::Resized(size)) => Some(
                RqMessage::WindowResized { window: window_id, width: size.width, height: size.height },
            ),
            iced::Event::Window(iced::window::Event::Closed) => {
                Some(RqMessage::WindowClosed { window: window_id })
            }
            // live 输入三族（Ignored 门——Captured = 宿主真 widget 已消费；
            // rqhost 窗内容 = canvas 无交互 widget，稳态恒 Ignored）。
            iced::Event::Keyboard(kb) if status == iced::event::Status::Ignored => {
                live_inputs_from_keyboard(&kb).into_iter().next().map(|input| {
                    RqMessage::Live { window: window_id, input }
                })
            }
            iced::Event::InputMethod(im) if status == iced::event::Status::Ignored => {
                live_input_from_input_method(&im)
                    .map(|input| RqMessage::Live { window: window_id, input })
            }
            iced::Event::Mouse(iced::mouse::Event::WheelScrolled { delta })
                if status == iced::event::Status::Ignored =>
            {
                live_input_from_wheel(&delta)
                    .map(|input| RqMessage::Live { window: window_id, input })
            }
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                Some(RqMessage::CursorMoved {
                    window: window_id,
                    x: position.x,
                    y: position.y,
                })
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(button)) => {
                Some(RqMessage::PointerPressed { window: window_id, button })
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(button)) => {
                Some(RqMessage::PointerReleased { window: window_id, button })
            }
            _ => None,
        }),
    ])
}

/// rqhost daemon 主入口（`auto rqhost` 子命令消费）。
///
/// 单实例仲裁：锁管道被占（已有实例）→ 观测行 + 干净退出（码 0）。
/// 阻塞直至末窗退出（iced 空窗不自动退出——`iced::exit` 自建，D5）。
pub fn run_daemon(wellknown: &str) -> Result<(), String> {
    let (serve, claim) = match RqServe::start(wellknown) {
        Ok(x) => x,
        Err(RqServeError::AlreadyRunning) => {
            eprintln!("[rqhost] 已有实例在服（{wellknown}-lock 被占）——第二实例退出");
            return Ok(());
        }
        Err(RqServeError::Transport(e)) => {
            return Err(format!("rqhost serve 启动失败: {e:?}"));
        }
    };
    eprintln!("[rqhost] serving on {wellknown}");
    let state = RqDaemon {
        serve,
        claim: Some(claim),
        wellknown: wellknown.to_string(),
        ids: RqIds::default(),
        clients: Vec::new(),
        had_window: false,
        opened: 0,
        last_cursor: BTreeMap::new(),
    };
    // boot 闭包 Fn 约束——RefCell 一次性提取（renderer.rs run_session 同型）。
    let init = std::cell::RefCell::new(Some(state));
    let boot = move || -> (RqDaemon, iced::Task<RqMessage>) {
        let state = init
            .borrow_mut()
            .take()
            .expect("boot should only be called once");
        (state, iced::Task::none())
    };
    iced::daemon(boot, rq_update, rq_view)
        .title(|state: &RqDaemon, window| state.title_of(window))
        .subscription(rq_subscription)
        .run()
        .map_err(|e| format!("rqhost daemon 退出异常: {e:?}"))
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
            surfaces: Vec::new(),
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

    /// T-02/T-05：ensure 就绪全序（注入孵化器替身——进程内延迟起 serve
    /// 模拟 spawn 就绪窗）：不在 → 孵化 → 探活退避 → 就绪。不采纳——
    /// per-app 管道实例一次性，ensure 只保 daemon 在线（设计修正实证：
    /// 采纳后弃端 = 烧实例，真客户端转连即死管）。
    #[test]
    fn ensure_ready_spawns_with_backoff() {
        let pipe = pid_pipe("ensure");
        std::env::set_var(RQHOST_WELLKNOWN_ENV, &pipe);
        let spawn_pipe = pipe.clone();
        let spawner: RqhostSpawner = Arc::new(move |_wellknown: &str| {
            let pipe = spawn_pipe.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(300));
                let _ = RqServe::start(&pipe);
            });
            Ok(())
        });
        ensure_ready_with(&spawner).expect("ensure ready");
        // 探活直连成功（daemon 在线；serve 线程留进程退出回收）。
        assert!(transport::connect(&pipe, 2000).is_ok());
        std::env::remove_var(RQHOST_WELLKNOWN_ENV);
    }

    /// T-03 集成：rqhost 装配 ×真客户端泵全循环（`native_client_full_
    /// cycle_over_pipe` 形态）——rendezvous 采纳 → Hello/Welcome/
    /// BufferAlloc → 帧 → resize 下发 → 宿主 Close → ExitRequest →
    /// BufferRelease → ClientExit::Closed（关窗 = app 退出码 0 的协议级
    /// 前身，D5②）。
    #[test]
    fn rqhost_full_cycle_over_pipe() {
        use crate::ui::desktop_protocol::client_runtime::{
            AppProjector, ClientConfig, ClientExit, ClientPump,
        };

        const SRC: &str = "widget RqCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n        text `count: ${.count}`\n    }\n}\n";
        let pipe = pid_pipe("cycle");
        let (serve, _claim) = start_serve(&pipe);

        // app 侧线程：采纳 → ClientPump（exit-on-EOF 档 = reconnect None）。
        let client_pipe = pipe.clone();
        let app = std::thread::spawn(move || {
            let (_, app_end) = adopt(&client_pipe, "rq-counter", 2000).expect("adopt");
            let component = crate::build_dynamic_component(SRC, None).expect("build");
            let projector = AppProjector::new(component, 480.0, 320.0);
            let config = ClientConfig {
                app_name: "rq-counter".into(),
                title: "rq-counter".into(),
                width: 480.0,
                height: 320.0,
            };
            let (exit, _p) = ClientPump::new(app_end, projector, config, None).run();
            exit
        });

        // 宿主侧：等 pending → 采纳到 Active。
        let (name, end) = {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                let got = serve.pending.lock().unwrap().pop();
                if let Some(x) = got {
                    break x;
                }
                assert!(std::time::Instant::now() < deadline, "pending 5s 未落");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };
        assert_eq!(name, "rq-counter");
        let mut ids = RqIds::default();
        let (mut client, events) =
            adopt_one(name, end, &mut ids, 3000).expect("泵到 Active");
        assert!(events.iter().any(|e| matches!(e, RqEvent::Adopted { .. })));

        // 帧合成：ClientPump 首帧（BufferAlloc 后自动产）→ pump 收 FrameAck
        // 回发 → 客户端持续供帧；宿主侧到 frames ≥ 1。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let (events, alive) = pump_client(&mut client, &mut ids);
            assert!(alive, "客户端泵中死亡");
            assert!(events.is_empty(), "稳态无事件: {events:?}");
            if client.frames >= 1 {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "首帧未到");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(composed(&client).is_some(), "合成面可取");

        // resize 下发（AC-06 协议级腿）：客户端 on_control 消费不炸、
        // 后续帧持续（AppProjector 重排）。
        let surface = client.inner.endpoint.surface.expect("surface");
        client
            .inner
            .end
            .send(&ProtocolMsg::Frame(FrameMsg::Resize {
                surface,
                width: 640.0,
                height: 400.0,
            }))
            .expect("resize 下发");
        std::thread::sleep(std::time::Duration::from_millis(60));
        let (_, alive) = pump_client(&mut client, &mut ids);
        assert!(alive, "resize 后客户端存活");

        // 宿主 Close（用户关窗宿主侧动作，D5②）→ app ExitRequest →
        // Reclaim → BufferRelease → ClientExit::Closed。
        let close = client.inner.endpoint.close().expect("close 产出");
        client.inner.end.send(&close).expect("close 下发");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut reclaimed = false;
        loop {
            let (events, alive) = pump_client(&mut client, &mut ids);
            if events
                .iter()
                .any(|e| matches!(e, RqEvent::Reclaimed { .. }))
            {
                let _ = alive; // 端点回 Listening，连接仍开
                reclaimed = true;
                break;
            }
            assert!(std::time::Instant::now() < deadline, "回收握手 5s 未收敛");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(reclaimed, "ReclaimWindow 落地");
        let exit = app.join().expect("app 线程");
        assert_eq!(exit, ClientExit::Closed, "关窗 = 干净退出（码 0 语义）");

        serve.stop(&pipe);
    }

    /// T-04：输入按窗路由不串扰——双客户端双窗，A 窗键盘/指针只达
    /// A 的连接（wid 随行正确）；B 端静默。
    #[test]
    fn input_routes_by_window_without_crosstalk() {
        use crate::ui::session::LiveInput;

        let pipe = pid_pipe("route");
        let (serve, _claim) = start_serve(&pipe);

        // 双客户端采纳到 Active（真实管道双端在手）。
        let mut ends = Vec::new();
        for name in ["alpha", "beta"] {
            let (_, mut app_end) = adopt(&pipe, name, 2000).expect("adopt");
            app_end.send(&hello(name, name)).unwrap();
            ends.push(app_end);
        }
        let mut state = RqDaemon {
            serve: Arc::clone(&serve),
            claim: None,
            wellknown: pipe.clone(),
            ids: RqIds::default(),
            clients: Vec::new(),
            had_window: false,
            opened: 0,
            last_cursor: BTreeMap::new(),
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while state.clients.len() < 2 {
            let pending: Vec<_> = state.serve.pending.lock().unwrap().drain(..).collect();
            for (name, end) in pending {
                if let Some((client, _)) = adopt_one(name, end, &mut state.ids, 3000) {
                    state.clients.push(client);
                }
            }
            assert!(std::time::Instant::now() < deadline, "双客户端 5s 未落地");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // 手工挂窗（daemon update 的开窗任务在此测试内不执行——路由键
        // 只看注册表，不依赖真窗）。
        let win_a = iced::window::Id::unique();
        let win_b = iced::window::Id::unique();
        state.clients[0].window = Some(win_a);
        state.clients[1].window = Some(win_b);
        let wid_a = state.clients[0].inner.wid.unwrap().0;
        let wid_b = state.clients[1].inner.wid.unwrap().0;
        assert_ne!(wid_a, wid_b, "wid 分配互异");

        // 消费握手回包（Welcome/BufferAlloc——防误读为输入）。
        for end in ends.iter_mut() {
            let _ = end.recv_wait(500);
            let _ = end.recv_wait(500);
        }

        // A 窗输入四型：光标 → 按下 → 键 → 字符串。
        let _ = rq_update(
            &mut state,
            RqMessage::CursorMoved { window: win_a, x: 12.0, y: 34.0 },
        );
        let _ = rq_update(
            &mut state,
            RqMessage::PointerPressed { window: win_a, button: iced::mouse::Button::Left },
        );
        let _ = rq_update(
            &mut state,
            RqMessage::Live {
                window: win_a,
                input: LiveInput::KeyPressed { key: 13, modifiers: 1 },
            },
        );
        let _ = rq_update(
            &mut state,
            RqMessage::Live {
                window: win_a,
                input: LiveInput::Chars { text: "hi".into() },
            },
        );

        // A 端按序收四组 Input（wid 全 = wid_a；按下坐标 = 光标簿记）。
        let expect = vec![
            (InputMsg::PointerMoved { wid: wid_a, x: 12.0, y: 34.0 }),
            (InputMsg::PointerPressed {
                wid: wid_a,
                button: MouseButton::Left,
                x: 12.0,
                y: 34.0,
                modifiers: 0,
            }),
            (InputMsg::KeyPressed { wid: wid_a, key: 13, modifiers: 1 }),
            (InputMsg::CharTyped { wid: wid_a, ch: 'h' }),
            (InputMsg::CharTyped { wid: wid_a, ch: 'i' }),
        ];
        for want in expect {
            let got = ends[0].recv_wait(2000).expect("A 端应收到").expect("解码");
            match (got, want) {
                (ProtocolMsg::Input(g), w) => assert_eq!(g, w),
                other => panic!("期待 Input: {other:?}"),
            }
        }
        // B 端静默（200ms 无串扰到达）。
        assert!(ends[1].recv_wait(200).is_none(), "B 端不应收到 A 窗输入");

        serve.stop(&pipe);
    }

    /// T-07/P031-R3：末窗退出门单测——rq_update 真行为驱动（WindowClosed
    /// 消息），三负一正：双窗在册不退 / 单窗在册不退 / 曾有窗但有待定
    /// 采纳不退 / 末窗且无待定且曾有窗 → 停机旗标置位（iced::exit 的
    /// 可观测前身）。
    #[test]
    fn last_window_close_exits_daemon() {
        let pipe = pid_pipe("lastwin");
        let (serve, _claim) = start_serve(&pipe);

        // 采纳双客户端到 Active 并手工挂窗。
        let mut ends = Vec::new();
        for name in ["alpha", "beta"] {
            let (_, mut app_end) = adopt(&pipe, name, 2000).expect("adopt");
            app_end.send(&hello(name, name)).unwrap();
            ends.push(app_end);
        }
        let mut state = RqDaemon {
            serve: Arc::clone(&serve),
            claim: None,
            wellknown: pipe.clone(),
            ids: RqIds::default(),
            clients: Vec::new(),
            had_window: false,
            opened: 0,
            last_cursor: BTreeMap::new(),
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while state.clients.len() < 2 {
            let pending: Vec<_> = state.serve.pending.lock().unwrap().drain(..).collect();
            for (name, end) in pending {
                if let Some((client, _)) = adopt_one(name, end, &mut state.ids, 3000) {
                    state.clients.push(client);
                }
            }
            assert!(std::time::Instant::now() < deadline, "双客户端 5s 未落地");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let win_a = iced::window::Id::unique();
        let win_b = iced::window::Id::unique();
        state.clients[0].window = Some(win_a);
        state.clients[1].window = Some(win_b);
        // 手工挂窗绕过了 open_window_for——had_window 需同刻置位（真
        // 路径由开窗臂维护）。
        state.had_window = true;

        // ① 双窗在册：关 A 不退。
        let _ = rq_update(&mut state, RqMessage::WindowClosed { window: win_a });
        assert!(!state.serve.stop_requested(), "双窗在册不退");
        // ② 单窗在册：关 B 前不退（① 已闭 A）——此刻关 B = 末窗。
        //    先注入待定采纳占位验证 ③：pending 非空 → 不退。
        state.serve.pending.lock().unwrap().push({
            // 真实在途采纳占位（adopt 已转连未 Hello——per-adoption 线程
            // 交付形态；不发 Hello 即驻 pending）。
            let (_, end) = adopt(&pipe, "gamma", 2000).expect("adopt gamma");
            ("gamma".into(), end)
        });
        let _ = rq_update(&mut state, RqMessage::WindowClosed { window: win_b });
        assert!(!state.serve.stop_requested(), "有待定采纳不退（关 B 暂缓退出）");
        // ④ 待定清空 → 再无在册窗（A/B 均已 Closed）→ 下一轮关窗语义：
        //    直接驱动门（再关一次任意已闭窗不触发——门看全局态，本腿
        //    以清空 pending 后驱动一次 A（已闭窗）验证全局态判定）。
        state.serve.pending.lock().unwrap().clear();
        let _ = rq_update(&mut state, RqMessage::WindowClosed { window: win_a });
        assert!(
            state.serve.stop_requested(),
            "无在册窗∧无待定∧曾有窗 → 停机旗标置位（末窗退出）"
        );

        // 负例：从未开过窗（boot 待命）——独立小现场。
        let pipe2 = pid_pipe("lastwin2");
        let (serve2, _claim2) = start_serve(&pipe2);
        let mut state2 = RqDaemon {
            serve: Arc::clone(&serve2),
            claim: None,
            wellknown: pipe2.clone(),
            ids: RqIds::default(),
            clients: Vec::new(),
            had_window: false,
            opened: 0,
            last_cursor: BTreeMap::new(),
        };
        let ghost = iced::window::Id::unique();
        let _ = rq_update(&mut state2, RqMessage::WindowClosed { window: ghost });
        assert!(
            !state2.serve.stop_requested(),
            "曾有窗=false（boot 零窗待命）不退"
        );
        serve2.stop(&pipe2);
        serve.stop(&pipe);
    }

    /// T-05：策略档选择（I2 断言面）——Rqhost=None（exit-on-EOF）；
    /// Direct/Broker = 30s/50ms 既有默认不变。
    #[test]
    fn reconnect_policy_variants() {
        use crate::ui::desktop_protocol::client_entry::{reconnect_for, ClientTarget};
        assert!(reconnect_for(
            &ClientTarget::Rqhost { wellknown: "w".into(), app_name: "a".into() },
            "p".into()
        )
        .is_none(), "rqhost 档 = exit-on-EOF");
        for target in [
            ClientTarget::Direct("p".into()),
            ClientTarget::Broker { broker_pipe: "b".into() },
        ] {
            let policy = reconnect_for(&target, "p".into()).expect("桌面档保持重连");
            assert_eq!(policy.budget_ms, 30_000);
            assert_eq!(policy.interval_ms, 50);
            assert_eq!(policy.pipe, "p");
        }
    }

    /// T-05：vm 轨分岔客户端（run_vm_rqhost_client）——凭据 env 消费
    /// （AUTO_VM_TITLE/AUTO_VM_WINDOW → Hello）+ 完整生命周期到 Close
    /// 干净退出；含 vm_window_size 解析档位。
    #[test]
    fn vm_fork_client_credentials_and_lifecycle() {
        // vm_window_size 档位（先于 env 设置断言缺省臂）。
        std::env::remove_var("AUTO_VM_WINDOW");
        assert_eq!(vm_window_size(), (480.0, 320.0), "缺席 = broker 子缺省");
        std::env::set_var("AUTO_VM_WINDOW", "640x480");
        assert_eq!(vm_window_size(), (640.0, 480.0), "WxH 解析");
        std::env::set_var("AUTO_VM_WINDOW", "fit");
        assert_eq!(vm_window_size(), (480.0, 320.0), "fit = 缺省档（非 480x680 独立窗语义）");
        std::env::set_var("AUTO_VM_WINDOW", "99x99");
        assert_eq!(vm_window_size(), (480.0, 320.0), "越界 = 缺省档");
        std::env::set_var("AUTO_VM_WINDOW", "640x480");

        let pipe = pid_pipe("vmfork");
        let (serve, _claim) = start_serve(&pipe);
        std::env::set_var("AUTO_VM_TITLE", "vm 轨测试窗");

        const SRC: &str = "widget VmForkCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n    }\n}\n";
        // vm fork 传 well-known：rendezvous 采纳内建（connect 臂）——
        // 宿主侧 serve 收到 adopt 记录后 pending 落端点。
        let fork_pipe = pipe.clone();
        let app = std::thread::spawn(move || {
            let comp = crate::build_dynamic_component(SRC, None).expect("build");
            run_vm_rqhost_client(comp, &fork_pipe)
        });

        // 宿主侧：采纳到 Active——Adopted 凭据来自 env（title/尺寸）。
        let (name, end) = {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                let got = serve.pending.lock().unwrap().pop();
                if let Some(x) = got {
                    break x;
                }
                assert!(std::time::Instant::now() < deadline, "pending 5s 未落");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };
        assert_eq!(name, "VmForkCounter", "app_name = widget 名");
        let mut ids = RqIds::default();
        let (mut client, events) = adopt_one(name, end, &mut ids, 3000).expect("泵到 Active");
        assert!(
            events.iter().any(|e| matches!(
                e,
                RqEvent::Adopted { title, width, height, .. }
                    if title == "vm 轨测试窗" && *width == 640.0 && *height == 480.0
            )),
            "Hello 凭据 = env 消费: {events:?}"
        );

        // 宿主 Close → vm 客户端干净退出。
        let close = client.inner.endpoint.close().expect("close");
        client.inner.end.send(&close).expect("close 下发");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let _ = pump_client(&mut client, &mut ids);
            if app.is_finished() {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "vm 客户端 5s 未退出");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let outcome = app.join().expect("vm 线程").expect("vm 客户端 Ok");
        assert_eq!(outcome, "rqhost client exited");

        std::env::remove_var("AUTO_VM_TITLE");
        std::env::remove_var("AUTO_VM_WINDOW");
        serve.stop(&pipe);
    }

    /// P031-R2/AC-01·02：vm 键入闭环（集成承载——e2e 真机合成输入在
    /// ToDesk 输入钩子类环境不生效，native_dock_e2e T4 同款环境事实）：
    /// 真管道 ×真 003-converter 源 ×真 ClientPump ×rq_update 输入臂
    ///（消息层臂 = listen_with 事件到达后的同一落点）——点击聚焦 +
    /// 键入 "100" → 客户端重排 → 宿主合成帧文本 212（p025 同级语义
    /// 证据）；hello 双客户端零串扰（帧文本不含 212 且无新 revision）。
    #[test]
    fn vm_typing_loop_over_pipe() {
        use crate::ui::desktop_protocol::client_runtime::{AppProjector, ClientConfig, ClientPump};
        use crate::ui::desktop_protocol::endpoint::FrameSource;
        use crate::ui::desktop_protocol::message::DrawOp;

        let conv_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/ui/003-converter/src/front/app.at"
        );
        let hello_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/ui/001-helloworld/src/front/app.at"
        );
        let conv_src = match std::fs::read_to_string(conv_path) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("[p031] skip: 003-converter 载体缺席");
                return;
            }
        };
        let hello_src = std::fs::read_to_string(hello_path).expect("read hello");

        let pipe = pid_pipe("typing");
        let (serve, _claim) = start_serve(&pipe);

        // 客户端线程 ×2（真 ClientPump，exit-on-EOF 档）。
        let cp = pipe.clone();
        let conv_app = std::thread::spawn(move || {
            let (_, end) = adopt(&cp, "App", 2000).expect("adopt conv");
            let comp = crate::build_dynamic_component(&conv_src, None).expect("build conv");
            let proj = AppProjector::new(comp, 480.0, 320.0);
            let (exit, _) = ClientPump::new(
                end,
                proj,
                ClientConfig {
                    app_name: "App".into(),
                    title: "003".into(),
                    width: 480.0,
                    height: 320.0,
                },
                None,
            )
            .run();
            exit
        });
        let hp = pipe.clone();
        let hello_app = std::thread::spawn(move || {
            let (_, end) = adopt(&hp, "App", 2000).expect("adopt hello");
            let comp = crate::build_dynamic_component(&hello_src, None).expect("build hello");
            let proj = AppProjector::new(comp, 480.0, 320.0);
            let (exit, _) = ClientPump::new(
                end,
                proj,
                ClientConfig {
                    app_name: "App".into(),
                    title: "hello".into(),
                    width: 480.0,
                    height: 320.0,
                },
                None,
            )
            .run();
            exit
        });

        // 宿主：采纳双端 + 挂窗。
        let mut state = RqDaemon {
            serve: Arc::clone(&serve),
            claim: None,
            wellknown: pipe.clone(),
            ids: RqIds::default(),
            clients: Vec::new(),
            had_window: false,
            opened: 0,
            last_cursor: BTreeMap::new(),
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while state.clients.len() < 2 {
            let pending: Vec<_> = state.serve.pending.lock().unwrap().drain(..).collect();
            for (name, end) in pending {
                if let Some((client, _)) = adopt_one(name, end, &mut state.ids, 3000) {
                    state.clients.push(client);
                }
            }
            assert!(std::time::Instant::now() < deadline, "双客户端 5s 未落地");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // 客户端顺序 = pending 交付序（conv 先 adopt）。以标题区分。
        let idx_of = |s: &RqDaemon, title: &str| {
            s.clients
                .iter()
                .position(|c| c.title == title)
                .unwrap_or_else(|| panic!("客户端 `{title}` 不在: {:?}", s.clients.iter().map(|c| &c.title).collect::<Vec<_>>()))
        };
        let conv_idx = idx_of(&state, "003");
        let hello_idx = idx_of(&state, "hello");
        let win_conv = iced::window::Id::unique();
        let win_hello = iced::window::Id::unique();
        state.clients[conv_idx].window = Some(win_conv);
        state.clients[hello_idx].window = Some(win_hello);
        state.had_window = true;

        // 输入盒几何：宿主侧现算（同投影器同尺寸——客户端首帧同源）。
        // conv_src 已 move 进客户端线程——重读文件。
        let geom_src = std::fs::read_to_string(conv_path).expect("re-read conv");
        let comp = crate::build_dynamic_component(&geom_src, None).expect("build for geom");
        let mut probe = AppProjector::new(comp, 480.0, 320.0);
        let frame0 = probe.render_frame();
        let (bx, by) = frame0
            .ops
            .iter()
            .find_map(|op| match op {
                DrawOp::Quad { rect, .. } if rect.w == 320.0 && rect.h == 32.0 => {
                    Some((rect.x, rect.y))
                }
                _ => None,
            })
            .expect("celsius 输入盒");

        fn texts_of(list: &DrawList) -> Vec<String> {
            list.ops
                .iter()
                .filter_map(|op| match op {
                    crate::ui::desktop_protocol::message::DrawOp::Text { text, .. }
                    | crate::ui::desktop_protocol::message::DrawOp::TextStyled { text, .. } => {
                        Some(text.clone())
                    }
                    _ => None,
                })
                .collect()
        }
        // 双端首帧落地后再取样（hello_before 早于其首帧 = 空表 vs 后值
        // 假阳性——首拍即败的现场）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while composed(&state.clients[hello_idx]).is_none()
            || composed(&state.clients[conv_idx]).is_none()
        {
            let _ = rq_update(&mut state, RqMessage::Tick);
            assert!(
                std::time::Instant::now() < deadline,
                "双端首帧 5s 未齐"
            );
            std::thread::sleep(std::time::Duration::from_millis(30));
        }
        let hello_before: Vec<String> = composed(&state.clients[hello_idx])
            .map(texts_of)
            .unwrap_or_default();

        // 键入序（rq_update 输入臂 = listen_with 事件到达后的同一落点）：
        // 光标 → 点击聚焦（宿主坐标 = 表面坐标 + 输入盒中心）→ "100"。
        let (cx, cy) = (bx + 160.0, by + 16.0);
        let _ = rq_update(&mut state, RqMessage::CursorMoved { window: win_conv, x: cx, y: cy });
        let _ = rq_update(&mut state, RqMessage::PointerPressed { window: win_conv, button: iced::mouse::Button::Left });
        let _ = rq_update(&mut state, RqMessage::PointerReleased { window: win_conv, button: iced::mouse::Button::Left });
        for ch in "100".chars() {
            let _ = rq_update(
                &mut state,
                RqMessage::Live {
                    window: win_conv,
                    input: crate::ui::session::LiveInput::Chars { text: ch.to_string() },
                },
            );
            // Tick 泵一轮（输入 → 客户端重排 → 新帧回宿主）。
            let _ = rq_update(&mut state, RqMessage::Tick);
            std::thread::sleep(std::time::Duration::from_millis(40));
        }

        // converter 帧文本 212（键入→换算联动——p025 同级语义）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        loop {
            let _ = rq_update(&mut state, RqMessage::Tick);
            let hit = composed(&state.clients[conv_idx])
                .map(|l| texts_of(l).iter().any(|t| t.starts_with("212")))
                .unwrap_or(false);
            if hit {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "003 键入→212 帧文本超时；当前帧: {:?}",
                composed(&state.clients[conv_idx]).map(texts_of)
            );
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        // hello 零串扰：帧文本不变 + revision 观测行零前进（client.last_revision
        // 直读——观测行打点同源）。
        let _ = rq_update(&mut state, RqMessage::Tick);
        let hello_after: Vec<String> = composed(&state.clients[hello_idx])
            .map(texts_of)
            .unwrap_or_default();
        assert_eq!(hello_before, hello_after, "hello 帧文本不变（不串扰）");
        let hello_rev = state.clients[hello_idx].last_revision;
        let _ = rq_update(&mut state, RqMessage::Tick);
        std::thread::sleep(std::time::Duration::from_millis(150));
        let _ = rq_update(&mut state, RqMessage::Tick);
        assert_eq!(
            state.clients[hello_idx].last_revision, hello_rev,
            "hello revision 零前进（输入零串扰）"
        );

        // 收尾：Close 双端 + Tick 泵到收敛（join 前握手必须走完——
        // 单发 Tick 不够会挂 join）。
        for idx in [conv_idx, hello_idx] {
            if let Ok(close) = state.clients[idx].inner.endpoint.close() {
                let _ = state.clients[idx].inner.end.send(&close);
            }
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !conv_app.is_finished() || !hello_app.is_finished() {
            let _ = rq_update(&mut state, RqMessage::Tick);
            assert!(
                std::time::Instant::now() < deadline,
                "Close 握手 5s 未收敛"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        serve.stop(&pipe);
        let _ = conv_app.join();
        let _ = hello_app.join();
    }

    /// T-07 回归钉：超大帧（20KB 文本，超 16KiB shm 槽）经管道内联回退
    /// 仍达宿主合成——push_frame 修复（此前 `if let Ok` 静默弃帧 = 冻结；
    /// e2e 首跑 003 卡死暴露）的进程内钉子。合成源替代 003 实例——
    /// 其首帧实测 558B 不走溢出路径（原前置断言证伪后的改型）。
    #[test]
    fn large_frame_falls_back_to_pipe_payload() {
        use crate::ui::desktop_protocol::client_runtime::{
            ClientConfig, ClientPump,
        };
        use crate::ui::desktop_protocol::endpoint::FrameSource;
        use crate::ui::desktop_protocol::message::{
            DrawOp, InputMsg as Msg, Rgba8,
        };

        struct BigSource;
        impl FrameSource for BigSource {
            fn revision(&self) -> u64 {
                1
            }
            fn render_frame(&mut self) -> DrawList {
                DrawList {
                    clear: None,
                    ops: vec![DrawOp::Text {
                        x: 0.0,
                        y: 0.0,
                        size: 14.0,
                        line_height: 18.0,
                        color: Rgba8::new(255, 255, 255, 255),
                        text: "x".repeat(20_000),
                    }],
                }
            }
            fn on_input(&mut self, _input: &Msg) {}
            fn on_control(&mut self, _control: &ControlMsg) {}
        }

        let pipe = pid_pipe("bigframe");
        let (serve, _claim) = start_serve(&pipe);
        let client_pipe = pipe.clone();
        let app = std::thread::spawn(move || {
            let (_, app_end) = adopt(&client_pipe, "App", 2000).expect("adopt");
            let config = ClientConfig {
                app_name: "App".into(),
                title: "big".into(),
                width: 480.0,
                height: 320.0,
            };
            let (exit, _) =
                ClientPump::new(app_end, BigSource, config, None).run();
            exit
        });

        let (name, end) = {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                let got = serve.pending.lock().unwrap().pop();
                if let Some(x) = got {
                    break x;
                }
                assert!(std::time::Instant::now() < deadline, "pending 5s 未落");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };
        let mut ids = RqIds::default();
        let (mut client, _) = adopt_one(name, end, &mut ids, 3000).expect("泵到 Active");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let (events, alive) = pump_client(&mut client, &mut ids);
            assert!(alive);
            assert!(events.is_empty());
            if client.frames >= 1 {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "超槽帧未合成（回退失效？）；frames={}",
                client.frames
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        serve.stop(&pipe);
        // 客户端收尾：宿主 Close 握手（无此则泵循环永续——join 挂死）。
        let close = client.inner.endpoint.close().expect("close 产出");
        let _ = client.inner.end.send(&close);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !app.is_finished() {
            let _ = pump_client(&mut client, &mut ids);
            assert!(
                std::time::Instant::now() < deadline,
                "大帧客户端 Close 握手 5s 未收敛"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let _ = app.join();
    }
}
