//! libmpv 引擎的**生命周期**：句柄创建/初始化/销毁，以及 render context 的
//! 创建与释放顺序（PLAN-617 T-16 的核心交付）。
//!
//! ## 为什么顺序是硬约束
//!
//! `include/mpv/render.h:119`（"Context and handle lifecycle"）：
//!
//! > You must free the context with mpv_render_context_free() before the mpv core
//! > is destroyed. If this doesn't happen, undefined behavior will result.
//!
//! 也就是说销毁顺序**只能**是：
//!
//! ```text
//! mpv_render_context_free()  →  mpv_terminate_destroy()
//! ```
//!
//! 反过来（先销毁 core 再去 free render context）是 UB。本模块把这个顺序钉在
//! [`Drop`] 里，调用方无法写错。
//!
//! 另外两条同源的约束（同样出自 render.h）：
//! - 一个 core **同时只能有 1 个** render context（`:114`）；
//! - render context 建好之前 video 初始化会失败、或退回自建窗口的 VO（`:111`）
//!   —— 故「先建 render context，再 loadfile」。
//!
//! ## 同线程要求只在旧版存在
//!
//! render.h:96-107 记载：API < 1.105（mpv < 0.30）时 `mpv_render_*` 必须与
//! `mpv_render_context_create()` 同线程，否则是 UB；**1.105 起该约束已解除**。
//! 本模块按运行时 `mpv_client_api_version()` 判断：新版一切照常，旧版在跨线程
//! 释放时打 warning（顺序仍然照上文执行——「销毁 core 前不 free」是 UB，
//! 比旧版的线程风险更严重）。
//!
//! ## 缺失即降级（AC-20）
//!
//! 本机没装 mpv 是**正常情形**：构造返回 [`MpvUnavailable::NoLibrary`]，
//! 全程不 panic、不黑屏。渲染层据此走今日的诚实占位。

use std::ffi::{c_char, c_int, c_void};
use std::path::Path;
use std::sync::Arc;
use std::thread::ThreadId;

use super::frame::SwTarget;
use super::locale;
use super::loader::{
    cstring, event_id, render_param, MpvApi, MpvLoadError, MpvSymbols, ParamList,
};

/// 一个事件里我们实际要用的字段——`mpv_event` 的**值拷贝**。
///
/// 不返回 `&MpvEvent` 的理由见 [`MpvEngine::wait_event`]（指针只活到下一次
/// `wait_event`，借用出去就会悬垂）。`data` 是各事件特有的负载（如
/// `END_FILE` 的 reason），T-18 需要时再按事件类型单独转换。
#[derive(Debug, Clone, Copy)]
pub struct MpvEventInfo {
    pub event_id: std::ffi::c_int,
    pub error: std::ffi::c_int,
    pub reply_userdata: u64,
    /// 仅 `END_FILE` 有值：结束原因（[`super::loader::end_file_reason`]）。
    /// 在 `wait_event` 内部立刻解出——`mpv_event.data` 指向的结构体只在下一次
    /// `wait_event` 之前有效，不能带出函数。
    pub end_file_reason: Option<std::ffi::c_int>,
    /// 仅 `END_FILE` 且原因不是 EOF 时才有意义：是否属于「出错」。
    pub end_file_is_error: bool,
}

/// 引擎不可用的原因。
///
/// **[`Self::NoLibrary`] 不是错误而是降级信号**——渲染层应当把它当成
/// 「本后端没有解码能力」的正常分支。
#[derive(Debug, Clone)]
pub enum MpvUnavailable {
    /// 本机没有 libmpv（未设 `AUTO_MPV_LIB` 且 exe 同目录没有运行库）。
    NoLibrary,
    /// 找到了运行库但加载/解析符号失败。
    LoadFailed(MpvLoadError),
    /// `mpv_create()` 返回 NULL —— 常见原因是 `LC_NUMERIC` 不是 `"C"`（见 [`super::locale`]）。
    CreateFailed { locale: Option<String> },
    /// `mpv_initialize()` 报错，附 mpv 自己的错误文案。
    InitFailed { code: c_int, detail: String },
    /// render context 创建失败。
    RenderContextFailed { code: c_int, detail: String },
}

