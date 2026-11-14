//! PLAN-979 T-H0-I：Schema 驱动的槽位限定拼写（`contextual` 表 / `name(of:)` 步）。
//!
//! 主张（V-H0-SCHEMA / V-H0-POSITION 的语法层）：
//! - 同一拼写只在声明的槽位被解释为结构名，普通位置仍是普通标识符（词法面不变）；
//! - 换 Schema 拼写（隔离测试 Schema `view`→`panel`）只改接受面，不改运行时；
//! - 角色（role）与拼写解耦：换拼写后接受的仍是同一角色；
//! - 缺字节事实源 = 失败关闭（不静默接受），声明表参与身份/二进制往返。

use ace_syntax_grammar::compile_profile;

/// 最小 AGE 风格测试 Schema：普通语句面 + 三个槽位（decl 头 / view / text）。
const SCHEMA: &str = r#"
grammar(name: "ctx-demo", revision: 1, newline: true, stmt-end: ["semi"], sep: ["semi"]) {
    profiles(version: 1) {
        profile(name: "age") { }
        profile(name: "plain") { }
    }
    tokens {
        ws(trivia: true) { charset: " \t\r\n" }
        ident { charset: "a-zA-Z_", cont: "a-zA-Z0-9_" }
        lbrace { literal: "{" }
        rbrace { literal: "}" }
        semi { literal: ";" }
        str-lit { quote: "\"", escape: true }
    }
    contextual(version: 1) {
        name(node-head) { spelling: "node", token: "ident", role: "node", profiles: [age] }
        name(view-head) { spelling: "view", token: "ident", role: "view", profiles: [age] }
        name(text-head) { spelling: "text", token: "ident", role: "text", profiles: [age] }
    }
    rules {
        program { list { item() {} } }
        item {
            alt { plain-item() {} }
            alt(profiles: [age]) { node-item() {} }
        }
        plain-item { ident(field: "kw") ident(field: "name") semi() {} }
        node-item(profiles: [age]) {
            name(of: "node-head", field: "kw")
            ident(field: "name")
            lbrace(same-line: true)
            view-body() {}
            rbrace() {}
        }
        view-body(profiles: [age]) {
            name(of: "view-head", field: "component")
            lbrace(same-line: true)
            text-item() {}
            rbrace() {}
        }
        text-item(profiles: [age]) {
            name(of: "text-head", field: "component")
            str-lit(field: "text")
            lbrace(same-line: true)
            rbrace() {}
        }
    }
}
"#;

const NODE_SRC: &str = "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n";

struct ParseOutcome {
    errors: bool,
    diags: Vec<String>,
}

fn parse(profile: &str, text: &str) -> ParseOutcome {
    use ace_syntax_runtime::lexer::CompiledScanner;
    use ace_syntax_runtime::parse::parse_snapshot_with_ctx;
    use ace_syntax_runtime::program::Budget;
    use ace_syntax_runtime::snapshot::{PageStore, Snapshot};
    let p = compile_profile(SCHEMA, profile).expect("schema compiles for profile");
    let cs = CompiledScanner::compile(&p).expect("scanner");
    let snap = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), text);
    let ctx = TextSource { snap: &snap };
    let out = parse_snapshot_with_ctx(&p, &cs, &snap, &Budget::default(), Some(&ctx))
        .expect("parse (no resource failure)");
    ParseOutcome {
        errors: out.tree.has_errors(),
        diags: out
            .tree
            .diags
            .iter()
            .map(|d| format!("{}:{}..{}", d.message, d.start, d.end))
            .collect(),
    }
}

struct TextSource<'s> {
    snap: &'s ace_syntax_runtime::snapshot::Snapshot,
}

impl<'s> ace_syntax_runtime::parse::CtxTokenText for TextSource<'s> {
    fn token_equals(&self, start: u64, end: u64, want: &str) -> bool {
        self.snap
            .read_bytes(start, end)
            .map(|b| b == want.as_bytes())
            .unwrap_or(false)
    }
}

#[test]
fn p979_ctx_name_slot_accepts_declared_spelling_only() {
    let ok = parse("age", NODE_SRC);
    assert!(!ok.errors, "declared spelling must parse: {:?}", ok.diags);

    // 同槽位换拼写 → 该槽位不成立（不是"接受任意 ident"）。
    let src = "node Hello {\n    panel {\n        text \"Hello\" {}\n    }\n}\n";
    let bad = parse("age", src);
    assert!(
        bad.errors,
        "undeclared spelling must not be accepted at the slot"
    );
    let slot_at = src.find("panel").expect("panel offset");
    assert!(
        bad.diags.iter().any(|d| {
            d.contains("expected `view`") && d.contains(&format!(":{}..{}", slot_at, slot_at + 5))
        }),
        "rejection keeps the offending word's original span ({slot_at}): {:?}",
        bad.diags
    );
}

