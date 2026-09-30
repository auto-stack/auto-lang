//! PLAN-617 T-16：原生播放引擎的**降级路径**与**生命周期**回归。
//!
//! 这是集成测试目标而非 lib 单测，原因是构建面：`mpv-native` 特性挂在 lib 的
//! `--test` 目标上会触发 rustc ICE（债务 P617-D1，查询栈在 `ui/mcp_server.rs`，
//! 与 mpv 代码无关）。集成测试把 lib 编成 rlib，避开该路径。
//!
//! 大多数用例**不需要**本机装 mpv——它们断言的正是「没有 mpv 时也一切正常」。
//! 只有真正要与 DLL 对话的用例会在 [`libmpv_path`] 返回 `None` 时自行跳过。
//!
//! 运行：
//! ```text
//! cargo test -p auto-lang --features mpv-native --test mpv_engine
//! # 有 libmpv 时（额外跑真实生命周期用例）：
//! AUTO_MPV_LIB=<...>\libmpv-2.dll cargo test -p auto-lang --features mpv-native --test mpv_engine
//! ```

use auto_lang::ui::mpv::engine::{MpvEngine, MpvUnavailable};
use auto_lang::ui::mpv::frame::{FrameBuffer, REQUIRED_ALIGN};
use auto_lang::ui::mpv::loader::{
    resolve_library, resolve_library_with, MpvApi, MpvLoadError, LIB_NAME,
};
use auto_lang::ui::mpv::locale;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// 本机可用的 libmpv 路径（未安装则 `None`）。
fn libmpv_path() -> Option<PathBuf> {
    resolve_library()
}

fn bogus_path() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"Z:\definitely\not\here\libmpv-2.dll")
    } else {
        PathBuf::from("/definitely/not/here/libmpv.so.2")
    }
}

// ───────────────────────── 解析序（纯函数，不碰进程环境） ─────────────────────────

/// 显式指定的路径**存在**时优先采用。
#[test]
fn resolve_prefers_explicit_env_path_when_it_exists() {
    // 用当前可执行文件充当一个「确实存在」的文件——解析序只看 is_file。
    let me = std::env::current_exe().expect("current_exe");
    let got = resolve_library_with(Some(me.as_os_str()), Some(Path::new(r"Z:\nope")));
    assert_eq!(got.as_deref(), Some(me.as_path()));
}

/// **关键降级语义**：显式指定了却不存在 → `None`，**不再回落**到 exe 同目录。
///
/// 否则「路径指错了」会表现为「莫名用了另一个版本的运行库」，比直接降级更难排查。
#[test]
fn resolve_explicit_missing_path_does_not_fall_back() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    // 先证明 exe 同目录这条回落在「env 缺席」时是可能生效的（哪怕本机没放 DLL，
    // 也只是 None），再证明 env 存在但无效时不会再考虑它。
    let with_env = resolve_library_with(Some(OsStr::new("Z:/nope/libmpv-2.dll")), exe_dir.as_deref());
    assert!(
        with_env.is_none(),
        "显式路径无效时必须直接降级为 None，不得回落到 exe 同目录"
    );
}

/// 没有任何线索 → `None`（「本机没装 mpv」是正常分支，不是错误）。
#[test]
fn resolve_without_any_hint_is_none() {
    assert!(resolve_library_with(None, None).is_none());
    // 传入一个**空目录**（用临时目录保证不含 LIB_NAME）同样应为 None。
    let dir = std::env::temp_dir().join("auto-lang-mpv-resolve-probe");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::remove_file(dir.join(LIB_NAME));
    assert!(resolve_library_with(None, Some(&dir)).is_none());
}

// ───────────────────────── 加载失败必须可报、不 panic ─────────────────────────

/// 对不存在的库做真实 `LoadLibrary` → `Err`，进程不崩、错误带路径。
#[test]
fn loading_a_missing_library_reports_error_instead_of_panicking() {
    let p = bogus_path();
    match MpvApi::load(&p) {
        Ok(_) => panic!("加载不存在的库竟然成功了：{}", p.display()),
        Err(MpvLoadError::LoadFailed { path, detail }) => {
            assert_eq!(path, p, "错误里必须带上是哪个路径失败");
            assert!(!detail.is_empty(), "错误详情不能为空");
        }
        Err(other) => panic!("不存在的库应当是 LoadFailed，实际：{other}"),
    }
}

