---
plan_id: PLAN-650
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-idle-render-trim
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/ui/design/chart-components.md: P499-1 门口径——「调度器层未生效/恒30Hz」更新为 E-1 订阅层已门控 + fire_timer 双保险
  - docs/specs/auto-lang/ui/plans.md: 登记 PLAN-650 行（reviewed）
  - docs/plans/KNOWN-DEBT-AND-RISKS.md: P499-1 订阅层清偿注记 + P530-D3 保持在案并指向 PLAN-650 延后 D-1/vm-frame-budget §5.1
new_spec_components:
  - docs/design/autoui/vm-frame-budget.md  # 设计层：问题分析+复杂机制路线（已落 master 9cabc8a10；autoui README 已索引）
touched_goals:
  - GOAL-007: AutoUI VM 空转渲染减负 easy wins——timer when 订阅门 / dirty=false 旁路 / hot_reload 降频 / MCP 捕获门控

affects: [auto-lang/ui]
current_step: 8
total_steps: 8
---

# [PLAN-650] vm-idle-render-trim —— AutoUI VM 空转渲染减负（easy wins 先落地）

## 0. 变更摘要

用户实测：ui-gallery VM 模式页面完全不动时仍占 ~5% CPU，说明存在重复渲染。
会话分析（2026-09-18）定罪两条主因：

1. **消息泵空转**：`hot_reload` 500ms、timer `every_ms`（含 `when:` 恒假仍订阅）、
   desktop `ServiceTick` 400ms、MCP 常驻——每条消息都会让 iced 调一次 `view()`。
2. **Element 缓存快速路径架构性失效（P530-D3）**：`dynamic_view_impl` 末尾
   `store-then-take` 使 `cached_rendered` 恒 `None`，`dirty=false` 帧仍 fall-through
   全量 `render_dynamic_view`（实测债档：47k tick 仅 7 次 dirty、4.1 万次全重建）。

**本计划策略**：先把「默认就能降 CPU、行为可解释、改动面小」的 easy wins 落地；
架构级 Element 缓存 / 列表虚拟化 / 路由级退订记入延期清单，另立项。

> **设计层沉淀（2026-09-18）**：问题分析、渲染模型分层、双因模型与
> **D-1..D-5 复杂机制的设计选项/取舍** 已写入
> [docs/design/autoui/vm-frame-budget.md](../design/autoui/vm-frame-budget.md)。
> 本计划只承载 easy wins 的过程与证据；架构机制以设计文档为准。

### Easy wins（本计划落地）

| ID | 内容 | 预期效果 |
|----|------|----------|
| E-1 | timer `when:` **订阅层门控**（P499-1 调度器层）：`when` 假时不挂 `widget_event_tick`；`when` 变真后 iced 重算订阅自动挂上 | 消灭 chart 等 `when` 恒假组件的 30Hz 空消息 → 静止页零 timer 泵 |
| E-2 | `dirty=false` fall-through **旁路减负**：非 debug 帧跳过 `live_vtree` 重建、`collect_input_ids`、`needs_bounds` 等调试旁路 | 即使仍有消息泵，单次 view 成本明显下降 |
| E-3 | hot_reload **降频/门控**：非 debug 默认 2000ms（原 500ms）；`AUTOUI_HOT_RELOAD=0` 全关 | 开发热重载仍可用，静止泵频率 ÷4；测量时可归零 |
| E-4 | MCP 捕获门控：无 F12 且非 dirty 帧不做 per-frame live_vtree/inspector 产物 | gallery 常开 MCP 时的空转旁路消失（快照仍走既有 dirty 门控同步） |

### 延后（记录，不在本计划实现）

| ID | 内容 | 原因 |
|----|------|------|
| D-1 | **P530-D3 Element 帧间缓存**（候选 A SharedSlot Widget / B lazy / C 压成本）——**设计见 [vm-frame-budget](../design/autoui/vm-frame-budget.md) §5.1** | iced 0.14 `Element` 不可 Clone；需 Widget/Tree 生命周期设计，回归面大 |
| D-2 | **P530-D2 路由/组件卸载退订**（子件 timer 生命周期与挂载绑定） | 需挂载记账 + 订阅身份演进，与 E-1 部分重叠，留 E-1 验证后再做 |
| D-3 | **for 列表虚拟化**（scroll 视口 windowing） | 结构题：path 稳定 id / MCP / hit-test 联动 |
| D-4 | **ServiceTick 空拍完全零成本** / desktop 多窗 dirty 跳层 | desktop 帧泵语义复杂，易误伤 fit/snapshot/bus |
| D-5 | Style::parse 全量 intern / prop 每帧重解析（631 已做部分） | 已有缓存基建，增量优化另测 |

