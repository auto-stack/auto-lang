# PLAN-741 Phase 8(r8)引用身份与控制执行修复验证报告(2026-10-06)

r8 基线:worktree `D:/autostack/.wt/lang-741/auto-lang` 分支 `plan-741-dev`,
基线提交 `d59298e95`(v0.6-dev 含复审方 r8 再激活提交)。本 Phase 仅
fixture_tests.py / final_assertions.py 两个 Python 验证脚本修复,**零 Rust/
依赖/canonical 改动**(合同 §Phase 8);组内只读依赖 auto-down(detached @ fba6563e)。

## 修复摘要与 AC 对账

| 项 | 修复 | 验证 |
|---|---|---|
| R7-QA-01(P3) | **导航控制真正执行**:两矩阵各新增 2 个外部落点用例(nav-controls-all-correct → exit 0;nav-controls-forgotten-nav-update → exit 1 且诊断定位 auto-ac/plans.md:)加入统一 cases 列表经 run_assert 真实执行;**总数从执行记录派生**(main 中固定 +2 已删除) | --source auto 实测:总执行数 25(含 4 个导航控制),FIXTURES-ALL-PASS;若门无法检出遗忘导航,该用例即失败→整个 fixture 非零(负例即元控制) |
| R7-QA-02(P3) | **引用存在性 + 目标身份检查**(final_assertions v4):①推导 canonical 目标(docs/plans[/archive]/741-ac-hir-native-core.md);②prototype README **必须存在 741 计划引用**——一个都没有即失败(R7-QA-02 反例);③README/两模块导航行的 741 引用解析后**必须等于 canonical 目标**(不等→失败,含 expected/got);④ledger 全部 P741-* review 指针同样逐项等于 canonical(激活期 canonical_archive 尚不存在=已列暂态);⑤失败均带文件/行号/entry ID | 身份负例 4 个沙盒 repo 全部正确非零:nav href 错目录(741 文件名保留)→ does not match the canonical plan;README 缺引用 → missing required 741 plan reference;README 错目录 → does not match the canonical plan target;ledger P741-8 file 错指 → P741-8 does not match the canonical plan target(含 entry ID) |

## 门禁证据(全部 worktree 实跑,2026-10-06;Category A 零 Cargo/docs_gen)

- fixture 矩阵:`--source auto` **25 用例执行全 PASS**——active 源 13 用例
  (2 正例 + 8 负例 + 2 导航控制 + identity-nav-wrong-target)、archive 源(git 历史重放
  r6 归档态)12 用例(2 正例 + 6 负例 + 2 导航控制 + identity-ledger-wrong-target、
  identity-readme-wrong-target);
- final_assertions v4 active 模式:ALL-ASSERTIONS-PASS(README/导航行/ledger 身份全过);
- 生产面零 diff:`git diff d59298e95 -- Cargo.toml Cargo.lock crates/ test/
  experimental/ac-core/src` 为空;
- 原型/60s/root 证据:按 r8 合同以源指纹复用 r7 复审记录(src 相对 r5..r7 零改动,
  r7 复审 source-manifest 绑定仍有效)。

## 遗留

- 归档模式断言由 merge 在真实归档(链接转换后)复跑(届时 current_step 44/44、
  全部任务勾选、README/导航 delivered+archive 形、全部 P741 指针归档可解析)。
- root daily 37 红(8 例超历史并集)非 741 归因(r6..r8 零 root 改动),保持 r6 复审
  记录口径。
