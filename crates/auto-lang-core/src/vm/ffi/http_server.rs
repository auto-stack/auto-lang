//! VM `#[api]` HTTP server core: route table, argument binding, middleware,
//! handler dispatch, SSE producers and the owned request/reply bridge.
//!
//! PLAN-699 (Design 33 阶段 B): the HTTP/1.1 wire protocol is Axum/Hyper's
//! job — the transport lives in [`super::http_transport`] on a dedicated net
//! thread and hands owned `ApiRequest`s to [`dispatch_api_request`] on the
//! VM owner thread. AutoVM is `!Send` (`Rc<RefCell>` in its type system);
//! only owned `Send` data crosses the bridge — no `usize` pointer
//! laundering, no `spawn_blocking` VM calls. This module is the VM server
//! only: the generated Rust track (auto-man api_gen) builds its own Axum
//! service, and `shim_http_server_listen`/`back_proxy` are separate server
//! paths (see docs/specs/stdlib/design/backend-assembly.md).

use std::io::{BufRead, Read, Write};
use std::net::TcpListener;
use crate::vm::host_capabilities::http_service::VmHttpBridge;

// ============================================================================
// PLAN-669: #[api] handler 形参签名侧信道
// ============================================================================
// Legacy embedding cache. Production compilers collect declarations into a
// local scope and freeze them onto the VM before main; owned service readers
// never use these process tables when that immutable metadata is selected.
// codegen 在 api_routes.push 处同步发布每个 #[api] fn 的形参（名+类型），
// 服务侧据此按名装配 handler 实参（路径段 → body 字段 → query，缺参 400；
// 见 bind_api_args_by_name）。生命周期模型同 stdlib 的 HTTP_ROUTES /
// axum_adapter 的 PARAM_SIGS：每驱动进程一个程序，run pipeline 编译前重置
// （clear_api_param_sigs），e2e 隔离经 clear_http_routes 连带清空。
// 条目缺失（legacy 生产者）→ 服务侧回退旧位序装配，行为不变。

static API_PARAM_SIGS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, Vec<ApiParamSig>>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// PLAN-698 T-02/SD-02: #[api] fn 返回类型显示串侧信道（与 API_PARAM_SIGS
/// 同生命周期模型）。SSE publisher 臂据此判 has_sse（任一 ~Stream 端点）
/// 并按 api_gen broadcast_event_name 同款约定拼 New{Type} 事件名。
static API_RETURN_TYPES: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, String>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// PLAN-776 T-06：用户同名遮蔽集——返回类型裸名命中编译单元类型声明
///（struct/type alias/enum，经 TypeStore）时登记。VM 编组门消费
///（fn_response_kind 先查遮蔽 → Json）：同裸名用户类型不冒领 stdlib
/// 专属身份（AC-07；无运行时模块上下文，来源在编译期冻结于此）。
static API_RETURN_SHADOWED: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashSet<String>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

/// PLAN-705 T-04/SD-01: 声明 `~T`（Future<T>）返回的函数名集——HTTP 编组
/// 阶段判定"返回值可能是 future bits"的**元数据门**（禁止把普通 int 的
/// 位模式猜成 future ID 后误消费；AC-01 普通 int 反例由此挡住）。对全部
/// fn 发布（非仅 #[api]——段驱动 handler/`__axum:` fn-ref closure 同样
/// 依此判定；closure 经 func_addr → exports 反查名，同 resolve_params）。
static API_ASYNC_RETURNS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

/// 编组期判定：该函数（或 fn-ref closure 反查出的函数）声明 `~T` 返回。
pub(crate) fn fn_is_api_async(fn_name: &str) -> bool {
    if let Some(found) = with_program_metadata(|p| p.async_returns.contains(fn_name)) {
        return found;
    }
    API_ASYNC_RETURNS
        .lock()
        .ok()
        .map(|t| t.contains(fn_name))
        .unwrap_or(false)
}

/// Plan 346 B6: process-wide rate limiter (fixed window per client IP) and
/// request-id minting for `dispatch_api_request` (PLAN-699: the legacy
/// inline handler this comment named was removed by the Axum transport).

/// Plan 346 5e (B6): per-IP fixed-window buckets `ip -> (window_start_ms, count)`.
static RATE_BUCKETS: std::sync::Mutex<Option<std::collections::HashMap<String, (u64, u32)>>> =
    std::sync::Mutex::new(None);
/// Plan 346 #12 (B6): request-id counter for minted ids (incoming ids pass
/// through unchanged).
static REQ_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

// ── Plan 346 5a (B6 multipart): server-side multipart/form-data parsing ──

/// PLAN-730 T-07：legacy 落盘记录（files JSON 项 + provisional 清理路径）。
pub(crate) struct LegacyStoredFile {
    pub field: String,
    pub filename: String,
    pub path: String,
    pub size: usize,
}