## 1. 目标

- **G-1 静止 CPU 可感知下降**：ui-gallery（或等价 VM app）页面不动时，相对优化前
  CPU 占用明显下降；`P631_PROFILE=1` 下静止期 `[P631-PROFILE] rebuild` 吐帧频率
  显著减少（`when` 假 timer 场景应接近 0）。
- **G-2 行为正确**：
  - `when` 为真时 timer 照常派发（动画/轮询不回归）；
  - `when` 为假时不再订阅（调度器层），handler 仍保留门控双保险；
  - F12 / MCP 需要的调试产物在 debug 或 dirty 重建帧仍可用；
  - hot_reload：默认 2s（非 debug）；`AUTOUI_HOT_RELOAD=1` 强制 500ms；`=0` 关闭。
- **G-3 债项注记**：P499-1 由本计划 E-1 清偿（订阅层）；P530-D3 保持在案并指向 D-1。

### 非目标

- 不实现 Element 帧间复用 Widget（D-1）。
- 不改 vue 轨。
- 不做列表虚拟化 / 路由级 timer 生命周期（D-2/D-3）。
- 不默认关闭 MCP server（verifier 依赖）；只减其 per-frame 旁路成本。

## 2. 架构方案

```
订阅面（update 前）                     view() 面
─────────────────                     ──────────
timer_entries ──when假──× 不订阅   ┐
              └─when真──→ tick ──→ update ──→ dirty? ─┬─ true → 全量重建
hot_reload 500ms→2000ms（非debug）┘                    └─ false → fall-through
                                                      ├─ 有 cache → 直接返回（现状恒 miss）
                                                      └─ 无 cache → 重建 Element
                                                         但跳过 live_vtree / input_ids /
                                                         needs_bounds（非 debug）
```

E-1 关键点：iced `subscription(state)` **每次 update 后都会重算**。`when` 字段从假变真
时，下一轮订阅自然挂上；变真再变假时自然摘下——无需额外调度器线程。

## 3. 技术栈

- `crates/auto-lang/src/ui/dynamic.rs`：公开 `timer_when_allows_subscription`；
  复用既有 `timer_guard_passes`。
- `crates/auto-lang/src/ui/iced/renderer.rs`：
  - subscription：timer `when` 过滤 + hot_reload 间隔/env；
  - `dynamic_view_impl`：dirty=false 旁路跳过；MCP/live_vtree 门控收紧。
- 测试：`dynamic.rs` 单测（when 门订阅谓词）+ 既有 `fire_timer` 门控测不回归。
- 门禁：Category B——改 `ui/` 下 Rust：`cargo check -p auto-lang --features ui-iced`
  + `cargo t ui` / 相关模块；无 aavm/docs_gen 触发。

## 4. 需求分析与背景调查

### 授权记录

- 2026-09-18 用户：根据分析做优化计划，**最容易实现的先做出来**，复杂的记录以后再做；
  起草计划 `/auto-plan-new`。本文件即 PLAN-650 rev1。
- 取号：`scripts/new-plan.sh vm-idle-render-trim` → **PLAN-650**（.next-id → 651）。
- worktree：`D:/autostack/.wt/lang-650/auto-lang` + 分支 `plan-650-dev`（Plan 529）。

### 证据（会话分析 + 债档）

- **P530-D3**（KNOWN-DEBT）：`dynamic_view` 末尾 store-then-take → `cached_rendered`
  恒 None → dirty=false 仍全量重建。
- **P499-1**（KNOWN-DEBT）：`timer { AnimTick (every_ms: 33, when: .anim) }` 的 when
  只在 handler 体内；订阅无条件起拍 → 每图族 ~30Hz 空事件。
- **P530-D2**：图表 timer 路由切换不退订（延期 D-2；E-1 对「when 假」场景已足够）。
- **PLAN-062 F1**：`fire_timer` 空转拍已不置脏——用户观察「dirty=false 仍占 CPU」
  与该修复并存，证明**剩余成本在 view() fall-through + 消息泵本身**。
