# AutoUI VM 帧预算与空转渲染（vm-frame-budget）

> **定性**：需求级/专题设计（Plan 468：slug 不带号，归 `autoui/`）。
> **状态**：draft（2026-09-18）——问题分析与机制路线图；easy wins 已由
> [PLAN-650](../../plans/650-vm-idle-render-trim.md) 先行落地，架构级机制见本文 §5–§6。
> **关联**：
> - [Design 08 UI Systems](../08-ui-systems.md)（AURA / 后端矩阵）
> - [Design 20 AutoUI 分离架构](../20-autoui-separation-architecture.md)（长期 Compositor 路线）
> - [Design 23 virtual-desktop](virtual-desktop.md)（DesktopSession / 多窗）
> - KNOWN-DEBT：P499-1 / P530-D2 / P530-D3 / PLAN-062 F1
> - Plan 631 profile：`docs/plans/evidence/631/profile.md`
>
> **知识分层**：本文回答「VM 轨为何空转、渲染模型是什么、复杂优化怎么做」；
> 「某次 easy win 怎么改」见 PLAN-650；「当前实现长什么样」见 `docs/specs/auto-lang/ui/`。

---

## 1. 问题陈述

### 1.1 用户实测（2026-09-18）

ui-gallery **VM 模式**下：

1. 比较卡；
2. **页面完全不动时仍占 ~5% CPU**；
3. 直觉：UI 不动时 `dirty=false`，不应反复渲染。

会话代码走查结论：**用户的 dirty 直觉是对的**——PLAN-062 F1 已让 timer 空转拍不置脏；
**问题不在 dirty 误置，而在「消息泵仍在投递 + Element 缓存快速路径从未生效」**。

### 1.2 双因模型

```
                    ┌─────────────────────────────────────┐
 静止 CPU ~5%  ≈   │  消息泵频率  ×  单次 view() 成本     │
                    └─────────────────────────────────────┘

 消息泵（每次 update 后 iced 必调 view）
   · hot_reload 500ms（有 source_path 即订）
   · timer every_ms（when 假仍订阅——P499-1）
   · 路由切换后图表 timer 不退订（P530-D2）
   · desktop ServiceTick 400ms
   · MCP 常驻 / 500ms 泵（__hot_reload、PollStream、__bounds_collected）

 单次 view() 成本（dirty=false 时本应 O(1) 取缓存）
   · Element 缓存 store-then-take → 恒 miss（P530-D3）
   · fall-through：cached AbstractView → render_dynamic_view 全量 Element 构造
   · 旁路：live_vtree / collect_input_ids / needs_bounds / Style::parse
```

债档定量（P530-D3）：实测 **47k tick 仅 7 次 dirty，4.1 万次全量 iced 树重建**。

---

## 2. 渲染模型：「全量」还是「只渲染可见」？

结论要**分层**，不能一句话答完。

| 层 | 现状（2026-09-18 走查） | 代码锚点 |
|---|---|---|
| **视图模板条件 `if`** | 只构建通过条件的子树（唯一真正的「不渲染」） | `aura_view_builder` convert 路径 |
| **路由页** | 只构建当前 `route` 页 + 壳层 | `render_outlet` / `__current_route` |
| **`for` 循环** | **全量展开** iterable，无列表虚拟化 | `aura_view_builder.rs` ForLoop / grid flatten |
| **子组件** | 父渲染时模板内子件**全部** `render_child_widget` | `aura_view_builder.rs` |
| **Scrollable** | 绘制期裁剪，**视口外子节点仍在 Element 树** | iced `Scrollable` + `clip` |
| **timer 订阅** | **与可见性无关**：组件初始化静态收集 root+child+store 全表 | `dynamic.rs` timers 收集；`renderer.rs` 订阅扇出 |
| **desktop 虚拟窗** | 有可见性过滤：minimized / hidden / 其他 workspace 不推层 | `view_desktop_fn` z_order 循环 |

**一句话**：视图树是「条件/路由裁剪」，**不是**「固定渲染所有组件」；但
**for 无虚拟化 + timer 不随可见性退订**，gallery 这类多 demo 聚合页会放大后两条。

```
 .at 源
   → AURA view 模板
   → AuraViewBuilder（条件/路由裁剪；for 全量展开）
   → AbstractView（cached_converted_view）
   → render_dynamic_view
   → iced Element（cached_rendered —— 现状恒空）
   → iced layout / draw（Scrollable 只裁绘制）
```

---

## 3. 帧管线与失效语义

### 3.1 三层「dirty」不要混淆

