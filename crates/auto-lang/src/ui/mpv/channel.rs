//! 帧 → GPU 纹理 的通道（PLAN-617 **T-17**）。
//!
//! # 为什么不是 iced 的图像通道
//!
//! T-15 用三条源码级证据否掉了它（见 [Design 30 §4.5]）：`Handle::from_rgba`
//! 每帧 mint 新的 `Id::unique()`（`Id` 构造器私有，仓外无法复用同一 id）；同 id
//! 在 `cache::load_image` 命中即**不再上传**（只会显示陈旧帧）；且真实帧尺寸
//! （1080p 8.29 MB / 4K 33.18 MB）全部超过 `MAX_SYNC_SIZE`（2 MB）必走异步 worker
//! 上传、4K 更超过 atlas 的 2048 上限。**故本通道自持纹理，完全不经过
//! `Handle`/atlas**——闪烁的成因随之被结构性移除。
//!
//! [Design 30 §4.5]: ../../../../docs/design/autoui/030-video-player.md
//!
//! # 形状（T-15 选定的「通道 C」）
//!
//! ```text
//! mpv SW renderer ──直接写──▶ 持久映射的 staging buffer ──copy_buffer_to_texture──▶ 持久 wgpu 纹理
//! ```
//!
//! 关键点：mpv 的软件渲染器**直接写进 GPU 可见的映射内存**，于是「帧 → GPU」
//! 只剩一次设备侧拷贝。实测（T-15 门控 B，60 Hz 节流 + 预热后取样）：
//! 1080p **0.28 ms/帧**、4K **0.41 ms/帧**；对照 `queue.write_texture`
//! 为 0.67 / 3.87 ms（4K 差约 9×），对照「每帧新建纹理」为 1.22 / 4.28 ms
//! 且 4K 的 p95 恶化到 19 ms、120 帧里有 8 帧离群。
//!
//! # 两条硬约束（写进类型，不靠自律）
//!
//! 1. **纹理只建一次**，每帧原地更新——绝不每帧新建（那条路正是
//!    `ui/iced/renderer.rs:2889-2898` 记录的闪烁机理）。
//! 2. **行距按 256 对齐**：wgpu 的 `copy_buffer_to_texture` 要求
//!    `bytes_per_row` 是 `COPY_BYTES_PER_ROW_ALIGNMENT = 256` 的倍数，而 mpv 要求
//!    stride 是 64 的倍数（`render.h:393-404`）。取 **256** 同时满足两者。
//!
//! # 那条稀有长尾
//!
//! T-15 实测到 4K 下 120 帧里有 1 帧 `copy_buffer_to_texture` 耗时 **957 ms**
//! （release 档同样出现，故非 debug 产物）。若在 UI 线程上等它，画面会卡死近一秒。
//! 本通道的处置是**回收等待带超时，超时即丢帧**（[`FrameOutcome::ReclaimTimeout`]）：
//! 视频场景里丢一帧远好于冻结界面，且**绝不会因此出现空白帧**——纹理始终保留上一帧
//! 的内容，这正是「无闪烁」的定义。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use iced_wgpu::wgpu;

use super::frame::SwTarget;

/// 复用 iced 渲染器所用的**同一个** wgpu 实例（而不是直接依赖 wgpu 另开一条边，
/// 那会有版本不一致的风险）。转出给调用方与测试用。
pub use iced_wgpu::wgpu as wgpu_reexport;

/// mpv 要求的对齐（`render.h:393-404`）。
const MPV_ALIGN: usize = 64;
/// wgpu `copy_buffer_to_texture` 对 `bytes_per_row` 的对齐要求。
/// 它同时满足 [`MPV_ALIGN`]（256 是 64 的倍数），故一个 stride 两边都用。
pub const GPU_ROW_ALIGN: usize = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;

/// 帧缓冲槽位数。
///
/// 3 槽的理由：一槽在被 mpv 写、一槽在 GPU 拷贝中、一槽空闲可立即取用——
/// 这样正常的帧节奏下**根本不需要等待回收**（等待只在异常长尾时发生）。
pub const RING_SLOTS: usize = 3;

/// 一帧的处理结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameOutcome {
    /// 已提交 GPU 拷贝。
    Submitted,
    /// 帧已过期（用户 seek / 换片，代际不匹配）——丢。
    DroppedStale,
    /// 环里没有空闲槽——丢（宁可丢帧也不阻塞 UI）。
    DroppedNoSlot,
    /// 等待回收槽位超时（长尾）——丢。
    ReclaimTimeout,
    /// mpv 侧渲染失败。
    RenderFailed(String),
    /// 上屏拷贝失败。
    UploadFailed(String),
}

