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
    dc.on_with_input_for("App", "Init", None);
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
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../apps/039-syslog/src/front/app.at");
    let src = match std::fs::read_to_string(&p) {
        Ok(s) => s,
        Err(e) => { eprintln!("[p042] skip (app file not found: {e})"); return; }
    };
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::parser::Parser::from(src.as_str()).with_session(session);
    match parser.parse() {
        Ok(_) => {}
        Err(e) => panic!("039-syslog app.at parse 失败: {e:?}"),
    }
}