| 标志 | 含义 | 写入方 | 消费方 |
|---|---|---|---|
| `DynamicComponent.dirty` | 组件状态/模板是否变过 | `on_*` / `write_state` / reload | update 尾部拷入 `view_dirty` |
| `AppState.view_dirty` | App 视图是否需重建 | 组件 dirty、投影写、clock、toast、显式强制 | `dynamic_view_impl` 快速路径门 |
| iced runtime「是否调 view()」 | **没有 app 级 dirty** | 每次 update 后**必定**调 view | — |

PLAN-062 F1：timer 派发以 `state_mutation_seq` 画界——空转拍可撤销置脏。
**即便 dirty 全 false，只要 update 被调用，view() 仍会跑**；快速路径能否接住
取决于 `cached_rendered`。

### 3.2 设计意图 vs 实际行为（Element 缓存）

**意图**（`dynamic_view_impl` 注释）：

```
dirty=true  → 模板→AbstractView→Element → cache → return
dirty=false → 若 cache 有 Element → take 并直接 return
```

**实际**（P530-D3）：

```rust
*state.app.cached_rendered.borrow_mut() = Some(result);
state.app.cached_rendered.borrow_mut().take().unwrap()
```

store-then-take 使**每次 view() 返回后缓存必为 `None`**。下一次 `dirty=false`
帧 100% miss → fall-through 全量 `render_dynamic_view`。

**为何不能简单「多存一份」**：iced 0.14 `Element<'static, M>` **不可 `Clone`**，
且 `view()` 必须返回**owned** Element。注释承诺的「同 Element 复用」在当前返回
约定下物理上不可达。

### 3.3 消息泵目录

| 泵 | 默认周期 | 静止是否停 | 备注 |
|---|---|---|---|
| `hot_reload_tick` | 500ms → **650 后非 debug 2000ms** | 否（文件未变只跳过 reload） | `AUTOUI_HOT_RELOAD=0/1` |
| timer `widget_event_tick` | `every_ms` | **when 假在 650 后不订阅** | 650 前：when 只在 handler 早退 |
| `ServiceTick`（desktop） | 400ms | 否 | fit/snapshot/bus/clock 驱动 |
| MCP action / 泵 | 常驻 | 否 | 同步已有 dirty/ws 门控（PLAN-062 T11） |
| toast 到期 | 任务驱动 | 有 toast 才有 | 650 前已「有变化才 dirty」 |

### 3.4 热区（Plan 631 实测）

027-file-manager 67 行列表、debug 档、脏重建帧：

| 分项 | p90 |
|---|---|
| Style::parse | ~13ms（~builder 80%） |
| render（AbstractView→Element） | ~5ms |
| builder 其余 | ~2ms |
| intern 缓存后 parse | ~1.4ms（~9×） |

结论：**泵频率决定「跑多少次」；Style/Element 构造决定「每次多贵」**。
E-1/E-3 降频，E-2/E-4 降单次旁路；**真正的 dirty=false O(1) 要靠 D-1 缓存**。

---

## 4. 已落地：easy wins（PLAN-650）

| ID | 机制 | 设计要点 |
|---|---|---|
| E-1 | timer `when` **订阅层门控** | `timer_when_allows_subscription`；iced 每 update 重算 subscription，when 翻转自动挂/摘；`fire_timer` 门保留为双保险。清偿 P499-1 调度器半边 |
| E-2 | dirty=false 旁路减负 | 非 debug：跳过 live_vtree / collect_input_ids；needs_bounds 仅 dirty 帧 |
| E-3 | hot_reload 降频/门控 | env 0 关 / 1=500ms / 缺省 debug 500·非 debug 2000 |
| E-4 | MCP 捕获收紧 | capture 以 F12 为主；MCP 仅 dirty 或 `mcp_active_recently(30)` |

**验证**：`cargo check --features ui-iced` + nextest 3/3（plan650 + fire_timer + mutation_seq）。

**预期效果边界**：

- chart / when 恒假页：静止 timer 泵 → 0，CPU 应显著下降；
- 纯 hot_reload 页：泵从 2Hz → 0.5Hz（或 env 归零）；
- **仍有泵时**：单次 view 仍可能 fall-through 重建 Element（D-1 未做）——
  5% 是否降到 1% 以下取决于页面树大小与剩余泵。

---

## 5. 延期机制设计（复杂项）

> 以下为 **design 选项与取舍**，尚未实现；实现时另拆 plan（引用本文节号）。

### 5.1 D-1 — Element 帧间缓存 / 帧跳过（P530-D3 根治）

**目标**：`dirty=false` 且结构未变时，**不**执行 `render_dynamic_view`。

**约束**：iced 0.14 `Element` 非 Clone；`view()` 返回 owned；widget Tree 由 iced
按结构 diff 维持（scroll 位置等交互态依赖稳定 Tree 身份）。