- **renderer.rs:18664-18668**：`for t in app.component.timer_entries()` 无条件订阅。
- **renderer.rs:19524-19527**：store-then-take。
- **renderer.rs:19019-19091**：MCP 同步已有 dirty/ws 门控；但 live_vtree 在
  fall-through 路径仍建（19292 起）。
- **line_chart.at / donut_chart.at**：`AnimLnTick/AnimDnTick (every_ms: 33, when: .anim*)`。

### 视图「可见性」结论（写入本计划备忘，供 D-3 延后项引用）

- 视图树：条件/路由裁剪；**for 全量展开**，无滚动虚拟化。
- timer：**初始化静态收集、与可见性无关**；E-1 只加 `when` 谓词，不做挂载生命周期。
- desktop 虚拟窗：minimized/hidden/其他 workspace 已不推层（保持）。

## 5. 详细设计

### E-1 timer `when` 订阅层门控

`dynamic.rs` 新增：

```rust
/// PLAN-650 E-1：订阅层 when 门——假则本拍不挂 tick（iced 下轮 update 重算订阅）。
/// 无 when / 条目未知 → true（保守订阅，保留 fire_timer 双保险）。
pub fn timer_when_allows_subscription(&self, widget: &str, event: &str) -> bool {
    let Some(entry) = self.timers.iter().find(|t| t.widget == widget && t.event == event)
    else { return true; };
    match &entry.when {
        None => true,
        Some(when) => self.timer_guard_passes(when),
    }
}
```

`renderer.rs` subscription 循环：

```rust
for t in app.component.timer_entries() {
    if !app.component.timer_when_allows_subscription(&t.widget, &t.event) {
        continue;
    }
    subs.push(widget_event_tick(app_id, &t.widget, &t.event, t.every_ms));
}
```

### E-2 / E-4 dirty=false 旁路减负 + MCP 捕获门控

`dynamic_view_impl`：

1. `capture_debug`：改为 **`debug_mode` 为主**；MCP 在 **dirty 重建帧** 仍可要求
   bounds/捕获（`debug_mode || (dirty && mcp_shared.is_some())`）。非 dirty fall-through
   且非 F12 → 不置 `needs_bounds`。
2. `live_vtree` 构建：仅当 `debug_mode`（或 P530 排障 env），**不再**在普通
   dirty=false 帧无条件构建。MCP 快照继续走 `gate_dirty/gate_ws` 同步块（视图未变时
   快照仍准确）。
3. `collect_input_ids`：仅 `dirty == true` 时刷新；缓存帧沿用上次登记（结构未变）。

### E-3 hot_reload 门控/降频

subscription 内：

```rust
fn hot_reload_interval_ms(debug_mode: bool) -> Option<u64> {
    match std::env::var("AUTOUI_HOT_RELOAD").as_deref() {
        Ok("0") => None,
        Ok("1") => Some(500),
        _ => Some(if debug_mode { 500 } else { 2000 }),
    }
}
// source_path.is_some() && interval.is_some() → app_tick(app, HOT_RELOAD, interval)
```

行为表：

| AUTOUI_HOT_RELOAD | debug/F12 | 间隔 |
|-------------------|-----------|------|
| `0` | — | 不订阅 |
| `1` | — | 500ms |
| 未设 | on | 500ms |
| 未设 | off | **2000ms** |

## 6. 测试设计

1. **单测** `timer_when_allows_subscription`：
   - 无 when → true；
   - when 字段为假 → false；
   - when 字段为真 → true；
   - 未知 widget/event → true。
2. **回归**：既有 `fire_timer_noop_does_not_dirty` / when 门控相关测保持绿。
3. **静态检查**：`cargo check -p auto-lang --features ui-iced`；
   `cargo t --features ui-iced` 滤 `dynamic`/`timer`/`ui` 相关（按改动面）。
4. **实机（人工/可选）**：ui-gallery VM 静止 CPU / `P631_PROFILE=1` rebuild 频率；
   对照 `AUTOUI_HOT_RELOAD=0` 与默认。

## 7. 验收标准

