//! libmpv 的运行时绑定：**解析运行库位置**并**解析符号表**。
//!
//! 用 `libloading` 在运行时 `LoadLibrary`（PLAN-617 T-15 裁定的路径），
//! 因此本能力**没有构建期原生依赖**：机器上没装 mpv 也能编译、也能过 CI，
//! 只是运行时走降级（AC-20）。
//!
//! 符号在 [`MpvApi::load`] 时**一次性**全部解析并拷成裸函数指针（[`MpvSymbols`]），
//! 于是调用方不必与 `libloading::Symbol<'lib, T>` 的生命周期搏斗——
//! 只要 [`MpvApi`] 活着，函数地址就有效。

use std::ffi::{c_char, c_double, c_int, c_ulong, c_void, OsStr};
use std::path::{Path, PathBuf};

/// 运行库文件名（**按平台**；本计划只在 Windows 上实测过，其它平台按惯例写，
/// 未经实测——首次在那些平台启用时必须先验证，别当成已验证）。
#[cfg(windows)]
pub const LIB_NAME: &str = "libmpv-2.dll";
#[cfg(target_os = "linux")]
pub const LIB_NAME: &str = "libmpv.so.2";
#[cfg(target_os = "macos")]
pub const LIB_NAME: &str = "libmpv.2.dylib";

/// 显式指定运行库的环境变量（解析序的第一位）。
pub const ENV_MPV_LIB: &str = "AUTO_MPV_LIB";

/// `mpv_render_param`（`include/mpv/render.h`）。
#[repr(C)]
pub struct RenderParam {
    pub type_: c_int,
    pub data: *mut c_void,
}

/// `mpv_render_param_type` 的取值（render.h）。
pub mod render_param {
    pub const INVALID: i32 = 0;
    pub const API_TYPE: i32 = 1;
    pub const OPENGL_INIT_PARAMS: i32 = 2;
    pub const ADVANCED_CONTROL: i32 = 10;
    pub const BLOCK_FOR_TARGET_TIME: i32 = 12;
    pub const SW_SIZE: i32 = 17;
    pub const SW_FORMAT: i32 = 18;
    pub const SW_STRIDE: i32 = 19;
    pub const SW_POINTER: i32 = 20;
    /// `mpv_render_context_update()` 返回位：有新帧可渲染。
    pub const UPDATE_FRAME: u64 = 1;
    /// `MPV_RENDER_API_TYPE_SW`（render.h）——本计划采用的唯一后端。
    pub const API_TYPE_SW: &[u8] = b"sw\0";
}

/// `mpv_event`（client.h）——只需 `event_id` 与 `error`。
#[repr(C)]
pub struct MpvEvent {
    pub event_id: c_int,
    pub error: c_int,
    pub reply_userdata: u64,
    pub data: *mut c_void,
}

/// `mpv_format`（client.h）——get/set_property 的数据类型标签。
pub mod format {
    pub const NONE: i32 = 0;
    pub const STRING: i32 = 1;
    pub const OSD_STRING: i32 = 2;
    pub const FLAG: i32 = 3;
    pub const INT64: i32 = 4;
    pub const DOUBLE: i32 = 5;
}

/// `mpv_event_end_file.reason`（client.h）——区分「放完了」与「出错」。
pub mod end_file_reason {
    /// 正常播放到结尾。
    pub const EOF: i32 = 0;
    /// 被显式停止（如我们的 stop 命令）。
    pub const STOP: i32 = 2;
    /// core 关闭。
    pub const QUIT: i32 = 3;
    /// **加载/解码失败**——这是要回灌成 `onmediaerror` 的那一类。
    pub const ERROR: i32 = 4;
    /// 重定向（`--` 内部的 playlist 行为）。
    pub const REDIRECT: i32 = 5;
}

