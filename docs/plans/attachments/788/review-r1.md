# PLAN-788 r1：T-01～T-05 独立阶段审查 R1

2026-10-16。`review_scope: phase`，`outcome: needs_fix`。本会话独立于实施会话，未创建代理；根据当前主检出计划、冻结契约、实际 diff 和独立反例重建结论。仅审 T-01～T-05，不授予最终 reviewed，不实施修复或合入。

## 冻结基线

- tree：`D:/autostack/.wt/lang-788/auto-lang`，branch `plan-788-dev`。
- reviewed_commit：`54e6a46caf1badddaba898a471492ae52e7a1fbf`。
- base_commit：`2f6b06c747c1318feeac217939a5e91ffd1307a9`；共 27 文件，6615 additions / 212 deletions。
- dependency：组内 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；ac-core/probe 为同树 path 依赖。审前、审后上述两树均 clean，HEAD 未变。
- 主检出既有其他计划/ledger WIP 保留；审查只写本计划附件、进度和债务登记。
- 规范输入及 SD 冻结见 `review-r1-baseline.json.txt`、`review-r1-spec-delta.md`。这些是审查快照，不发布 canonical Specs。

## 实际验证

工作树中执行，`RUSTC_WRAPPER=`。所有均为定向检查，未跑整档 t/tv/tt/tb/tu/taa/tf。

