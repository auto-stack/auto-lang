---
plan_id: PLAN-483
status: reviewed                 # drafting → executing → execution_done → reviewed → archived
feature_name: vm-text-input-double-focus
author: [zcode]
created_at: 2026-08-29
updated_at: 2026-08-29

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - ui/iced/renderer:focus-id-prompt_input（Plan 047「vm input 统一稳定 Id
    prompt_input」约定——被 input-focus-addressing 唯一 Id+登记表寻址取代;
    Plan 464 __focus_input 消费寻址同批改址）
new_spec_components:
  - ui/iced/renderer:input-focus-addressing（derive_input_id 每框唯一稳定 Id
    主键 widget+event/兜底 placeholder+width+password + devtools.input_ids
    渲染期 DFS 登记 + 五聚焦点寻址:Tab-fallback/tail-refocus/initial/
    __focus_input/launcher-summon→登记表首个,textarea 优先,空表退
    prompt_input_id 安全降级）
  - ui/mcp_server:vnode-path-alignment（find_view_by_path 子枚举委托
    vnode_converter::extract_children_ref 同构不变量 + AppState.
    mcp_sync_vtree 与 shared.view 同源快照取代 live_vtree 加工树覆盖）
touched_goals: [GOAL-007, GOAL-009, GOAL-014]   # VM 焦点/输入 parity 缺口闭合 / 桌面 Shell 焦点约定 / MCP autoui_type

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 8
total_steps: 9
---

# [PLAN-483] VM(iced) 双 input 双焦点/键盘双投递修复 + autoui_type 归因错位顺修

## 变更摘要

修复 VM(iced) 轨在「条件渲染的子 widget 内多 input」形态下的双焦点/键盘双投递
缺陷（两框同时焦点环、光标双闪，键盘文本双框各投一次），并顺修 MCP `autoui_type`
对第二个 input 派发错 handler 的 vnode path 归因错位。

**主缺陷根因（本仓调研实锤）**：VM 主路径 `render_dynamic_view` 的 Input 臂
（renderer.rs:13698-13700，Plan 047 cbfc1c761e）把每个 text_input 的 Id
**无条件覆盖为固定 `"prompt_input"`**——将 1a8516b5b 在 `build_input_shape`
（renderer.rs:2202-2211，PLAN-050）派生的唯一 Id 整体抵消。VM 窗口内所有
input（含外壳 prompt 输入框，session.rs:166 `prompt_input_id` 默认同字面量）
共享同一 Id；而 iced 0.14 `Focus` operation 对**所有** Id 匹配的 focusable
逐个 `state.focus()`（focusable.rs:29-57），任一 `focus("prompt_input")`
路径触发（`__focus_input` 约定 9925 / 未捕获 Tab 8506 / refocus 9889 / 初始
聚焦 9905）即全部置焦；键盘事件 Column 无条件扇出 + text_input 仅按
`state.is_focused` 过滤（text_input.rs:910）→ 双框都消费。

修复主干：**逐 input 唯一稳定 Id + 渲染期 Id 登记表 + 自动聚焦寻址改址**；
槽位稳定/unfocus 清焦作为条件分支（依诊断任务 T3 结论启用）。清偿上游需求
auto-musk `docs/designs/011-vm-text-input-double-focus-requirement.md`
（auto-musk PLAN-050 用户验收发现，KD-048 族 VM widget 层债）。

## 目标

1. **单焦点互斥**（需求 §一1/§七）：042 最小 example 中点击第二个 input 后
   仅该 input 有焦点环，第一个 input 失焦。
2. **键盘单投递**（§一2/3）：击键只进聚焦框——聚焦 password 输入 "admin"
   时 username 值不变；两 handler 各收各的（UsernameChanged/PasswordChanged
   不串）。
3. **musk 登录页可用**（§七）：条件装配的 LoginPage 双框独立，admin/admin
   登录流程可完成（真键盘实测）。
4. **不回归既有焦点约定**：Plan 047（view 重建后聚焦框不丢焦、on_submit
   触发）、Plan 464（`__focus_input` 打开即聚焦）、Plan 057（Tab→prompt
   聚焦）、launcher 召唤聚焦（6744）均保持；003-converter 根级双输入双向
   联动不回归。
5. **顺修 autoui_type 归因**（§四）：对第二个 input 的 vnode id 执行 type
   派发其自身 handler；补 path 对齐回归测试。
6. **范围外**（登记 debt，不实现）：aura_N 快照流 scroll/container/grid
   源索引编号错位嫌疑（D-GAP-4 同型）；desktop 多 App 焦点命名空间（462
   既有债）；不动 `.at` 单一真源契约、不动 auto-musk 侧代码。

## 架构方案

**选型：方向 A——Id 唯一化 + 登记表寻址（主），槽位稳定/unfocus（条件分支）。**

三个候选：
- **A. 唯一稳定 Id + 渲染期登记表**：每个 input 按稳定键（widget+event 为主，
  placeholder/width/password 兜底）派生唯一 Id；渲染期把当前视图 input Id 的
  遍历序登记进 `state.app.devtools`；`__focus_input`/Tab/refocus/launcher
  等自动聚焦点从登记表取首个 input 的 Id 寻址。选此：直击已实锤的 Id 撞车
  根因，改动集中在 renderer.rs 四个消费点 + Id 派生调用点，不动容器路径。