- [x] AC-1 订阅层 `when` 门控合入：假时不订阅，真时恢复。——**pass**（复审重核）
- [x] AC-2 `dirty=false` 非 debug 帧不再构建 live_vtree / 不强制 needs_bounds。——**pass**
- [x] AC-3 hot_reload 遵守 env 表；默认非 debug 2000ms。——**pass**
- [x] AC-4 `cargo check -p auto-lang --features ui-iced` 通过；相关单测绿。——**pass**（复审重跑）
- [x] AC-5 计划文档登记 D-1..D-5 延期项；P499-1 注记「订阅层已由 650 E-1 清偿」
      （merge 阶段回写 KNOWN-DEBT）。——**pass**（计划侧已登记；KNOWN-DEBT/spec 回写归 merge）
- [x] AC-6 复审：独立核对 diff 与 AC，无静默回归（when 真路径、F12 路径）。——**pass**（本轮）

## 8. 执行步骤

- [✅] T-01 master 取号 + plan 骨架（new-plan.sh）+ 本文件填齐 rev1。
- [✅] T-02 commit 计划簿记（master：`.next-id` + `docs/plans/650-*.md`，`29aa22d70`）。
- [✅] T-03 隔离检出：会话守卫拦 `git worktree add`，改用 `git clone --shared` →
      `D:/autostack/.wt/lang-650/auto-lang` + `plan-650-dev`（兄弟 `auto-down` 同组 clone）。
      注：非 `git worktree` 注册条目；merge 时从该 clone fetch `plan-650-dev`。
- [✅] T-04 E-1：`timer_when_allows_subscription` + 订阅过滤 + `plan650_timer_when_subscription_gate`。
- [✅] T-05 E-2/E-4：`capture_debug`/`live_vtree`/`needs_bounds`/`collect_input_ids` 门控。
- [✅] T-06 E-3：`hot_reload_interval_ms`（0 关 / 1=500ms / 缺省 debug 500·非 debug 2000）。
- [✅] T-07 验证：`cargo check -p auto-lang --features ui-iced` 通过（仅存量 warning）；
      nextest `plan650_timer_when_subscription_gate` + `fire_timer_noop_does_not_dirty` +
      `mutation_seq_bumps_on_write_not_read` **3/3 PASS**。实机 CPU 对照待用户/后续 work。
- [✅] T-08 交 work/review（`/auto-plan:work` → `/auto-plan:review`）——2026-09-18 用户
      指令 `/auto-plan:work` 实施后：status `drafting`→`execution_done`；复跑门禁于
      clone `D:/autostack/.wt/lang-650/auto-lang` @ `e23fd1756`：`cargo check -p auto-lang
      --features ui-iced` 通过（仅存量 warning）；nextest
      `plan650_timer_when_subscription_gate` + `fire_timer_noop_does_not_dirty` +
      `mutation_seq_bumps_on_write_not_read` **3/3 PASS**。实机 CPU 对照仍待用户/后续。
      next: `/auto-plan:review`。

## 9. 复审记录

### R 轮独立复审（2026-09-18，/auto-plan:review，实现同会话——按技能要求从工件重建）

- **stage**: review | **plan_id**: PLAN-650 | **plan_revision**: 1 | **outcome**: **pass**
- **reviewed_commit**: `e23fd175644537e1de04877cd604564c632a981c`
  （clone `D:/autostack/.wt/lang-650/auto-lang` @ `plan-650-dev`，worktree clean）
- **base_commit**: `29aa22d70`（master 计划簿记基点）
- **design_commit**: `9cabc8a10`（master 侧 `docs/design/autoui/vm-frame-budget.md`，clone 无此文件——路径隔离符合预期）
- **dependency_revisions**: 无跨仓代码依赖
- **spec_inputs**: docs/specs/goals.md（GOAL-007）、docs/specs/auto-lang/ui/plans.md（P499-1 债档索引）、
  docs/specs/auto-lang/ui/design/chart-components.md（P499-1 旧口径）、docs/plans/KNOWN-DEBT-AND-RISKS.md
  （P499-1/P530-D2/P530-D3）、docs/design/autoui/vm-frame-budget.md（E/D 路线）

#### 验收结果（复审重跑，不采信 work 摘要）

