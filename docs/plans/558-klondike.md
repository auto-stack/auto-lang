---
plan_id: PLAN-558
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: klondike
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 7
---

# [PLAN-558] 037-klondike——经典纸牌接龙（dnd 里程碑）

## 变更摘要

游戏第二波之二（PLAN-556/557 后续）：填编号空洞 **037**，实现 Windows
经典 **Klondike 接龙**（52 牌、7 列牌桌 1..7 发牌、发牌堆/弃牌堆、4 基础
堆 A→K 同花、牌桌降序红黑交替、双击上基础、胜利检测）。体量 3–5 天，
定位 **拖放（dnd）里程碑**——022-kanban 已实证 vue 轨 HTML5 dnd
（`draggable`/`ondragstart`/`ondragover.prevent`/`ondrop`），本计划把它
带到游戏级密度 + **点击选择-点击目标**双通道（VM 轨 dnd 兼容性未知，
点击移动是双端保底）。

依赖：PLAN-552 `desktop:` 字段（软）；PLAN-556 的 LCG 随机式（软，T1
自带）。

## 目标

1. `examples/ui/037-klondike/`：vue/vm 双端可玩。
2. 完整规则：发牌（7 列 1..7、顶牌翻开）/ 发牌堆点击→弃牌堆（空堆重
   循环 v1 单张翻）/ 基础堆 A→K 同花递增 / 牌桌列降序红黑交替、可整段
   移动已翻开连续段 / 空列只收 K（段首 K）/ 双击自动上基础 / 胜利（4 基础
   满 52）检测 + 庆祝层 / 新局重发。
3. 移动双通道：HTML5 拖放（vue 轨）+ 点击选择→点击目标（双端保底）；
   非法移动拒绝（抖动/静默，v1 静默）。
4. 步数 + 计时（Tick 分频分钟:秒）+ best（最少步，storage
   `klondike.best`）。
5. 桌面上架（`category: "game"`、`desktop: "true"`）+ 画廊 05-games。

## 架构方案

- **牌编码**：int `card = suit*13 + rank`（suit 0–3 ♠♥♦♣，rank 1–13），
  `-1` = 空位——全 int 列表规避 B12。
- **状态**：`stock = []`、`waste = []`、`found = [-1,-1,-1,-1]`（各堆顶
  card，空 -1）、`tab = [[列0..6]]`（7 个平行列表——VM 嵌套列表字段读
  风险 → **平铺 + 定长列偏移**：`tabflat = [×13×7]` + `tablen = [0×7]`，
  T1 探针定形态）、`tabup = ["0"×7]`（各列翻开位数）。
- **洗牌**：Fisher–Yates + LCG（`Time.now_ms` 种子）。
- **移动判定 handler**：`.CanTab(card, onto)` 红黑交替降序 /
  `.CanFound(card, f)` 同花递增 / 空列 K。选中态 `sel = {from, idx}`（列
  + 段首下标——handler 自建 Obj，单对象读写已证可用）。
- **dnd**：卡片 `draggable: true` + `ondragstart: .DragStart(from, idx)` +
  目标 `ondragover.prevent: .AllowDrop` + `ondrop: .Drop(to)`（022
  board.at 45–72 行同款）；**VM 轨探针 T1b**——iced 端 HTML5 事件语义
  若缺，vm 轨只走点击通道（SPEC 登记差异，非阻塞）。
- **渲染**：牌面 = 圆角白卡（rank+suit 文本 + 红黑双色 class），背面 =
  蓝底花纹 class；列内 **固定错位**（`-mt-*` 负边距叠卡——绝对定位重叠
  DSL 不支持，015/029 布局先例用流式错位）；52+ 元素渲染量无虞。

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

- **GOAL-010**：2026-09-05 盘点游戏第二波；接龙 = Windows 标志性默认游戏
  + dnd 密度最高的常规形态（022 kanban 先例仅三列卡片）。
- **dnd 现状**：`examples/ui/022-kanban/src/front/pages/board.at:40-72`
  HTML5 五件套实证（vue 轨）；VM/iced 轨兼容性未证（P537-D1/D2 同族
  探针债风格——T1b 钉死，不通则点击通道兜底，dnd vm 化登记 KNOWN-DEBT）。
- **Tick 计时**：025 机制（分钟:秒显示用，1s 分频）。
- **编号**：037 空号（036 归 PLAN-557）。
- **icon**：lucide "layers" 候选（T1c 闭集核验）。

## 详细设计

### model（要点）

```
var stock = []  var waste = []            // int 列表（card 编码）
var found = [-1,-1,-1,-1]                 // 基础堆顶
var tabflat = [0 × 91]  var tablen = [0×7]  var tabup = [0×7]
var sel_from str = ""  var sel_idx int = -1   // ""=无选中
var moves int = 0  var elapsed int = 0  var best int = 0
var won str = "0"  var lcg int = 0
var interval int = 1000                    // 计时
```