/// `mpv_event_id` 的取值（client.h）——本计划用到的子集。
pub mod event_id {
    pub const NONE: i32 = 0;
    pub const LOG_MESSAGE: i32 = 2;
    pub const END_FILE: i32 = 7;
    pub const FILE_LOADED: i32 = 8;
    pub const IDLE: i32 = 11;
    pub const VIDEO_RECONFIG: i32 = 17;
    pub const AUDIO_RECONFIG: i32 = 18;
    pub const SEEK: i32 = 20;
    pub const PLAYBACK_RESTART: i32 = 21;
    pub const PROPERTY_CHANGE: i32 = 22;
}

/// 解析结果：要么拿到一个库路径，要么**明确地**什么都没有。
///
/// 注意 [`resolve_library`] 返回 `None` 不是错误——「本机没装 mpv」是正常情形，
/// 调用方据此走诚实降级（`render_support.rs` 的 fallback），不 panic、不黑屏。
pub fn resolve_library() -> Option<PathBuf> {
    resolve_library_with(
        std::env::var_os(ENV_MPV_LIB).as_deref(),
        std::env::current_exe()
            .ok()
            .as_deref()
            .and_then(Path::parent),
    )
}

/// [`resolve_library`] 的**纯函数**形式（便于测试，不读进程环境/不依赖 exe 位置）：
///
/// 1. `env_value` —— `AUTO_MPV_LIB` 指定的**文件**路径（存在才采用）；
/// 2. `exe_dir`   —— 可执行文件同目录下的 [`LIB_NAME`]；
/// 3. 都没有 → `None`（**不**去碰系统搜索路径：那会让「装没装」变得不可预期；
///    要系统路径就把 `AUTO_MPV_LIB` 指过去）。
pub fn resolve_library_with(env_value: Option<&OsStr>, exe_dir: Option<&Path>) -> Option<PathBuf> {
    if let Some(v) = env_value {
        let p = PathBuf::from(v);
        if p.is_file() {
            return Some(p);
        }
        // 显式指定了却不存在：不再默默回落到别处——否则「指错了」会表现为
        // 「莫名用了别的版本」。返回 None，让调用方看到降级。
        return None;
    }
    let dir = exe_dir?;
    let cand = dir.join(LIB_NAME);
    cand.is_file().then_some(cand)
}

/// 加载运行库失败的原因。
#[derive(Debug, Clone)]
pub enum MpvLoadError {
    /// 缺某个符号——版本太旧或不是 libmpv。
    MissingSymbol { symbol: String, detail: String },
    /// 库本身加载不起来（位数不符、依赖缺失、文件损坏…）。
    LoadFailed { path: PathBuf, detail: String },
}

impl std::fmt::Display for MpvLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSymbol { symbol, detail } => {
                write!(f, "libmpv 缺少符号 {symbol}：{detail}")
            }
            Self::LoadFailed { path, detail } => {
                write!(f, "加载 libmpv 失败（{}）：{detail}", path.display())
            }
        }
    }
}

impl std::error::Error for MpvLoadError {}

/// libmpv 导出的、本计划用到的全部 C 函数指针。
///
/// 全是 `unsafe extern "C" fn`——调用方（[`super::engine`]）负责满足各函数的
/// 前置条件（空指针、字符串生命周期、线程约定）。
#[derive(Clone, Copy)]
pub struct MpvSymbols {
    pub create: unsafe extern "C" fn() -> *mut c_void,
    pub initialize: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub set_option_string:
        unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int,
    /// `mpv_get_property(handle, name, mpv_format, void*)`。
    pub get_property: unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut c_void) -> c_int,
    /// `mpv_set_property(handle, name, mpv_format, const void*)`。
    pub set_property: unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut c_void) -> c_int,
    /// `mpv_get_property_string(handle, name)` —— 返回 mpv 分配的字符串，
    /// 调用方**必须**用 [`MpvSymbols::free`] 释放。
    pub get_property_string:
        unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_char,
    /// `mpv_free(void*)` —— 释放 mpv 分配的返回值（只用于本模块拿到的指针）。
    pub free: unsafe extern "C" fn(*mut c_void),
    /// `mpv_get_time_us(handle)` —— 单调递增的墙钟微秒，用作 A/V 与帧节奏的时基。
    pub get_time_us: unsafe extern "C" fn(*mut c_void) -> i64,
    pub command: unsafe extern "C" fn(*mut c_void, *const *const c_char) -> c_int,
    pub wait_event: unsafe extern "C" fn(*mut c_void, c_double) -> *mut MpvEvent,
    pub error_string: unsafe extern "C" fn(c_int) -> *const c_char,
    pub client_api_version: unsafe extern "C" fn() -> c_ulong,
    pub terminate_destroy: unsafe extern "C" fn(*mut c_void),
    pub render_context_create:
        unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *const RenderParam) -> c_int,
    pub render_context_free: unsafe extern "C" fn(*mut c_void),
    pub render_context_render: unsafe extern "C" fn(*mut c_void, *const RenderParam) -> c_int,
    pub render_context_update: unsafe extern "C" fn(*mut c_void) -> u64,
}

