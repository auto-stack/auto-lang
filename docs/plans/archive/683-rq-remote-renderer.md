---
plan_id: PLAN-683
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: rq-remote-renderer
author: [zcode]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 desktop-protocol-v1 DisplayList v2（渲染原语 display-list 追加式）, SD-02 ui overview desktop_render 四值渲染模式（+remote，review r1 F-4 勘正）]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design/autoui/desktop-protocol-v1.md, docs/design/autoui/rq-remote-renderer.md, docs/specs/auto-lang/ui/overview.md]
current_step: 6
total_steps: 6
---

# [PLAN-683] rq-remote-renderer——方案 2：iced 组件照常渲染，渲染原语过 RenderQueue

> 来源：2026-09-22 四方案架构评审收敛（用户裁定现阶段走方案 2）。设计依据
> [rq-remote-renderer](../design/autoui/rq-remote-renderer.md)（含四方案对比、
> 方案 4 否决记录、方案 3 未来选项与重估触发条件）。本轮直接驱动案例：004
> 双轨对拍差距（PLAN-679 Phase 1/2 已修 v1 wire 内可修项；剩余差距——头像圆
> 裁/真渐变/hover 态/组件覆盖爬坡——均属「手写投影器」结构性成本，本计划以
> 方案 2 构造性消除）。

## 变更摘要

RQ 轨命令生产者从「手写投影器（RqProjector）」替换为「iced 组件树照常渲染 +
RecordRenderer 原语截获」：AuraView → iced 组件树照常构建（AuraViewBuilder
不变）→ **iced 自己布局/命中/聚焦/IME**（headless UserInterface 驱动，无窗
口无 GPU）→ RecordRenderer 把渲染原语序列化为 **DisplayList v2** → 管道 →
RQHost 重放绘制（paint_ops 扩展）。RqProjector 冻结退役路径在册；DrawList
v1 保留为 legacy 兼容 wire。

## 目标

- **G1（可行性）**：headless 驱动可行性勘定——`UserInterface` + 段落引擎
  （cosmic-text）在无 wgpu feature 下可用；RecordRenderer 录制 001 首帧并经
  现有 paint_ops 回放成功。产出决策档（go/no-go；红 = 回退方案 3 续航 +
  StyleSpec 单源兜底，本计划终结）。
- **G2（wire v2）**：DisplayList v2 codec（v1 原语超集：quad/quad-r/text
  styled/image/scissor/layer-clip/transformation），round-trip golden。
- **G3（headless 宿主）**：远程事件回路径（daemon 捕获 → iced::Event 注入
  `UserInterface::update`）+ 帧循环（revision 门控）。
- **G4（试点）**：001/003/004 三例 `desktop_render: remote` 实机渲染 + 交互
  闭环（点击/键入/IME），与独立轨视觉对照（同构截图走查）。
- **G5（迁移与退役）**：RqProjector 冻结清单 + DrawList v1 legacy 兼容位 +
  coverage 门禁作用域收缩（legacy 模式限定）。

**非目标**：不实现方案 3（自有渲染引擎——未来选项，触发条件见设计文档 §5）；
不动 VM 独立轨与 a2r 轨；不做跨机远程（v1 管道为本机）；不新增视觉能力
（parity 为纲——iced 有什么渲染什么）。

**成功标准**：三试点 app 在 remote 模式下渲染与交互完整、与独立轨观感一致
（同源 iced 组件树构造保证），且既有门禁零新增红。

## 架构方案

见设计文档 [rq-remote-renderer](../design/autoui/rq-remote-renderer.md) §4。
要点：**不走自定义 `iced::daemon` Compositor**（高风险面），改走
`iced_runtime::user_interface::UserInterface` 公开 API headless 直驱
（iced_test 仿真同一驱动方式）：view → layout(Size) → draw(RecordRenderer)
三步；输入 = 远端事件转 `iced::Event` 注入 `update`。RecordRenderer 实现
`iced_core::Renderer` + 文本子 trait（App 内 cosmic-text 度量整形，字体已
内嵌，CPU；无 wgpu 面——内存论点保住）。

## 需求分析与背景调查