impl std::fmt::Display for MpvUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoLibrary => write!(f, "本机没有 libmpv（无解码能力，走降级）"),
            Self::LoadFailed(e) => write!(f, "{e}"),
            Self::CreateFailed { locale } => write!(
                f,
                "mpv_create() 返回 NULL（LC_NUMERIC 当前为 {:?}，mpv 要求 \"C\"）",
                locale.as_deref().unwrap_or("<未知>")
            ),
            Self::InitFailed { code, detail } => {
                write!(f, "mpv_initialize() 失败（{code}: {detail}）")
            }
            Self::RenderContextFailed { code, detail } => {
                write!(f, "mpv_render_context_create() 失败（{code}: {detail}）")
            }
        }
    }
}

impl std::error::Error for MpvUnavailable {}

/// 一个 libmpv 实例。
///
/// 含裸指针，故天然 `!Send + !Sync`——这是有意的保守选择：`mpv_wait_event`
/// 明确只允许一个线程调用（client.h:1699），T-17 若需要跨线程搬运，
/// 应显式 `unsafe impl Send` 并在此处写明理由。
pub struct MpvEngine {
    /// 必须比句柄活得久：符号地址由它担保。
    api: Arc<MpvApi>,
    handle: *mut c_void,
    /// 一个 core 同时只允许 1 个 render context（render.h:114）。
    render: Option<*mut c_void>,
    api_version: u32,
    created_on: ThreadId,
}

impl MpvEngine {
    /// 按默认选项创建引擎：`vo=libmpv`（由 render API 接管 VO）、关终端回声、
    /// 只把 error 级日志接进 `log`。
    ///
    /// 需要别的选项时用 [`Self::new_with_options`]。
    pub fn new() -> Result<Self, MpvUnavailable> {
        Self::new_with_options(&[
            ("vo", "libmpv"),
            ("terminal", "no"),
            ("msg-level", "all=error"),
        ])
    }

    /// 创建引擎并按顺序下发 `options`（在 `mpv_initialize()` **之前**，
    /// 因为其中多数只有在初始化前设置才生效）。
    ///
    /// 单个选项失败**不致命**（不同 mpv 版本选项集不同），只记 warning——
    /// 否则一个新版本新增/改名的选项就会让整个播放能力不可用。
    pub fn new_with_options(options: &[(&str, &str)]) -> Result<Self, MpvUnavailable> {
        // ① mpv 的 C 环境前提（client.h:147）。必须早于 mpv_create()。
        if !locale::ensure_c_numeric_locale() {
            log::warn!("mpv: setlocale(LC_NUMERIC, \"C\") 失败——mpv_create 可能返回 NULL");
        }

        // ② 解析 + 加载运行库；没有就是正常降级。
        let api = match MpvApi::resolve_and_load() {
            Ok(Some(api)) => Arc::new(api),
            Ok(None) => return Err(MpvUnavailable::NoLibrary),
            Err(e) => return Err(MpvUnavailable::LoadFailed(e)),
        };
        // client.h:479 把 LC_NUMERIC 列为 mpv_create 返回 NULL 的原因之一，
        // 提前把当前值取出来，好在失败时说得清。
        let api_version = api.client_api_version();
        let sym: MpvSymbols = api.symbols;

        // ③ 句柄。
        // SAFETY: 选项字符串在本次调用期间有效；initialize 前 set_option 是该
        // 选项生效的正常时序。
        let handle = unsafe { (sym.create)() };
        if handle.is_null() {
            return Err(MpvUnavailable::CreateFailed {
                locale: locale::numeric_locale(),
            });
        }

        for (name, value) in options {
            let (Ok(n), Ok(v)) = (cstring(name), cstring(value)) else {
                log::warn!("mpv: 跳过无法编码的选项 {name}={value}");
                continue;
            };
            // SAFETY: handle 非空且未销毁；两个 C 字符串在调用期间有效。
            let rc = unsafe { (sym.set_option_string)(handle, n.as_ptr(), v.as_ptr()) };
            if rc < 0 {
                log::warn!(
                    "mpv: 选项 {name}={value} 被拒绝（{}）——已跳过",
                    api.error_text(rc)
                );
            }
        }

        // SAFETY: handle 非空、未被其它线程使用。
        let rc = unsafe { (sym.initialize)(handle) };
        if rc < 0 {
            let detail = api.error_text(rc);
            // 初始化失败也要按顺序拆掉（此时还没有 render context）。
            // SAFETY: handle 非空且仍归我们所有。
            unsafe { (sym.terminate_destroy)(handle) };
            return Err(MpvUnavailable::InitFailed { code: rc, detail });
        }

        log::info!(
            "mpv: 已加载 {}（client API {}.{}）",
            api.path().display(),
            api_version >> 16,
            api_version & 0xFFFF
        );

        Ok(Self {
            api,
            handle,
            render: None,
            api_version,
            created_on: std::thread::current().id(),
        })
    }