/// 缺库时构造引擎必须是**可处理的 `Err`**（而不是 panic），且原因可判定为
/// `NoLibrary` 或 `LoadFailed`——渲染层据此走诚实降级（AC-20）。
///
/// 本机装了 mpv 时这条也会成立（那时会真的构造成功），故不断言方向，只断言
/// 「要么成功、要么是一个能说清原因的 Err」。
#[test]
fn engine_construction_never_panics_without_a_library() {
    match MpvEngine::new() {
        Ok(engine) => {
            // 本机确实装了 mpv：顺手验证拿到的元信息自洽。
            assert!(engine.api_version() > 0, "client API 版本应非零");
            assert!(engine.library_path().is_file());
            assert!(!engine.has_render_context(), "刚构造时不应有 render context");
        }
        Err(MpvUnavailable::NoLibrary) => { /* 预期中的降级路径 */ }
        Err(MpvUnavailable::LoadFailed(e)) => {
            panic!("找到了运行库却加载失败，应排查而非静默降级：{e}")
        }
        Err(other) => panic!("非预期的失败分类：{other}"),
    }
}

/// `is_available()` 必须与「解析得到路径」一致（廉价探测，供渲染层决定降级）。
#[test]
fn is_available_matches_actual_resolution() {
    assert_eq!(MpvEngine::is_available(), libmpv_path().is_some());
}

// ───────────────────────── LC_NUMERIC=C（mpv 的 C 环境前提） ─────────────────────────

/// mpv 要求 `LC_NUMERIC` 为 `"C"`（client.h:147），否则 `mpv_create()` 可能返回
/// NULL。守卫必须是幂等的、且真的把值设成 `"C"`。
#[test]
fn c_numeric_locale_guard_is_idempotent_and_effective() {
    assert!(locale::ensure_c_numeric_locale(), "设置 LC_NUMERIC=C 不应失败");
    assert!(
        locale::numeric_locale_is_c(),
        "设置后应读回 \"C\"，实际 {:?}",
        locale::numeric_locale()
    );
    // 幂等：再设一次仍然成立。
    assert!(locale::ensure_c_numeric_locale());
    assert!(locale::numeric_locale_is_c());
}

// ───────────────────────── 真实生命周期（有 DLL 才跑） ─────────────────────────

/// **T-16 的核心断言**：句柄 + render context 的完整生命周期，顺序正确、可反复
/// 建拆。顺序错（先销毁 core 再 free render context）是 UB（render.h:119），
/// 故这里刻意构造「建 context → 用一帧 → Drop」的完整路径。
#[test]
fn real_render_context_lifecycle_when_libmpv_is_present() {
    if libmpv_path().is_none() {
        eprintln!(
            "SKIP: 本机无 libmpv（设 AUTO_MPV_LIB 或把 {LIB_NAME} 放到 exe 同目录后重跑）"
        );
        return;
    }

    // 建引擎（内部已做 locale 守卫 + 选项下发 + initialize）。
    let mut engine = MpvEngine::new().expect("有 libmpv 时构造引擎应成功");
    assert!(engine.api_version() > 0);
    eprintln!(
        "libmpv: {} (client API {}.{})",
        engine.library_path().display(),
        engine.api_version() >> 16,
        engine.api_version() & 0xFFFF
    );

    // render context 必须在 loadfile **之前**建（render.h:111：否则 video 初始化
    // 会失败或退回自建窗口 VO）。
    engine
        .create_sw_render_context()
        .expect("SW render context 应创建成功");
    assert!(engine.has_render_context());

    // 一个 core 同时只允许 1 个 render context（render.h:114）。
    let dup = engine.create_sw_render_context();
    assert!(dup.is_err(), "重复创建 render context 必须被拒绝");

    // Drop 走「先 free context 再 terminate_destroy」——若顺序写反这里就是 UB，
    // 本用例存在的意义就是让这条路径在真实 DLL 上被执行一次。
    drop(engine);

    // 再建一次：证明释放干净、可反复建拆（没有把全局状态弄坏）。
    let mut again = MpvEngine::new().expect("第二次构造也应成功");
    again
        .create_sw_render_context()
        .expect("第二次创建 render context 也应成功");
    drop(again);
}

