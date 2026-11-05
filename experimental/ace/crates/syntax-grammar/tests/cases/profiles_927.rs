//! PLAN-927 T-04/T-05：`profiles` 裁剪编译。共享 auto-source.at → 七个
//! profile 专属语法程序；身份隔离（GrammarId 互异）、门控语义（被裁
//! alt/rule/op 不在 canonical bytes）、误用（plain compile 门控语法）拒绝。
//! PLAN-928（T-03/T-09）：集合扩至九（collections/closures）。

use ace_syntax_grammar::{compile, compile_profile, declared_profiles};

const SOURCE: &str = include_str!("../../../../grammars/auto-source.at");

const NINE: &[&str] = &[
    "core",
    "scalar",
    "text",
    "record",
    "text-record",
    "entry-unit",
    "host-console",
    // PLAN-928：集合/闭包两面。
    "collections",
    "closures",
    // PLAN-935：有限模板面（scalar 全表面 + fn <T>；仅实例能力前门使用；
    // 声明序在闭包之后）。
    "scalar-instances",
    // PLAN-950: explicit finite public-function module capability.
    "scalar-modules",
    // PLAN-936（T-04）：有限 CT 表达式面（scalar-instances 全表面 +
    // #{ expr } 原子；仅 CT 能力前门使用；声明序在模板面之后）。
    "scalar-ct",];

#[test]
fn profiles_declared_set_is_exact() {
    let names = declared_profiles(SOURCE).expect("declared profiles");
    assert_eq!(names, NINE);
}

#[test]
fn profiles_compile_all_and_ids_isolate() {
    let mut ids = Vec::new();
    for p in NINE {
        let prog = compile_profile(SOURCE, p).expect(p);
        assert_eq!(prog.name, format!("auto-source/{p}"), "{p} program name");
        assert!(!prog.rules.is_empty(), "{p} rules");
        ids.push(format!("{}", prog.identity.grammar_id.0));
    }
    // 身份隔离：各 GrammarId 互异（canonical bytes 不同）。
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), NINE.len(), "grammar ids must be distinct");
}

#[test]
fn profiles_trim_core_wide_ops_and_while_swap() {
    let core = compile_profile(SOURCE, "core").expect("core");
    let scalar = compile_profile(SOURCE, "scalar").expect("scalar");
    // core 表只有 `<` `+` `*` 三个算子；wide 表十三个。
    let core_table = &core.pratt[0];
    assert_eq!(core_table.ops.len(), 3, "core ops = < + *");
    let scalar_table = &scalar.pratt[0];
    assert_eq!(scalar_table.ops.len(), 13, "wide ops = full set");
    // while-stmt 只在 core；for-cond-stmt 不在 core。
    let core_rules: Vec<&str> = core.rules.iter().map(|r| r.name.as_str()).collect();
    let scalar_rules: Vec<&str> = scalar.rules.iter().map(|r| r.name.as_str()).collect();
    assert!(core_rules.contains(&"while-stmt"));
    assert!(!core_rules.contains(&"for-cond-stmt"));
    assert!(!scalar_rules.contains(&"while-stmt"));
    assert!(scalar_rules.contains(&"for-cond-stmt"));
    // ctor/leading-dot 只在 record 族。
    assert!(!scalar_rules.contains(&"ctor-expr"));
    let record = compile_profile(SOURCE, "record").expect("record");
    let record_rules: Vec<&str> = record.rules.iter().map(|r| r.name.as_str()).collect();
    assert!(record_rules.contains(&"ctor-expr"));
    assert!(record_rules.contains(&"leading-dot"));
    // script-item 只在 entry-unit。
    let entry = compile_profile(SOURCE, "entry-unit").expect("entry-unit");
    let entry_rules: Vec<&str> = entry.rules.iter().map(|r| r.name.as_str()).collect();
    assert!(entry_rules.contains(&"script-item"));
    assert!(!record_rules.contains(&"script-item"));
}

#[test]
fn profiles_plain_compile_of_gated_grammar_is_refused() {
    let err = compile(SOURCE).expect_err("plain compile must refuse gated grammar");
    assert!(
        err.iter().any(|d| d.message.contains("compile_profile")),
        "diags must name compile_profile: {err:?}"
    );
}

#[test]
fn profiles_unknown_profile_is_refused() {
    // PLAN-928：`collections` 已是真声明 profile（T-03），本例改用仍非法的
    // slug——前提与 auto-source.at 的 profiles 声明同步。
    let err = compile_profile(SOURCE, "mega-profile").expect_err("undeclared profile");
    assert!(err.iter().any(|d| d.message.contains("not declared")));
}
