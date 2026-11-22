//! PLAN-108（T-03b M1）：UI 异步 HTTP **源级原语目录**（`http.client/1`）。
//!
//! 背景：组件面（AGE component/package/h0 profile）此前无任何模块级
//! receiver——`Http.post_json(url, body)` 在 resolve 层即 `source.undefined:
//! no local named Http in scope`（VM 管线的 Http 只在 auto-lang-core ffi，
//! 不经本前门）。本表是组件面 Http 效应族的**单一真源**：源级 callee →
//! 固定符号 → 精确参数 VT/结果 VT（与 config_ops 同纪律，签名一处声明，
//! resolve 选重载、adapter 发射签名、av-core admission 核符号三处共用）。
//!
//! 语义锚：VM shim（auto-lang-core `vm/ffi/stdlib/http_client.rs`）——
//! `post_json(url, body)` 返回 **body 文本**（HTTP 响应体字符串），故组件
//! 面 result=StrOwned。这是最小链：M1 只准编译（执行 fail-closed，无
//! provider 注册=既有 938 不变式）；M2 由 UI 桥注册阻塞式 foreign stub。
//!
//! 闭合族与显式差额：`get_msg`（消息桥，需 http_msg_subscription 机械）
//! 不在本表——具名拒绝，另案；`delete` 无体单参；`get/get_json` 单参。

use super::resolve::VT;

/// HTTP 客户端能力键（进入模块 requires / 运行时 per-run grant 核）。
pub const HTTP_CAPABILITY: &str = "http.client/1";

/// 一个源级 Http 原语。
#[derive(Clone, Copy, Debug)]
pub struct HttpSourceOp {
    /// 源级限定 callee（`Http.post_json` 形，含 receiver）。
    pub callee: &'static str,
    /// 固定 host 符号（`auto.http.<verb>.v1`）。
    pub symbol: &'static str,
    pub params: &'static [VT],
    pub result: VT,
}

const P_URL: &[VT] = &[VT::Str];
const P_URL_BODY: &[VT] = &[VT::Str, VT::Str];

/// Http 原语目录（闭合族；签名与 VM shim 同语义）。
pub const HTTP_SOURCE_OPS: &[HttpSourceOp] = &[
    HttpSourceOp {
        callee: "Http.get",
        symbol: "auto.http.get.v1",
        params: P_URL,
        result: VT::StrOwned,
    },
    HttpSourceOp {
        callee: "Http.get_json",
        symbol: "auto.http.get-json.v1",
        params: P_URL,
        result: VT::StrOwned,
    },
    HttpSourceOp {
        callee: "Http.post",
        symbol: "auto.http.post.v1",
        params: P_URL_BODY,
        result: VT::StrOwned,
    },
    HttpSourceOp {
        callee: "Http.post_json",
        symbol: "auto.http.post-json.v1",
        params: P_URL_BODY,
        result: VT::StrOwned,
    },
    HttpSourceOp {
        callee: "Http.put",
        symbol: "auto.http.put.v1",
        params: P_URL_BODY,
        result: VT::StrOwned,
    },
    HttpSourceOp {
        callee: "Http.put_json",
        symbol: "auto.http.put-json.v1",
        params: P_URL_BODY,
        result: VT::StrOwned,
    },
    HttpSourceOp {
        callee: "Http.patch",
        symbol: "auto.http.patch.v1",
        params: P_URL_BODY,
        result: VT::StrOwned,
    },
    HttpSourceOp {
        callee: "Http.patch_json",
        symbol: "auto.http.patch-json.v1",
        params: P_URL_BODY,
        result: VT::StrOwned,
    },
    HttpSourceOp {
        callee: "Http.delete",
        symbol: "auto.http.delete.v1",
        params: P_URL,
        result: VT::StrOwned,
    },
];

/// receiver 名 + 方法名 → 目录 callee 拼写。
pub fn dotted(receiver: &str, verb: &str) -> String {
    format!("{receiver}.{verb}")
}

/// 该拼写（`Http.<verb>` 全拼写）是否在目录内。
pub fn is_callee(name: &str) -> bool {
    HTTP_SOURCE_OPS.iter().any(|o| o.callee == name)
}

fn vt_admits(expected: VT, actual: VT) -> bool {
    // 文本参数接 `str`/`Str` 两种文本值（借/自有同一语义）。
    match expected {
        VT::Str => matches!(actual, VT::Str | VT::StrOwned),
        other => expected == actual,
    }
}

/// 按 callee 全拼写 + 实参 VT 选重载（无匹配 = None，调用方给具名错误）。
pub fn lookup_callee(name: &str, args: &[VT]) -> Option<&'static HttpSourceOp> {
    HTTP_SOURCE_OPS.iter().find(|o| {
        o.callee == name
            && o.params.len() == args.len()
            && o
                .params
                .iter()
                .zip(args.iter())
                .all(|(p, a)| vt_admits(*p, *a))
    })
}

/// 符号 → 原语（lowering/admission/运行期按符号查）。
pub fn by_symbol(sym: &str) -> Option<&'static HttpSourceOp> {
    HTTP_SOURCE_OPS.iter().find(|o| o.symbol == sym)
}

/// 符号的能力键（未知符号 = None）。
pub fn capability_of(sym: &str) -> Option<&'static str> {
    by_symbol(sym).map(|_| HTTP_CAPABILITY)
}