/// 一个已加载的 libmpv 实例：持有 [`libloading::Library`] 与已解析的 [`MpvSymbols`]。
///
/// 只要它活着，[`MpvSymbols`] 里的函数地址就一直有效——这是把符号拷成裸指针
/// 的前提，也是 [`super::engine::MpvEngine`] 必须持有它（`Arc`）的原因。
pub struct MpvApi {
    // Drop 顺序即声明顺序：先丢符号表（无 Drop），再卸载库。实际上 `symbols`
    // 是裸指针无所谓，但库必须**晚于**所有句柄卸载——由引擎持有 Arc 保证。
    pub symbols: MpvSymbols,
    _lib: libloading::Library,
    path: PathBuf,
}

impl MpvApi {
    /// 从指定路径加载并解析全部符号。
    pub fn load(path: &Path) -> Result<Self, MpvLoadError> {
        // SAFETY: 只是把动态库映射进地址空间。符号地址的有效期由本结构体持有的
        // `Library` 担保；我们不在加载期调用任何东西（调用在 engine 里，且逐个
        // 遵守各函数的前置条件）。
        let lib = unsafe { libloading::Library::new(path) }.map_err(|e| MpvLoadError::LoadFailed {
            path: path.to_path_buf(),
            detail: e.to_string(),
        })?;

        // SAFETY: 每次 `get` 取的都是该符号的正确签名（照 include/mpv/*.h 抄写）。
        // `*sym` 解引用得到函数指针（Copy），因此拷贝出来后不再借用 `lib`。
        macro_rules! sym {
            ($name:literal, $ty:ty) => {{
                let s: libloading::Symbol<$ty> =
                    unsafe { lib.get($name) }.map_err(|e| MpvLoadError::MissingSymbol {
                        symbol: String::from_utf8_lossy(
                            &$name[..$name.len().saturating_sub(1)],
                        )
                        .into_owned(),
                        detail: e.to_string(),
                    })?;
                *s
            }};
        }

        let symbols = MpvSymbols {
            create: sym!(b"mpv_create\0", unsafe extern "C" fn() -> *mut c_void),
            initialize: sym!(
                b"mpv_initialize\0",
                unsafe extern "C" fn(*mut c_void) -> c_int
            ),
            set_option_string: sym!(
                b"mpv_set_option_string\0",
                unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int
            ),
            get_property: sym!(
                b"mpv_get_property\0",
                unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut c_void) -> c_int
            ),
            set_property: sym!(
                b"mpv_set_property\0",
                unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut c_void) -> c_int
            ),
            get_property_string: sym!(
                b"mpv_get_property_string\0",
                unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_char
            ),
            free: sym!(b"mpv_free\0", unsafe extern "C" fn(*mut c_void)),
            get_time_us: sym!(
                b"mpv_get_time_us\0",
                unsafe extern "C" fn(*mut c_void) -> i64
            ),
            command: sym!(
                b"mpv_command\0",
                unsafe extern "C" fn(*mut c_void, *const *const c_char) -> c_int
            ),
            wait_event: sym!(
                b"mpv_wait_event\0",
                unsafe extern "C" fn(*mut c_void, c_double) -> *mut MpvEvent
            ),
            error_string: sym!(
                b"mpv_error_string\0",
                unsafe extern "C" fn(c_int) -> *const c_char
            ),
            client_api_version: sym!(
                b"mpv_client_api_version\0",
                unsafe extern "C" fn() -> c_ulong
            ),
            terminate_destroy: sym!(
                b"mpv_terminate_destroy\0",
                unsafe extern "C" fn(*mut c_void)
            ),
            render_context_create: sym!(
                b"mpv_render_context_create\0",
                unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *const RenderParam) -> c_int
            ),
            render_context_free: sym!(
                b"mpv_render_context_free\0",
                unsafe extern "C" fn(*mut c_void)
            ),
            render_context_render: sym!(
                b"mpv_render_context_render\0",
                unsafe extern "C" fn(*mut c_void, *const RenderParam) -> c_int
            ),
            render_context_update: sym!(
                b"mpv_render_context_update\0",
                unsafe extern "C" fn(*mut c_void) -> u64
            ),
        };

        Ok(Self {
            symbols,
            _lib: lib,
            path: path.to_path_buf(),
        })
    }

    /// 解析并加载 [`resolve_library`] 找到的库；没有则 `Ok(None)`（**正常降级**）。
    pub fn resolve_and_load() -> Result<Option<Self>, MpvLoadError> {
        match resolve_library() {
            Some(p) => Self::load(&p).map(Some),
            None => Ok(None),
        }
    }

    /// 实际加载的运行库路径（诊断用）。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `MPV_CLIENT_API_VERSION`（major << 16 | minor）。
    ///
    /// 用途之一是判断「render context 是否要求同线程 create/free」：
    /// `render.h` 记载 API < 1.105（mpv < 0.30）才有该约束。
    pub fn client_api_version(&self) -> u32 {
        // SAFETY: 无参纯查询函数。
        unsafe { (self.symbols.client_api_version)() as u32 }
    }

    /// 把 `c_int` 错误码翻译成 mpv 自己的可读文案。
    pub fn error_text(&self, code: c_int) -> String {
        // SAFETY: mpv_error_string 对任意 int 都返回静态字符串（未知码也有兜底）。
        unsafe {
            let p = (self.symbols.error_string)(code);
            if p.is_null() {
                format!("错误码 {code}")
            } else {
                std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        }
    }
}