/// Plan 346 5a / PLAN-730 T-07：save file parts under the upload dir (env
/// `AUTO_UPLOAD_DIR`, default `./uploads`)。文件名归约同历史（basename +
/// 计数前缀防同名互踩）。**错误必须传播**（目录创建/写失败 → Err——
/// 不返回不存在的假路径）；在宿主 spawn_blocking 执行，不在 owner。
pub(crate) fn legacy_store_files(
    files: &[(String, String, Vec<u8>)],
) -> Result<Vec<LegacyStoredFile>, String> {
    static FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let dir = std::env::var("AUTO_UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    std::fs::create_dir_all(&dir).map_err(|e| format!("upload dir create failed: {e}"))?;
    let mut stored = Vec::with_capacity(files.len());
    for (field, filename, data) in files {
        let base = filename
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("upload")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect::<String>();
        let base = if base.is_empty() {
            "upload".to_string()
        } else {
            base
        };
        let n = FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let path = std::path::Path::new(&dir).join(format!("{}_{}", n, base));
        std::fs::write(&path, data)
            .map_err(|e| format!("upload write failed for {}: {e}", path.to_string_lossy()))?;
        stored.push(LegacyStoredFile {
            field: field.clone(),
            filename: filename.clone(),
            path: path.to_string_lossy().to_string(),
            size: data.len(),
        });
    }
    Ok(stored)
}

/// PLAN-730 T-07：宿主落盘结果 → files JSON 项（历史形状逐字节保持）。
pub(crate) fn multipart_files_json(stored: &[LegacyStoredFile]) -> String {
    stored
        .iter()
        .map(|f| {
            format!(
                "{{\"field\":\"{}\",\"filename\":\"{}\",\"path\":\"{}\",\"size\":{}}}",
                f.field.replace('"', "\\\""),
                f.filename.replace('"', "\\\""),
                f.path.replace('\\', "/").replace('"', "\\\""),
                f.size
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// PLAN-730 T-07：fields + files 合并为 handler body（B6 历史成功形状）。
pub(crate) fn multipart_handler_json(fields: &str, files: &str) -> String {
    format!("{{\"fields\":{{{}}},\"files\":[{}]}}", fields, files)
}

/// Escape a string as a JSON string literal (with surrounding quotes).
pub(crate) fn json_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// =============================================================================
// PLAN-755 T-02/R2（P755-R2-01）：e2e 启动结果有界上报共享核心
// =============================================================================
/// 三处 e2e 启动助手（plan730 `start_server`、plan326 `start_server`、
/// `start_example_api_server`）统一经此 spawn：`crate::run` 的 Err 与
/// **panic 原始载荷**都经通道回传，父线程在 ready 窗口内即时呈现真实
/// 原因，不落入客户端 connect-refused 等待（751-R2：吞错 × 本机 ~2.15s/次
/// 拒绝连接把启动失败藏成 nextest 120s TERMINATING）。
///
/// panic 载荷必须 downcast 提取字符串——`{panic:?}` 对 `Box<dyn Any>` 只会
/// 得到 `Any { .. }`，丢失原因（R2 负向反例实证）。
#[cfg(any(test, feature = "test-http-e2e"))]
pub(crate) fn spawn_outcome_reporter<F, T>(
    stack_size: usize,
    label: &str,
    body: F,
) -> std::sync::mpsc::Receiver<String>
where
    F: FnOnce() -> crate::error::AutoResult<T> + Send + 'static,
    T: Send + 'static,
{
    let label = label.to_string();
    let (outcome_tx, outcome_rx) = std::sync::mpsc::channel::<String>();
    std::thread::Builder::new()
        .stack_size(stack_size)
        .spawn(move || {
            let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)) {
                Ok(Ok(_)) => format!("{label} returned Ok without serving"),
                Ok(Err(err)) => format!("{label} failed: {err}"),
                Err(panic) => format!("{label} panicked: {}", panic_payload_text(panic)),
            };
            let _ = outcome_tx.send(outcome);
        })
        .expect("spawn server thread");
    outcome_rx
}

/// Cross-crate HTTP test harness port; not present in ordinary product builds.
#[cfg(feature = "test-http-e2e")]
#[doc(hidden)]
pub fn spawn_outcome_reporter_for_test<F, T>(
    stack_size: usize,
    label: &str,
    body: F,
) -> std::sync::mpsc::Receiver<String>
where
    F: FnOnce() -> crate::error::AutoResult<T> + Send + 'static,
    T: Send + 'static,
{
    spawn_outcome_reporter(stack_size, label, body)
}

/// catch_unwind 载荷的字符串提取（`&'static str` / `String` / 未知兜底）。
#[cfg(any(test, feature = "test-http-e2e"))]
pub(crate) fn panic_payload_text(panic: Box<dyn std::any::Any + Send>) -> String {
    match panic.downcast::<String>() {
        Ok(text) => *text,
        Err(panic) => match panic.downcast::<&'static str>() {
            Ok(text) => (*text).to_string(),
            Err(_) => "<non-string panic payload>".to_string(),
        },
    }
}

#[cfg(test)]
mod plan755_outcome_reporter_tests {
    /// P755-R2-01 负向冻结：真实 spawn/通道/catch_unwind 路径必须转发 panic
    /// 原始载荷——父线程侧消息携带原因，而不是 `Any { .. }`。
    #[test]
    fn panic_payload_is_forwarded_through_real_spawn_path() {
        let rx = super::spawn_outcome_reporter::<_, String>(8 * 1024 * 1024, "test-run", || {
            panic!("PLAN755_PANIC_PAYLOAD_MARKER")
        });
        let outcome = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("outcome reported");
        assert!(
            outcome.contains("PLAN755_PANIC_PAYLOAD_MARKER"),
            "父线程消息丢失 panic 载荷: {outcome}"
        );
    }

    #[test]
    fn panic_payload_text_covers_string_static_and_foreign() {
        assert_eq!(
            super::panic_payload_text(Box::new(String::from("owned"))),
            "owned"
        );
        assert_eq!(super::panic_payload_text(Box::new("static")), "static");
        assert_eq!(
            super::panic_payload_text(Box::new(755usize)),
            "<non-string panic payload>"
        );
    }
}

// =============================================================================
// Plan 326 Phase 3: serialization unit tests
// =============================================================================
#[cfg(test)]
mod plan326_tests {
    use super::{format_f64_json, json_escape_string};

    #[test]
    fn json_escape_basic() {
        assert_eq!(json_escape_string("hello"), r#""hello""#);
    }
    #[test]
    fn json_escape_quotes_and_backslash() {
        assert_eq!(json_escape_string(r#"a"b\c"#), r#""a\"b\\c""#);
    }

    #[test]
    fn json_escape_control_chars() {
        assert_eq!(json_escape_string("a\nb\tc"), r#""a\nb\tc""#);
    }

    #[test]
    fn json_escape_unicode_control() {
        // 0x01 is a control char → \u0001
        assert_eq!(json_escape_string("\u{0001}"), r#""\u0001""#);
    }

    #[test]
    fn f64_integer_no_trailing_dot() {
        assert_eq!(format_f64_json(42.0), "42");
        assert_eq!(format_f64_json(-7.0), "-7");
    }

    #[test]
    fn f64_fractional_preserved() {
        assert_eq!(format_f64_json(3.14), "3.14");
    }

    #[test]
    fn f64_nan_and_inf_become_null() {
        assert_eq!(format_f64_json(f64::NAN), "null");
        assert_eq!(format_f64_json(f64::INFINITY), "null");
        assert_eq!(format_f64_json(f64::NEG_INFINITY), "null");
    }

    /// Verify the probe-based id detection: a small plain int (not in any VM
    /// table) must serialize as a plain number, never as an object/array.
    #[test]
    fn plain_int_not_treated_as_id() {
        let vm = fresh_vm();
        // 999999 is below all VM id bases and not inserted anywhere.
        let nv = auto_val::encode_i32(999999);
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some("999999".to_string()));
    }

    // ---------------------------------------------------------------------
    // VM-backed integration tests: construct a real AutoVM, insert objects,
    // and verify nv_to_json expands them correctly.
    // ---------------------------------------------------------------------

    use crate::vm::engine::AutoVM;
    use crate::vm::generic_registry::GenericInstanceData;
    use crate::vm::virt_memory::VirtualFlash;

    fn fresh_vm() -> AutoVM {
        // Empty flash is fine — nv_to_json only touches heap_objects/arrays/
        // objects/string pool, none of which need compiled code.
        let flash = VirtualFlash::new_with_code(vec![]);
        AutoVM::new(flash, 1024)
    }

    /// Plan 317 §11 Phase 11 (Bug fix regression): `get_fn_n_args` must read the
    /// declared parameter count from a function's FN_PROLOG. This is the
    /// primitive `build_handler_args` uses to decide whether to push the
    /// cookies/auth metadata — the root cause of the `e2e_int_path_param` /
    /// `e2e_notes_crud` failures (1-param handlers received the metadata JSON as
    /// their first arg). This test pins the lookup without needing a live HTTP
    /// server, so it runs in the regular (non-`--ignored`) suite.
    #[test]
    fn get_fn_n_args_reads_declared_arity() {
        use crate::vm::opcode::OpCode;
        // Bytecode for two exported "functions":
        //   - "echo_id" at addr 0: FN_PROLOG, n_args=1, n_locals=0, RET
        //   - "create_note" at addr 4: FN_PROLOG, n_args=2, n_locals=0, RET
        let bytecode: Vec<u8> = vec![
            OpCode::FN_PROLOG as u8,
            1,
            0,
            OpCode::RET as u8, // echo_id (1 param)
            OpCode::FN_PROLOG as u8,
            2,
            0,
            OpCode::RET as u8, // create_note (2 params)
        ];
        let mut flash = VirtualFlash::new_with_code(bytecode);
        flash.exports_by_name.insert("echo_id".to_string(), 0);
        flash.exports_by_name.insert("create_note".to_string(), 4);
        let vm = AutoVM::new(flash, 1024);

        assert_eq!(
            vm.get_fn_n_args("echo_id"),
            Some(1),
            "1-param handler arity"
        );
        assert_eq!(
            vm.get_fn_n_args("create_note"),
            Some(2),
            "2-param handler arity"
        );
        assert_eq!(vm.get_fn_n_args("nonexistent"), None, "unknown fn -> None");
    }

    #[test]
    fn nv_to_json_plain_int() {
        let vm = fresh_vm();
        let nv = auto_val::encode_i32(42);
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some("42".to_string()));
    }

    #[test]
    fn nv_to_json_string() {
        let vm = fresh_vm();
        let idx = {
            let mut strings = vm.strings.write().unwrap();
            strings.push(b"hello".to_vec());
            strings.len() - 1
        };
        let nv = auto_val::encode_string(idx as u32);
        assert_eq!(
            super::nv_to_json(&vm, nv, 0),
            Some(r#""hello""#.to_string())
        );
    }

    #[test]
    fn nv_to_json_null() {
        let vm = fresh_vm();
        let nv = auto_val::encode_null();
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some("null".to_string()));
    }

    /// Struct return: the handler leaves a heap object ID (>= 4_000_000) on the
    /// stack as i32. nv_to_json must expand it into {"field": value, ...}.
    #[test]
    fn nv_to_json_struct_expansion() {
        let vm = fresh_vm();
        let inst = GenericInstanceData::new_with_names(
            "Note".to_string(),
            vec![
                auto_val::Value::Int(1),
                auto_val::Value::Str(auto_val::AutoStr::from("hello")),
            ],
            vec!["id".to_string(), "title".to_string()],
        );
        let id = vm.insert_heap_object(inst);
        // The handler return path pushes this id as i32 (see CONSTRUCT_INSTANCE).
        let nv = auto_val::encode_i32(id as i32);
        let json = super::nv_to_json(&vm, nv, 0).unwrap();
        assert_eq!(json, r#"{"id": 1, "title": "hello"}"#);
    }

    /// Array of structs: the handler returns Vec<Value> where each element is
    /// a struct stored as Value::Int(heap_id). Array id is allocated the same
    /// way CREATE_ARRAY does (Plan 390 §15 H3b: ListData<Value> in
    /// heap_objects). nv_to_json must recurse.
    #[test]
    fn nv_to_json_array_of_structs() {
        let vm = fresh_vm();
        let a = GenericInstanceData::new_with_names(
            "Note".to_string(),
            vec![
                auto_val::Value::Int(0),
                auto_val::Value::Str(auto_val::AutoStr::from("a")),
            ],
            vec!["id".to_string(), "title".to_string()],
        );
        let b = GenericInstanceData::new_with_names(
            "Note".to_string(),
            vec![
                auto_val::Value::Int(1),
                auto_val::Value::Str(auto_val::AutoStr::from("b")),
            ],
            vec!["id".to_string(), "title".to_string()],
        );
        let id_a = vm.insert_heap_object(a) as i32;
        let id_b = vm.insert_heap_object(b) as i32;
        // Allocate an array id the same way CREATE_ARRAY does now.
        let arr_id = vm.insert_heap_object(crate::vm::types::ListData {
            elems: vec![auto_val::Value::Int(id_a), auto_val::Value::Int(id_b)],
            storage: None,
        });
        // The handler returns the array id as i32.
        let nv = auto_val::encode_i32(arr_id as i32);
        let json = super::nv_to_json(&vm, nv, 0).unwrap();
        assert_eq!(
            json,
            r#"[{"id": 0, "title": "a"}, {"id": 1, "title": "b"}]"#
        );
    }

    /// Option.Some(x) → unwrap to inner value's JSON.
    #[test]
    fn nv_to_json_option_some() {
        let vm = fresh_vm();
        let inst = GenericInstanceData::new_with_names(
            "Option.Some".to_string(),
            vec![auto_val::Value::Str(auto_val::AutoStr::from("found"))],
            vec!["_0".to_string()],
        );
        let id = vm.insert_heap_object(inst);
        let nv = auto_val::encode_i32(id as i32);
        assert_eq!(
            super::nv_to_json(&vm, nv, 0),
            Some(r#""found""#.to_string())
        );
    }

    /// Option.None → JSON null (Plan 326: we serialize None as `null` to keep
    /// the JSON response well-formed; a 404 mapping can be layered on later).
    #[test]
    fn nv_to_json_option_none() {
        let vm = fresh_vm();
        let inst = GenericInstanceData::new_with_names("Option.None".to_string(), vec![], vec![]);
        let id = vm.insert_heap_object(inst);
        let nv = auto_val::encode_i32(id as i32);
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some("null".to_string()));
    }

    /// Nested struct: a field whose value is itself a struct (VmRef / Int heap-id).
    #[test]
    fn nv_to_json_nested_struct() {
        let vm = fresh_vm();
        let inner = GenericInstanceData::new_with_names(
            "Point".to_string(),
            vec![auto_val::Value::Int(3), auto_val::Value::Int(4)],
            vec!["x".to_string(), "y".to_string()],
        );
        let inner_id = vm.insert_heap_object(inner) as i32;
        let outer = GenericInstanceData::new_with_names(
            "Box".to_string(),
            vec![auto_val::Value::Int(inner_id)],
            vec!["p".to_string()],
        );
        let outer_id = vm.insert_heap_object(outer);
        let nv = auto_val::encode_i32(outer_id as i32);
        let json = super::nv_to_json(&vm, nv, 0).unwrap();
        assert_eq!(json, r#"{"p": {"x": 3, "y": 4}}"#);
    }

    // ---------------------------------------------------------------------
    // PLAN-669: #[api] param-sig side channel (registry round-trip; the
    // codegen publish site is covered end-to-end by the http_e2e battery).
    // ---------------------------------------------------------------------
    #[test]
    fn plan669_api_param_sigs_roundtrip() {
        use super::{api_param_sigs, clear_api_param_sigs, record_api_param_sigs, ApiParamSig};
        clear_api_param_sigs();
        record_api_param_sigs(
            "create_note",
            vec![
                ApiParamSig {
                    name: "title".into(),
                    ty: "str".into(),
                },
                ApiParamSig {
                    name: "id".into(),
                    ty: "int".into(),
                },
            ],
        );
        let sigs = api_param_sigs("create_note").expect("recorded");
        assert_eq!(sigs.len(), 2);
        assert_eq!(sigs[0].name, "title");
        assert_eq!(sigs[1].ty, "int");
        clear_api_param_sigs();
        assert!(api_param_sigs("create_note").is_none());
    }

    // ---------------------------------------------------------------------
    // PLAN-669: by-name binder unit matrix (no live server — the marshalled
    // stack values are inspected directly; e2e shapes run in http_e2e).
    // ---------------------------------------------------------------------
    mod plan669_bind_tests {
        use super::super::{bind_api_args_by_name, ApiArgBindError, ApiParamSig, RouteMatch};
        use crate::vm::engine::AutoVM;
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;

        fn sig(name: &str, ty: &str) -> ApiParamSig {
            ApiParamSig {
                name: name.into(),
                ty: ty.into(),
            }
        }
        fn rm(path_params: Vec<(&str, &str)>, query_params: Vec<(&str, &str)>) -> RouteMatch {
            RouteMatch {
                fn_name: "h".into(),
                path_params: path_params
                    .into_iter()
                    .map(|(n, v)| (n.to_string(), v.to_string()))
                    .collect(),
                query_params: query_params
                    .into_iter()
                    .map(|(n, v)| (n.to_string(), v.to_string()))
                    .collect(),
            }
        }
        fn rig() -> (AutoVM, AutoTask) {
            (
                AutoVM::new(VirtualFlash::new_with_code(vec![]), 1024),
                AutoTask::new(0, 4096, 0),
            )
        }
        fn pop_str(vm: &AutoVM, task: &mut AutoTask) -> String {
            let nv = task.ram.pop_nv();
            assert!(auto_val::is_string(nv), "expected string nv, got {nv:?}");
            let idx = auto_val::decode_string(nv);
            vm.strings
                .read()
                .unwrap()
                .get(idx as usize)
                .map(|b| String::from_utf8_lossy(b).to_string())
                .expect("string pool entry")
        }

        #[test]
        fn body_fields_bind_by_name_in_declaration_order() {
            let (vm, mut task) = rig();
            let body: serde_json::Value =
                serde_json::from_str(r#"{"body":"second","title":"first"}"#).unwrap();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("title", "str"), sig("body", "str")],
                &rm(vec![], vec![]),
                Some(&body),
                "",
                None,
                "POST",
                "/api/notes",
                None,
            )
            .expect("bind");
            assert_eq!(n, 2);
            // LIFO: last-pushed pops first (body was pushed second).
            assert_eq!(pop_str(&vm, &mut task), "second");
            assert_eq!(pop_str(&vm, &mut task), "first");
        }

        #[test]
        fn query_binds_by_name() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("query", "str")],
                &rm(vec![], vec![("q", "ignored"), ("query", "Build")]),
                None,
                "",
                None,
                "GET",
                "/api/search",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            assert_eq!(pop_str(&vm, &mut task), "Build");
        }

        #[test]
        fn typed_int_query_converts_and_rejects() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("page", "int")],
                &rm(vec![], vec![("page", "7")]),
                None,
                "",
                None,
                "GET",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            let nv = task.ram.pop_nv();
            assert!(
                auto_val::is_i32(nv) || auto_val::decode_i32(nv) == 7,
                "int nv {nv:?}"
            );

            let (vm2, mut task2) = rig();
            let err = bind_api_args_by_name(
                &vm2,
                &mut task2,
                &[sig("page", "int")],
                &rm(vec![], vec![("page", "seven")]),
                None,
                "",
                None,
                "GET",
                "/x",
                None,
            )
            .unwrap_err();
            match err {
                ApiArgBindError::BadRequest(m) => {
                    assert!(m.contains("page") && m.contains("int"), "{m}");
                }
                other => panic!("expected BadRequest, got {other:?}"),
            }
        }

        #[test]
        fn bool_query_converts() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("done", "bool")],
                &rm(vec![], vec![("done", "true")]),
                None,
                "",
                None,
                "GET",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            let nv = task.ram.pop_nv();
            assert!(
                auto_val::is_bool(nv) && auto_val::decode_bool(nv),
                "bool nv {nv:?}"
            );
        }

        #[test]
        fn path_wins_over_body_and_query() {
            let (vm, mut task) = rig();
            let body: serde_json::Value = serde_json::from_str(r#"{"id": 999}"#).unwrap();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("id", "int")],
                &rm(vec![("id", "42")], vec![("id", "7")]),
                Some(&body),
                "",
                None,
                "GET",
                "/api/notes/42",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            let nv = task.ram.pop_nv();
            assert!(
                auto_val::decode_i32(nv) == 42,
                "path source must win, nv {nv:?}"
            );
        }

        #[test]
        fn missing_param_is_400_naming_param() {
            let (vm, mut task) = rig();
            let err = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("title", "str"), sig("body", "str")],
                &rm(vec![], vec![]),
                None,
                "",
                None,
                "POST",
                "/api/notes",
                None,
            )
            .unwrap_err();
            match err {
                ApiArgBindError::BadRequest(m) => {
                    assert!(m.contains("missing param `title`"), "{m}");
                }
                other => panic!("expected BadRequest, got {other:?}"),
            }
        }

        #[test]
        fn trailing_unbound_param_receives_metadata() {
            let (vm, mut task) = rig();
            let body: serde_json::Value = serde_json::from_str(r#"{"title":"t"}"#).unwrap();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("title", "str"), sig("meta", "str")],
                &rm(vec![], vec![]),
                Some(&body),
                "",
                Some(r#"{"cookies":{}}"#),
                "POST",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 2);
            assert_eq!(pop_str(&vm, &mut task), r#"{"cookies":{}}"#);
            assert_eq!(pop_str(&vm, &mut task), "t");
        }

        #[test]
        fn non_trailing_unbound_is_missing_even_with_metadata() {
            let (vm, mut task) = rig();
            let err = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("a", "str"), sig("meta", "str")],
                &rm(vec![], vec![]),
                None,
                "",
                Some("{}"),
                "GET",
                "/x",
                None,
            )
            .unwrap_err();
            assert!(matches!(err, ApiArgBindError::BadRequest(_)));
        }

        #[test]
        fn raw_body_single_param_tolerance() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("data", "str")],
                &rm(vec![], vec![]),
                None,
                "plain text not json",
                None,
                "POST",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            assert_eq!(pop_str(&vm, &mut task), "plain text not json");
        }

        #[test]
        fn whole_body_tolerance_covers_parseable_object_too() {
            // Multipart/B6-`form` shape: body parses as JSON but the lone
            // param's name isn't a field of it — legacy single-arg body
            // contract hands it the whole body source.
            let (vm, mut task) = rig();
            let body: serde_json::Value =
                serde_json::from_str(r#"{"fields":{"title":"hello"}}"#).unwrap();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("form", "str")],
                &rm(vec![], vec![]),
                Some(&body),
                r#"{"fields":{"title":"hello"}}"#,
                None,
                "POST",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            assert_eq!(pop_str(&vm, &mut task), r#"{"fields":{"title":"hello"}}"#);
        }

        #[test]
        fn metadata_optin_requires_meta_param_name() {
            // A trailing unbound param NOT named meta/metadata/req/request is
            // a missing param (400), never the cookies/auth payload — this is
            // what makes POSTs with a dropped last field fail loudly.
            let (vm, mut task) = rig();
            let body: serde_json::Value = serde_json::from_str(r#"{"title":"t"}"#).unwrap();
            let err = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("title", "str"), sig("body", "str")],
                &rm(vec![], vec![]),
                Some(&body),
                r#"{"title":"t"}"#,
                Some("{}"),
                "POST",
                "/x",
                None,
            )
            .unwrap_err();
            match err {
                ApiArgBindError::BadRequest(m) => {
                    assert!(m.contains("missing param `body`"), "{m}");
                }
                other => panic!("expected BadRequest, got {other:?}"),
            }
        }

        #[test]
        fn empty_sigs_bind_zero() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[],
                &rm(vec![], vec![]),
                None,
                "",
                None,
                "GET",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 0);
        }

        /// F-669-R1 / AC-06: with no API_PARAM_SIGS entry the sync-serve
        /// helper falls back to the pre-669 positional convention verbatim —
        /// path params in order (i32-heuristic), query ignored, raw body as
        /// one trailing arg. e2e always has sigs (codegen publishes them), so
        /// the fallback arm is only reachable here.
        #[test]
        fn legacy_fallback_without_sigs_is_positional() {
            use super::super::{bind_api_args_or_legacy, clear_api_param_sigs};
            clear_api_param_sigs();
            let (vm, mut task) = rig();
            let n = bind_api_args_or_legacy(
                &vm,
                &mut task,
                "h_legacy_never_published",
                &[
                    ("id".to_string(), "42".to_string()),
                    ("slug".to_string(), "hello".to_string()),
                ],
                &[("ignored".to_string(), "q".to_string())],
                "raw-body",
                "GET",
                "/api/notes/42/hello",
            )
            .expect("legacy bind");
            assert_eq!(n, 3, "id + slug + body, query dropped");
            // LIFO: body (last pushed) pops first, then slug, then id as i32.
            assert_eq!(pop_str(&vm, &mut task), "raw-body");
            assert_eq!(pop_str(&vm, &mut task), "hello");
            let nv = task.ram.pop_nv();
            assert!(
                auto_val::decode_i32(nv) == 42,
                "numeric path param as i32, nv {nv:?}"
            );
        }
    }

    // ---------------------------------------------------------------------
    // Plan 326 Phase 3 end-to-end: spawn the real AutoVM HTTP server with a
    // minimal #[api] program that returns a struct, then assert the HTTP
    // response body is well-formed JSON (not the bare heap-id "4000000").
    // ---------------------------------------------------------------------
    //
    // Plan 317 §11 Phase 11: these e2e tests start a real TCP HTTP server and
    // are gated behind the `test-http-e2e` feature (off by default). They must
    // run serially (`--test-threads=1`) because each server thread is detached
    // (runs until process exit) and they share process-global state
    // (AUTO_HTTP_PORT env var, HTTP_ROUTES table). Each test clears the global
    // route table first and binds a dynamic port to avoid cross-test
    // contamination. Run with:
    //   cargo test -p auto-lang --lib --features test-http-e2e -- --test-threads=1
    #[cfg(feature = "test-http-e2e")]
    mod http_e2e {
        use super::super::spawn_outcome_reporter;
        use crate::vm::ffi::stdlib::clear_http_routes;
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::path::PathBuf;
        use std::time::Duration;

        /// Send a raw HTTP request to localhost:port and return the full response.
        fn http_get(port: u16, path: &str) -> String {
            // Retry-connect for up to ~5s while the server comes up.
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("could not connect to test HTTP server");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            write!(
                stream,
                "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
                path
            )
            .unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).ok();
            resp
        }

        /// Plan 346 B6: GET with extra request headers (e.g. X-Request-Id
        /// passthrough test).
        fn http_get_with_headers(port: u16, path: &str, headers: &[(&str, &str)]) -> String {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("could not connect to test HTTP server");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let mut req = format!(
                "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n",
                path
            );
            for (k, v) in headers {
                req.push_str(&format!("{}: {}\r\n", k, v));
            }
            req.push_str("\r\n");
            write!(stream, "{}", req).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).ok();
            resp
        }

        /// Audit B1: JSON POST with extra request headers (Authorization).
        fn http_post_json_with_headers(
            port: u16,
            path: &str,
            json_body: &str,
            headers: &[(&str, &str)],
        ) -> String {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("connect to test server");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let mut req = format!(
                "POST {} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
                path,
                json_body.len()
            );
            for (k, v) in headers {
                req.push_str(&format!("{}: {}\r\n", k, v));
            }
            req.push_str("\r\n");
            req.push_str(json_body);
            write!(stream, "{}", req).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).ok();
            resp
        }

        /// PLAN-696 T-02: POST a body after the headers in multiple TCP writes.
        /// The delays make the write boundaries observable to the server instead
        /// of letting the OS coalesce the whole request into one read.
        fn http_post_json_in_chunks(port: u16, path: &str, body: &str) -> String {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("connect to test server");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let head = format!(
                "POST {} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                path,
                body.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            std::thread::sleep(Duration::from_millis(40));

            let bytes = body.as_bytes();
            let cuts = [bytes.len() / 3, (bytes.len() * 2) / 3];
            let mut start = 0;
            for end in [cuts[0], cuts[1], bytes.len()] {
                if stream.write_all(&bytes[start..end]).is_err() {
                    break;
                }
                let _ = stream.flush();
                start = end;
                std::thread::sleep(Duration::from_millis(40));
            }
            let _ = stream.shutdown(std::net::Shutdown::Write);
            let mut resp = String::new();
            stream.read_to_string(&mut resp).ok();
            resp
        }

        /// Extract the body (after the blank line) from a raw HTTP response.
        fn body_of(resp: &str) -> &str {
            resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or(resp)
        }

        /// Spin up the AutoVM HTTP server for `code` on a fixed unique `port`,
        /// then block until the server is accepting connections (up to ~5s).
        ///
        /// Fixed ports (not dynamic `:0`) are used deliberately: each test gets
        /// a distinct port, and the OS reliably hands a connect back to the
        /// bound listener. Dynamic `:0` introduced a TOCTOU race (after dropping
        /// the probe listener, the OS could reassign the port to a prior test's
        /// detached server thread), which made `e2e_concurrent_sse` flaky.
        ///
        /// Clears the global route table first (isolation from prior tests'
        /// detached servers, which may still be reading the global HTTP_ROUTES
        /// snapshot). The server thread is detached (blocks forever; the test
        /// process reaps it on exit — fine because CI runs each `cargo test`
        /// invocation as a separate process).
        ///
        /// PLAN-755 T-02/R2: bounded startup diagnostics via the shared
        /// outcome reporter (`spawn_outcome_reporter`). A startup failure (e.g.
        /// SIGNATURE_UNVERIFIED, `crate::run` returning Err within ~30ms, or a
        /// panic whose payload is forwarded) must surface with its real reason
        /// inside the ready window instead of being swallowed by `let _ =` and
        /// buried in client-side connect-refused retries (~2.15s per attempt on
        /// this host — the 751-R2 120s TERMINATING mechanism).
        fn start_server(code: &str, port: u16) -> u16 {
            clear_http_routes();
            std::env::set_var("AUTO_HTTP_PORT", port.to_string());
            let code = code.to_string();
            let outcome_rx =
                spawn_outcome_reporter(8 * 1024 * 1024, "crate::run", move || crate::run(&code));
            // Wait for the server to accept connections before returning, so the
            // first http_get doesn't race against a not-yet-bound listener.
            for _ in 0..50 {
                if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                    return port;
                }
                if let Ok(outcome) = outcome_rx.try_recv() {
                    panic!("test server failed to start on port {port}: {outcome}");
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            panic!(
                "test server never accepted connections on port {port}: {}",
                outcome_rx.try_recv().unwrap_or_else(|_| {
                    "crate::run still blocking but the port is not accepting".to_string()
                })
            );
        }

        /// Start the real back/api.at with its sibling db.at resolved from the
        /// example directory, so Plan 696 exercises the runtime entry point
        /// against the same source the app uses.
        fn start_example_api_server(example: &str, port: u16) -> u16 {
            clear_http_routes();
            std::env::set_var("AUTO_HTTP_PORT", port.to_string());
            let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .expect("repo root")
                .to_path_buf();
            let entry = repo_root
                .join("examples/ui")
                .join(example)
                .join("src/back/api.at");
            let source = std::fs::read_to_string(&entry)
                .unwrap_or_else(|error| panic!("read {}: {error}", entry.display()));
            let source_path = entry.to_string_lossy().into_owned();
            // PLAN-755 T-02/R2：同 start_server 的有界启动诊断（共享上报核心，
            // Err/panic 载荷不吞）。
            let outcome_rx =
                spawn_outcome_reporter(16 * 1024 * 1024, "run_with_capture_and_path", move || {
                    crate::run_with_capture_and_path(&source, &source_path)
                });
            for _ in 0..50 {
                if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                    return port;
                }
                if let Ok(outcome) = outcome_rx.try_recv() {
                    panic!("{example} VM API failed to start on port {port}: {outcome}");
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            panic!(
                "{} VM API did not listen on port {port}: {}",
                example,
                outcome_rx.try_recv().unwrap_or_else(|_| {
                    "run_with_capture_and_path still blocking but the port is not accepting"
                        .to_string()
                })
            );
        }

        /// P442-4: e2e 端口唯一性守卫。nextest 每测试一进程并行执行，两个
        /// 测试复用同一端口时，后绑定进程的请求会打到先绑定进程的服务器上
        /// （无该路由 → 连接被静默丢弃 → 空 body 假红）。历史三组撞号
        /// （Plan 442 §8.5）由本守卫防复发：扫描本文件全部端口字面量，任何
        /// 五位端口号后随右括号出现两次即红。新测试请选用未占用端口。
        #[test]
        fn e2e_ports_unique() {
            let src = include_str!("http_server.rs");
            let mut seen: std::collections::BTreeMap<&str, usize> = Default::default();
            let mut dup: Vec<&str> = Vec::new();
            for (i, line) in src.lines().enumerate() {
                let b = line.as_bytes();
                let mut j = 0;
                while j < b.len() {
                    if b[j].is_ascii_digit() {
                        let start = j;
                        while j < b.len() && b[j].is_ascii_digit() {
                            j += 1;
                        }
                        let run = &line[start..j];
                        let paren_next = j < b.len() && b[j] == b')';
                        if run.len() == 5 && run.starts_with("18") && paren_next {
                            if seen.insert(run, i + 1).is_some() {
                                dup.push(run);
                            }
                        }
                    } else {
                        j += 1;
                    }
                }
            }
            assert!(
                dup.is_empty(),
                "e2e 端口撞号（nextest 并行下输家连到赢家服务器假红）: {:?}",
                dup
            );
        }

        #[test]
        fn e2e_struct_handler_returns_json() {
            let port = start_server(
                r#"
type Note { id int; title str }

#[api(method = "GET", path = "/api/notes/test")]
fn get_note() Note {
    Note { id: 1, title: "hello" }
}
"#,
                18731,
            );
            let resp = http_get(port, "/api/notes/test");
            let body = body_of(&resp);
            // The fix: body must be JSON object, not the bare heap-id "4000000".
            assert_eq!(
                body, r#"{"id": 1, "title": "hello"}"#,
                "struct handler JSON: full resp = {:?}",
                resp
            );
        }

        #[test]
        fn e2e_int_path_param_handler() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/echo/:id")]
fn echo_id(id int) int {
    id
}
"#,
                18732,
            );
            let resp = http_get(port, "/api/echo/42");
            let body = body_of(&resp);
            // Phase 5: :id injected as int 42, returned as-is.
            assert_eq!(body, "42", "int path param: full resp = {:?}", resp);
        }

        /// Plan 317 Phase 3: SSE handler returning a generator (~Iter<int>).
        /// Each yield becomes an SSE data frame. Lazy evaluation means each
        /// next() runs only to the next yield (not the whole body upfront).
        #[test]
        fn e2e_sse_generator_handler() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/counter")]
fn counter_handler() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
"#,
                18733,
            );
            let resp = http_get(port, "/api/counter");
            // SSE response: the body should contain three "data: N\n\n" frames.
            let body = body_of(&resp);
            assert!(body.contains("data: 1"), "SSE frame 1: body={:?}", body);
            assert!(body.contains("data: 2"), "SSE frame 2: body={:?}", body);
            assert!(body.contains("data: 3"), "SSE frame 3: body={:?}", body);
        }

        /// Plan 317 Phase 3 遗留: SSE handler that INDIRECTLY calls a generator
        /// (handler itself has no yield; it calls a generator fn). The handler
        /// returns the iter_id from the inner generator; SSE detection must still
        /// fire on that iter_id.
        #[test]
        fn e2e_sse_indirect_generator() {
            let port = start_server(
                r#"
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
#[api(method = "GET", path = "/api/stream")]
fn stream_handler() ~Iter<int> {
    return counter()
}
"#,
                18734,
            );
            let resp = http_get(port, "/api/stream");
            let body = body_of(&resp);
            assert!(
                body.contains("data: 1"),
                "indirect SSE frame 1: body={:?}",
                body
            );
            assert!(
                body.contains("data: 2"),
                "indirect SSE frame 2: body={:?}",
                body
            );
            assert!(
                body.contains("data: 3"),
                "indirect SSE frame 3: body={:?}",
                body
            );
        }

        /// Plan 442 C2 item ②: the SSE form — a generator yielding
        /// `sse_named_event(name, payload)` objects must stream proper
        /// `event: <name>\ndata: <payload>` frames (not the raw heap handle).
        /// The bare `sse_named_event` is resolved to a VM shim which builds an
        /// opaque Event object; `sse_frame_from_nv` formats it.
        #[test]
        fn e2e_sse_named_event_frames() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/events")]
fn events_handler() ~Iter<int> {
    yield sse_named_event("e1", "hello")
    yield sse_named_event("e2", "world")
}
"#,
                18738,
            );
            let resp = http_get(port, "/api/events");
            let body = body_of(&resp);
            assert!(
                body.contains("event: e1\ndata: \"hello\""),
                "named SSE frame 1: body={:?}",
                body
            );
            assert!(
                body.contains("event: e2\ndata: \"world\""),
                "named SSE frame 2: body={:?}",
                body
            );
        }

        /// Plan 442 C2 item ②: the full `Sse.new(stream).keep_alive(
        /// KeepAlive.new()).into_response()` chain — `into_response()` returns
        /// the generator's iterator id, so the server's iterator→SSE branch
        /// streams the yielded Event frames through the dispatch-3000 SSE arms.
        #[test]
        fn e2e_sse_chain() {
            let port = start_server(
                r#"
dep axum
use.rs axum::response::sse::{Event, KeepAlive, Sse}

#[api(method = "GET", path = "/api/sse-chain")]
fn chain_handler() int {
    var sse = Sse.new(events_stream())
    return sse.keep_alive(KeepAlive.new()).into_response()
}

fn events_stream() ~Iter<int> {
    yield sse_named_event("tick", "42")
}
"#,
                18746,
            );
            let resp = http_get(port, "/api/sse-chain");
            let body = body_of(&resp);
            assert!(
                body.contains("event: tick\ndata: \"42\""),
                "Sse chain SSE frame: body={:?}",
                body
            );
        }

        /// Plan 442 C2 item ③ path (b): pure-logic value-accessor externs
        /// (`value_get_str`/`value_get_bool`/`value_is_null`) read fields out of
        /// a `Value` built by `json.to_value`. They resolve as bare natives.
        #[test]
        fn e2e_value_accessors() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/acc")]
