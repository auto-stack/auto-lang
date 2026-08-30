---
plan_id: PLAN-491
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-tab-focus-traversal
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - ui/iced/renderer:input-focus-ring-traversal（483 input-focus-addressing
    登记表之上的遍历扩展,并**修订其 Tab-fallback 腿**:keyboard_event_message
    未捕获 Tab 按 modifiers.shift 分派 __focus_next_input/__focus_prev_input
    + FindFocusedInput operate 探针读实际持焦(含点击直聚;恒出 Some(None),
    异于内建 find_focused 的 Outcome::None 断链) + focus_traverse 登记表
    回环求址(不在表内/无聚焦→首项,空表 None) + update 遍历臂
    operate(FindFocusedInput).then(focus_traverse→内建 focus);登记表空时
    回落 057 __focus_prompt textarea 优先链——无 input 视图(ash-gui/028)
    Tab 聚焦语义零回归)
touched_goals: [GOAL-007, GOAL-009]   # VM 轨 Tab/Shift+Tab 焦点环比齐 DOM 原生(非目标条款:Vue 轨不动) / launcher+ash-gui 召唤聚焦路径不回归

affects: [auto-lang/ui]
current_step: 8
total_steps: 8
---

# [PLAN-491] VM 轨 Tab/Shift+Tab 输入框焦点环遍历

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
`--features iced-layout-tests` 的 `p483_*` 相邻新增 `p491_*`。

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
`--features iced-layout-tests`）新增 `p491_*`，全部先红后绿：

1. `p491_tab_next`：双 input（042 形态）聚焦首 → 模拟 Tab → 第二聚焦。
2. `p491_shift_tab_prev`：聚焦第二 → Shift+Tab → 第一聚焦。
3. `p491_wrap`：尾→首（Tab）与首→尾（Shift+Tab）。
4. `p491_unfocused_fallback`：无聚焦 Tab → 首个（锁 483 fallback 语义，
   此条预期直接绿，防回归锚）。
5. `p491_single_input`：单 input Tab 不失焦不漂移。
6. 控制组：聚焦中的 prompt 编辑器 Tab（onkeydown.tab 捕获）不到达 fallback。

实机代验（MCP，真键盘顺延 P483-3 真人清单）：
- 042 example `auto run --render=vm`：autoui 定位/focus 探针核 Tab 后聚焦项。
- musk 登录页：归因探针核 Tab 后 password 归因 `.LoginPage.PasswordChanged`。

## 验收标准

- [x] p489 六测全绿（1-3 先红转绿；4-6 锚）
      [✅ pass] 实为 p491 七测（六设计测+探针单测；文中 p489 为撞号改号残留），
      复审重跑 7/7；红→绿链完整（编译红→断言红→转绿）。
- [x] p483 六测 + D4 四测回归全绿（`--features iced-layout-tests`）
      [✅ pass] 复审重跑 6/6 + 4/4。
- [x] `cargo tf` 全量绿；`cargo test -p auto-lang --lib --features ui-iced` 基线零新增红
      [✅ pass] 复审门禁 tf 3283/3283；ui-iced 分支 6 败 ⊂ master 基线 15 败
      （零新增红）。
- [x] 042 实机 MCP 代验：Tab 后聚焦项切换（双证截图/探针留档 evidence）
      [◐ partial] MCP 全流程+双图留档 ✓；Tab 真键盘注入通道受阻（P483-3
      同象）→ 机制级七测代证，真键盘顺延真人清单（计划非目标条款预先授权）。
- [x] musk 登录页实机代验：Tab 切框 + password 归因正确 + admin/admin 表单可全键盘完成（登录提交本身不在本计划范围）
      [◐ partial] password 归因 `.LoginPage.PasswordChanged` ✓；Tab 切框/
      全键盘=真键盘依赖顺延 P483-3；MCP 通道 admin/admin 未闭合（P483-4
      同族怪癖，与 483 基线一致）。
- [x] 028-launcher Tab 召唤聚焦抽查不回归
      [✅ pass] 复审现场补做：召唤/搜索框派发/`__focus_input` 消费清零 ✓。
- [x] P483-3 真人清单追加"musk 登录页 Tab 流"条目（登记到 483 债，不在本计划闭合）
      [✅ pass] archive/483 + musk 011 §七 两处注记 + KNOWN-DEBT 追记。