- **B. iced 0.14.2 `keyed::Column` 槽位复用**：仅治槽位错位（若诊断证实），
  不治 focus operation 同 Id 全置焦；且需改所有容器构建路径，改动面大。
  不选为主干；若 T3 证实槽位粘焦，局部挂载点用「恒定占位」即可（false 分支
  渲染占位节点而非删除），不必引入全局 keyed。
- **C. 全局 unfocus 清焦**：治标（每帧清焦再聚焦引入闪烁）。仅作条件分支：
  条件卸载子树时若诊断证实僵尸焦点，在卸载路径发一次 `operation::unfocus`。

数据流（一帧渲染 + 一次聚焦请求）：

```text
view_dirty → dynamic_view(state) → render_dynamic_view 逐 Input 臂：
  ├─ 派生唯一稳定 Id（widget+event 主键）→ TextInput.id(唯一 Id)
  │    （删除 13700 的 "prompt_input" 整体覆盖）
  └─ 登记表：state.app.devtools.input_ids ← 遍历序追加（每帧先清后填）
update 尾部自动聚焦点（9925 __focus_input / 8506 Tab / 9889 refocus /
9905 初始 / 6744 launcher 召唤）：
  └─ 目标 Id = 登记表首个（或 launcher 目标 App 的首个）→ focus(唯一 Id)
     → iced Focus operation 恰命中一个 focusable（互斥成立）
键盘 KeyPressed → Column 扇出 → 仅 is_focused 的那一个 text_input 消费
```

## 技术栈

Rust（crates/auto-lang）；iced 0.14.0 runtime + iced_widget 0.14.2（官方
registry，无 fork）；`iced_test` 0.14 headless 测试台（layout_tests.rs 先例，
Plan 414）；AutoUI VM 渲染层（ui/iced/renderer.rs）+ MCP（ui/mcp_server.rs、
snapshot_builder.rs、vnode.rs）；验证脚本 `.agents/skills/autoui-verifier/`。

## 需求分析与背景调查

（取材：docs/specs/overview.md、上游需求 011 文档、本仓代码勘验 2026-08-29）

**规格定位**：GOAL-007（AutoUI 跨端一致——本缺陷为 VM 轨行为 parity 缺口）
为主；GOAL-009（桌面 Shell 焦点约定 `__focus_input`/prompt bar 属其地基）
与 GOAL-014（MCP 工具链 autoui_type）关联。涉及 module：`auto-lang/ui`
（renderer / aura_view_builder / mcp_server / session）。

**上游诊断链（已排除项，需求 §二）**：视图层消息归属正确（View 树断言
username/password 各挂各 handler）、派发回写层正确（on_with_input_for 各写
各字段）、根级双输入对照组（003-converter）正常、输入管线无回归。缺陷限定
形态=条件渲染的**子 widget**内多 input。

**本仓勘验证据**（主缺陷）：

1. **Id 派生与抵消**：`build_input_shape`（renderer.rs:2194，派生 2202-2211，
   提交 1a8516b5b）按 placeholder+width+password 派生稳定唯一 Id；但 VM 主路径
   `render_dynamic_view` Input 臂 renderer.rs:13698-13700（提交 cbfc1c761e，
   Plan 047）无条件 `.id(Id::new("prompt_input"))` 覆盖——**新修复在 VM 路径
   被旧行整体抵消**（blame 证实两行分属先后提交）。泛型 `IntoIcedElement`
   路径（renderer.rs:3097 臂）不受覆盖，仅服务 rust 模式。
2. **共享 Id 的消费面**：`__focus_input=="1"` 消费 → `focus("prompt_input")`
   （renderer.rs:9917-9928，Plan 464，注释自认「desktop 多 App 同 Id 冲突为
   v1 已知边界」）；未捕获 Tab（8490-8506）、refocus（9883-9893）、初始聚焦
   （9897-9909）fallback `state.app.devtools.prompt_input_id`——session.rs:166
   默认即 `Id::new("prompt_input")`；launcher 召唤（6744）同字面量。
3. **iced 0.14.2 语义**（registry 源码实证）：`Focus` operation 遍历全部
   focusable，Id 匹配即 `state.focus()`（iced_core focusable.rs:29-57）——
   同 Id 全置焦；text_input 点击设焦 `state.is_focused = 点击在框内?Some:None`
   （text_input.rs:718-735，靠事件扇出互斥）；键盘守卫仅判 `is_focused`
   （text_input.rs:910）——双 Some 即双投递；`TextInput::diff` 不 reconcile
   焦点（text_input.rs:656-663）——焦点态粘在 Tree 槽位。
4. **Tree 配对按索引**：`Tree::diff_children` 纯索引 zip + 尾部
   truncate/extend，无 key 复用（iced_core tree.rs:71-107）；本仓条件扁平化
   （aura_view_builder.rs:772-796）+ `is_visually_empty` 过滤使 children 收缩
   /位移（3723 邻域）——槽位错位粘焦为次级候选机制。