fn acc() str {
    let obj = json.to_value("{\"msg\":\"hi\",\"ok\":true}")
    let s = value_get_str(obj.view, "msg")
    let b = value_get_bool(obj.view, "ok")
    if b {
        return s
    } else {
        return "no"
    }
}

#[api(method = "GET", path = "/isnull")]
fn isnull() str {
    let obj = json.to_value("null")
    if value_is_null(obj.view) {
        return "yes"
    } else {
        return "no"
    }
}

#[api(method = "GET", path = "/arr")]
fn arrrt() int {
    let obj = json.to_value("{\"items\":[1,2,3]}")
    return ok_response(value_get_array(obj.view, "items"))
}

#[api(method = "GET", path = "/nested")]
fn nested() int {
    let obj = json.to_value("{\"nested\":{\"x\":7}}")
    return ok_response(value_get(obj.view, "nested"))
}

#[api(method = "GET", path = "/hex")]
fn hex() str {
    return random_hex(4)
}

#[api(method = "GET", path = "/newid")]
fn newid() str {
    return new_id(8)
}

#[api(method = "GET", path = "/hash")]
fn hash() str {
    return hash_password("p", "salt")
}

#[api(method = "GET", path = "/path")]
fn path() str {
    return path_inner("seg/ment")
}
"#,
                18747,
            );

            let acc = http_get(port, "/acc");
            assert!(
                acc.starts_with("HTTP/1.1 200"),
                "value accessor status: {}",
                acc.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&acc),
                "\"hi\"",
                "value_get_str/value_get_bool: full = {:?}",
                acc
            );

            let isnull = http_get(port, "/isnull");
            assert_eq!(
                body_of(&isnull),
                "\"yes\"",
                "value_is_null on JSON null: full = {:?}",
                isnull
            );

            let arr = http_get(port, "/arr");
            assert_eq!(
                body_of(&arr),
                "[1, 2, 3]",
                "value_get_array: full = {:?}",
                arr
            );

            let nested = http_get(port, "/nested");
            assert_eq!(
                body_of(&nested),
                r#"{"x": 7}"#,
                "value_get nested object: full = {:?}",
                nested
            );

            // random_hex/new_id return a hex string of the request length.
            let h = body_of(&http_get(port, "/hex"))
                .trim_matches('"')
                .to_string();
            assert_eq!(h.len(), 8, "random_hex(4) should be 8 hex chars: {:?}", h);
            assert!(
                h.chars().all(|c| c.is_ascii_hexdigit()),
                "random_hex not hex: {:?}",
                h
            );

            let n = body_of(&http_get(port, "/newid"))
                .trim_matches('"')
                .to_string();
            assert_eq!(n.len(), 16, "new_id(8) should be 16 hex chars: {:?}", n);

            // hash_password = sha256(salt || p) hex (64 chars, deterministic).
            let h = body_of(&http_get(port, "/hash"))
                .trim_matches('"')
                .to_string();
            assert_eq!(h.len(), 64, "hash_password should be 64 hex chars: {:?}", h);
            assert!(
                h.chars().all(|c| c.is_ascii_hexdigit()),
                "hash_password not hex: {:?}",
                h
            );

            // path_inner on a string is identity (axum Path pushes a string).
            assert_eq!(
                body_of(&http_get(port, "/path")),
                "\"seg/ment\"",
                "path_inner identity: full = {:?}",
                http_get(port, "/path")
            );
        }

        /// Plan 442 C2 item ③ path (a): a data-source extern
        /// (`app_config_effective_daemon_url`) forwards to a registered host call
        /// (`host_bridge`) when one is present, else serves a default constant.
        /// Proves the extern→backend routing mechanism works when a backend
        /// registers the call (auto-musk cdylib). String-returning → no nested
        /// object RC pitfalls.
        #[test]
        fn e2e_host_forward_app_config_daemon() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/daemon")]
fn daemon() str {
    return app_config_effective_daemon_url(0)
}
"#,
                18748,
            );

            // Phase 1: no host call registered → default constant.
            let d = http_get(port, "/daemon");
            assert_eq!(
                body_of(&d),
                "\"http://127.0.0.1:17654\"",
                "app_config_effective_daemon_url default: full = {:?}",
                d
            );

            // Phase 2: register a host call → the VM extern forwards to it.
            crate::vm::host_bridge::register_host_call(
                "app_config_effective_daemon_url",
                std::sync::Arc::new(move |_args: &str| -> Result<String, String> {
                    Ok(r#""http://10.0.0.5:9999""#.to_string())
                }),
            );
            let f = http_get(port, "/daemon");
            assert_eq!(
                body_of(&f),
                "\"http://10.0.0.5:9999\"",
                "app_config_effective_daemon_url host-forwarded: full = {:?}",
                f
            );
        }

        /// Plan 442 C2 item ③ path (a): a Value-returning data extern
        /// (`relay_runs_list`) forwards to a registered host call, else serves a
        /// default. Exercises the heap-ref dead-zone compensation: the fresh
        /// `{runs: []}` object survives the CALL_NAT dead-zone release.
        #[test]
        fn e2e_host_forward_relay_runs() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/runs")]
fn runs() int {
    return ok_response(relay_runs_list(0, 0))
}
"#,
                18749,
            );

            // Phase 1: no host call → empty-store default shape.
            let d = http_get(port, "/runs");
            assert_eq!(
                body_of(&d),
                r#"{"runs": []}"#,
                "relay_runs_list default (empty store): full = {:?}",
                d
            );

            // Phase 2: register host call → the VM extern forwards to it,
            // passing the (marshalled) request args in args_json.
            crate::vm::host_bridge::register_host_call(
                "relay_runs_list",
                std::sync::Arc::new(move |args: &str| -> Result<String, String> {
                    Ok(format!(r#"{{"runs":[{{"run_id":"r1","arg":{}}}]}}"#, args))
                }),
            );
            let f = http_get(port, "/runs");
            let fbody = body_of(&f);
            // The shim marshalled q (an int 0 in this test) → "0", and forwarded it.
            assert!(
                fbody.contains(r#""run_id": "r1""#) && fbody.contains(r#""arg": 0"#),
                "relay_runs_list host-forwarded with args: full = {:?}",
                fbody
            );
        }

        /// Fetch `path` repeatedly until the body contains all `need_fragments`,
        /// or `max_attempts` is exhausted. Returns the last body.
        ///
        /// The concurrent SSE test fires two simultaneous connections at the
        /// single-worker `serve_async`. The two `spawn_local` handler tasks
        /// interleave via `yield_now`, but on a loaded machine one connection's
        /// SSE stream can be cut short by the client read-timeout (5s) before
        /// all frames flush — a real nondeterminism in the cooperative schedule,
        /// not a server bug. Since the endpoint is an idempotent GET, retrying
        /// the connection is a faithful client behavior and removes the test's
        /// dependence on scheduler timing.
        fn http_get_until(
            port: u16,
            path: &str,
            need_fragments: &[&str],
            max_attempts: usize,
        ) -> String {
            let mut last_body = String::new();
            for _ in 0..max_attempts {
                let resp = http_get(port, path);
                last_body = body_of(&resp).to_string();
                if need_fragments.iter().all(|f| last_body.contains(f)) {
                    return last_body;
                }
            }
            last_body
        }

        /// Plan 317 Phase 4: concurrent SSE — two simultaneous connections to the
        /// same streaming endpoint. Both must receive complete data. Under the old
        /// serial server, the second connection would block until the first's
        /// generator exhausted. With serve_async + spawn_local + yield_now, the
        /// two handlers interleave (Goroutine-style cooperative scheduling).
        #[test]
        fn e2e_concurrent_sse() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/count")]
fn counter_handler() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
"#,
                18735,
            );

            let need = &["data: 1", "data: 2", "data: 3"];
            // Fire two connections concurrently from separate threads; each retries
            // its own connection until it sees all three frames (cooperative
            // scheduling can truncate a stream under load — see http_get_until).
            // Generous attempt count (8) tolerates accumulated detached-server
            // threads from prior tests slowing the tokio runtime within a suite.
            let h1 = std::thread::spawn(move || http_get_until(port, "/api/count", need, 8));
            let h2 = std::thread::spawn(move || http_get_until(port, "/api/count", need, 8));
            let body1 = h1.join().expect("conn1");
            let body2 = h2.join().expect("conn2");
            // Both connections must receive all three frames.
            assert!(
                body1.contains("data: 1") && body1.contains("data: 2") && body1.contains("data: 3"),
                "conn1 incomplete: body={:?}",
                body1
            );
            assert!(
                body2.contains("data: 1") && body2.contains("data: 2") && body2.contains("data: 3"),
                "conn2 incomplete: body={:?}",
                body2
            );
        }

        /// PLAN-696 T-02 red sample: ordinary request bodies must be read in
        /// full even when headers and body, and then body chunks, arrive in
        /// separate TCP writes. The JSON itself is pretty-printed and carries
        /// an escaped newline in a string value.
        #[test]
        fn e2e_plan696_body_split_across_tcp_writes() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/echo")]
fn echo(text str) str { text }
"#,
                18760,
            );
            let body = "{\n  \"text\": \"first\\nsecond\"\n}";
            let resp = http_post_json_in_chunks(port, "/api/echo", body);
            assert!(
                resp.starts_with("HTTP/1.1 200 OK"),
                "split body status: {:?}",
                resp
            );
            assert_eq!(
                body_of(&resp),
                r#""first\nsecond""#,
                "split body response: {:?}",
                resp
            );
        }

        /// PLAN-696 T-02 red sample: EOF before Content-Length is satisfied
        /// must reject the request before dispatching a body-independent route.
        #[test]
        fn e2e_plan696_short_body_rejected() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/ready")]
fn ready() str { "ready" }
"#,
                18761,
            );
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            stream.write_all(
                b"POST /api/ready HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 12\r\nConnection: close\r\n\r\nabc",
            ).unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).unwrap();
            assert!(
                resp.starts_with("HTTP/1.1 400"),
                "short body must be rejected, got: {:?}",
                resp
            );
        }

        /// PLAN-696 T-02: reject Content-Length above the shared request-body
        /// limit before allocating or dispatching the request.
        #[test]
        fn e2e_plan696_over_limit_body_rejected() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/ready")]
fn ready() str { "ready" }
"#,
                18763,
            );
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            stream.write_all(
                b"POST /api/ready HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 10485761\r\nConnection: close\r\n\r\n",
            ).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).unwrap();
            assert!(
                resp.starts_with("HTTP/1.1 413"),
                "oversize body must be rejected, got: {:?}",
                resp
            );
        }

        /// PLAN-696 T-03: reject malformed Content-Length instead of silently
        /// treating it as a bodyless request.
        #[test]
        fn e2e_plan696_invalid_content_length_rejected() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/ready")]
fn ready() str { "ready" }
"#,
                18764,
            );
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            stream.write_all(
                b"POST /api/ready HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: not-a-number\r\nConnection: close\r\n\r\n",
            ).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).unwrap();
            assert!(
                resp.starts_with("HTTP/1.1 400"),
                "malformed length must be rejected, got: {:?}",
                resp
            );
        }

        /// PLAN-696 T-02 red sample: a delayed generator must not keep the
        /// single-thread I/O reactor from serving an unrelated health request.
        #[test]
        fn e2e_plan696_slow_sse_does_not_block_health() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/slow")]
fn slow() ~Iter<int> {
    yield 1
    Time.sleep_ms(2500)
    yield 2
}
#[api(method = "GET", path = "/health")]
fn health() str { "ok" }
"#,
                18762,
            );

            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            stream
                .write_all(
                    b"GET /api/slow HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: keep-alive\r\n\r\n",
                )
                .unwrap();
            let mut received = Vec::new();
            let mut chunk = [0u8; 512];
            while !received
                .windows(b"data: 1\n\n".len())
                .any(|w| w == b"data: 1\n\n")
            {
                let n = stream.read(&mut chunk).expect("read first SSE frame");
                assert!(n > 0, "SSE closed before first frame: {:?}", received);
                received.extend_from_slice(&chunk[..n]);
            }
            let first_frame_at = std::time::Instant::now();

            let started = std::time::Instant::now();
            let health = http_get(port, "/health");
            let elapsed = started.elapsed();
            assert!(
                health.starts_with("HTTP/1.1 200"),
                "health response: {:?}",
                health
            );
            assert!(
                elapsed < Duration::from_millis(1500),
                "health request waited {elapsed:?} behind the slow SSE producer"
            );

            while !received
                .windows(b"data: 2\n\n".len())
                .any(|w| w == b"data: 2\n\n")
            {
                let n = stream.read(&mut chunk).expect("read second SSE frame");
                assert!(n > 0, "SSE closed before second frame: {:?}", received);
                received.extend_from_slice(&chunk[..n]);
            }
            let producer_delay = first_frame_at.elapsed();
            assert!(
                producer_delay >= Duration::from_secs(2),
                "Time.sleep_ms delay was not preserved: {producer_delay:?}"
            );
        }

        /// PLAN-696 T-06: closing the SSE client stops a sleeping generator
        /// before its post-sleep side effect runs.
        #[test]
        fn e2e_plan696_sse_disconnect_cancels_generator() {
            let port = start_server(
                r#"
var produced int = 0
#[api(method = "GET", path = "/api/cancel")]
fn cancel_stream() ~Iter<int> {
    yield 1
    Time.sleep_ms(2500)
    produced = 1
    yield 2
}
#[api(method = "GET", path = "/api/produced")]
fn produced_count() int { produced }
"#,
                18765,
            );

            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            stream.write_all(
                b"GET /api/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: keep-alive\r\n\r\n",
            ).unwrap();
            let mut received = Vec::new();
            let mut chunk = [0u8; 512];
            while !received
                .windows(b"data: 1\n\n".len())
                .any(|w| w == b"data: 1\n\n")
            {
                let n = stream.read(&mut chunk).expect("read first SSE frame");
                assert!(n > 0, "SSE closed before first frame: {:?}", received);
                received.extend_from_slice(&chunk[..n]);
            }
            drop(stream);

            std::thread::sleep(Duration::from_millis(3200));
            let state = http_get(port, "/api/produced");
            assert_eq!(
                body_of(&state),
                "0",
                "disconnected SSE generator continued: {state:?}"
            );
        }

        /// PLAN-696 T-07: the real 015-notes api.at/db.at pair preserves CRUD
        /// response shape and session state through the owner-thread server.
        #[test]
        fn e2e_plan696_real_015_notes_crud_parity() {
            let port = start_example_api_server("015-notes", 18766);
            let initial = http_get(port, "/api/notes");
            assert!(
                initial.starts_with("HTTP/1.1 200"),
                "initial list: {initial}"
            );
            let notes: serde_json::Value = serde_json::from_str(body_of(&initial)).unwrap();
            assert!(notes
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["title"] == "Welcome"));

            let created = http_post_json_with_headers(
                port,
                "/api/notes",
                r#"{"title":"Plan 696 parity","body":"body split","folder":"work"}"#,
                &[],
            );
            assert!(
                created.starts_with("HTTP/1.1 200"),
                "create response: {created}"
            );
            let created_note: serde_json::Value =
                serde_json::from_str(body_of(&created)).expect("created note JSON");
            assert_eq!(created_note["title"], "Plan 696 parity");
            assert_eq!(created_note["id"], 6);

            let persisted = http_get(port, "/api/notes");
            let notes: serde_json::Value = serde_json::from_str(body_of(&persisted)).unwrap();
            assert!(notes.as_array().unwrap().iter().any(|n| n["id"] == 6));
        }

        /// PLAN-696 T-07: the real 017-chat API keeps the message response and
        /// subsequent list response consistent across the VM HTTP requests.
        #[test]
        fn e2e_plan696_real_017_chat_crud_parity() {
            let port = start_example_api_server("017-chat", 18768);
            let contacts = http_get(port, "/api/contacts");
            assert!(contacts.starts_with("HTTP/1.1 200"), "contacts: {contacts}");
            assert!(
                body_of(&contacts).contains("Alice"),
                "contact seed: {contacts}"
            );

            let sent = http_post_json_with_headers(
                port,
                "/api/messages",
                r#"{"sender":"You","text":"Plan 696 parity"}"#,
                &[],
            );
            assert!(sent.starts_with("HTTP/1.1 200"), "send response: {sent}");
            let sent_message: serde_json::Value =
                serde_json::from_str(body_of(&sent)).expect("message JSON");
            assert_eq!(sent_message["text"], "Plan 696 parity");

            let listed = http_get(port, "/api/messages");
            assert!(listed.starts_with("HTTP/1.1 200"), "message list: {listed}");
            let messages: serde_json::Value = serde_json::from_str(body_of(&listed)).unwrap();
            assert!(messages
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["text"] == "Plan 696 parity"));
        }

        /// PLAN-696 T-07: real 023-realworld auth and article responses retain
        /// the current empty-record failure conventions and use the bearer
        /// token metadata binding for authenticated routes.
        #[test]
        fn e2e_plan696_real_023_auth_parity() {
            let port = start_example_api_server("023-realworld", 18767);
            let login = http_post_json_with_headers(
                port,
                "/api/users/login",
                r#"{"email":"sarah@vercel.com","password":"sarah-secret"}"#,
                &[],
            );
            assert!(login.starts_with("HTTP/1.1 200"), "login response: {login}");
            let user: serde_json::Value = serde_json::from_str(body_of(&login)).unwrap();
            assert_eq!(user["username"], "Sarah Chen");
            assert!(user["token"]
                .as_str()
                .unwrap_or_default()
                .starts_with("tok-"));
            assert!(user.get("password").is_none(), "password leaked: {user}");

            let token = user["token"].as_str().unwrap();
            let authorization = format!("Bearer {token}");
            let current = http_get_with_headers(
                port,
                "/api/user",
                &[("Authorization", authorization.as_str())],
            );
            assert!(
                current.starts_with("HTTP/1.1 200"),
                "current user: {current}"
            );
            let current: serde_json::Value = serde_json::from_str(body_of(&current)).unwrap();
            assert_eq!(current["username"], "Sarah Chen");

            let article = http_post_json_with_headers(
                port,
                "/api/articles",
                r#"{"slug":"plan-696-parity","title":"Plan 696","description":"runtime check","body":"auth owner","tagList":"verification"}"#,
                &[("Authorization", authorization.as_str())],
            );
            assert!(
                article.starts_with("HTTP/1.1 200"),
                "authenticated article: {article}"
            );
            let article: serde_json::Value = serde_json::from_str(body_of(&article)).unwrap();
            assert_eq!(article["slug"], "plan-696-parity");
            assert_eq!(article["author"], "Sarah Chen");

            let anonymous_article = http_post_json_with_headers(
                port,
                "/api/articles",
                r#"{"slug":"anonymous","title":"Anonymous","description":"","body":"","tagList":""}"#,
                &[],
            );
            assert!(
                anonymous_article.starts_with("HTTP/1.1 200"),
                "anonymous article convention changed: {anonymous_article}"
            );
            let anonymous_article: serde_json::Value =
                serde_json::from_str(body_of(&anonymous_article)).unwrap();
            assert_eq!(anonymous_article["slug"], "");

            let rejected = http_post_json_with_headers(
                port,
                "/api/users/login",
                r#"{"email":"sarah@vercel.com","password":"wrong"}"#,
                &[],
            );
            assert!(
                rejected.starts_with("HTTP/1.1 200"),
                "invalid-login convention changed: {rejected}"
            );
            let rejected: serde_json::Value = serde_json::from_str(body_of(&rejected)).unwrap();
            assert_eq!(rejected["id"], 0);
        }

        /// Plan 346 3c: `http.response_redirect(url, 302)` end-to-end. The
        /// handler returns a redirect Response object; the client must see
        /// 302 + Location, and following it reaches the target endpoint.
        ///
        /// Named `e2e_a_...` so it sorts FIRST in the serial suite: a prior
        /// test's detached server thread can auto-start late and read the
        /// process-global AUTO_HTTP_PORT while THIS test owns it, binding our
        /// port with its stale route table (404). Running first avoids the
        /// pre-existing harness race (see Plan 317 §11 detached-server notes).
        #[test]
        fn e2e_a_redirect_302_with_location() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/old")]
fn old_handler() int {
    return http.response_redirect("/new", 302)
}

#[api(method = "GET", path = "/new")]
fn new_handler() str {
    return "arrived"
}
"#,
                18740,
            );
            let resp = http_get(port, "/old");
            assert!(
                resp.starts_with("HTTP/1.1 302"),
                "expected 302 status line, got full response: {:?}",
                resp
            );
            assert!(
                resp.to_lowercase().contains("location: /new"),
                "expected Location: /new header, got: {:?}",
                resp
            );
            // Following the redirect reaches the target.
            let follow = http_get(port, "/new");
            assert!(
                body_of(&follow).contains("arrived"),
                "follow target should serve body, got: {:?}",
                follow
            );
        }

        /// Plan 442 C2: the musk backend's extern response-constructor shims
        /// (`ok_response`/`err_response`/`text_response` …) must produce real
        /// HttpResponseData objects served by the response-object path —
        /// instead of the empty `extern_sigs` no-op that answered `200 null`.
        /// Bare names resolve via BIGVM_NATIVES → CALL_NAT, so no `use
        /// extern_sigs` is needed here.
        #[test]
        fn e2e_musk_response_constructors() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/ok")]
