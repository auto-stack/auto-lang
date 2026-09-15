---
plan_id: PLAN-636
status: drafting
feature_name: klondike-vm-parity
author: [agent]
created_at: 2026-09-15T07:32:00Z
updated_at: 2026-09-15T07:32:00Z
plan_revision: 1
current_step: 0
total_steps: 6
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [auto-os/apps/037-klondike]
---

# [PLAN-636] Klondike VM/Iced 端视觉对齐修复

## 0. 变更摘要

对比 Vue 端与 VM/Iced 端的 037-klondike 截图，发现 VM 端存在 3 个 P0 级布局结构问题和图标渲染缺失。本 Plan 通过修改 `.at` 源文件（`auto-os/apps/037-klondike/src/front/`）将 Iced 不支持的 CSS 写法替换为兼容写法，无需修改 Rust 渲染器。

## 1. 目标

- 修复 VM 端顶栏溢出（`flex-wrap` → 两行固定布局）
- 修复 `card_face.at` 中 `absolute inset-0` 牌面中心区定位丢失
- 修复 `min-h-*` 被忽略导致牌桌列高度压缩
- 修复发牌堆角标 `absolute bottom-1 right-1.5` 定位失效
- 验证 icon/CardSuit 渲染在 VM 截图中的实际状态

**非目标**：不修改 auto-lang Rust 渲染器；不引入新功能；不改变游戏逻辑。

## 2. 架构方案

全部改动在 `auto-os/apps/037-klondike/src/front/` 的 `.at` 源文件内，使用 Iced 兼容的声明式布局替代 CSS absolute 定位和 flex-wrap。改动后需双端截图验证。

## 3. 技术栈

- AutoUI `.at` 源文件（`app.at`, `card_face.at`）
- VM 验证：`auto run -r vm` + MCP Python 截图脚本
- Vue 验证：`auto build -r vue` + Playwright 截图

## 4. 需求分析与背景调查

**已知降级（VM 日志明确报警）**：
- `flex-wrap` → Plan 412 §5 降级，顶栏三组控件溢出
- `inset-N` → Plan 412 §5 降级，`card_face.at` 中心区定位丢失

**截图证据**（2026-09-15 采集）：
- `tests/screenshots/vm_initial.png` — VM 端初始态
- `tests/screenshots/vue_initial.png` — Vue 端初始态
- `tests/screenshots/vm_snapshot.txt` — AURA 树快照

**已授权范围**：用户明确要求修复 VM 端视觉差距，涉及 `auto-os/apps/037-klondike` `.at` 源文件修改。

## 5. 详细设计

### 各问题修复方案

**P0-1: 顶栏 flex-wrap → 两行布局**
`app.at` 顶栏外层 `row { style: "... flex-wrap gap-4" }` 改为 `col`，内部按两行 row 排列：
- 第一行：logo + 皮肤切换
- 第二行：状态指标 + 操作按钮

**P0-2: card_face.at inset-0 → 非定位居中**
将中心区：
```
col { style: "absolute inset-0 items-center justify-center ..." }
```
改为不依赖 absolute 的写法，把整个牌面改为三段式 `col { justify-between }`，中段用 `col { style: "flex-1 items-center justify-center" }` 撑开。

**P0-3: min-h 牌桌列高度 → 固定 h**
将 `min-h-[340px]` 改为 `h-[340px]`，`min-h-screen` 改为 `h-screen`（顶层容器）。

**P1-5: 发牌堆角标 absolute → row 末追加**
将 `text .store.stock_count_label { style: "absolute bottom-1 right-1.5 ..." }` 改为 col 末行追加，使用 `justify-end items-end` 对齐。

### 规范增量

| delta_id | 类型 | 目标 | 说明 | AC |
|---|---|---|---|---|
| SD-01 | modify | auto-os/apps/037-klondike | `.at` 布局兼容性改写 | AC-01~AC-04 |

## 6. 测试设计

每个任务完成后运行 VM 截图脚本，与 Vue 截图对比。无需运行 `cargo t`（未修改 Rust 源码，Category A 任务）。

## 7. 验收标准

| ID | 验收项 | 验证方法 |
|---|---|---|
| AC-01 | VM 端顶栏三组控件完整显示，无横向溢出或截断 | VM 截图目视对比 |
| AC-02 | VM 端卡牌面中心花色/人头字居中显示（A 牌大花色、J/Q/K 艺术字） | VM 截图目视对比 |
| AC-03 | VM 端 7 列牌桌列高度不压缩，叠牌层次可见 | VM 截图目视对比 |
| AC-04 | VM 端发牌堆张数角标显示在卡牌右下区域 | VM 截图目视对比 |
| AC-05 | Vue 端外观与修改前一致（无回退） | Vue 截图对比 |

## 8. 执行步骤

- [ ] **T-01** 创建 worktree `D:/autostack/.wt/lang-636/auto-lang`，分支 `plan-636-dev`；在 auto-os 侧建兄弟 worktree `D:/autostack/.wt/lang-636/auto-os`
- [ ] **T-02** 修复 `app.at` 顶栏：`flex-wrap` → 两行 col 布局（AC-01）
- [ ] **T-03** 修复 `card_face.at` 中心区：`absolute inset-0` → `flex-1 items-center justify-center`（AC-02）
- [ ] **T-04** 修复 `app.at` 发牌堆角标：`absolute` → 流式布局末行（AC-04）
- [ ] **T-05** 修复 `app.at` 牌桌列/顶层容器 `min-h-*` → `h-*`（AC-03）
- [ ] **T-06** 双端截图验证（VM + Vue），目视核查 AC-01~AC-05，提交结果

## 9. 复审记录

stage: new | PLAN-636 | revision 1 | outcome: pass | next: work
authorized scope: auto-os/apps/037-klondike .at 源文件修改，无 Rust 变更

## 10. 待澄清事项

- icon 渲染在 VM 截图中实际显示的是图形还是占位文本？AURA 快照 `[Image]` 是树文本的惯例标签（非错误），需从截图图像本身判断（T-06 确认）。