| 命令 | 结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-case-worker/Cargo.toml --test protocol_t03_ --test cache_t05_ --test feasibility_t01_` | 10 + 7 + 3 passed，0 failed/ignored |
| `cargo test -p auto-test-case --test contract_v2 --test contract_decode` | 17 + 19 passed，0 failed/ignored |
| `cargo test --manifest-path experimental/ac-core/Cargo.toml --test link_capture_` | 7 passed，0 failed/ignored |
| `cargo build -p auto-lang --features test-cases --bin auto-case` | exit 0；重建后才签收 runner 反例证据 |
| `git diff --check 2f6b06c74 HEAD` | exit 0 |
| `python D:/autostack/auto-lang/docs/plans/attachments/788/review-r1-probes.py --tree D:/autostack/.wt/lang-788/auto-lang` | 原始观察见 `review-r1-probes.log.txt` / `review-r1-results.json.txt`；脚本完成不代表产品 PASS |

63 项现有定向测试通过，不能覆盖下述反例。初次旧 runner 探针及解析脚本中途失败不作为最终证据；最终日志来自重建后 runner，原始报告另存 `review-r1-raw/`。非法 JSON 只在明确标记的副本上修正逗号，以便继续检查状态和 stale；原件保留。

## Findings（均为既有合同缺口，不是新规模要求）

### F788-R1-01 / P1：product_from 另走无界、无 containment 的执行器

T-03/T-04，AC-03，V-04/05。`ac.rs:535–609` 的 `consume_and_run` 用 `Command::output()`，完全不消费 check 的 run/total 预算，无 stdout/stderr cap，无 Job。合法 build 的死循环 exe 一旦被消费便可无限等待；输出风暴无限增长内存。此路径也用 pid+check.id 的共享 temp 路径先递归删除，脱离 run/case/check owned scratch，平行 run 可互相覆盖。

修复：消费者沿同一受控 rawbytes executor/worker 生命周期执行，使用唯一 owned scratch；验证死循环、双管输出超限、后代持管、Job 失败与并发同 check.id。未在无界当前路径实际启动永久循环，避免制造悬挂；该结论由完整调用路径静态核查成立。

### F788-R1-02 / P1：消费者执行 ERROR 被吞掉，且声明观察不完整

T-04，AC-02/03，V-04/06。`mod.rs:632–634` 只合并 `v.mismatches`/native，不传播 `v.status` 或 detail。`consume_and_run` 启动坏 exe 返回 `Ok(Verdict::error(...))`，mismatches 为空，外层遂设置 PASS。独立 `outside-artifact-invalid-consumer`：实际产物为文本 `not a PE executable`，producer/consumer 都 PASS，gate 0，consumer 声明 exit 3 也未获观察。

同时 `mod.rs:527–536` 的旧通用 guard 直接拒绝 ac/run 消费者的 stdout/stderr；`consumer-stream-rejected` 实际 ERROR/gate 2。消费分支也不比较接受的 perf。修复须传播 ERROR/FAIL/BLOCKED，完整执行支持的 exit/streams/perf，未支持组合在 admission 拒绝；增加启动失败、错 exit、正确/错误流和 perf 控制。

### F788-R1-03 / P1：worker IPC 不做完整 admission，资源回收有旁路

T-03/T-04，AC-01/03，V-04/05/07。`ac.rs:115–170` 只比对 nonce/hash：不核 schema、退出状态、ok/error 互斥及 action 必需结果。独立 `wrong-schema` 返回 acworker/999 仍 PASS；`nonzero-worker` exit 7 仍 PASS；`missing-run` 的 ok={} 也让 ac/run success PASS。nonce/hash 对照不能替代这些核验。

父 stdout `read_to_string` 无 4MiB cap，stderr 管道不排空；spawn 为裸 `Command`。timeout 仅 kill worker，再无截止 join/wait；worker 后代持管或关闭 stdout 后继续运行可使承诺截止失效。worker `main.rs` 输入 `.lines()` 无限长；180s 只在下一请求前检查，空闲和跨界请求不受生命周期时钟终止。协议 struct 未拒未知字段。

修复：严格版本化有界 framing 和 typed response，核结果形状、退出/异常状态；父子均用已有 containment/单截止/并发排空设施，收集和 reap 包含在时限内；加错 schema、缺字段、未知字段、超 IPC、stderr 堵塞、后代持管和 session idle/cross-deadline 反控。

### F788-R1-04 / P1：manifest 路径/集合/长度和分配前 cap 没有校验

T-03/T-04，AC-03，V-04/07。`ac.rs:362–397` 按响应 path 直接 read，随后只比较响应自带 sha；不核 path 位于本请求 scratch、祖先 reparse、declared bytes、kind/logical 固定集合、重复项、单产物 64MiB 或构建必需产物。总量检查发生在完整分配之后。独立产物 path 在 scratch 外，bytes 故意声明 999 而实际 19 字节，父仍接受并持久化。仅 sanitize logical 不构成路径或身份校验。

worker 的 scratch 检查只看末级 symlink；AC_CASE_STORE 也没有与 fixture/oracle 域的双向不相交审计。修复需复用全链 reparse/输出域检查、先 metadata/有界读取、固定 manifest 集合/唯一性及本次 request/source/profile/target/entry/compiler/toolchain 绑定；加有效 frame 替换、越界、错长度、重复 logical、缺 exe 和 cap 边界控制。

### F788-R1-05 / P1：build/total 预算在真实 cache 路径失效

T-03/T-05，AC-03，V-05。`exec.rs:55–59` 直接 return `exec_via_driver`；后者不检查 build_ms/total_ms，driver/link 使用既有独立截止。直接路径也未使用 build_ms，check 提前 return 不检查 total，run 完成后不查 total。父兜底还额外加 15000ms，并在阻塞 stdin 写完后才 recv_timeout。

独立 `budget-one-ms`：build=1ms/total=1ms 的真实源码构建成功返回，trace 自身已证明超过预算（具体耗时见 raw）；不是机器竞争下毫秒断言问题，而是接受的预算从未执行。

修复：请求单绝对截止自 admission/materialize 起贯穿 build/link/run/IPC/收集，阶段预算取剩余时间，禁止额外宽限冒充 total；成功/失败/命中/重放各出口均核预算，测试收紧预算和 kill/reap。

### F788-R1-06 / P1：所有带 native 的 JSON 报告非法，v2 runner 仍标 v1

T-02/T-04，AC-01，V-15。`report.rs:289` duration_ms 后缺逗号，`report.rs:339–342` native 末尾有多余逗号。真实 `real-native-report` gate 0/PASS，但标准 JSON parser 在 native 处报错。runner 从未把 schema 从 `RunReport::new` 的 /1 改为 /2。现有 `contract_v2_report_native_record_renders` 手工设 /2，仅 contains 检查，故全绿仍漏掉非法 JSON。

修复：合法序列化和真实 suite/case→report 版本分派；测试用标准 JSON parser round-trip，覆盖 native 空/完整、错误和多行、v1 不变；证明原生报告可被 stale/工具链实际读取。

### F788-R1-07 / P2：stale 不比较实际 worker/toolchain

T-02/T-04，AC-01/03，V-15。`stale.rs:31–54` 新轴仅常量 acworker/1；`check` 不读 row.native.toolchain，也不计算 worker 文件、lld/SDK/compiler 内容身份。独立将 AC_CASE_WORKER 从真实 worker 改为不同 mock exe，旧 native 报告仍 FRESH/gate 0（为绕过 F06 仅修复报告副本语法，未改身份内容）。

修复：执行与 stale 共用实际 recipe/worker/compiler/tool 文件、锁和配置身份，相关轴缺失不能视为一致；换 worker、换工具、改 oracle 各自独立反控。

### F788-R1-08 / P1：新 worker 未实现冻结的编译缓存身份

T-05，AC-04，V-08/09。`exec.rs:423` toolchain_fp 只含 lld sha 和恒定 crate versions，且传入 driver 只参与 A10；A07/A09 key 仍由 snapshot/profile/domain/codec/固定 binder/backend 字串/entry 构成。worker/compiler 源码或锁依赖改变而版本未 bump 时，复用 store 会继续命中旧 HIR/obj，工具指纹也不能令对象失效。raw-HIR/check 则接受 cache=read-only 等选项却完全绕开 store（父还标 requested cache mode）。

修复：把已承诺的实际 compiler/worker recipe、lock/deps、runtime/support/backend/配置轴按依赖层接入 key；不为用户配置造空身份。对不参与缓存的组合应显式 admission 或明确可信的非参与语义。增加 compiler/lock/工具配置改变、entry/profile、坏/缺块及阶段禁调反控。此项静态验证，没有通过篡改编译器制造缓存污染。

### F788-R1-09 / P2：cache 构建的 backend 语言诊断降为 infra

T-03/T-05，AC-02，V-02。`exec.rs:443–458` 仅 Frontend/Verify 做 typed replay；Backend 一律 driver.failed。相同合法源码 options.entry=absent，direct 路径可产生 backend/entry.not-found，带 store 的正常 runner 路径却 ERROR，无法满足已接受的 typed backend oracle。独立 `backend-missing-entry` 已重现。

修复：从真实阶段边界保留 typed backend/capability 诊断；工具/IO/linker 故障仍 infra，禁止根据 render 字符串分类。覆盖带/不带 store 一致性及基础设施负控。

### F788-R1-10 / P2：trace/计数缺失或失真，不能作为 T-05 观测验收

T-03/T-05，AC-04/06，V-08/10/12。独立 `negative-trace` resolve 错误 trace=[]：frontend 错误分支在 trace.extend(obs.spans) 前返回，typed_replay 也丢 observer spans。TracingObserver.last 初值 None，首 parse 固定记 0ms。cache driver 只有合并 frontend span，缺 parse/resolve/typecheck/bind/cache I/O 等阶段；未测资源无 gap/not-applicable 表达。

独立 `hir-golden-hot` object hit 声明 parse/resolve_typecheck/bind=0，但 include_hir 实际再次调用完整 compile_parts_with_observer。既有 driver 为 summary 先 parse，再在 compile_parts_with 中 parse，parse 仍报 1。vertical_and_run_observed 的 run 计时起点在 vertical_inner 之前，混入 build。上述均非延后 T-08 统计配方即可修复。

修复：真实 API 边界计时/计数（含 golden/replay/失败路径），首阶段正确起钟，父请求→发布端到端和嵌套 span 分开，缺测明确标 gap；阶段禁止调用反控验证命中不是只改数字。T-08 六场景采样仍待做，不在本次要求提前完成。

### F788-R1-11 / P3：新增代码存在未处理告警

T-02/T-04/T-05 健康检查：contract_v2.rs 的 CASE_SCHEMA、ac.rs 的 Status unused imports；driver.rs:363 自赋值 outcome.total_ms。根库另有大量预存告警，不能笼统归成本计划；但新增三项须清理。diff --check 通过不等于 rustfmt/告警清零；本轮未授予健康 pass。

## 验收映射与下一步

| 任务 | 判定 | 依据 |
|---|---|---|
| T-01 | pass（有限可行性范围） | 3 个真实可行性测试独立重放通过；不延伸为资源/身份实现正确 |
| T-02 | fail | F06/F07/F11；严格 case/suite 版本与 v1 正控 36 通过仍保留 |
| T-03 | fail | F01/F03/F04/F05/F09/F10；执行器 rawbytes/cap 定向 7 通过不覆盖旁路 |
| T-04 | fail | F01–F07/F11；真实基本 exit 路径通过，不能抵消消费者假绿 |
| T-05 | fail | F05/F08/F09/F10/F11；7 个 cache 正控通过，身份/预算/观测合同未闭合 |

AC-01/02/03/04 为 fail；AC-06 当前接线 partial，性能阈值未验；AC-05/07 及 AC-08 最终审查/AC-09 合入均 pending，不因未实施 T-06～T-08 本身判本轮失败。SD-01～06 方向可追溯，但受上述实际缺口影响，不签收最终规范增量；touched_goals=[] 保留（未引入新语言目标，仅实验测试入口）。

Plan 保持 executing，保留 T-01，重开 T-02～T-05，current_step=1/10；保留原 handoff 历史。先在同一树修复全部 findings，补定向正反控并提交，再复审。T-06 仍受 D-787 约束。裸 t、tv/tt 触面及 worker/probe/783/787 独立套件/closeout 全部是最终审查后 merge-owned pending；本轮不执行、不合入、不归档、不清树。