#[test]
fn p979_ctx_name_spelling_keeps_ordinary_identifier_role_free() {
    // 同一 `view` 拼写在普通语句位置仍是普通标识符（词法面没有被保留字化）。
    let ok = parse("age", "let view;\n");
    assert!(
        !ok.errors,
        "ordinary `view` binding must parse: {:?}",
        ok.diags
    );
    // 普通 profile 里同拼写同样合法（无 AGE 激活）。
    let plain = parse("plain", "let view;\n");
    assert!(!plain.errors, "plain profile keeps `view` ordinary");
}

#[test]
fn p979_schema_variant_changes_spelling_not_runtime() {
    // 隔离测试 Schema：节点头 `view` 配置为 `panel`（不改 runtime，只改 Schema 数据）。
    let variant = SCHEMA.replace(
        r#"name(view-head) { spelling: "view", token: "ident", role: "view", profiles: [age] }"#,
        r#"name(view-head) { spelling: "panel", token: "ident", role: "view", profiles: [age] }"#,
    );
    assert_ne!(variant, SCHEMA, "variant replacement applied");
    let p = compile_profile(&variant, "age").expect("variant compiles");
    // 角色仍按 Schema 声明（拼写换、角色不变）。
    let role = p
        .ctx_names
        .iter()
        .find(|n| n.name == "view-head")
        .map(|n| n.role.clone())
        .expect("view-head declared");
    assert_eq!(role, "view", "role is declared independently of spelling");

    use ace_syntax_runtime::lexer::CompiledScanner;
    use ace_syntax_runtime::parse::parse_snapshot_with_ctx;
    use ace_syntax_runtime::program::Budget;
    use ace_syntax_runtime::snapshot::{PageStore, Snapshot};
    let cs = CompiledScanner::compile(&p).unwrap();
    let run = |text: &str| -> bool {
        let snap = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), text);
        let ctx = TextSource { snap: &snap };
        let out = parse_snapshot_with_ctx(&p, &cs, &snap, &Budget::default(), Some(&ctx)).unwrap();
        out.tree.has_errors()
    };
    assert!(
        !run("node Hello {\n    panel {\n        text \"Hello\" {}\n    }\n}\n"),
        "variant-declared `panel` is accepted at the slot"
    );
    assert!(
        run("node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n"),
        "undeclared `view` is rejected in the variant schema"
    );
}

#[test]
fn p979_ctx_name_is_profile_gated_and_trimmed_reference_rejected() {
    // 未启用 profile：声明被裁剪、规则不进入程序。
    let plain = compile_profile(SCHEMA, "plain").expect("plain compiles");
    assert!(
        plain.ctx_names.is_empty(),
        "plain profile carries no contextual names"
    );
    assert!(
        !plain.rules.iter().any(|r| r.name == "node-item"),
        "plain profile carries no AGE rule"
    );
    // 引用被裁项是编译错误（不静默降级成 token 步）。
    let missing = SCHEMA.replace(
        r#"alt(profiles: [age]) { node-item() {} }"#,
        r#"alt { node-item() {} }"#,
    );
    let diags = compile_profile(&missing, "plain").expect_err("trimmed rule reference must error");
    assert!(
        diags.iter().any(|d| d.message.contains("trimmed")),
        "trimmed reference is named: {diags:?}"
    );
}

#[test]
fn p979_ctx_name_program_is_codec_and_identity_round_trip() {
    let p = compile_profile(SCHEMA, "age").expect("compiles");
    assert_eq!(p.ctx_names.len(), 3);
    assert_eq!(p.ctx_names[1].spelling, "view");
    assert_eq!(p.ctx_names[1].role, "view");
    let bytes = p.canonical_bytes();
    let q = ace_syntax_runtime::program_codec::decode_canonical(&bytes).expect("decode");
    assert_eq!(
        p.identity, q.identity,
        "identity binds the contextual table"
    );
    assert_eq!(p.ctx_names, q.ctx_names);
    // 换拼写 = 身份改变（领域包版本/拼写参与内容身份）。
    let variant = SCHEMA.replace(
        r#"spelling: "view", token: "ident", role: "view""#,
        r#"spelling: "panel", token: "ident", role: "view""#,
    );
    let v = compile_profile(&variant, "age").unwrap();
    assert_ne!(p.identity, v.identity);
}

#[test]
fn p979_ctx_name_fails_closed_without_token_text() {
    use ace_syntax_runtime::lexer::CompiledScanner;
    use ace_syntax_runtime::parse::parse_snapshot;
    use ace_syntax_runtime::program::Budget;
    use ace_syntax_runtime::snapshot::{PageStore, Snapshot};
    let p = compile_profile(SCHEMA, "age").unwrap();
    let cs = CompiledScanner::compile(&p).unwrap();
    let snap = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), NODE_SRC);
    let err = parse_snapshot(&p, &cs, &snap, &Budget::default())
        .expect_err("missing token text must not silently accept");
    assert!(
        format!("{err}").contains("token text source"),
        "fail-closed error is named: {err}"
    );
}
