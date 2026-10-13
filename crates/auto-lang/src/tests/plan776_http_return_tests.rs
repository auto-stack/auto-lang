//! PLAN-776 T-01 表示探针——HTTP 返回资源凭据的载体假设冻结（决策报告 §6）。
//!
//! 冻结方案 A：资源凭据 = heap_objects 表 entry（TypeTag 专属变体，T-02 落地），
//! NanoValue 侧 TAG_OBJECT `encode_object`，Value 侧 `Value::VmRef`。本探针用
//! 现有 `RustStdlibObject` 作资源对象替身，验证三条地基假设：
//!
//! (1) **传播保身份**：TAG_OBJECT 凭据经 globals / FutureValue.result /
//!     closure captures（Value::VmRef 载体）传播后 downcast 不变；
//! (2) **数字不可伪造**：任意 encode_i32/encode_i64——包括与活跃 heap id
//!     同值、Future 0xF0 位模式、算术结果——is_object 恒 false；
//! (3) **查表制对照**：现状 `insert_http_response`/`lookup_http_response`
//!     是表命中制（本计划要叠加资源臂的兼容面），同值 int 不是对象。
//!
//! 正式资源类型与编组分派由 T-02/T-03 交付。T-07 覆盖矩阵（§6）：
//! - 真碰撞（AC-01）：Response 族 e2e（iterator 族 = e2e_sse_chain/plan707
//!   既有矩阵，查表命中制对照面保留）；
//! - 逐值身份（AC-02）：探针 + T-02 传播/接线；
//! - sync/async（AC-03）：wire 三态 + i64 全宽 + record 值域；
//! - 终态负向（AC-04）：Failed→500 + stale/wrong-owner 单测；
//! - 存活（AC-06）：1000 闭环 + SSE 断连回收 + rc 链路单测；
//! - 兼容/来源（AC-07/08）：提取级遮蔽 + VM 门 + plan326/705/734/738/755/
//!   442 scoped 族 + auto-man api_gen 46（legacy 6 wire 用例 = plan326
//!   http_e2e 34 项内含 redirect/SSE/accessors/musk 常量）。

use crate::vm::engine::{AutoVM, FutureKind, FutureState, FutureValue};
use crate::vm::ffi::rust_stdlib::RustStdlibObject;
use crate::vm::ffi::stdlib;
use crate::vm::virt_memory::VirtualFlash;

fn fresh_vm() -> AutoVM {
    AutoVM::new(VirtualFlash::new(1024), 1024)
}

/// 探针 (1)：对象凭据的 NanoValue/Value 双载体在共享全局、Future 终值、
/// closure 捕获三传播形态下身份保持（AC-02 的表示地基）。
#[test]
fn plan776_probe_resource_identity_survives_propagation() {
    let vm = fresh_vm();
    let sentinel = 776_776u64;
    let handle = vm.insert_heap_object(RustStdlibObject::new("plan776.probe.Sentinel", sentinel));
    let nv = auto_val::encode_object(handle as u32);
    assert!(auto_val::is_object(nv), "资源凭据必须是 TAG_OBJECT 载体");

    // 共享全局（NanoValue 传播）。
    vm.globals.insert("plan776_probe_resource".to_string(), nv);
    let back = vm
        .globals
        .get("plan776_probe_resource")
        .expect("globals 读回");
    assert!(auto_val::is_object(*back), "globals 传播丢 TAG_OBJECT");
    assert_eq!(auto_val::decode_object(*back) as u64, handle);

    // Future 终值（Value::VmRef 传播——External/Internal 终值同一载体）。
    let value = auto_val::Value::VmRef(auto_val::VmRef {
        id: handle as usize,
    });
    vm.futures.insert(
        1,
        std::sync::Arc::new(std::sync::RwLock::new(FutureValue {
            body_offset: 0,
            state: FutureState::Ready,
            result: Some(value),
            owner_task_id: 0,
            captures: std::collections::HashMap::new(),
            kind: FutureKind::External,
        })),
    );
    let fut = vm.futures.get(&1).expect("future 读回");
    let result = fut.read().unwrap().result.clone().expect("result 在位");
    match result {
        auto_val::Value::VmRef(r) => assert_eq!(r.id as u64, handle),
        other => panic!("Future 终值载体应为 VmRef，实际 {other:?}"),
    }
    drop(fut);

    // closure 捕获（captures: HashMap<String, Value>）。
    let mut fut2 = vm.futures.get_mut(&1).expect("future 可变读");
    fut2.write().unwrap().captures.insert(
        "res".to_string(),
        auto_val::Value::VmRef(auto_val::VmRef {
            id: handle as usize,
        }),
    );
    drop(fut2);
    let cap = vm
        .futures
        .get(&1)
        .unwrap()
        .read()
        .unwrap()
        .captures
        .get("res")
        .cloned()
        .expect("capture 在位");
    match cap {
        auto_val::Value::VmRef(r) => assert_eq!(r.id as u64, handle),
        other => panic!("closure 捕获载体应为 VmRef，实际 {other:?}"),
    }

    // 三形态读回后 downcast 均成功（身份未衰减为普通值）。
    let obj = vm.get_heap_object(handle).expect("堆对象存活");
    let guard = obj.read().unwrap();
    let ro = guard
        .as_any()
        .downcast_ref::<RustStdlibObject>()
        .expect("downcast 保持（传播后仍是同一对象类型）");
    let inner = ro.downcast_ref::<u64>().expect("载荷类型保持");
    assert_eq!(*inner, sentinel);
}

