//! PLAN-617 T-15 门控 spike 探针（**已改建在 T-16 的引擎之上**）。
//!
//! 本 example 是 T-15 那轮的**门控证据件**，现在复用它来复跑同一组数字：
//! 门控 A（libmpv 的逐帧渲染代价）改为直接使用 [`auto_lang::ui::mpv`]——
//! 也就是 T-16 落地的正式引擎（解析序 / LC_NUMERIC / 符号表 / 句柄与 render
//! context 生命周期都在那里），example 里不再重复一份 FFI 绑定。
//! 门控 B（帧上屏代价）仍留在本文件：它测的是 wgpu 侧，属 T-17 的输入。
//!
//! 数字与结论见 [PLAN-617 §9.14](../../docs/plans/617-030-video-player-real-rebuild.md)
//! 与 [Design 30 §4.2](../../docs/design/autoui/030-video-player.md)。
//!
//! # 运行
//!
//! ```text
//! # 需要本机有 libmpv（本仓不收录该二进制；取得方式见 scripts/mpv_spike/README.md）
//! set AUTO_MPV_LIB=D:\path\to\libmpv-2.dll
//! cargo run -p auto-lang --features mpv-spike --example mpv_spike -- all
//! cargo run -p auto-lang --features mpv-spike --example mpv_spike -- selfcheck
//! ```
//!
//! 子命令：`selfcheck`（缺库降级路径）/ `mpv`（逐帧渲染代价）/ `wgpu`（帧上传代价对照）/ `all`。

use std::time::Instant;

use auto_lang::ui::mpv::frame::FrameBuffer;
use auto_lang::ui::mpv::{locale, MpvEngine, MpvUnavailable};

/// 实测计时汇总。
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

fn percentile(xs: &[f64], q: f64) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((v.len() as f64 * q) as usize).min(v.len() - 1);
    v[idx]
}

