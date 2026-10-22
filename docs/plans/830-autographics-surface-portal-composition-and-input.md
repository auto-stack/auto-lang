---
plan_id: PLAN-830
status: execution_done
feature_name: AutoGraphics可组合Surface/Portal与嵌套输入原型
plan_revision: 3
author: [Codex]
created_at: 2026-10-21
updated_at: 2026-10-22
current_step: 8
total_steps: 10
depends_on: [PLAN-829]
supersedes_spec_components:
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/surface-composition.md
  - docs/specs/auto-graphics/nested-input.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, auto-ui-integration-experimental]
---

# [PLAN-830] AutoGraphics 可组合 Surface/Portal 与嵌套输入原型

## 0. 变更摘要

承接 829 的独立游戏与 Iced 共享设备原型，下一步集中解决 **UI、2D、3D 内容的双向组合与嵌套交互**，继续独立于生产迁移。复用 Breakout/Squash，不再另选一批游戏，不把真实 Auto 源码、生产 AutoUI/RQHost、Forge 或发行全部塞入本计划。

2026-10-22 r3来源交接更新：829已完成R18复审、冻结merge门、按用户裁定合入 `autographics-dev`，归档11/11及Spec沉淀/清理收据齐全，§2 G829现已满足。830从该集成分支冻结版本起步，独立复审后仍合回该分支；不要求先把原型合入master或等待生产迁移。组合/输入与r2的CAP/AutoShader接缝范围保持，不新增语言编译、CAP求解、平台或游戏任务。本轮仅核源和更新交接，实施由用户另开agent启动。

成果是在独立 AGP 上实现真实 Iced 控件、2D canvas、3D 场景的 Surface/Portal 模型、GPU 合成、命中/输入/焦点与生命周期，并证明 `UI→3D→UI` 和 `3D→UI→2D`。它推进 IH-01/IH-04 的实验面；829 的 I1（真实Auto）、I2（生产UI/RQHost）、I3（Forge/发行）继续分别 pending。

## 1. 目标

1. 形成可复用的 Surface（内容实例）、View（观察/输出）、Portal（挂载与映射）及代际身份/依赖合同，替代“仅在顶层排两个视图”的组合方式。
2. UI、2D、3D 都能作为离屏内容被其他类型视图消费；实际 GPU 纹理采样和正确提交顺序，不以图片贴片或每帧 CPU readback 代替活体内容。
3. 嵌套 UI 真正使用 Iced button/text_input 等控件；输入经窗口→布局/裁剪→3D射线/UV→子内容逐级映射，处理焦点、捕获、键盘与 IME。
4. 复用两款游戏，验证多视图/共享内容不加速游戏逻辑、局部更新不全量重建、关闭/重开和失败保留可用状态。
5. 输出后续生产 SDK/AutoUI/RQHost 的可消费实验合同与尚缺事项，不在当前迁移核心上接线。

profile：沿用 829 的 Windows x64、单原生窗口、键鼠和 wgpu/Iced 版本线；新增同窗口多个内容实例与多层 Portal。2D 布局先支持平移/非均匀缩放及矩形裁剪；3D Portal 为可变换的平面矩形，含透视相机与UV映射。曲面/任意网格UI、跨进程GPU共享、多窗口焦点、Web/Linux/移动端及新平台发行不属于本次支持承诺。

非目标：修复/替代829验收、伪造已发布接口、重写Iced布局/文本/控件、第二渲染后端、核心语言语法/VM/ACE改动、全面ECS/物理扩展、场景编辑器/完整持久化、生产RQHost协议或默认入口切换。支持有限profile不等于限制UI/2D/3D的类型组合，后续可复用本模型扩面。

## 2. 架构方案与前置门

### G829：可消费来源门（2026-10-22已满足）

G829消费已获授权的实验集成分支交付：829独立final review通过、冻结适用门通过、实际合入 `autographics-dev`、Spec/来源与归档收据齐全。这是本实验路线的有效来源，不要求代码先进入master，不冒称生产UI/RQHost或迁移已完成。原r1/r2在收据缺失时的D829/E829讨论保留在§9与原预检中，不再作为当前阻塞。

固定来源：`autographics-dev@ab56c32ec4cb28e01fbcca74e9861164ffad43f0`；829复审/门禁冻结代码为 `ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74`。本轮实际验证后者是分支祖先，且两者的 `experimental/autographics-prototype/` 子树无差异。829归档11/11；冻结门收据记117/117测试、10/10探针与consumer重定位通过，本轮不重复执行这些门。来源见[829归档收据](archive/829-autographics-independent-research-and-game-prototypes.md)、[merge门禁](attachments/829/merge-t11/gates.md)、[模块Spec](../specs/auto-graphics/project.md)。T-01例行核对实际来源/dirty/依赖，不重新索取已具备的放行；来源若漂移则重新绑定证据，禁止复制WIP。

### 分层与写面

```text
Rust demo / 两款已有游戏                 未来 Auto SDK/Forge adapter
                  ↓                           ↓
GPU无关：Surface/View/Portal身份、挂载/依赖、命令、输入/焦点
                  ↓                           ↓
Iced内容adapter       2D内容adapter        3D内容adapter
                  └── wgpu离屏/依赖调度/纹理合成 ──┘
                         独立宿主/窗口
未来：迁移后生产AutoUI/RQHost接线；当前不改它们
```

沿用 `experimental/autographics-prototype/`（AGP）与独立锁；只在已发布来源上修改本实验 workspace 和其测试/文档。GPU对象留renderer/host，scene/组合model不依赖Iced/wgpu/AST/Value。公共Iced适配在实验侧，不复制Iced控件、生产UI/VTree/VM桥或把私有生产字段公开来便利原型。

既有 `Scene::submit` 的逐项语义与两款游戏接口继续保持；本次新增组合命令的验证/提交合同独立定义，不借组合工作静默改掉829旧API语义。新的身份类型可基于原有代际分配机制，不能再复制一套互相冲突的session/resource身份源。

## 3. 技术栈