/// 探针 (2)：普通数字永无资源身份（AC-01 的表示地基）。
/// 与活跃 heap id 同值的 i32/i64、Future 0xF0 位模式、算术结果——
/// 任何可由数值运算制造的形态都不是 TAG_OBJECT。
#[test]
fn plan776_probe_plain_numbers_never_gain_resource_identity() {
    let vm = fresh_vm();
    let handle = vm.insert_heap_object(RustStdlibObject::new("plan776.probe.Collision", 1u32));
    // 同值普通数（真碰撞形态：id 已知且资源活跃）。
    let i32_same = auto_val::encode_i32(handle as i32);
    let i64_same = auto_val::try_encode_i64(handle as i64).expect("小值 i64 应可 48 位内联编码");
    assert!(!auto_val::is_object(i32_same), "同值 i32 不得命中对象 tag");
    assert!(!auto_val::is_object(i64_same), "同值 i64 不得命中对象 tag");
    assert!(auto_val::is_i32(i32_same) && auto_val::is_i64(i64_same));

    // Future 探测位模式（marshal 0xF0 探测的输入形态）仍是普通数。
    let fid = 42u32;
    let future_pattern = auto_val::encode_i32(((fid << 8) | 0xF0) as i32);
    assert!(!auto_val::is_object(future_pattern));

    // 算术结果不可能跨 tag：对象 tag 位型无法由 int 运算制造。
    let computed = auto_val::encode_i32((auto_val::decode_i32(i32_same) + 1) - 1);
    assert!(!auto_val::is_object(computed));
    assert!(auto_val::is_i32(computed));

    // TAG_OBJECT 只能经 encode_object（受控点）产生。
    let real = auto_val::encode_object(handle as u32);
    assert!(auto_val::is_object(real));
    assert_ne!(real, i32_same);
    assert_ne!(real, i64_same);
}

/// 探针 (3)：现状查表制对照——insert/lookup_http_response 是 id 表命中，
/// 同值 int 的 NanoValue 仍是数据形态。资源臂（T-03）叠加后，真实资源走
/// 对象分派，legacy int 继续查表（6 个 wire 用例兼容面）。
#[test]
fn plan776_probe_legacy_registry_lookup_vs_object_identity() {
    let handle = stdlib::insert_http_response(
        302,
        vec![("Location".to_string(), "/moved".to_string())],
        b"moved".to_vec(),
    );
    // 表命中制：lookup 命中并给出 owned 快照（clone 不消费——重复返回语义）。
    let first = stdlib::lookup_http_response(handle).expect("首次查表命中");
    assert_eq!(first.0, 302);
    let second = stdlib::lookup_http_response(handle).expect("重复查表仍命中");
    assert_eq!(
        second.0, 302,
        "lookup 不消费（Response 可重复返回的现行为）"
    );

    // 同值 int 是数据：查表命中不等于持有对象身份。
    let as_int = auto_val::encode_i32(handle as i32);
    assert!(!auto_val::is_object(as_int));
    // 未登记 id 查表不命中（表命中制无凭据校验的对照面）。
    assert!(stdlib::lookup_http_response(handle + 1).is_none());
}

