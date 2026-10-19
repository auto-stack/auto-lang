# PLAN-811 独立复审 R6

2026-10-19（Asia/Shanghai） · stage: review · review_scope: final · **outcome: needs_fix**

受审 `2ffb87755e482fb39f01123d28e3f82fa3e373c1`，Plan revision 1。本轮已关闭占用残留和新增warning；当前只剩**满通道验收用例的负载前置条件不充分、实跑不稳定**，归原F811-R2-02的必要oracle，不新增产品能力或压力规模要求。

## 1. 基线

- 原唯一树 `D:/autostack/.wt/lang-811/auto-lang`，branch `codex/plan-811-dev`，base `ab08955c241f6a824091c53d3b02894ca17e193b`；本轮单提交只改core image_pipeline（+144/-31），累计16个实施提交。
- auto-down兄弟 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场双方clean；[baseline.json.txt](baseline.json.txt) 冻结提交、依赖、实施/config、规范、Plan/SD、fix-r5和work原始输入。结束稳定性见 [final-baseline.json.txt](final-baseline.json.txt)。
- reviewer_identity：本用户请求的Codex复审上下文，本会话未实施；具体模型/会话ID未暴露。使用 `/auto-plan:review` 及既有autoui-verifier MCP客户端；未创建agent或新工作树。
- 仅在原树构建/定向验证；主检出仅附件和Plan簿记。未修实施、运行整档t/tv/tu/tt/tb/taa/tf或docs_gen，未发布canonical/live ledger、合入、归档或清树。

## 2. 当前修复已闭合

| Finding / 控制 | R6实际结果 |
|---|---|
| **F811-R4-01 占用残留/下溢** | **CLOSED**。先join decode、后Stop、后join resize，堵住Stop后接受Work；send成功后计数排除阻塞发送者；停止快路径处理已接收Work。空闲关闭0且不panic；原occupancy-r5三次输入均在首图Transforming且Pending、次图读出metadata后关闭，均after_shutdown=0、workers=0、registry_shutdown=true、pending=0 |
| **F811-R5-01 unused import** | **CLOSED**。新增未用Ordering已删；core `cargo check --offline -p auto-media-core --tests` exit0，warning0；core test cfg也无warning |
| F811-R4-02 重复关闭完成语义 | 继续CLOSED。重复调用等待3924ms，返回registry已关/pending0，新ticket与Reading均拒绝；first join完成 |
| F811-R3-02 同步/async无timer与回调换图 | 继续CLOSED。no-timer a=1、async-no-timer a=1、cascade a=1/b=1，ticks均0 |
| F811-R3-01 handler身份 | 继续CLOSED。distinct/same/swap a=1/b=1，bad-same ea=1/eb=1 |
| R2 open关闭竞态、R1准入/scan/Weak | 原输入继续通过；open返回空session且unknown、live正控通过；三次scan关闭后代次9→9、12→12、15→15；普通Drop释放decoded Weak |

关键原件：[occupancy-r5-run.stdout.txt](occupancy-r5-run.stdout.txt)、[core-r4.stdout.txt](core-r4.stdout.txt)、[core-check-tests.stderr.txt](core-check-tests.stderr.txt)、[typed-results.json.txt](typed-results.json.txt)。探针源与R5相同，已绑定hash。本轮七窗口真实生产runner按finally回收进程；terminate不作为媒体Drop证据。旧R2脚本的复制scoped helper仍打印1500ms，仅历史对照，不评价现行helper。

占用计数是跨线程累计采样；0不能单独证明所有worker闲置，本轮完成性使用join、registry终态及关闭后计数0联合验证。满通道的负载建立仍单列为下节oracle；没有把采样包装为整个系统瞬时线性化快照。

## 3. F811-R2-02 继续OPEN · P2 · 满通道fixture仍依赖提交/取队列的调度时序

**AC-03 / V-03 / T-01、T-07、T-08。** 当前实施的计数与退出原例已修；此处剩余的是原合同必需的“真正满通道+关闭”验收控制，不是新的生产缺陷指控。

独立模块定向命令：

```text
cargo test --offline -p auto-media-core --lib -- --test-threads=1
```

实际为 **42 passed / 1 failed / 0 ignored，30.03s**。唯一红项 `shutdown_with_real_decode_backlog_completes_within_external_deadline`：

```text
panicked at image_pipeline.rs:3240:9:
in-flight states observed: sampled running peak 8
```

见 [core-tests.stdout.txt](core-tests.stdout.txt)。失败发生在建立忙/满前置条件，**尚未进入受外部截止保护的shutdown调用**；不是本轮观察到的shutdown超时。

复核现存已编译、同hash的test binary，单独执行原测试三次：**3/3 pass**，各4.59s/4.53s/4.48s，全部原始结果保留于 [full-channel-trials.json.txt](full-channel-trials.json.txt)。首次模块测试内部为单线程串行；当时本轮另有小型公开在途探针和VM/Man定向编译在执行，未做机器独占隔离。重放支持该用例对调度敏感；没有用后续绿覆盖初次红，也没有证明是某一特定外部负载造成。

### 不充分的前置条件

`image_pipeline.rs:3203–3225` 只紧循环尝试12次submit，失败后丢弃该次机会；`:3236–3238`只要求accepted≥8。但入口 `MediaPriorityQueue::CAPACITY` 为8（`:902`），同优先级满队列拒绝submit。因此合法调度下，主线程先塞满入口、decode尚未取出时，这12次尝试可以只接受8个任务。

