# PLAN-830 final review R4 — needs_fix

2026-10-22。stage: review；review_scope: final；reviewer_identity: 当前 Codex 独立审核上下文（与实施独立，未获知独立模型/会话 ID）；plan_id: PLAN-830；plan_revision: 3；outcome: needs_fix。未创建其他 agent/chat、未修改实现、未合入或归档。

本轮确认 R3 原样例全部关闭，公开 focus 的非法 Surface 拒绝无副作用、V09 计量字段遗漏也已修复。剩余两项是合批取消出口和新计量 oracle；不能把原例通过外推为这两个出口通过。

## 冻结输入与范围

- 主计划最近提交 `fd04d0be7f72ccb4366e0ad2b661acbc2ef003ca`，进场 execution_done 8/10、r3；[冻结计划](plan-input.md) SHA256 `fbd9ee45675cdcb0837ab5fbdd92df71f464765fbe2f5337aa127f6431c18e41`。当前合同没有改动，本次进度/审核簿记不改变 revision。
- 实施树 `D:/autostack/.wt/lang-830/auto-lang`，分支 `plan-830-dev`，`reviewed_commit: 5cd03b861832b9de49e2f04637a7fce130b1e7dd`；`base_commit: ab56c32ec4cb28e01fbcca74e9861164ffad43f0`；`repair_diff_base: 5aec0a63429d0a4cd3253751b195e8f74ddc2976`。原树进/出场 clean、HEAD 无漂移；主检出其他任务 dirty 保持。
- 依赖 AGP Cargo.lock SHA256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`，stock iced 0.14/iced_widget 0.14.2/wgpu 27；本轮 diff 不改锁、manifest、测试配置或冻结游戏/golden。G829 来源仍为已授权 autographics-dev 基面及主计划既有来源收据，不要求代码先进入 master。
- canonical Spec 与 R3 同输入：project `a706ef0e5f2f6db3a82ec7c8c2cb6abedd2f408644656129221dc0a2012eaeb3`；prototype-profile `59b42126811545156b6d5339aacde156974f43259fab51bf6191dc174d527832`；integration-contract `1553c8f94b6a407946e7380d9e14edc0e137cfd6be65f851eeedb01c81f05669`。
- SD01 [冻结副本](SD-01-frozen.md) SHA256 `55712abcb3e40c3372aef1be3fabfdbaeecfa2edfd6c4de0d9b6fdd23af38eb7`；SD02 [冻结副本](SD-02-frozen.md) SHA256 `4786a5b5682cc452ae447239cbc5512ff5ca9e7d6c27432c91d53647894906b1`。SD03/04仍是 plan-input §5.5 对当前829 Spec的 modify 合同；两个add/两个modify的路径、GOAL007/008元数据保持。未发布canonical/ledger。
- 原R2 holdout SHA256 `87e824c835baad4b4032dd4341b7e4c733924bb3cd97da821435c3773a4534d7`；原R3 combinations SHA256 `4d142c4740885546f7276c04f9b9238727cd694065a8dcb3032c756b11cd7890`。均从原文件直接重链，不改断言。

文件行号绑定5cd03b861；下述实现路径默认相对 `experimental/autographics-prototype/`。正式JSON的fingerprint均为 `5cd03b861832`。审核副本/源码/日志可在清树后继续引用。

## 本轮独立执行与证据

| 检查 | 本轮实际结果 |
|---|---|
| locked check / fmt / release build | 全通过、产品零warning；[check](check.txt)、[fmt](fmt.txt)、[build](build.txt) |
| 定向测试 | contract29 + r2五例 + r3六例，合计40通过；[前34](r3-replay.txt)、[新增六例](r3-tests.txt)。作者157/157未冒记为本轮独立全套 |
| R3原组合输入 | 原文件3/3通过：合批 widgets=(false,true)、blur=(false,false)、foreign拒绝保持(true,false)，见[r3-replay](r3-replay.txt) |
| R2原四项holdout | 4/4通过，[输出](r2-holdouts.txt) |
| 六surface release | [offscreen](surface-offscreen.json)6/6、[compose](surface-compose.json)17/17、[input](surface-input.json)18/18、[IME](surface-ime-os.json)14/14、[scenarios](surface-scenarios.json)15/15、[lifecycle](surface-lifecycle.json)19/19，全部exit0 |
| 实际OS输入 | 窗口点击、真实CJK commit=`abc你好你好`、真实caret输入/滚动/投影均过；普通流程通过不覆盖同帧cancel反例 |
| 原场景oracle故障 | 当前场景源SHA256仍为`fc07862c6c161d98ea5d8683416c5380d9191e4d9fa59725636503a35f82180b`，与R3冻结副本相同；原wrapper本轮实际执行，[合法](scenario-control.json)15/15 exit0，[9999](scenario-corrupt-local.json)8门红/exit1 |
| 829兼容 | [Breakout](ag2d-breakout.json)、[Squash](ag3d-squash.json)exit0；Iced首轮[OS点击超时](iced-shared-device.json)exit1（901 ticks、presses0），同提交串行[重跑](iced-shared-device-retry.json)exit0。两份均保留，不把首轮红删除或擅称产品回归；merge仍须正式重放 |
| 独立消费/重定位 | 本轮[实际执行](independent-consumer.json)pass，A→B29字段0差异、根生产0 collateral |
| 新取消出口 | [源码](focus-exits.rs)、[输出](focus-exits.txt)：6例中4合法控制通过、2取消/卸载失败，R4-01 |
| 上传oracle实际故障 | 同字节正式lifecycle断言，[合法](upload-control.json)19/19；真实省略平面计数[仍19/19](upload-without-plane.json)、真实省略内容汇总[仍19/19](upload-without-content.json)，全部exit0，R4-02 |

未跑根t/tv/tt/tb/tu/taa/tf/docs_gen；未跑AGP整套157测试替代merge门。构建和运行均在原实施树，审核探针和冻结源只存主检出附件，编译产物存实施树scratch。复现入口为[replay.ps1](replay.ps1)。

## 原R3闭合对账

| Finding | 本轮结论 |
|---|---|
| R3-01 合批唯一真实编辑目标 | 原A→B、B→A、先前按钮动作不丢、分帧对照及渲染后blur已闭；完整取消状态仍PARTIAL，R4-01 |
| R3-02 非法focus拒绝无副作用 | CLOSED：目标先查存在且为本runtime iced，再清其他焦点；foreign、真实stale与非iced拒绝和后续合法迁移通过。没有新增不存在widget Id的API语义 |
| R3-03 V09上传/内存字段遗漏 | 原遗漏CLOSED：真实write_buffer计数、内容汇总、owned尺寸字节和RSS/不可用边界已有证据；新增plane/content活控存在假绿，作为R4-02单独路由，不宣称当前代码漏计或GPU泄漏 |

R2-01/03/04/05/06已闭成果保持；R1结构dirty/限额事务/相机/遮挡/静态alpha/卫生不重开。未revalidate直接release仍是已排除的调用协议边界；本轮卸载反例明确执行了revalidate，不能与该边界混同。Null剪贴板、组合管线error-scope缺口、GPU timing不可用、AutoShader/CAP与I1–I3仍按原合同pending，不扩大为新任务。

## F-830-R4-01 / P1 / 代码：取消先于pending渲染时重新恢复编辑焦点

**既有规则**：AC04/05、T05/T06、计划§5.3和SD02§3/4要求失焦/离窗/路径销毁取消后清capture、键盘和真实编辑状态。R3-01闭合oracle已要求合批与blur后状态一致；这是其取消出口，不新增输入规模或新交付承诺。

**实际路径**：真实 `surface-input` 宿主在窗口事件中累积pending，到about_to_wait才渲染。`surface_input.rs:1021-1032`的apply_focus_loss调用 `input.window_focus_lost()`、`rt.release_iced_focus(surface)`，再把Window::Unfocused/InputMethod::Closed追加到该面pending；IME宿主 `surface_ime_os.rs:534-545`同构，并未清除其他面之前积累的点击。

**绑定反例**：`focus-exits.rs::cancel_before_flushing_pending_does_not_restore_focus`，沿真实router正常press/release A→B，把实际返回surface/local收进pending，随后逐条执行上述宿主blur，再一次frame。blur后router=None、graph stamp=None均已确认；**期望两个widget均false，实际(true,true)**。不是声称在原生OS现场观测了这一合批次序，而是从当前宿主实际pending/cancel入口重放合法批次。

同源 `revalidated_unmount_before_pending_flush_keeps_no_editor`：同一批A→B后移除B挂载，显式 `input.revalidate(&rt.graph)` 返回B，再 `release_iced_focus(B)` 并关闭IME，然后渲染；router=None，但仍widgets=(true,true)。正确调用协议也失败。先把合批渲染完再blur的对照过；原R3三个样例也过。

**已确认原因**：`surface_runtime.rs:586-593`读取的目标为None；`release_iced_focus`/revalidate已正确清stamp，但 `surface_runtime.rs:776`把None解释为 `focus_allowed=true`。随后原pending仍被真实widget处理、重新聚焦；`iced_surface.rs:294`因允许聚焦跳过unfocus，末尾Unfocused/Closed也没有解除真实widget的focus。不是graph未清，而是“已取消且无目标”被当成允许所有面保留焦点。

**闭合oracle**：正常合批后的终态仍唯一；失焦/离窗或正确revalidate发生在pending渲染前与后，最终均无真实焦点/组合态/caret，无键盘IME fallback；恢复后的新合法点击仍生效。保持先前合法动作不丢的控制，不用强制每点击单独frame回避。修复须贯通真实宿主与runtime的取消语义；不指定未经验证的具体API改法。

## F-830-R4-02 / P2 / 验收oracle：上传活控可被其他管线的计数顶替

**既有规则**：AC06/08、T08、V09要求上传事实真实且有可复核证据；R3-03要求实际计量而不是理论常量。新增门禁明确自称证明平面/内容路径也被计量，必须能识别对应计数缺失。这是新oracle的出口缺陷，当前生产计数实现本身是正确的。

**正常事实已闭**：release实测组合66次/10560B、全路径84次/11616B；owned4目标1511424B、进程工作集238.2→238.4MB，GPU内存available=false且driver residency单列。采样点/单位/排除项明确；owned字节是Σ真实w*h*4的逻辑payload估算，不是驱动实际驻留。Iced内部上传排除如实记录，不要求本轮实现DXGI或新性能阈值。

**实际故障/原断言不改**：[upload-mutation.rs](upload-mutation.rs)包含冻结runtime、composition和lifecycle源码，以同一AGP依赖构建；[lifecycle-source.rs](lifecycle-source.rs)与提交字节相同，SHA256 `a069dc3c73d4ff78ef7276cb22bd9eba5f7861430e2885a5b9c3e28c6399f0c6`。只在两个真实计数入口增加受控省略分支，真实queue.write_buffer、绘制和正式断言保留。trace明确记录实际省略；没有修改JSON/gate布尔值。

| 对照 | 实际变化 | 原正式oracle结果 |
|---|---|---|
| 合法 | 平面活控2次/288B；全路径84/11616 | 19/19、exit0 |
| `REVIEW_DROP_PLANE_ACCOUNTING=1` | [composition-fault.rs](composition-fault.rs)仅跳过平面真实128B写入之后的record；平面活控变1次/160B，来自同时重绘的2D portal | `plane_upload_counter_is_live`仍true，19/19、exit0；[trace](upload-without-plane.txt) |
| `REVIEW_DROP_CONTENT_ACCOUNTING=1` | [runtime-fault.rs](runtime-fault.rs)仅跳过真实内容counter snapshot汇总；全路径84/11616降到与组合相等的66/10560 | `upload_counters_cover_the_content_path`仍true，19/19、exit0；[trace](upload-without-content.txt) |

**已确认原因**：`surface_lifecycle.rs:573-574`仅要求聚合delta“至少1次/128B”，已有quad的160B足以顶替plane；`surface_lifecycle.rs:514-515`只比较path_total>=composition，内容完全漏汇总时相等仍过。主60帧样例plane_delta=0，另一条精确公式门不能补上此缺口。

**闭合oracle**：在对应路径有真实活动时，让平面遗漏和内容遗漏各自使正式计量门失败/整体非零，合法对照仍过；断言能区分测量对象，可用分管线delta或排除其他管线的有界场景，不扩大成所有未来资源的性能框架。保持当前上传与内存字段及真实写操作。后续修复须从新提交重新冻结源和断言再重建变异，不能拿本目录旧oracle冒充新验证。

探针边界：第一次lab编译因metrics引用agp私有Squash helper而失败，随后纳入同字节Squash辅助源解决；这是审核harness装配问题，不是产品finding。辅助源SHA256 `e537d0369d9db9c7c7f21c58e17771116cd92ec98b8bfe9bf5a42fd55aff0eff`与当前提交相同。另一次向graphics-probe探询--help得到“不支持参数”后按usage运行，不计产品故障。所有最终证据均来自成功编译的正式路径副本。

## AC、规范增量与路由

| AC | 本轮结论 |
|---|---|
| 01/02 | pass：来源/闭包/重定位/身份与批次契约通过；新Cell焦点stamp没有改Surface身份/分配来源 |
| 03 | pass于既有实验profile：双向组合、alpha、同设备、静态/活体与正式负控通过 |
| 04/05 | partial：合批正常切换/拒绝/真实caret/OS IME过，取消后pending复活未闭，R4-01 |
| 06 | partial：资源计数/last-good/20轮与上传/内存事实过，两个计量活控遗漏故障仍假绿，R4-02 |
| 07 | pass于冻结游戏/场景范围；Iced一次OS超时及同commit重跑通过均记录，正式merge重放仍pending |
| 08 | partial：SD01计量事实与多数handoff真实；SD02/输入handoff的无悬空编辑态保证被R4-01反例击穿，正式计量证据需R4-02补齐 |
| 09 | needs_fix：两项OPEN；卫生通过 |
| 10 | pending merge |

回executing **4/10**：T01..04保留完成，T05/T06及其依赖T07/T08重开，T09/T10未完成。保留历史；不改语义revision。第四次final needs_fix，r5已给出真实根因和多数修复，但覆盖仅“合批→frame→取消”和“计数存在”的正控；下次work诊断需补“合批→取消/正确revalidate→frame”和“测量对象缺失→原oracle红”两个既有出口，不重做已闭成果、不换Plan、不借记债缩面延期。

merge-owned仍pending：最终review pass后在新冻结统一AGP locked全套test/release、六surface真实OS输入/IME、829兼容与consumer重定位、正式负控/资源门，再合入autographics-dev、Spec/ledger/index沉淀、归档与guard清树。根整档不因仅experimental改动触发；tf仍主检出到期单实例。未合入、未归档、未宣称交付完成。