- **授权记录**：用户 2026-09-22 裁定「现阶段方案 2 更好（不用重写 iced 渲染
  逻辑）……写一个新的计划，同时更新设计文档（方案 2 和 3 区别说清楚，方案 3
  未来仍可走）」。四方案评审过程与用户四条反对（带宽/daemon 单体化/崩溃/
  头重脚轻）全文记录于设计文档 §1-§2。
- **代码实勘**（本会话完成）：
  - `client_entry.rs:85/104/245`——现 RQ 命令生产者 = RqProjector（本计划
    替换对象）；`native_projector.rs` ~2300 行手写块流+命令发射（冻结对象）；
  - `broker_surface.rs::paint_ops`——daemon 重放画笔（扩展基底，QuadR 已入）；
  - `rqhost.rs:656 run_vm_rqhost_client` / `client_entry::run_dynamic_client`
    ——装配点（新增 remote 分岔位）；
  - `message.rs` DrawList codec（v1；v2 为追加式超集）;
  - `iced 0.14`：`iced::daemon` 泛型 `Renderer: program::Renderer`（官方
    wgpu/tiny_skia 后端即第三方实现先例）；`UserInterface` 公开
    （iced_runtime::user_interface）。
- **Spec 现状**：`docs/specs/auto-lang/ui/overview.md` desktop_render 二态
  （auto/queue/independent + DrawList v1）——本计划增补 remote 三态与
  DisplayList v2（见详细设计规范增量）。

## 详细设计

### RecordRenderer（原语截获）

实现 `iced_core::Renderer` 全表面 → DisplayList v2 记录：

| iced Renderer 原语 | v2 记录 |
|---|---|
| fill_quad(Quad, Background) | Quad/QuadR（radius 语义同 v1 tag 7；Background::Color → 色；渐变 Background::Gradient → 原生 stop 记录或条带展开——T-01 勘定） |
| 文本（text::Renderer/Paragraph） | Text/TextStyled runs（App 内 cosmic 整形定行盒，字体=内嵌 Inter 族+回退声明） |
| with_layer/start_layer | Layer push/pop（clip bounds） |
| with_transformation/translation | Transform push/pop（矩阵） |
| image::Renderer allocate/Upload | Image 引用（URL/bytes 句柄——沿 v1 by-ref 纪律） |
| fill_circle/Path（canvas 场景） | Path 序列化或位图降级（T-01 勘定——v1 先 bitmap 降级） |

文本整形定界：App 内 cosmic-text 完成整形与行盒（UserInterface::layout 需要真
度量），daemon 只按行盒绘制字形——字体字节随 Hello/BufferAlloc 握手下发
（FontBlob 通道已存在，413 §7.2）或 App 内栅格化字形纹理过线（T-01 勘定二
选一：字形过线=像素精确，字体下发=wire 更瘦；默认后者，观感差登记）。

### headless 宿主与事件回路径

- 帧循环：revision 门控（state 变更 → view() → UI::layout(Size) →
  UI::draw(RecordRenderer) → serialize → push）；尺寸来源 =
  pac window/fit（fit 语义沿用现有测量回收臂）；
- 输入：daemon 捕获窗口事件 → wire 上行 → App 转 `iced::Event`
  （Pointer/Keyboard/IME/Window）→ `UserInterface::update` → iced 分发进
  widget 树（命中/聚焦/IME 全原生）→ 状态变更 → 下一帧；
- IME：App 侧 cosmic 组合态（daemon 只转译原始键/组合串；Windows 中文 IME
  组合行为为 T-02 重点验证项，风险登记）。

### RqProjector 退役路径

试点验收后：`desktop_render` 缺省翻转 remote（coverage 概念对 remote 模式
失效——组件覆盖=iced 全集）；RqProjector + DrawList v1 代码冻结保留（legacy
客户端兼容），删除与否留待方案 3 重估时一并裁定。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/design/autoui/desktop-protocol-v1.md | 帧通道仅 DrawList v1 → 增补 DisplayList v2（原语超集：layer/transform/quad-r/font runs）与双版本共存规则 | 方案 2 wire 载体 | AC-04 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | desktop_render 二态（auto/queue/independent）→ 三态（+remote）；remote 模式下组件覆盖=iced 全集 | 方案 2 渲染模式 | AC-01 |

## 测试设计

