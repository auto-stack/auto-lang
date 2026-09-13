//! PLAN-617 T-17：**帧上屏通道**的实测回归。
//!
//! 这个目标存在的意义是把 T-17 的验收条件变成可复跑的数字与像素断言：
//! ① 目标分辨率下的实测帧率与每帧上屏代价；② 「无闪烁」的可证伪形式；
//! ③ 背压时的丢帧策略与长尾有界性。
//!
//! 与 `mpv_engine.rs` 一样是集成目标（避开 lib `--test` 的 rustc ICE，债务 P617-D1），
//! 但要求更高一档的 feature：`mpv-gpu`（通道需要 wgpu）。
//!
//! 运行：
//! ```text
//! cargo test -p auto-lang --features mpv-gpu --test mpv_channel
//! # 有 libmpv + 样本时额外跑真实帧率与像素用例：
//! AUTO_MPV_LIB=<...>\libmpv-2.dll AUTO_SPIKE_VIDEO=<video> \
//!   cargo test -p auto-lang --features mpv-gpu --test mpv_channel -- --nocapture
//! ```

use auto_lang::ui::mpv::channel::{
    is_native_stride, stride_for, FrameOutcome, VideoFrameChannel, VideoLatestWins,
    GPU_ROW_ALIGN, RING_SLOTS,
};
use auto_lang::ui::mpv::engine::MpvEngine;
use auto_lang::ui::mpv::frame::REQUIRED_ALIGN;
use auto_lang::ui::mpv::loader::resolve_library;
use auto_lang::ui::mpv::present::{headless_device, VideoPresenter};
// 经通道取出 iced 用的那份 wgpu（测试目标不直接依赖 wgpu，避免版本漂移）。
use auto_lang::ui::mpv::channel::wgpu_reexport as wgpu;
use std::path::PathBuf;

/// 本机可用的 libmpv（未安装则 `None`）。
fn libmpv_path() -> Option<PathBuf> {
    resolve_library()
}

/// 样本视频（`AUTO_SPIKE_VIDEO`）；未设或不存在则 `None`。
fn sample_video() -> Option<PathBuf> {
    let p = PathBuf::from(std::env::var_os("AUTO_SPIKE_VIDEO")?);
    p.is_file().then_some(p)
}

/// 4K 行的样本：优先 `AUTO_SPIKE_VIDEO_4K`，否则退回 [`sample_video`]。
///
/// 分开的理由：若只有一个 1080p 样本，在 4K 通道尺寸上跑出来的数字只覆盖
/// **上屏链路**（mpv 把 1080p 放大到 4K），不覆盖 4K 解码。想得到真正的
/// 4K 端到端数字，就把 `AUTO_SPIKE_VIDEO_4K` 指向 4K 片源。这一步的差别
/// 是「4K 通道」与「4K 端到端」的差别，报告时不能混。
fn sample_video_4k() -> Option<PathBuf> {
    if let Some(v) = std::env::var_os("AUTO_SPIKE_VIDEO_4K") {
        let p = PathBuf::from(v);
        if p.is_file() {
            return Some(p);
        }
    }
    sample_video()
}

// ───────────────────────── 行距：两条对齐约束的交点 ─────────────────────────

/// 行距必须**同时**满足 wgpu（`bytes_per_row` 需 256 对齐）与 mpv（stride 需 64 对齐）。
/// 这是通道成立的前提，不是实现细节。
#[test]
fn stride_satisfies_both_wgpu_and_mpv_alignment() {
    assert_eq!(GPU_ROW_ALIGN, 256, "wgpu 的 copy_buffer_to_texture 行对齐是 256");
    for width in [1u32, 2, 159, 320, 640, 1919, 1920, 1921, 2560, 3840, 4096] {
        let s = stride_for(width);
        assert_eq!(s % GPU_ROW_ALIGN, 0, "width={width} 的 stride {s} 不满足 256 对齐");
        assert_eq!(s % REQUIRED_ALIGN, 0, "width={width} 的 stride {s} 不满足 mpv 的 64 对齐");
        assert!(
            s >= width as usize * 4,
            "width={width} 的 stride {s} 装不下该行"
        );
    }
}

