# PLAN-811 独立复审 R3

2026-10-19（Asia/Shanghai） · stage: review · review_scope: final · **outcome: needs_fix**

受审 `4a1d8fcf3a72fcb6a524b642669ff0340dabcf5d`，revision 1。上一轮关闭/开会话竞态已关闭，核心38项测试和修改文件格式通过；新 merged 分发有实际生产入口，但订阅身份与空闲唤醒未满足原事件合同。截止 helper 的及时报错已改善，资源回收和满通道 oracle 仍未闭合。

## 1. 基线与方法

- 原树 `D:/autostack/.wt/lang-811/auto-lang`，branch `codex/plan-811-dev`；base `ab08955c241f6a824091c53d3b02894ca17e193b`，上一轮受审 `33fc23208fb6e7b22f4b57b7101efaa553b4c8f1`。本次修复仅改 core image_pipeline 和 UI renderer（+413/-82），累计13个实施提交。
- 同组 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。原树/兄弟入场 clean；最终稳定性收据见 `final-baseline.json.txt`。
- reviewer_identity：当前用户请求的 Codex 复审上下文，本会话未实施代码；具体模型/会话ID未暴露。未建新树/agent。
- [baseline.json.txt](baseline.json.txt) 冻结代码、依赖、配置、主检出规范、Plan/交接、修复报告、work收据和探针源码/二进制 hash。权威规范仍取主检出，VM/UI概述含807/808沉淀，实施树副本较旧；主检出Spec hash与R2相同。
- 使用 `/auto-plan:review` 与 `autoui-verifier`：真实 Rust/Iced 窗口通过仓库 `test_vm_mcp.py` 的 AutoUiMcpClient 操作生产 `run_app_devtools`。没有复制 typed sweep 算法来充当运行证明，也没有修改实施源码。
- 独立消费者 [typed-consumer.rs](typed-consumer.rs) 链接受审树本轮 cargo 构建返回的精确 auto-lang/core artifact。该入口也是现有 merged 生成应用实际调用的入口（其 main.rs 明确调用 `run_app_devtools::<App>()`）。图像为真实 fixtures，进入窗口前已通过共享registry确认Ready。

## 2. 旧例闭合

| Finding / invariant | R3结果 |
|---|---|
| R1-01 原扫描在Drop后继续写registry | 原4096文件探针三次代次9→9、12→12、15→15，producer_continued=false；原例CLOSED |
| R1-02 顺序关闭后票据/会话准入 | dead ticket不可发布，fresh_pending=0，新会话unknown；原例CLOSED |
| **F811-R2-01** 关闭与路径转换暂停交错 | 原探针重放返回session空串、unknown、close=false，正常打开正控仍通过；**CLOSED** |
| F811-R2-02 截止/满通道oracle | 及时报错子项改善，但回收与实际满通道证明仍OPEN，见§5 |
| F811-R1-03 真实事件合同 | merged已能分发单表面零参事件；原“完全无派发路径”已改。完整身份/空闲唤醒仍OPEN，有本轮新增生产反例 |
| F811-R1-04 格式 | 上轮十文件通过；本轮两个修改文件同口径check通过，未触文件按相同提交内容限定复用；CLOSED |

[probe-r1.stdout.txt](probe-r1.stdout.txt) / [probe-r2.stdout.txt](probe-r2.stdout.txt) 为本轮独立日志。R2旧probe中复制的旧scoped helper仍打印1500ms，那是旧结构控制，不代表当前新helper表现；当前helper的证据单独记录在§5。

## 3. F811-R3-01 · P2 · typed通知只按src去重，合并了不同handler

**AC-04，V-05，T-06/T-07/T-08；新merged接线的不完整实现，原合同身份规则未变。**

canonical `docs/specs/auto-lang/ui/design/image-surface-events.md` 明确单次门按 **handler身份+src**。新 `typed_media_notify_sweep` 在 renderer.rs:32238/32244/32248 只按src保留和查找表项；不同handler共用ready_seen、loaded_emitted与error_emitted。

真实生产窗口结果（每项独立进程，持续定时器正控均稳定运行）：

| 输入 | 预期 | 实际 |
|---|---|---|
| 两个不同URI，各自LoadedA/LoadedB（正控） | A=1、B=1 | **A=1、B=1** |
| 同一Ready URI，同时LoadedA与LoadedB | 两个不同订阅各通知1次 | **A=0、B=1** |
| 一个表面保持同一URI，handler从A换B | 新handler订阅得到通知 | **A=1、B=0**（35个tick后） |
| 同一坏票URI，两个不同error handler | error A=1、B=1 | **error A=1、B=0** |

