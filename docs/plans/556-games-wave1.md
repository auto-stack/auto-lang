---
plan_id: PLAN-556
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: games-wave1
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-man]   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 10
---

# [PLAN-556] 游戏第一波——032-2048 / 033-snake / 034-gomoku / 035-memory

## 变更摘要

AutoOS 游戏目前只有 038-minesweeper 一个。本计划落地 2026-09-05 盘点确认的
**游戏第一波四个小体量游戏**（填编号空洞 032–035），并给 ui-gallery 增
**05-games 分类**（038 一并归入）：

| 编号 | 游戏 | 驱动 | 体量预估 |
|---|---|---|---|
| 032 | 2048 | 纯事件（键盘/按钮） | ~1 天 |
| 033 | snake 贪吃蛇 | Tick 定时（025 机制） | ~1.5 天 |
| 034 | gomoku 五子棋 | 纯事件（双人本地） | ~1.5 天 |
| 035 | memory 记忆翻牌 | Tick（延时翻回） | ~0.5 天 |

依赖：PLAN-552 `desktop:` 字段（软依赖）；PLAN-554 T1 的 Tick 契约结论
（若 554 未执行，本计划 T1 自带同款探针）。

## 目标

1. 四游戏各自成目录 `examples/ui/0XX-<name>/`，vue/vm 双端可玩。
2. 各游戏有：胜负判定、restart、成绩（best/分数 storage 持久化，适用项）、
   键盘 + 屏上按钮双操作（2048/snake）。
3. 画廊收录四游戏且分类 "05-games"；038-minesweeper 同步归入 05-games。
4. `examples/ui/README.md` 总览表补四行 + 空洞注记。
5. 桌面上架（pac `desktop: "true"` + `category: "game"`，552 合入后生效）。

## 架构方案

**共享技术底座**（全为先例实证）：

- **状态表示**：棋盘/蛇身/牌组一律 **int 或 str 平铺列表**（B12 规避；
  028 `used/scored` int 列表下标读写先例）；view 行对象 handler 自建。
- **Tick 自驱动**（snake/memory）：`interval int = <ms>` + `.Tick`
  （025 实证；契约探针 T1）。
- **随机数**：VM 无可用 Random 原生（stdlib `Normal` 为桩、`random_hex`
  非 int 族——552 前置盘点钉死）→ **LCG handler 自实现**：Init 以
  `Time.now_ms()` 播种，`next = (state*1103515245+12345) % m` 取位；双端
  同式（种子不同无妨，游戏不需可复现）。
- **键盘**：`bind` 块（028 方向键/Enter 先例；038 同）。
- **grid 渲染**：静态 cols（P537-D2 债绕开）；225 格（gomoku 15×15）渲染
  量 VM 探针 T1c，超限降 13×13。

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

- **GOAL-010**（AutoOS 默认应用集）：游戏缺口盘点 2026-09-05（"游戏现在
  只有扫雷"）；第一波取纯 grid/事件可完成的四款，Tetris/接龙（需更强
  Tick/拖放）拆 PLAN-557/558。
- **能力现状**：Tick 机制（025 `interval`+`.Tick`，renderer `AppTickRecipe`）；
  `Time.now_ms/sec`（stdlib.rs:644+）；`bind` 键盘（028）；grid 先例
  038；storage best 榜先例 028 recent 槽；B12 平行列表先例 028 头注。
- **编号**：032–035 空号（现员 031 由 PLAN-553 预订，无冲突——553 先行
  则顺延无碰撞；README 空洞优先规则）。
- **画廊**：`auto-man/src/vue.rs` 分类 if 链 + getCategories 数组——
  现四分类硬编码，032+ 落 else 臂 "04-systems"（错类）→ T9 增
  "05-games" 臂（032–038 前缀）+ 038 从 03-apps…实况：038 现走 else
  → 04-systems，T9 一并纠正。
- **icon**：lucide VM 闭集（P537-D1 债）——T1b 核验候选图标（2048
  "hash"、snake "move"、gomoku "circle-dot"、memory "brain"），不在集
  回退闭集近似（如 "app-window"/"grid-2x2"）。

## 详细设计

### 032-2048

- model：`board = [0×16]`（int 列表，0 空，值=2^n 原值）；`score int`、
  `best int`（storage `2048.best`）；`over/won str` 门控；`seed/state`
  （LCG）。
- 移动统一化：四方向 → 先行列提取旋转成"左移"标准形 → 压缩/合并/回写；
  合并规则同排只并一次（左优先）；`moved` 无变化不 spawn。
- spawn：LCG 取空位 + 90% 值 2 / 10% 值 4。
- 胜负：任一格 2048 → won（可继续）；无空位且无可合并 → over。
- view：4×4 grid，tile 色阶（值→bg/text 色 chip 映射表，handler 拼）；
  屏上方向按钮（移动端/无键盘）+ New Game + 分数条。
- bind：←↑→↓ → `.Move(dir)`。

### 033-snake

- model：`body = [头idx,...]`（int 列表，15×15 格 idx）；`dir str`（待转
  向缓冲防 180° 回头）；`food int`；`score/best`（storage `snake.best`）；
  `running/over str`；`interval int = 250`（吃 5 食 -10ms，下限 120）。
- `.Tick`：缓冲转向生效→算新头→撞墙/撞身 over→吃食长身+LCG 新食（不在
  蛇身）→否则尾缩。`.Turn(d)`：合法转向入缓冲。Pause/Restart。
- view：15×15 grid（蛇头深/蛇身渐次/食物红点——色 chip 表）；方向十字
  按钮 + 分数条 + over 覆盖层。
- bind：方向键 → `.Turn`。

### 034-gomoku

