# PLAN-811 独立复审 R5

2026-10-19（Asia/Shanghai） · stage: review · review_scope: final · **outcome: needs_fix**

受审提交 `53f1111a2923a2bcca0096f973eb714a94147a06`，Plan revision 1。重复关闭的完成语义、异步启动和回调内换图原例均已闭合。占用仪表只修复了空闲关闭下溢，在真实解码与关闭交错时仍留下非零读数；另有一条本次新增测试 warning。

## 1. 基线与范围

- 唯一原实现树 `D:/autostack/.wt/lang-811/auto-lang`，branch `codex/plan-811-dev`；base `ab08955c241f6a824091c53d3b02894ca17e193b`。R4→R5 为同树单提交、两个实施文件；累计15个实施提交。
- 同组 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；入场双方 clean。原树内构建及定向测试，主检出仅审查附件与Plan簿记；未修实现、发布canonical、更新live ledger、合入或清树。
- reviewer_identity：当前用户请求的Codex复审上下文，本会话未实施；具体模型/会话ID未暴露。未创建agent。使用 `/auto-plan:review` 与 `autoui-verifier` 既有MCP客户端。
- [baseline.json.txt](baseline.json.txt) 冻结实施/配置/依赖、当前canonical、Plan/SD/交接、fix-r4及work原始收据。结束时另核 [final-baseline.json.txt](final-baseline.json.txt)。

## 2. 旧例闭合与当前不变量

| Finding / 旧例 | R5结论与独立结果 |
|---|---|
| F811-R4-01 空闲shutdown后下溢 | **旧例CLOSED**：`panicked=false value=Some(0)`；完整占用/关闭归零不变量仍OPEN，见§3 |
| **F811-R4-02** 重复shutdown提前返回 | **CLOSED**：第二调用等待6479ms，返回时registry_shutdown=true、pending=0，新ticket与Reading均拒绝；first join正常 |
| **F811-R3-02** async无timer、回调内换图 | **CLOSED**：async-no-timer a=1；cascade a=1/b=1；两者ticks=0；同步no-timer a=1 |
| F811-R3-01 handler身份 | 继续CLOSED：same/swap a=1/b=1；bad-same ea=1/eb=1；distinct a=1/b=1 |
| F811-R2-01 open/shutdown受控交错 | 继续CLOSED：returned_session空、unknown、close=false；live session正控通过 |
| R1 scan/准入/Weak | 三次scan关闭后generation分别9→9、12→12、15→15；关闭后票据不复活；普通Drop释放decoded Weak |
| F811-R2-02 满通道/失败清理oracle | 完成等待与协作回收子项成立；“backlog≥8证明8槽全满”仍由不准确仪表支撑，不能标整体CLOSED |

公开核心重放日志 [core-r4.stdout.txt](core-r4.stdout.txt)，旧R1/R2探针及七窗口日志均在本目录。R2探针自带旧scoped helper的1500ms输出仅为历史对照，不用它评价现行helper。真实窗口各自独立进程并在finally回收；terminate不是媒体Drop通过证据。

## 3. F811-R4-01 继续OPEN · P2 · 在途关闭后resize_backlog仍为1

**AC-03 / V-03 / T-01、T-03、T-07、T-08；R4仪表修复不完整，不是新增负载要求。**

现在send前预留Work计数、Stop不计、getter饱和减，空闲下溢已修。但生产路径仍是：

1. `MediaWorkerPool::shutdown` 在 `image_pipeline.rs:1458` 发送Stop，然后join worker；此时已取出的decode任务仍可继续。
2. decode在 `:1523–1524` 预留并发送Work。若Stop已经排队、resize正在处理前一张图，后来Work可成功排在Stop后。
3. resize在 `:1579–1584` 收到Stop即break，只有真正recv到的Work会增加handled；Stop后的已接受Work随receiver销毁，却没有结算计数。

独立消费者 [occupancy-r5.rs](occupancy-r5.rs) 直接调用本提交公开API，没有复制调度算法或修改实施。使用修复测试同尺度的合法3200×2400 PNG、1920×1440 rendition：第一张已进入Transforming且仍Pending，第二张已读出metadata、仍在解码时关闭runtime。三次均得到：

```text
OCCUPANCY trial=0 first_pending_at_second_submit=true before_shutdown=0 after_shutdown=1 workers=0 registry_shutdown=true pending=0
OCCUPANCY trial=1 first_pending_at_second_submit=true before_shutdown=0 after_shutdown=1 workers=0 registry_shutdown=true pending=0
OCCUPANCY trial=2 first_pending_at_second_submit=true before_shutdown=0 after_shutdown=1 workers=0 registry_shutdown=true pending=0
```

原始 [occupancy-r5-run.stdout.txt](occupancy-r5-run.stdout.txt)。线程数、registry终态均正常，所以这是**错误仪表读数**，不是已证明的worker/像素泄漏。不能以pending=0消掉这个用于验收的错误读数。新增生命周期测试先等全部Ready再shutdown，只覆盖排空后的关闭，没有覆盖此交错。

此外，send前预留也包含尚在阻塞发送的decode线程；该差值是“预留但未接收”的数量，不能直接称为channel占用。先读旧handled、再读新enqueued还可能高报，注释“只会低报、永不高报”方向错误。当前满队列测试 `:3114–3133` 仍用peak_backlog≥8证明容量8已满。本轮直接实测的是关闭后残留1；不把源码分析冒记为已经观察到某个瞬时假满样本。