### handler（要点）

- `.NewGame`：洗牌 → 发 7 列（列 i 长 i+1，顶翻开）→ 余 24 入 stock。
- `.DrawStock`：stock→waste 顶；stock 空 → waste 逆序回 stock（v1 单张
  翻模式）。
- `.ClickTab(col, idx)`：首击=选中段（idx..列尾，须全翻开）；再击目标
  合法则 `.MoveTab(col, idx, to)`。
- `.TabDrop(from, idx, to)`：dnd 版同判定（022 Drop 同型）。
- `.CanTabCard(c, onto) / .CanFoundCard(c, fi) / .FoundDrop(fi)` /
  `.TabDoubleClick(col)`（自动上基础）。
- `.Tick`：won==0 时 elapsed++（显示 mm:ss）。
- 胜利检测：found 四堆均 13 → won=1 + 庆祝覆盖层 + best 更新。

### view（要点）

顶行：发牌堆（背面上叠数）+ 弃牌顶 + 4 基础堆；下方 7 列 grid；列内
卡片流式负边距错位（翻开牌全露 rank 行、盖牌只露边条）；选中段高亮
`ring` class；步数/计时/新局条。

### pac.at

```
name: "klondike"  title: "Solitaire"  icon: "layers"
category: "game"  render: "vue"  desktop: "true"  window: "fit"
```

## 测试设计

`tests/desktop_mcp.py`（双端，点击通道为主轴——dnd 为 vue 轨增量）：

1. 新局断言：7 列长度 1..7、翻开数各 1、stock 24（发牌堆叠数 UI 断言）。
2. 点击通道：选中 A（任意基础可收牌）→ 点基础堆 → found 更新 + moves+1；
   非法（红桃上红桃）→ 状态不变。
3. 空列只收 K：非 K 拒绝断言。
4. 双击上基础断言。
5. vue 轨 dnd：dragstart→drop 一次合法移动（playwright drag 模拟）。
6. 胜利检测：Debug 钩子 `.DebugWinDeal`（发必胜局——测试面消息，SPEC
   登记仅测试构建可用）→ 走完 → won 层。

## 验收标准

1. 双端点击通道完整可玩一局；vue 轨 dnd 通道可用。
2. `desktop_mcp.py` 双轨全绿（vm 轨 dnd 断言豁免——T1b 结论定豁免范围）。
3. vm 轨 dnd 缺口（若 T1b 不通）登记 `docs/plans/KNOWN-DEBT-AND-RISKS.md`。
4. 画廊收录 05-games；README 总览表补 037 行。

## 执行步骤
（原子任务：精确文件路径 + 精确操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T1 探针**
  a) VM 嵌套 int 列表 handler 下标读写（tabflat 平铺 vs tab 嵌套——定
  状态形态）；b) VM 轨 HTML5 dnd 五件套事件是否派发（022 最小复刻探针）；
  c) lucide "layers" 闭集核验。
  验证：scratch/p558/ 三份探针输出
- [ ] **T2 骨架与发牌**
  `examples/ui/037-klondike/`：pac.at + SPEC.md + `src/front/app.at`
  （model + `.NewGame` 洗牌发牌 + 静态渲染全牌面布局）。
  验证：`auto build` + `auto run` 新局目检 7 列
- [ ] **T3 判定族**
  `.CanTabCard/.CanFoundCard/空列 K/.DrawStock`（单张翻 + 回循环）。
  验证：`auto run` 手玩发牌/收基础
- [ ] **T4 点击通道**
  选中/移动/整段移动/非法拒绝/双击上基础 + moves 计。
  验证：`auto run` 点击流手玩
- [ ] **T5 dnd 通道（vue）**
  五件套事件 + `.TabDrop`（022 同型）+ 选中段 draggable 化。
  验证：`auto run`（vue）拖放移动冒烟
- [ ] **T6 计时/胜利/新局/best**
  `.Tick` 计时 + won 检测/庆祝层 + best storage + `.DebugWinDeal` 钩子。
  验证：`auto run` Debug 局走到胜利层
- [ ] **T7 测试与回写**
  `tests/desktop_mcp.py` 六断言组（双端，vm dnd 按 T1b 豁免）；vm 轨
  点击全流程；README 037 行 + SPEC 双端注记 + KNOWN-DEBT 登记（如有）。
  验证：mcp 双轨绿 + `auto run -r vm` 冒烟

## 复审记录

## 待澄清事项

1. 发牌模式：v1 单张翻（经典入门）；三张翻（Vegas）远期开关。
2. vm 轨 dnd：T1b 不通则登记债 + 点击通道为准（不阻塞上架）；后续
   iced 端 pointer-drag 原语立项另议。
3. 计分体系（Vegas/标准 Windows 计分）：v1 只记步数与用时。
