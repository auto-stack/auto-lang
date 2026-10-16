# PLAN-788 验收矩阵（T-08，2026-10-16）

聚合入口：`powershell -NoProfile -File scripts/verify-autodown-ac-788.ps1 -Scope acceptance`
（ALL PASS，2026-10-16 实测；零测族拒绝、失败计数、P95 硬门内嵌）。
`-Scope closeout` 为 merge-owned 冻结提交触面（t/tv/tt + worker/probe/ac-syntax 独立套件）。

| ID | AC | 反控/证据（测试入口→状态） |
|---|---|---|
| V-01 | AC-01 | auto-test-case 全量 88 绿（contract_v2_ 20：版本分派/未知字段/预算门/cache 参与门/JSON round-trip；contract_decode 19 v1 回归） |
| V-02 | AC-02 | protocol_t03_ 10/10（parse/resolve/typecheck 边界归因=观察器唯一来源；typed bind.dangling-ref+span）+ syntax_t06_ 7/7 + cache_t05_ typed 重放 + r1 backend-missing-entry typed（带 store） |
| V-03 | AC-02 | worker ac_native 面：ac_case_tests exit oracle+红兄（p01/p04 真源码→PE→run）；ac-native 语料 core 8 + scalar 3（exit 70/3/14/0/1/4/73） |
| V-04 | AC-03 | ac_product：语料 red-controls（金样 FAIL≠阻断真产物，消费者跑真 exe exit3）+ ac_case_tests product_handoff；r1_response_admission_table（schema/ok-error/run 形状 5 负例） |
| V-05 | AC-03 | link_capture_ 7/7（deadline 杀树/输出 cap 立即终止+精确边界/contained 拒绝重放）+ protocol_t03_ 真死循环 budget.run 杀 + review_r1_ 1ms 预算真构建必红 + consume 经 worker（F01） |
| V-06 | AC-03 | link_capture_ native_bytes_（无效 UTF-8/NUL/CRLF 字节级 round-trip 对照 lossy wrapper）；worker 路径非 ASCII 生产者随 text 批（contract-decision §9.4） |
| V-07 | AC-03 | ac_case_tests 零匹配 selector/v1 载入拒 ac；manifest 分配前全校验（固定集合/唯一/scratch 内/长度/64·128MiB）+ r1 越界产物拒（审查反例族） |
| V-08 | AC-04 | cache_t05_ 7/7（fresh→object-hit 计数 1,1,1,1,1→0,0,0,1 link/run 恒真；fresh 模式热库强制重算；HIR 重验=783 native_resume_ 5 绿）+ 语料跨 check 复用 |
| V-09 | AC-04 | cache_t05_ body 编辑全 miss exit4 + 783 invalidation_ 3 绿 + plan788_r1_ compiler recipe 换 fp 全 miss；改 expected→FAIL（语料 red-controls + ac_case_tests 错 exit 红） |
| V-10 | AC-04 | 783 concurrency_/store_boundary_ 9 绿 + cache_t05_ read-only 缺根 infra/命中消费/miss 永不发布 + gc-concurrent 场景 30 样本功能恒验 |
| V-11 | AC-05 | syntax_t06_ 7/7（两语法 lossless/typed 诊断+span/多 decl graft 复用≥1 或显式 fallback、rescan≤64KiB、reparse≤2）+ 语料 syntax 3 case 金样 |
| V-12 | AC-06 | turnaround_t08_ 1/1：六场景×30 样本，P95 硬门 **hot 458ms / body-edit 468ms < 1000ms**；raw 全量留存 turnaround-samples.json；详见 perf-report.md |
| V-13 | AC-06 | revision_trace_sequential_edits（revision 0→2、逐步 span、最终产物属最终修订）；50×100ms 修订流复用 787 VersionedPublish 语义（compatibility_），端到端流采样归 syntax 性能配方后续 |
| V-14 | AC-07 | ac_native_suite_full_run：32 case（≥24）全符合（2 红控 by-design FAIL）+ legacy-map 32 行双向对账 + 聚合门=1；v1/CLI 兼容 test_cases_tests 26 绿 |
| V-15 | AC-01/07 | r1 报告 v2 合法 JSON（serde_json round-trip）+ schema 随 suite 分派 + stale check_with_native 同轴 Fresh/换 worker Stale（身份轴 worker-exe-sha256 每路径恒在） |

未导入面（如实）：text/record/text-record profile（旧门禁守护）；p15 评估序 trace-marks
变体（需 runtime-lib 批次，账本标注）；V-13 50 修订流端到端采样；V-06 worker 路径非
ASCII 生产者。均不冒称已支持。
