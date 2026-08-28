//! Plan 446 批三：动态值管线语义统一回归（§D/§G 报告项）。
//!
//! 现场背景（auto-os-config Plan 007，2026-08-25 基线）：
//!   - D1: `json.parse` 是占位 shim（原样返回字符串）——点访问/方法链全垃圾；
//!   - D2: handler 内两跳数组读取/局部中转读取静默失效（fn 模块正常）；
//!   - D3: 数组跨 fn 边界作实参丢失（arr_len(fetch()) == 0）；
//!   - D6: json.keys 字母序（serde_json 未开 preserve_order），与 vue 轨
//!     插入序不一致（跨后端 UI 字段顺序 parity 破坏）。
//! 探针先实证现 master 状态（多批合入后可能有已愈项），红项即修复靶。

#[cfg(test)]
mod plan446_batch3_probes {
    fn run(src: &str) -> Result<String, String> {
        match std::panic::catch_unwind(|| crate::run_with_capture(src)) {
            Ok(Ok((_result, stdout))) => Ok(stdout),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("panicked".to_string()),
        }
    }

    // ------------------------------------------------------------------
    // D1: json.parse 必须产出可点访问的复合值（book-reader 模式）。
    // ------------------------------------------------------------------
    #[test]
    fn d1_json_parse_supports_dot_access() {
        let src = "use auto.json\n\
                   fn main() {\n\
                   \x20   let body = json.parse(\"{\\\"provider\\\":{\\\"kind\\\":\\\"daemon\\\"},\\\"count\\\":3}\")\n\
                   \x20   print(body.provider.kind)\n\
                   \x20   print(body.count.to_string())\n\
                   }\n";
        let out = run(src).expect("D1 program must run");
        assert!(
            out.contains("daemon"),
            "D1: nested dot access after json.parse broken, stdout={out:?}"
        );
        assert!(out.contains('3'), "D1: scalar field access broken, stdout={out:?}");
        // 占位 shim 的症状:原样返回 JSON 文本本身。
        assert!(
            !out.contains("{\\\"provider"),
            "D1: placeholder passthrough leaked raw JSON, stdout={out:?}"
        );
    }

    // ------------------------------------------------------------------
    // D1 附带: parse 结果上的数组迭代(for-in + 字段读)。
    // ------------------------------------------------------------------
    #[test]
    fn d1_json_parse_array_iteration() {
        let src = "use auto.json\n\
                   fn main() {\n\
                   \x20   let body = json.parse(\"{\\\"mods\\\":[{\\\"id\\\":\\\"a\\\"},{\\\"id\\\":\\\"b\\\"}]}\")\n\
                   \x20   for m in body.mods {\n\
                   \x20       print(m.id)\n\
                   \x20   }\n\
                   }\n";
        let out = run(src).expect("D1 array program must run");
        assert!(
            out.contains('a') && out.contains('b'),
            "D1: array iteration after json.parse broken, stdout={out:?}"
        );
    }

    // ------------------------------------------------------------------
    // D3: 数组跨 fn 边界作实参。
    // ------------------------------------------------------------------
    #[test]
    fn d3_array_cross_fn_arg() {
        let src = "fn make_list() []any {\n\
                   \x20   return [{\"id\": \"x1\"}, {\"id\": \"x2\"}, {\"id\": \"x3\"}]\n\
                   }\n\
                   fn arr_len(a []any) int {\n\
                   \x20   return a.len()\n\
                   }\n\
                   fn main() {\n\
                   \x20   print(arr_len(make_list()).to_string())\n\
                   }\n";
        let out = run(src).expect("D3 program must run");
        assert!(
            out.contains('3'),
            "D3: array lost across fn boundary (expected 3), stdout={out:?}"
        );
    }

    // ------------------------------------------------------------------
    // D6: json.keys 保持插入序（跨后端 parity）。
    // ------------------------------------------------------------------
    #[test]
    fn d6_json_keys_insertion_order() {
        let src = "use auto.json\n\
                   fn main() {\n\
                   \x20   let raw = \"{\\\"zebra\\\":1,\\\"alpha\\\":2,\\\"mid\\\":3}\"\n\
                   \x20   let ks = json.keys(raw)\n\
                   \x20   for k in ks {\n\
                   \x20       print(k)\n\
                   \x20   }\n\
                   }\n";
        let out = run(src).expect("D6 program must run");
        let z = out.find("zebra");
        let a = out.find("alpha");
        let m = out.find("mid");
        assert!(
            z.is_some() && a.is_some() && m.is_some(),
            "D6: keys missing, stdout={out:?}"
        );
        // 插入序: zebra(1st) → alpha(2nd) → mid(3rd)。字母序会是 alpha,mid,zebra。
        assert!(
            z.unwrap() < a.unwrap() && a.unwrap() < m.unwrap(),
            "D6: keys not in insertion order (zebra,alpha,mid expected), stdout={out:?}"
        );
    }
}

#[cfg(all(test, feature = "ui-iced"))]
mod plan446_batch3_d2_ui {
    use crate::plan370_test_support::build_component_from_app;

    /// D2 现场矩阵里 handler 侧的循环变量字段读（"model 数组循环变量 m.id
    /// 失效(静默不匹配)"行）。corpus: test/ui/plan446_d2_handler_read/。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d2_handler_loop_var_field_read() {
        let rel = "test/ui/plan446_d2_handler_read/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        let path = match candidates.into_iter().flatten().find(|p| p.exists()) {
            Some(p) => p,
            None => panic!("D2 corpus not found"),
        };
        let mut dc = build_component_from_app(&path).expect("D2 corpus must build");
        // .Probe: for m in .items { if m.name == "alpha" { .picked = m.name } }
        dc.on_with_input_for("App", "Probe", None);
        let picked = dc.read_state("picked").expect("picked readable");
        assert!(
            matches!(&picked, auto_val::Value::Str(s) if s.as_str() == "alpha"),
            "D2: loop-var field read in handler broken, got {:?}",
            picked
        );
    }
}
