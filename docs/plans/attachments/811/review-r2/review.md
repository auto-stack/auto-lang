# PLAN-811 独立复审 R2

2026-10-19（Asia/Shanghai） · stage: review · review_scope: final · **outcome: needs_fix**

受审 `33fc23208fb6e7b22f4b57b7101efaa553b4c8f1`，Plan revision 1。R1 的顺序关闭原例与格式问题已修复；完整关闭不变量、有效生命周期 oracle 和 AC-04 仍未完成。本轮为再次就绪核查与受影响路径复审，不授予全体验收 pass。

## 1. 基线、证据与范围

- 原实施树：`D:/autostack/.wt/lang-811/auto-lang`；branch `codex/plan-811-dev`。原 base `ab08955c241f6a824091c53d3b02894ca17e193b`，上一轮受审 `0f2ab56a991e5c1ca7ac599ba2ca0bd68bf4f488`，修复提交 `c66ca5a6ae47d2eb286bae6a3e3d24cefbaca996`、`33fc23208fb6e7b22f4b57b7101efaa553b4c8f1`；修复 diff 为八文件。
- 依赖 auto-down 仍为 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场原树/依赖 clean；结束复核见 `final-baseline.json.txt`。
- reviewer_identity：当前用户请求的 Codex 复审上下文，本会话没有实施这些代码；具体模型/会话 ID 未暴露。未创建其他 agent/checkout。
- 主检出 Plan 为准，输入原件为 [plan-input.md](plan-input.md)、[handoff-input.md](handoff-input.md)、[fix-r1-input.md](fix-r1-input.md)。[baseline.json.txt](baseline.json.txt) 冻结提交、源文件、依赖、Cargo.lock、测试配置、技能、规范及实际消费收据/脚本 hash。
- 权威 Spec 采用主检出当前版本；VM/UI 概述包含 807/808 的并行沉淀，实施树副本较旧。主检出规范 hash 与 R1 相同；不能把旧树副本当最新要求。
- work 已明确承认 merged loaded/error 未接线、静止期绑定漏注册，并仍将它们交 R04。按就绪规则，必需 work-owned oracle 未完成须退回；本轮只重放受影响关闭反例和核对新收据，不重跑全部 VM/Man/发行/浏览器档。

## 2. R1 闭合表

| 原 finding | 原例闭合 | 完整不变量 / 本轮结论 |
|---|---|---|
| F811-R1-01 生产者未回收 | 原 4096 文件探针三次均 `producer_continued=false`，Drop 后代次分别 9→9、12→12、15→15；修复了原 detached scan 例 | partial：tracked producer 接线存在，但关闭交错仍漏会话准入；新截止与满队列 oracle 也未成立，见 R2-01/02 |
| F811-R1-02 关闭后顺序准入 | `fresh_pending=0`，transition/publish/Ready 均 false，顺序 post-close open 返回空串，旧票据仍拒绝 | partial：关闭与 open 交错后仍插入有效会话，见 R2-01；不能标整个 invariant CLOSED |
| F811-R1-03 三形态/事件验收 | 新增 VM、Vue、merged、生成 HTTP 脚本与 JSON；生成路由语法、merged 依赖两个实际缺陷有代码修复 | **OPEN**：merged 收据主动排除 loaded/error；静止期绑定、卸载/投影等仍不全，见 §5 |
| F811-R1-04 格式 | 本轮按原八文件加 api_gen/rust_ui 共十文件，`rustfmt --check --edition 2021 --config skip_children=true` exit 0 | **CLOSED**；[format.log.txt](format.log.txt) 无差异 |

旧探针源 [probe-r1.rs](probe-r1.rs) 与 R1 源逐字节相同，R2 日志独立保存，没有再次写入 R1 文件。[probe-r1.stdout.txt](probe-r1.stdout.txt) 为本轮实跑；公开入口已重建核心 artifact，不根据任意缓存 executable 判断。

## 3. F811-R2-01 · P2 · open_session 的关闭检查与会话插入未同步

**AC-02/03，T-01/T-03/T-07/T-08；R1-02 的完整关闭不变量残留，属于修复未覆盖的交错。**

`crates/auto-media-core/src/image_pipeline.rs:1846–1875` 仅在入场检查 `is_closed()`，随后转换路径、获取 session 锁并插入；持 session 锁时没有再次检查。`:1743–1763` 的 shutdown 可以在这两步之间完成清表。`spawn_tracked_producer` 的锁存只控制线程创建，不能撤销此前插入的会话，返回的 bool 也未被 open_session 消费。

独立公开 API 反例通过合法 `Into<PathBuf>` 参数控制调度窗口：open 通过首次检查，在路径转换中发信号并暂停；另一个线程完成 shutdown；恢复 open。没有修改实施源码、访问私有状态或伪造 ID。相同窗口也可以来自普通 PathBuf 调用中的线程调度；自定义转换仅使它可重复。

