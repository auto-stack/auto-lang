# PLAN-811 独立复审 R4

2026-10-19（Asia/Shanghai） · stage: review · review_scope: final · **outcome: needs_fix**

受审 `27d3387af988fd7f88b3cc392e043fb98bb3026e`，revision 1。旧订阅身份例、普通无timer例与关闭准入例已通过；新增占用仪表和shutdown重入门出现实际回归，媒体唤醒仍遗漏异步入口和回调内换图。核心40测全绿不覆盖这些生产反例。

## 1. 受审基线

- 原树 `D:/autostack/.wt/lang-811/auto-lang`；branch `codex/plan-811-dev`；base `ab08955c241f6a824091c53d3b02894ca17e193b`。R3受审 `4a1d8fcf3a72fcb6a524b642669ff0340dabcf5d`，本轮单提交修改core image_pipeline与renderer（+289/-42）；累计14个实施提交。
- 同组auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；入场原树/兄弟clean，最终稳定性见 `final-baseline.json.txt`。
- reviewer_identity：当前用户请求的Codex复审上下文；本会话未实施，具体模型/会话ID未暴露；未新建树/agent。
- [baseline.json.txt](baseline.json.txt) 冻结代码、配置、依赖、规范、Plan/交接、fix-r3、work收据及本轮输入/产物hash。入场未改提案原件另存 [plan-entry-input.md](plan-entry-input.md)。
- 使用 `/auto-plan:review` 和已加载的 `autoui-verifier`。真实窗口仍通过仓库既有AutoUiMcpClient调用生产runner；没有复制事件算法代替生产行为。所有编译/测试在原实现树，证据位于主检出附件，临时构建输出在原树ignored target内。

## 2. 原例闭合与本轮实际结果

| 原项 | 本轮结论 |
|---|---|
| **F811-R3-01** 同src不同handler/换handler | **CLOSED**：same a=1/b=1，swap a=1/b=1，bad-same ea=1/eb=1；distinct正控a=1/b=1 |
| F811-R3-02 普通run_app_devtools无timer | 旧例CLOSED：挂载Ready图片后a=1/ticks=0；完整唤醒不变量仍OPEN，见§5 |
| F811-R2-01 open与shutdown受控交错 | 继续CLOSED：返回空session、unknown、close=false；正常打开正控通过 |
| R1顺序关闭、scan退出、Weak释放 | 原公开探针三次scan代次9→9、12→12、15→15；顺序dead ticket拒绝、普通Drop释放decoded Weak；原例持续通过 |
| F811-R2-02 失败清理/满通道oracle | 新协作回收与真实owner测试通过；完整证明受§3/4的仪表/关闭语义回归阻断，不能标整体CLOSED |
| 格式与核心测试 | 两个修改文件scoped format pass；核心 **40/40 pass，0 ignored，单线程43.81s** |

本轮重建auto-lang/core库并使用cargo返回的精确artifact；[probe-r1.stdout.txt](probe-r1.stdout.txt)、[probe-r2.stdout.txt](probe-r2.stdout.txt)、[core-r4.stdout.txt](core-r4.stdout.txt)、[typed-results.json.txt](typed-results.json.txt) 为新日志。R2旧probe中的旧scoped helper输出仅是历史结构控制，不用它评价当前新helper。

## 3. F811-R4-01 · P2 · resize_backlog计数可下溢，仪表不等于真实通道占用

**AC-03，V-03，T-03/T-07/T-08；本轮新增仪表回归。**

`image_pipeline.rs:1438–1440` 用两个独立原子值直接相减；`:1519` 在send成功返回后才增加enqueued，`:1570` 在recv后、区分Work/Stop前增加handled。因此：

1. 接收线程可能先recv，发送线程还未给enqueued加一，活动时也存在负差窗口。
2. Stop增加handled但没有对应enqueued，关闭后负差是确定性的。
3. 两次独立load没有一致快照保障；当前“exact occupancy”的注释与代码不符。

独立公开API只创建空闲runtime、shutdown、读取getter：

```text
BACKLOG before_close=0
BACKLOG after_idle_shutdown_panicked=true value=None
panic at image_pipeline.rs:1439:9: attempt to subtract with overflow
```

反例在catch_unwind内完成，进程exit0表示取证执行完成，不表示产品通过。若某构建关闭溢出检查，负差还可能表现为巨大u64并被误判“≥8已满”；本轮没有运行release来虚构该结果。