/// 常见分辨率本来就落在 256 对齐上（无需 padding），这解释了为什么 1080p/4K 的
/// 实测数字里没有 padding 开销。
#[test]
fn common_widths_have_native_stride() {
    for width in [320u32, 640, 1280, 1920, 2560, 3840] {
        assert!(is_native_stride(width), "width={width} 应无需 padding");
        assert_eq!(stride_for(width), width as usize * 4);
    }
    // 反例：刻意制造不齐的宽度，必须被补齐而不是原样返回。
    assert!(!is_native_stride(318));
    assert!(stride_for(318) > 318 * 4);
}

// ───────────────────────── 最新优先门（镜像 MediaLatestWins） ─────────────────────────

/// 门控语义：只有 (generation, seq) 完全匹配才放行；不匹配即计一次丢帧。
#[test]
fn latest_wins_gate_accepts_only_the_current_pair() {
    let mut gate = VideoLatestWins::new(0, 0);
    assert!(gate.accept_enqueue(0, 0), "初始组合应放行");
    assert!(!gate.accept_enqueue(0, 1), "seq 未前进时的新序号应被拒");
    assert_eq!(gate.dropped(), 1, "被拒即计一次丢帧");

    gate.advance(1, 5);
    assert!(!gate.accept_publish(0, 0), "seek 后旧代际的上屏应被拒");
    assert!(gate.accept_publish(1, 5), "当前组合应放行");
    assert_eq!(gate.dropped(), 2);
    assert_eq!(gate.current(), (1, 5));
}

/// 通道要暴露的丢帧计数必须**分类**，否则「为什么丢帧」无从判断。
#[test]
fn frame_outcome_classifies_drops_vs_errors() {
    assert!(FrameOutcome::Submitted.is_submitted());
    for d in [
        FrameOutcome::DroppedStale,
        FrameOutcome::DroppedNoSlot,
        FrameOutcome::ReclaimTimeout,
    ] {
        assert!(d.is_deliberate_drop(), "{d:?} 是有意的丢帧策略");
        assert!(!d.is_submitted());
        assert!(!matches!(d, FrameOutcome::RenderFailed(_) | FrameOutcome::UploadFailed(_)));
    }
    assert!(!FrameOutcome::RenderFailed("x".into()).is_deliberate_drop());
    assert!(!FrameOutcome::UploadFailed("x".into()).is_submitted());
}

// ───────────────────────── 通道：结构性约束（无需 GPU 之外的依赖） ─────────────────────────

/// **硬约束：纹理只建一次**。跑若干帧后 `textures_created` 必须仍是 1——
/// 这正是 T-15 认定的闪烁成因（每帧新建纹理 → 缓存未命中 + 重上传与帧时钟竞争）。
#[test]
fn channel_creates_exactly_one_texture_ever() {
    let (device, queue) = match headless_device() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("SKIP: 无可用 wgpu 适配器（{e}）");
            return;
        }
    };
    let mut ch = VideoFrameChannel::new(&device, &queue, 320, 180);
    assert_eq!(ch.textures_created(), 1, "构造后应恰好 1 张纹理");

    // 走 30 帧（内容无关：只填像素，验证通道自身的形态不变）。
    for seq in 0..30u64 {
        ch.advance(0, seq);
        let out = ch.with_frame(0, seq, |target| {
            // SAFETY: target 由通道给出，指向已映射、长度足够的 staging 缓冲。
            unsafe {
                std::ptr::write_bytes(target.ptr, 0x40, target.stride * target.height as usize);
            }
            Ok(())
        });
        assert!(out.is_submitted(), "第 {seq} 帧应提交成功，实际 {out:?}");
    }
    assert_eq!(
        ch.textures_created(),
        1,
        "跑了 30 帧后纹理数仍必须是 1（每帧新建纹理就是闪烁成因）"
    );
    assert_eq!(ch.stats().submitted, 30);
    assert_eq!(ch.stats().dropped(), 0, "无背压时不应丢帧");
    assert_eq!(ch.view().texture(), ch.texture()); // 视图始终指向同一张纹理
}

/// 门控在**通道内部**生效：过期帧连渲染都不做，且被计入 `dropped_stale`。
#[test]
fn channel_drops_stale_frames_before_rendering() {
    let (device, queue) = match headless_device() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("SKIP: 无可用 wgpu 适配器（{e}）");
            return;
        }
    };
    let mut ch = VideoFrameChannel::new(&device, &queue, 64, 64);
    ch.advance(0, 0);
    let mut rendered = 0;
    let out = ch.with_frame(1, 0, |_| {
        rendered += 1;
        Ok(())
    });
    assert_eq!(out, FrameOutcome::DroppedStale);
    assert_eq!(rendered, 0, "过期帧不得进入渲染");
    assert_eq!(ch.stats().dropped_stale, 1);
}