// ============================================================================
// T-02：producer 接线（shim 级——构造点返回 TAG_OBJECT 凭据）
// ============================================================================

use crate::vm::ffi::http_return_resource::{self, ResourceClaim};
use crate::vm::task::AutoTask;

fn fresh_task() -> AutoTask {
    AutoTask::new(0, 1024, 0)
}

/// redirect 构造点：`http.response.redirect(url, 302)` → Response 凭据对象
/// （不进 HTTP_RESPONSES 表；快照 302 + Location）。
#[test]
fn plan776_t02_redirect_shim_pushes_resource() {
    let vm = fresh_vm();
    let mut task = fresh_task();
    use crate::vm::ffi::VMConvertible;
    "/next".to_string().push_to_stack(&mut task, &vm).unwrap();
    task.ram.push_i32(302);
    stdlib::shim_http_response_redirect(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_object(nv), "redirect 返回凭据对象");
    match http_return_resource::resolve_resource(&vm, nv).unwrap() {
        ResourceClaim::Response(s) => {
            assert_eq!(s.status, 302);
            assert_eq!(s.headers[0].1, "/next");
        }
        other => panic!("错种类：{other:?}"),
    }
}

/// musk ctor 接线：`ok_response(v)` → 200 JSON Response 凭据。
#[test]
fn plan776_t02_musk_ok_response_pushes_resource() {
    let vm = fresh_vm();
    let mut task = fresh_task();
    task.ram.push_i32(7);
    crate::vm::ffi::musk_response_ctor::shim_ok_response(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_object(nv));
    match http_return_resource::resolve_resource(&vm, nv).unwrap() {
        ResourceClaim::Response(s) => {
            assert_eq!(s.status, 200);
            assert_eq!(s.body, b"7".to_vec());
            assert_eq!(s.headers[0].1, "application/json");
        }
        other => panic!("错种类：{other:?}"),
    }
}

/// file 构造点：`http.file_response(root, path, "{}")` → File 凭据对象 +
/// 描述符表项在位（take 单次移交合同保留）。
#[test]
fn plan776_t02_file_response_shim_pushes_resource() {
    let vm = fresh_vm();
    let mut task = fresh_task();
    use crate::vm::ffi::VMConvertible;
    "root".to_string().push_to_stack(&mut task, &vm).unwrap();
    "x.bin".to_string().push_to_stack(&mut task, &vm).unwrap();
    "{}".to_string().push_to_stack(&mut task, &vm).unwrap();
    crate::vm::ffi::http_server_file::shim_http_file_response(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_object(nv));
    match http_return_resource::resolve_resource(&vm, nv).unwrap() {
        ResourceClaim::File { descriptor_id } => {
            // 表项在位（take 可消费——单次移交）。
            assert!(crate::vm::ffi::http_server_file::take_file_response(descriptor_id).is_some());
            assert!(crate::vm::ffi::http_server_file::take_file_response(descriptor_id).is_none());
        }
        other => panic!("错种类：{other:?}"),
    }
}

// ============================================================================
// T-03：wire 级（真 TCP，test-http-e2e）——资源回复/快照重放/真碰撞
// ============================================================================

#[cfg(feature = "test-http-e2e")]
mod http_e2e {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    const PORT: u16 = 19010;

    /// 服务程序：redirect 资源全局缓存（重放）+ 同值普通 int 端点（碰撞）。
    /// 调用形态对齐真实 wire 契约 `http.response_redirect`（CALL_NAT 直查名，
    /// e2e_a_redirect_302_with_location 同款；点链 CALL_SPEC 形式主线预存
    /// 静默 None，不在本计划面内）。
    fn program() -> String {
        r#"
var cached_redirect = 0

#[api(method = "GET", path = "/api/make")]
fn make() int {
    cached_redirect = http.response_redirect("/target", 302)
    return cached_redirect
}

#[api(method = "GET", path = "/api/replay")]
fn replay() int {
    return cached_redirect
}

#[api(method = "GET", path = "/api/plain/:n")]
fn plain(n int) int {
    return n
}
"#
        .to_string()
    }