证据：[core-r4.rs](core-r4.rs)、[core-r4.stderr.txt](core-r4.stderr.txt)。修复应定义并实现与实际队列一致的工作消息占用，排除Stop、处理send/recv竞争及一致读取；不能仅用饱和减法遮住判定错误。补空闲、在途、排空、关闭和竞争读数控制，再用真实满8槽反例验证oracle。

## 4. F811-R4-02 · P2 · shutdown_started将“关闭中”当作“关闭完成”

**AC-02/03，T-01/T-03/T-07/T-08；为超时负控新增门导致的生产生命周期回归。**

`shutdown_runtime_inner(:2109–2135)` 首调用取得swap门，第二调用在`:2114–2115`直接返回；registry关闭仍在worker join之后。这避免重复Stop，但也让重复/并发shutdown在第一调用尚未结束时报告返回。

独立公开消费者使用与work负控相同的合法3200×2400 PNG与1920×1440 rendition。待观察到真实工作在途，线程1调用shutdown，线程2等到closing锁存后再次shutdown：

```text
SHUTDOWN accepted=10 running_before=4
SHUTDOWN second_returned_ms=0 registry_shutdown=false pending=24
new_ticket_pending=true new_transition_accepted=true
CONTROL first_joined=true registry_shutdown=true pending=0
```

第二次shutdown返回后，同一个registry仍未关闭，还接受新ticket和Reading转换；等待第一次真正完成后正控才归终态。pending=24包含低层queue后未成功submit的ticket，不指24个线程或24个在途codec；缺陷依据是关闭返回时registry仍开放和后续实际准入。

本例复现的是重复shutdown。Drop也复用同一helper，因此新门同样绕过了正在进行的完成等待；本轮不把源代码推导写成另一个已实测Drop释放反例。

work的真实owner负控把第二次关闭“廉价返回”作为通过条件，随后另等pending清零，正好没有验证“shutdown返回即完成”的合同。canonical/Plan§5.2要求owned shutdown/drop收完自己的scan/open/decode/resize，而允许异步的外部session close不授权改变这个完成语义。

修复要求：区分closing/closed；首调用独占执行，重复调用等待同一个完成结果，不重复发送Stop，也不提前宣称完成。超时设施在独立有界清理域验证，不通过改变生产生命周期完成语义让计时为绿。补并发重复关闭、Drop与在途关闭、旧新票据和资源回收控制。

## 5. F811-R3-02 继续OPEN：异步入口和回调内换src仍无后续媒体唤醒

**AC-04，V-05，T-06/T-07/T-08；原完整唤醒/换源合同的剩余路径，未新增范围。**

### 异步启动路径未接心跳

新增50ms媒体订阅只在 `run_app_devtools(:32863)`；生产支持的 `run_app_with_task_devtools(:33229)` 的subscription仍只合并普通事件与应用tick，没有该订阅。它也是生成async-init应用的正常runner，不是新增平台。

用相同独立组件通过此公开入口启动、boot返回空Task（合法正控条件）、不声明应用timer：MCP挂载同一Ready图片后等待2秒，`a=0 b=0 ticks=0 started=true`；同步入口同样输入为`a=1`。

### loaded回调内换到已Ready图片，表未重新注册

同步入口中，A的loaded handler把当前src与handler改成已Ready的B（B在窗口启动前已实际解码）。sweep依据回调前视图更新表，派发A后组件视图变B；旧表A已标loaded，pending为0，`typed_media_active`于是停发消息，B没有注册机会。

真实窗口七场景结果：

| mode | 运行入口与条件 | 实际 |
|---|---|---|
| distinct | 不同URI、应用timer正控 | a=1 b=1 |
| same | 同URI、不同handler | a=1 b=1，旧例已闭 |
| swap | 同URI换handler、应用timer | a=1 b=1，旧例已闭 |
| bad-same | 同坏URI、不同error handler | ea=1 eb=1，旧例已闭 |
| no-timer | 普通runner、无timer、按钮挂载Ready A | a=1，旧例已闭 |
| **async-no-timer** | with-task runner、无timer、相同挂载 | **a=0** |
| **cascade** | 普通runner、无timer，A.loaded改绑Ready B | **a=1 b=0**，应分别单发 |

每项独立进程、真实Iced生产runner、真实共享registry图像；源[typed-consumer.rs](typed-consumer.rs)，日志[typed-results.json.txt](typed-results.json.txt)、各模式snapshot/stdout。测试进程全部由探针finally回收，没有把terminate作为媒体生命周期通过证据。

