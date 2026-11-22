//! PLAN-108（T-03b M1）：组件面 UI 异步 HTTP 客户端原语 admission
//! （`http.client/1`）。
//!
//! 与 `platform_io`/`config_document` 同纪律：av-core 自持一份**符号目录**
//! （ace-core `http_ops` 的跨层同契约镜像——签名真源在源面表，本表只管
//! symbol→capability/max_call_bytes 的 admission 事实）。M1 只准编译：
//! 执行侧无 provider 注册时 `ForeignCall` 按既有 938 不变式 fail-closed
//! （table-less = `foreign.unsupported` Infra）；M2 由 UI 桥注册阻塞式
//! foreign stub（worker 线程派发，桥级异步——见 PLAN-108 §9 T-03a'）。
//!
//! 闭合族与 ace-core `http_ops` 一致；`get_msg` 不在族（另案）。

use crate::host::AdmissionError;
use crate::mir::MirHostImport;

pub const CATEGORY_HTTP: &str = "http.client/1";
/// per-run grant 键（execute_with_grants 系按 import.capability 核）。
pub const CAP_HTTP: &str = "http.client";

/// 响应体/请求体的调用字节预算上界（1 MiB；M1 只记 admission 事实）。
pub const MAX_CALL_BYTES: usize = 1 << 20;

struct HttpOpSpec {
    symbol: &'static str,
    category: &'static str,
    capability: &'static str,
    max_call_bytes: u32,
}

const fn spec(symbol: &'static str) -> HttpOpSpec {
    HttpOpSpec {
        symbol,
        category: CATEGORY_HTTP,
        capability: CAP_HTTP,
        max_call_bytes: MAX_CALL_BYTES as u32,
    }
}

/// Http 原语符号目录（与 ace-core `http_ops::HTTP_SOURCE_OPS` 同族同拼写）。
pub const HTTP_OP_SPECS: &[HttpOpSpec] = &[
    spec("auto.http.get.v1"),
    spec("auto.http.get-json.v1"),
    spec("auto.http.post.v1"),
    spec("auto.http.post-json.v1"),
    spec("auto.http.put.v1"),
    spec("auto.http.put-json.v1"),
    spec("auto.http.patch.v1"),
    spec("auto.http.patch-json.v1"),
    spec("auto.http.delete.v1"),
];

fn lookup_symbol(symbol: &str) -> Option<&'static HttpOpSpec> {
    HTTP_OP_SPECS.iter().find(|s| s.symbol == symbol)
}

/// 符号是否在 Http 目录内（lowering 前缀路由亦用）。
pub fn is_http_symbol(symbol: &str) -> bool {
    lookup_symbol(symbol).is_some()
}

/// host import admission：未知符号 = 具名拒绝；不隐式授权（grant 运行时核）。
pub fn declaration(symbol: &str, sig: u32) -> Result<MirHostImport, AdmissionError> {
    let spec = lookup_symbol(symbol).ok_or_else(|| AdmissionError {
        code: "host.unknown-symbol",
        message: "unknown http operation symbol".into(),
    })?;
    Ok(MirHostImport {
        category: spec.category.into(),
        symbol: spec.symbol.into(),
        sig,
        capability: spec.capability.into(),
        max_call_bytes: spec.max_call_bytes,
    })
}

/// PLAN-108（T-03c M3）：宿主 HTTP 执行器（job 式——执行器挂起语义）。
/// UI 桥实现：`start` 把真实请求派发上 worker 线程立即返回 job id；
/// `poll` 非阻塞查完成（None=在途）。执行器首过 start+挂起 `LoopState`，
/// 完成后经会话续跑（同一会话、提交序=完成序——handler 提交在体尾，挂起
/// 期间其它派发自然交错，对齐 PLAN-702 段化语义）。
pub trait HttpProvider: Send {
    /// 启动请求；返回 job id（桥侧单调分配）。
    fn http_start(&mut self, verb: &str, url: &str, body: Option<&str>) -> u64;
    /// 非阻塞完成查询：None=在途；Some=终值（Ok=响应体文本，VM shim 同语义）。
    fn http_poll(&mut self, job: u64) -> Option<Result<String, String>>;
}