    pub(super) fn start_server(code: &str, port: u16) -> u16 {
        crate::vm::ffi::stdlib::clear_http_routes();
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = code.to_string();
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let _ = crate::run(&code);
            })
            .expect("spawn server thread");
        for _ in 0..50 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        port
    }

    pub(super) fn get(port: u16, path: &str) -> (u16, Vec<(String, String)>, Vec<u8>) {
        let mut stream = None;
        for _ in 0..60 {
            if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                stream = Some(s);
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let mut stream = stream.expect("connect");
        stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
        let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
        stream.write_all(req.as_bytes()).unwrap();
        let _ = stream.flush();
        let mut raw = Vec::new();
        let mut buf = [0u8; 65536];
        let header_end = loop {
            let n = stream.read(&mut buf).unwrap_or(0);
            if n == 0 {
                break None;
            }
            raw.extend_from_slice(&buf[..n]);
            if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                break Some(pos + 4);
            }
        };
        let header_end = header_end.expect("response headers");
        let head = String::from_utf8_lossy(&raw[..header_end]).into_owned();
        let mut lines = head.split("\r\n");
        let status: u16 = lines
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let mut hdrs = Vec::new();
        for line in lines {
            if let Some((k, v)) = line.split_once(':') {
                hdrs.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
            }
        }
        let cl: usize = hdrs
            .iter()
            .find(|(k, _)| k == "content-length")
            .and_then(|(_, v)| v.parse().ok())
            .unwrap_or(0);
        let mut resp = raw[header_end..].to_vec();
        while resp.len() < cl {
            let n = stream.read(&mut buf).unwrap_or(0);
            if n == 0 {
                break;
            }
            resp.extend_from_slice(&buf[..n]);
        }
        resp.truncate(cl);
        (status, hdrs, resp)
    }

    /// AC-01 真碰撞：活跃 Response 资源的真实同值普通 int → 200 JSON 数字；
    /// 资源不被消费，随后仍按原协议回复（302）。实际 id 经观察钩子取得
    /// （非任意常量冒充）。
    #[test]
    fn http_e2e_plan776_collision_resource_vs_plain_int() {
        let port = start_server(&program(), PORT);
        // 1) 构造真实资源并返回 → 302 + Location。
        let (status, headers, _) = get(port, "/api/make");
        assert_eq!(status, 302, "资源首次返回按原协议回复");
        let location = headers
            .iter()
            .find(|(k, _)| k == "location")
            .map(|(_, v)| v.clone())
            .expect("Location header");
        assert_eq!(location, "/target");

        // 2) 观察钩子取活跃资源的实际 heap id，构造同值普通 int。
        let handle = crate::vm::ffi::http_return_resource::probe_last_resource_handle();
        assert!(handle > 0, "观察钩子应捕获构造");
        let (s2, _, body2) = get(port, &format!("/api/plain/{handle}"));
        assert_eq!(s2, 200, "同值普通 int 是数据（不得被劫持为资源）");
        assert_eq!(body2, handle.to_string().into_bytes());

        // 3) 资源未被消费：重放全局常量 → 仍 302（快照回复，可重复返回）。
        let (s3, headers3, _) = get(port, "/api/replay");
        assert_eq!(s3, 302, "资源未被 plain 请求消费");
        let location3 = headers3
            .iter()
            .find(|(k, _)| k == "location")
            .map(|(_, v)| v.clone())
            .expect("Location on replay");
        assert_eq!(location3, "/target");
    }

    /// AC-06 wire 面：全局常量资源连续多次请求正确（快照 clone，不首回消费）。
    #[test]
    fn http_e2e_plan776_response_constant_replays_across_requests() {
        let port = start_server(&program(), PORT + 1);
        for i in 0..3 {
            let path = if i == 0 { "/api/make" } else { "/api/replay" };
            let (status, headers, _) = get(port, path);
            assert_eq!(status, 302, "第 {} 次请求", i + 1);
            assert!(headers
                .iter()
                .any(|(k, v)| k == "location" && v == "/target"));
        }
    }
}