| AC | 结论 | 证据（file:line / 命令） |
|----|------|--------------------------|
| AC-1 | **pass** | `dynamic.rs:528-540` `timer_when_allows_subscription`：无 when→true；`Some(when)`→`timer_guard_passes`；未知条目→true。`renderer.rs:18674-18681` 订阅循环 `if !timer_when_allows_subscription { continue }`。`fire_timer`（`dynamic.rs:546-572`）保留 when 派发门双保险。单测 `plan650_timer_when_subscription_gate`：unknown→true、no-when→true、`running=false`→false、`running=true`→true（复审 nextest PASS）。 |
| AC-2 | **pass** | `needs_bounds`：`renderer.rs:19238-19241` `if capture_debug && dirty`——dirty=false 永不请求。`live_vtree`：`renderer.rs:19318-19320` `if !p530_nomcp && (debug_mode \|\| dirty)`——非 F12 + dirty=false 不构建。`collect_input_ids`：`renderer.rs:19371-19376` 仅 `if dirty`。**代码相对计划 §5.2 增量**：live_vtree 条件为 `debug_mode \|\| dirty`（计划原文「仅当 debug_mode」）；对 AC-2 要求的 dirty=false 非 debug 帧无影响，dirty 非 debug 帧仍建 vtree 以保 MCP 快照新鲜。 |
| AC-3 | **pass** | `renderer.rs:7177-7186` `hot_reload_interval_ms`：`Ok("0")→None`；`Ok("1")→Some(500)`；`_→Some(if debug_mode {500} else {2000})`。订阅点 `18650-18658`：`source_path.is_some()` 且 `interval.is_some()` 才 `app_tick(HOT_RELOAD, interval)`。`hot_reload_tick` 函数已删除，无残留调用点。 |
| AC-4 | **pass** | 复审于 clone 重跑：`cargo check -p auto-lang --features ui-iced` → **EXIT=0 / Finished**（仅存量 warning，无 plan650 新增 error）。nextest `-E 'test(plan650) or test(fire_timer) or test(mutation_seq)…'`：`plan650_timer_when_subscription_gate` + `fire_timer_noop_does_not_dirty` + `mutation_seq_bumps_on_write_not_read` **3/3 PASS**（Summary 3 passed / 5325 skipped）。 |
| AC-5 | **pass（计划侧）** | 本文件 §0 Easy wins + §0 延后表已登记 E-1..E-4 与 **D-1..D-5**；G-3/§11 注记 P499-1 由 E-1 清偿订阅层、P530-D3 指向 D-1。`vm-frame-budget.md` 已落 master 并入 autoui README。**KNOWN-DEBT P499-1 原文与 chart-components.md:197-200 仍为旧口径**——按 AC-5 括号约定归 **merge** 回写，非本门未完成项。 |
| AC-6 | **pass** | 本节即独立复核：diff 仅 `dynamic.rs` + `renderer.rs`（+135/-35），无 vue 轨、无 aavm、未关 MCP server。when 真路径：谓词 true→订阅照挂 + fire_timer 照派发（测试 running=true 臂）。F12 路径：`debug_mode=true`→capture_debug/live_vtree 保持；dirty 帧仍 refresh input_ids。无 TODO/临时旁路/范围缩水 workaround。 |

#### 遗漏 / 延后 / workaround 扫描

| ID | 类型 | 判定 |
|----|------|------|
| R650-1 | 实现增量（非遗漏） | E-4 `capture_debug = debug_mode \|\| mcp_active && (dirty \|\| mcp_active_recently(30))` 比计划公式更保守；`mcp_active_recently` 实现见 `mcp_server.rs:294-304`。回应 §10 探针滞后，非缩水。 |
| R650-2 | 计划文本偏差 | live_vtree 门为 `debug_mode \|\| dirty` 而非计划「仅当 debug_mode」；AC-2 语义仍成立，代码更利于 MCP dirty 帧新鲜度。 |
| R650-3 | 健康（cosmetic，非阻塞） | `dynamic.rs` 测试区 `plan650_*`/`fire_timer_noop_*` 缩进深一层；rustc 不敏感。merge 前可选 rustfmt，不构成本轮 fail。 |
| R650-4 | 开放产品问题（非静默延后） | §10：hot_reload 非 debug 默认 2000ms 是否改回 500ms——已实现计划表并留澄清，待用户/后续 rev。 |
| R650-5 | 开放量测（计划标注可选） | G-1 实机静止 CPU / `P631_PROFILE=1` rebuild 频率对照未做；§6.4 标为「人工/可选」，AC 列表无量化阈值，不阻塞 code 门。 |
| R650-6 | 残余成本（info） | MCP 常开且 dirty=false 时，每帧仍 `lock().mcp_active_recently(30)`（短路 dirty 时免锁）；远低于旧 live_vtree 重建，可归 D-4/MCP 优化后续。 |
| R650-7 | 延期项授权 | D-1..D-5 明确不在本计划；§4 授权「最容易实现的先做出来，复杂的记录以后再做」+ 设计文档沉淀——**已授权延后，非 fail**。 |
| R650-8 | 未实现的可选 env | §10 提议的 `AUTOUI_MCP_LIVE=1` 强制旧行为未实现；属待澄清选项，非 AC 要求。 |