- model：`board = [0×225]`（0 空 1 黑 2 白）；`turn int = 1`；`moves int`；
  `history = []`（落子 idx 栈，undo 用）；`winner str = ""`；`last int`
  （最后一子高亮）。
- `.Place(i)`：空格且未分胜负→落子→五连判（以 i 为心 4 方向窗口扫描）→
  换手；`.Undo`（双人各退一手=两步）；`.Restart`。
- view：15×15 线盘形态（格底木色 `bg-[#d3b17a]`，子为圆形 chip——黑
  `bg-gray-900`/白 `bg-white border`）；当前手指示 + 手数；胜局覆盖层。
- 纯事件无 Tick；键盘可选（Tab 列导航远期，v1 点击）。

### 035-memory

- model：`deck = [16 emoji 串]`（8 对，Init LCG 洗牌 Fisher–Yates）；
  `face = ["0"×16]`（翻开位）；`matched = ["0"×16]`；`pick int = -1`
  （首翻位）；`lock str`（两翻待回锁）；`moves int`；`interval = 800`。
- `.Flip(i)`：未翻/未配/未锁→翻；首翻记 pick；次翻→moves+1，配对则
  matched 置位，否则 lock + 等 Tick 翻回；全配对 → won。
- view：4×4 卡片 grid（翻面 = emoji 显/隐 + 翻转过渡 class）；步数 +
  重开 + 胜利覆盖层。

### 共用 pac 形态

```
name: "2048" / "snake" / "gomoku" / "memory"
title: "2048" / "Snake" / "Gomoku" / "Memory"
icon: <T1b 核验后定>
category: "game"
render: "vue"
desktop: "true"
window: "fit"
```

## 测试设计

每游戏 `tests/desktop_mcp.py`（双端）：

- 2048：初始两 tile；合并断言（构造：连按同方向至可预见合并，断言分数
  增与格子数变化）；spawn 不覆盖已有格；over 覆盖层出现路径（LCG 不可
  控 → 断言机制函数级由 handler 纯状态变换承担，mcp 断言 UI 侧行为）。
- snake：Start→Tick 后蛇头位移一格断言（读格底色类）；吃食身长 +1；
  撞墙 over 层。
- gomoku：交替落五连（同排 5 点击）→ 胜局层 + 后续点击无效；undo 两步。
- memory：翻两张不同 → Tick 后回盖；翻同对 → 常亮；8 对全配 → won。

## 验收标准

1. 四游戏 `auto run` / `auto run -r vm` 双端可玩、胜负/重开/成绩全通。
2. 四套 `desktop_mcp.py` 双轨全绿。
3. 画廊重生成后四游戏 + 038 均分类 "05-games"。
4. README 总览表四行 + 空洞注记；`cargo check -p auto-lang -p auto-man` 绿。

## 执行步骤
（原子任务：精确文件路径 + 确确操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T1 共享探针**
  a) Tick 契约（025 形态复刻最小探针：interval+Tick 计数器，vm/vue 双端）；
  b) lucide 候选图标闭集核验（hash/move/circle-dot/brain）；
  c) 15×15=225 格 VM 渲染冒烟（超帧降 13×13 裁定写回）。
  验证：探针产物 scratch/p556/（三份输出）
- [ ] **T2 032-2048 骨架**
  `examples/ui/032-2048/`：pac.at + SPEC.md + `src/front/app.at`（model +
  view 4×4 grid + 分数条 + 屏上键）。
  验证：`auto build` 0 错误
- [ ] **T3 032-2048 逻辑**
  Move 四向统一化/合并/spawn(LCG)/胜负/best 持久化 + `bind` 方向键。
  验证：`auto run` 手玩一局到合并可复现
- [ ] **T4 033-snake 骨架与逻辑**
  `examples/ui/033-snake/`：pac + SPEC + app.at（body/food/Tick 步进/转向
  缓冲/撞判/分数/加速/over）。
  验证：`auto run` 手玩 30s
- [ ] **T5 033-snake vm 对拍**
  `auto run -r vm` 同流程（Tick 机制 vm 轨首用游戏——差异登记 SPEC）。
  验证：vm 轨手玩冒烟
- [ ] **T6 034-gomoku**
  `examples/ui/034-gomoku/`：pac + SPEC + app.at（落子/五连判/undo/重开/
  线盘 view；15×15 或 T1c 裁定格数）。
  验证：`auto run` 双人一局含 undo
- [ ] **T7 035-memory**
  `examples/ui/035-memory/`：pac + SPEC + app.at（洗牌/翻牌/配对/延时回盖/
  步数/won）。
  验证：`auto run` 完整一局
- [ ] **T8 四套 desktop_mcp**
  各目录 `tests/desktop_mcp.py`（测试设计节断言组）。
  验证：四目录 mcp 双轨全绿
- [ ] **T9 画廊 05-games 分类**
  `crates/auto-man/src/vue.rs`：分类 if 链增 `032|033|034|035|038` →
  "05-games"；getCategories 数组追加；（tags if 链补四游戏标签）。
  验证：`cargo check -p auto-man` + ui-gallery 重生成人工过目
- [ ] **T10 README 回写与终检**
  `examples/ui/README.md`：总览表补 032–035 四行 + 空洞注记（031 归 553、
  036/037 归 557/558 预订说明）。
  验证：`cargo check -p auto-lang -p auto-man`（零警告）

## 复审记录

## 待澄清事项

1. gomoku AI（minimax/贪心）远期——v1 双人本地（盘点结论）。
2. 游戏音效：无音频 FFI，全波次不做。
3. 225 格 VM 渲染若超帧：降 13×13（T1c 裁定权），README 注记原因。
4. 039/040 空号留给后续（Wordle/数独/推箱子备选池，未立项）。
