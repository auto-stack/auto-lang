//! `video` 元素的 iced 渲染面（PLAN-617 **T-19**）：把 T-16/T-17/T-18 的引擎、
//! 帧上屏通道与受控契约接成**一个 iced widget**，从而把 `video` 从 `fallback`
//! 提升为可用（此前 VM 端渲染成空容器）。
//!
//! # 为什么是自定义 shader widget，而不是 iced 的图像通道
//!
//! 见 [Design 30 §4.5]：`Handle::from_rgba` 每帧 mint 新的 `Id::unique()`
//! （`Id` 构造器私有，无法复用），同 id 命中缓存即不再上传（只剩陈旧帧），
//! 且真实帧尺寸全部超过 `MAX_SYNC_SIZE`(2MB) 必走异步 worker、4K 更超过 atlas
//! 的 2048 上限——那条路正是 `renderer.rs:2889` 记录的闪烁机理。
//! 故这里走 `iced_widget::shader::Program`：管线自持纹理，
//! **纹理只建一次、每帧原地更新**（T-17 已把这条钉成类型不变量）。
//!
//! [Design 30 §4.5]: ../../../../docs/design/autoui/030-video-player.md
//!
//! # 三段职责的接法
//!
//! ```text
//! Program::draw   ──▶ Primitive{ id, 目标尺寸, 本帧下行值 }        （纯数据，Send+Sync）
//! Primitive::prepare ──▶ [thread_local 取引擎] apply 下行 / poll 上行
//!                        ──▶ channel.with_frame(mpv 渲染)          （每帧一帧）
//! Primitive::draw  ──▶ 用管线里那张持久纹理画满 bounds
//! ```
//!
//! # 非 Send 的 mpv 引擎放哪
//!
//! `Primitive` 与 `Pipeline` 都要求 `MaybeSend + MaybeSync`（native 上就是
//! `Send + Sync`），而 [`MpvEngine`] 含裸指针、**刻意**是 `!Send + !Sync`
//! （T-16 的保守选择：`mpv_wait_event` 只允许一个线程调用）。
//! 解法是**不把它塞进 Primitive/Pipeline**，而是放进 **thread-local 注册表**，
//! 由 widget id 索引——widget 与渲染都在 iced 主线程上，正是引擎被创建与使用的
//! 那个线程。这样 `Primitive` 只剩纯数据，Send/Sync 自然成立；
//! 万一真被换线程调用，表现是「查不到运行时 → 不画」，**降级而非 UB**。
//!
//! # 帧由谁驱动（AC-19 的剩余一段）
//!
//! iced 只在有重绘时调用 `prepare`，而视频需要持续出帧。本 widget **不自带定时器**
//! ——它复用应用既有的 tick 机制（`tick_interval_ms()` → `iced::time::every`，
//! 见 `run_app_with_title`）。即：`.at` 应用声明一个 tick（如 16ms），
//! 每次 tick 触发重绘 → `prepare` 推进一帧。**若应用不声明 tick，视频会停在
//! 首帧**——这一条写进了 `video` 的规范注记，不是隐藏行为。
//!
//! 上行事件（`ontimeupdate` 等）由 `prepare` 采集进 thread-local 队列，
//! 经 [`drain_events`] 取走；`.at` 侧如何把它们分发到应用的
//! `OnTime`/`OnDuration`… handler 属 AC-19 的接线（见计划 §9.18）。

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use iced::advanced::graphics::Viewport;
use iced::mouse;
use iced::widget::shader;
use iced::Rectangle;

use super::channel::{stride_for, VideoFrameChannel};
use super::contract::{MediaContract, VideoContractDown, VideoContractEvent};
use super::engine::{MpvEngine, MpvUnavailable};
use super::present::VideoPresenter;

/// widget 的稳定标识（每次 `Program::State::default()` 分配一个新值，
/// 由 iced 持久化，故同一 widget 在其生命周期内 id 稳定）。
static NEXT_WIDGET_ID: AtomicU64 = AtomicU64::new(1);