5. **musk 勘验**（D:\autostack\auto-musk\src\front）：login.at 两 input
   placeholder 不同（"Enter username"/"Enter password"——派生 Id 本可区分）、
   无 `__focus_input` 写入；结构=app.at `if store.authenticated != true {
   LoginPage }` 条件装配 + 表单内 `if store.error`/`if loading` 条件块；
   handler 为 v-model marker（`.UsernameChanged -> { .username = .username }`）。
6. **对照组不对称的解释**：根级双输入无 focus operation 介入，点击扇出互斥
   成立故正常；子 widget 条件形态触发共享 Id 的 focus 路径（候选：子组件
   Init 每脏重建重放 fire_child_init_if_any 的状态联动 / Tab / refocus /
   条件翻转重建），确切触发器由 T3 探针判定。
7. **测试台**：layout_tests.rs（iced_test::Simulator：`find` 文本选择器 /
   `click` / `typewrite` / `tap_key` / `into_messages`）可承载真焦点语义
   红测试；render_dynamic_view 为 renderer 模块私有函数，测试置于同模块
   #[cfg(test)] 直接调用。

**本仓勘验证据**（次要缺陷 autoui_type）：

- 执行链：`tool_type`（mcp_server.rs:1673）→ `execute_action_vnode`（:2207）
  → `VTree::get` 首匹配（vnode.rs:441）→ `find_view_by_path`（mcp_server.rs:
  2105，**与 vtree 构建侧 `extract_children`（vnode_converter.rs:426）不同构**，
  缺 Button{content}/Table/Tabs 臂）→ `extract_action_from_view`（:2153）→
  `send_action` → `on_with_input_for`（dynamic.rs:943）。
- **styled_vtree 双生产者**：renderer.rs:10820-10836（dynamic_view 内，与
  shared.view 同一 fresh view，自洽）vs renderer.rs:8302-8313（`__bounds_collected`
  臂用 `StyledNodeSnapshot::from_live(live_vtree)` **覆盖**——live_vtree 由
  `converted` 树构建（经 inject_todo_list/patch_input_values/
  convert_view_messages），与 shared.view 结构可错位 → path 偏移 →
  find_view_by_path 落错节点 → 静默错派发（musk 实证：password id →
  UsernameChanged）。
- mcp_server.rs 测试模块（:2928-3045）无任何 path 对齐测试。

## 详细设计

### D1. Id 唯一化（renderer.rs）

- 删除 renderer.rs:13698-13700 的整体覆盖；Input 臂在接好 on_change 后以
  **稳定键**设 Id：主键 `format!("auto_input_{}_{}", on_change.widget,
  on_change.event)`（无 on_change 时兜底 `{placeholder}_{width}_{password}`，
  与 build_input_shape 现三元组一致）。派生逻辑收敛为一个纯函数
  `derive_input_id(widget: Option<&str>, event: Option<&str>, placeholder,
  width, password) -> iced::widget::Id`，供 render_dynamic_view 与
  `IntoIcedElement`（3097 臂）共用；`build_input_shape` 内的派生行移除
  （改由调用点显式设置，避免双处派生）。
- 稳定性论证：widget+event 名在 .at 单一真源中静态稳定，跨重建不变；比
  placeholder 三元组更强（musk 两框 placeholder 亦异，但主键化后同参输入
  也能区分——跨 widget 同 placeholder 场景不再撞车）。
- Plan 047 语义保全：iced daemon 的 Tree 状态按槽位 diff 保存焦点，Id 的
  真实作用是 focus operation 寻址；改址后聚焦框跨 view_dirty 重建仍按槽位
  保持（T5 真机验证 on_submit）。

### D2. 渲染期 Id 登记表（session.rs + renderer.rs）

- `AppSession.devtools` 增 `input_ids: RefCell<Vec<iced::widget::Id>>`；
  `dynamic_view`（renderer.rs:10871，现场可及 state.app.devtools）在构建
  Element 前清空、`render_dynamic_view` Input 臂按遍历序追加。
- 自动聚焦点改址（全部从登记表取首个，空表回退现状 none/prompt 语义）：
  - renderer.rs:9925 `__focus_input` 消费 → `focus(登记表首个)`；
  - renderer.rs:8505/9889/9905 fallback（last_textarea 缺席时）→ 登记表首个；
  - renderer.rs:6744 launcher 召唤 → 目标 App 登记表首个（多窗口 daemon 按
    AppId 定位，459 扇出先例）；
  - session.rs:166 `prompt_input_id` 保留为外壳 prompt 自身输入框的 Id
    （外壳经动态路径渲染时其登记 Id 被寻址，不再与用户 input 撞车）。

### D3. 条件分支（依 T3 结论，二选一或都不动）

- 若证实**槽位错位粘焦**：条件包装扁平化处（aura_view_builder.rs:772-796
  邻域）给「条件为假但结构需稳定」的挂载点渲染恒定占位（`View::Empty` 不再
  参与收缩的路径），或卸载路径发一次 `operation::unfocus`（iced_core
  focusable.rs:60-79）。
