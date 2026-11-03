# ACE慢测归属与后续处理（2026-10-30）

> 2026-11-01执行入口改为[893 r3](../893-ace-test-framework-and-slow-case-attribution.md)，drafting0/12。下方旧叶ID、源码位置、tm/888工具、原日志保留路径均属当日历史；当前固定cargo t/原生package-target-test名、统一plan-artifacts及merge清理政策优先，不复活旧目录/runner或raw前置。13义务不删，两个828旧名待T-01核迁移和断言闭包；其他11同名声明静态可定位，仍非实际发现/运行证据。本轮只规划，无重型验证。

状态：已接受888 r3定向交接；初订仅路线登记与静态分析。用户随后授权正式立项[PLAN-893](../893-ace-test-framework-and-slow-case-attribution.md)，drafting r2、0/12，将共同ACE测试接入与13慢例归因合包（原11项+§6新增2项）；仍未改测试、未建实施树或运行重型验证。读取主线HEAD为`e8bfaf8ec`，不将其当作888候选验收版本。

## 1. 授权与责任边界

888转交的人类原话：“这些慢的的测试都是最近新作的ACE引擎里的？那么可以都归入ACE单独的测试模块，然后通知ACE路线去处理？”据此，888负责归组、清单、模块入口与历史证据交接；ACE路线接受后续耗时归因、优化与长期分层规划责任。来源为787/796/798/817/818/828，不全是最近几天新增。

保留11个独立叶、原子身份与单项选择能力，不创建重复执行义务。原测试保持启用，原oracle、样本、规模、资源门及负控不变；本轮888测量不再重跑这11项，不等于永久免测或已通过当前候选验收。暂关须用户逐项明确决定；归入ACE-slow导航组本身不改变既有门禁适用性。

888当前r3按模块归组/数量及每模块一次墙钟、累计CPU、峰值内存推进；已取消全目录三轮/P90与新A/B/C矩阵作为其交付前置。ACE各能力原有正式周转样本和P90≤2000ms合同保持，慢测总墙钟也不是单次请求的P90。

## 2. 接受的11项

以下秒数仅为历史单例墙钟（85.224–526.267s），不是当前候选测量或回归结论。完整独立模块ID见原交接清单及assignment，后续必须按原ID选择，不按文件名整族重跑。

| 分组/原能力 | 原子测试 | 历史秒 | 静态分析入口与待归因阶段 |
|---|---|---:|---|
| AV/828 | `t07_v07_512_revisions_bounded_history_stable_watermark` | 111.450 | `experimental/ac-av-probe/probe/tests/function_incremental_828.rs`；512轮真实编辑、各生产阶段、历史水位与收尾 |
| AV/828 | `t07_v07_full_storage_axis_512_edits_gc_restart_sentinel` | 153.543 | 同文件；真实持久化、GC、restart和sentinel，核每轮重复打开/扫描成本 |
| CTE/817 | `resources_real_watermark_gc_physical_delete_and_converge` | 291.851 | `experimental/ac-cte-probe/probe/tests/cte_resources_tests.rs`；填充真实水位、lease、物理删除与收敛 |
| CTE/817 | `resources_thirty_two_mib_pages_and_512_distinct_revisions` | 197.175 | 同文件；32MiB真实页构造与512个不同输入分列 |
| CTE/817 | `cte_turnaround_injected_spike_control_is_red` | 218.420 | `experimental/ac-cte-probe/probe/tests/cte_turnaround_tests.rs`；30样本、真实worker与2500ms注入分列，保真实变红能力 |
| CTE/817 | `cte_turnaround_resident_degrade_gate_and_control` | 110.056 | 同文件；正常居民路径与强制重建反控分列，保原构建配置对应样本/阈值 |
| native/818 | `mir_native_resources_cold_set_churn_gc_convergence` | 407.777 | `experimental/ac-mir-native/tests/resources_tier.rs`；32MiB COFF集合、512次真编辑、链接/执行/GC分列 |
| semantics/796 | `sema_budget_deep_chain_4096_full_path` | 218.385 | `experimental/ac-semantics/probe/tests/sema_budget_pressure.rs`；parse/project/infer/canonical/publish/read-back分列 |
| syntax/787 | `syntax_budget_64mib_streaming_small_queries` | 526.267 | `experimental/ac-syntax/adapters/artifacts/tests/syntax_budget_.rs`；独立生成器、64MiB流式构建、持久化/恢复/小查询分列 |
| AV/798（language harness） | `tests::ac_av_probe_tests::ac_av_probe_formal_turnaround_gates` | 106.263 | `crates/auto-lang/src/tests/ac_av_probe_tests.rs`；原正式样本、父侧调度/worker/工件与判定分列 |
| native/818（language harness） | `tests::ac_mn_probe_tests::mir_native_turnaround_body_edit_gate` | 85.224 | `crates/auto-lang/src/tests/ac_mn_probe_tests.rs`；每轮warm与body-edit、原生执行与父侧调度分列 |

