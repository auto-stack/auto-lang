//! PLAN-927 T-04/T-05：`profiles` 裁剪编译。共享 auto-source.at → 七个
//! profile 专属语法程序；身份隔离（GrammarId 互异）、门控语义（被裁
//! alt/rule/op 不在 canonical bytes）、误用（plain compile 门控语法）拒绝。
//! PLAN-928（T-03/T-09）：集合扩至九（collections/closures）。

use ace_syntax_grammar::{compile, compile_profile, declared_profiles};

const SOURCE: &str = include_str!("../../../../grammars/auto-source.at");

const DECLARED: &[&str] = &[
    "core",
    "scalar",
    "text",
    "record",
    // PLAN-963（T-02）：记录有限模板面（record 全表面 + fn `<T>`/`<T has S>`），
    // 声明序紧接 record；961/962 合入时同款漏更新，963 接收侧按
    // auto-source.at 已声明现实刷新（合同语义不变）。
    "record-instances",
    // PLAN-963（T-02）：文本+记录有限模板面，声明序在 record-instances 之后、
    // text-record 之前。
    "text-record-instances",
    "text-record",
    // PLAN-982（B-CONFIG-I，E1）：配置文档面（text-record 全表面 ∪ 顶层脚本
    // 语句序列；声明序紧随 text-record——新增名字只进原本放行 text-record 的
    // 门控列表，既有 profile 的裁剪字节不变）。
    "config-document",
    // PLAN-979（T-H0-I）：AGE 最小 Node 领域面（text-record 全表面 + 领域节点
    // 声明，声明序紧随 text-record）。
    "age-h0",
    // PLAN-979（T-03）：AGE 组件面（`node { model … on … view … }`，同一
    // auto-source.at 的另一裁剪；声明序紧随 age-h0，H0 面逐字节不变）。
    "age-component",
    // PLAN-981（T-01）：AGE 多模块包面（包模块图 ∪ AGE 组件，同一
    // auto-source.at 的第三个 AGE 裁剪；声明序紧随 age-component，
    // H0/组件面逐字节不变）。
    "age-package",
    "entry-unit",
    "host-console",
    // PLAN-928：集合/闭包两面。
    "collections",
    "closures",
    // PLAN-961：数字面（scalar 全表面 + 类型词/数字算子 profile；声明序
    // 在闭包之后、模板面之前——961 合入时漏更新本冻结集，941 接收侧
    // 按已交付现实刷新，合同语义不变）。
    "numeric",
    // PLAN-962：聚合面（grid literal/hold/.view 等单元的声明清单；声明
    // 序在 numeric 之后、模板面之前——962 合入时同样漏更新本冻结集，
    // 1025 收口接收侧按已交付现实刷新，合同语义不变，同 961→941 先例）。
    "aggregates",
    // PLAN-1078：普通 AV 语言面（aggregates 面身份；声明序紧随
    // aggregates——`ac-av/language-source-run-v1` 的 Auto profile 裁剪）。
    "auto",
    // PLAN-1085: ordinary Auto bodies in the shared package graph.
    "auto-packages",
    // PLAN-968：调度器面（task/on 语句域；声明序紧随 aggregates——
    // 972 rebase 接收侧按 master 已交付现实刷新冻结集，合同语义不变）。
    "scheduler",
    // PLAN-935：有限模板面（scalar 全表面 + fn <T>；仅实例能力前门使用；
    // 声明序在闭包之后）。
    "scalar-instances",
    // PLAN-950: explicit finite public-function module capability.
    "scalar-modules",
    // PLAN-960: complete package-module capability (scalar-modules surface +
    // text/record + const/var items + use aliases/re-exports); declared
    // right after scalar-modules.
    "packages-modules",
    // PLAN-1043（T-03）：packages-ct 面（packages-modules 全表面 + `#in`
    // 编译期命名空间上下文 + 单行 use 符号表 + `pub spec fn` 需求声明）；
    // 仅 IN 能力前门使用；声明序紧随 packages-modules。
    "packages-ct",
    // PLAN-946（T-02）：packages-aggregates 面（packages-modules 全表面 +
    // 集合族最小语句面/可选类型位/List 接收原子）；946 合入门禁并集未含
    // ace-syntax，966 接收侧按 auto-source.at 已声明现实刷新（合同语义不变，
    // 同 961→941、962→1025 先例）。声明序紧随 packages-ct、scalar-ct 之前。
    "packages-aggregates",
    // PLAN-972（T-02）：packages-aggregates-ct 面（packages-aggregates 全表面
    // + packages-ct 装配项 + 闭包类型注解/字面量）；装配 aggregates 线
    //（std 装配标准库承载面）。声明序紧随 packages-aggregates。
    "packages-aggregates-ct",
    // PLAN-936（T-04）：有限 CT 表达式面（scalar-instances 全表面 +
    // #{ expr } 原子；仅 CT 能力前门使用；声明序在模板面之后）。
    "scalar-ct",
];

#[test]
fn profiles_declared_set_is_exact() {
    let names = declared_profiles(SOURCE).expect("declared profiles");
    assert_eq!(names, DECLARED);
}

#[test]
fn profiles_compile_all_and_ids_isolate() {
    let mut ids = Vec::new();
    for p in DECLARED {
        let prog = compile_profile(SOURCE, p).expect(p);
        assert_eq!(prog.name, format!("auto-source/{p}"), "{p} program name");
        assert!(!prog.rules.is_empty(), "{p} rules");
        ids.push(format!("{}", prog.identity.grammar_id.0));
    }
    // 身份隔离：各 GrammarId 互异（canonical bytes 不同）。
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), DECLARED.len(), "grammar ids must be distinct");
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

/// PLAN-963（T-02）：record 面新开泛型类型应用（`Point<int>` 使用位），
/// 但 `?T` 仍只在集合族面——新形状只到"显式类型实参"为止。
#[test]
fn profiles_record_opens_generic_ty_only() {
    let record = compile_profile(SOURCE, "record").expect("record");
    let record_rules: Vec<&str> = record.rules.iter().map(|r| r.name.as_str()).collect();
    assert!(record_rules.contains(&"generic-ty"));
    assert!(record_rules.contains(&"type-args"));
    assert!(record_rules.contains(&"type-arg"));
    assert!(!record_rules.contains(&"optional-ty"));
    let collections = compile_profile(SOURCE, "collections").expect("collections");
    let collections_rules: Vec<&str> = collections.rules.iter().map(|r| r.name.as_str()).collect();
    assert!(collections_rules.contains(&"optional-ty"));
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
