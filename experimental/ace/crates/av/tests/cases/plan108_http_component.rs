//! PLAN-108（T-03b M1）：组件面 Http 效应族（`http.client/1`）准入固定用例。
//!
//! oracle：① 含 `Http.post_json(url, body)` 的 handler 经组件面前门
//! `ComponentEventSessionProgram::compile` **编译通过**（resolve 准入 →
//! lowering host import `auto.http.post-json.v1` → verify；M1 不给执行
//! provider）；② 执行 fail-closed——派发 Load 返回具名 foreign 拒绝
//! （938 不变式：table-less = `foreign.unsupported`），不静默、不挂起；
//! ③ 负例：未知 verb / 参数不匹配 = 具名源诊断（`source.http-*`）；
//! ④ 用户遮蔽 `Http` 名时不抢（回落普通解析）。
//!
//! 语义锚（VM shim 同契）：post_json 返回响应体文本（str）。

use ace_av::component_event_sessions::ComponentEventSessionProgram;
use ace_core::source::age_h0::LegacyWidgetAlias;

/// 032 形探针（node 声明 + 65s 慢端点 handler 族闭合面最小切片）。
const HTTP_PROBE_SRC: &str = "node Probe {\n    msg Msg { Load, Bump, Tick }\n    model {\n        result str = \"(idle)\"\n        bumps int = 0\n    }\n    on {\n        .Load -> {\n            var resp = Http.post_json(\"http://127.0.0.1:7072/slow\", \"q=1\")\n            .result = resp\n        }\n        .Bump -> {\n            .bumps = .bumps + 1\n        }\n    }\n    on Tick {\n        bumps = bumps + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

fn compile(src: &str) -> Result<(), String> {
    ComponentEventSessionProgram::compile(src, "plan108.http.probe", LegacyWidgetAlias::Accept)
        .map(|_| ())
        .map_err(|e| e)
}

/// 正例：Http.post_json 过编译前门（M1 准入）；执行 fail-closed。
#[test]
fn p108_http_post_json_admitted_and_fail_closed() {
    compile(HTTP_PROBE_SRC).unwrap_or_else(|e| panic!("M1 准入应过编译: {e}"));
    let program = ComponentEventSessionProgram::compile(
        HTTP_PROBE_SRC,
        "plan108.http.probe",
        LegacyWidgetAlias::Accept,
    )
    .unwrap_or_else(|e| panic!("recompile: {e}"));
    let mut session = program
        .open_session(&[])
        .unwrap_or_else(|e| panic!("open_session: {e}"));
    // M1 无 provider：派发须具名拒绝（不挂起、不静默成功）。
    let err = session
        .dispatch("Load", &[])
        .expect_err("无 http provider 时执行必须 fail-closed");
    assert!(
        err.contains("http.client") && err.contains("fail-closed"),
        "fail-closed 须为 http.client 具名拒绝，实际: {err}"
    );
    // Bump/Tick 等纯状态 handler 不受准入影响（兄弟保全）。
    session.dispatch("Bump", &[]).expect("Bump 正常派发");
}

/// 负例 1：闭合族外 verb 具名拒绝。
#[test]
fn p108_unknown_verb_named_rejection() {
    let src = HTTP_PROBE_SRC.replace("Http.post_json", "Http.teleport");
    let err = compile(&src).expect_err("未知 verb 必须拒");
    assert!(
        err.contains("source.http-unsupported"),
        "须为 source.http-unsupported，实际: {err}"
    );
}

/// 负例 2：参数个数不匹配具名拒绝（post_json 须两参）。
#[test]
fn p108_arity_mismatch_named_rejection() {
    let src = HTTP_PROBE_SRC.replace(
        "Http.post_json(\"http://127.0.0.1:7072/slow\", \"q=1\")",
        "Http.post_json(\"http://127.0.0.1:7072/slow\")",
    );
    let err = compile(&src).expect_err("一参 post_json 必须拒");
    assert!(
        err.contains("source.http-argument"),
        "须为 source.http-argument，实际: {err}"
    );
}

/// 负例 3：参数类型不匹配具名拒绝（int 实参不接）。
#[test]
fn p108_type_mismatch_named_rejection() {
    let src = HTTP_PROBE_SRC.replace(
        "Http.post_json(\"http://127.0.0.1:7072/slow\", \"q=1\")",
        "Http.post_json(7072, \"q=1\")",
    );
    let err = compile(&src).expect_err("int URL 必须拒");
    assert!(
        err.contains("source.http-argument"),
        "须为 source.http-argument，实际: {err}"
    );
}

/// 遮蔽让位：局部 `Http` 存在时Http效应拦截不抢名（回落普通解析——
/// 普通解析对局部 receiver 走方法面，无此方法时给非 http 族诊断）。
#[test]
fn p108_user_shadow_wins() {
    let src = "node Shade {\n    model {\n        out str = \"init\"\n    }\n    on Go {\n        var Http = 1\n        var r = Http.post_json(\"u\", \"b\")\n        out = r\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let err = compile(src).expect_err("遮蔽形不得走 http 效应准入");
    assert!(
        !err.contains("source.http-"),
        "遮蔽形须回落普通解析（非 http 族诊断），实际: {err}"
    );
}