#[test]
fn plan776_diag_redirect_shape() {
    // 调用形态对照：CALL_NAT 下划线形态（真实 wire 契约）可执行；VM 退出后
    // 资源随 owner 回收（live 归零——AC-06 生命期的进程级观察）。
    let code = r#"
fn main() {
    let r = http.response_redirect("/t", 302)
    print("done")
}
"#;
    let (stderr, stdout) = crate::run_with_capture(code).expect("run");
    assert!(
        stdout.contains("done"),
        "stdout: {stdout:?} stderr: {stderr:?}"
    );
    assert_eq!(
        http_return_resource::live_resource_count(),
        0,
        "VM 退出后资源随 owner 回收归零"
    );
}

#[cfg(feature = "test-http-e2e")]
mod http_e2e_t04 {
    use super::http_e2e::{get, start_server};

    fn program() -> String {
        r#"
#[api(method = "GET", path = "/api/async-redirect")]
fn async_redirect() ~int {
    delay_async(30)
    return http.response_redirect("/async-target", 302)
}

#[api(method = "GET", path = "/api/async-big")]
fn async_big() ~int {
    delay_async(30)
    return 5000000000
}

#[api(method = "GET", path = "/api/async-fail")]
fn async_fail() ~int {
    delay_async(30)
    var d = 0
    var boom = 10 / d
    return boom
}
"#
        .to_string()
    }

    /// AC-03：Internal async body 成功终值携带资源凭据（VmRef 桥保身份）
    /// → 与 sync 同一编组（307 + Location）。
    #[test]
    fn http_e2e_plan776_async_terminal_resource_reply() {
        let port = start_server(&program(), 19020);
        let (status, headers, _) = get(port, "/api/async-redirect");
        assert_eq!(status, 302, "async 资源终值按原协议回复");
        let loc = headers
            .iter()
            .find(|(k, _)| k == "location")
            .map(|(_, v)| v.clone())
            .expect("Location");
        assert_eq!(loc, "/async-target");
    }

    /// AC-03：async 终值 i64 全宽（超出 i32 域）不截断不降级 null。
    #[test]
    fn http_e2e_plan776_async_terminal_i64_full_width() {
        let port = start_server(&program(), 19021);
        let (status, _, body) = get(port, "/api/async-big");
        assert_eq!(status, 200);
        assert_eq!(body, b"5000000000".to_vec(), "i64 全宽终值");
    }

    /// AC-03：async 终值复杂 JSON（record）与 sync 同形——VmRef 物化 +
    /// nv_to_json GenericInstance 序列化（不降级 null）。
    #[test]
    fn http_e2e_plan776_async_terminal_record_json() {
        let port = start_server(
            r#"
type Note = { id: int, title: str }

#[api(method = "GET", path = "/api/async-note")]
fn async_note() ~Note {
    delay_async(30)
    return Note { id: 7, title: "plan776" }
}
"#,
            19023,
        );
        let (status, _, body) = get(port, "/api/async-note");
        assert_eq!(status, 200);
        let text = String::from_utf8_lossy(&body).into_owned();
        assert!(text.contains("\"id\": 7"), "record 字段: {text}");
        assert!(
            text.contains("\"title\": \"plan776\""),
            "record 串字段: {text}"
        );
    }

    /// AC-04：Failed 终态 → 有界 500（不再伪装 200 null）。
    #[test]
    fn http_e2e_plan776_async_failed_is_500_not_null() {
        let port = start_server(&program(), 19022);
        let (status, _, body) = get(port, "/api/async-fail");
        assert_eq!(status, 500, "Failed 终态必须有界 500");
        let text = String::from_utf8_lossy(&body).into_owned();
        assert!(text.contains("error"), "失败体含 error 诊断: {text}");
    }
}