静态可确认这些用例含真实规模/循环/负控，不能因总墙钟长就判执行引擎单次请求超门。4096链现有源码已约束SCC构建次数≤2，不能再把历史O(V²)问题直接登记为当前未修缺陷。其余准备/进程/存储成本是否占主导仍待实际阶段计量，未证明可缓存或可减少。

## 3. 后续规划及验收要求

1. **先形成一个有界归因合同，再按真实修复面拆实施包。** 当前由[893](../893-ace-test-framework-and-slow-case-attribution.md)认领共同测试接入/13例归因（原11+新增2）；产品算法/协议优化据实后继拆包，仍不预占后继号。逐项固定实际源码、锁、配置、worker和命令，使用原11叶做阶段墙钟/CPU/内存与I/O归因；已有日志先读，避免把全部慢例当每次开发门重跑。不预设必须合成一个横跨syntax/sema/AV/native/CTE的大改Plan。
2. **优化候选顺序**：先查syntax大规模构建与sema深链的算法/序列化成本，再查native及AV/CTE的存储扫描、重复准备与进程生命周期；原正式周转/注入负控单列。顺序是调查建议，不是已经证实的瓶颈。只复用经身份/依赖校验的不可变准备，不抹掉原cold、restart或独立store义务。
3. **分层候选**：日常affected选择与ACE规模/长时间压力验证分别提供明确入口、触发条件和owner；11叶仍可独立重放，长期慢档必须明确实际跑在哪里、何时跑及失败如何阻断。单纯移出常规模块、改ignore或减少规模不能当优化交付，既有正式门的迁移需显式修订合同。
4. **修复验收**：保原oracle/数据量/512修订/4096深度/32–64MiB真实数据及样本/负控，记录同身份前后阶段与端到端数据；性能敏感性控制仍真实变红，失败原子性、GC/sentinel、恢复重验、未改函数生产0、资源与清理门保持。若建议改变规模或样本，另列给用户决定，不实施为默认优化。
5. **写面/采样交接**：888其余模块单轮采样结束前只读代码和本地历史证据、写路线文档；不改被测源/工具/配置/worker/helper/测试注册，不运行重型验证或干扰采样。之后消费888实际交付/阶段收据的commit、源锁、11叶映射及模块资源报告，再逐文件接受S/N/H窗口。与891/892/890按真实符号交接，不消费候选WIP，不把888整包完工设成纯规划前置。归组不要求重开原归档Plan。

## 4. 原始交接及只读核验收据

本地证据根：`D:/autostack/.wt/lang-888/auto-lang/docs/plans/attachments/888/`。原始证据不复制入Git；888收尾须先保留外部稳定路径并回交，避免清树后链接失效。本报告为路线摘要，不是执行PASS。

| 交接文件 | SHA256 |
|---|---|
| `ace-slow-tests-r3.md` | `0cdeac9b3e8b084436a05d25485efdb7ac12d63daf035aa07b7baeefe3dc38f1` |
| `baseline/r3-ace-slow-assignment.json` | `c43374054a2722bf608e6f1d8752a8b4289f5bd920593f8de8983113c07afffd` |
| `baseline/r3-long-case-decisions/measurement-deferrals.json` | `d6d59555c2bf70ea1049c7c934ff21e7815c35405b1bdc93a0877d442ccd8a7d` |

