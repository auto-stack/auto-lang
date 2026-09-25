// PLAN-042 T-08：ScanProbe 最小复现 + D6 根修回归钉。
//
// 病灶（2026-09-23 P041-D6 勘定 + 本计划勘定收口）：桌面音乐 app 曲库页
// entries 解析恒 0。根因**不在 VM 异步延续**（延续推值正确——本计划
// 探针实证）而在 **PLAN-080 F-2③ 编译期改写 × 文档配方相戗**：
// `Http.get_json(url)` 自此编译为 `get_json + json.to_value`（web 轨
// fetch().json() 语义），PLAN-617 T-10 文档配方
// `json.to_value(Http.get_json(url))` 成为**二次转换**——旧 to_value 把
// 已解析 Obj 强转 String 解析失败 → 静默 null → `data.entries ?? []`
// 恒空（延续值呈 20 位数字串 = NV 位型显示）。
//
// 根修（本仓 T-08）：`json.to_value` 幂等——TAG_OBJECT/TAG_LIST 接收者
// 直通。两测分别钉裸形态（PLAN-080 后契约）与显式配方（存量 .at 消费
// 方形态）；配方测在根修前红（entries=0/null）、根修后绿（回归钉永驻）。

/// 进程内 HTTP 服务：循环 accept（组件 mount 自发 Init + 探针显式 Init
/// 双派发——两次请求都要吃到同一 body），逐连接回固定 JSON body。
fn spawn_scan_server(body: &'static str) -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        use std::io::{Read, Write};
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
        // 端口随 listener drop 释放（测试进程退出）。
    });
    port
}

fn probe_app(load_stmt: &str, port: u16) -> crate::ui::dynamic::DynamicComponent {
    let src = format!(
        r#"widget App {{
    msg {{ Init }}
    model {{
        var entries int = -1
    }}
    view {{ col {{ text .entries }} }}
    on {{
        .Init -> {{
            .entries = -10
            let data = {load_stmt}
            .entries = -11
            .entries = data.entries.len()
        }}
    }}
}}
"#,
        load_stmt = load_stmt.replace("{PORT}", &port.to_string()),
    );
    crate::build_dynamic_component(&src, Some("scan_probe/app.at")).expect("probe production build")
}

fn init_and_read_entries(dc: &mut crate::ui::dynamic::DynamicComponent) -> i32 {
    // PLAN-702 段驱动适配：Init 内含 api 调用即 park（挂载自发 Init 先
    // park，显式 Init 因重入互斥被忽略）——测试侧驱动恢复泵至落账
    // （生产 = `__parked_resume_tick` 泵；同步驱动时代"dispatch 即落账"
    // 的假设已随段驱动退役，见 plan370_test_support::drive_parked_segments）。
    dc.on_with_input_for("App", "Init", None);
    crate::plan370_test_support::drive_parked_segments(
        dc,
        |dc| {
            matches!(
                dc.bridge().read_state("entries"),
                Ok(auto_val::Value::Int(n)) if n >= 0
            )
        },
        "scan_probe entries",
    );
    match dc.bridge().read_state("entries").expect("entries state") {
        auto_val::Value::Int(n) => n,
        other => panic!("entries 非整型: {other:?}"),
    }
}

