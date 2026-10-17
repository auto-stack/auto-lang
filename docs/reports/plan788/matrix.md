# PLAN-788 验收矩阵（T-08，2026-10-16；R3–R8 修订 2026-10-17）

聚合入口：`powershell -NoProfile -File scripts/verify-autodown-ac-788.ps1 -Scope acceptance`
（ALL PASS，2026-10-17 R3 修复后实测；零测族拒绝、失败计数、P95 硬门内嵌；required ID
按 executed-ok 精确名逐 ID 绑定所属 test binary，逐 binary 最小计数，prose/前缀/
兄弟 binary 总数/无摘要支撑的 ok 名/伪状态词（okay-but-not-executed）均不能替代，逐 binary 摘要守恒
（ok 行数==passed），nextest 按真实格式（零 failed 省略/单数/(i/N) 序号/`(N slow)` 注记，id 可含空格）核对+MinPassed 下限（零测试/run<下限拒）+**执行守恒 passed+failed==run**+PASS 行计数守恒+摘要整行锚定（未知尾部仍拒）+分档筛选排除单列不拒（required 被筛除仍拒）——普通/slow/显式 -E 筛选/default-filter 四份真实日志正控，负控入 selftest 34 用例；R7 纠正了 R5/R6 的错误算式 run==passed+failed+skipped；R8：closeout t/tv/tt 实际调用经 `nextest list --message-format json` 绑定各档冻结选择的必验全名集合（tv 档实测绑定 163 必验、实跑 family ok、filter-excluded=6721 单列收据），选中未执行/兄弟替代/零绑定均拒（tier-bound 负控永久化））。
`-Scope closeout` 为 merge-owned 冻结提交触面（t/tv/tt 按 nextest 实际输出核对 +
worker/probe/ac-syntax 独立套件）。

