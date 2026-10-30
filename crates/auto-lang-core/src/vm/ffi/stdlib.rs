//! Plan 094: Built-in stdlib functions using FFI
//!
//! This module provides high-level FFI functions for common operations
//! like file I/O, environment variables, time, path manipulation, and string operations.
//!
//! These functions use the VMConvertible trait for automatic type conversion.

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::ffi::convert::VMConvertible;
use crate::vm::ffi::rust_stdlib::RustStdlibObject;
use crate::vm::task::AutoTask;
use std::any::Any;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener as StdTcpListener, TcpStream as StdTcpStream};
use std::path::Path;
use std::path::PathBuf as StdPathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Debug logging macro - only prints when VM debug mode is enabled
macro_rules! vm_debug {
    ($($arg:tt)*) => {
        if crate::is_vm_debug() {
            eprintln!($($arg)*);
        }
    };
}

// PLAN-814 T-02 迁移勘误：本组测试仅依赖 SDK 面（auto_media_core +
// ffi stdlib 内部），原 ui-iced 门为旧 crate feature 残留——拆除恢复选择。
#[cfg(test)]
mod image_pipeline_vm_http_tests {
    use super::*;

    #[test]
    fn image_natives_register_opaque_ticket_operations() {
        assert_eq!(IMAGE_NATIVE_NAMES.len(), 17);
        assert!(IMAGE_NATIVE_NAMES
            .iter()
            .all(|name| name.starts_with("auto.image.")));
    }
}

// ============================================================================
// Native Function IDs (1000-4999 for built-in stdlib)
// ============================================================================

// File functions: 1000-1099
pub const NATIVE_FILE_READ_TEXT: u16 = 1000;
pub const NATIVE_FILE_WRITE_TEXT: u16 = 1001;
pub const NATIVE_FILE_EXISTS: u16 = 1002;
pub const NATIVE_FILE_DELETE: u16 = 1003;
pub const NATIVE_FILE_CREATE_DIR: u16 = 1004;
pub const NATIVE_FILE_READ_BYTES: u16 = 1005;
pub const NATIVE_FILE_WRITE_BYTES: u16 = 1006;
pub const NATIVE_FILE_COPY: u16 = 1007;
pub const NATIVE_FILE_SIZE: u16 = 1008;
pub const NATIVE_FILE_IS_DIR: u16 = 1009;
pub const NATIVE_FILE_WALK: u16 = 1010;
pub const NATIVE_FILE_APPEND_TEXT: u16 = 1011;
pub const NATIVE_FILE_READ_LINES: u16 = 1012;

// Env functions: 1100-1199
pub const NATIVE_ENV_GET: u16 = 1100;
pub const NATIVE_ENV_SET: u16 = 1101;
pub const NATIVE_ENV_REMOVE: u16 = 1102;
pub const NATIVE_ENV_LOCAL_DATA_DIR: u16 = 1104;
pub const NATIVE_ENV_HOME_DIR: u16 = 1105;

// Storage functions: 1106-1108 (Plan 401: localStorage-style KV store)
pub const NATIVE_STORAGE_GET: u16 = 1106;
pub const NATIVE_STORAGE_SET: u16 = 1107;
pub const NATIVE_STORAGE_REMOVE: u16 = 1108;

// IO functions: 1150-1169
pub const NATIVE_IO_READ_LINE: u16 = 1150;

// Time functions: 1200-1299
pub const NATIVE_TIME_NOW_MS: u16 = 1200;
pub const NATIVE_TIME_NOW_SEC: u16 = 1201;
pub const NATIVE_TIME_SLEEP_MS: u16 = 1202;
pub const NATIVE_TIME_NOW: u16 = 1205;

// Process functions: 1300-1399
pub const NATIVE_PROCESS_EXIT: u16 = 1300;
pub const NATIVE_PROCESS_ARGS: u16 = 1301;
pub const NATIVE_PROCESS_CURRENT_DIR: u16 = 1302;
pub const NATIVE_PROCESS_SET_CURRENT_DIR: u16 = 1303;
pub const NATIVE_PROCESS_SPAWN: u16 = 1304;
pub const NATIVE_PROCESS_SPAWN_WITH_OUTPUT: u16 = 1305;

// Math functions: 1700-1799
pub const NATIVE_MATH_ABS: u16 = 1700;
pub const NATIVE_MATH_MIN: u16 = 1701;
pub const NATIVE_MATH_MAX: u16 = 1702;
pub const NATIVE_MATH_SQRT: u16 = 1703;
pub const NATIVE_MATH_FLOOR: u16 = 1710;
pub const NATIVE_MATH_CEIL: u16 = 1711;
pub const NATIVE_MATH_ROUND: u16 = 1712;
pub const NATIVE_MATH_POW: u16 = 1713;
pub const NATIVE_MATH_MIN_F: u16 = 1714;
pub const NATIVE_MATH_MAX_F: u16 = 1715;
pub const NATIVE_MATH_SIN: u16 = 1716;
pub const NATIVE_MATH_COS: u16 = 1717;
pub const NATIVE_MATH_TAN: u16 = 1718;
pub const NATIVE_MATH_EXP: u16 = 1719;
pub const NATIVE_MATH_LN: u16 = 1720;
pub const NATIVE_MATH_LOG2: u16 = 1721;
pub const NATIVE_MATH_LOG10: u16 = 1722;
pub const NATIVE_MATH_ABS_F: u16 = 1723;
pub const NATIVE_MATH_SIGNUM: u16 = 1724;
pub const NATIVE_MATH_CLAMP: u16 = 1725;

// Log functions: 1800-1899
pub const NATIVE_LOG_DEBUG: u16 = 1800;
pub const NATIVE_LOG_INFO: u16 = 1801;
pub const NATIVE_LOG_WARN: u16 = 1802;
pub const NATIVE_LOG_ERROR: u16 = 1803;

// JSON functions: 1900-1999
pub const NATIVE_JSON_ENCODE: u16 = 1900;
pub const NATIVE_JSON_DECODE: u16 = 1901;
pub const NATIVE_JSON_PARSE: u16 = 1902;
pub const NATIVE_JSON_PRETTIFY: u16 = 1903;
pub const NATIVE_JSON_MINIFY: u16 = 1904;
pub const NATIVE_JSON_IS_VALID: u16 = 1905;
pub const NATIVE_JSON_GET: u16 = 1906;
pub const NATIVE_JSON_GET_AT: u16 = 1907;
pub const NATIVE_JSON_LEN: u16 = 1908;
pub const NATIVE_JSON_TYPE: u16 = 1909;
pub const NATIVE_JSON_AS_STRING: u16 = 1910;
pub const NATIVE_JSON_AS_NUMBER: u16 = 1911;
pub const NATIVE_JSON_AS_INT: u16 = 1912;
pub const NATIVE_JSON_AS_BOOL: u16 = 1913;
pub const NATIVE_JSON_IS_NULL: u16 = 1914;
pub const NATIVE_JSON_KEYS: u16 = 1915;
pub const NATIVE_JSON_AS_ARRAY: u16 = 1916;
pub const NATIVE_JSON_HAS_KEY: u16 = 1917;

// URL functions: 2000-2099
pub const NATIVE_URL_ENCODE: u16 = 2000;
pub const NATIVE_URL_DECODE: u16 = 2001;
pub const NATIVE_URL_ENCODE_QUERY: u16 = 2002;
pub const NATIVE_URL_DECODE_QUERY: u16 = 2003;
pub const NATIVE_URL_ENCODE_PATH_SEGMENT: u16 = 2004;
pub const NATIVE_URL_DECODE_QUERY_COMPONENT: u16 = 2005;
pub const NATIVE_URL_PARSE: u16 = 2006;
pub const NATIVE_URL_SCHEME: u16 = 2007;
pub const NATIVE_URL_HOST: u16 = 2008;
pub const NATIVE_URL_PORT: u16 = 2009;
pub const NATIVE_URL_PATH: u16 = 2010;
pub const NATIVE_URL_QUERY: u16 = 2011;
pub const NATIVE_URL_FRAGMENT: u16 = 2012;
pub const NATIVE_URL_QUERY_PARAM: u16 = 2013;
pub const NATIVE_URL_QUERY_PARAMS: u16 = 2014;
pub const NATIVE_URL_JOIN_PATH: u16 = 2015;

// Net functions: 2100-2199
pub const NATIVE_NET_TCP_BIND: u16 = 2100;
pub const NATIVE_NET_TCP_LISTENER_ACCEPT: u16 = 2101;
pub const NATIVE_NET_TCP_LISTENER_LOCAL_ADDR: u16 = 2102;
pub const NATIVE_NET_TCP_LISTENER_CLOSE: u16 = 2103;
pub const NATIVE_NET_TCP_CONNECT: u16 = 2104;
pub const NATIVE_NET_TCP_STREAM_READ: u16 = 2105;
pub const NATIVE_NET_TCP_STREAM_WRITE: u16 = 2106;
pub const NATIVE_NET_TCP_STREAM_READ_ALL: u16 = 2107;
pub const NATIVE_NET_TCP_STREAM_READ_LINE: u16 = 2108;
pub const NATIVE_NET_TCP_STREAM_WRITE_STR: u16 = 2109;
pub const NATIVE_NET_TCP_STREAM_CLOSE: u16 = 2110;
pub const NATIVE_NET_TCP_STREAM_PEER_ADDR: u16 = 2111;
pub const NATIVE_NET_TCP_STREAM_SET_READ_TIMEOUT: u16 = 2112;
pub const NATIVE_NET_TCP_STREAM_SET_WRITE_TIMEOUT: u16 = 2113;

// HTTP functions: 2200-2299
pub const NATIVE_HTTP_SERVER: u16 = 2200;
pub const NATIVE_HTTP_SERVER_GET: u16 = 2201;
pub const NATIVE_HTTP_SERVER_POST: u16 = 2202;
pub const NATIVE_HTTP_SERVER_PUT: u16 = 2203;
pub const NATIVE_HTTP_SERVER_DELETE: u16 = 2204;
pub const NATIVE_HTTP_SERVER_STATIC: u16 = 2205;
pub const NATIVE_HTTP_SERVER_LISTEN: u16 = 2206;
pub const NATIVE_HTTP_RESPONSE: u16 = 2210;
pub const NATIVE_HTTP_RESPONSE_STATUS: u16 = 2211;
pub const NATIVE_HTTP_RESPONSE_HEADER: u16 = 2212;
pub const NATIVE_HTTP_RESPONSE_TEXT: u16 = 2213;
pub const NATIVE_HTTP_RESPONSE_HTML: u16 = 2214;
pub const NATIVE_HTTP_RESPONSE_BYTES: u16 = 2215;
pub const NATIVE_HTTP_OK: u16 = 2220;
pub const NATIVE_HTTP_CREATED: u16 = 2221;
pub const NATIVE_HTTP_BAD_REQUEST: u16 = 2222;
pub const NATIVE_HTTP_NOT_FOUND: u16 = 2223;
pub const NATIVE_HTTP_INTERNAL_ERROR: u16 = 2224;
pub const NATIVE_HTTP_GET: u16 = 2230;
pub const NATIVE_HTTP_POST: u16 = 2231;
pub const NATIVE_HTTP_PUT: u16 = 2232;
pub const NATIVE_HTTP_DELETE: u16 = 2233;

// Plan 195: RequestBuilder (2234-2239)
pub const NATIVE_HTTP_REQUEST: u16 = 2234;
pub const NATIVE_HTTP_REQUEST_BUILDER_HEADER: u16 = 2235;
pub const NATIVE_HTTP_REQUEST_BUILDER_BODY: u16 = 2236;
pub const NATIVE_HTTP_REQUEST_BUILDER_TIMEOUT: u16 = 2237;
pub const NATIVE_HTTP_REQUEST_BUILDER_JSON: u16 = 2238;
pub const NATIVE_HTTP_REQUEST_BUILDER_SEND: u16 = 2239;

