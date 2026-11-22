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

use ace_av::component_event_sessions::{ComponentEventSessionProgram, EventOutcome};
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

/// M2 正例：注入宿主 provider 后 Http.post_json 真实执行（同步语义——
/// 阻塞由调用侧承担；UI 桥的 worker 线程派发是消费侧责任），结果体
/// 文本落账 model 字段；provider 缺席仍 fail-closed（M1 不变式保持）。
struct FakeHttp {
    seen: std::sync::Arc<std::sync::Mutex<Vec<(String, String, Option<String>)>>>,
    /// job → 完成值（None=在途；测试注入）。
    results: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u64, Option<Result<String, String>>>>>,
    next_job: std::sync::Mutex<u64>,
}

impl ace_av::component_event_sessions::HttpProvider for FakeHttp {
    fn http_start(&mut self, verb: &str, url: &str, body: Option<&str>) -> u64 {
        self.seen
            .lock()
            .unwrap()
            .push((verb.to_string(), url.to_string(), body.map(str::to_string)));
        let mut next = self.next_job.lock().unwrap();
        let job = *next;
        *next += 1;
        self.results.lock().unwrap().insert(job, None);
        job
    }

    fn http_poll(&mut self, job: u64) -> Option<Result<String, String>> {
        self.results.lock().unwrap().get(&job)?.clone()
    }
}

#[test]
fn p108_http_provider_executes_and_lands() {
    let program = ComponentEventSessionProgram::compile(
        HTTP_PROBE_SRC,
        "plan108.http.probe",
        LegacyWidgetAlias::Accept,
    )
    .expect("M2 正例编译");
    let mut session = program.open_session(&[]).expect("open_session");
    // 无 provider：仍具名 fail-closed（M1 不变式）。
    let err = session.dispatch("Load", &[]).expect_err("无 provider 须 fail-closed");
    assert!(err.contains("http.client"), "实际: {err}");
    // 注入 provider：start 挂起 → 完成注入 → resume 落账。
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let results = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
    let provider = std::sync::Arc::new(std::sync::Mutex::new(FakeHttp {
        seen: seen.clone(),
        results: results.clone(),
        next_job: std::sync::Mutex::new(1),
    }));
    session.set_http_provider(provider);
    let (outcome, state) = session
        .dispatch_state("Load", &[])
        .expect("Load 派发");
    assert_eq!(outcome, EventOutcome::Dispatched);
    assert_eq!(
        state,
        ace_av::component_event_sessions::DispatchState::SuspendedHttp,
        "job 未完成前须挂起"
    );
    // 挂起期间同会话其它派发照常（提交在体尾——M3 交互性）。
    session.dispatch("Bump", &[]).expect("Bump 挂起期照常");
    // 挂起期 Bump 的提交即时可见（不等待 resume）。
    let during = session.read_states().unwrap();
    let during_bumps = during
        .iter()
        .find(|(n, _)| n == "bumps")
        .map(|(_, v)| format!("{v:?}"))
        .unwrap_or_default();
    assert_eq!(during_bumps, "Int(1)", "挂起期 Bump 须即时提交，实际: {during_bumps}");
    // job 完成注入 → resume 落账。
    results
        .lock()
        .unwrap()
        .insert(1, Some(Ok("{\"ok\":true,\"slow\":65,fake-body}".to_string())));
    let states = session
        .resume_http()
        .expect("resume 应有结果")
        .expect("resume 应成功");
    let result = states
        .iter()
        .find(|(n, _)| n == "result")
        .map(|(_, v)| format!("{v:?}"))
        .expect("result 字段在案");
    assert!(
        result.contains("fake-body"),
        "result 须落账响应体，实际: {result}"
    );
    let bumps = states
        .iter()
        .find(|(n, _)| n == "bumps")
        .map(|(_, v)| format!("{v:?}"))
        .unwrap_or_default();
    assert_eq!(bumps, "Int(1)", "挂起期的 Bump 须保留（保留-重播种），实际: {bumps}");
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1, "provider 恰一调（重入不双发）");
    assert_eq!(seen[0].0, "auto.http.post-json.v1");
    assert_eq!(seen[0].1, "http://127.0.0.1:7072/slow");
    assert_eq!(seen[0].2.as_deref(), Some("q=1"));
}

/// M3 取消/卸载见证：挂起 → 会话关闭（卸载）→ job 完成 → resume 具名拒写，
/// 状态无任何迟到落账。
#[test]
fn p108_cancel_unload_late_result_isolated() {
    let program = ComponentEventSessionProgram::compile(
        HTTP_PROBE_SRC,
        "plan108.http.probe",
        LegacyWidgetAlias::Accept,
    )
    .expect("编译");
    let mut session = program.open_session(&[]).expect("open_session");
    let results = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
    let provider = std::sync::Arc::new(std::sync::Mutex::new(FakeHttp {
        seen: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        results: results.clone(),
        next_job: std::sync::Mutex::new(1),
    }));
    session.set_http_provider(provider);
    let (_, state) = session.dispatch_state("Load", &[]).expect("Load 派发");
    assert_eq!(
        state,
        ace_av::component_event_sessions::DispatchState::SuspendedHttp
    );
    // 卸载：会话关闭（in-flight handler 随会话死）。
    session.close();
    // 迟到完成到达——resume 须具名拒写，不 panic、不落账。
    results
        .lock()
        .unwrap()
        .insert(1, Some(Ok("{\"ok\":true,late}".to_string())));
    let err = session
        .resume_http()
        .expect("resume 应有裁决")
        .expect_err("关闭会话的 resume 必须拒写");
    assert!(
        err.contains("late result isolated"),
        "须为迟到隔离具名拒，实际: {err}"
    );
}