#### 候选 A（推荐评估）：SharedSlot Widget

App 级持有：

```rust
// AppState
frame_slot: Rc<RefCell<Option<iced::Element<'static, IcedMessage>>>>,
```

- `dirty=true`：重建 Element，**写入 slot 并保留**（不再 take 清空）；
  `view()` 返回 `FrameShell { slot: slot.clone() }`。
- `dirty=false`：同样返回 `FrameShell`；`FrameShell` 作为 Widget 在
  `layout`/`draw`/`on_event`/`overlay` **委托** slot 内 Element。
- `Widget::children` / `diff`：子 Tree 对准 slot 内 Element；dirty 重建时
  `tree` 与新 Element 对齐。

**优点**：语义正确；交互态可随 Tree 保留。  
**风险**：自定义 Widget 的 Tree 生命周期是最大坑（nested Element 的 state 槽位）；
DevTools wrap_debug / PointerPressArea / toast Stack 包装层需一并考虑身份稳定。

#### 候选 B：iced `lazy` / memo 子树

对页级或组件级子树包 `widget::lazy(state_key, build)`，key 未变则跳过重建。

**优点**：官方机制，Tree 由 iced 管理。  
**风险**：解释器路径每帧产出**新** AbstractView/字符串，key 难廉价稳定；
需「内容指纹」或「状态版本号」作为 key（可与 `state_mutation_seq` 联动）。

#### 候选 C：接受 Element 重建，压到毫秒级

- Style intern 全量默认开启（631 已验证 parse ~9×）；
- prop / class 串解析结果挂 AbstractView 节点，避免 builder 段重复 parse；
- dirty=false 仍走 cached AbstractView，但 render 段增量（见 5.5）。

**适用**：在 A/B 落地前的过渡；**不能**单独把静止 CPU 打到接近 0。

#### 验收草案（供未来 plan）

1. 单元：dirty=false 连续 N 次 view，断言 `render_dynamic_view` 调用次数 = 0
   （或 FrameShell 委托计数）；
2. 实机：ui-gallery 静止 + `P631_PROFILE=1`，rebuild 吐帧 ≈ 0（无泵时）；
3. 交互：scroll 偏移、input focus、toast 在 dirty=false 帧后不丢；
4. F12 开/关、MCP 读快照不回归。

---

### 5.2 D-2 — 挂载生命周期退订（P530-D2）

> **2026-09-18 承接**：D-2 与「子组件 `.Tick` 不订阅」同族，系统设计见
> [component-time-and-events](component-time-and-events.md)；阶段 1 实现计划
> **PLAN-652**（TimeSource + mounted 类型过滤）。下文机制草案作历史输入。

**问题**：LineChart/DonutChart 的 `AnimLnTick`（33ms）在路由离开后仍订阅；
`when` 假时 650 后可不订，但 **when 真且组件已卸载** 时仍空转。

**机制草案**：

1. **挂载记账**：AuraViewBuilder / render_child_widget 在构建期写入
   `mounted_widgets: HashSet<String>`（本帧实际实例化的 widget 名）；
2. **订阅过滤**：`timer_entries` 仅当 `t.widget ∈ mounted`（或 root/store 恒订）
   才 `widget_event_tick`；
3. **身份**：iced subscription 已按 (app, widget, event, ms) 扩展；
   mounted 变化会自然重订阅。

**与 E-1 关系**：E-1 管 when；D-2 管「组件是否还在树上」。两者正交。

**风险**：store-as-child / 包组件 timer 归属名与渲染名可能不一致；
「挂载」在 for 展开里是多实例——需按 **widget 类型** 还是 **实例 path** 订阅？
建议 v1 按类型名（与现 `TimerEntryRuntime.widget` 一致）。

---

### 5.3 D-3 — for 列表虚拟化（viewport windowing）

**目标**：scroll 容器内长列表只构建可见项 + overscan。

**依赖**：

| 面 | 约束 |
|---|---|
| 路径 / VNodeId | ForLoop 多实例 path 唯一性（Plan 307/323 已有 path hash 语义）——虚拟化后「同一 index 不同帧」必须稳定 |
| MCP / DevTools | 树结构随滚动变化 → 快照/bounds 语义要声明「仅可见子集」或提供完整逻辑树 + 几何仅可见 |
| 命中 / 滚动 | 命中测试只在可见节点；滚动条尺寸需要 **逻辑高度估计**（行高模型） |
| 状态 | 行内 input/选中态：不可见行的 iced Tree state 会被回收，需 app 层状态完备 |

**形态草案**：

- 扩展 `scroll` / list 语义：`virtual: true` + 行高策略（固定 px / 估计）；
- builder：`render_for_children` 改为「取窗口切片」而非全量；
- 非 scroll 上下文默认全量（行为不变）。

