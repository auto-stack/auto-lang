# PLAN-774 第六轮独立最终复审

2026-10-14，review_scope=`final`，outcome=`pass`。P774-R5-01已闭合，未发现新的阻断项。AC01..06通过，T07完成；T08仍是待授权的merge-owned收口。

## 基线与独立性

- 唯一实施树：`D:/autostack/.wt/lang-774/auto-lang`，branch=`plan-774-dev`；git worktree登记、仓库与路径均已核对。reviewed_commit=`c713aaafb5dfe73fd587a2e52f53f83c3f40355d`，repair_code_commit=`78537a80bcbcfe10c9bcc63923f5222b7f6c1a66`；base=`5e21561d2c303a02319ff7a29e4607933e09a38f`，previous_reviewed_commit=`8e4fb77289ad95eacfa2538f33dd460da2d707cf`。入/出场同HEAD，实现树clean，diff-check clean。
- 主检出HEAD=`0a0950bc025d1a7d601e3f89f5d4166f091e35c5`；Plan revision3，入场execution_done6/8。合同和历史记录来自主检出，审前全文见 [plan-before-review.md](plan-before-review.md)。其他主检出WIP保留。
- 当前Codex /root审阅上下文未参与774实施/修复；沿用独立审阅上下文，未另建agent、模型或checkout，不冒称另起新会话。用户明确授权第六轮复审；本轮不含合入授权。
- 当前Spec、Design/真实消费者、770/771、锁文件、源码与验证配置/技能绑定见 [input-hashes.tsv](input-hashes.tsv)。相对第五轮的输入变化见 [input-drift.tsv](input-drift.tsv)：主Plan/债务簿记与本次解析修复六文件；相关canonical Specs、规则模型、shim、ac-core/旧门禁/根配置无漂移。所有输入在收尾重新核对hash。
- 冻结SD见 [proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md)，SHA256=`83a721c3986952ab55009cde220bd8799bca4b747efa127c0d82db0f68c8f71a`。本轮SD文件与第五轮相同，但其引用的D06实现/文档已修正，重新审阅后通过；相同hash本身不作为通过理由。

## 实际验证

| 入口 | 本轮结果 | 持久证据 |
|---|---|---|
| 774 gate：locked check/full probe/fmt、警告正反控制、状态探针、隔离/hash | 8步通过；64 probe tests，永久状态探针1/1；无failed/ignored probe | [gate](774-gate.log)、[收据](774-gate-receipt.txt)、[check](probe-check.txt)、[test](probe-test.txt)、[fmt](probe-fmt.txt)、[state](probe-state-probe.txt) |
| 完整ac-core locked test | 145 passed，既有1 ignored | [日志](core-full.log) |
| 741 -SkipMainGates | 13步通过 | [日志](741-gate.log) |
| 745 | 13步、74例通过 | [日志](745-gate.log) |
| 772（脚本历史名767） | 14步、45例通过 | [日志](767-gate.log) |
| r1/r2/r3/r4/r5独立反例及合法控制 | 7/7、3/3、4/4、1/1、2/2，全部exit0 | [r1](r1-replay.log)、[r2](r2-replay.log)、[r3](r3-replay.log)、[r4](r4-replay.log)、[r5](r5-replay.log) |
| 新实际formatter→parser组合控制 | 3/3通过，exit0 | [日志](roundtrip.log)、[断言](summary-roundtrip.rs)、[再生入口](replay-roundtrip.ps1)、[锁文件](replay-Cargo.lock) |

ac-core与旧gates串行，原树测试；无crates/root Rust、Schema或语法参考改动，未跑根cargo t/tv/docs_gen。健康检查、fmt、警告匹配正反控制和失败传播核查通过；源码没有新增未处理warning、TODO绕过或生产debug输出，trace均属明确oracle。

