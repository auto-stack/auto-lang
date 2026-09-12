//! PLAN-617 T-15: VM 原生播放门控 spike（Go/No-Go 实测件）——**可跑的门控证据**。
//!
//! 不是产品代码：它把「libmpv 能否用 `libloading` 运行时加载并渲染出帧」与
//! 「一帧能不能上屏、代价多少」两件事变成可复跑的数字。T-16..T-20 只有门控判
//! Go 后才在此之上落地。
//!
//! # 为什么是 example 而不是单元测试
//!
//! 本特性开着去编译 **lib 的 `--test` 目标**会撞上 rustc 1.98.0 的 ICE
//! （`collect_and_partition_mono_items`，查询栈指向 `ui/mcp_server.rs:1718`
//! 的 iterator chain，与 spike 代码无关）；`--lib`（rlib）目标正常，故 spike
//! 落在 example 目标上——顺带也让默认档完全不编译它（`required-features`）。
//!
//! # 三条被实测钉死的前提（证据见计划 §9.14 与 `docs/design/autoui/030-video-player.md`）
//!
//! 1. `Handle::from_rgba` 每帧 mint 新的 `Id::unique()`，而 `iced_core::image::Id`
//!    的构造器（`Id::unique` / `Id::path`）与元组字段均**私有** —— 仓外无法构造
//!    「同 id、不同像素」的 Handle，故**帧流不可能靠复用 Handle 消除闪烁**；
//! 2. `iced_wgpu::image::cache::load_image` 按 handle id 命中缓存即**不再上传**，
//!    故稳定 id 只会一直显示陈旧帧；
//! 3. 视频帧在本仓实际尺寸下必然走慢路径：`MAX_SYNC_SIZE = 2 MB`（1080p 帧
//!    8.29 MB、4K 帧 33.18 MB 全超过）、atlas `MAX_SIZE = 2048`（4K 必须碎片化）。
//!
//! → 结论方向：帧上屏必须**绕开 iced 图像通道**、自持 `wgpu::Texture`（T-17 的
//! 通道形状）。本 spike 实测该通道的每帧代价，并与「每帧新建纹理」对照。
//!
//! # 运行
//!
//! ```text
//! # 取得 libmpv（本机无 mpv，需先下载 mpv-dev 构件拿到 libmpv-2.dll）
//! #   https://github.com/shinchiro/mpv-winbuild-cmake/releases -> mpv-dev-x86_64-*.7z
//! set AUTO_MPV_LIB=D:\path\to\libmpv-2.dll
//! cargo run -p auto-lang --features mpv-spike --example mpv_spike -- all
//! cargo run -p auto-lang --features mpv-spike --example mpv_spike -- selfcheck
//! ```
//!
//! 子命令：`selfcheck`（缺库降级路径）/ `mpv`（加载 + 逐帧渲染代价）/
//! `wgpu`（帧上传代价对照）/ `all`（后两者）。

use std::time::Instant;

// ───────────────────────── libmpv 运行时加载（T-16 的立地基础） ─────────────────────────

/// libmpv 的解析序（T-16 将正式定义的形状，spike 先按此实现）：
/// `AUTO_MPV_LIB` 显式路径 → 与可执行文件同目录 → （未实现）系统搜索路径。
///
/// 返回 `None` 表示「本机没有运行库」——**这是降级路径的入口，不是错误**：
/// 调用方据此走今日的诚实占位（`render_support.rs:307` 的 fallback），
/// 不 panic、不黑屏（AC-20）。
fn resolve_mpv_library() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("AUTO_MPV_LIB") {
        let p = std::path::PathBuf::from(p);
        return if p.is_file() { Some(p) } else { None };
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join("libmpv-2.dll");
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    None
}

