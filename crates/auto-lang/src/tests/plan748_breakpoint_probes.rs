//! PLAN-748 T-00：三断点定音探针（干净基面重取——基 e5a77106a，含 PLAN-749
//! 条件求值器 G1-G3 修复与 STR_CAT/ADD 显示修复）。
//!
//! musk PLAN-101 §10 原形态（2026-10-09 dirty 构建观测）：
//! - 断点①：模板 `if` 条件的**循环变量成员访问**恒不渲染（裸 binding 布尔、
//!   text 位成员读为实证可用对照；字面重标后仍不渲染）。
//! - 断点②：handler 续体内**静态名字段写**（`item.x = v`）RuntimeError
//!  （resume FAILED）；动态键 `item["x"] = v` 同 receiver 通过；无 await
//!   对照组通过。
//! - 断点③：动态 `style: .w.row_style` 不产出（静态串、裸 binding 的
//!   if-else 全静态串可用）。
//!
//! 本文件是 T-00 定音探针（判定产物=定音报告，红格即复现证据）；T-01/T-02/
//! T-03 修复时在本文件基础上固化红相矩阵（命名切 plan748_*_tests）。

#![cfg(all(test, feature = "ui-iced"))]

use crate::vm::engine::SegmentOutcome;
use crate::vm::task::AutoTask;

// ═══════════ 断点①②：fn 段驱动（与 handler 段同一 engine 段机制）═══════════

fn p748_compile(code: &str) -> crate::vm::engine::AutoVM {
    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(code).expect("compile");
    vm
}

/// 起 1 回合本地 HTTP 服务器（plan705/749 同款骨架）。
fn p748_spawn_server(body: &'static str) -> (std::thread::JoinHandle<()>, u16) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });
    (server, port)
}

/// 段驱动到 Parked(HttpRequest)，等结果就绪后 resume，返回续体终态与返回串。
fn p748_drive_park_resume(vm: &crate::vm::engine::AutoVM, name: &str) -> Result<String, String> {
    let mut task = AutoTask::new(0, 65536, 0);
    let outcome = vm.call_fn_by_name_segment(&mut task, name, 0);
    let seg = match &outcome {
        SegmentOutcome::Parked {
            wait: crate::vm::engine::ParkedWait::HttpRequest(id),
            seg,
        } => {
            let req_id = *id;
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
                assert!(std::time::Instant::now() < deadline, "server 应答超时");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            seg.clone()
        }
        other => return Err(format!("预期 Parked(HttpRequest)，得到 {other:?}")),
    };
    match vm.resume_fn_by_name_segment(&mut task, &seg) {
        SegmentOutcome::Completed(Ok(())) => {}
        SegmentOutcome::Completed(Err(e)) => {
            return Err(format!("续体 RuntimeError（musk 101 断点②形态）: {e:?}"))
        }
        other => return Err(format!("续体异常终态: {other:?}")),
    }
    let nv = task.ram.pop_nv();
    if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv) as usize;
        let pool = vm.strings.read().unwrap();
        let bytes = pool
            .get(idx)
            .cloned()
            .unwrap_or_else(|| b"<string-pool-oob>".to_vec());
        drop(pool);
        Ok(String::from_utf8_lossy(&bytes).to_string())
    } else {
        Err(format!("返回非串 nv raw={nv:016x}"))
    }
}

/// 断点②-主格：**await 续体内静态名字段写**（`item.x = v`）。
/// musk 101 §10①：RuntimeError（Init resume FAILED）。本格 fn 段同机制复现。
#[test]
fn p748_t00_resume_static_write() {
    let (server, port) = p748_spawn_server("{\"ok\":true}");
    let code = r#"
fn probe_static_write() str {
    let item = JSON.parse("{\"x\":0,\"y\":0}")
    let payload = Http.post_json("http://127.0.0.1:7482/seed", "")
    item.x = 5
    return "W:" + item.x.to_string()
}
"#
    .replace("7482", &port.to_string());
    let vm = p748_compile(&code);
    let res = p748_drive_park_resume(&vm, "probe_static_write");
    server.join().expect("server thread");
    match res {
        Ok(got) => {
            eprintln!("[t00-②static] 续体完成返回 = {got:?}");
            assert_eq!(got, "W:5", "静态名写在续体后须与无 await 对照同值");
        }
        Err(e) => {
            panic!("断点②复现（续体静态写失败）: {e}");
        }
    }
}

/// 断点②-对照A：动态键写（musk 实证通过形态）。
#[test]
fn p748_t00_resume_dynamic_write() {
    let (server, port) = p748_spawn_server("{\"ok\":true}");
    let code = r#"
fn probe_dynamic_write() str {
    let item = JSON.parse("{\"x\":0,\"y\":0}")
    let payload = Http.post_json("http://127.0.0.1:7482/seed", "")
    item["x"] = 6
    return "D:" + item.x.to_string()
}
"#
    .replace("7482", &port.to_string());
    let vm = p748_compile(&code);
    let res = p748_drive_park_resume(&vm, "probe_dynamic_write");
    server.join().expect("server thread");
    match res {
        Ok(got) => {
            eprintln!("[t00-②dynamic] 续体完成返回 = {got:?}");
            assert_eq!(got, "D:6", "动态键写为 musk 实证可用通道");
        }
        Err(e) => panic!("动态键写在续体后应通过（musk 实证对照）: {e}"),
    }
}