- codec round-trip golden（DisplayList v2 全原语 + 旧 v1 帧兼容解码）；
- headless 宿主单测：事件注入 → 状态变更 → 帧序列递增（typing 环 remote 变体）；
- 试点三例双轨对照：remote 帧与独立轨 wgpu 帧做**结构对照**（widget 锚点/
  相对几何——跨光栅化像素 diff 不可行，登记为近似验证；同源构造保证为主证据）；
- 既有门禁零新增红（desktop_protocol/fit/typing 全套）。

## 验收标准

- [x] AC-01 001-helloworld `desktop_render: remote` 渲染完整（标题文本/
      主题底），与独立轨视觉对照走查通过（截图在档）。
      [✅] 走查 2026-09-22：深底 #090E1A（theme 单源）+ 蓝紫
      "Hello, World!" text-4xl；截图 p683-shots/001-remote.png。
- [x] AC-02 003-converter remote 模式交互闭环：点击按钮/输入聚焦/键入联动
      （经事件回路径，端到端帧更新）。
      [✅] 双证：单测 p683_t02_typing_remote_v2_loop（进程内 pipe 环）+
      实机真鼠标/键盘（revision 1→2→3；Celsius 1000/Fahrenheit 1832
      联动 + iced 原生 focus 蓝环渲染）；截图 003-remote-{initial,typed}.png。
- [x] AC-03 004-profile-card remote 模式：渐变头带/圆角/文本换行/头像由
      iced 构造保证（与独立轨对照一致）。
      [✅] 走查：渐变头带 blue-500→purple-600 原生重放 + rounded-t-lg
      圆角衔接 + 圆头像 + 全文本（Jane Cooper/bio/Follow/Message）；
      截图 004-remote.png。**PLAN-679 遗留差距（头像圆裁/真渐变）构造性
      消除实证**。
- [x] AC-04 DisplayList v2 codec round-trip golden + v1 帧兼容解码测试绿。
      [✅] display_list_v2_round_trip_and_golden（全原语+字节锚点+未知 tag
      拒收）+ display_list_v2_v1_compat_and_lift（v1 tag 互斥+from_v1 语义
      保真）+ FrameReadyV2 信封 round-trip；v1 message 17/17 零扰动。
- [x] AC-05 既有门禁零新增红（desktop_protocol/fit/typing 全套 + 裸
      `cargo t` 基线持平）。
      [✅] 裸 t 5438 测 9 红全预存归因（musk×6+counter+shell_pack=
      5c2ce28cb/P672 在案；dock_pager=985c5b944 在案+master 版归属实验
      佐证）——零新增。
- [x] AC-06 RqProjector 冻结清单与 DrawList v1 legacy 兼容位文档在档
      （specs 沉淀随 T-06）。
      [✅] 设计档 §6.1 四行表（native_projector 冻结/DrawList v1 线格式
      冻结/coverage 作用域收缩/679 修复对 v1 有效）+ SD-01/02 落表。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] T-00 可行性 spike（bounded investigation，决策档）：headless
      UserInterface + 段落引擎无 wgpu 可用性勘定；RecordRenderer 最小面
      （quad/text/layer）录制 001 首帧 → paint_ops 回放；渐变/Path 原语
      记录形态二选一。产出 go/no-go 决策档（红 = 本计划终结 + 方案 3
      StyleSpec 兜底立项建议）。验证：001 首帧回放成功 + 决策档入库。
      [关联 AC-01/04]
      [✅ 已完成] **GO**（df28f10ad + eeef08b2b @ plan-683-dev）。
      关键架构发现：自研 RecordRenderer 不可行（into_iced() 组件树构造期
      钉死 iced::Renderer 类型），改用 `iced::Renderer::Secondary(
      iced_tiny_skia::Renderer)` 公开枚举臂——纯 CPU 构造、类型同源直驱
      UserInterface，Layer 记录层即截获面（`headless.rs` 降格层 ~470 行）。
      证据：`p683_t00_helloworld/converter_headless_first_frame` 双绿
      （001 首帧=1×TextStyled "Hello, World!" + clear #090E1A 单源深底，
      wire 51B，round-trip 恒等，paint_ops `()` 后端回放零异常；单测
      <0.13s）。决策档 `docs/plans/reports/p683-t00-spike-go.md`。
      渐变=v1 停点均值近似+观测行（v2 原生 T-01）；mesh=位图降级 T-01。