impl FrameOutcome {
    pub fn is_submitted(&self) -> bool {
        matches!(self, Self::Submitted)
    }
    /// 是否属于「按策略主动丢帧」（而非错误）。
    pub fn is_deliberate_drop(&self) -> bool {
        matches!(
            self,
            Self::DroppedStale | Self::DroppedNoSlot | Self::ReclaimTimeout
        )
    }
}

/// 最新优先门（**镜像 `image_pipeline.rs` 的 `MediaLatestWins`**：同样的
/// `(generation, revision)` 二元组 + `accept_*` 返回 bool 并计丢帧数）。
///
/// 视频语境的语义：`generation` 在**用户 seek / 换片**时前进，`seq` 是我们要
/// 接受的那一帧的序号。已经前进的 generation 会把在途的旧帧判为过期——否则
/// seek 之后旧帧的拷贝会盖掉新位置的画面。
#[derive(Debug, Clone)]
pub struct VideoLatestWins {
    generation: u64,
    seq: u64,
    dropped: u64,
}

impl VideoLatestWins {
    pub const fn new(generation: u64, seq: u64) -> Self {
        Self {
            generation,
            seq,
            dropped: 0,
        }
    }

    /// 前进到新的 (generation, seq)：调用方声明「接下来只接受这个组合」。
    pub fn advance(&mut self, generation: u64, seq: u64) {
        self.generation = generation;
        self.seq = seq;
    }

    fn accept(&mut self, generation: u64, seq: u64) -> bool {
        let ok = (generation, seq) == (self.generation, self.seq);
        if !ok {
            self.dropped = self.dropped.saturating_add(1);
        }
        ok
    }

    /// 在把一帧交给 mpv 渲染**之前**判一次（过期就不必渲染了）。
    pub fn accept_enqueue(&mut self, generation: u64, seq: u64) -> bool {
        self.accept(generation, seq)
    }

    /// 在把渲染结果上屏**之前**再判一次（渲染期间可能已经 seek 了）。
    pub fn accept_publish(&mut self, generation: u64, seq: u64) -> bool {
        self.accept(generation, seq)
    }

    pub const fn dropped(&self) -> u64 {
        self.dropped
    }

    pub const fn current(&self) -> (u64, u64) {
        (self.generation, self.seq)
    }
}

/// 通道的累计统计（AC-18/AC-19 的实测数字来源）。
#[derive(Debug, Clone, Default)]
pub struct FrameChannelStats {
    pub submitted: u64,
    pub dropped_stale: u64,
    pub dropped_no_slot: u64,
    pub reclaim_timeouts: u64,
    pub reclaim_waits: u64,
    /// 单帧「映射 + 上屏提交」的耗时（毫秒），用于分位与长尾观测。
    pub upload_ms: Vec<f64>,
}

impl FrameChannelStats {
    /// 丢帧总数（三类）。稳态下应远小于 `submitted`。
    pub fn dropped(&self) -> u64 {
        self.dropped_stale + self.dropped_no_slot + self.reclaim_timeouts
    }

    pub fn p50(&self) -> f64 {
        percentile(&self.upload_ms, 0.50)
    }

    pub fn p95(&self) -> f64 {
        percentile(&self.upload_ms, 0.95)
    }

    pub fn max(&self) -> f64 {
        self.upload_ms.iter().cloned().fold(0.0, f64::max)
    }

    /// 超过 p50 三倍的「离群帧」数——长尾的直接证据。
    pub fn outliers(&self) -> usize {
        let p50 = self.p50();
        self.upload_ms.iter().filter(|&&v| v > p50 * 3.0).count()
    }

    /// 是否已经不再保留逐帧耗时（长会话下避免无界增长）。
    pub fn is_trimmed(&self) -> bool {
        STATS_SAMPLE_CAP > 0 && self.upload_ms.len() >= STATS_SAMPLE_CAP
    }
}

/// 逐帧耗时的保留上限：只用于诊断，超过就停止累积（避免长会话内存无界增长）。
const STATS_SAMPLE_CAP: usize = 4096;

fn percentile(xs: &[f64], q: f64) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() as f64 * q) as usize).min(v.len() - 1)]
}

/// 一个环槽。
struct Slot {
    buffer: wgpu::Buffer,
    /// 该槽最后一次提交（用于**定向**回收：只等这个 submission，不串行化整个 GPU）。
    last_submit: Option<wgpu::SubmissionIndex>,
    /// `map_async` 的回调置位标志（是否已映射完成）。
    mapped: Arc<AtomicBool>,
    /// 缓冲是否处于「已映射」状态（映射中/在途都不算）。
    is_mapped: bool,
}