随后却要求peak_running≥9（`:3240–3242`）和真实resize队列8槽占满（`:3244–3246`）。仅8个被接受任务无法保证这两个前提；至少还要计入已经被resize取走的任务，且要防止它们在造载期间完成。增大轮询时长不能创造已被入口拒绝的任务。首次日志没有打印accepted/peak_backlog，本轮**不声称已实测该次恰好accepted=8**；实际peak_running=8与源码前提不充分分别为运行证据与结构分析。

### 修复范围与验收

- 仅需修正验收造载/观测：在有界期限内明确保证足够任务被接受，建立可观察、可控的满resize通道状态；失败应报告accepted/current-running/backlog等实际前提。
- 可在现有单元测试接缝控制接收/变换进度或有界补入任务；不要求新增生产调度协议、扩大图像规模或提高资源上限。
- 需要“未满”的有效反控、达到真实满状态的正控，以及从该状态shutdown完成/资源回收。不可删除满队列断言、把running阈值降到8冒充通道满，或依靠重跑取绿。
- 保留本轮已通过的旧在途关闭归零、重复关闭、失败清理、坏输入和正常消费控制。原helper的协作回收及真实owner超时回收子项已通过；F811-R2-02只剩该fixture前置条件，不能整体标CLOSED。

## 4. 其他实际检查与证据复用

| 检查 | R6结果 |
|---|---|
| 精确库build、旧公开消费者 | 通过；链接cargo返回的当前artifact |
| 核心模块43项 | 首跑42/43，唯一红及三次单测重放如上；不写“全绿43/43” |
| VM/core/17接口接线定向 | plan811 **7/7**，0.04s |
| Man真实core消费与生成接线 | **3/3**，0.02s |
| 真实生产Iced七窗口 | 七场景取证执行完成，预期计数全部满足 |
| 生成媒体HTTP真实TCP | 独立新跑 **13/13**；[新收据](http-replay/gen_media_http.json.txt)绑定当前服务器hash；未夸称本轮重建服务器 |
| 777 strict / scoped rustfmt | exit0 / exit0；[scoped-checks.json.txt](scoped-checks.json.txt) |
| core test cfg check / warning | exit0 / 0条；auto-lang历史test target仍有562条warning，不冒记全仓零warning |

六份work收据均记当前 `2ffb87755…`，共VM12/Vue10/merged11/identity6/async4/HTTP13。来源/现存二进制hash核对见 [work-artifact-binding.json.txt](work-artifact-binding.json.txt)；五形态全部匹配，旧HTTP收据服务器hash与现存exe不同，已通过本轮独立13项重跑重绑，未猜测差异来源。此处work收据是核对/复用，不冒记review全部重新驱动。

792当前work原件`driver-792-r5fix.json` **25/25**已冻结，未独立重跑driver。继承前轮对未变化发行/生成/manifest部分的审查：本轮仅core关闭及计数/测试文件变化，依赖、配置、模板、fixture与验证脚本摘要已核对；运行受影响核心/消费者/HTTP验证。最终冻结合流/发行门仍由merge执行。

可重放脚本：`replay.py`（保留核心失败exit101）、`extra-replay.py`、`replay-full-channel.py`、`scoped-checks.py`、`check-bindings.py`，均位于本目录；传原树绝对路径。后续work请把输出放新目录，避免覆盖本轮红绿原件。

## 5. AC、SD与状态

| AC | R6结果 | 说明 |
|---|---|---|
| AC-01 | pass | 单源/轻闭包/门面/native接线未改；当前精确build、7项接线及消费者证据 |
| AC-02 | pass | 原准入/open/重复关闭与生命周期正反控保持 |
| AC-03 | partial | 退出顺序/占用归零/格式/预算/原资源控制通过；满通道必需oracle当前不稳定，不能完整pass |
| AC-04 | pass | 七窗口、当前三形态work证据核对、HTTP13新跑，保留R5审查边界 |
| AC-05 | pass | 未改发行/依赖/生成闭包有指定证据复用；当前777、消费及SD审查；最终合流门pending |
| AC-06 | fail | 新warning闭合，但必需定向模块用例有未处理红；本轮不能最终pass |
| AC-07 | pending | merge-owned |

SD原件 [spec-delta-input.md](spec-delta-input.md)冻结，五个supersedes、一个new与GOAL-003/007保留；canonical hashes同本轮baseline，当前Man 809已迁规则源/effect adapter分工优先于旧handoff历史“未迁”文句。SD01/02有界关闭的生产原例已过，完整验收记录须待满通道oracle稳定后完成；SD03/04/05/06无新增实现/边界冲突。未改canonical/live ledger或语义合同/revision。

Plan → **executing 5/11**：保留T02/T03/T04/T05/T06，重开T01/T07/T08，T09未通过。本轮关闭F811-R4-01及F811-R5-01；当前仅F811-R2-02的满通道fixture未闭，不要求重修已通过的生产计数/关闭或事件实现。

next：原树work只修受控满队列oracle与诊断，提交后按影响复审。最终pass后merge-owned冻结裸cargo t+tv、最终并集/777/受影响发行、canonical/ledger、归档/guard清理仍pending；当前无合入或交付完成声明。