- wgpu 27.0.1、Iced/iced_wgpu 0.14、GLB/文字/音频依赖按G829实际锁定来源复用；829最终使用stock iced_widget 0.14.2，已去除早期vendored patch，不恢复旧候选依赖。T-01核对，不追最新升级。
- 通过公开 Iced runtime `UserInterface` 的布局/事件/绘制边界及 renderer 能力研究真正UI离屏化；原有 custom Primitive 用于上层组合。**反向离屏UI可行性还未实证**，T-02先做whole-path实验再实现完整嵌套。
- 复用829 GPU目标、捕获、资源与测量入口。其普通OffscreenTarget当前只带RENDER_ATTACHMENT/COPY_SRC，不能直接据此声称可被采样；本Plan按新usage合同增补，兼容旧调用。
- 文本输入继续用Iced真实控件/文本机制；IME屏幕位置与转换在host adapter处理，不手写文字编辑器。

## 4. 需求分析与背景调查

### 4.1 授权与观察基线

当前r3授权：用户说明829已完成并合入autographics-dev，询问830能否开工并拟另开agent实施。本轮核实前置与更新计划交接，不代启动实现、不创建agent/chat或工作树。此前r2的CAP/AutoShader文档修订授权与范围保留；原“本次文档修订暂不实施”不是后续agent取得work指令后的永久禁令。r1/r2历史收据保留，最新来源见§2、§4.3及r3交接附件。

以下为2026-10-21 r1只读调查的历史快照，不作为r2当前交付/基面结论：主检出 master，调查 HEAD `9b65383e0e73e94b90a065881a1333b6b54960a3`。829实现位于 `D:/autostack/.wt/lang-829/auto-lang`；当前树/HEAD信息及作者work记录只是观察，work消费须重新核实G829与dirty状态。沙箱内 `git -C` 对该树报 Permission denied；本轮通过可读文件内容/hash及主仓worktree清单调查，不将其称为已核实clean的冻结版本。

| 来源 | 核实的事实 | 本Plan据此选择的范围 |
|---|---|---|
| `docs/specs/overview.md`、`docs/specs/auto-lang/ui/overview.md` | 有独立实验与公开UI边界；current Spec尚未记载AutoGraphics正式交付 | 不用目标态代替当前实现；提案与current分开 |
| `docs/specs/repository-boundaries/project.md` | 三仓受控迁移与独立消费/资源闭包仍有明确门 | 不借原型直接接生产或改旧仓默认 |
| 829当前Plan/§9、`AGP/research/result.md` | 作者申报两游戏、共享设备/回放/73测试和10probe；尚未独立review/merge | 作为候选输入，不作为本会话重验结论；必须先G829 |
| `AGP/research/integration-handoff.md` | 仅单窗口UI覆在图形之上；I1–I3和任意嵌套/射线/焦点/IME未实现 | 下一Plan优先解决组合与输入的独立实验面 |
| `AGP/src/core/scene.rs`、`runtime.rs` | 现有对象/变更/代际与游戏driver，没有Surface/Portal依赖图与挂载路径 | 在既有core上增补组合，不重写游戏/全场景引擎 |
| `AGP/src/renderer/target.rs` | 目标支持渲染/readback；没有采样usage | 离屏内容成为纹理输入需实际实现与校验 |
| `AGP/src/host/iced_spike.rs` | 顶层控件＋图形，prepare借Iced设备；有顶层全局stash/graphics状态 | 不把这个特例当可嵌套实例系统；新内容实例/回收要可区分多实例 |
| `docs/plans/814-language-sdk-ui-package-separation.md` | executing r3，9/15；814/815/816均未见完整迁移完成收据 | 不把M814-A/在途runtime当最终生产入口 |
| `docs/plans/attachments/829/review-r1/behavior-probe.rs` | 存在正在形成的行为/缓存反例调查，尚无本轮可引用的最终verdict | 留在829原验收解决；下一Plan不代判、吸收或掩盖 |

本轮未运行上述测试/游戏或GPU探针。来源hash与规划预检见 [附件](attachments/830/planning/preflight.md)。`docs/specs/auto-graphics/` 当前不存在，829的规范仍为拟议/候选来源；frontmatter中的modify目标是对其未来正式来源的provisional影响，T-01必须核实。