/// 分配一个新的 widget id。
pub fn next_widget_id() -> u64 {
    NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed)
}

// ───────────────────────── 线程本地运行时（非 Send 的 mpv 引擎的家） ─────────────────────────

/// 一个 video widget 的播放运行时。
struct VideoRuntime {
    engine: Option<MpvEngine>,
    contract: MediaContract,
    /// 不可用原因（缺库/初始化失败）。`Some` 时 widget 不绘制，
    /// 由上层渲染诚实降级面板（AC-10/AC-11）。
    unavailable: Option<String>,
    /// 上行事件队列：`prepare` 采集，[`drain_events`] 取走。
    pending: Vec<VideoContractEvent>,
}

impl VideoRuntime {
    fn new() -> Self {
        match MpvEngine::new() {
            Ok(engine) => Self {
                engine: Some(engine),
                contract: MediaContract::new(),
                unavailable: None,
                pending: Vec::new(),
            },
            Err(MpvUnavailable::NoLibrary) => Self {
                engine: None,
                contract: MediaContract::new(),
                unavailable: Some("本后端没有 libmpv（未安装运行库），暂无视频解码能力".into()),
                pending: Vec::new(),
            },
            Err(e) => Self {
                engine: None,
                contract: MediaContract::new(),
                unavailable: Some(format!("libmpv 不可用：{e}")),
                pending: Vec::new(),
            },
        }
    }
}

thread_local! {
    /// 按 widget id 索引的运行时。引擎**只在这里**——见模块文档「非 Send 的 mpv 引擎放哪」。
    static RUNTIMES: RefCell<HashMap<u64, VideoRuntime>> = RefCell::new(HashMap::new());
}

/// 取（或建）某 widget 的运行时。
fn with_runtime<R>(id: u64, f: impl FnOnce(&mut VideoRuntime) -> R) -> Option<R> {
    RUNTIMES.with(|m| {
        let mut m = m.borrow_mut();
        let rt = m.entry(id).or_insert_with(VideoRuntime::new);
        Some(f(rt))
    })
}

/// 取走某 widget 挂起的上行事件（由调用方分发给应用的 handler）。
///
/// 返回空 `Vec` 表示「目前没有新事件」——**这是常态**，不是错误。
pub fn drain_events(id: u64) -> Vec<VideoContractEvent> {
    with_runtime(id, |rt| std::mem::take(&mut rt.pending)).unwrap_or_default()
}

/// 某 widget 当前是否因缺库/初始化失败而不可用；`Some(原因)` 供上层显示降级文案。
pub fn unavailable_reason(id: u64) -> Option<String> {
    with_runtime(id, |rt| rt.unavailable.clone()).flatten()
}

/// 某 widget 是否已经拿到引擎（可播）。
pub fn is_native_available(id: u64) -> bool {
    with_runtime(id, |rt| rt.engine.is_some()).unwrap_or(false)
}

/// 释放某 widget 的运行时（Drop 会把 mpv 按
/// 「先 render context 后 core」的正确顺序拆掉，见 T-16）。
pub fn release_runtime(id: u64) {
    RUNTIMES.with(|m| {
        m.borrow_mut().remove(&id);
    });
}

// ───────────────────────── Program / Primitive / Pipeline ─────────────────────────

/// `.at` 侧对 `video` 声明的受控下行 + 目标尺寸（每帧由视图构建器给出）。
#[derive(Debug, Clone, Default)]
pub struct VideoWidgetProps {
    /// 受控下行（§2.3）。
    pub down: VideoContractDown,
    /// 上屏尺寸（逻辑像素；本 widget 用物理像素近似，见 `prepare`）。
    pub width: u32,
    pub height: u32,
}

/// `video` 的 iced shader widget 程序。
#[derive(Debug)]
pub struct VideoProgram {
    props: VideoWidgetProps,
}

