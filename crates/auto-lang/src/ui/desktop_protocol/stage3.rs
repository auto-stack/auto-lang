// Plan 480 S4 —— Stage 3 多 App 共享 host 引擎 + 压测 harness。
//
// [`BrokerClient`]：一条孵化连接的宿主侧全部状态（端点 + 双缓冲表面 +
// shm 段 + wid 映射）——驻留 [`DesktopSession`](crate::ui::session::
// DesktopSession) 的 `broker_clients` 表，宿主因此可以**并发**承载 N 个
// child（`HostEndpoint` 单 client 的"多 App 并发归 Stage 3"正是本计划
// 兑现的预留）。attach / pump / 指针路由的作用逻辑在 session.rs 的
// `broker_*` 方法族（与 `host::ProtocolHost::handle` 动作臂同构；
// ProtocolHost 单 client 机件与 Stage 1/2 测试不动）。
//
// 压测 harness（本文件测试）：父进程 = 桌面侧（enable_broker + 注册表
// resolver + attach/pump 泵）；子进程 = **测试二进制 re-exec**
// （`stage3_child_body`，env 注入 broker 管道名与 app 名）经
// `request_incubation` 孵化 + `ClientPump::run` 真协议主循环。
// N=3/5 child 全 Active → 逐 App 点击帧递增 → 30s 稳定存活。

use std::collections::BTreeMap;

use crate::ui::desktop_protocol::endpoint::HostEndpoint;
use crate::ui::desktop_protocol::host::SurfaceStore;
use crate::ui::desktop_protocol::message::{DrawList, FrameMsg};
use crate::ui::desktop_protocol::shm::SharedFrameBuffer;
use crate::ui::desktop_protocol::transport::Transport;
use crate::ui::session::{AppId, Wid};

/// 像素臂前缓冲（v1.3）：一条 surface 的最新 RGBA 帧（宿主渲染臂据此
/// 上传纹理；宿主侧"每 App 一份"表面驻留，与 SurfaceStore 命令帧同型）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelsSurface {
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
    pub stride: u32,
    pub revision: u64,
}

/// 一条孵化连接的宿主侧状态（多 client 宿主的"每 App 一份"部分）。
pub struct BrokerClient {
    /// per-app 管道名（map 键冗余存一份，便于日志/回收）。
    pub pipe: String,
    pub end: Box<dyn Transport + Send>,
    pub endpoint: HostEndpoint,
    pub(crate) surfaces: SurfaceStore,
    /// surface → shm 段（FrameReadyShared/FrameReadyPixels 载荷源）。
    pub(crate) shm: BTreeMap<u64, SharedFrameBuffer>,
    /// PLAN-034：surface → 位图段（BitmapReady 载荷源——专用第二段）。
    pub(crate) bm_shm: BTreeMap<u64, SharedFrameBuffer>,
    /// PLAN-034：surface → 已上传位图键（`bitmap://{id}`——回收/断连
    /// 逐出用，防 pid 复用串扰 + 残键泄漏）。
    pub(crate) bitmap_keys: BTreeMap<u64, Vec<String>>,
    /// wid → surface 句柄。
    pub(crate) wid_surface: BTreeMap<u64, u64>,
    /// surface → 像素前缓冲（v1.3 independent 臂；Commands 臂不用）。
    pub(crate) pixels: BTreeMap<u64, PixelsSurface>,
    /// 落地后的会话对象（Active 后有效）。
    pub app_id: Option<AppId>,
    pub wid: Option<Wid>,
    /// 孵化上报的 app 名（ResolveAndAttach 落地时回填；压测按名寻窗）。
    pub app_name: Option<String>,
}

impl BrokerClient {
    pub fn new(pipe: String, end: Box<dyn Transport + Send>) -> Self {
        Self {
            pipe,
            end,
            endpoint: HostEndpoint::listen(),
            surfaces: SurfaceStore::new(),
            shm: BTreeMap::new(),
            bm_shm: BTreeMap::new(),
            bitmap_keys: BTreeMap::new(),
            wid_surface: BTreeMap::new(),
            pixels: BTreeMap::new(),
            app_id: None,
            wid: None,
            app_name: None,
        }
    }

    /// 该 client 的虚拟窗当前合成面（压测帧断言口）。
    pub fn composed(&self) -> Option<&DrawList> {
        let surface = *self.wid_surface.get(&self.wid?.0)?;
        self.surfaces.front(surface)
    }

    /// PLAN-030：wid 归属判定（多表面客户端第二 wid 含于 wid_surface——
    /// 输入路由去单值化）。
    pub fn owns_wid(&self, wid: crate::ui::session::Wid) -> bool {
        self.wid == Some(wid) || self.wid_surface.contains_key(&wid.0)
    }

    /// 该 client 的像素前缓冲（v1.3 渲染臂/测试断言口）。
    pub fn composed_pixels(&self) -> Option<&PixelsSurface> {
        let surface = *self.wid_surface.get(&self.wid?.0)?;
        self.pixels.get(&surface)
    }

    /// 像素帧合成（v1.3）：shm 槽读 RGBA → 前缓冲翻面。返回 FrameAck
    /// （frame_id 原样回带；None = 无段/读失败，调用方不回 ack）。
    pub fn compose_pixels(
        &mut self,
        surface: u64,
        wid: u64,
        frame_id: u64,
        slot: u8,
        revision: u64,
        w: u32,
        h: u32,
        stride: u32,
    ) -> Option<FrameMsg> {
        let rgba = self.shm.get(&surface)?.read_slot(slot).ok()?;
        self.pixels.insert(
            surface,
            PixelsSurface { rgba, w, h, stride, revision },
        );
        Some(FrameMsg::FrameAck { wid, frame_id, slot })
    }
}

// ---------------------------------------------------------------------------
// S5：child 进程内存采样（Windows `K32GetProcessMemoryInfo` FFI，零新
// 依赖）。度量口径 = **边际增量**（N=1→3→5 每增一个 child 的均摊增量，
// 见计划待澄清①）；WorkingSet / PrivateUsage 双字段。
// ---------------------------------------------------------------------------

/// 一次进程内存采样（字节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessMemorySample {
    /// WorkingSet（物理驻留）。
    pub working_set: u64,
    /// PrivateUsage（提交私有字节；比 WorkingSet 更贴近"App 净增成本"）。
    pub private_bytes: u64,
}

#[cfg(windows)]
mod mem_ffi {
    // kernel32 导出（Win7+；避免引入 psapi.lib 链接面）。
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
        fn CloseHandle(hObject: isize) -> i32;
        fn K32GetProcessMemoryInfo(
            hProcess: isize,
            ppsmemCounters: *mut PROCESS_MEMORY_COUNTERS,
            cb: u32,
        ) -> i32;
    }

    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    /// PROCESS_MEMORY_COUNTERS + EX 尾字段 PrivateUsage（cb 按全结构
    /// 传入，PSAPI 会把 PrivateUsage 一并填充）。
    #[repr(C)]
    pub struct PROCESS_MEMORY_COUNTERS {
        pub cb: u32,
        pub PageFaultCount: u32,
        pub PeakWorkingSetSize: usize,
        pub WorkingSetSize: usize,
        pub QuotaPeakPagedPoolUsage: usize,
        pub QuotaPagedPoolUsage: usize,
        pub QuotaPeakNonPagedPoolUsage: usize,
        pub QuotaNonPagedPoolUsage: usize,
        pub PagefileUsage: usize,
        pub PeakPagefileUsage: usize,
        pub PrivateUsage: usize,
    }

    pub fn query(pid: u32) -> Result<super::ProcessMemorySample, String> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle == 0 {
                return Err(format!("OpenProcess({pid}): {}", std::io::Error::last_os_error()));
            }
            let mut counters = PROCESS_MEMORY_COUNTERS {
                cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                PageFaultCount: 0,
                PeakWorkingSetSize: 0,
                WorkingSetSize: 0,
                QuotaPeakPagedPoolUsage: 0,
                QuotaPagedPoolUsage: 0,
                QuotaPeakNonPagedPoolUsage: 0,
                QuotaNonPagedPoolUsage: 0,
                PagefileUsage: 0,
                PeakPagefileUsage: 0,
                PrivateUsage: 0,
            };
            let ok = K32GetProcessMemoryInfo(
                handle,
                &mut counters,
                counters.cb,
            );
            let _ = CloseHandle(handle);
            if ok == 0 {
                return Err(format!(
                    "K32GetProcessMemoryInfo({pid}): {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(super::ProcessMemorySample {
                working_set: counters.WorkingSetSize as u64,
                private_bytes: counters.PrivateUsage as u64,
            })
        }
    }
}

/// 采样一个进程的内存（字节）。非 Windows 平台返回 Err（v1 压测宿主
/// = Windows 桌面；Linux memfd 宿主见 shm.rs 注记）。
pub fn sample_process_memory(pid: u32) -> Result<ProcessMemorySample, String> {
    #[cfg(windows)]
    {
        mem_ffi::query(pid)
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        Err("memory sampling is windows-only in v1".into())
    }
}

/// L3 v2a：融合态 App 的 AutoVM 状态快照编码（revision + 全部原始状态
/// 字段；复合类型落 Nil 占位，见 client_runtime::encode_state_snapshot）。
pub fn fused_state_snapshot(
    component: &crate::ui::dynamic::DynamicComponent,
    revision: u64,
) -> Vec<u8> {
    let fields: Vec<(String, auto_val::Value)> =
        component.read_all_state().into_iter().collect();
    super::client_runtime::encode_state_snapshot(revision, &fields)
}

