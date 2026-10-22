# PLAN-830 final review R5 — pass

2026-10-22。stage: review；review_scope: final；reviewer_identity: 当前Codex独立审核上下文（与实施独立，未获知独立模型/会话ID）；plan_id: PLAN-830；plan_revision: 3；outcome: pass。未创建其他agent/chat，未修改实现、未合入/归档。

**R4两项CLOSED，无开放阻塞finding。** 本结论是当前冻结代码、验收和规范增量审查通过；正式merge-owned门仍pending，不等于已合入或交付完成。

## 冻结输入

- 主计划最近提交`473acf318668f10b567aa258d029d8db9c22b462`；进场execution_done 8/10，r3。冻结[计划输入](plan-input.md) SHA256 `06cb4c1167315a698a9bd3d07355b8904d050ae9db7093eb2dc73f663993a9bc`。本次簿记不改变语义revision。
- 原实施树`D:/autostack/.wt/lang-830/auto-lang`，分支`plan-830-dev`；`reviewed_commit: 764e87559cff016d9e0d6b2141e5f1f9903398c5`；`base_commit: ab56c32ec4cb28e01fbcca74e9861164ffad43f0`；`repair_diff_base: 5cd03b861832b9de49e2f04637a7fce130b1e7dd`。进/出场clean、HEAD无漂移。主检出其他任务dirty保留。
- 依赖AGP Cargo.lock SHA256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`；stock iced0.14/iced_widget0.14.2/wgpu27。与R4相同锁/manifest/测试配置/游戏资产与golden；没有根生产Rust或依赖变更。G829来源沿用已审核的autographics-dev基面与主计划既有落地收据，不要求先合master。
- canonical Spec输入仍为：project `a706ef0e5f2f6db3a82ec7c8c2cb6abedd2f408644656129221dc0a2012eaeb3`；prototype-profile `59b42126811545156b6d5339aacde156974f43259fab51bf6191dc174d527832`；integration-contract `1553c8f94b6a407946e7380d9e14edc0e137cfd6be65f851eeedb01c81f05669`。
- SD01[冻结副本](SD-01-frozen.md) SHA256 `e68bba42d91e76feea78fe0c57bd6bc01468c09658c268f9c2b428f118a95d5b`；SD02[冻结副本](SD-02-frozen.md) SHA256 `68b3b38fda584254923be56476299f3709c2733eb3e45365afd2074503d6c5c1`。SD03/04见plan-input§5.5。未发布canonical Spec或修改live ledger。

代码行号/冻结源均绑定764e87559；实现路径默认相对`experimental/autographics-prototype/`。所有本轮正式JSON fingerprint均为`764e87559cff`，故障副本报告也显式绑定此输入，但其省略模式由独立trace说明，不能当成原版正控。

## 本轮独立验证

| 检查 | 实际结果与证据 |
|---|---|
| 定向契约/修复回归 | contract29+r2五例+r3六例+r4五例，**45/45**；[日志](targeted-tests.txt)。作者162/162不是本轮独立全套结果 |
| 类型/格式/release构建 | locked check、fmt --check、locked release build均exit0，产品零warning；[check](check.txt)、[fmt](fmt.txt)、[build](build.txt) |
| R4原始focus-exits | 原文件不改一字重链，**6/6**；[输出](focus-exits.txt)。原文件SHA256 `11e838509862efa1049d94fe5a89fb034b0d79fe82f8ec3711ec83c3705b9719`；其中包含R3原三个组合控制，仍全部通过 |
| 新三态组合出口 | [独立源码](focus-states.rs)、[输出](focus-states.txt)，9/9（前6保留原输入，新增3个）：取消→同一批新press、取消→非法focus拒绝并后续空帧、离窗→pending渲染；与真实widget焦点/caret/router状态联合断言 |
| R2原四holdout | 原文件同字节重链4/4，[输出](r2-holdouts.txt)；源hash仍`87e824c835baad4b4032dd4341b7e4c733924bb3cd97da821435c3773a4534d7` |
| 六surface release | [offscreen](surface-offscreen.json)6/6、[compose](surface-compose.json)17/17、[input](surface-input.json)18/18、[scenarios](surface-scenarios.json)15/15、[lifecycle](surface-lifecycle.json)23/23、[IME](surface-ime-os.json)14/14；本轮均首轮exit0 |
| 真实OS输入/IME | 真实窗口点击、CJK commit=`爱彼此你好你好`，caret输入/清除/滚动与活跃挂载投影通过；未用合成事件替代OS证据 |
| 829兼容 | [Breakout](ag2d-breakout.json)、[Squash](ag3d-squash.json)、[iced-shared-device](iced-shared-device.json)均exit0，本轮无点击超时 |
| 独立消费/重定位 | 本轮[实际运行](independent-consumer.json)pass，A→B29字段0差异、根生产0 collateral；两份SD草稿单列 |
| 场景路由oracle | 当前源与R3冻结副本同SHA256 `fc07862c6c161d98ea5d8683416c5380d9191e4d9fa59725636503a35f82180b`；原wrapper对新库重链，[合法](scenario-control.json)15/15 exit0，[真实9999故障](scenario-corrupt-local.json)8门红/exit1，[trace](scenario-corrupt-local.txt) |
| 新上传oracle外部故障 | 从当前提交重新冻结真实runtime/composition/lifecycle源并重建原两种省略；[合法](upload-control.json)23/23 exit0，[漏plane](upload-without-plane.json)与[漏content](upload-without-content.json)均正式gate红、**exit1**，详见下文 |

构建/运行均在原实施树；审核探针只放主检出附件，产物放scratch。未执行根t/tv/tt/tb/tu/taa/tf/docs_gen，也未用AGP整套测试提前代替merge门。[replay.ps1](replay.ps1)保存焦点/计量定向复现步骤，脚本语法已核；表中结果来自本轮实际分步命令。

## R4闭合判定

### F-830-R4-01 / P1 — CLOSED

AC04/05、T05/T06。r6诊断及覆盖调整已核：原来把None解释为允许所有面focus，现用`RoutedFocus::{Unset,Target,Cancelled}`区分尚未路由与已经取消；`revalidate_focus`进入Cancelled，真实宿主blur/离窗返回待释放面，`release_iced_focus`把匹配的已路由目标设为Cancelled；frame对所有iced面设focus_allowed=false，在事件交付后、绘制前清真实焦点/组合态。新合法press/公开widget focus恢复Target。没有靠丢弃先前事件修复。

- 原A→B pending→真实宿主blur→frame：router=None，widget由原(true,true)变为**(false,false)**。
- 原A→B pending→移除B挂载→显式revalidate/unfocus/Closed→frame：同样**(false,false)**；与旧的“未revalidate直接release”边界无关。
- 保留合批→frame→blur、A→B/B→A正常切换、先前按钮动作仍生效、非法引用拒绝无副作用控制（45定向测试及原holdout）。
- 新测试确认取消→同批新合法press可建立A唯一目标；非法foreign focus不能解除Cancelled，后续3空帧也无widget焦点/caret；pointer_left_window同源取消有效。永久用例另核取消后新点击/公开widget focus恢复。
- 六surface和OS中文IME本轮通过，焦点、真实caret及挂载锚点没有发生回归。公开widget focus仍按既有宿主协议与路由建立配合使用；没有把实验辅助API扩成生产自动路由/多窗口支持。

本轮未发现此取消/恢复矩阵中的开放缺口，不把已测代表性控制外推为任意输入规模或并发保证。

### F-830-R4-02 / P2 — CLOSED

AC06/08、T08。`plane_uploads()/portal_quad_uploads()/content_uploads()`读取真实独立counter；正式predicate对plane/quad分别精确比较真实composite数与128/160B写入，total严格等于三项之和；新空canvas活控恰1次/64B。V08内置负控保留真实write_buffer，只关闭记账/汇总，并用同一predicate检测和恢复。

本轮release实测：quad66/10560B、content18/1056B、total84/11616B；平面专用块1/128B，不能再用同时发生的quad160B顶替。owned4目标=1511424B；进程工作集236.4→236.6MB；GPU/驱动驻留具名不可用。owned字节是按真实尺寸计算的纹理payload，排除驱动对齐/开销，不冒充实测驻留；iced内部上传排除如实保留。

**独立外部故障重新冻结/实际执行**：本轮[lifecycle-source.rs](lifecycle-source.rs)与764e87559原断言同字节，SHA256 `665f2b816045e12e64b7d3b3e8d4c2ed0bee2956394556f0bec93b1a4c21f68a`；[runtime-source.rs](runtime-source.rs)为`379a0acf955f4c79ed9520707e22060285f21634022df87405ed722ac097b287`；[composition-source.rs](composition-source.rs)为`0611eaf3dab6fd91a26b0568bd02a3a535db23e91cfa0e26dfca2e59f437ff7d`。没有复用R4冻结旧oracle。

[upload-mutation.rs](upload-mutation.rs)只把当前源码中的真实平面record或总量content merge置于环境省略分支；真实GPU写入和正式gate/JSON代码不变。独立分支不受产品恢复hook控制，因此也检验持续遗漏会向整体非零传播。

| 变异 | 正式gate实际结果 | 整体 |
|---|---|---|
| 无省略 | 23/23；内置plane/content遗漏能被捕获且恢复为绿 | exit0 |
| 漏plane record | `plane_upload_counter_matches_composites`红：composites=1、recorded=0/0；恢复hook无法修复外部持续遗漏，相关恢复门也红；[trace](upload-without-plane.txt)证明真实写后省略 | exit1 |
| 漏total中的content merge | `upload_total_is_the_sum_of_its_sources`红：66/10560 vs84/11616；plane块total2/288 vs分项4/512也红；[trace](upload-without-content.txt) | exit1 |

计量事实与oracle均闭合。此判定不要求未来所有renderer内部上传均可测，也不把未测GPU驻留记作已支持。

## AC与规范增量

| AC | 结论及依据 |
|---|---|
| AC01 | pass：G829来源/独立workspace与锁保持，consumer本轮复核根生产0改动和重定位29字段一致 |
| AC02 | pass：29契约与原holdout，代际/批次/循环/depth/dirty/资源失败边界保持；新增Cell状态不改变身份分配 |
| AC03 | pass于批准的Windows实验profile：offscreen/compose/scenarios、两方向嵌套与alpha、同设备和正式负控通过 |
| AC04 | pass：路径/相机/遮挡/捕获/共享与独立实例的既有契约和正式回放通过，R4取消组合已闭 |
| AC05 | pass：真实widget唯一焦点、合批/取消/恢复、键盘/IME、真实caret与OS中文流程通过 |
| AC06 | pass：局部dirty/共享tick/事务/资源20轮真实计数保持，上传/内存事实与计量故障敏感性已闭 |
| AC07 | pass：原游戏/场景未改golden，829三案和独立消费/重定位本轮均过 |
| AC08 | pass：SD01/02、surface-result与handoff的本轮增量按真实producer/consumer/取消状态与上传源核对；shader布局/错误、不可用项与生产pending边界保持 |
| AC09 | pass：独立冻结复核、反例/合法控制/新路径与规范增量/卫生均通过，无开放阻塞finding |
| AC10 | **pending merge**：不因上述pass勾为完成 |

R1/R2已闭且本次未改变的图/相机/遮挡/预算等详细原因仍引用[此前R3](../review-r3/review.md)、[R4](../review-r4/review.md)的逐项审核；本轮复跑当前29契约、原holdout、六case，且确认相关依赖/配置/资产未变。这不是复用旧输出冒记当前执行。未revalidate直接release仍是既有调用约定；本轮正确revalidate反例已通过。

规范增量已核：

- SD01 add `docs/specs/auto-graphics/surface-composition.md`：内容/挂载/DAG/资源/计量合同符合实验实现（AC02/03/06）。
- SD02 add `docs/specs/auto-graphics/nested-input.md`：路径、唯一编辑目标、三态取消与恢复、真实caret/IME合同符合本轮证据（AC04/05）。
- SD03 modify `docs/specs/auto-graphics/prototype-profile.md`：以当前829 canonical版本扩充本Plan批准的Windows单窗/平面组合与测量事实；不外推到任意平台/深度/生产UI（AC01/06/07）。
- SD04 modify `docs/specs/auto-graphics/integration-contract.md`：回写IH01/IH04实验增量及当前Shader资源/布局/错误接缝，IH02/03/05、I1–I3、AutoShader/CAP保持pending（AC08）。

两个add/两个modify、supersedes/new路径和GOAL007/008元数据保持，无semantic重订。合入阶段应按上述冻结输入沉淀；若canonical目标并发变化需重新对账。handoff中PortalQuad的“前24B”描述的是有效字段区域；本轮以实际write_buffer整段**160B**为上传计量事实（WGSL结构32B、分配160B），沉淀时应区分有效参数与传输长度，不能按24B改计数。此用语不改变当前已正确计量的代码和SD合同。

已批准边界仍为：Null剪贴板、每帧iced/3D重绘、组合管线创建尚无wgpu error scope、GPU timing=false、S03取景不与829 Squash逐像素对拍（829自身golden另守护）、候选窗模板逐像素对比及AutoShader/CAP/I1–I3原层级pending。没有新授权的缩面或隐瞒work-owned未完成项。

## 状态与下一步

计划进入**reviewed 9/10**：T01..08保持完成，T09本轮完成，T10仍未完成。R4-01/02 CLOSED，R3-02/03及其他已闭成果保持；历史needs_fix记录和OS注入失败记录不删。本轮实际全部正控首轮通过，不据此担保后续机器/前台环境永不超时；后续失败必须保留原始报告并归因。

**merge-owned pending**：最终review之后在本冻结提交统一AGP locked全套test/release、六surface真实OS输入/IME、829兼容/独立consumer重定位、正式负控/资源门；全部通过后才可合入autographics-dev（不合master实现）、Spec/ledger/index沉淀、归档与guard clean清树。代码、依赖、输入/配置发生变化则复核受影响内容，再在新冻结执行受影响门；不得用本pass覆盖后续实现变化。

根t/tv/tu/docs_gen不因本Plan仅experimental改动自动触发，tf仍为主检出到期单实例批量门。当前未合入、未归档/清树、未宣称交付完成。