/// 断点②-对照B：无 await 静态名写（musk 实证通过形态）。
#[test]
fn p748_t00_noawait_static_write() {
    let code = r#"
fn probe_noawait_write() str {
    let item = JSON.parse("{\"x\":0,\"y\":0}")
    item.x = 7
    return "N:" + item.x.to_string()
}
"#;
    let vm = p748_compile(code);
    let mut task = AutoTask::new(0, 65536, 0);
    match vm.call_fn_by_name_segment(&mut task, "probe_noawait_write", 0) {
        SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("无 await 对照应直接完成: {other:?}"),
    }
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_string(nv));
    let idx = auto_val::decode_string(nv) as usize;
    let pool = vm.strings.read().unwrap();
    let got = String::from_utf8_lossy(pool.get(idx).unwrap_or(&b"?".to_vec())).to_string();
    drop(pool);
    eprintln!("[t00-②noawait] 返回 = {got:?}");
    assert_eq!(got, "N:7", "无 await 静态名写为 musk 实证可用通道");
}

// ═══════════ 断点①③：comp 驱动（store + 视图）═══════════

const P748_T00_HOST: &str = r#"
use sel_store: SelStore

widget App {
    view {
        col {
            // 对照（musk 实证可用）：裸 binding 布尔条件。
            if .store.collapsed {
                text "BARE_TRUE"
            } else {
                text "BARE_FALSE"
            }
            for w in .store.items {
                // 对照（musk 实证可用）：text 位成员读。
                text "ROW:" + w.name
                // 断点①主格：循环变量成员访问条件（is_current 形态）。
                if w.current {
                    text "CUR:" + w.name
                } else {
                    text "NC:" + w.name
                }
                // 混合形态：.store.X == w.id。
                if .store.sel_id == w.id {
                    text "SEL:" + w.name
                } else {
                    text "NSEL:" + w.name
                }
            }
            // 断点③：动态 style 字段引用 vs 静态串（row=绑定感知对照；
            // checkbox/progress=静态臂嫌疑元素）。
            row {
                style: .store.row_style
                text "STYLE_DYN_ROW"
            }
            row {
                style: "bg-primary"
                text "STYLE_STATIC_ROW"
            }
            checkbox {
                style: .store.box_style
            }
            progress (value: 50, max: 100) {
                style: .store.prog_style
            }
        }
    }
}
"#;

const P748_T00_STORE: &str = r#"
store SelStore {
    model {
        var items list = [{ id: "a", name: "A", current: true }, { id: "b", name: "B", current: false }]
        var sel_id str = "a"
        var collapsed bool = false
        var row_style str = "bg-primary"
        var box_style str = "border-primary"
        var prog_style str = "text-primary"
    }
}
"#;

fn build_p748_t00() -> crate::ui::dynamic::DynamicComponent {
    let dir = tempfile::TempDir::new().unwrap();
    let base = dir.path();
    for (rel, src) in [("host.at", P748_T00_HOST), ("sel_store.at", P748_T00_STORE)] {
        std::fs::write(base.join(rel), src).unwrap();
    }
    let host_path: std::path::PathBuf = base.join("host.at").to_path_buf();
    let host_src = std::fs::read_to_string(&host_path).unwrap();
    let comp = crate::build_dynamic_component(&host_src, Some(&host_path.to_string_lossy()))
        .expect("p748 fixture component builds");
    std::mem::forget(dir);
    comp
}

fn p748_view_debug(comp: &mut crate::ui::dynamic::DynamicComponent) -> String {
    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(10_000);
    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(10_000);
    let (view, _, _) = comp.view_with_debug_gated(false);
    format!("{view:?}")
}

/// 断点①：根件上下文条件矩阵（裸 binding 对照/成员访问主格/store 混合）。
#[test]
fn p748_t00_condition_matrix_root() {
    let mut comp = build_p748_t00();
    let debug = p748_view_debug(&mut comp);
    eprintln!(
        "[t00-①root] {}",
        debug.chars().take(2000).collect::<String>()
    );

    assert!(
        debug.contains("BARE_FALSE"),
        "对照：裸 binding 布尔条件可用（.collapsed=false 须走 else 臂）"
    );
    assert!(
        debug.contains("ROW:A") && debug.contains("ROW:B"),
        "对照：text 位循环成员读可用（两行都在）"
    );
    assert!(
        debug.contains("CUR:A") && debug.contains("NC:B") && !debug.contains("CUR:B"),
        "断点①主格：if w.current 成员访问条件须按行数据真值分支\
         （musk 101 §10②原形态——dirty 构建恒不渲染）"
    );
    assert!(
        debug.contains("SEL:A") && debug.contains("NSEL:B"),
        "混合形态：.store.sel_id == w.id 须按行数据真值分支"
    );
}

