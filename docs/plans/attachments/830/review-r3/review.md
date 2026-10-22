# PLAN-830 final review R3 — needs_fix

2026-10-22。本次是第三轮 final review，延续 R1/R2、独立于实施的审核上下文。未创建子 agent，未修改实现、未合入/归档。原 R2 六条的多数修复已确认，剩余为焦点不变量的两个出口，以及原 V09 上传/内存证据遗漏。

stage: review；review_scope: final；reviewer_identity: 当前独立审核上下文（Codex）；plan_id: PLAN-830；plan_revision: 3；outcome: needs_fix。未获知或虚构独立模型/会话ID。

## 冻结输入

- 主检出计划最近提交 `ba60f35723d2a422fd1f96439d80c5e21b81ab83`，r3、进场 execution_done 8/10；[计划输入副本](plan-input.md) SHA256 `e0019e6d968e30a53d9283f7a5bf9e8247d2535fff5f2afda09595caa675887c`。本轮不改变语义 revision。
- 同一实施树 `D:/autostack/.wt/lang-830/auto-lang`，分支 `plan-830-dev`，`reviewed_commit: 5aec0a63429d0a4cd3253751b195e8f74ddc2976`；`base_commit: ab56c32ec4cb28e01fbcca74e9861164ffad43f0`；`repair_diff_base: dbaea29e122d2fa2a18d18ce3f09444583098efd`。进/出场 clean、HEAD 无漂移；主检出其他任务 dirty 保持。
- AGP Cargo.lock 未变，SHA256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`。本轮重新 locked build，正式报告 fingerprint 均为 `5aec0a63429d`。
- canonical Spec 输入与 R2 同 hash：project `a706ef0e5f2f6db3a82ec7c8c2cb6abedd2f408644656129221dc0a2012eaeb3`；prototype-profile `59b42126811545156b6d5339aacde156974f43259fab51bf6191dc174d527832`；integration-contract `1553c8f94b6a407946e7380d9e14edc0e137cfd6be65f851eeedb01c81f05669`。
- SD01 [冻结副本](SD-01-frozen.md) SHA256 `cde57f5f326b4cef39ef1ca2193be5501ead233d182614a81a5d4e3988436eb6`；SD02 [冻结副本](SD-02-frozen.md) SHA256 `042473ab2801ec05b19f0405c4a9cd4bf6d718c5fb61a2e3571db28179d16ace`。SD03/04仍在 plan-input §5.5；两个modify/两个add与GOAL007/008的元数据归属保持。未发布canonical或ledger。
- 技能来源延续R1/R2所记录的auto-plan-review/verification-contract，本轮再次读取review技能，并核实第二轮要求的work诊断/闭合表已写入plan §9 work r4；记录完整不等于事实已过，以下从真实调用再验证。

文件行号均绑定5aec0a634，路径默认前缀 `experimental/autographics-prototype/`。实现可用git show恢复；本目录原始证据清树后仍可引用。

## 本轮独立执行

| 检查 | 实际结果/证据 |
|---|---|
| locked check / fmt / release build | 全exit0、产品零warning；[check](check.txt)、[fmt](fmt.txt)、[build](build.txt) |
| 定向契约/新增修复回归 | surface_contract **29/29** + surface_review_r2 **5/5**，[输出](contract-tests.txt)；作者151/151总数未在本轮重复冒记为独立全测 |
| 原R2四项holdout | 原文件同字节SHA256 `87e824c835baad4b4032dd4341b7e4c733924bb3cd97da821435c3773a4534d7`，对新rlib重链，**4/4通过**，[输出](r2-holdouts.txt) |
| 六surface正控 | offscreen6/compose17/input18/IME14/scenarios15/lifecycle15，全exit0；[offscreen](surface-offscreen.json)、[compose](surface-compose.json)、[input](surface-input.json)、[IME](surface-ime-os.json)、[scenarios](surface-scenarios.json)、[lifecycle](surface-lifecycle.json) |
| 真实OS输入/IME | 窗口输入通过、preedit/CJK commit=`abc你好你好`，真实caret与滚动/挂载门14/14，见IME原始报告；不使用合成事件顶替OS证据 |
| 829兼容 | [Breakout](ag2d-breakout.json)、[Squash](ag3d-squash.json)、[iced-shared-device](iced-shared-device.json)全部exit0 |
| 独立消费/重定位 | [报告](independent-consumer.json) pass，A→B 29字段0 mismatch、根生产0 collateral；本轮实际执行 |
| 修复后场景oracle反控 | 新同字节场景源SHA256 `fc07862c6c161d98ea5d8683416c5380d9191e4d9fa59725636503a35f82180b`，Squash辅助源 `e537d0369d9db9c7c7f21c58e17771116cd92ec98b8bfe9bf5a42fd55aff0eff`；[control](scenario-control.json)15/15 exit0；原9999故障现在[8门红、exit1](scenario-corrupt-local.json)，[trace](scenario-corrupt-local.txt)证明mutation真实发生。未拿旧R2源作新证据 |
| 新焦点组合/拒绝出口 | [独立源码](combinations.rs)、[输出](combinations.txt)、[脚本](replay.ps1)：**1合法对照通过、2失败**，详见R3-01/02 |

未跑根cargo t/tv/tt/tb/tu/taa/tf/docs_gen，未执行merge步骤。所有构建/运行均在原实施树；审核源码/冻结副本仅写入审核附件，产物在scratch。

## 原R2闭合与保留边界

| R2 finding | 本轮结论 |
|---|---|
| 01 活体输出传播 | CLOSED：真实message→子面→所有消费者同帧；原holdout、双消费者回归、静态alpha/正式case通过，adapter的输出变化分支已核 |
| 02 控件焦点迁移 | 原分帧A→B和blur/public正常focus例已关闭；**完整不变量PARTIAL**：同帧合批与非法focus拒绝出口仍失败，R3-01/02 |
| 03 真实caret锚点 | CLOSED：State被保留，RedrawRequested产生真实cursor，OS每帧更新真实caret投影；单框输入/删除/滚动与真实IME通过，保留已排除的候选窗模板比对边界 |
| 04 设备全adapter | CLOSED：3D新增归属，Iced/2D/3D外设备/未知raw全部拒绝、同设备正控通过，原holdout与正式负控通过 |
| 05 创建/drop计数 | CLOSED：原create→resize→drop变为created2/drop2/live0，no-op/失败准备/三类型与正式守恒门通过；这部分的关闭不抵扣R1另有的上传/内存证据要求 |
| 06 场景消费路由结果 | CLOSED：正式控件消费实际surface/local、按键消费实际目的地；新源9999变异8红/exit1，原假绿已关闭 |

R1代际/批次/cycle/depth/结构dirty/递归遮挡/相机/资源last-good/卫生的已闭成果保持；相关29契约和正式回放在新基线通过。管线静态error-scope缺口、Null剪贴板、GPU timing=false、生产I1–I3/AutoShader/CAP仍按原边界如实披露，不将其扩为新验收任务。

## Findings（3项，均既有合同内；不授权延期）

### F-830-R3-01 / P1 / 代码 / R2-02不变量未闭：一帧多个目标的输入合批仍留下双焦点

AC04/05、T05/T06，§5.3唯一焦点/取消与SD02 §4。现有真实宿主以`pending: HashMap<SurfaceId,Vec<Event>>`积累事件，到about_to_wait才交给frame（`src/host/cases/surface_input.rs:1084-1217`；IME宿主同样收集再渲染）。不是人为要求新输入规模：仅同一帧两个正常点击。

实际入口/反例：`combinations.rs::two_clicks_queued_before_one_frame_keep_only_last_focus`，从CompositionInput按先A后B调用press/release，把返回的真实surface/local积累到宿主同构pending/cursors，再**一次**frame_with_cursors。router最终=B；两个真实text_input实际focused=(true,true)。随后window_focus_lost返回B并释放B，A仍focused=(true,false)。合法分帧A→B对照为(false,true)；原R2焦点/blur holdout另行通过。

已确认原因：`src/host/surface_runtime.rs:546-576`只从HashMap取一个有ButtonPressed的目标，在本帧任何事件交付前清掉其他控件；随后不同面的事件列表都运行，各自又获得focus。这里没有当前路由最终目标/跨面事件顺序。新增FocusTransfer只记录在InputState，真实宿主分支没有消费它来建立这一时序；单个最后Option也不能自行重建整帧事件的交付次序。HashMap首项不是当前编辑者的合同依据。

闭合oracle：真实宿主的合批输入必须维持唯一真实编辑目标；A→B及B→A的同帧合法序列，最终router与唯一widget一致，blur后均无焦点/preedit残留；前面的合法操作不得静默被丢弃。保留分帧、shared mount、非UI目标、键盘/IME焦点目的地与静态输出正控。修复真实桥与runtime的状态传递，不能用所有测试强制每点击单独frame来回避当前pending入口。

### F-830-R3-02 / P2 / 代码 / focus修复兄弟拒绝出口：无效Surface拒绝已破坏旧合法焦点

AC04/05、T05/T06，§5.3“stale/session引用明确拒绝且不污染当前状态”；也属于r4明确补齐的公开focus兄弟出口。`src/host/surface_runtime.rs:997-1022`先遍历释放其他全部iced focus，然后才查请求id的slot；id不存在时返回false，但旧目标已丢失编辑状态。

实际入口/反例：`combinations.rs::rejected_foreign_focus_keeps_current_edit_state`。先公开focus合法A并确认(true,false)，再用另一CompositionGraph真实mint的跨session SurfaceId调用focus_iced_text_input。返回false（确实拒绝），但A由focused=true变为false，结果(false,false)。无伪造id字段，也没有无控件/失效setup；此前合法focus断言已过。

闭合oracle：先确认本次Surface目标存在、归属/代际有效且可执行focus，再改变旧编辑目标；拒绝跨session/stale/不存在Surface不清理当前合法焦点/IME状态，后续合法focus仍迁移唯一目标。仅针对既有Surface身份拒绝合同，不新增某个不存在widget Id的API语义。该分支在上一修复版本也可存在，属于未覆盖兄弟出口，不宣称它是5aec新引入的代码回归。

### F-830-R3-03 / P2 / 验收证据遗漏 / 原R1-F09子项：V09组合上传量与内存事实仍未产出

AC06/08、T08；主计划§6 V09（当前主文第203行）明确要求“报告CPU/GPU可用计时、dirty/绘制/**上传**/readback/目标重建量与**内存**”；V07同时要求把OS缓存/驱动驻留与所属资源关系分开，不能只看RSS。

本轮实际`surface-lifecycle`15/15的`body.metrics.composition`仅12个字段：cpu_frame_ms_max/p50/p95、dirty_skips_delta、frames、gpu_timing_available、portal_composites_delta、readbacks_delta、surfaces_rendered_delta、target_creations_delta、target_drops_delta、warm_target_build_seqs。`src/host/cases/surface_lifecycle.rs:395-418`构造相同JSON；没有上传计数/字节或内存观测/可用性说明。surface-result §4也仅列这些现有字段。真实目标计数已经正确，但数量守恒不是上传量或内存的替代。

这是R1报告F09已指出的“V09上传/内存计量缺口”，不是第三轮新增产品性能承诺。R2审核没有把此子项单列清楚，本轮修正其路由；不把这次审核表达遗漏归咎为新的实施回归，也不能据此删除原合同。829既有metrics case有上传instrumentation/RSS观测，但它不是本次新增Surface组合路径的证据。

闭合oracle：对本次组合/生命周期路径给出来自实际调用的上传计数/字节及有单位、采样点和边界说明的内存事实，或对确实不可测项具名记录不可用及原因；把进程/驱动驻留与owned目标/资源事实分开。保持当前正确的创建/drop/raw rounds、readback与GPU timing=false。没有要求竞品对比、跨平台或新性能阈值，也不以假常量/理论值冒充实测。

## 本轮诊断更新 / 下一次ready

第三次final needs_fix。work r4的六行诊断/闭合表形式与多数真实修复有效，但焦点检查仍只覆盖“一次frame一个点击目标”；新的错误出口也没有“拒绝前后不变”断言。下次ready需要补的范围已收敛：

| 未闭不变量 | 本轮为何漏检 | 下一轮实际入口/闭合证据 |
|---|---|---|
| 实际编辑目标唯一 | 从分帧样例外推；runtime用HashMap某个press作唯一目标，忽略合批时序 | 宿主pending合批A→B/B→A、真实双widget/IME与blur后状态 |
| 非法引用拒绝无副作用 | 只测合法公开focus迁移；先释放旧目标再查新目标 | foreign/stale/nonexistent Surface拒绝前后编辑状态保持+合法迁移对照 |
| V09所有要求有观测 | 目标数量计数闭合被当成完整计量；审核R2未单列旧子项 | 本次组合的实际上传、内存、不可用边界与原始报告 |

不要求新计划或改变已批准profile；同树work完成上述三项并对照本轮输入提交新冻结，再送独立review。原R2其他五项CLOSED不重开，不重跑与修复无关的重档。

## AC / SD / 状态与合入门

| AC | 结论 |
|---|---|
| AC01/02 | pass，来源/独立消费/重定位/图契约与29测试通过 |
| AC03 | pass于现有实验profile，两方向合成/活体内容传播/alpha/同设备正反控通过 |
| AC04/05 | partial，R3-01/02未闭；原相机/遮挡/caret/单框真实IME已过 |
| AC06 | partial，资源计数/事务/生命周期回放已过，R3-03旧计量证据未闭 |
| AC07 | pass于既有冻结游戏/场景范围，829兼容、共享Squash、路由故障敏感性与重定位过 |
| AC08 | partial，SD01与多数handoff增量真实；SD02“即使宿主忘记也不允许双编辑态”的保证仍被R3-01反例击穿，另需补齐V09事实 |
| AC09 | needs_fix，三项OPEN，编译/格式卫生通过 |
| AC10 | pending merge |

计划回executing **4/10**：T01..T04保留完成，T05/T06与依赖的T07/T08重开，T09/T10仍未完成，历史记录不删。SD01/02保持本轮冻结副本供对照；SD03/04知识归属及AutoShader/CAP、I1–I3 pending不变；未发布canonical/ledger。

merge-owned pending：最终review pass后，在新冻结提交统一AGP locked release/test、六surface（真实OS输入/IME）、829兼容/consumer重定位与正式负控/资源门；再合入autographics-dev、沉淀Spec/ledger/index、归档/guard clean清树。根t/tv/tu/docs_gen不因仅experimental改动触发，tf仍主检出到期单实例。本轮未合入、未归档、未称交付。