同URI两个表面还会在同一次for循环中共用ready_seen：首个表面设置它，第二个立即消费loaded标记。这里仅据实认定不同订阅的通知丢失；本轮没有用此计数直接证明具体GPU帧时间。

证据：[typed-results.json.txt](typed-results.json.txt)、各模式 `*.snapshot.txt` / `*.app.stdout.txt`，源码 [typed-consumer.rs](typed-consumer.rs)。MCP读取rendered快照，进程均实际进入生产runner并在取证后由探针回收。

修复要求：按稳定handler身份与src形成订阅身份，换handler、换src、卸载/重挂载独立处理；保持同一订阅重绘不重复。补同src/不同handler的loaded与error、handler替换、A→B→A和正常不同src正控，核对两拍状态不能被同一循环内第二个表面提前消费。

## 4. F811-R3-02 · P2 · merged事件依赖应用自带timer，空闲窗口不唤醒

**AC-04，V-05，T-06/T-07/T-08；新merged路径遗漏原合同的媒体独立唤醒。**

新sweep只挂在 `devtools_update` 尾部（renderer.rs:32546附近）。`devtools_subscription`（:32643）没有独立媒体50ms唤醒；应用的tick仅在其自身声明 `tick_interval_ms()` 时加入。MCP action分支还在到达sweep前返回。

独立组件正控带50ms应用timer时能触发loaded；同一个真实生产消费方的 `no-timer` 模式不声明timer：初始无表面，Ready资产已经就绪，通过MCP按钮挂载表面，然后等待2秒并读取rendered树：

```text
ActionResult status: ok, handler: .<path [1]>
COUNTS a=0 b=0 ea=0 eb=0 ticks=0 started=true
tree contains: [ImageSurface]
```

合法已挂载的Ready表面没有loaded。HTTP MCP轮询本身不会补发renderer更新，50ms唤醒不能由应用自带80ms SettleTick代替。现有work探针都带timer，掩盖了该遗漏。

证据：[no-timer.snapshot.txt](no-timer.snapshot.txt)、[typed-results.json.txt](typed-results.json.txt)；同二进制的带timer distinct正控成功。这里验证事件与更新链，没有声称完成跨端像素对拍。

修复要求：为merged提供由媒体完成/待通知订阅驱动的独立唤醒，控制空闲消息量，并处理所有提前返回分支后的注册/退订；补无应用timer下Ready后挂载、decode异步完成、坏票error、卸载和静止期重挂载。仍须确保loaded在真实渲染消费后派发。

## 5. F811-R2-02 仍OPEN：及时抛错不等于超时清理

**AC-03，V-03，T-01/T-03/T-07/T-08；旧finding的回收/覆盖缺口，不增加资源规模。**

本轮直接提取当前源码的 `bounded_teardown`（image_pipeline.rs:2642）作为validator反控，原文冻结于 [bounded-helper.rs.txt](bounded-helper.rs.txt)，SHA-256 `d084c2ed438c9872133eac784a7f26197e55ca9aea4a3799bd0d97d6cf836ae4`。任务持有4096字节且等待释放信号；helper使用100ms截止。

```text
TIMEOUT nominal_ms=100 returned_ms=102 failed=true
owned_worker_still_active=true owned_bytes_still_alive=true
PROBE_MANUAL_CLEANUP worker_active=false bytes_alive=false
```

及时失败子项确实修复；helper明确把线程detach，失败后没有终止/回收该自有工作。上行控制由独立探针另行释放，所以本轮没有遗留该任务。证据：[oracle.stdout.txt](oracle.stdout.txt)、[oracle_replay.py](oracle_replay.py)。当前常驻负控只测sleep任务能在800ms内catch panic，未检查资源回收；真实shutdown测试仍保留拥有MediaRuntime的栈对象，超时unwind后的Drop还会重新进入shutdown路径，该失败出口也未被当前反控覆盖。本轮不声称实际38测出现死锁。

满channel证明也不足：通道容量8，源码:2744仅断言running≥9；代码自己的说明是2个decode+1个resize+至少6个等待。**6个等待不构成8槽已满的证据**。这是oracle推断不足，不是说运行中通道事实上从未满。需要准确占用观测/可控阶段，而不是给“FULL”标签降低含义。scan测试实际仍保留sleep(15ms)，与fix-r2报告所述已改活动门不一致；新mutation_generation比stats强，但未取代受控进入窗口证明。

修复要求沿用R2§4：外层设施可在截止内判红并回收自有进程/线程/资源；负控必须连同真实owner的失败清理验证。对满resize、decode在途、scan/open关闭窗口用可观察状态或阶段闸门取证；不能仅用成功测试总数或detached线程忽略清理。

## 6. 实际验证、work收据和AC