fn ok_handler() int {
    return ok_response("hello")
}

#[api(method = "GET", path = "/err")]
fn err_handler() int {
    return err_response("boom", 500)
}

#[api(method = "GET", path = "/text")]
fn text_handler() int {
    return text_response("not found", 404)
}

#[api(method = "GET", path = "/to")]
fn to_handler() int {
    return to_response(null, "failed", 500)
}
"#,
                18739,
            );

            let ok = http_get(port, "/ok");
            assert!(
                ok.starts_with("HTTP/1.1 200"),
                "ok_response status: {}",
                ok.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&ok),
                "\"hello\"",
                "ok_response JSON body: full = {:?}",
                ok
            );

            let err = http_get(port, "/err");
            assert!(
                err.starts_with("HTTP/1.1 500"),
                "err_response status: {}",
                err.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&err),
                r#"{"error":"boom"}"#,
                "err_response body: full = {:?}",
                err
            );

            let text = http_get(port, "/text");
            assert!(
                text.starts_with("HTTP/1.1 404"),
                "text_response status: {}",
                text.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&text),
                "not found",
                "text_response body: full = {:?}",
                text
            );

            // to_response with a null value degrades to err_response.
            let to = http_get(port, "/to");
            assert!(
                to.starts_with("HTTP/1.1 500"),
                "to_response(null,…) status: {}",
                to.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&to),
                r#"{"error":"failed"}"#,
                "to_response(null,…) body: full = {:?}",
                to
            );
        }

        /// Audit B1 (023-realworld real token auth): loads the REAL 023
        /// back-end sources (api.at types + db.at auth logic) and drives the
        /// full auth flow over HTTP. The VM server binds a POST body as ONE
        /// string arg (per-field binding is the api_gen rust path), so thin
        /// #[api] shims (h_login etc.) parse fields via the real json_str
        /// helper; the db logic (password check, token minting/validation,
        /// author-from-token) is the verbatim 023 source.
        #[test]
        fn e2e_b1_realworld_token_auth() {
            let back = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/ui/023-realworld/src/back");
            let api_src = match std::fs::read_to_string(back.join("api.at")) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("skip: 023 api.at unreadable: {}", e);
                    return;
                }
            };
            let db_src =
                std::fs::read_to_string(back.join("db.at")).expect("023 db.at must exist in-repo");

            // Types block from api.at (up to `use db`), db.at minus its own
            // `use api:` import, plus bearer_token/json_str extracted verbatim.
            let types_block: String = api_src.split("use db").next().unwrap_or("").to_string();
            let db_body = db_src
                .lines()
                .filter(|l| !l.starts_with("use api:"))
                .collect::<Vec<_>>()
                .join("\n");
            let bearer_fn = api_src
                .split("pub fn bearer_token")
                .nth(1)
                .and_then(|rest| rest.split("// --- Stage 1: auth ---").next())
                .map(|body| format!("pub fn bearer_token{}", body))
                .expect("bearer_token fn present");
            let json_str_fn = api_src
                .split("pub fn json_str")
                .nth(1)
                .and_then(|rest| rest.split("/// Extract the bearer").next())
                .map(|body| format!("pub fn json_str{}", body))
                .expect("json_str fn present");

            let code = format!(
                r#"{}{}
{}
{}
#[api(method = "POST", path = "/api/users/login")]
fn h_login(email str, password str) User {{
    return login(email, password)
}}

#[api(method = "POST", path = "/api/users")]
fn h_register(username str, email str, password str) User {{
    return register(username, email, password)
}}

#[api(method = "GET", path = "/api/user")]
fn h_current_user(meta str) User {{
    return current_user(bearer_token(meta))
}}


#[api(method = "POST", path = "/api/articles")]
fn h_create_article(slug str, title str, description str, body str, tagList str, meta str) Article {{
    let u User = current_user(bearer_token(meta))
    if u.id == 0 {{
        let rejected Article = Article {{ slug: "", title: "", description: "", body: "", tagList: "", author: "", favoritesCount: 0, createdAt: "" }}
        return rejected
    }}
    return create_article(slug, title, description, body, tagList, u.username)
}}
"#,
                types_block, bearer_fn, json_str_fn, db_body
            );
            let port = start_server(&code, 18745);

            // 1. Wrong password → empty user (the old stub matched email alone).
            let bad = body_of(&http_post_json_with_headers(
                port,
                "/api/users/login",
                r#"{"email":"sarah@vercel.com","password":"WRONG"}"#,
                &[],
            ))
            .to_string();
            assert!(
                bad.contains("\"id\": 0"),
                "wrong password must fail: {}",
                bad
            );

            // 2. Correct login → real user + fresh unique token.
            let ok = body_of(&http_post_json_with_headers(
                port,
                "/api/users/login",
                r#"{"email":"sarah@vercel.com","password":"sarah-secret"}"#,
                &[],
            ))
            .to_string();
            assert!(ok.contains("\"id\": 1"), "login id: {}", ok);
            assert!(ok.contains("\"token\": \"tok-"), "minted token: {}", ok);
            let tok = {
                let s = ok.find("\"token\": \"tok-").expect("token idx") + 10;
                let e = ok[s..].find('"').expect("token end") + s;
                ok[s..e].to_string()
            };

            // 3. GET /api/user with Bearer → the logged-in user (meta path).
            let me = body_of(&http_get_with_headers(
                port,
                "/api/user",
                &[("Authorization", &format!("Bearer {}", tok))],
            ))
            .to_string();
            assert!(me.contains("\"username\": \"Sarah Chen\""), "me: {}", me);

            // 4. Without a token → logged out.
            let anon = body_of(&http_get(port, "/api/user")).to_string();
            assert!(anon.contains("\"id\": 0"), "anon: {}", anon);

            // 5. Author comes from the token, not the client.
            let created = body_of(&http_post_json_with_headers(
                port,
                "/api/articles",
                r#"{"slug":"authed-post","title":"Authed","description":"d","body":"b","tagList":"x"}"#,
                &[("Authorization", &format!("Bearer {}", tok))],
            ))
            .to_string();
            assert!(
                created.contains("\"author\": \"Sarah Chen\""),
                "author from token: {}",
                created
            );
            let rejected = body_of(&http_post_json_with_headers(
                port,
                "/api/articles",
                r#"{"slug":"anon-post","title":"A","description":"d","body":"b","tagList":"x"}"#,
                &[],
            ))
            .to_string();
            assert!(
                rejected.contains("\"slug\": \"\""),
                "anonymous create rejected: {}",
                rejected
            );
        }

        /// Plan 346 5a (B6): server-side multipart/form-data — POST a text
        /// field + a 20KB binary file (incl. 0xFF bytes and near-boundary
        /// CRLF--X sequences, and a body > the initial 8KB read to exercise
        /// the byte-level continuation). The handler receives
        /// {"fields":{...},"files":[{field,filename,path,size}]} as its body
        /// param; the test also verifies the persisted file's bytes.
        #[test]
        fn e2e_b6_multipart_upload_field_and_file() {
            let upload_dir = std::env::temp_dir().join("auto_multipart_e2e");
            let _ = std::fs::remove_dir_all(&upload_dir);
            std::env::set_var("AUTO_UPLOAD_DIR", upload_dir.to_str().unwrap());
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/upload")]
fn upload(form str) str {
    return form
}
"#,
                18744,
            );

            let boundary = "AutoBoundary7381";
            let mut file_data: Vec<u8> = Vec::new();
            for i in 0..20_000usize {
                // Pseudo-varied binary: high bytes, plain ascii, and a
                // near-boundary sequence that must NOT split a part.
                file_data.push([0xFFu8, b'A', b'\r', b'\n', b'-', b'-', b'X'][i % 7]);
            }
            let mut body: Vec<u8> = Vec::new();
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(
                b"Content-Disposition: form-data; name=\"title\"\r\n\r\nhello upload\r\n",
            );
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(
                b"Content-Disposition: form-data; name=\"avatar\"; filename=\"pic a.bin\"\r\n\
                  Content-Type: application/octet-stream\r\n\r\n",
            );
            body.extend_from_slice(&file_data);
            body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

            let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let req = format!(
                "POST /api/upload HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: multipart/form-data; boundary={}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                boundary,
                body.len()
            );
            use std::io::Write as _;
            stream.write_all(req.as_bytes()).unwrap();
            stream.write_all(&body).unwrap();
            let mut resp = String::new();
            use std::io::Read as _;
            stream.read_to_string(&mut resp).ok();

            let resp_body = body_of(&resp).to_string();
            assert!(
                resp_body.contains("hello upload"),
                "text field must reach the handler, got: {:?}",
                &resp_body[..resp_body.len().min(400)]
            );
            assert!(
                resp_body.contains("pic a.bin"),
                "file part metadata expected, got: {:?}",
                &resp_body[..resp_body.len().min(400)]
            );
            // The response wraps the handler JSON as a JSON string (inner
            // quotes escaped), so pin size via the unescaped `:<len>` tail.
            assert!(
                resp_body.contains(&format!(":{}", file_data.len())),
                "binary size must survive byte-level reads, got: {:?}",
                &resp_body[..resp_body.len().min(400)]
            );
            // The persisted file is discoverable in the upload dir — verify
            // its bytes match the uploaded binary exactly (escape-free).
            let mut stored_files: Vec<_> = std::fs::read_dir(&upload_dir)
                .expect("upload dir exists")
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file())
                .collect();
            stored_files.sort();
            assert_eq!(
                stored_files.len(),
                1,
                "exactly one stored file: {:?}",
                stored_files
            );
            let stored = std::fs::read(&stored_files[0])
                .unwrap_or_else(|e| panic!("stored file readable: {}", e));
            assert_eq!(
                stored, file_data,
                "persisted bytes must match uploaded bytes exactly"
            );

            let _ = std::fs::remove_dir_all(&upload_dir);
        }

        /// Plan 346 #12 (B6): every response carries X-Request-Id — minted
        /// (`req-<ms>-<n>`) when the client sends none.
        #[test]
        fn e2e_b6_request_id_generated_and_echoed() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/rid")]
fn rid() int {
    return 7
}
"#,
                18741,
            );
            let resp = http_get(port, "/api/rid");
            assert!(
                resp.to_lowercase().contains("x-request-id: req-"),
                "minted request id header expected, got: {:?}",
                resp
            );
            // 404 responses carry it too.
            let not_found = http_get(port, "/api/nope");
            assert!(
                not_found.to_lowercase().contains("x-request-id: req-"),
                "404 should carry request id, got: {:?}",
                not_found
            );
        }

        /// Plan 346 #12 (B6): an incoming X-Request-Id passes through verbatim
        /// (trace propagation).
        #[test]
        fn e2e_b6_request_id_incoming_passthrough() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/rid2")]
fn rid2() int {
    return 8
}
"#,
                18742,
            );
            let resp = http_get_with_headers(port, "/api/rid2", &[("X-Request-Id", "my-trace-42")]);
            assert!(
                resp.to_lowercase().contains("x-request-id: my-trace-42"),
                "incoming request id should be echoed verbatim, got: {:?}",
                resp
            );
            assert!(
                !resp.to_lowercase().contains("x-request-id: req-"),
                "minted id should not override the incoming one, got: {:?}",
                resp
            );
        }

        /// Plan 346 5e (B6): http.rate_limit(2, 60000) — the third request
        /// from the same IP gets 429 + Retry-After. start_server resets the
        /// limiter via clear_http_routes, so this test cannot poison others.
        #[test]
        fn e2e_zz_rate_limit_429_after_quota() {
            let port = start_server(
                r#"
http.rate_limit(2, 60000)

#[api(method = "GET", path = "/api/rl")]
fn rl() int {
    return 1
}
"#,
                18743,
            );
            let first = http_get(port, "/api/rl");
            let second = http_get(port, "/api/rl");
            assert!(
                first.starts_with("HTTP/1.1 200"),
                "first should pass, got: {:?}",
                first
            );
            assert!(
                second.starts_with("HTTP/1.1 200"),
                "second should pass, got: {:?}",
                second
            );
            let third = http_get(port, "/api/rl");
            assert!(
                third.starts_with("HTTP/1.1 429"),
                "third request should be rate limited, got: {:?}",
                third
            );
            assert!(
                third.to_lowercase().contains("retry-after:"),
                "429 should carry Retry-After, got: {:?}",
                third
            );
            assert!(
                body_of(&third).contains("rate limit exceeded"),
                "429 body should explain, got: {:?}",
                third
            );
            assert!(
                third.to_lowercase().contains("x-request-id: "),
                "429 should carry the request id, got: {:?}",
                third
            );
        }

        /// Plan 317 Phase 4 validation: 015-notes-style CRUD on the async HTTP
        /// server. Exercises the same patterns as examples/ui/015-notes/src/back:
        ///   - list: returns []Note (array of structs → JSON array of objects)
        ///   - get:  :id path param + ?Note (Option → inner value or null)
        ///   - create: POST body (title/body) → Note
        /// Uses a module-level var for in-memory storage (like db.at's `var notes`).
        #[test]
        fn e2e_notes_crud() {
            let port = start_server(
                r#"
type Note { id int; title str; body str; time str }

var notes = [
    Note { id: 0, title: "Welcome", body: "first", time: "now" },
    Note { id: 1, title: "Shopping", body: "milk", time: "ago" },
]
var nextid int = 2

#[api(method = "GET", path = "/api/notes")]
fn list_notes() []Note {
    return notes
}

#[api(method = "GET", path = "/api/notes/:id")]
fn get_note(id int) ?Note {
    for note in notes {
        if note.id == id {
            return Some(note)
        }
    }
    return None
}

#[api(method = "POST", path = "/api/notes")]
fn create_note(title str, body str) Note {
    let note = Note { id: nextid, title: title, body: body, time: "now" }
    nextid = nextid + 1
    return note
}
"#,
                18736,
            );

            // GET /api/notes → JSON array of Note objects
            let resp_list = http_get(port, "/api/notes");
            let body_list = body_of(&resp_list);
            assert!(
                body_list.contains("\"title\": \"Welcome\""),
                "list: body={:?}",
                body_list
            );
            assert!(
                body_list.contains("\"title\": \"Shopping\""),
                "list: body={:?}",
                body_list
            );
            // Should be a JSON array: starts with [
            assert!(
                body_list.trim_start().starts_with('['),
                "list not array: body={:?}",
                body_list
            );

            // GET /api/notes/1 → single Note (Option.Some unwrapped)
            let resp_get = http_get(port, "/api/notes/1");
            let body_get = body_of(&resp_get);
            assert!(
                body_get.contains("\"id\": 1"),
                "get id=1: body={:?}",
                body_get
            );
            assert!(
                body_get.contains("\"title\": \"Shopping\""),
                "get title: body={:?}",
                body_get
            );

            // PLAN-669 AC-01: POST body fields bind by param name — the
            // handler's `title`/`body` slots receive the JSON fields, not the
            // raw body string (this declared-but-untested surface was the
            // defect's hiding place; F7 in the plan).
            let resp_post = http_post_json_with_headers(
                port,
                "/api/notes",
                r#"{"title": "probe-item", "body": "probe-body"}"#,
                &[],
            );
            let body_post = body_of(&resp_post);
            assert!(
                body_post.contains("\"title\": \"probe-item\""),
                "post title by name: body={:?}",
                body_post
            );
            assert!(
                body_post.contains("\"body\": \"probe-body\""),
                "post body by name: body={:?}",
                body_post
            );
            assert!(
                !body_post.contains("{\\\"title\\\""),
                "raw body literal must not leak into fields: body={:?}",
                body_post
            );
        }

        /// PLAN-669 AC-01: POST JSON body fields bind by param name — the
        /// 013-todo create_todo probe shape from the defect report (raw body
        /// string used to land in `text` as a JSON literal).
        #[test]
        fn http_e2e_api_post_body_by_name() {
            let port = start_server(
                r#"
type Todo { id int; text str; done bool }
var nextid int = 4

#[api(method = "POST", path = "/api/todos")]
fn create_todo(text str) Todo {
    let todo = Todo { id: nextid, text: text, done: false }
    nextid = nextid + 1
    return todo
}
"#,
                18750,
            );

            let resp =
                http_post_json_with_headers(port, "/api/todos", r#"{"text":"probe-item"}"#, &[]);
            let body = body_of(&resp);
            assert!(
                body.contains("\"text\": \"probe-item\""),
                "text bound from body field: body={:?}",
                body
            );
            assert!(
                !body.contains("{\\\"text\\\""),
                "raw body literal leaked into field: body={:?}",
                body
            );
        }

        /// PLAN-669 AC-02: GET query params bind by name (the 015-notes
        /// search_notes shape; both `q` and `query` present — the declared
        /// name wins).
        #[test]
        fn http_e2e_api_query_by_name() {
            let port = start_server(
                r#"
type Hit { text str }

#[api(method = "GET", path = "/api/search")]
fn search(query str) Hit {
    return Hit { text: query }
}
"#,
                18751,
            );

            let resp = http_get(port, "/api/search?q=wrong&query=Build");
            let body = body_of(&resp);
            assert!(
                body.contains("\"text\": \"Build\""),
                "query bound by name: body={:?}",
                body
            );
        }

        /// PLAN-669 AC-03: declared `int` params convert exactly from the
        /// query string; non-numeric input is a named 400.
        #[test]
        fn http_e2e_api_typed_query_int() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/page")]
fn page(p int) int {
    return p
}
"#,
                18752,
            );

            let resp = http_get(port, "/api/page?p=7");
            let body = body_of(&resp);
            assert!(body.trim() == "7", "int query converted: body={:?}", body);

            let resp_bad = http_get(port, "/api/page?p=seven");
            assert!(
                resp_bad.starts_with("HTTP/1.1 400"),
                "non-numeric int → 400: resp={:?}",
                &resp_bad[..resp_bad.len().min(80)]
            );
            assert!(
                body_of(&resp_bad).contains("param `p`"),
                "400 names the param: body={:?}",
                body_of(&resp_bad)
            );
        }

        /// PLAN-669 AC-04: a missing body field is a 400 naming the param
        /// (back_proxy parity — silently wrong args are the defect itself).
        #[test]
        fn http_e2e_api_missing_param_400() {
            let port = start_server(
                r#"
type Note { title str; body str }

#[api(method = "POST", path = "/api/notes")]
fn create_note(title str, body str) Note {
    return Note { title: title, body: body }
}
"#,
                18753,
            );

            let resp =
                http_post_json_with_headers(port, "/api/notes", r#"{"title": "only-title"}"#, &[]);
            assert!(
                resp.starts_with("HTTP/1.1 400"),
                "missing body field → 400: resp={:?}",
                &resp[..resp.len().min(80)]
            );
            let body = body_of(&resp);
            assert!(
                body.contains("missing param `body`"),
                "400 names the missing param: body={:?}",
                body
            );
        }

        /// PLAN-669 AC-07: single str param + unparseable body → the raw
        /// body string passes through (back_proxy tolerance; the positional
        /// convention's one correct case, preserved).
        #[test]
        fn http_e2e_api_raw_body_single_param() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/save")]
fn save(data str) str {
    return data
}
"#,
                18754,
            );

            // Not JSON — parse fails, lone str param receives it verbatim.
            let resp = http_post_json_with_headers(port, "/api/save", "plain text probe", &[]);
            let body = body_of(&resp);
            assert!(
                body.contains("plain text probe"),
                "raw body passthrough: body={:?}",
                body
            );
        }

        /// Plan 317 final validation: 015-notes backend pattern with List<Note>
        /// generic + module-level var + #[api] handler returning the list.
        /// This mirrors db.at's `var notes List<Note>` + `fn all_notes() []Note`.
        #[test]
        fn e2e_notes_list_generic() {
            let port = start_server(
                r#"
type Note { id int; title str; body str; time str }

var notes = [
    Note { id: 0, title: "Welcome", body: "first", time: "now" },
    Note { id: 1, title: "Shopping", body: "milk", time: "ago" },
]

#[api(method = "GET", path = "/api/notes")]
fn list_notes() []Note {
    return notes
}
"#,
                18737,
            );

            let resp = http_get(port, "/api/notes");
            let body = body_of(&resp);
            // List<Note> serialized as JSON array of Note objects.
            assert!(
                body.contains("\"title\": \"Welcome\""),
                "list generic frame 1: body={:?}",
                body
            );
            assert!(
                body.contains("\"title\": \"Shopping\""),
                "list generic frame 2: body={:?}",
                body
            );
            assert!(
                body.trim_start().starts_with('['),
                "list generic should be JSON array: body={:?}",
                body
            );
        }

        // ============ PLAN-699: Axum/Hyper transport protocol probes ========
        // Raw-TCP probes against the real VM server (AC-01 framing/keep-alive,
        // AC-03 header bounds, AC-05 injected-signal shutdown). Ports
        // 18770-18775 (unique per e2e_ports_unique).

        /// Connect to the test server with the standard retry loop.
        fn connect_retry(port: u16) -> std::net::TcpStream {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = std::net::TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("connect to test HTTP server");
            stream.set_read_timeout(Some(Duration::from_secs(25))).ok();
            stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
            stream
        }

        /// AC-01: a chunked request body is decoded by the transport — the
        /// legacy hand-written parser rejected Transfer-Encoding outright.
        #[test]
        fn e2e_plan699_chunked_request_body_accepted() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/echo")]