    /// 把 mpv 的错误码翻成它自己的可读文案（事件里的 `error` 字段用）。
    pub fn error_text(&self, code: std::ffi::c_int) -> String {
        self.api.error_text(code)
    }

    /// `MPV_CLIENT_API_VERSION`（major << 16 | minor）。
    pub fn api_version(&self) -> u32 {
        self.api_version
    }

    /// 实际加载的运行库路径（诊断/文档用）。
    pub fn library_path(&self) -> &Path {
        self.api.path()
    }

    /// 是否已建 render context。
    pub fn has_render_context(&self) -> bool {
        self.render.is_some()
    }

    /// 建 **SW** render context（本计划采用的唯一后端，T-15 裁定）。
    ///
    /// 采用 `ADVANCED_CONTROL`：由 `mpv_render_context_update()` 驱动
    /// 「有没有新帧」，这是 T-15 实测的生产形状（每次 render 都是真实帧）。
    ///
    /// **必须在 loadfile 之前调用**——否则 video 初始化会失败或退回自建窗口 VO
    /// （render.h:111）。重复调用返回错误（一个 core 只允许 1 个 context）。
    pub fn create_sw_render_context(&mut self) -> Result<(), MpvUnavailable> {
        if self.render.is_some() {
            return Err(MpvUnavailable::RenderContextFailed {
                code: 0,
                detail: "该实例已有 render context（一个 core 同时只允许 1 个）".into(),
            });
        }
        let sym = self.api.symbols;
        let mut ctx: *mut c_void = std::ptr::null_mut();
        let mut advanced: c_int = 1;
        let params = ParamList::new()
            .push(
                render_param::API_TYPE,
                render_param::API_TYPE_SW.as_ptr() as *mut c_void,
            )
            .push(
                render_param::ADVANCED_CONTROL,
                &mut advanced as *mut c_int as *mut c_void,
            );
        // SAFETY: handle 是我们的且非空；params 的每个 data 都指向本函数栈上
        // 仍然存活的值（advanced 活到调用结束），符合「调用期间有效」的约定。
        let rc = unsafe { (sym.render_context_create)(&mut ctx, self.handle, params.as_ptr()) };
        if rc < 0 {
            return Err(MpvUnavailable::RenderContextFailed {
                code: rc,
                detail: self.api.error_text(rc),
            });
        }
        self.render = Some(ctx);
        Ok(())
    }

    /// `mpv_render_context_update()`：返回位含 [`render_param::UPDATE_FRAME`]
    /// 时才有新帧可渲染。
    pub fn update(&self) -> u64 {
        match self.render {
            // SAFETY: render 非空即由 create_sw_render_context 成功建立。
            Some(ctx) => unsafe { (self.api.symbols.render_context_update)(ctx) },
            None => 0,
        }
    }

    /// 是否有新帧待渲染（[`Self::update`] 的语义化包装）。
    pub fn has_new_frame(&self) -> bool {
        self.update() & render_param::UPDATE_FRAME != 0
    }

    /// 把当前帧渲染进 `target`（RGBA / `rgb0` 布局，4 字节每像素）。
    ///
    /// `block_for_target_time = false`：不等 mpv 的对齐显示时间，使这次调用的
    /// 耗时就是**纯缩放/色彩转换/写入**代价（T-15 即按此口径测量）。
    /// 需要 A/V 节奏时由调用方自己按帧时钟调度（T-18）。
    pub fn render_sw_frame(&self, target: &SwTarget) -> Result<(), String> {
        let Some(ctx) = self.render else {
            return Err("尚未创建 render context（先调 create_sw_render_context）".into());
        };
        let size = [target.width as c_int, target.height as c_int];
        let stride = target.stride;
        let mut no_block: c_int = 0;
        let params = ParamList::new()
            .push(render_param::SW_SIZE, size.as_ptr() as *mut c_void)
            .push(
                render_param::SW_FORMAT,
                b"rgb0\0".as_ptr() as *mut c_void,
            )
            .push(
                render_param::SW_STRIDE,
                &stride as *const usize as *mut c_void,
            )
            .push(render_param::SW_POINTER, target.ptr as *mut c_void)
            .push(
                render_param::BLOCK_FOR_TARGET_TIME,
                &mut no_block as *mut c_int as *mut c_void,
            );
        // SAFETY: 调用方以 unsafe SwTarget::new 保证 ptr 可写且覆盖
        // stride*height 字节；size/stride/format 均在本函数栈上存活。
        let rc = unsafe { (self.api.symbols.render_context_render)(ctx, params.as_ptr()) };
        if rc < 0 {
            return Err(format!(
                "mpv_render_context_render 失败：{}",
                self.api.error_text(rc)
            ));
        }
        Ok(())
    }

