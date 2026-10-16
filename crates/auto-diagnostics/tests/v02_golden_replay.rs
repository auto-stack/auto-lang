//! PLAN-793 V02 golden 重放（AC-02）。
//!
//! 期望值来自**冻结件** `docs/plans/attachments/793/v02-golden-frozen.json`
//! （基面 579f6dec0 上旧 `auto_lang::error` 公开面生成，38 例/12 族，
//! SHA256 `e97b9450…2a09d`；本 crate `tests/fixtures/v02-golden.json` 为其
//! 字节副本，一致性由复审侧核对），当前实现（auto-diagnostics 抽取版）
//! 的输出与之逐字段比对——display/code/severity/help/url/labels/
//! related/debug 与固定宽度(60)/无色 GraphicalReportHandler 渲染。
//! 新实现不参与期望生成（oracle 界，V02）。

use auto_diagnostics::error::{
    attach_source, AutoError, ComptimeError, NameError, RuntimeError, SyntaxError, TypeError,
    Warning,
};
use auto_lex::error::{LexError, LexerError};
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme, SourceSpan};
use serde_json::{json, Value};

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("v02-golden.json")
}

fn render_fixed(err: &AutoError) -> String {
    let handler = GraphicalReportHandler::new()
        .with_width(60)
        .with_theme(GraphicalTheme::unicode_nocolor());
    let mut out = String::new();
    handler.render_report(&mut out, err).expect("render report");
    out
}

fn labels_json(err: &AutoError) -> Value {
    match err.labels() {
        None => Value::Null,
        Some(iter) => json!(iter
            .map(|ls| json!({
                "text": ls.label(),
                "offset": ls.offset(),
                "len": ls.len(),
            }))
            .collect::<Vec<_>>()),
    }
}

fn related_codes(err: &AutoError) -> Value {
    match err.related() {
        None => Value::Null,
        Some(iter) => json!(iter
            .map(|d| d.code().map(|c| c.to_string()))
            .collect::<Vec<_>>()),
    }
}

fn proj_json(err: &AutoError) -> Value {
    json!({
        "display": err.to_string(),
        "code": err.code().map(|c| c.to_string()),
        "severity": err.severity().map(|s| format!("{:?}", s)),
        "help": err.help().map(|h| h.to_string()),
        "url": err.url().map(|u| u.to_string()),
        "labels": labels_json(err),
        "related_codes": related_codes(err),
        "debug": format!("{:?}", err),
        "rendered": render_fixed(err),
    })
}

fn span(offset: usize, len: usize) -> SourceSpan {
    SourceSpan::new(offset.into(), len.into())
}