fn echo(text str) str { text }
"#,
                18770,
            );
            let mut s = connect_retry(port);
            let req = "POST /api/echo HTTP/1.1\r\nHost: 127.0.0.1\r\n\
                       Content-Type: text/plain\r\nTransfer-Encoding: chunked\r\n\
                       Connection: close\r\n\r\n5\r\nhello\r\nE\r\n chunked world\r\n0\r\n\r\n";
            s.write_all(req.as_bytes()).unwrap();
            let mut resp = String::new();
            s.read_to_string(&mut resp).ok();
            assert!(
                resp.starts_with("HTTP/1.1 200"),
                "chunked body accepted, got: {:?}",
                resp
            );
            assert!(
                resp.contains("hello chunked world"),
                "decoded body echoed, got: {:?}",
                resp
            );
        }

        /// AC-01: two sequential requests on one connection (keep-alive);
        /// the legacy server closed after every response.
        #[test]
        fn e2e_plan699_keepalive_two_requests_one_connection() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/ping")]
fn ping() str { "pong" }
"#,
                18771,
            );
            let mut s = connect_retry(port);
            let req = "GET /api/ping HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
            s.write_all(req.as_bytes()).unwrap();
            s.write_all(req.as_bytes()).unwrap();
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                let n = match s.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                buf.extend_from_slice(&chunk[..n]);
                let text = String::from_utf8_lossy(&buf);
                let heads = text.matches("HTTP/1.1").count();
                if heads >= 2 && text.matches("pong").count() >= 2 {
                    break;
                }
                if buf.len() > 64 * 1024 {
                    break;
                }
            }
            let resp = String::from_utf8_lossy(&buf).into_owned();
            assert_eq!(
                resp.matches("HTTP/1.1 200").count(),
                2,
                "two keep-alive responses on one connection, got: {:?}",
                resp
            );
        }

        /// AC-03: oversized request head (beyond the 64 KiB connection
        /// buffer) is rejected with hyper's native 431.
        #[test]
        fn e2e_plan699_oversized_headers_431() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/ping")]
fn ping() str { "pong" }
"#,
                18772,
            );
            let mut s = connect_retry(port);
            let big = format!("X-Big: {}\r\n", "a".repeat(70 * 1024));
            let req = format!("GET /api/ping HTTP/1.1\r\nHost: 127.0.0.1\r\n{}\r\n", big);
            let _ = s.write_all(req.as_bytes());
            let mut resp = String::new();
            let _ = s.read_to_string(&mut resp);
            assert!(
                resp.starts_with("HTTP/1.1 431"),
                "oversized head must be rejected with 431, got: {:?}",
                resp.lines().next()
            );
        }

        /// AC-03: a request head that never completes is closed by the 10s
        /// header read timeout (deterministic close, no unbounded hold).
        #[test]
        fn e2e_plan699_slow_headers_timeout_close() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/ping")]
fn ping() str { "pong" }
"#,
                18773,
            );
            let mut s = connect_retry(port);
            let _ = s.write_all(b"GET /api/ping HTTP/1.1\r\nHost: 127.0.0.1\r\n");
            let _ = s.flush();
            let started = std::time::Instant::now();
            let mut sink = Vec::new();
            let _ = s.read_to_end(&mut sink);
            let elapsed = started.elapsed();
            assert!(
                elapsed >= Duration::from_secs(8) && elapsed <= Duration::from_secs(20),
                "head timeout should close at ~10s, elapsed={:?}",
                elapsed
            );
        }

        /// AC-05: the injected shutdown flag stops the accept loop, the owner
        /// loop drains and exits, and the port is rebindable. A bare VM (no
        /// routes) is enough — every request 404s through the real dispatch.
        #[test]
        fn e2e_plan699_injected_shutdown_releases_port() {
            let port: u16 = 18775;
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let cfg = crate::vm::ffi::http_transport::TransportConfig::from_env();
            let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
            // Rc<AutoVM> is !Send — build the VM inside the owner thread
            // (same pattern as start_server).
            let _server = std::thread::Builder::new()
                .stack_size(8 * 1024 * 1024)
                .spawn(move || {
                    let vm = std::rc::Rc::new(crate::vm::engine::AutoVM::new(
                        crate::vm::virt_memory::VirtualFlash::new_with_code(vec![
                            crate::vm::opcode::OpCode::RET as u8,
                        ]),
                        1024,
                    ));
                    crate::block_on_autovm_local(async move {
                        crate::vm::ffi::http_server::serve_with(
                            vm,
                            &format!("0.0.0.0:{}", port),
                            cfg,
                            shutdown_rx,
                        )
                        .await;
                    });
                    let _ = done_tx.send(());
                })
                .expect("spawn shutdown-test server thread");

            // Wait for accept, then exercise the real dispatch (404 path).
            let mut s = connect_retry(port);
            s.write_all(
                "GET /api/nope HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n".as_bytes(),
            )
            .unwrap();
            let mut resp = String::new();
            let _ = s.read_to_string(&mut resp);
            assert!(
                resp.starts_with("HTTP/1.1 404"),
                "bare VM serves 404 through the bridge, got: {:?}",
                resp
            );

            // Flip the shutdown flag: the server must exit promptly.
            let _ = shutdown_tx.send(true);
            done_rx
                .recv_timeout(Duration::from_secs(15))
                .expect("server exits after injected shutdown");
            // Port is released: a fresh bind on the same port must succeed.
            let rebind = std::net::TcpListener::bind(("127.0.0.1", port));
            assert!(rebind.is_ok(), "port must be rebindable after shutdown");
        }
    }
}

// ============================================================================
// PLAN-699 T-02: owned request/reply bridge + VM-owner dispatch core
// ============================================================================

/// CORS response headers as typed pairs (the string block in `cors_headers`
/// stays for the sync stdnet server's string writer).
pub(crate) fn cors_header_pairs() -> Vec<(String, String)> {
    // PLAN-736 T-04: 服务策略激活时 legacy `*` 头块停用——附件由桥层按
    // 配置决策（精确 origin/Vary），避免双份/泄漏。
    if crate::http_service_config::service_policy_active() {
        return Vec::new();
    }
    let origin = cors_origin();
    vec![
        ("Access-Control-Allow-Origin".to_string(), origin),
        (
            "Access-Control-Allow-Methods".to_string(),
            "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string(),
        ),
        (
            "Access-Control-Allow-Headers".to_string(),
            "Content-Type, Authorization".to_string(),
        ),
        ("Access-Control-Max-Age".to_string(), "86400".to_string()),
    ]
}

/// Sec-WebSocket-Accept = base64(sha1(key + magic GUID)) — same handshake the
/// legacy inline path computed; pure protocol math, no VM involvement.
pub(crate) fn compute_ws_accept(key: &str) -> String {
    use base64::Engine;
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let hash = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(&hash)
}

/// Plan 317 Phase 4: Concurrent HTTP server using tokio async I/O.
///
/// Replaces the serial `serve_blocking_stdnet` for the Goroutine-style
/// concurrency model. The VM owner enters this on a Tokio `LocalSet`, and all
/// connection tasks remain on that same thread because `AutoVM` is `!Send`.
///
/// Each accepted connection becomes a `spawn_local` task:
///   - JSON handlers: call_fn_by_name (synchronous), write response, done.
///   - SSE handlers: a bounded local producer steps the VM in instruction
///     batches while the connection task writes frames.
/// PLAN-736 T-05：进程内服务关停发送端（`POST /__auto/shutdown` 触发同一
/// 关闭入口；headless 测试的确定性注入路径——真实 Ctrl+C 证据范围见 SD-01）。
static SERVICE_SHUTDOWN_TX: std::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>> =
    std::sync::Mutex::new(None);

/// PLAN-736 T-05/AC-02/05：服务控制面（`/__auto/*`，装配期与业务同名路由
/// 冲突=启动诊断）。live/ready 开放（身份最小体）；snapshot/shutdown 仅
/// loopback peer（不对代理/外部暴露操作面）。
pub(crate) fn handle_service_control_path(
    method: &str,
    path: &str,
    peer: std::net::SocketAddr,
    service: Option<&crate::http_service_config::ServiceRuntime>,
) -> Option<axum::response::Response> {
    use crate::http_service_observability as obs;
    let observation = service
        .map(|runtime| runtime.observation.clone())
        .unwrap_or_else(obs::process_observation);
    if !path.starts_with("/__auto/") {
        return None;
    }
    let json_resp = |status: u16, body: serde_json::Value| -> axum::response::Response {
        axum::response::Response::builder()
            .status(
                axum::http::StatusCode::from_u16(status)
                    .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
            )
            .header("content-type", "application/json")
            .body(axum::body::Body::from(body.to_string()))
            .unwrap()
    };
    match (method, path) {
        ("GET", "/__auto/health/live") => Some(json_resp(
            200,
            serde_json::json!({ "state": observation.service_state().as_str() }),
        )),
        ("GET", "/__auto/health/ready") => {
            if observation.is_ready() {
                Some(json_resp(
                    200,
                    observation
                        .service_identity()
                        .unwrap_or_else(|| serde_json::json!({})),
                ))
            } else {
                Some(json_resp(
                    503,
                    serde_json::json!({ "state": observation.service_state().as_str() }),
                ))
            }
        }
        ("GET", "/__auto/health/snapshot") => {
            if !peer_is_loopback(peer) {
                return Some(json_resp(
                    403,
                    serde_json::json!({ "error": "loopback only" }),
                ));
            }
            Some(json_resp(
                200,
                serde_json::json!({
                    "state": observation.service_state().as_str(),
                    "identity": observation.service_identity(),
                    "counters": observation.counters().snapshot(),
                }),
            ))
        }
        ("POST", "/__auto/shutdown") => {
            if !peer_is_loopback(peer) {
                return Some(json_resp(
                    403,
                    serde_json::json!({ "error": "loopback only" }),
                ));
            }
            let sent = service
                .map(|runtime| runtime.request_shutdown())
                .unwrap_or_else(|| {
                    SERVICE_SHUTDOWN_TX
                        .lock()
                        .ok()
                        .and_then(|slot| slot.as_ref().map(|tx| tx.send(true).is_ok()))
                        .unwrap_or(false)
                });
            observation.emit_event(serde_json::json!({
                "event": "drain", "trigger": "shutdown_endpoint", "delivered": sent,
            }));
            Some(json_resp(202, serde_json::json!({ "draining": sent })))
        }
        _ => Some(json_resp(404, serde_json::json!({ "error": "not found" }))),
    }
}

/// PLAN-736 AC-01/02: 最近一次 serve 的致命错误（bind 失败/net 线程死亡）。
/// serve 入口（`auto serve`/run_file_with_service 消费者）据此以非零退出；
/// legacy `auto run` 面不消费——保持既有"打印后继续"语义不变。
static SERVE_FATAL: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
/// PLAN-736 AC-02: 最近一次 serve 成功 bind 的真实地址（port=0 解析值）。
static SERVE_BOUND_ADDR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub async fn serve_async(vm: std::rc::Rc<crate::vm::engine::AutoVM>, addr: &str) {
    // PLAN-699 T-03: Axum/Hyper HTTP/1.1 transport (Design 33 阶段 B). The VM
    // owner keeps this thread's `LocalSet` and consumes a bounded queue of
    // owned `ApiRequest`s; the network layer runs on a dedicated net thread
    // (super::http_transport). Default shutdown signals: Ctrl+C (+ SIGTERM on
    // unix); tests/embedders inject theirs through `serve_with`.
    let cfg = super::http_transport::TransportConfig::from_env();
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    // PLAN-705 T-05（仅测试面）：注册全局关停触发器——E2E 经
    // test_trigger_shutdown 注入优雅关停（默认信号面 Ctrl+C 不变）。
    // PLAN-814: 跨 crate 测试支持面（上层 plan705 e2e 关停探针）
    {
        let _ = TEST_SHUTDOWN_TX.set(shutdown_tx.clone());
    }
    let _ = &shutdown_tx;
    // PLAN-736 T-05：/__auto/shutdown 控制面与信号汇入同一 watch。
    register_service_shutdown_tx(shutdown_tx.clone());
    tokio::task::spawn_local({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            eprintln!("[HTTP] Ctrl+C — shutting down gracefully");
            let _ = shutdown_tx.send(true);
        }
    });
    #[cfg(unix)]
    tokio::task::spawn_local({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            if let Ok(mut sig) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            {
                sig.recv().await;
                eprintln!("[HTTP] SIGTERM — shutting down gracefully");
                let _ = shutdown_tx.send(true);
            }
        }
    });
    serve_with(vm, addr, cfg, shutdown_rx).await;
}

/// PLAN-736 AC-03: 显式预算注入面——`auto service` 的 VM 轨用服务配置驱动
/// TransportConfig（env 不参与）；信号面与 `serve_async` 相同。
pub async fn serve_async_with(
    vm: std::rc::Rc<crate::vm::engine::AutoVM>,
    addr: &str,
    cfg: super::http_transport::TransportConfig,
) {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    if let Some(service) = &cfg.service {
        if !service.bind_shutdown(shutdown_tx.clone()) {
            record_serve_fatal("service runtime shutdown already bound");
            return;
        }
    }
    // PLAN-814: 跨 crate 测试支持面（上层 plan705 e2e 关停探针）
    {
        let _ = TEST_SHUTDOWN_TX.set(shutdown_tx.clone());
    }
    let _ = &shutdown_tx;
    register_service_shutdown_tx(shutdown_tx.clone());
    tokio::task::spawn_local({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            eprintln!("[HTTP] Ctrl+C — shutting down gracefully");
            let _ = shutdown_tx.send(true);
        }
    });
    #[cfg(unix)]
    tokio::task::spawn_local({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            if let Ok(mut sig) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            {
                sig.recv().await;
                eprintln!("[HTTP] SIGTERM — shutting down gracefully");
                let _ = shutdown_tx.send(true);
            }
        }
    });
    serve_with(vm, addr, cfg, shutdown_rx).await;
}

/// PLAN-705 T-05（仅测试面）：构造最小 GET ApiRequest（失效队列单元用）。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
pub fn test_api_request_get(path: &str) -> ApiRequest {
    ApiRequest {
        method: "GET".to_string(),
        path: path.to_string(),
        headers: Vec::new(),
        body: Vec::new(),
        raw_upload_body: None,
        peer: None,
    }
}

/// PLAN-705 T-05（仅测试面）：serve_async 注册的最新实例关停发送端。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
static TEST_SHUTDOWN_TX: std::sync::OnceLock<tokio::sync::watch::Sender<bool>> =
    std::sync::OnceLock::new();

/// PLAN-705 T-05（仅测试面）：触发 serve_async 已注册实例的优雅关停。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
pub fn test_trigger_shutdown() -> bool {
    TEST_SHUTDOWN_TX
        .get()
        .map(|tx| tx.send(true).is_ok())
        .unwrap_or(false)
}

/// Injectable-shutdown entry (tests / embedders). Owns the VM for the server
/// lifetime; returns after the shutdown flag fired and draining finished
/// (port released).
pub(crate) async fn serve_with(
    vm: std::rc::Rc<crate::vm::engine::AutoVM>,
    addr: &str,
    cfg: super::http_transport::TransportConfig,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    crate::http_service_config::with_service_policy(
        cfg.service.is_some(),
        server_route::with_owner_rate_buckets(serve_with_inner(vm, addr, cfg, shutdown_rx)),
    )
    .await;
}

async fn serve_with_inner(
    vm: std::rc::Rc<crate::vm::engine::AutoVM>,
    addr: &str,
    cfg: super::http_transport::TransportConfig,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    // PLAN-730 T-05：安装上传宿主 executor（幂等；a2r facade 的 receive/
    // commit/reject 经此执行——VM 桥与生成 Rust 服务共用单源）。
    crate::http_upload_service::install_upload_executor_service();
    let routes = get_routes();
    let (req_tx, mut req_rx) = tokio::sync::mpsc::channel::<(
        ApiRequest,
        tokio::sync::oneshot::Sender<ApiReply>,
        u64, // scope id (PLAN-705 T-05)
    )>(cfg.queue_capacity);

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<Result<String, String>>();
    let net_addr = addr.to_string();
    let net_cfg = cfg.clone();
    let net_shutdown = shutdown_rx.clone();
    let net_metadata = current_program_metadata();
    let spawned = std::thread::Builder::new()
        .name("auto-http-net".to_string())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = ready_tx.send(Err(format!("net runtime: {e}")));
                    return;
                }
            };
            rt.block_on(super::http_transport::serve_network(
                net_addr,
                net_cfg,
                req_tx,
                net_shutdown,
                ready_tx,
                net_metadata,
            ));
        });
    let network_thread = match spawned {
        Ok(thread) => thread,
        Err(error) => {
            eprintln!("[HTTP] failed to spawn the auto-http-net thread: {error}");
            record_serve_fatal(format!("failed to spawn the auto-http-net thread: {error}"));
            return;
        }
    };
    match ready_rx.await {
        Ok(Ok(bound)) => {
            // PLAN-736: 真实 bound 地址（port=0 时为内核分配值）供 serve 入口/
            // health 身份消费。
            if let Ok(mut slot) = SERVE_BOUND_ADDR.lock() {
                *slot = Some(bound.clone());
            }
            // PLAN-736 T-05/AC-02：bind+初始化完成 → Ready（身份四元组）。
            // 业务路由/VM 初始化在此之前已发生（run_file 编译先于 serve hook）。
            if let Some(svc) = &cfg.service {
                let c = &svc.config;
                svc.observation.set_service_identity(
                    &bound,
                    c.profile.as_str(),
                    c.effective_config_hash(),
                );
                svc.observation.set_service_state(
                    if *shutdown_rx.borrow() {
                        crate::http_service_observability::ServiceState::Draining
                    } else {
                        crate::http_service_observability::ServiceState::Ready
                    },
                );
            }
        }
        Ok(Err(e)) => {
            eprintln!("[HTTP] Async server bind failed on {}: {}", addr, e);
            record_serve_fatal(format!("bind failed on {addr}: {e}"));
            join_network_thread(network_thread).await;
            return;
        }
        Err(_) => {
            eprintln!("[HTTP] net thread died before binding");
            record_serve_fatal("net thread died before binding");
            join_network_thread(network_thread).await;
            return;
        }
    }
    eprintln!(
        "[HTTP] Axum/Hyper transport listening on {} ({} route(s), VM owner thread)",
        addr,
        routes.len()
    );

    // Owner loop (PLAN-705 T-04): 事件驱动的段调度——请求到达即段驱动
    // （首 park 前同步完成），park 后挂入 parked 表；完成通知
    // （COMPLETION_NOTIFY，T-02 完成端写点发出）唤醒后按 park 顺序恢复
    // 就绪条目。零固定间隔轮询；enable→复查→await 三段式消除丢唤醒窗口
    // （spike 已证）。等待上游的请求不再占住 owner——期间新请求/健康
    // 检查照常服务（AC-01）。CPU 段仍不可抢占（10M 指令预算内跑完），
    // 这是 Design 33 §C 的既有边界。
    let mut shutdown_rx = shutdown_rx;
    let mut parked: Vec<ParkedRequest> = Vec::new();
    let mut drain_deadline = (*shutdown_rx.borrow())
        .then(|| tokio::time::Instant::now() + cfg.shutdown_drain);
    let mut loop_result = ();
    loop {
        if drain_deadline.is_some_and(|deadline| tokio::time::Instant::now() >= deadline) {
            cancel_all_parked(&vm, &mut parked);
            break loop_result;
        }
        // (0) 事件驱动扫描：scope 失效废弃 + 恢复就绪 parked（本轮通知/
        // 事件唤醒后的消费点——取消信号同经 COMPLETION_NOTIFY）。
        drain_ready_parked(&vm, &mut parked);
        // 最早 deadline 定时臂（parked 非空时唤醒失效检查，防无事件悬挂）。
        let next_deadline = parked
            .iter()
            .filter_map(|p| lookup_scope(p.scope_id))
            .map(|s| tokio::time::Instant::from_std(*s.deadline.read().unwrap()))
            .min();
        // (1) 注册完成通知兴趣（Notified 首次 poll 才登记 waiter，enable()
        // 把登记提前到检查之前）。
        let notify_fut = VmHttpBridge::completion_notified();
        tokio::pin!(notify_fut);
        notify_fut.as_mut().enable();
        // (2) enable 后复查（覆盖"通知在登记前发出"的窗口）。
        if parked.iter().any(|p| parked_is_ready(&vm, p)) {
            continue;
        }
        // (3) 事件等待：新请求 / 完成通知 / 关闭。
        if let Some(drain_deadline) = drain_deadline {
            // 关闭排水窗：继续应答已排队请求；parked 请求等待其完成或
            // 排水截止（超时统一取消——T-05 的 scope 取消语义；窗宽随
            // cfg，AUTO_HTTP_SHUTDOWN_DRAIN_MS 可调）。
            tokio::select! {
                req = req_rx.recv() => match req {
                    Some((api_req, reply_tx, scope_id)) => {
                        dispatch_owner_request(&vm, &routes, api_req, reply_tx, scope_id, &mut parked);
                    }
                    None => {
                        cancel_all_parked(&vm, &mut parked);
                        break loop_result;
                    }
                },
                _ = &mut notify_fut => {}
                _ = tokio::time::sleep_until(drain_deadline) => {
                    cancel_all_parked(&vm, &mut parked);
                    break loop_result;
                }
            }
        } else {
            tokio::select! {
                req = req_rx.recv() => match req {
                    Some((api_req, reply_tx, scope_id)) => {
                        dispatch_owner_request(&vm, &routes, api_req, reply_tx, scope_id, &mut parked);
                    }
                    None => {
                        // net 侧发送端已清（端口释放）——放弃 parked
                        //（连接已不在，回复无处投递）。
                        cancel_all_parked(&vm, &mut parked);
                        break loop_result;
                    }
                },
                _ = &mut notify_fut => {}
                _ = async {
                    match next_deadline {
                        Some(d) => tokio::time::sleep_until(d).await,
                        None => std::future::pending::<()>().await,
                    }
                } => {
                    // 最早 parked deadline 到期：回到循环顶做失效废弃。
                }
                _ = wait_for_service_shutdown(&mut shutdown_rx) => {
                    // 排水窗开启：net 侧同步停收（watch 广播同源），已排队
                    // 请求继续应答；parked 请求等待完成或排水截止。
                    drain_deadline = Some(tokio::time::Instant::now() + cfg.shutdown_drain);
                    if let Some(service) = &cfg.service {
                        service.observation.set_service_state(
                            crate::http_service_observability::ServiceState::Draining,
                        );
                    }
                }
            }
        }
    }
    drop(req_rx);
    join_network_thread(network_thread).await;
    if let Some(service) = &cfg.service {
        service.observation.set_service_state(
            crate::http_service_observability::ServiceState::Stopped,
        );
    }
    eprintln!("[HTTP] VM owner loop exited (port released)");
}