/// 帧 → 纹理 的通道。
///
/// 持有**一张持久纹理**（每帧原地更新）与一个 **3 槽 staging 环**。
/// 不持有任何 `iced` 图像类型——这是 T-15 结论的直接落实。
pub struct VideoFrameChannel {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    /// 行距（256 对齐，同时满足 mpv 的 64 与 wgpu 的 256）。
    stride: usize,
    slots: Vec<Slot>,
    gate: VideoLatestWins,
    stats: FrameChannelStats,
    /// 回收槽位时最多等多久；超时即丢帧（长尾处置）。
    reclaim_timeout: std::time::Duration,
    /// 本通道创建过的纹理总数——**恒为 1**，用于自证「没有每帧新建纹理」。
    textures_created: u32,
}

impl VideoFrameChannel {
    /// 建通道：一张持久纹理 + [`RING_SLOTS`] 个 staging 缓冲。
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let stride = stride_for(width);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("video frame (persistent)"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // mpv 的 "rgb0" 是 8 位/分量、字节序 R,G,B,X；用 sRGB 让采样时
            // 正确线性化（mpv 已做过色调映射，输出即显示域）。
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let buffer_size = (stride * height as usize) as u64;
        let slots = (0..RING_SLOTS)
            .map(|i| Slot {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("video frame staging #{i}")),
                    size: buffer_size,
                    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
                    mapped_at_creation: false,
                }),
                last_submit: None,
                mapped: Arc::new(AtomicBool::new(false)),
                is_mapped: false,
            })
            .collect();

        Self {
            device: device.clone(),
            queue: queue.clone(),
            texture,
            view,
            width,
            height,
            stride,
            slots,
            gate: VideoLatestWins::new(0, 0),
            stats: FrameChannelStats::default(),
            reclaim_timeout: std::time::Duration::from_millis(4),
            textures_created: 1,
        }
    }

    /// 纹理尺寸。
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// 行距（256 对齐）。
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// 供渲染用的纹理视图——**每帧都用同一个**，调用方不得缓存成「每帧新建」。
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn gate(&self) -> &VideoLatestWins {
        &self.gate
    }

    pub fn stats(&self) -> &FrameChannelStats {
        &self.stats
    }

    /// 创建过的纹理数——**自证常量 1**（T-15 硬约束②：绝不每帧新建纹理）。
    pub fn textures_created(&self) -> u32 {
        self.textures_created
    }

    /// 设回收等待上限（测试与调优用）。
    pub fn set_reclaim_timeout(&mut self, timeout: std::time::Duration) {
        self.reclaim_timeout = timeout;
    }

    /// 声明「接下来只接受 (generation, seq)」——用户 seek / 换片时调用。
    pub fn advance(&mut self, generation: u64, seq: u64) {
        self.gate.advance(generation, seq);
    }

    /// **一帧的完整流程**：门控 → 取槽 → 映射 → 交给 `render` 写 → 解映射 → 提交拷贝。
    ///
    /// 用闭包而不是「返回槽句柄」是为了让借用关系无法写错：整个流程持有 `&mut self`，
    /// 调用方拿到的只有一个 [`SwTarget`]。
    ///
    /// `render` 负责让 mpv 把帧写进这个 target（典型实现：
    /// [`super::engine::MpvEngine::render_sw_frame`]）。返回 `Ok(())` 表示写入成功。
    pub fn with_frame<F>(&mut self, generation: u64, seq: u64, render: F) -> FrameOutcome
    where
        F: FnOnce(SwTarget) -> Result<(), String>,
    {
        // ① 门控：过期帧连渲染都不必做。
        if !self.gate.accept_enqueue(generation, seq) {
            self.stats.dropped_stale += 1;
            return FrameOutcome::DroppedStale;
        }

        // ② 取一个空闲槽（必要时定向回收；回收超时则丢帧而不是阻塞 UI）。
        let Some(idx) = self.acquire_slot() else {
            self.stats.dropped_no_slot += 1;
            return FrameOutcome::DroppedNoSlot;
        };

        let t0 = std::time::Instant::now();

        // ③ 映射该槽，让 mpv 直接写进 GPU 可见内存。
        if let Err(e) = self.map_slot(idx) {
            self.stats.dropped_no_slot += 1;
            return FrameOutcome::RenderFailed(format!("映射 staging 失败：{e}"));
        }

        // ④ 把帧写进去（mpv 的 SW renderer 直接写这块映射内存）。
        let target = {
            let ptr = self.slot_ptr(idx);
            // SAFETY: ptr 指向已映射的 slot 缓冲，长度 >= stride*height；
            // 映射在本次闭包返回后、unmap 之前一直有效。
            unsafe { SwTarget::new(ptr, self.width, self.height, self.stride) }
        };
        if let Err(e) = render(target) {
            // 渲染失败也要把槽收回（解映射），否则这块槽会永久卡住。
            self.unmap_slot(idx);
            return FrameOutcome::RenderFailed(e);
        }

        // ⑤ 上屏前再判一次门（渲染期间用户可能已经 seek）。
        if !self.gate.accept_publish(generation, seq) {
            self.unmap_slot(idx);
            self.stats.dropped_stale += 1;
            return FrameOutcome::DroppedStale;
        }

        // ⑥ 解映射 → 编码拷贝 → 提交。
        self.unmap_slot(idx);
        let extent = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("video frame upload"),
            });
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &self.slots[idx].buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.stride as u32),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            extent,
        );
        let submission = self.queue.submit([encoder.finish()]);
        self.slots[idx].last_submit = Some(submission);

        let dt = t0.elapsed().as_secs_f64() * 1000.0;
        self.stats.submitted += 1;
        if self.stats.upload_ms.len() < STATS_SAMPLE_CAP {
            self.stats.upload_ms.push(dt);
        }
        FrameOutcome::Submitted
    }

    /// 找一个可立即使用的槽；把在途槽定向回收成可用。
    ///
    /// 定向回收（`submission_index: Some(..)`）而非「等所有提交完成」是关键：
    /// 后者会把 GPU 串行化，把流水线优势全部抹掉。
    fn acquire_slot(&mut self) -> Option<usize> {
        // 先找从未提交或已完成回收的槽。
        for i in 0..self.slots.len() {
            if self.slots[i].last_submit.is_none() {
                return Some(i);
            }
        }
        // 都在途：定向等一个，带超时。
        for i in 0..self.slots.len() {
            let Some(idx) = self.slots[i].last_submit.clone() else {
                continue;
            };
            self.stats.reclaim_waits += 1;
            let ok = self
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(idx),
                    timeout: Some(self.reclaim_timeout),
                })
                .is_ok();
            if ok {
                self.slots[i].last_submit = None;
                return Some(i);
            }
            self.stats.reclaim_timeouts += 1;
        }
        None
    }

    fn map_slot(&mut self, idx: usize) -> Result<(), String> {
        let slot = &self.slots[idx];
        if slot.is_mapped {
            return Ok(());
        }
        slot.mapped.store(false, Ordering::SeqCst);
        let flag = Arc::clone(&slot.mapped);
        slot.buffer
            .slice(..)
            .map_async(wgpu::MapMode::Write, move |res| {
                if res.is_ok() {
                    flag.store(true, Ordering::SeqCst);
                }
            });
        // 映射完成需要设备轮询；此处缓冲不在途，故这一次轮询很轻。
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(50);
        while !self.slots[idx].mapped.load(Ordering::SeqCst) {
            let _ = self.device.poll(wgpu::PollType::Poll);
            if std::time::Instant::now() > deadline {
                return Err("map_async 未在 50 ms 内完成".into());
            }
            std::thread::yield_now();
        }
        self.slots[idx].is_mapped = true;
        Ok(())
    }

    fn unmap_slot(&mut self, idx: usize) {
        if self.slots[idx].is_mapped {
            self.slots[idx].buffer.unmap();
            self.slots[idx].is_mapped = false;
        }
    }

    /// 已映射槽的写入起点指针。
    fn slot_ptr(&self, idx: usize) -> *mut u8 {
        // SAFETY: 调用点保证该槽已映射（map_slot 成功返回）且未 unmap。
        // `get_mapped_range_mut` 的借用在这里立即结束——我们只取基址，
        // 其后由 mpv 在「本槽仍映射」的窗口内写这块内存。
        let view = self.slots[idx].buffer.slice(..).get_mapped_range_mut();
        view.as_ptr() as *mut u8
    }
}