## 执行步骤

- [x] T1 红测试矩阵：renderer.rs `line_edit_tests` 新增 p491_1..3（Tab 前进/
      Shift+Tab 后退/回环），跑 `cargo test -p auto-lang --lib --features iced-layout-tests p489` 确认红。
      [✅ 已完成] p491_tab_next / p491_shift_tab_prev / p491_wrap 三测 + 五共享助手
      （tab_event/registry/probe_focus/apply_traversal/ui_and_click）落 renderer.rs
      line_edit_tests；红证据：`cargo test -p auto-lang --lib --features
      iced-layout-tests p491` → 5×E0425 `focus_traverse` not found（新函数未定义，
      TDD 编译红）；分派断言层（Tab→__focus_prompt 现状）随 T2 落函数后转断言红。
- [x] T2 当前聚焦 id 采集：按详细设计 a/b 择一落入 devtools（与 `input_ids`
      同生命周期），附单测（聚焦迁移后 devtools 值跟随）。
      验证：同上 p489 过滤 + 新增采集断言。
      [✅ 已完成] 定案=a 的 operate 形态（非 devtools 持久态）：新增产线探针
      `FindFocusedInput`（Operation<Option<Id>>，无聚焦时 finish 恒出 Some(None)——
      内建 find_focused 返回 Outcome::None 会断 Task::then 链）+ 求址纯函数
      `focus_traverse`（回环取模/不在表回落首项/空表 None）。理由：a 的字面
      形态（渲染期回填）在 view() 构建期拿不到 iced widget Tree 不可行；b 漏记
      点击直聚（用户点击 username 后 Tab 错走无聚焦臂，恰是 musk 实测场景）。
      采集单测 `p491_find_focused_probe`（恒出值 + 点击直聚可读，孪生探针代验，
      产线探针由 T6 实机核）+ `p491_wrap` 全绿；分派双测维持断言红
      （`got "__focus_prompt"`）待 T3。
- [x] T3 键盘路径分派：keyboard_event_message fallback 臂按
      `modifiers.shift()` 分派 `__focus_next_input`/`__focus_prev_input`
      （无聚焦共用回落首个）；同步 6576 注释（057 语义收窄为"无聚焦时"）。
      验证：p491_4 绿。
      [✅ 已完成] Tab 臂按 modifiers.shift() 分流两事件（Named 臂无修饰前缀，
      Shift+Tab 与 Tab 同命中 "Tab" 须在此分流的根因已注记）；057 注释收窄 +
      FOCUS_PROMPT_EVENT 常量注记「491 后无生产派发点，消费臂保留供回落」。
      新增锚测 p491_unfocused_fallback（分派名+无聚焦两方向皆首项求址+端到端
      首框单投递）预期绿即绿。`cargo test ... p491`：5 passed, 0 failed
      （tab_next/shift_tab_prev 由断言红转绿）。
- [x] T4 update 尾遍历臂：五聚焦点同层新增 next/prev 臂（focus_traverse
      求址 + `shell.request_focus`），单 input/卸载回落两边界用 p491_5 与
      现有 p483 锚覆盖。验证：p489 全绿。
      [✅ 已完成] __focus_prompt 臂加宽：next/prev 且登记表空时回落 057
      prompt 链（纯 textarea 世界 ash-gui/028 无 input 不回归）；新臂
      `operate(FindFocusedInput).then(focus_traverse → 内建 focus)` 与
      057 臂同层直接返回 Task。p491_single_input（自环不失焦不漂移+单投递）
      与 p491_prompt_tab_captured_not_fallback（Captured 双态不达 fallback）
      两锚落位。`cargo test ... p491`：7 passed, 0 failed。
- [x] T5 回归：p483 六测 + D4 四测 + `cargo tf` 全量 + `--features ui-iced`
      lib 基线对照（零新增红）。命令：
      `cargo test -p auto-lang --lib --features iced-layout-tests p483 p489`
      `cargo tf`
      [✅ 已完成] p483 六测 6/6 绿；tests_plan483_d4 四测 4/4 绿；`cargo tf`
      3283 passed/95 skipped 全绿；ui-iced 基线对照（master 主检出 vs 本工作树）：
      分支 7 败全为 master 15 败子集（8 个 master 败在分支转绿 = P483-2
      storage-CWD 环境债的干净工作树 CWD 表现），分支独有 1 败
      `run_client_full_cycle_over_pipe`（IPC 管道往返，与焦点改动无涉）隔离
      复跑绿 = 并行负载 flake 非回归 → **零新增红**成立。