/// ============================================================================
/// PLAN-705 T-04/SD-01: 段驱动的 owner dispatch——可 park 的请求状态机
/// ============================================================================
///
/// 请求生命周期从"一次同步 fn 调用"升级为可跨 park 的阶段机：
/// 预处理（CORS/限流/路由/WS，纯同步）→ middleware 链（逐个段驱动）→
/// handler 段驱动（named / `__axum:` closure）→ 编组（SSE / Response
/// handle / ~T 最终值 / JSON）。任一段 Yield（异步 HTTP / External
/// Future 等待）即返回 [`DispatchOutcome::Parked`]，owner loop 把它挂入
/// parked 表，由 [`COMPLETION_NOTIFY`] 唤醒后经 [`resume_parked_request`]
/// 续跑。等待期间 owner 可自由服务其他请求（AC-01：等待上游时健康请求
/// 仍可完成）。
///
/// 所有用到的 VM 状态访问都发生在 owner 线程（AutoVM !Send 契约不变）；
/// 跨线程仍只有 owned `ApiRequest`/`ApiReply`。单执行段顺序执行、跨
/// await 非原子事务（SD-01 契约：副作用不因取消/交错回滚）。
use crate::vm::engine::{AutoVM, ParkedSegment, ParkedWait, SegmentOutcome};

/// 段入口的产出：回复已发出（reply_tx 已消费），或请求 parked。
pub(crate) enum DispatchOutcome {
    Replied,
    Parked(Box<ParkedRequest>),
}

/// owner dispatch 段形态入口：预处理 + middleware + handler，跑到回复或
/// 首 park。回复经 `reply_tx` 发出（Replied 时 tx 已消费）。
pub(crate) fn dispatch_api_request_segment(
    vm: &std::rc::Rc<AutoVM>,
    routes: &[HttpRoute],
    mut req: ApiRequest,
    reply_tx: tokio::sync::oneshot::Sender<ApiReply>,
    scope_id: u64,
) -> DispatchOutcome {
    let req_method = req.method.to_uppercase();
    let req_path = req.path.clone();

    if let Some(_preflight) = handle_cors_preflight(&req_method) {
        let _ = reply_tx.send(ApiReply::Full {
            status: 204,
            headers: cors_header_pairs(),
            body: ApiBody::Text(Vec::new()),
        });
        return DispatchOutcome::Replied;
    }

    let content_type_raw = req.header("content-type").unwrap_or("").to_string();
    let content_type = content_type_raw.to_ascii_lowercase();
    let cookie_header = req.header("cookie").unwrap_or("").to_string();
    let auth_header = req.header("authorization").unwrap_or("").to_string();
    let incoming_request_id = req.header("x-request-id").unwrap_or("").to_string();
    let is_websocket = req
        .header("upgrade")
        .map(|v| v.to_ascii_lowercase().contains("websocket"))
        .unwrap_or(false);

    // PLAN-736 T-04/AC-05: 入站 request-id 校验——1..64 可见 ASCII 安全子集
    // （[0-9A-Za-z._:@-]）；不合者重生成（原样透传 = 日志注入面）。
    fn request_id_ok(id: &str) -> bool {
        !id.is_empty()
            && id.len() <= 64
            && id
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '@' | '-'))
    }
    let request_id = if request_id_ok(&incoming_request_id) {
        incoming_request_id
    } else {
        gen_request_id()
    };

    let client_ip = req
        .peer
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    if let Some(retry_after_ms) = rate_limit_take(vm, &client_ip) {
        let body = format!(
            "{{\"error\":\"rate limit exceeded\",\"retry_after_ms\":{}}}",
            retry_after_ms
        );
        let mut headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "Retry-After".to_string(),
                ((retry_after_ms + 999) / 1000).to_string(),
            ),
        ];
        headers.extend(cors_header_pairs());
        headers.push(("X-Request-Id".to_string(), request_id.clone()));
        eprintln!(
            "[HTTP] {} {} [{}] → 429 rate limited (ip {})",
            req_method, req_path, request_id, client_ip
        );
        let _ = reply_tx.send(ApiReply::Full {
            status: 429,
            headers,
            body: ApiBody::Text(body.into_bytes()),
        });
        return DispatchOutcome::Replied;
    }

    let body = String::from_utf8_lossy(&req.body).into_owned();

    // PLAN-730 T-05/T-07：路由先行（404/方法不匹配零解析零写盘）；上传
    // 路由按类型分类跳过 legacy multipart（body 为延迟能力，不在此解析）。
    let route_match = match match_route(routes, &req_method, &req_path) {
        Some(rm) => rm,
        None => {
            let mut headers = cors_header_pairs();
            headers.push(("X-Request-Id".to_string(), request_id.clone()));
            let _ = reply_tx.send(ApiReply::Full {
                status: 404,
                headers,
                body: ApiBody::Text(Vec::new()),
            });
            return DispatchOutcome::Replied;
        }
    };
    let is_upload_route = route_declares_upload(&route_match.fn_name);
    let upload_headers = req.headers.clone();
    let upload_pending = if is_upload_route {
        if !matches!(req_method.as_str(), "POST" | "PUT") {
            let mut headers = cors_header_pairs();
            headers.push(("X-Request-Id".to_string(), request_id.clone()));
            headers.push(("Allow".to_string(), "POST, PUT".to_string()));
            let _ = reply_tx.send(ApiReply::Full {
                status: 405,
                headers,
                body: ApiBody::Text(
                    br#"{"error":"upload endpoints support POST/PUT only"}"#.to_vec(),
                ),
            });
            return DispatchOutcome::Replied;
        }
        match req.raw_upload_body.take() {
            Some(b) => Some(PendingUpload {
                body: b,
                headers: upload_headers,
            }),
            // bridge 与 owner 的分类不一致（非 transport 入口的派发）——契约
            // 破坏，fail-closed 500，不执行业务函数。
            None => {
                eprintln!(
                    "[HTTP] {} {} [{}] → 500 (upload route without body capability)",
                    req_method, req_path, request_id
                );
                let mut headers = cors_json_headers(&request_id);
                headers[0].1 = "application/json".to_string();
                let _ = reply_tx.send(ApiReply::Full {
                    status: 500,
                    headers,
                    body: ApiBody::Text(
                        br#"{"error":"upload route missing deferred body"}"#.to_vec(),
                    ),
                });
                return DispatchOutcome::Replied;
            }
        }
    } else {
        None
    };

    // PLAN-730 T-07：解析（纯内存，路由命中后）与落盘（middleware 后宿主
    // 执行）分离——404/method 不匹配/中间件拒绝零解析零写盘。
    let mut legacy_parts: Option<Vec<MultipartPart>> = None;
    let mut legacy_fields_json = String::new();
    let mut multipart_json: Option<String> = None;
    if !is_upload_route && content_type.starts_with("multipart/form-data") {
        let boundary = content_type_raw
            .split(';')
            .find_map(|p| p.trim().strip_prefix("boundary="))
            .map(|b| b.trim_matches('"').to_string());
        if let Some(boundary) = boundary {
            let parts = parse_multipart(&req.body, &boundary);
            legacy_fields_json = multipart_fields_json(&parts);
            let has_files = parts.iter().any(|p| p.filename.is_some());
            if has_files {
                legacy_parts = Some(parts);
            } else {
                multipart_json = Some(multipart_handler_json(&legacy_fields_json, ""));
            }
            eprintln!(
                "[HTTP] {} {} [{}] multipart: {} bytes parsed (store deferred)",
                req_method,
                req_path,
                request_id,
                req.body.len()
            );
        } else {
            eprintln!(
                "[HTTP] {} {} [{}] multipart without boundary — ignored",
                req_method, req_path, request_id
            );
        }
    }

    // PLAN-729 T-04：文件端点仅 GET/HEAD（决策报告 §4）——非 GET/HEAD 的
    // 文件返回注解在路由命中处即 405（不执行 handler）。
    if fn_is_api_file_return(&route_match.fn_name) && !matches!(req_method.as_str(), "GET" | "HEAD")
    {
        let mut headers = cors_header_pairs();
        headers.push(("X-Request-Id".to_string(), request_id.clone()));
        headers.push(("Allow".to_string(), "GET, HEAD".to_string()));
        let _ = reply_tx.send(ApiReply::Full {
            status: 405,
            headers,
            body: ApiBody::Text(br#"{"error":"file endpoints support GET/HEAD only"}"#.to_vec()),
        });
        return DispatchOutcome::Replied;
    }

    if is_websocket {
        if let Some(key) = req.header("sec-websocket-key") {
            let _ = reply_tx.send(ApiReply::WebSocket {
                accept: compute_ws_accept(key),
            });
            return DispatchOutcome::Replied;
        }
    }

    let request_info = format!(
        r#"{{"method":"{}","path":"{}","content_type":"{}","has_body":{},"request_id":"{}"}}"#,
        req_method,
        req_path,
        content_type,
        !body.is_empty() || upload_pending.is_some(),
        request_id
    );
    let middleware_names: Vec<String> = crate::vm::ffi::stdlib::MIDDLEWARE_CHAIN
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();

    let ctx = DispatchCtx {
        req_method,
        req_path,
        request_id,
        body,
        content_type,
        content_type_raw,
        cookie_header,
        auth_header,
        multipart_json,
        route: route_match,
        axum_route: None,
        started: std::time::Instant::now(),
        middleware_names,
        request_info,
        handler_task_id: None,
        scope_id,
        upload_pending,
        legacy_parts,
        legacy_fields_json,
        legacy_stored: Vec::new(),
    };
    advance_dispatch(vm, ctx, 0, Some(reply_tx))
}

/// PLAN-730 T-07：legacy 落盘结果表（op_id → Option<Result>；None=Pending）。
lazy_static::lazy_static! {
    static ref LEGACY_STORE_RESULTS: std::sync::Mutex<
        std::collections::HashMap<u64, Option<Result<(Vec<String>, String), String>>>,
    > = std::sync::Mutex::new(std::collections::HashMap::new());
}

// ============================================================================
// PLAN-705 T-05: 请求作用域与生命期许可（queued+running+parked 总上限）
// ============================================================================
//
// 许可语义升级（决策报告 §3，兼容性明示）：AUTO_HTTP_MAX_INFLIGHT 从
// "队列容量"升级为"请求生命期总上限"——许可在桥入队前获取，scope 终结
// （回复送达 / SSE 流结束 / 取消 / 关闭）幂等释放。mpsc 队列容量仍为同值
// （队满 503 先于许可上限发生）。取消/超时的请求不再无效排队与复活结果
// 槽：cancel 是幂等终结入口，parked 等待废弃时同步回收 live-op。

/// 作用域状态机（AtomicU8 载荷）。
pub(crate) const SCOPE_QUEUED: u8 = 0;
pub(crate) const SCOPE_RUNNING: u8 = 1;
pub(crate) const SCOPE_PARKED: u8 = 2;
pub(crate) const SCOPE_DONE: u8 = 3;
pub(crate) const SCOPE_CANCELLED: u8 = 4;

lazy_static::lazy_static! {
    static ref REQUEST_SCOPES: std::sync::Mutex<std::collections::HashMap<u64, std::sync::Arc<RequestScope>>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
}

static SCOPE_ID_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

static LIFE_PERMITS: std::sync::OnceLock<std::sync::Arc<tokio::sync::Semaphore>> =
    std::sync::OnceLock::new();

static TEST_PERMIT_CAPACITY: std::sync::Mutex<Option<usize>> = std::sync::Mutex::new(None);

/// 仅供测试：覆写生命期许可容量（首次 life_permits() 前调用生效；
/// nextest 每测试进程隔离）。
#[cfg(test)]
pub(crate) fn set_life_permit_capacity_for_test(cap: usize) {
    *TEST_PERMIT_CAPACITY.lock().unwrap() = Some(cap);
}

/// 正常完成（回复送达非 SSE / SSE 流结束）。
pub(crate) fn complete_scope(scope: &RequestScope) {
    finalize_scope(scope, SCOPE_DONE);
}

/// 按 id 查 scope（owner 侧）。
pub(crate) fn lookup_scope(id: u64) -> Option<std::sync::Arc<RequestScope>> {
    REQUEST_SCOPES.lock().ok().and_then(|m| m.get(&id).cloned())
}

/// PLAN-730 T-05：延展 scope deadline（只增不减；上传 receive 启动时从
/// 30s 切换到上传总期限）。watch 唤醒桥 reply 等待循环重臂；完成通知唤醒
/// owner parked 定时臂重算。
pub(crate) fn extend_scope_deadline(scope_id: u64, new_deadline: std::time::Instant) {
    if let Some(scope) = lookup_scope(scope_id) {
        let extended = {
            let mut d = scope.deadline.write().unwrap();
            if new_deadline > *d {
                *d = new_deadline;
                true
            } else {
                false
            }
        };
        if extended {
            let _ = scope.deadline_tx.send(new_deadline);
            VmHttpBridge::notify_completion();
        }
    }
}

/// 连接终结（net 侧 watcher）：取消该连接名下全部 scope（已确证销毁判据，
/// 决策报告 §5-3）。
pub(crate) fn cancel_scopes_for_conn(conn_id: u64) {
    let scopes: Vec<std::sync::Arc<RequestScope>> = match REQUEST_SCOPES.lock() {
        Ok(map) => map
            .values()
            .filter(|s| s.conn_id == conn_id)
            .cloned()
            .collect(),
        Err(_) => return,
    };
    for s in scopes {
        cancel_scope(&s);
    }
}

// PLAN-707 T-06（D-7）：handler 段执行的当前 scope——流 open shim 经
// [`register_scope_stream`] 把新建流挂进请求资源组；非 request 上下文
// （UI/CLI 程序）无 scope，资源归显式 close 管理（决策 §5.4）。
/// Bind the host-owned context for dispatch/resume; Drop restores its parent.
pub(crate) fn bind_current_scope(scope_id: u64) -> ScopeGuard {
    let context = lookup_scope(scope_id).map(|scope| {
        scope as std::sync::Arc<dyn crate::vm::host_capabilities::request_context::RequestContext>
    });
    crate::vm::host_capabilities::request_context::bind_request_context(context)
}