```text
RACE shutdown_returned=true is_closed=true registry_shutdown=true
RACE returned_session="image-session-1"
snapshot={"session":"image-session-1","selected":0,"count":0,"generation":1,"name":"","uri":""}
close_session_after_shutdown=true
CONTROL sequential_postclose_session=""
CONTROL live_session_nonempty=true live_close=true
```

已关闭实例重新出现可查询、可关闭的有效 session。新线程被 closing 锁存拒绝，故本轮不声称该例还会产生 scan 线程或解码资源；实际错误是关闭后复活的空会话及不一致的准入返回。

修复要求：让会话插入与 shutdown 清表/准入共用同步规则，核对 producer 注册失败和并发关闭的清理/返回行为。补旧顺序例、此受控交错与正常打开正控；不能再以关闭后最终 counters 为零证明 session 表没有复活。

证据：[probe-r2.rs](probe-r2.rs)、[probe-r2.stdout.txt](probe-r2.stdout.txt)。

## 4. F811-R2-02 · P2 · 生命周期 oracle 仍不能兑现外部截止与满 resize 覆盖

**AC-03，V-03，T-01/T-03/T-07/T-08；R1-01 已指出的验收缺口未完整修复。**

新 `shutdown_under_external_deadline`（`:2594–2610`）在 `thread::scope` 中启动 shutdown，然后 `recv_timeout` 失败时 panic。退出 scope（包括 unwind）仍必须 join 该 scoped thread；若 shutdown 卡住，超时的父测试仍会等待它。更外层原样 runtime Drop 也不能代替有界清理。

独立反控复制该 helper 的相同结构，仅把 shutdown 换成可控有限任务，以避免制造真实永久挂死，100ms 截止的结果为：

| 控制 | 工作时长 | 返回时间 | panic |
|---|---:|---:|---|
| accepted-control | 10ms | 10ms | false |
| deadline-negative | 1500ms | **1500ms** | true |

反控确实触发错误，但直到工作自然完成才返回。它证明这个 helper 不是独立截止/清理设施；不据此声称本轮实际媒体 shutdown 卡死，34 项核心测试仍全部通过。

另外，`:2614–2651` 的真 PNG 改进了 codec 路径，却只断言累计 ingress accepted≥8，随即 shutdown 清队列；没有阶段闸门或观测证实 resize channel 确实满、指定 decode/resize 正在途。`:2676–2694` 仍以 sleep(15ms) 猜扫描在途，并比较不包含 Error 条目计数的 stats；本轮原 4096 文件代次探针已通过，但不能把这个新增 stats 断言当全部生产者状态矩阵的证明。

修复要求：采用可在外部终止并回收自有测试进程的有界 runner，或现有合适设施，给超时负控证明会在截止内失败且清理完成；有效图像配合可控阶段，实际覆盖满 resize channel、decode 在途、scan/open 与关闭竞争。范围仍为原 V-03，没有新增资源规模承诺。至少保存一个真正超时的负控和不超时的正控。

## 5. F811-R1-03 继续 OPEN：收据没有补齐既定事件验收

**AC-04/05，T-06/T-07/T-08；原 finding 延续，不新编号。**

已核对 work 收据和 harness 源，冻结副本均在本目录（文件以 `.json.txt` / `.py.txt` 保存）：

- **VM**：九项记录验证打开、loaded 计数、A→B→A、坏票和恢复；值得保留。脚本明确使绑定伴随 request_view/populate 的代次变化，并没有补静止期新挂载反例、卸载退订或全部 handler 形参投影。它检测到 loaded 状态，没有记录用于区分 Ready 与实际渲染帧消费的时间/像素 oracle。
- **Vue**：浏览器实尺寸 2×1、单次 loaded、稳定和坏票 HTTP 404 有明确记录；脚本的 HTTP fetch 404 并不等于 `<img>` 的 error 事件断言。没有完整换源/卸载/投影矩阵。
- **merged**：[merged_media_run.json.txt](merged_media_run.json.txt) 明确写 `loaded/error ... not asserted`，绑定和“渲染帧后稳定”的实际状态始终是 `view_state: loading`。harness 仅检查 URI 前缀、snapshot 树存在、进程存活；源头说 pixel rendering via screenshot，实际实现又注明没有截图工具并且未保存像素证据。六项全绿不证明事件或像素已经消费。Windows `proc.terminate()` 也不能作为 runtime Drop/worker join 的收据。
- **生成 HTTP**：[gen_media_http.json.txt](gen_media_http.json.txt) 有13项具体状态/头/Range 字节比较，支持真实数据面补强。路由 `:id/:revision` 和 merged manifest `auto-media-core.workspace=true` 的代码修复存在。该证据不覆盖 UI 事件。

work 的 [fix-r1-input.md](fix-r1-input.md) 同时承认两项实际缺口：merged 的 `devtools_update` 没有 notify sweep；媒体代次不变、表空/pending0 时新 surface 可被早退门跳过。这两项与 AC-04 明文事件验收相冲突；本 Plan §10 没有授权撤销这项必需验收。预存性质可用于归因，不能使已有合同中的未完成项通过，R04 交接不是批准延期。

