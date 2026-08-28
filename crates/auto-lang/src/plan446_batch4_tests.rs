//! Plan 446 批四：打磨项回归（§F/§B/§D/§E 报告项）。
//!
//! 现场背景（auto-os-config Plan 007）：
//!   - F3: map 字面量内空数组字面量（{members: []}）崩溃；字面量内
//!     `.len()` 等方法调用求值为 0——均无告警；
//!   - D4: `.find(闭包)` 在 store handler 静默失效（fn 模块正常，与 D2 同源）；
//!   - D5: `json.get_at` 仅接受 JSON 文本——对 VM 数组返回空、无告警；
//!   - E3: `res.body()` 在错误响应上返回垃圾值（偶见巨大数字串）。
//! 探针先实证现 master 状态（D2/D5 同族项已在批二/批三治愈，预期部分已愈）。

#[cfg(test)]
mod plan446_batch4_probes {
    fn run(src: &str) -> Result<String, String> {
        match std::panic::catch_unwind(|| crate::run_with_capture(src)) {
            Ok(Ok((_result, stdout))) => Ok(stdout),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("panicked".to_string()),
        }
    }

    // ------------------------------------------------------------------
    // F3: map 字面量内空数组字面量不得崩溃。
    // ------------------------------------------------------------------
    #[test]
    fn f3_map_literal_with_empty_array() {
        let src = "fn main() {\n\
                   \x20   let g = {members: [], name: \"g1\"}\n\
                   \x20   print(g.name)\n\
                   \x20   print(g.members.len().to_string())\n\
                   }\n";
        let out = run(src).expect("F3 empty-array literal must not crash");
        assert!(
            out.contains("g1") && out.contains('0'),
            "F3: map literal with empty array broken, stdout={out:?}"
        );
    }

    // ------------------------------------------------------------------
    // F3(续): 字面量值位的方法调用不得静默求值为 0。
    // ------------------------------------------------------------------
    #[test]
    fn f3_method_call_inside_map_literal() {
        let src = "fn make() []str {\n\
                   \x20   return [\"a\", \"b\", \"c\"]\n\
                   }\n\
                   fn main() {\n\
                   \x20   let arr = make()\n\
                   \x20   let g = {name: \"g2\", count: arr.len()}\n\
                   \x20   print(g.name)\n\
                   \x20   print(g.count.to_string())\n\
                   }\n";
        let out = run(src).expect("F3 method-in-literal must run");
        assert!(
            out.contains("g2"),
            "F3: literal field lost, stdout={out:?}"
        );
        assert!(
            out.contains('3'),
            "F3: method call inside map literal evaluated to garbage (expected 3), stdout={out:?}"
        );
    }

    // ------------------------------------------------------------------
    // D5: json.get_at 接受 VM 数组（json.keys 的返回）——批三双态化的覆盖面
    // 探针（Vec<String> 推栈形态是否命中 ListData<Value> 判定）。
    // ------------------------------------------------------------------
    #[test]
    fn d5_json_get_at_on_vm_array() {
        let src = "use auto.json\n\
                   fn main() {\n\
                   \x20   let ks = json.keys(\"{\\\"zebra\\\":1,\\\"alpha\\\":2}\")\n\
                   \x20   let first = json.get_at(ks, 0)\n\
                   \x20   print(first)\n\
                   }\n";
        let out = run(src).expect("D5 program must run");
        assert!(
            out.contains("zebra"),
            "D5: json.get_at on VM array broken (expected first key), stdout={out:?}"
        );
    }

    // ------------------------------------------------------------------
    // D4(fn 模块对照): 闭包 find 在 script 语境可用（现场已知 ✓，回归守卫）。
    // ------------------------------------------------------------------
    #[test]
    fn d4_find_closure_fn_module_baseline() {
        let src = "fn pick(xs []any) str {\n\
                   \x20   let hit = xs.find(m => m.name == \"beta\")\n\
                   \x20   return hit.name\n\
                   }\n\
                   fn main() {\n\
                   \x20   print(pick([{\"name\": \"alpha\"}, {\"name\": \"beta\"}]))\n\
                   }\n";
        let out = run(src).expect("D4 baseline must run");
        assert!(
            out.contains("beta"),
            "D4: closure find in fn module broken, stdout={out:?}"
        );
    }

    // ------------------------------------------------------------------
    // E3: 错误响应上 res.body() 不得返回垃圾——返回真实 body 文本/字节。
    // ------------------------------------------------------------------
    #[test]
    fn e3_res_body_on_error_response() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let resp = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nContent-Length: 5\r\nConnection: close\r\n\r\nboom!";
            let _ = s.write_all(resp.as_bytes());
        });
        let src = format!(
            "use auto.http\n\
             fn main() {{\n\
             \x20   let res = http.get(\"http://127.0.0.1:{port}/err\")\n\
             \x20   print(res.status().to_string())\n\
             \x20   print(res.body())\n\
             }}\n"
        );
        let out = run(&src).expect("E3 program must run");
        assert!(
            out.contains("500"),
            "E3: error status missing, stdout={out:?}"
        );
        // body 须是真实负载("boom!")或其字节表示——不得是巨大数字串垃圾。
        assert!(
            out.contains("boom"),
            "E3: res.body() on error response garbage (expected boom!), stdout={out:?}"
        );
        let garbage = out.lines().any(|l| {
            l.len() > 12 && l.chars().all(|c| c.is_ascii_digit())
        });
        assert!(
            !garbage,
            "E3: garbage digit-run leaked into body, stdout={out:?}"
        );
    }
}

#[cfg(all(test, feature = "ui-iced"))]
mod plan446_batch4_ui {
    use crate::plan370_test_support::build_component_from_app;

    fn build_d2_corpus() -> Option<crate::ui::dynamic::DynamicComponent> {
        let rel = "test/ui/plan446_d2_handler_read/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        build_component_from_app(&candidates.into_iter().flatten().find(|p| p.exists())?)
    }

    /// D4(修复靶): `.find(闭包)` 在 store/App handler 语境——现场"静默不匹配"
    /// （同数据 for 循环正常）。复用 D2 corpus 的 store 数据面，经 Pick(name)
    /// 与 .Probe2(handler 内 find) 双路径对照。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d4_find_closure_in_handler_context() {
        let mut dc = match build_d2_corpus() {
            Some(c) => c,
            None => panic!("D2/D4 corpus not found"),
        };
        // .Probe2 handler 体: let hit = .items.find(m => m.name == "beta")
        //                    .picked = hit.name
        dc.on_with_input_for("App", "Probe2", None);
        let picked = dc.read_state("picked").expect("picked readable");
        assert!(
            matches!(&picked, auto_val::Value::Str(s) if s.as_str() == "beta"),
            "D4: closure find in handler broken, got {:?}",
            picked
        );
    }
}