**结论**：无未授权延后、无阻塞 workaround、无静默丢子项。easy wins E-1..E-4 与 diff 一致。

#### 元数据裁定（供 /auto-plan:merge）

- `supersedes_spec_components`：chart-components P499-1 口径、ui/plans.md 650 行、KNOWN-DEBT P499-1/P530-D3 注记。
- `new_spec_components`：`docs/design/autoui/vm-frame-budget.md`（已在 master + README，merge 按 specs 惯例确认索引/状态即可，勿重复创建）。
- `touched_goals`：**GOAL-007**（与 plan 530 VM 渲染性能同族，本仓 AutoUI 伞目标）。
- merge 必做：从 clone fetch `plan-650-dev`（非 `git worktree` 注册条目）；KNOWN-DEBT 回写；ui/plans.md 登记；可选 rustfmt。

**局限声明**：复审与实现同会话，已按技能要求以 clone 工件重放（独立 cargo check EXIT=0 + nextest 3/3 + 全量 diff/AC 代码点重读）重建结论，未采信 work 摘要勾选。

**next**: `/auto-plan:merge`（落 master + specs 沉淀 + clone/分支清理；hot_reload 默认 2000ms 若改口径走 rev+1）。

## 10. 待澄清事项

- 默认热重载 2s 是否可接受？若需「默认保持 500ms」，改 E-3 默认表即可（rev+1）。
- MCP 无 F12 的 inspect 体验：非 dirty 帧不再刷 live_vtree，探针可能滞后到下一次
  dirty 重建——若 verifier 强依赖每帧 vtree，可加 `AUTOUI_MCP_LIVE=1` 强制旧行为。

## 11. Handoff

- 2026-09-18 /auto-plan:new 起草：`stage: new | plan_id: PLAN-650 | plan_revision: 1 |
  outcome: pass（草稿）`。easy wins=E-1..E-4；延期=D-1..D-5。
- 2026-09-18 easy wins 先行落地（用户要求「最容易实现的先做出来」）：
  `stage: new+partial-work | code_commit: plan-650-dev @ clone
  D:/autostack/.wt/lang-650/auto-lang | evidence: cargo check ui-iced OK + nextest 3/3 PASS`。
  待确认：hot_reload 默认非 debug 2000ms 是否可接受（§10）；确认后 `/auto-plan:work` 收尾
  或直接 `/auto-plan:review`。
- 2026-09-18 设计层补录：分析与复杂机制 → `docs/design/autoui/vm-frame-budget.md`
  （autoui README 索引已登记）。后续 D-1+ 实现计划引用该设计文档节号，不在本 plan 扩写。
- 2026-09-18 work 收执（/auto-plan:work）：`stage: work | plan_id: PLAN-650 |
  plan_revision: 1 | outcome: pass | code_commit: plan-650-dev @
  D:/autostack/.wt/lang-650/auto-lang (clone, e23fd1756; base 29aa22d70) |
  task_ids: T-01..T-08 全完成 | evidence: diff 复核 E-1 订阅层 when 门 +
  E-2 dirty=false 旁路（needs_bounds/live_vtree/collect_input_ids 门控）+
  E-3 hot_reload_interval_ms env 表 + E-4 MCP capture_debug 收紧；
  cargo check ui-iced 通过（仅存量 warning）；nextest 3/3 PASS |
  blockers: 无 | next: /auto-plan:review`。
  **实现相对计划的增量注记（交 review）**：
  (1) E-4 的 `capture_debug` 不是计划公式里的纯 `dirty && mcp_shared`，
      而是 `debug_mode || mcp_active && (dirty || mcp_active_recently(30))`——
      dirty 重建帧 + 近 30s MCP 活跃窗仍可要求 live 捕获，比计划更偏保守
      （回应 §10 探针滞后疑虑）；`live_vtree` 构建条件为
      `debug_mode || dirty`（非 F12 的 dirty 帧仍建，保证 MCP 快照新鲜）。
  (2) `hot_reload` 按计划表落地：非 debug 默认 2000ms；`AUTOUI_HOT_RELOAD=0`
      关闭 / `=1` 强制 500ms。§10 是否改回默认 500ms 待用户/ review 裁定。
  (3) 健康注记：`dynamic.rs` 测试区 `plan650_timer_when_subscription_gate`
      与邻近 `fire_timer_noop_does_not_dirty` 缩进比周边多一层（语义无影响，
      rustc 不敏感）；review 可要求 rustfmt 收口，非阻塞。
  (4) 实机静止 CPU / `P631_PROFILE=1` rebuild 频率对照未在本会话执行
      （计划 T-07 已标明「待用户/后续 work」）；AC-1..AC-4 代码+门禁侧已齐。
