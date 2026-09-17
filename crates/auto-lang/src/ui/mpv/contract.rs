//! VM/iced 端的**受控媒体契约**（PLAN-617 **T-18**）——§2.3 在 mpv 侧的同一份实现。
//!
//! # 契约（`.at` 作者视角，两端必须一致）
//!
//! ```auto
//! video {
//!     src: .current_url        position: .seek_target      paused: .is_playing == false
//!     volume: .volume          muted: .is_muted            rate: .playback_rate
//!     ontimeupdate: .OnTime($0)          // $0: float 秒
//!     onloadedmetadata: .OnDuration($0)  // $0: float 秒
//!     onplaystatechange: .OnPlayState($0)// $0: bool
//!     onended: .OnEnded                  onmediaerror: .OnMediaError($0)  // $0: str
//! }
//! ```
//!
//! **共享边界是「作者面的状态值」，不是 DOM 语义**：`volume` 是 0..100、`position`/
//! 时长是 float 秒、`rate` 是 float。Vue 侧由生成器把它翻成 `HTMLMediaElement`
//! 的 0..1（`el.volume = state.volume / 100`），VM 侧直接对上 mpv 的 `volume`
//! 属性——**两边各自完成各自后端的单位换算，`.at` 源码无需分叉**。
//!
//! # 为什么连 `src` 也不需要分叉
//!
//! mpv 自带网络栈，`loadfile` 既接受本地路径也接受 `http(s)://` URL。所以后端
//! 给 Vue 的 `/api/media/stream/<id>` 地址在 VM 侧可以**原样**丢给 mpv——这正是
//! 「同一份 `app.at` 两端都能跑」在**数据通路**上成立的原因，而不只是属性名对齐。
//!
//! # 三个必须写对的细节
//!
//! 1. **`position` 的语义是「目标变化时 seek」，不是「与当前播放位置不同就 seek」。**
//!    后者会让每一帧都触发一次 seek：播放在前进，`time-pos` 永远不等于目标，
//!    于是播放被钉死在目标点上（表现为「一拖动进度条就再也不前进」）。
//!    故这里只在**目标值本身发生改变**时下发。
//! 2. **下行必须做差量**：视图每帧都会被重建，若无条件把 `paused`/`volume`/`speed`
//!    全写一遍，会产生大量无意义的属性写（并且会与 mpv 自身的状态抖动打架）。
//! 3. **`generation` 在换片与 seek 时前进**，供 T-17 的 `VideoLatestWins`
//!    把在途的旧帧判为过期——否则 seek 后旧帧的上屏会盖掉新位置的画面。

use super::engine::{MpvEngine, MpvEventInfo};
use super::loader::{end_file_reason, event_id};

/// §2.3 的**受控下行**：`.at` 作者写的状态值，逐字段对应 `video` 元素收到的 prop。
///
/// 单位（与 Vue 端一致，因为这是作者面的约定）：
/// - `position`：**秒**（float）
/// - `volume`：**0..100**（int；Vue 侧生成器把它除以 100 给 `HTMLMediaElement.volume`）
/// - `rate`：倍速（float，1.0 为原速）
#[derive(Debug, Clone, PartialEq)]
pub struct VideoContractDown {
    /// 元素侧 `paused`（作者写 `paused: .is_playing == false`）。
    pub paused: bool,
    /// `position`：seek 目标（秒）。`None` = 本次不下发位置。
    pub position: Option<f64>,
    /// `volume`：0..100（**不是** 0..1）。
    pub volume: i32,
    pub muted: bool,
    /// `rate`：倍速。`<= 0` 会被忽略（见 `apply` 的注释）。
    pub rate: f64,
    /// `src`：媒体地址。**本地路径与 http(s) URL 都接受**（mpv 自带网络栈）。
    /// `None` = 不换片。
    pub src: Option<String>,
}