impl VideoProgram {
    pub fn new(props: VideoWidgetProps) -> Self {
        Self { props }
    }
}

/// 每个 widget 的持久状态（iced 会在 widget 生命周期内保留它）。
#[derive(Debug)]
pub struct VideoWidgetState {
    id: u64,
}

impl Default for VideoWidgetState {
    fn default() -> Self {
        Self {
            id: next_widget_id(),
        }
    }
}

impl<Message: 'static> shader::Program<Message> for VideoProgram {
    type State = VideoWidgetState;
    type Primitive = VideoPrimitive;

    fn draw(
        &self,
        state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        VideoPrimitive {
            id: state.id,
            down: self.props.down.clone(),
            width: self.props.width,
            height: self.props.height,
        }
    }
}

/// 交给 iced 的图元：**纯数据**，故天然 `Send + Sync`。
#[derive(Debug, Clone)]
pub struct VideoPrimitive {
    id: u64,
    down: VideoContractDown,
    width: u32,
    height: u32,
}

impl VideoPrimitive {
    /// 这次绘制想要的目标纹理尺寸（至少 1×1，避免 0 尺寸纹理）。
    fn target_size(&self) -> (u32, u32) {
        (self.width.max(1), self.height.max(1))
    }
}

impl iced_wgpu::primitive::Primitive for VideoPrimitive {
    type Pipeline = VideoPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &iced_wgpu::wgpu::Device,
        queue: &iced_wgpu::wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        // 尺寸以 bounds 为准（布局最终说了算），props 只作兜底。
        let (mut w, mut h) = self.target_size();
        if bounds.width >= 1.0 && bounds.height >= 1.0 {
            w = bounds.width.round().max(1.0) as u32;
            h = bounds.height.round().max(1.0) as u32;
        }

        // ① 驱动播放：下行同步 + 上行采集 + 渲染一帧到我们的纹理。
        //
        // 注意顺序：**先 apply/poll 再由 channel 取帧**，这样 seek/暂停这一类
        // 下行在同一帧内就生效，不会多显示一帧旧状态。
        let drained = with_runtime(self.id, |rt| {
            let Some(engine) = rt.engine.as_ref() else {
                // 无引擎：保持不可用状态（上层已画降级面板）。
                return Vec::new();
            };
            rt.contract.apply(engine, &self.down);
            let events = rt.contract.poll(engine);
            rt.pending.extend(events);
            // 只在这帧真有新帧可渲染时才推进（`has_new_frame` 来自
            // ADVANCED_CONTROL，是 T-15 实测的生产形状）。
            if engine.has_new_frame() {
                let (gw, gh) = (w, h);
                pipeline.ensure_target(device, queue, self.id, gw, gh);
                if let Some(targets) = pipeline.targets.get_mut(&self.id) {
                    let gen = rt.contract.generation();
                    let seq = targets.seq;
                    let out = targets.channel.with_frame(gen, seq, |t| engine.render_sw_frame(&t));
                    targets.seq = seq.wrapping_add(1);
                    targets.last_outcome = Some(out);
                }
            }
            rt.pending.clone()
        });
        let _ = drained;
    }

    fn draw(
        &self,
        pipeline: &Self::Pipeline,
        render_pass: &mut iced_wgpu::wgpu::RenderPass<'_>,
    ) -> bool {
        let Some(targets) = pipeline.targets.get(&self.id) else {
            // 还没建目标（首帧前 / 无引擎）→ 不画，让上层背景显示。
            return false;
        };
        let Some(bind_group) = targets.bind_group.as_ref() else {
            return false;
        };
        render_pass.set_pipeline(&pipeline.render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        true
    }
}

/// 一个 video widget 的 GPU 资源（持久纹理 + staging 环 + bind group）。
struct VideoTargets {
    size: (u32, u32),
    channel: VideoFrameChannel,
    bind_group: Option<iced_wgpu::wgpu::BindGroup>,
    /// 帧序号（配合契约的 generation 喂给 `VideoLatestWins`）。
    seq: u64,
    last_outcome: Option<super::channel::FrameOutcome>,
}