    /// 下发一条命令（`mpv_command`，argv 形式，如 `["loadfile", path]`）。
    ///
    /// **应在建立 render context 之后调用**（见 [`Self::create_sw_render_context`]）。
    pub fn command(&self, argv: &[&str]) -> Result<(), String> {
        if argv.is_empty() {
            return Err("命令参数为空".into());
        }
        let owned: Vec<std::ffi::CString> = argv
            .iter()
            .map(|s| cstring(s))
            .collect::<Result<_, _>>()?;
        let mut ptrs: Vec<*const c_char> = owned.iter().map(|c| c.as_ptr()).collect();
        ptrs.push(std::ptr::null());
        // SAFETY: handle 非空；argv 以 NULL 结尾，且每个指针指向的 CString 在
        // 本次调用期间由 `owned` 持有。
        let rc = unsafe { (self.api.symbols.command)(self.handle, ptrs.as_ptr()) };
        if rc < 0 {
            return Err(format!(
                "mpv_command 失败：{}",
                self.api.error_text(rc)
            ));
        }
        Ok(())
    }

    /// 阻塞至多 `timeout_secs` 秒取一个事件（`None` = 超时）。
    ///
    /// client.h:1699 —— **同一 handle 同时只允许一个线程**调它。
    ///
    /// 刻意返回**值拷贝**而不是 `&MpvEvent`：mpv 只保证事件指针有效到下一次
    /// `wait_event()`，若借用出去，调用方完全可以再调一次 `wait_event()`（只需
    /// `&self`）而让旧引用悬垂。返回拷贝把这个生命周期陷阱从 API 上消掉。
    pub fn wait_event(&self, timeout_secs: f64) -> Option<MpvEventInfo> {
        // SAFETY: handle 非空。返回的指针由 mpv 拥有且只在下一次 wait_event 前有效，
        // 故这里**立即**读完所需字段，不把引用带出函数。
        let ev = unsafe { (self.api.symbols.wait_event)(self.handle, timeout_secs) };
        if ev.is_null() {
            return None;
        }
        // SAFETY: ev 非空且此刻有效——紧接着就拷贝出来。
        let ev = unsafe { &*ev };
        // **关键**：超时/无事件时 mpv 返回的是**有效指针 + `MPV_EVENT_NONE`**，
        // 而不是 NULL。若只判空，`while let Some(..)` 形式的事件抽取循环会永远
        // 拿到 `Some(NONE)` 而**死循环**（T-18 实测踩到：一次 poll 直接把测试挂死）。
        // 故「无事件」必须在这里归一成 `None`。
        if ev.event_id == event_id::NONE {
            return None;
        }
        let mut end_file_reason = None;
        let mut end_file_is_error = false;
        if ev.event_id == event_id::END_FILE && !ev.data.is_null() {
            // `mpv_event.data` 在 END_FILE 时指向 `mpv_event_end_file`：
            // { int reason; int error; int64 playlist_entry_id; ... }
            #[repr(C)]
            struct EndFile {
                reason: std::ffi::c_int,
                error: std::ffi::c_int,
            }
            // SAFETY: 按 client.h 的事件契约，END_FILE 的 data 就是该结构体；
            // 只读前两个 int，生命周期覆盖本次调用。
            let ef = unsafe { &*(ev.data as *const EndFile) };
            end_file_reason = Some(ef.reason);
            end_file_is_error = ef.reason == super::loader::end_file_reason::ERROR;
        }
        Some(MpvEventInfo {
            event_id: ev.event_id,
            error: ev.error,
            reply_userdata: ev.reply_userdata,
            end_file_reason,
            end_file_is_error,
        })
    }

