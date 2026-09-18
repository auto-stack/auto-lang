---
plan_id: PLAN-650
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-idle-render-trim
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-007]

affects: [auto-lang/ui]
current_step: 7
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
| D-1 | **P530-D3 Element 帧间缓存**：`Rc<RefCell<Option<Element>>>` + 自定义 Widget 持有复用，或 iced `lazy` 子树 memo | iced 0.14 `Element` 不可 Clone；需 Widget/Tree 生命周期设计，回归面大 |
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

- [ ] AC-1 订阅层 `when` 门控合入：假时不订阅，真时恢复。
- [ ] AC-2 `dirty=false` 非 debug 帧不再构建 live_vtree / 不强制 needs_bounds。
- [ ] AC-3 hot_reload 遵守 env 表；默认非 debug 2000ms。
- [ ] AC-4 `cargo check -p auto-lang --features ui-iced` 通过；相关单测绿。
- [ ] AC-5 计划文档登记 D-1..D-5 延期项；P499-1 注记「订阅层已由 650 E-1 清偿」
      （merge 阶段回写 KNOWN-DEBT）。
- [ ] AC-6 复审：独立核对 diff 与 AC，无静默回归（when 真路径、F12 路径）。

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
- [ ] T-08 交 work/review（`/auto-plan:work` → `/auto-plan:review`）——本会话已先落 easy wins，
      status 仍 `drafting`，待用户确认后翻 `executing` 并走正式 review/merge。

## 9. 复审记录

（review 阶段填写）

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