/// 真实渲染一帧到 CPU 缓冲：验证 ① 参数表构造正确、② 目标缓冲真被写入、
/// ③ 64 字节对齐的 stride 被接受。
///
/// 这是 T-17 帧通道的地基；本用例只走「mpv → 我们的内存」这一段，不涉及 GPU。
#[test]
fn real_sw_render_writes_into_our_buffer_when_libmpv_is_present() {
    let Some(lib) = libmpv_path() else {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    };
    // 样本：本仓无视频资产，故用环境变量指定；缺样本则跳过（不是失败）。
    let Some(video) = std::env::var_os("AUTO_SPIKE_VIDEO").map(PathBuf::from) else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO（需要一个真实视频文件），libmpv={}", lib.display());
        return;
    };
    if !video.is_file() {
        eprintln!("SKIP: AUTO_SPIKE_VIDEO 指向的文件不存在：{}", video.display());
        return;
    }

    let mut engine = MpvEngine::new().expect("构造引擎");
    engine.create_sw_render_context().expect("建 render context");
    engine
        .command(&["loadfile", &video.to_string_lossy()])
        .expect("loadfile 应被接受");

    let waited = engine
        .wait_first_frame_event(20.0)
        .expect("等待首帧事件不应报错");
    assert!(waited, "20s 内应等到 FILE_LOADED / VIDEO_RECONFIG");

    const W: u32 = 320;
    const H: u32 = 180;

    // 目标缓冲必须走 FrameBuffer：`Vec<u8>` 的自然对齐只有 1（实测本机 16），
    // 直接把它交给 mpv 会静默掉进「整帧拷贝」的慢路径（render.h:393-404）。
    let mut fb = FrameBuffer::new(W, H);
    assert!(
        fb.base_is_aligned(),
        "FrameBuffer 的基址必须满足 mpv 的 64 字节对齐要求，实际 mod 64 = {}",
        fb.as_slice().as_ptr() as usize % REQUIRED_ALIGN
    );
    assert_eq!(
        fb.stride() % REQUIRED_ALIGN,
        0,
        "stride 也必须是 {REQUIRED_ALIGN} 的倍数"
    );
    assert!(
        !fb.has_content(),
        "尚未渲染的缓冲应当是全零（否则下面的「有内容」断言无意义）"
    );

    // 等到真的有帧可渲染再画（无帧时 render 可能直接报错）。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let mut rendered = false;
    while std::time::Instant::now() < deadline {
        if engine.has_new_frame() {
            let target = fb.as_target();
            engine.render_sw_frame(&target).expect("渲染一帧应成功");
            rendered = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(rendered, "20s 内应至少渲染出一帧");

    // 画面应当真的写进来了（不是全零缓冲）。
    assert!(
        fb.has_content(),
        "渲染后缓冲不应仍全为 0——说明帧确实写进了我们的内存"
    );
}

/// PLAN-712 **T-02/DP-3 探针**：dump libmpv 对样本视频的**色彩协商结果**——
/// `video-params`（解码面）与 `video-out-params`（渲染输出面）的 matrix/range/
/// primaries/gamma，以及影响 SW 转换的选项现值。VM 端上屏字节 ≡ SW rgb0 输出
/// 字节（目标/纹理双 Srgb + shader 直通，见 T-01 结论），故两终端（Web
/// Chromium vs VM mpv）的色度差**只能**来自这里的协商差异。
///
/// 决策工件而非门禁：无 DLL / 无样本时静默跳过。运行：
/// ```text
/// AUTO_MPV_LIB=<...>\libmpv-2.dll AUTO_SPIKE_VIDEO=E:\Video\caelestia.mp4 \
///   cargo test -p auto-lang --features mpv-native --test mpv_engine \
///   probe_color_negotiation -- --nocapture
/// ```
#[test]
fn probe_color_negotiation_dump() {
    let Some(lib) = libmpv_path() else {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    };
    let Some(video) = std::env::var_os("AUTO_SPIKE_VIDEO").map(PathBuf::from) else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO");
        return;
    };
    if !video.is_file() {
        eprintln!("SKIP: AUTO_SPIKE_VIDEO 不存在：{}", video.display());
        return;
    }

    let mut engine = MpvEngine::new().expect("构造引擎");
    engine.create_sw_render_context().expect("建 render context");
    engine
        .set_flag("pause", true)
        .expect("pause 下发应成功（首帧后仍可查询参数）");
    engine
        .command(&["loadfile", &video.to_string_lossy()])
        .expect("loadfile 应被接受");
    assert!(
        engine
            .wait_first_frame_event(20.0)
            .expect("等待首帧事件不应报错"),
        "20s 内应等到首帧"
    );

    // 解码面：视频自己声明的（或缺席时 mpv 的猜测依据）。
    eprintln!("== video-params（解码面） ==");
    for key in [
        "pixelformat",
        "colormatrix",
        "colorlevels",
        "primaries",
        "gamma",
        "sig-peak",
    ] {
        eprintln!("  {key} = {:?}", engine.get_string(&format!("video-params/{key}")));
    }
    // 渲染输出面：SW rgb0 实际交出的参数（= 我们纹理里的字节语义）。
    eprintln!("== video-out-params（渲染输出面） ==");
    for key in [
        "pixelformat",
        "colormatrix",
        "colorlevels",
        "primaries",
        "gamma",
    ] {
        eprintln!(
            "  {key} = {:?}",
            engine.get_string(&format!("video-out-params/{key}"))
        );
    }
    // 影响 SW 转换的选项现值（协商可调面）。
    eprintln!("== 转换选项 ==");
    for key in [
        "sws-scaler",
        "sws-allow-zimg",
        "video-output-levels",
        "target-prim",
        "target-trc",
    ] {
        eprintln!("  {key} = {:?}", engine.get_string(key));
    }
}

