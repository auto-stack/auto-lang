---
plan_id: PLAN-489
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-tab-focus-traversal
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 8
---

# [PLAN-489] VM 轨 Tab/Shift+Tab 输入框焦点环遍历

## 变更摘要

VM(iced) 轨 text_input 目前没有焦点环遍历：聚焦 username 按 Tab 不会切到
password，未捕获 Tab 走 Plan 057 的 terminal-style fallback——聚焦登记表
**第一个** input（renderer.rs:6576-6587 → `__focus_prompt` → 登记表首个）。
Plan 483 已交付基建：input 唯一稳定 Id（`derive_input_id`）+ 渲染期 DFS 序
登记表（`state.app.devtools.input_ids`，每帧清填）+ 五聚焦点改址。本计划在
登记表上实现完整遍历：

1. **Tab 前进**：当前有 input 聚焦时，未捕获 Tab → 聚焦登记表中**下一个**
   input（DFS 序），到尾回环到首。
2. **Shift+Tab 后退**：未捕获 Shift+Tab → 聚焦**上一个** input，到头回环。
3. **无聚焦保持旧行为**：没有任何 input 聚焦时 Tab → 登记表首个（与 483
   fallback 一致；ash-gui prompt 编辑器捕获 Tab 的路径不受影响——Captured
   事件根本到不了该 fallback）。

需求源：auto-musk `docs/designs/011-vm-text-input-double-focus-requirement.md`
§三点五（"Tab 焦点遍历缺失……Plan 483 的登记表基建正好是它的承载面"）。
用户实测场景：musk 登录页 username 按 Tab 切 password。

## 目标

- 042-two-inputs-child 形态：聚焦 user 按 Tab → pass 聚焦；Shift+Tab 反向；
  尾→首/首→尾回环；无双焦点、无键盘双投递回归。
- musk 登录页（`login.at` 条件子 widget 双 input）：Tab 可完成
  username→password→button 的键盘流，admin/admin 全键盘可登录。
- 机制级测试（iced_test）全绿，`cargo tf` 全量无回归。

## 非目标

- 非 input 可聚焦件（button/select 等）的通用焦点环——登记表目前只收
  text_input；通用可聚焦件登记留后续计划。
- 鼠标点击聚焦路径不动（已正常）。
- P483-3 真人真键盘复验——本环境 OS 级键盘注入对 winit 无效，本计划交付
  机制级代验，并把 musk Tab 流并入 P483-3 真人清单。
- Web(Vue) 轨不动——DOM 原生提供 Tab 焦点环。

## 架构方案

修在 `crates/auto-lang/src/ui/iced/renderer.rs`，不动 .at 单一真源契约、
不动 musk 侧：

1. **键盘路径携带 shift 语义**：未捕获 Tab 的派发点（keyboard_event_message，
   ~6328-6339 采集 / 6576-6587 fallback 臂）。现 `key_str == "Tab"` 对
   Shift+Tab 同样命中（Named 臂不加修饰前缀）。改为按 `modifiers.shift()`
   分派两个事件：`__focus_next_input` / `__focus_prev_input`（无聚焦时的
   两者都落到登记表首个，等价现 `__focus_prompt` fallback 语义）。
2. **当前聚焦 id 可得**：遍历需知道"现在聚焦的是谁"。483 的 refocus 路径
   已持有聚焦地址；把"当前聚焦 input id"记录进 devtools（与 `input_ids`
   同生命周期：渲染期由 iced 焦点状态回填，或 focus operation 改址时同步
   写——以实现最小者为准，T2 定案）。
3. **update 尾遍历臂**：在五聚焦点同层（~9925 `__focus_input` /
   ~8506 Tab / 9889 refocus）新增 next/prev 两臂：查登记表
   `input_ids`（DFS 序即视觉树序），定位当前 id 下/上一项，
   `shell.request_focus(该 Id)`；找不到当前 id（无聚焦/已失焦）→ 首个。
   回环取模。
4. **ash-gui 回归守卫**：prompt 编辑器 onkeydown.tab 捕获路径（Captured）
   不经过 fallback，零改动；028-launcher 召唤聚焦抽查入回归。

## 技术栈

Rust / iced 0.14.2（`shell.request_focus`、Focus operation 按 Id 寻址）、
iced_test（机制级键盘模拟）。测试沿用 483 门面：
`--features iced-layout-tests` 的 `p483_*` 相邻新增 `p489_*`。

## 需求分析与背景调查

- spec 台账：P483 新组件 `input-focus-addressing`（唯一 Id+登记表+五聚焦点
  改址）与 `focus-id-prompt_input`（被 supersede）——本计划是
  input-focus-addressing 的遍历扩展。
- 上游需求书：auto-musk 011 §三点五（Tab 遍历缺失 + 承载面判断），及
  §七验收清单第 2 条"musk 登录页实测"。
- 483 根因结论（archive/483 §根因）：iced text_input 无 Tab 臂 → 未捕获
  Tab → `__focus_prompt` → `focus(prompt_input_id)` 全置焦。该缺陷已修，
  本计划只做遍历，不触焦点生命周期。
- 现状锚点：renderer.rs:6576-6587（Tab fallback 臂）、:13921/:13998
  （五聚焦点注释与改址）、483 交付的 `collect_input_ids` DFS 登记。

## 详细设计

### 事件契约

