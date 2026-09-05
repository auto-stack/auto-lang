---
plan_id: PLAN-557
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: tetris
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 6
---

# [PLAN-557] 036-tetris——俄罗斯方块

## 变更摘要

游戏第二波之一（PLAN-556 后续）：填编号空洞 **036**，实现经典俄罗斯方块
（10×20 板、7 种四格块、旋转/软降/硬降、消行计分/等级加速、next 预览、
暂停、best 持久化）。是 Tick 自驱动 + 键盘流的标帜性游戏（体量 2–3 天）。

依赖：PLAN-552 `desktop:` 字段（软）；PLAN-556 T1 的 Tick 契约与 LCG 随机
结论（若 556 未先行，本计划 T1 自带同款探针）。

## 目标

1. `examples/ui/036-tetris/`：vue/vm 双端可玩。
2. 完整规则：7 bag（v1 用独立 LCG 抽块即可，不必 7-bag 均衡）→ 出生
   （顶部中间）→ 重力下落（Tick）→ 旋转（SRS 简化：无踢墙 v1，踢墙
   见待澄清②）→ 软降（↓）/硬降（space）/左右移 → 锁定 → 满行消除 →
   计分（1/2/3/4 行 = 100/300/500/800 × level；软降 +1/格、硬降 +2/格）。
3. 等级：每 10 行 level+1，落速 `max(120, 600 - (level-1)*60)` ms。
4. next 预览（4×4 小格）；暂停（P）；game over（出生位重叠）覆盖层 +
   restart；best（storage `tetris.best`）。
5. 桌面上架（`category: "game"`、`desktop: "true"`）+ 画廊 05-games
   （556 T9 分类臂如已含 036 前缀则零改动，否则补）。

## 架构方案

- **板**：`board = [0×200]` int 列表（0 空 / 1–7 色号——B12 规避，下标
  读写先例 028）。
- **当前块**：`piece int`（0–6）、`rot int`（0–3）、`x/y int`（锚点）；
  形状表 = 7×4 旋转 × 4 cell {dx,dy} 硬编码常量（handler 内 switch 取
  出转 int 列表 `[dx1,dy1,dx2,dy2,...]`——Obj 常量表规避，平铺 int 列表
  保真）。
- **Tick**：`interval` 变量按 level 动态改写（025 机制是否支持运行时改
  周期——T1 探针；不支持则固定 100ms Tick + 计数分频调速，**保底方案**）。
- **渲染**：左信息栏（score/lines/level/best/next 4×4 小格）+ 中 10×20
  主盘（200 格 grid，静态 cols=10；色 chip 表 7 色）+ 底部键位说明。
- **LCG 随机**：`Time.now_ms` 种子（556 同款式）。

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

- **GOAL-010**：2026-09-05 盘点游戏第二波（Tetris 标志性、接龙 dnd 里程碑
  = PLAN-558）。
- **能力现状**：Tick 机制（025 `interval`+`.Tick`；renderer `AppTickRecipe`
  tokio interval + `__tick` 事件）；键盘 `bind`（028/038 先例，space/
  字母键可绑）；`Time.now_ms`（stdlib.rs:644）；grid 静态 cols + 200 格
  渲染（038 16×16=256 格同量级，VM 可承受）；storage 定长槽（028）。
- **编号**：036 空号（README 空洞优先；032–035 由 PLAN-556 预订）。
- **icon**：lucide "blocks" 候选（VM 闭集核验 T1b，回退闭集近似）。

## 详细设计

### model（要点）

```
var board = [0 × 200]
var piece int = 0  var rot int = 0  var px int = 3  var py int = 0
var next_piece int = 0
var score int = 0  var lines int = 0  var level int = 1  var best int = 0
var running str = "1"   // 1|0|pause|over
var lcg int = 0         // 随机状态
var interval int = 600  // T1 结论定：可变 or 固定+分频
var cells = []          // handler 自建 {i,chip} 行对象（view）
var next_cells = []     // next 预览格
```

