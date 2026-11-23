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

use ace_av::component_event_sessions::{ComponentEventSessionProgram, EventOutcome, HttpFailure};
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

    fn http_poll(&mut self, job: u64) -> Option<Result<String, HttpFailure>> {
        self.results.lock().unwrap().get(&job)?.clone()
            .map(|result| result.map_err(HttpFailure::Request))
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

/// PLAN-117（round-25 ⑤）：`Http.get_json` 组件面**按动词分语义**——结果 =
/// 解析后的 json 值（Dyn；029 真源把结果当结构化值用：成员读/`??`/判空），
/// 与 VM 方言 `get_json`+to_value 同语义；post 族保持响应体文本（032 串
/// 消费不变，既有 M2 用例继续钉住）。成员读 missing = Nil（JSON 语义），
/// `??` 兜底；解析失败 = 具名 Error（`http.client:` 前缀，非静默落账）。
/// PLAN-117（round-26）：view-data 集合初值物化 + for 条件里的 view-data 读
/// （029 真形 `for fi2 < .fav_records.len()` 的最小切片）——事件道端到端：
/// 物化 Let → 读改写 → `.len()` → 循环 → typed 字段落账。
#[test]
fn p117_view_data_list_read_in_for_cond_executes() {
    use ace_av::component_event_sessions::ComponentEventSessionProgram;
    use ace_av::age_component::LegacyWidgetAlias;
    let src = r#"widget App {
    msg { Init }
    model { var photos = ["a", "b"]; var n int = 0 }
    on { .Init -> {
        var i = 0
        for i < .photos.len() {
            i = i + 1
        }
        .n = i
    } }
    view { col { text "h" {} } }
}"#;
    let program = ComponentEventSessionProgram::compile(src, "t.at", LegacyWidgetAlias::Accept)
        .expect("view-data 集合读 for-cond 事件程序须编译");
    let mut session = program.open_session(&[]).expect("session opens");
    session.dispatch("Init", &[]).expect("Init dispatches");
    let states = session.read_states().expect("states");
    let n = states
        .iter()
        .find(|(name, _)| name == "n")
        .map(|(_, v)| format!("{v:?}"))
        .expect("n 字段在案");
    assert_eq!(n, "Int(2)", "for 条件的 view-data `.len()` 真实执行，实际: {n}");
}

/// PLAN-117（round-27）：场景车道（029 真窗路径）的 view-data 读物化——
/// try 体 + 嵌套 for 条件读（ReloadDir 真形切片）经 check-fn 物化编译通过。
#[test]
fn p117_scenario_lane_view_data_for_cond_compiles() {
    use ace_av::age_component::LegacyWidgetAlias;
    let src = r#"widget App {
    msg { Init }
    model { var photos = []; var load_error str = ""; var n int = 0 }
    on { .Init -> {
        try {
            var i = 0
            for i < .photos.len() {
                if .photos[i].id == "x" { i = i + 1 }
            }
            .load_error = ""
            .n = i
        } catch {
            .load_error = "scan failed"
        }
    } }
    view { col { text "h" {} } }
}"#;
    let scenario = ace_av::age_component::ComponentScenario { instances: 1, ops: Vec::new() };
    let initials = ace_core::source::age_component::ComponentInitials { per_instance: vec![Vec::new()] };
    let caps = ["ac.runtime.text.v1".to_string()];
    ace_core::source::age_component::compile_component_scenario_with_initials(
        src, "t.at", LegacyWidgetAlias::Accept, &caps, &scenario, None, &initials,
    )
    .expect("场景车道 view-data for-cond 读编译");
}

const GET_JSON_PROBE_SRC: &str = "widget App {\n    msg { Load }\n    model {\n        var photos = []\n        var flag bool = false\n        var label str = \"(none)\"\n    }\n    on {\n        .Load -> {\n            let data = Http.get_json(\"http://127.0.0.1:7072/api\")\n            .flag = data.ok ?? false\n            .label = data.name ?? \"fallback\"\n        }\n    }\n    view { text \"x\" {} }\n}\n";

#[test]
fn p117_initial_state_with_json_imports_needs_no_provider() {
    use ace_av::age_component::{
        observe_component_scenario_states, ComponentFieldValue, ComponentInitials, ComponentScenario,
    };

    let states = observe_component_scenario_states(
        GET_JSON_PROBE_SRC,
        "plan117.http.getjson",
        LegacyWidgetAlias::Accept,
        &ComponentScenario { instances: 1, ops: Vec::new() },
        &ComponentInitials { per_instance: vec![Vec::new()] },
    )
    .expect("initial-state observation with an uncalled HTTP/JSON handler");
    assert_eq!(states.per_instance[0], vec![
        ("flag".to_string(), ComponentFieldValue::Bool(false)),
        ("label".to_string(), ComponentFieldValue::Str("(none)".to_string())),
    ]);
}

#[test]
fn p117_component_event_calls_module_function() {
    use ace_av::age_component::ComponentFieldValue;

    let src = r#"
fn next(n int) int { return n + 2 }
widget App {
    msg { Step }
    model { var count int = 1 }
    on { .Step -> { .count = next(.count) } }
    view { text "counter" {} }
}
"#;
    let program =
        ComponentEventSessionProgram::compile(src, "plan117.helpers", LegacyWidgetAlias::Accept)
            .expect("event handler can resolve its module function");
    let mut session = program.open_session(&[]).expect("open session");
    let outcome = session.dispatch("Step", &[]).expect("dispatch Step");
    assert_eq!(outcome, EventOutcome::Dispatched);
    assert_eq!(session.read_state("count").unwrap(), Some(ComponentFieldValue::Int(3)));
}

#[test]
fn p117_get_json_component_face_returns_json_value() {
    let program = ComponentEventSessionProgram::compile(
        GET_JSON_PROBE_SRC,
        "plan117.http.getjson",
        LegacyWidgetAlias::Accept,
    )
    .expect("get_json 混合 http+json host import 须过编译+wire 门");
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
    results
        .lock()
        .unwrap()
        .insert(1, Some(Ok("{\"ok\":true,\"name\":\"gallery\"}".to_string())));
    let states = session
        .resume_http()
        .expect("resume 应有结果")
        .expect("resume 应成功");
    let ok = states
        .iter()
        .find(|(n, _)| n == "flag")
        .map(|(_, v)| format!("{v:?}"))
        .expect("flag 字段在案");
    assert_eq!(ok, "Bool(true)", "成员读真实取值落账，实际: {ok}");
    let label = states
        .iter()
        .find(|(n, _)| n == "label")
        .map(|(_, v)| format!("{v:?}"))
        .expect("label 字段在案");
    assert_eq!(
        label, "Str(\"gallery\")",
        "成员读经 `??` 非空折叠 + Dyn→str 物化落账，实际: {label}"
    );
}

#[test]
fn p117_get_json_invalid_response_is_named_error() {
    let program = ComponentEventSessionProgram::compile(
        GET_JSON_PROBE_SRC,
        "plan117.http.getjson",
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
    session.dispatch_state("Load", &[]).expect("Load 派发");
    results
        .lock()
        .unwrap()
        .insert(1, Some(Ok("not-json{".to_string())));
    let err = session
        .resume_http()
        .expect("resume 应有裁决")
        .expect_err("解析失败须具名 Error");
    assert!(
        err.contains("http.client"),
        "解析失败须为 http.client 具名错误，实际: {err}"
    );
}
