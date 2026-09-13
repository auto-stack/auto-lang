//! PLAN-617 T-18：**受控媒体契约**在 mpv 侧的实测回归。
//!
//! 断言的是 §2.3 那套契约真的作用于播放内核（不是「看起来像」）：
//! 起播后时间前进、seek 落到目标、音量/静音/倍速可读可写、上行事件按契约形态回灌。
//!
//! 需要 libmpv（`AUTO_MPV_LIB` 或 exe 同目录）与一个真实片源（`AUTO_SPIKE_VIDEO`）；
//! 缺任一则整体 SKIP——**降级是正常路径**（AC-20）。
//!
//! 运行：
//! ```text
//! AUTO_MPV_LIB=<...>\libmpv-2.dll AUTO_SPIKE_VIDEO=<video> \
//!   cargo test -p auto-lang --features mpv-native --test mpv_contract -- --nocapture
//! ```

use auto_lang::ui::mpv::contract::{
    volume_from_element, volume_to_element, MediaContract, VideoContractDown, VideoContractEvent,
};
use auto_lang::ui::mpv::engine::MpvEngine;
use auto_lang::ui::mpv::loader::resolve_library;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn libmpv_path() -> Option<PathBuf> {
    resolve_library()
}

fn sample_video() -> Option<PathBuf> {
    let p = PathBuf::from(std::env::var_os("AUTO_SPIKE_VIDEO")?);
    p.is_file().then_some(p)
}

/// 契约用的引擎：**`ao=null`** 提供 mpv 的时钟。
///
/// 这一点很关键：T-15 实测过，没有音频输出（`audio=no`）时 mpv **不按实时节奏走**
/// （会自由跑，媒体时钟 30×）。`ao=null` 给了一个「有节奏但不出声」的时钟，
/// 正好适合断言「时间在前进」而不惊动扬声器。真实运行时用默认 ao（见音频用例）。
fn contract_engine() -> Option<MpvEngine> {
    MpvEngine::new_with_options(&[
        ("vo", "libmpv"),
        ("ao", "null"),
        ("terminal", "no"),
        ("msg-level", "all=error"),
    ])
    .ok()
}