/// 裸形态（PLAN-080 F-2③ 后契约）：`Http.get_json(url)` 表达式值 =
/// 已解析 body 对象，entries 直读 = 1。
#[test]
fn scan_probe_bare_get_json_yields_parsed_object() {
    const BODY: &str = r#"{"entries":[{"id":"a","title":"A"}],"root_missing":false}"#;
    let port = spawn_scan_server(BODY);
    let mut dc = probe_app(r#"Http.get_json("http://127.0.0.1:{PORT}/scan")"#, port);
    assert_eq!(init_and_read_entries(&mut dc), 1, "裸形态 entries 解析");
}

/// 显式配方（PLAN-617 T-10 文档配方 / 音乐 030-video 存量形态）：
/// `json.to_value(Http.get_json(url))` 经 to_value 幂等根修后 entries=1。
/// （根修前：二次 to_value 把 Obj 强转 String 解析失败 → null →
/// `data.entries ?? []` 落空 → 本测红。）
#[test]
fn scan_probe_documented_recipe_double_to_value_idempotent() {
    const BODY: &str = r#"{"entries":[{"id":"a","title":"A"}],"root_missing":false}"#;
    let port = spawn_scan_server(BODY);
    let mut dc = probe_app(
        r#"json.to_value(Http.get_json("http://127.0.0.1:{PORT}/scan"))"#,
        port,
    );
    assert_eq!(
        init_and_read_entries(&mut dc),
        1,
        "文档配方（幂等根修后）entries 解析"
    );
}

#[test]
fn p042_app_at_parses() {
    // 跨仓兄弟解析（AGENTS.md §2 序）——组内 .wt/os-042/auto-lang 与主检出
    // D:/autostack/{auto-lang,auto-os} 两形态同构（CARGO_MANIFEST_DIR 上三
    // 级即组/伞根）。R-1 修复：原路径少一级 auto-os，工作树内恒 skip 空绿。
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("auto-os")
        .join("apps")
        .join("039-syslog")
        .join("src")
        .join("front")
        .join("app.at");
    let src = match std::fs::read_to_string(&p) {
        Ok(s) => s,
        Err(e) => {
            // 兄弟检出缺席（CI 纯 auto-lang 检出）= 合法跳过，响亮留痕。
            eprintln!("[p042] skip (auto-os sibling checkout absent: {e})");
            return;
        }
    };
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::parser::Parser::from(src.as_str()).with_session(session);
    match parser.parse() {
        Ok(_) => {}
        Err(e) => panic!("039-syslog app.at parse 失败: {e:?}"),
    }
}

/// PLAN-042 T-09（AC-03 headless 断言）：坏 handler app 的失败行入环
/// （T-03 trap；源含 face 标识 `vm:<dir>/<stem>`）。并发进程内唯一
/// source 过滤自证（ring 全局共享同 syslog 单测口径）。
#[test]
fn p042_bad_handler_error_lands_in_ring() {
    let src = r#"widget Bad {
    msg { Init }
    model { var probe str = "" }
    view { col { text .probe } }
    on {
        .Init -> {
            .probe = .nonexistent_field.len()
        }
    }
}
"#;
    let mut dc = crate::build_dynamic_component(src, Some("scan_probe/bad.at"))
        .expect("bad app build");
    dc.on_with_input_for("Bad", "Init", None);
    let face = "vm:scan_probe/bad";
    let hit = crate::ui::syslog::snapshot()
        .into_iter()
        .find(|e| e.source == face && e.msg.contains("[VM-HANDLER]") && e.msg.contains("failed"));
    assert!(
        hit.is_some(),
        "坏 handler 失败行应入环（source={face}）: got {:?}",
        hit
    );
    let hit = hit.unwrap();
    assert!(matches!(
        hit.level,
        crate::ui::syslog::SyslogLevel::Error
    ));
}

/// PLAN-042 review R-2：HostLogger 直测（AC-02 log crate 腿）——error/
/// warn/info 入环（source=host、target 前缀、级别映射），Debug/Trace 不入
/// 环。并发进程内唯一 target 过滤自证（ring 全局共享同 syslog 单测口径）。
#[test]
fn p042_host_logger_levels_into_ring() {
    use log::{Level, Record};
    use log::Log as _;
    let logger = crate::ui::syslog::HostLogger;
    // format_args! 借用局部——Record 逐条就地构建（无闭包返回）。
    macro_rules! rec {
        ($level:expr, $msg:expr) => {{
            let r = Record::builder()
                .level($level)
                .target("p042_host")
                .args(format_args!("{}", $msg))
                .build();
            logger.log(&r);
        }};
    }
    rec!(Level::Error, "err line");
    rec!(Level::Warn, "warn line");
    rec!(Level::Info, "info line");
    rec!(Level::Debug, "debug line");
    rec!(Level::Trace, "trace line");
    let got: Vec<_> = crate::ui::syslog::snapshot()
        .into_iter()
        .filter(|e| e.msg.starts_with("[p042_host]"))
        .collect();
    assert_eq!(got.len(), 3, "error/warn/info 入环，Debug/Trace 不入");
    assert!(got.iter().all(|e| e.source == "host"));
    assert_eq!(got[0].msg, "[p042_host] err line");
    assert!(matches!(
        got[0].level,
        crate::ui::syslog::SyslogLevel::Error
    ));
    assert!(matches!(got[2].level, crate::ui::syslog::SyslogLevel::Info));
}