// Plan 195: Response access (2216-2218)
pub const NATIVE_RESPONSE_STATUS_CODE: u16 = 2216;
pub const NATIVE_RESPONSE_HEADER_GET: u16 = 2217;
pub const NATIVE_RESPONSE_BODY: u16 = 2218;
pub const NATIVE_RESPONSE_BODY_BYTES: u16 = 2225;
pub const NATIVE_RESPONSE_BODY_TO_FILE: u16 = 2226;

// Plan 152: 流式 HTTP (2240-2249)
pub const NATIVE_HTTP_GET_STREAM: u16 = 2240;
pub const NATIVE_HTTP_POST_STREAM: u16 = 2241;
pub const NATIVE_HTTP_STREAM_NEXT: u16 = 2242;
pub const NATIVE_HTTP_STREAM_IS_DONE: u16 = 2243;
pub const NATIVE_HTTP_STREAM_CLOSE: u16 = 2244;

// Plan 152: SSE Parser (2245-2249)
pub const NATIVE_SSE_EVENT_NEW: u16 = 2245;
pub const NATIVE_SSE_EVENT_DATA: u16 = 2246;
pub const NATIVE_SSE_EVENT_EVENT: u16 = 2247;
pub const NATIVE_SSE_EVENT_ID: u16 = 2248;
pub const NATIVE_SSE_EVENT_IS_DONE: u16 = 2249;
pub const NATIVE_SSE_PARSE: u16 = 2250;

// Plan 159: HTTP streaming with custom headers (2255-2259)
pub const NATIVE_HTTP_POST_STREAM_WITH_HEADERS: u16 = 2255;

// Regex functions (Plan 159): 2400-2499
pub const NATIVE_REGEX_IS_MATCH: u16 = 2400;
pub const NATIVE_REGEX_FIND_ALL: u16 = 2401;
// PLAN-053 P-053-6: web 生态静态形态（musk forge_helpers/mention_helpers
// 消息渲染链——`Regex.replace(text, pat, repl, flags)` /
// `Regex.test(text, pat)`，CALL_SPEC 路由自动.regex.replace/test）。
pub const NATIVE_REGEX_REPLACE: u16 = 2402;
pub const NATIVE_REGEX_TEST: u16 = 2403;

// Task/Msg functions (Plan 121): 2300-2399
pub const NATIVE_TASK_SPAWN: u16 = 2300;
pub const NATIVE_TASK_SEND: u16 = 2301;
pub const NATIVE_TASK_HANDLE_IS_NULL: u16 = 2302;
pub const NATIVE_TASK_HANDLE_TYPE: u16 = 2303;
pub const NATIVE_TASK_HANDLE_ID: u16 = 2304;
pub const NATIVE_TASK_SYSTEM_START: u16 = 2305;
pub const NATIVE_TASK_SYSTEM_RUN: u16 = 2306; // Plan 124: Sync bridge for async code
pub const NATIVE_TASK_SYSTEM_STOP: u16 = 2307; // Plan 127: Stop the task system scheduler
pub const NATIVE_TASK_SEND_AWAIT: u16 = 2308; // Plan 124 Phase 2.2: send().await backpressure
pub const NATIVE_TASK_ASK: u16 = 2309; // Plan 124 Phase 2.3: ask/reply RPC
pub const NATIVE_CTX_REPLY: u16 = 2310; // Plan 127: ctx.reply() for message handlers
pub const NATIVE_TASK_SINGLETON_SEND: u16 = 2311; // Plan 127: Task.send for singleton tasks

// Path functions: 1400-1499
pub const NATIVE_PATH_JOIN: u16 = 1400;
pub const NATIVE_PATH_PARENT: u16 = 1401;
pub const NATIVE_PATH_EXTENSION: u16 = 1402;
pub const NATIVE_PATH_FILENAME: u16 = 1403;
pub const NATIVE_PATH_CANONICALIZE: u16 = 1404;

// String functions: 1500-1599
pub const NATIVE_STR_LEN: u16 = 1500;
pub const NATIVE_STR_IS_EMPTY: u16 = 1501;
pub const NATIVE_STR_CHAR_AT: u16 = 1502;
pub const NATIVE_STR_SUBSTR: u16 = 1503;
pub const NATIVE_STR_CONTAINS: u16 = 1504;
pub const NATIVE_STR_STARTS_WITH: u16 = 1505;
pub const NATIVE_STR_ENDS_WITH: u16 = 1506;
pub const NATIVE_STR_TRIM: u16 = 1507;
pub const NATIVE_STR_SPLIT: u16 = 1508;
pub const NATIVE_STR_REPEAT: u16 = 1509;
pub const NATIVE_STR_REPLACE: u16 = 1510;
pub const NATIVE_STR_TO_UPPER: u16 = 1511;
pub const NATIVE_STR_TO_LOWER: u16 = 1512;
pub const NATIVE_STR_REVERSE: u16 = 1513;
pub const NATIVE_STR_FIND: u16 = 1514;
pub const NATIVE_STR_LINES: u16 = 1515;
pub const NATIVE_STR_PARSE_INT: u16 = 1516;
pub const NATIVE_STR_PARSE_FLOAT: u16 = 1517;
pub const NATIVE_STR_SPLIT_ONCE: u16 = 1518;
pub const NATIVE_STR_MATCH_COUNT: u16 = 1519;
pub const NATIVE_STR_REPLACE_FIRST: u16 = 1520;
pub const NATIVE_STR_UUID: u16 = 1521;
pub const NATIVE_STR_FROM_UINT: u16 = 1522;
pub const NATIVE_STR_TO_UINT: u16 = 1523;

// Char functions: 1600-1699
pub const NATIVE_CHAR_IS_ALPHA: u16 = 1600;
pub const NATIVE_CHAR_IS_DIGIT: u16 = 1601;
pub const NATIVE_CHAR_IS_ALPHANUM: u16 = 1602;
pub const NATIVE_CHAR_IS_WHITESPACE: u16 = 1603;
pub const NATIVE_CHAR_IS_IDENT: u16 = 1604;
pub const NATIVE_CHAR_TO_LOWER: u16 = 1605;
pub const NATIVE_CHAR_TO_UPPER: u16 = 1606;

// Option functions: 1550-1559 (Plan 200 Task 2.4)
pub const NATIVE_OPTION_OR: u16 = 1550;
pub const NATIVE_OPTION_UNWRAP_OR: u16 = 1551;

// Rust stdlib dynamic dispatch: 3000-3099 (Plan 192)
pub const NATIVE_RUST_STDLIB_DISPATCH: u16 = 3000;

// ============================================================================
// File Functions
// ============================================================================

/// Plan 673 §5 chunked-read envelope — the serde shape shared verbatim with
/// the a2r-std mirror (`a2r_std::fs::read_text_range`): same field order,
/// same EOF/error rules. P670-D1: both tracks emit byte-identical JSON.
#[derive(serde::Serialize)]
struct ReadTextRangeOut {
    text: String,
    total: i64,
    next_offset: Option<u64>,
}

fn read_text_range_json(text: String, total: i64, next_offset: Option<u64>) -> String {
    serde_json::to_string(&ReadTextRangeOut {
        text,
        total,
        next_offset,
    })
    .expect("read_text_range envelope serialization cannot fail")
}

/// Write text content to a file
// ============================================================================
// Environment Functions
// ============================================================================

// ── Plan 309 Task 1.2 P5: PATH FFI ──────────────────────────────────────
// TODO: Env.path_add/prepend/remove FFI deferred — #[rust_fn] registration
// requires BIGVM_NATIVES updates in native.rs. The shell-side env.path
// commands (cmd_env_path) already provide this functionality for interactive
// use. FFI is only needed for .at scripts calling Env.path_add() directly.

// ============================================================================
// Time Functions
// ============================================================================

// ============================================================================
// Process Functions
// ============================================================================

#[cfg(test)]
mod exit_audit_tests {
    use super::*;

    /// 临时审计路径注入（PLAN-575 测试设计：不碰全局 env）。
    fn tmp_audit_path(tag: &str) -> StdPathBuf {
        std::env::temp_dir().join(format!(
            "p575-exit-audit-{}-{}.log",
            tag,
            std::process::id()
        ))
    }

    #[test]
    fn exit_audit_to_appends_one_line_per_call() {
        let path = tmp_audit_path("append");
        let _ = fs::remove_file(&path);
        exit_audit_to(&path, 0, "main_return");
        exit_audit_to(&path, 7, "vm_process_exit");
        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2, "每次调用恰一行，实得: {content:?}");
        assert!(lines[0].contains("code=0") && lines[0].contains("site=main_return"));
        assert!(lines[1].contains("code=7") && lines[1].contains("site=vm_process_exit"));
        assert!(lines[0].contains("pid=") && lines[1].contains("pid="));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn exit_audit_to_creates_missing_parent_dir() {
        let path = tmp_audit_path("mkdir")
            .with_file_name(format!("sub-{}", std::process::id()))
            .join("audit.log");
        let _ = fs::remove_file(&path);
        exit_audit_to(&path, 0, "main_return");
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("site=main_return"), "实得: {content:?}");
        let _ = fs::remove_file(&path);
    }

    /// 挂点②：panic hook 落审计（env 注入临时路径；先装 hook 再 catch_unwind
    /// 触发 panic，链式调用保留既有 hook——G3 零行为变更）。
    #[test]
    fn panic_hook_writes_audit_line_before_prev_hook() {
        let path = tmp_audit_path("panic");
        let _ = fs::remove_file(&path);
        std::env::set_var("AUTO_DESKTOP_EXIT_LOG", &path);
        install_exit_audit_panic_hook();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            panic!("p575 audit probe");
        }));
        assert!(result.is_err(), "panic 应照常 unwind（仅追加审计）");
        std::env::remove_var("AUTO_DESKTOP_EXIT_LOG");
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("code=101") && content.contains("site=panic msg=p575 audit probe"),
            "审计行应含 code=101 与 panic 消息，实得: {content:?}"
        );
        let _ = fs::remove_file(&path);
    }

    /// PLAN-747：容错报告文本与默认 hook 同形（thread 名/位置/消息/提示行）。
    #[test]
    fn plan747_format_panic_report_shape() {
        let text = format_panic_report("main", "src\\app.rs:42:13", "boom");
        assert!(text.starts_with("thread 'main' panicked at src\\app.rs:42:13:\n"));
        assert!(text.contains("\nboom\n"));
        assert!(text.contains("note: run with `RUST_BACKTRACE=1`"));
        assert!(text.ends_with('\n'));
    }

    /// PLAN-747：backtrace 请求判定与 std 默认 hook 同口径（未设/"0" = 否，
    /// 其余 = 是）。env 恢复保证不影响后续测试。
    #[test]
    fn plan747_backtrace_requested_env_states() {
        let saved = std::env::var_os("RUST_BACKTRACE");
        std::env::remove_var("RUST_BACKTRACE");
        assert!(!backtrace_requested(), "未设 = 容错打印");
        std::env::set_var("RUST_BACKTRACE", "0");
        assert!(!backtrace_requested(), "\"0\" = 容错打印");
        std::env::set_var("RUST_BACKTRACE", "1");
        assert!(backtrace_requested(), "\"1\" = 原生 hook");
        std::env::set_var("RUST_BACKTRACE", "full");
        assert!(backtrace_requested(), "\"full\" = 原生 hook");
        match saved {
            Some(v) => std::env::set_var("RUST_BACKTRACE", v),
            None => std::env::remove_var("RUST_BACKTRACE"),
        }
    }

    /// 探针子进程模式：env 置位时真调 shim_process_exit(7)——进程在
    /// harness 上报前以码 7 退出（见父测试）。
    #[test]
    fn p575_probe_child() {
        if std::env::var("P575_PROBE_EXIT").is_ok() {
            shim_process_exit(7);
        }
    }

    /// PLAN-765 r2：IAT 追踪钩子进程模式——env 置位时装钩后裸
    /// std::process::exit(9)（不经本仓任何审计 site，模拟依赖直调形态）。
    #[test]
    fn p765_iat_probe_child() {
        if std::env::var("P765_IAT_PROBE").is_ok() {
            install_exit_process_trace();
            std::process::exit(9);
        }
    }

    /// PLAN-765 r2：裸 exit 经 IAT 钩 → 审计册落 exitproc_trace 行（含
    /// frames 偏移）且退出码原样保持 9（只加观测不改语义）。
    #[test]
    fn p765_iat_trace_writes_audit_and_preserves_exit_code() {
        let exe = std::env::current_exe().unwrap();
        let path = tmp_audit_path("p765-iat");
        let _ = fs::remove_file(&path);
        let status = std::process::Command::new(exe)
            .args(["p765_iat_probe_child", "--nocapture"])
            .env("P765_IAT_PROBE", "1")
            .env("AUTO_DESKTOP_EXIT_LOG", &path)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(9), "退出码必须保持 9（钩只加观测）");
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("exitproc_trace") && content.contains("ExitProcess(9)"),
            "IAT 追踪行缺失，实得: {content:?}"
        );
        assert!(
            content.contains("frames=[+"),
            "调用栈偏移应存在，实得: {content:?}"
        );
        let _ = fs::remove_file(&path);
    }

    /// 挂点①+②组合：shim_process_exit 落审计后退出码不变（子进程探针，
    /// PLAN-575 测试设计第 2 行）。过滤串 `p575_probe_child` 与本测试名
    /// 无子串关系，子进程里只命中探针测试本身（防递归 spawn）。
    #[test]
    fn p575_exit_code_survives_probe() {
        let exe = std::env::current_exe().unwrap();
        let path = tmp_audit_path("probe");
        let _ = fs::remove_file(&path);
        let status = std::process::Command::new(exe)
            .args(["p575_probe_child", "--nocapture"])
            .env("P575_PROBE_EXIT", "1")
            .env("AUTO_DESKTOP_EXIT_LOG", &path)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(7), "退出码必须保持 7（G3）");
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("code=7") && content.contains("site=vm_process_exit"),
            "探针审计行缺失，实得: {content:?}"
        );
        let _ = fs::remove_file(&path);
    }
}