774收据为当前HEAD，`gate-20261014-074319`；其真实raw目录=`raw-20261014-074249`，已按收据实际路径复制全文。native产物指纹见 [native-hashes.tsv](native-hashes.tsv)（本轮门禁开始后重新生成的774 PE索引；不承诺复制临时exe），生产shim artifact hash见门禁收据。状态适配器快照见 [永久探针](permanent-state-probe.frozen.rs) 与 [r4重放](r4-state-probe.frozen.rs)。

## 第五轮缺口及观测链闭合

原协议顺序为 `(attempts,allocs,frees,live,live_bytes,peak)`。当前 `src/lib.rs:651` 返回 `(u32,u32,u32,u32,u64,u64)`，live回归对象计数，字节两域直接返回u64。原R5输入 `live_bytes=peak=4294967298` 不再读成2；u32上界正控与跨界控制都通过。

永久测试 `v08_7_summary_parse_full_precision_boundary` 同时断言六字段，覆盖上界与跨界，没有降低expected或只测试helper。decisions §3.12明确勘误§3.11历史错位元组；interface与probe-cases同步，verification §12记录64+1实际面。历史错误描述有明确取代关系，不作为现行合同。

另选三组独立组合控制，使用**当前原shim的Counters、emit_summary、Line.u/uu、WriteFile输出路径**，再将其实际stderr交给**当前原summary_of/freestat_of**：

| 控制 | 实际输出与期望解析 |
|---|---|
| u32上界，异质计数 | attempts11、allocs5、frees2、live3，live_bytes=peak4294967295 |
| 宽值，两字节字段不同 | attempts11、allocs5、frees2、live3，live_bytes4294967298、peak4294967315 |
| 全释放后保留历史宽峰值 | attempts11、allocs5、frees5、live0、live_bytes0、peak4294967315 |

原输出分别见 [edge](edge.stderr.txt)、[wide](wide.stderr.txt)、[released](released.stderr.txt)。构建适配仅剥离crate级no_std/no_builtins与panic_handler，原formatter/输出函数体不改；[footer](summary-emitter-footer.rs)只设置计数并调用emit_summary，[生成源码快照](summary-emitter.frozen.rs)可核查。每个输出进程5秒截止且隐藏窗口；读取真实stderr，不在断言端重造expected格式。

这三组是宿主受控状态/协议组合检查，不是COFF/PE大内存申请，不宣称4GiB native验收。它们补足了原状态探针直接读Counters、合成解析控制只读字符串之间的观测传递证据；真实ABI/生命周期/预算小规模native面另由64探针验证。

## 历轮闭合、遗漏与范围

| 发现组 | 本轮闭合依据 |
|---|---|
| R1-01/05、R2-03 | 完整外部账本、65并发/预算恢复、零/负预算、成功free与invalid分离、首表与扩表失败回滚；r1与故障族重新通过 |
| R1-02/03、R2-01 | 来源/访问模式矩阵、借入参数EndView、mut/static合法兄弟、借用返回PE、LIFO/FIFO控制、嵌套退出/临时/move/clone；r2与native族通过 |
| R1-04、R2-02 | 默认view与显式模式、跨行/静态返回合法面、字符源API、下游唯一身份、嵌套record与R2公平对照；源码/决定/实际770/771同步核对 |
| R1-06、R2-04 | 60s/locked、contained executor、AV/Deadline前驱、警告控制、失败收据、当前HEAD/raw绑定与SD非生产边界；本轮重新通过 |
| R3-01/02、D1 | i64 setter原值、真跨容量恢复与空位复用独立、有效合同/统计整理；r3四控制重新通过 |
| R4-01 | u64累计保存/比较/释放后余量与额度恢复；r4与永久状态探针通过，观测宽度另由本轮组合控制证明 |
| R5-01 | 正确字段类型、永久上界/跨界解析控制、实际格式化输出到解析的三组组合控制；文档勘误与有效语义一致 |

原反例清偿与完整不变量闭合分开核对；无未授权削减、必需open项或workaround绕过。全部14个历史阻断发现已闭合，R3-D1整理完成，历史发现ID/失败证据保留。未新增阻断或需另开债务项。

