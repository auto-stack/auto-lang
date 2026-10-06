# PLAN-741 Phase 6(r6)生命周期断言与证据版本修复验证报告(2026-10-06)

r6 基线:worktree `D:/autostack/.wt/lang-741/auto-lang` 分支 `plan-741-dev`,
基线提交 `ad1fe269c`(v0.6-dev 含复审方 r6 再激活提交)。源码指纹见
[source-hashes.txt](source-hashes.txt);experimental/ac-core/src、Cargo.toml/lock
相对 r5 **零改动**(本 Phase 仅文档/验证修复,合同 §Phase 6 明确);组内只读依赖
`D:/autostack/.wt/lang-741/auto-down`(detached @ fba6563e,同 r1..r5)。

## 修复摘要与 AC 对账

| 项 | 修复 | 验证 |
|---|---|---|
| R5-QA-01(P3) | **final_assertions v2**(`docs/reports/741-phase6-lifecycle-gate/final_assertions.py`,显式替代 phase5 版):解析 frontmatter 必需字段(plan_id/status/plan_revision/current_step/total_steps,缺失/非数字即失败);**current_step == 完成唯一任务数、total_steps == 唯一任务总数(由当前合同派生,无 35/38 硬编码)**;archived 必须全完成且位于 archive;同 ID 勾选冲突失败 | 正负 fixture 矩阵(`fixture_tests.py`,真实子进程退出码)8/8:valid=0;current_step=0/total_steps=999/缺失字段/非数字 revision/激活位 archived 状态/同 ID 冲突 → 全部非零 |
| R5-QA-02(P3) | **active 模式先验真实父目录**(docs/plans/,链接为 active 形 `../reports`——计划 7→12 处链接已在激活时回退为 active 形),再验**显式转换后的模拟归档副本**(../reports→../../reports,自 archive 父目录解析);两者都必须通过,模拟不再冒充实际 active;归档后按真实父目录验转换后文件(merge 收口) | active 模式实测:real active parent 10 链接解析 + simulated archive 10 链接解析后转换双过;fixture negative-active-with-archive-form-links(复刻 QA-02 缺陷形态)正确非零 |
| R5-QA-03(P3) | **证据版本绑定**:采纳复审的 reproduce-exact-artifact.ps1(署名来源,verbatim)+ link-cleanup-probe.rs——库经 `cargo build --lib --bins --message-format=json` 按 package/target/profile 唯一绑定(无 rlib glob),Cargo 输出与 artifact 哈希落 scratch;**计数更正**:r5 verification.md 标注 54→**58**(lib15+43,日志派生),归档计划注记随 r6 归档落位 | exact-artifact 全量重放(-ProductionDeadline):nonzero 与 60s timeout 两出口 link.cleanup 全要素(os error 32/残留路径/NOT committed)、旧三件套不变、old exe exit 5、released_cleanup_ok=true;public-deadline 60.063s、后代 death-watch 232ms 回收([exact-artifact-rerun.txt](exact-artifact-rerun.txt)) |

## 门禁证据(按 r6 合同以指纹复用 r5 证据;本轮变更面=文档/验证脚本零 Rust)

- r5 复审已实测并被本轮指纹核对覆盖:原型 **58 通过 + 1 ignored**(lib 15 + 其余 43)、
  一键 13/13、fmt/all-targets 原型零 warning、正式 60s(60016/60095ms)+ 共享锁 77ms 反例
  ——源/Spec/计划指纹见 [source-hashes.txt](source-hashes.txt) 与复审
  [source-manifest](../741-r5-quality-review-20261006/source-manifest.json)
  (link.rs/main.rs/verify.rs/native.rs 相对 r5 零改动,本轮复跑 final_assertions +
  fixture + exact-artifact 三件均为脚本/文档面)。
- 生产面零 diff:`git diff ad1fe269c -- Cargo.toml Cargo.lock crates/ test/ experimental/ac-core/src`
  为空。
- r6 断言 active 模式:ALL-ASSERTIONS-PASS(见上);归档模式由 merge 在真实归档后复跑
  (届时 current_step 38/38、全部勾选、链接已转换、7 项 P741 review 指针归档可解析)。
- root 门禁:复审本轮已记录 root check 0/t 100(37 红,8 例超出历史并集待归因)/tv 0
  ——r6 零 Rust 改动,不做新归因;既有红不因文档 phase 冒充解决。

## 遗留

- 归档模式断言 + 链接转换由 merge 在真实归档时执行(final_assertions --location archive)。
- root daily 37 红中 8 例超出历史并集:非 741 归因(741 零 root 改动),按 THR-D3/D4 与
  并行计划影响在 root 门禁自有流程中归因,不在本 Phase 扩围。