#[cfg(feature = "test-http-e2e")]
mod http_e2e_t05 {
    use super::http_e2e::start_server;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    /// AC-06：临时资源 1000 次构造-返回闭环后，实际存活凭据数回到预热基线
    ///（观察钩子计数，非 RSS）。dying 宽限窗 8192 解释步由 drain 请求跨过。
    #[test]
    fn http_e2e_plan776_thousand_loop_lifecycle_returns_to_baseline() {
        let port = start_server(
            r#"
#[api(method = "GET", path = "/api/make")]
fn make() int {
    return http.response_redirect("/target", 302)
}

#[api(method = "GET", path = "/api/drain")]
fn drain() int {
    var i = 0
    while i < 300 {
        i = i + 1
    }
    return i
}
"#,
            19030,
        );
        let baseline = crate::vm::ffi::http_return_resource::live_resource_count();
        let mut ok = 0u32;
        for _ in 0..1000 {
            let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
            s.set_read_timeout(Some(Duration::from_secs(30))).ok();
            let _ = s.write_all(b"GET /api/make HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n");
            let mut buf = [0u8; 4096];
            let mut raw = Vec::new();
            loop {
                let n = s.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break;
                }
                raw.extend_from_slice(&buf[..n]);
                if raw.windows(4).position(|w| w == b"\r\n\r\n").is_some()
                    && raw.starts_with(b"HTTP/1.1 302")
                {
                    break;
                }
            }
            if raw.starts_with(b"HTTP/1.1 302") {
                ok += 1;
            }
        }
        assert_eq!(ok, 1000, "每次闭环都正确回复 302");
        // dying 宽限窗（8192 解释步）：drain 请求推进解释步，有界轮询到归零
        //（每请求解释步数有限，需多轮跨窗；10s 上界）。
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            if crate::vm::ffi::http_return_resource::live_resource_count() <= baseline {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "1000 闭环后凭据未回基线: live={}",
                crate::vm::ffi::http_return_resource::live_resource_count()
            );
            let mut s = TcpStream::connect(("127.0.0.1", port)).expect("drain connect");
            let _ = s.write_all(b"GET /api/drain HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n");
            let mut buf = [0u8; 512];
            let _ = s.read(&mut buf);
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// AC-06：Sse 凭据（Sse.new 链资源路径）在下游断连后回收——长流读首帧
    /// 即断开，iterator 由 cleanup 守卫收口，凭据随引用归零回收。
    #[test]
    fn http_e2e_plan776_sse_credential_released_on_disconnect() {
        let port = start_server(
            r#"
dep axum
use.rs axum::response::sse::{Event, KeepAlive, Sse}

#[api(method = "GET", path = "/api/sse")]
fn chain_handler() int {
    var sse = Sse.new(events_stream())
    return sse.keep_alive(KeepAlive.new()).into_response()
}

#[api(method = "GET", path = "/api/drain")]
fn drain() int {
    var i = 0
    while i < 300 {
        i = i + 1
    }
    return i
}

fn events_stream() ~Iter<int> {
    var i = 0
    while i < 50000 {
        yield sse_named_event("tick", "42")
        i = i + 1
    }
}
"#,
            19031,
        );
        let baseline = crate::vm::ffi::http_return_resource::live_resource_count();
        // 读到第一帧 data: 后立即断开。
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        s.set_read_timeout(Some(Duration::from_secs(30))).ok();
        let _ = s.write_all(b"GET /api/sse HTTP/1.1\r\nHost: x\r\n\r\n");
        let mut raw = Vec::new();
        let mut buf = [0u8; 4096];
        let got_frame = loop {
            let n = s.read(&mut buf).unwrap_or(0);
            if n == 0 {
                break false;
            }
            raw.extend_from_slice(&buf[..n]);
            if raw.windows(6).any(|w| w == b"event:") || raw.windows(5).any(|w| w == b"data:") {
                break true;
            }
        };
        assert!(got_frame, "应收到首帧（headers+data）");
        drop(s); // 断连
                 // 断连传播 + 清理 + 宽限窗：轮询等待凭据回收（有界 10s）。
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let mut drained = 0;
        loop {
            if crate::vm::ffi::http_return_resource::live_resource_count() <= baseline {
                break;
            }
            if std::time::Instant::now() > deadline {
                panic!(
                    "断连后凭据未回收: live={} baseline={}",
                    crate::vm::ffi::http_return_resource::live_resource_count(),
                    baseline
                );
            }
            // 推进解释步跨 dying 宽限窗（drain 端点不构造凭据）。
            let mut d = TcpStream::connect(("127.0.0.1", port)).expect("drain");
            let _ = d.write_all(b"GET /api/drain HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n");
            let mut db = [0u8; 512];
            let _ = d.read(&mut db);
            drop(d);
            drained += 1;
            let _ = drained;
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

/// T-05：rc 释放链路单元级——push(rc=1) → release 归零入 dying →
/// reap_all 全收（静止点语义）；live 计数随 Drop 回落。
#[test]
fn plan776_t05_rc_release_chain_reaps_credential() {
    let vm = fresh_vm();
    let mut task = fresh_task();
    http_return_resource::push_response_resource(&mut task, &vm, 200, Vec::new(), b"x".to_vec())
        .unwrap();
    let nv = task.ram.pop_nv();
    let id = auto_val::decode_object(nv) as u64;
    let c1 = vm
        .heap_rc
        .get(&id)
        .map(|e| e.load(std::sync::atomic::Ordering::Acquire));
    vm.rc_release(nv);
    let c2 = vm
        .heap_rc
        .get(&id)
        .map(|e| e.load(std::sync::atomic::Ordering::Acquire));
    let dying = vm.dying_heap.lock().map(|q| q.len()).unwrap_or(usize::MAX);
    assert_eq!(c1, Some(1), "构造后唯一栈引用");
    assert_eq!(c2, None, "release 归零（条目移除）");
    assert_eq!(dying, 1, "归零入 dying 宽限队列");
    let before = http_return_resource::live_resource_count();
    vm.reap_all();
    assert_eq!(
        http_return_resource::live_resource_count(),
        before - 1,
        "reap_all（静止点）全收凭据"
    );
}

// ============================================================================
// T-06：解析来源身份（提取期分类 + VM 门遮蔽）
// ============================================================================

/// AC-07：同裸名用户类型（`Response`/`FileResponse` 被用户声明）按真实来源
/// 分类为 Json；未声明的裸名保持 stdlib 专属身份；enum 声明同样遮蔽。
#[test]
fn plan776_t06_extract_source_identity_shadowing() {
    let code = r#"
type Response {
    code int
}

type FileResponse {
    name str
}

enum Iter {
    A,
    B,
}

#[api(method = "GET", path = "/api/user-response")]
fn user_response() Response {
    return Response { code: 1 }
}

#[api(method = "GET", path = "/api/user-file")]
fn user_file() FileResponse {
    return FileResponse { name: "x" }
}

#[api(method = "GET", path = "/api/user-iter")]
fn user_iter() Iter {
    return Iter::A
}

#[api(method = "GET", path = "/api/real-upload")]
fn real_upload() UploadReceipt {
    return upload_error(400, "no")
}
"#;
    let mut parser = crate::parser::Parser::from(code);
    let ast = parser.parse().expect("parse");
    let module = crate::api::ApiExtractor::new().extract("test", &ast.stmts);
    let kind_of = |name: &str| {
        module
            .endpoints
            .iter()
            .find(|e| e.fn_name == name)
            .map(|e| e.response)
            .unwrap_or_else(|| panic!("endpoint {name} 缺失"))
    };
    use crate::api::contract::ResponseKind;
    // 用户同名声明（struct×2 + enum）→ Json（不冒领专属身份）。
    assert_eq!(kind_of("user_response"), ResponseKind::Json);
    assert_eq!(kind_of("user_file"), ResponseKind::Json);
    assert_eq!(kind_of("user_iter"), ResponseKind::Json);
    // 未遮蔽的裸名 → stdlib 内建身份保持。
    assert_eq!(kind_of("real_upload"), ResponseKind::Upload);
}

/// AC-07：VM 编组门的编译期遮蔽——登记 shadowed 后 File 声明不再开门
///（用户同名 FileResponse 返回按 Json 编组，不误消费描述符表）。
#[test]
fn plan776_t06_vm_gate_respects_compile_time_shadowing() {
    use crate::api::contract::ResponseKind;
    let name = "plan776_t06_shadowed_fn";
    crate::vm::ffi::http_server::record_api_return_type(name, "FileResponse".into());
    assert_eq!(
        crate::vm::ffi::http_server::fn_response_kind(name),
        ResponseKind::File,
        "未遮蔽：File 门保持"
    );
    crate::vm::ffi::http_server::record_api_return_shadowed(name);
    assert_eq!(
        crate::vm::ffi::http_server::fn_response_kind(name),
        ResponseKind::Json,
        "编译期冻结的用户同名遮蔽 → Json"
    );
    assert!(!crate::vm::ffi::http_server::fn_is_api_file_return(name));
}