完整所有权/公开生产ABI/GC、完整int宽度、源码text/record实现、组合text-record/List/enum/use、ACC自举仍是明确非目标/后续owner，不因本轮通过变为已实现。4096循环与65并发是验证规模，不变成语言上限或无限规模承诺。

## AC → 任务 → 证据

| AC | 本轮结果与映射 |
|---|---|
| AC-01 | pass；T01/02/05：D01..07的Spec/Design/消费者溯源、现行修订§3.9..3.12、770/771身份与API、完整资源/计数合同及V01核查 |
| AC-02 | pass；T03：V02五类真实内部ABI、异质顺序、字段/嵌套布局/嵌套调用/out-return，probe-test与原assertions |
| AC-03 | pass；T03/04：V03..06寿命/别名/逃逸/拒绝native-count0、四申请点/账本故障恢复、AV/Deadline前驱与contained runner，旧反例/完整probe/state输出 |
| AC-04 | pass；T02/03/05：V07/08 UTF-8/标量/NUL/组合/索引正反边界，预算0/负/临界/i64范围、累计精确与恢复、宽位观测；下游无未决必需前提 |
| AC-05 | pass；T06：V09隔离及旧三gate，四SD有效语义冻结，metadata与当前Spec核对，不冒称feature/公开ABI/native自举 |
| AC-06 | pass；T07：独立源码/assertion/历史闭合/遗漏扫描、当前HEAD与输入/SD冻结、本轮实际重放及新增组合控制，无合同阻断 |
| AC-07 | pending；T08 merge-owned：Spec/导航/ledger/index沉淀、归档计数、清理守卫等尚未执行，不预勾 |

## 规范增量与合入边界

| SD | 已核对的目标与有效规则 |
|---|---|
| SD-01 add | docs/specs/auto-ac/data-resource-contract.md；目标当前不存在，冻结D01..07及探针实证，保留experimental边界；不替换现有语言实现 |
| SD-02 modify | docs/specs/auto-ac/project.md；当前core/scalar能力不变，增加text/标量record共同合同指针，List仍后续owner |
| SD-03 modify | docs/specs/auto-hir/stage-contract.md；保留Checked唯一门/X6，追加数据资源阶段责任，production verify仍planned；不扩注册域 |
| SD-04 modify | docs/specs/auto-acc/project.md；required清单和native自举现状不因合同冻结关闭，只追加有限决定/后续owner取证 |

无retire；精确frontmatter：supersedes三目标auto-ac/project、auto-hir/stage-contract、auto-acc/project；new为auto-ac/data-resource-contract。touched_goals=[]有书面解释，本合同不推进或关闭全语言/native自举Goal，GOAL-017的AAVM谱系不受影响。

冻结提案的“观测u64全精度”与当前实现/有效文档相符。本轮确认其引用应按**现行覆盖规则**读取：§3.9/3.10、§3.11加§3.12勘误；探针数按verification最新§12为**64+状态1**。SD早期39/过程63与旧解析元组不是canonical现状。此处是提案已有覆盖条款的明确映射，不变更语义/验收；merge沉淀必须重排有效合同，不能把历史过程全文搬成现行规则。相关canonical Specs未并行语义漂移，未在review发布canonical或修改live ledger。

## 状态与交接

最终pass仅绑定本次Plan revision3、HEAD与输入；以后实现/语义改动须重新核对受影响review。主Plan改为reviewed7/8，勾T07，T08留空；KNOWN-DEBT追加闭合记录，保留五轮历史。

pending_closeout_gates：用户T08合入授权；SD/导航/ledger/index实际沉淀；归档与任务总数；批回归到期判定（到期由主检出单实例regress）；wt-guard clean后同树/branch/组清理与恢复收据。review没有执行合入/归档/清理，也没有修实现。

next：获得合入授权后按auto-plan-merge在原唯一树交接；合入前复核本HEAD、输入/配置、实现洁净及主整合基线，证据漂移按受影响面处理。