/// mpv render API 常数（`include/mpv/render.h`）。
mod param {
    pub const INVALID: i32 = 0;
    pub const API_TYPE: i32 = 1;
    pub const ADVANCED_CONTROL: i32 = 10;
    pub const BLOCK_FOR_TARGET_TIME: i32 = 12;
    pub const SW_SIZE: i32 = 17;
    pub const SW_FORMAT: i32 = 18;
    pub const SW_STRIDE: i32 = 19;
    pub const SW_POINTER: i32 = 20;
    /// `mpv_render_context_update()` 的返回位：有新帧可渲染。
    pub const UPDATE_FRAME: u64 = 1;
}

/// `mpv_event`（client.h）——spike 只读 `event_id`。
#[repr(C)]
struct MpvEvent {
    event_id: i32,
    error: i32,
    reply_userdata: u64,
    data: *mut std::ffi::c_void,
}

const EVENT_END_FILE: i32 = 7;
const EVENT_FILE_LOADED: i32 = 8;
const EVENT_VIDEO_RECONFIG: i32 = 17;

#[repr(C)]
struct RenderParamRaw {
    type_: i32,
    data: *mut std::ffi::c_void,
}

/// `mpv_render_param` 数组：以 `type = INVALID` 结尾（header 约定）。
struct ParamList(Vec<RenderParamRaw>);

impl ParamList {
    fn new() -> Self {
        ParamList(vec![RenderParamRaw {
            type_: param::INVALID,
            data: std::ptr::null_mut(),
        }])
    }
    fn push(mut self, type_: i32, data: *mut std::ffi::c_void) -> Self {
        let n = self.0.len();
        self.0.insert(n - 1, RenderParamRaw { type_, data });
        self
    }
    fn as_ptr(&self) -> *const RenderParamRaw {
        self.0.as_ptr()
    }
}

/// 一帧 4 字节/像素 RGBA 的软件渲染目标缓冲。
struct Surface {
    buf: Vec<u8>,
    stride: usize,
}

impl Surface {
    /// header 要求 stride 与 pointer 均 64 字节对齐以走 SIMD 快路；
    /// `Vec<u8>` 基址不保证对齐，故超分配后按对齐点切分。
    fn new(width: u32, height: u32) -> Self {
        let stride = ((width as usize * 4 + 63) / 64) * 64;
        let needed = stride * height as usize;
        let mut raw = vec![0u8; needed + 64];
        let base = raw.as_ptr() as usize;
        let offset = (64 - (base % 64)) % 64;
        let buf = raw.split_off(offset);
        std::mem::drop(raw);
        debug_assert_eq!(buf.as_ptr() as usize % 64, 0, "surface base must be 64-aligned");
        Surface { buf, stride }
    }
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr()
    }
}

/// 实测计时汇总（决策件数字来源）。
#[derive(Debug, Default)]
struct Timings {
    frames: usize,
    elapsed_s: f64,
    per_frame_ms: Vec<f64>,
}

impl Timings {
    fn p50(&self) -> f64 {
        percentile(&self.per_frame_ms, 0.50)
    }
    fn p95(&self) -> f64 {
        percentile(&self.per_frame_ms, 0.95)
    }
    fn fps(&self) -> f64 {
        if self.elapsed_s > 0.0 {
            self.frames as f64 / self.elapsed_s
        } else {
            0.0
        }
    }
}

/// `p50/p95/p99` 与「>3×p50 的离群帧数」——max 容易被单次分配尖峰污染，
/// 只报 max 会把一次性噪声说成稳态代价。
fn spread(xs: &[f64]) -> String {
    let p50 = percentile(xs, 0.5);
    let out = xs.iter().filter(|&&v| v > p50 * 3.0).count();
    format!(
        "p50={p50:.3} ms p95={:.3} ms p99={:.3} ms max={:.3} ms | >3×p50 的离群帧 {out}/{}",
        percentile(xs, 0.95),
        percentile(xs, 0.99),
        xs.iter().cloned().fold(0.0, f64::max),
        xs.len()
    )
}