impl Default for VideoContractDown {
    /// 默认值刻意选「能播但不自动播」：原速、满音量、未静音、暂停、无源。
    ///
    /// 不 derive 是因为 `rate` 的默认值必须是 1.0 —— `f64::default() == 0.0`
    /// 会被 mpv 当成「0 倍速」而把播放冻住。
    fn default() -> Self {
        Self {
            paused: true,
            position: None,
            volume: 100,
            muted: false,
            rate: 1.0,
            src: None,
        }
    }
}

/// §2.3 的**上行回灌**：逐项对应作者写的 `on*` handler。
#[derive(Debug, Clone, PartialEq)]
pub enum VideoContractEvent {
    /// `ontimeupdate`：当前播放位置（秒）。
    TimeUpdate(f64),
    /// `onloadedmetadata`：总时长（秒）首次可知时发一次。
    LoadedMetadata(f64),
    /// `onplaystatechange`：是否正在播放（由 `pause` 属性合成，边缘触发）。
    PlayStateChange(bool),
    /// `onended`：正常播放到结尾。
    Ended,
    /// `onmediaerror`：加载/解码失败，附 mpv 的错误文案。
    MediaError(String),
}

/// 契约适配器：把「作者面状态」翻成 mpv 的属性与命令，并把 mpv 的状态变化
/// 翻回契约事件。
///
/// 无内部线程、无锁——由渲染循环每帧调一次 [`Self::apply`] 与 [`Self::poll`]。
#[derive(Debug)]
pub struct MediaContract {
    applied: VideoContractDown,
    /// `src` 换片与每次 seek 都会前进；供 T-17 的 `VideoLatestWins` 判过期帧。
    generation: u64,
    /// 已经下发给 mpv 的 seek 目标（用于「目标变化才 seek」的判定）。
    last_seek_target: Option<f64>,
    last_time_update: Option<f64>,
    last_play_state: Option<bool>,
    emitted_metadata: bool,
    loaded_src: Option<String>,
    media_error: Option<String>,
}

impl Default for MediaContract {
    fn default() -> Self {
        Self::new()
    }
}

/// `time-pos` 变化到多少才回灌一次 `ontimeupdate`（秒）。
///
/// 这个阈值不是精度问题而是**流量**问题：属性会被高频查询，逐帧回灌等值事件
/// 只会让上层重算 OSD。0.25 s 对秒级显示足够（Vue 侧 `timeupdate` 的实际频率
/// 也在 4 Hz 量级）。**注意它与 seek 判定无关**——seek 走 [`MediaContract::apply`]
/// 的「目标变化」逻辑，不受此阈值影响。
pub const TIME_UPDATE_EPSILON: f64 = 0.25;

impl MediaContract {
    pub fn new() -> Self {
        Self {
            applied: VideoContractDown {
                // 用一个「绝不等于任何真实下发」的初值，保证第一次 apply 会全量下发。
                paused: false,
                position: None,
                volume: -1,
                muted: false,
                rate: f64::NAN,
                src: None,
            },
            generation: 0,
            last_seek_target: None,
            last_time_update: None,
            last_play_state: None,
            emitted_metadata: false,
            loaded_src: None,
            media_error: None,
        }
    }

    /// 当前的代际（换片/seek 后前进）。渲染侧应把它交给
    /// [`super::channel::VideoLatestWins`]，让在途的旧帧作废。
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// 最近一次媒体错误（若有）。
    pub fn media_error(&self) -> Option<&str> {
        self.media_error.as_deref()
    }

    /// 已经加载的源（换片判定用）。
    pub fn loaded_src(&self) -> Option<&str> {
        self.loaded_src.as_deref()
    }

    /// 最近一次下发的下行快照（诊断/测试用）。
    pub fn applied(&self) -> &VideoContractDown {
        &self.applied
    }