/// **背压策略的安全性**：环被反复争用时，每一次 `with_frame` 都必须
/// ① 快速返回（有界），且 ② 结果只能是「已提交」或「按策略丢帧」——
/// 绝不允许是错误、更不允许无限期阻塞。
///
/// 关于「为什么不直接断言必然丢帧」：T-15 实测的那条长尾（4K 下 120 帧里 1 帧
/// `copy_buffer_to_texture` 耗时 957 ms）是**驱动偶发停顿**，无法按需复现；
/// 小尺寸拷贝毫秒级完成，环根本顶不满。所以这里断言的是**策略与上界**，
/// 而「一定丢帧」那样的断言只能靠运气通过。要守住的是：长尾发生时丢的是帧，
/// 不是界面。
#[test]
fn ring_never_blocks_and_only_yields_submitted_or_deliberate_drops() {
    let (device, queue) = match headless_device() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("SKIP: 无可用 wgpu 适配器（{e}）");
            return;
        }
    };
    let mut ch = VideoFrameChannel::new(&device, &queue, 640, 360);
    // 回收等待压到极小，模拟「GPU 被长尾卡住」的极端情形。
    ch.set_reclaim_timeout(std::time::Duration::from_millis(1));

    const N: u64 = 24; // = 8 × RING_SLOTS，确保回收路径被反复走到
    let mut drops = 0;
    let mut submitted = 0;
    let t0 = std::time::Instant::now();
    for seq in 0..N {
        ch.advance(0, seq);
        let out = ch.with_frame(0, seq, |target| {
            // SAFETY: 通道给出的映射缓冲，长度 >= stride*height。
            unsafe {
                std::ptr::write_bytes(target.ptr, 0x20, target.stride * target.height as usize);
            }
            Ok(())
        });
        match out {
            FrameOutcome::Submitted => submitted += 1,
            o if o.is_deliberate_drop() => drops += 1,
            other => panic!("背压下只允许「已提交」或「按策略丢帧」，实际 {other:?}"),
        }
    }
    let elapsed = t0.elapsed();

    assert_eq!(submitted + drops, N);
    assert!(submitted >= 1, "至少应有帧被提交");
    assert!(
        ch.stats().reclaim_waits > 0,
        "环只有 {RING_SLOTS} 槽而投了 {N} 帧，回收路径必须被走到（否则说明取槽逻辑没生效）"
    );
    // 有界性：每次调用最坏情形是「等回收超时」，上界即 槽数 × 超时 + 帧内工作量。
    let bound_ms = 1.0 * RING_SLOTS as f64 * N as f64 + 500.0;
    assert!(
        elapsed.as_secs_f64() * 1000.0 < bound_ms,
        "总耗时 {elapsed:?} 超出预期上界 {bound_ms:.0} ms——说明存在阻塞而非丢帧/快速回收"
    );
    eprintln!(
        "背压安全性：submitted={submitted} dropped={drops} 总耗时={elapsed:?}｜         回收等待 {} 次、超时 {} 次（环 {RING_SLOTS} 槽，超时 1 ms）",
        ch.stats().reclaim_waits,
        ch.stats().reclaim_timeouts,
    );
}