- [x] T-01 RecordRenderer 完全面（quad/text styled/layers/transform/image
      引用）+ DisplayList v2 codec + round-trip golden。[关联 AC-04]
      验证：codec 测试绿。
      [✅ 已完成]（0e0b82138 @ plan-683-dev）。v2 载荷 tag 2 追加式超集：
      Quad 全参（Fill 渐变 angle+stops 原生/四角半径/border/shadow）+
      Transform 2×3 + FrameReadyV2 信封 tag 12 + from_v1 直搬 + v1 tag
      互斥。headless `render_frame_v2` 原生降格（图像句柄三态 → src/
      bitmap:// 上传通道）。golden 三测绿（全原语 round-trip+字节锚点+
      v1 兼容 lift+001/003 双降格文本面等价），v1 message 17/17 + shm
      24/24 零扰动。**待澄清①②裁定落档**：文本=字体下发（FontBlob
      通道）；渐变=原生 stop（angle 模型与 iced/canvas 同构，非条带展开）。
      登记债：P683-D1 mesh 位图降级 / P683-D2 旋转矩阵元 iced 未公开
      （transform 近似 scale+translate）/ P683-D3 svg 通道。
- [x] T-02 headless 宿主（帧循环/事件注入/IME 组合态）+ typing 环 remote
      变体。[关联 AC-02] 验证：003 键入联动端到端绿。
      [✅ 已完成→review r1 重开→修复复勾（修复批 commit：F-1 import 清除
      后 check 零新增警告 + p683/display_list 8/8 复绿；F-2 债 P683-D6
      在册；F-3 缺省翻转=merge 裁定呈报项非修项）]
      [✅ 已完成]（a08c625d9 @ plan-683-dev，rebase 过 master 01c3164e4
      零冲突）。HeadlessFrameSource（FrameSource 全缝：InputMsg→
      iced::Event 全族译码含 IME 三态、光标同步、双 tick 源、位图/命令
      排水、revision 门控）+ ClientPump v2 位 + HostAction::
      ComposeFrameV2 三宿主臂 + RenderMode::Remote 三态 + 双轨 remote
      分岔。**根修一枚**：update() 事件消息漏回灌组件（键入联动假死）
      ——修后端到端绿：`p683_t02_typing_remote_v2_loop`（003 点击聚焦→
      键入 100→fahrenheit 1832 联动 v2 帧 + revision 前进）。v1 typing
      环不受扰。Windows 中文 IME 实机完备性留 T-04 走查（待澄清③
      映射已接 input_method::Event 三态）。
- [x] T-03 daemon 重放扩展（paint_ops v2 原语）+ per-window 接入。[关联
      AC-01] 验证：001 remote 实机窗渲染。
      [✅ 已完成]（f7be566f8 @ plan-683-dev）。DisplayListPainter +
      `paint_ops_v2` 全原语 canvas 重放（渐变 to_distance 重建/四角半径/
      border 内描边/Transform 压出栈配对扫描/shadow 垫层近似）+
      `rq_view`/`broker_client_content` v2 合成面优先路由（v1 窗零扰）+
      pac 档链贯通（`desktop_render: remote` → AUTO_VM_RENDER →
      run_vm_rqhost_client 分岔；cmd_autodesk 386 臂同缝）。单测：
      `p683_t03_v2_paint_replay`（001/003 实帧 `()` 后端全原语零炸）；
      broker_surface 9/9 + v1 typing 环绿。shadow blur 无 canvas 原语
      → 垫层近似登记 **P683-D4**。实机窗渲染归 T-04 走查（本任务单测
      面=降格路径，实机=AC-01 证据采集位）。
- [x] T-04 试点三例（001/003/004）`desktop_render: remote` + 对照走查
      （AC-01/02/03 证据采集）。
      [✅ 已完成]（@ plan-683-dev）。三例 pac 落 remote 档 + 实机窗走查
      全过（截图 `docs/plans/reports/p683-shots/` 四张在档）：
      **AC-01** 001 = 深底 #090E1A + 蓝紫 "Hello, World!"（text-4xl）；
      **AC-02** 003 = 真鼠标点击聚焦（iced 原生 focus 蓝环）+ 真键盘
      键入 100 → Celsius 1000 / Fahrenheit 1832 联动（revision 1→2→3
      前进）；**AC-03** 004 = 渐变头带 blue-500→purple-600 原生重放 +
      rounded-t-lg 圆角衔接 + 圆头像 + 全文本。daemon 瘦宿主实测
      private 12MB / working set 43MB。**PLAN-679 遗留差距中头像圆裁/
      真渐变两项构造性消除实证**。走查环境注记：本机他方窗口争焦会
      最小化 daemon 窗（环境怪非本路径缺陷——SW_RESTORE 即复）。