    /// **下行同步**：把作者面状态应用到播放内核，只下发**发生变化**的字段。
    ///
    /// 返回本次实际执行的 mpv 操作数（诊断用；稳态下应接近 0）。
    pub fn apply(&mut self, engine: &MpvEngine, down: &VideoContractDown) -> usize {
        let mut ops = 0;

        // ① 换片：src 变化才 loadfile。**必须先于其它属性**——否则会先给旧片
        //    设置暂停/音量，无谓地扰动。
        if down.src != self.applied.src {
            if let Some(src) = down.src.as_ref() {
                let effective_src = if src.starts_with('/') {
                    if let Ok(base) = std::env::var("AUTO_HTTP_BASE") {
                        let base = base.trim().trim_end_matches('/');
                        if !base.is_empty() {
                            format!("{}{}", base, src)
                        } else {
                            src.clone()
                        }
                    } else if let Ok(port) = std::env::var("AUTO_HTTP_PORT") {
                        let port = port.trim();
                        if !port.is_empty() {
                            format!("http://127.0.0.1:{}{}", port, src)
                        } else {
                            src.clone()
                        }
                    } else {
                        src.clone()
                    }
                } else {
                    src.clone()
                };
                match engine.command(&["loadfile", &effective_src]) {
                    Ok(()) => {
                        // 换片 = 新一代：清掉上一部的 seek/时长/错误等物化状态，
                        // 并让 T-17 的在途帧失效。
                        self.generation = self.generation.wrapping_add(1);
                        self.emitted_metadata = false;
                        self.loaded_src = Some(src.clone());
                        self.media_error = None;
                        self.last_seek_target = None;
                        self.last_time_update = None;
                        self.last_play_state = None;
                        ops += 1;
                    }
                    Err(e) => {
                        // 加载失败就是 onmediaerror 的来源之一（另一处是 END_FILE/ERROR）。
                        self.media_error = Some(e);
                    }
                }
            }
        }

        // ② 属性下行（全部做差量）。
        if down.paused != self.applied.paused {
            if engine.set_flag("pause", down.paused).is_ok() {
                ops += 1;
            }
        }
        if down.muted != self.applied.muted {
            if engine.set_flag("mute", down.muted).is_ok() {
                ops += 1;
            }
        }
        let volume = down.volume.clamp(0, 100) as f64;
        if (volume - self.applied.volume as f64).abs() > f64::EPSILON {
            if engine.set_f64("volume", volume).is_ok() {
                ops += 1;
            }
        }
        // `rate <= 0` 一律忽略：mpv 的 `speed` 为 0 会把播放冻住，而「0 倍速」
        // 从来不是作者想表达的意图（多半是状态还没初始化）。默认值 1.0 已在
        // `VideoContractDown::default()` 里保证。
        if down.rate > 0.0 && (down.rate - self.applied.rate).abs() > f64::EPSILON {
            if engine.set_f64("speed", down.rate).is_ok() {
                ops += 1;
            }
        }

        // ③ seek：**只在「目标值本身变化」时下发**（理由见模块文档第 1 条）。
        if let Some(target) = down.position {
            let changed = match self.last_seek_target {
                Some(prev) => (prev - target).abs() > f64::EPSILON,
                None => true,
            };
            if changed && engine.set_f64("time-pos", target).is_ok() {
                ops += 1;
                self.last_seek_target = Some(target);
                // seek 也是一代：在途旧帧必须作废，否则会盖掉新位置的画面。
                self.generation = self.generation.wrapping_add(1);
                // seek 后 OSD 的当前时间会立刻跳到新位置，允许立刻回灌一次。
                self.last_time_update = None;
            }
        } else {
            // 作者不再表达位置 → 允许下一次 position 出现时重新 seek。
            self.last_seek_target = None;
        }

        self.applied = down.clone();
        ops
    }

