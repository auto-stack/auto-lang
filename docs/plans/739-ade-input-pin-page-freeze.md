---
plan_id: PLAN-739
status: execution_done       # drafting → executing → execution_done → reviewed → archived
feature_name: ade-input-pin-page-freeze
author: [agent]
created_at: 2026-10-03
updated_at: 2026-10-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: ["auto-lang/ui/frame-pipeline-incremental§输入回写视面补丁边界"]
touched_goals: []

affects: ["auto-lang/ui"]
current_step: 5
total_steps: 5
---

# [PLAN-739] ade-input-pin-page-freeze（F-037-R1 跨仓缺陷修复）

## 0. 变更摘要

jade-edit PLAN-037 探针在 PLAN-737 交付后报 F-037-R1：autodown_editor 应用
键入后换页（文件树点击或正文 wikilink 导航），视面冻结在前页且粘滞（后续
store 突变亦不刷新）；store 权威态正确。本计划根因修复：**`patch_input_values`
的 `AutodownEditor` 臂把 update 期追踪的键入全文永久覆写回重建视面的编辑器
value**——键入事件不在 `input_state_map`（ADE 元素不参与该扫描）→ retain
永不清除条目 → 每帧重建都被钉回旧页文本；`sync_external` 对覆写值按回声
守卫 no-op → 粘滞。修复 = 摘除 ADE patch 臂（PLAN-057 回声守卫已覆盖其
历史动机），并把 update 期 input_values 记账提取为单源辅助函数供真实节拍
回归测试同驱动面消费。

## 1. 目标

- **G1（根修）**：编辑（oninput 回写全环）后首次换页，编辑器视面必须随
  store 切换（重建 vtree 的 ADE value = 新页 state 内容，不经键入文本覆写）。
- **G2（消粘滞）**：冻结机制消除——键入事件不再在 `state.app.input_values`
  中永生，无任何消费者再把旧键入文本回灌视面。
- **G3（零回退）**：PLAN-732 供给语义与 13 测试、PLAN-737 交付语义与 2 测试
  全绿；ADE 单向流既有行为（自回显不清焦点/光标、外部真变化重建）不变；
  Input/Textarea/CodeEditor 的视面补丁行为不变。
- **G4（回归围栏）**：本仓新增真实节拍回归测试：键入节拍（on_change 消息
  → input_values 记账 → .at handler 落 state）→ 换页节拍（nav 消息派发 +
  retain 记账）→ 重建断言 vtree ADE value 随 store 切换。
- **非目标**：不改 jade-edit 仓（其 probe 为外部验收）；不动 PLAN-732/737
  交付面（`ADE_LINK_DISPATCH`/应用栅栏/ade core）；不改 Input/Textarea/
  CodeEditor 的 patch 语义（其 iced widget 单向 value 无核内回声守卫，
  patch 仍必要）；不动 memo/epoch 域（缺陷单主假设已推翻，无需改动）。

## 2. 架构方案

单写点摘除 + 记账单源化，零新机制：

1. **摘除 ADE patch 臂**（`renderer.rs::patch_input_values`）：从
   `Input|Textarea|CodeEditor|AutodownEditor` 联合匹配臂中移除
   `AutodownEditor`，ADE 节点 value 恒 = 模板求值（state 权威投影）。
   正确性论证：ADE 的单一事实源是编辑器核；`value` prop 唯一消费口
   `build_autodown_editor_generic → autodown_editor_sync` 已有三层守卫
   （`last_external` 差分快路 / `emit_document` 自回显快路 / `emitted_echo`
   PLAN-057 回声集），覆盖 patch 臂的历史动机（PLAN-019 批次九防连打
   重建风暴——早于回声守卫成熟的遗产保护），摘除后行为由守卫承担。
2. **记账单源**：update 主链的 `input_values.insert`（键入消息）与
   `input_values.retain`（handler 后清理）提取为 `renderer.rs` 自由函数
   `track_input_text` / `retain_input_values_after_handler`，两调用点
   原位替换；回归测试调用同函数（零漂移驱动 update 记账节拍）。
3. **测试**：语料驱动（`plan370_test_support::build_component_from_app`）
   + 单元两枚（ADE 不被 patch / 字面 content 载体键入后 sync(literal)
   零重建——防回声守卫回退）。

## 3. 技术栈

Rust（crates/auto-lang，ui::iced::renderer）；测试沿用仓内
nextest + 语料 harness；无新依赖。

## 4. 需求分析与背景调查

### 4.1 授权与范围