/// 长尾有界性：即便槽被在途提交占住且回收必然超时，`with_frame` 也必须在
/// 「超时 × 槽数」的量级内返回，绝不允许无限期挂住 UI 线程。
#[test]
fn reclaim_wait_is_bounded_by_timeout() {
    let (device, queue) = match headless_device() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("SKIP: 无可用 wgpu 适配器（{e}）");
            return;
        }
    };
    let mut ch = VideoFrameChannel::new(&device, &queue, 256, 256);
    ch.set_reclaim_timeout(std::time::Duration::from_millis(2));
    for seq in 0..RING_SLOTS as u64 {
        ch.advance(0, seq);
        let _ = ch.with_frame(0, seq, |t| {
            // SAFETY: 通道给出的映射缓冲。
            unsafe { std::ptr::write_bytes(t.ptr, 1, t.stride * t.height as usize) };
            Ok(())
        });
    }
    // 此刻槽全在途；下一帧要么拿到回收成功的槽，要么超时丢帧——两者都必须快速。
    ch.advance(0, RING_SLOTS as u64);
    let t0 = std::time::Instant::now();
    let out = ch.with_frame(0, RING_SLOTS as u64, |t| {
        // SAFETY: 同上。
        unsafe { std::ptr::write_bytes(t.ptr, 1, t.stride * t.height as usize) };
        Ok(())
    });
    let dt = t0.elapsed();
    assert!(out.is_submitted() || out.is_deliberate_drop(), "不得为错误：{out:?}");
    assert!(
        dt.as_secs_f64() < 1.0,
        "回收等待必须被超时界定（实际 {dt:?}），否则长尾会冻结界面"
    );
}

// ───────────────────────── 真实帧：帧率 + 像素级「无闪烁」 ─────────────────────────

/// 离屏目标 + 读回：把通道的纹理真的**画出来**并取回像素。
///
/// 这是「上屏」这一步的证据来源，也是「无闪烁」判定的输入。
fn render_and_readback(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    presenter: &mut VideoPresenter,
    channel: &VideoFrameChannel,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen present target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    // 读回缓冲的 bytes_per_row 同样必须 256 对齐。
    let row = stride_for(width);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("present readback"),
        size: (row * height as usize) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("present + readback"),
    });
    presenter.draw(device, &mut encoder, channel.view(), &view);
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row as u32),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    // map_async 的回调要跑完才拿得到数据；这里阻塞到完成。
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    let data = readback.slice(..).get_mapped_range().to_vec();
    readback.unmap();
    data
}