// ============================================================================
// String Functions (ID 1500-1599)
// ============================================================================

/// PLAN-781 阶段 E（T-24）单测：UTF-8 字节口径 + 与
/// `File.read_text_range`（nat#1016）游标语义的一致性——章 meta 索引
/// 的偏移累计与窗读共用同一字节空间。
#[cfg(test)]
mod plan781_byte_len_tests {
    use super::*;

    #[test]
    fn plan781_str_byte_len_utf8_bytes() {
        assert_eq!(shim_str_byte_len(String::new()), 0);
        assert_eq!(shim_str_byte_len("abc".into()), 3);
        // 与 `.length`（nat#170，PLAN-055 字符语义）互补：中文 2 字符 = 6
        // 字节（PLAN-872 订正：字符语义的生效名是 `.length` 属性，
        // `str.len()` 生效实现同为字节数，id 1500）。
        assert_eq!(shim_str_byte_len("中文".into()), 6);
        assert_eq!(shim_str_byte_len("a中b".into()), 5);
        // emoji 4 字节（代理对之外的合法 UTF-8 面）。
        assert_eq!(shim_str_byte_len("😀".into()), 4);
        // 换行/空白按 1 字节计（行偏移累计依赖）：3×3+1+3×3+1 = 20。
        assert_eq!(shim_str_byte_len("第一行\n第二行\n".into()), 20);
        // 双轨 parity：a2r 镜像（vue 模式转译后端消费同一语义）。
        assert_eq!(a2r_std::str::str_byte_len("中文"), 6);
        assert_eq!(
            a2r_std::str::str_byte_len("第一行\n第二行\n"),
            shim_str_byte_len("第一行\n第二行\n".into())
        );
    }

    #[test]
    fn plan781_str_byte_len_matches_read_text_range_cursor() {
        let dir = std::env::temp_dir().join("auto_lang_plan781_byte_len");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("cjk.txt");
        let body = "第1章 标题\n正文中文两行\n";
        std::fs::write(&p, body).unwrap();
        let total = shim_str_byte_len(body.to_string());
        // 整窗读：envelope 的 total（文件字节数）必须与 byte_len 一致；
        // 读到真 EOF 时 next_offset=null。
        let env = shim_file_read_text_range(p.to_str().unwrap().to_string(), 0, total + 16);
        assert!(
            env.contains(&format!("\"total\":{total}")),
            "envelope total == str.byte_len: {env}"
        );
        assert!(env.contains("\"next_offset\":null"), "真 EOF: {env}");
        // 半窗读：next_offset = byte cursor（=窗内文本的 byte_len）。
        let half = shim_file_read_text_range(
            p.to_str().unwrap().to_string(),
            0,
            shim_str_byte_len("第1章 标题\n".to_string()),
        );
        assert!(
            half.contains(&format!(
                "\"next_offset\":{}",
                shim_str_byte_len("第1章 标题\n".to_string())
            )),
            "next_offset == 窗文本 byte_len: {half}"
        );
        // 中文字节界内偏移回退到边界后仍完整无半字（meta 章窗行对齐前提）。
        let mid = shim_str_byte_len("第1章 ".to_string()) + 1; // 落在「标」字中间
        let env2 = shim_file_read_text_range(p.to_str().unwrap().to_string(), mid, 6);
        // 回退语义（PLAN-687）：start 回退到前边界、窗尾回退——产出完整
        // 字符（"标题\n"），绝不半字。
        assert!(
            env2.contains("标题"),
            "mid-char 回退到边界产出完整字: {env2}"
        );
        let _ = std::fs::remove_file(&p);
    }
}

// ============================================================================
// Character Functions (ID 1600-1699)
// ============================================================================

// ============================================================================
// Math Functions (ID 1700-1799)
// ============================================================================

// ============================================================================
// JSON Functions (ID 1900-1999)
// ============================================================================

// ============================================================================
// URL Functions (ID 2000-2099)
// ============================================================================

// ============================================================================
// Plan 195: RequestBuilder FFI
// ============================================================================

// ============================================================================
// Plan 350: TLS configuration natives
// ============================================================================

// ============================================================================
// Plan 349: Multipart file upload natives
// ============================================================================
// Plan 349 step 8: Cookie / retry / compression natives
// ============================================================================

// ============================================================================

// ============================================================================
// Essential File System VM Shims
// ============================================================================

// ============================================================================
// SQLite VM Shims (Plan 744 / D-VM-SQLITE)
// ============================================================================
// The VM stack carries an opaque i64 handle (image-module opaque-ticket
// precedent). Connections belong to the VM instance and opening task; HTTP
// request task cleanup releases them. SQL execution is delegated to
// a2r_std::sqlite so both tracks share one engine and sentinel conventions.

// ============================================================================
// Registration Function
// ============================================================================

/// Register all stdlib FFI functions with the NativeInterface
/// Register manual shims that cannot use #[rust_fn] (custom VM access, variadic args, etc.).
///
/// #[rust_fn]-annotated functions are auto-registered via inventory (build_from_inventory).
/// This function only registers the ~54 manual shims that need special handling.
pub fn register_request_io_shims(natives: &mut crate::vm::native::NativeInterface) {
    natives.register_shim_by_name("auto.io.read_line", shim_io_read_line_request);
    natives.register_shim_by_name("IO.read_line", shim_io_read_line_request);
    natives.register_shim_by_name("io.read_line", shim_io_read_line_request);
    natives.declare_contract_identity("auto.io.read_line", &[], &[], "str");
}