**修复要求**：分别定义真实通道占用、阻塞发送及停止时丢弃/结算；所有返回完成后的读数与实际终态一致。若保留预留量，应如实命名，并为“8槽满”提供可区分的真实oracle，不能继续直接由该量≥8推出。补解码在途、Stop先于后续Work、正常排空、关闭与并发读数控制；保留当前已闭下溢/重复关闭例。仅饱和减或事后清零不能证明满通道oracle正确。

## 4. F811-R5-01 · P3 · 新增测试引入unused import

**AC-06 / T-07、T-08**。`image_pipeline.rs:2914` 新增 `use std::sync::atomic::Ordering;` 在并发关闭测试中未用。

本轮核心测试42/42通过，但实际stderr为：

```text
warning: unused import: `std::sync::atomic::Ordering`
    --> crates\auto-media-core\src\image_pipeline.rs:2914:13
warning: `auto-media-core` (lib test) generated 1 warning
```

见 [core-tests.stderr.txt](core-tests.stderr.txt)。普通lib build无warning不覆盖test cfg；fix-r4“零warning”陈述不足。清除此新增导入并按相同定向命令确认即可，不要求修复无关历史warning。

## 5. 验证与收据归属

- 新构建精确cargo artifact，独立公共消费者链接该artifact；七个生产Iced窗口由既有AutoUiMcpClient驱动，输入与R4同源。
- `cargo test --offline -p auto-media-core --lib -- --test-threads=1`：**42/42，0 ignored，46.28s**，有上述1条新warning。大图族串行。
- `cargo test --offline -p auto-lang --lib plan811 -- --test-threads=1`：**7/7，0.03s**；真实VM shim、16固定ID+thumb按名、共享类型与消费路径通过。该历史test target另有562条warning，未将其冒记“零warning”或全归本Plan。
- 两个修改文件scoped rustfmt检查通过。
- Man定向consumer **3/3**、777 strict **exit0**，最终结果见 [scoped-checks.json.txt](scoped-checks.json.txt)；只运行所列定向命令，未跑整档t/tv/tu/tt/tb/taa/tf或docs_gen。
- 六份work JSON均记录当前完整 `53f1111a2…`，计数为VM12、Vue10、merged11、identity6、async4、HTTP13，原始断言均绿；此轮修复了旧SHA元数据问题。VM/Vue/merged/identity/async的来源与现存执行产物hash也相符，见 [work-artifact-binding.json.txt](work-artifact-binding.json.txt)。这些标为work证据核对，不冒记review独立实跑。
- 生成HTTP旧收据的服务器hash与入场现存exe不同，原因未从hash本身推断；本轮直接对现存exe独立重放，**13/13**，新产物hash/来源/commit见 [HTTP新收据](http-replay/gen_media_http.json.txt)。该运行没有重建服务器，不夸称本轮fresh构建。
- 792的当前work收据`driver-792-r4fix.json`为**25/25**，已冻结；本轮没有重新运行发行driver，最终冻结合流/受影响发行仍属merge门。

重放：

```text
python docs/plans/attachments/811/review-r5/replay.py D:/autostack/.wt/lang-811/auto-lang
python docs/plans/attachments/811/review-r5/extra-replay.py D:/autostack/.wt/lang-811/auto-lang
python docs/plans/attachments/811/review-r5/scoped-checks.py D:/autostack/.wt/lang-811/auto-lang
```

## 6. AC、SD与下一步

| AC | R5结果 | 证据/边界 |
|---|---|---|
| AC-01 | pass | 结构/原生接线与门面无本轮修改；新精确库build、7项VM/core接线、当前work媒体/发行证据核对 |
| AC-02 | pass | 原open竞态、顺序准入、当前重复关闭完成等待及低层新票据反控通过 |
| AC-03 | fail | 核心42测通过，旧资源例保持；占用/满通道oracle由§3阻断 |
| AC-04 | pass | 原身份矩阵及同步/async/回调换图七窗口独立重放、当前work三形态断言/来源核对、HTTP13新跑；形参投影沿canonical VM轨当前收据，typed探针仅零参，不冒称其投影全覆盖 |
| AC-05 | partial | 当前work792/真消费者与SD05事实调和有效；§3使SD01关闭/满通道完整证明未完成 |
| AC-06 | fail | 当前独立验收未通过；新unused import尚未处理 |
| AC-07 | pending | merge-owned |

有效SD冻结 [spec-delta-input.md](spec-delta-input.md)，保留五个supersedes、一个new及GOAL-003/007。当前canonical Man已落地的809规则源/语言效果adapter分工优先，旧handoff未迁文句由当前Plan SD05及冻结调和段覆盖；overview新增其他模块记录未改变媒体合同。SD01/02完整生命周期/仪表承诺需先修正实现和oracle，SD04/05已闭事件缺口不再重开。不修改canonical或live ledger。

Plan退回 **executing 4/11**：保留T02/T04/T05及本轮已通过的T06，重开T01/T03/T07/T08；T09未通过。原树work修复剩余仪表不变量及1条新增warning，提交后按影响复审。不得再次只重放空闲关闭来代替在途关闭/满队列证明。

merge-owned仍pending：最终pass后在冻结提交上裸cargo t+tv、最终并集/777/受影响发行、canonical/ledger沉淀与归档guard清理；review通过也不等于已交付。此轮未改变语义合同、Plan revision或批准延期/缩面。