/// 断点③：动态 style 字段引用产出 vs 静态串对照。
#[test]
fn p748_t00_style_faces() {
    let mut comp = build_p748_t00();
    let debug = p748_view_debug(&mut comp);
    eprintln!(
        "[t00-③style] {}",
        debug.chars().take(2400).collect::<String>()
    );

    // 文本子节点在 = 元素渲染骨架在；style 载体断言：渲染树存解析后枚举
    // 形态（bg-primary→BackgroundColor(Primary)），定位 DYN row 自身区段。
    let pos = debug.find("STYLE_DYN_ROW").expect("DYN row 骨架在");
    let seg = &debug[pos..(pos + 260).min(debug.len())];
    let dyn_has_style = seg.contains("style: Some(Style");
    let dyn_has_class = seg.contains("BackgroundColor(Primary)");
    eprintln!("[t00-③style] DYN row 区段 style 载体={dyn_has_style} 类={dyn_has_class}");
    assert!(
        dyn_has_style && dyn_has_class,
        "断点③：row style: .store.row_style 动态引用须产出（绑定感知对照臂）\
         （实测干净 master 产出——musk 101 观测属 dirty 构建陈旧或未测元素）"
    );
    // 静态臂清单元素（plan §4 定位：checkbox :8326 / progress :8265）动态
    // 引用产出钉死（round-1 实测均产出）。
    assert!(
        debug.contains("BorderColor(Primary)"),
        "checkbox style: .store.box_style 动态引用须产出"
    );
    assert!(
        debug.contains("TextColor(Primary)"),
        "progress style: .store.prog_style 动态引用须产出"
    );
}

// ═══════════ 断点②升级面：store handler 续体（musk 原形态=Init await 后静态写）═══════════

/// store handler 段 park/resume 续体写矩阵（msg-pump 派发路径——musk 101
/// §10① 原形态：Init 内 await Http → resume → `item.x = v` RuntimeError；
/// 动态键同 receiver 通过）。fn 段已绿（上组探针），本组升级到 comp
/// harness 的 handler 派发链。arm: static / dynamic / noawait。
fn p748_store_handler_write_face(write_arm: &str) -> String {
    let (server, port) = p748_spawn_server("{\"ok\":true,\"v\":9}");
    let (write_stmt, expect) = match write_arm {
        "static" => ("item.x = 5", "W:5"),
        "dynamic" => ("item[\"x\"] = 6", "W:6"),
        "noawait" => ("item.x = 7", "W:7"),
        _ => unreachable!(),
    };
    let await_stmt = if write_arm == "noawait" {
        String::new()
    } else {
        format!("let payload = Http.post_json(\"http://127.0.0.1:{port}/seed\", \"\")")
    };
    let host = r#"
use w_store: WStore

widget App {
    view {
        col {
            text f"marker=${.store.marker}"
        }
    }
    msg Msg { Boot }
    on {
        .Init -> { store.Boot() }
    }
}
"#
    .to_string();
    let store = format!(
        r#"
store WStore {{
    model {{
        var marker str = "unset"
    }}
    msg Msg {{ Boot }}
    on {{
        .Boot -> {{
            let item = JSON.parse("{{\"x\":0}}")
            {await_stmt}
            {write_stmt}
            .marker = "W:" + item.x.to_string()
        }}
    }}
}}
"#
    );
    let dir = tempfile::TempDir::new().unwrap();
    let base = dir.path();
    std::fs::write(base.join("host.at"), host).unwrap();
    std::fs::write(base.join("w_store.at"), store).unwrap();
    let host_path = base.join("host.at").to_path_buf();
    let host_src = std::fs::read_to_string(&host_path).unwrap();
    let mut comp = crate::build_dynamic_component(&host_src, Some(&host_path.to_string_lossy()))
        .expect("p748 store-handler fixture builds");
    std::mem::forget(dir);

    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(15_000);
    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(15_000);
    if write_arm != "noawait" {
        server.join().expect("server thread");
    }

    let marker = match comp.read_state("marker") {
        Ok(auto_val::Value::Str(s)) => s.as_str().to_string(),
        other => format!("<marker not str: {other:?}>"),
    };
    eprintln!("[t00-②handler-{write_arm}] marker = {marker:?} (expect {expect})");
    marker
}

#[test]
fn p748_t00_handler_static_arm() {
    assert_eq!(
        p748_store_handler_write_face("static"),
        "W:5",
        "store handler 续体静态写须成功（musk 101 §10① 形态）"
    );
}

#[test]
fn p748_t00_handler_dynamic_arm() {
    assert_eq!(
        p748_store_handler_write_face("dynamic"),
        "W:6",
        "续体动态键写为 musk 实证可用通道"
    );
}

#[test]
fn p748_t00_handler_noawait_arm() {
    assert_eq!(
        p748_store_handler_write_face("noawait"),
        "W:7",
        "无 await 静态写为 musk 实证可用通道"
    );
}