**不建议**在 D-1 之前做：虚拟化后 dirty 帧仍频繁，缓存收益会被滚动交互抵消；
两者可并行设计，但落地顺序上 **D-1 先**更划算。

---

### 5.4 D-4 — ServiceTick / desktop 多窗帧预算

**问题**：desktop `view_desktop_fn` 每次 view 遍历 wallpaper + desktop surface +
dashboard + **全部可见 vwin**，每个 App 各自 `dynamic_view`；Element 缓存 miss
时成本 × 窗数。

**机制草案**：

1. **层 dirty 位**：仅 `view_dirty` 的 App 走重建；其余层返回「上次 Element」
   （依赖 D-1 slot，或 per-layer slot）；
2. **ServiceTick 空拍短路**：tick 内无 fit/snapshot/bus/inject/clock 变更时，
   不置任何 view_dirty（部分已做到）；进一步可 **合并 tick 消息**（drain 后再
   一次 view）；
3. **快照面板**：switcher 打开时的 400ms 抓拍改为「有脏窗才抓」。

**风险**：fit 测量、投影指纹、热重载桥接都挂在 ServiceTick——短路条件必须
白名单化，禁止「默认短路 + 例外补丁」。

---

### 5.5 D-5 — 样式与 prop 解析预算

- **Style intern**：`AUTO_STYLE_CACHE` 已验证 debug 9×；评估默认开启与缓存键
  （类串 + dark_mode + accent + window_w）。
- **prop 每帧重解析**（536 已注记）：builder 对 prop 字符串/插值每帧解析——
  将解析结果缓存到 **模板节点扩展槽** 或 AbstractView 构建期一次化。
- **class 串 → IcedStyle**：与 token 配方（Design 29）联动，避免运行期反复
  `family_of` / 颜色解析。

---

### 5.6 长期：与 Design 20 分离架构的关系

Design 20（Compositor + RenderQueue）解决的是 **多 App GPU/内存** 与
**进程隔离**，不是单进程 VM 的 view 空转。  
**两者正交**：

- 20：App 不持有 GPU，宿主合成 → 百应用内存；
- 本文 D-1：单会话内 **逻辑 view 帧预算** → 静止 CPU / 交互帧时。

若 386 queue 臂复活，投影器 v1「只裁剪不缓存」与本文 D-1 是同一问题在
DrawList 轨的镜像——**缓存/增量语义应在协议层一并考虑**（避免 live 轨
做了 FrameShell，queue 轨每帧全量 DrawList）。

---

## 6. 优化路线图（建议顺序）

```
 [已完成] PLAN-650 E-1..E-4
            │  降泵频率 + 降 fall-through 旁路
            ▼
 [建议下一刀] D-1 Element 帧间缓存（候选 A/B 试验）
            │  静止帧 O(1)；交互帧仍全量但 Style intern
            ▼
 [并行可做]  D-5 Style/prop 缓存默认化（压交互帧时）
            ▼
 [其后]      D-2 挂载退订（多页 gallery / 图表）
            ▼
 [其后]      D-4 desktop 层 dirty 跳帧（多窗）
            ▼
 [按需]      D-3 for 虚拟化（长列表应用）
```

**测量门禁（每个机制落地都跑）**：

```text
P631_PROFILE=1          # rebuild 频率与 builder/render 毫秒
AUTOUI_HOT_RELOAD=0     # 排除热重载泵做 A/B
P530_NOMCP=1            # 排除 MCP 旁路 A/B（诊断用）
```

---

## 7. 裁定与开放问题

| # | 问题 | 现状倾向 |
|---|---|---|
| Q1 | hot_reload 非 debug 默认 2000ms 是否可接受？ | 650 已按 2000 落地；要默认 500 可改 E-3 |
| Q2 | MCP 无 F12 的 inspect 滞后（到下一次 dirty） | 可接受则保持；强依赖则加 `AUTOUI_MCP_LIVE=1` |
| Q3 | D-1 候选 A vs B | 建议 **先 spike A**（语义完整）；B 做页级 lazy 试点 |
| Q4 | 虚拟化是否进 VM DSL 语法 | 倾向 scroll/list **可选属性**，默认全量兼容 |
| Q5 | 桌面 shell 时钟/投影是否必须每分钟 dirty | 已是「有变化才 dirty」范例，可推广 |

---

## 8. 变更记录

| 日期 | 说明 |
|---|---|
| 2026-09-18 | 首稿：问题分析 + 渲染模型分层 + Element 缓存失效机制 + D-1..D-5 设计选项 + 路线图；关联 PLAN-650 easy wins |