/// PLAN-781 T-22（R04 修复轮；R06 修正为按服务代际归属）：优雅关停
/// 排水截止注册表——**key = serve_network 服务代际**（进程内可并存多服
/// 务实例；旧服务的截止只影响自身 scope，不污染同进程后续/并发实例
/// ——R06 复审反例：同进程关停后重启，全局单值残留导致新实例长请求
/// 首次探测即被误取消）。net 线程在 shutdown watch 翻转时设置
/// now+shutdown_drain，net 循环退出时清除（服务生命周期绑定）；owner
/// 可能正被 RUNNING CPU 段阻塞（到达不了 cancel_all_parked），engine
/// 驱动环的协作取消探测按 scope.server_gen 查询本表。
static SERVER_DRAIN_CUTOFFS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<u64, std::time::Instant>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// 服务代际发号（serve_network 启动时递增；0 保留给进程前历史调用方）。
static SERVER_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(crate) fn next_server_generation() -> u64 {
    SERVER_GEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// net 线程 shutdown 翻转时调用（见 http_transport serve_network）。
pub(crate) fn set_server_drain_cutoff(server_gen: u64, cutoff: std::time::Instant) {
    if let Ok(mut m) = SERVER_DRAIN_CUTOFFS.lock() {
        m.insert(server_gen, cutoff);
    }
}

/// net 循环退出时调用——本服务生命周期终结，截止随之移除（不影响
/// 其他在途服务实例的排水）。
pub(crate) fn clear_server_drain_cutoff(server_gen: u64) {
    if let Ok(mut m) = SERVER_DRAIN_CUTOFFS.lock() {
        m.remove(&server_gen);
    }
}

/// PLAN-781 T-22（R04 修复轮；R06 归属修正）：owner 驱动环的协作式取
/// 消探测——当前段绑定 HTTP scope 且（a）scope 已终结（取消/deadline/
/// 关闭，含已出表）或（b）**本服务**（scope.server_gen）的排水截止已
/// 过时 true；无绑定（CLI/UI/legacy 程序段）恒 false（Legacy 语义不
/// 变）。连续 CPU 段无 park 边界，deadline/RST/关闭终态后不得继续执
/// 行业务副作用——由 engine 驱动环周期调用本探测（安全指令边界，非
/// 抢占）。
pub(crate) fn current_request_cancelled() -> bool {
    crate::vm::host_capabilities::request_context::current_request_cancelled()
}

/// 流 open shim 调用：把新流登记进当前请求资源组（无 scope → no-op）。
pub(crate) fn register_scope_stream(stream_id: u64) {
    use crate::vm::host_capabilities::request_context::*;
    register_request_resource(RequestResourceKind::Stream, stream_id);
}

/// PLAN-727 T-05：transfer 提交 shim 调用——传输登记进当前请求的**有类型**
/// 传输资源组（无 scope → no-op；CLI/UI 程序资源归显式 cancel/wait 管理）。
pub(crate) fn register_scope_transfer(transfer_id: u64) {
    use crate::vm::host_capabilities::request_context::*;
    register_request_resource(RequestResourceKind::Transfer, transfer_id);
}

/// PLAN-729 T-04：file_response 构造 shim 调用——描述符登记进当前请求的
/// 文件描述符资源组（无 scope → no-op；编组取出或 scope 终结时移除）。
pub(crate) fn register_scope_file_response(descriptor_id: u64) {
    use crate::vm::host_capabilities::request_context::*;
    register_request_resource(RequestResourceKind::FileResponse, descriptor_id);
}

/// PLAN-730 T-02：上传句柄（注入能力/会话/收据）登记进当前请求的上传
/// 资源组（无 scope → no-op；非请求上下文构造不能进入 HTTP 编组面）。
pub(crate) fn register_scope_upload(upload_id: u64) {
    use crate::vm::host_capabilities::request_context::*;
    register_request_resource(RequestResourceKind::Upload, upload_id);
}

/// 资源基线探针（测试）：可用生命期许可数。
#[cfg(test)]
pub(crate) fn life_permit_available() -> usize {
    life_permits().available_permits()
}

/// 就绪探测（parked 表扫描用；不消费任何状态）。
pub(crate) fn parked_is_ready(vm: &std::rc::Rc<AutoVM>, p: &ParkedRequest) -> bool {
    match &p.wait {
        ParkedWait::HttpRequest(req_id) => crate::vm::ffi::stdlib::async_http_result_ready(*req_id),
        ParkedWait::Future(fid) => vm
            .futures
            .get(fid)
            .map(|f| f.read().unwrap().state != crate::vm::engine::FutureState::Pending)
            .unwrap_or(true), // future 消失 → 唤醒（引擎恢复臂同款 nil fallback）
        // PLAN-707 T-05: 外部流——数据入队或终态即就绪（stream_ready 含
        // 条目消失的终结臂）。
        ParkedWait::HttpStream(stream_id) => crate::vm::ffi::http_stream::stream_ready(*stream_id),
        // PLAN-711 T-11: CPU continuation 归 vm_bridge 的 CPU 泵，HTTP 泵
        // 不拾取（防御臂；HTTP 轨不产该凭据）。
        ParkedWait::CpuRunnable => false,
    }
}

/// 恢复 parked 请求到下一停点或终态。
pub(crate) enum ParkedResume {
    /// 终态回复（任务已清理；caller 经条目内 reply_tx 发送并移除条目）。
    Reply(ApiReply),
    /// 仍未完成（wait/seg 凭据已刷新）。
    StillParked,
    /// 本条目作废，改挂交接条目（middleware 续链重新 park 的形态）。
    Handoff(Box<ParkedRequest>),
    /// PLAN-730 T-07：回复已由下游自送（start_handler 内部消费 reply_tx；
    /// 条目直接移除，caller 不得再送）。
    Consumed,
}

pub(crate) fn resume_parked_request(
    vm: &std::rc::Rc<AutoVM>,
    p: &mut ParkedRequest,
) -> ParkedResume {
    // PLAN-707 T-06: 恢复段与首段同属请求资源组（流 open 同样登记）。
    let _scope_bind = bind_current_scope(p.scope_id);
    match p.stage {
        ParkStage::Middleware { index } => {
            enum MWResume {
                Parked(ParkedWait, ParkedSegment),
                ContinueChain,
                ShortCircuit(ApiReply),
                Missing,
            }
            let task_id = p.task_id;
            let seg = p.seg.clone();
            let step = if let Some(t_arc) = vm.tasks.get(&task_id) {
                let Ok(mut t) = t_arc.try_lock() else {
                    return ParkedResume::StillParked; // 锁忙 → 下一轮重试
                };
                // PLAN-781 T-15: 请求级累计预算随恢复段延续（不重置）。
                match vm.resume_fn_by_name_segment_http(&mut t, &seg) {
                    SegmentOutcome::Parked { wait, seg } => MWResume::Parked(wait, seg),
                    SegmentOutcome::Completed(Ok(())) => {
                        let nv = t.ram.pop_nv();
                        if auto_val::is_null(nv) {
                            MWResume::ContinueChain
                        } else {
                            match nv_to_json(vm, nv, 0) {
                                Some(resp) if !resp.is_empty() && resp != "null" => {
                                    let mut headers = vec![(
                                        "Content-Type".to_string(),
                                        "application/json".to_string(),
                                    )];
                                    headers.extend(cors_header_pairs());
                                    headers.push((
                                        "X-Request-Id".to_string(),
                                        p.ctx.request_id.clone(),
                                    ));
                                    MWResume::ShortCircuit(ApiReply::Full {
                                        status: 200,
                                        headers,
                                        body: ApiBody::Text(resp.into_bytes()),
                                    })
                                }
                                _ => MWResume::ContinueChain,
                            }
                        }
                    }
                    // legacy：middleware Err → 无响应继续链。
                    SegmentOutcome::Completed(Err(_)) => MWResume::ContinueChain,
                    // PLAN-711 T-11: HTTP 轨不产 CPU continuation（防御臂：
                    // 未完成=仍 park，凭据 CpuRunnable 对 HTTP 泵恒不就绪）。
                    SegmentOutcome::Runnable { seg } => {
                        MWResume::Parked(ParkedWait::CpuRunnable, seg)
                    }
                }
            } else {
                MWResume::Missing
            };
            match step {
                MWResume::Missing => ParkedResume::Reply(ApiReply::Full {
                    status: 200,
                    headers: cors_json_headers(&p.ctx.request_id),
                    body: ApiBody::Text(Vec::new()),
                }),
                MWResume::Parked(wait, seg) => {
                    p.wait = wait;
                    p.seg = seg;
                    ParkedResume::StillParked
                }
                MWResume::ShortCircuit(reply) => {
                    vm.remove_task(task_id);
                    ParkedResume::Reply(reply)
                }
                MWResume::ContinueChain => {
                    vm.remove_task(task_id);
                    // 续链：ctx 移交进 advance_dispatch；其再次 park 时产出
                    // 新条目（Handoff）——本条目由 caller 移除。
                    let ctx = std::mem::replace(&mut p.ctx, placeholder_ctx());
                    match advance_dispatch(vm, ctx, index + 1, p.reply_tx.take()) {
                        DispatchOutcome::Replied => ParkedResume::Reply(placeholder_reply()),
                        DispatchOutcome::Parked(np) => ParkedResume::Handoff(np),
                    }
                }
            }
        }
        // PLAN-730 T-07：legacy 落盘完成——合并 files JSON 进 body 后进
        // handler；写失败 = 真实 500（不返回不存在的假路径）。
        ParkStage::LegacyStore { op_id } => {
            let result = LEGACY_STORE_RESULTS
                .lock()
                .ok()
                .and_then(|mut m| m.remove(&op_id))
                .flatten();
            // live-op marker 出表（complete 侧落 Body("")；take 走防泄漏）。
            VmHttpBridge::discard_async_result(op_id);
            match result {
                Some(Ok((paths, files_json))) => {
                    p.ctx.legacy_stored = paths;
                    p.ctx.body = multipart_handler_json(&p.ctx.legacy_fields_json, &files_json);
                    p.ctx.multipart_json = Some(p.ctx.body.clone());
                    advance_after_legacy_store(vm, p)
                }
                Some(Err(message)) => {
                    eprintln!(
                        "[HTTP] {} {} [{}] → 500 (legacy multipart store: {})",
                        p.ctx.req_method, p.ctx.req_path, p.ctx.request_id, message
                    );
                    let mut headers = cors_json_headers(&p.ctx.request_id);
                    headers[0].1 = "application/json".to_string();
                    ParkedResume::Reply(ApiReply::Full {
                        status: 500,
                        headers,
                        body: ApiBody::Text(
                            format!("{{\"error\":{}}}", json_escape_string(&message)).into_bytes(),
                        ),
                    })
                }
                None => ParkedResume::StillParked,
            }
        }
        ParkStage::Handler => {
            enum HResume {
                Parked(ParkedWait, ParkedSegment),
                Value(auto_val::NanoValue),
                Err(crate::vm::engine::VMError),
                Missing,
            }
            let task_id = p.task_id;
            let seg = p.seg.clone();
            let step = if let Some(t_arc) = vm.tasks.get(&task_id) {
                let Ok(mut t) = t_arc.try_lock() else {
                    return ParkedResume::StillParked;
                };
                // PLAN-781 T-15: 请求级累计预算随恢复段延续（不重置）。
                match vm.resume_fn_by_name_segment_http(&mut t, &seg) {
                    SegmentOutcome::Parked { wait, seg } => HResume::Parked(wait, seg),
                    SegmentOutcome::Completed(Ok(())) => HResume::Value(t.ram.pop_nv()),
                    SegmentOutcome::Completed(Err(e)) => HResume::Err(e),
                    // PLAN-711 T-11: HTTP 轨不产 CPU continuation（防御臂）。
                    SegmentOutcome::Runnable { .. } => {
                        HResume::Err(crate::vm::engine::VMError::RuntimeError(
                            "internal: cpu-slice outcome escaped legacy http resume (unreachable)"
                                .into(),
                        ))
                    }
                }
            } else {
                HResume::Missing
            };
            match step {
                HResume::Missing => ParkedResume::Reply(ApiReply::Full {
                    status: 200,
                    headers: cors_json_headers(&p.ctx.request_id),
                    body: ApiBody::Text(Vec::new()),
                }),
                HResume::Parked(wait, seg) => {
                    p.wait = wait;
                    p.seg = seg;
                    ParkedResume::StillParked
                }
                HResume::Err(e) => {
                    vm.remove_task(task_id);
                    ParkedResume::Reply(handler_error_reply(&p.ctx, &e))
                }
                HResume::Value(nv) => match marshal_handler_value(vm, &p.ctx, nv) {
                    MarshalOutcome::Reply(reply) => {
                        // PLAN-776 T-05：与 sync 分派同款——编组产物 owned，
                        // 返回值栈槽 RC 释放（park 恢复路径的凭据回收）。
                        // R1-F02：仅 TAG_OBJECT 槽持份额；同号普通数值
                        //（rc::heap_ref_id 的历史 i32 启发面）不得扣无关资源。
                        if auto_val::is_object(nv) {
                            vm.rc_release(nv);
                        }
                        vm.remove_task(task_id);
                        ParkedResume::Reply(reply)
                    }
                    MarshalOutcome::ParkFuture(fid) => {
                        p.wait = ParkedWait::Future(fid);
                        p.stage = ParkStage::AwaitReturnFuture;
                        ParkedResume::StillParked
                    }
                    MarshalOutcome::ParkBody(internal_fid, outer_fid) => {
                        p.wait = ParkedWait::Future(outer_fid);
                        p.stage = ParkStage::AsyncReturnBody { internal_fid };
                        ParkedResume::StillParked
                    }
                },
            }
        }
        ParkStage::AsyncReturnBody { internal_fid } => {
            // 外层 await 已完成：恢复臂推送其结果并续跑挂起的 `~{}` 体
            //（resume_suspended_body 完成后 internal future 置 Ready），
            // 然后按内部 future 终态编组。
            let seg = p.seg.clone();
            let task_id = p.task_id;
            let step = if let Some(t_arc) = vm.tasks.get(&task_id) {
                match t_arc.try_lock() {
                    Ok(mut t) => Some(vm.resume_fn_by_name_segment_http(&mut t, &seg)),
                    Err(_) => return ParkedResume::StillParked,
                }
            } else {
                None
            };
            match step {
                None => {
                    let reply = final_value_reply(vm, &p.ctx, None);
                    vm.remove_task(task_id);
                    return ParkedResume::Reply(reply);
                }
                Some(SegmentOutcome::Completed(Err(_))) => {
                    let reply = final_value_reply(vm, &p.ctx, None);
                    vm.remove_task(task_id);
                    return ParkedResume::Reply(reply);
                }
                Some(SegmentOutcome::Parked { wait, seg }) => {
                    p.wait = wait;
                    p.seg = seg;
                    return ParkedResume::StillParked;
                }
                // PLAN-711 T-11: HTTP 轨不产 CPU continuation（防御臂：视为
                // 未完成继续 park，凭据 CpuRunnable 对 HTTP 泵恒不就绪）。
                Some(SegmentOutcome::Runnable { .. }) => {
                    p.wait = ParkedWait::CpuRunnable;
                    return ParkedResume::StillParked;
                }
                Some(SegmentOutcome::Completed(Ok(()))) => {}
            }
            return probe_return_future(vm, p, internal_fid);
        }
        ParkStage::AwaitReturnFuture => {
            let fid = match &p.wait {
                ParkedWait::Future(fid) => *fid,
                _ => return ParkedResume::StillParked,
            };
            let Some(fut) = vm.futures.get(&fid) else {
                // future 消失 → nil fallback（引擎恢复臂同款）。终值编组需要
                // handler 任务栈做 scratch——先编组后清理。
                let reply = final_value_reply(vm, &p.ctx, None);
                vm.remove_task(p.task_id);
                return ParkedResume::Reply(reply);
            };
            if fut.read().unwrap().state == crate::vm::engine::FutureState::Pending {
                return ParkedResume::StillParked;
            }
            let value = fut.read().unwrap().result.clone();
            let reply = final_value_reply(vm, &p.ctx, value);
            vm.remove_task(p.task_id);
            ParkedResume::Reply(reply)
        }
    }
}

/// 事件驱动的就绪扫描：按 park 顺序恢复全部就绪条目（公平、零轮询——
/// 扫描只发生在完成通知/事件唤醒后）。
pub(crate) fn drain_ready_parked(vm: &std::rc::Rc<AutoVM>, parked: &mut Vec<ParkedRequest>) {
    let mut i = 0;
    while i < parked.len() {
        // T-05: scope 失效（取消 / deadline 过期）优先于就绪检查——失效
        // 请求废弃：live-op 回收（迟到完成被 presence 守卫丢弃）、任务
        // 清理、503 终态。业务函数不再被无效驱动（AC-04）。
        let scope_dead = lookup_scope(parked[i].scope_id)
            .map(|s| !scope_usable(&s))
            .unwrap_or(true);
        if scope_dead {
            let mut p = parked.remove(i);
            abort_parked_request(vm, &mut p);
            continue;
        }
        if parked_is_ready(vm, &parked[i]) {
            let mut p = parked.remove(i);
            match resume_parked_request(vm, &mut p) {
                ParkedResume::Reply(reply) => {
                    // R04 修复轮：owner 侧幂等收口（同 Err 臂）——普通回复
                    // 发送后立即终结 scope（桥侧 complete 幂等无害），
                    // 不与 owner 退出/运行时拆除竞速；SSE/File 体许可随
                    // scope 移交响应体，保持桥侧收口。
                    let holds_in_body = matches!(
                        &reply,
                        ApiReply::Full {
                            body: ApiBody::Sse(_),
                            ..
                        } | ApiReply::Full {
                            body: ApiBody::File(_),
                            ..
                        }
                    );
                    let _ = p.reply_tx.take().map(|tx| tx.send(reply));
                    if !holds_in_body {
                        if let Some(s) = lookup_scope(p.scope_id) {
                            complete_scope(&s);
                        }
                    }
                }
                ParkedResume::StillParked => {
                    parked.insert(i, p);
                    i += 1;
                }
                ParkedResume::Handoff(np) => {
                    parked.insert(i, *np);
                    i += 1;
                }
                ParkedResume::Consumed => {} // 回复已自送——条目移除即可
            }
        } else {
            i += 1;
        }
    }
}

/// 测试访问器：next_sse_generator_value（私有 async）驱动。
pub async fn next_sse_generator_value_for_test(
    vm: &std::rc::Rc<crate::vm::engine::AutoVM>,
    iterator_id: u32,
) -> Option<u64> {
    next_sse_generator_value(vm, iterator_id).await
}

async fn next_sse_generator_value(
    vm: &std::rc::Rc<crate::vm::engine::AutoVM>,
    iterator_id: u32,
) -> Option<u64> {
    loop {
        if let Some(deadline) = sse_generator_wake_deadline(vm, iterator_id) {
            if deadline > std::time::Instant::now() {
                tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
            }
            wake_sse_generator(vm, iterator_id);
        }
        let next_task_id = vm.spawn_task(0, 1024);
        let step_result = {
            // Generator startup and bytecode may insert/remove tasks. Own the Arc
            // before driving, so no DashMap shard guard survives re-entry.
            if let Some(next_task) = vm.tasks.get(&next_task_id).map(|entry| entry.clone()) {
                match next_task.try_lock() {
                    Ok(mut next_task) => {
                        next_task.ram.push_i32(iterator_id as i32);
                        match crate::vm::native::shim_iterator_next_cooperative(
                            &mut next_task,
                            vm,
                            4096,
                        ) {
                            Ok(true) => Ok(Some(next_task.ram.pop_nv())),
                            Ok(false) => Ok(None),
                            Err(error) => Err(error),
                        }
                    }
                    Err(_) => Err(crate::vm::engine::VMError::RuntimeError(
                        "SSE iterator task is busy".to_string(),
                    )),
                }
            } else {
                Err(crate::vm::engine::VMError::RuntimeError(
                    "SSE iterator task is missing".to_string(),
                ))
            }
        };
        vm.remove_task(next_task_id);

        match step_result {
            Ok(Some(value)) => {
                if auto_val::is_i32(value) && auto_val::decode_i32(value) == -1 {
                    return None;
                }
                return Some(value);
            }
            Ok(None) => {
                // PLAN-707 T-05（D-6）：凭据化等待——外部流（ready Notify +
                // COMPLETION_NOTIFY 三段式）或外部 future（完成通知），
                // 零固定间隔轮询/自旋；cooperative sleep 走既有 wake_time
                // 臂；未知 pending 兜底 yield_now（保守不忙等死锁）。
                match generator_wait_credential(vm, iterator_id) {
                    GeneratorWait::Stream(stream_id) => {
                        VmHttpBridge::wait_stream_ready(stream_id).await;
                    }
                    GeneratorWait::LegacyStream(stream_id) => {
                        VmHttpBridge::wait_legacy_stream_ready(stream_id).await;
                    }
                    GeneratorWait::Future(_) => {
                        let notified = VmHttpBridge::completion_notified();
                        tokio::pin!(notified);
                        notified.as_mut().enable();
                        // enable 后复查：凭据已清除（先完成再 park）则直接
                        // 回拉，否则等完成通知（complete_external_future 发出）。
                        if matches!(
                            generator_wait_credential(vm, iterator_id),
                            GeneratorWait::Future(_)
                        ) {
                            notified.await;
                        }
                    }
                    GeneratorWait::None => {
                        if let Some(deadline) = sse_generator_wake_deadline(vm, iterator_id) {
                            if deadline > std::time::Instant::now() {
                                tokio::time::sleep_until(tokio::time::Instant::from_std(deadline))
                                    .await;
                            }
                            wake_sse_generator(vm, iterator_id);
                        } else {
                            tokio::task::yield_now().await;
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("[HTTP] SSE generator step failed: {:?}", error);
                return None;
            }
        }
    }
}

async fn produce_sse_frames(
    vm: std::rc::Rc<crate::vm::engine::AutoVM>,
    iterator_id: u32,
    sender: tokio::sync::mpsc::Sender<String>,
) {
    let _cleanup = SseIteratorCleanup {
        vm: vm.clone(),
        iterator_id,
    };
    let mut heartbeat = tokio::time::interval_at(
        tokio::time::Instant::now() + std::time::Duration::from_secs(1),
        std::time::Duration::from_secs(1),
    );

    loop {
        tokio::select! {
            _ = sender.closed() => break,
            _ = heartbeat.tick() => {
                if sender.send(": keep-alive\n\n".to_string()).await.is_err() {
                    break;
                }
            }
            value = next_sse_generator_value(&vm, iterator_id) => {
                let Some(value) = value else { break; };
                if let Some(frame) = crate::vm::ffi::musk_response_ctor::sse_frame_from_nv(&vm, value) {
                    if sender.send(frame).await.is_err() {
                        break;
                    }
                }
            }
        }
        tokio::task::yield_now().await;
    }
}

pub(crate) fn encode_ws_text_frame(text: &str) -> Vec<u8> {
    let payload = text.as_bytes();
    let len = payload.len();
    let mut frame = Vec::new();
    // FIN + text opcode (0x81).
    frame.push(0x81);
    // Payload length (server→client frames are NOT masked).
    if len <= 125 {
        frame.push(len as u8);
    } else if len <= 65535 {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }
    frame.extend_from_slice(payload);
    frame
}

// ============================================================================
// PLAN-669: #[api] by-name argument binding
// ============================================================================
// Shared binder for the legacy positional assembly sites (async
// build_handler_args, shim_http_server_listen, serve_blocking_stdnet,
// run_http_server_blocking). Per declared param, in declaration order,
// binding precedence is: path segment (by name) → body JSON field → query
// param (back_proxy Plan 658 parity). Missing params → HTTP 400 naming the
// param; a lone trailing unbound param whose name follows the meta-param
// convention (META_PARAM_NAMES) receives the cookies/auth metadata JSON
// (Plan 317 Phase 11 opt-in — async site passes Some). Sites whose fn
// has no API_PARAM_SIGS entry keep the legacy positional behavior unchanged.

/// Bind `#[api]` handler args by name onto the task stack. Returns the
/// number of args pushed. `body_json` is the parsed request body (when
/// parseable); `whole_body` the body-source string (multipart fields JSON,
/// converted form object, or the raw body — a lone str param that doesn't
/// bind by name receives it whole, the legacy single-arg body contract);
/// `metadata_json` the cookies/auth payload a trailing opt-in param receives
/// (recognized by param name — see `META_PARAM_NAMES`).
pub(crate) fn bind_api_args_by_name(
    vm: &crate::vm::engine::AutoVM,
    task: &mut crate::vm::task::AutoTask,
    sigs: &[ApiParamSig],
    route_match: &RouteMatch,
    body_json: Option<&serde_json::Value>,
    whole_body: &str,
    metadata_json: Option<&str>,
    method: &str,
    req_path: &str,
    upload_handle: Option<i64>,
) -> Result<usize, ApiArgBindError> {
    let mut n_args = 0usize;
    let mut unbound: Vec<&ApiParamSig> = Vec::new();
    for sig in sigs {
        // PLAN-730 T-05：UploadRequest 注入参数——类型身份识别（734 契约
        // 单源）置于 path/body/query/meta 全部规则**之前**（防 "req" meta
        // 名约定与 whole-body 单参容忍吞掉注入参数；AC-01 反例防线）。
        if crate::api::contract::is_upload_param(&sig.ty) {
            match upload_handle {
                Some(h) => {
                    task.ram.push_i32(h as i32);
                    n_args += 1;
                    continue;
                }
                None => {
                    return Err(ApiArgBindError::Internal(format!(
                        "upload param `{}` has no injected request capability",
                        sig.name
                    )));
                }
            }
        }
        if let Some((_, v)) = route_match.path_params.iter().find(|(n, _)| n == &sig.name) {
            push_typed_string_arg(vm, task, sig, v, method, req_path)?;
            n_args += 1;
            continue;
        }
        if let Some(v) = body_json.and_then(|b| b.get(&sig.name)) {
            // PLAN-734 T-03：body 值按声明类型校验（错形态 400，不静默
            // 按值自身形态 marshal；AC-02）。int 语义 i64 全域。
            if let Err(msg) = validate_body_value(&sig.ty, v) {
                return Err(ApiArgBindError::BadRequest(format!(
                    "invalid value for param `{}`: {}",
                    sig.name, msg
                )));
            }
            crate::vm::ffi::stdlib::json_to_vm_value(task, vm, v, 0).map_err(|e| {
                ApiArgBindError::Internal(format!(
                    "body param `{}` marshal failed: {e:?}",
                    sig.name
                ))
            })?;
            n_args += 1;
            continue;
        }
        if let Some((_, v)) = route_match
            .query_params
            .iter()
            .find(|(n, _)| n == &sig.name)
        {
            push_typed_string_arg(vm, task, sig, v, method, req_path)?;
            n_args += 1;
            continue;
        }
        unbound.push(sig);
    }

    // Whole-body single-param tolerance (legacy single-arg body contract —
    // Plan 346 "Push body"; checked BEFORE the metadata rule so `fn save(data
    // str)` + any non-matching body, `fn upload(form str)` + multipart fields
    // JSON, and unparseable text bodies all keep receiving the whole body).
    if unbound.len() == 1 && sigs.len() == 1 && !whole_body.is_empty() {
        push_str_arg(vm, task, whole_body);
        return Ok(n_args + 1);
    }

    // Trailing metadata opt-in (Plan 317 Phase 11 semantics, by-name form):
    // exactly one trailing declared param unbound AND its name follows the
    // metadata-param convention (the 023-realworld / e2e corpus names it
    // `meta`) → cookies/auth JSON. Anything else unbound is a missing param.
    if unbound.len() == 1
        && std::ptr::eq(
            unbound[0],
            sigs.last().expect("unbound implies sigs nonempty"),
        )
        && crate::api::contract::is_meta_alias(&unbound[0].name)
    {
        if let Some(meta) = metadata_json {
            push_str_arg(vm, task, meta);
            return Ok(n_args + 1);
        }
    }

    if let Some(sig) = unbound.first() {
        return Err(ApiArgBindError::BadRequest(format!(
            "missing param `{}` for {} {}",
            sig.name, method, req_path
        )));
    }
    Ok(n_args)
}

/// Param names that opt into the cookies/auth metadata payload (Plan 317
/// Phase 11 / Plan 346 stage 4 convention — the declared trailing param that
/// receives `{"cookies":...,"auth":...}`). Case-insensitive.
const META_PARAM_NAMES: [&str; 4] = ["meta", "metadata", "req", "request"];

/// PLAN-669 sync-serve sites (serve_blocking_stdnet, run_http_server_blocking,
/// shim_http_server_listen): by-name bind when the fn's sigs are published,
/// else the legacy positional convention. Err → caller writes a 400/500
/// response and skips the handler call. No metadata source on these loops —
/// a meta-convention param 400s with a named error instead of garbage.
pub(crate) fn bind_api_args_or_legacy(
    vm: &crate::vm::engine::AutoVM,
    ht: &mut crate::vm::task::AutoTask,
    fn_name: &str,
    path_params: &[(String, String)],
    query_params: &[(String, String)],
    body: &str,
    method: &str,
    req_path: &str,
) -> Result<usize, ApiArgBindError> {
    if let Some(sigs) = api_param_sigs(fn_name) {
        let route_match = RouteMatch {
            fn_name: fn_name.to_string(),
            path_params: path_params.to_vec(),
            query_params: query_params.to_vec(),
        };
        let body_json: Option<serde_json::Value> = serde_json::from_str(body).ok();
        bind_api_args_by_name(
            vm,
            ht,
            &sigs,
            &route_match,
            body_json.as_ref(),
            body,
            None,
            method,
            req_path,
            None,
        )
    } else {
        let mut n_args = 0;
        for (_param_name, param_val) in path_params {
            // Plan 326 Phase 5 legacy heuristic (no sigs published).
            if let Ok(i) = param_val.parse::<i32>() {
                ht.ram.push_i32(i);
            } else {
                push_str_arg(vm, ht, param_val);
            }
            n_args += 1;
        }
        if !body.is_empty() {
            push_str_arg(vm, ht, body);
            n_args += 1;
        }
        Ok(n_args)
    }
}

/// PLAN-698 T-02/SD-02: POST 广播臂单测——事件 payload 形态与生成 Axum
/// 侧（api_gen broadcast_event_name + events::broadcast）逐字段对拍。
#[cfg(test)]
mod plan698_publisher_tests {
    use super::cleanup_sse_iterator;
    use super::{
        publish_post_broadcast, record_api_param_sigs, record_api_return_type, ApiParamSig,
    };
    use crate::vm::ffi::stdlib::bus_subscribe_for_test;

    fn sig(name: &str, ty: &str) -> ApiParamSig {
        ApiParamSig {
            name: name.to_string(),
            ty: ty.to_string(),
        }
    }

    #[test]
    fn typing_post_broadcasts_typing_event_with_first_str_param() {
        record_api_param_sigs("set_typing", vec![sig("sender", "str")]);
        record_api_return_type("set_typing", "()".into());
        record_api_return_type("stream", "~Stream<ChatEvent>".into());
        let mut rx = bus_subscribe_for_test();

        publish_post_broadcast("set_typing", r#"{"sender":"Carol"}"#, "null");

        let json = rx.blocking_recv().unwrap();
        let evt: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(evt["event"], "Typing");
        assert_eq!(evt["name"], "Carol");
    }

    #[test]
    fn entity_post_broadcasts_newtype_event_with_injected_discriminator() {
        record_api_return_type("send_message", "Message".into());
        record_api_return_type("stream", "~Stream<ChatEvent>".into());
        let mut rx = bus_subscribe_for_test();

        publish_post_broadcast(
            "send_message",
            r#"{"sender":"Bob","text":"hi"}"#,
            r#"{"id":5,"sender":"Bob","text":"hi","time":"Just now","mine":true}"#,
        );

        let json = rx.blocking_recv().unwrap();
        let evt: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            evt,
            serde_json::json!({
                "id": 5, "sender": "Bob", "text": "hi",
                "time": "Just now", "mine": true,
                "event": "NewMessage",
            })
        );
    }

    #[test]
    fn non_object_or_non_entity_responses_are_not_broadcast() {
        record_api_return_type("stream", "~Stream<ChatEvent>".into());
        record_api_return_type("stream_only_fn", "~Stream<ChatEvent>".into());
        let mut rx = bus_subscribe_for_test();

        // ~Stream 返回类型的 fn 自身不是实体创建——不广播。
        publish_post_broadcast("stream_only_fn", "{}", r#"{"any":1}"#);
        // 响应体不是 JSON 对象（标量/数组/坏 JSON）——不广播。
        record_api_return_type("scalar_fn", "int".into());
        publish_post_broadcast("scalar_fn", "{}", "42");
        publish_post_broadcast("scalar_fn", "{}", "not-json");

        // 无事件应到达（有界等待确认空）。
        let mut arrived = false;
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(300);
        while std::time::Instant::now() < deadline {
            if rx.try_recv().is_ok() {
                arrived = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(!arrived, "no event expected for stream/scalar responses");
    }
}

/// PLAN-698 F-1（review #1）: cleanup_sse_iterator 回收 AsyncHttpStream
/// 的 ASYNC_STREAMS 句柄——rx drop 即 bus 转发线程 blocking_send 失败
/// 退出（否则订阅端断连后线程永生）。
#[cfg(test)]
mod plan698_f1_cleanup_tests {
    use super::cleanup_sse_iterator;

    #[test]
    fn cleanup_reclaims_async_stream_handle() {
        let vm = crate::vm::engine::AutoVM::new(
            crate::vm::virt_memory::VirtualFlash::new_with_code(vec![
                crate::vm::opcode::OpCode::RET as u8,
            ]),
            1024,
        );
        let (task_id, iter_id, stream_id) = {
            let tid = vm.spawn_task(0, 8192);
            let arc = vm.tasks.get(&tid).expect("task spawned");
            let mut task = arc.blocking_lock();
            crate::vm::ffi::stdlib::shim_bus_subscribe(&mut task, &vm)
                .expect("subscribe shim runs");
            let iter_id = auto_val::decode_i32(task.ram.peek_nv(0)) as u32;
            let stream_id = match vm.iterators.get(&iter_id).map(|it| it.clone()) {
                Some(crate::vm::engine::Iterator::AsyncHttpStream(a)) => a.stream_id,
                other => panic!("expected AsyncHttpStream, got {other:?}"),
            };
            (tid, iter_id, stream_id)
        };
        assert!(
            crate::vm::ffi::stdlib::ASYNC_STREAMS
                .lock()
                .unwrap()
                .contains_key(&stream_id),
            "handle registered by subscribe"
        );

        // DashMap 读守卫全 drop 后再 cleanup（同 shard 自死锁陷阱纪律）。
        vm.remove_task(task_id);
        cleanup_sse_iterator(&vm, iter_id);

        assert!(
            !crate::vm::ffi::stdlib::ASYNC_STREAMS
                .lock()
                .unwrap()
                .contains_key(&stream_id),
            "F-1: handle reclaimed on cleanup — forwarder thread unblocks and exits"
        );
        assert!(
            !vm.iterators.contains_key(&iter_id),
            "iterator entry removed"
        );
    }
}

/// PLAN-699 T-01 spike: prove the Axum/Hyper transport topology compiles and
/// runs before fixing the bridge types. Decision evidence lands in
/// docs/plans/reports/699-bridge-decision.md. Kept as the seed of the new
/// protocol E2E suite (raw-TCP framing probes); will be folded into the
/// real implementation's test module.
#[cfg(test)]
mod spike699_axum_transport {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    use axum::extract::Request;
    use axum::response::Response;
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto::Builder as ConnBuilder;
    use hyper_util::server::graceful::GracefulShutdown;
    use hyper_util::service::TowerToHyperService;

    const HEADER_BUF: usize = 64 * 1024;
    const HEADER_READ_TIMEOUT: Duration = Duration::from_secs(2);

    struct SpikeServer {
        port: u16,
        shutdown_tx: tokio::sync::watch::Sender<bool>,
        done_rx: std::sync::mpsc::Receiver<()>,
    }

    impl SpikeServer {
        fn start(router: axum::Router) -> Self {
            let (ready_tx, ready_rx) = std::sync::mpsc::channel::<u16>();
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let (done_tx, done_rx) = std::sync::mpsc::channel();
            let _handle = std::thread::Builder::new()
                .name("spike699-net".into())
                .stack_size(4 * 1024 * 1024)
                .spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("spike net runtime");
                    rt.block_on(async move {
                        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                            .await
                            .expect("spike bind");
                        let port = listener.local_addr().unwrap().port();
                        // Ready signal BEFORE the accept loop — start() waits
                        // on it; sending after the loop would deadlock.
                        let _ = ready_tx.send(port);
                        let graceful = GracefulShutdown::new();
                        let mut shutdown_rx = shutdown_rx;
                        loop {
                            tokio::select! {
                                accepted = listener.accept() => {
                                    let (stream, _peer) = match accepted {
                                        Ok(x) => x,
                                        Err(_) => break,
                                    };
                                    let svc = TowerToHyperService::new(router.clone());
                                    let watcher = graceful.watcher();
                                    tokio::spawn(async move {
                                        // http1_only: the transport contract is
                                        // HTTP/1.1 — the auto builder would
                                        // otherwise also serve h2c prior-knowledge
                                        // connections, silently widening scope.
                                        let mut builder = ConnBuilder::new(TokioExecutor::new())
                                            .http1_only();
                                        builder
                                            .http1()
                                            .timer(hyper_util::rt::TokioTimer::new())
                                            .max_buf_size(HEADER_BUF)
                                            .header_read_timeout(HEADER_READ_TIMEOUT);
                                        let conn = builder.serve_connection_with_upgrades(
                                            TokioIo::new(stream),
                                            svc,
                                        );
                                        let _ = watcher.watch(conn).await;
                                    });
                                }
                                _ = shutdown_rx.changed() => {
                                    // Graceful: stop accepting, drain in-flight
                                    // connections with a deadline, then force out.
                                    let _ = tokio::time::timeout(
                                        Duration::from_secs(2),
                                        graceful.shutdown(),
                                    )
                                    .await;
                                    break;
                                }
                            }
                        }
                        drop(listener);
                        let _ = done_tx.send(());
                    });
                })
                .expect("spawn spike net thread");
            let port = ready_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("spike server ready");
            SpikeServer {
                port,
                shutdown_tx,
                done_rx,
            }
        }
    }

    fn connect(port: u16) -> TcpStream {
        let stream = TcpStream::connect(("127.0.0.1", port)).expect("spike connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
    }

    fn read_response(stream: &mut TcpStream) -> String {
        read_response_until(stream, &|resp: &str, head_done: bool, _bytes: usize| {
            // One complete response head (+ its content-length body when present).
            if !head_done {
                return false;
            }
            let lower = resp.to_ascii_lowercase();
            let cl = lower
                .split("\r\n")
                .find_map(|l| {
                    l.strip_prefix("content-length:")
                        .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                })
                .unwrap_or(0);
            // No content-length = streaming body: the caller supplies its own
            // `until` predicate; default waits for EOF via head_done=false… so
            // keep the head-only contract here and let custom predicates decide.
            let pos = find_head_end(resp.as_bytes()).unwrap_or(resp.len());
            resp.len() >= pos + cl
        })
    }

    /// Read from the socket until `until` returns true (checked after every
    /// read), EOF, or the 5s read timeout fires.
    fn read_response_until(
        stream: &mut TcpStream,
        until: &dyn Fn(&str, bool, usize) -> bool,
    ) -> String {
        let mut resp = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = match stream.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            resp.extend_from_slice(&buf[..n]);
            let s = String::from_utf8_lossy(&resp).to_string();
            let head_done = find_head_end(&resp).is_some();
            if until(&s, head_done, resp.len()) {
                break;
            }
            if resp.len() > 256 * 1024 {
                break;
            }
        }
        String::from_utf8_lossy(&resp).into_owned()
    }

    fn find_head_end(b: &[u8]) -> Option<usize> {
        b.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4)
    }

    /// SSE body adapter: tokio mpsc Receiver → Stream with correct waker
    /// wiring (poll_recv registers the waker; a naive try_recv+Pending
    /// poll_fn never wakes and the body starves — spike-verified pitfall).
    struct FrameStream {
        rx: tokio::sync::mpsc::Receiver<String>,
    }

    impl futures::Stream for FrameStream {
        type Item = Result<String, std::convert::Infallible>;
        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            std::pin::Pin::new(&mut self.rx)
                .poll_recv(cx)
                .map(|opt| opt.map(Ok))
        }
    }

    async fn spike_router() -> axum::Router {
        axum::Router::new()
            .route(
                "/api/echo-len",
                axum::routing::post(|body: String| async move { body.len().to_string() }),
            )
            .route("/api/hello", axum::routing::get(|| async { "hi" }))
            .route(
                "/api/sse",
                axum::routing::get(|| async {
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(1);
                    tokio::spawn(async move {
                        for i in 0..3 {
                            if tx.send(format!("data: {i}\n\n")).await.is_err() {
                                break;
                            }
                        }
                    });
                    let stream = FrameStream { rx };
                    axum::response::Response::builder()
                        .status(200)
                        .header("content-type", "text/event-stream")
                        .header("cache-control", "no-cache")
                        .body(axum::body::Body::from_stream(stream))
                        .unwrap()
                }),
            )
    }

    /// AC-01 probe: keep-alive — two requests on one connection.
    #[test]
    fn spike_keepalive_two_requests_one_connection() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        let req = "GET /api/hello HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        s.write_all(req.as_bytes()).unwrap();
        s.write_all(req.as_bytes()).unwrap();
        let resp = read_response_until(&mut s, &|resp, _head, _bytes| {
            resp.matches("HTTP/1.1").count() >= 2 && {
                // Second head must also have its (content-length) body.
                let second = resp.split("HTTP/1.1").nth(2).unwrap_or("");
                second.contains("\r\n\r\n")
            }
        });
        let count = resp.matches("HTTP/1.1 200 OK").count();
        assert_eq!(
            count, 2,
            "two keep-alive responses on one connection, got: {resp:?}"
        );
    }

    /// AC-01 probe: chunked request body is decoded by the transport.
    #[test]
    fn spike_chunked_request_body_decoded() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        let body = b"hello chunked world";
        let req = format!(
            "POST /api/echo-len HTTP/1.1\r\nHost: 127.0.0.1\r\nTransfer-Encoding: chunked\r\n\r\n\
             {:x}\r\n{}\r\n0\r\n\r\n",
            body.len(),
            String::from_utf8_lossy(body)
        );
        s.write_all(req.as_bytes()).unwrap();
        let resp = read_response(&mut s);
        assert!(
            resp.starts_with("HTTP/1.1 200"),
            "chunked body accepted, got: {resp:?}"
        );
        assert!(
            resp.contains(&body.len().to_string()),
            "decoded length echoed, got: {resp:?}"
        );
    }

    /// AC-03 probe: oversized headers must produce a determinate outcome.
    #[test]
    fn spike_oversized_headers_rejected() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        let big = "X-Big: ".to_string() + &"a".repeat(HEADER_BUF + 4096);
        let req = format!("GET /api/hello HTTP/1.1\r\nHost: 127.0.0.1\r\n{big}\r\n\r\n");
        let _ = s.write_all(req.as_bytes());
        let resp = read_response(&mut s);
        let status = resp.lines().next().unwrap_or("").to_string();
        assert!(
            status.contains("431") || status.contains("400") || resp.is_empty(),
            "oversized headers → 431/400/close, got: {status:?}"
        );
        eprintln!(
            "[spike699] oversized headers outcome: {status:?} (bytes={})",
            resp.len()
        );
    }

    /// AC-03 probe: slow headers must hit the header read timeout and close.
    #[test]
    fn spike_slow_headers_timeout() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        let _ = s.write_all(b"GET /api/hello HTTP/1.1\r\nHost: 127.0.0.1\r\n");
        let _ = s.flush();
        let started = std::time::Instant::now();
        let mut resp = Vec::new();
        let _ = s.read_to_end(&mut resp);
        let elapsed = started.elapsed();
        assert!(
            elapsed >= Duration::from_millis(1500) && elapsed < Duration::from_secs(6),
            "slow headers closed after header_read_timeout, elapsed={elapsed:?}"
        );
        eprintln!(
            "[spike699] slow headers outcome: {:?} after {elapsed:?}",
            String::from_utf8_lossy(&resp)
                .lines()
                .next()
                .unwrap_or("<eof>")
        );
    }

    /// AC-04 probe: SSE streams over the transport.
    #[test]
    fn spike_sse_frames_stream() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        s.write_all(b"GET /api/sse HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        let resp = read_response_until(&mut s, &|resp, _head, _bytes| resp.contains("data: 2"));
        assert!(resp.contains("200 OK"), "SSE status, got {resp:?}");
        assert!(resp.contains("text/event-stream"), "SSE content-type");
        assert!(
            resp.contains("data: 0") && resp.contains("data: 2"),
            "frames streamed"
        );
    }

    /// AC-05 probe: graceful shutdown stops accept, drains, releases the port.
    #[test]
    fn spike_graceful_shutdown_releases_port() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        s.write_all(b"GET /api/hello HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        let _ = read_response(&mut s);
        server.shutdown_tx.send(true).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        let done = server.done_rx.recv_timeout(Duration::from_secs(4));
        assert!(done.is_ok(), "net loop exited after graceful shutdown");
        // Port must be free again.
        let rebind = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { tokio::net::TcpListener::bind(("127.0.0.1", server.port)).await });
        assert!(rebind.is_ok(), "port rebindable after shutdown");
    }
}

