//! PLAN-793 V03 互操作消费方（AC-02/03）。
//!
//! 本 crate 模拟同时消费 auto-lang 与 auto-diagnostics 的外部使用者，
//! 证明 `auto_lang::error::*` 兼容门面与 `auto_diagnostics::error::*`
//! 新路径是**同一类型**（编译期互传 + 运行期投影一致）：真实 parse/
//! type/name/route 管线错误经 auto_lang 入口产生，再经 auto_diagnostics
//! 的 attach_source/format_error/From 走完整链路，错误阶段保持、?
//! 传播单调、坏输入不成功/不吞错。lib 本体无运行时面，全部测试代码收在
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

    /// UI 会话解析入口（真实生产路径：CompilerSession::ui()，与
    /// error_spans_tests::parse_ui 同一形态；check_field_type 的类型检查
    /// 只在该场景接线触发）。
    fn parse_ui(src: &str) -> Result<auto_lang::ast::Code, LangError> {
        auto_lang::parser::Parser::from(src)
            .with_session(auto_lang::session::CompilerSession::ui())
            .parse()
    }

    #[test]
    fn real_type_error_keeps_stage() {
        // 生产类型检查接线：check_field_type（infer/mod.rs，经 parser.rs
        // Pair 臂在解析 `P { x: <expr> }` 结构字段赋值时调用）。
        // 探针冻结（R1 修复轮，UI 会话）：坏字段 → TypeError::
        // FieldMismatch/E0106 span(57,1)；有效控制程序完整 parse。
        let ctrl = "type P {\n    x int\n}\nfn main() {\n    let p = P { x: 1 }\n    print(p)\n}\n";
        assert!(parse_ui(ctrl).is_ok(), "valid control must parse");

        let bad =
            "type P {\n    x int\n}\nfn main() {\n    let p = P { x: \"s\" }\n    print(p)\n}\n";
        let err = match parse_ui(bad) {
            Ok(_) => panic!("field type mismatch must not parse"),
            Err(e) => e,
        };
        // Type 族真实阶段 + 具体 code/display/span
        match &err {
            LangError::MultipleErrors { errors, .. } => match errors.first() {
                Some(LangError::Type(te)) => {
                    assert_eq!(
                        te.code().map(|c| c.to_string()).as_deref(),
                        Some("auto_type_E0106")
                    );
                    assert_eq!(te.to_string(), "field type mismatch");
                }
                other => panic!("expected inner Type, got {other:?}"),
            },
            other => panic!("expected MultipleErrors, got {other:?}"),
        }
        let span_pair = {
            let mut labels = err.labels().expect("labels");
            let label = labels.next().expect("first label");
            (label.offset(), label.len())
        };
        assert_eq!(span_pair, (57, 1));
        // 经 auto_diagnostics 投影链路：attach 不降级 Type 子错误，code 保持
        let attached = attach_source(err, "field.at".into(), bad.into());
        match &attached {
            LangError::MultipleErrors { errors, .. } => {
                assert!(matches!(errors.first(), Some(LangError::Type(_))));
            }
            other => panic!("expected MultipleErrors after attach, got {other:?}"),
        }
        let rendered = takes_diag(attached);
        assert!(rendered.contains("auto_type_E0106"));
        assert!(rendered.contains("field type mismatch"));
    }

    #[test]
    fn real_name_check_error_keeps_stage() {
        // 生产名称检查接线：check_symbol（parser.rs）解析期符号解析，
        // 未定义变量 → NameError::UndefinedVariable/E0201；help 走
        // NameError 手写 Diagnostic（建议助手 find_best_match 同一实现
        // 在 auto-diagnostics——无近邻候选时无建议句式）。
        let ctrl = "fn main() {\n    let x = 1\n    let y = x\n}\n";
        assert!(
            auto_lang::parse_preserve_error(ctrl).is_ok(),
            "valid control must parse"
        );

        let bad = "fn main() {\n    let y = undefined_var_793\n}\n";
        let err = match auto_lang::parse_preserve_error(bad) {
            Ok(_) => panic!("undefined variable must not pass name check"),
            Err(e) => e,
        };
        match &err {
            LangError::MultipleErrors { errors, .. } => match errors.first() {
                Some(LangError::Name(ne)) => {
                    assert_eq!(
                        ne.code().map(|c| c.to_string()).as_deref(),
                        Some("auto_name_E0201")
                    );
                    let help = ne.help().map(|h| h.to_string()).unwrap();
                    assert!(
                        help.contains("Variable 'undefined_var_793' is not defined"),
                        "help={help}"
                    );
                }
                other => panic!("expected inner Name, got {other:?}"),
            },
            other => panic!("expected MultipleErrors, got {other:?}"),
        }
        // 经 auto_diagnostics 链路投影，code/display 保持
        let rendered = takes_diag(err);
        assert!(rendered.contains("auto_name_E0201"));
        assert!(rendered.contains("undefined variable"));
    }

    #[test]
    fn real_compile_parser_recovery_error() {
        // 阶段归属（R1 F-02 修正）："must have either a type annotation" 是
        // parser 错误恢复（SyntaxError::Generic，parser.rs），**不是**类型
        // 推断；compile_once 返回前已把源码 attach 到子错误（内层为
        // SyntaxWithSource）。本用例钉住该入口行为：坏程序单调返回
        // （不吞错），内层仍居 Syntax 族。
        let src = "fn main() {\n    let x: int = \"s\"\n}\n";
        let err = match auto_lang::compile::compile_once(src, "t.at") {
            Ok(_) => panic!("bad program must not compile"),
            Err(e) => e,
        };
        match &err {
            LangError::MultipleErrors { errors, .. } => match errors.first() {
                Some(LangError::SyntaxWithSource(sw)) => {
                    assert_eq!(
                        sw.error.code().map(|c| c.to_string()).as_deref(),
                        Some("auto_syntax_E0007")
                    );
                    assert_eq!(sw.source.name(), "t.at");
                }
                other => panic!("expected inner SyntaxWithSource, got {other:?}"),
            },
            other => panic!("expected MultipleErrors, got {other:?}"),
        }
        assert!(
            err.to_string()
                .contains("must have either a type annotation"),
            "display={}",
            err
        );
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
