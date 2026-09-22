---
plan_id: PLAN-683
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: rq-remote-renderer
author: [zcode]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 desktop-protocol-v1 DisplayList v2（渲染原语 display-list 追加式）, SD-02 ui overview desktop_render=remote 三态渲染模式]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design/autoui/desktop-protocol-v1.md, docs/design/autoui/rq-remote-renderer.md, docs/specs/auto-lang/ui/overview.md]
current_step: 0
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

- [ ] AC-01 001-helloworld `desktop_render: remote` 渲染完整（标题文本/
      主题底），与独立轨视觉对照走查通过（截图在档）。
- [ ] AC-02 003-converter remote 模式交互闭环：点击按钮/输入聚焦/键入联动
      （经事件回路径，端到端帧更新）。
- [ ] AC-03 004-profile-card remote 模式：渐变头带/圆角/文本换行/头像由
      iced 构造保证（与独立轨对照一致）。
- [ ] AC-04 DisplayList v2 codec round-trip golden + v1 帧兼容解码测试绿。
- [ ] AC-05 既有门禁零新增红（desktop_protocol/fit/typing 全套 + 裸
      `cargo t` 基线持平）。
- [ ] AC-06 RqProjector 冻结清单与 DrawList v1 legacy 兼容位文档在档
      （specs 沉淀随 T-06）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] T-00 可行性 spike（bounded investigation，决策档）：headless
      UserInterface + 段落引擎无 wgpu 可用性勘定；RecordRenderer 最小面
      （quad/text/layer）录制 001 首帧 → paint_ops 回放；渐变/Path 原语
      记录形态二选一。产出 go/no-go 决策档（红 = 本计划终结 + 方案 3
      StyleSpec 兜底立项建议）。验证：001 首帧回放成功 + 决策档入库。
      [关联 AC-01/04]
- [ ] T-01 RecordRenderer 完全面（quad/text styled/layers/transform/image
      引用）+ DisplayList v2 codec + round-trip golden。[关联 AC-04]
      验证：codec 测试绿。
- [ ] T-02 headless 宿主（帧循环/事件注入/IME 组合态）+ typing 环 remote
      变体。[关联 AC-02] 验证：003 键入联动端到端绿。
- [ ] T-03 daemon 重放扩展（paint_ops v2 原语）+ per-window 接入。[关联
      AC-01] 验证：001 remote 实机窗渲染。
- [ ] T-04 试点三例（001/003/004）`desktop_render: remote` + 对照走查
      （AC-01/02/03 证据采集）。
- [ ] T-05 门禁全量回归 + 试点问题清零。[关联 AC-05]
- [ ] T-06 复审收口：specs 沉淀（SD-01/02 落表）、RqProjector 冻结清单、
      Design 状态更新。[关联 AC-06]

## 复审记录

- draft 交付（2026-09-22）：stage=new，PLAN-683 rev1。outcome=pass（T-00
  spike 为内嵌 go/no-go 门，红则按目标 G1 决策档终结——该终结路径已在本契
  约内授权）。next=work。

## 待澄清事项

1. 文本通道二选一（字体下发 vs 字形栅格过线）——T-01 勘定后定，默认字体
   下发（wire 瘦）。
2. 渐变 Background::Gradient 的记录形态（原生 stop vs 条带展开）——T-01 勘定。
3. Windows 中文 IME 组合态经事件回路径的完备性——T-02 重点验证；若残缺，
   混合方案（text_input 类交互保留 daemon 侧原生组件）作为兜底登记。