/// 给定宽度下的行距：向上取到 [`GPU_ROW_ALIGN`]（=256）的倍数。
///
/// 256 同时满足 wgpu（`bytes_per_row` 需 256 对齐）与 mpv（stride 需 64 对齐），
/// 故一个 stride 两边通用——这是本通道能成立的一个前提，值得显式写出。
pub fn stride_for(width: u32) -> usize {
    let raw = width as usize * 4;
    raw.div_ceil(GPU_ROW_ALIGN) * GPU_ROW_ALIGN
}

/// 该宽度是否无需额外 padding（即 `width*4` 本就是 256 的倍数）。
pub fn is_native_stride(width: u32) -> bool {
    (width as usize * 4) % GPU_ROW_ALIGN == 0
}

/// [`MPV_ALIGN`] 对外的只读入口（测试用）。
pub fn mpv_align() -> usize {
    MPV_ALIGN
}

impl std::fmt::Debug for VideoFrameChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoFrameChannel")
            .field("size", &format_args!("{}x{}", self.width, self.height))
            .field("stride", &self.stride)
            .field("slots", &self.slots.len())
            .field("textures_created", &self.textures_created)
            .field("gate", &self.gate.current())
            .field("submitted", &self.stats.submitted)
            .field("dropped", &self.stats.dropped())
            .finish()
    }
}