修复要求：复用一致的媒体订阅装配覆盖同步/async启动；回调改变视图后保证注册/退订与下一渲染拍持续调度，不要求应用额外timer或用户再点一次。补Ready/Pending、handler回调内换源/卸载/恢复与异步入口的组合控制。继续保护空闲零消息及渲染后单发。

## 6. 实际检查、证据限制和知识调和

重放：

```text
python D:/autostack/auto-lang/docs/plans/attachments/811/review-r4/replay.py D:/autostack/.wt/lang-811/auto-lang
```

实际通过：原树精确库构建；旧R1/R2原例；七个真实窗口的具体取证执行完成；核心 `cargo test --offline -p auto-media-core --lib -- --test-threads=1` **40/40**；两个改动文件rustfmt check。核心压力用例串行，避免两条大图负载同时跑。build/test无warning。未运行裸t/tv/tu/tt/tb/taa/tf或docs_gen；没有实施修复/合入/清理树。

work的五份JSON原件已保存为同目录`.json.txt`，脚本与probe-sources也冻结。虽然fix-r3称它们全绑定27d3387af，**原件provenance实际上都记录4a1d8fcf3**。可能是提交前WIP测试，但需要WIP→提交的源码/依赖/产物等价收据，或受影响重放；不能自动改写原始SHA再算独立pass。本轮当前commit的生产反例完全使用本轮重建的27库。最新792旧收据保存而未重跑，不冒记为当前独立发行验收。

主检出已新增PLAN-809的canonical管理核心分工（`docs/specs/auto-man/project.md:371`），旧SD-05“项目模型/UI工具搬迁仍未发生”过时。review已将Plan的拟议SD-05改为保留auto-man-core单一规则源与语言效果adapter分工；此为并行知识调和，没有改变AC或plan_revision，也没有修改canonical。入场旧提案有独立副本，handoff历史文句由当前提案明确覆盖。

有效SD冻结为 [spec-delta-input.md](spec-delta-input.md)，SHA-256 `de17319fa411c7e9f112bc9b4733f252f1928bfe186408620c87a3e8bfd31b09`。SD01/02完整关闭承诺受R4-01/02阻断；SD04/05完整事件合同受R3-02阻断，typed零参探针不等于参数投影/真实渲染拍全覆盖；SD03保留807/808已迁事实，SD06需要最终受影响产物绑定。五个supersedes、一个new目标及GOAL-003/007保留，未写live ledger。

## 7. AC与下一步

| AC | R4结果 | 依据 |
|---|---|---|
| AC-01 | partial | 单源抽取/门面及native接线无新变更；精确core/auto-lang库编译通过，完整17shim矩阵未本轮全部重跑 |
| AC-02 | fail | 重复关闭返回后registry仍接受新票据；旧open/close顺序与竞态例保持已闭 |
| AC-03 | fail | R4-01/02；40测通过、协作回收子项成立，不能证明不正确仪表/提前关闭语义 |
| AC-04 | fail | R3-01与同步无timer旧例关闭；完整唤醒还有with-task/callback换源缺口；参数/渲染拍完整证据仍待收口 |
| AC-05 | partial | 发行接线/旧792与本轮冻结存在；收据revision等价需说明，SD05已作当前canonical调和 |
| AC-06 | partial | 格式/构建健康通过，必需实现/验收不通过，T09未完成 |
| AC-07 | pending | T10/11，merge-owned |

Plan → **executing 3/11**，保留T02/T04/T05，重开T01/T03/T06/T07/T08。下轮work诊断应区别已闭旧例、完整不变量和修复回归：

| 原因 | 设计/oracle调整 |
|---|---|
| 将send/recv计数差当一致队列占用，漏Stop/竞争 | 准确工作消息占用与排空/关闭反控，不以溢出/钳位冒充满通道 |
| 为负控避免等待而让重复shutdown提前返回 | 独占执行与所有调用者的完成等待分开建模，失败清理在独立有界设施验收 |
| 只验同步入口、用户按钮/timer驱动更新 | 同步/async统一装配，事件回调改变视图后的下一拍与注册生命周期覆盖 |

next：原树work修复/补证据，提交后按影响复审；保留已闭身份/原准入/格式与正常消费正控，不重新修无关实现。merge-owned仍为最终pass后冻结裸cargo t+tv、最终并集合流/受影响发行、规范ledger沉淀、归档与guard清理。当前未触ui_gen/trans；后续若为完整参数合同改生成器，须重算门禁触面。