- **授权**（跨仓缺陷修复单，jade-edit PLAN-037 F-037-R1，2026-10-03）：
  立项诊断修复；根因修复 + 本仓真实节拍回归测试；门禁按分级惯例；
  merge 后重建 master debug binary 并在回执注明新指纹；**不得改动
  jade-edit 仓**（其探针重跑 P2b/P5 转绿 = 缺陷闭合，属 jade 侧 T-02 复验）。
- 允许仓库：仅 D:/autostack/auto-lang（jade-edit 仓只读——本调查已只读
  消费其 src/front/*.at 与 tests/probe_native_wiki_link.mjs）。

### 4.2 根因定谳（调查证据链，推翻缺陷单主假设）

缺陷单主假设「PLAN-057 回声守卫 sync 跳帧 × ADR-24 写队列/PLAN-708
ui_epoch 传播竞态，dirty/epoch 被吞」经代码勘察**否定**：

- update→view 主链对真实消息稳健：`on_with_input_for` Ok 臂恒置
  `component.dirty`（dynamic.rs:2604）→ update 尾回填 `view_dirty`
  （renderer.rs:20378）→ view() 脏重建；VM 突变臂/桥写全 bump 全局
  `state_mutation_seq`（engine.rs:1373）→ memo 快路径不可能陈旧命中；
  500ms 热重载泵兜底 update 间隙异步写。
- **真实根因 = 视面补丁钉死**（证据锚点，全部在 master@59ff4e66f）：
  1. jade 编辑器声明 `oninput: .Edit`（jade src/front/app.at:1645，ADE
     元素）；`scan_node_for_inputs`（dynamic.rs:3140）只扫
     `input|textarea|Input` 标签元素 → `.Edit` **不入 `input_state_map`**。
  2. 键入消息带全文 → update 记账 `input_values.insert("Edit", 全文)`
     （renderer.rs:19586）；后续 nav 消息 handler 后 retain（renderer.rs:19981）
     谓词 `ev_name == event_name || !input_map.contains_key(ev_name)`——
     `.Edit` 不在 map → **条目永生**。
  3. 每次重建 `patch_input_values`（renderer.rs:28075，ADE 臂）把模板求值
     的新页内容**覆写**为追踪的旧页键入全文。
  4. 覆写值进 `autodown_editor_sync → sync_external`（autodown_editor/
     core.rs:865）：等于 `emit_document()`/`emitted_echo` 集内值 → 按回声
     no-op → 编辑器核停留旧页文本。**粘滞 = 条目永生 × 每帧覆写**。
- 症状全映射：P2b（键入后链接导航 navOk/viewSwapped 分叉）= store 已切
  （NavCommit→TabActivate 单口同步，jade editor_store.at:1697/305）而
  编辑器视面被钉 ✓；P5 粘滞冻结（同按钮重按不恢复——探针重按期望文本
  本就被冻结面平凡满足/冻结持续）✓；对照（不键入换页正常，input_values
  空 → 无覆写）✓；`autoui_snapshot` vtree 编辑器子树停留旧页文本 ✓；
  无 panic/无 dirty 异常日志 ✓。缺陷单「疑似 epoch/序号被吞一次后失步」
  的直觉实为「input_values 条目永不过期」。
- patch ADE 臂出身：PLAN-019 批次九（1062837e8，042 可编辑段期）——
  早于 PLAN-057 回声守卫（`last_external` 快路批次十 + `emitted_echo`）；
  历史动机（非 state 回写应用的连打重建风暴/焦点丢失）已由守卫覆盖
  （见 §5 正确性论证）。

### 4.3 影响面与约束

- 改动仅 `crates/auto-lang/src/ui/iced/renderer.rs`（patch 臂 + 记账辅助
  函数 + 测试）；不触 `ui/autodown_editor/**`、`ui/dynamic.rs`、PLAN-732
  派发表/PLAN-737 栅栏。
- 消费方（jade）依赖的公开契约（wikilink 派发/应用栅栏/editor on_change
  消息形态）零变化；PLAN-725 T-02 的 `input_payload_consumed` 静态判定
  零变化（`.Edit(text)` 单参 handler 空载荷仍命中「空 payload 首实参」
  携带全文——handler 形参绑定不受影响）。
- 已知边界（记录不修）：handler 对文本做变换后回写的应用（如大写化），
  摘除 patch 后重建会按 state 值重建编辑器（焦点丢失）——与 Input/Textarea
  既有单向语义一致，且优于 patch 的静默视面/store 分叉；jade 无此形态。

## 5. 详细设计

### 5.1 patch_input_values 摘臂

```rust
// renderer.rs — 修前
AbstractView::Input { value, on_change, .. }
| AbstractView::Textarea { value, on_change, .. }
| AbstractView::CodeEditor { value, on_change, .. }
| AbstractView::AutodownEditor { value, on_change, .. } => { ...覆写 value... }
// 修后：ADE 从臂中移除，arm 体前新增注释块（F-037-R1 根因 + 守卫承担论
// 证 + PLAN-739 回归测试名锚点）
```

正确性论证（摘除安全性的三案例走查）：
- **state 回写应用（jade 族）**：键入帧重建 sync(回写值) = `emit_document()`
  → 自回显快路 no-op；换页帧重建 sync(新页值) → 非回声 → `rebuild_with`
  → 视面随 store。✓（修复目标）
- **字面 content 应用（PLAN-037 O 臂族，无 state 回写）**：键入后重建
  sync(字面量)：`last_external` 于上次 sync 时已记字面量 → 差分快路
  no-op → 零重建风暴、焦点/光标保留、用户文本存活。✓（历史动机由守卫
  承担的实证形态）
- **回声窗口中途帧（handler 写落盘前的重建）**：模板值 = 旧 state 值 =
  `last_external` → no-op；核内新文本不受影响。✓

### 5.2 记账单源辅助函数

```rust
/// PLAN-739：update 主链 input_values 记账单源（真实节拍回归测试同驱动面）。
pub(crate) fn track_input_text(map: &mut HashMap<String, String>, event_name: &str, text: &str);
pub(crate) fn retain_input_values_after_handler(
    map: &mut HashMap<String, String>,
    component: &DynamicComponent,
    event_name: &str,
);
```
调用点原位替换：19586（insert）与 19981（retain）。语义逐行不变
（retain 谓词原样搬运，含「触发事件条目保留」语义——PLAN-053 M4 注释）。

### 5.3 规范增量

| delta_id | add/modify/retire | docs/specs/... 目标 | before/after 规则 | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-lang/ui/design/frame-pipeline-incremental.md（新 §5「输入回写视面补丁边界」） | before：`patch_input_values` 覆盖 Input/Textarea/CodeEditor/AutodownEditor 四族；after：ADE 移出——**核为单一事实源，value prop 仅经 `autodown_editor_sync` 三层守卫消费；ADE 事件不要求入 `input_state_map`，其追踪条目不参与视面补丁** | F-037-R1 根因边界沉淀：视面权威序列 = state → sync 守卫 → 核；视面补丁不得旁路核守卫 | AC-01/AC-03 |

## 6. 测试设计

- **T-A 真实节拍回归（主围栏，renderer.rs tests 模块）**
  `plan739_edit_then_page_switch_view_follows_store`：
  语料 `test/ui/plan739_input_pin/src/front/app.at`（App：`.doc/.page`
  state + autodown_editor(key: .page, oninput: .Edit, content: .doc) +
  button onclick: .GoP2；`.Edit(t)->.doc=t`；`.GoP2->.page="p2"; .doc=.p2`）。
  节拍：view 首建（ADE value=第一页）→ 键入节拍
  （`track_input_text` + `on_with_input_for(".Edit", Some(全文))`）→
  换页节拍（`on_with_input_for(".GoP2", None)` + 
  `retain_input_values_after_handler(…, "GoP2")`——与 update 19670/19981
  同序）→ `view + patch_input_values` 重建 → 断言 ADE value == 第二页
  正文（修前 = 键入全文，红）；随动断言
  `autodown_editor_sync(新页值, true) == true`（视面核真重建）。
  gate：`cargo t plan739`（autodown feature 面）。
- **T-B 单元**：`plan739_patch_input_values_leaves_autodown_editor`
  （ADE 节点 value 不被同名条目覆写；Input 节点仍被覆写——既有语义
  反例锚）。
- **T-C 守卫回退围栏**：`plan739_literal_content_typing_sync_no_rebuild`
  （字面 content 载体：core 键入 → `sync_external(字面量原文)` 必须
  false——锁 §5.1 案例二，防未来守卫回退复活连打风暴）。
- **既有面**：`cargo t autodown`（79 面）、plan732 13、plan737 2、
  复审裸 `cargo t`（14 预存红逐名基线 diff）。

## 7. 验收标准

- **AC-01**：语料键入→换页序列重建后，ADE 节点 value == 新页 state 内容
  （不等于键入追踪文本）。验证：T-A 断言（修前红/修后绿双向实证——
  实施时先在未修基线复红一次留痕）。
- **AC-02**：粘滞消除——换页后再次任意重建帧 ADE value 恒随 store；
  `input_values` 中 ADE 事件条目对视面零影响（T-B 断言 patch 零作用）。
- **AC-03**：摘臂零行为回退——Input/Textarea/CodeEditor 仍被 patch
  （T-B 反例锚 + Plan 319 grid 测试绿）；字面 content 应用键入零重建
  风暴（T-C 绿）；ADE 自回显/外部真变化语义绿（既有 plan057/plan063/
  plan732 族全绿）。
- **AC-04**：PLAN-732 13/13 + PLAN-737 2/2 零回退；`cargo t autodown`
  零新红；复审裸 `cargo t` 预存红与 master 基线逐名全等。
- **AC-05**：merge 后重建 master debug binary，回执注明版本串 + SHA256
  新指纹；jade 侧探针 P2b/P5 重跑验收在其仓执行（本仓不冒称）。

## 8. 执行步骤

- **T-01 worktree 建立**（AC 前置）：`git worktree add
  D:/autostack/.wt/lang-739/auto-lang -b plan-739-dev` + 组内 auto-down
  兄弟 detached（895f8d0）。[✅ 已完成] worktree@9fcbdb5de 干净建组。
- **T-02 根修实施**（AC-01/02/03）：renderer.rs 摘 ADE 臂（论证注释在位）
  + `track_input_text`/`retain_input_values_after_handler` 提取 + 两调用点
  原位替换（insert@19586/retain@19981）。[✅ 已完成] `cargo check` 默认/
  autodown/ui-iced 三 feature 零错（392/397/392 警告=仓内既有基线）。
- **T-03 回归测试**（AC-01/02/03）：语料 `test/ui/plan739_input_pin/` +
  T-A/T-B/T-C 三测试。[✅ 已完成] 修后 3/3 绿；**修前复红留痕**：临时
  恢复 ADE 臂跑 T-A → `left: "第一页正文。XYZ" right: "第二页正文。"`
  （视面钉死旧页+键入文本，与缺陷现象逐字节同构）→ 撤临时臂复绿。
  实施修订两处（契约零变化）：①T-A 视面核半边改两段降层节律断言
  `autodown_editor_text`（sync 自由函数在槽位缺席时 no-op——
  PLAN-732 回执⑤同口径，生产节律经 into_iced 两拍实证）；②T-C 载体
  字面量去 wikilink 语法 + 渲染帧排布几何后点击建焦（emit 脱括号/
  重建后焦点 None 裸按键被丢——plan057 同因）。
- **T-04 分级门禁**（AC-04）：[✅ 已完成] plan732 13/13 ✅；plan737 2/2 ✅；
  `cargo t autodown` 80/80 ✅；`cargo fmt --check` 我方文件零 diff
  （仓内 28 处预存与 737 基线同数）。
- **T-05 复审+merge+binary**（AC-04/05）：[✅ 已完成] 裸 `cargo t`
  （--no-fail-fast 全景）：worktree 5057/5072，红集=14 预存红与 master
  基线（master 同口径 32 行去重 16 红）**逐名全等**；差异项全为在案
  flake：worktree 侧 plan730_commit_target_matrix 隔离复跑绿（737 回执
  点名 flake），master 侧 plan484_024/plan502_m3 本轮抖红（732 回执在案
  flake）——零新红。merge/binary/回执见 §9 复审记录与交付回执。

### 附带勘定（非本计划改动，登记 KNOWN-DEBT）

- `crates/auto-cache` 测试目标**仓内腐坏**（`cargo check -p auto-cache
  --tests` E0063：`ShimMethod` 缺 `trait_name`——PLAN-596 T-03 加字段
  未同步该测试构造）。日常档门禁只建 `-p auto-lang` 目标，auto-cache
  自身测试从不参与编译，腐坏不可见。master@9fcbdb5de 实证同破。

## 9. 复审记录

- （drafting→executing 交接，2026-10-03）stage=new，PLAN-739 rev1。
  根因定谳 §4.2（缺陷单 dirty/epoch 假设推翻，视面补丁钉死实证）；
  授权范围 §4.1（仅本仓；jade 只读）；任务 T-01..T-05 覆盖 AC-01..05
  与 SD-01。outcome: pass，next: work。

- （work 阶段修订注记，2026-10-03，revision 1 内）T-A 语料落地形态微调：
  button 事件在 VM 桥经 `onclick: .GoP2` 装配（DynamicMessage::Typed），
  测试换页节拍直接 `on_with_input_for("App", "GoP2", None)` 同构驱动
  （与 renderer.rs:19670 生产派发同口）；`.GoP2` 落 `.doc = .p2` 需
  `.p2` 常量字段承载第二页正文（字面量直赋 str 字段在语料内展开）。
  契约（目标/验收/SD）零变化。

## 10. 待澄清事项

- 无阻塞项。jade 侧 N4（别名页解析）与 ECONNRESET 残余为 PLAN-737 回执
  §5 既有分界域，不属本计划。