impl std::fmt::Debug for MpvApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpvApi")
            .field("path", &self.path)
            .field("api_version", &self.client_api_version())
            .finish()
    }
}

/// 组装 `mpv_render_param` 数组（以 `type = INVALID` 结尾，header 约定）。
///
/// 单独提供是因为调用方常在循环里按帧构造参数表（T-17）。
pub struct ParamList(Vec<RenderParam>);

impl ParamList {
    pub fn new() -> Self {
        ParamList(vec![RenderParam {
            type_: render_param::INVALID,
            data: std::ptr::null_mut(),
        }])
    }

    /// 追加一个参数（插入到结尾哨兵之前）。
    ///
    /// `data` 必须是**在 `mpv_render_*` 调用期间保持有效**的指针；
    /// 调用方负责把被指向的值绑定到足够长的生命周期上。
    pub fn push(mut self, type_: c_int, data: *mut c_void) -> Self {
        let n = self.0.len();
        self.0.insert(n - 1, RenderParam { type_, data });
        self
    }

    pub fn as_ptr(&self) -> *const RenderParam {
        self.0.as_ptr()
    }
}

impl Default for ParamList {
    fn default() -> Self {
        Self::new()
    }
}

/// 把 Rust `&str` 转成 mpv 需要的 NUL 结尾 C 字符串。
///
/// 返回 `Err` 当且仅当字符串内部含 NUL（mpv 的参数不能带 NUL）。
pub fn cstring(s: &str) -> Result<std::ffi::CString, String> {
    std::ffi::CString::new(s).map_err(|_| format!("字符串含 NUL 字节，无法传给 mpv：{s:?}"))
}