核验命令：PowerShell `ConvertFrom-Json`读取assignment，逐条`Get-FileHash -Algorithm SHA256`核11个entry引用日志；结果11/11一致（共享文件按entry分别核验），未执行任何原测试。以下均相对上述证据根的`baseline/r10-module-costs-recovered/round-1/`：

| 原日志 | SHA256 |
|---|---|
| `728-run.log`（AV 2项） | `7bd4d431802672d3e3a26003274b44634fbfaae5f34f1a539f2a4452d1098500` |
| `791-run.log`（CTE资源2项） | `71a20529fa242e2de82bb61d22bdc9bba0221237217904b03838174b7efc66b8` |
| `794-run.log`（CTE周转2项） | `6adbbad6a5897f9836159968ea7f5880dedb0d64192e27c1fa4cf690cb98eada` |
| `764-run.log`（native资源） | `36fbe2941205495d60ce22deae30597b1d4bb51685f69275e2b43a2fe978f182` |
| `630-run.log`（sema） | `e043fc4f995e05e2c2fbc35c6e4b4b3d19d06cd5f4d6f9967e3dd15765d797e4` |
| `590-run.log`（syntax） | `780f9cc33d3e133ba33032a26874a1bf447be210ecf265ded7e06e6ecdf9dbba` |
| `59-run.log`（language AV） | `9e33eb2039ea4b1ae6835b83174dbf1e4f4084509d420c3164abb49c4785bb79` |
| `80-run.log`（language native） | `3371c7c6584cc2b50de55a36c78e20d68eee5499b8aadf418b0ef4291149d7d7` |

## 5. 独立叶ID保全

以下为888 r3交接时的原身份导航；未来改名须由治理owner提供一一映射，不据此复制用例或改执行合同。

| 原子测试 | 独立模块ID |
|---|---|
| `t07_v07_512_revisions_bounded_history_stable_watermark` | `ace-av/auto-av-probe-function_incremental_828/auto-av-probe-function_incremental_828/direct/017` |
| `t07_v07_full_storage_axis_512_edits_gc_restart_sentinel` | `ace-av/auto-av-probe-function_incremental_828/auto-av-probe-function_incremental_828/direct/018` |
| `resources_real_watermark_gc_physical_delete_and_converge` | `ace-comptime/auto-cte-probe-cte_resources_tests/auto-cte-probe-cte_resources_tests/direct/015` |
| `resources_thirty_two_mib_pages_and_512_distinct_revisions` | `ace-comptime/auto-cte-probe-cte_resources_tests/auto-cte-probe-cte_resources_tests/direct/017` |
| `cte_turnaround_injected_spike_control_is_red` | `ace-comptime/auto-cte-probe-cte_turnaround_tests/auto-cte-probe-cte_turnaround_tests/direct/003` |
| `cte_turnaround_resident_degrade_gate_and_control` | `ace-comptime/auto-cte-probe-cte_turnaround_tests/auto-cte-probe-cte_turnaround_tests/direct/007` |
| `mir_native_resources_cold_set_churn_gc_convergence` | `ace-native/ac-mn-resources_tier/ac-mn-resources_tier/direct/001` |
| `sema_budget_deep_chain_4096_full_path` | `ace-semantics/ac-sema-probe-sema_budget_pressure/ac-sema-probe-sema_budget_pressure/direct/003` |
| `syntax_budget_64mib_streaming_small_queries` | `ace-syntax/ac-syntax-artifact-adapter-syntax_budget_/ac-syntax-artifact-adapter-syntax_budget_/direct/001` |
| `tests::ac_av_probe_tests::ac_av_probe_formal_turnaround_gates` | `language/auto-lang/ac-av-probe/direct/001` |
| `tests::ac_mn_probe_tests::mir_native_turnaround_body_edit_gate` | `language/auto-lang/ac-mn-probe/t-tt/018` |


