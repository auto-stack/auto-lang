//! PLAN-793 V01 外部诊断消费方（AC-01）。
//!
//! 本 crate 的 manifest 是验收证据：正常依赖只有 auto-diagnostics
//! （诊断单一来源）、auto-lex（LexError/LexerError 构造面，本就在
//! auto-diagnostics 闭包内）与 miette（Diagnostic 投影 trait）——构建并
//! 运行这些测试证明诊断基础（各族错误/attach source/格式化/建议）
//! 可在没有 auto-lang / auto-man / auto-vm / UI 的依赖树里真实工作，
//! 不借主库 rlib。lib 本体无运行时面（纯消费方探针），全部测试代码
//! 收在 `#[cfg(test)]` 模块内——正常 lib 编译零警告（对齐 F-789-R1-03）。
//!
//! 期望值取自 PLAN-793 V02 冻结件（docs/plans/attachments/793/
//! v02-golden-frozen.json 对应案例的逐字段值，非新造）。

#[cfg(test)]
mod tests {
    use auto_diagnostics::error::{
        attach_source, find_best_match, format_error, pos_to_span, span_from, AutoError,
        ComptimeError, LexerError, NameError, RuntimeError, SyntaxError, TypeError, Warning,
    };
    use auto_lex::error::LexError;
    use auto_lex::token::Pos;
    use miette::Diagnostic;