- 若证实**Init 重放联动**（fire_child_init_if_any 每脏重建重放触发状态写入
  → 间接触发共享 Id 聚焦）：D1/D2 已切断撞车链，重放语义本身不动（476 系
  既有行为），登记结论即可。

### D4. autoui_type 归因顺修（renderer.rs + mcp_server.rs）

- 探针先行：`execute_action_vnode` 临时打印 vnode.path 与 find_view_by_path
  命中节点 placeholder，跑 042 确认错位点（预期命中 8302-8313 覆盖臂）。
- 修「双生产者不一致」：`__bounds_collected` 臂不再用 converted 树覆盖
  styled_vtree（改为与 shared.view 同源的 fresh view 重建快照，或覆盖快照的
  同时同步刷新 shared.view 指向的树——取实施时更小侵入者）。
- 补 `find_view_by_path` 缺失臂（Button{content}/Table/Tabs，对齐
  vnode_converter::extract_children 结构），消除两套遍历不同构。
- 回归测试（mcp_server.rs 测试模块）：构建「条件子 widget + 双 input」View
  + VTree，断言第二个 input 的 VNodeId → `extract_action_from_view` 得
  PasswordChanged（非 UsernameChanged）。

### D5. 不变量

- 不动 `.at` 契约、不动 auto-musk、不动 Vue 轨（vue.rs 零改动）；
- 泛型 IntoIcedElement 路径（rust 模式）仅同步 Id 派生收敛，行为不变；
- `cargo tf` 门禁（Category B：ui 渲染层 Rust 改动，合入前一次全量）。

## 测试设计

**TDD 先红后绿**（需求 §七）：

1. **红测试 A（真焦点语义，iced_test）**：renderer.rs #[cfg(test)] 新增
   `vm_two_inputs_focus_is_exclusive`：手工构建 `AbstractView::Column` 内两个
   `AbstractView::Input`（on_change 各挂 DynamicMessage
   LoginChild/UsernameChanged、LoginChild/PasswordChanged，placeholder 对齐
   musk），经 `render_dynamic_view`（私有同模块可及）构建 Element →
   iced_test::Simulator → `click(find("Enter password"))` → `typewrite("admin")`
   → `into_messages()` 断言**仅** PasswordChanged 消息（当前红：两 handler
   齐发）。另 `vm_two_inputs_ids_distinct`：断言两 input 登记的 Id 不同
   （当前红：同为 prompt_input）。
2. **单元测试 B（Id 派生纯函数）**：`derive_input_id` 唯一性（widget/event
   主键区分同参输入）与跨调用稳定性（同输入两次派生相等）。
3. **MCP 回归 C**：D4 的 path 对齐测试（先红：现共用 vue→vnode 路径下第二
   input 归因第一个——以测试构建的最小场景复现为红的前提，若最小场景未红
   则按探针结论调整场景至红）。
4. **Example 级真人/真机验证**：042 example（T1）VM 真键盘：点击第二框仅单
   焦点环、击键单投递、值断言（username 不变）；Vue 模式对照正常；musk
   登录页 admin/admin 全流程。
5. **回归矩阵**：003-converter 双向联动（真键盘）、028-launcher 召唤聚焦、
   Tab/refocus 路径抽查；`cargo t ui` 局部 + `cargo tf` 全量。

## 验收标准

- [ ] 042 最小 example：VM 真键盘单焦点、无键盘双投递（需求 §七-1）
- [ ] musk 登录页实测：双框独立、admin/admin 登录可完成（§七-2）
- [ ] 003-converter 双向联动不回归（§七-3）
- [ ] `cargo tf` 全绿 + `cargo test -p auto-lang --lib --features ui-iced` 全绿（§七-4）
- [ ] autoui_type 对第二个 input 派发正确 handler + path 对齐回归测试（§七-5）
- [ ] Plan 047/057/464 焦点约定与 launcher 召唤不回归（T5/T7 抽查证据）
- [ ] Vue 轨零改动（vue.rs diff 为空）

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T1 最小复现 example**
  新建 `examples/ui/042-two-inputs-child/`（pac.at：name two-inputs-child/
  scene ui/render vue；src/front/app.at：根 widget `if !.authed { LoginChild }
  else { text "in" }`；src/front/login_child.at：子 widget 双 input——
  `value: .user, oninput: .UserChanged, placeholder: "Enter username"` 与
  `value: .pass, oninput: .PassChanged, placeholder: "Enter password"` +
  `if .err != "" { text .err }` 条定块 + 提交按钮写 .authed=true，镜像 musk
  形态；README.md 复现步骤）。对齐 005-login 目录布局。
  验证：`auto run -r vm` 启动渲染正常，真键盘复现双焦点/双投递（截图或文字
  证据记入本计划）。[Category A：无 Rust 改动，禁 cargo t]
  [✅ 已完成] c5333cd82（example 落盘）。**复现通道勘误（T3 期间发现）**：
  本环境 OS 级键盘注入不可达——PostMessage WM_CHAR/WM_KEYDOWN（含正确
  scancode lParam、前台态）对 winit 0.30 均无效；computer-use 前台通道被
  并行会话焦点抢占/像素帧持续 stale。T1 初版「视觉取证」（r4/r5
  username=adminadmin）经中性提示重验确认为**视觉模型诱导性幻觉**（MCP
  autoui_vtree 真值：两框恒空）——该组 png 已删，复现证据改由 T3 机制级
  无头复现（e8c26a92a,双红测试）+ musk 真键盘实测（上游 011）承担。
  042 example 保留为 T5 验收载体（README 记录手工复现步骤）。
