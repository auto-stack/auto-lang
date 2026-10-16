//! PLAN-789 V02 golden 重放（AC-02）。
//!
//! 期望值来自**冻结件** `docs/plans/attachments/789/v02-golden-frozen.json`
//! （基面 e4ac72a62 上旧 `auto-lang::lexer::Lexer` 生成，v2，
//! SHA256 `37beb238…dc21`；本 crate `tests/fixtures/v02-golden.json` 为其
//! 字节副本，一致性由复审侧核对），当前实现（auto-lex 抽取版）的输出与之
//! 逐字段比对——kind(Debug)/text/line/at/pos/len 与错误投影
//! code/display/debug。新实现不参与期望生成（oracle 界，V02）。

use auto_lex::error::{LexError, LexResult};
use auto_lex::lexer::Lexer;
use auto_lex::token::TokenKind;
use miette::Diagnostic;
use serde_json::{json, Value};

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("v02-golden.json")
}

fn token_json(t: &auto_lex::token::Token) -> Value {
    json!({
        "kind": format!("{:?}", t.kind),
        "text": t.text.to_string(),
        "line": t.pos.line,
        "at": t.pos.at,
        "pos": t.pos.pos,
        "len": t.pos.len,
    })
}

fn error_json(e: &LexError) -> Value {
    match e {
        LexError::Lexer(le) => json!({
            "kind": "lexer",
            "code": le.code().map(|c| c.to_string()),
            "display": le.to_string(),
            "debug": format!("{:?}", le),
        }),
        LexError::Raw(m) => json!({
            "kind": "msg",
            "display": m,
        }),
    }
}

fn replay(case: &Value) -> Value {
    let input = case["input"].as_str().expect("input");
    let mode = case["mode"].as_str().expect("mode");
    let mut lexer = Lexer::new(input);
    if let Some(note) = case.get("note").and_then(|n| n.as_str()) {
        let c = note.chars().next().expect("note char");
        lexer.set_fstr_note(c);
    }
    let mut record = json!({});
    match mode {
        "raw_block" => {
            let first: LexResult<_> = lexer.next();
            assert_eq!(first.expect("first token").kind, TokenKind::LBrace);
            match lexer.capture_raw_block() {
                Ok(raw) => record["raw"] = json!(raw),
                Err(e) => record["error"] = error_json(&e),
            }
        }
        _ => {
            let mut tokens = Vec::new();
            let mut err = Value::Null;
            loop {
                match lexer.next() {
                    Ok(t) => {
                        let is_eof = t.kind == TokenKind::EOF;
                        tokens.push(token_json(&t));
                        if is_eof {
                            break;
                        }
                    }
                    Err(e) => {
                        err = error_json(&e);
                        break;
                    }
                }
            }
            record["tokens"] = Value::Array(tokens);
            record["error"] = err;
        }
    }
    record
}

#[test]
fn v02_golden_replay_matches_frozen_arm() {
    let bytes = std::fs::read(fixture_path()).expect("fixture readable");
    let parsed: Value = serde_json::from_slice(&bytes).expect("fixture valid JSON");
    let cases = parsed.as_array().expect("cases array");
    // 语料规模钉：冻结件为 34 例（防期望侧被部分截断/扩充后静默对拍）。
    // fixture 与冻结件的字节一致性（SHA256 37beb238…dc21）由 PLAN-789 复审
    // 侧核对，见 docs/plans/attachments/789/t01-frontend-dag.md；fixture 以
    // `git add -f` 入库（.gitignore *.json 的精确例外，F-789-R1-01），
    // 干净检出可重放。
    assert_eq!(cases.len(), 34, "frozen corpus is 34 cases");

    let mut mismatches = Vec::new();
    for case in cases {
        let id = case["id"].as_str().expect("case id");
        let actual = replay(case);
        for field in ["tokens", "error", "raw"] {
            let expected = &case[field];
            if !expected.is_null() || !actual[field].is_null() {
                if expected != &actual[field] {
                    mismatches.push(format!(
                        "case `{}` field `{}`:\n  expected: {}\n  actual:   {}",
                        id,
                        field,
                        serde_json::to_string_pretty(expected).unwrap_or_default(),
                        serde_json::to_string_pretty(&actual[field]).unwrap_or_default(),
                    ));
                }
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "V02 golden replay mismatches ({}):\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