- [x] T-05 门禁全量回归 + 试点问题清零。[关联 AC-05]
      [✅ 已完成]（@ plan-683-dev，收据 commit 1554890b6）。作用域套件：
      desktop_protocol 200/201（唯一红 = master 在案 counter×1）、
      ui::iced 238/239（唯一红 = master 在案 dock_pager〔985c5b944 注记
      疑 608 引入〕——**归属实验**：master 版 renderer.rs 替换后仍红，
      与本 diff 零交集）、fit 20/20、typing 3/3 全绿。裸 `cargo t`
      5438 测 **9 红全预存归因**（musk p053/p054×6 + counter×1 =
      5c2ce28cb 普查在案；shell_pack×1 = P672 册在案；dock_pager×1 =
      985c5b944 在案）——**PLAN-683 零新增红**。
- [x] T-06 复审收口：specs 沉淀（SD-01/02 落表）、RqProjector 冻结清单、
      Design 状态更新。[关联 AC-06]
      [✅ 已完成]（1554890b6 @ plan-683-dev）。SD-01 = desktop-protocol-v1
      增量表新增「帧载荷 v2」行；SD-02 = 协议表三态行升四值 + specs
      overview.md remote 段（覆盖门不适用构造保证 + 冻结位指针）；
      设计档 rq-remote-renderer 状态翻「已实施」+ §6.1 RqProjector 冻结
      清单四行表；债册 P683-D1..D5 在册（mesh 词汇外/旋转矩阵/svg/
      shadow 垫层/daemon 窗最小化环境怪）。

## 复审记录

- draft 交付（2026-09-22）：stage=new，PLAN-683 rev1。outcome=pass（T-00
  spike 为内嵌 go/no-go 门，红则按目标 G1 决策档终结——该终结路径已在本契
  约内授权）。next=work。