    /// 等到「首个可渲染事件」（`FILE_LOADED` / `VIDEO_RECONFIG`）。
    ///
    /// 返回 `Ok(true)` 表示等到了；`Ok(false)` 表示超时；`Err` 是文件提前结束。
    pub fn wait_first_frame_event(&self, timeout_secs: f64) -> Result<bool, String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs_f64(timeout_secs);
        while std::time::Instant::now() < deadline {
            let Some(ev) = self.wait_event(0.05) else {
                // `wait_event` 已阻塞至多 0.05s；这里不再额外 sleep，直接再等一轮。
                continue;
            };
            match ev.event_id {
                event_id::FILE_LOADED | event_id::VIDEO_RECONFIG => return Ok(true),
                event_id::END_FILE => return Err("文件在出首帧前就结束了".into()),
                _ => {}
            }
        }
        Ok(false)
    }

    /// 事件 id 的中文可读名（诊断/日志用）。
    pub fn event_name(id: c_int) -> &'static str {
        match id {
            event_id::NONE => "NONE",
            event_id::LOG_MESSAGE => "LOG_MESSAGE",
            event_id::END_FILE => "END_FILE",
            event_id::FILE_LOADED => "FILE_LOADED",
            event_id::IDLE => "IDLE",
            event_id::VIDEO_RECONFIG => "VIDEO_RECONFIG",
            event_id::AUDIO_RECONFIG => "AUDIO_RECONFIG",
            event_id::SEEK => "SEEK",
            event_id::PLAYBACK_RESTART => "PLAYBACK_RESTART",
            event_id::PROPERTY_CHANGE => "PROPERTY_CHANGE",
            _ => "OTHER",
        }
    }

    // ───────────────────────── 属性读写（T-18 的受控媒体契约要用） ─────────────────────────
    //
    // 一律走 mpv 的**属性**接口而不是命令：属性是 mpv 的规范状态面，读写对称，
    // 且 `paused`/`volume`/`mute`/`speed`/`time-pos`/`duration` 这些正是 §2.3
    // 契约里的字段（见 `contract.rs`）。

    /// 读一个 double 属性（`time-pos`/`duration`/`speed`…）。
    ///
    /// 属性暂时不可用（例如尚未 loadfile、或该属性此刻无效）返回 `None`——
    /// **这不是错误**：mpv 对无效属性就是返回错误码。
    pub fn get_f64(&self, name: &str) -> Option<f64> {
        let cname = cstring(name).ok()?;
        let mut out: f64 = 0.0;
        let rc = unsafe {
            // SAFETY: handle 非空；out 是栈上 double，符合 DOUBLE 格式要求。
            (self.api.symbols.get_property)(
                self.handle,
                cname.as_ptr(),
                super::loader::format::DOUBLE,
                &mut out as *mut f64 as *mut c_void,
            )
        };
        (rc >= 0).then_some(out)
    }

    /// 读一个 int64 属性。
    pub fn get_i64(&self, name: &str) -> Option<i64> {
        let cname = cstring(name).ok()?;
        let mut out: i64 = 0;
        let rc = unsafe {
            // SAFETY: 同上，INT64 需要 int64_t*。
            (self.api.symbols.get_property)(
                self.handle,
                cname.as_ptr(),
                super::loader::format::INT64,
                &mut out as *mut i64 as *mut c_void,
            )
        };
        (rc >= 0).then_some(out)
    }

    /// 读一个 bool 属性（mpv 用 FLAG，底层是 int）。
    pub fn get_flag(&self, name: &str) -> Option<bool> {
        let cname = cstring(name).ok()?;
        let mut out: c_int = 0;
        let rc = unsafe {
            // SAFETY: 同上，FLAG 需要 int*。
            (self.api.symbols.get_property)(
                self.handle,
                cname.as_ptr(),
                super::loader::format::FLAG,
                &mut out as *mut c_int as *mut c_void,
            )
        };
        (rc >= 0).then_some(out != 0)
    }

    /// 读一个字符串属性。mpv 分配的字符串在本函数内用 `mpv_free` 释放。
    pub fn get_string(&self, name: &str) -> Option<String> {
        let cname = cstring(name).ok()?;
        // SAFETY: handle 非空；返回的 char* 由 mpv 拥有且必须 free，
        // 下面立刻拷成 String 再 free，不留悬垂。
        let p = unsafe { (self.api.symbols.get_property_string)(self.handle, cname.as_ptr()) };
        if p.is_null() {
            return None;
        }
        // SAFETY: p 非空且以 NUL 结尾（mpv 约定）。
        let s = unsafe { std::ffi::CStr::from_ptr(p) }
            .to_string_lossy()
            .into_owned();
        // SAFETY: p 确由本次 get_property_string 分配。
        unsafe { (self.api.symbols.free)(p as *mut c_void) };
        Some(s)
    }

    /// 设一个 double 属性。
    pub fn set_f64(&self, name: &str, value: f64) -> Result<(), String> {
        let cname = cstring(name)?;
        let mut v = value;
        let rc = unsafe {
            // SAFETY: handle 非空；v 是栈上 double。
            (self.api.symbols.set_property)(
                self.handle,
                cname.as_ptr(),
                super::loader::format::DOUBLE,
                &mut v as *mut f64 as *mut c_void,
            )
        };
        if rc < 0 {
            return Err(format!(
                "设置 {name}={value} 失败：{}",
                self.api.error_text(rc)
            ));
        }
        Ok(())
    }

    /// 设一个 int64 属性。
    pub fn set_i64(&self, name: &str, value: i64) -> Result<(), String> {
        let cname = cstring(name)?;
        let mut v = value;
        let rc = unsafe {
            // SAFETY: 同上。
            (self.api.symbols.set_property)(
                self.handle,
                cname.as_ptr(),
                super::loader::format::INT64,
                &mut v as *mut i64 as *mut c_void,
            )
        };
        if rc < 0 {
            return Err(format!(
                "设置 {name}={value} 失败：{}",
                self.api.error_text(rc)
            ));
        }
        Ok(())
    }

    /// 设一个 bool（FLAG，底层 int）。
    pub fn set_flag(&self, name: &str, value: bool) -> Result<(), String> {
        let cname = cstring(name)?;
        let mut v: c_int = if value { 1 } else { 0 };
        let rc = unsafe {
            // SAFETY: 同上，FLAG 需要 int*。
            (self.api.symbols.set_property)(
                self.handle,
                cname.as_ptr(),
                super::loader::format::FLAG,
                &mut v as *mut c_int as *mut c_void,
            )
        };
        if rc < 0 {
            return Err(format!(
                "设置 {name}={value} 失败：{}",
                self.api.error_text(rc)
            ));
        }
        Ok(())
    }

    /// `mpv_get_time_us()`：mpv 内部单调递增时钟（微秒）。
    pub fn time_us(&self) -> i64 {
        // SAFETY: 无参查询。
        unsafe { (self.api.symbols.get_time_us)(self.handle) }
    }

    /// 运行库是否可用（不构造引擎的廉价探测；渲染层用它决定是否走降级）。
    pub fn is_available() -> bool {
        super::loader::resolve_library().is_some()
    }
}