| ID | AC | 反控/证据（测试入口→状态） |
|---|---|---|
| V-01 | AC-01 | auto-test-case 全量 88 绿（contract_v2_ 20：版本分派/未知字段/预算门/cache 参与门/JSON round-trip；contract_decode 19 v1 回归） |
| V-02 | AC-02 | protocol_t03_ 10/10（parse/resolve/typecheck 边界归因=观察器唯一来源；typed bind.dangling-ref+span）+ syntax_t06_ 7/7 + cache_t05_ typed 重放 + r1 backend-missing-entry typed（带 store） |
| V-03 | AC-02 | worker ac_native 面：ac_case_tests exit oracle+红兄（p01/p04 真源码→PE→run）；ac-native 语料 core 8 + scalar 3（exit 70/3/14/0/1/4/73） |
| V-04 | AC-03 | ac_product：语料 red-controls（金样 FAIL≠阻断真产物，消费者跑真 exe exit3）+ ac_case_tests product_handoff；r1_response_admission_table（schema/ok-error/run 形状 5 负例）；R3：`r3_hir_backend_negative_keeps_legal_partial_hir`——语言失败携带合法 hir.atom 部分产物 PASS 且持久化，diagnostics+obj/exe=伪造拒绝（`r3_parent_io…` (d)） |
| V-05 | AC-03 | link_capture_ 7/7（deadline 杀树/输出 cap 立即终止+精确边界/contained 拒绝重放）+ protocol_t03_ 真死循环 budget.run 杀 + review_r1_ 1ms 预算真构建必红 + consume 经 worker（F01）；R3：`r3_parent_io_lifecycle_overcap_and_deadline`——stdin 写入纳入墙钟（不读 stdin 的 worker 100ms 预算被及时杀，<900ms，非旧 ~1267ms 满管阻塞）；R4：同测试 (b2)——stderr flood+1200ms linger 形状在 cap 即时终止（<700ms，收据声明 TERMINATED at the cap），不再等 worker 自然退出 |
| V-06 | AC-03 | link_capture_ native_bytes_（无效 UTF-8/NUL/CRLF 字节级 round-trip 对照 lossy wrapper）+ 执行器层；R3：`r3_native_bytes_real_worker_path_producers`——真实 worker 产品消费路径生产者：NUL+CRLF 为可比对 oracle 字节（consume_and_run 全 runner 路径 PASS），非法 UTF-8（ff/fe/80）经协议链路逐字节保真且完整 admission 接受；解码后 1MiB cap 与 oracle 声明无关（同测试 (a)/(e) 边界） |
| V-07 | AC-03 | ac_case_tests 零匹配 selector/v1 载入拒 ac；manifest 分配前全校验（固定集合/唯一/scratch 内/长度/64·128MiB）+ r1 越界产物拒（审查反例族） |
| V-08 | AC-04 | cache_t05_ 7/7（fresh→object-hit 计数 1,1,1,1,1→0,0,0,1 link/run 恒真；fresh 模式热库强制重算；HIR 重验=783 native_resume_ 5 绿）+ 语料跨 check 复用 |
| V-09 | AC-04 | cache_t05_ body 编辑全 miss exit4 + 783 invalidation_ 3 绿 + plan788_r1_ compiler recipe 换 fp 全 miss；改 expected→FAIL（语料 red-controls + ac_case_tests 错 exit 红） |
| V-10 | AC-04 | 783 concurrency_/store_boundary_ 9 绿 + cache_t05_ read-only 缺根 infra/命中消费/miss 永不发布 + gc-concurrent 场景 30 样本功能恒验（R3：真实 GC 独立进程 + barrier 起止区间，30/30 重叠证明） |
| V-11 | AC-05 | syntax_t06_ 7/7（两语法 lossless/typed 诊断+span/多 decl graft 复用≥1 或显式 fallback、rescan≤64KiB、reparse≤2）+ 语料 syntax 3 case 金样 |
| V-12 | AC-06 | `turnaround_native_six_scenarios_p95_gates`（R3，auto-lang `--test plan788_r3_`）：六场景×30 样本，墙钟=请求构造→IPC→parse→validate_response→manifest 准入+持久化→oracle 比较（runner 同名入口，无判前止钟）；body-edit 循环界 3↔4 真交替+唯一 marker、逐样本 parse==1/object=lowered；GC=真实 store-gc 进程并发、30/30 barrier 重叠；P95 硬门 **hot 487ms / body-edit 551ms < 1000ms**；raw JSONL 逐样本先落盘（含 spans/counters/hits/gc 区间/错误） |
| V-13 | AC-06 | `turnaround_syntax_gate_and_revision_stream`（R3）：syntax edit 门 P95=1ms（墙钟含 admission+持久化+字节 oracle）；修订流=双真实 worker 车道、固定 100ms 到达时钟（无累积漂移断言）、heavy（31 连插+31 连删 64KiB，~520ms）vs light（~3ms）产生 **7 次真实错序完成**，latest-wins 仲裁全部拒绝（refused/cancelled 留痕），发布序列严格递增、最终发布=50，积压深度≥2 可见，50 条记录全留存；R4：期望文档=前缀+插入+后缀（50/50 真实重建与 oracle 匹配），发布仲裁为独立 oracle（Ok(false)/Err/错 counter/旧于已入队最新均拒，负控单元 `r4_publication_oracle_negative_controls`），counter==steps 全量核验；R5：期望身份取**请求侧** nonce/hash（响应回显仅关联），中继改写全部 request_hash 的实际入口负控固化（`r5_wrong_hash_refused_and_recorded`：拒绝+落盘），坏 JSON 中继证明读/解析失败先落盘再失败（`r5_bad_json_failure_recorded`）；R6：语法门实际读取统一经 read_response_line（round_trip 只写），预热后注入非法 UTF-8 的中继负控固化（`r6_invalid_utf8_read_recorded`：读错先落盘再失败），到达时钟预计算纯化；syntax_t06_ revision_trace_sequential_edits（0→2 逐步 span） |
| V-14 | AC-07 | ac_native_suite_full_run：42 case（≥24）全符合（2 红控 by-design FAIL；HIR 负例 7=引用 2/结构 2/类型 2——R3 增 ac.hir.type-param-mismatch 真实 fixture 第二类型负例）+ legacy-map 43 行双向对账 + 聚合门=1；v1/CLI 兼容 test_cases_tests 26 绿 |
| V-15 | AC-01/07 | r1 报告 v2 合法 JSON（serde_json round-trip）+ schema 随 suite 分派；R3：`r3_stale_production_paths_per_adapter_action`——production stale（auto-case 子进程）：ac/check 未变 FRESH（lld/sdk 不适用于 check 行）、链接过的 ac/run 未变 FRESH（lld/sdk 内容轴适用且相等）、syntax 与 check 换 worker（AC_CASE_WORKER=内容哈希不同的副本）均 STALE 且指名 worker-exe-sha256 轴；轴适用集合=adapter/action/linked/support_lib（链接判定读报告真实键 `path`——R4 修复了 build 行误 STALE），多轴或缺轴均漂移；build 方向补未变 FRESH/换 worker STALE |

未导入面（如实）：text/record/text-record profile（旧门禁守护）；语言内非 ASCII 输出
生产者（core/scalar 无输出内建，随 text 批次导入；真实 worker 路径的字节保真已由
产品消费腿覆盖，见 V-06）。p15/p16 指定语义已通过 trace-marks 支持库落地（语料
ac.core.named-eval-order-marks / ac.core.observable-before-trap，stderr 字节 oracle）。