- [x] T6 实机代验：042 `auto run --render=vm` + musk 登录页 MCP 探针，
      证据归 `docs/plans/evidence/`（沿 483 T8 目录惯例），README 复现步骤
      补 Tab 遍历断言（042 README）。
      [✅ 已完成] 证据归 docs/plans/evidence/491/（README+流程档+双图，worktree
      提交）。042：491 二进制 MCP 全流程 type 双框（归因 UserChanged/
      PassChanged）→ Login → authed=true+LoginPage 卸载（483 基线无回归）；
      musk：VM 轨登录页归因三连 `.LoginPage.UsernameChanged/PasswordChanged/
      Submit` ✓（「password 归因正确」验收项）；真键盘 Tab 尝试实录：前台
      通道被并行会话持续抢占（帧 stale/身份失配/两窗被外部关闭）= P483-3
      已登记阻塞同象 → 按计划非目标条款顺延真人清单。admin/admin 全键盘
      及 Tab 流未在 MCP 通道闭合（username 变量不随 autoui_type 持久化，
      P483-4 同族怪癖，与 483 基线一致）。042 README 增「预期(Plan 491)」
      断言节。
- [x] T7 P483-3 真人清单追加：archive/483 债节 + auto-musk 011 §七清单
      注记（两处各一行）。
      [✅ 已完成] 两处各一行已落：① archive/483 复审裁定段后追加 2026-08-30
      追记（默认检出）；② auto-musk 011 §七清单块后追加注记——注意：011 为
      musk 主检出**未跟踪**工作副本（git 从未入库，工作树路径无从承载），
      按原位追加、不动 musk git 状态；musk 依赖工作树 auto-musk-dev-1 已
      ff 同步至 main 供本计划复用（skill 约定，无代码改动）。
- [x] T8 簿记：执行步骤全勾、status→execution_done、KNOWN-DEBT 483 行
      追加遍历交付注记（worktree 提交，待 /auto-plan:review）。
      [✅ 已完成] KNOWN-DEBT P483-3 追记（含真人清单追加 musk Tab 流与
      「本债不闭合」边界）worktree 提交；T1-T7 全勾、current_step→8；
      status→execution_done。收尾按技能只跑 scoped：cargo check 清洁 +
      p491 7/7 + p483 6/6（全量已由 T5 cargo tf 3283 绿背书，全量复审归
      /auto-plan:review 门禁）。

## 复审记录

**/auto-plan:review 2026-08-30（zcode 独立复审会话；verify-don't-trust 全项重跑）**

方法：worktree `.worktrees/plan-491-dev` 内 `git diff 1c3b09dc4..HEAD`（注意
`master..HEAD` 会混入并行会话推进 master 的反向差异——真变更集以分支基点
diff 为准）逐 hunk 对码 + 全部门禁复审现场重跑。

