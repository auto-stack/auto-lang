---
plan_id: PLAN-534
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: VM overlay 家族余量补齐（dialog/sheet/drawer/hovercard）
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

**2026-09-03 重定界**：立项时承接的 PLAN-528 W12（toggle_group VM 映射）与
W13（alert-dialog 模态化）已由 **PLAN-530 在其执行期内一并落地并复审 PASS**
（commit 93d933a62：toggle_group 横排连体消费臂 + alert_dialog 复用 Popover
原语 PopoverPlacement::Modal + scrim）。本计划收敛为 **P3 余量**：overlay
家族剩余成员 `dialog` / `sheet` / `drawer` / `hovercard` 仍为 fallback
"renders as Column"，按 530 已验证的同一模式批量迁移为真悬浮。

**前置依赖 PLAN-530 已解除**（其 status: reviewed）。

## 目标

1. `dialog`：居中模态（复用 PopoverPlacement::Modal + scrim，open 属性/
   trigger 双形态），DialogContent/Header/Footer/Title/Description/Close 子臂。
2. `sheet` / `drawer`：侧滑面板（新增 side placement：left/right/top/bottom
   贴边 + scrim，复用锚定/snap 原语）。
3. `hovercard`：悬浮锚定（Bottom/Top placement + hover 触发语义,现有
   Popover 原语 click 触发需扩展 hover 通道或降级 click）。
4. gallery 对应文档页双端观感对齐 + 全站 69 页 VM 扫描回归。

## 架构方案

底座全部就绪,本计划为模式复制：`crates/auto-lang/src/ui/iced/popover.rs`
（Plan 422 原语 + 530 的 Modal/scrim 扩展）为唯一悬浮通道;
`aura_view_builder.rs` 按成员增 convert 臂（对齐 530 的 convert_alert_dialog
先例:子标签拆解/自管开合注册表/placement 映射）;面板缺省 chrome 沿用
PLAN-528 W9 约定（modal 宽版 lg/shadow-lg,侧滑全高窄版）。sheet/drawer 需
新增贴边 placement 变体（4 向）;hovercard 需 hover 触发通道（或 v1 降级
click 并文档化差异）。

## 需求分析与背景调查

- 源条目：PLAN-528 W12/W13（**已由 PLAN-530 落地**,本计划只承接 P3 余量）、
  W9（popover 默认 chrome + 居中/翻转/防撞全套,直接复用）。
- 530 已交付可直接复用的资产：PopoverPlacement::Modal + scrim + 面板外
  点击整吞、toggle_group 消费臂双层 D-GAP 镜像模式、绘制一致性修复。
- VM 现状：overlay 家族 fallback "renders as Column" 仅剩
  dialog/sheet/drawer/hovercard 四员。
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

1. [ ] P3a-dialog：convert_dialog（Modal 复用）+ Close 子臂 + open/trigger 双形态
2. [ ] P3b-sheet/drawer：贴边 placement 四向 + scrim + 高度语义
3. [ ] P3c-hovercard：hover 触发通道（或 v1 click 降级文档化）
4. [ ] 双端截图对齐 + 全站扫描回归

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

1. 遮罩点击"不关闭"但 Esc 关闭（shadcn 语义）——VM dismiss 通道需
   区分来源,是否 P1 就做 Esc 还是降级记录。
2. P3 范围伸缩：若 530 修复周期长,P3 是否拆独立计划另排。
3. toggle_group 的 VM 端 v-model（value 状态回写）语义与 vue 轨对齐
   细节（single 返回 str vs multiple 返回 List）。