/// 轮询直到 `pred` 为真或超时；返回期间收到的全部契约事件。
fn poll_until(
    engine: &MpvEngine,
    contract: &mut MediaContract,
    timeout: Duration,
    mut pred: impl FnMut(&[VideoContractEvent], &MediaContract) -> bool,
) -> Vec<VideoContractEvent> {
    let deadline = Instant::now() + timeout;
    let mut all = Vec::new();
    while Instant::now() < deadline {
        let evs = contract.poll(engine);
        all.extend(evs.iter().cloned());
        if pred(&evs, contract) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    all
}

/// 加载并等到 `LoadedMetadata`。**返回期间收到的全部事件**——`poll` 是「取出即消费」
/// 的语义，所以关心某个事件的用例必须从返回值里看，事后再 poll 是看不到的
/// （最初写错的正是这一点）。
fn load(engine: &MpvEngine, contract: &mut MediaContract, src: &std::path::Path) -> Vec<VideoContractEvent> {
    let mut down = VideoContractDown::default();
    down.src = Some(src.to_string_lossy().into_owned());
    contract.apply(engine, &down);
    // 等「时长可知」——等价于浏览器的 loadedmetadata，之后 seek/时长才有意义。
    let evs = poll_until(engine, contract, Duration::from_secs(15), |evs, _| {
        evs.iter()
            .any(|e| matches!(e, VideoContractEvent::LoadedMetadata(_)))
    });
    assert!(
        evs.iter()
            .any(|e| matches!(e, VideoContractEvent::LoadedMetadata(_))),
        "15s 内应收到 LoadedMetadata（片源是否可解？）"
    );
    assert!(
        contract.media_error().is_none(),
        "加载不应报错：{:?}",
        contract.media_error()
    );
    evs
}

// ───────────────────────── 单位与默认值（不需要 mpv） ─────────────────────────

/// `volume` 的共享边界是**作者面的 0..100**；Vue 侧把它翻成元素的 0..1。
/// 这条换算必须有单一出处，否则两端会各写一遍并逐渐漂移。
#[test]
fn volume_unit_conversion_is_pinned() {
    assert_eq!(volume_to_element(0), 0.0);
    assert_eq!(volume_to_element(100), 1.0);
    assert_eq!(volume_to_element(50), 0.5);
    // 越界一律夹紧，不产生 >1 的元素音量（浏览器会拒）或负值。
    assert_eq!(volume_to_element(150), 1.0);
    assert_eq!(volume_to_element(-20), 0.0);
    // 往返一致（0..100 内无信息损失）。
    for v in [0, 1, 37, 50, 99, 100] {
        assert_eq!(volume_from_element(volume_to_element(v)), v);
    }
}

/// 默认下行必须是「能播但不自动播」，且 `rate` **必须**是 1.0：
/// `f64::default() == 0.0` 会被 mpv 当作 0 倍速把播放冻住——这是 derive(Default)
/// 会引入的一个真实坑，故这里用测试钉住。
#[test]
fn default_down_is_playable_and_rate_is_one() {
    let d = VideoContractDown::default();
    assert_eq!(d.rate, 1.0, "rate 默认必须是 1.0（0.0 会冻住播放）");
    assert_eq!(d.volume, 100);
    assert!(!d.muted);
    assert!(d.paused, "默认应处于暂停（由 is_playing=false 推导）");
    assert!(d.src.is_none());
    assert!(d.position.is_none());
}

// ───────────────────────── 真实播放：起播 / seek / 音量 ─────────────────────────

/// **起播**：把 `paused` 置 false 后，契约应回灌 `PlayStateChange(true)`，
/// 且 `time-pos` 真的前进（这是 AC-19「真实播放」的内核级证据）。
#[test]
fn real_playback_advances_time_and_reports_play_state() {
    if libmpv_path().is_none() {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    }
    let Some(video) = sample_video() else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO");
        return;
    };
    let Some(engine) = contract_engine() else {
        eprintln!("SKIP: 引擎构造失败");
        return;
    };
    let mut contract = MediaContract::new();
    let _ = load(&engine, &mut contract, &video);

    let mut down = VideoContractDown::default();
    down.src = Some(video.to_string_lossy().into_owned());
    down.paused = false; // ← 作者写 `paused: .is_playing == false`
    // 注意：下行必须真的 apply 下去；只构造 `down` 不会改变播放内核状态。
    contract.apply(&engine, &down);

    // 起播并等待时间前进。
    let evs = poll_until(&engine, &mut contract, Duration::from_secs(10), |evs, _| {
        evs.iter()
            .any(|e| matches!(e, VideoContractEvent::TimeUpdate(t) if *t > 0.05))
    });
    assert!(
        evs.iter().any(|e| matches!(e, VideoContractEvent::PlayStateChange(true))),
        "起播应回灌 PlayStateChange(true)，实际事件：{evs:?}"
    );
    let first_time = evs
        .iter()
        .find_map(|e| match e {
            VideoContractEvent::TimeUpdate(t) => Some(*t),
            _ => None,
        })
        .expect("应有 TimeUpdate");
    assert!(first_time > 0.0, "起播后时间应前进，实际 {first_time}");

    // 再跑一会儿，必须继续前进（不是卡在某一帧）。
    let evs2 = poll_until(&engine, &mut contract, Duration::from_secs(10), |evs, _| {
        evs.iter()
            .any(|e| matches!(e, VideoContractEvent::TimeUpdate(t) if *t > first_time + 0.2))
    });
    let advanced = evs2.iter().any(|e| matches!(e, VideoContractEvent::TimeUpdate(t) if *t > first_time + 0.2));
    assert!(advanced, "时间应持续前进（first={first_time}）");

    // 暂停：状态回灌 false，且时间不再前进。
    down.paused = true;
    contract.apply(&engine, &down);
    let paused_events = poll_until(&engine, &mut contract, Duration::from_secs(5), |evs, _| {
        evs.iter().any(|e| matches!(e, VideoContractEvent::PlayStateChange(false)))
    });
    assert!(
        paused_events.iter().any(|e| matches!(e, VideoContractEvent::PlayStateChange(false))),
        "暂停应回灌 PlayStateChange(false)"
    );
    let t_before = engine.get_f64("time-pos").unwrap_or(0.0);
    std::thread::sleep(Duration::from_millis(600));
    let t_after = engine.get_f64("time-pos").unwrap_or(0.0);
    assert!(
        (t_after - t_before).abs() < 0.2,
        "暂停后时间不应继续前进（{t_before} → {t_after}）"
    );
}