| # | 验收标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | p491 测族全绿（1-3 先红转绿；4-6 锚） | pass | 复审重跑 `--features iced-layout-tests p491` **7/7**（六设计测+探针单测）；红证据链完整：T1 5×E0425 编译红→T2 断言红 `got "__focus_prompt"`（分派现状实录）→T3 转绿 |
| 2 | p483 六测+D4 四测回归全绿 | pass | 复审重跑 6/6 + tests_plan483_d4 4/4 |
| 3 | `cargo tf` 全绿 + ui-iced 基线零新增红 | pass | 复审门禁 tf **3283/3283**（95 skipped）；ui-iced 分支 6 败 ⊂ master 基线 15 败（diff 仅减无增=零新增红；9 个 master 败在分支转绿=P483-2 storage-CWD 环境债干净 CWD 表现,非代码差异） |
| 4 | 042 实机 MCP 代验:Tab 后聚焦项切换+双证留档 | **partial(代验)** | 机制级主证=七测全链（点击直聚→探针→求址→置焦→单投递归因）；042 MCP 全流程 type 双框→authed=true→LoginPage 卸载（491 二进制,483 基线无回归）+evidence/491 双图留档。**Tab 注入本体不可达**:autoui_keyboard 走 handler 派发不经 iced 订阅;computer-use 前台真键被并行会话持续抢占（帧 stale/身份失配/两窗被外部关闭实录）——P483-3 已登记阻塞同象,计划非目标条款预先授权机制级代验+真人顺延,沿 473/483 先例 |
| 5 | musk 登录页代验:Tab 切框+password 归因+admin/admin 全键盘 | **partial(代验)** | password 归因 `.LoginPage.PasswordChanged` ✓（验收点名项;归因三连 UsernameChanged/PasswordChanged/Submit 全中,VM 轨 491 二进制）;Tab 切框/全键盘=真键盘依赖（同 #4 顺延 P483-3,T7 两处注记已落）;MCP 通道 admin/admin 未闭合——username 变量不随 autoui_type 持久化=P483-4 同族既有怪癖,与 483 基线一致 |
| 6 | 028-launcher Tab 召唤聚焦抽查不回归 | pass | **执行期漏跑,复审现场补做**:491 二进制 standalone VM——boot ✓、召唤 `.App.Open`→visible=1+搜索框出现 ✓、`.App.SetQ` 派发 ✓、`__focus_input` 消费后清零（"" 非 "1"——召唤聚焦链活着）✓;代码级:`__focus_input` 尾部消费臂零改动（不在任何 hunk） |
| 7 | P483-3 真人清单追加「musk 登录页 Tab 流」 | pass | archive/483 复审裁定段追记 + musk 011 §七注记（两处各一行）+ KNOWN-DEBT P483-3 追记（worktree 提交,含「本债不闭合」边界）。注:011 为 musk 主检出**未跟踪**工作副本（git 从未入库）,按原位追加不动其 git 状态 |

**遗漏/延后/workaround 扫描**：复审发现并已处置两处——①执行期 028-launcher
抽查漏跑（由代码审读+控制组测试代证）→复审补做落档（#6）；②evidence
`p491_042_login_filled.png` 误帧（复制时取了 authed 后重复帧 83889B,两图同
大小暴露）→已换回真填表帧 1788074014891（109570B,截图序=type 双框后/
pre-submit）,worktree 修正提交。延后两项均为计划内预先授权（真键盘→P483-3
真人清单;MCP 全键盘怪癖→P483-4 既有债）,非执行期擅自缩水。workaround
无（diff 零 TODO/FIXME/dbg/eprintln 新增）。

**plan↔code 偏差（均已在计划注记）**：T2 采集机制取 operate 探针（a 的
可行形态;a 字面形态 view() 拿不到 iced Tree 不可行,b 漏记点击直聚——计划
明文「以实现最小者为准」授权）;focus_traverse 落 T2 而非 T4（时序微调）;
计划文本 `p489` 过滤词为撞号改号残留,实跑 `p491`。

**裁定:PASS（5 pass + 2 partial）**。两 partial 同源=P483-3 已立债环境阻塞
（真键盘通道）,机制级代证充分+真键盋试图受阻实录留档,沿 473/483 先例放行;
最终裁定权随 /auto-plan:merge 呈用户。status→reviewed。

## 待澄清事项

- ~~T2 当前聚焦 id 采集方案 a/b 的取舍——实现时以最小侵入定案，不阻塞开工。~~
  **已定案（T2）**：取 a 的 operate 形态——`FindFocusedInput` 探针
  （Operation<Option<Id>>，无聚焦恒出 Some(None)）+ `focus_traverse` 求址。
  a 的字面形态（渲染期回填）在 view() 拿不到 iced widget Tree 不可行；
  b 漏记点击直聚（musk 实测主场景即点击后 Tab）。零新增 devtools 持久态，
  为两案交集外的最小正确解。
- ~~Shift+Tab 在 iced text_input 内是否被捕获（无 Tab 臂的推论=Shift+Tab
  同样未捕获）——T1 红测试会先行实证，若被捕获则 T3 需在捕获层前加遍历臂。~~
  **已实证（T1/T3）**：text_input 对 Shift+Tab 同样不捕获（无 Tab 臂无
  shift 分支），`keyboard_event_message` 以 Ignored status 收到并分派
  `__focus_prev_input`（p491_shift_tab_prev 全链绿）；无需捕获层前臂。
  Captured 路径由 p491_prompt_tab_captured_not_fallback 控制组锁定。