/// PLAN-712 **T-02/DP-3 差分探针**：同一帧在「默认协商」与「pre-init 显式协商」
/// 下的 SW rgb0 输出是否不同——即 SW 转换是否吃 `target-prim`/`target-trc`
/// （运行时 set 已证无效：`unsupported format for accessing property`，且
/// `video-output-levels=full` 运行时设后输出**逐位不变**）。各配置独立建引擎
/// （target-* 是 pre-init 选项面），seek 同一关键帧取帧。决策工件而非门禁。
#[test]
fn probe_color_output_tunable() {
    let Some(lib) = libmpv_path() else {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    };
    let Some(video) = std::env::var_os("AUTO_SPIKE_VIDEO").map(PathBuf::from) else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO");
        return;
    };
    if !video.is_file() {
        eprintln!("SKIP: AUTO_SPIKE_VIDEO 不存在：{}", video.display());
        return;
    }

    // seek 到 0.5s 关键帧并渲染一帧，返回全帧均值（降采样步进防大帧超时）。
    fn mean_at(video: &str, options: &[(&str, &str)]) -> [f64; 3] {
        // 与 MpvEngine::new() 同款默认前置（vo=libmpv 等），再叠加探针配置——
        // 空表会把 vo=libmpv 也丢掉，帧面整个不成立。
        let mut all: Vec<(&str, &str)> = vec![
            ("vo", "libmpv"),
            ("terminal", "no"),
            ("msg-level", "all=error"),
        ];
        all.extend_from_slice(options);
        let mut engine = MpvEngine::new_with_options(&all).expect("构造引擎");
        engine.create_sw_render_context().expect("建 render context");
        engine.set_flag("pause", true).expect("pause");
        engine.command(&["loadfile", video]).expect("loadfile");
        assert!(
            engine.wait_first_frame_event(20.0).expect("等待首帧"),
            "首帧超时"
        );
        // 先消费掉首帧的 UPDATE_FRAME 位（渲染一次），后续 seek 的新帧位才是
        // 真信号——首帧位未消费时会立即「假就绪」。
        {
            let mut warm = FrameBuffer::new(64, 64);
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            loop {
                if engine.has_new_frame() {
                    engine.render_sw_frame(&warm.as_target()).expect("warm 渲染");
                    break;
                }
                assert!(std::time::Instant::now() < deadline, "warm 无新帧");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        engine
            .command(&["seek", "0.5", "absolute+keyframes"])
            .expect("seek");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            if engine.has_new_frame() {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "seek 后无新帧");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        eprintln!(
            "    seek 后 time-pos={:?}",
            engine.get_f64("time-pos")
        );

        const W: u32 = 640;
        const H: u32 = 360;
        let mut fb = FrameBuffer::new(W, H);
        engine.render_sw_frame(&fb.as_target()).expect("渲染一帧");
        let buf = fb.as_slice();
        let stride = fb.stride() as usize;
        let mut acc = [0f64; 3];
        let mut n = 0u64;
        for y in (0..H as usize).step_by(7) {
            for x in (0..W as usize).step_by(7) {
                let px = &buf[y * stride + x * 4..];
                acc[0] += px[0] as f64;
                acc[1] += px[1] as f64;
                acc[2] += px[2] as f64;
                n += 1;
            }
        }
        eprintln!(
            "    out gamma={:?} levels={:?} matrix={:?}",
            engine.get_string("video-out-params/gamma"),
            engine.get_string("video-out-params/colorlevels"),
            engine.get_string("video-out-params/colormatrix")
        );
        [acc[0] / n as f64, acc[1] / n as f64, acc[2] / n as f64]
    }

    let v = video.to_string_lossy().into_owned();

    let baseline = mean_at(&v, &[]);
    eprintln!("[1] baseline（默认协商）              mean RGB = {baseline:.2?}");

    let srgb = mean_at(&v, &[("target-prim", "srgb"), ("target-trc", "srgb")]);
    eprintln!("[2] pre-init target-prim/trc=srgb     mean RGB = {srgb:.2?}");

    let full = mean_at(&v, &[("video-output-levels", "full")]);
    eprintln!("[3] pre-init video-output-levels=full mean RGB = {full:.2?}");

    eprintln!(
        "判读：[2]/[3] 与 [1] 的均值差即 SW 转换是否吃该协商面的直接证据；lib={}",
        lib.display()
    );
}