pub fn register_stdlib_ffi(natives: &mut crate::vm::native::NativeInterface) {
    // File utility functions (walk/read_lines/is_binary now via #[rust_fn] registration)
    // PLAN-884 T-04：按域拆分注册（连续切片，调用序=原语句序——plan872 有效绑定序载荷）。
    register_string_find_shims(natives);
    register_process_spawn_shims(natives);
    register_sys_exec_option_shims(natives);
    register_net_tcp_shims(natives);
    register_http_server_shims(natives);
    register_sqlite_shims(natives);
    register_image_shims(natives);
    register_http_server_feature_shims(natives);
    register_http_client_builder_shims(natives);
    register_json_value_shims(natives);
    register_json_http_helpers_shims(natives);
    register_sse_stream_shims(natives);
    register_response_stream_shims(natives);
    register_regex_i18n_shims(natives);
    register_task_run_shims(natives);
    register_rust_stdlib_dispatch_shims(natives);
    register_warn_noop_shims(natives);
    register_test_runner_shims(natives);
    register_io_stream_shims(natives);
    // Plan 442 C2: musk backend extern response-constructor shims. The bare
    // names resolve via NATIVE_ID_MAP (native_catalog) → fixed ids here; the
    // shims build HttpResponseData handles served by the response-object path.
    use crate::vm::ffi::musk_response_ctor::*;
    natives.register_shim_by_name("ok_response", shim_ok_response);
    natives.register_shim_by_name("err_response", shim_err_response);
    natives.register_shim_by_name("json_response", shim_json_response);
    natives.register_shim_by_name("error_response", shim_error_response);
    natives.register_shim_by_name("text_response", shim_text_response);
    natives.register_shim_by_name("empty_response", shim_empty_response);
    natives.register_shim_by_name("err_json_response", shim_err_json_response);
    natives.register_shim_by_name("to_response", shim_to_response);
    natives.register_shim_by_name("resp_is_err", shim_resp_is_err);
    natives.register_shim_by_name("resp_err_code", shim_resp_err_code);
    natives.register_shim_by_name("resp_err_message", shim_resp_err_message);
    natives.register_shim_by_name("sse_named_event", shim_sse_named_event);
    natives.register_shim_by_name("sse_event", shim_sse_event);
    natives.register_shim_by_name("sse_plain_event", shim_sse_plain_event);

    // Plan 442 C2 item ③ path (b): pure-logic value accessor externs.
    natives.register_shim_by_name("value_get_str", shim_value_get_str);
    natives.register_shim_by_name("value_get_bool", shim_value_get_bool);
    natives.register_shim_by_name("value_is_null", shim_value_is_null);
    natives.register_shim_by_name("value_get", shim_value_get);
    natives.register_shim_by_name("value_get_array", shim_value_get_array);

    // Pure utility externs (random IDs/hex).
    natives.register_shim_by_name("new_id", shim_new_id);
    natives.register_shim_by_name("random_hex", shim_random_hex);
    natives.register_shim_by_name("hash_password", shim_hash_password);
    natives.register_shim_by_name("path_inner", shim_path_inner);

    // Data-source extern (path (a) forwarding; default = constant).
    natives.register_shim_by_name(
        "app_config_effective_daemon_url",
        shim_app_config_effective_daemon_url,
    );
    // Value-returning data extern (path (a) forwarding; default = empty store).
    natives.register_shim_by_name("relay_runs_list", shim_relay_runs_list);
    // PLAN-044: generic gate for the whole musk extern surface — the
    // extern_sigs.at stub bodies call musk_extern_dispatch(name, args).
    natives.register_shim_by_name("musk_extern_dispatch", shim_musk_extern_dispatch);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod e4_default_http_tests {
    use super::*;

    fn reset_defaults() {
        let mut reg = http_defaults().lock().unwrap();
        reg.headers.clear();
        reg.queries.clear();
    }

    // NOTE: 两断言面共(Process 级)注册表,并行测试互踩——并入 e2e 单测试串行执行。
    #[test]
    #[ignore = "merged into default_headers_reach_wire_on_plain_get (shared global registry)"]
    fn default_query_append_and_replace() {
        reset_defaults();
        {
            let mut reg = http_defaults().lock().unwrap();
            http_set_default_entry(&mut reg.queries, "workspace".into(), "w1".into());
            http_set_default_entry(&mut reg.queries, "workspace".into(), "w2".into());
            http_set_default_entry(&mut reg.queries, "tab".into(), "a b".into());
        }
        assert_eq!(
            append_default_queries("http://x/api"),
            "http://x/api?workspace=w2&tab=a b"
        );
        assert_eq!(
            append_default_queries("http://x/api?p=1"),
            "http://x/api?p=1&workspace=w2&tab=a b"
        );
        reset_defaults();
        assert_eq!(append_default_queries("http://x/api"), "http://x/api");
    }

    /// E4 端到端:真实 TcpListener 服务断言 Http.get 收到默认头与默认 query。
    /// 直接驱动 spawn_async_http_handle/spawn_async_http(两组 send 汇聚点,
    /// 公开 native shim——plain handle 族与 get_json/post_json 族——登记后
    /// 全部经此处落线,默认头/query/body 的 wire 覆盖即公开入口覆盖)。
    /// PLAN-724 T-02(P707-R1 根因修复):managed 提交必须先 register_live_op
    /// 再 submit(生产 shim 协议);未登记令牌会被 707 取消竞态守卫中止
    /// (submit_client_job 的 still-live 闭合)——曾致本测试 30s 恒红。
    /// 断言后按协议消费/取消配对清理,并在末尾钉住未登记反例。
    #[test]
    fn default_headers_reach_wire_on_plain_get() {
        reset_defaults();
        // 单元面:同名替换 + 多键追加 + 已含 ? 分隔
        {
            let mut reg = http_defaults().lock().unwrap();
            http_set_default_entry(&mut reg.queries, "workspace".into(), "w1".into());
            http_set_default_entry(&mut reg.queries, "workspace".into(), "w2".into());
        }
        assert_eq!(
            append_default_queries("http://x/api"),
            "http://x/api?workspace=w2"
        );
        assert_eq!(
            append_default_queries("http://x/api?p=1"),
            "http://x/api?p=1&workspace=w2"
        );
        reset_defaults();
        {
            let mut reg = http_defaults().lock().unwrap();
            http_set_default_entry(
                &mut reg.headers,
                "Authorization".into(),
                "Bearer t-ok".into(),
            );
            http_set_default_entry(&mut reg.queries, "workspace".into(), "ws-9".into());
        }
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        // fix-test-stability: accept/join 无截止会无限挂（2026-09-30 双 run
        // 现行实录——异步 op 未落线时 mock 永远等不到连接）。channel+
        // recv_timeout 定界，超时即红并给出可归因信息。
        let (tx_req, rx_req) = std::sync::mpsc::channel::<String>();
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = s.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let resp = "HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: 2

{}";
            s.write_all(resp.as_bytes()).unwrap();
            let _ = tx_req.send(req);
        });
        // 生产协议:先登记 live 令牌再提交(否则被取消竞态守卫中止,见 707 T-02)。
        crate::vm::ffi::async_http::register_live_op(999_001);
        spawn_async_http_handle(
            "GET".into(),
            format!("http://127.0.0.1:{port}/api/probe"),
            None,
            999_001,
        );
        let req = rx_req
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("e4 plain 臂 30s 内 mock 未收到请求——异步 http op 未落线");
        assert!(
            req.contains("GET /api/probe?workspace=ws-9 "),
            "query missing: {req}"
        );
        let req_lc = req.to_ascii_lowercase();
        assert!(
            req_lc.contains("authorization: bearer t-ok"),
            "header missing: {req}"
        );
        // 消费配对:managed job 完成必须以 Structured 200 落表(登记→完成→消费)。
        let plain_result = e4_wait_live_op(999_001);
        assert!(
            matches!(
                &plain_result,
                Ok(AsyncResult::Structured { status: 200, .. })
            ),
            "e4 plain 臂 managed 完成形态不符(status={:?}, err={})",
            match &plain_result {
                Ok(AsyncResult::Structured { status, .. }) => Some(*status),
                Ok(AsyncResult::Auth { status, .. }) => Some(*status as u16),
                _ => None,
            },
            match &plain_result {
                Err(e) => e.clone(),
                _ => String::new(),
            }
        );
        crate::vm::ffi::async_http::cancel_live_op(999_001); // 幂等清理
        reset_defaults();

        // PLAN-048 T2 (musk VM 数据桥): auto.http.get_json / post_json —— 即
        // emit_api_http_call 契约改写所用的两条臂 —— 必须同样应用 446-E4
        // 默认头/默认查询。修复前只接在通用 request 汇聚点
        // (spawn_async_http_handle),get_json/post_json 走 simple_http_json
        // 裸发 → musk VM 前端契约调用缺 Bearer+workspace 全线 401。
        // 本段并入同一串行测试(共享进程级注册表,并行必互踩——见本文件
        // e4 模块头注),直接驱动 spawn_async_http(Body 汇聚点)。
        {
            let mut reg = http_defaults().lock().unwrap();
            http_set_default_entry(
                &mut reg.headers,
                "Authorization".into(),
                "Bearer t-json".into(),
            );
            http_set_default_entry(&mut reg.queries, "workspace".into(), "ws-j".into());
        }
        // GET 臂(chats_list_sessions 形态)。
        let listener_json = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port_json = listener_json.local_addr().unwrap().port();
        let (tx_json, rx_json) = std::sync::mpsc::channel::<String>();
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            let (mut s, _) = listener_json.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = s.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let resp =
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
            s.write_all(resp.as_bytes()).unwrap();
            let _ = tx_json.send(req);
        });
        crate::vm::ffi::async_http::register_live_op(999_101);
        spawn_async_http(
            "GET".into(),
            format!("http://127.0.0.1:{port_json}/api/chats/sessions"),
            None,
            999_101,
        );
        let req_json = rx_json
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("e4 get_json 臂 30s 内 mock 未收到请求——异步 http op 未落线");
        assert!(
            req_json.contains("GET /api/chats/sessions?workspace=ws-j "),
            "get_json query missing: {req_json}"
        );
        assert!(
            req_json
                .to_ascii_lowercase()
                .contains("authorization: bearer t-json"),
            "get_json header missing: {req_json}"
        );
        crate::vm::ffi::async_http::cancel_live_op(999_101);
        // POST 臂(auth_login 形态):默认头/查询同在 + body 完整落线。
        let listener_post = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port_post = listener_post.local_addr().unwrap().port();
        let (tx_post, rx_post) = std::sync::mpsc::channel::<String>();
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            let (mut s, _) = listener_post.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = s.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let resp =
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
            s.write_all(resp.as_bytes()).unwrap();
            let _ = tx_post.send(req);
        });
        crate::vm::ffi::async_http::register_live_op(999_102);
        spawn_async_http(
            "POST".into(),
            format!("http://127.0.0.1:{port_post}/api/auth/login"),
            Some(r#"{"username":"u","password":"p"}"#.into()),
            999_102,
        );
        let req_post = rx_post
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("e4 post_json 臂 30s 内 mock 未收到请求——异步 http op 未落线");
        assert!(
            req_post.contains("POST /api/auth/login?workspace=ws-j "),
            "post_json query missing: {req_post}"
        );
        assert!(
            req_post
                .to_ascii_lowercase()
                .contains("authorization: bearer t-json"),
            "post_json header missing: {req_post}"
        );
        assert!(
            req_post.contains(r#"{"username":"u","password":"p"}"#),
            "post_json body missing: {req_post}"
        );
        crate::vm::ffi::async_http::cancel_live_op(999_102);
        reset_defaults();

        // 未登记反例(协议钉,PLAN-724 T-02):不经 register_live_op 直接提交
        // managed job → 707 取消竞态守卫在 submit 返回前补 abort(句柄安装即
        // 回收),迟到完成因令牌缺席被 presence 守卫丢弃、不复活不完成。
        // 连接是否发出取决于 abort 与首 await 的竞态,不作 wire 断言(非确定);
        // 确定面 = 句柄表回基线 + 无令牌永无可消费结果。
        let before = crate::vm::ffi::async_http::job_abort_count();
        let submitted = spawn_async_http(
            "GET".into(),
            format!("http://127.0.0.1:{port_post}/api/unregistered"),
            None,
            999_201,
        );
        assert!(submitted, "未登记臂:队列未满时提交必须被接受");
        assert_eq!(
            crate::vm::ffi::async_http::job_abort_count(),
            before,
            "未登记 id 的 abort 句柄必须在 submit 返回前被取消守卫回收"
        );
        let settle = std::time::Instant::now() + std::time::Duration::from_millis(300);
        while std::time::Instant::now() < settle {
            assert!(
                crate::vm::ffi::async_http::take_live_op(999_201).is_none(),
                "未登记令牌不得产出可消费结果(presence 守卫被绕过)"
            );
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        reset_defaults();
    }

    /// e4 专用:有界等待 managed job 完成并消费(登记→完成→消费协议的消费臂)。
    fn e4_wait_live_op(req_id: u64) -> Result<AsyncResult, String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(r) = crate::vm::ffi::async_http::take_live_op(req_id) {
                return r;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "e4 臂 5s 内 managed job 未完成(req_id={req_id})"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

// P487-2 债家族的测试面收敛：存储测试共用进程级 STORAGE_MAP 与
// AUTO_VM_STORAGE_FILE env，并行线程互踩。凡触碰 storage 的测试
// （含 renderer 侧 t2_isolate_storage 家族）先取此锁串行化。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
static STORAGE_TEST_SERIALIZER: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 取存储测试串行锁（持有至返回的 guard 亡）。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
pub fn lock_storage_for_test() -> std::sync::MutexGuard<'static, ()> {
    match STORAGE_TEST_SERIALIZER.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    }
}

// F-759-R2（PLAN-759 落地后审核）:AUTO_HTTP_BASE 的进程内修改者
// （plan606 photo stub 锚定 + 本模块 http_base_url_resolution）共用同一
// env，裸 libtest 并行线程互踩。凡触碰此 env 的测试先取此锁串行化，
// 并用 [`HttpBaseEnvGuard`] 做 panic-safe 恢复。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
static HTTP_BASE_TEST_SERIALIZER: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 取 AUTO_HTTP_BASE 测试串行锁（持有至返回的 guard 亡）。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
pub fn lock_http_base_for_test() -> std::sync::MutexGuard<'static, ()> {
    match HTTP_BASE_TEST_SERIALIZER.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// AUTO_HTTP_BASE 的 panic-safe 窗口守卫（F-759-R2）：set 即捕获原值，
/// Drop 恢复（含"原本未设"形态）；窗口内 panic/early-return 均恢复，
/// 不再依赖测试尾部手工还原。
// PLAN-814: 跨 crate 测试支持面
pub struct HttpBaseEnvGuard {
    prev: Option<String>,
}

// PLAN-814: 跨 crate 测试支持面
impl HttpBaseEnvGuard {
    pub fn set(base: &str) -> Self {
        let prev = std::env::var("AUTO_HTTP_BASE").ok();
        std::env::set_var("AUTO_HTTP_BASE", base);
        HttpBaseEnvGuard { prev }
    }
}

// PLAN-814 R3-06/R5：Drop 不设 cfg(test)——HttpBaseEnvGuard 是跨 crate
// 测试支持面（上层 plan606 等经 SDK rlib 消费），门内 Drop 使依赖方
// 测试拿不到恢复语义且非测试构建 `prev` 未读告警。
impl Drop for HttpBaseEnvGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(v) => std::env::set_var("AUTO_HTTP_BASE", v),
            None => std::env::remove_var("AUTO_HTTP_BASE"),
        }
    }
}