// ---------------------------------------------------------------------------
// 压测 harness：N child → broker 孵化 → 全 Active → 逐 App 点击帧递增 →
// 30s 稳定存活（子进程 = 测试二进制 re-exec 走真协议主循环）。
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::ui::desktop_protocol::broker;
    use crate::ui::desktop_protocol::client_runtime::{ClientConfig, ReconnectPolicy};
    use crate::ui::desktop_protocol::native_projector::RqProjector;
    use crate::ui::desktop_protocol::message::{DrawOp, FrameMode, MouseButton};
    use crate::ui::desktop_protocol::transport;
    use crate::ui::session::{DesktopSession, LaunchSpec};

    /// 子进程识别 env：broker 管道名 + app 名。
    const CHILD_BROKER_ENV: &str = "AUTO_480_BROKER";
    const CHILD_APP_ENV: &str = "AUTO_480_APP";
    const CHILD_MARKER: &str = "AUTO480-CHILD";

    /// 压测 App 源（每个 child 一份同源计数器；widget 名无关进程边界）。
    const STRESS_SRC: &str = "widget StressCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n        text `count: ${.count}`\n    }\n}\n";

    /// 子进程体：request_incubation → ClientPump::run 真协议主循环，直至
    /// 宿主 Close（压测收尾）或重连预算耗尽。直接跑套件（无 env）时跳过。
    #[test]
    fn stage3_child_body() {
        let Ok(broker_pipe) = std::env::var(CHILD_BROKER_ENV) else {
            return;
        };
        let app_name = std::env::var(CHILD_APP_ENV).expect("app name env");

        let (per_app_pipe, end) =
            broker::request_incubation(&broker_pipe, &app_name, 10_000).expect("incubate");
        let component = crate::build_dynamic_component(STRESS_SRC, None).expect("child build");
        let config = ClientConfig {
            app_name: app_name.clone(),
            title: app_name,
            width: 480.0,
            height: 320.0,
        };
        let reconnect =
            ReconnectPolicy { pipe: per_app_pipe, budget_ms: 30_000, interval_ms: 50 };
        // PLAN-033 T-04 迁移：压测子进程走 native 投影臂（AppProjector 退役）。
        let mut projector = RqProjector::new(component, 480.0, 320.0);
        projector.ensure_covered().expect("stress covered");
        let (exit, proj) =
            crate::ui::desktop_protocol::client_runtime::run_client_session(
                end, projector, config, Some(reconnect),
            );
        println!("{CHILD_MARKER} exit={exit:?} rev={}", proj.revision());
    }

    /// 压测窗口布局：落地后把每个虚拟窗搬开（非重叠网格），点击可定向。
    fn place_window(session: &mut DesktopSession, wid: Wid, index: usize) {
        if let Some(host) = session.host.as_mut() {
            if let Some(v) = host.wm.wins.get_mut(&wid) {
                let mut rect = *v.rect.borrow();
                rect.x = 16.0 + 500.0 * index as f32;
                rect.y = 16.0;
                *v.rect.borrow_mut() = rect;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Plan 500 步骤 8 —— T3 re-exec 集成：001–005 queue 端到端 +
    // independent 像素帧 + 双模并存。
    // -----------------------------------------------------------------------

    /// 子进程识别 env（模式档：queue | independent）。
    const T3_BROKER_ENV: &str = "AUTO_500_BROKER";
    const T3_APP_ENV: &str = "AUTO_500_APP";
    const T3_MODE_ENV: &str = "AUTO_500_MODE";

    /// T3 表面尺寸（005-login 内容高 ~812px，320 高度会溢出表面）。
    const T3_W: f32 = 480.0;
    const T3_H: f32 = 900.0;

    /// 001–005 示例名（examples/ui）+ p507/p515 构造语料（PLAN-552 探针
    /// 清退起位于 examples/capability-tests，example_source 双根解析）。
    const T3_EXAMPLES: [&str; 7] = [
        "001-helloworld",
        "002-counter",
        "003-converter",
        "004-profile-card",
        "005-login",
        // Plan 507 T10：Tier1+Tier2 构造覆盖示例（实机 queue 模式语料）。
        "p507-tier-coverage",
        // Plan 515 G1：scrollable 溢出裁剪构造示例（scissor 栈 e2e 语料）。
        "p515-scroll-overflow",
    ];

    fn example_source(dir: &str) -> String {
        // PLAN-552：构造语料双根——主根 examples/ui（001–005）优先，探针根
        // examples/capability-tests 兜底（p507/p515 随探针清退迁出）。
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
        let path = ["examples/ui", "examples/capability-tests"]
            .iter()
            .map(|root| format!("{base}{root}/{dir}/src/front/app.at"))
            .find(|p| std::path::Path::new(p).is_file())
            .unwrap_or_else(|| format!("{base}examples/ui/{dir}/src/front/app.at"));
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
    }

    /// PLAN-032 T-07：front 全源语料——`src/front/*.at` 合并（多文件例
    /// 018/021/024/041 的组件/页面引用需合体解析；仪器 native_flip_
    /// coverage_data_row 同径）。
    fn example_source_all(dir: &str) -> String {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/ui/");
        let front = format!("{base}{dir}/src/front");
        let mut srcs: Vec<std::path::PathBuf> = std::fs::read_dir(&front)
            .unwrap_or_else(|e| panic!("read {front}: {e}"))
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "at"))
            .collect();
        srcs.sort();
        let mut combined = String::new();
        for s in &srcs {
            combined.push_str(&std::fs::read_to_string(s).unwrap_or_default());
            combined.push('\u{a}');
        }
        combined
    }

    /// PLAN-029 T-03：native 档子进程共用体——孵化 + ensure_covered 门 +
    /// RqProjector + run_client_session（镜像 client_entry 生产分支）。
    fn run_native_t3_child<C>(
        broker_pipe: &str,
        app: &str,
        component: C,
    ) where
        C: crate::ui::Component + 'static,
        C::Msg: Clone + std::fmt::Debug + Send + 'static,
    {
        let (_pipe, end) = broker::request_incubation_render(
            broker_pipe,
            app,
            broker::RequestedRender::default(),
            10_000,
        )
        .expect("incubate");
        let config = ClientConfig {
            app_name: app.to_string(),
            title: app.to_string(),
            width: T3_W,
            height: T3_H,
        };
        let reconnect = ReconnectPolicy { pipe: _pipe, budget_ms: 30_000, interval_ms: 50 };
        let projector = crate::ui::desktop_protocol::native_projector::RqProjector::new(
            component, T3_W, T3_H,
        );
        if let Err(gate) = projector.ensure_covered() {
            panic!("native child 覆盖门拒绝: {gate}");
        }
        let (exit, proj) = crate::ui::desktop_protocol::client_runtime::run_client_session(
            end, projector, config, Some(reconnect),
        );
        println!("AUTO029-CHILD exit={exit:?} rev={}", proj.revision());
    }

    /// PLAN-029 T-03：p029 live 输入 e2e 载体——typed Component（a2r 生成
    /// 形态的手写等价）：单 input 绑定 `.buf`，on() 读 `last_input_text()`
    /// 回写（a2r 输入合同，login 例 main.rs 同款）；echo 文本随 buf 联动
    /// （帧断言面）。滚轮腿配一个 scrollable 使 Scroll 消费可断言。
    #[derive(Clone, Debug, PartialEq)]
    pub enum P029Msg {
        BufChanged,
        ScrollMoved(f32, f32),
    }

    #[derive(Debug, Default)]
    pub struct P029TypedInputs {
        pub buf: String,
        pub scroll_y: f32,
    }

    impl crate::ui::Component for P029TypedInputs {
        type Msg = P029Msg;

        fn on(&mut self, msg: Self::Msg) {
            match msg {
                P029Msg::BufChanged => {
                    let text = crate::ui::iced::last_input_text();
                    self.buf = text;
                }
                P029Msg::ScrollMoved(_x, y) => {
                    self.scroll_y = y;
                }
            }
        }

        fn view(&self) -> crate::ui::View<Self::Msg> {
            use crate::ui::View;
            View::col()
                .style("p-2 gap-2")
                .child(
                    View::input("type here")
                        .value(self.buf.clone())
                        .w_full()
                        .on_change(P029Msg::BufChanged)
                        .build(),
                )
                .child(View::text_styled(
                    format!("echo:{}", self.buf),
                    "text-sm",
                ))
                .child(View::text_styled(
                    format!("scroll:{}", self.scroll_y),
                    "text-sm",
                ))
                .build()
        }
    }

    /// PLAN-029 T-09：shell 面语料——popover（锚按钮开合）/window_
    /// thumbnail/workspace_preview/mouse-area/icon（lucide: src）五件套
    ///（p029_shell_face_arm 载体；命中闭环 = 菜单开 → 面板项 → 关）。
    #[derive(Debug, Clone, PartialEq)]
    pub enum ShellFaceMsg {
        ToggleMenu,
        MenuAction,
        Dismiss,
        AreaClick,
    }

    #[derive(Debug, Default)]
    pub struct P029ShellFace {
        pub open: bool,
        pub log: String,
    }

    impl crate::ui::Component for P029ShellFace {
        type Msg = ShellFaceMsg;

        fn on(&mut self, msg: Self::Msg) {
            match msg {
                ShellFaceMsg::ToggleMenu => self.open = !self.open,
                ShellFaceMsg::MenuAction => self.log = "action".into(),
                ShellFaceMsg::Dismiss => {
                    self.open = false;
                    self.log = "dismissed".into();
                }
                ShellFaceMsg::AreaClick => self.log = "area".into(),
            }
        }

        fn view(&self) -> crate::ui::View<Self::Msg> {
            use crate::ui::view::{PopoverAnchor, PopoverPlacement};
            use crate::ui::View;
            View::col()
                .style("p-2 gap-2")
                .child(View::Popover {
                    anchor: PopoverAnchor::Widget(Box::new(
                        View::button("menu").on_click(|_| ShellFaceMsg::ToggleMenu).build(),
                    )),
                    content: Box::new(
                        View::col()
                            .child(View::text("menu-item"))
                            .child(
                                View::button("act")
                                    .on_click(|_| ShellFaceMsg::MenuAction)
                                    .build(),
                            )
                            .build(),
                    ),
                    placement: PopoverPlacement::BottomStart,
                    open: self.open,
                    on_dismiss: Some(ShellFaceMsg::Dismiss),
                })
                .child(View::WindowThumbnail {
                    wid: "42842".into(),
                    fallback_icon: "app-window".into(),
                    style: None,
                })
                .child(View::WorkspacePreview {
                    ws: "0".into(),
                    fallback_icon: "app-window".into(),
                    style: None,
                })
                .child(View::image_styled("lucide:panel-top", "w-4 h-4"))
                .child(View::MouseArea {
                    content: Box::new(View::text("hit-area")),
                    on_enter: None,
                    on_exit: None,
                    on_double_click: None,
                    on_click: Some(ShellFaceMsg::AreaClick),
                    on_context_menu: None,
                    on_release: None,
                    on_move: None,
                    logical_extent: Some((120.0, 24.0)),
                    style: None,
                })
                .child(View::text_styled(
                    format!("log:{}", self.log),
                    "text-sm",
                ))
                .build()
        }
    }

    /// T3 子进程体：三态裁决的 child 侧（queue = 真协议泵；independent =
    /// 真 iced 隐藏窗 + 截图泵）。直接跑套件（无 env）时跳过。
    #[test]
    fn t3_child_body() {
        let Ok(broker_pipe) = std::env::var(T3_BROKER_ENV) else {
            return;
        };
        let app = std::env::var(T3_APP_ENV).expect("app env");
        let mode = std::env::var(T3_MODE_ENV).unwrap_or_else(|_| "queue".into());
        // PLAN-029 T-03：p029 类型化语料——typed Component + a2r 输入合同
        //（on() 读 last_input_text 回写绑定字段）。native+dynamic 组合无
        // 生产形态（VM 桥不读 thread-local——store_input_text 合同为 a2r
        // 生成侧专属），故 native 档键入语料走 typed 形态。
        if mode == "native" && app == "p029-typed-inputs" {
            run_native_t3_child(&broker_pipe, &app, P029TypedInputs::default());
            return;
        }
        // PLAN-029 T-09：shell 面语料（popover/thumbnail/preview/mousearea/
        // lucide icon 五件套——p029_shell_face_arm 载体）。
        if mode == "native" && app == "p029-shell-face" {
            run_native_t3_child(&broker_pipe, &app, P029ShellFace::default());
            return;
        }
        // PLAN-030 T-08：p030 壳装配子进程——shell_client 真身 + 点击钩子。
        if mode == "shell" && app == "p030-shell" {
            run_p030_shell_child(&broker_pipe);
            return;
        }
        // PLAN-032 T-07：六例全源档——native-full = 显式 queue（front 全
        /// .at 合并解析 + ensure_covered 门 + RqProjector）；native-
        /// auto-full = auto 档翻转抽样腿（resolve_native_frame_mode
        ///（Auto）裁决断言 Commands——flipped@ramp3 后缺省 queue，观测
        /// 行随行）。
        if mode == "native-full" || mode == "native-auto-full" {
            // 生产同型装载：单 app.at + path 上下文（`use` 模块声明由
            // loader 按 path 兄弟文件解析——018 book_store/021 页组件）。
            let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/ui/");
            let path = format!("{base}{app}/src/front/app.at");
            let src = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {path}: {e}"));
            let component = crate::build_dynamic_component(&src, Some(&path))
                .unwrap_or_else(|e| panic!("child build (path ctx {app}): {e:?}"));
            if mode == "native-auto-full" {
                use crate::ui::desktop_protocol::client_entry::resolve_native_frame_mode;
                use crate::ui::desktop_protocol::coverage::RenderMode;
                use crate::ui::desktop_protocol::message::FrameMode;
                let view = crate::ui::Component::view(&component);
                let (frame_mode, _downgraded, line) =
                    resolve_native_frame_mode(RenderMode::Auto, &app, &view);
                if let Some(l) = &line {
                    println!("[p032-auto] {l}");
                }
                assert_eq!(
                    frame_mode, FrameMode::Commands,
                    "auto 档翻转后缺省 queue（flipped@ramp3）"
                );
                assert!(
                    line.as_deref().is_some_and(|l| l.contains("flipped@ramp3")),
                    "翻转观测行: {line:?}"
                );
            }
            run_native_t3_child(&broker_pipe, &app, component);
            return;
        }
        let src = example_source(&app);
        let component = crate::build_dynamic_component(&src, None).expect("child build");
        match mode.as_str() {
            // PLAN-029 T-03：native 档——View 树 native 投影器子进程
            //（镜像 client_entry::run_native_client 的 Commands 生产分支：
            // ensure_covered 门 + RqProjector 全输入臂）。
            "native" => {
                run_native_t3_child(&broker_pipe, &app, component);
            }
            // PLAN-033 T-04 迁移：默认 queue 档 = native 投影臂（解释投影
            // 器退役——`-q`/孵化链统一单投影器；"independent" 档随解释
            // pixels 臂退役删除）。
            _ => {
                run_native_t3_child(&broker_pipe, &app, component);
            }
        }
    }

    /// re-exec 一个 T3 子进程（env 注入 broker 管道/app 名/模式档）。
    fn spawn_t3_child(broker_pipe: &str, app: &str, mode: &str) -> std::process::Child {
        let exe = std::env::current_exe().expect("current_exe");
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["t3_child_body", "--test-threads", "1", "--nocapture"])
            .env(T3_BROKER_ENV, broker_pipe)
            .env(T3_APP_ENV, app)
            .env(T3_MODE_ENV, mode)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());
        for (k, _) in std::env::vars() {
            if k.starts_with("NEXTEST_") {
                cmd.env_remove(&k);
            }
        }
        cmd.spawn().expect("spawn t3 child")
    }

    /// 按示例名取该 client 的合成帧（queue 臂 DrawList）。
    /// PLAN-025 T-07 —— native input 族 e2e（`AUTO_DESKTOP_E2E=1` 门 +
    /// 025 载体 env；p020_native_exe_arm 同型）。两载体真 exe 孵化：
    /// ①003-converter（真源 a2r，queue 档）——双 input 帧渲染 →
    /// broker_pointer_down 聚焦 celsius → broker_char 生产路径键入
    /// "100" → 换算联动帧（212）断言；②inputs025 fixture——slider 轨道
    /// 点击 f32 派发 + select 开合 → 选项命中 → 帧值变。键入口径 =
    /// 宿主生产路径协议承载（⑤——真机 iced 事件注入面缺席，D4 随注）。
    /// 载体缺省寻址：`target/debug/{converter,inputs025}.exe` +
    /// `../scratch025/{003-converter,025-inputs}`；env AUTO_025_* 覆盖。
    #[test]
    fn p025_native_input_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        let manifest = env!("CARGO_MANIFEST_DIR");
        let exe_c = std::env::var("AUTO_025_NATIVE_EXE")
            .unwrap_or_else(|_| format!("{manifest}/../../target/debug/converter.exe"));
        let dir_c = std::path::PathBuf::from(
            std::env::var("AUTO_025_NATIVE_APP_DIR")
                .unwrap_or_else(|_| format!("{manifest}/../../scratch025/003-converter")),
        );
        let exe_i = std::env::var("AUTO_025_INPUTS_EXE")
            .unwrap_or_else(|_| format!("{manifest}/../../target/debug/inputs025.exe"));
        let dir_i = std::path::PathBuf::from(
            std::env::var("AUTO_025_INPUTS_APP_DIR")
                .unwrap_or_else(|_| format!("{manifest}/../../scratch025/025-inputs")),
        );
        if !std::path::Path::new(&exe_c).is_file()
            || !dir_c.join("src/front/app.at").is_file()
            || !std::path::Path::new(&exe_i).is_file()
            || !dir_i.join("src/front/app.at").is_file()
        {
            eprintln!(
                "[p025] skip: 载体缺席（exe_c={exe_c} dir_c={} exe_i={exe_i} dir_i={}）",
                dir_c.display(),
                dir_i.display()
            );
            return;
        }

        use crate::ui::desktop_protocol::message::{DrawOp, FrameMode};
        use crate::ui::session::LaunchSpec;
        let broker_pipe = format!("autodesk-broker-025-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());

        let code_c = std::fs::read_to_string(dir_c.join("src/front/app.at")).expect("read 003");
        let code_c_for = code_c.clone();
        let exe_c_for = exe_c.clone();
        let dir_c_for = dir_c.clone();
        let code_i_for = std::fs::read_to_string(dir_i.join("src/front/app.at")).expect("read inputs");
        let exe_i_for = exe_i.clone();
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| match name {
                "003-converter" => Some(LaunchSpec {
                    code: code_c_for.clone(),
                    source_path: Some(
                        dir_c_for.join("src/front/app.at").to_string_lossy().to_string(),
                    ),
                    title: Some("Converter".into()),
                    name: Some("converter".into()),
                    daemon: None,
                    back_root: None,
                    fit: false,
                    exe: Some(std::path::PathBuf::from(&exe_c_for)),
                    render_decl: Some("queue".into()),
                    opens: Vec::new(),
                }),
                "025-inputs" => Some(LaunchSpec {
                    code: code_i_for.clone(),
                    source_path: Some(
                        dir_i.join("src/front/app.at").to_string_lossy().to_string(),
                    ),
                    title: Some("Inputs025".into()),
                    name: Some("inputs025".into()),
                    daemon: None,
                    back_root: None,
                    fit: false,
                    exe: Some(std::path::PathBuf::from(&exe_i_for)),
                    render_decl: Some("queue".into()),
                    opens: Vec::new(),
                }),
                _ => None,
            }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        fn origin_of(session: &DesktopSession, wid: Wid) -> (f32, f32) {
            session
                .host
                .as_ref()
                .and_then(|h| h.wm.wins.get(&wid))
                .map(|v| {
                    let r = *v.rect.borrow();
                    (r.x, r.y)
                })
                .expect("窗原点")
        }
        fn wait_frame(
            session: &mut DesktopSession,
            app: &str,
            pred: impl Fn(&[DrawOp]) -> bool,
            what: &str,
        ) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                let hit = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app))
                    .and_then(|c| c.composed())
                    .is_some_and(|l| pred(&l.ops));
                if hit {
                    return;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "{what} 超时: {:?}",
                    session
                        .broker_clients
                        .values()
                        .find(|c| c.app_name.as_deref() == Some(app))
                        .and_then(|c| c.composed())
                        .map(|l| l.ops.iter().map(|o| format!("{o:?}")).collect::<Vec<_>>())
                );
                std::thread::yield_now();
            }
        }
        fn quads_of(ops: &[DrawOp]) -> Vec<(f32, f32, f32, f32)> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Quad { rect, .. } => Some((rect.x, rect.y, rect.w, rect.h)),
                    _ => None,
                })
                .collect()
        }
        fn texts_of(ops: &[DrawOp]) -> Vec<String> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Text { text, .. } => Some(text.clone()),
                    _ => None,
                })
                .collect()
        }

        // —— ①003-converter：queue 孵化 + 双 input + broker_char 键入联动。
        let wid_c = session.launch_app("003-converter").expect("converter launch");
        place_window(&mut session, wid_c, 0);
        // 先泵到首帧（wid 在 ResolveAndAttach 落地期回填 client）。
        wait_frame(
            &mut session,
            "003-converter",
            |ops| {
                quads_of(ops)
                    .iter()
                    .any(|r| r.2 == 320.0 && r.3 == 32.0)
            },
            "003 input 帧",
        );
        let mode_c = session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_c))
            .map(|c| c.endpoint.frame_mode)
            .expect("converter client");
        assert_eq!(mode_c, FrameMode::Commands, "003 显式 queue → Commands");
        // 首个 (320,32) 盒 = celsius input；窗内点击聚焦 → broker_char 键入。
        let (ox, oy) = origin_of(&session, wid_c);
        let (ix, iy, _, _) = *session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_c))
            .and_then(|c| c.composed())
            .map(|l| quads_of(&l.ops))
            .expect("composed")
            .iter()
            .find(|r| r.2 == 320.0 && r.3 == 32.0)
            .expect("celsius input 盒");
        assert!(session.broker_pointer_down(ox + ix + 160.0, oy + iy + 16.0, MouseButton::Left));
        for ch in "100".chars() {
            assert!(session.broker_char(ch), "broker_char 路由");
        }
        wait_frame(
            &mut session,
            "003-converter",
            |ops| texts_of(ops).iter().any(|t| t.starts_with("212")),
            "003 键入换算联动（fahrenheit=212）",
        );
        println!("AUTO025-NATIVE converter typing PASS (celsius 100 -> fahrenheit 212)");

        // —— ②inputs025：slider 点击定位 + select 开合选项命中。
        let wid_i = session.launch_app("025-inputs").expect("inputs launch");
        place_window(&mut session, wid_i, 1);
        wait_frame(
            &mut session,
            "025-inputs",
            |ops| texts_of(ops).iter().any(|t| t == "vol: 50"),
            "inputs 首帧",
        );
        let (ox, oy) = origin_of(&session, wid_i);
        // slider 轨道 = 4px 高 Quad；75% 处点击 → vol = 75。
        let (tx, ty, tw, _) = *session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_i))
            .and_then(|c| c.composed())
            .map(|l| quads_of(&l.ops))
            .expect("composed")
            .iter()
            .find(|r| r.3 == 4.0)
            .expect("slider track");
        assert!(session.broker_pointer_down(ox + tx + tw * 0.75, oy + ty + 2.0, MouseButton::Left));
        wait_frame(
            &mut session,
            "025-inputs",
            |ops| texts_of(ops).iter().any(|t| t == "vol: 75"),
            "slider 轨道点击 f32 派发（vol 75）",
        );
        println!("AUTO025-NATIVE slider click PASS (vol 50 -> 75)");

        // select：值盒 = 第二个 (320,32) Quad；点击开 → 选项列 → 命中
        // Medium（开态选项第 2 项）→ pick 帧变 + 回闭态。
        wait_frame(
            &mut session,
            "025-inputs",
            |ops| {
                quads_of(ops)
                    .iter()
                    .filter(|r| r.2 == 320.0 && r.3 == 32.0)
                    .count()
                    >= 2
            },
            "select 盒在册",
        );
        let quads = session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_i))
            .and_then(|c| c.composed())
            .map(|l| quads_of(&l.ops))
            .expect("composed");
        let boxes: Vec<_> = quads
            .iter()
            .filter(|r| r.2 == 320.0 && r.3 == 32.0)
            .collect();
        let (sx, sy, _, _) = *boxes[1];
        assert!(session.broker_pointer_down(ox + sx + 160.0, oy + sy + 16.0, MouseButton::Left));
        wait_frame(
            &mut session,
            "025-inputs",
            |ops| {
                quads_of(ops)
                    .iter()
                    .filter(|r| r.2 == 320.0 && r.3 == 32.0)
                    .count()
                    >= 5
            },
            "select 开态选项列（3 项）",
        );
        let quads = session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_i))
            .and_then(|c| c.composed())
            .map(|l| quads_of(&l.ops))
            .expect("composed");
        let boxes: Vec<_> = quads
            .iter()
            .filter(|r| r.2 == 320.0 && r.3 == 32.0)
            .collect();
        let (mx, my, _, _) = *boxes[boxes.len() - 2]; // 开态选项第 2 项 = Medium
        assert!(session.broker_pointer_down(ox + mx + 160.0, oy + my + 16.0, MouseButton::Left));
        wait_frame(
            &mut session,
            "025-inputs",
            |ops| {
                texts_of(ops).iter().any(|t| t == "pick: Medium")
                    && quads_of(ops)
                        .iter()
                        .filter(|r| r.2 == 320.0 && r.3 == 32.0)
                        .count()
                        == 2
            },
            "select 选项命中（pick: Medium + 回闭态）",
        );
        println!("AUTO025-NATIVE select pick PASS (Small -> Medium)");

        // 帧留痕（AUTO_025_ASSETS=1 → docs/plans/reports/assets/025/）。
        if std::env::var("AUTO_025_ASSETS").is_ok() {
            let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/plans/reports/assets/025");
            let _ = std::fs::create_dir_all(&assets);
            for (app, file) in [
                ("003-converter", "converter-frame.txt"),
                ("025-inputs", "inputs025-frame.txt"),
            ] {
                if let Some(list) = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app))
                    .and_then(|c| c.composed())
                {
                    let out = crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(list);
                    let _ = std::fs::write(assets.join(file), out);
                }
            }
        }

        // 兜底清理。
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// PLAN-030 T-08：p030 壳子进程体——shell_client 装配真身
    /// （`--autodesk-shell` 产品入口同体；AUTO_SHELL_GEOM 由 spawn 侧
    /// env 注入）+ e2e 点击钩子（chrome 面真按钮命中区坐标自注入
    /// press/release——宿主 pointer 路由的等价载荷，前 3 拍投影各一次）。
    fn run_p030_shell_child(broker_pipe: &str) {
        use crate::ui::desktop_protocol::message::{shell_face, InputMsg, MouseButton};
        use crate::ui::desktop_protocol::shell_client::{ShellFaces, ShellGeometry, ShellPump};
        let geometry = ShellGeometry::from_env().unwrap_or_else(ShellGeometry::fallback);
        let faces = ShellFaces::load(geometry).expect("p030 壳面装载");
        let pump = ShellPump::start(broker_pipe, geometry, faces).expect("p030 壳泵");
        let mut clicks = 0u32;
        let pump = pump.with_on_applied(Box::new(move |faces: &mut ShellFaces| {
            clicks += 1;
            if clicks > 3 {
                return;
            }
            let rects = faces.hit_rects(shell_face::SHELL);
            let Some(rect) = rects.first().copied() else {
                println!("AUTO030-CLICK no-hit (apply {clicks})");
                return;
            };
            let handler = format!("hit#{}", rects.len());
            let (cx, cy) = (rect.x + rect.w / 2.0, rect.y + rect.h / 2.0);
            for press in [true, false] {
                let input = if press {
                    InputMsg::PointerPressed {
                        wid: 0,
                        button: MouseButton::Left,
                        x: cx,
                        y: cy,
                        modifiers: 0,
                    }
                } else {
                    InputMsg::PointerReleased {
                        wid: 0,
                        button: MouseButton::Left,
                        x: cx,
                        y: cy,
                        modifiers: 0,
                    }
                };
                faces.on_input(shell_face::SHELL, &input);
            }
            println!("AUTO030-CLICK {cx:.0},{cy:.0} handler={handler} (apply {clicks})");
        }));
        pump.run().expect("p030 shell child run");
    }

    /// PLAN-030 T-08：壳 outproc 全链 e2e——真子进程壳（t3 re-exec +
    /// shell_client 真身）四腿：①双表面首帧（background 全屏 + chrome
    /// 带，DrawList 合成）②任务栏真按钮点击 → DesktopBus 上行 →
    /// registry_id 归因（"shell"）③投影推送 → dock 列表帧变（假窗
    /// "P030Win" 入帧）④kill 壳 → 看门兵退避重启 → attach 指纹失效 →
    /// 全量重推恢复。`AUTO_DESKTOP_E2E=1` 门；留痕
    /// `AUTO_030_ASSETS=1` → assets/030/。
    #[test]
    fn p030_shell_outproc_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        use crate::ui::desktop_protocol::message::{shell_face, DrawOp};
        use crate::ui::desktop_protocol::shell_client::ShellGeometry;
        use crate::ui::session::{DesktopSession, ShellModel};

        let broker_pipe = format!("autodesk-broker-030-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        session.desktop.shell_model = ShellModel::Outproc;
        session.desktop.shell_geometry =
            Some(ShellGeometry { viewport_w: 1280.0, viewport_h: 800.0, band_h: 48.0 });
        let pipe_for_spawn = broker_pipe.clone();
        session.desktop.shell_spawner = Some(Arc::new(move |geom, _pipe| {
            // 几何经 env 传递（spawn_t3_child 继承父 env）。
            std::env::set_var("AUTO_SHELL_GEOM", geom.encode());
            Ok(spawn_t3_child(&pipe_for_spawn, "p030-shell", "shell"))
        }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        // attach 泵辅助：孵化排队 → attach → 壳 pipe 落地。
        fn pump_until_shell_attached(session: &mut DesktopSession) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while session.desktop.shell_pipe.is_none() {
                if session.pending_incubations() > 0 {
                    session.attach_pending_incubations(5000);
                }
                session.pump_broker_clients();
                assert!(std::time::Instant::now() < deadline, "p030 壳 attach 超时");
                std::thread::yield_now();
            }
        }
        fn shell_frame_of(
            session: &DesktopSession,
            face: u8,
        ) -> Option<crate::ui::desktop_protocol::message::DrawList> {
            let pipe = session.desktop.shell_pipe.as_ref()?;
            let client = session.broker_clients.get(pipe)?;
            // chrome = pseudo[1]、background = pseudo[0]。
            let idx = if face == shell_face::SHELL { 1 } else { 0 };
            let wid = session.desktop.shell_pseudo_wids.get(idx).copied()?;
            let surface = client.wid_surface.get(&wid.0)?;
            client.surfaces.front(*surface).cloned()
        }

        // —— 腿 0：孵化 + attach + 双伪窗在案。
        session.launch_shell_outproc().expect("p030 壳 spawn");
        pump_until_shell_attached(&mut session);
        assert_eq!(session.desktop.shell_pseudo_wids.len(), 2, "双伪窗在案");
        println!("AUTO030 leg0 attach PASS");

        // —— 腿 1：双表面首帧（两面 DrawList 合成在册）。
        {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                let bg = shell_frame_of(&session, shell_face::DESKTOP_SURFACE);
                let chrome = shell_frame_of(&session, shell_face::SHELL);
                if bg.is_some() && chrome.is_some() {
                    break;
                }
                assert!(std::time::Instant::now() < deadline, "p030 腿1 双表面首帧超时");
                std::thread::yield_now();
            }
        }
        println!("AUTO030 leg1 dual-surface-first-frame PASS");

        // —— 腿 3：投影推送 → 任务栏帧变（假窗入投影 → chrome 帧变化；
        // 断言 = op 数增量——dock 窗口按钮渲染形态（quad/icon）不假设文本）。
        let baseline_ops;
        {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                if let Some(l) = shell_frame_of(&session, shell_face::SHELL) {
                    baseline_ops = l.ops.len();
                    break;
                }
                assert!(std::time::Instant::now() < deadline, "p030 腿3 基线帧超时");
                std::thread::yield_now();
            }
        }
        {
            let comp = crate::build_dynamic_component(
                r#"widget t { view { text "P030Win" } }"#,
                None,
            )
            .expect("假窗组件");
            let app = session.allocate_app(comp);
            let fake_wid = session.wm_add_win(
                app,
                "P030Win".into(),
                iced::Rectangle::new(
                    iced::Point::new(64.0, 64.0),
                    iced::Size::new(320.0, 200.0),
                ),
            );
            // registry_id 回填：running 集派生入投影（dock/任务栏可变面）。
            if let Some(host) = session.host.as_mut() {
                if let Some(v) = host.wm.wins.get_mut(&fake_wid) {
                    v.registry_id = Some("p030-fake".into());
                }
            }
        }
        {
            crate::ui::iced::renderer::push_shell_projection_outproc(&mut session);
            println!(
                "AUTO030 leg3 pushed (shell_fp={:?} desk_fp={:?})",
                session.desktop.shell_push_fp.is_some(),
                session.desktop.shell_desk_fp.is_some()
            );
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                let hit = shell_frame_of(&session, shell_face::SHELL).is_some_and(|l| {
                    let grew = l.ops.len() > baseline_ops;
                    if grew {
                        let texts: Vec<String> = l
                            .ops
                            .iter()
                            .filter_map(|op| match op {
                                DrawOp::Text { text, .. } => Some(text.clone()),
                                _ => None,
                            })
                            .collect();
                        println!("AUTO030 leg3 ops {} -> {} texts={:?}", baseline_ops, l.ops.len(), texts);
                    }
                    grew
                });
                if hit {
                    break;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "p030 腿3 投影推送帧变超时 (shell_pipe={:?} baseline={})",
                    session.desktop.shell_pipe,
                    baseline_ops
                );
                std::thread::yield_now();
            }
        }
        println!("AUTO030 leg3 projection-push-frame-change PASS");

        // —— 腿 2：任务栏真按钮点击 → DesktopBus 上行 → 归因 + 执行。
        {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                if !session.desktop.desktop_bus_inbox.is_empty() {
                    break;
                }
                assert!(std::time::Instant::now() < deadline, "p030 腿2 DesktopBus 上行超时");
                std::thread::yield_now();
            }
            let inbox = std::mem::take(&mut session.desktop.desktop_bus_inbox);
            for (src, cmd) in &inbox {
                println!("AUTO030 leg2 bus: source={src} cmd={cmd:?}");
                assert_eq!(src, "shell", "registry_id 归因（chrome 伪窗 = shell）");
            }
            assert!(!inbox.is_empty(), "上行记录 >= 1");
            // 回填后经 drain 执行（真执行臂——动词效果随词表，断言不炸 + 取尽）。
            session.desktop.desktop_bus_inbox = inbox;
            let _ = crate::ui::iced::renderer::drain_desktop_bus_inbox(&mut session);
            assert!(session.desktop.desktop_bus_inbox.is_empty(), "drain 取尽");
        }
        println!("AUTO030 leg2 taskbar-click-bus-uplink PASS");

        // —— 腿 4：kill 壳 → 看门兵检出 → 退避 respawn → 全量重推恢复。
        {
            let child = session
                .desktop
                .outproc_children
                .last_mut()
                .expect("壳子进程句柄在案");
            let _ = child.kill();
            let _ = child.wait();
        }
        {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                if session.desktop.shell_pipe.is_none()
                    && session.desktop.shell_respawn.is_some()
                {
                    break;
                }
                assert!(std::time::Instant::now() < deadline, "p030 腿4 死亡检出超时");
                std::thread::yield_now();
            }
        }
        println!("AUTO030 leg4 death-detected PASS");
        // 退避第一档 1s：到期后 watchdog 步发起 respawn。
        std::thread::sleep(std::time::Duration::from_millis(1100));
        assert!(session.shell_watchdog_step(), "respawn 发起");
        pump_until_shell_attached(&mut session);
        // attach 即指纹强制失效 → 全量重推可达。
        assert!(session.desktop.shell_push_fp.is_none(), "respawn 后指纹失效");
        crate::ui::iced::renderer::push_shell_projection_outproc(&mut session);
        assert!(session.desktop.shell_push_fp.is_some(), "重推后指纹回填");
        {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                if shell_frame_of(&session, shell_face::SHELL).is_some() {
                    break;
                }
                assert!(std::time::Instant::now() < deadline, "p030 腿4 恢复帧超时");
                std::thread::yield_now();
            }
        }
        println!("AUTO030 leg4 watchdog-respawn-recovered PASS");

        // 帧留痕（AUTO_030_ASSETS=1 → docs/plans/reports/assets/030/）。
        if std::env::var("AUTO_030_ASSETS").is_ok() {
            let dir = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../docs/plans/reports/assets/030"
            );
            let _ = std::fs::create_dir_all(dir);
            for (name, face) in [
                ("bg-frame.txt", shell_face::DESKTOP_SURFACE),
                ("chrome-frame.txt", shell_face::SHELL),
            ] {
                if let Some(list) = shell_frame_of(&session, face) {
                    let out =
                        crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(&list);
                    let _ = std::fs::write(format!("{dir}/{name}"), out);
                }
            }
        }

        // 兜底清理。
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// PLAN-025 T-05：broker_key_event/broker_char/broker_scroll 路由
    /// 语义单测（真管道对端落 wire 断言——键盘/字符走焦点窗，滚轮走
    /// 指针命中窗；无焦点/未命中 = false 不路由）。
    #[test]
    fn broker_input_production_routes() {
        use crate::ui::desktop_protocol::message::{InputMsg, MouseButton, ProtocolMsg};
        use crate::ui::desktop_protocol::transport;
        use crate::ui::session::{AppId, DesktopSession};

        let pipe = format!("autodesk-broker-input-{}", std::process::id());
        let listener = transport::listen(&pipe).expect("listen");
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());

        // 虚拟窗 + broker client 装配（真管道对端）。
        let component = crate::build_dynamic_component(
            r#"widget t { view { text "x" } }"#,
            None,
        )
        .expect("build");
        let app_id = session.allocate_app(component);
        let wid = session.wm_add_win(
            app_id,
            "t".into(),
            iced::Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(480.0, 320.0)),
        );
        session.wm_focus(wid);
        let mut child_end = transport::connect(&pipe, 2000).expect("child connect");
        let host_end = listener.wait_connect().expect("host accept");
        let mut client =
            crate::ui::desktop_protocol::stage3::BrokerClient::new(pipe.clone(), host_end);
        client.wid = Some(wid);
        session.broker_clients.insert(pipe.clone(), client);

        // 管道投递有传输时延——预算内自旋收帧。
        fn wait_msg(
            child_end: &mut Box<dyn transport::Transport + Send>,
        ) -> Option<ProtocolMsg> {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            loop {
                if let Some(loaded) = child_end.try_recv() {
                    return Some(loaded.expect("解码"));
                }
                if std::time::Instant::now() >= deadline {
                    return None;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }

        // broker_char：焦点窗 → CharTyped 落 wire。
        assert!(session.broker_char('x'), "char 路由");
        match wait_msg(&mut child_end) {
            Some(ProtocolMsg::Input(InputMsg::CharTyped { wid: w, ch })) => {
                assert_eq!((w, ch), (wid.0, 'x'));
            }
            other => panic!("CharTyped 未落 wire: {other:?}"),
        }

        // broker_key_event：焦点窗 → KeyPressed 落 wire（VK_BACK）。
        assert!(session.broker_key_event(8, 0), "key 路由");
        match wait_msg(&mut child_end) {
            Some(ProtocolMsg::Input(InputMsg::KeyPressed { wid: w, key, .. })) => {
                assert_eq!((w, key), (wid.0, 8));
            }
            other => panic!("KeyPressed 未落 wire: {other:?}"),
        }

        // broker_scroll：指针命中窗 → Scroll 落 wire。
        assert!(session.broker_scroll(100.0, 100.0, 0.0, 15.0), "scroll 路由");
        match wait_msg(&mut child_end) {
            Some(ProtocolMsg::Input(InputMsg::Scroll { wid: w, dx, dy })) => {
                assert_eq!((w, dx, dy), (wid.0, 0.0, 15.0));
            }
            other => panic!("Scroll 未落 wire: {other:?}"),
        }
        // 窗外滚轮不路由。
        assert!(!session.broker_scroll(5000.0, 5000.0, 0.0, 1.0), "窗外不路由");

        // 焦点窗回收后键盘不路由（焦点窗语义——区别于 hit_test）。
        let _ = MouseButton::Left; // 触碰导入（button 族断言在 pointer_down 侧）
        let _ = AppId(0);
        let host = session.host.as_mut().unwrap();
        host.wm.wins.clear();
        host.wm.focused = None;
        assert!(!session.broker_char('y'), "焦点丢失不路由");
    }

    /// PLAN-026 T-05：broker_ime_commit/preedit/cancelled 路由（焦点窗
    /// → Ime* 落 wire；025 键盘路由同型——D2 定案协议级承载）。
    #[test]
    fn broker_ime_production_routes() {
        use crate::ui::desktop_protocol::message::{InputMsg, ProtocolMsg};
        use crate::ui::desktop_protocol::transport;
        use crate::ui::session::DesktopSession;
        // 独立装配第二会话（IME 断言与上一测试的焦点回收段解耦）。
        // 管道投递有传输时延——预算内自旋收帧（025 键盘路由测试同款）。
        fn wait_msg(
            child_end: &mut Box<dyn transport::Transport + Send>,
        ) -> Option<ProtocolMsg> {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            loop {
                if let Some(loaded) = child_end.try_recv() {
                    return Some(loaded.expect("解码"));
                }
                if std::time::Instant::now() >= deadline {
                    return None;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
        let pipe2 = format!("autodesk-broker-ime-{}", std::process::id());
        let listener2 = transport::listen(&pipe2).expect("listen2");
        let mut session2 = DesktopSession::__test_session();
        session2.open_desktop(iced::window::Id::unique());
        let component2 = crate::build_dynamic_component(
            r#"widget t2 { view { input "n" } }"#,
            None,
        )
        .expect("build2");
        let app_id2 = session2.allocate_app(component2);
        let wid2 = session2.wm_add_win(
            app_id2,
            "t2".into(),
            iced::Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(480.0, 320.0)),
        );
        session2.wm_focus(wid2);
        let mut child_end2 = transport::connect(&pipe2, 2000).expect("child connect2");
        let host_end2 = listener2.wait_connect().expect("host accept2");
        let mut client2 =
            crate::ui::desktop_protocol::stage3::BrokerClient::new(pipe2.clone(), host_end2);
        client2.wid = Some(wid2);
        session2.broker_clients.insert(pipe2.clone(), client2);

        assert!(session2.broker_ime_preedit("中文"), "preedit 路由");
        match wait_msg(&mut child_end2) {
            Some(ProtocolMsg::Input(InputMsg::ImePreedit { wid: w, text, .. })) => {
                assert_eq!((w, text.as_str()), (wid2.0, "中文"));
            }
            other => panic!("ImePreedit 未落 wire: {other:?}"),
        }
        assert!(session2.broker_ime_commit("中文"), "commit 路由");
        match wait_msg(&mut child_end2) {
            Some(ProtocolMsg::Input(InputMsg::ImeCommit { wid: w, text })) => {
                assert_eq!((w, text.as_str()), (wid2.0, "中文"));
            }
            other => panic!("ImeCommit 未落 wire: {other:?}"),
        }
        assert!(session2.broker_ime_cancelled(), "cancelled 路由");
        match wait_msg(&mut child_end2) {
            Some(ProtocolMsg::Input(InputMsg::ImeCancelled { wid: w })) => {
                assert_eq!(w, wid2.0);
            }
            other => panic!("ImeCancelled 未落 wire: {other:?}"),
        }
        // 焦点丢失不路由（与 025 键盘语义同界）。
        let host2 = session2.host.as_mut().unwrap();
        host2.wm.wins.clear();
        host2.wm.focused = None;
        assert!(!session2.broker_ime_commit("x"), "焦点丢失不路由");
    }

    /// PLAN-026 T-07 —— native display 族 e2e（`AUTO_DESKTOP_E2E=1` 门 +
    /// 026 载体 env；p025_native_input_arm 同型）。三腿真 exe 孵化：
    /// ①profile-card（004-profile-card 真源 a2r，queue 档）——image 占位 80×80 Quad
    /// + 渐变 col + 按钮帧（AC-02）；②display026 fixture——display 族
    /// 全件（icon lucide 占位 14×14 / badge / divider / avatar / spacer /
    /// scroll / grid 2×2 / center）帧断言（AC-02/03/05）；③003-converter
    /// IME——broker_ime_commit("100") 协议级注入 → 换算联动帧 212
    /// （AC-04，P020-D4 口径协议承载）。帧留痕 AUTO_026_ASSETS →
    /// docs/plans/reports/assets/026/（帧 dump 代截图——⑤口径）。
    /// 载体缺省寻址：`target/debug/{profile-card,display026,converter}.exe`
    /// + `../scratch026/{profile-card,026-display,003-converter}`；
    /// env AUTO_026_* 覆盖。
    #[test]
    fn p026_native_display_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        let manifest = env!("CARGO_MANIFEST_DIR");
        let exe_p = std::env::var("AUTO_026_PROFILE_EXE")
            .unwrap_or_else(|_| format!("{manifest}/../../target/debug/profile-card.exe"));
        let dir_p = std::path::PathBuf::from(
            std::env::var("AUTO_026_PROFILE_APP_DIR")
                .unwrap_or_else(|_| format!("{manifest}/../../scratch026/profile-card")),
        );
        let exe_d = std::env::var("AUTO_026_DISPLAY_EXE")
            .unwrap_or_else(|_| format!("{manifest}/../../target/debug/display026.exe"));
        let dir_d = std::path::PathBuf::from(
            std::env::var("AUTO_026_DISPLAY_APP_DIR")
                .unwrap_or_else(|_| format!("{manifest}/../../scratch026/026-display")),
        );
        let exe_c = std::env::var("AUTO_026_IME_EXE")
            .unwrap_or_else(|_| format!("{manifest}/../../target/debug/converter.exe"));
        let dir_c = std::path::PathBuf::from(
            std::env::var("AUTO_026_IME_APP_DIR")
                .unwrap_or_else(|_| format!("{manifest}/../../scratch026/003-converter")),
        );
        if !std::path::Path::new(&exe_p).is_file()
            || !dir_p.join("src/front/app.at").is_file()
            || !std::path::Path::new(&exe_d).is_file()
            || !dir_d.join("src/front/app.at").is_file()
            || !std::path::Path::new(&exe_c).is_file()
            || !dir_c.join("src/front/app.at").is_file()
        {
            eprintln!(
                "[p026] skip: 载体缺席（exe_p={exe_p} dir_p={} exe_d={exe_d} dir_d={} exe_c={exe_c} dir_c={}）",
                dir_p.display(),
                dir_d.display(),
                dir_c.display()
            );
            return;
        }

        use crate::ui::desktop_protocol::message::{DrawOp, FrameMode};
        use crate::ui::session::LaunchSpec;
        let broker_pipe = format!("autodesk-broker-026-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());

        let code_p =
            std::fs::read_to_string(dir_p.join("src/front/app.at")).expect("read 004");
        let code_d =
            std::fs::read_to_string(dir_d.join("src/front/app.at")).expect("read display");
        let code_c =
            std::fs::read_to_string(dir_c.join("src/front/app.at")).expect("read 003");
        let (exe_p, dir_p) = (exe_p.clone(), dir_p.clone());
        let (exe_d, dir_d) = (exe_d.clone(), dir_d.clone());
        let (exe_c, dir_c) = (exe_c.clone(), dir_c.clone());
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| match name {
                "profile-card" => Some(LaunchSpec {
                    code: code_p.clone(),
                    source_path: Some(
                        dir_p.join("src/front/app.at").to_string_lossy().to_string(),
                    ),
                    title: Some("Profile Card".into()),
                    name: Some("profile-card".into()),
                    daemon: None,
                    back_root: None,
                    fit: false,
                    exe: Some(std::path::PathBuf::from(&exe_p)),
                    render_decl: Some("queue".into()),
                    opens: Vec::new(),
                }),
                "026-display" => Some(LaunchSpec {
                    code: code_d.clone(),
                    source_path: Some(
                        dir_d.join("src/front/app.at").to_string_lossy().to_string(),
                    ),
                    title: Some("Display026".into()),
                    name: Some("display026".into()),
                    daemon: None,
                    back_root: None,
                    fit: false,
                    exe: Some(std::path::PathBuf::from(&exe_d)),
                    render_decl: Some("queue".into()),
                    opens: Vec::new(),
                }),
                "003-converter" => Some(LaunchSpec {
                    code: code_c.clone(),
                    source_path: Some(
                        dir_c.join("src/front/app.at").to_string_lossy().to_string(),
                    ),
                    title: Some("Converter".into()),
                    name: Some("converter".into()),
                    daemon: None,
                    back_root: None,
                    fit: false,
                    exe: Some(std::path::PathBuf::from(&exe_c)),
                    render_decl: Some("queue".into()),
                    opens: Vec::new(),
                }),
                _ => None,
            }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        fn origin_of(session: &DesktopSession, wid: Wid) -> (f32, f32) {
            session
                .host
                .as_ref()
                .and_then(|h| h.wm.wins.get(&wid))
                .map(|v| {
                    let r = *v.rect.borrow();
                    (r.x, r.y)
                })
                .expect("窗原点")
        }
        fn wait_frame(
            session: &mut DesktopSession,
            app: &str,
            pred: impl Fn(&[DrawOp]) -> bool,
            what: &str,
        ) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                let hit = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app))
                    .and_then(|c| c.composed())
                    .is_some_and(|l| pred(&l.ops));
                if hit {
                    return;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "{what} 超时: {:?}",
                    session
                        .broker_clients
                        .values()
                        .find(|c| c.app_name.as_deref() == Some(app))
                        .and_then(|c| c.composed())
                        .map(|l| l.ops.iter().map(|o| format!("{o:?}")).collect::<Vec<_>>())
                );
                std::thread::yield_now();
            }
        }
        fn quads_of(ops: &[DrawOp]) -> Vec<(f32, f32, f32, f32)> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Quad { rect, .. } => Some((rect.x, rect.y, rect.w, rect.h)),
                    _ => None,
                })
                .collect()
        }
        fn texts_of(ops: &[DrawOp]) -> Vec<String> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Text { text, .. } | DrawOp::TextStyled { text, .. } => {
                        Some(text.clone())
                    }
                    _ => None,
                })
                .collect()
        }
        // PLAN-028：image op 定位器（帧内定位法扩展——真图升级断言面）。
        fn images_of(ops: &[DrawOp]) -> Vec<(f32, f32, f32, f32, String)> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Image { rect, src, .. } => {
                        Some((rect.x, rect.y, rect.w, rect.h, src.clone()))
                    }
                    _ => None,
                })
                .collect()
        }

        // —— ①profile-card（004 真源）：queue 孵化 + image 真图 op/
        // 渐变 col/按钮。PLAN-028 归因：原 80×80 占位 Quad 断言改写为
        // Image op 断言（占位保真 → 真图 op 同位替换，rect 推导零变化）。
        let wid_p = session.launch_app("profile-card").expect("profile launch");
        place_window(&mut session, wid_p, 0);
        wait_frame(
            &mut session,
            "profile-card",
            |ops| {
                images_of(ops)
                    .iter()
                    .any(|r| r.2 == 80.0 && r.3 == 80.0 && r.4.contains("cravatar"))
            },
            "004 image 真图 op 帧",
        );
        let mode_p = session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_p))
            .map(|c| c.endpoint.frame_mode)
            .expect("profile client");
        assert_eq!(mode_p, FrameMode::Commands, "004 显式 queue → Commands");
        if let Some(list) = session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_p))
            .and_then(|c| c.composed())
        {
            let out = crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(list);
            eprintln!("[p026-dbg] 004 frame:
{out}");
        }
        {
            let texts = session
                .broker_clients
                .values()
                .find(|c| c.wid == Some(wid_p))
                .and_then(|c| c.composed())
                .map(|l| texts_of(&l.ops))
                .expect("004 composed");
            assert!(
                texts.iter().any(|t| t.contains("Jane Cooper")),
                "004 姓名文本在帧: {texts:?}"
            );
            assert!(
                texts.iter().any(|t| t.contains("Follow")),
                "004 按钮标签在帧: {texts:?}"
            );
        }

        // —— ②display026：display 族全件帧断言。
        let wid_d = session.launch_app("026-display").expect("display launch");
        place_window(&mut session, wid_d, 1);
        wait_frame(
            &mut session,
            "026-display",
            |ops| {
                texts_of(ops).iter().any(|t| t == "ga")
                    && texts_of(ops).iter().any(|t| t == "gd")
            },
            "display grid 帧",
        );
        {
            let frame = session
                .broker_clients
                .values()
                .find(|c| c.wid == Some(wid_d))
                .and_then(|c| c.composed())
                .expect("display composed");
            let mode_d = session
                .broker_clients
                .values()
                .find(|c| c.wid == Some(wid_d))
                .map(|c| c.endpoint.frame_mode)
                .expect("display client");
            assert_eq!(mode_d, FrameMode::Commands, "display 显式 queue → Commands");
            let texts = texts_of(&frame.ops);
            let quads = quads_of(&frame.ops);
            for want in ["Active", "ga", "gb", "gc", "gd", "centered", "JC", "long content"] {
                assert!(
                    texts.iter().any(|t| t.contains(want)),
                    "display 帧 text {want}: {texts:?}"
                );
            }
            // PLAN-028 归因：icon 原 14×14 占位方块断言改写为 Image op
            // 断言——icon a2r 降级形态（lucide:）随图像通道升级入线，
            // 宿主字形解析 not-yet（P026-D1 后半维持）→ 未解析降级占位
            // 仍兜底，解释态/native 行为连续。
            assert!(
                images_of(&frame.ops)
                    .iter()
                    .any(|r| r.2 == 14.0 && r.3 == 14.0 && r.4.starts_with("lucide:")),
                "icon 14×14 image op（lucide 降级形态）: {:?}",
                images_of(&frame.ops)
            );
            // divider h-1 → 4px 线 quad。
            assert!(
                quads.iter().any(|r| r.3 == 4.0 && r.2 > 40.0),
                "divider 4px 线: {quads:?}"
            );
            // scroll 内容文本已断言（"long content"）——视口无 bg 不产
            // quad，几何由 grid_layout/display golden 单测钉。
        }

        // —— ③003-converter IME：协议级 ImeCommit 注入 → 联动帧。
        let wid_c = session.launch_app("003-converter").expect("converter launch");
        place_window(&mut session, wid_c, 2);
        wait_frame(
            &mut session,
            "003-converter",
            |ops| {
                quads_of(ops)
                    .iter()
                    .any(|r| r.2 == 320.0 && r.3 == 32.0)
            },
            "003 input 帧",
        );
        let (ox, oy) = origin_of(&session, wid_c);
        let (ix, iy) = session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_c))
            .and_then(|c| c.composed())
            .map(|l| {
                let q = quads_of(&l.ops);
                (q[0].0, q[0].1)
            })
            .expect("composed quads");
        assert!(
            session.broker_pointer_down(ox + ix + 160.0, oy + iy + 16.0, MouseButton::Left),
            "IME 腿聚焦 celsius"
        );
        // 协议级 ImeCommit 注入（数字串经 IME 组合提交——⑤口径承载）。
        assert!(session.broker_ime_commit("100"), "broker_ime_commit 路由");
        wait_frame(
            &mut session,
            "003-converter",
            |ops| texts_of(ops).iter().any(|t| t.contains("212")),
            "IME 注入联动帧（celsius=100 → f=212）",
        );

        // 帧留痕（AUTO_026_ASSETS=1 → docs/plans/reports/assets/026/）。
        if std::env::var("AUTO_026_ASSETS").is_ok() {
            let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/plans/reports/assets/026");
            let _ = std::fs::create_dir_all(&assets);
            for (app, file) in [
                ("profile-card", "profile-card-frame.txt"),
                ("026-display", "display-frame.txt"),
                ("003-converter", "converter-ime-frame.txt"),
            ] {
                if let Some(list) = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app))
                    .and_then(|c| c.composed())
                {
                    let out = crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(list);
                    let _ = std::fs::write(assets.join(file), out);
                }
            }
        }

        // 兜底清理。
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// PLAN-028 T-07 —— 图像通道 e2e（`AUTO_DESKTOP_E2E=1` 门；t3 真子
    /// 进程 re-exec 模式）。腿：①004（http 远程 URL src——真子进程 queue
    /// 帧含 80×80 Image op + src 代入）；②p028 语料（capability-tests
    /// 构造件——data:/builtin:/thumbnail://vault/本地文件/不可达 http 五
    /// 形态一帧全数入帧；029-photo-gallery 源解释态编译器不可 parse
    /// （探针实证 20 错），本地文件腿按计划 §5.6 "029/fixture" 措辞由
    /// fixture 承载 + 直接消费 029 缩略文件）；③宿主侧解析三路径实驱
    /// （thumbnail 命中/本地文件/离线负缓存——快照替身注入）+ 度量行
    /// （解码成本 ms / 帧字节增量 ≈ src 串长）。帧留痕 AUTO_028_ASSETS=1
    /// → docs/plans/reports/assets/028/。
    #[test]
    fn p028_image_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        use crate::ui::desktop_protocol::message::DrawOp;
        fn images_of(ops: &[DrawOp]) -> Vec<(f32, f32, f32, f32, String)> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Image { rect, src, .. } => {
                        Some((rect.x, rect.y, rect.w, rect.h, src.clone()))
                    }
                    _ => None,
                })
                .collect()
        }
        // p026 同款帧谓词轮询（局部 helper——同模块各 e2e 自带）。
        fn wait_frame(
            session: &mut DesktopSession,
            app: &str,
            pred: impl Fn(&[DrawOp]) -> bool,
            what: &str,
        ) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                session.pump_broker_clients();
                let hit = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app))
                    .and_then(|c| c.composed())
                    .is_some_and(|l| pred(&l.ops));
                if hit {
                    return;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "{what} 超时: {:?}",
                    session
                        .broker_clients
                        .values()
                        .find(|c| c.app_name.as_deref() == Some(app))
                        .and_then(|c| c.composed())
                        .map(|l| l.ops.iter().map(|o| format!("{o:?}")).collect::<Vec<_>>())
                );
                std::thread::yield_now();
            }
        }
        let broker_pipe = format!("autodesk-broker-028-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());

        // 载体源：004 + p028 语料（example_source 双根解析命中
        // capability-tests）。
        let names = ["004-profile-card", "p028-image-channel"];
        let sources: Vec<(String, String)> = names
            .iter()
            .map(|n| (n.to_string(), example_source(n)))
            .collect();
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| {
                sources
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(n, src)| LaunchSpec {
                        code: src.clone(),
                        source_path: None,
                        title: Some(n.to_string()),
                        name: None,
                        fit: false,
                        daemon: None,
                        back_root: None,
                        exe: None,
                        opens: Vec::new(),
                        render_decl: Some("queue".into()),
                    })
            }));
        let broker_for_spawn = broker_pipe.clone();
        session.desktop.outproc_spawner = Some(std::sync::Arc::new(move |child_name| {
            Ok(spawn_t3_child(&broker_for_spawn, child_name, "queue"))
        }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        // —— ①004：http 远程 URL src → 80×80 Image op + src 代入。
        let wid_p = session.launch_app("004-profile-card").expect("profile launch");
        place_window(&mut session, wid_p, 0);
        wait_frame(
            &mut session,
            "004-profile-card",
            |ops| {
                images_of(ops)
                    .iter()
                    .any(|r| r.2 == 80.0 && r.3 == 80.0 && r.4.contains("cravatar"))
            },
            "004 image 真图 op 帧",
        );

        // —— ②p028 语料：五 src 形态一帧全数入帧（rect = Tailwind 刻度
        // 40×40 / 64×36 / 128×72 / 48×48 / 32×32——占位同位推导零变化）。
        let wid_i = session.launch_app("p028-image-channel").expect("corpus launch");
        place_window(&mut session, wid_i, 1);
        wait_frame(
            &mut session,
            "p028-image-channel",
            |ops| images_of(ops).len() == 5,
            "p028 语料五 Image op 帧",
        );
        {
            let frame = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some("p028-image-channel"))
                .and_then(|c| c.composed())
                .expect("corpus composed");
            let ims = images_of(&frame.ops);
            for (w, h) in [(40.0, 40.0), (64.0, 36.0), (128.0, 72.0), (48.0, 48.0), (32.0, 32.0)] {
                assert!(
                    ims.iter().any(|r| r.2 == w && r.3 == h),
                    "image rect {w}x{h}: {ims:?}"
                );
            }
            assert!(
                ims.iter().any(|r| r.4.starts_with("data:image/png;base64,")),
                "data: 形态: {ims:?}"
            );
            assert!(ims.iter().any(|r| r.4 == "builtin:ricepaper"), "builtin: 形态");
            assert!(ims.iter().any(|r| r.4 == "thumbnail://42842"), "thumbnail:// 形态");
            assert!(
                ims.iter().any(|r| r.4.ends_with("thumb_001.jpg")),
                "本地文件形态: {ims:?}"
            );
            assert!(
                ims.iter().any(|r| r.4 == "http://127.0.0.1:1/zero28.png"),
                "不可达 http 形态（帧内合法——降级归宿主侧）"
            );
            // 度量行：帧字节增量 ≈ src 串长（编码帧长 vs src 长度合计）。
            let mut buf = Vec::new();
            frame.encode(&mut buf);
            let src_bytes: usize = ims.iter().map(|r| r.4.len()).sum();
            eprintln!(
                "[p028-metric] frame_bytes={} src_bytes={src_bytes} ops={}",
                buf.len(),
                frame.ops.len()
            );
        }

        // —— ③宿主侧解析三路径实驱（快照替身注入；解码/缓存/降级语义
        // 单测面 = broker_surface t028_*，此处为 e2e 进程内实证）。
        crate::ui::iced::snapshot::cache_put(
            crate::ui::session::Wid(42842),
            crate::ui::iced::snapshot::WindowSnapshot {
                rgba: vec![1, 2, 3, 255],
                w: 1,
                h: 1,
            },
        );
        let handle =
            crate::ui::iced::broker_surface::resolve_drawlist_image("thumbnail://42842", 96, 56)
                .expect("thumbnail 命中（cache_put 注入替身后直出）");
        drop(handle);
        // 命中后 SNAPSHOT_TTL 2s 内为新鲜——清除请求队列作 SWR 断言基线。
        let _ = crate::ui::iced::snapshot::take_capture_requests();
        // 本地文件腿：nextest cwd = crate manifest 目录——仓库根相对形
        // 态经 manifest 拼绝对路径驱动（语料内相对字面量仅承载帧级断言）。
        let thumb_abs = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/ui/029-photo-gallery/src/front/thumbnails/thumb_001.jpg"
        );
        let t0 = std::time::Instant::now();
        let file_handle =
            crate::ui::iced::broker_surface::resolve_drawlist_image(thumb_abs, 96, 56)
                .expect("本地文件解析（绝对路径）");
        drop(file_handle);
        let decode_ms = t0.elapsed().as_millis();
        eprintln!("[p028-metric] local file resolve = {decode_ms} ms（缓存命中后重复解析 <1ms）");
        let t1 = std::time::Instant::now();
        assert!(
            crate::ui::iced::broker_surface::resolve_drawlist_image(thumb_abs, 96, 56).is_some(),
            "二次解析 = 缓存命中"
        );
        eprintln!("[p028-metric] cached resolve = {} ms", t1.elapsed().as_millis());
        // 离线降级腿：不可达 http → 当帧占位（None）+ 后台负缓存落地。
        assert!(
            crate::ui::iced::broker_surface::resolve_drawlist_image(
                "http://127.0.0.1:1/zero28.png",
                96,
                56
            )
            .is_none(),
            "不可达 http 首帧 = 占位（不阻塞）"
        );

        // 帧留痕（AUTO_028_ASSETS=1 → docs/plans/reports/assets/028/）。
        if std::env::var("AUTO_028_ASSETS").is_ok() {
            let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/plans/reports/assets/028");
            let _ = std::fs::create_dir_all(&assets);
            for (app, file) in [
                ("004-profile-card", "profile-card-frame.txt"),
                ("p028-image-channel", "image-channel-frame.txt"),
            ] {
                if let Some(list) = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app))
                    .and_then(|c| c.composed())
                {
                    let out =
                        crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(list);
                    let _ = std::fs::write(assets.join(file), out);
                }
            }
        }

        // 兜底清理。
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// PLAN-029 T-03（D2 主腿）：live 输入泵 e2e——`DesktopEvent::LiveInput`
    /// 的生产路由入口 `route_live_input`（真子进程 003-converter，queue 臂
    /// re-exec 孵化——p028 免载体形态）驱动真键入语义链：⑤协议级直调腿
    /// （broker_char 先例延续）+ live 四型腿（Chars 键入联动 / KeyPressed
    /// 退格 / ImeCommit 并入 / Wheel last_cursor 命中窗路由）。
    /// `AUTO_DESKTOP_E2E=1` 门。留痕 `AUTO_029_ASSETS=1` → assets/029/。
    #[test]
    fn p029_live_input_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        use crate::ui::desktop_protocol::message::{DrawOp, FrameMode, MouseButton};
        use crate::ui::session::{DesktopSession, LaunchSpec, LiveInput};

        let broker_pipe = format!("autodesk-broker-029-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());

        session.desktop.app_resolver = Some(std::sync::Arc::new(move |name: &str| {
            // code = 裁决/挂载用最小占位（host 侧 attach 建视图判定）；
            // 子真身走 t3_child_body typed 分支（P029TypedInputs）。
            (name == "p029-typed-inputs").then(|| LaunchSpec {
                code: r#"widget t { view { text "x" } }"#.to_string(),
                source_path: None,
                title: Some("P029Inputs".into()),
                name: Some("p029-inputs".into()),
                fit: false,
                daemon: None,
                back_root: None,
                exe: None,
                opens: Vec::new(),
                render_decl: Some("queue".into()),
            })
        }));
        let broker_for_spawn = broker_pipe.clone();
        session.desktop.outproc_spawner = Some(std::sync::Arc::new(move |child_name| {
            Ok(spawn_t3_child(&broker_for_spawn, child_name, "native"))
        }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        fn wait_frame(
            session: &mut DesktopSession,
            app: &str,
            pred: impl Fn(&[DrawOp]) -> bool,
            what: &str,
        ) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                let hit = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app))
                    .and_then(|c| c.composed())
                    .is_some_and(|l| pred(&l.ops));
                if hit {
                    return;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "{what} 超时: {:?}",
                    session
                        .broker_clients
                        .values()
                        .find(|c| c.app_name.as_deref() == Some(app))
                        .and_then(|c| c.composed())
                        .map(|l| l.ops.iter().map(|o| format!("{o:?}")).collect::<Vec<_>>())
                );
                std::thread::yield_now();
            }
        }
        fn quads_of(ops: &[DrawOp]) -> Vec<(f32, f32, f32, f32)> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Quad { rect, .. } => Some((rect.x, rect.y, rect.w, rect.h)),
                    _ => None,
                })
                .collect()
        }
        fn texts_of(ops: &[DrawOp]) -> Vec<String> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Text { text, .. } => Some(text.clone()),
                    _ => None,
                })
                .collect()
        }

        // —— p029-typed-inputs：native queue 孵化 → 首帧（placeholder 在场）。
        let wid_c = session.launch_app("p029-typed-inputs").expect("p029 inputs launch");
        if let Some(host) = session.host.as_mut() {
            if let Some(v) = host.wm.wins.get_mut(&wid_c) {
                let mut rect = *v.rect.borrow();
                rect.x = 16.0;
                rect.y = 16.0;
                *v.rect.borrow_mut() = rect;
            }
        }
        wait_frame(
            &mut session,
            "p029-typed-inputs",
            |ops| texts_of(ops).iter().any(|t| t.contains("type here")),
            "p029 首帧（placeholder 在场）",
        );
        let mode_c = session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_c))
            .map(|c| c.endpoint.frame_mode)
            .expect("p029 client");
        assert_eq!(mode_c, FrameMode::Commands, "p029 显式 queue → Commands");

        let (ox, oy) = {
            let host = session.host.as_ref().unwrap();
            let r = *host.wm.wins.get(&wid_c).unwrap().rect.borrow();
            (r.x, r.y)
        };
        // input 定位：placeholder 文本 op 坐标（首帧 buffer 空 → placeholder
        // 显示在 input 内部左上）。
        let (tx, ty) = session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid_c))
            .and_then(|c| c.composed())
            .and_then(|l| {
                l.ops.iter().find_map(|op| match op {
                    DrawOp::Text { x, y, text, .. } if text.contains("type here") => {
                        Some((*x, *y))
                    }
                    _ => None,
                })
            })
            .expect("placeholder 坐标");
        assert!(session.broker_pointer_down(ox + tx + 4.0, oy + ty + 2.0, MouseButton::Left));

        // —— ⑤协议级直调腿（broker_char 先例延续；p025 口径在册）。
        assert!(session.broker_char('h'), "⑤ broker_char 路由");
        wait_frame(
            &mut session,
            "p029-typed-inputs",
            |ops| texts_of(ops).iter().any(|t| t == "echo:h"),
            "⑤ 协议级键入（broker_char h -> echo:h）",
        );
        println!("AUTO029-LIVE protocol leg PASS (broker_char h -> echo:h)");

        // —— live Chars 腿：LiveInput 泵入（生产路由入口）→ "i" 并入。
        assert!(session.route_live_input(&LiveInput::Chars { text: "i".into() }));
        wait_frame(
            &mut session,
            "p029-typed-inputs",
            |ops| texts_of(ops).iter().any(|t| t == "echo:hi"),
            "live Chars 键入联动（h+i -> echo:hi）",
        );
        println!("AUTO029-LIVE chars leg PASS (live Chars i -> echo:hi)");

        // —— live KeyPressed 腿：VK_BACK(8) 退格 → "h"。
        assert!(session.route_live_input(&LiveInput::KeyPressed { key: 8, modifiers: 0 }));
        wait_frame(
            &mut session,
            "p029-typed-inputs",
            |ops| texts_of(ops).iter().any(|t| t == "echo:h"),
            "live VK_BACK 退格（hi -> echo:h）",
        );
        println!("AUTO029-LIVE key leg PASS (VK_BACK -> echo:h)");

        // —— live ImeCommit 腿：组合串并入 buffer → "h文"。
        assert!(session.route_live_input(&LiveInput::ImeCommit { text: "文".into() }));
        wait_frame(
            &mut session,
            "p029-typed-inputs",
            |ops| texts_of(ops).iter().any(|t| t == "echo:h文"),
            "live ImeCommit 并入（h+文 -> echo:h文）",
        );
        println!("AUTO029-LIVE ime leg PASS (ImeCommit 文 -> echo:h文)");

        // —— live Wheel 腿：last_cursor 置窗心 → 命中窗路由（载体无
        //    Scrollable，路由成功 = child 管道投递 ok）。
        if let Some(host) = session.host.as_mut() {
            host.wm.last_cursor.set(iced::Point::new(ox + 100.0, oy + 100.0));
        }
        assert!(
            session.route_live_input(&LiveInput::Wheel { dx: 0.0, dy: -40.0 }),
            "live Wheel 命中窗路由"
        );
        println!("AUTO029-LIVE wheel leg PASS (routed to hit window)");

        // 帧留痕（AUTO_029_ASSETS=1 → docs/plans/reports/assets/029/）。
        if std::env::var("AUTO_029_ASSETS").is_ok() {
            let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/plans/reports/assets/029");
            let _ = std::fs::create_dir_all(&assets);
            if let Some(list) = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some("p029-typed-inputs"))
                .and_then(|c| c.composed())
            {
                let out =
                    crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(list);
                let _ = std::fs::write(assets.join("live-input-frame.txt"), out);
            }
        }

        // 兜底清理。
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// PLAN-029 T-09：shell 面渲染 + 命中闭环 e2e——真 outproc native 子
    /// 进程（p029-shell-face 五件套语料）经 wire 合成帧断言：thumbnail://
    /// 与 lucide: 的 Image op 落 wire（宿主解析侧 T-07 单测在册）+
    /// popover 命中闭环（锚开 → 面板项派发 → 外点 on_dismiss）+
    /// mousearea 命中。留痕 `AUTO_029_ASSETS=1` → assets/029/。
    #[test]
    fn p029_shell_face_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        use crate::ui::desktop_protocol::message::{DrawOp, FrameMode, MouseButton};
        use crate::ui::session::{DesktopSession, LaunchSpec};

        let broker_pipe = format!("autodesk-broker-029f-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());

        session.desktop.app_resolver = Some(std::sync::Arc::new(move |name: &str| {
            (name == "p029-shell-face").then(|| LaunchSpec {
                code: r#"widget t { view { text "x" } }"#.to_string(),
                source_path: None,
                title: Some("P029ShellFace".into()),
                name: Some("p029-shell-face".into()),
                fit: false,
                daemon: None,
                back_root: None,
                exe: None,
                opens: Vec::new(),
                render_decl: Some("queue".into()),
            })
        }));
        let broker_for_spawn = broker_pipe.clone();
        session.desktop.outproc_spawner = Some(std::sync::Arc::new(move |child_name| {
            Ok(spawn_t3_child(&broker_for_spawn, child_name, "native"))
        }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        fn wait_frame(
            session: &mut DesktopSession,
            app: &str,
            pred: impl Fn(&[DrawOp]) -> bool,
            what: &str,
        ) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                let hit = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app))
                    .and_then(|c| c.composed())
                    .is_some_and(|l| pred(&l.ops));
                if hit {
                    return;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "{what} 超时: {:?}",
                    session
                        .broker_clients
                        .values()
                        .find(|c| c.app_name.as_deref() == Some(app))
                        .and_then(|c| c.composed())
                        .map(|l| l.ops.iter().map(|o| format!("{o:?}")).collect::<Vec<_>>())
                );
                std::thread::yield_now();
            }
        }
        fn texts_of(ops: &[DrawOp]) -> Vec<String> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Text { text, .. } => Some(text.clone()),
                    _ => None,
                })
                .collect()
        }
        fn image_srcs_of(ops: &[DrawOp]) -> Vec<String> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Image { src, .. } => Some(src.clone()),
                    _ => None,
                })
                .collect()
        }
        // 命中区（text op 坐标 → 窗内点击点）。
        fn text_pos_of(ops: &[DrawOp], needle: &str) -> Option<(f32, f32)> {
            ops.iter().find_map(|op| match op {
                DrawOp::Text { x, y, text, .. } if text.contains(needle) => Some((*x, *y)),
                _ => None,
            })
        }

        let wid = session.launch_app("p029-shell-face").expect("shell-face launch");
        if let Some(host) = session.host.as_mut() {
            if let Some(v) = host.wm.wins.get_mut(&wid) {
                let mut rect = *v.rect.borrow();
                rect.x = 16.0;
                rect.y = 16.0;
                *v.rect.borrow_mut() = rect;
            }
        }
        // ①首帧：thumbnail/lucide Image op 落 wire（五件套渲染面）。
        wait_frame(
            &mut session,
            "p029-shell-face",
            |ops| {
                let srcs = image_srcs_of(ops);
                srcs.iter().any(|s| s.starts_with("thumbnail://42842!"))
                    && srcs.iter().any(|s| s == "lucide:panel-top")
                    && srcs.iter().any(|s| s.starts_with("workspace://0!"))
            },
            "shell 面五件套首帧（thumbnail/lucide/workspace Image op）",
        );
        let (ox, oy) = {
            let host = session.host.as_ref().unwrap();
            let r = *host.wm.wins.get(&wid).unwrap().rect.borrow();
            (r.x, r.y)
        };
        println!("AUTO029-FACE five-kind ops PASS (thumbnail:// + lucide: + workspace:// on wire)");

        // ②popover 命中闭环：点锚按钮（menu）→ 开 → 面板项在帧。
        let ops = face_ops(&session);
        let (mx, my) = text_pos_of(&ops, "menu").expect("锚按钮坐标");
        assert!(session.broker_pointer_down(ox + mx + 2.0, oy + my + 2.0, MouseButton::Left));
        wait_frame(
            &mut session,
            "p029-shell-face",
            |ops| texts_of(ops).iter().any(|t| *t == "menu-item"),
            "popover 开态面板渲染",
        );
        println!("AUTO029-FACE popover open PASS (menu-item panel on wire)");

        // ③面板项命中 → MenuAction 派发（log:action）——点面板内按钮
        //（text 节点无命中项，点文本会落 catcher 关面板——按设计）。
        let ops = face_ops(&session);
        let (ax, ay) = text_pos_of(&ops, "act").expect("面板按钮坐标");
        assert!(session.broker_pointer_down(ox + ax + 2.0, oy + ay + 2.0, MouseButton::Left));
        wait_frame(
            &mut session,
            "p029-shell-face",
            |ops| texts_of(ops).iter().any(|t| t == "log:action"),
            "面板项命中派发（MenuAction）",
        );
        println!("AUTO029-FACE popover item PASS (MenuAction dispatched)");

        // ④外点 → on_dismiss（log:dismissed + 面板收）。
        assert!(session.broker_pointer_down(ox + 460.0, oy + 300.0, MouseButton::Left));
        wait_frame(
            &mut session,
            "p029-shell-face",
            |ops| {
                texts_of(ops).iter().any(|t| t == "log:dismissed")
                    && !texts_of(ops).iter().any(|t| *t == "menu-item")
            },
            "外点 on_dismiss + 面板收",
        );
        println!("AUTO029-FACE popover dismiss PASS (on_dismiss + panel closed)");

        // ⑤mousearea 命中 → AreaClick。
        let ops = face_ops(&session);
        let (hx, hy) = text_pos_of(&ops, "hit-area").expect("area 坐标");
        assert!(session.broker_pointer_down(ox + hx + 2.0, oy + hy + 2.0, MouseButton::Left));
        wait_frame(
            &mut session,
            "p029-shell-face",
            |ops| texts_of(ops).iter().any(|t| t == "log:area"),
            "mousearea 命中（AreaClick）",
        );
        println!("AUTO029-FACE mousearea PASS (AreaClick dispatched)");

        // 帧留痕（AUTO_029_ASSETS=1 → docs/plans/reports/assets/029/）。
        if std::env::var("AUTO_029_ASSETS").is_ok() {
            let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/plans/reports/assets/029");
            let _ = std::fs::create_dir_all(&assets);
            if let Some(list) = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some("p029-shell-face"))
                .and_then(|c| c.composed())
            {
                let out =
                    crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(list);
                let _ = std::fs::write(assets.join("shell-face-frame.txt"), out);
            }
        }

        // 兜底清理。
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// p029_shell_face_arm：当前合成帧 ops（免借用冲突的快照取用）。
    fn face_ops(session: &DesktopSession) -> Vec<DrawOp> {
        session
            .broker_clients
            .values()
            .find(|c| c.app_name.as_deref() == Some("p029-shell-face"))
            .and_then(|c| c.composed())
            .map(|l| l.ops.clone())
            .unwrap_or_default()
    }

    fn composed_texts(session: &DesktopSession, app: &str) -> Vec<String> {
        session
            .broker_clients
            .values()
            .find(|c| c.app_name.as_deref() == Some(app))
            .and_then(|c| c.composed())
            .map(|list| {
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
            })
            .unwrap_or_default()
    }

    /// 等待条件成立（泵帧 + 谓词轮询）。
    fn wait_frames(
        session: &mut DesktopSession,
        mut pred: impl FnMut(&DesktopSession) -> bool,
    ) -> bool {
        for _ in 0..600 {
            session.pump_broker_clients();
            if pred(session) {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        false
    }

    /// T3 主体一：001–005 五示例 queue 模式端到端（re-exec 真子进程 +
    /// 真协议泵）——孵化全 Active → 逐示例交互闭环（本地孪生投影器出
    /// 命中坐标，与 child 同引擎同源确定性布局）→ 帧内容断言 → 收尾。
    #[test]
    fn t3_examples_queue_end_to_end() {
        let broker_pipe = format!("autodesk-broker-t3-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let names: Vec<String> = T3_EXAMPLES.iter().map(|s| s.to_string()).collect();
        let sources: Vec<(String, String)> = names
            .iter()
            .map(|n| (n.clone(), example_source(n)))
            .collect();
        session.desktop.app_resolver = Some(std::sync::Arc::new(move |name: &str| {
            sources
                .iter()
                .find(|(n, _)| n == name)
                .map(|(n, src)| LaunchSpec {
                    code: src.clone(),
                    source_path: None,
                    title: Some(n.clone()),
                    name: None,
                    fit: false,
                    daemon: None,
                    back_root: None,
        exe: None,
            opens: Vec::new(),
        render_decl: None,    })
        }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        // 孪生投影器（同源布局 → 命中坐标；PLAN-033 T-04 迁 native——
        // 与子进程同布局引擎）。
        let twins: Vec<(String, RqProjector<crate::ui::dynamic::DynamicComponent>)> =
            names
                .iter()
                .map(|n| {
                    let src = example_source(n);
                    let comp = crate::build_dynamic_component(&src, None).expect("twin build");
                    let mut p = RqProjector::new(
                        comp, T3_W, T3_H,
                    );
                    {
                        use crate::ui::desktop_protocol::endpoint::FrameSource;
                        p.render_frame();
                    }
                    (n.clone(), p)
                })
                .collect();
        let twin_hit = |app: &str, needle: &str| -> Option<(f32, f32)> {
            twins
                .iter()
                .find(|(n, _)| n == app)
                .and_then(|(_, p)| {
                    p.hit_regions()
                        .into_iter()
                        .find(|(_, k)| k.contains(needle))
                        .map(|(r, _)| (r.x + r.w / 2.0, r.y + r.h / 2.0))
                })
        };
        /// 命中区序选（row 内第 idx 个匹配；002 的 "+" 是第三个按钮）。
        let twin_hit_nth = |app: &str, needle: &str, idx: usize| -> Option<(f32, f32)> {
            twins
                .iter()
                .find(|(n, _)| n == app)
                .and_then(|(_, p)| {
                    p.hit_regions()
                        .into_iter()
                        .filter(|(_, k)| k.contains(needle))
                        .nth(idx)
                        .map(|(r, _)| (r.x + r.w / 2.0, r.y + r.h / 2.0))
                })
        };

        let mut children: Vec<std::process::Child> =
            names.iter().map(|n| spawn_t3_child(&broker_pipe, n, "queue")).collect();

        attach_until(&mut session, names.len());
        let landed = session.broker_clients.values().filter_map(|c| c.wid).count();
        assert_eq!(landed, names.len(), "五示例全孵化落地");
        let wids: Vec<Wid> =
            session.broker_clients.values().filter_map(|c| c.wid).collect();
        for (i, wid) in wids.into_iter().enumerate() {
            place_window(&mut session, wid, i);
        }

        // ---- 逐示例交互闭环（坐标 = 孪生命中区中心 + 窗原点）。 ----
        let origin_of = |session: &DesktopSession, app: &str| -> Option<(f32, f32)> {
            session.broker_clients.values().find(|c| c.app_name.as_deref() == Some(app))
                .and_then(|c| c.wid)
                .and_then(|wid| {
                    session.host.as_ref().and_then(|h| h.wm.wins.get(&wid))
                        .map(|v| { let r = *v.rect.borrow(); (r.x, r.y) })
                })
        };

        // 001/004：帧到位即断言（无交互 widget）。
        assert!(
            wait_frames(&mut session, |s| composed_texts(s, "001-helloworld")
                .iter()
                .any(|t| t == "Hello, World!")),
            "001 帧文本到位"
        );
        assert!(
            wait_frames(&mut session, |s| composed_texts(s, "004-profile-card")
                .iter()
                .any(|t| t == "Jane Cooper")),
            "004 帧文本到位"
        );

        // Plan 515 G1：scroll 溢出 → scissor 栈帧到位（端到端视觉断言：
        // 双视口矩形 + 栈平衡（嵌套 ≤2 层）+ 溢出内容仍在 ops——裁剪是
        // 宿主栅格职责，投影器不删内容）+ 视口外按钮命中区被过滤。
        assert!(
            wait_frames(&mut session, |s| {
                s.broker_clients.values().any(|c| {
                    c.app_name.as_deref() == Some("p515-scroll-overflow")
                        && c.composed().is_some_and(|l| {
                            let mut depth = 0i32;
                            let mut max_depth = 0i32;
                            let mut saw_outer = false;
                            let mut saw_inner = false;
                            for op in &l.ops {
                                match op {
                                    crate::ui::desktop_protocol::message::DrawOp::Scissor { rect } => {
                                        depth += 1;
                                        max_depth = max_depth.max(depth);
                                        if (rect.h - 128.0).abs() < 0.5 {
                                            saw_outer = true;
                                        }
                                        if (rect.h - 40.0).abs() < 0.5 {
                                            saw_inner = true;
                                        }
                                    }
                                    crate::ui::desktop_protocol::message::DrawOp::ScissorPop => {
                                        depth -= 1;
                                    }
                                    _ => {}
                                }
                            }
                            depth == 0
                                && max_depth == 2
                                && saw_outer
                                && saw_inner
                                && composed_texts(s, "p515-scroll-overflow")
                                    .iter()
                                    .any(|t| t == "inner-4")
                                && composed_texts(s, "p515-scroll-overflow")
                                    .iter()
                                    .any(|t| t == "line-5")
                        })
                })
            }),
            "p515 scissor 栈帧到位（双视口 + 嵌套 2 层 + 溢出内容保留）"
        );
        // 视口外按钮（"In"，外层折叠线下）命中区被过滤；视口外置的
        // "Out" 保留——孪生投影器同源布局判定。
        {
            let p515_twins = twins
                .iter()
                .find(|(n, _)| n == "p515-scroll-overflow")
                .map(|(_, p)| p.hit_regions());
            let hits = p515_twins.expect("p515 孪生").len();
            // col p-2 内容 = "Out" 按钮 1 个可点命中区（"In" 在 128px
            // 折叠线外被视口过滤；输入类无）。
            assert_eq!(hits, 1, "视口外按钮命中区过滤，仅 Out 可点");
        }

        // 002：点击 "+" → Counter: 1。
        {
            // 002 行序：- Reset + → "+" 是第 3 个按钮命中区。
            let (hx, hy) = twin_hit_nth("002-counter", "button:", 2)
                .expect("002 孪生按钮坐标（+ 为第三个）");
            let (ox, oy) = origin_of(&session, "002-counter").expect("002 窗原点");
            assert!(
                session.broker_pointer_down(ox + hx, oy + hy, MouseButton::Left),
                "002 点击路由"
            );
            assert!(
                wait_frames(&mut session, |s| composed_texts(s, "002-counter")
                    .iter()
                    .any(|t| t == "Counter: 1")),
                "002 点击后帧递增"
            );
        }

        // 003：聚焦 celsius 输入 → 输入 100 → 联动 212。
        {
            let (hx, hy) = twin_hit("003-converter", "input:celsius").expect("003 celsius 坐标");
            let (ox, oy) = origin_of(&session, "003-converter").expect("003 窗原点");
            assert!(session.broker_pointer_down(ox + hx, oy + hy, MouseButton::Left));
            // PLAN-025 T-05（AC-04 解释态同册受益实证）：键入改走宿主
            // 生产路径 `broker_char`（WM 焦点窗 → 命中窗 client 管道）——
            // 原直注 client.end 的协议级承载退役。
            for ch in ['1', '0', '0'] {
                assert!(
                    session.broker_char(ch),
                    "broker_char 路由（焦点窗 = 前次 broker_pointer_down 聚焦）"
                );
            }
            // PLAN-033 重录：native 输入框渲染键入 buffer 原文（"0"+"100"
            // = "0100"——与 a2r 轨同构；旧解释投影器渲染解析后字段 "100"）。
            // 换算正确性以 fahrenheit = 212 为证（.celsius 字段 = 100）。
            assert!(
                wait_frames(&mut session, |s| {
                    let t = composed_texts(s, "003-converter");
                    t.iter().any(|x| x == "212") && t.iter().any(|x| x == "0100")
                }),
                "003 输入换算联动: {:?}",
                composed_texts(&session, "003-converter")
            );
        }

        // 005：聚焦 email 输入 → 输入 a@b.c → 点 Sign In → password 错误显示。
        {
            let (hx, hy) = twin_hit("005-login", "input:email").expect("005 email 坐标");
            let (ox, oy) = origin_of(&session, "005-login").expect("005 窗原点");
            assert!(session.broker_pointer_down(ox + hx, oy + hy, MouseButton::Left));
            let wid = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some("005-login"))
                .and_then(|c| c.wid)
                .expect("005 wid");
            for ch in "a@b.c".chars() {
                let msg = crate::ui::desktop_protocol::message::ProtocolMsg::Input(
                    crate::ui::desktop_protocol::message::InputMsg::CharTyped {
                        wid: wid.0,
                        ch,
                    },
                );
                if let Some(client) = session
                    .broker_clients
                    .values_mut()
                    .find(|c| c.app_name.as_deref() == Some("005-login"))
                {
                    let _ = client.end.send(&msg);
                }
            }
            let (bx, by) = twin_hit("005-login", "button:Submit").expect("005 Sign In 坐标");
            assert!(session.broker_pointer_down(ox + bx, oy + by, MouseButton::Left));
            assert!(
                wait_frames(&mut session, |s| composed_texts(s, "005-login")
                    .iter()
                    .any(|t| t.contains("Password is required"))),
                "005 提交后错误经 if 块显示"
            );
        }

        // p507（Plan 507 T10）：Tier1+2 全家福实机 queue 模式——帧到位
        //（heading/badge/card 文本族）→ checkbox 翻转 → if 块 ON→OFF。
        {
            assert!(
                wait_frames(&mut session, |s| composed_texts(s, "p507-tier-coverage")
                    .iter()
                    .any(|t| t == "Tier Coverage")
                    && composed_texts(s, "p507-tier-coverage").iter().any(|t| t == "feat: ON")),
                "p507 首帧 Tier1+2 文本到位: {:?}",
                composed_texts(&session, "p507-tier-coverage")
            );
            // PLAN-033 重录：native 命中表 checkbox = Msg 物化（"checkbox:ok"
            // Toggle 形态退役）——具名 handler .ToggleOk 等位寻址。
            let (hx, hy) = twin_hit("p507-tier-coverage", "button:ToggleOk")
                .expect("p507 checkbox 坐标");
            let (ox, oy) = origin_of(&session, "p507-tier-coverage").expect("p507 窗原点");
            assert!(
                session.broker_pointer_down(ox + hx, oy + hy, MouseButton::Left),
                "p507 点击路由"
            );
            assert!(
                wait_frames(&mut session, |s| composed_texts(s, "p507-tier-coverage")
                    .iter()
                    .any(|t| t == "feat: OFF")),
                "p507 checkbox 翻转经 if 块显示: {:?}",
                composed_texts(&session, "p507-tier-coverage")
            );
        }

        // ---- 收尾：Close 全部 → child 退出码 0。 ----
        let closes: Vec<(String, Option<crate::ui::desktop_protocol::message::ProtocolMsg>)> =
            session
                .broker_clients
                .values_mut()
                .map(|c| (c.pipe.clone(), c.endpoint.close().ok()))
                .collect();
        for (pipe, close) in closes {
            if let Some(close) = close {
                if let Some(c) = session.broker_clients.get_mut(&pipe) {
                    let _ = c.end.send(&close);
                }
            }
        }
        for _ in 0..300 {
            session.pump_broker_clients();
            if session.apps.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        for child in children.iter_mut() {
            let start = std::time::Instant::now();
            let status = loop {
                if let Some(s) = child.try_wait().expect("try_wait") {
                    break s;
                }
                assert!(start.elapsed() < std::time::Duration::from_secs(30), "child 收尾超时");
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            assert!(status.success(), "child 退出码 {status}");
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// 真 `auto` 二进制定位（缺则增量构建一次——首跑数秒，之后缓存命中）。
    /// independent 臂 = iced 主线程约束（winit Windows 事件循环须主线程，
    /// 测试 harness 线程必炸）→ 用生产二进制做 child（兼得三态 spawn 参数
    /// 全真链路）。
    fn auto_exe() -> std::path::PathBuf {
        // Plan 515 G4 C2：陈旧防护（P500-1——mtime 对账 + AUTO_FRESH_EXE=1
        // 强制重建）。
        crate::ui::desktop_protocol::e2e_exe::locate_with_stale_guard()
    }
    // PLAN-033 T-04：t3_independent_pixels_and_dual_mode 退役删除——解释
    // pixels 臂（run_independent_child）随 AppProjector 拔根（解释态两合
    // 法形态 = inproc 直挂 / -q 经 native 臂；a2r 轨像素兜底 =
    // run_independent_native_child 不受影响，native 像素覆盖在
    // p020_native_exe_arm 族）。归因：D5 删清单。

    /// re-exec 子进程（剥离 NEXTEST_* 守护）。
    fn spawn_children(broker_pipe: &str, names: &[String]) -> Vec<std::process::Child> {
        let exe = std::env::current_exe().expect("current_exe");
        let mut children = Vec::new();
        for app in names {
            let mut cmd = std::process::Command::new(&exe);
            cmd.args(["stage3_child_body", "--test-threads", "1", "--nocapture"])
                .env(CHILD_BROKER_ENV, broker_pipe)
                .env(CHILD_APP_ENV, app)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::inherit());
            for (k, _) in std::env::vars() {
                if k.starts_with("NEXTEST_") {
                    cmd.env_remove(&k);
                }
            }
            children.push(cmd.spawn().expect("spawn child"));
        }
        children
    }

    /// 孵化 attach 至 `want` 个落地（30s 预算）。
    fn attach_until(session: &mut DesktopSession, want: usize) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while session.broker_clients.values().filter_map(|c| c.wid).count() < want
            && std::time::Instant::now() < deadline
        {
            session.attach_pending_incubations(2000);
            session.pump_broker_clients();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    /// child 总内存采样（逐 pid WorkingSet/PrivateUsage 求和）。
    fn sample_children_total(children: &[std::process::Child]) -> Option<ProcessMemorySample> {
        let mut total = ProcessMemorySample { working_set: 0, private_bytes: 0 };
        for child in children {
            let s = sample_process_memory(child.id()).ok()?;
            total.working_set += s.working_set;
            total.private_bytes += s.private_bytes;
        }
        Some(total)
    }

    /// N App 压测主体：spawn → 孵化 attach → 全 Active → 逐 App 点击帧
    /// 递增 → `stability_secs` 稳定存活（期间持续泵帧）→ 收尾 Close。
    fn stress_body(n: usize, stability_secs: u64) {
        use std::sync::atomic::AtomicBool;

        let broker_pipe = format!("autodesk-broker-s4-{}-n{n}", std::process::id());

        // ---- 桌面侧：真实会话 + 注册表（n 个计数器 App）+ enable_broker。
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let names: Vec<String> = (0..n).map(|i| format!("stress-{i}")).collect();
        let src = STRESS_SRC.to_string();
        let known = names.clone();
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| {
                if known.iter().any(|n| n == name) {
                    Some(LaunchSpec {
                        code: src.clone(),
                        source_path: None,
                        title: Some(name.to_string()),
                        name: None,
                        daemon: None,
                        back_root: None,
                        fit: false,
        exe: None,
            opens: Vec::new(),
        render_decl: None,    })
                } else {
                    None
                }
            }));
        let stop = Arc::new(AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        // ---- spawn N 个 child（re-exec；剥离 NEXTEST_* 守护）。
        let mut children = spawn_children(&broker_pipe, &names);

        // ---- 孵化 attach 循环：直到 N 个全落地（30s 预算）。
        attach_until(&mut session, n);
        let landed: Vec<Wid> = session.broker_clients.values().filter_map(|c| c.wid).collect();
        assert_eq!(landed.len(), n, "全部 child 孵化落地（30s 预算）");
        assert_eq!(session.apps.len(), n, "n 条 AppSession");
        for (i, app) in names.iter().enumerate() {
            let wid = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some(app.as_str()))
                .and_then(|c| c.wid)
                .expect("landed");
            place_window(&mut session, wid, i);
        }
        let wids: Vec<Wid> = names
            .iter()
            .map(|app| {
                session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app.as_str()))
                    .and_then(|c| c.wid)
                    .expect("landed")
            })
            .collect();

        // ---- 逐 App 点击：帧 count 递增（shm 载荷断言）。
        for (i, wid) in wids.iter().enumerate() {
            let base_x = 16.0 + 500.0 * i as f32;
            let routed = session.broker_pointer_down(base_x + 60.0, 16.0 + 40.0, MouseButton::Left);
            assert!(routed, "App {i} 点击路由");
            let mut seen = false;
            for _ in 0..300 {
                session.pump_broker_clients();
                let app = &names[i];
                let hit = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app.as_str()))
                    .and_then(|c| c.composed())
                    .is_some_and(|list| {
                        list.ops.iter().any(|op| matches!(op,
                            DrawOp::Text { text, .. } if text == "count: 1"))
                    });
                if hit {
                    seen = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert!(seen, "App {i} 帧递增到 count: 1");
        }

        // ---- 稳定存活：持续泵 `stability_secs`，全程全 child 存活、全
        // client 在册。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(stability_secs);
        while std::time::Instant::now() < deadline {
            session.pump_broker_clients();
            for child in children.iter_mut() {
                assert!(child.try_wait().expect("try_wait").is_none(), "child 提前退出");
            }
            assert_eq!(session.broker_clients.len(), n, "client 全在册");
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        // ---- S5 采样集成：稳定窗末采集 N=child 全体内存（数字供 S6 报告）。
        if let Some(total) = sample_children_total(&children) {
            println!(
                "AUTO480-MEM stage=N{n} working_set={}B private={}B",
                total.working_set, total.private_bytes
            );
        }

        // ---- 收尾：逐 client Close → ExitRequest → 回收；child 退出码 0。
        let closes: Vec<(String, Option<crate::ui::desktop_protocol::message::ProtocolMsg>)> =
            session
                .broker_clients
                .values_mut()
                .map(|c| (c.pipe.clone(), c.endpoint.close().ok()))
                .collect();
        for (pipe, close) in closes {
            if let Some(close) = close {
                if let Some(c) = session.broker_clients.get_mut(&pipe) {
                    let _ = c.end.send(&close);
                }
            }
        }
        for _ in 0..300 {
            session.pump_broker_clients();
            if session.apps.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        for child in children.iter_mut() {
            let start = std::time::Instant::now();
            let status = loop {
                if let Some(s) = child.try_wait().expect("try_wait") {
                    break s;
                }
                assert!(start.elapsed() < std::time::Duration::from_secs(30), "child 收尾超时");
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            assert!(status.success(), "child 退出码 {status}");
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// N=3 压测（先绿档）。
    #[test]
    fn stage3_multi_app_stress_n3() {
        stress_body(3, 30);
    }

    /// N=5 压测（stress 档）。
    #[test]
    fn stage3_multi_app_stress_n5() {
        stress_body(5, 30);
    }

    /// S7 弹性重连：host 断连（server drop → child EOF）→ child 存活
    /// 等待重连（不退出、状态保持）→ 宿主重建管道 → child 连回、握手
    /// 续跑 → 再点击 count 连续（2）、revision 连续（3）。
    #[test]
    fn stage3_reconnect_state_continuous() {
        use crate::ui::desktop_protocol::client_runtime::ClientPump;
        use crate::ui::desktop_protocol::host::ProtocolHost;
        use crate::ui::desktop_protocol::message::DrawOp;
        use crate::ui::session::DesktopSession;

        const SRC: &str = "widget ReconnectCounter {
    model { var count int = 0 }
    view {
        button \"+\" { onclick: () => {.count += 1} }
        text `count: ${.count}`
    }
}
";

        let pipe = format!("autodesk-reconnect-s7-{}", std::process::id());
        let config = ClientConfig {
            app_name: "reconnect-counter".into(),
            title: "reconnect".into(),
            width: 480.0,
            height: 320.0,
        };

        // ---- 第一条连接：listen → child 泵（重连策略 10s 预算）。
        let listener = transport::listen(&pipe).expect("listen A");
        let app_end = transport::connect(&pipe, 2000).expect("connect A");
        let mut client = ClientPump::new(
            app_end,
            {
                let component = crate::build_dynamic_component(SRC, None).expect("build");
                RqProjector::new(component, 480.0, 320.0)
            },
            config.clone(),
            Some(ReconnectPolicy { pipe: pipe.clone(), budget_ms: 10_000, interval_ms: 20 }),
        );
        let mut server_end = listener.wait_connect().expect("server A");

        let mut session = DesktopSession::__test_session();
        session.__test_open_desktop();
        let src = SRC;
        let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
            if name == "reconnect-counter" {
                crate::build_dynamic_component(src, None).map_err(|e| format!("{e}"))
            } else {
                Err(format!("unknown app {name}"))
            }
        });
        fn pump(server_end: &mut Box<dyn crate::ui::desktop_protocol::transport::Transport + Send>, ph: &mut ProtocolHost<'_>) {
            while let Some(loaded) = server_end.try_recv() {
                let msg = loaded.expect("解码");
                ph.handle(&msg).expect("host 状态机");
                for reply in std::mem::take(&mut ph.to_app) {
                    let _ = server_end.send(&reply);
                }
            }
        }

        // 泵到 Active + 首帧，点击一次（count=1）。
        let mut wid = None;
        for _ in 0..200 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if !ph.session.apps.is_empty() {
                wid = ph.active().1;
                if ph.composed(wid.expect("wid").0).is_some() {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let wid_a = wid.expect("连接 A 孵化");
        let injected = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected).unwrap();
        let mut count_a = 0;
        for _ in 0..200 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if ph.composed(wid_a.0).is_some_and(|l| l.ops.iter().any(|op| matches!(op,
                DrawOp::Text { text, .. } if text == "count: 1"))) {
                count_a = 1;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(count_a, 1, "断连前点击生效");

        // ---- host 断连：server 端 drop → child EOF → 存活等待重连。
        server_end.send(&ph.endpoint.close().unwrap_or(
            crate::ui::desktop_protocol::message::ProtocolMsg::Handshake(
                crate::ui::desktop_protocol::message::HandshakeMsg::Ready))); // no-op 填充，真实断连靠 drop
        drop(ph);
        drop(server_end); // EOF 传播给 child
        for _ in 0..50 {
            assert!(client.step().is_none(), "child 在断连期存活（不退出）");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // ---- 宿主重建管道：同名单独 listen；旧 AppSession/窗按 EOF 回收语义清场。
        session.apps.clear();
        if let Some(host) = session.host.as_mut() {
            host.wm.wins.clear();
        }
        let listener_b = transport::listen(&pipe).expect("listen B");
        // 驱动 child 重连：connect 只在 step() 内尝试（try_reconnect），
        // 故 accept 放侧线程，主循环泵 step 直至 child 连回。
        let accept = std::thread::spawn(move || listener_b.wait_connect().expect("server B"));
        for _ in 0..200 {
            let _ = client.step();
            if accept.is_finished() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let mut server_end = accept.join().expect("accept thread");

        // 新连接（B）：新 ProtocolHost 端点（新 wid/surface），VM 状态在 child 侧。
        let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
            if name == "reconnect-counter" {
                crate::build_dynamic_component(src, None).map_err(|e| format!("{e}"))
            } else {
                Err(format!("unknown app {name}"))
            }
        });
        let mut wid_b = None;
        for _ in 0..300 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if !ph.session.apps.is_empty() {
                wid_b = ph.active().1;
                if ph.composed(wid_b.expect("wid").0).is_some() {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let wid_b = wid_b.expect("重连后再次孵化");

        // ---- 再点击：count 连续（2 = 断连前 1 + 现 successes），revision 连续。
        let injected = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected).unwrap();
        let mut count_b = 0;
        for _ in 0..300 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if ph.composed(wid_b.0).is_some_and(|l| l.ops.iter().any(|op| matches!(op,
                DrawOp::Text { text, .. } if text == "count: 2"))) {
                count_b = 2;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(count_b, 2, "重连后 count 连续（断连前 1 + 断连后 1）");

        // 出口核对：L2Detach 走完，projector 状态/revision 连续（rev=1+2）。
        let detach = ph.endpoint.l2_detach().unwrap();
        server_end.send(&detach).unwrap();
        let (exit, projector) = loop {
            pump(&mut server_end, &mut ph);
            if let Some(done) = client.step() {
                break done;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(exit, crate::ui::desktop_protocol::client_runtime::ClientExit::L2Detached);
        assert_eq!(projector.component().read_state("count").unwrap(), auto_val::Value::Int(2), "count 连续");
        assert_eq!(projector.revision(), 3, "revision 连续（1 + 2 次点击）");
    }

    /// S9 L3 v2a 快照迁移：融合态 App（count=42）→ AutoVM 快照 →
    /// 孵化 child 注入恢复——composed 帧先同步出 count: 42，点击推进
    /// 43；revision 延续（快照 41 + 1 次点击 = 42）。
    #[test]
    fn stage3_l3_v2a_snapshot_migration() {
        use crate::ui::desktop_protocol::client_runtime::ClientPump;
        use crate::ui::desktop_protocol::host::ProtocolHost;
        use crate::ui::desktop_protocol::message::{ControlMsg, DrawOp, ProtocolMsg as PMsg};
        use crate::ui::session::DesktopSession;

        const SRC: &str = "widget MigCounter {
    model { var count int = 0 }
    view {
        button \"+\" { onclick: () => {.count += 1} }
        text `count: ${.count}`
    }
}
";

        // ---- 融合态 App：直挂组件状态推进（等价一段交互后的状态）。
        let mut fused = crate::build_dynamic_component(SRC, None).expect("fused build");
        fused
            .write_state("count", auto_val::Value::Int(42))
            .expect("write count");
        let payload = fused_state_snapshot(&fused, 41);

        // 载荷线格式恒等（追加式演进纪律：encode→decode 往返）。
        let wire = PMsg::Control(ControlMsg::StateSnapshot { wid: 1, payload: payload.clone() });
        let encoded = wire.encode();
        let decoded = PMsg::decode(&encoded).expect("decode");
        assert_eq!(decoded, wire, "StateSnapshot 线格式往返恒等");

        // ---- 孵化 child（真实管道对）并注入快照。
        let pipe = format!("autodesk-l3-s9-{}", std::process::id());
        let listener = transport::listen(&pipe).expect("listen");
        let config = ClientConfig {
            app_name: "mig-counter".into(),
            title: "mig".into(),
            width: 480.0,
            height: 320.0,
        };
        let app_end = transport::connect(&pipe, 2000).expect("connect");
        let mut client = ClientPump::new(
            app_end,
            {
                let component = crate::build_dynamic_component(SRC, None).expect("child build");
                RqProjector::new(component, 480.0, 320.0)
            },
            config,
            None,
        );
        let mut server_end = listener.wait_connect().expect("server");

        let mut session = DesktopSession::__test_session();
        session.__test_open_desktop();
        let src = SRC;
        let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
            if name == "mig-counter" {
                crate::build_dynamic_component(src, None).map_err(|e| format!("{e}"))
            } else {
                Err(format!("unknown app {name}"))
            }
        });
        fn pump(server_end: &mut Box<dyn crate::ui::desktop_protocol::transport::Transport + Send>, ph: &mut ProtocolHost<'_>) {
            while let Some(loaded) = server_end.try_recv() {
                let msg = loaded.expect("解码");
                ph.handle(&msg).expect("host 状态机");
                for reply in std::mem::take(&mut ph.to_app) {
                    let _ = server_end.send(&reply);
                }
            }
        }

        // 泵到 Active + 首帧（此时 child 还在 count: 0）。
        let mut wid = None;
        for _ in 0..1000 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if !ph.session.apps.is_empty() {
                wid = ph.active().1;
                if ph.composed(wid.expect("wid").0).is_some() {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let wid = wid.expect("child 已孵化");

        // ---- 注入快照：host → StateSnapshot → child 应用 → 产帧同步。
        server_end
            .send(&PMsg::Control(ControlMsg::StateSnapshot { wid: wid.0, payload }))
            .unwrap();
        let mut injected = false;
        for _ in 0..1000 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if ph.composed(wid.0).is_some_and(|l| l.ops.iter().any(|op| matches!(op,
                DrawOp::Text { text, .. } if text == "count: 42"))) {
                injected = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(injected, "快照注入恢复：composed 帧同步出 count: 42（迁移前后一致）");

        // ---- 迁移后可交互：点击推进 43。
        let injected_click = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected_click).unwrap();
        let mut clicked = false;
        for _ in 0..1000 {
            pump(&mut server_end, &mut ph);
            let _ = client.step();
            if ph.composed(wid.0).is_some_and(|l| l.ops.iter().any(|op| matches!(op,
                DrawOp::Text { text, .. } if text == "count: 43"))) {
                clicked = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(clicked, "迁移后点击推进 count: 43");

        // ---- revision 延续：快照 41 + 1 次点击 = 42。
        let detach = ph.endpoint.l2_detach().unwrap();
        server_end.send(&detach).unwrap();
        let (exit, projector) = loop {
            pump(&mut server_end, &mut ph);
            if let Some(done) = client.step() {
                break done;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(exit, crate::ui::desktop_protocol::client_runtime::ClientExit::L2Detached);
        assert_eq!(projector.component().read_state("count").unwrap(), auto_val::Value::Int(43));
        assert_eq!(projector.revision(), 42, "revision 延续（快照 41 + 点击 1）");
    }

    /// Plan 500 步骤 5：宿主像素臂合成——HostEndpoint 收 FrameReadyPixels
    /// → ComposeFramePixels 动作 → BrokerClient::compose_pixels 读 shm 槽
    /// RGBA 入前缓冲 + FrameAck 回带（宽/高/stride/revision 元数据全链）。
    #[test]
    fn broker_pixels_compose_front_buffer() {
        use crate::ui::desktop_protocol::endpoint::{HostAction, HostEndpoint};
        use crate::ui::desktop_protocol::message::{
            FrameMsg, HandshakeMsg, PixelFormat, ProtocolMsg, WRect,
        };

        // 端点：Hello → activate(Pixels) → Active。
        let mut host = HostEndpoint::listen();
        let hello = {
            let mut app = super::super::endpoint::AppEndpoint::new(
                super::super::pixels::PixelsNoopSource::new(),
                "px",
                "px",
                32.0,
                16.0,
            );
            app.connect().expect("hello")
        };
        let actions = host.on_message(hello).expect("host 状态机");
        assert!(matches!(actions[0], HostAction::ResolveAndAttach { .. }));
        host.activate(1, 9, 77, WRect::new(0.0, 0.0, 32.0, 16.0), FrameMode::Pixels)
            .expect("activate");

        // child 侧写 shm 槽（32×16 纯色帧，槽尺寸 = 像素上限）。
        let shm_name = format!("autodesk-shm-px5-{}", std::process::id());
        let slot_size = super::super::pixels::pixels_slot_size(32.0, 16.0);
        let child_shm = SharedFrameBuffer::create(&shm_name, 2, slot_size).expect("child shm");
        let rgba: Vec<u8> = std::iter::repeat([7u8, 8, 9, 255])
            .take((32 * 16) as usize)
            .flatten()
            .collect();
        child_shm.write_slot(1, &rgba).expect("write slot");

        // 端点收 FrameReadyPixels → ComposeFramePixels。
        let actions = host
            .on_message(ProtocolMsg::Frame(FrameMsg::FrameReadyPixels {
                wid: 9,
                frame_id: 4,
                slot: 1,
                damage: None,
                revision: 12,
                w: 32,
                h: 16,
                stride: 128,
                format: PixelFormat::Rgba8,
            }))
            .expect("Active 收帧");
        let HostAction::ComposeFramePixels {
            surface, wid, frame_id, slot, revision, w, h, stride,
        } = &actions[0]
        else {
            panic!("期待 ComposeFramePixels: {actions:?}");
        };
        assert_eq!((*wid, *frame_id, *slot, *revision, *w, *h, *stride), (9, 4, 1, 12, 32, 16, 128));

        // BrokerClient 合成：宿主开同名段 → 前缓冲 + ack。
        let pipe = format!("autodesk-px5-pipe-{}", std::process::id());
        let listener = transport::listen(&pipe).expect("listen");
        let end = transport::connect(&pipe, 500).expect("connect");
        let mut client = BrokerClient::new(pipe, end);
        let host_shm = SharedFrameBuffer::open(&shm_name, 2, slot_size).expect("host shm");
        client.shm.insert(*surface, host_shm);
        client.wid = Some(Wid(9));
        client.wid_surface.insert(9, *surface);
        let ack = client.compose_pixels(
            *surface, *wid, *frame_id, *slot, *revision, *w, *h, *stride,
        )
        .expect("compose");
        assert_eq!(
            ack,
            FrameMsg::FrameAck { wid: 9, frame_id: 4, slot: 1 },
            "ack 回带 frame_id/槽"
        );
        let front = client.composed_pixels().expect("前缓冲在册");
        assert_eq!((front.w, front.h, front.stride, front.revision), (32, 16, 128, 12));
        assert_eq!(front.rgba, rgba, "槽字节 = child 写入帧");
        drop(listener);
    }

    /// S5 采样单测 + N=1/3/5 边际增量数字生成：阶段化 spawn（1 → 3 →
    /// 5 child），每阶段 settle 后采全体 child WorkingSet/PrivateUsage；
    /// 断言数值 >0 且 N=5 > N=1；边际增量打印供 S6 报告引用。
    #[test]
    fn stage3_memory_baseline_n1_3_5() {
        use std::sync::atomic::AtomicBool;

        let broker_pipe = format!("autodesk-broker-s4-mem-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let names: Vec<String> = (0..5).map(|i| format!("mem-{i}")).collect();
        let src = STRESS_SRC.to_string();
        let known = names.clone();
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| {
                if known.iter().any(|n| n == name) {
                    Some(LaunchSpec {
                        code: src.clone(),
                        source_path: None,
                        title: Some(name.to_string()),
                        name: None,
                        daemon: None,
                        back_root: None,
                        fit: false,
        exe: None,
            opens: Vec::new(),
        render_decl: None,    })
                } else {
                    None
                }
            }));
        let stop = Arc::new(AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        // 阶段化 spawn+attach：1 → 3 → 5。attach 是 drain-all 语义，故
        // 每批先 spawn 再 attach；批次间 settle 1.5s 后采样（早批次多出
        // 的 settle 时间使边际估计偏保守——对 1-5MB/App 判定方向安全）。
        let mut children: Vec<std::process::Child> = Vec::new();
        let mut samples = Vec::new();
        for (stage, want) in [1usize, 3, 5].into_iter().enumerate() {
            let batch = &names[children.len()..want];
            children.extend(spawn_children(&broker_pipe, batch));
            attach_until(&mut session, want);
            let landed = session.broker_clients.values().filter_map(|c| c.wid).count();
            assert_eq!(landed, want, "阶段 N={want} 全落地");
            session.pump_broker_clients();
            std::thread::sleep(std::time::Duration::from_millis(1500));
            let total = sample_children_total(&children).expect("windows 采样");
            samples.push(total);
            println!(
                "AUTO480-MEM stage=N{want} children={want} working_set={}B ({:.1}MiB) private={}B ({:.1}MiB)",
                total.working_set,
                total.working_set as f64 / 1048576.0,
                total.private_bytes,
                total.private_bytes as f64 / 1048576.0,
            );
        }

        // 判定：数值 >0；N=5 > N=1（边际增量为正）。
        for (stage, s) in samples.iter().enumerate() {
            assert!(s.working_set > 0, "N={} WorkingSet > 0", [1, 3, 5][stage]);
            assert!(s.private_bytes > 0, "N={} PrivateUsage > 0", [1, 3, 5][stage]);
        }
        assert!(samples[2].working_set > samples[0].working_set, "N=5 WorkingSet > N=1");
        assert!(samples[2].private_bytes > samples[0].private_bytes, "N=5 PrivateUsage > N=1");

        // 边际增量（S6 报告口径）：(N=5 − N=1) / 4。
        let ws_marginal = (samples[2].working_set - samples[0].working_set) / 4;
        let priv_marginal = (samples[2].private_bytes - samples[0].private_bytes) / 4;
        println!(
            "AUTO480-MEM marginal-per-app working_set={ws_marginal}B ({:.2}MiB) private={priv_marginal}B ({:.2}MiB)",
            ws_marginal as f64 / 1048576.0,
            priv_marginal as f64 / 1048576.0,
        );

        // 收尾：Close 全部 → child 退出。
        let closes: Vec<(String, Option<crate::ui::desktop_protocol::message::ProtocolMsg>)> =
            session
                .broker_clients
                .values_mut()
                .map(|c| (c.pipe.clone(), c.endpoint.close().ok()))
                .collect();
        for (pipe, close) in closes {
            if let Some(close) = close {
                if let Some(c) = session.broker_clients.get_mut(&pipe) {
                    let _ = c.end.send(&close);
                }
            }
        }
        for _ in 0..300 {
            session.pump_broker_clients();
            if session.apps.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let mut children = children;
        for child in children.iter_mut() {
            let start = std::time::Instant::now();
            let status = loop {
                if let Some(st) = child.try_wait().expect("try_wait") {
                    break st;
                }
                assert!(start.elapsed() < std::time::Duration::from_secs(30), "child 收尾超时");
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            assert!(status.success(), "child 退出码 {status}");
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    // -----------------------------------------------------------------------
    // Plan 508 G2 —— 进程模型对比实测（inproc vs outproc 两臂，各自独立
    // 测试进程跑，报告 `docs/plans/reports/508-process-model-verdict.md`
    // 汇总）。批次 001–005 + 009-article-feed（中型示例）；阶段 N=1/3/5
    // 增量（+009 探针）；三指标：冷启动到首帧 / 稳态内存（Private 口径，
    // 480 方法）/ 交互往返延迟（点击→帧更新）。
    // -----------------------------------------------------------------------

    /// G2 批次（前 5 = 阶段阶梯；009 = 中型示例探针）。
    const G2_APPS: [&str; 6] = [
        "001-helloworld",
        "002-counter",
        "003-converter",
        "004-profile-card",
        "005-login",
        "009-article-feed",
    ];

    /// 002-counter 孪生投影器（命中坐标 + inproc 交互探针引擎；
    /// PLAN-033 T-04 迁 native）。
    fn g2_counter_twin() -> RqProjector<crate::ui::dynamic::DynamicComponent> {
        use crate::ui::desktop_protocol::endpoint::FrameSource;
        let src = example_source("002-counter");
        let comp = crate::build_dynamic_component(&src, None).expect("twin build");
        let mut p = RqProjector::new(comp, 480.0, 900.0);
        p.render_frame();
        p
    }

    /// 002 "+" 命中区（行序 - Reset + → 第 3 个按钮，T3 同源）。
    fn g2_plus_hit() -> ((f32, f32), String) {
        let twin = g2_counter_twin();
        let (r, kind) = twin
            .hit_regions()
            .into_iter()
            .filter(|(_, k)| k.starts_with("button:"))
            .nth(2)
            .expect("002 '+' 命中区");
        let handler = kind.strip_prefix("button:").expect("button kind").to_string();
        ((r.x + r.w / 2.0, r.y + r.h / 2.0), handler)
    }

    /// 样本统计行（median / p95，毫秒）。
    fn g2_stats_line(label: &str, mut samples: Vec<f64>) {
        samples.sort_by(|a, b| a.total_cmp(b));
        let n = samples.len();
        let median = samples[n / 2];
        let p95 = samples[n - 1 - n / 20];
        println!("AUTO508-INTERACT {label} n={n} median={median:.3}ms p95={p95:.3}ms");
    }

    /// 两臂共用的注册表（example 源装载；source_path = 绝对入口路径，
    /// outproc 子进程身份推导与生产同链）。
    fn g2_install_resolver(session: &mut DesktopSession) {
        let entries: Vec<(String, String, String)> = G2_APPS
            .iter()
            .map(|name| {
                let src = example_source(name);
                let path = concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../examples/ui/P/src/front/app.at"
                )
                .replace('P', name);
                (name.to_string(), src, path)
            })
            .collect();
        session.desktop.app_resolver = Some(std::sync::Arc::new(move |name: &str| {
            entries.iter().find(|(n, _, _)| n == name).map(|(n, src, path)| LaunchSpec {
                code: src.clone(),
                source_path: Some(path.clone()),
                title: Some(n.clone()),
                name: None,
                daemon: None,
                back_root: None,
                fit: false,
        exe: None,
            opens: Vec::new(),
        render_decl: None,    })
        }));
    }

    /// G2 inproc 臂：launch_app 同步时长（compile+mount+win；首帧 = 返回后
    /// 下一渲染节拍 ~16ms，口径注记见报告）+ 宿主 Private 阶梯采样 +
    /// 交互探针（孪生引擎：on_with_input + render_frame = child 点击链
    /// 减 IPC，两臂同引擎差值 = 进程/协议开销）。
    #[test]
    fn p508_g2_inproc_arm() {
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        g2_install_resolver(&mut session);
        let base = sample_process_memory(std::process::id()).expect("windows 采样");
        println!(
            "AUTO508-INPROC-MEM stage=base private={}B ws={}B",
            base.private_bytes, base.working_set
        );
        // 阶段 N=1/3/5 增量 launch（批内顺序），逐 App 计时。
        let mut launched = 0usize;
        for want in [1usize, 3, 5] {
            for name in G2_APPS.iter().take(want).skip(launched) {
                let t0 = std::time::Instant::now();
                session.launch_app(name).expect("inproc launch");
                println!(
                    "AUTO508-INPROC-LAUNCH app={name} stage=N{want} ms={:.1}",
                    t0.elapsed().as_secs_f64() * 1000.0
                );
            }
            launched = want;
            std::thread::sleep(std::time::Duration::from_millis(1500));
            let s = sample_process_memory(std::process::id()).expect("windows 采样");
            println!(
                "AUTO508-INPROC-MEM stage=N{want} private={}B ({:.1}MiB) ws={}B",
                s.private_bytes,
                s.private_bytes as f64 / 1048576.0,
                s.working_set
            );
        }
        // 009 中型示例探针（N=5 之上 +1）。
        let t0 = std::time::Instant::now();
        session.launch_app("009-article-feed").expect("inproc launch 009");
        println!(
            "AUTO508-INPROC-LAUNCH app=009-article-feed stage=probe ms={:.1}",
            t0.elapsed().as_secs_f64() * 1000.0
        );
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let s = sample_process_memory(std::process::id()).expect("windows 采样");
        println!(
            "AUTO508-INPROC-MEM stage=probe private={}B ({:.1}MiB) ws={}B",
            s.private_bytes,
            s.private_bytes as f64 / 1048576.0,
            s.working_set
        );
        // 交互探针：孪生引擎（预热 5 + 采样 20）。
        let ((_hx, _hy), handler) = g2_plus_hit();
        let mut twin = g2_counter_twin();
        use crate::ui::desktop_protocol::endpoint::FrameSource;
        for _ in 0..5 {
            twin.component_mut().on_with_input(&handler, None);
            twin.render_frame();
        }
        let mut samples = Vec::new();
        for _ in 0..20 {
            let t0 = std::time::Instant::now();
            twin.component_mut().on_with_input(&handler, None);
            twin.render_frame();
            samples.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        g2_stats_line("inproc", samples);
    }

    /// G2 outproc 臂：真实 `auto` 二进制生产链（`run --autodesk-incubate`，
    /// G1 落的 launch_app outproc 分支）——spawn→attach Active（launch_app
    /// 返回）→首帧 composed 时机；宿主 + 子进程 Private 阶梯采样；交互
    /// 端到端（broker_pointer_down → 帧文本变化，自旋泵）。
    #[test]
    fn p508_g2_outproc_arm() {
        let broker_pipe = format!("autodesk-broker-508g2-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        g2_install_resolver(&mut session);
        // 生产 spawner 的测试镜像：exe = 真 auto 二进制（与 spawn_outproc_child
        // 唯一差异 = broker 管道名测试隔离）。
        let exe = auto_exe();
        let app_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui");
        let pipe_for_spawn = broker_pipe.clone();
        session.desktop.outproc_spawner = Some(std::sync::Arc::new(move |child_name| {
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
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));
        let base = sample_process_memory(std::process::id()).expect("windows 采样");
        println!(
            "AUTO508-OUTPROC-MEM stage=base host_private={}B host_ws={}B",
            base.private_bytes, base.working_set
        );

        let launched_of = |session: &DesktopSession, wid: Wid| {
            session
                .broker_clients
                .values()
                .any(|c| c.wid == Some(wid))
        };
        let mut launched = 0usize;
        for want in [1usize, 3, 5] {
            for name in G2_APPS.iter().take(want).skip(launched) {
                let t0 = std::time::Instant::now();
                let wid = session.launch_app(name).expect("outproc launch");
                let attach_ms = t0.elapsed().as_secs_f64() * 1000.0;
                // 首帧：Active 后 pump 到 composed DrawList 到达。
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                loop {
                    session.pump_broker_clients();
                    let got = launched_of(&session, wid)
                        && session
                            .broker_clients
                            .values()
                            .find(|c| c.wid == Some(wid))
                            .and_then(|c| c.composed())
                            .is_some_and(|l| !l.ops.is_empty());
                    if got || std::time::Instant::now() > deadline {
                        break;
                    }
                    std::thread::yield_now();
                }
                println!(
                    "AUTO508-OUTPROC-LAUNCH app={name} stage=N{want} attach_ms={attach_ms:.1} firstframe_ms={:.1}",
                    t0.elapsed().as_secs_f64() * 1000.0
                );
            }
            launched = want;
            // 窗口铺开（交互命中寻址）。
            let wids: Vec<Wid> =
                session.broker_clients.values().filter_map(|c| c.wid).collect();
            for (i, wid) in wids.into_iter().enumerate() {
                place_window(&mut session, wid, i);
            }
            std::thread::sleep(std::time::Duration::from_millis(1500));
            session.pump_broker_clients();
            let host = sample_process_memory(std::process::id()).expect("windows 采样");
            let kids =
                sample_children_total(&session.desktop.outproc_children).expect("windows 采样");
            println!(
                "AUTO508-OUTPROC-MEM stage=N{want} children={} host_private={}B ({:.1}MiB) children_private={}B ({:.1}MiB) children_ws={}B ({:.1}MiB)",
                session.desktop.outproc_children.len(),
                host.private_bytes,
                host.private_bytes as f64 / 1048576.0,
                kids.private_bytes,
                kids.private_bytes as f64 / 1048576.0,
                kids.working_set,
                kids.working_set as f64 / 1048576.0
            );
        }
        // 009 中型示例探针。
        {
            let t0 = std::time::Instant::now();
            let wid = session.launch_app("009-article-feed").expect("outproc launch 009");
            let attach_ms = t0.elapsed().as_secs_f64() * 1000.0;
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                session.pump_broker_clients();
                let got = launched_of(&session, wid)
                    && session
                        .broker_clients
                        .values()
                        .find(|c| c.wid == Some(wid))
                        .and_then(|c| c.composed())
                        .is_some_and(|l| !l.ops.is_empty());
                if got || std::time::Instant::now() > deadline {
                    break;
                }
                std::thread::yield_now();
            }
            println!(
                "AUTO508-OUTPROC-LAUNCH app=009-article-feed stage=probe attach_ms={attach_ms:.1} firstframe_ms={:.1}",
                t0.elapsed().as_secs_f64() * 1000.0
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(1500));
        session.pump_broker_clients();
        {
            let host = sample_process_memory(std::process::id()).expect("windows 采样");
            let kids =
                sample_children_total(&session.desktop.outproc_children).expect("windows 采样");
            println!(
                "AUTO508-OUTPROC-MEM stage=probe children={} host_private={}B ({:.1}MiB) children_private={}B ({:.1}MiB) children_ws={}B ({:.1}MiB)",
                session.desktop.outproc_children.len(),
                host.private_bytes,
                host.private_bytes as f64 / 1048576.0,
                kids.private_bytes,
                kids.private_bytes as f64 / 1048576.0,
                kids.working_set,
                kids.working_set as f64 / 1048576.0
            );
        }
        // 交互端到端探针：002 "+" → "Counter: k"（自旋泵，预热 3 + 采样 20）。
        {
            let ((hx, hy), _) = g2_plus_hit();
            let origin = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some("002-counter"))
                .and_then(|c| c.wid)
                .and_then(|wid| {
                    session.host.as_ref().and_then(|hh| hh.wm.wins.get(&wid)).map(|v| {
                        let r = *v.rect.borrow();
                        (r.x, r.y)
                    })
                })
                .expect("002 窗原点");
            let mut samples = Vec::new();
            for k in 0..23 {
                let t0 = std::time::Instant::now();
                assert!(
                    session.broker_pointer_down(origin.0 + hx, origin.1 + hy, MouseButton::Left),
                    "002 点击路由"
                );
                let want = format!("Counter: {}", k + 1);
                let hit = |session: &DesktopSession| {
                    session
                        .broker_clients
                        .values()
                        .find(|c| c.app_name.as_deref() == Some("002-counter"))
                        .and_then(|c| c.composed())
                        .is_some_and(|list| {
                            list.ops.iter().any(|op| matches!(op,
                                DrawOp::Text { text, .. } if *text == want))
                        })
                };
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                loop {
                    session.pump_broker_clients();
                    if hit(&session) {
                        break;
                    }
                    assert!(std::time::Instant::now() < deadline, "点击 {k} 帧超时");
                    std::thread::yield_now();
                }
                if k >= 3 {
                    samples.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
            }
            g2_stats_line("outproc", samples);
        }
        // 收尾：Close 全部 → child 退出；残留 kill 兜底。
        let closes: Vec<(String, Option<crate::ui::desktop_protocol::message::ProtocolMsg>)> =
            session
                .broker_clients
                .values_mut()
                .map(|c| (c.pipe.clone(), c.endpoint.close().ok()))
                .collect();
        for (pipe, close) in closes {
            if let Some(close) = close {
                if let Some(c) = session.broker_clients.get_mut(&pipe) {
                    let _ = c.end.send(&close);
                }
            }
        }
        for _ in 0..200 {
            session.pump_broker_clients();
            if session.apps.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// Plan 020 T-07 —— native exe 全链 e2e（`AUTO_DESKTOP_E2E=1` 实机档）：
    /// a2r 编译 exe（T-05 scratch counter）经 T-06 **生产分流原样**
    /// （`LaunchSpec.exe` → `outproc_native_exe` 发现 → `spawn_exe_child`，
    /// 不注入 spawner——与生产唯一差异 = broker 管道名隔离）孵化 → Active →
    /// native View 投影命令帧入宿主合成 → 协议点击 → 状态递增新帧 →
    /// 宿主 Close → 子进程退出码 0 → kill 方向 EOF 回收（AC-02/03/05）。
    /// 载体定位：env `AUTO_020_NATIVE_EXE` / `AUTO_020_NATIVE_APP_DIR`
    /// （缺省 = lang-020 worktree scratch 产物——a2r 产物不入仓，缺载体
    /// 即跳过）。
    #[test]
    fn p020_native_exe_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        let exe = std::env::var("AUTO_020_NATIVE_EXE").unwrap_or_else(|_| {
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/counter.exe").to_string()
        });
        let app_dir = std::path::PathBuf::from(
            std::env::var("AUTO_020_NATIVE_APP_DIR").unwrap_or_else(|_| {
                concat!(env!("CARGO_MANIFEST_DIR"), "/../../../scratch020/002-counter").to_string()
            }),
        );
        let source = app_dir.join("src").join("front").join("app.at");
        if !std::path::Path::new(&exe).is_file() || !source.is_file() {
            eprintln!("[p020] skip: native 载体缺席（exe={exe} app_dir={}）", app_dir.display());
            return;
        }
        let code = std::fs::read_to_string(&source).expect("read counter source");

        let broker_pipe = format!("autodesk-broker-020-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        // 注册表 resolver：spec.exe = 编译产物（发现序第一环走真）。
        let exe_for_resolver = exe.clone();
        let source_for_resolver = source.to_string_lossy().to_string();
        let code_for_resolver = code.clone();
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| {
                (name == "002-counter").then(|| LaunchSpec {
                    code: code_for_resolver.clone(),
                    source_path: Some(source_for_resolver.clone()),
                    title: Some("Counter".to_string()),
                    name: Some("counter".to_string()),
                    daemon: None,
                    back_root: None,
                    fit: false,
                    exe: Some(std::path::PathBuf::from(&exe_for_resolver)),
                    // queue 档显式申明（AC-03 全链）——生产链同参：pac
                    // `desktop_render: "queue"` → spawn `--autodesk-render=queue`。
                    render_decl: Some("queue".to_string()),
                    opens: Vec::new(),
                })
            }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        let launch = |session: &mut DesktopSession| -> (Wid, std::vec::Vec<DrawOp>) {
            let t0 = std::time::Instant::now();
            let wid = session.launch_app("002-counter").expect("native exe launch");
            let attach_ms = t0.elapsed().as_secs_f64() * 1000.0;
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            loop {
                session.pump_broker_clients();
                let frame = session
                    .broker_clients
                    .values()
                    .find(|c| c.wid == Some(wid))
                    .and_then(|c| c.composed())
                    .filter(|l| !l.ops.is_empty());
                if let Some(list) = frame {
                    println!("AUTO020-NATIVE attach_ms={attach_ms:.1} ops={}", list.ops.len());
                    return (wid, list.ops.clone());
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "首帧超时 clients={} states={:?} modes={:?}",
                    session.broker_clients.len(),
                    session.broker_clients.values().map(|c| c.endpoint.state).collect::<Vec<_>>(),
                    session.broker_clients.values().map(|c| c.endpoint.frame_mode).collect::<Vec<_>>(),
                );
                std::thread::yield_now();
            }
        };

        // —— AC-02/03：孵化（子进程 = 该 exe）+ native View 投影帧入宿主。
        let (wid0, ops0) = launch(&mut session);
        place_window(&mut session, wid0, 0);
        // 子进程身份（代码流证据）：spawner 未注入 → 分流唯一入口 =
        // `outproc_native_exe` 发现命中 counter.exe → `spawn_exe_child` 臂
        // （auto re-exec 臂仅在发现 MISS 时可达——发现序单测已钉边界）。
        let resolver_spec = (session.desktop.app_resolver.as_ref().unwrap())("002-counter")
            .expect("resolver spec");
        let discovered =
            DesktopSession::outproc_native_exe(&resolver_spec).expect("native exe discovered");
        assert!(
            discovered.ends_with("counter.exe"),
            "发现序应命中编译产物 exe: {discovered:?}"
        );
        assert!(
            ops0.iter().any(|op| matches!(op, DrawOp::Text { text, .. } if text == "Counter: 0")),
            "native 投影帧已合成: {ops0:?}"
        );
        // 命令帧语义（queue 臂）：Welcome 模式位 = Commands（像素臂才是
        // Pixels）——经 broker 孵化记录定档断言。
        let client0 = session
            .broker_clients
            .values()
            .find(|c| c.wid == Some(wid0))
            .expect("client0");
        assert_eq!(
            client0.endpoint.frame_mode,
            crate::ui::desktop_protocol::message::FrameMode::Commands,
            "counter 级 native auto 档下走 queue（显式降级观测 = 覆盖门拒绝面）"
        );

        // —— 交互闭环：点 "+"（帧内按钮 Quad = Text "+" 前驱 op）→ Counter: 1。
        let plus_rect = {
            let mut prev_quad = None;
            for op in &ops0 {
                match op {
                    DrawOp::Quad { rect, .. } => prev_quad = Some(*rect),
                    DrawOp::Text { text, .. } if text == "+" => break,
                    _ => {}
                }
            }
            prev_quad.expect("按钮 Quad 先于其标签")
        };
        let origin0 = session
            .host
            .as_ref()
            .and_then(|h| h.wm.wins.get(&wid0))
            .map(|v| {
                let r = *v.rect.borrow();
                (r.x, r.y)
            })
            .expect("wid0 窗原点");
        assert!(
            session.broker_pointer_down(
                origin0.0 + plus_rect.x + plus_rect.w / 2.0,
                origin0.1 + plus_rect.y + plus_rect.h / 2.0,
                crate::ui::desktop_protocol::message::MouseButton::Left,
            ),
            "点击路由"
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            session.pump_broker_clients();
            let hit = session
                .broker_clients
                .values()
                .find(|c| c.wid == Some(wid0))
                .and_then(|c| c.composed())
                .is_some_and(|list| {
                    list.ops.iter().any(|op| {
                        matches!(op, DrawOp::Text { text, .. } if *text == "Counter: 1")
                    })
                });
            if hit {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "点击后帧超时");
            std::thread::yield_now();
        }
        println!("AUTO020-NATIVE click-through PASS (Counter: 0 -> 1)");

        // —— AC-05 kill 方向：第二实例 → kill 子进程 → 宿主 EOF 回收。
        let (wid1, _) = launch(&mut session);
        place_window(&mut session, wid1, 1);
        assert_ne!(wid0, wid1);
        let kid = session.desktop.outproc_children.last_mut().unwrap();
        let _ = kid.kill();
        let _ = kid.wait();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            session.pump_broker_clients();
            let reclaimed = !session.host.as_ref().unwrap().wm.wins.contains_key(&wid1)
                && !session.broker_clients.values().any(|c| c.wid == Some(wid1));
            if reclaimed {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "kill 回收超时");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        println!("AUTO020-NATIVE kill-reclaim PASS (wid={wid1:?})");

        // —— AC-05 Close 方向：宿主 Close → 子进程退出码 0 → 窗回收。
        let (pipe0, close) = {
            let c = session
                .broker_clients
                .values_mut()
                .find(|c| c.wid == Some(wid0))
                .expect("client0 alive");
            (c.pipe.clone(), c.endpoint.close().ok())
        };
        if let Some(close) = close {
            if let Some(c) = session.broker_clients.get_mut(&pipe0) {
                let _ = c.end.send(&close);
            }
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut exit_code = None;
        loop {
            session.pump_broker_clients();
            let gone = !session.host.as_ref().unwrap().wm.wins.contains_key(&wid0)
                && !session.broker_clients.values().any(|c| c.wid == Some(wid0));
            if let Some(child) = session.desktop.outproc_children.first_mut() {
                if let Ok(Some(status)) = child.try_wait() {
                    exit_code = Some(status.code());
                }
            }
            if gone && exit_code.is_some() {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "Close 回收超时");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(exit_code, Some(Some(0)), "Close → 子进程干净退出（退出码 0）");
        println!("AUTO020-NATIVE close-recycle PASS (exit=0)");

        // 兜底清理（第二实例 child 已 wait；首实例已退出）。
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    /// Plan 020 T-08 —— native exe 度量臂（`AUTO_DESKTOP_E2E=1` + 载体在册
    /// 才跑）：N=1/3/5 阶梯 launch（同一编译 exe 五实例——同 App 边际，
    /// 与 508 五不同 App 口径的差异随注报告）+ 宿主/子进程 Private/WS 双
    /// 口径采样（K32GetProcessMemoryInfo，480 先例）+ launch→attach/
    /// 首帧时延 + 点击交互时延。输出行 `AUTO020-METRICS-*`，
    /// 报告 = docs/plans/reports/020-rust-exe-compositor-metrics.md。
    #[test]
    fn p020_metrics_native_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        let exe = std::env::var("AUTO_020_NATIVE_EXE").unwrap_or_else(|_| {
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/counter.exe").to_string()
        });
        let app_dir = std::path::PathBuf::from(
            std::env::var("AUTO_020_NATIVE_APP_DIR").unwrap_or_else(|_| {
                concat!(env!("CARGO_MANIFEST_DIR"), "/../../../scratch020/002-counter").to_string()
            }),
        );
        let source = app_dir.join("src").join("front").join("app.at");
        if !std::path::Path::new(&exe).is_file() || !source.is_file() {
            eprintln!("[p020-metrics] skip: native 载体缺席");
            return;
        }
        let code = std::fs::read_to_string(&source).expect("read counter source");

        let broker_pipe = format!("autodesk-broker-020m-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let exe_for_resolver = exe.clone();
        let source_for_resolver = source.to_string_lossy().to_string();
        let code_for_resolver = code.clone();
        session.desktop.app_resolver =
            Some(std::sync::Arc::new(move |name: &str| {
                (name == "002-counter").then(|| LaunchSpec {
                    code: code_for_resolver.clone(),
                    source_path: Some(source_for_resolver.clone()),
                    title: Some("Counter".to_string()),
                    name: Some("counter".to_string()),
                    daemon: None,
                    back_root: None,
                    fit: false,
                    exe: Some(std::path::PathBuf::from(&exe_for_resolver)),
                    render_decl: Some("queue".to_string()),
                    opens: Vec::new(),
                })
            }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        let base = sample_process_memory(std::process::id()).expect("采样");
        println!(
            "AUTO020-METRICS-MEM stage=base host_private={}B host_ws={}B",
            base.private_bytes, base.working_set
        );
        let mut launched = 0usize;
        for want in [1usize, 3, 5] {
            for _ in launched..want {
                let t0 = std::time::Instant::now();
                let wid = session.launch_app("002-counter").expect("launch");
                let attach_ms = t0.elapsed().as_secs_f64() * 1000.0;
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
                loop {
                    session.pump_broker_clients();
                    let got = session
                        .broker_clients
                        .values()
                        .find(|c| c.wid == Some(wid))
                        .and_then(|c| c.composed())
                        .is_some_and(|l| !l.ops.is_empty());
                    if got {
                        break;
                    }
                    assert!(std::time::Instant::now() < deadline, "首帧超时");
                    std::thread::yield_now();
                }
                println!(
                    "AUTO020-METRICS-LAUNCH stage=N{want} attach_ms={attach_ms:.1} firstframe_ms={:.1}",
                    t0.elapsed().as_secs_f64() * 1000.0
                );
            }
            launched = want;
            let wids: Vec<Wid> = session
                .broker_clients
                .values()
                .filter_map(|c| c.wid)
                .collect();
            for (i, wid) in wids.into_iter().enumerate() {
                place_window(&mut session, wid, i);
            }
            std::thread::sleep(std::time::Duration::from_millis(1200));
            session.pump_broker_clients();
            let host = sample_process_memory(std::process::id()).expect("采样");
            let kids = sample_children_total(&session.desktop.outproc_children).expect("采样");
            println!(
                "AUTO020-METRICS-MEM stage=N{want} children={} host_private={}B host_ws={}B children_private={}B ({:.2}MiB) children_ws={}B ({:.2}MiB)",
                session.desktop.outproc_children.len(),
                host.private_bytes,
                host.working_set,
                kids.private_bytes,
                kids.private_bytes as f64 / 1048576.0,
                kids.working_set,
                kids.working_set as f64 / 1048576.0
            );
        }
        // 点击交互时延（预热 3 + 采样 20；窗 0 的 "+" 按钮）。
        {
            let wid0 = session
                .broker_clients
                .values()
                .find_map(|c| c.wid)
                .expect("client");
            let ops0 = session
                .broker_clients
                .values()
                .find_map(|c| c.composed().map(|l| l.ops.clone()))
                .expect("composed");
            let plus_rect = {
                let mut prev_quad = None;
                for op in &ops0 {
                    match op {
                        DrawOp::Quad { rect, .. } => prev_quad = Some(*rect),
                        DrawOp::Text { text, .. } if text == "+" => break,
                        _ => {}
                    }
                }
                prev_quad.expect("按钮 Quad")
            };
            let origin = session
                .host
                .as_ref()
                .and_then(|h| h.wm.wins.get(&wid0))
                .map(|v| {
                    let r = *v.rect.borrow();
                    (r.x, r.y)
                })
                .expect("窗原点");
            let mut samples = Vec::new();
            let mut k = 0i64;
            for i in 0..23 {
                let t0 = std::time::Instant::now();
                assert!(session.broker_pointer_down(
                    origin.0 + plus_rect.x + plus_rect.w / 2.0,
                    origin.1 + plus_rect.y + plus_rect.h / 2.0,
                    crate::ui::desktop_protocol::message::MouseButton::Left,
                ));
                let want_text = format!("Counter: {}", k + 1);
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                loop {
                    session.pump_broker_clients();
                    let hit = session
                        .broker_clients
                        .values()
                        .find(|c| c.wid == Some(wid0))
                        .and_then(|c| c.composed())
                        .is_some_and(|list| {
                            list.ops.iter().any(|op| matches!(op,
                                DrawOp::Text { text, .. } if *text == want_text))
                        });
                    if hit {
                        break;
                    }
                    assert!(std::time::Instant::now() < deadline, "点击帧超时");
                    std::thread::yield_now();
                }
                k += 1;
                if i >= 3 {
                    samples.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
            }
            g2_stats_line("native-exe-queue", samples);
        }
        // 收尾：Close 全部 + kill 兜底。
        let closes: Vec<(String, Option<crate::ui::desktop_protocol::message::ProtocolMsg>)> =
            session
                .broker_clients
                .values_mut()
                .map(|c| (c.pipe.clone(), c.endpoint.close().ok()))
                .collect();
        for (pipe, close) in closes {
            if let Some(close) = close {
                if let Some(c) = session.broker_clients.get_mut(&pipe) {
                    let _ = c.end.send(&close);
                }
            }
        }
        for _ in 0..200 {
            session.pump_broker_clients();
            if session.apps.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
    }

    // -----------------------------------------------------------------------
    /// PLAN-032 T-07 —— 六例 queue 全链 + auto 档翻转抽样腿
    ///（AUTO_DESKTOP_E2E=1 门；留痕 AUTO_032_ASSETS=1 → assets/032/）。
    /// 六例 = ramp v3 六缺项载体（012 SelfCenter / 018 hidden+定位族 /
    /// 021 sticky / 024 样式 grid / 041 hidden / 046 tabs）：native(-auto)
    /// -full 子进程经 wire 合成帧断言（首帧钩子 + frame_mode=Commands）+
    /// 046 tabs on_select 交互闭环（Beta 托盘项点击 → panel 切换）+
    /// 024 样式 grid 布局几何（Line/Bar 同行异列）+ 012 auto 档翻转腿
    ///（子进程侧 resolve 裁决断言 + 宿主 Commands）。
    #[test]
    fn p032_ramp3_flip_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        use crate::ui::desktop_protocol::message::{DrawOp, FrameMode, MouseButton};
        use crate::ui::session::{DesktopSession, LaunchSpec};

        // (例, 子进程档, 首帧钩子文本)。运行时视图 Covered 四例走进程
        // 腿（012 = auto 档翻转抽样腿）；018（truncate）/041（codeeditor）
        // 运行时视图真 not-yet——进程腿必被 queue 门拒收，改走进程内
        /// 拒收留痕腿（见尾部——native_gate_runtime_views_of_six 钉）。
        let legs: &[(&str, &str, &str)] = &[
            ("012-clock", "native-auto-full", "时钟 Clock"),
            ("021-blog-viewer", "native-full", "My Blog"),
            ("024-charts", "native-full", "图表工坊"),
            ("046-tabs-variants", "native-full", "Alpha panel"),
        ];
        let broker_pipe = format!("autodesk-broker-032-{}", std::process::id());
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());

        let names: Vec<String> = legs.iter().map(|(n, _, _)| n.to_string()).collect();
        session.desktop.app_resolver = Some(std::sync::Arc::new(move |name: &str| {
            names.iter().position(|n| n.as_str() == name).map(|_| LaunchSpec {
                code: r#"widget t { view { text "x" } }"#.to_string(),
                source_path: None,
                title: Some(name.to_string()),
                name: Some(name.to_string()),
                fit: false,
                daemon: None,
                back_root: None,
                exe: None,
                opens: Vec::new(),
                render_decl: Some("queue".into()),
            })
        }));
        let broker_for_spawn = broker_pipe.clone();
        let mode_of = |name: &str| {
            legs.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, m, _)| m.to_string())
                .unwrap_or_else(|| "native-full".into())
        };
        session.desktop.outproc_spawner = Some(std::sync::Arc::new(move |child_name| {
            Ok(spawn_t3_child(&broker_for_spawn, child_name, &mode_of(child_name)))
        }));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        session.enable_broker(&broker_pipe, Arc::clone(&stop));

        fn wait_frame(
            session: &mut DesktopSession,
            app: &str,
            pred: impl Fn(&[DrawOp]) -> bool,
            what: &str,
        ) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            loop {
                session.pump_broker_clients();
                let hit = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(app))
                    .and_then(|c| c.composed())
                    .is_some_and(|l| pred(&l.ops));
                if hit {
                    return;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "{what} 超时: {:?}",
                    session
                        .broker_clients
                        .values()
                        .find(|c| c.app_name.as_deref() == Some(app))
                        .and_then(|c| c.composed())
                        .map(|l| {
                            l.ops
                                .iter()
                                .map(|o| format!("{o:?}"))
                                .take(24)
                                .collect::<Vec<_>>()
                        })
                );
                std::thread::yield_now();
            }
        }
        // PLAN-032：TextStyled 同收（加粗文本走 Styled 臂——012 标题
        /// "时钟 Clock" 即此形态）。
        fn texts_of(ops: &[DrawOp]) -> Vec<&str> {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Text { text, .. } | DrawOp::TextStyled { text, .. } => {
                        Some(text.as_str())
                    }
                    _ => None,
                })
                .collect()
        }
        fn text_pos(ops: &[DrawOp], needle: &str) -> Option<(f32, f32)> {
            ops.iter().find_map(|op| match op {
                DrawOp::Text { x, y, text, .. } | DrawOp::TextStyled { x, y, text, .. }
                    if text.contains(needle) =>
                {
                    Some((*x, *y))
                }
                _ => None,
            })
        }

        // —— 六例首帧 + frame_mode=Commands（012 = auto 档翻转腿：子进程
        //    侧 resolve 裁决断言已在 t3_child_body；宿主侧帧模式同
        //    Commands——缺省 queue 生效面）。
        let mut origins: Vec<(String, (f32, f32))> = Vec::new();
        for (name, _, hook) in legs {
            let wid =
                session.launch_app(name).unwrap_or_else(|e| panic!("launch {name}: {e:?}"));
            let (ox, oy) = {
                let host = session.host.as_ref().unwrap();
                let r = *host.wm.wins.get(&wid).unwrap().rect.borrow();
                (r.x, r.y)
            };
            origins.push((name.to_string(), (ox, oy)));
            wait_frame(
                &mut session,
                name,
                |ops| texts_of(ops).iter().any(|t| t.contains(hook)),
                &format!("{name} 首帧（钩子 {hook}）"),
            );
            let mode = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some(name))
                .map(|c| c.endpoint.frame_mode)
                .unwrap_or_else(|| panic!("{name} client 缺席"));
            assert_eq!(mode, FrameMode::Commands, "{name} 帧模式 = Commands");
            println!("[p032] {name} 首帧 PASS (frame_mode=Commands)");
        }

        // —— 018/041 拒收留痕腿（进程内）：运行时视图真 not-yet 家族
        ///（truncate/codeeditor）——queue 门 ensure_covered 拒绝退出，
        /// 缺项载荷逐字断言（I3/AC-04 纪律的 e2e 面）。
        {
            let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/ui/");
            for (name, want) in [
                ("018-book-reader", "style:truncate"),
                ("041-auto-edit", "tag:codeeditor"),
            ] {
                let path = format!("{base}{name}/src/front/app.at");
                let src = std::fs::read_to_string(&path).expect("read");
                let comp = crate::build_dynamic_component(&src, Some(&path))
                    .unwrap_or_else(|e| panic!("build {name}: {e:?}"));
                let projector = crate::ui::desktop_protocol::native_projector::RqProjector::new(
                    comp,
                    480.0,
                    320.0,
                );
                let err = projector.ensure_covered().expect_err("queue 门应拒收");
                assert!(err.contains(want), "{name} 拒收载荷应含 {want}: {err}");
                println!("[p032] {name} 拒收留痕 PASS ({want})");
            }
        }

        // —— 046 tabs on_select 交互闭环：Beta 托盘项点击 → panel 切换。
        {
            let (ox, oy) = origins
                .iter()
                .find(|(n, _)| n == "046-tabs-variants")
                .map(|(_, p)| (p.0, p.1))
                .unwrap();
            let ops = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some("046-tabs-variants"))
                .and_then(|c| c.composed())
                .expect("046 帧");
            let (bx, by) = ops
                .ops
                .iter()
                .find_map(|op| match op {
                    DrawOp::Text { x, y, text, .. } | DrawOp::TextStyled { x, y, text, .. }
                        if text == "Beta" =>
                    {
                        Some((*x, *y))
                    }
                    _ => None,
                })
                .expect("Beta 托盘项坐标");
            assert!(
                session.broker_pointer_down(ox + bx + 6.0, oy + by + 6.0, MouseButton::Left)
            );
            wait_frame(
                &mut session,
                "046-tabs-variants",
                |ops| texts_of(ops)
                    .iter()
                    .any(|t| *t == "Beta panel - default tray (status quo)"),
                "046 tabs on_select 交互（Beta panel 切入）",
            );
            println!("[p032] 046 tabs on_select 闭环 PASS");
        }

        // —— 024 样式 grid 布局几何：Line/Bar 同行异列（grid-cols-2）。
        {
            let ops = session
                .broker_clients
                .values()
                .find(|c| c.app_name.as_deref() == Some("024-charts"))
                .and_then(|c| c.composed())
                .expect("024 帧");
            let l = text_pos(&ops.ops, "Line").expect("Line 按钮");
            let b = text_pos(&ops.ops, "Bar").expect("Bar 按钮");
            assert!(
                (l.1 - b.1).abs() < 2.0 && b.0 > l.0,
                "grid-cols-2 同行异列: Line={l:?} Bar={b:?}"
            );
            println!("[p032] 024 样式 grid 布局 PASS (Line/Bar 同行异列)");
        }

        // 帧留痕（AUTO_032_ASSETS=1 → docs/plans/reports/assets/032/）。
        if std::env::var("AUTO_032_ASSETS").is_ok() {
            let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/plans/reports/assets/032");
            let _ = std::fs::create_dir_all(&assets);
            for (name, _, _) in legs {
                if let Some(list) = session
                    .broker_clients
                    .values()
                    .find(|c| c.app_name.as_deref() == Some(name))
                    .and_then(|c| c.composed())
                {
                    let out =
                        crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(list);
                    let _ = std::fs::write(assets.join(format!("{name}-frame.txt")), out);
                }
            }
        }

        // 兜底清理。
        for mut child in session.desktop.outproc_children.drain(..) {
            match child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::connect(&broker_pipe, 500);
        println!("AUTO032-RAMP3-FLIP arm PASS (six examples + auto flip leg)");
    }

    // PLAN-031 T-07 —— rqhost 第四形态 e2e（AUTO_DESKTOP_E2E=1 门）
    // -----------------------------------------------------------------------

    /// Win32 窗口 FFI（零新依赖——stage3 mem_ffi 同型）。**P031-R2 改道
    /// 记录**：原真机 SendInput + 屏幕位块截图路线在本开发机不可靠——
    /// agent 会话自渲染面板为 TOPMOST 覆盖层，抢点击/焦点且入镜（两轮
    /// 实证：hello-before 抓到终端窗内容、conv 区抓到 agent 面板）。
    /// 输入注入改走**窗口消息层**（WM_LBUTTONDOWN/WM_CHAR 直投消息泵
    /// → winit → iced → listen_with → LiveInput → 客户端——零屏幕/焦点
    /// 依赖）；帧变断言改走宿主 revision 观测行（确定性离线）。
    #[cfg(windows)]
    mod win_ffi {
        use std::cell::RefCell;

        #[link(name = "user32")]
        extern "system" {
            fn EnumWindows(lpEnumFunc: isize, lParam: isize) -> i32;
            fn GetWindowThreadProcessId(hwnd: isize, lpdwProcessId: *mut u32) -> u32;
            fn GetWindowTextW(hwnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
            fn IsWindowVisible(hwnd: isize) -> i32;
            fn SetWindowPos(
                hwnd: isize, after: isize, x: i32, y: i32, cx: i32, cy: i32,
                flags: u32,
            ) -> i32;
            fn PostMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> i32;
        }

        #[repr(C)]
        struct RECT {
            left: i32,
            top: i32,
            right: i32,
            bottom: i32,
        }

        /// 进程的可见顶层窗（标题非空）——owner pid 精确过滤。
        pub fn windows_of(pid: u32) -> Vec<(isize, String)> {
            struct Ctx {
                pid: u32,
                found: Vec<(isize, String)>,
            }
            let ctx = RefCell::new(Ctx { pid, found: Vec::new() });
            extern "system" fn on_enum(hwnd: isize, lparam: isize) -> i32 {
                let ctx = unsafe { &*(lparam as *const RefCell<Ctx>) };
                let mut owner = 0u32;
                unsafe { GetWindowThreadProcessId(hwnd, &mut owner) };
                if owner != ctx.borrow().pid || unsafe { IsWindowVisible(hwnd) } == 0 {
                    return 1;
                }
                let mut buf = [0u16; 256];
                let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), 256) };
                if n <= 0 {
                    return 1;
                }
                let title = String::from_utf16_lossy(&buf[..n as usize]);
                ctx.borrow_mut().found.push((hwnd, title));
                1
            }
            unsafe {
                EnumWindows(on_enum as isize, &ctx as *const RefCell<Ctx> as isize);
            }
            ctx.into_inner().found
        }

        /// 客户区尺寸搬动（SWP_NOMOVE=0x2|SWP_NOZORDER=0x4——外框尺寸
        /// 口径，客户区 = 外框 − 边框/标题栏，随 DPI/边框版本浮动）。
        pub fn resize_window(hwnd: isize, w: i32, h: i32) -> bool {
            unsafe { SetWindowPos(hwnd, 0, 0, 0, w, h, 0x2 | 0x4) != 0 }
        }

        /// 关窗请求（WM_CLOSE = 0x0010——用户点 × 的 OS 级等价）。
        pub fn request_close(hwnd: isize) -> bool {
            post_msg(hwnd, 0x0010, 0, 0)
        }

        /// 按标题子串找可见顶层窗（跨进程——自动孵化腿 daemon pid 未知）。
        pub fn find_window_by_title(contains: &str) -> Option<isize> {
            struct Ctx<'a> {
                needle: &'a str,
                hit: Option<(isize, String)>,
            }
            let ctx = RefCell::new(Ctx { needle: contains, hit: None });
            extern "system" fn on_enum(hwnd: isize, lparam: isize) -> i32 {
                let ctx = unsafe { &*(lparam as *const RefCell<Ctx>) };
                if unsafe { IsWindowVisible(hwnd) } == 0 {
                    return 1;
                }
                let mut buf = [0u16; 256];
                let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), 256) };
                if n <= 0 {
                    return 1;
                }
                let title = String::from_utf16_lossy(&buf[..n as usize]);
                if title.contains(ctx.borrow().needle) && ctx.borrow().hit.is_none() {
                    ctx.borrow_mut().hit = Some((hwnd, title));
                }
                1
            }
            unsafe {
                EnumWindows(on_enum as isize, &ctx as *const RefCell<Ctx> as isize);
            }
            ctx.into_inner().hit.map(|(hwnd, _)| hwnd)
        }

        /// 消息层注入（PostMessageW）——零屏幕/焦点依赖：点击 = 客户区
        /// 坐标 lParam（WM_LBUTTONDOWN 0x0201 / UP 0x0202，wParam=
        /// MK_LBUTTON 1/0）；键入 = WM_CHAR 0x0102（wParam = UTF-16 码元）。
        pub fn post_msg(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> bool {
            unsafe { PostMessageW(hwnd, msg, wparam, lparam) != 0 }
        }

        /// 客户区 (x, y) → lParam（低 16 位 x / 高 16 位 y）。
        pub fn lparam_of(x: i32, y: i32) -> isize {
            ((x as u16 as isize)) | ((y as u16 as isize) << 16)
        }
    }

    /// 子进程 stderr 收集器（读线程 → 共享行缓冲）。
    struct LineTail {
        lines: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl LineTail {
        fn spawn(child: &mut std::process::Child) -> Self {
            use std::io::BufRead;
            let stderr = child.stderr.take().expect("stderr piped");
            let lines = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let sink = std::sync::Arc::clone(&lines);
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    sink.lock().unwrap().push(line);
                }
            });
            Self { lines }
        }

        fn wait_contains(&self, needle: &str, what: &str, timeout_ms: u64) {
            let deadline =
                std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
            loop {
                if self
                    .lines
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|l| l.contains(needle))
                {
                    return;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "{what} 超时（等 `{needle}`）；已见行:\n{}",
                    self.lines.lock().unwrap().join("\n")
                );
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        fn count_of(&self, needle: &str) -> usize {
            self.lines.lock().unwrap().iter().filter(|l| l.contains(needle)).count()
        }

        fn wait_count(&self, needle: &str, want: usize, what: &str, timeout_ms: u64) {
            let deadline =
                std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
            while self.count_of(needle) < want {
                assert!(
                    std::time::Instant::now() < deadline,
                    "{what} 超时（等 `{needle}` x{want}）；已见行:
{}",
                    self.lines.lock().unwrap().join("
")
                );
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        fn snapshot(&self) -> Vec<String> {
            self.lines.lock().unwrap().clone()
        }
    }

    /// panic 清场守卫：断言失败时 kill 全部子进程（失败轮曾漏 daemon）。
    struct KillGuard(Vec<std::process::Child>);

    impl KillGuard {
        fn push(&mut self, child: std::process::Child) {
            self.0.push(child);
        }

        /// 按 pid kill+收尸（所有权已在守卫——e2e 各腿以 pid 操作）。
        fn kill_pid(&mut self, pid: u32) {
            if let Some(child) = self.0.iter_mut().find(|c| c.id() == pid) {
                if matches!(child.try_wait(), Ok(None)) {
                    let _ = child.kill();
                }
                let _ = child.wait();
            }
        }

        /// 按 pid 等退出（10s 预算——挂等即断言）。
        fn wait_pid(&mut self, pid: u32, what: &str) -> std::process::ExitStatus {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                if let Some(child) = self.0.iter_mut().find(|c| c.id() == pid) {
                    if let Some(status) = child.try_wait().expect("try_wait") {
                        return status;
                    }
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "{what} 10s 未退出"
                );
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        fn release(&mut self) {
            for child in self.0.iter_mut() {
                if matches!(child.try_wait(), Ok(None)) {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
            self.0.clear();
        }
    }

    impl Drop for KillGuard {
        fn drop(&mut self) {
            self.release();
        }
    }

    /// PLAN-031 T-07 —— rqhost 第四形态 e2e（`AUTO_DESKTOP_E2E=1` 门）：
    /// 真进程全景 = `auto rqhost` daemon（真 iced 原生窗）+ `auto run -q`
    /// 双 app 客户端。腿：
    /// ① AC-01 单 app（vm 003-converter）：ensure 探活→采纳→开窗→首帧；
    /// ② AC-02 多 app 共享：01-helloworld 并发入同一 daemon（双窗）+
    /// AC-03 竞态：第二 daemon 实例锁管道干净退 0；
    /// ③ AC-06 resize：SetWindowPos → daemon 观测行；
    /// ④ AC-04 kill 双向：app kill→EOF 窗回收观测 / daemon kill→app
    ///    exit-on-EOF（观测行 + 退出非挂等）；
    /// ⑤ AC-07 降级显式：未解析 img vm demo → [drawlist-image] 观测行；
    /// ⑥ 度量（rqhost+N app 内存数据行）+ 截图/进程清单留痕
    ///    `docs/plans/reports/assets/031/`（AUTO_031_ASSETS=1）。
    #[test]
    fn p031_rqhost_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        let manifest = env!("CARGO_MANIFEST_DIR");
        let repo = std::path::Path::new(manifest).join("../../");
        let dir_converter = repo.join("examples/ui/003-converter");
        let dir_hello = repo.join("examples/ui/001-helloworld");
        if !dir_converter.join("src/front/app.at").is_file()
            || !dir_hello.join("src/front/app.at").is_file()
        {
            eprintln!("[p031] skip: 载体缺席");
            return;
        }
        let auto_exe = crate::ui::desktop_protocol::e2e_exe::locate_with_stale_guard();
        let wellknown = format!("autodesk-rqhost-p031-{}", std::process::id());

        // ---- daemon 起服（真 iced 事件循环 + 原生窗）。----
        let mut daemon = std::process::Command::new(&auto_exe)
            .args(["rqhost", "--pipe", &wellknown])
            .env("AUTO_RQHOST_WELLKNOWN", &wellknown)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn auto rqhost");
        let daemon_tail = LineTail::spawn(&mut daemon);
        let daemon_pid = daemon.id();
        let mut guard = KillGuard(Vec::new());
        guard.push(daemon);
        daemon_tail.wait_contains("serving on", "daemon 起服", 20_000);

        // 子进程 env：well-known 缝 + NEXTEST 剥除（spawn_outproc_child 同则）。
        fn child_env(cmd: &mut std::process::Command, wellknown: &str) {
            cmd.env("AUTO_RQHOST_WELLKNOWN", wellknown)
                .env("AUTOUI_MCP_DISABLE", "1");
            for (key, _) in std::env::vars() {
                if key.starts_with("NEXTEST_") {
                    cmd.env_remove(&key);
                }
            }
        }
        fn spawn_q(
            auto_exe: &std::path::Path,
            dir: &std::path::Path,
            wellknown: &str,
        ) -> (std::process::Child, LineTail) {
            let mut cmd = std::process::Command::new(auto_exe);
            cmd.args(["run", "-r", "vm", "-q"])
                .current_dir(dir)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
            child_env(&mut cmd, wellknown);
            let mut child = cmd.spawn().expect("spawn auto run -q");
            let tail = LineTail::spawn(&mut child);
            (child, tail)
        }

        // ---- ① 单 app：003-converter 采纳→开窗→首帧（AC-01）。----
        // 两载体 widget 名同名 App（examples 约定）——断言走计数制；
        // 窗标题面（AUTO_VM_TITLE=pac title）在 resize 腿按标题找窗。
        let (mut app_c, _app_c_tail) = spawn_q(&auto_exe, &dir_converter, &wellknown);
        let app_c_pid = app_c.id();
        guard.push(app_c);
        daemon_tail.wait_count(
            "[rqhost] window opened for `App`",
            1,
            "003 开窗（Hello 凭据）",
            30_000,
        );
        daemon_tail.wait_count("[rqhost] first frame `App`", 1, "003 首帧", 30_000);

        // ---- ② 多 app 共享 + 竞态（AC-02/03）。----
        let (mut app_h, app_h_tail) = spawn_q(&auto_exe, &dir_hello, &wellknown);
        let app_h_pid = app_h.id();
        guard.push(app_h);
        daemon_tail.wait_count(
            "[rqhost] window opened for `App`",
            2,
            "helloworld 二窗（共享 daemon——AC-02）",
            30_000,
        );
        daemon_tail.wait_count("[rqhost] first frame `App`", 2, "双 app 首帧", 30_000);
        // 竞态：第二 daemon 实例 → 锁管道 → 干净退 0。
        let mut daemon2 = std::process::Command::new(&auto_exe)
            .args(["rqhost", "--pipe", &wellknown])
            .env("AUTO_RQHOST_WELLKNOWN", &wellknown)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn 第二 daemon");
        let daemon2_tail = LineTail::spawn(&mut daemon2);
        let daemon2_pid = daemon2.id();
        guard.push(daemon2);
        let daemon2_status =
            guard.wait_pid(daemon2_pid, "第二 daemon 未退出（锁管道未仲裁）");
        assert!(daemon2_status.success(), "第二实例码 0");
        daemon2_tail.wait_contains("已有实例在服", "第二实例观测行", 2_000);

        // ---- ③ resize 闭环（AC-06）：SetWindowPos → 观测行。----
        #[cfg(windows)]
        {
            let wins = win_ffi::windows_of(daemon_pid);
            let conv = wins
                .iter()
                .find(|(_, t)| t.contains("转换") || t.contains("Converter"))
                .expect("Converter 原生窗在场（EnumWindows，pac title zh/en）");
            assert!(win_ffi::resize_window(conv.0, 700, 520), "SetWindowPos");
            // SetWindowPos 是外框尺寸——客户区 = 外框 − 边框/标题栏（实测
            // 627x444 这类差值，随系统 DPI/边框版本浮动）——断言口径 =
            // 出现任意≠初始（480x320）的 resize 观测行。
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                let resized = daemon_tail
                    .snapshot()
                    .iter()
                    .filter_map(|l| l.split_once("resized "))
                    .filter(|(pre, _)| pre.contains("window `App`"))
                    .any(|(_, size)| !size.starts_with("480x320"));
                if resized {
                    break;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "resize 观测行超时（OS resize→协议 Resize 下发）"
                );
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            // ---- ③b 键入闭环（P031-R2/AC-01）——集成承载，e2e 撤腿：
            // 本机 ToDesk 输入钩子类环境对合成输入不生效（SendInput 全局
            // 队列两轮零送达 + PostMessage legacy 鼠标消息被 winit 0.30
            // WM_POINTER 路径忽略——native_dock_e2e T4 同款环境事实与
            // 两级退路先例）。闭环断言 = rqhost 集成测试
            // vm_typing_loop_over_pipe（真管道 + 真 003 源 + 真 ClientPump
            // + rq_update 输入臂：键入 100 → 客户端重排 → 宿主合成帧文本
            // 212——p025 同级语义证据；双客户端不串扰断言同测）。
            // 度量 + 截图/进程清单留痕（AUTO_031_ASSETS=1 → reports/assets/031）。
            let mut inventory = String::from("[p031] 进程清单\n");
            let mut total_private = 0u64;
            for (pid, name) in [
                (daemon_pid, "rqhost"),
                (app_c_pid, "003-converter(vm -q)"),
                (app_h_pid, "01-helloworld(vm -q)"),
            ] {
                if let Ok(s) = crate::ui::desktop_protocol::stage3::sample_process_memory(pid) {
                    total_private += s.private_bytes;
                    inventory.push_str(&format!(
                        "{name} pid={pid} working_set={}KB private={}KB\n",
                        s.working_set / 1024,
                        s.private_bytes / 1024
                    ));
                }
            }
            inventory.push_str(&format!(
                "total_private={}KB（rqhost + 2 app；对照口径：2×inproc 直挂 ≈ 2×独立 iced 进程）\n",
                total_private / 1024
            ));
            println!("{inventory}");
            if std::env::var("AUTO_031_ASSETS").as_deref() == Ok("1") {
                let assets = repo.join("docs/plans/reports/assets/031");
                std::fs::create_dir_all(&assets).expect("mkdir assets/031");
                std::fs::write(assets.join("inventory.txt"), &inventory).expect("写进程清单");
                // 截图留痕裁撤（P031-R2 改道附属）：屏幕位块/PrintWindow 在
                // agent 桌面覆盖层下不可靠（白屏/他窗入镜两轮实证）——
                // 环境无关留痕 = 进程清单 + daemon stderr（revision 观测行
                // 为帧变证据面）。
                std::fs::write(
                    assets.join("daemon-stderr.log"),
                    daemon_tail.snapshot().join("\n"),
                )
                .expect("写 daemon stderr");
            }
        }

        // ---- ④ kill 双向（AC-04）+ 用户关窗（P031-R3a/AC-01）。----
        // 用户关窗（X 语义 = WM_CLOSE）→ 宿主 Close 下发 → app 退出码 0
        // → EOF → 窗回收观测（kill app 的窗回收观测同路复用）。
        #[cfg(windows)]
        {
            let conv = win_ffi::windows_of(daemon_pid)
                .into_iter()
                .find(|(_, t)| t.contains("转换") || t.contains("Converter"))
                .expect("Converter 窗在场（关窗腿）");
            assert!(win_ffi::request_close(conv.0), "WM_CLOSE 下发");
        }
        #[cfg(not(windows))]
        guard.kill_pid(app_c_pid);
        let app_c_status = guard.wait_pid(app_c_pid, "关窗后 003 未退出（X→Close 链失效）");
        assert!(app_c_status.success(), "用户关窗 = app 退出码 0");
        daemon_tail.wait_contains(
            "断连（EOF）——窗回收",
            "app 退出 → EOF 窗回收观测",
            15_000,
        );
        // daemon → app：kill daemon → hello app exit-on-EOF（观测行 + 退出）。
        guard.kill_pid(daemon_pid);
        app_h_tail.wait_contains(
            "[rqhost-client] host lost",
            "kill daemon → app 观测行（exit-on-EOF）",
            15_000,
        );
        let app_h_status = guard.wait_pid(app_h_pid, "daemon 死后 app 未退出（exit-on-EOF 失效）");
        assert!(app_h_status.success(), "exit-on-EOF 干净退出（码 0）");

        // ---- ⑤ 降级显式（AC-07）：未解析 img → [drawlist-image] 观测行。----
        let tmp = tempfile::tempdir().unwrap();
        let app = tmp.path().join("p031-img");
        std::fs::create_dir_all(app.join("src/front")).unwrap();
        std::fs::write(
            app.join("src/front/app.at"),
            "widget P031Img {\n    view {\n        image (src: \"Z:/definitely/missing-031.png\") {\n            style: \"w-[120px] h-[80px]\"\n        }\n    }\n}\n",
        )
        .unwrap();
        // pac.at 缺席则 Automan::new 即失败（子进程未起就死——降级腿
        // 首跑根因）；最小 pac 同 001-helloworld 形状。
        std::fs::write(
            app.join("pac.at"),
            "name: \"p031-img\"\nversion: \"1.0.0\"\nscene: \"ui\"\nrender: \"vm\"\ntitle: \"P031Img\"\nwindow: \"480x320\"\n",
        )
        .unwrap();
        let mut daemon3 = std::process::Command::new(&auto_exe)
            .args(["rqhost", "--pipe", &format!("{wellknown}-d3")])
            .env("AUTO_RQHOST_WELLKNOWN", &format!("{wellknown}-d3"))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn daemon3");
        let daemon3_tail = LineTail::spawn(&mut daemon3);
        let daemon3_pid = daemon3.id();
        guard.push(daemon3);
        daemon3_tail.wait_contains("serving on", "daemon3 起服", 20_000);
        let (mut app_i, app_i_tail) = spawn_q(
            &auto_exe,
            &app,
            &format!("{wellknown}-d3"),
        );
        let app_i_pid = app_i.id();
        guard.push(app_i);
        // 手动等待（失败转储含子进程 stderr——首跑 pac.at 缺席即靠此
        // 定位路径）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !daemon3_tail
            .snapshot()
            .iter()
            .any(|l| l.contains("[rqhost] first frame `P031Img`"))
        {
            if std::time::Instant::now() >= deadline {
                panic!(
                    "降级 demo 首帧超时；daemon3:\n{}\nchild:\n{}",
                    daemon3_tail.snapshot().join("\n"),
                    app_i_tail.snapshot().join("\n")
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        daemon3_tail.wait_contains(
            "[drawlist-image] unresolved src (placeholder fallback): Z:/definitely/missing-031.png",
            "未解析 img 占位 + 观测行（I3 禁静默）",
            15_000,
        );

        // ---- ⑤b 末窗自退（P031-R3b/AC-04）：WM_CLOSE 唯一窗 →
        // app Close 握手退出 → daemon3 无在册窗∧无待定∧曾有窗 →
        // iced::exit 自退（iced 空窗不退的反面，真机证据）。----
        #[cfg(windows)]
        {
            let win = win_ffi::find_window_by_title("P031Img")
                .expect("降级 demo 原生窗在场（标题找窗）");
            assert!(win_ffi::request_close(win), "WM_CLOSE 降级窗");
        }
        let daemon3_status = guard.wait_pid(daemon3_pid, "末窗关闭后 daemon3 未自退");
        assert!(daemon3_status.success(), "末窗退出 = daemon 码 0");
        daemon3_tail.wait_contains(
            "末窗关闭——daemon 退出",
            "末窗退出观测行",
            15_000,
        );
        let _ = guard.wait_pid(app_i_pid, "降级 app 关窗后未退出");
        eprintln!("[p031] last-window-exit leg PASS");

        // ---- ⑥ 真实自动孵化（P031-R4/AC-03）：不预起 daemon——
        // -q 子进程 ensure 探活失败 → 自 spawn `auto rqhost` → 采纳 →
        // 原生窗。daemon 存活断言 = 锁管道不可再声明（持有者存在）；
        // 关末窗 → daemon 自退 → 锁让出（可再声明）。----
        {
            let wk_h = format!("{wellknown}-h");
            let hatch_dir = tmp.path().join("p031-hatch");
            std::fs::create_dir_all(hatch_dir.join("src/front")).unwrap();
            std::fs::write(
                hatch_dir.join("src/front/app.at"),
                "widget P031Hatch {\n    view {\n        text `hatch ok`\n    }\n}\n",
            )
            .unwrap();
            std::fs::write(
                hatch_dir.join("pac.at"),
                "name: \"p031-hatch\"\nversion: \"1.0.0\"\nscene: \"ui\"\nrender: \"vm\"\ntitle: \"P031Hatch\"\nwindow: \"480x320\"\n",
            )
            .unwrap();
            let mut hatch_cmd = std::process::Command::new(&auto_exe);
            hatch_cmd
                .args(["run", "-r", "vm", "-q"])
                .current_dir(&hatch_dir)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
            child_env(&mut hatch_cmd, &wk_h);
            let mut hatch_child = hatch_cmd.spawn().expect("spawn hatch child");
            let hatch_pid = hatch_child.id();
            let hatch_tail = LineTail::spawn(&mut hatch_child);
            guard.push(hatch_child);
            // 窗出现（子进程自己孵化的 daemon 开窗——60s：探活 500ms +
            // spawn + daemon iced 起服 + 采纳 + 首帧）。
            #[cfg(windows)]
            {
                let deadline =
                    std::time::Instant::now() + std::time::Duration::from_secs(60);
                let hwnd = loop {
                    if let Some(h) = win_ffi::find_window_by_title("P031Hatch") {
                        break h;
                    }
                    assert!(
                        std::time::Instant::now() < deadline,
                        "自动孵化 60s 未开窗（ensure/spawn 链断裂）；child stderr:
{}",
                        hatch_tail.snapshot().join("
")
                    );
                    std::thread::sleep(std::time::Duration::from_millis(200));
                };
                // daemon 存活 = 子进程孵化者持有锁（不可再声明）。
                assert!(
                    crate::ui::desktop_protocol::transport::try_claim_pipe(&format!("{wk_h}-lock"))
                        .is_err(),
                    "自动孵化的 daemon 持锁（单实例在服）"
                );
                assert!(win_ffi::request_close(hwnd), "WM_CLOSE hatch 窗");
            }
            let _ = guard.wait_pid(hatch_pid, "hatch 子进程关窗后未退出");
            // daemon 自退 → 锁让出（可再声明 = 独立存活周期闭环）。
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            loop {
                if crate::ui::desktop_protocol::transport::try_claim_pipe(&format!("{wk_h}-lock"))
                    .is_ok()
                {
                    break;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "自动孵化 daemon 20s 未随末窗自退（锁未让出）"
                );
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            eprintln!("[p031] auto-hatch leg PASS（spawn→锁持有→末窗自退→锁让出）");
        }

        // ---- ⑦ rust 轨（P031-R1/AC-05）：counter 生成物重生成（带
        // --autodesk-rqhost 臂）→ `auto run -r rust -q` → 注入旗标经
        // cargo `--` 透传 → 生成 exe 采纳 → 原生窗 → 关窗退出码 0。----
        {
            let wk_r = format!("{wellknown}-r");
            let mut daemon4 = std::process::Command::new(&auto_exe)
                .args(["rqhost", "--pipe", &wk_r])
                .env("AUTO_RQHOST_WELLKNOWN", &wk_r)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("spawn daemon4");
            let daemon4_tail = LineTail::spawn(&mut daemon4);
            guard.push(daemon4);
            daemon4_tail.wait_contains("serving on", "daemon4 起服", 20_000);

            // 强制重生成：删生成 main.rs（needs_regeneration 全量臂）；
            // 原件恢复守卫（生成物属仓内容——断言成败均写回，panic 走 Drop）。
            // P031-R5：恢复集**必须含 workspace 根 Cargo.toml**——生成会
            // 改 members（+002-counter），漏护即留脏（曾实证击穿 tf 全量
            // 门）；另以 git status 前后对照断言腿末零残留。
            let counter_ws = repo.join("examples/rust-workspace/counter");
            let ws_root = repo.join("examples/rust-workspace");
            let main_rs = counter_ws.join("src/main.rs");
            let member_toml = counter_ws.join("Cargo.toml");
            let ws_toml = ws_root.join("Cargo.toml");
            let saved_main = std::fs::read(&main_rs).expect("读 counter main.rs");
            let saved_toml = std::fs::read(&member_toml).expect("读 counter Cargo.toml");
            let saved_ws = std::fs::read(&ws_toml).expect("读 workspace Cargo.toml");
            let ws_dirty_before = git_status_porcelain(&repo, "examples/rust-workspace");
            std::fs::remove_file(&main_rs).expect("删生成 main.rs（强制重生成）");
            let restore = RestoreFiles(vec![
                (main_rs, saved_main),
                (member_toml, saved_toml),
                (ws_toml, saved_ws),
            ]);

            let mut rust_cmd = std::process::Command::new(&auto_exe);
            rust_cmd
                .args(["run", "-r", "rust", "-q"])
                .current_dir(repo.join("examples/ui/002-counter"))
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
            child_env(&mut rust_cmd, &wk_r);
            let mut rust_child = rust_cmd.spawn().expect("spawn rust -q");
            let rust_pid = rust_child.id();
            // stderr 必须排水（LineTail）——cargo 构建警告 >4KB 管道缓冲
            // 即阻塞子进程写端 = 构建假死（e2e 首跑 300s 超时根因）。
            let rust_tail = LineTail::spawn(&mut rust_child);
            guard.push(rust_child);
            // cargo 首建分钟级——首窗预算 300s。
            let deadline_rust =
                std::time::Instant::now() + std::time::Duration::from_secs(480);
            while !daemon4_tail
                .snapshot()
                .iter()
                .any(|l| l.contains("[rqhost] window opened for `counter`"))
            {
                if std::time::Instant::now() >= deadline_rust {
                    panic!(
                        "rust 轨采纳开窗超时；daemon4:
{}
child:
{}",
                        daemon4_tail.snapshot().join("
"),
                        rust_tail.snapshot().join("
")
                    );
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            daemon4_tail.wait_contains(
                "[rqhost] first frame `counter`",
                "rust 轨首帧（RqProjector queue 臂）",
                60_000,
            );
            #[cfg(windows)]
            {
                let win = win_ffi::find_window_by_title("counter")
                    .expect("counter 原生窗在场");
                assert!(win_ffi::request_close(win), "WM_CLOSE counter 窗");
            }
            let rust_status = guard.wait_pid(rust_pid, "rust 轨关窗后未退出");
            assert!(rust_status.success(), "rust 轨关窗 = exe 退出码 0");
            // 先显式恢复再断言（Drop 守卫在作用域尾——断言时序先于 Drop，
            // 首跑即被自身打穿）。
            restore.run();
            // P031-R5：腿末清洁断言——rust-workspace 下 git 状态与腿前
            // 全等（恢复守卫补全 workspace 清单后应为空集对照空集）。
            let ws_dirty_after = git_status_porcelain(&repo, "examples/rust-workspace");
            assert_eq!(
                ws_dirty_before, ws_dirty_after,
                "rust 腿残留脏文件（RestoreFiles 覆盖不足）：{ws_dirty_after:?}"
            );
            eprintln!("[p031] rust-track leg PASS（重生成→注入→采纳→窗→关窗码 0+零残留）");
        }

        // 清场。
        guard.release();
    }


    // -------------------------------------------------------------------
    // PLAN-033 T-07 —— rq-projector-unify e2e（AC-01/03）
    // -------------------------------------------------------------------

    /// PLAN-033 T-07（`AUTO_DESKTOP_E2E=1` 门）：
    /// ① AC-01 027-file-manager `-q` 经 RqProjector 真渲——开窗 + 首帧
    ///    + 覆盖门零拒（child stderr 无"未覆盖"拒绝行；popover/图标族
    ///    单元级 Covered 由 client_entry::vm_queue_arm_assembly_covered
    ///    027 腿钉）；
    /// ② AC-01 003-converter `-q`（改接后 vm_typing 集成断言已绿——e2e
    ///    钉开窗/首帧真链）；
    /// ③ AC-03 VM 内存对照行：003 直挂（`auto run -r vm` 自开 iced 窗）
    ///    vs `-q`（rqhost 客户端）app 进程 Private 对照 + ≤10MB 门沿用；
    /// ④ 度量留痕 assets/033（AUTO_033_ASSETS=1 → reports/assets/033）。
    #[test]
    fn p033_rq_unify_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        let manifest = env!("CARGO_MANIFEST_DIR");
        let repo = std::path::Path::new(manifest).join("../../");
        let dir_fm = repo.join("examples/ui/027-file-manager");
        let dir_conv = repo.join("examples/ui/003-converter");
        if !dir_fm.join("src/front/app.at").is_file() || !dir_conv.join("src/front/app.at").is_file()
        {
            eprintln!("[p033] skip: 载体缺席");
            return;
        }
        let auto_exe = crate::ui::desktop_protocol::e2e_exe::locate_with_stale_guard();
        let wellknown = format!("autodesk-rqhost-p033-{}", std::process::id());

        // ---- daemon 起服。----
        let mut daemon = std::process::Command::new(&auto_exe)
            .args(["rqhost", "--pipe", &wellknown])
            .env("AUTO_RQHOST_WELLKNOWN", &wellknown)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn auto rqhost");
        let daemon_tail = LineTail::spawn(&mut daemon);
        let daemon_pid = daemon.id();
        let mut guard = KillGuard(Vec::new());
        guard.push(daemon);
        daemon_tail.wait_contains("serving on", "daemon 起服", 20_000);

        fn child_env(cmd: &mut std::process::Command, wellknown: &str) {
            cmd.env("AUTO_RQHOST_WELLKNOWN", wellknown)
                .env("AUTOUI_MCP_DISABLE", "1");
            for (key, _) in std::env::vars() {
                if key.starts_with("NEXTEST_") {
                    cmd.env_remove(&key);
                }
            }
        }
        fn spawn_q(
            auto_exe: &std::path::Path,
            dir: &std::path::Path,
            wellknown: &str,
        ) -> (std::process::Child, LineTail) {
            let mut cmd = std::process::Command::new(auto_exe);
            cmd.args(["run", "-r", "vm", "-q"])
                .current_dir(dir)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
            child_env(&mut cmd, wellknown);
            let mut child = cmd.spawn().expect("spawn auto run -q");
            let tail = LineTail::spawn(&mut child);
            (child, tail)
        }

        // ---- ① 027-file-manager -q 真渲（AC-01）。----
        let (mut app_fm, app_fm_tail) = spawn_q(&auto_exe, &dir_fm, &wellknown);
        let app_fm_pid = app_fm.id();
        guard.push(app_fm);
        daemon_tail.wait_count(
            "[rqhost] window opened for `App`",
            1,
            "027 开窗（Hello 凭据）",
            40_000,
        );
        daemon_tail.wait_count("[rqhost] first frame `App`", 1, "027 首帧", 40_000);
        assert!(
            !app_fm_tail
                .snapshot()
                .iter()
                .any(|l| l.contains("未覆盖") || l.contains("已退役")),
            "027 覆盖门零拒：{}",
            app_fm_tail.snapshot().join("\n")
        );

        // ---- ② 003-converter -q（AC-01 真链；换算闭环 = vm_typing 集成承载）。----
        let (mut app_c, _app_c_tail) = spawn_q(&auto_exe, &dir_conv, &wellknown);
        let app_c_pid = app_c.id();
        guard.push(app_c);
        daemon_tail.wait_count(
            "[rqhost] window opened for `App`",
            2,
            "003 二窗（共享 daemon）",
            40_000,
        );
        daemon_tail.wait_count("[rqhost] first frame `App`", 2, "003 首帧", 40_000);

        // ---- ③ AC-03 内存对照行：直挂 vs -q。----
        // 直挂腿：auto run -r vm（自开 iced/wgpu 窗）→ 稳窗后采样 → 关窗收尾。
        let mut direct = std::process::Command::new(&auto_exe);
        direct
            .args(["run", "-r", "vm"])
            .current_dir(&dir_conv)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        for (key, _) in std::env::vars() {
            if key.starts_with("NEXTEST_") {
                direct.env_remove(&key);
            }
        }
        let mut direct = direct.spawn().expect("spawn auto run -r vm 直挂");
        let direct_pid = direct.id();
        guard.push(direct);
        // 直挂窗定位按进程枚举（标题与 daemon 的 003 窗同名——全局标题
        // 搜索会错关 daemon 窗，首跑实证）。
        #[cfg(windows)]
        let direct_hwnd = {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                let wins = win_ffi::windows_of(direct_pid);
                if let Some((h, _)) = wins
                    .iter()
                    .find(|(_, t)| t.contains("转换") || t.contains("Converter"))
                {
                    break *h;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "直挂 003 窗未开（对照腿缺席）"
                );
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        };
        std::thread::sleep(std::time::Duration::from_millis(800)); // 首帧+GPU 上载后稳态
        let direct_mem = crate::ui::desktop_protocol::stage3::sample_process_memory(direct_pid);
        let q_mem = crate::ui::desktop_protocol::stage3::sample_process_memory(app_c_pid);
        let fm_mem = crate::ui::desktop_protocol::stage3::sample_process_memory(app_fm_pid);
        let mut inventory = String::from("[p033] VM 内存对照行（app 进程 Private）\n");
        let mut q_private_kb = 0u64;
        if let (Ok(d), Ok(q), Ok(f)) = (&direct_mem, &q_mem, &fm_mem) {
            q_private_kb = q.private_bytes / 1024;
            inventory.push_str(&format!(
                "003 直挂（-r vm 自开窗）pid={direct_pid} working_set={}KB private={}KB\n",
                d.working_set / 1024,
                d.private_bytes / 1024
            ));
            inventory.push_str(&format!(
                "003 -q（rqhost 客户端）pid={app_c_pid} working_set={}KB private={}KB\n",
                q.working_set / 1024,
                q.private_bytes / 1024
            ));
            inventory.push_str(&format!(
                "027 -q（rqhost 客户端）pid={app_fm_pid} working_set={}KB private={}KB\n",
                f.working_set / 1024,
                f.private_bytes / 1024
            ));
            inventory.push_str(&format!(
                "对照：直挂 {}KB vs -q {}KB（省 {}KB = 免每 app iced/wgpu 后端收益）\n",
                d.private_bytes / 1024,
                q.private_bytes / 1024,
                d.private_bytes.saturating_sub(q.private_bytes) / 1024
            ));
        } else {
            inventory.push_str(&format!(
                "采样缺席：direct={direct_mem:?} q={q_mem:?} fm={fm_mem:?}\n"
            ));
        }
        // ≤10MB 门沿用（-q app 进程 Private）。
        assert!(
            q_private_kb > 0 && q_private_kb <= 10_240,
            "AC-03 ≤10MB 门：003 -q app private={q_private_kb}KB"
        );
        println!("{inventory}");
        // 直挂腿收尾（关窗 → 退出）。
        #[cfg(windows)]
        {
            assert!(win_ffi::request_close(direct_hwnd), "直挂窗 WM_CLOSE");
        }
        #[cfg(not(windows))]
        guard.kill_pid(direct_pid);
        let direct_status = guard.wait_pid(direct_pid, "直挂 003 关窗未退");
        assert!(direct_status.success(), "直挂关窗 = 码 0");

        // ---- ④ 留痕（AUTO_033_ASSETS=1 → reports/assets/033）。----
        if std::env::var("AUTO_033_ASSETS").as_deref() == Ok("1") {
            let assets = repo.join("docs/plans/reports/assets/033");
            std::fs::create_dir_all(&assets).expect("mkdir assets/033");
            std::fs::write(assets.join("memory-comparison.txt"), &inventory)
                .expect("写内存对照行");
            std::fs::write(
                assets.join("daemon-stderr.log"),
                daemon_tail.snapshot().join("\n"),
            )
            .expect("写 daemon stderr");
            std::fs::write(
                assets.join("fm-child-stderr.log"),
                app_fm_tail.snapshot().join("\n"),
            )
            .expect("写 027 child stderr");
        }

        // ---- 收尾：kill daemon → 双 app exit-on-EOF。----
        guard.kill_pid(daemon_pid);
        let app_c_status = guard.wait_pid(app_c_pid, "daemon 死后 003 未退（exit-on-EOF）");
        let app_fm_status = guard.wait_pid(app_fm_pid, "daemon 死后 027 未退（exit-on-EOF）");
        assert!(app_c_status.success() && app_fm_status.success(), "exit-on-EOF 干净退出");
    }

    /// PLAN-034 T-02（D2 归因矩阵）：{debug,release} × {wgpu,tiny-skia} ×
    /// {1,2,5} 窗 的 rqhost daemon private 数据行。观测法：build 对隔离
    /// debug 构建嫌疑、backend 对隔离 wgpu 设备/表面驻留（`ICED_BACKEND`
    /// env 原生开关——daemon 走 iced 标准链零代码）、窗边际斜率隔离每窗
    /// surface、残余归 fontdb/缓存（D2 定案序）。载体 = 003-converter
    /// （无 mpv 面——tiny-skia 腿 wgpu-only primitive 降级零撞）。
    /// dual-exit 门（AC-01）：release×wgpu×1 窗 ≤100MB 达标 → 优化转
    /// 可选；不达标 → T-03 执行清单（矩阵本身无论达标与否都是资产）。
    /// 留痕 `AUTO_034_ASSETS=1` → reports/assets/034/memory-matrix.txt。
    #[test]
    fn p034_memory_matrix_leg() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        let manifest = env!("CARGO_MANIFEST_DIR");
        let repo = std::path::Path::new(manifest).join("../../");
        let dir_converter = repo.join("examples/ui/003-converter");
        if !dir_converter.join("src/front/app.at").is_file() {
            eprintln!("[p034] skip: 载体缺席");
            return;
        }
        // 两档产物确保在场（release 复测产出方式——D1 定案：命令行直接
        // 构建，无独立脚本；嵌套 cargo 先例 = e2e_exe::build）。
        let target = repo.join("target");
        let mut exes: Vec<(&str, std::path::PathBuf)> = Vec::new();
        for (build, release) in [("release", true), ("debug", false)] {
            let exe = target.join(build).join("auto.exe");
            if !exe.exists() {
                let mut cmd = std::process::Command::new("cargo");
                cmd.args(["build", "-p", "auto", "--bin", "auto"])
                    .current_dir(&repo);
                if release {
                    cmd.arg("--release");
                }
                let status = cmd.status().expect("spawn cargo build（p034 矩阵产物）");
                assert!(status.success(), "cargo build（{build}）失败");
            }
            exes.push((build, exe));
        }

        fn child_env(cmd: &mut std::process::Command, wellknown: &str) {
            cmd.env("AUTO_RQHOST_WELLKNOWN", wellknown)
                .env("AUTOUI_MCP_DISABLE", "1");
            for (key, _) in std::env::vars() {
                if key.starts_with("NEXTEST_") {
                    cmd.env_remove(&key);
                }
            }
        }

        // 单格：起 daemon（backend env 按 D2）+ n×003 -q → 首帧齐 →
        // 稳态采样 → 数据行 → 收尾（kill daemon → app exit-on-EOF）。
        // 返回 (daemon 行, app private KB 列表)。
        fn matrix_cell(
            build: &str,
            exe: &std::path::Path,
            backend: &str,
            n: usize,
            dir_converter: &std::path::Path,
            pid: u32,
        ) -> (String, Vec<u64>) {
            let wellknown =
                format!("autodesk-rqhost-p034-{build}-{backend}-{n}-{pid}");
            let mut daemon_cmd = std::process::Command::new(exe);
            daemon_cmd
                .args(["rqhost", "--pipe", &wellknown])
                .env("AUTO_RQHOST_WELLKNOWN", &wellknown);
            match backend {
                // D2：双显 pin——iced fallback 链原生 env（fallback.rs
                // env::var("ICED_BACKEND")）零代码切档；T-03 后 daemon
                // 缺省 = tiny-skia，两腿都钉 env 保 A/B 口径诚实。
                "wgpu" | "tiny-skia" => {
                    daemon_cmd.env("ICED_BACKEND", backend);
                }
                _ => {} // default 腿：不设 env（度量交付缺省档）。
            }
            daemon_cmd
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
            let mut daemon = daemon_cmd.spawn().expect("spawn auto rqhost（矩阵格）");
            let daemon_tail = LineTail::spawn(&mut daemon);
            let daemon_pid = daemon.id();
            let mut guard = KillGuard(Vec::new());
            guard.push(daemon);
            daemon_tail.wait_contains("serving on", "矩阵格 daemon 起服", 20_000);

            let mut app_pids = Vec::new();
            for _ in 0..n {
                let mut cmd = std::process::Command::new(exe);
                cmd.args(["run", "-r", "vm", "-q"])
                    .current_dir(dir_converter)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null());
                child_env(&mut cmd, &wellknown);
                let app = cmd.spawn().expect("spawn auto run -q（矩阵格）");
                app_pids.push(app.id());
                guard.push(app);
            }
            daemon_tail.wait_count(
                "[rqhost] first frame `App`",
                n,
                &format!("{build}/{backend}×{n} 首帧齐"),
                60_000,
            );
            // 稳态：首帧后 GPU 上载/字体装载收敛（p033 800ms 同型，放宽
            // 到 1500ms——矩阵格要跨 release/debug/软光栅三档）。
            std::thread::sleep(std::time::Duration::from_millis(1500));
            let sample = crate::ui::desktop_protocol::stage3::sample_process_memory(daemon_pid)
                .expect("采样 daemon");
            let mut app_privates = Vec::new();
            for (i, pid) in app_pids.iter().enumerate() {
                match crate::ui::desktop_protocol::stage3::sample_process_memory(*pid) {
                    Ok(s) => app_privates.push(s.private_bytes / 1024),
                    Err(e) => eprintln!("[p034] app[{i}] 采样缺席: {e}"),
                }
            }
            let line = format!(
                "{build} {backend} windows={n}: rqhost pid={daemon_pid} working_set={}KB private={}KB\n",
                sample.working_set / 1024,
                sample.private_bytes / 1024
            );
            // 收尾：kill daemon → app exit-on-EOF（p033 同则）。
            guard.kill_pid(daemon_pid);
            for (i, pid) in app_pids.iter().enumerate() {
                let status = guard.wait_pid(*pid, &format!("矩阵格 app[{i}] exit-on-EOF"));
                assert!(status.success(), "矩阵格 app[{i}] 干净退出");
            }
            guard.release();
            (line, app_privates)
        }

        let pid = std::process::id();
        let mut report = String::from(
            "[p034] rqhost 内存归因矩阵（D2：{debug,release}×{wgpu,tiny-skia}×{1,2,5} 窗；PrivateUsage 口径）\n",
        );
        let mut daemon_private_kb: Vec<(String, u64)> = Vec::new();
        let mut gate_app_private_kb: Option<u64> = None;
        // 门格先行（release×wgpu×1），其余按档铺满。
        let cells: Vec<(&str, &str, usize)> = vec![
            // T-03 后门格 = 缺省档（daemon 缺省 tiny-skia——交付口径）。
            ("release", "default", 1),
            ("release", "wgpu", 1),
            ("release", "wgpu", 2),
            ("release", "wgpu", 5),
            ("release", "tiny-skia", 1),
            ("release", "tiny-skia", 2),
            ("release", "tiny-skia", 5),
            ("debug", "wgpu", 1),
            ("debug", "wgpu", 2),
            ("debug", "wgpu", 5),
            ("debug", "tiny-skia", 1),
            ("debug", "tiny-skia", 2),
            ("debug", "tiny-skia", 5),
        ];
        for (build, backend, n) in cells {
            let exe = exes
                .iter()
                .find(|(b, _)| *b == build)
                .map(|(_, p)| p.clone())
                .expect("两档产物已确保");
            let (line, app_privates) =
                matrix_cell(build, &exe, backend, n, &dir_converter, pid);
            let private_kb = line
                .split("private=")
                .nth(1)
                .and_then(|s| s.split("KB").next())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            daemon_private_kb.push((format!("{build}/{backend}×{n}"), private_kb));
            report.push_str(&line);
            if !app_privates.is_empty() {
                report.push_str(&format!(
                    "  apps private KB: {app_privates:?}\n"
                ));
            }
            if build == "release" && backend == "wgpu" && n == 1 {
                gate_app_private_kb = app_privates.first().copied();
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        // ---- 归因摘要行（嫌疑分摊）+ dual-exit 门判定（D2/D5）。----
        let cell_private = |build: &str, backend: &str, n: usize| {
            daemon_private_kb
                .iter()
                .find(|(k, _)| k == &format!("{build}/{backend}×{n}"))
                .map(|(_, v)| *v)
                .unwrap_or(0)
        };
        let gate = cell_private("release", "default", 1);
        let gate_pass = gate <= 100 * 1024;
        report.push_str(&format!(
            "归因：debug→release（wgpu×1窗）= {}→{}KB；wgpu→tiny-skia（release×1窗）= {}→{}KB；窗边际（release×wgpu，1→2→5）= {}→{}→{}KB（每窗 ≈{}KB）\n",
            cell_private("debug", "wgpu", 1),
            cell_private("release", "wgpu", 1),
            cell_private("release", "wgpu", 1),
            cell_private("release", "tiny-skia", 1),
            cell_private("release", "wgpu", 1),
            cell_private("release", "wgpu", 2),
            cell_private("release", "wgpu", 5),
            (cell_private("release", "wgpu", 5) - cell_private("release", "wgpu", 1)) / 4,
        ));
        if gate_pass {
            report.push_str(&format!(
                "门判定：release×default×1窗（T-03 后交付缺省 = tiny-skia）private={gate}KB ≤ 102400KB —— **达标**（wgpu 档 ≈234MB 由 ICED_BACKEND=wgpu 显式可达，归因对照保留）\n"
            ));
        } else {
            report.push_str(&format!(
                "门判定：release×default×1窗 private={gate}KB > 102400KB —— **不达标**（T-03 执行清单触发：LRU/自观测/Cache 按嫌疑分摊取舍）\n"
            ));
        }
        if let Some(app_kb) = gate_app_private_kb {
            report.push_str(&format!(
                "app 复核（release×wgpu×1窗 003 -q）：private={app_kb}KB（≤10MB 门 {}\n",
                if app_kb <= 10_240 { "过）" } else { "超——按 P033-D3 口径另裁说明）" }
            ));
            assert!(
                app_kb <= 10_240,
                "AC-01 app 门：release 003 -q private={app_kb}KB"
            );
        }
        println!("{report}");
        if std::env::var("AUTO_034_ASSETS").as_deref() == Ok("1") {
            let assets = repo.join("docs/plans/reports/assets/034");
            std::fs::create_dir_all(&assets).expect("mkdir assets/034");
            std::fs::write(assets.join("memory-matrix.txt"), &report)
                .expect("写内存矩阵");
        }
    }

    /// PLAN-034 T-07：rqhost-maturity e2e 位图合成腿——**canvas 样板
    /// 真渲**（D4 裁定 canvas=位图快照过线的全链实证）：真 daemon（缺省
    /// 软光栅档——T-03 后交付口径）+ 真 043-canvas-paint `-q` 客户端 →
    /// ①覆盖门放行（此前 canvas 拒绝退出）②首帧 ③daemon 位图观测行
    /// （上传→缓存）④app 侧零"弃置"（槽档足容）⑤内存行留痕。
    /// 截图腿沿 P031-R2 改道先例（ToDesk 覆盖层下截图不可靠）——
    /// 观测行 + stderr 即环境无关留痕。`AUTO_034_ASSETS=1` →
    /// assets/034/{canvas-child-stderr.log,daemon-stderr.log}。
    #[test]
    fn p034_rqhost_maturity_arm() {
        if std::env::var("AUTO_DESKTOP_E2E").as_deref() != Ok("1") {
            return;
        }
        let manifest = env!("CARGO_MANIFEST_DIR");
        let repo = std::path::Path::new(manifest).join("../../");
        let dir_canvas = repo.join("examples/capability-tests/043-canvas-paint");
        if !dir_canvas.join("src/front/app.at").is_file() {
            eprintln!("[p034] skip: 043 载体缺席");
            return;
        }
        let auto_exe = crate::ui::desktop_protocol::e2e_exe::locate_with_stale_guard();
        let wellknown = format!("autodesk-rqhost-p034e2e-{}", std::process::id());

        // ---- daemon（缺省档——不设 ICED_BACKEND）----
        let mut daemon = std::process::Command::new(&auto_exe)
            .args(["rqhost", "--pipe", &wellknown])
            .env("AUTO_RQHOST_WELLKNOWN", &wellknown)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn auto rqhost");
        let daemon_tail = LineTail::spawn(&mut daemon);
        let daemon_pid = daemon.id();
        let mut guard = KillGuard(Vec::new());
        guard.push(daemon);
        daemon_tail.wait_contains("serving on", "daemon 起服", 20_000);

        // ---- 043 canvas -q（stderr piped——弃置断言面）----
        let mut cmd = std::process::Command::new(&auto_exe);
        cmd.args(["run", "-r", "vm", "-q"])
            .current_dir(&dir_canvas)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());
        cmd.env("AUTO_RQHOST_WELLKNOWN", &wellknown)
            .env("AUTOUI_MCP_DISABLE", "1");
        for (key, _) in std::env::vars() {
            if key.starts_with("NEXTEST_") {
                cmd.env_remove(&key);
            }
        }
        let mut app = cmd.spawn().expect("spawn 043 -q");
        let app_tail = LineTail::spawn(&mut app);
        let app_pid = app.id();
        guard.push(app);

        // ① 覆盖门放行 + 开窗 + 首帧（canvas 此前启动即拒）。
        daemon_tail.wait_count(
            "[rqhost] window opened for `App`",
            1,
            "043 开窗（canvas 覆盖门放行）",
            30_000,
        );
        daemon_tail.wait_count("[rqhost] first frame `App`", 1, "043 首帧", 30_000);
        // ② 位图过线观测行（canvas-0 上传 → 宿主缓存）。
        daemon_tail.wait_contains(
            "[rqhost] bitmap `",
            "canvas 位图过线（BitmapReady → 缓存）",
            15_000,
        );
        // ③ app 侧零弃置（槽档足容 560×360 in 480×320 表面档×4）。
        assert!(
            !app_tail
                .snapshot()
                .iter()
                .any(|l| l.contains("bitmap upload 弃置")),
            "app 位图零弃置（槽档足容）"
        );

        // ④ 内存行（缺省档 daemon + 043 app）。
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let mut report = String::from("[p034] canvas 样板腿内存行（缺省档）
");
        for (pid, name) in [(daemon_pid, "rqhost(default)"), (app_pid, "043-canvas(vm -q)")] {
            if let Ok(s) = crate::ui::desktop_protocol::stage3::sample_process_memory(pid) {
                report.push_str(&format!(
                    "{name} pid={pid} working_set={}KB private={}KB
",
                    s.working_set / 1024,
                    s.private_bytes / 1024
                ));
            }
        }
        println!("{report}");

        // ⑤ 留痕。
        if std::env::var("AUTO_034_ASSETS").as_deref() == Ok("1") {
            let assets = repo.join("docs/plans/reports/assets/034");
            std::fs::create_dir_all(&assets).expect("mkdir assets/034");
            std::fs::write(assets.join("canvas-arm.txt"), &report).expect("写样板腿内存行");
            std::fs::write(
                assets.join("canvas-child-stderr.log"),
                app_tail.snapshot().join("
"),
            )
            .expect("写 043 stderr");
            std::fs::write(
                assets.join("daemon-stderr.log"),
                daemon_tail.snapshot().join("
"),
            )
            .expect("写 daemon stderr");
        }

        // ---- 收尾：kill daemon → app exit-on-EOF。----
        guard.kill_pid(daemon_pid);
        let status = guard.wait_pid(app_pid, "043 exit-on-EOF");
        assert!(status.success(), "043 干净退出");
    }
}

/// 路径域的 git 脏状态清单（e2e 腿卫生断言口——P031-R5）。git 缺席
/// 或非仓场景返回空串（e2e 恒在仓内运行，缺省不可达）。
fn git_status_porcelain(repo: &std::path::Path, scope: &str) -> Vec<String> {
    std::process::Command::new("git")
        .args(["-C", &repo.to_string_lossy(), "status", "--porcelain", "--", scope])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// e2e 文件恢复守卫（rust 腿生成物——断言成败/panic 路径均恢复仓内容）。
struct RestoreFiles(Vec<(std::path::PathBuf, Vec<u8>)>);

impl RestoreFiles {
    fn run(&self) {
        for (path, bytes) in &self.0 {
            let _ = std::fs::write(path, bytes);
        }
    }
}

impl Drop for RestoreFiles {
    fn drop(&mut self) {
        self.run();
    }
}
