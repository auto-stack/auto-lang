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
| V-12 | AC-06 | turnaround_t08_（R2 重测）：六场景×30 样本（全路径墙钟含 spawn/收割；body-edit 逐样本唯一内容+计数断言；GC 真并发 3 轮重叠），P95 硬门 **hot 736ms / body-edit 753ms < 1000ms**，syntax edit 门 P95 亚毫秒；完整 raw（native/syntax 两文件，含 counters/hits/input sha/worker sha/错误，先写后判） |
| V-13 | AC-06 | turnaround revision-stream：50 次 ~100ms 到达修订请求，每请求 k 步顺序应用终止修订=k 严格递增至 50，最终发布重建含全部编辑（latest-wins）；单飞 IPC 构造上无积压/错序窗口；syntax_t06_ revision_trace_sequential_edits（0→2 逐步 span） |
| V-14 | AC-07 | ac_native_suite_full_run：32 case（≥24）全符合（2 红控 by-design FAIL）+ legacy-map 32 行双向对账 + 聚合门=1；v1/CLI 兼容 test_cases_tests 26 绿 |
| V-15 | AC-01/07 | r1 报告 v2 合法 JSON（serde_json round-trip）+ schema 随 suite 分派 + stale check_with_native 同轴 Fresh/换 worker Stale（身份轴 worker-exe-sha256 每路径恒在） |

未导入面（如实）：text/record/text-record profile（旧门禁守护）；V-06 worker 路径非
ASCII 生产者（本批 core/scalar 程序无输出内建；字节保真已在执行器/协议层反控）。
p15/p16 指定语义已通过 trace-marks 支持库落地（语料 ac.core.named-eval-order-marks /
ac.core.observable-before-trap，stderr 字节 oracle）。