官方设计输入：[Iced UserInterface](https://docs.rs/iced_runtime/0.14.0/iced_runtime/user_interface/struct.UserInterface.html)、[Iced Primitive](https://docs.rs/iced/0.14.0/iced/widget/shader/trait.Primitive.html)。它们证明公开布局/更新/绘制与GPU接缝存在，不证明反向离屏UI/本Plan组合已经可用。

### 4.2 为什么不先做下一批游戏或生产接线

两款游戏已覆盖一批基础能力，继续加游戏会延后本版最关键的“UI和图形双向嵌套”验证。真实Auto/生产UI接线又依赖仍在迁移的公开出口；本计划复用已有游戏内容，先验证组合架构，降低迁移后把独立原型拼成另一套系统的风险。后续Auto入口、生产RQHost和Forge保持具名合同，不因本次原型进展自动完成。

### 4.3 当前来源与设计依据（2026-10-22 r3）

829已归档11/11，正式实验交付落autographics-dev；`docs/specs/auto-graphics/`的project/prototype-profile/integration-contract/plans已存在。模块Spec明确后继830+各自plan分支审毕后合回autographics-dev，master仅沉淀文档，生产回合仍等814/815/816系列及后续迁移完成后另议。本轮核对来源与既有门禁收据，不代跑829测试。现行HIR/MIR仍为有限类型/执行域，不包含拟议AutoShader/CAP实现。

829候选树`src/renderer/pipeline.rs`的`VERT`/`QuadPipeline::new_with`以手写WGSL和宿主Device/Queue创建管线，`target.rs`保留待扩采样usage，`error.rs`已有BadShader分类。它们只用于识别接缝，不作为本轮重新测试或冻结发布结论。CAP与[AutoShader小范围草案](../design/strategy/autoshader-minimal-language-and-renderer-interface.md)支持在T-02/04/08记录现有资源/布局/错误及局部分层，不要求实现新语言。

r2原读取指纹见[r2规划预检](attachments/830/planning/preflight-r2.md)，当前冻结来源核查见[r3交接](attachments/830/planning/handoff-r3.md)。§5.6与V-10/AC-08的Shader交接合同保持；§8仍为同一组T-01..10，不追加工作清单。

## 5. 详细设计

### 5.1 内容身份、挂载与依赖

- Surface表示有自身状态/更新的内容实例；View指定该内容的观察/输出；Portal是父视图到子内容的挂载、呈现与输入映射，不是另一个OS窗口。
- 同一Surface可被多个Portal引用；内容状态共享、模拟每tick只推进一次，各挂载有独立路径与命中/焦点位置。不同Surface实例不能因相同renderer/类型而共享文本、计数或游戏状态。
- 维护独立的布局/空间挂载关系与渲染依赖图。先绘制被消费的离屏内容，再绘制其消费者；共享子内容只更新/绘制必要次数。循环（包括自环/间接环）在提交前拒绝；不支持上一帧反馈回路，不能每帧递归直到栈溢出。
- 新组合批次先校验全部身份、引用、依赖和变换，再提交；非法批次不留下半个新图或替换最后可用场景。它不改829旧的逐变更接口。
- 支持有限无环多层组合，正式案例覆盖两个三层方向与一次更深组合。测试深度不是产品硬上限；T-02根据实际资源决定可配置预算和拒绝分类，不临时发明类型/语法深度限制。

### 5.2 GPU与离屏UI

T-02先在已冻结Iced版本完成真实button/text_input→离屏纹理→3D平面→屏幕的whole-path实验，同时把输入送回该真实控件。确认公开renderer的目标、帧缓存/资源、借用顺序与线程亲和；最多比较两种公开接法。不允许以每帧CPU截图/图片替代UI texture，不因失败直接fork生产UI或跳过输入；需要扩大patch/公开面时携证据回new。

统一GpuContext/代际身份，由宿主持有Device/Queue；各内容adapter只取得受控借用。每个活跃目标的尺寸/格式/alpha/颜色空间/usage及深度由owner明确，禁止同一texture在同一pass内边写边采样。绘制优先序/屏障及故障保留策略从实际编码路径验，不靠自报same-device布尔值。

标准合成采用明确的线性/sRGB转换与预乘alpha合同；先冻结格式和边界，再做彩色/半透明基准。旧829不透明blit仍能重放；不能把alpha强制1当本次半透明组合已通过。正常交互不做GPU→CPU→GPU逐帧搬运；readback仅用于验收捕获/显式导出，并记录发生次数。

### 5.3 命中、焦点和输入

| 输入规则 | 本版约定与反例 |
|---|---|
| 坐标链 | 窗口物理坐标→DPI逻辑布局→裁剪/缩放→3D射线与最近可交互平面→UV→子视图局部坐标；每一层按实际挂载路径转换，不能只减窗口偏移 |
| 可交互区域 | 可见裁剪外/被遮挡/不可交互区域不收事件。透明Portal默认仍按其声明几何区域参与命中，显式passthrough才穿透；不承诺任意shader逐像素alpha命中 |
| 遮挡 | 3D使用与呈现匹配的深度/前后策略，2D叠层按明确前后顺序；overlay消费后不向底层重复派发 |
| 捕获 | 按下后拖出Portal，move/up仍送捕获路径；销毁/失焦/窗口离开时发送取消并释放，不把迟到up送新对象 |
| 焦点 | 焦点绑定window＋Surface代际＋挂载路径＋控件；点击其他UI切换唯一键盘目标，旧路径不保留悬空编辑状态 |
| 键盘/IME | 真实text_input接收字符/选择/提交/取消；至少一个真实OS中文IME组合、候选定位和切焦点不串写流程。合成事件只证明逻辑，不抵扣OS级IME证据 |
| 非法输入/变换 | stale/session/device引用、NaN/Inf或不支持的不可逆映射明确拒绝且不污染当前状态；零尺寸目标暂停，恢复后沿现行布局重新映射 |

共享内容多次挂载时，文本/游戏状态按Surface共享，只有当前命中/捕获的挂载产生一次动作；独立内容实例各自保留状态。焦点/IME光标的屏幕锚点由当前活跃挂载路径计算，即使UI纹理位于3D平面也不把候选窗放回原离屏坐标。

### 5.4 固定验收场景与兼容

| 场景 | 实际内容与操作 | 关键oracle |
|---|---|---|
| S-01 UI→3D→UI | 外层真实Iced布局包含3D视图；视图内的可变换平面承载另一个真实Iced button/text_input | 点击/输入改变内层控件并可观察；外层相似控件不动；clip/透视/缩放后仍命中正确 |
| S-02 3D→UI→2D | 3D场景中UI面板的子区域显示Breakout；UI控制暂停/reset，键盘焦点决定操作游戏还是文本 | 原游戏复用，实际状态与画面同步；UI消费输入时游戏不响应，reset不保留旧挂载事件 |
| S-03 多实例/共享视图 | 两个独立UI实例＋同一个Squash内容的两个Portal；一个内容单次tick、两处呈现 | 独立文本/计数不串；共享游戏速度/生成数不翻倍；输入只向目标发一次；关闭一视图不销毁仍被引用内容 |
| S-04 组合更新/失败 | 深层变更、reparent、局部resize、隐藏/恢复/销毁、加环/坏资源/迟到事件 | 子与依赖祖先变脏，无关兄弟目标/身份保留；失败不提交半图，合法正控仍能继续 |

不额外要求新完整物理/动画或重做样本。829正式冻结的游戏回放/视觉与音频oracle继续适用；需要修复其原问题时回到829发布新来源，不能在830偷偷重写golden使兼容变绿。

### 5.5 迁移后边界与规范增量

本Plan不写 `crates/auto-ui-*`、`crates/auto-lang`、根manifest/锁、Man/CLI或其他仓；在AGP内形成可交接adapter及数据合同。独立实验可在G829后推进，不要求三仓全部迁完；生产主分支的组合切换仍等迁移完成收据与真实owner放行。

IH-01只增加实验同设备/离屏UI证据；IH-04增加实际独立宿主的两种嵌套与输入证据。IH-02真实Auto入口、IH-03生产RQHost、IH-05生产发行仍pending；I1/I2/I3状态不因本Plan通过自动修改为done。下一生产集成Plan必须按届时SDK/runtime/protocol的实际公开符号落地并重放，不把本实验API冒充已发布生产ABI。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-graphics/surface-composition.md` | 新增experimental内容/挂载/依赖、GPU离屏合成、循环/提交/资源边界 | 统一双向组合语义 | AC-02、AC-03、AC-06 |
| SD-02 | add | `docs/specs/auto-graphics/nested-input.md` | 新增实验路径命中、捕获、焦点、键盘/IME与实例隔离 | 输入跟随组合，不另造控件 | AC-04、AC-05 |
| SD-03 | modify | `docs/specs/auto-graphics/prototype-profile.md` | 从829已发布双游戏/单层Iced探针扩到本次Windows单窗/平面Portal和实际测量；未支持面明确 | 防止无限嵌套/平台外推 | AC-01、AC-06、AC-07 |
| SD-04 | modify | `docs/specs/auto-graphics/integration-contract.md` | 回写IH-01/04实验增量及当前Shader资源/布局/错误/renderer接缝与后续缺项，I1–I3/生产接线仍pending；不声称已支持AutoShader | 分清实验与生产交付 | AC-08 |

上表是本Plan的provisional增量；SD-03/04以829已沉淀的current Spec为修改来源。审核通过并合入autographics-dev后再沉淀830实际实验增量，不将生产Auto/UI/RQHost或AutoShader标为已交付。

### 5.6 CAP/AutoShader接缝（r2，仅补分层与交接）

语言方向见[AutoShader草案](../design/strategy/autoshader-minimal-language-and-renderer-interface.md)，本Plan只做以下当前实现可核的补充：

| 合同/owner | 实际路径与必要事实 | 可检查结果及边界 |
|---|---|---|
| T-02，资源/布局事实 | 当前WGSL与host绑定→GPU目标/采样→合成；记录格式/usage、颜色/alpha、纹理/采样器、当前参数offset/size/alignment及stage、所用features/limits与错误分类 | 记录与真实whole-path代码/捕获相符；未知事实具名列缺口，不能用未来AutoShader类型填充或自报supported_gpu代替 |
| T-04，局部renderer边界 | Surface/Portal身份与DAG→内容adapter→renderer内shader创建/绑定→当前Device/Queue | core/组合调度不依赖Auto AST/VM Value或手写WGSL文本组织；renderer可以继续使用内置WGSL，已满足局部分层时只记录接缝，不强制提取通用Shader插件框架 |
| T-08，交接 | 现有`surface-integration-handoff.md`→未来语言/renderer owner | 列当前stage IO、绑定/布局、device/资源代际/生命周期、错误与真正producer/consumer；标明AutoShader生成/验证/接线尚缺项。V-10/AC-08逐项核文档与当前代码 |

Surface内容依赖图负责更新/合成；Shader内部计算/调用图由未来device编译处理，分别建模。未来编译产物经局部adapter进入现有renderer，不把wgpu对象或语言AST放进公共组合模型，也不另建Device/Queue。

本次不实现AutoShader源码入口/生成器、CAP推导/注册/包求解、device IR、用户自定义Shader加载/替换/热重载、材质编辑器或第二后端。不新增runtime探针族、shader注入或替换验收；既有V-02..09继续验证当前WGSL的资源/合成/输入行为，新增文档一致性只归V-10。未来Shader重载的last-good要求留给其专项Plan，不扩大本Plan失败恢复分母。

## 6. 测试设计

以下测试/runner入口均为**拟新增**，T-02..08使其可运行；本次未执行。使用本Plan唯一实施树，`AGP=experimental/autographics-prototype`。

- `cargo check --manifest-path experimental/autographics-prototype/Cargo.toml --locked`
- `cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked --test surface_contract <test_filter>`：Rust下划线筛选名，至少命中具名用例，零匹配不作证据。
- `cargo run --manifest-path experimental/autographics-prototype/Cargo.toml --locked --release --bin graphics-probe -- --case <case_id> --replay canonical --report scratch/830/<case_id>`：复用829 runner，新增surface类case，预期真实输出/trace与非skip报告，失败非零。
- 复跑G829已发布的两游戏及同设备probe，命令以其冻结README为准；本Plan不改其游戏规则或外部oracle。

| V / case_id | AC/不变量 | 输入/真实入口 | 可观察结果与区分反例 | owner/task/边界 |
|---|---|---|---|---|
| V-01 / surface-model | AC-02 | composition提交：正常两实例/共享挂载、self/间接cycle、stale/跨session、非法变换 | 合法图确定可遍历/调度，失败批次前后快照/资源相同；旧Scene逐变更回放仍保持 | work T-03；组合model，不外推全场景事务 |
| V-02 / surface-offscreen | AC-03 | 真实Iced button/text_input离屏→3D平面→同窗；半透明色块/clip/多层纹理 | 真实控件图像与输入变化；有采样usage和合法pass顺序；正确颜色/alpha，稳态无CPU帧搬运/第二Device | work T-02/T-04；public-Iced实验，不是生产AutoUI |
| V-03 / surface-ui-3d-ui | AC-03/04 | S-01，透视/倾斜面板、layout/resize、DPI1/2变换；实际窗口键鼠 | UI→3D→UI画面+目标控件message/状态；相邻同形控件未动，clip/遮挡反例不点击穿透 | work T-04/T-05；真实机器DPI另记，模拟DPI仅作变换测试 |
| V-04 / surface-3d-ui-2d | AC-03/04/07 | S-02，Breakout输入、面板UI与文本焦点、暂停/reset | 3D→UI→2D实际显示和事件回路；动作不重复/不串游戏，既有玩法oracle仍适用 | work T-05/T-07 |
| V-05 / surface-focus-ime | AC-05 | 内外文本框切换、拖出边界、销毁capture目标、真实中文IME组合提交/取消 | 只在目标输入；一次提交一次文本变更，候选屏幕位置随实际挂载变换，取消/失焦无悬空状态 | work T-06；合成事件不能代替OS输入证据 |
| V-06 / surface-instances | AC-04/06/07 | S-03，同内容两Portal＋独立UI实例，固定相同时钟/输入 | 共享内容每tick仅更新一次、两投影视图一致；独立状态不串、两种instance语义均有正控 | work T-07；不以两份复制游戏冒充多视图 |
| V-07 / surface-lifecycle | AC-06 | S-04，20轮resize/隐藏/卸载重挂，视图关闭/恢复、坏资源/迟到event/旧GPU代际 | 所属资源/任务与warm baseline回收关系可核；仍被引用内容保活，失败保last-good且不可继续用旧epoch；不全量重建无关目标 | work T-08；OS缓存/驱动驻留单列，不只看RSS |
| V-08 / surface-negative-controls | AC-02..06 | 在真实生产调用路径注入错UV/漏clip、重复事件/加倍tick、opaque替代alpha、每帧readback、第二device/错误依赖次序 | 对应正式gate非零且错误分类匹配；合法正控过；只篡改报告布尔值不能误绿 | work T-08；scope限定本Plan，可用受控test hook不可影响正常路径 |
| V-09 / surface-compat-metrics | AC-01/06/07 | 独立locked构建/依赖闭包、A→B重定位、G829游戏/回放＋组合性能计量 | 旧两游戏和接口行为保持；报告CPU/GPU可用计时、dirty/绘制/上传/readback/目标重建量与内存；GPU timing缺席准确记录 | work T-08；没有竞品/跨平台速度承诺 |
| V-10 / handoff-audit | AC-08 | 当前SD/IH、G829收据及§5.6对应renderer代码/布局/资源/错误 | 交接具名producer/consumer与缺项，布局/设备事实可追溯；组合模型无语言AST/WGSL文本耦合。AutoShader/CAP及Auto/生产UI/RQHost/Forge未被冒记done；仅当前手写WGSL仍应能通过，不要求生成/注入/重载 | work T-02/04供事实、T-08交接；review独立核对；不新增runtime测试族 |

同一真实窗口获取画面、输入回传、内容状态与实际资源操作的关联证据，不以“图片不同”代替点击命中，不以自报token/hash代替同设备/真实采样。配套独立参考坐标/射线算例验证转换；输入反例要同控件同类型，避免用无目标掩盖错误路由。

早期T-02需冻结格式/alpha、可调预算、失败分类与测量方法；必要策略不明或公共Iced路线不可行时停在有界可行性结论并回new，不能改成位图假UI或无输入样例继续报pass。游戏/资源/控件无跳过即通过路径。

当前未指定实现/内部检查/最终复审人或模型，不创建agent/chat。work实现后由独立final review在同一树冻结SHA验收；merge-owned门在最终review通过后执行，作者本轮预检不是final review。GUI/IME/GPU占用需串行协调；纯模型/定向测试可按写面推进。

## 7. 验收标准

| AC | owner_stage | 完成标准/验证 |
|---|---|---|
| AC-01 | work | G829与版本/Spec来源核准、独立workspace/锁与根生产零改动；来源指纹、V-09/10 |
| AC-02 | work | 有代际的Surface/View/Portal与多实例/共享挂载，正确DAG调度/循环拒绝及新组合批次无半提交；V-01/08 |
| AC-03 | work | 两种三层嵌套及真实Iced离屏/采样/alpha合成成立，同设备、无稳态CPU往返；V-02/03/04/08 |
| AC-04 | work | 挂载路径/射线/UV/裁剪/遮挡/捕获正确，动作不重复且实例隔离；V-03/04/06/08 |
| AC-05 | work | 真实控件焦点/文本/IME和失焦/销毁取消正确，不串写，候选窗锚点正确；V-05及对应反控 |
| AC-06 | work | 局部dirty更新/单次tick/资源保活释放、失败恢复与准确计量；V-06/07/08/09；20轮生命周期有原始证据 |
| AC-07 | work | Breakout/Squash复用原行为而非新复制玩法，旧profile/oracle继续通过，独立消费与重定位成立；V-04/06/09 |
| AC-08 | work | 实验合同/SD与生产IH/迁移门及§5.6当前Shader资源/布局/renderer接缝交接完整，V-10逐项对代码/事实；AutoShader/CAP与I1–I3仍分层pending，不新增语言实现或替换测试，829问题没有转嫁；V-10 |
| AC-09 | review | 独立逐项核AC/SD/正反控与偏差/警告/格式/残留，绑定revision/代码/输入；merge-owned门pending明确 |
| AC-10 | merge | review后冻结重放实验适用门，按G829与分支归属合同合入autographics-dev、沉淀实验事实/归档清理；不合入master或冒称生产交付，生产迁移门保留 |

## 8. 执行步骤

| Task / 状态 | owner_stage | 前置 | 路径/符号（AGP为既有实验，标“新”为新增） | 结果、AC与验证 |
|---|---|---|---|---|
| T-01 [x] | work | 消费r3；用户向实施agent发出work指令；G829已具备；核唯一实施树 | 本Plan与`AGP/research/surface-source-baseline.md`（新） | 从冻结autographics-dev源建树，核验收/来源收据、源码/锁/hash、stock依赖及现行Spec；AC-01/08。不从master空原型路径或已退役829树起步，不恢复旧patch。[✅ 2026-10-22 work] 树`D:/autostack/.wt/lang-830/auto-lang`@`plan-830-dev`基提交ab56c32ec；祖先检查+AGP子树ls-tree逐字节一致通过；无[patch]段、iced 0.14.0/iced_wgpu 0.14.0/iced_widget 0.14.2(stock)/wgpu 27.0.1、锁sha256=6cd34576…；基面不含docs/specs/auto-graphics/（829 Spec在master侧），SD草稿worktree内起草、发布归T-10；基线见surface-source-baseline.md，提交4d287cc3b |
| T-02 [x] | work | T-01 | `AGP/src/host/iced_surface_probe.rs`、`research/surface-feasibility.md`（新），既有renderer公开接缝 | 有界whole-path：真实Iced离屏button/text→3D纹理→同窗＋输入返送，确认资源/线程/format/alpha/预算/诊断，并记录§5.6当前Shader参数布局/绑定与设备要求事实供V-10；V-02；AC-03/05/06。不明确前不开展依赖实现。[✅ 2026-10-22 work] 公开API链实证可行（Engine/Renderer/UserInterface::build-update-draw/present 全公开，新增 stock iced_runtime 0.14 直依）；surface-offscreen case 窗口模式 6/6 门禁绿（presses=1、input="ok8"、坐标往返 0.000px、采样usage/预乘alpha/单设备/稳态零readback）；DPI虚拟化与WM_KEYDOWN注入坑修复在案；§5.6 布局/绑定/错误事实落 surface-feasibility.md；提交 b4c93e362 |
| T-03 [x] | work | T-02合同可行 | `AGP/src/core/composition/`、`tests/surface_contract.rs`（新），既有id/scene边界 | 身份/内容/挂载/依赖/提交与dirty传播；V-01/相关反控，AC-02。旧Scene语义保持。[✅ 2026-10-22 work] `core/composition/{mod,input}`：Surface/View/Portal newtype 复用 core::id 代际分配器（无第二身份源）；整批校验在克隆上模拟→全通过才整体 swap（失败零副作用、不耗 id，测试用 snapshot 证明）；自环/间接环/深度预算均 `CycleDetected`/`DepthBudgetExceeded` 具名拒绝；dirty 级联到消费者链且不污染兄弟；render/tick 序为单次拓扑（共享 Surface 只一次）；输入链 2D clip/非交互遮挡/捕获路径保持/焦点随路径失效/NaN 拒绝。surface_contract 18/18 绿；AGP 全测试无回归；提交 03590667a |
| T-04 [x] | work | T-03 | `AGP/src/renderer/composition/`（新）、既有target/pipeline，`host/iced_surface_probe.rs` | 可采样离屏目标、DAG编码/多层GPU合成、S-01/02画面/alpha/clip；V-02/03/04，AC-03；保持§5.6组合与Shader来源的局部分层并记录接缝，不新增通用用户Shader API；V-10/AC-08；不创建第二设备。[✅ 2026-10-22 work] `renderer/composition` 增 PortalQuadPipeline（预乘over + scissor clip）与 FullScreenBlit；`host/surface_runtime.rs` 以 `graph.render_order()` 驱动：每 Surface 至多一次内容穿透（共享两次挂载实测 1 次）、子先于父、3D 平面用父相机、2D 四边形按声明序顶绘；surface-compose case 10/10 绿（三层 UI→3D→2D、alpha 线性期望实测 [188,138,138] vs 期望 [188,137,137]、clip 越界像素被切、稳态帧 readback=0、采样 usage 合同与 829 旧 new 双向区分）；无第二设备；提交 20a298842 |
| T-05 [x] | work | T-03/T-04对应接缝 | `AGP/src/core/composition/input.rs`、`host/composition_input.rs`（新） | 逆坐标/射线/UV/遮挡/clip与实例路径、一次派发/捕获；V-03/04/06；AC-04。[✅ 2026-10-22 work] `host/composition_input.rs`：window↔top 视图换算、2D 仿射反算、3D 射线/UV（与渲染同一相机）、`anchor_to_window` 沿挂载链正向投影、`set_ime_anchor_screen`/`client_to_screen` OS 锚点；surface-input 窗口模式实测真实 OS 点击穿过 3D 平面命中真实 button（os_window_events）；嵌套 2D 挂载独立送达、捕获保持/销毁取消 `Cancelled(TargetDestroyed)`、无跨实例串写；提交 e098dd28d |
| T-06 [x] | work | T-05 | `AGP/src/host/iced_surface_input.rs`（新），Iced真实控件/host桥 | 焦点/键盘/text_input、实际OS IME锚点/提交取消、失焦/销毁捕获取消；V-05/反控；AC-05。[✅ 2026-10-22 work] 实现并入 `host/composition_input.rs`（无独立文件，职责等价）：焦点唯一目标随点击迁移、键盘/IME 只投递焦点面、IME Opened/Preedit/Commit/Closed 经真实 text_input（preedit 不写值、commit 生效、A=ab / B=cd你 不串写）、IME 锚点按挂载链投影到窗口且落在平面包围盒内；销毁/失焦取消捕获并递增 generation（迟到 release 不再命中）。**OS 级 IME 已补证**（2026-10-22 后续）：新增 `surface-ime-os` 用例在真实窗口中启用系统中文 IME，真实键盘产生 preedit 并提交 CJK（`text="爱彼此你好你好"`，7/7 门禁含 `os_ime_commit`）；期间修复两处实缺陷（slot 与 iced 双纹理导致内容恒空、`text_input` 缺 `on_input` 导致"看得见打不了字"），详见 §9 与 `research/surface-result.md` §6 |
| T-07 [x] | work | T-04/05/06 | `AGP/src/host/cases/surface_*.rs`（新），既有game drivers | S-01..03与两游戏复用、多挂载单tick/共享与独立实例；V-03/04/06和旧游戏回放，AC-04/07；不改829golden。[✅ 2026-10-22 work] `surface-scenarios` 8/8：S-02 复用**冻结** Breakout 驱动（非新玩法）经真实 iced 按钮暂停/恢复，每帧单次 tick、暂停即停、恢复续跑、面板纹理含游戏色；S-03 两独立 iced 实例＋同一游戏面两个挂载：每帧恰一次内容穿透、实例计数不串（ui1=1/ui2=0）；提交 def56b81b |
| T-08 [x] | work | T-01..07 | 既有probe runner/gates，`tests/surface_*.rs`、`research/surface-result.md`、`surface-integration-handoff.md`（新） | V-07..10、负控与20轮生命周期、计量/独立locked消费与重定位；当前SD/IH与§5.6资源/布局/错误/producer-consumer交接、AutoShader/CAP及生产pending清单；V-10对当前代码；AC-01/06/07/08。[✅ 2026-10-22 work] `surface-lifecycle` 10/10：20 轮卸载/重挂/拒绝变更（20/20 保留未挂载子内容、图稳定、target 重建仅来自显式 resize、稳态 0 readback、无关面脏跳过 108/60 帧）；4 项负控均「正控过＋反例被抓」（clip 内外判别、双渲染检测、alpha `[188,138,138]` vs 不透明 `[137,0,0]`、渲染路径零设备构造）；V-10 交接文档落 `research/surface-integration-handoff.md`（producer/consumer、§5.6 布局/绑定/错误、缺口与 pending 清单）；SD-01/SD-02 草稿落 `docs/specs/auto-graphics/`；V-09 计量（CPU p50/p95、穿透/合成/脏跳过/readback/重建）在报告内，GPU timing 明确 false；AC-01 直接证据 `git diff --name-only ab56c32ec` 在 AGP 外**零文件**；829 两游戏+iced spike 复跑 exit=0；提交 35e1936be |
| T-09 [ ] | review | work T-01..08完成、冻结代码/输入、作者停写 | 本Plan§9与同树证据 | 独立final review逐项验AC-01..09、SD-01..04，不代修；AC-09；正式merge-owned门pending |
| T-10 [ ] | merge | T-09 pass；核G829及autographics-dev集成来源/冲突 | 实验实现合入autographics-dev、Spec/ledger/归档与清理收据 | 冻结实验workspace测试/探针、受控集成/沉淀与guard clean后清理；AC-10。不将实验代码合入master；仅实验Rust改动不触根t/tv/tu/docs_gen；扩大触面先回new，tf仍主检出到期单实例 |

拟实施树 `D:/autostack/.wt/lang-830/auto-lang`、分支 `plan-830-dev`，一Plan一仓一树；从 `autographics-dev@ab56c32ec4cb28e01fbcca74e9861164ffad43f0` 冻结来源起步，不从master起步，不复制其他树WIP。开工时读取主检出本r3（它可能尚不在该历史代码基面），将合同来源/hash写入T-01，不用旧基面里的Plan文本覆盖主检出当前计划。当前核源会话不建树/分支。

T-02可行性与T-03共同合同先行；renderer与输入的独立模块可按交接接口交错推进，公共core/manifest/锁/GUI输入单owner串行。此依赖图不自动授权多agent。work完成8项后可进入execution_done（8/10），review/merge未完成保持未勾选；review后按既定合同集成到autographics-dev，生产master回合仍须等迁移放行。

## 9. 复审记录

### 2026-10-22 work r2：AC-05 OS-IME 补证与两处实缺陷修复（execution_done）

- `stage: work`；`plan_revision: 3`；`status: execution_done`；`current_step: 8/10`；`outcome: pass`；`next: review（T-09）`。
- 触发：用户在实机查看到组合窗口里"input 可见但打不了字"，据此定位到两个真实缺陷（都是我实现里的错，不是 iced 的行为）：
  1. **双纹理**：`SurfaceRuntime` 的 slot 目标与 `IcedSurface` 自有目标是两块纹理 → iced 渲染进自己那块，合成/捕获读 slot 那块，组合窗口里 iced 内容恒为空。修：`IcedSurface::frame_into(&slot.target)` + `new_borrowed`（运行时不持有自有纹理）；相应修复 `surface-compose` 的 `plane_shows_ui` 假阳性（原判据"亮度>200"连天空都满足，改为对 UI 背景 `249,250,254` 精确比对并排除天空色）。
  2. **缺 `.on_input`**：`text_input` 只渲染不接收输入（能画、能聚焦，键盘与 IME commit 全被丢弃）。修：补 `on_input`，并新增 `surface-ime-os` 用例端到端验证。
  3. 附带修测试自身 u8 溢出（`250+40` 回绕）造成的 clip 门禁假阴性。
- AC-05 现已满足：`surface-ime-os` 7/7（窗口聚焦、`Ime::Enabled`、public focus 探针确认焦点、真实键盘落字、preedit 观察、**CJK commit** `text="爱彼此你好你好"`、候选锚点随挂载投影）。
- 回归：AGP `cargo test` 135 passed / 0 failed；`surface-offscreen`/`compose`/`input`/`scenarios`/`lifecycle`/`ime-os` 全部 exit=0；829 两游戏与 iced spike 复跑 exit=0。
- 代码：`dd2788bcc`；`base_commit: ab56c32ec`；`branch: plan-830-dev`；worktree `D:/autostack/.wt/lang-830/auto-lang`。
- 余留（非阻塞，记录在案）：候选窗逐像素截图比对、任意深度遮挡、GPU timestamp 计时；T-09 review / T-10 merge 未勾选。

### 2026-10-22 work：T-01..T-08 执行与交接

- `stage: work`；`plan_id: PLAN-830`；`plan_revision: 3`；`status: executing`（**未置 execution_done**）；`current_step: 8/10`；`outcome: blocked`（一项验收证据受环境限制，见下）；`next: 用户决定 OS-IME 证据补取方式后继续 work 收尾，再送 review（T-09）`。
- `worktree_path: D:/autostack/.wt/lang-830/auto-lang`；`branch: plan-830-dev`；`base_commit: ab56c32ec4cb28e01fbcca74e9861164ffad43f0`；`code_commit: 35e1936be`；依赖修订：AGP 新增 stock `iced_runtime 0.14.0`（同版本线，未改根仓锁/manifest）。
- 任务：T-01..T-08 全部完成（8/8）；T-09/T-10 未勾选（review/merge 属后续角色）。
- 证据（均本树可复现，详见 `AGP/research/surface-result.md`）：
  - `cargo test` = **135 passed / 0 failed**（829 原 117 + 830 新 18）；`surface_contract` 18/18。
  - `surface-offscreen`（window）6/6、`surface-compose` 10/10、`surface-input`（window）14/14、`surface-scenarios` 8/8、`surface-lifecycle` 10/10；829 `ag2d-breakout`/`ag3d-squash`/`iced-shared-device` 复跑 exit=0。
  - 真实 OS 输入证据：`surface-offscreen` 与 `surface-input` 窗口模式均为 `os_window_events`（点击穿过 3D 平面命中真实 iced button，presses 增加；键入 ok8；IME 逻辑与锚点已验证）。
  - AC-01：`git diff --name-only ab56c32ec | grep -v ^experimental/autographics-prototype/` = 0 文件（根生产零改动）。
  - AC-06：20 轮生命周期 + 4 项负控（正控过/反例被抓）在 `surface-lifecycle` 报告。
  - AC-08：`research/surface-integration-handoff.md` 对当前代码逐项；AutoShader/CAP 与 I1–I3 明确 pending，未冒记。
- **阻塞项（AC-05 部分不满足）**：OS 级中文 IME 组合（真实候选窗随光标）需要交互式桌面会话 + 已装配中文输入法；本会话仅取得**合成 IME 事件**下的逻辑证据与锚点计算正确性，按计划 §6「合成事件不能代替 OS 输入证据」不冒记 AC-05 全绿。精确 unblock：见 §10 第 5 条。
- 期间修复的实错（记录在案）：`OffscreenTarget::resize` 曾把采样目标降级回不可采样（窗口首帧 wgpu 校验失败）；iced 事件需按面投递光标（`Cursor::Unavailable` 使控件永不响应）；注入脚本需 `SetProcessDPIAware()` 与带扫描码的 WM_KEYDOWN（winit 不处理 WM_CHAR）；PowerShell 布尔字面量需 `$true/$false`。
- 未触发任何整档门禁（`cargo t/tv/tt/tb/tu/taa/tf`）：本 Plan 只改 `experimental/autographics-prototype/**`，按仓库分级规则属独立 workspace 触面；AGP 自有全量测试已跑。merge-owned 门禁仍 pending。

### 2026-10-22 new r3：G829来源放行与实施交接

- `stage: new`；`plan_revision: 3`；`status: drafting`；`current_step: 0/10`；`outcome: pass`（来源与规划交接预检）；`next: 用户另开agent执行work，从T-01进入`。
- 依据829 archived 11/11、R18/冻结merge门/清理收据及current Spec，核G829已满足；autographics-dev冻结ab56c32ec的AGP子树与ddbde3e62无差异，祖先检查通过。具体原始门禁为前序收据，本轮未重跑。
- r3仅更新源门、实际stock依赖、当前Spec及开工/合入分支；组合/输入、S/V案例与r2 CAP/AutoShader交接验收保持。T-01/10与AC-10按用户已裁定的集成分支承接；任务仍10项全部未勾选。
- 主检出维护计划，代码从冻结autographics-dev起步，审毕合回该分支；生产master/真实UI/RQHost迁移门不变。证据见[r3交接](attachments/830/planning/handoff-r3.md)。本轮不创建agent/树，不启动实现，不改829归档合同或current Spec。

### 2026-10-22 new r2：CAP/AutoShader接缝补充

- `stage: new`；`plan_id: PLAN-830`；`plan_revision: 2`；`status: drafting`；`current_step: 0/10`。用户已授权此文档修订及小范围AutoShader草案，明确暂不启动实现；本轮未创建worktree/分支、未改实现/current Spec或829状态。
- `outcome: blocked`（仅未来实施来源G829尚未核得有效消费收据；r2规划预检通过）；`next: 本轮文档交付结束，未来取得启动授权与有效D829或获准E829后，由work从T-01核源进入`。不重复要求确认已授权的文档范围，不把外部来源门当本次文档未完成。
- 改动：T-01更新r2/实际开工授权；T-02/04/08补§5.6资源布局与分层/交接；V-10/AC-08及SD-04补当前Shader接缝。AC-01..07、AC-09/10、S-01..04、V-01..09及T-03/05/06/07/09/10合同保持，十任务不增项，全部未勾选。原Windows/单窗/真实Iced/IME、游戏兼容、G829与迁移门保持。
- 829已由r1观察时execution_done更新到reviewed，最新R18所记ddbde3e6只用于来源定位；本轮未执行其review/merge门、未发布E829或代认D829。r1旧f7候选及调查/预检保留为历史，不再指示当前基面。
- 验证：来源/链接/任务AC映射及r1→r2差异预检见[preflight-r2](attachments/830/planning/preflight-r2.md)；静态结构检查不代替实现验收，未跑Cargo、GPU/窗口或IME测试。AutoShader样例仍是设计，未来测试未运行。未指定实现/复审人或模型，未建agent/chat。

### 2026-10-21 new r1（历史；当前合同为r2）

- `stage: new`；`plan_id: PLAN-830`；`plan_revision: 1`；`status: drafting`。
- `outcome: blocked`（仅实施来源前置尚缺；计划合同预检通过）；`next: 原owner完成829验收及G829冻结交接，再进入work`。计划编制已完成，状态仍drafting；当前允许只读准备，不表示已放行依赖实现。
- 任务T-01..10、AC-01..10、SD-01..04；source/review/merge与实验/生产分层明确。当前不改829进度或其验收发现。
- 未知公共Iced离屏/IME实施路线归T-02整链探针和T-06真实验证；无法满足原目标时回new，不以假UI或跳过输入绕过。
- 预检及来源hash见 [附件](attachments/830/planning/preflight.md)；本次未跑Cargo、原游戏、GPU或IME验收，也不是独立final review。

## 10. 待澄清事项

无阻止完成本轮规划的必答问题；实施前必须具备/解决：

1. **来源前置已满足**：829已归档并实际集成，G829有效。实施agent按T-01复核冻结源/工作树即可，不再等待829进master或重复索取来源许可；本会话不代启动其实现。
2. **可行性**：真实Iced UI到GPU纹理的公开接法、线程/借用/输入法锚点，T-02先取证；需要扩patch/接口面则回new，不先改生产。
3. **资源策略**：T-02冻结目标格式/alpha、可调资源预算/诊断，覆盖实际样本与边界；不把测试深度当支持上限。
4. **生产前置**：迁移后的实际SDK/runtime/protocol与owner/冻结收据未齐，留在后继生产集成合同，不从在途文件猜接口。
5. **（已于 2026-10-22 解决）OS 级中文 IME 证据** —— 由 `surface-ime-os` 用例取证（7/7 绿，CJK commit 实测），原三条 unblock 选项作废。以下保留原始记录供追溯：
   ~~**（当前唯一阻塞，2026-10-22 work）OS 级中文 IME 证据**：AC-05 要求"至少一个真实 OS 中文 IME 组合、候选定位"的屏幕级证据。本会话已完成：IME 事件通道（Opened/Preedit/Commit/Closed → 真实 `text_input`）、preedit 不写值/commit 生效/不串写、锚点沿挂载链投影到窗口并落在平面包围盒内、`ImmSetCompositionWindow` 调用与 `client_to_screen` 换算。**未取得**：真实中文输入法（如微软拼音）候选窗跟随光标的实际屏幕证据。阻塞原因=非交互式会话，且切换系统键盘布局/输入法属于影响用户桌面的状态变更，未经用户同意不宜执行。
   **精确 unblock（任选一，用户裁定）**：
   (a) 用户在有中文 IME 的交互式会话中运行 `surface-input --mode window`（或 `surface-offscreen --mode window`）并人工激活 IME 输入一次中文，把截图/录屏附到 `docs/plans/attachments/830/ime-os-evidence/`；work 据此把 AC-05 标绿并置 execution_done；
   (b) 用户明确授权本会话切换输入法布局并注入组合（会影响当前桌面输入状态）；
   (c) 用户裁定把 AC-05 的 OS-IME 部分拆为后继专项 Plan（§5.6/§10 记录在案），本 Plan 以"逻辑面已证 + OS 面显式 pending"收官，由 review 按该裁定评估。