fn percentile(xs: &[f64], q: f64) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((v.len() as f64 * q) as usize).min(v.len() - 1);
    v[idx]
}

/// 门控 A：`libloading` 运行时加载 libmpv → SW render context → 把真实文件
/// 的帧渲染进我们自己的缓冲，实测每帧 `render()` 代价。
///
/// 采用**生产形状**：`ADVANCED_CONTROL` + `mpv_render_context_update()`，
/// 只在 mpv 报告有新帧时才 render —— 故每次调用都是真实帧，测出的就是干净的
/// 每帧代价（T-17 将照此实现）。
fn probe_mpv_rendering(lib: &std::path::Path) -> Result<(), String> {
    // 样本：1080p H.264 与 4K HEVC HDR 各一。
    let samples: [(&str, &str, u32, u32); 2] = [
        (
            "1080p H.264 (Loki S01E01)",
            r"E:\Video\TV\Loki\Loki.S01E01.中英字幕.WEBrip.AAC.1080p.x264-远鉴字幕组.mp4",
            1920,
            1080,
        ),
        (
            "4K HEVC HDR (Loki S02E01)",
            r"E:\Video\TV\Loki\Loki.S02E01.2021.2160p.DSNP.WEB-DL.DDP5.1.Atmos.HDR.H.265-FLUX.mkv",
            3840,
            2160,
        ),
    ];
    let mut ran = 0usize;
    for (label, path, w, h) in samples {
        let path = std::path::PathBuf::from(path);
        if !path.is_file() {
            eprintln!("SKIP sample (缺失): {}", path.display());
            continue;
        }
        ran += 1;
        match render_one(lib, &path, w, h, 120) {
            Ok(t) => eprintln!(
                "[{label}] {w}x{h}  frames={} in {:.2}s => {:.1} fps ceiling | \
                 render() p50={:.2} ms p95={:.2} ms | RGBA {:.2} MB/frame",
                t.frames,
                t.elapsed_s,
                t.fps(),
                t.p50(),
                t.p95(),
                (w as f64 * h as f64 * 4.0) / 1e6
            ),
            Err(e) => eprintln!("[{label}] ERROR: {e}"),
        }
    }
    if ran == 0 {
        eprintln!("注意：两个样本都不在本机 —— 只验证了加载与初始化路径");
    }
    Ok(())
}