/// 门控 A：逐帧渲染代价（`mpv_render_context_render()` 的纯缩放/色彩转换/写入）。
///
/// 走 T-16 引擎的**生产形状**：`ADVANCED_CONTROL` + `has_new_frame()`，
/// 只在 mpv 报告有新帧时才 render，故每次调用都是真实帧。
/// 采样 `AUTO_SPIKE_FRAMES`（默认 120）帧后给出分位。
fn probe_mpv_rendering() -> Result<(), MpvUnavailable> {
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
    let want: usize = std::env::var("AUTO_SPIKE_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120);

    let mut ran = 0usize;
    for (label, path, w, h) in samples {
        let path = std::path::PathBuf::from(path);
        if !path.is_file() {
            eprintln!("SKIP sample (缺失): {}", path.display());
            continue;
        }
        ran += 1;
        match render_one(&path, w, h, want) {
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
            Err(e) => eprintln!("[{label}] 构造失败：{e}"),
        }
    }
    if ran == 0 {
        eprintln!("注意：两个样本都不在本机 —— 只验证了加载与初始化路径");
    }
    Ok(())
}

/// 渲染 `want_frames` 帧并计时。
fn render_one(
    video: &std::path::Path,
    w: u32,
    h: u32,
    want_frames: usize,
) -> Result<Timings, MpvUnavailable> {
    // untimed：让 mpv 跑满以取吞吐上界；关音频（本探针只测帧管线）。
    let mut engine = MpvEngine::new_with_options(&[
        ("vo", "libmpv"),
        ("audio", "no"),
        ("untimed", "yes"),
        ("terminal", "no"),
        ("msg-level", "all=error"),
    ])?;
    // render context 必须在 loadfile 之前建（render.h:111）。
    engine.create_sw_render_context()?;

    if let Err(e) = engine.command(&["loadfile", &video.to_string_lossy()]) {
        eprintln!("loadfile 失败：{e}");
        return Ok(Timings::default());
    }
    // 等首个可渲染事件；拿到之后才可能有帧。
    let _ = engine.wait_first_frame_event(20.0);

    let mut fb = FrameBuffer::new(w, h);
    let t0 = Instant::now();
    let mut per_frame_ms = Vec::with_capacity(want_frames);
    while per_frame_ms.len() < want_frames {
        if !engine.has_new_frame() {
            if t0.elapsed().as_secs_f64() > 30.0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }
        let target = fb.as_target();
        let f0 = Instant::now();
        if let Err(e) = engine.render_sw_frame(&target) {
            eprintln!("render 失败：{e}");
            break;
        }
        per_frame_ms.push(f0.elapsed().as_secs_f64() * 1000.0);
    }
    Ok(Timings {
        frames: per_frame_ms.len(),
        elapsed_s: t0.elapsed().as_secs_f64(),
        per_frame_ms,
    })
}

// ───────────────────────── 帧上屏代价（T-17 的输入，仍在 spike 内） ─────────────────────────

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

/// 门控 B：一帧 RGBA 上屏的**每帧代价**，三条通道对照（按 60 Hz 节流并预热后取样）。
///
/// * **A 持久纹理 + `write_texture`**：最简实现。
/// * **B 每帧新纹理**（`Handle::from_rgba` 的等价物）：量出 iced 图像通道强加的分配代价。
/// * **C 持久 staging buffer + `copy_buffer_to_texture`**（T-17 拟采用）：
///   staging 可持久映射，mpv 的 SW renderer 直接写进去，故「帧 → GPU」只剩一次
///   设备侧拷贝。
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
    eprintln!(
        "wgpu adapter: {} ({:?}, {:?})",
        info.name, info.backend, info.device_type
    );
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
    // 现实的帧间隔：不节流的话 120 帧 × 33 MB 会把 wgpu 的 staging belt 顶爆，
    // 量出的 p95 是分配风暴而非每帧代价。
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

        // ---- 通道 A：持久纹理，原地 write_texture ----
        let persistent = device.create_texture(&desc("video frame (persistent)"));
        let mut ms = Vec::with_capacity(N);
        for i in 0..N + 10 {
            let s = Instant::now();
            queue.write_texture(tex_copy_dest(&persistent), &pixels, layout, extent);
            let dt = s.elapsed().as_secs_f64() * 1000.0;
            if i >= 10 {
                ms.push(dt); // 前 10 帧预热（staging 分配）
            }
            pace(FRAME_INTERVAL);
        }
        eprintln!(
            "[{label} A: persistent tex + write_texture ] {:.2} MB/frame | 每帧上限 {:.0} fps\n         {}",
            bytes as f64 / 1e6,
            ms.len() as f64 / (ms.iter().sum::<f64>() / 1000.0),
            spread(&ms)
        );

        // ---- 通道 B：每帧新建纹理（`Handle::from_rgba` 的等价代价） ----
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
        eprintln!(
            "[{label} B: 每帧新建纹理 (iced Handle 等价) ] 每帧上限 {:.0} fps\n         {}",
            ms2.len() as f64 / (ms2.iter().sum::<f64>() / 1000.0),
            spread(&ms2)
        );

        // ---- 通道 C（T-17 拟采用）：持久 staging + copy_buffer_to_texture ----
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
        eprintln!(
            "[{label} C: staging buffer + copy_buffer_to_texture ] 每帧上限 {:.0} fps\n         {}",
            ms3.len() as f64 / (ms3.iter().sum::<f64>() / 1000.0),
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
    match auto_lang::ui::mpv::loader::resolve_library() {
        Some(p) => println!("[selfcheck] 本机解析到 libmpv：{}", p.display()),
        None => println!(
            "[selfcheck] 本机无 libmpv（AUTO_MPV_LIB 未设且 exe 同目录无运行库）\
             —— 这正是 AC-20 要求的降级入口，不是失败"
        ),
    }
    println!(
        "[selfcheck] LC_NUMERIC 当前 = {:?}（mpv 要求 \"C\"）",
        locale::numeric_locale()
    );
    match MpvEngine::new() {
        Ok(e) => println!("[selfcheck] 引擎可用：{e:?}"),
        Err(MpvUnavailable::NoLibrary) => println!("[selfcheck] 引擎降级：NoLibrary（预期路径）"),
        Err(e) => println!("[selfcheck] 引擎不可用：{e}"),
    }
    // 不存在的库必须返回 Err 而不是崩。
    match auto_lang::ui::mpv::MpvApi::load(std::path::Path::new(
        r"Z:\definitely\not\here\libmpv-2.dll",
    )) {
        Ok(_) => println!("[selfcheck] FAIL: 加载不存在的库竟然成功"),
        Err(e) => println!("[selfcheck] 加载不存在的库 -> Err（降级路径）：{e}"),
    }
}

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "all".to_string());
    println!("=== PLAN-617 T-15 门控 spike（建于 T-16 引擎之上）| 子命令: {cmd} ===");

    if cmd == "selfcheck" || cmd == "all" {
        selfcheck();
    }

    if cmd == "mpv" || cmd == "all" {
        println!("\n--- 门控 A: libmpv 逐帧渲染代价 ---");
        match probe_mpv_rendering() {
            Ok(()) => {}
            Err(MpvUnavailable::NoLibrary) => {
                eprintln!("SKIP: 本机无 libmpv（降级路径，见 selfcheck）")
            }
            Err(e) => {
                eprintln!("门控 A 失败: {e}");
                std::process::exit(2);
            }
        }
    }

    if cmd == "wgpu" || cmd == "all" {
        println!("\n--- 门控 B: 帧上屏每帧代价（持久纹理 vs 每帧新建 vs staging） ---");
        if let Err(e) = probe_wgpu_upload() {
            eprintln!("门控 B 失败: {e}");
            std::process::exit(3);
        }
    }
    println!("\n=== done ===");
}