// =============================================================================
// PLAN-093 (G-8 系) catch-all 路由匹配回归：axum `{*rest}` 翻译为 `*rest` 段
// 后须消费全部剩余段——此前段数严格相等检查使多段剩余路径永远 404。
// =============================================================================
#[cfg(test)]
mod plan093_catch_all_tests {
    use super::{match_route, HttpRoute};

    fn routes() -> Vec<HttpRoute> {
        vec![
            HttpRoute {
                method: "GET".into(),
                path: "/api/files/:workspace_id/*path".into(),
                fn_name: "workspace_file".into(),
            },
            HttpRoute {
                method: "GET".into(),
                path: "/api/chats/session/:id".into(),
                fn_name: "chat_get".into(),
            },
        ]
    }

    #[test]
    fn catch_all_consumes_remaining_segments() {
        let m = match_route(
            &routes(),
            "GET",
            "/api/files/default/targets/probe-a/pac.at",
        )
        .expect("multi-segment catch-all must match");
        assert_eq!(m.fn_name, "workspace_file");
        let ws = m
            .path_params
            .iter()
            .find(|(k, _)| k == "workspace_id")
            .unwrap();
        assert_eq!(ws.1, "default");
        let rest = m.path_params.iter().find(|(k, _)| k == "path").unwrap();
        assert_eq!(rest.1, "targets/probe-a/pac.at");
    }

    #[test]
    fn catch_all_still_requires_prefix() {
        // 少于路由前缀段数 → 不匹配（workspace_id 缺失）。
        assert!(match_route(&routes(), "GET", "/api/files").is_none());
    }

    #[test]
    fn catch_all_single_segment_remainder() {
        let m = match_route(&routes(), "GET", "/api/files/default/notes.md")
            .expect("single-segment remainder must match");
        let rest = m.path_params.iter().find(|(k, _)| k == "path").unwrap();
        assert_eq!(rest.1, "notes.md");
    }

    #[test]
    fn non_catch_all_still_strict_length() {
        // 非 catch-all 路由保持严格段数：多余段不匹配。
        assert!(match_route(&routes(), "GET", "/api/chats/session/a/b").is_none());
        let m = match_route(&routes(), "GET", "/api/chats/session/abc")
            .expect("exact-length param route must match");
        assert_eq!(m.fn_name, "chat_get");
    }

    #[test]
    fn catch_all_decodes_percent_encoded_remainder() {
        let m = match_route(&routes(), "GET", "/api/files/default/wiki/Hello%20World.ad")
            .expect("encoded remainder must match");
        let rest = m.path_params.iter().find(|(k, _)| k == "path").unwrap();
        assert_eq!(rest.1, "wiki/Hello World.ad");
    }
}

// ---- PLAN-843 结构拆分：服务职责分族（原单文件热点，纯搬移） ----
#[path = "http_server/server_argbind.rs"]
mod server_argbind;
pub use server_argbind::*;
#[path = "http_server/server_dispatch.rs"]
mod server_dispatch;
pub use server_dispatch::*;
#[path = "http_server/server_model.rs"]
mod server_model;
pub use server_model::*;
#[path = "http_server/server_metadata.rs"]
mod server_metadata;
pub(crate) use server_metadata::*;
#[path = "http_server/server_route.rs"]
mod server_route;
pub use server_route::*;
#[path = "http_server/server_scope.rs"]
mod server_scope;
pub use server_scope::*;

#[path = "http_server_handle.rs"]
pub mod server_blocking;

#[path = "http_server/server_legacy.rs"]
pub mod server_legacy;