- **T2 红测试**
  `crates/auto-lang/src/ui/iced/renderer.rs` 测试模块新增红测试 A 两例
  （vm_two_inputs_focus_is_exclusive / vm_two_inputs_ids_distinct）+
  derive_input_id 单测占位（T4 实装纯函数后启用）。
  验证：`cargo test -p auto-lang --lib --features ui-iced two_inputs`——
  两例如期红（双投递消息/同 Id）。
  [✅ 已完成] e8c26a92a（line_edit_tests 模块,iced-layout-tests 门控——
  iced_test 依赖挂该特性,验证命令实为
  `cargo test -p auto-lang --lib --features iced-layout-tests p483`）。
  **双红**：`p483_vm_two_inputs_have_distinct_iced_ids`（两框同 Id
  "prompt_input"）+ `p483_focus_op_then_typing_stays_single_delivery`
  （注入产线同款 focus operation 后打字,UserChanged+PassChanged 齐发=
  机制级双投递复现）;**三绿控制组**：静态树点击打字单投递 / 真实 042
  形态（parse→registry→component→render_dynamic_view e2e）bounds 不重叠
  且单投递 / 跨重建（UserInterface+into_cache 换 Element）单投递。
- **T3 诊断取证（探针，临时）**
  在 renderer.rs 四个 focus 调用点（6744/8506/9889/9925）加 eprintln 探针
  （沿 ASH_DEBUG_FOCUS 既有开关），并在 042 运行中记录：点击 password 前后
  各 focus operation 是否触发、触发时目标 Id；结合 T2 红测试现象判定主触发
  机制（A 共享 Id focus op / B 槽位错位粘焦 / C overlay 捕获）。结论写入
  本节下方「根因结论」；探针撤除（不进合入提交）。
  验证：结论段落 + 判定依据（日志摘录）落档。
  [✅ 已完成] 探针 6 处（5 focus 位点+Input 臂构建现场）+ AUTO_DEBUG_MSGS
  入口门实测后全部撤除（合入提交零残留）。判定过程：探针证明 042 直发
  键盘不可达→OS 注入放弃；三层无头验证（静态树/真实形态/跨重建）全绿
  →排除 B 槽位错位与 C overlay/布局重叠;iced_widget 0.14.2 源码实锤
  text_input 无 Tab 臂 → 机制 A 的 Tab 触发链闭环（详见根因结论）。
- **T4 主修复（Id 唯一化 + 登记表 + 改址）**
  renderer.rs：删 13698-13700 覆盖；新增 `derive_input_id` 纯函数并收敛两
  路径（render_dynamic_view Input 臂 + IntoIcedElement 3097 臂，
  build_input_shape 内派生行移除）；session.rs：devtools 增 input_ids 登记；
  renderer.rs：dynamic_view 每帧清填 + 四个 focus 消费点改址（9925/8505/
  9889/9905/6744）；依 T3 结论实施 D3 条件分支（或登记"无需"）。
  验证：`cargo check -p auto-lang` 零警告 → T2 测试转绿
  （`cargo test -p auto-lang --lib --features ui-iced two_inputs`）→
  `cargo t ui`。
  [✅ 已完成] 5f10fca0f（T4 主体）+ c613a7704（merge master 同步,带入 482
  nav_contract 特性门修复 3e173633d——worktree 基点早于该修复,曾致
  cargo t ui 编译失败,merge 后 774 测全绿）。实现:derive_input_id 纯函数
  （主键 widget+event,兜底三元组,build_input_shape 泛型路径行为持平）;
  render_dynamic_view Input 臂删除固定 prompt_input 覆盖,改派生唯一 Id
  （主键在 on_change move 前取好）;session.rs devtools 增 input_ids 登记
  （RefCell<Vec<Id>>）;collect_input_ids 依 DFS 序收集
  （Column/Row/List/Container/Scrollable/Grid/Overlay/Popover 八容器臂）;
  dynamic_view 每次脏重建清填;五聚焦点改址:Tab fallback（8532）/
  tail-refocus（9917）/初始聚焦（9933）/__focus_input（9951）/launcher
  召唤（6764）→ 登记表首个,空表退 prompt_input_id/旧字面量（无匹配=
  空聚焦,安全降级）。验证:cargo check 零新警告;p483 六测全绿
  （distinct_iced_ids 与 focus_op_then_typing 双红转绿——机制级红测改
  聚焦「首个 input 派生 Id」,跨修复前后均有意义）;cargo t ui 774 全绿。
  D3 条件分支确认无需启用（T3 已排除 B/C 机制）。
