//! PLAN-793 V03 互操作消费方（AC-02/03）。
//!
//! 本 crate 模拟同时消费 auto-lang 与 auto-diagnostics 的外部使用者，
//! 证明 `auto_lang::error::*` 兼容门面与 `auto_diagnostics::error::*`
//! 新路径是**同一类型**（编译期互传 + 运行期投影一致）：真实 parse/
//! compile/route 管线错误经 auto_lang 入口产生，再经 auto_diagnostics 的
//! attach_source/format_error/From 走完整链路，错误阶段保持、? 传播
//! 单调、坏输入不成功/不吞错。lib 本体无运行时面，全部测试代码收在
//! `#[cfg(test)]` 模块内（对齐 F-789-R1-03；探针冻结值见基面 579f6dec0）。

#[cfg(test)]
mod tests {
    use auto_diagnostics::error::{
        attach_source, format_error, AutoError as DiagError, SyntaxError as DiagSyntaxError,
    };
    use auto_lang::error::{AutoError as LangError, AutoResult, SyntaxError as LangSyntaxError};
    use miette::Diagnostic;

    // ---------- 编译期同一性（编译通过即证明） ----------

    /// 以 auto_diagnostics 的类型签名，接受 auto_lang 的值。
    fn takes_diag(e: DiagError) -> String {
        format_error(&e)
    }

    /// 以 auto_lang 的类型签名，接受 auto_diagnostics 构造的值。
    fn takes_lang(e: LangError) -> LangError {
        e
    }

    #[test]
    fn same_type_compiles_both_directions() {
        let via_lang: LangError = LangSyntaxError::Generic {
            message: "same".into(),
            span: auto_lang::error::span_from(0, 4),
        }
        .into();
        // LangError → DiagError 函数 → String（零转换互传）
        assert!(takes_diag(via_lang.clone()).contains("same"));

        let via_diag: DiagError = DiagSyntaxError::Generic {
            message: "same2".into(),
            span: auto_diagnostics::error::span_from(0, 5),
        }
        .into();
        // DiagError → LangError 函数 → match（跨路径模式匹配同一枚举）
        match takes_lang(via_diag) {
            LangError::Syntax(_) => {}
            other => panic!("expected Syntax variant, got {other:?}"),
        }

        // span 助手两路径产同一类型值
        let a = auto_lang::error::span_from(7, 3);
        let b = auto_diagnostics::error::span_from(7, 3);
        assert_eq!((a.offset(), a.len()), (b.offset(), b.len()));
    }

    // ---------- 真实管线错误 ----------

    #[test]
    fn real_parse_error_flows_through_both_paths() {
        // 探针冻结（基面 579f6dec0）：坏语法 → MultipleErrors/E0099
        let err = match auto_lang::parse_preserve_error("let x = 1 +") {
            Ok(_) => panic!("bad syntax must not parse"),
            Err(e) => e,
        };
        assert_eq!(
            err.code().map(|c| c.to_string()).as_deref(),
            Some("auto_syntax_E0099")
        );
        assert!(err.to_string().contains("Expected term, got EOF"));

        // 经 auto_diagnostics attach/format（同一值，阶段/身份不变）
        let attached = attach_source(err, "bad.at".into(), "let x = 1 +".into());
        match &attached {
            // 语法阶段错误 attach 后子错误升级为 SyntaxWithSource（仍在语法
            // 族，不降级为字符串），聚合结构保持
            LangError::MultipleErrors { errors, .. } => {
                assert!(matches!(
                    errors.first(),
                    Some(LangError::SyntaxWithSource(_))
                ));
            }
            other => panic!("expected MultipleErrors after attach, got {other:?}"),
        }
        let rendered = format_error(&attached);
        assert!(rendered.contains("bad.at"));
        assert!(rendered.contains("Expected term, got EOF"));
    }

    #[test]
    fn real_compile_error_keeps_stage() {
        // 探针冻结：infer/语义阶段错误经 compile_once 单调返回（不吞错）
        let src = "fn main() {\n    let x: int = \"s\"\n}\n";
        let err = match auto_lang::compile::compile_once(src, "t.at") {
            Ok(_) => panic!("bad program must not compile"),
            Err(e) => e,
        };
        assert!(
            err.to_string()
                .contains("must have either a type annotation"),
            "display={}",
            err
        );
        // 同一错误值经 auto_diagnostics 路径再投影，code 逐字保持
        let rendered = takes_diag(err.clone());
        assert!(rendered.contains("auto_syntax_E0099"));
    }

    #[test]
    fn real_route_error_maps_to_msg() {
        // 探针冻结：模块专有 From（owner 侧 route/discovery.rs）经门面可用
        let rd = auto_lang::route::RouteDiscoveryError::DirectoryNotFound(
            std::path::PathBuf::from("no-such-routes"),
        );
        let err: LangError = rd.into();
        assert!(matches!(err, LangError::Msg(_)));
        assert_eq!(
            err.to_string(),
            "Routes directory not found: no-such-routes"
        );
        // 同一错误值经 auto_diagnostics 投影链路，Display 逐字保持
        let rendered = takes_diag(err.clone());
        assert!(rendered.contains("Routes directory not found: no-such-routes"));
    }

    #[test]
    fn question_mark_propagation_is_monotonic() {
        use auto_lang::error::AutoError as E;

        fn inner() -> Result<String, E> {
            // String → AutoError（From 在 auto-diagnostics 单一定义）
            Err(E::from("inner failed"))
        }
        fn outer() -> AutoResult<usize> {
            let s = inner()?; // ? 经同一 From 链传播
            Ok(s.len())
        }
        assert!(outer().is_err());

        // io::Error → Io（Windows/Unix OS 消息不含路径，只断言族与内容非空）
        fn io_path() -> AutoResult<()> {
            std::fs::read_to_string("definitely-missing-793.at")?;
            Ok(())
        }
        match io_path() {
            Err(E::Io(msg)) => assert!(!msg.is_empty()),
            other => panic!("expected Io, got {other:?}"),
        }
    }
}