## 6. 888追加两项：当前共13项（2026-10-30）

沿用户原授权，由888定向补交，893已修订r2（仍drafting、0/12）。原§2/4/5的11项及其历史hash不改；当前为syntax 1、semantics 1、AV 4、native 3、CTE 4。两项均保持启用/原规模/原样本和负控，纳入T-05/06成本归因与用途准入建议；未授权自动关闭、缩量或改变日常/周期触发。

| 原子 | 独立叶ID | 已观察事实与限制 |
|---|---|---|
| `av_resources_32mib_product_random_read_and_churn` | `ace-av/auto-av-probe-resources_tier/auto-av-probe-resources_tier/direct/002` | 原17项组内PASS 126.085s，组总128.105s；现已拆独立叶，至少32MiB产物/随机读取/churn保持。CPU/内存仅有旧组总量，独立值未知，不为补数本轮重跑重负载 |
| `tests::ac_mn_probe_tests::mir_native_turnaround_signature_rebuild_gate` | `language/auto-lang/ac-mn-probe/t-tt/022` | 原模块61.760s；定向复测PASS，总69.418s、test body58.812s；原30样本/2000ms门保持，总窗含发现/启动等，不能与测试体混称 |

原17项AV组当前分为快检15项、32MiB独立叶、六场景smoke独立叶。Smoke原3样本/6场景保持，不是全目录三轮。888当前只对修复影响25叶补测，尚在运行；893继续只读登记，不并发改共享代码/工具或跑重型验证。

本地证据根仍为`D:/autostack/.wt/lang-888/auto-lang/docs/plans/attachments/888/`。本轮只读计算文件hash，没有执行测试；完整源绑定/CPU/内存以对应costs.json及最终release为准，不从PASS行推造独立资源值：

| 相对日志 | SHA256 |
|---|---|
| `baseline/r3-single-module-profile/round-1/798-run.log` | `2e6be41bff14fe3f4d254bfd16900bb211de0b140a814b4e4256b012d08e9ddc` |
| `baseline/r3-repaired-module-observations/round-1/84-run.log` | `991625ff0a34180b0a23e3ef40f63324cbbe99dfd02f3f698cf50d14ce30a923` |

## 7. 888最终释放与证据迁移（2026-10-30）

888已归档10/10，delivery `5a6c23afacd3c40412c5561a5ba2ae322a119c2e`、最终cleanup `129ef605763e59ca13bf312fd4a391eded10c13a`均以`git merge-base --is-ancestor <sha> HEAD`返回0核实；`git worktree list`确认lang-888消失。共享worker/父侧与原M888窗口释放，893可在用户指定实施后进入T-01，不再因888本身等待。后续新owner/机器负载仍须现场核查。

当前交接Markdown在`D:/autostack/auto-lang/docs/plans/attachments/888/ace-slow-tests-r3.md`；模块/资源摘要在同目录`module-inventory-r3.md`及`module-resource-observations-r3.md`。§4/6原worktree路径是历史来源，现不再存在；原始日志/JSON的baseline相对路径现在从`D:/autostack/plan-artifacts/plan-888/evidence/`解析，原收据字节/原runtime路径保持。保全与4个PDB定向回收收据在`D:/autostack/plan-artifacts/plan-888/{preservation-receipt.json,removed-pdb-receipt.json}`，其他证据不改。

当前13项权威assignment为`evidence/baseline/r3-ace-slow-assignment-final.json`，schema `plan888-ace-slow-assignment/2`，本轮只读核entries=13、SHA256 `3fc96c11365f19068b7251222737644035f24f31417c69461f0e0a262d044919`。原11例历史耗时未重测；新增32MiB仍只有旧组内126.085s，独立CPU/内存未知；signature总69.418s/body58.812s及原30样本/2000ms门保持。快资源组15项1.409s、六场景smoke23.990s为888移交值，不是893本轮执行。未关闭/减量/改oracle，893仍drafting r2、0/12。