fn replay_case(id: &str) -> Value {
    let src_a = "let conut = 1\nlet count = 2";
    let src_cjk = "让 x = 「你好世界」\n返回 x";
    let cjk_span = span(11, 12);

    let err: AutoError = match id {
        "syntax_unexpected_token" => SyntaxError::UnexpectedToken {
            expected: "';'".into(),
            found: "'let'".into(),
            span: span(12, 3),
        }
        .into(),
        "syntax_generic" => SyntaxError::Generic {
            message: "expected expression".into(),
            span: span(5, 7),
        }
        .into(),
        "syntax_unterminated_string" => SyntaxError::UnterminatedString { span: span(0, 6) }.into(),
        "lexer_unknown_escape" => LexerError::UnknownEscapeSequence {
            sequence: "\\q".into(),
            span: span(3, 2),
        }
        .into(),
        "lexer_unterminated_char" => LexerError::UnterminatedChar { span: span(7, 4) }.into(),
        "lex_raw_via_from" => AutoError::from(LexError::Raw(
            "raw block not closed: missing '}' for style block".into(),
        )),
        "type_mismatch" => TypeError::Mismatch {
            expected: "Int".into(),
            found: "String".into(),
            span: span(9, 5),
        }
        .into(),
        "type_cannot_modify_view_param_cjk" => TypeError::CannotModifyViewParam {
            param: "计数器".into(),
            span: span(4, 9),
        }
        .into(),
        "type_constraint_violation" => TypeError::ConstraintViolation {
            type_name: "MyType".into(),
            spec_name: "Drawable".into(),
            span: span(20, 6),
        }
        .into(),
        "name_undefined_with_suggestion" => NameError::undefined_variable(
            "conut".into(),
            span(4, 5),
            &["count".to_string(), "account".to_string()],
        )
        .into(),
        "name_undefined_no_suggestion" => {
            NameError::undefined_variable("zzzzz".into(), span(4, 5), &["count".to_string()]).into()
        }
        "name_duplicate_with_original_span" => NameError::DuplicateDefinition {
            name: "x".into(),
            span: span(30, 1),
            original_span: Some(span(4, 1)),
        }
        .into(),
        "name_immutable_assignment" => NameError::ImmutableAssignment {
            name: "total".into(),
            span: span(15, 5),
        }
        .into(),
        "name_undefined_function_suggestion" => {
            NameError::undefined_function("prinln".into(), span(0, 6), &["println".to_string()])
                .into()
        }
        "runtime_division_by_zero" => RuntimeError::DivisionByZero { span: span(8, 1) }.into(),
        "runtime_index_out_of_bounds" => RuntimeError::IndexOutOfBounds {
            index: 7,
            len: 3,
            span: span(2, 3),
        }
        .into(),
        "runtime_out_of_memory_no_span" => RuntimeError::OutOfMemory.into(),
        "comptime_compile_error" => ComptimeError::CompileError {
            message: "feature disabled in strict mode".into(),
            span: span(10, 13),
        }
        .into(),
        "comptime_resource_limit" => ComptimeError::ResourceLimit {
            limit: "steps".into(),
            value: "1000000".into(),
            max: "65536".into(),
            span: span(0, 5),
        }
        .into(),
        "warning_unused_variable" => Warning::UnusedVariable {
            name: "unused_var".into(),
            span: span(4, 10),
        }
        .into(),
        "warning_suspicious_block_keyword" => Warning::SuspiciousBlockKeyword {
            found: "veiw".into(),
            suggestion: "view".into(),
            span: span(0, 4),
        }
        .into(),
        "warning_escape_fallback" => Warning::EscapeFallback {
            name: "items".into(),
            reason: "borrowed across await".into(),
            tier_desc: "Rc<RefCell<T>> (Tier 3)".into(),
            span: span(6, 5),
        }
        .into(),
        "warning_rc_cycle" => Warning::RcCycle {
            name: "Node".into(),
            reason: "field contains List<Self>".into(),
            span: span(0, 4),
        }
        .into(),
        "from_string" => AutoError::from(String::from("module root not found")),
        "from_str" => AutoError::from("static message"),
        "from_io_custom" => AutoError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "config not found",
        )),
        "from_lex_error_structured" => {
            AutoError::from(LexError::Lexer(LexerError::UnknownCharacter {
                character: "$".into(),
                span: span(1, 1),
            }))
        }
        "attach_syntax" => attach_source(
            AutoError::from(SyntaxError::Generic {
                message: "expected expression".into(),
                span: span(5, 7),
            }),
            "bad.at".into(),
            src_a.to_string(),
        ),
        "attach_double_idempotent" => attach_source(
            attach_source(
                AutoError::from(SyntaxError::Generic {
                    message: "expected expression".into(),
                    span: span(5, 7),
                }),
                "bad.at".into(),
                src_a.to_string(),
            ),
            "other.at".into(),
            src_cjk.to_string(),
        ),
        "attach_msg_zero_span" => attach_source(
            AutoError::from("link failed for module 'pac'"),
            "link.at".into(),
            src_a.to_string(),
        ),
        "attach_lexer_structured" => attach_source(
            LexerError::UnknownEscapeSequence {
                sequence: "\\q".into(),
                span: span(3, 2),
            }
            .into(),
            "esc.at".into(),
            "let c = '\\q'".to_string(),
        ),
        "with_source_constructor" => AutoError::with_source(
            SyntaxError::MissingDelimiter {
                delimiter: "}".into(),
                context: "block".into(),
                span: span(0, 1),
            },
            "block.at".into(),
            "{ let x = 1".to_string(),
        ),
        "with_source_lexer_constructor" => AutoError::with_source_lexer(
            LexerError::UnterminatedChar { span: span(7, 4) },
            "char.at".into(),
            "let c = 'ab".to_string(),
        ),
        "unicode_cjk_span_attached" => attach_source(
            AutoError::from(SyntaxError::Generic {
                message: "此处需要表达式".into(),
                span: cjk_span,
            }),
            "cjk.at".into(),
            src_cjk.to_string(),
        ),
        "unicode_cjk_type_param_bare" => TypeError::CannotModifyViewParam {
            param: "視圖參數".into(),
            span: cjk_span,
        }
        .into(),
        "multiple_mixed_bare" => AutoError::MultipleErrors {
            count: 3,
            plural: "s".into(),
            errors: vec![
                SyntaxError::UnexpectedToken {
                    expected: "')'".into(),
                    found: "'}'".into(),
                    span: span(10, 1),
                }
                .into(),
                NameError::undefined_variable("conut".into(), span(4, 5), &["count".to_string()])
                    .into(),
                AutoError::Warning(Warning::UnusedVariable {
                    name: "tmp".into(),
                    span: span(20, 3),
                }),
            ],
        },
        "multiple_attach_nested" => attach_source(
            AutoError::MultipleErrors {
                count: 3,
                plural: "s".into(),
                errors: vec![
                    SyntaxError::UnexpectedToken {
                        expected: "')'".into(),
                        found: "'}'".into(),
                        span: span(10, 1),
                    }
                    .into(),
                    NameError::undefined_variable(
                        "conut".into(),
                        span(4, 5),
                        &["count".to_string()],
                    )
                    .into(),
                    AutoError::Warning(Warning::UnusedVariable {
                        name: "tmp".into(),
                        span: span(20, 3),
                    }),
                ],
            },
            "multi.at".into(),
            src_a.to_string(),
        ),
        "multiple_single_no_plural" => AutoError::MultipleErrors {
            count: 1,
            plural: String::new(),
            errors: vec![RuntimeError::DivisionByZero { span: span(8, 1) }.into()],
        },
        other => panic!("unknown golden case id: {other}"),
    };
    proj_json(&err)
}

#[test]
fn v02_golden_replay() {
    let frozen: Value =
        serde_json::from_slice(&std::fs::read(fixture_path()).expect("fixture json"))
            .expect("parse fixture");
    let cases = frozen.as_array().expect("cases array");
    assert_eq!(cases.len(), 38, "frozen case count");

    let mut mismatches = Vec::new();
    for case in cases {
        let id = case["id"].as_str().expect("id");
        let family = case["family"].as_str().expect("family");
        let expect = &case["proj"];
        let actual = replay_case(id);
        if expect != &actual {
            mismatches.push(format!(
                "case {} (family {}):\n  expect: {}\n  actual: {}",
                id, family, expect, actual
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "V02 golden mismatch ({} cases):\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