/// `video` 的渲染管线。iced 每种 `Primitive` 类型只建一次。
pub struct VideoPipeline {
    device: iced_wgpu::wgpu::Device,
    render_pipeline: iced_wgpu::wgpu::RenderPipeline,
    layout: iced_wgpu::wgpu::BindGroupLayout,
    sampler: iced_wgpu::wgpu::Sampler,
    targets: HashMap<u64, VideoTargets>,
    /// 只用于诊断/测试：管线建过几次目标（应等于 widget 数，不随帧数增长）。
    targets_created: u32,
}

impl VideoPipeline {
    /// 建目标（纹理 + ring + bind group）。**只在尺寸变化时重建**——
    /// 每帧重建纹理就是 T-15 认定的闪烁成因。
    fn ensure_target(
        &mut self,
        device: &iced_wgpu::wgpu::Device,
        queue: &iced_wgpu::wgpu::Queue,
        id: u64,
        width: u32,
        height: u32,
    ) {
        let need = match self.targets.get(&id) {
            Some(t) => t.size != (width, height),
            None => true,
        };
        if !need {
            return;
        }
        let channel = VideoFrameChannel::new(device, queue, width, height);
        let view = channel.view();
        let bind_group = device.create_bind_group(&iced_wgpu::wgpu::BindGroupDescriptor {
            label: Some("video widget bind group"),
            layout: &self.layout,
            entries: &[
                iced_wgpu::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: iced_wgpu::wgpu::BindingResource::TextureView(view),
                },
                iced_wgpu::wgpu::BindGroupEntry {
                    binding: 1,
                    resource: iced_wgpu::wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        self.targets_created += 1;
        self.targets.insert(
            id,
            VideoTargets {
                size: (width, height),
                channel,
                bind_group: Some(bind_group),
                seq: 0,
                last_outcome: None,
            },
        );
    }

    /// 建过的目标数（诊断/测试：应等于 widget 数，**不随帧数增长**）。
    pub fn targets_created(&self) -> u32 {
        self.targets_created
    }

    /// 某 widget 最近一帧的上屏结果（诊断/测试）。
    pub fn last_outcome(&self, id: u64) -> Option<&super::channel::FrameOutcome> {
        self.targets.get(&id).and_then(|t| t.last_outcome.as_ref())
    }

    /// 某 widget 的通道统计（AC-18/AC-19 的实测来源）。
    pub fn stats(&self, id: u64) -> Option<&super::channel::FrameChannelStats> {
        self.targets.get(&id).map(|t| t.channel.stats())
    }
}

impl iced_wgpu::primitive::Pipeline for VideoPipeline {
    fn new(
        device: &iced_wgpu::wgpu::Device,
        _queue: &iced_wgpu::wgpu::Queue,
        format: iced_wgpu::wgpu::TextureFormat,
    ) -> Self {
        use iced_wgpu::wgpu;

        // 复用 T-17 的 WGSL：全屏三角形 + 强制 alpha=1（mpv 的 "rgb0" 第 4 字节是垃圾）。
        let presenter = VideoPresenter::new(device, format);
        let (render_pipeline, layout, sampler) = presenter.into_parts();
        Self {
            device: device.clone(),
            render_pipeline,
            layout,
            sampler,
            targets: HashMap::new(),
            targets_created: 0,
        }
    }
}

impl std::fmt::Debug for VideoPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoPipeline")
            .field("targets", &self.targets.len())
            .field("targets_created", &self.targets_created)
            .finish()
    }
}

/// 目标尺寸下的行距（256 对齐；同时满足 mpv 的 64 与 wgpu 的 256）。
pub fn target_stride(width: u32) -> usize {
    stride_for(width)
}
