---
plan_id: PLAN-534
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: VM overlay/表单组件族补齐（widgets-gallery 双端 parity）
author: [zhaopuming, ZCode]
created_at: 2026-09-03
updated_at: 2026-09-03T17:00:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 0
---

# [PLAN-534] VM overlay/表单组件族补齐（widgets-gallery 双端 parity）

## 变更摘要

承接 PLAN-528（widgets-gallery 检查跟踪）遗留的两个已定位、被阻塞的实现项，
并顺势覆盖同根因的 overlay 家族：VM 端 `alert-dialog`/`dialog`/`sheet`/
`drawer` 等 overlay 族目前整体 fallback "renders as Column"（内容内联文档流、
无悬浮层），`toggle_group` 族同样缺 VM 映射（纵向裸排）。本计划以 Plan 422
Popover 锚定弹层原语为底座，配 PLAN-528 W9 的默认面板 chrome 成果，把
overlay 家族 VM 端真悬浮化，并补齐 toggle_group 的横排按钮组映射。

**前置依赖：PLAN-530**（VM shell 绘制一致性 + 崩溃专项）完成——overlay
悬浮层与 toggle_group 的正确渲染依赖绘制/树一致性先行修复。

## 目标

1. **W12 承接**：VM 端 `toggle_group`/`toggle_group_item` 映射为横排
   joined 按钮组（消费 variant/size/aria-label；single/multiple 语义），
   gallery `/togglegroup` 页双端观感对齐（连体三连按钮）。
2. **W13 承接**：VM 端 `alert-dialog` 按 Popover 原语实现真悬浮：点击
   trigger 弹出**居中**模态面板 + 全屏半透明遮罩 + 默认面板 chrome
   （复用 W9 成果），action/cancel 按钮 onclick 可用，点遮罩不可关
   （shadcn AlertDialog 语义）。
3. **同族扩展**（分档，按余力取舍）：`dialog`/`sheet`/`drawer`/
   `hovercard` 同为 fallback 家族，alert-dialog 跑通后按同一模式批量
   迁移（dialog 居中模态、sheet/drawer 侧滑、hovercard 悬浮锚定）。

## 架构方案

- 底座：`crates/auto-lang/src/ui/iced/popover.rs`（Plan 422 锚定弹层
  原语——overlay 机制、定位/翻转/防撞、dismiss 捕获语义已在 W9 打磨）。
- 构建层：`crates/auto-lang/src/ui/aura_view_builder.rs` 增加
  `convert_alert_dialog`（对齐 `convert_popover` 三形态先例）：
  trigger/content 子标签拆解、自管开合注册表（`__ad_toggle` 同
  `__popover_toggle` 型）、placement=居中（新 PopoverPlacement::Center
  或坐标锚窗口中心）、遮罩层（基础树首个全屏半透明容器,open 时插位）。
- chrome：面板缺省样式复用 W9 约定 `w-[max] bg-popover border
  rounded-lg shadow-lg p-6`（alert-dialog 用 lg/宽版,与 W9 的 w-72 区分）。
- toggle_group：`render_support.rs` TagSupport + 动态渲染路径增
  toggle_group 映射——横排 Row + 相邻边框叠压（对齐 W8 vue 侧连体方案），
  variant/size 透传按钮 preset。
- 不动 vue 轨（W7/W8 已就绪）；schema 层 W7 已补全,零 schema 改动。

## 需求分析与背景调查

- 源条目：PLAN-528 W12/W13（实现路径笔记在案）、W9（popover 默认 chrome
  + 居中/翻转/防撞全套,直接复用）、W10/W11（绘制一致性→PLAN-530 前置）。
- VM 现状：`render_support.rs:255` overlay 家族 fallback "renders as
  Column";`toggle_group` 族 TagSupport 缺失。
- Plan 422 三形态（坐标锚/widget 锚/shadcn 嵌套）与自管开合注册表
  （slot id 按构建路径键）均为成熟模式,alert-dialog 直接套用。
- 遮罩先例：Plan 412 Stack 层序（toast 窗口级悬浮层）可参考;基础树
  捕获语义（点遮罩不关 = alert-dialog 特有,与 popover dismiss 相反）。

## 详细设计

（执行时按 T 粒度展开,当前为立项框架。关键决策点：）
1. 居中放置：PopoverPlacement 增 Center 变体（锚=窗口中心,零尺寸）vs
   复用坐标锚 at_point(窗口中心)——倾向前者（语义清晰,snap 逻辑复用）。
2. 遮罩实现：View::Popover 外包全屏半透明 Column（bg-black/80,open 时
   才挂载）vs iced overlay Group 双元素——倾向前者（树结构简单,MCP 可见）。
3. toggle_group 连体：VM 侧按钮 preset variant=outline 已有,连体用
   W8 同款思路（相邻 -ml-px + 首尾圆角）在 VM 布局层的表达。
4. 批量迁移分档：P1 alert-dialog（用户直接需求）→ P2 toggle_group →
   P3 dialog/sheet/drawer/hovercard（同模式复制,P3 可裁剪到下期）。

## 测试设计

- 单元：convert_alert_dialog 三形态拆解断言（trigger/content/无嵌套）;
  居中 placement 布局断言;toggle_group 映射 TagSupport 断言。
- 集成：`cargo t --features test-vm-files` 档补 VM 文件测试样本
  （alert-dialog 开合/action onclick/toggle_group 单选多选）。
- 双端验证：autoui-verifier MCP 驱动——/togglegroup、/alertdialog 页
  VM 截图对齐 vue 基线（W7/W8 截图在案）;全站 69 页 VM 扫描回归。

## 验收标准

- [ ] `/togglegroup` VM 页：横排连体 B/I/U（outline）+ single 示例可交互,
      与 vue 版（W8 截图）观感一致。
- [ ] `/alertdialog` VM 页：点 Show Dialog 弹出居中面板+遮罩;Continue/
      Cancel 触发 onclick（W1 的 toast handler 在 VM 端 toast 层可见）;
      点遮罩不关闭（模态语义正确）。
- [ ] P3 若执行：dialog/sheet/drawer/hovercard 各自页 VM 端悬浮正确。
- [ ] 全站 69 页 VM 扫描无新增异常;`cargo t iced` + VM 文件测试绿。

## 执行步骤

（PLAN-530 完成后展开为原子任务;当前立项目录:）

1. [ ] P1-alert-dialog：convert_alert_dialog + 居中放置 + 遮罩 + chrome
2. [ ] P2-toggle_group：VM 映射 + 连体布局 + variant/size
3. [ ] P3-overlay 家族批量迁移（dialog/sheet/drawer/hovercard,可裁剪）
4. [ ] 双端截图对齐 + 全站扫描回归

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

1. 遮罩点击"不关闭"但 Esc 关闭（shadcn 语义）——VM dismiss 通道需
   区分来源,是否 P1 就做 Esc 还是降级记录。
2. P3 范围伸缩：若 530 修复周期长,P3 是否拆独立计划另排。
3. toggle_group 的 VM 端 v-model（value 状态回写）语义与 vue 轨对齐
   细节（single 返回 str vs multiple 返回 List）。