- **T5 example 真机验证**
  `auto run -r vm`（042）：真键盘点击 password 输入 admin——单焦点环、
  user 值不变、Login 流程可完成；`auto run`（Vue 模式）对照正常；
  003-converter `auto run -r vm` 双向联动回归；musk 登录页
  （D:\autostack\auto-musk，plan-050-dev worktree 对照）：admin/admin 登录
  可完成（若 musk 环境不可达，登记顺延并在 042 上以等价断言代验）。
  验证：四项证据（截图/文字）记入本计划。
  [✅ 已完成] 2026-08-29 实机（修复版 worktree 二进制,MCP 通道取状态真值
  ——OS 级键盘注入在本环境对 winit 无效,真键盘 Tab 复验顺延真人清单,见
  待澄清 1/5）:
  ① 042 VM:双框 MCP type 各自独立写回（user/pass 互不污染）→ Login 按钮
  press → handler .LoginChild.Submit → state_changes authed:false→true →
  根条件翻转显示 "in"、inputs 清零（完整登录流程）;
  ② 042 Vue:auto run 编译+Vite 服务正常（title=two-inputs-child）;
  ③ 003-converter VM:C/F 双向 MCP type 驱动,行为与 master 二进制**逐值
  一致**（正向 C=100→f=0 的 MCP-type 路径怪癖为既有语义,master 同样表现,
  非本修复回归;反向 F=32→c=0 正确）;
  ④ musk 登录页（plan-050-dev@4e184b9,修复版二进制）:password 框 MCP
  type "admin" → 归因 .LoginPage.PasswordChanged（T6 修复实机生效）→
  password="admin" 独立写回,username 无污染;password:true 掩码声明呈现
  正常。admin/admin 真键盘全流程因输入通道限制列入真人清单。
- **T6 autoui_type 顺修**
  mcp_server.rs:2207 execute_action_vnode 加临时探针（vnode.path vs 命中
  placeholder）跑 042 确认错位层 → 修 renderer.rs:8302-8313 双生产者不一致
  （快照与 shared.view 同源）+ 补 find_view_by_path 缺失臂（Button content/
  Table/Tabs）→ mcp_server.rs 测试模块新增 path 对齐回归测试（第二 input
  vnode id → PasswordChanged）。探针撤除。
  验证：新测试绿 + `cargo t ui`。
  [✅ 已完成（2026-08-29 并行执行者会话，plan-483-dev@6bf36aca5）] 勘察修正：
  错位层不经 042 探针实锤而是代码层双证据——①mcp_server `find_view_by_path`
  手写子枚举缺 Button{content}/Table/Tabs 三臂（与 extract_children 同构
  不变量破坏，vnode_converter.rs:467 文档注释明载 MUST 同构）；②双生产者=
  styled_vtree 在 `__bounds_collected` 回路被 live_vtree
  （convert_view_messages 加工树，Tabs/Accordion/NavigationRail/Slider
  回调型变体折 Empty——renderer.rs:4537 文档注释）覆盖，而 shared.view 来自
  view() MCP 同步块的裸树，两树结构性不同时 vnode.path 对位错位。另实测
  （master release exe）：最小双 input/条件子 widget 双 input/真实 musk 登录页
  三场景派发均已正确——Plan 446 J1 的每帧同源推送已关稳态窗口，原始错位为
  加工树覆盖窗口内的时序态。修复：find_view_by_path 委托
  extract_children_ref（同构永随）；AppState 增 mcp_sync_vtree 缓存（view()
  同步块与 shared.view 同源建），bounds_collected 改取该缓存（缺失退
  live_vtree 旧行为）。tests_plan483_d4 4 测先红（三臂 3 红+登录形态锚 1 绿）
  后绿；--lib 3981 败 6=master 基线原样（plan050×2/notif×2/
  code_editor_natives+clipboard 环境）零交集。提交同时收入 T2 执行者的
  p483_* 三测（提交信息附注已披露）。
- **T7 回归与门禁**
  028-launcher `auto run -r vm` 召唤聚焦抽查；Tab→prompt、PromptBar
  refocus 路径抽查（ASh prompt 场景）；`cargo test -p auto-lang --lib
  --features ui-iced` 全绿；终检 `cargo tf`（合入前一次，Category B）。
  验证：命令输出摘要记入本计划。
  [✅ 已完成] ① 028-launcher 修复版二进制启动正常(MCP 响应;召唤式 UI 默认
  不可见属设计,热键召唤聚焦路径由改址代码+p483 测试覆盖);② ui-iced 全量
  (cargo test):本分支 5 败(plan050×2/notif×2/code_editor_natives)与
  master 基线同集合零交集——master 同口径全量 6 败(多 dock×1),dock 差异
  经二分+存储取证定位为 **P483-2 环境债**(VM storage CWD 落盘跨进程存活,
  worktree CWD 被本计划实机运行污染,清理 %TEMP%/auto-vm-storage/{hash}.json
  的 shell.dock.* 后单测即绿,非代码回归;已登记);③ **cargo tf 3260/3260
  全绿**。