    /// **上行回灌**：抽一次 mpv 状态与事件，产出契约事件。
    ///
    /// 由渲染循环每帧调一次。返回空 `Vec` 是常态（多数帧没有新信息）。
    pub fn poll(&mut self, engine: &MpvEngine) -> Vec<VideoContractEvent> {
        let mut out = Vec::new();

        // ① 事件面：结束/错误。放最前，因为它决定本帧后续是否还有意义。
        while let Some(info) = engine.wait_event(0.0) {
            self.absorb_event(engine, &info, &mut out);
        }

        // ② 时长：`duration` 在 loadfile 后一段时间才可用，故「首次可知」即发
        //    `onloadedmetadata`（对应浏览器 loadedmetadata）。
        if !self.emitted_metadata {
            if let Some(d) = engine.get_f64("duration") {
                if d > 0.0 && d.is_finite() {
                    self.emitted_metadata = true;
                    out.push(VideoContractEvent::LoadedMetadata(d));
                }
            }
        }

        // ③ 播放位置：变化超过阈值才回灌。
        if let Some(pos) = engine.get_f64("time-pos") {
            let changed = match self.last_time_update {
                Some(prev) => (prev - pos).abs() >= TIME_UPDATE_EPSILON,
                None => true,
            };
            if changed {
                self.last_time_update = Some(pos);
                out.push(VideoContractEvent::TimeUpdate(pos));
            }
        }

        // ④ 播放状态：由 `pause` 属性**合成** `onplaystatechange`（§2.3 明写它
        //    不是原生 DOM 事件，是由 play/pause 合成的），故这里做边缘检测。
        if let Some(paused) = engine.get_flag("pause") {
            let playing = !paused;
            if self.last_play_state != Some(playing) {
                self.last_play_state = Some(playing);
                out.push(VideoContractEvent::PlayStateChange(playing));
            }
        }

        out
    }

    /// 处理单个事件：只把与契约有关的两类翻成事件，其余忽略。
    fn absorb_event(
        &mut self,
        engine: &MpvEngine,
        info: &MpvEventInfo,
        out: &mut Vec<VideoContractEvent>,
    ) {
        if info.event_id != event_id::END_FILE {
            return;
        }
        if !info.end_file_is_error {
            if info.end_file_reason == Some(end_file_reason::EOF) {
                out.push(VideoContractEvent::Ended);
            }
            // STOP/QUIT/REDIRECT 是我们自己或 core 的行为：不是媒体错误，也不代表放完。
            return;
        }
        // 加载/解码失败。**注意**：mpv 在 END_FILE/ERROR 上**不总是**给出错误码——
        // 实测加载不存在的文件时 `error == 0`，直接拿 `mpv_error_string(0)` 会得到
        // 「success」这种毫无信息量的文案。故这里以错误码为可选信息，主信息是
        // 「哪个源加载失败」。
        let src = engine
            .get_string("path")
            .or_else(|| engine.get_string("filename"))
            .or_else(|| self.loaded_src.clone());
        let detail = match (info.error, src) {
            (0, Some(s)) => format!("无法播放该媒体（加载或解码失败）：{s}"),
            (0, None) => "无法播放该媒体（加载或解码失败，mpv 未给出错误码）".to_string(),
            (code, Some(s)) => format!("无法播放该媒体：{}（错误码 {code}）：{s}", engine.error_text(code)),
            (code, None) => format!("无法播放该媒体：{}（错误码 {code}）", engine.error_text(code)),
        };
        self.media_error = Some(detail.clone());
        out.push(VideoContractEvent::MediaError(detail));
    }
}

/// 把作者面的 `volume`(0..100) 翻成 `HTMLMediaElement.volume` 的 0..1。
///
/// 只给**测试与文档**用：Vue 侧这一步由生成器在 JS 里做，VM 侧不需要（mpv 的
/// `volume` 本就是 0..100）。留在这里是为了让「两端到底怎么换算」有单一出处，
/// 且能被断言钉住。
pub fn volume_to_element(volume_0_100: i32) -> f32 {
    volume_0_100.clamp(0, 100) as f32 / 100.0
}

/// [`volume_to_element`] 的逆。
pub fn volume_from_element(v: f32) -> i32 {
    (v.clamp(0.0, 1.0) * 100.0).round() as i32
}