收据 JSON 本身也未记录 commit/生成源码/输入/binary hash；脚本消费 scratch 中的预构建应用，而探针应用源未作为本轮附件完整冻结。下一轮需保存实际输入与生成/构建命令及来源 hash，确保能从受审提交重建生产消费者；不应仅凭 exe 路径复用旧运行数。本轮没有把这些旧 work JSON 重新记成独立实跑。

原792收据仍绑定 R1 uiminimal 生成物。本次修改了 merged 成员 manifest 接线，不能笼统说模板未触发行判定面便将旧构建覆盖全部新生成路径；最终需用受影响产物的来源等价证据或重放补齐。此项归现有 AC-05/交接收口，不增设整档复审要求。

## 6. 实际命令、AC 与 Spec

重放命令（原实施树执行）：

```text
python D:/autostack/auto-lang/docs/plans/attachments/811/review-r2/replay.py D:/autostack/.wt/lang-811/auto-lang
```

实际结果：核心 `cargo build --offline -p auto-media-core --message-format=json` pass；R1 原例重放完成且原输入关闭；R2 受控交错/截止正反控如上；`cargo test --offline -p auto-media-core --lib` **34/34 pass，0 ignored，2.02s**；十文件 scoped format pass。原始 cargo/探针输出保存为同目录 `*.stdout.txt` / `*.stderr.txt`。core 构建/测试未出现 warning。

| AC | 本轮结果 | 映射与范围 |
|---|---|---|
| AC-01 | partial | T02/04/05 的单源抽取与接口迁移不因本轮关闭修复失效；17真实shim、完整跨消费者链本轮未全重放，不授予最终pass |
| AC-02 | fail | R2-01，T01/03/07；顺序新旧票据正控通过，完整 session admission 不变量未闭 |
| AC-03 | fail | R2-01/02，T01/03/07；原扫描退出/Weak释放正控通过，受控生命周期矩阵仍有缺口 |
| AC-04 | fail | R1-03，T06/07；merged事件与静止期绑定等明确未完成 |
| AC-05 | partial | T08；有发行接线和既有收据，受影响新产物/当前完整Spec与来源绑定仍需收口 |
| AC-06 | partial | T09；格式旧问题关闭，核心无warning；其他必需项不通过，不能授予reviewed |
| AC-07 | pending | T10/11，merge-owned |

有效 SD（原表+handoff 的 R1 增补）冻结为 [spec-delta-input.md](spec-delta-input.md)，SHA-256 `4d691813f6ef420a38f1b368fe3a8182e6846e84f8cc084bcf21753a1807672d`。SD01/02 的 dead-ticket 顺序准入已验证，但 shutdown/session 的完整关闭承诺受 R2-01/02 阻断；SD04/05 的事件承诺受 R1-03 阻断；SD03 仍需基于主检出 807/808 调和；SD06 仍需受影响生成物的来源证据。现有五个 supersedes、新 core 目标及 GOAL-003/007 保留，未改 canonical 或 ledger，也未弱化原验收合同。

## 7. 退回与下轮就绪条件

这是第二次 final-scope 请求未通过。按 review/work 规约，下次 work 交接应先附诊断与覆盖/设计调整表，再宣称 execution_done：

| 剩余不变量 | 此前漏检原因 | 需要的调整 |
|---|---|---|
| 关闭后 session 永不重新插入 | 只验顺序 post-close；producer 注册锁不覆盖 session 插入 | 分析准入/清表/注册的原子边界，永久保存受控交错与正常/关闭正控 |
| 生命周期测试可有界失败、实际覆盖满通道 | 将 scoped panic 当外部截止；累计 accepted 当 resize 满；sleep 猜在途 | 真正外部截止+清理反控；可控 decode/resize/scan/open 阶段和可观察退出/bytes证据 |
| 真实消费者满足渲染后事件合同 | 缺少事件的 merged 也记全绿；把必需项按预存债务移交 | 完整 AC04→三形态/状态→真实断言矩阵，明确原始输入/产物来源，修复原合同内缺口 |

Plan 退回 **executing 3/11**，保留 T02/T04/T05；重开 T01/T03/T06/T07/T08，T09 未完成。下一步为原树 work 修复/补验收、提交后按 affected scope 复审。保留本轮已通过的旧反例和格式证据，按依赖/代码/配置/输入是否改变限定复用，不重复归零。

merge-owned pending：最终审查 pass 后在冻结提交跑裸 cargo t + cargo tv，最终并集/受影响生成发行检查，Spec/ledger沉淀、归档、guard clean 清理。当前 diff 未改 ui_gen/trans，不额外触发 tu/tt。本轮未运行整档或 docs_gen，未改实施源码、合入或清理 worktree。
