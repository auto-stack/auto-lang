//! PLAN-982（EXEC · round 189）：`AceTestRunRequest::filter`——legacy
//! `test --filter` 契约在 ACE 测试门的落点：
//! ①**执行前筛选**（子串匹配于限定名 `module::name`；未选中的测试**不执行**
//! ——legacy 是跑完再跳报告，ACE 车道不执行被排除者）；
//! ②**反控**：选中 `good` 过、而同一工程不过滤时 `bad_check` 失败（证明
//! 未选中者若运行就会失败，过滤真实起效）；
//! ③**零匹配**：过滤词无命中 ⇒ 什么都不跑、`Passed`、退出码 0（legacy
//! 退出语义），`discovered` 仍携带完整生产者事实；
//! ④选中失败者 ⇒ `Failed`、退出码 1；
//! ⑤限定名形（`app::filt::good`）与裸名形同命中。

use auto_lang_ace_host::install;
use auto_lang_core::{
    run_ace_project_tests, AceEngineChoice, AceModuleSpec, AceTestRunOutcome, AceTestRunRequest,
};

const SOURCE: &str = "fn main() int { return 0 }\nfn good_check() int { return 0 }\nfn bad_check() int { return 1 }\n";

fn request<'a>(
    specs: &'a [AceModuleSpec<'a>],
    capabilities: &'a [String],
    filter: Option<&'a str>,
) -> AceTestRunRequest<'a> {
    AceTestRunRequest {
        modules: specs,
        entry_module: "app::filt",
        entry_def_id: "d_main",
        capabilities,
        filter,
        wall: None,
        cancel: None,
    }
}

#[test]
fn p982_test_filter_selects_before_execution() {
    install().expect("executor installs");
    let specs = [AceModuleSpec {
        package: "app",
        version: "1.0.0",
        id: "filt",
        source: SOURCE,
    }];
    let capabilities = vec!["ac.source.modules.packages/v1".to_string()];

    // 反控基座：不过滤 ⇒ bad_check 失败（未选中者若运行就会失败）。
    let unfiltered = run_ace_project_tests(
        AceEngineChoice::AceAV,
        &request(&specs, &capabilities, None),
    );
    assert!(
        matches!(unfiltered.outcome, AceTestRunOutcome::Failed { ref message } if message.contains("bad_check")),
        "the unfiltered run fails on bad_check: {:?}",
        unfiltered.outcome
    );
    assert_eq!(unfiltered.exit_code, 1);
    assert_eq!(unfiltered.tests.len(), 2, "both tests ran unfiltered");

    // (1) 选中 good：bad_check 不执行 ⇒ 全过、退出 0。
    let good = run_ace_project_tests(
        AceEngineChoice::AceAV,
        &request(&specs, &capabilities, Some("good")),
    );
    assert!(
        matches!(good.outcome, AceTestRunOutcome::Passed),
        "{:?}",
        good.outcome
    );
    assert_eq!(good.exit_code, 0);
    assert_eq!(good.tests.len(), 1, "only the selected test runs");
    assert_eq!(good.tests[0].name, "good_check");
    assert!(good.tests[0].passed);
    // discovered 仍是完整生产者事实（未因过滤收缩）。
    assert_eq!(
        good.discovered.len(),
        2,
        "discovery stays the full producer fact"
    );

    // (2) 限定名形（module::name 子串）与裸名同命中。
    let qualified = run_ace_project_tests(
        AceEngineChoice::AceAV,
        &request(&specs, &capabilities, Some("app::filt::good")),
    );
    assert!(
        matches!(qualified.outcome, AceTestRunOutcome::Passed),
        "{:?}",
        qualified.outcome
    );
    assert_eq!(qualified.tests.len(), 1);
    assert_eq!(qualified.tests[0].name, "good_check");

    // (3) 零匹配：什么都不跑、Passed、退出 0，discovered 完整。
    let zero = run_ace_project_tests(
        AceEngineChoice::AceAV,
        &request(&specs, &capabilities, Some("no-such-test")),
    );
    assert!(
        matches!(zero.outcome, AceTestRunOutcome::Passed),
        "{:?}",
        zero.outcome
    );
    assert_eq!(zero.exit_code, 0);
    assert!(
        zero.tests.is_empty(),
        "nothing executes on a zero-match filter"
    );
    assert_eq!(zero.discovered.len(), 2);

    // (4) 只选中失败者 ⇒ Failed、退出 1。
    let bad = run_ace_project_tests(
        AceEngineChoice::AceAV,
        &request(&specs, &capabilities, Some("bad")),
    );
    assert!(
        matches!(bad.outcome, AceTestRunOutcome::Failed { ref message } if message.contains("bad_check")),
        "{:?}",
        bad.outcome
    );
    assert_eq!(bad.exit_code, 1);
    assert_eq!(bad.tests.len(), 1);
    assert_eq!(bad.tests[0].name, "bad_check");
    assert!(!bad.tests[0].passed);
}