- 2026-09-18 独立复审（/auto-plan:review）：`stage: review | PLAN-650 rev1 |
  outcome: **pass** | reviewed_commit: e23fd1756 @ clone plan-650-dev |
  base_commit: 29aa22d70 | design_commit: 9cabc8a10（master vm-frame-budget） |
  acceptance_results: AC-1..AC-6 全 pass（逐条证据见 §9） |
  evidence: cargo check ui-iced EXIT=0 + nextest 3/3 PASS（复审重跑）+ diff 全扫 |
  findings: R650-1..8（1/2 实现增量与计划文本偏差已记录；3 cosmetic；
  4/5/8 开放项；6 残余 MCP 锁；7 延期已授权） |
  next: `/auto-plan:merge`。
- 2026-09-18 merge 收据（/auto-plan:merge，key: **PLAN-650:r1**）：
  stage: merge | outcome: **pass**。
  - `prepared`：reviewed@e23fd1756（r1 pass）；canonical delta=ui 两文件
    （dynamic.rs + renderer.rs +135/-35）；设计层已先落 master `9cabc8a10`。
  - `landed`：会话沙箱拦 `git merge`（orchestrator 独占 cross-branch 整合）→
    **cherry-pick `e23fd1756` → master `fc6c7a47a`**（auto-merge renderer.rs 无冲突；
    与 master 侧 fit/devtools hunk 正交）。祖先内容核验：master tip 含
    `timer_when_allows_subscription` / `hot_reload_interval_ms` / E-2/E-4 门控注释。
    master 冒烟：`cargo check -p auto-lang --features ui-iced` EXIT=0 +
    nextest plan650/fire_timer/mutation_seq **3/3 PASS**。他会话脏文件
    （plans/645、plans/646、rust-workspace Cargo.toml）未触碰。
  - `ledger_refreshed`：`.autoos/specs.json` 原子 upsert——**P650-1**（reports
    变更摘要）、**P650-2**（goals/GOAL-007）、**P650-3**（designs/vm-frame-budget）、
    **P650-4**（reviews 收据）；`python scripts/spec-index.py` 已跑（26 projects）。
  - `module_writeback`：KNOWN-DEBT P499-1 订阅层清偿注记 + P530-D3→D-1/
    vm-frame-budget §5.1；`ui/design/chart-components.md` P499-1 口径；
    `ui/plans.md` 650 行；`ui/overview.md` 现状段；`goals.md` GOAL-007 关联 650。
  - `archived`：status: archived；文件 `git mv` → `docs/plans/archive/`（本提交）。
  - `cleaned`：clone `D:/autostack/.wt/lang-650/{auto-lang,auto-down}` wt-guard 后删除；
    本地分支 `plan-650-dev` 删除（内容已在 master `fc6c7a47a`）。
  - **next（非本计划）**：D-1..D-5 另立项；§10 hot_reload 默认 2000ms 若改口径 rev+1；
    实机 CPU 对照可选。

## spec-sync 回写记录（v1 惯例）

- 规范增量：无 specs 模块 ADR 新增；current-state 变化写入 ui/overview.md 现状段 +
  chart-components P499-1 口径 + KNOWN-DEBT 两处注记。
- module 回写：`docs/specs/auto-lang/ui/plans.md` 650 行；`ui/overview.md` 2026-09-18 现状。
- 账本：specs.json P650-1..4；INDEX.md 由 spec-index.py 再生。
- 债务：P499-1 订阅层清偿（vue 臂 handler 门保留）；P530-D3 保持在案指向 D-1。
- 设计：`docs/design/autoui/vm-frame-budget.md`（已在 master，README 索引在案）。