    fn variant(e: &AutoError) -> &'static str {
        match e {
            AutoError::Lexer(_) => "Lexer",
            AutoError::LexerWithSource(_) => "LexerWithSource",
            AutoError::Syntax(_) => "Syntax",
            AutoError::SyntaxWithSource(_) => "SyntaxWithSource",
            AutoError::Type(_) => "Type",
            AutoError::Name(_) => "Name",
            AutoError::Runtime(_) => "Runtime",
            AutoError::Comptime(_) => "Comptime",
            AutoError::MultipleErrors { .. } => "MultipleErrors",
            AutoError::Warning(_) => "Warning",
            AutoError::Io(_) => "Io",
            AutoError::Msg(_) => "Msg",
            AutoError::MsgWithSource(_) => "MsgWithSource",
        }
    }

    #[test]
    fn families_construct_and_project_standalone() {
        // 语法族（golden: syntax_unexpected_token / syntax_generic）
        let syn: AutoError = SyntaxError::UnexpectedToken {
            expected: "';'".into(),
            found: "'let'".into(),
            span: span_from(12, 3),
        }
        .into();
        assert_eq!(variant(&syn), "Syntax");
        assert_eq!(
            syn.code().map(|c| c.to_string()).as_deref(),
            Some("auto_syntax_E0001")
        );
        assert_eq!(syn.to_string(), "unexpected token");

        // 类型族（golden: type_cannot_modify_view_param_cjk——Name=AutoStr 身份
        // 在无 AST 的外部消费侧直接可用）
        let ty: AutoError = TypeError::CannotModifyViewParam {
            param: "计数器".into(),
            span: span_from(4, 9),
        }
        .into();
        assert_eq!(variant(&ty), "Type");
        assert_eq!(
            ty.code().map(|c| c.to_string()).as_deref(),
            Some("auto_type_E0204")
        );
        assert_eq!(ty.to_string(), "Cannot modify view parameter '计数器'");

        // 名称族建议（golden: name_undefined_with_suggestion）
        let nm: AutoError = NameError::undefined_variable(
            "conut".into(),
            span_from(4, 5),
            &["count".to_string(), "account".to_string()],
        )
        .into();
        assert_eq!(variant(&nm), "Name");
        let help = nm.help().map(|h| h.to_string()).unwrap();
        assert!(help.contains("Did you mean 'count'?"), "help={help}");

        // 运行时族（golden: runtime_out_of_memory_no_span——无 span 变体）
        let rt: AutoError = RuntimeError::OutOfMemory.into();
        assert_eq!(variant(&rt), "Runtime");
        assert!(rt.labels().is_none());

        // comptime / warning（golden: comptime_compile_error / warning_rc_cycle）
        let ct: AutoError = ComptimeError::CompileError {
            message: "feature disabled in strict mode".into(),
            span: span_from(10, 13),
        }
        .into();
        assert_eq!(
            ct.code().map(|c| c.to_string()).as_deref(),
            Some("auto_comptime_E0401")
        );
        let wn: AutoError = Warning::RcCycle {
            name: "Node".into(),
            reason: "field contains List<Self>".into(),
            span: span_from(0, 4),
        }
        .into();
        assert_eq!(
            wn.severity().map(|s| format!("{:?}", s)).as_deref(),
            Some("Warning")
        );
    }

    #[test]
    fn lex_error_mapping_preserved() {
        // golden: from_lex_error_structured——结构化 → AutoError::Lexer 同一类型
        let structured: AutoError = LexError::Lexer(LexerError::UnknownCharacter {
            character: "$".into(),
            span: span_from(1, 1),
        })
        .into();
        assert_eq!(variant(&structured), "Lexer");
        assert_eq!(
            structured.code().map(|c| c.to_string()).as_deref(),
            Some("auto_lexer_E0005")
        );
        // golden: lex_raw_via_from——Raw → Msg，无诊断 code
        let raw: AutoError = LexError::Raw("raw block not closed".into()).into();
        assert_eq!(variant(&raw), "Msg");
        assert_eq!(raw.to_string(), "raw block not closed");
        assert!(raw.code().is_none());
    }

    #[test]
    fn from_conversions_and_question_propagation() {
        fn propagate(s: String) -> Result<usize, AutoError> {
            let n: usize = s
                .trim()
                .parse::<usize>()
                .map_err(|e| AutoError::from(e.to_string()))?;
            Ok(n + 1)
        }
        assert!(propagate("41".into()).is_ok());
        let err = propagate("not-a-number".into()).unwrap_err();
        assert_eq!(variant(&err), "Msg");
        assert_eq!(err.to_string(), "invalid digit found in string");

        let io_err: AutoError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "config not found").into();
        assert_eq!(variant(&io_err), "Io");
        assert_eq!(io_err.to_string(), "config not found");
    }

    #[test]
    fn attach_source_and_rendering_standalone() {
        // golden: attach_syntax + attach_msg_zero_span
        let attached = attach_source(
            SyntaxError::Generic {
                message: "expected expression".into(),
                span: span_from(5, 7),
            }
            .into(),
            "bad.at".into(),
            "let conut = 1\nlet count = 2".into(),
        );
        assert_eq!(variant(&attached), "SyntaxWithSource");
        assert!(attached.source_code().is_some());
        let rendered = format_error(&attached);
        assert!(rendered.contains("bad.at"), "rendered={rendered}");
        assert!(rendered.contains("expected expression"));

        // Msg → MsgWithSource（零 span）
        let msg_attached = attach_source(
            AutoError::from("link failed for module 'pac'"),
            "link.at".into(),
            "let conut = 1".into(),
        );
        assert_eq!(variant(&msg_attached), "MsgWithSource");
        let labels: Vec<_> = msg_attached.labels().unwrap().collect();
        assert_eq!(labels.len(), 1);
        assert_eq!((labels[0].offset(), labels[0].len()), (0, 0));

        // 重复 attach 幂等（golden: attach_double_idempotent）
        let twice = attach_source(attached, "other.at".into(), "让 x = 1".into());
        assert_eq!(variant(&twice), "SyntaxWithSource");
        let rendered2 = format_error(&twice);
        assert!(
            rendered2.contains("bad.at"),
            "second attach must not replace source"
        );
    }

    #[test]
    fn suggestion_helpers_and_span_helpers() {
        assert_eq!(
            find_best_match("conut", &["count".to_string(), "account".to_string()]),
            Some("count".to_string())
        );
        assert_eq!(find_best_match("zzzzz", &["count".to_string()]), None);
        assert_eq!(
            find_best_match("prinln", &["println".to_string()]),
            Some("println".to_string())
        );

        let span = pos_to_span(Pos {
            line: 5,
            at: 10,
            pos: 100,
            len: 5,
        });
        assert_eq!((span.offset(), span.len()), (100, 5));
        assert_eq!((span_from(0, 3).offset(), span_from(0, 3).len()), (0, 3));
    }
}