- **T8 债登记与文档**
  `docs/plans/KNOWN-DEBT-AND-RISKS.md`：登记 aura_N 流 scroll/container/grid
  源索引编号错位嫌疑（D-GAP-4 同型，未修，仅 vnode 流已修）与 462 多 App
  焦点命名空间现状声明；docs/specs/auto-lang/ui/overview.md 若需提及
  input Id 登记表机制，补一段（与 review 的 spec-impact 填写衔接）。
  验证：diff 审阅。
  [✅ 已完成] KNOWN-DEBT-AND-RISKS.md 增 P483-1..5(aura_N 编号错位嫌疑/
  storage 测试密闭性/真键盘复验顺延/MCP closure oninput 正向怪癖/462 焦点
  命名空间现状);ui/overview.md 已知坑节补「VM input 焦点寻址(Plan 483)」
  段(唯一 Id+登记表+严禁回退固定 prompt_input);042 evidence 幻觉截图
  撤除+README 复现步骤补 Tab 触发链与自动化等价断言(06473bb5b)。
- **T9 独立复审（/auto-plan:review 范式）**
  清单审计（验收标准逐项对码取证）、遗漏/延后/workaround 扫描、health
  check（零警告/格式/无残留探针 print）、填 spec-impact 元数据
  （supersedes/new_spec_components/touched_goals——预期 touched_goals:
  [GOAL-007, GOAL-009, GOAL-014]）。状态翻 reviewed。
  验证：复审记录段落完整。

### 根因结论（T3 填写）

**主触发机制 = A（共享 Id 的 focus operation）,Tab 触发链。证据链逐环实锤：**

1. **共享 Id（缺陷本体）**：VM 主路径 `render_dynamic_view` Input 臂
   （renderer.rs:13700 邻域）把每个 text_input 的 Id 无条件覆盖为固定
   `"prompt_input"`——抵消 `build_input_shape`（1a8516b5b）派生的唯一 Id。
   红测试 `p483_vm_two_inputs_have_distinct_iced_ids` 锁定（两框
   `Id(Custom("prompt_input"))` 相等）。
2. **触发器 = 未捕获 Tab**：iced_widget 0.14.2 `text_input.rs` 的按键臂仅
   Enter/Backspace/Delete/Home/End/ArrowLeft/ArrowRight/Escape（源码逐臂
   核对）——**无 Tab 臂**,聚焦框内按 Tab 不被捕获 → 本仓
   `keyboard_event_message`（renderer.rs:6328-6339）对未捕获 Tab 发
   `__focus_prompt` → update 8532 臂 `focus(prompt_input_id)`（session.rs:166
   默认字面量 `"prompt_input"`）。musk 登录流「username 输完按 Tab 切框」
   是标准操作——与用户实测时序吻合（username 已有 "admin" 后从第一个
   password 键起整段双投递成 "adminadmin"）。
3. **双置焦语义**：iced_core `Focus` operation（focusable.rs:29-57）对
   **所有** Id 匹配的 focusable 逐个 `state.focus()` → 两框同 Id → 双
   `is_focused=Some` → 双焦点环/光标双闪（musk 症状 1）。
4. **键盘双投递**：Column 无条件扇出 + text_input 键盘守卫仅判
   `is_focused`（text_input.rs:910）→ 两框各消费各的 on_input →
   UsernameChanged/PasswordChanged 齐发（musk 症状 2/3）。机制级无头复现：
   红测试 `p483_focus_op_then_typing_stays_single_delivery`（注入产线同款
   focus operation 后 typewrite,UserChanged×N + PassChanged×N 齐发）。
5. **排除项（三层绿控制组）**：静态树点击/打字单投递;真实 042 形态
   （条件子 widget+嵌套 labeled col）e2e bounds 不重叠且单投递（排除布局
   塌陷/C overlay/扇出断裂）;跨重建（UserInterface+into_cache 换 Element,
   镜像 iced daemon 槽位 diff）单投递（排除 B 槽位错位粘焦）。
6. **对照组不对称解释**：003-converter 用户测试为鼠标点击切框（无 Tab）,
   且单 input 聚焦下未捕获 Tab 不发生（Tab 时无聚焦框或用户未按）——
   042/musk 形态按了 Tab 即触发。

**修复含义**：T4 主修复（唯一稳定 Id + 渲染期登记表 + 四聚焦点改址）同时
切断环 1（不再共享）与环 2 的寻址（focus 目标解析到唯一 input）,机制级
红测试即转绿;D3 条件分支（槽位稳定/unfocus）**无需启用**（B/C 已排除）。

## 复审记录

**/auto-plan:review 2026-08-29（zcode,复审会话;双执行者成果一并核验）**

复审方法:worktree `.worktrees/plan-483-dev` 内 `git diff master...HEAD`
逐项对码 + 全部门禁复审现场重跑(不信勾选框)。

