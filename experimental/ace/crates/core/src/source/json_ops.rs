//! PLAN-117（B-05 S1a）：checked 域 **JSON 值语义**源级原语目录（`json.value/1`）。
//!
//! 背景：029/031 的原源在 handler 里把 `json.parse(...)` / `Http.get_json(url)`
//! 的结果当**结构化值**用（`.entries` 成员读、`?? []` 兜底、`!= null` 判空）。
//! PLAN-964（Object/Nil）/PLAN-824（Dyn）已交付运行时值域与指令面——本表只把
//! AGE 执行面的**语言皮**接上：`json.parse(str)` → `VT::Dyn`（动态值；成员读
//! 经既有 Dyn-object 机制落到 ObjectGet，missing 的 JSON 语义 = Nil）。
//!
//! 与 http_ops 同纪律：源级 callee → 固定符号 → 精确参数 VT/结果 VT，签名一处
//! 声明，resolve 选重载、adapter 发射签名、av-core admission 核符号三处共用。
//! S1a 里程碑形态 = 只准编译（执行 fail-closed：无 provider 注册 = 既有 938
//! 不变式）；S1b 由 av-core 执行器交付解析。
//!
//! capability：`json.value/1`（进入 requires；运行时 per-run grant 核——与
//! http.client/1 同款，bridge/编译面按 capability 参数显式授予）。

use super::resolve::VT;

/// JSON 值域能力键。
pub const JSON_CAPABILITY: &str = "json.value/1";

/// 一个源级 JSON 值原语。
#[derive(Clone, Copy, Debug)]
pub struct JsonSourceOp {
    /// 源级限定 callee（`json.parse` 形）。
    pub callee: &'static str,
    /// 固定 host 符号（`auto.json.<verb>.v1`）。
    pub symbol: &'static str,
    pub params: &'static [VT],
    pub result: VT,
}

const P_TEXT: &[VT] = &[VT::Str];
/// PLAN-117（round-23，029 真语料④）：`json.stringify(value)` 的参数是
/// json 值域任意值（Dyn 承载；Object/List/标量在调用点 coerce）。
const P_VALUE: &[VT] = &[VT::Dyn];

/// A typed AGE binding can materialize a finite JSON number at f32 precision.
pub const JSON_NUMBER_F32: JsonSourceOp = JsonSourceOp {
    callee: "",
    symbol: "auto.json.number-f32.v1",
    params: P_VALUE,
    result: VT::F32,
};

/// json 值原语目录（闭合族：parse + stringify）。
pub const JSON_SOURCE_OPS: &[JsonSourceOp] = &[
    JsonSourceOp {
        callee: "json.parse",
        symbol: "auto.json.parse.v1",
        params: P_TEXT,
        result: VT::Dyn,
    },
    JsonSourceOp {
        callee: "json.stringify",
        symbol: "auto.json.stringify.v1",
        params: P_VALUE,
        result: VT::StrOwned,
    },
    // PLAN-117：VM 方言别名 `json.encode`（029 真语料拼写）——与 stringify
    // 同一符号/签名。
    JsonSourceOp {
        callee: "json.encode",
        symbol: "auto.json.stringify.v1",
        params: P_VALUE,
        result: VT::StrOwned,
    },
];

/// receiver 名 + 方法名 → 目录 callee 拼写。
pub fn dotted(receiver: &str, verb: &str) -> String {
    format!("{receiver}.{verb}")
}

/// 该拼写（`json.<verb>` 全拼写）是否在目录内。
pub fn is_callee(name: &str) -> bool {
    JSON_SOURCE_OPS.iter().any(|o| o.callee == name)
}

fn vt_admits(expected: VT, actual: VT) -> bool {
    // 文本参数接 `str`/`Str` 两种文本值（借/自有同一语义）。
    match expected {
        VT::Str => matches!(actual, VT::Str | VT::StrOwned),
        other => expected == actual,
    }
}

/// 按 callee 全拼写 + 实参 VT 选重载（无匹配 = None，调用方给具名错误）。
pub fn lookup_callee(name: &str, args: &[VT]) -> Option<&'static JsonSourceOp> {
    JSON_SOURCE_OPS.iter().find(|o| {
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
pub fn by_symbol(sym: &str) -> Option<&'static JsonSourceOp> {
    if sym == JSON_NUMBER_F32.symbol {
        Some(&JSON_NUMBER_F32)
    } else {
        JSON_SOURCE_OPS.iter().find(|o| o.symbol == sym)
    }
}

/// 符号的能力键（未知符号 = None）。
pub fn capability_of(sym: &str) -> Option<&'static str> {
    by_symbol(sym).map(|_| JSON_CAPABILITY)
}