- merge 收口（2026-09-22，PLAN-683:r1）：stage=merge | outcome=pass | delivery_commit=c553ed402a1b（rebase 映射：01c3164e4 基 8 commit〔e0cd21cc8→9bc685826/…/7a92a96b4→c4875741b 等全程 range-diff 全等〕+投影 commit 142222276→c553ed402；master 因计划簿记 d5fcee0a 前移二步 rebase，9/9 全等）| canonical=docs/design/autoui/desktop-protocol-v1.md（SD-01 行）+docs/specs/auto-lang/ui/overview.md（SD-02 段）+docs/design/autoui/rq-remote-renderer.md（§6.1 冻结清单）| ledger=.autoos/specs.json P683-1（designs）/P683-2（reviews）+ui/plans.md 683 行+INDEX 再生（readback 验证）| checkpoints：prepared✓/landed✓（ff-only，master tip=delivery；主检出 p683 smoke 5/5）/ledger_refreshed✓/archived✓（git mv + status archived）/cleaned✓（双仓 worktree 全清：auto-lang+auto-down 侧均 prune；plan-683-dev 已删；组目录 lang-683 零残留；他方 WIP〔P679-D1 bisect 调试残留 001 pac.at+003 app.at〕patch 保全 .wt/foreign-wip-683-merge-letway.patch 并已弹回主检出原状） | completion_kind: delivered | F-3 呈报：desktop_render 缺省未翻 remote（保守解释在案 §6.1——翻转需独立爬坡数据门另立）。
- review r2 终判（2026-09-22）：stage=review | PLAN-683 rev1 | outcome=**pass** | reviewed_commit=7a92a96b430ee09b5981d3d4a081a59473d4ae86（=1554890b6 + 修复批 2 文件 2+/1-，语义面零变化）| base_commit=01c3164e4 | acceptance=AC-01..06 全 pass（r1 全量复验证据沿用明示理由：修复批仅 1 行 import 移除+纯文档，代码语义/依赖/测试配置零变化；p683/display_list 8/8 修复后复绿在案）| findings=F-1 清（check 零新增警告）/F-2 清（P683-D6 在册）/F-4 清（frontmatter 勘正）；F-3（缺省翻转保守解释）非修项随 merge 呈报 | spec delta 终版=SD-01（desktop-protocol-v1.md 帧载荷 v2 行）+SD-02（协议表四值行+overview.md remote 段），描述现行行为与持久决策，无废弃施工日记残留 | supersedes_spec_components=[]（v1 线格式冻结不退役，追加式共存）/new_spec_components=[SD-01,SD-02 四值勘正版]/touched_goals=[]（goals.md 无本域 GOAL 锚——UI 协议面无在册 goal 条目，留空有据）| next=merge（F-3 缺省翻转裁定项随 merge 呈报用户；master 已前移至 8360bc583，merge 时先对齐）。
- review r1（2026-09-22，同会话复审——独立性声明：以产物重建裁决，重跑全部验证直读断言，不以执行者自述为凭）：stage=review | PLAN-683 rev1 | outcome=**needs_fix** | reviewed_commit=1554890b655ad5ead5edd443dfceb99ff5028f1f | base_commit=01c3164e4 | deps=iced_runtime 0.14/iced_tiny_skia 0.14（+auto-down 组内 fba6563e） | spec_inputs=desktop-protocol-v1.md@1554890b6/ui overview.md@1554890b6 | acceptance=AC-01..06 全 pass（AC-04 golden 重跑 8/8+负例 0x7F 拒收+字节锚直读；AC-05 裸 t 9 红全预存在案〔musk×6+counter=5c2ce28cb/shell_pack=P672/dock_pager=985c5b944+master 版 renderer.rs 归属实验〕+tf 5439 测 10 红=同 9+ffi_dual_019 单跑即绿=满载并行 flake 非回归；AC-01/03 截图 md5 与视觉验证件字节同） | findings=**F-1**（trivial，health）：headless.rs:543 未用 import MouseButton——本计划唯一新增警告，违零未处理警告门 → T-02 重开；**F-2**（low）：IME 组合态实机验证未执行（待澄清③半开：input_method 三态映射已接+ASCII 键入端到端绿，中文 IME 实机组合未测；无 AC 覆盖）→ 登记 P683-D6；**F-4**（trivial）：frontmatter SD-02 描述"三态"陈词应改"四值（+remote）"；**F-3**（observation，非修项）：desktop_render 缺省未翻 remote——计划正文"试点验收后翻转"与任务清单（无翻转任务/AC）张力，取保守解释（翻转=独立爬坡复核面，032 先例数据门）已在设计档 §6.1 显式注记，呈报 merge/用户裁定 | next=work 修 F-1/F-2/F-4（单 commit）→ review r2。
- work 交付（2026-09-22）：stage=work | PLAN-683 rev1 | outcome=pass |
  code_commit=plan-683-dev @ 1554890b6（df28f10ad T-00 / 0e0b82138 T-01 /
  a08c625d9 T-02〔rebase 过 master 01c3164e4〕/ f7be566f8 T-03 / T-04 试点
  commit / 1554890b6 T-05/06）| task_ids=T-00..T-06 全勾 + AC-01..06 全勾 |
  evidence=T-00 GO 决策档（tiny_skia Secondary 截获面架构发现）+ golden
  三测 + typing remote 双证（单测+实机）+ 三试点走查截图四张 + 裸 t 9 红
  全预存归因零新增 | blockers=无 | next=review。执行注记三枚：①自研
  RecordRenderer 因 into_iced() 类型钉死不可行——tiny_skia Secondary 公开
  枚举臂为等价截获面（设计档已更正）；②update() 事件消息漏回灌根修
  （键入联动假死）；③master 双预存红（counter/dock_pager）经 master 版
  renderer.rs 归属实验与在案注记双重佐证非本计划引入。

## 待澄清事项

1. 文本通道二选一（字体下发 vs 字形栅格过线）——T-01 勘定后定，默认字体
   下发（wire 瘦）。
2. 渐变 Background::Gradient 的记录形态（原生 stop vs 条带展开）——T-01 勘定。
3. Windows 中文 IME 组合态经事件回路径的完备性——T-02 重点验证；若残缺，
   混合方案（text_input 类交互保留 daemon 侧原生组件）作为兜底登记。