| # | 验收标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | 042 VM 真键盘单焦点/无双投递 | **partial(代验)** | 代验双证:① iced_test 六测（含机制级 focus-op→打字双投递复现转绿+三控制组,`--features iced-layout-tests p483` 复审重跑 6/6）;② 042 实机 MCP 全流程（type→独立写回→Submit→authed 翻转→"in"）。真键盘注入对本环境 winit 0.30 不可达（PostMessage WM_CHAR/WM_KEYDOWN+scancode 实测不达）——顺延真人清单（P483-3,042 README 三步）,沿 473 先例 |
| 2 | musk 登录页双框独立/admin-admin 可完成 | **partial(代验)** | 修复版二进制+plan-050-dev 实机:password 归因 `.LoginPage.PasswordChanged`、独立写回、user 无污染;admin/admin 真键盘全流程顺延真人（P483-3） |
| 3 | 003-converter 不回归 | pass | MCP 驱动 C/F 双向,与 master 二进制**逐值一致**（正向 C→F=0 怪癖 master 同现,P483-4 立债非回归） |
| 4 | cargo tf 全绿 + ui-iced 全绿 | pass | tf 复审重跑 **3260/3260**;ui-iced 5 败=master 基线同集合零交集（master 同口径 6 败,dock 差经二分+%TEMP% 存储取证定位为 P483-2 环境债,清 shell.dock.* 后单测绿,非代码） |
| 5 | autoui_type 第二 input 归因+path 回归测试 | pass | tests_plan483_d4 复审重跑 4/4（登录形态锚/Button/Table/Tabs 寻址）+ 实机复核 PassChanged |
| 6 | Plan 047/057/464/launcher 焦点不回归 | pass | 五聚焦点改址链完整（textarea 优先→登记表→prompt_input_id 兜底,git diff 逐点核）;on_submit 跨重建语义由 p483_rebuild 测试锁定;launcher 修复版启动正常。注记:ash prompt 真机 refocus 未驱动（代码审读,低风险） |
| 7 | Vue 轨零改动 | pass | `git diff master...HEAD -- crates/auto-lang/src/ui_gen/` = 0 行 |

**遗漏/延后/workaround 扫描**:遗漏无（T2 两测以 focus_op 形式交付语义覆盖;
D3 免启有 T3 排除证据;derive_input_id 单测已交付）;延后三笔均已立债
（P483-1 aura_N 编号/范围外、P483-3 真键盘复验、P483-4 MCP closure 怪癖/
master 既有）;workaround 无（diff 零 TODO/临时标记,eprintln/dbg/483-PROBE
残留=0,cargo check 零新警告）。T1 视觉幻觉证据已撤除并留档勘误。

**裁定:PASS（5 pass + 2 partial）**。两 partial 均为同一环境阻塞（OS 键盘
注入不可达）的真人清单项,代验证据充分,沿 473 先例（E2E 代验+真人顺延）
立债放行;最终裁定权随 /auto-plan:merge 呈用户。status→reviewed。

## 待澄清事项

1. **musk 真机实测执行方**：T5 默认由本计划执行者在 auto-musk 仓对照实测
   （参照 473 先例可 E2E 代验）；若环境不可达则顺延为真人清单并登记。
   **已收束（T5）**：MCP 通道实机代验完成（password 归因/独立写回 ✓）；
   真键盘 admin/admin 全流程与 Tab 复验顺延真人清单（P483-3，环境键盘
   注入对 winit 无效）。
2. **D3 条件分支形态**依 T3 结论二选一或均不启用（计划内已约束为最小侵入）。
3. 042 example 编号取 042（041 之后、459 之前空闲段）；若与其他并行计划撞号，
   以 examples/ui 目录实际空闲号顺延并同步 README。
4. **⚠ 执行冲突（2026-08-29 18:54 发现,执行暂停）**：worktree
   `.worktrees/plan-483-dev` 内出现**另一并行执行者**的未提交改动
   （renderer.rs:8407 D4 快照同源化 + mcp_server.rs + session.rs
   `mcp_sync_vtree` 字段,注释自称「Plan 483 D4」,mtime 18:51-18:53,
   与本会话实时并发）,且该在写代码存在 E0515 编译错误（renderer.rs:8420
   borrow 临时值）,阻塞 T2 红测试编译。本会话已完成 T1（commit c5333cd82）
   并写入 T2 红测试（tests 模块尾部,与对方改动不相交,未提交）。
   **需用户裁定**:同一计划双执行者如何收束——(a) 本会话让渡 T6/D4 给对方
   并继续 T3-T5（但 T4 主修复与对方同文件,仍需对方收笔或先合）;或 (b) 对方
   停手,本会话集成其 D4 草稿后继续全计划。裁定前本会话暂停共享文件编辑。
   **状态更新（2026-08-29 19:05,「对方」= 用户主会话派出的 D4 调研修复会话）**：
   对方已收笔——E0515 借用错已修,D4 双修完成并提交 6bf36aca5（T6 ✅,详见 T6
   标记;该提交同时收入本会话的 T2 红测试 p483_* 三测,iced-layout-tests 门控,
   distinct_iced_ids 一条红=计划内 TDD 红态待 T3-T5 转绿）。共享文件 renderer.rs
   现处于干净已提交状态,本会话可裁定后续:T3-T5 继续（推荐,主缺陷焦点修复
   未动工）,T2 红测已在库等转绿。