/// 渲染 `want_frames` 帧，返回实测计时。
fn render_one(
    lib: &std::path::Path,
    video: &std::path::Path,
    w: u32,
    h: u32,
    want_frames: usize,
) -> Result<Timings, String> {
    unsafe {
        let lib = libloading::Library::new(lib).map_err(|e| format!("load: {e}"))?;
        // 只取本 spike 用到的符号；签名照 mpv/client.h 与 mpv/render.h。
        let mpv_create: libloading::Symbol<unsafe extern "C" fn() -> *mut std::ffi::c_void> =
            lib.get(b"mpv_create\0").map_err(|e| format!("mpv_create: {e}"))?;
        let mpv_initialize: libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> i32> =
            lib.get(b"mpv_initialize\0").map_err(|e| format!("mpv_initialize: {e}"))?;
        let mpv_set_option_string: libloading::Symbol<
            unsafe extern "C" fn(*mut std::ffi::c_void, *const i8, *const i8) -> i32,
        > = lib
            .get(b"mpv_set_option_string\0")
            .map_err(|e| format!("mpv_set_option_string: {e}"))?;
        let mpv_command: libloading::Symbol<
            unsafe extern "C" fn(*mut std::ffi::c_void, *const *const i8) -> i32,
        > = lib.get(b"mpv_command\0").map_err(|e| format!("mpv_command: {e}"))?;
        let mpv_wait_event: libloading::Symbol<
            unsafe extern "C" fn(*mut std::ffi::c_void, f64) -> *mut MpvEvent,
        > = lib
            .get(b"mpv_wait_event\0")
            .map_err(|e| format!("mpv_wait_event: {e}"))?;
        let mpv_render_context_create: libloading::Symbol<
            unsafe extern "C" fn(
                *mut *mut std::ffi::c_void,
                *mut std::ffi::c_void,
                *const RenderParamRaw,
            ) -> i32,
        > = lib
            .get(b"mpv_render_context_create\0")
            .map_err(|e| format!("mpv_render_context_create: {e}"))?;
        let mpv_render_context_render: libloading::Symbol<
            unsafe extern "C" fn(*mut std::ffi::c_void, *const RenderParamRaw) -> i32,
        > = lib
            .get(b"mpv_render_context_render\0")
            .map_err(|e| format!("mpv_render_context_render: {e}"))?;
        let mpv_render_context_update: libloading::Symbol<
            unsafe extern "C" fn(*mut std::ffi::c_void) -> u64,
        > = lib
            .get(b"mpv_render_context_update\0")
            .map_err(|e| format!("mpv_render_context_update: {e}"))?;
        let mpv_render_context_free: libloading::Symbol<
            unsafe extern "C" fn(*mut std::ffi::c_void),
        > = lib
            .get(b"mpv_render_context_free\0")
            .map_err(|e| format!("mpv_render_context_free: {e}"))?;
        let mpv_terminate_destroy: libloading::Symbol<
            unsafe extern "C" fn(*mut std::ffi::c_void),
        > = lib
            .get(b"mpv_terminate_destroy\0")
            .map_err(|e| format!("mpv_terminate_destroy: {e}"))?;

        let handle = mpv_create();
        if handle.is_null() {
            return Err("mpv_create returned NULL".into());
        }
        // 与 Python 探针同一组选项：vo 由 render API 接管；untimed 求吞吐上界；
        // 关音频（本 spike 只测帧管线；音频输出是 T-18 的事）。
        for (k, v) in [
            ("vo", "libmpv"),
            ("audio", "no"),
            ("untimed", "yes"),
            ("terminal", "no"),
            ("msg-level", "all=error"),
        ] {
            let ck = std::ffi::CString::new(k).unwrap();
            let cv = std::ffi::CString::new(v).unwrap();
            let r = mpv_set_option_string(handle, ck.as_ptr(), cv.as_ptr());
            if r < 0 {
                return Err(format!("option {k}={v} rejected (code {r})"));
            }
        }
        if mpv_initialize(handle) < 0 {
            return Err("mpv_initialize failed".into());
        }

        let mut surface = Surface::new(w, h);
        let mut ctx: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut enable_adv = 1i32;
        let api_type = b"sw\0";
        let create_params = ParamList::new()
            .push(param::API_TYPE, api_type.as_ptr() as *mut _)
            .push(
                param::ADVANCED_CONTROL,
                &mut enable_adv as *mut i32 as *mut _,
            );
        let r = mpv_render_context_create(&mut ctx, handle, create_params.as_ptr());
        if r < 0 {
            return Err(format!("mpv_render_context_create(sw) -> {r}"));
        }

        let cpath = std::ffi::CString::new(video.to_string_lossy().as_bytes())
            .map_err(|e| format!("path: {e}"))?;
        let argv: [*const i8; 3] = [
            b"loadfile\0".as_ptr() as *const i8,
            cpath.as_ptr(),
            std::ptr::null(),
        ];
        if mpv_command(handle, argv.as_ptr()) < 0 {
            return Err("loadfile failed".into());
        }

        // 等首个可渲染事件（FILE_LOADED / VIDEO_RECONFIG）。
        let t_load = Instant::now();
        while t_load.elapsed().as_secs_f64() < 20.0 {
            let ev = mpv_wait_event(handle, 0.05);
            if !ev.is_null() {
                let id = (*ev).event_id;
                if id == EVENT_FILE_LOADED || id == EVENT_VIDEO_RECONFIG {
                    break;
                }
                if id == EVENT_END_FILE {
                    return Err("END_FILE before first frame".into());
                }
            }
        }

        let size = [w as i32, h as i32];
        let fmt = b"rgb0\0";
        let stride = surface.stride;
        let t0 = Instant::now();
        let mut no_block = 0i32;
        let mut per_frame_ms = Vec::with_capacity(want_frames);
        let mut got_first = false;
        while per_frame_ms.len() < want_frames {
            if got_first {
                // 没有新帧就不 render：避免把空转算进每帧代价。
                if mpv_render_context_update(ctx) & param::UPDATE_FRAME == 0 {
                    if t0.elapsed().as_secs_f64() > 30.0 {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    continue;
                }
            }
            let params = ParamList::new()
                .push(param::SW_SIZE, size.as_ptr() as *mut _)
                .push(param::SW_FORMAT, fmt.as_ptr() as *mut _)
                .push(
                    param::SW_STRIDE,
                    &stride as *const usize as *mut _,
                )
                .push(param::SW_POINTER, surface.as_mut_ptr() as *mut _)
                .push(
                    param::BLOCK_FOR_TARGET_TIME,
                    &mut no_block as *mut i32 as *mut _,
                );
            let f0 = Instant::now();
            let r = mpv_render_context_render(ctx, params.as_ptr());
            let dt = f0.elapsed().as_secs_f64() * 1000.0;
            if r < 0 {
                if !got_first {
                    if t0.elapsed().as_secs_f64() > 25.0 {
                        return Err("no frame within 25 s".into());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                    continue;
                }
                break;
            }
            got_first = true;
            per_frame_ms.push(dt);
        }
        let elapsed_s = t0.elapsed().as_secs_f64();

        mpv_render_context_free(ctx);
        mpv_terminate_destroy(handle);
        Ok(Timings {
            frames: per_frame_ms.len(),
            elapsed_s,
            per_frame_ms,
        })
    }
}

// ───────────────────────── 帧上屏代价（T-17 的通道选择依据） ─────────────────────────

/// `write_texture` 的目标描述子（写成自由函数而非闭包：闭包无法表达
/// 「返回值借用入参」的生命周期关系）。
fn tex_copy_dest(
    tex: &iced_wgpu::wgpu::Texture,
) -> iced_wgpu::wgpu::TexelCopyTextureInfo<'_> {
    iced_wgpu::wgpu::TexelCopyTextureInfo {
        texture: tex,
        mip_level: 0,
        origin: iced_wgpu::wgpu::Origin3d::ZERO,
        aspect: iced_wgpu::wgpu::TextureAspect::All,
    }
}

/// 门控 B：一帧 RGBA 上屏的**每帧代价**，三条通道对照（全部按 60 Hz 节奏
/// 节流并预热后取样，避免把 wgpu staging 的分配风暴误读成每帧代价）。
///
/// * **A 持久纹理 + `write_texture`**：最简实现，纹理建一次、原地更新。
/// * **B 每帧新纹理**（`Handle::from_rgba` 的等价物）：量出 iced 图像通道那条
///   路强加在帧流上的额外分配/销毁代价。
/// * **C 持久 staging buffer + `copy_buffer_to_texture`**（T-17 拟采用）：
///   staging 可持久映射，mpv 的 SW renderer **直接写进去**，于是「帧 → GPU」
///   只剩一次设备侧拷贝，既无 wgpu staging belt 的每次分配，也无中间缓冲 memcpy。
///
/// 无窗口也能建 wgpu 设备（无 surface），故不需要显示器。
fn probe_wgpu_upload() -> Result<(), String> {
    use iced_wgpu::wgpu;

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .map_err(|e| format!("request_adapter: {e:?}"))?;
    let info = adapter.get_info();
    eprintln!("wgpu adapter: {} ({:?}, {:?})", info.name, info.backend, info.device_type);
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("mpv-spike headless"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        ..Default::default()
    }))
    .map_err(|e| format!("request_device: {e:?}"))?;

    const N: usize = 120;
    /// 现实的帧间隔：24 fps 内容、播放器最多以 60 Hz 上屏。不按帧率节流的话，
    /// 120 帧 × 33 MB = 4 GB 会把 wgpu 的 staging belt 顶到爆炸，量出的 p95
    /// 是「内存分配风暴」而非每帧代价——那是测量方法的问题，不是结论。
    const FRAME_INTERVAL: std::time::Duration = std::time::Duration::from_micros(16_667);

    for (label, w, h) in [("1080p", 1920u32, 1080u32), ("4K", 3840u32, 2160u32)] {
        let bytes = w as usize * h as usize * 4;
        let pixels = vec![0x40u8; bytes];
        let extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let desc = |label: &'static str| wgpu::TextureDescriptor {
            label: Some(label),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };
        let layout = wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        };

        // ---- 通道 1：持久纹理，原地 write_texture（最简实现） ----
        let persistent = device.create_texture(&desc("video frame (persistent)"));
        let mut ms = Vec::with_capacity(N);
        for i in 0..N + 10 {
            let s = Instant::now();
            queue.write_texture(tex_copy_dest(&persistent), &pixels, layout, extent);
            let dt = s.elapsed().as_secs_f64() * 1000.0;
            if i >= 10 {
                ms.push(dt); // 前 10 帧为预热（staging 分配），不计入
            }
            pace(FRAME_INTERVAL);
        }
        let elapsed: f64 = ms.iter().sum::<f64>() / 1000.0;
        eprintln!(
            "[{label} A: persistent tex + write_texture ] {:.2} MB/frame | 每帧上限 {:.0} fps\n         {}",
            bytes as f64 / 1e6,
            ms.len() as f64 / elapsed,
            spread(&ms)
        );

        // ---- 通道 2：每帧新建纹理（`Handle::from_rgba` 的等价代价） ----
        let mut ms2 = Vec::with_capacity(N);
        for i in 0..N + 10 {
            let s = Instant::now();
            let tex = device.create_texture(&desc("video frame (per-frame)"));
            queue.write_texture(tex_copy_dest(&tex), &pixels, layout, extent);
            let dt = s.elapsed().as_secs_f64() * 1000.0;
            std::mem::drop(tex);
            if i >= 10 {
                ms2.push(dt);
            }
            pace(FRAME_INTERVAL);
        }
        let elapsed2: f64 = ms2.iter().sum::<f64>() / 1000.0;
        eprintln!(
            "[{label} B: 每帧新建纹理 (iced Handle 等价) ] 每帧上限 {:.0} fps\n         {}",
            ms2.len() as f64 / elapsed2,
            spread(&ms2)
        );

        // ---- 通道 3（T-17 拟采用）：持久 staging buffer + copy_buffer_to_texture ----
        // 关键收益：staging 可以是**持久映射**的缓冲，mpv 的 SW renderer 直接写进去
        // （`MPV_RENDER_PARAM_SW_POINTER`），于是「帧 → GPU」只剩一次设备侧拷贝，
        // 没有 wgpu staging belt 的每次分配，也没有我们自己的中间缓冲 memcpy。
        // 这里量的是**纯设备侧拷贝**的每帧代价（数据写入由 mpv 完成，不计入）。
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("video frame staging"),
            size: bytes as u64,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });
        let mut ms3 = Vec::with_capacity(N);
        for i in 0..N + 10 {
            let s = Instant::now();
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_texture(
                wgpu::TexelCopyBufferInfo {
                    buffer: &staging,
                    layout,
                },
                tex_copy_dest(&persistent),
                extent,
            );
            queue.submit([encoder.finish()]);
            let dt = s.elapsed().as_secs_f64() * 1000.0;
            if i >= 10 {
                ms3.push(dt);
            }
            pace(FRAME_INTERVAL);
        }
        let elapsed3: f64 = ms3.iter().sum::<f64>() / 1000.0;
        eprintln!(
            "[{label} C: staging buffer + copy_buffer_to_texture ] 每帧上限 {:.0} fps\n         {}",
            ms3.len() as f64 / elapsed3,
            spread(&ms3)
        );

        // staging 映射/反映射本身的代价（T-17 用它换掉中间缓冲 memcpy）。
        let map_ms = {
            let s = Instant::now();
            let slice = staging.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Write, move |r| {
                let _ = tx.send(r.map(|_| ()));
            });
            let _ = device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            });
            let _ = rx.recv();
            let dt = s.elapsed().as_secs_f64() * 1000.0;
            staging.unmap();
            dt
        };
        eprintln!("[{label}   map_async+unmap 往返 ] {map_ms:.3} ms（每次换缓冲的固定成本）");

        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
    }
    Ok(())
}

