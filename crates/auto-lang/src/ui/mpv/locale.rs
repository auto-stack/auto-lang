//! mpv 的 C 环境前提：`LC_NUMERIC` 必须是 `"C"`。
//!
//! `include/mpv/client.h:147` 明写这条要求，并且在同一 header 的
//! `mpv_create()` 文档里把它列为**返回 NULL 的原因之一**（`:479`）：
//!
//! > The LC_NUMERIC locale category must be set to "C". If your program calls
//! > setlocale(), be sure not to use LC_ALL, or if you do, reset LC_NUMERIC to
//! > its sane default: setlocale(LC_NUMERIC, "C").
//!
//! 为什么需要我们主动做：Rust std **从不调用** `setlocale`，但进程里其它 C 库
//! （本仓就有 oniguruma / sqlite3 等 native 依赖）可能调 `setlocale(LC_ALL, "")`，
//! 而那是**进程级**状态，会把 LC_NUMERIC 一起改掉。mpv 的选项解析依赖 C 数值
//! 格式（小数点必须是 `.`），被改掉就会解析失败。
//!
//! 因此进入 mpv 之前**防御性**钉一次，并且**只动 LC_NUMERIC**——不用 `LC_ALL`，
//! 以免连带改掉调用方的其它分类。把 LC_NUMERIC 钉在 `"C"` 对进程是安全的：
//! Rust std 的浮点解析/格式化（`str::parse::<f64>`、`format!`）不经 locale。
//!
//! 用 CRT 自己的 `setlocale`（Rust 已链接 CRT，**零新增依赖**；本仓没有 `libc`）。

use std::ffi::{c_char, c_int, CStr};

// LC_NUMERIC 的数值由各 CRT 的 locale.h 定义，**平台相关**，不能凭印象写：
//   MSVC/ucrt + mingw(msvcrt)：LC_ALL=0, LC_COLLATE=1, LC_CTYPE=2, LC_MONETARY=3, LC_NUMERIC=4, LC_TIME=5
//   glibc / musl          ：LC_CTYPE=0, LC_NUMERIC=1, LC_TIME=2, LC_COLLATE=3, LC_MONETARY=4, LC_MESSAGES=5
#[cfg(windows)]
const LC_NUMERIC: c_int = 4;
#[cfg(not(windows))]
const LC_NUMERIC: c_int = 1;

extern "C" {
    /// `category` 传 `LC_NUMERIC`、`locale` 传空指针时是**查询**当前值。
    fn setlocale(category: c_int, locale: *const c_char) -> *mut c_char;
}

/// 把 `LC_NUMERIC` 设为 `"C"`，返回是否成功。
///
/// 幂等；应在每次 `mpv_create()` 之前调用（`MpvEngine::new` 已代为调用）。
pub fn ensure_c_numeric_locale() -> bool {
    // SAFETY: 只传静态 C 字符串与合法分类常量；setlocale 是线程安全的查询/设置，
    // 返回值在下次调用前有效，此处不保留指针（仅判空）。
    unsafe { !setlocale(LC_NUMERIC, c"C".as_ptr()).is_null() }
}

/// 读回当前 `LC_NUMERIC`（供自检与测试断言；正常应为 `"C"`）。
pub fn numeric_locale() -> Option<String> {
    // SAFETY: 空指针是 setlocale 的查询形式，返回的指针由 CRT 拥有且在下一次
    // setlocale 调用前有效——这里立刻拷成 String，不留悬垂引用。
    unsafe {
        let p = setlocale(LC_NUMERIC, std::ptr::null());
        if p.is_null() {
            None
        } else {
            Some(CStr::from_ptr(p).to_string_lossy().into_owned())
        }
    }
}

/// `LC_NUMERIC` 是否已是 mpv 要求的 `"C"`。
pub fn numeric_locale_is_c() -> bool {
    matches!(numeric_locale().as_deref(), Some("C"))
}