| 触发 | 旧 | 新 |
|---|---|---|
| 未捕获 Tab，有 input 聚焦 | `__focus_prompt`（跳首个） | `__focus_next_input`（下一个，尾回环） |
| 未捕获 Shift+Tab，有 input 聚焦 | 同上（bug：当 Tab 处理） | `__focus_prev_input`（上一个，首回环） |
| 未捕获 Tab/Shift+Tab，无 input 聚焦 | 登记表首个 | 登记表首个（不变，两事件同臂） |

### 遍历求址

```
fn focus_traverse(ids: &[InputId], current: Option<InputId>, dir) -> InputId {
    match current.and_then(|c| ids.iter().position(|&i| i == c)) {
        Some(p) => ids[(p + dir + ids.len()) % ids.len()],   // 回环
        None => ids[0],                                       // 无聚焦 → 首
    }
}
```

当前聚焦 id 的采集点在 T2 实现时二选一（择最小侵入）：
a) 渲染期从 iced `State` 焦点状态回填 devtools；
b) 五聚焦点 `request_focus` 改址时同步写"最后聚焦 id"。

### 与 483 语义的边界

- 登记表每帧清填：聚焦中的 input 被条件渲染卸载（如 musk 登录后 LoginPage
  整体卸载）→ current 不在表内 → 回落 `ids[0]`，与"无聚焦"同臂，安全。
- 单 input 场景：next/prev 都是自身（取模回环），行为=现 fallback，无回归。

## 测试设计

`crates/auto-lang/src/ui/iced/renderer.rs` `line_edit_tests`（`p483_*` 相邻，
`--features iced-layout-tests`）新增 `p489_*`，全部先红后绿：

1. `p489_tab_next`：双 input（042 形态）聚焦首 → 模拟 Tab → 第二聚焦。
2. `p489_shift_tab_prev`：聚焦第二 → Shift+Tab → 第一聚焦。
3. `p489_wrap`：尾→首（Tab）与首→尾（Shift+Tab）。
4. `p489_unfocused_fallback`：无聚焦 Tab → 首个（锁 483 fallback 语义，
   此条预期直接绿，防回归锚）。
5. `p489_single_input`：单 input Tab 不失焦不漂移。
6. 控制组：聚焦中的 prompt 编辑器 Tab（onkeydown.tab 捕获）不到达 fallback。

实机代验（MCP，真键盘顺延 P483-3 真人清单）：
- 042 example `auto run --render=vm`：autoui 定位/focus 探针核 Tab 后聚焦项。
- musk 登录页：归因探针核 Tab 后 password 归因 `.LoginPage.PasswordChanged`。

## 验收标准

- [ ] p489 六测全绿（1-3 先红转绿；4-6 锚）
- [ ] p483 六测 + D4 四测回归全绿（`--features iced-layout-tests`）
- [ ] `cargo tf` 全量绿；`cargo test -p auto-lang --lib --features ui-iced` 基线零新增红
- [ ] 042 实机 MCP 代验：Tab 后聚焦项切换（双证截图/探针留档 evidence）
- [ ] musk 登录页实机代验：Tab 切框 + password 归因正确 + admin/admin 表单可全键盘完成（登录提交本身不在本计划范围）
- [ ] 028-launcher Tab 召唤聚焦抽查不回归
- [ ] P483-3 真人清单追加"musk 登录页 Tab 流"条目（登记到 483 债，不在本计划闭合）

## 执行步骤

- [ ] T1 红测试矩阵：renderer.rs `line_edit_tests` 新增 p489_1..3（Tab 前进/
      Shift+Tab 后退/回环），跑 `cargo test -p auto-lang --lib --features iced-layout-tests p489` 确认红。
- [ ] T2 当前聚焦 id 采集：按详细设计 a/b 择一落入 devtools（与 `input_ids`
      同生命周期），附单测（聚焦迁移后 devtools 值跟随）。
      验证：同上 p489 过滤 + 新增采集断言。
- [ ] T3 键盘路径分派：keyboard_event_message fallback 臂按
      `modifiers.shift()` 分派 `__focus_next_input`/`__focus_prev_input`
      （无聚焦共用回落首个）；同步 6576 注释（057 语义收窄为"无聚焦时"）。
      验证：p489_4 绿。
- [ ] T4 update 尾遍历臂：五聚焦点同层新增 next/prev 臂（focus_traverse
      求址 + `shell.request_focus`），单 input/卸载回落两边界用 p489_5 与
      现有 p483 锚覆盖。验证：p489 全绿。
- [ ] T5 回归：p483 六测 + D4 四测 + `cargo tf` 全量 + `--features ui-iced`
      lib 基线对照（零新增红）。命令：
      `cargo test -p auto-lang --lib --features iced-layout-tests p483 p489`
      `cargo tf`
- [ ] T6 实机代验：042 `auto run --render=vm` + musk 登录页 MCP 探针，
      证据归 `docs/plans/evidence/`（沿 483 T8 目录惯例），README 复现步骤
      补 Tab 遍历断言（042 README）。
- [ ] T7 P483-3 真人清单追加：archive/483 债节 + auto-musk 011 §七清单
      注记（两处各一行）。
- [ ] T8 簿记：执行步骤全勾、status→execution_done、KNOWN-DEBT 483 行
      追加遍历交付注记（worktree 提交，待 /auto-plan:review）。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- T2 当前聚焦 id 采集方案 a/b 的取舍——实现时以最小侵入定案，不阻塞开工。
- Shift+Tab 在 iced text_input 内是否被捕获（无 Tab 臂的推论=Shift+Tab 同样
  未捕获）——T1 红测试会先行实证，若被捕获则 T3 需在捕获层前加遍历臂。