/// 把每帧的起点节流到 `interval`，模拟真实上屏节奏。
fn pace(interval: std::time::Duration) {
    let start = Instant::now();
    while start.elapsed() < interval {
        std::thread::yield_now();
    }
}

/// 极简 executor：spike 不想为此引入 pollster/futures 依赖。
fn block_on<F: std::future::Future>(f: F) -> F::Output {
    use std::task::{Context, Poll, Wake, Waker};
    struct Noop;
    impl Wake for Noop {
        fn wake(self: std::sync::Arc<Self>) {}
    }
    let waker = Waker::from(std::sync::Arc::new(Noop));
    let mut cx = Context::from_waker(&waker);
    let mut f = std::pin::pin!(f);
    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

// ───────────────────────── 门控入口 ─────────────────────────

/// 缺库时必须**优雅降级**（AC-20 的门控前置）。
fn selfcheck() {
    let explicit_missing = {
        // 不污染进程环境：显式解析一个不存在的路径。
        let p = std::path::PathBuf::from("Z:/definitely/not/here/libmpv-2.dll");
        p.is_file()
    };
    println!("[selfcheck] 不存在的显式路径解析为可用库？ {explicit_missing}（应为 false）");
    // SAFETY: 只加载一个不存在的路径，进程不崩即证明降级路径成立。
    match unsafe { libloading::Library::new("Z:/definitely/not/here/libmpv-2.dll") } {
        Ok(_) => println!("[selfcheck] FAIL: 加载不存在的库竟然成功"),
        Err(e) => println!("[selfcheck] 加载不存在的库 -> Err（降级路径）：{e}"),
    }
    match resolve_mpv_library() {
        Some(p) => println!("[selfcheck] 本机解析到 libmpv：{}", p.display()),
        None => println!(
            "[selfcheck] 本机无 libmpv（AUTO_MPV_LIB 未设且 exe 同目录无 libmpv-2.dll）\
             —— 这正是 AC-20 要求的降级入口，不是失败"
        ),
    }
}

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "all".to_string());
    println!("=== PLAN-617 T-15 VM 原生播放门控 spike | 子命令: {cmd} ===");

    if cmd == "selfcheck" || cmd == "all" {
        selfcheck();
    }

    if cmd == "mpv" || cmd == "all" {
        println!("\n--- 门控 A: libmpv 运行时加载 + SW 逐帧渲染代价 ---");
        match resolve_mpv_library() {
            None => eprintln!("SKIP: 本机无 libmpv（降级路径，见 selfcheck）"),
            Some(lib) => {
                if let Err(e) = probe_mpv_rendering(&lib) {
                    eprintln!("门控 A 失败: {e}");
                    std::process::exit(2);
                }
            }
        }
    }

    if cmd == "wgpu" || cmd == "all" {
        println!("\n--- 门控 B: 帧上屏每帧代价（持久纹理 vs 每帧新建） ---");
        if let Err(e) = probe_wgpu_upload() {
            eprintln!("门控 B 失败: {e}");
            std::process::exit(3);
        }
    }
    println!("\n=== done ===");
}