/// **seek**：把 `position` 设到目标后，`time-pos` 应落在目标附近，
/// 且 `generation` 前进（供 T-17 的 `VideoLatestWins` 作废在途旧帧）。
#[test]
fn real_seek_lands_and_advances_generation() {
    if libmpv_path().is_none() {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    }
    let Some(video) = sample_video() else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO");
        return;
    };
    let Some(engine) = contract_engine() else {
        eprintln!("SKIP: 引擎构造失败");
        return;
    };
    let mut contract = MediaContract::new();
    let _ = load(&engine, &mut contract, &video);

    let duration = engine.get_f64("duration").expect("应有时长");
    let target = (duration * 0.4).clamp(1.0, duration - 1.0);

    let generation_before = contract.generation();
    let mut down = VideoContractDown::default();
    down.src = Some(video.to_string_lossy().into_owned());
    down.paused = false;
    down.position = Some(target);
    let ops = contract.apply(&engine, &down);
    assert!(ops > 0, "seek 应产生实际下发");
    assert_eq!(
        contract.generation(),
        generation_before + 1,
        "seek 必须让代际前进（否则 T-17 会在途旧帧会盖掉新位置的画面）"
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut landed = None;
    while Instant::now() < deadline {
        contract.poll(&engine);
        if let Some(pos) = engine.get_f64("time-pos") {
            if (pos - target).abs() < 1.0 {
                landed = Some(pos);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let landed = landed.unwrap_or_else(|| {
        panic!(
            "seek 未落到目标 {target:.2}（当前 {:?}）",
            engine.get_f64("time-pos")
        )
    });
    eprintln!("seek 实测：目标 {target:.2}s → 落点 {landed:.2}s（时长 {duration:.2}s）");
}

/// **`position` 的语义回归（本任务最容易写错的一处）**：
/// 作者保持 `position` 不变时，契约**不得**逐帧重复 seek。
///
/// 若实现成「与当前播放位置不同就 seek」，播放在前进、`time-pos` 永远不等于目标，
/// 于是每帧都 seek 一次 → 播放被钉死在目标点（表现为「拖动进度条后画面再也不动」）。
/// 这里先 seek 到一个目标，然后**持续喂同一个目标**并断言：① `ops == 0`（不再下发）、
/// ② 时间照常前进。
#[test]
fn constant_position_does_not_re_seek_every_frame() {
    if libmpv_path().is_none() {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    }
    let Some(video) = sample_video() else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO");
        return;
    };
    let Some(engine) = contract_engine() else {
        eprintln!("SKIP: 引擎构造失败");
        return;
    };
    let mut contract = MediaContract::new();
    let _ = load(&engine, &mut contract, &video);

    let duration = engine.get_f64("duration").expect("应有时长");
    let target = (duration * 0.3).clamp(1.0, duration - 2.0);

    let mut down = VideoContractDown::default();
    down.src = Some(video.to_string_lossy().into_owned());
    down.paused = false;
    down.position = Some(target);
    assert!(contract.apply(&engine, &down) > 0, "首次 seek 应下发");

    // 等它真的落到目标并开始前进。
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        contract.poll(&engine);
        if engine
            .get_f64("time-pos")
            .map(|p| (p - target).abs() < 1.0)
            .unwrap_or(false)
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    // 关键：**同一个 down 反复 apply**，必须全为 0 次下发。
    let mut total_ops = 0;
    for _ in 0..30 {
        total_ops += contract.apply(&engine, &down);
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        total_ops, 0,
        "position 不变时不得重复下发（重复 seek 会把播放钉死在目标点）"
    );

    // 并且时间在前进 —— 这才是「没有被钉死」的实证。
    let t_before = engine.get_f64("time-pos").unwrap_or(0.0);
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(2) {
        contract.poll(&engine);
        std::thread::sleep(Duration::from_millis(20));
    }
    let t_after = engine.get_f64("time-pos").unwrap_or(0.0);
    assert!(
        t_after > t_before + 0.3,
        "position 不变时播放应继续前进（{t_before} → {t_after}）"
    );
}

/// **音量 / 静音 / 倍速**：下行写入必须落到 mpv 属性上（读回校验，不是看界面）。
#[test]
fn real_volume_mute_rate_reach_the_playback_core() {
    if libmpv_path().is_none() {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    }
    let Some(video) = sample_video() else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO");
        return;
    };
    let Some(engine) = contract_engine() else {
        eprintln!("SKIP: 引擎构造失败");
        return;
    };
    let mut contract = MediaContract::new();
    let _ = load(&engine, &mut contract, &video);

    let mut down = VideoContractDown::default();
    down.src = Some(video.to_string_lossy().into_owned());

    // 音量 0..100 直接对上 mpv 的 volume 属性。
    down.volume = 37;
    down.muted = true;
    down.rate = 1.5;
    contract.apply(&engine, &down);
    assert_eq!(engine.get_f64("volume"), Some(37.0), "mpv 的 volume 应被设为 37");
    assert_eq!(engine.get_flag("mute"), Some(true), "mute 应为 true");
    assert_eq!(engine.get_f64("speed"), Some(1.5), "speed 应为 1.5");
    assert_eq!(contract.applied().volume, 37);

    // 静音解除 + 改倍速 + 改音量
    down.volume = 100;
    down.muted = false;
    down.rate = 0.5;
    contract.apply(&engine, &down);
    assert_eq!(engine.get_f64("volume"), Some(100.0));
    assert_eq!(engine.get_flag("mute"), Some(false));
    assert_eq!(engine.get_f64("speed"), Some(0.5));

    // `rate <= 0` 必须被忽略（0 倍速会把播放冻住）。
    down.rate = 0.0;
    contract.apply(&engine, &down);
    assert_eq!(
        engine.get_f64("speed"),
        Some(0.5),
        "rate=0 应被忽略，而不是把 speed 写成 0（那会冻住播放）"
    );

    // 音量越界必须夹紧。
    down.volume = 300;
    contract.apply(&engine, &down);
    assert_eq!(engine.get_f64("volume"), Some(100.0), "越界音量应夹到 100");
}

/// **上行 `onloadedmetadata` 只发一次**：浏览器语义如此，重复发会让上层反复重置
/// 进度条长度（表现为拖动条抖）。
#[test]
fn loaded_metadata_is_emitted_exactly_once() {
    if libmpv_path().is_none() {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    }
    let Some(video) = sample_video() else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO");
        return;
    };
    let Some(engine) = contract_engine() else {
        eprintln!("SKIP: 引擎构造失败");
        return;
    };
    let mut contract = MediaContract::new();
    let evs = load(&engine, &mut contract, &video);

    // 时长事件在 load 阶段就被消费了，故从返回值里数（见 `load` 的注释）。
    let mut count = 0usize;
    let mut last_duration = 0.0f64;
    for e in evs.iter() {
        if let VideoContractEvent::LoadedMetadata(d) = e {
            count += 1;
            last_duration = *d;
        }
    }
    assert_eq!(count, 1, "LoadedMetadata 应在加载期恰好发一次，实际 {count} 次");

    // 之后再持续轮询，**不得**再发第二次（重复发会让上层反复重置进度条长度）。
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut extra = 0usize;
    while Instant::now() < deadline {
        for e in contract.poll(&engine) {
            if matches!(e, VideoContractEvent::LoadedMetadata(_)) {
                extra += 1;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(extra, 0, "加载后不应再次回灌 LoadedMetadata，实际多发了 {extra} 次");
    assert!(last_duration > 0.0, "时长应为正数");
    // 与 mpv 自报的时长一致（不是我们编的）。
    let from_mpv = engine.get_f64("duration").unwrap_or(-1.0);
    assert!(
        (last_duration - from_mpv).abs() < 0.01,
        "回灌时长应等于 mpv 的 duration（{last_duration} vs {from_mpv}）"
    );
}

/// **`onended`**：正常播到结尾应回灌 `Ended`（而不是 `MediaError`）。
/// 做法：seek 到片尾前 0.4 s、用 2 倍速加速。
#[test]
fn real_eof_reports_ended_not_error() {
    if libmpv_path().is_none() {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    }
    let Some(video) = sample_video() else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO");
        return;
    };
    let Some(engine) = contract_engine() else {
        eprintln!("SKIP: 引擎构造失败");
        return;
    };
    let mut contract = MediaContract::new();
    let _ = load(&engine, &mut contract, &video);

    let duration = engine.get_f64("duration").expect("应有时长");
    let mut down = VideoContractDown::default();
    down.src = Some(video.to_string_lossy().into_owned());
    down.paused = false;
    down.rate = 2.0;
    down.position = Some((duration - 0.4).max(0.0));
    contract.apply(&engine, &down);

    let evs = poll_until(&engine, &mut contract, Duration::from_secs(15), |evs, _| {
        evs.iter().any(|e| matches!(e, VideoContractEvent::Ended))
    });
    assert!(
        evs.iter().any(|e| matches!(e, VideoContractEvent::Ended)),
        "播到结尾应回灌 Ended，实际事件：{evs:?}"
    );
    assert!(
        !evs.iter().any(|e| matches!(e, VideoContractEvent::MediaError(_))),
        "正常结束不得被误报为错误"
    );
    assert!(contract.media_error().is_none(), "正常结束不应留下 media_error");
}

/// **`onmediaerror`**：加载不存在的源应回灌 `MediaError`（而非静默）。
#[test]
fn real_bad_source_reports_media_error() {
    if libmpv_path().is_none() {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    }
    let Some(engine) = contract_engine() else {
        eprintln!("SKIP: 引擎构造失败");
        return;
    };
    let mut contract = MediaContract::new();
    let mut down = VideoContractDown::default();
    down.src = Some(if cfg!(windows) {
        r"Z:\definitely\not\here\nope.mp4".to_string()
    } else {
        "/definitely/not/here/nope.mp4".to_string()
    });
    contract.apply(&engine, &down);

    let evs = poll_until(&engine, &mut contract, Duration::from_secs(15), |evs, _| {
        evs.iter().any(|e| matches!(e, VideoContractEvent::MediaError(_)))
    });
    let err = evs.iter().find_map(|e| match e {
        VideoContractEvent::MediaError(m) => Some(m.clone()),
        _ => None,
    });
    let err = err.expect("加载不存在的源应回灌 MediaError");
    assert!(!err.is_empty(), "错误文案不能为空");
    assert_eq!(contract.media_error(), Some(err.as_str()));
    eprintln!("坏源实测错误文案：{err}");
}

/// **音频输出是 mpv 的活**（T-18 明确不引入 `cpal`）。
///
/// 做法：用**默认 ao**（不设 `ao=null`/`audio=no`）加载带音轨的片源，随即暂停，
/// 然后检查 mpv 真的开出了一个音频输出、且解码器认识这条轨。暂停是为了不出声。
#[test]
fn audio_output_is_delegated_to_mpv() {
    if libmpv_path().is_none() {
        eprintln!("SKIP: 本机无 libmpv");
        return;
    }
    let Some(video) = sample_video() else {
        eprintln!("SKIP: 未设 AUTO_SPIKE_VIDEO");
        return;
    };
    // 不设 ao/audio：让 mpv 用它自己的默认音频输出（Windows 上是 wasapi 之类）。
    let Ok(engine) = MpvEngine::new_with_options(&[
        ("vo", "libmpv"),
        ("terminal", "no"),
        ("msg-level", "all=error"),
        ("pause", "yes"), // 先暂停，避免测试发出声响
    ]) else {
        eprintln!("SKIP: 引擎构造失败");
        return;
    };
    let mut contract = MediaContract::new();
    let mut down = VideoContractDown::default();
    down.src = Some(video.to_string_lossy().into_owned());
    down.paused = true;
    contract.apply(&engine, &down);

    // 等音频轨配置出来。
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut ao = None;
    while Instant::now() < deadline {
        contract.poll(&engine);
        // `current-ao` 是 mpv 当前实际使用的音频输出设备名（空串表示还没开）。
        if let Some(name) = engine.get_string("current-ao") {
            if !name.is_empty() {
                ao = Some(name);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let codec = engine.get_string("audio-codec-name");
    let params = engine.get_string("audio-out-params");
    eprintln!("音频实测：current-ao={ao:?} codec={codec:?} out-params={params:?}");

    assert!(
        ao.is_some(),
        "mpv 应开出音频输出（current-ao 非空）——音频是它的职责，我们不用 cpal"
    );
    assert!(
        codec.is_some(),
        "应能读到音轨编码名（说明音轨被识别，而非被当成无音轨文件）"
    );
}