```text
python .../review-r3/replay.py D:/autostack/.wt/lang-811/auto-lang
python .../review-r3/oracle_replay.py D:/autostack/.wt/lang-811/auto-lang
```

本轮实际完成：

- `cargo build --offline --lib -p auto-media-core -p auto-lang --message-format=json` pass；精确artifact与native链接来源保存在 `build.stdout.txt`。编译完成后UI探针首次仅因review harness沿用旧ViewBuilder调用而编译失败，已修正自身API调用，复用同一受审库完成真实运行；实施源码没有改变。
- R1/R2公开入口反例重放，旧关闭竞态CLOSED；五种真实生产Rust窗口场景，四个反例/一个不同URI正控（均是具体输入的观测，不是宣称产品验收全部通过）。
- core **38/38 pass，0 ignored，42.82s**；[core-tests.stdout.txt](core-tests.stdout.txt)。core/auto-lang本轮构建未见warning。
- 两个本轮修改文件 scoped rustfmt check pass；[format.log.txt](format.log.txt)。其余R2格式证据限定复用未改内容。最终committed diff whitespace另见稳定性收据。
- 新helper的准确超时+资源反控，如§5。

work收据已冻结为本目录 `.json.txt`、`.py.txt` 和 `work-probe-sources/`，未重写其原始结论。probe source hash与其provenance相符；其中**VM、Vue、生成HTTP JSON的worktree_commit仍写33fc23208，merged写4a1d8fcf3**。这些收据不能整体冒记为本轮对4a全部独立实跑；后续须根据binary/源码/依赖绑定证据复用或重放受影响项。最新792收据有25项全绿，本轮保存副本、不重复记作本轮实跑。

| AC | R3结果 | task / evidence |
|---|---|---|
| AC-01 | partial | T02/04/05单源/门面/VM接线保留；真实core/auto-lang编译过。17shim完整矩阵本轮未全部重跑，不发最终pass |
| AC-02 | partial | 关闭竞态及顺序正控已闭；cap/引用/TTL完整矩阵需最终验收核对 |
| AC-03 | fail | 原扫描/Weak释放正控和38测过；R2-02失败出口/满channel oracle未闭 |
| AC-04 | fail | merged实际接线已存在；R3-01/02生产反例。typed handler形参投影、真实渲染拍边界等完整矩阵仍不能据零参/timer探针标pass；源码还将收集的w/h与error reason在dispatch循环中丢弃，须补实际参数入口的证明/修复 |
| AC-05 | partial | 有发行接线与更新work792；受影响提交/输入绑定及最终知识delta尚未审结 |
| AC-06 | partial | 格式及本轮构建健康通过，当前功能/必需oracle不通过，T09未完成 |
| AC-07 | pending | T10/11，merge-owned |

没有运行裸t/tv/tu/tt/tb/taa或tf，没有跑docs_gen；没有修复实施或发布canonical/ledger/合入/清理树。真实窗口探针finally只停止其自有PID并wait，不把强制退出当媒体析构证据。

## 7. Spec delta与路由

有效SD（表+R1/R2增补）冻结于 [spec-delta-input.md](spec-delta-input.md)，SHA-256 `0aa78d4289caeb0df1e6202032a5447d1a16efdfdf127643972bcac90edac38f`。六个目标和GOAL-003/007保留；SD01生命周期完整闭合表述仍受R2-02阻断；SD04/05的“两拍/单次/身份/50ms/形参”完整行为受R3-01/02及未验证项阻断；SD03须保留主检出807/808沉淀，SD06仍需当前产物来源绑定。未改变原验收语义或plan_revision，也未通过改写Spec降低要求。

Plan → **executing 3/11**，保留T02/T04/T05，重开T01/T03/T06/T07/T08，T09仍未完成。F811-R2-01确实关闭，勿在下一轮重复修已闭原例；R2-02与完整事件合同继续work。下轮就绪前的诊断/覆盖调整至少包含：

| 漏检原因 | 需要的调整 |
|---|---|
| 单表面/单handler探针掩盖src全局去重 | 订阅身份完整矩阵：同src不同handler、handler替换、loaded/error各自正反控 |
| 所有探针都带应用timer | 无timer合法组件的媒体独立唤醒与静止期/卸载/异步完成证据 |
| 仅测通用sleep及时抛错，未验证owner失败回收；running门槛不足 | 真实owner清理的外部截止负控；准确channel/工作阶段观测 |

next：在原树work修复、完整提交与绑定闭合表后复审。merge-owned pending仍为最终review pass后冻结裸cargo t+tv、最终并集/受影响发行验收、规范/ledger沉淀、归档及guard清理。当前未改ui_gen/trans；若后续参数修复触及这些生成器，须重新计算触面门，不能沿用本轮范围。