/// 一帧像素的「空白」判定与签名：用于识别闪烁（画面消失又出现）与陈旧帧。
fn frame_signature(pixels: &[u8]) -> (bool, u64) {
    // 只看 RGB，忽略 alpha（格式是 sRGB 且 alpha 恒为 1）。
    let mut non_black = 0usize;
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for px in pixels.chunks_exact(4).step_by(37) {
        let (r, g, b) = (px[0], px[1], px[2]);
        if r as u32 + g as u32 + b as u32 > 24 {
            non_black += 1;
        }
        for c in [r, g, b] {
            hash ^= c as u64;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
    }
    (non_black > 0, hash)
}

/// **T-17 的主证据**：用真实 mpv 解出的帧跑完整通道，测量
/// ① 目标分辨率下的可达帧率 ② 每帧上屏代价（含映射/解映射/提交）
/// ③ 像素级「无闪烁」——画面真的上了屏，且没有「空白帧夹在内容帧之间」。
#[test]
fn real_frames_upload_at_target_resolution_without_flicker() {
    if libmpv_path().is_none() {
        eprintln!("SKIP: 本机无 libmpv（设 AUTO_MPV_LIB 后重跑）");
        return;
    }
    let Some(video) = sample_video() else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO（需要一个真实视频文件）");
        return;
    };
    let (device, queue) = match headless_device() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("SKIP: 无可用 wgpu 适配器（{e}）");
            return;
        }
    };

    // 在**两个目标分辨率**上各跑一轮（1080p / 4K 的通道尺寸，取自片源典型值）。
    let rows: [(&str, u32, u32, usize, PathBuf); 2] = [
        ("1080p", 1920, 1080, 60, video.clone()),
        (
            "4K",
            3840,
            2160,
            30,
            sample_video_4k().unwrap_or_else(|| video.clone()),
        ),
    ];
    for (label, w, h, frames, src) in rows {
        let mut engine = match MpvEngine::new_with_options(&[
            ("vo", "libmpv"),
            ("audio", "no"),
            ("untimed", "yes"),
            ("terminal", "no"),
            ("msg-level", "all=error"),
        ]) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("SKIP: 构造引擎失败：{e}");
                return;
            }
        };
        if let Err(e) = engine.create_sw_render_context() {
            eprintln!("SKIP: 建 render context 失败：{e}");
            return;
        }
        if let Err(e) = engine.command(&["loadfile", &src.to_string_lossy()]) {
            eprintln!("SKIP: loadfile 失败：{e}");
            return;
        }
        let _ = engine.wait_first_frame_event(20.0);

        let mut ch = VideoFrameChannel::new(&device, &queue, w, h);
        let mut presenter = VideoPresenter::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

        // 离屏目标用小尺寸：像素证据只需证明「画面上了屏」，不必按片源尺寸读回。
        const OW: u32 = 256;
        const OH: u32 = 144;
        let mut signatures = Vec::with_capacity(frames);
        let mut blanks = Vec::with_capacity(frames);

        let t0 = std::time::Instant::now();
        let mut seq = 0u64;
        let mut submitted = 0usize;
        while submitted < frames && t0.elapsed().as_secs_f64() < 60.0 {
            if !engine.has_new_frame() {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }
            ch.advance(0, seq);
            let out = ch.with_frame(0, seq, |target| engine.render_sw_frame(&target));
            match out {
                FrameOutcome::Submitted => {
                    submitted += 1;
                    // 上屏 + 读回：证明纹理内容真的能被画出来。
                    let px = render_and_readback(&device, &queue, &mut presenter, &ch, OW, OH);
                    let (non_blank, sig) = frame_signature(&px);
                    blanks.push(!non_blank);
                    signatures.push(sig);
                }
                o if o.is_deliberate_drop() => { /* 允许（背压/长尾），计入统计 */ }
                o => panic!("通道出现错误类结果：{o:?}"),
            }
            seq += 1;
        }
        let elapsed = t0.elapsed();
        let st = ch.stats();

        eprintln!(
            "[{label} real] 通道 {w}x{h}（stride {}）源 {}| 提交 {submitted} 帧 / 上屏 {submitted} 帧，\
             用时 {:.2}s => {:.1} fps 端到端｜上屏代价 p50={:.2} ms p95={:.2} ms max={:.2} ms \
             （离群 {} 帧）｜丢帧 stale={} no_slot={} reclaim_timeout={} \
             ｜实际上屏时最多只建了 {} 张纹理",
            ch.stride(),
            src.file_name().map(|n| format!("{} ", n.to_string_lossy())).unwrap_or_default(),
            elapsed.as_secs_f64(),
            submitted as f64 / elapsed.as_secs_f64(),
            st.p50(),
            st.p95(),
            st.max(),
            st.outliers(),
            st.dropped_stale,
            st.dropped_no_slot,
            st.reclaim_timeouts,
            ch.textures_created(),
        );

        assert!(submitted >= frames / 2, "[{label}] 提交帧数过少（{submitted}）");
        assert_eq!(ch.textures_created(), 1, "[{label}] 纹理数必须恒为 1");
        assert_eq!(
            engine.api_version() >> 16,
            2,
            "[{label}] libmpv 主版本应为 2"
        );

        // ── 无闪烁（可证伪形式）──
        // ① 画面确实上了屏（否则是「presenter 坏了」而不是「无闪烁」）。
        let non_blank = blanks.iter().filter(|&&b| !b).count();
        assert!(
            non_blank > 0,
            "[{label}] 没有任何一帧画出内容——上屏通道没把画面送到目标上"
        );
        // ② 没有「空白帧夹在内容帧之间」——这正是 renderer.rs:2889 描述的闪烁
        //    （画面消失又出现）。跳过首个采样做预热，避免把片头的合法黑场误判。
        let warm = signatures.len().min(3);
        let mut isolated_blank = Vec::new();
        for i in warm..signatures.len().saturating_sub(1) {
            if blanks[i] && !blanks[i - 1] && !blanks[i + 1] {
                isolated_blank.push(i);
            }
        }
        assert!(
            isolated_blank.is_empty(),
            "[{label}] 出现「内容帧之间夹空白帧」= 闪烁签名，位置 {isolated_blank:?}"
        );
        // ③ 画面在推进：不是卡在同一张陈旧纹理上（稳定 id 命中缓存即不再上传那种）。
        let distinct: std::collections::HashSet<_> = signatures.iter().collect();
        assert!(
            distinct.len() >= 2,
            "[{label}] 所有帧签名相同——纹理没有更新（陈旧帧）"
        );
        // ④ 逐帧耗时分布应当存在（通道确实在记），且不应整体贴近超时上界。
        assert!(
            st.upload_ms.len() >= submitted,
            "[{label}] 逐帧耗时样本数应覆盖所有提交帧"
        );
    }
}
