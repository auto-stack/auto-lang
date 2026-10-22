# PLAN-830 final review R2 — needs_fix

2026-10-22。仍为独立于实施的同一审核上下文，延续 R1；本轮未创建 agent、未修实现、未合入/归档。修复确实关闭了大量原样例，但剩余不变量和新修复分支不满足最终验收。

## 基线与指纹

- 主计划 r3，主检出 `c2ee32b297978bb64580f3bc447408f1e5a5481e`，进场 execution_done 8/10，输入 SHA256 `842a44f7de6a0f3a8b150dca285d074228c2028ee77b00d6cd7984135a9cdefb`；[输入副本](plan-input.md)。保留 revision=3，本轮仅修改审核/计划簿记与风险记录。
- 原实施树 `D:/autostack/.wt/lang-830/auto-lang`，分支 `plan-830-dev`，冻结 `dbaea29e122d2fa2a18d18ce3f09444583098efd`；base `ab56c32ec4cb28e01fbcca74e9861164ffad43f0`；修复差异 base 为 R1 `dd2788bcc16ce7cff247895346afbbf343352496`。进场/出场 HEAD 未漂移、实施源码 clean。主检出其他任务 dirty 不动。
- 依赖锁与 R1 相同：AGP Cargo.lock SHA256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`。未以 R1 旧二进制测试新代码；本轮 locked build 生成当前产物，正式 JSON 均 fingerprint=`dbaea29e122d`。
- canonical Spec 来源与 R1 相同（实际重核 hash）：project `a706ef0e5f2f6db3a82ec7c8c2cb6abedd2f408644656129221dc0a2012eaeb3`；prototype-profile `59b42126811545156b6d5339aacde156974f43259fab51bf6191dc174d527832`；integration-contract `1553c8f94b6a407946e7380d9e14edc0e137cfd6be65f851eeedb01c81f05669`。
- 新受审 SD-01 [冻结副本](SD-01-frozen.md) SHA256 `84947d0c377074153f2dfecc2083841dc4fc17f3d40588e8f1c75a1095970015`；SD-02 [冻结副本](SD-02-frozen.md) SHA256 `ed790ff47e2b18b5f821e607ac3d7e9fd4f7a998afbb22ff64642ebc951849c7`。SD03/04仍为 plan-input §5.5 的 modify 提案。frontmatter 两个modify/两个add与GOAL007/008归属保持；未改canonical或ledger。
- 场景gate反控使用同字节副本：scenario-source SHA256 `23a8764e2b8692de8a07c89ec63ba339eb0f26aee48d4d922a621a9fe0f10801`；squash-source `e537d0369d9db9c7c7f21c58e17771116cd92ec98b8bfe9bf5a42fd55aff0eff`，与原树源码逐一重核相同。
- 采用 auto-plan-review 与 verification-contract 的同一来源（R1已记录技能全文/hash）；再次读取，遵守复审只跑必要定向验收、最终通过后才做merge-owned门的时序。本轮没有根t/tv/tt/tb/tu/taa/tf/docs_gen。

下文文件默认以 `experimental/autographics-prototype/` 为前缀，行号绑定 dbaea29e1。持久输入/输出均在本目录；实现可用 `git show dbaea29e1:<path>` 恢复。

## 实际复跑结果

| 范围 | 本轮观察 |
|---|---|
| locked check / fmt / release build | 全部exit0、无产品warning；[check](check.txt)、[fmt](fmt.txt)、[build](build.txt) |
| 定向 surface_contract | **29/29**，[原始输出](surface-contract.txt)；未宣称本轮重跑作者146/146总数 |
| 原R1独立反例 | 同字节源码SHA256 `c22c22f26c9fc17625b454f5a55abc94a3080b682689e21e4fb3633ce284df02`，新rlib重链，**10/11通过**，[原始输出](old-counterexamples.txt)。唯一失败为R1已标注的“未revalidate直接release”协议边界；显式revalidate正控过，不升级为阻塞缺陷 |
| 六surface正控 | offscreen6/6、compose17/17、input17/17、scenarios14/14、lifecycle14/14、ime-os11/11，全部exit0；[offscreen](surface-offscreen.json)、[compose](surface-compose.json)、[input](surface-input.json)、[scenarios](surface-scenarios.json)、[lifecycle](surface-lifecycle.json)、[IME](surface-ime-os.json) |
| 真实窗口/中文IME | OS点击、preedit和CJK commit通过，真实输入没有回退为合成证明；本机本轮具体文字见IME原始JSON |
| 829兼容 | [Breakout](ag2d-breakout.json)、[Squash](ag3d-squash.json)、[iced-shared-device](iced-shared-device.json)全部exit0；未改冻结游戏规则/golden |
| 独立消费与重定位 | [independent-consumer](independent-consumer.json) exit0，closure/locked build/29字段0 mismatch/root collateral通过；这是本轮实际重跑，不沿用文案 |
| 新修复组合holdout | [源码](holdouts.rs)、[输出](holdouts.txt)，**4/4失败**，每项含合法对照；见R2-01/02/04/05 |
| 场景oracle故障敏感性 | [wrapper](scenario_mutation.rs)、同字节冻结[场景源](scenario-source.rs)/[Squash辅助源](squash-source.rs)，合法[control](scenario-control.json)14/14 exit0；将实际hit.local改为[9999,9999]后，[故障JSON](scenario-corrupt-local.json)仍14/14 exit0；[变异trace](scenario-corrupt-local.txt)证明实际命中输出被改变，见R2-06 |

[复跑脚本](replay.ps1)在原树运行，仅生成scratch/审核附件，不改实现。场景wrapper只替换路由结果边界，原scenario代码逐字节复制；初次wrapper缺模块/extern的组装编译错误已修正，不作为产品finding。今后重放修复后的scenario oracle应从新提交刷新副本并记录hash，不能拿R2旧oracle证明新oracle通过。

## R1 闭合对账

| 原finding | 原样例 | 完整不变量/当前状态 |
|---|---|---|
| F01 结构dirty | 已关闭 | Add/Remove/SetMount/View/Size结构失效路径通过；当前graph修复保留，不因新runtime内容传播问题重开整个T03 |
| F02 跨帧alpha | 已关闭 | 静态帧稳定；修复引入clean父面跳过后，实际UI更新不能传播，**partial / R2-01** |
| F03 GPU资源事务 | 已关闭 | 超限add/resize具名拒绝、last-good和合法恢复通过；组合静态管线error scope未装已如实披露，不把非用户shader入口扩成新验收任务 |
| F04 host相机 | 已关闭 | 原双父面24次0 mismatch、未注册miss和29契约断言通过 |
| F05 递归遮挡 | 已关闭 | 原例及递归Occluded传播通过 |
| F06 取消/焦点 | 旧window cancel已关闭 | 真实两控件切换仍各自focused，blur只清最后一个，**partial / R2-02** |
| F07 活跃挂载锚点 | 旧双挂载/root已关闭 | 实际caret仍未接入，固定点/整平面门冒作caret区域，**partial / R2-03** |
| F08 固定场景 | 缺失场景/游戏类型已补 | S01/UI→3D→UI、S02/reset、S03共享Squash有新增事实；事件仍向预设坐标发，负控仍假绿，**partial / R2-06** |
| F09 负控/生命周期 | 7项故障比R1有实质改善；重定位已关闭 | same-device遗漏Scene3d、drop遗漏resize，**partial / R2-04/05**；真实gate输入消费又受R2-06影响 |
| F10 规范/交接 | 测试数、OS证据、Null剪贴板等错误已修 | 单设备全入口/真实drop/caret等陈述仍受代码缺口影响，**partial**，随修复复核SD，不另计重复finding |
| F11 warnings/fmt | 已关闭 | 本轮check/fmt过、无新产品警告 |

## Findings（6项，均既有合同内，OPEN）

### F-830-R2-01 / P1 / 代码 / F02修复回归：Iced子内容更新，clean Canvas消费者仍显示旧纹理

AC-03/06、T04/T08、§5.1/5.2，既有“活体内容被消费者采样/局部dirty级联”要求。`src/host/surface_runtime.rs:486-533` 在事件处理前冻结dirty，只按graph dirty和Iced/Scene3d强制绘制决定rendered；portal只向rendered父面合成。Iced真实message更新后的child没有失效其Canvas消费者。

独立 `iced_event_updates_every_canvas_consumer`：通过实际CompositionInput命中、真实iced按钮press/release，把子UI底色红→蓝；child=[0,0,255,255]，同帧parent仍[255,0,0,255]，后续也保留；显式mark_content_dirty(child)合法对照才让parent变蓝。不是外部偷偷改对象却遗漏手工dirty，而是frame本身消费事件、应用消息后留下不一致输出。

闭合：真实事件/内容adapter产生的输出变更也刷新全部消费者链，二维缓存中间层与共享挂载都看到同帧新内容；静态alpha跨帧稳定、无关兄弟保留的正控继续通过。不用要求调用者在frame更新之后额外mark来掩盖内部缺口。

### F-830-R2-02 / P1 / 代码 / F06不变量未闭：路由焦点切换后，旧Iced文本框仍保留focus

AC-04/05、T05/T06，§5.3唯一焦点与旧编辑态清理。`src/core/composition/input.rs:262-285` press直接覆盖focus，不报告/清理旧面；真实宿主mouse press分支也未释放旧Iced focus。新增window_focus_lost只返回最后一个面。

独立 `changing_surface_focus_unfocuses_the_old_iced_control` 使用两个真实text_input和当前宿主相同的routed press/release流程：先A后B，router= B，而iced_a=true且iced_b=true；随后window blur返回B，release B后**A仍focused**。首个A确实focused及B确实接收点击均有断言，未以无控件/错点击作为反例。

闭合：焦点迁移和cancel必须贯通router身份与真实控件缓存，旧控件unfocus/组合态清理；真实两框的preedit中切换、blur后两者均无悬空编辑态。补实际控件状态和消息内容断言，不能仅看keyboard_target唯一。保留R1显式revalidate调用协议，不重开已排除的无revalidate release边界。

### F-830-R2-03 / P2 / 输入/门禁 / F07部分未实现：实际caret未接入，所谓caret-region仍是整个平面

AC-05、T05/T06/T08，§5.3候选窗锚点由活跃挂载路径下的文本光标决定；不要求逐像素模板截图。`src/host/cases/surface_ime_os.rs:224,771` 两处caret_local均硬编码[40,26]，窗口App缓存anchor后在输入、选择、光标移动期间始终复用；`:887`名为anchor_inside_projected_caret_region的gate使用`:994`明确为FULL UV square的projected_plane_rect。

`src/host/iced_surface.rs:216` 丢弃UserInterface::update的返回State。当前锁定iced_runtime公开State::Updated已经携带input_method；text_input生成的InputMethod::Enabled.cursor来自布局、真实caret index/selection/scroll。现有adapter未向host输出/消费这些事实。新active_path的确修了共享挂载定位，但没有把实际光标接上；本轮真实CJK11/11不能证明此不变量。

闭合：获取真实控件caret/输入法区域并沿当前挂载投影；挂载保持不动、输入/光标/选择/滚动改变时OS锚点跟随；正式门以真实caret区域而非整个平面或窗口为oracle。有界精度误差可记录，不新增候选窗模板比对要求。不以把“光标”改写为任意平面内固定点关闭。

### F-830-R2-04 / P1 / 代码/门禁 / F09同源出口遗漏：外设备Scene3d绕过设备归属校验

AC-03/06、T04/T08，§5.2统一Device/Queue和SD01当前承诺。`src/host/surface_runtime.rs:123-129` 的device_ordinal对Iced/Game2d有值，Scene3d一律None；`:284`只检查Some。新正式负控只构造外设备Game2d。

独立 `foreign_scene_content_is_refused_like_foreign_canvas`：同一个真实second GpuDevice构造Game2d被拒绝（合法反控先验），但用该second创MeshPipeline的Scene3d被**Ok**接纳，graph_preserved=false。这个对象确含另设备GPU管线，不是篡改报告device token。构造只使用现有公开adapter字段，不新增支持profile。

闭合：对所有当前内容adapter及实际管线/资源记录并验证同设备归属；外设备3D与2D/UI均具名拒绝、状态不变，同设备全类型仍接受；负控覆盖同源变体，不能在None分支静默放行而Spec声称统一验证。

### F-830-R2-05 / P2 / 计量/门禁 / F09真实drop计数未闭：resize替换旧纹理不计入drops

AC-06、T08、V07/V09准确计量。`src/host/surface_runtime.rs:454` 的slot.target = new_target实际释放旧target，但仅target_creations+=1；target_drops只在`:397`显式drop_surface递增。没有目标drop witness补上这一出口。

独立 `resize_drop_accounting_balances_live_targets`：create20×20 → resize30×20 → drop该面，live=0而created=2、dropped=1。两次创建确来自不同build_seq/目标，未变尺寸no-op不是该例。**这证明计数漏记，不宣称GPU内存泄漏**；当前生命周期report把这些字段写成真实所有权计数，不足以闭合warm资源回收关系。

闭合：明确计数单位/采样时机，跟实际target生命周期计入resize替换、失败准备资源与close/最后owner等适用出口；没有外部pin时live与创建/释放关系可核，同尺寸no-op不误增。若特定字段只统计slot删除必须如实名名，另提供合同所需的实际目标释放证据，不能简单改名绕过验收。

### F-830-R2-06 / P1 / 门禁 / F08路由oracle未闭：实际命中坐标完全错误，场景仍14/14通过

AC-04/05/07/09、T07/T08，V03/04/06/08“窗口→挂载→控件”的同一路径与反控要求。`src/host/cases/surface_scenarios.rs:513-523` 只看命中Surface，随即feed_iced_click(rt,预设surface,预设local)，未使用路由输出local；`:881-885`同样直接指定ui1。S02`:575`的text_after_game_focus是在没有向输入桥发送该键的情况下读取原值，游戏Paddle动作另行硬编码入队。因此实际键路由错误也不会由这条“UI未收到”断言区分。

受控实测：当前scenario与Squash辅助源同字节副本，以wrapper调用**原实际router**；正常返回值为合法对照，全部14/14；fault仅在实际Delivered结果处把local替成[9999,9999]，trace逐次证明mutation发生，**相同原scenario仍14/14 exit0**。既没有改报告布尔值，也没有改变源case断言。调用命令/源码/原始JSON均在本目录。这是门禁缺陷，未据此声称当前正常射线算法仍有相同错UV。

闭合：正式用例实际消费路由返回的surface/local、输入桥给出的键/IME目的地；从同一入口把事件送到真实控件/游戏。合法control过；错误UV/坐标和错误焦点目的地变异令相应正式断言失败并整体非零。允许合成输入补逻辑，但不能命中校验与实际控件事件走两条互不约束的路。保留当前已补S01/S02Reset/S03Squash成果，不重写原玩法或golden。

## 第二次未通过后的诊断与下一轮要求

这是同一Plan的第二次final needs_fix，不重置轮次。下表是本轮根据代码/反例确认的遗漏原因；work须在下次execution_done前补自己的修复/覆盖闭合表，并验证这些调整已经进入真实入口。

| 未闭不变量 | 上轮修复/本轮旧例为何漏检 | 需要补的覆盖/设计边界 |
|---|---|---|
| 活体输出失效消费者 | 只用静态alpha或显式mark_dirty；强制绘制的UI输出不在dirty集合 | 真按钮message→子像素→缓存中间层→最终消费者；保留静态/无关兄弟对照 |
| 控件焦点唯一 | 只证明router id或单框window cancel，没查旧面缓存 | 两个真实控件切换与preedit/blur后双侧状态；焦点迁移通道明确owner |
| 光标锚点 | 把固定局部点和整平面命名为caret，没消费实际widget输出 | 同挂载下真实caret/selection/scroll变化的可观察OS锚点 |
| 同设备全入口 | 只对Game2d注入，Scene3d None没测 | 全现有adapter变体的真实GPU资源归属与合法同设备对照 |
| 目标释放计量 | 只测显式slot删除，resize隐式drop漏记 | 创建/替换/失败/最后owner的计数守恒及warm原始数据 |
| 场景真实输入链 | 命中bool与预定控件注入分离，坐标返回值没消费 | 相同入口消费实际返回值；本轮9999故障对正式gate必须红 |

## 验收与规范路由

| AC | 当前结论 |
|---|---|
| AC01 | pass：来源/独立锁/零根生产写面/本轮重定位通过 |
| AC02 | pass：graph原子提交、代际/cycle/depth、结构dirty与旧例/29契约通过；不据此授runtime/输入额外pass |
| AC03 | partial：两方向构造、真实采样与静态alpha通过；R2-01/04仍OPEN |
| AC04 | partial：相机/遮挡旧例闭合；真实焦点R2-02与路由oracleR2-06未闭 |
| AC05 | partial：真实单框CJK和活跃挂载/root数学通过；R2-02/03未闭 |
| AC06 | partial：结构/尺寸事务恢复通过；活体失效/同设备/drop计量R2-01/04/05未闭 |
| AC07 | partial：旧游戏/重定位、共享Squash成果通过；输入oracleR2-06未闭 |
| AC08 | partial：handoff多数事实已改正；全设备/caret/drop陈述仍受未闭代码影响，SD01/02不授最终pass |
| AC09 | needs_fix：6项OPEN；卫生已通过 |
| AC10 | pending merge，不执行 |

计划回executing **3/10**：保留T01/T02/T03完成，重开T04..T08；T09/T10仍待完成，所有旧记录保留。SD03/04的方向与AutoShader/CAP、I1–I3 pending归属保持；等修复代码后复核SD01/02及两个modify的实际规则，不在review发布知识ledger。

`next: 同一实施树work按上述诊断一次修齐，提交并冻结新的实现/测试/SD，携原反例与合法对照及每条不变量闭合表重新送审`。本轮不代修、不新增计划、不合入。

merge-owned仍pending：最终review pass之后，在新冻结提交上统一AGP locked test/release、完整surface/真实OS输入IME、829兼容/独立consumer重定位、真实负控/资源门；再合入autographics-dev、Spec/ledger/index沉淀、归档/guard clean清树。原请求只审，未执行这些合入步骤，未宣称交付。根t/tv/tu/docs_gen不因仅experimental变化触发，tf仍主检出到期单实例。