impl Drop for MpvEngine {
    /// **销毁顺序是本类型的核心不变量**（render.h:119）：
    /// 先 `mpv_render_context_free()`，再 `mpv_terminate_destroy()`。
    /// 反序是 UB，故这个顺序无法由调用方改写。
    fn drop(&mut self) {
        let sym = self.api.symbols;

        // 旧版（API < 1.105）才有「render 必须与 create 同线程」的约束
        // （render.h:96-107）。即便违反，这里仍然按正确顺序释放——不释放就
        // 销毁 core 是 UB，比旧版的线程风险更严重。
        if self.render.is_some() && self.api_version < (1 << 16 | 105) {
            if std::thread::current().id() != self.created_on {
                log::warn!(
                    "mpv: 在非创建线程释放 render context（client API {}.{} < 1.105）——\
                     旧版此路径为 UB；新版本无此约束",
                    self.api_version >> 16,
                    self.api_version & 0xFFFF
                );
            }
        }

        if let Some(ctx) = self.render.take() {
            // SAFETY: ctx 由 create_sw_render_context 成功建立且尚未释放；
            // 此刻 core 仍然存活（下一句才销毁），满足 render.h:119 的顺序要求。
            unsafe { (sym.render_context_free)(ctx) };
        }
        if !self.handle.is_null() {
            // SAFETY: handle 由 mpv_create 得到且尚未销毁；render context 已在
            // 上面释放。client.h 说明该函数线程安全。
            unsafe { (sym.terminate_destroy)(self.handle) };
            self.handle = std::ptr::null_mut();
        }
    }
}

impl std::fmt::Debug for MpvEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpvEngine")
            .field("library", &self.api.path())
            .field("api_version", &self.api_version)
            .field("has_render_context", &self.has_render_context())
            .finish()
    }
}