#[cfg(test)]
mod sqlite_vm_tests {
    use super::*;

    fn test_vm() -> AutoVM {
        AutoVM::new(
            crate::vm::virt_memory::VirtualFlash::new_with_code(Vec::new()),
            1024,
        )
    }

    #[test]
    fn sqlite_vm_open_exec_query_roundtrip() {
        let vm = test_vm();
        let dir = std::env::temp_dir().join(format!("auto-vm-sqlite-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("t.db");
        let db_path = db_path.to_str().unwrap().to_string();

        let h = shim_sqlite_open_impl(&vm, 17, db_path.clone());
        assert!(h > 0);
        assert_eq!(
            shim_sqlite_exec_impl(
                &vm,
                h,
                "CREATE TABLE cats (id INTEGER PRIMARY KEY, name TEXT)".into()
            ),
            0
        );
        assert_eq!(
            shim_sqlite_exec_impl(&vm, h, "INSERT INTO cats (name) VALUES ('Michi')".into()),
            1
        );
        assert_eq!(shim_sqlite_last_insert_rowid_impl(&vm, h), 1);
        let rows = shim_sqlite_query_impl(&vm, h, "SELECT id, name FROM cats ORDER BY id".into());
        assert_eq!(rows, vec![vec!["1".to_string(), "Michi".to_string()]]);

        // 语法错误：-1 + last_error 非空（哨兵约定）
        assert_eq!(shim_sqlite_exec_impl(&vm, h, "NOT SQL".into()), -1);
        assert!(a2r_std::sqlite::last_error() != "");

        // 未知句柄：哨兵值（-1/0/空），不 panic
        assert_eq!(shim_sqlite_exec_impl(&vm, 999999, "SELECT 1".into()), -1);
        assert!(shim_sqlite_query_impl(&vm, 999999, "SELECT 1".into()).is_empty());
        assert_eq!(shim_sqlite_last_insert_rowid_impl(&vm, 999999), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sqlite_vm_handles_are_vm_local_and_released_with_request_task() {
        let vm_a = test_vm();
        let vm_b = test_vm();
        let dir = std::env::temp_dir().join(format!("auto-vm-sqlite-scope-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path_a = dir.join("a.db").to_string_lossy().into_owned();
        let path_b = dir.join("b.db").to_string_lossy().into_owned();

        let task_a = vm_a.spawn_task(0, 1024);
        let task_b = vm_b.spawn_task(0, 1024);
        let handle_a = shim_sqlite_open_impl(&vm_a, task_a, path_a);
        let handle_b = shim_sqlite_open_impl(&vm_b, task_b, path_b);
        assert_eq!(shim_sqlite_exec_impl(&vm_a, handle_a, "SELECT 1".into()), 0);
        assert_eq!(shim_sqlite_exec_impl(&vm_b, handle_b, "SELECT 1".into()), 0);

        vm_a.remove_task(task_a);
        assert_eq!(
            shim_sqlite_exec_impl(&vm_a, handle_a, "SELECT 1".into()),
            -1
        );
        assert_eq!(shim_sqlite_exec_impl(&vm_b, handle_b, "SELECT 1".into()), 0);

        vm_b.remove_task(task_b);
        assert_eq!(
            shim_sqlite_exec_impl(&vm_b, handle_b, "SELECT 1".into()),
            -1
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// F744-R1-01: the contract's nested-list oracle must travel the real VM
    /// return path. Arguments ride the task stack exactly like engine-
    /// dispatched calls, all five public calls go through their task/stack
    /// shims (not the Rust `*_impl` helpers), and `query` is consumed as the
    /// heap layout the .at surface receives: outer `ListData<Value>` of
    /// VmRefs into inner `ListData<i32>` rows whose cells are negative
    /// string-pool sentinels.
    #[test]
    fn sqlite_vm_query_nested_heap_list_via_vm_stack() {
        fn vm_exec(task: &mut AutoTask, vm: &AutoVM, handle: i64, sql: &str) -> i64 {
            VMConvertible::push_to_stack(&handle, task, vm).unwrap();
            VMConvertible::push_to_stack(&sql.to_string(), task, vm).unwrap();
            shim_sqlite_exec(task, vm).unwrap();
            task.ram.pop_i64()
        }

        // rc_push_id leaves the outer heap id as an i32 stack slot.
        fn vm_query_outer(task: &mut AutoTask, vm: &AutoVM, handle: i64, sql: &str) -> i32 {
            VMConvertible::push_to_stack(&handle, task, vm).unwrap();
            VMConvertible::push_to_stack(&sql.to_string(), task, vm).unwrap();
            shim_sqlite_query(task, vm).unwrap();
            task.ram.pop_i32()
        }

        // Consume both list levels and the string pool the way .at code
        // would: outer VmRef rows -> inner sentinel cells -> utf-8 strings.
        fn heap_rows(vm: &AutoVM, outer_id: i32) -> Vec<Vec<String>> {
            assert!(outer_id > 0, "outer list id must be a heap id");
            let outer_obj = vm
                .get_heap_object(outer_id as u64)
                .expect("outer list on heap");
            let outer = outer_obj
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let outer_list = outer
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                .expect("outer is ListData<Value>");
            let mut rows = Vec::new();
            for elem in &outer_list.elems {
                let row_id = match elem {
                    auto_val::Value::VmRef(r) => r.id as u64,
                    other => panic!("row element must be a VmRef, got {other:?}"),
                };
                let inner_obj = vm.get_heap_object(row_id).expect("inner row list on heap");
                let inner = inner_obj
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let inner_list = inner
                    .as_any()
                    .downcast_ref::<crate::vm::types::ListData<i32>>()
                    .expect("inner row is ListData<i32>");
                let mut cells = Vec::new();
                for s in &inner_list.elems {
                    assert!(*s < 0, "cells ride as negative pool sentinels, got {s}");
                    let idx = (-*s - 1) as u32;
                    let bytes = vm.get_string(idx).expect("pooled string");
                    cells.push(String::from_utf8(bytes).expect("utf-8 cell"));
                }
                rows.push(cells);
            }
            rows
        }

        let vm = test_vm();
        let dir = std::env::temp_dir().join(format!("auto-vm-sqlite-heap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("heap.db").to_string_lossy().into_owned();

        let task_id = vm.spawn_task(0, 1024);
        let task_arc = vm
            .tasks
            .get(&task_id)
            .map(|r| r.value().clone())
            .expect("spawned task registered");
        // Sync libtest context: tokio Mutex needs blocking_lock (uncontended
        // here — the engine is not running this task).
        let mut task = task_arc.blocking_lock();

        // open(path) — handle returns on the task stack.
        VMConvertible::push_to_stack(&db_path, &mut task, &vm).unwrap();
        shim_sqlite_open(&mut task, &vm).unwrap();
        let handle = task.ram.pop_i64();
        assert!(handle > 0);

        // exec: DDL + 3 rows covering NULL, unicode, quotes, and a BLOB.
        assert_eq!(
            vm_exec(
                &mut task,
                &vm,
                handle,
                "CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT, note TEXT, blob BLOB)"
            ),
            0
        );
        assert_eq!(
            vm_exec(
                &mut task,
                &vm,
                handle,
                "INSERT INTO t (name, note, blob) VALUES ('Michi', 'café 中文', X'414243')"
            ),
            1
        );
        assert_eq!(
            vm_exec(
                &mut task,
                &vm,
                handle,
                "INSERT INTO t (name, note, blob) VALUES (NULL, 'it''s \"dq\"', NULL)"
            ),
            1
        );
        assert_eq!(
            vm_exec(
                &mut task,
                &vm,
                handle,
                "INSERT INTO t (name, note, blob) VALUES ('第三行', '', X'00')"
            ),
            1
        );

        // last_insert_rowid via its shim.
        VMConvertible::push_to_stack(&handle, &mut task, &vm).unwrap();
        shim_sqlite_last_insert_rowid(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i64(), 3);

        // query: multi-row, multi-column, per-cell readback through the
        // nested heap list (NULL -> "", BLOB -> "<N bytes>").
        let outer_id = vm_query_outer(
            &mut task,
            &vm,
            handle,
            "SELECT id, name, note, blob FROM t ORDER BY id",
        );
        assert_eq!(
            heap_rows(&vm, outer_id),
            vec![
                vec!["1", "Michi", "café 中文", "<3 bytes>"],
                vec!["2", "", "it's \"dq\"", ""],
                vec!["3", "第三行", "", "<1 bytes>"],
            ]
        );

        // Empty result: a real outer list object with zero rows.
        let empty_id = vm_query_outer(&mut task, &vm, handle, "SELECT id FROM t WHERE id > 100");
        assert!(vm.get_heap_object(empty_id as u64).is_some());
        assert_eq!(heap_rows(&vm, empty_id), Vec::<Vec<String>>::new());

        // Error sentinel: bad SQL -> empty list, and last_error() is a
        // non-empty string coming back through its own shim.
        assert_eq!(
            vm_exec(&mut task, &vm, handle, "NOT SQL"),
            -1,
            "exec reports the -1 sentinel before the query error check"
        );
        let err_id = vm_query_outer(&mut task, &vm, handle, "ALSO NOT SQL");
        assert_eq!(heap_rows(&vm, err_id), Vec::<Vec<String>>::new());
        shim_sqlite_last_error(&mut task, &vm).unwrap();
        let msg: String = VMConvertible::pop_from_stack(&mut task, &vm).unwrap();
        assert!(!msg.is_empty());

        drop(task);
        vm.remove_task(task_id);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAN-044 T-02：键级合并写——盘上他进程的键不被本进程 persist 冲掉；
    /// 本进程脏键（含 remove 删除胜）正确落盘。
    #[test]
    fn storage_persist_merges_disk_with_dirty_keys() {
        let _ser = lock_storage_for_test();
        let dir = std::env::temp_dir().join(format!("auto-t02-merge-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("storage.json");
        std::env::set_var("AUTO_VM_STORAGE_FILE", &path);
        // 盘上预置「他进程」的键（本进程尚未写过它）
        std::fs::write(&path, r#"{"other.key":"fresh"}"#).unwrap();
        shim_storage_set("mine.a".into(), "va".into());
        shim_storage_set("mine.b".into(), "vb".into());
        let map: std::collections::HashMap<String, String> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            map.get("other.key").map(String::as_str),
            Some("fresh"),
            "他进程键被冲掉 = 合并写失效"
        );
        assert_eq!(map.get("mine.a").map(String::as_str), Some("va"));
        assert_eq!(map.get("mine.b").map(String::as_str), Some("vb"));
        // remove 删除胜：盘上消失，他进程键不受牵连
        shim_storage_remove("mine.a".into());
        let map: std::collections::HashMap<String, String> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(!map.contains_key("mine.a"));
        assert_eq!(map.get("other.key").map(String::as_str), Some("fresh"));
        std::env::remove_var("AUTO_VM_STORAGE_FILE");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PLAN-044 T-02：盘文件坏 JSON（撕裂/损坏）→ `.corrupt.bak` 留证后
    /// 以内存重建，persist 自愈不空库。
    #[test]
    fn storage_persist_rebuilds_from_corrupt_disk() {
        let _ser = lock_storage_for_test();
        let dir = std::env::temp_dir().join(format!("auto-t02-corrupt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("storage.json");
        std::env::set_var("AUTO_VM_STORAGE_FILE", &path);
        std::fs::write(&path, "{\"torn\":").unwrap();
        shim_storage_set("k".into(), "v".into());
        let raw = std::fs::read_to_string(&path).unwrap();
        let map: std::collections::HashMap<String, String> =
            serde_json::from_str(&raw).expect("重建后必须合法 JSON");
        assert_eq!(map.get("k").map(String::as_str), Some("v"));
        assert!(
            path.with_extension("json.corrupt.bak").exists(),
            "坏文件必须留证 .corrupt.bak"
        );
        std::env::remove_var("AUTO_VM_STORAGE_FILE");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PLAN-617 T-10: 相对 URL（`/` 开头）按 `AUTO_HTTP_BASE` 展开为绝对地址；
    /// 绝对 URL 与未设 env 时原样透传（幂等：展开后的绝对 URL 再过一遍不变）。
    /// F-759-R2:与 plan606 photo stub 测试共用串行锁 + panic-safe 守卫
    /// （裸 libtest 并行形态下两测试曾可互相覆盖 env）。
    #[test]
    fn http_base_url_resolution() {
        let _ser = lock_http_base_for_test();
        let _env = HttpBaseEnvGuard::set("http://127.0.0.1:8330");
        // 绝对 URL 永不改动（env 设不设都一样）。
        assert_eq!(
            resolve_http_base_url("http://example.com/api/x"),
            "http://example.com/api/x"
        );
        // 相对 URL + 基址 → 展开；基址尾部斜杠不产生双斜杠。
        assert_eq!(
            resolve_http_base_url("/api/media/scan"),
            "http://127.0.0.1:8330/api/media/scan"
        );
        std::env::set_var("AUTO_HTTP_BASE", "http://127.0.0.1:8330/");
        assert_eq!(
            resolve_http_base_url("/api/media/scan"),
            "http://127.0.0.1:8330/api/media/scan"
        );
        // env 为空白 → 无基址可用，原样透传（与今日行为一致，交给上层报错）。
        std::env::set_var("AUTO_HTTP_BASE", "   ");
        assert_eq!(resolve_http_base_url("/api/x"), "/api/x");
        std::env::remove_var("AUTO_HTTP_BASE");
        assert_eq!(resolve_http_base_url("/api/x"), "/api/x");
        // 尾部无需手工恢复：_env Drop 回窗口前原值（含 panic 路径）。
    }

    /// F-759-R2:守卫的 panic-safe 恢复——窗口内 panic 后 Drop 仍回窗口前
    /// 原值；串行锁 poisoned 后 lock_http_base_for_test 可接管（into_inner）。
    #[test]
    fn http_base_env_guard_restores_after_panic() {
        let _ser = lock_http_base_for_test();
        let prev = std::env::var("AUTO_HTTP_BASE").ok();
        std::env::set_var("AUTO_HTTP_BASE", "http://pre-window");
        let h = std::thread::spawn(|| {
            let _g = HttpBaseEnvGuard::set("http://in-window");
            assert_eq!(std::env::var("AUTO_HTTP_BASE").unwrap(), "http://in-window");
            panic!("window 内 panic");
        });
        let _ = h.join();
        assert_eq!(
            std::env::var("AUTO_HTTP_BASE").unwrap(),
            "http://pre-window",
            "panic 后守卫必须恢复窗口前原值"
        );
        match prev {
            Some(v) => std::env::set_var("AUTO_HTTP_BASE", v),
            None => std::env::remove_var("AUTO_HTTP_BASE"),
        }
    }

    /// PLAN-027 缺陷 C:Err 条目必须**可解析地终结**——take 已在
    /// check 内发生,返回 None 会让 shim 重入分支判"仍在等待"而永远挂起。
    /// PLAN-705 T-02:登记→完成(Err)→take→单次终结走统一 live-op 协议。
    /// PLAN-712 T-17:Err 条目的消费形态从「包成错误 body 字符串」改判
    /// 「**Err 透传**」——shim 重入臂转可捕获 VMError（.at try/catch 可接），
    /// 本测钉住 Err 形态本身（body 包装退役）。
    #[test]
    fn p027_check_async_result_err_resolves_once_as_err() {
        let id = alloc_async_id();
        crate::vm::ffi::async_http::register_live_op(id);
        assert!(crate::vm::ffi::async_http::complete_live_op(
            id,
            Err("conn refused".to_string())
        ));
        let res = check_async_http_result(id).expect("Err entry must resolve, not None");
        assert!(
            matches!(res, Err(ref e) if e.contains("conn refused")),
            "got: {res:?}"
        );
        // 条目已被消费(二次 check 无结果)。
        assert!(check_async_http_result(id).is_none());
    }

    /// PLAN-027 缺陷 A:超时回收——drop_async_result（cancel 终结）移除
    /// 条目,缺席时幂等(engine drain 超时臂调用);此后 worker 迟到完成被
    /// presence 守卫丢弃（PLAN-705 T-02）。
    #[test]
    fn p027_drop_async_result_removes_entry() {
        let id = alloc_async_id();
        crate::vm::ffi::async_http::register_live_op(id);
        assert!(!async_http_result_ready(id));
        drop_async_result(id);
        assert!(check_async_http_result(id).is_none());
        drop_async_result(id);
        // 迟到完成不得复活。
        assert!(!crate::vm::ffi::async_http::complete_live_op(
            id,
            Ok(AsyncResult::Body("late".to_string()))
        ));
        assert!(crate::vm::ffi::async_http::take_live_op(id).is_none());
    }

    /// Plan 524 三态：process.args() = [程序路径] + CLI 透传参数（List 契约，
    /// 对齐 a2r 路径消费形态）；无透传 → 仅程序路径。修复前异端 = join 空格
    /// 字符串（517 W3 "argc=76" 即对 join 串取长度的伪 argc）。
    #[test]
    fn process_args_three_states() {
        crate::set_script_args(vec![]);
        let a0 = shim_process_args();
        assert_eq!(a0.len(), 1, "无参 → 仅程序路径");
        assert!(!a0[0].is_empty(), "argv0 非空（宿主进程路径）");

        crate::set_script_args(vec!["alpha".to_string()]);
        let a1 = shim_process_args();
        assert_eq!(a1.len(), 2, "单参 → 程序路径 + 1 透传");
        assert_eq!(a1[0], a0[0]);
        assert_eq!(a1[1], "alpha");

        crate::set_script_args(vec![
            "x.at".to_string(),
            "--flag".to_string(),
            "b c".to_string(),
        ]);
        let a2 = shim_process_args();
        assert_eq!(a2.len(), 4, "多参 → 程序路径 + 3 透传");
        assert_eq!(
            &a2[1..],
            &["x.at".to_string(), "--flag".to_string(), "b c".to_string()],
            "含空格参数原样保留（join 形态不可表达）"
        );

        // 复位，防跨测试泄漏（script_args 是进程级全局）
        crate::set_script_args(vec![]);
    }

    /// KD-048「KV 会话恢复断裂」回归：raw localStorage 家族（.at 的
    /// localStorage.* 后端）必须 load-once + write-through 落盘——
    /// 此前 set 只进进程内存 MAP，musk 登录凭据/jwt 跨重启即失。
    /// 隔离沿 t2_isolate_storage 惯例（AUTO_VM_STORAGE_FILE 指向一次性
    /// 文件；env 为进程级全局，同 P487-2 已案债）。
    #[test]
    fn storage_raw_set_persists_to_backing_file() {
        let _serial = lock_storage_for_test();
        let path =
            std::env::temp_dir().join(format!("auto-raw-storage-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        std::env::set_var("AUTO_VM_STORAGE_FILE", &path);

        storage_raw_set("musk_login_username".into(), "admin".into());
        assert_eq!(
            storage_raw_get("musk_login_username"),
            Some("admin".to_string()),
            "in-process read-back"
        );
        // write-through: 盘上文件必须已含该键（不依赖进程存续）。
        let raw = std::fs::read_to_string(&path).expect("backing file written");
        assert!(
            raw.contains("musk_login_username") && raw.contains("admin"),
            "backing file must contain the key: {raw}"
        );

        // remove 同样穿透到盘。
        storage_raw_remove("musk_login_username");
        assert_eq!(storage_raw_get("musk_login_username"), None);
        let raw = std::fs::read_to_string(&path).expect("backing file readable");
        assert!(
            !raw.contains("musk_login_username"),
            "removed key must not persist: {raw}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// PLAN-028 T-03:新鲜读穿透语义——storage_host_read_fresh 直读盘上
    /// 最新值(模拟他进程发布端直写),不受本进程 load-once `or_insert`
    /// 首读冻结;host_read 保持首读语义不变(对偶面回归)。
    #[test]
    fn storage_host_read_fresh_sees_external_writes() {
        let _serial = lock_storage_for_test();
        let path =
            std::env::temp_dir().join(format!("auto-fresh-storage-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        std::env::set_var("AUTO_VM_STORAGE_FILE", &path);

        storage_host_publish("p028.win.w", "1182".into());
        assert_eq!(
            storage_host_read_fresh("p028.win.w").as_deref(),
            Some("1182")
        );
        // 模拟分体轨他进程(前端渲染器)直写 backing 文件,绕过本进程 MAP。
        let mut m: std::collections::HashMap<String, String> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        m.insert("p028.win.w".into(), "999".into());
        std::fs::write(&path, serde_json::to_string(&m).unwrap()).unwrap();
        assert_eq!(
            storage_host_read_fresh("p028.win.w").as_deref(),
            Some("999"),
            "新鲜读必须穿透本进程缓存"
        );
        assert_eq!(
            storage_host_read("p028.win.w").as_deref(),
            Some("1182"),
            "host_read 保持首读语义(对偶面不回退)"
        );
        // 缺键回落进程内 MAP,盘上无键不 panic。
        assert_eq!(storage_host_read_fresh("p028.absent"), None);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_substr_clamps_to_char_boundaries() {
        // str-parity: (start, LEN) semantics — ASCII cases updated from the
        // old (start, END) reading to match a2r_std::str_substr.
        assert_eq!(shim_str_substr("hello".to_string(), 1, 3), "ell");
        assert_eq!(shim_str_substr("hello".to_string(), 2, 2), "ll");
        // Byte-walk inside a multi-byte char clamps instead of panicking.
        // "送" occupies bytes 0..3; lo/hi inside one char → empty after clamp.
        assert_eq!(shim_str_substr("送你".to_string(), 1, 2), "");
        assert_eq!(shim_str_substr("送你".to_string(), 2, 2), "");
        // Range spanning a char boundary clamps inward: lo→3, hi→3 → empty.
        assert_eq!(shim_str_substr("送你".to_string(), 1, 4), "");
        assert_eq!(shim_str_substr("a送b".to_string(), 1, 2), "");
        // A range that fully contains a char keeps it.
        assert_eq!(shim_str_substr("送你".to_string(), 0, 6), "送你");
        assert_eq!(shim_str_substr("送你".to_string(), 3, 6), "你");
        // Out-of-range / negative indices stay empty (or clamp to the tail).
        assert_eq!(shim_str_substr("送".to_string(), 3, 3), "");
        assert_eq!(shim_str_substr("送".to_string(), 0, 9), "送");
        assert_eq!(shim_str_substr("送".to_string(), 2, -1), "");
    }

    /// str-parity (audit B1 finding): the VM shim and a2r_std must agree on
    /// `.substr(start, LEN)` — before the fix the same call gave "world" in
    /// transpiled Rust but "" in the VM (start, END reading).
    #[test]
    fn test_substr_parity_with_a2r_std() {
        for (s, start, len) in [
            ("hello world", 6, 5),
            ("hello world", 0, 5),
            ("hello", 1, 3),
            ("hello", 2, 2),
            ("a送b", 0, 4),
            ("送你", 3, 3),
        ] {
            assert_eq!(
                shim_str_substr(s.to_string(), start, len),
                crate::a2r_std::str_substr(s, start, len),
                "VM shim vs a2r_std mismatch on ({:?}, {}, {})",
                s,
                start,
                len
            );
        }
        // The exact audit-B1 divergence case: old END semantics returned "".
        assert_eq!(shim_str_substr("hello world".to_string(), 6, 5), "world");
    }

    /// str-parity routing lock: `.slice`/`.sub` route to the END-semantics
    /// shim, `.substr` to the LEN one — three spellings, two shims, all
    /// matching a2r emission for the same calls.
    #[test]
    fn vm_slice_sub_end_substr_len() {
        let (_r, out) = crate::run_with_capture(
            r#"
fn main() {
    var s str = "hello world"
    print(s.slice(3, 6))
    print(s.sub(3, 6))
    print(s.substr(6, 5))
}
"#,
        )
        .unwrap();
        let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines, vec!["lo ", "lo ", "world"]);
    }

    /// str-parity end-to-end: the same .at source run in the VM must see the
    /// LEN semantics for .substr, END semantics for .slice, and index/-1 for
    /// .find — matching what a2r emits for the same calls.
    #[test]
    fn vm_substr_slice_find_semantics() {
        let (_r, out) = crate::run_with_capture(
            r#"
fn main() {
    var s str = "hello world"
    print(s.substr(6, 5))
    print(s.slice(6, 11))
    print(s.sub(6, 11))
    print(s.find("world"))
    print(s.find("o", 5))
    print(s.substr(6, 5) == s.slice(6, 11))
}
"#,
        )
        .unwrap();
        let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(
            lines,
            vec!["world", "world", "world", "6", "7", "true"], // Plan 474 待澄清#3: bool 显示形态 true/false
            "VM str semantics: substr(len) / slice(end) / find(idx[, from])"
        );
    }

    #[test]
    fn test_native_ids_are_in_range() {
        // Verify all IDs are in the static range (0-9999)
        assert!(NATIVE_FILE_READ_TEXT < 10000);
        assert!(NATIVE_FILE_WRITE_TEXT < 10000);
        assert!(NATIVE_FILE_EXISTS < 10000);
        assert!(NATIVE_FILE_READ_BYTES < 10000);
        assert!(NATIVE_FILE_WRITE_BYTES < 10000);
        assert!(NATIVE_FILE_COPY < 10000);
        assert!(NATIVE_FILE_SIZE < 10000);
        assert!(NATIVE_FILE_IS_DIR < 10000);
        assert!(NATIVE_ENV_GET < 10000);
        assert!(NATIVE_TIME_NOW_MS < 10000);
        assert!(NATIVE_PROCESS_EXIT < 10000);
        assert!(NATIVE_PROCESS_ARGS < 10000);
        assert!(NATIVE_PROCESS_CURRENT_DIR < 10000);
        assert!(NATIVE_PROCESS_SPAWN < 10000);
        assert!(NATIVE_PATH_JOIN < 10000);
        assert!(NATIVE_STR_LEN < 10000);
        assert!(NATIVE_CHAR_IS_ALPHA < 10000);
        assert!(NATIVE_MATH_ABS < 10000);
        assert!(NATIVE_MATH_MIN < 10000);
        assert!(NATIVE_MATH_MAX < 10000);
        assert!(NATIVE_MATH_SQRT < 10000);
    }

    #[test]
    fn test_id_ranges_are_correct() {
        // File: 1000-1099
        assert!((1000..1100).contains(&NATIVE_FILE_READ_TEXT));
        assert!((1000..1100).contains(&NATIVE_FILE_READ_BYTES));
        assert!((1000..1100).contains(&NATIVE_FILE_WRITE_BYTES));
        assert!((1000..1100).contains(&NATIVE_FILE_COPY));
        assert!((1000..1100).contains(&NATIVE_FILE_SIZE));
        assert!((1000..1100).contains(&NATIVE_FILE_IS_DIR));
        assert!((1000..1100).contains(&NATIVE_FILE_WALK));
        assert!((1000..1100).contains(&NATIVE_FILE_APPEND_TEXT));
        assert!((1000..1100).contains(&NATIVE_FILE_READ_LINES));

        // Env: 1100-1199
        assert!((1100..1200).contains(&NATIVE_ENV_GET));

        // Time: 1200-1299
        assert!((1200..1300).contains(&NATIVE_TIME_NOW_MS));

        // Process: 1300-1399
        assert!((1300..1400).contains(&NATIVE_PROCESS_EXIT));
        assert!((1300..1400).contains(&NATIVE_PROCESS_ARGS));
        assert!((1300..1400).contains(&NATIVE_PROCESS_CURRENT_DIR));
        assert!((1300..1400).contains(&NATIVE_PROCESS_SPAWN));
        assert!((1300..1400).contains(&NATIVE_PROCESS_SPAWN_WITH_OUTPUT));

        // HTTP streaming with headers: 2255-2259
        assert!((2250..2260).contains(&NATIVE_HTTP_POST_STREAM_WITH_HEADERS));

        // Regex: 2400-2499
        assert!((2400..2500).contains(&NATIVE_REGEX_IS_MATCH));
        assert!((2400..2500).contains(&NATIVE_REGEX_FIND_ALL));

        // Path: 1400-1499
        assert!((1400..1500).contains(&NATIVE_PATH_JOIN));

        // String: 1500-1599
        assert!((1500..1600).contains(&NATIVE_STR_LEN));

        // Char: 1600-1699
        assert!((1600..1700).contains(&NATIVE_CHAR_IS_ALPHA));

        // Math: 1700-1799
        assert!((1700..1800).contains(&NATIVE_MATH_ABS));
        assert!((1700..1800).contains(&NATIVE_MATH_MIN));
        assert!((1700..1800).contains(&NATIVE_MATH_MAX));
        assert!((1700..1800).contains(&NATIVE_MATH_SQRT));

        // Task/Msg: 2300-2399
        assert!((2300..2400).contains(&NATIVE_TASK_SPAWN));
        assert!((2300..2400).contains(&NATIVE_TASK_SEND));
        assert!((2300..2400).contains(&NATIVE_TASK_HANDLE_IS_NULL));
        assert!((2300..2400).contains(&NATIVE_TASK_HANDLE_TYPE));
        assert!((2300..2400).contains(&NATIVE_TASK_HANDLE_ID));
        assert!((2300..2400).contains(&NATIVE_TASK_SYSTEM_START));
    }

    /// PLAN-680 Q1: track identification — the VM runtime is the "vm" track
    /// (the vue generator folds `Env.track()` to "vue" at generation time).
    #[test]
    fn env_track_shim_returns_vm() {
        assert_eq!(shim_env_track(), "vm");
    }

    // Plan 121: Task spawn tests
    #[test]
    fn test_task_spawn() {
        let handle_id = shim_task_spawn("TestTask".to_string(), 64);
        assert!(handle_id > 0);
    }

    #[test]
    fn test_task_spawn_default_capacity() {
        // Test with negative capacity (should use default 64)
        let handle_id = shim_task_spawn("TestTask".to_string(), -1);
        assert!(handle_id > 0);
    }

    #[test]
    fn test_task_handle_is_null() {
        // Handle ID 0 should be null
        let result = shim_task_handle_is_null(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Non-existent handle should also be treated as null
        let result = shim_task_handle_is_null(999999);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // A spawned task should not be null
        let handle_id = shim_task_spawn("TestTask".to_string(), 64) as i64;

        let result = shim_task_handle_is_null(handle_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_task_handle_type() {
        let handle_id = shim_task_spawn("MyCounterTask".to_string(), 64) as i64;

        let result = shim_task_handle_type(handle_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "MyCounterTask");
    }

    #[test]
    fn test_task_handle_id() {
        let handle_id = shim_task_spawn("TestTask".to_string(), 64);

        let result = shim_task_handle_id(handle_id as i64);
        assert!(result.is_ok());
        // Instance ID should be > 0 (global counter)
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_task_send() {
        let handle_id = shim_task_spawn("TestTask".to_string(), 64);

        // shim_task_send takes (i32, i32) - handle_id and message enum tag
        let result = shim_task_send(handle_id, 1); // 1 is some message tag
        assert_eq!(result, 1); // 1 = success
    }

    #[test]
    fn test_task_send_invalid_handle() {
        // Invalid handle should return 0 (failure)
        let result = shim_task_send(999999, 1);
        assert_eq!(result, 0); // 0 = failure
    }

    #[test]
    fn test_multiple_spawn_unique_ids() {
        let id1 = shim_task_spawn("Task1".to_string(), 64);
        let id2 = shim_task_spawn("Task2".to_string(), 64);

        // Each spawn should return a unique handle ID
        assert_ne!(id1, id2);
    }

    // Plan 159: New FFI function tests

    #[test]
    fn test_file_append_text() {
        let dir = std::env::temp_dir().join("ac-ffi-test-append");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("append_test.txt");

        // First append creates the file
        shim_file_append_text(path.to_str().unwrap().to_string(), "line one\n".to_string())
            .unwrap();

        // Second append adds to existing file
        shim_file_append_text(path.to_str().unwrap().to_string(), "line two\n".to_string())
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "line one\nline two\n");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_append_creates_parent_dirs() {
        let dir = std::env::temp_dir().join("ac-ffi-test-append-nested");
        let _ = fs::remove_dir_all(&dir);

        let path = dir.join("a/b/c/test.txt");
        shim_file_append_text(path.to_str().unwrap().to_string(), "hello".to_string()).unwrap();

        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_regex_is_match() {
        // Match found
        let result = shim_regex_is_match("hello\\s+world".to_string(), "hello world".to_string());
        assert_eq!(result.unwrap(), 1);

        // No match
        let result = shim_regex_is_match("xyz\\d+".to_string(), "hello world".to_string());
        assert_eq!(result.unwrap(), 0);

        // Invalid regex
        let result = shim_regex_is_match("[invalid(".to_string(), "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_regex_find_all_simple() {
        let text = "hello world, hello rust";
        let re = regex::Regex::new("hello").unwrap();
        let matches: Vec<_> = re.find_iter(text).collect();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].as_str(), "hello");
        assert_eq!(matches[1].as_str(), "hello");
    }

    #[test]
    fn test_regex_find_all_with_groups() {
        let text = "foo123bar456baz";
        let re = regex::Regex::new("\\d+").unwrap();
        let matches: Vec<_> = re.find_iter(text).collect();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].as_str(), "123");
        assert_eq!(matches[1].as_str(), "456");
    }
}

// ============================================================================
// Plan 263 Phase 2: Test runner FFI — run_a2r_dir
// ============================================================================

#[cfg(test)]
mod plan536_date_format_tests {
    use super::format_date_ms;

    /// PLAN-536 T4(题3): epoch **秒**口径（后端 created_at 原始单位,musk
    /// 实录 1788436450 = 2026-09-03 19:54:10+08）此前被当毫秒换算 → 产出
    /// 1970 年垃圾时刻（非整时区偏移假象）。秒/毫秒双口径必须产出同一
    /// 时刻标签；归一阈值沿业界惯例 1e11（< 1e11 视为秒）。
    #[test]
    fn p536_seconds_and_millis_agree() {
        // 2026-09-03T11:54:10Z 的两种口径
        let secs: i64 = 1_788_436_450;
        let millis: i64 = secs * 1000;
        assert_eq!(
            format_date_ms(secs, "HH:mm:ss"),
            format_date_ms(millis, "HH:mm:ss"),
            "dual-unit inputs must label the same instant"
        );
        assert_eq!(
            format_date_ms(secs, "yyyy/MM/dd HH:mm:ss"),
            format_date_ms(millis, "yyyy/MM/dd HH:mm:ss"),
        );
        // 毫秒样本含亚秒:SSS 口径双口径一致(秒×1000 无亚秒残差)
        assert_eq!(
            format_date_ms(secs, "yyyy-MM-dd HH:mm:ss.SSS"),
            format_date_ms(millis, "yyyy-MM-dd HH:mm:ss.SSS"),
        );
    }

    /// 不再产出 1970 垃圾:2026 纪元秒的 yyyy 必须是 2026(与本机时区无关
    /// ——秒归一发生在 Local 换算之前)。
    #[test]
    fn p536_seconds_no_1970_garbage() {
        let secs: i64 = 1_788_436_450;
        let label = format_date_ms(secs, "yyyy");
        assert_eq!(
            label, "2026",
            "seconds input must land in 2026, got {label}"
        );
    }

    /// 毫秒口径回归锚:Date.now 产毫秒,1e12 级值仍按毫秒(归一不误伤)。
    #[test]
    fn p536_millis_path_unchanged() {
        let ms: i64 = 1_788_436_450_000;
        assert_eq!(format_date_ms(ms, "yyyy"), "2026");
        // 0 纪元:两口径同锚(秒 0 → 1970-01-01 本地;不宣称可读,只锁同—)
        assert_eq!(format_date_ms(0, "HH:mm:ss"), format_date_ms(0, "HH:mm:ss"));
    }

    /// 时区边界:归一只动单位不动时区——任取秒值,与同瞬毫秒在本地时刻
    /// 逐位一致(含 UTC 跨日边界/闰日样本;Local 换算不受归一影响)。
    /// 样本限 ≥1e11 毫秒形态的现代时刻——<1973-03 的毫秒值(<1e11)落在
    /// 双义带内被按秒读出,属 1e11 阈值惯例的固有边界,不在承诺内。
    #[test]
    fn p536_normalization_precedes_local_conversion() {
        for s in [
            951_782_399i64, /* 2000-02-29 跨日闰日 */
            1_756_857_599,  /* 2025-09 跨日 */
            1_788_436_450,
        ] {
            assert_eq!(
                format_date_ms(s, "yyyy/MM/dd HH:mm:ss"),
                format_date_ms(s.saturating_mul(1000), "yyyy/MM/dd HH:mm:ss"),
                "sec={s}"
            );
        }
    }

    // ── auto-os Plan 013 T2:AutoTerm 引擎 VM 绑定(auto.term.*)──────
    //
    // 语义对齐 auto-term at-gen src/engine.rs(句柄表 + 快照表 +
    // engine_rows 内联 feed 全量重采);DLL 解析顺序 env → 宿主同目录
    // (003 §5)→ exe 祖先 target/{debug,release}。DLL 缺席时 spawn
    // 返 0(契约内失败),其余按句柄无效静默 no-op——宿主桌面不因
    // 部署缺件崩 VM。

    /// 定位 auto-term 构建产物(manifest 祖先链上的 auto-term 仓:
    /// 组 worktree 与主检出两形态都覆盖;缺席 = 显式 skip,沿
    /// ash_integration 惯例)。
    fn autoterm_dll_for_test() -> Option<std::path::PathBuf> {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .ancestors()
            .map(|a| {
                a.join("auto-term")
                    .join("target")
                    .join("debug")
                    .join("autoterm_core.dll")
            })
            .find(|p| p.is_file())
    }

    /// 注册 → VM 内调用 → echo 往返:spawn 得非零句柄、write_line 后
    /// rows() 快照里能收割到回显、is_exited 为假、free 收尾。
    #[test]
    fn term_engine_shims_echo_roundtrip() {
        let Some(dll) = autoterm_dll_for_test() else {
            eprintln!(
                "[term-engine] autoterm_core.dll not found — skipping (build auto-term first)"
            );
            return;
        };
        std::env::set_var("AUTOTERM_ENGINE_DLL", &dll);
        let (_r, out) = crate::run_with_capture(r#"
use auto.term: engine_spawn, engine_write_line, engine_rows, engine_resize, engine_interrupt, engine_is_exited, engine_free
fn find_marker(handle int) int {
    var lines = Term.engine_rows(handle)
    var count = lines.len()
    var j = 0
    loop {
        if j >= count {
            break
        }
        if lines.get(j).find("autoterm_vm_ok") >= 0 {
            break
        }
        j = j + 1
    }
    return j
}

fn main() {
    var h = Term.engine_spawn(80, 24)
    print(h > 0)
    Term.engine_write_line(h, "echo autoterm_vm_ok")
    var tries = 0
    var found = 999
    loop {
        if tries >= 30 {
            break
        }
        found = find_marker(h)
        if found < 24 {
            break
        }
        Time.sleep_ms(100)
        tries = tries + 1
    }
    print(found)
    Term.engine_resize(h, 100, 30)
    var lines2 = Term.engine_rows(h)
    print(lines2.len())
    print(Term.engine_interrupt(h) >= 0)
    print(Term.engine_is_exited(h) == false)
    Term.engine_free(h)
    print("done")
}
"#)
        .expect("term engine script must run");
        let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 6, "got {lines:?}");
        assert_eq!(lines[0], "true", "spawn non-zero");
        let found: i32 = lines[1].parse().unwrap();
        assert!(
            found >= 0 && found < 24,
            "echo marker found at row {found} of 24"
        );
        assert_eq!(lines[2], "30", "resize to 100x30");
        assert_eq!(lines[3], "true", "interrupt ok");
        assert_eq!(lines[4], "true", "not exited");
        assert_eq!(lines[5], "done", "script completed");
    }

    /// PLAN-013 T1:裸调用面探针——`use auto.term` 导入后**不加 `Term.` 前
    /// 缀**直接裸调七符号。at/autoterm.at 现行调用面即裸调用(a2r 侧
    /// `use crate::term::` + 裸调用天然闭合);本测试钉住 VM 侧同形可用,
    /// D1(单一调用面跨三后端)的 VM 臂验收锚点。
    #[test]
    fn term_engine_shims_bare_call_roundtrip() {
        let Some(dll) = autoterm_dll_for_test() else {
            eprintln!(
                "[term-engine] autoterm_core.dll not found — skipping (build auto-term first)"
            );
            return;
        };
        std::env::set_var("AUTOTERM_ENGINE_DLL", &dll);
        let (_r, out) = crate::run_with_capture(r#"
use auto.term: engine_spawn, engine_write_line, engine_rows, engine_resize, engine_interrupt, engine_is_exited, engine_free
fn main() {
    var h = engine_spawn(80, 24)
    print(h > 0)
    engine_write_line(h, "echo autoterm_bare_ok")
    var found = 999
    var tries = 0
    loop {
        if tries >= 30 {
            break
        }
        var lines List<str> = engine_rows(h)
        var j = 0
        loop {
            if j >= lines.len() {
                break
            }
            // 597 §9 缺陷规避:裸 C-FFI 调用不得内联于 if 条件位(挂起),
            // 先赋值再比较;缺陷修复属 auto-lang 独立账。
            var hit = lines.get(j).find("autoterm_bare_ok")
            if hit >= 0 {
                found = j
                break
            }
            j = j + 1
        }
        if found < 24 {
            break
        }
        Time.sleep_ms(100)
        tries = tries + 1
    }
    print(found)
    engine_resize(h, 100, 30)
    var after List<str> = engine_rows(h)
    print(after.len())
    print(engine_interrupt(h) >= 0)
    print(engine_is_exited(h) == false)
    engine_free(h)
    print("done")
}
"#)
        .expect("term engine bare-call script must run");
        let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 6, "got {lines:?}");
        assert_eq!(lines[0], "true", "bare engine_spawn non-zero");
        let found: i32 = lines[1].parse().unwrap();
        assert!(
            found >= 0 && found < 24,
            "bare-call echo marker at row {found} of 24"
        );
        assert_eq!(lines[2], "30", "bare engine_resize to 100x30");
        assert_eq!(lines[3], "true", "bare engine_interrupt ok");
        assert_eq!(lines[4], "true", "bare engine_is_exited false");
        assert_eq!(lines[5], "done", "bare-call script completed");
    }
}

/// PLAN-698 T-02/SD-02: VM 事件总线（bus.subscribe 运行面）。
#[cfg(test)]
mod bus_subscribe_tests {
    use super::*;

    #[test]
    fn bus_broadcast_fans_out_and_ignores_missing_subscribers() {
        let mut s1 = EVENT_BUS.subscribe();
        let mut s2 = EVENT_BUS.subscribe();
        bus_broadcast(r#"{"event":"Typing"}"#.into());
        assert_eq!(
            s1.blocking_recv().unwrap(),
            r#"{"event":"Typing"}"#.to_string()
        );
        assert_eq!(
            s2.blocking_recv().unwrap(),
            r#"{"event":"Typing"}"#.to_string()
        );
        // 无订阅者广播不报错（镜像生成侧 `let _ = bus().send(json)`）。
        bus_broadcast("unheard".into());
    }

    #[test]
    fn shim_bus_subscribe_registers_async_stream_iterator_and_delivers() {
        let vm = crate::vm::engine::AutoVM::new(
            crate::vm::virt_memory::VirtualFlash::new_with_code(vec![
                crate::vm::opcode::OpCode::RET as u8,
            ]),
            1024,
        );
        // DashMap 读守卫作用域内只读；vm.tasks.remove 必须等守卫全 drop
        // （同 shard 自死锁陷阱，见 http_server.rs Plan 346 3c 注记）。
        let (task_id, iter_id, stream_id) = {
            let task_id = vm.spawn_task(0, 8192);
            let task_arc = vm.tasks.get(&task_id).expect("task spawned");
            let mut task = task_arc.blocking_lock();

            shim_bus_subscribe(&mut task, &vm).expect("subscribe shim runs");

            let iter_id = auto_val::decode_i32(task.ram.peek_nv(0)) as u32;
            assert!(
                iter_id >= 0,
                "subscribe returns a real iterator id, got {iter_id}"
            );
            let stream_id = match vm.iterators.get(&iter_id).map(|it| it.clone()) {
                Some(crate::vm::engine::Iterator::AsyncHttpStream(a)) => a.stream_id,
                other => panic!("expected AsyncHttpStream iterator, got {other:?}"),
            };
            (task_id, iter_id, stream_id)
        };
        vm.remove_task(task_id);

        let handle = ASYNC_STREAMS
            .lock()
            .unwrap()
            .get(&stream_id)
            .cloned()
            .expect("stream handle registered");
        bus_broadcast(r#"{"event":"NewMessage"}"#.into());
        // 转发线程异步泵送——有界轮询等待 Data 帧。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut got: Option<AsyncStreamEvent> = None;
        while std::time::Instant::now() < deadline {
            if let Ok(ev) = handle.rx.lock().unwrap().try_recv() {
                got = Some(ev);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        match got {
            Some(AsyncStreamEvent::Data(json)) => {
                assert_eq!(json, r#"{"event":"NewMessage"}"#.to_string());
            }
            other => panic!("expected Data frame from bus forwarder, got {other:?}"),
        }
    }
}

// ---- PLAN-843 结构拆分：按域分族（原单文件热点，纯搬移） ----
// ---- PLAN-884 T-04：基础设施域（原样搬移出壳） ----
mod request_root;
pub use request_root::*;
mod storage;
pub use storage::*;
mod exit_audit;
pub use exit_audit::*;
mod http_async;
pub use http_async::*;
mod rust_stdlib_dispatch;
pub(crate) use rust_stdlib_dispatch::*;

mod crash_trace;
pub use crash_trace::*;
mod zip_io;
pub use zip_io::*;
mod file_io;
pub use file_io::*;
mod http_client;
pub use http_client::*;
mod image;
pub use image::*;
mod math;
pub use math::*;
mod sqlite;
pub use sqlite::*;
mod strings_json;
pub use strings_json::*;

// ---- PLAN-843 结构拆分：按域分族（原单文件热点，纯搬移） ----
mod sys_env;
pub use sys_env::*;