### handler（要点）

- `.Cells`（取块 cell 平铺表）：`piece*16 + rot*4` 段切硬编码 int 列表。
- `.CanMove(dx,dy,rot2)`：四 cell 越界/叠已占（board 值>0）判定。
- `.Tick`：running==1 → 可降则 py+1，否则 `.Lock()`。
- `.Lock`：写板 → 消行（满行删除前移，行数计分/升级/调速）→ spawn
  next（出生位可放性判 over）。
- `.Left/.Right/.Rotate/.SoftDrop/.HardDrop/.Pause/.Restart`。
- 视图格行对象重建：`cells` 每次状态变更后由 handler 全量重建（板 200 +
  当前块叠加投影）。

### view（要点）

主盘 grid cols=10（格 aspect 方形 `aspect-square`）；over/pause 覆盖层；
底部屏上按钮（← ↻ → ↓ ⤓ P——移动端/无键盘兜底）。

### pac.at

```
name: "tetris"  title: "Tetris"  icon: "blocks"
category: "game"  render: "vue"  desktop: "true"  window: "fit"
```

## 测试设计

`tests/desktop_mcp.py`（双端）：

1. 出生块出现（板顶非空格断言）；
2. Tick 后下落一格（同列两时刻格位断言）；
3. ← → 位移断言；硬降触底（瞬间到底部行断言 + 分数 +2×格）；
4. 消行：无法预置板 → 断言机制代理（硬降若干块后 lines 计数与板格总数
   守恒：200 = 空格 + 占格 + 当前方块）；
5. 撞顶 game over 覆盖层（连续硬降至顶，超时轮询）。

## 验收标准

1. 双端完整可玩：移动/旋转/软硬降/消行/计分/升级加速/next/暂停/重开/best。
2. `desktop_mcp.py` 双轨全绿。
3. 画廊收录分类 05-games；README 总览表补 036 行。
4. `cargo check`（若触及 vue.rs 分类链）绿。

## 执行步骤
（原子任务：精确文件路径 + 精确操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T1 探针**
  a) interval 运行时改周期是否生效（025 机制读点 + 最小双端探针；不通
  → 固定 100ms + 分频，结论写回）；b) lucide "blocks" 闭集核验。
  验证：scratch/p557/ 探针输出
- [ ] **T2 骨架**
  `examples/ui/036-tetris/`：pac.at + SPEC.md + `src/front/app.at`
  （model 全量 + 空板渲染 + 信息栏布局）。
  验证：`auto build` 0 错误
- [ ] **T3 块表与移动族**
  形状常量表 + `.Cells/.CanMove/.Left/.Right/.Rotate/.SoftDrop/.HardDrop`
  + cells 重建。
  验证：`auto run` 手玩移动/旋转/硬降
- [ ] **T4 Tick 重力与锁定**
  `.Tick` 降落 + `.Lock` 写板/消行/计分/升级/调速（T1a 方案）+ spawn/over。
  验证：`auto run` 完整一局手玩
- [ ] **T5 键盘/暂停/next/best**
  `bind` 全键位 + P 暂停 + next 预览 + best storage。
  验证：`auto run` 键盘流 + best 重开保留
- [ ] **T6 测试与回写**
  `tests/desktop_mcp.py` 五断言组（双端）；vm 轨全流程；README 036 行 +
  SPEC 双端注记。
  验证：mcp 双轨绿 + `auto run -r vm` 冒烟

## 复审记录

## 待澄清事项

1. 7-bag 均衡抽取 vs 独立 LCG：v1 LCG（简单）；7-bag 远期。
2. 旋转踢墙（SRS kick table）：v1 无踢墙（贴边旋转失败即不变）；踢墙
   随后续 polish。
3. ghost piece（落点投影）：v1 可选加分项，非验收项。
