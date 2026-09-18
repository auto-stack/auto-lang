---
plan_id: PLAN-649
status: execution_done    # drafting → executing → execution_done → reviewed → archived
feature_name: bp-consumption-groundwork（bp 消费地基：L1 连字符解析 + icon 词汇面 + data-table 半句复核）
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/blueprint/contract.md#Q5-解析链规则（补连字符变体探测）
new_spec_components:
  - docs/specs/blueprint/contract.md#验证面-palette-词面（icon 归属落点）
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [blueprint]          # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 5
total_steps: 5
---

# [PLAN-649] bp-consumption-groundwork —— bp 消费地基

## 0. 变更摘要

PLAN-070（auto-down）消费侧铺开已在本仓留下两个地基缺口 + 一条措辞失准的债：

1. **L1 连字符解析不可达**（KNOWN-DEBT:90，640 登记）：`resolve_module_path`
   的 `probe()` 只做字面量点号→分隔符转换（`lib.rs:2825-2840` 一带），
   `use bps.feedback.empty_state...` 探测 `feedback/empty_state/` 落空——
   8/13 包（`empty-state`、`sidebar-shell`、`data-display/*` 等）L1 直连导入
   不可达（640 AC-08 走查实证：signup 双轨直连成功、empty-state 全黑）。
   绕开态（bind 工件/L2 拷贝）掩盖了主通道失效。
2. **icon 词汇面缺口**（P643-D1，643 merge 对账登记）：`icon` 在 schema 为
   `builtin_widget`（iced native，Plan 620/621 语义，`schema/aura.at:713-718`；
   VM 侧实现在 `ui/iced/native_icon.rs`），但 ui_gen `WidgetRegistry` 零注册——
   070 落地的 `blueprints/navigation/filetree` 被迫 **`palette = []`** 过门禁
   （门禁被临时绕过的欠账态，实勘确认 spec 第 3 行）。
3. **data-table 半句复核**（640-D89 残余）：债行措辞"`data-table` 未入
   WidgetRegistry"**失准**——`registry.rs:2017-2019` DataTable 注册带
   `with_alias("data-table")`；真实缺口是**无 aura vue 映射**（grep aura.at
   零命中），回避是否仍成立需复核改写。

本计划修复前两项、复核第三项，三债一并核销/改写。全部是消费侧地基，
PLAN-647 的版本面裁定与 PLAN-643 的 palette 词面规则不受影响。

## 1. 目标

- **G1 L1 直连全通**：连字符命名的 bp 包经 `use bps.<kind>.<name>.reference.<variant>`
  （下划线书写）直连导入可达；8/13 包恢复主通道；无连字符包不回归。
- **G2 icon 词汇面闭环**：`icon` 进 palette 合法面（修向两候选见 §5.2），
  `filetree` palette 恢复非空（最小集 `["icon", "text"]`），门禁不再靠置空绕过。
- **G3 data-table 半句复核**：债行按事实改写（alias 已注册、缺 aura vue 映射），
  并裁定：补映射（若 vue 消费面确有需求）或维持回避（措辞更正）。
- **G4 三债收口**：KNOWN-DEBT:90、P643-D1、640-D89 残余半句核销/改写。

### 非目标

- 不改包命名约定（8 包目录不改下划线——选解析侧修复，改名是 breaking 且
  债面两选项中明确次选）。
- 不动 647 版本面（探测变体是解析规则，不是版本语义）。
- 不修 P642-D2/D4（027-file-manager 内嵌/包命名空间化——gallery 基建债另归）。
- 不做 vm-component-parity（639-D1 三约束，独立计划）。
- 不扩 Tier 1 目录、不做 L1 组装样板（本计划是它们的地基，不是它们本身）。

## 2. 架构方案

三件互相独立的小修，共享"消费地基"主题：

1. **解析侧变体探测**（G1）：`resolve_module_path` 的 `probe()` 闭包扩候选集——
   字面量路径优先，未命中时对含下划线的路径段逐段追加连字符变体再探
   （`empty_state` → `empty-state`）。仅解析层；`use` 语法、模块书写形态不变
   （点号+下划线），磁盘命名约定不变（kebab）。probe 是纯路径存在性探测，
   无语义风险；变体仅在下划线段上生成，碰撞由"字面量优先"保序。
2. **icon 修向**（G2，两候选，T-01 裁定）：
   - **候选①（默认）**：vue registry 补 `icon` 注册（对照 VM 原生实现
     `ui/iced/native_icon.rs` 的 props 契约）——若 vue 发射面已有承接
     （schema `backends.web: "native"` 的发射语义），注册即通；
   - 候选②：palette 合法集扩 schema `builtin_widget` tier 白名单（643 词面
     规则的第二次扩展）——若 vue 发射面确实空白，②是诚实面（词面放行但
     双端支持面如实标注 VM-only）。
3. **data-table 复核**（G3）：纯调查+簿记，最多补一条 aura 映射。

## 3. 技术栈

- Rust：`crates/auto-lang/src/lib.rs`（resolve_module_path probe 扩展）、
  `crates/auto-lang/src/ui_gen/widget/registry.rs`（icon 注册，候选①）或
  `crates/auto-lang/src/ui_gen/bp/registry.rs`（palette 合法集，候选②）。
- 测试：probe 单测（tmp 目录 fixture）+ palette_drift 正/负 + L1 直连端到端
  （复用 640 AC-08 走查形态：empty-state 双轨渲染）。
- 门禁：Category B（`cargo check -p auto-lang` + scoped `cargo t`）；解析链属
  编译器面 → review 前裸 `cargo tv` 兜底；无 aavm/docs_gen 触发。

## 4. 需求分析与背景调查

### 授权记录

- 2026-09-18 会话：用户确认起草"bp 消费地基"计划（此前后续清单中的紧迫项
  合并为 PLAN-649；下一号 649 空闲实勘确认）。仓库=auto-lang 单仓；worktree
  `D:/autostack/.wt/lang-649/auto-lang`。
- 协同注意：PLAN-070（auto-down）在途且已回流 `navigation/filetree`——T-03
  恢复 filetree palette 可能与 070 后续提交交叠，执行期保持 rebase 敏感；
  filetree spec 本义的最终演进归 070，本计划只做最小恢复。

### 证据（路径实勘，2026-09-18）

- **解析链**：`crates/auto-lang/src/lib.rs:2825` 起 `resolve_module_path`——
  `rel = module.replace('.', MAIN_SEPARATOR)` 后 `probe()` 只探 `{rel}.at` /
  `{rel}/mod.at` 两种字面量形态（base_dir → 父目录 Plan 327 → deps 链）；
  无任何变体探测分支。（2768-2824 是 647 的 dep 版本键护栏，非本计划面。）
- **债实证**：KNOWN-DEBT:90 原文——"8/13 包经 L1 直接导入不可达
  （`bps.feedback.empty_state...` 探测 `feedback/empty_state/` 落空）；无连字符
  包（form/signup、form/login、dashboard/overview 等 5 包）正常可达"；640 AC-08
  走查：signup VM+vue 双轨直连成功、empty-state 全黑。
- **icon**：`schema/aura.at:713-718`——`tier: "builtin_widget"`、
  `backends: { web: "native", iced: "native" }`、`aliases: ["Icon"]`；
  VM 实现 `crates/auto-lang/src/ui/iced/native_icon.rs` + `icon_file.rs`；
  `WidgetRegistry` grep `"icon"`/`"Icon"` 零命中。P643-D1 原文含两修向候选。
- **filetree 现状**：`blueprints/navigation/filetree/spec.md:3` `palette = []`
  （070 为过门禁的置空态）；dataSource `nodes = "[]Node"` 在案。
- **data-table**：`registry.rs:2017-2019` `WidgetSpec::new("DataTable", Data)
  .with_alias("data-table")`——alias 在 registry；`schema/aura.at` 无
  `data-table` 元素（grep 零命中）——缺的是 vue 映射面，不是注册。

### 风险

- probe 变体探测对非 bp 场景（`back.X` 外部根、普通模块解析）同样生效——
  语义是"字面量未命中才试变体"，对现有全绿语料零扰动（tv 兜底验证）。
- icon 候选①需要 vue 发射面真承接；若 schema `web: "native"` 只是被 VM 臂
  消费的解释，注册后 vue 轨可能是空发射——T-01 必须先验发射面再选向。

## 5. 详细设计

### 5.1 连字符变体探测（G1）

`probe()` 扩为候选序列：字面量 `{rel}.at` / `{rel}/mod.at` 优先；未命中且
`rel` 含下划线段时，对每个下划线段生成连字符变体（单段单替换，多段组合按
段序笛卡尔积上限 N≤4 防爆炸；实际 bp 路径最多 1-2 个变体段）。规则成文进
contract.md Q5 解析链（SD-01）。

### 5.2 icon 修向（G2，T-01 裁定后走其一）

- **①registry 补注册**：`register_display_widgets`（或 media 类目归口）加
  Icon spec，props 契约对照 `native_icon.rs`；vue BackendMapping 若 aura
  overlay 能解析 `web: "native"` 则零额外配置，否则补 schema `vue:` 声明。
- **②palette 合法集扩 tier 白名单**：`palette_drift` 合法集 =
  WidgetRegistry ∪ schema package-origin（643）∪ schema `builtin_widget`
  tier（本项）——防杂音约束：仅限显式 tier 值，unclassified 不入。
- filetree palette 恢复 `["icon", "text"]`（最小集；070 后续演进自便）。

### 5.3 data-table 复核（G3）

T-01 附带调查：vue 轨对 DataTable 的实际消费/发射现状（charts-gallery 等语料
是否用到）→ 有消费则补 aura 映射；无消费则债行改写为"alias 已注册、vue 映射
缺位、palette 回避维持（理由更正）"。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/blueprint/contract.md（Q5 打包与解析） | before：解析走 `resolve_module_path` 既有链（无变体规则）；after：补一句——点号路径字面量未命中时对下划线段探测连字符变体（字面量优先，变体段数上限 4） | L1 直连主通道对 kebab 命名包可达；书写形态（下划线）与磁盘约定（kebab）各自不变 | AC-01 |
| SD-02 | modify | docs/specs/blueprint/contract.md（验证面 palette 词面） | before：合法集 = WidgetRegistry ∪ schema package-origin（643）；after：追加 icon 归属落点句（候选①=registry 注册补齐；候选②=合法集扩 `builtin_widget` tier 白名单，unclassified 不入） | icon 是首个 schema-native 内建进 palette 的案例，词面规则需记录归属 | AC-02 |

（data-table 复核与三债收口为簿记，非 spec。）

## 6. 测试设计

- **probe 单测**（lib.rs 测试模块）：tmp fixture——`feedback/empty-state/
  reference/default.at` 存在时 `feedback.empty_state.reference.default` 命中；
  字面量同名优先（`foo_bar/` 与 `foo-bar/` 并存时前者胜）；无下划线路径行为
  不变。
- **L1 端到端**：复用 640 AC-08 形态——`empty-state` 直连 use 导入双轨渲染
  （VM view 断言 + vue SFC 断言；640 走查的"全黑"对照转绿）。
- **palette 正/负**：filetree `["icon","text"]` 零漂移（正）；未知 tag 仍拒
  （负，既有测试不回归）；候选②落地时加 tier 白名单正/负（builtin_widget 放
  行、unclassified 拒绝）。
- **门禁**：`cargo check -p auto-lang` 零警告；`cargo t plan649` scoped；
  review 前裸 `cargo tv`（解析链属编译器面兜底）。

## 7. 验收标准

- **AC-01 L1 直连可达**：8/13 连字符包抽样 ≥3（含 `feedback/empty-state`、
  `navigation/sidebar-shell`、`data-display/data-table-crud`）直连 use 导入
  双轨渲染成功；无连字符包（`form/signup`）不回归。验证：端到端测试 +
  640 AC-08 对照记录。
- **AC-02 icon 词汇面闭环**：`filetree` palette 恢复 `["icon","text"]` 零漂移；
  修向（①/②）落地并成文（SD-02）。验证：`cargo t plan649` palette 正/负 +
  spec diff。
- **AC-03 data-table 半句复核**：债行按事实改写；若裁定补 aura 映射则附
  vue 发射验证。验证：DEBT diff + 复审 checklist。
- **AC-04 三债收口**：KNOWN-DEBT:90 核销、P643-D1 核销、640-D89 残余改写。
  验证：DEBT 文件 diff。
- **AC-05 spec 沉淀**：SD-01/SD-02 落地；merge 时 specs.json upsert +
  `python scripts/spec-index.py`。

## 8. 执行步骤

> worktree `D:/autostack/.wt/lang-649/auto-lang`（分支 `plan-649-dev`）；
> plan 簿记留主检出。T-02 独立可先行。

- **T-01 [x] 有界调查：icon 发射面与 data-table 消费面**（2026-09-18，
  commit 20789bec1）
  裁定=**候选①**（registry 补注册）：vue 轨 `node_to_html` 有 icon 专臂
  （`tag == "icon" || "Icon"` 前置返回 + `lucide_icons` 收集机制，
  vue.rs:1610 一带实测）——注册不截胡发射，仅补词面准入；候选②不需要。
  data-table 复核：alias 注册在案 + vue 映射经 schema `datatable` 元素
  （折叠键）由 P4-4 overlay（apply_schema_vue_mappings）实灌进 spec
  （t07 运行时断言 `get_primary_component("vue","data-table")==DataTable` 锁定）；
  回避真实理由=vue 语料零消费（唯一提及 026-database 实为原生 table），
  **不补新映射**。filetree palette 终值=`["icon","text"]`。
- **T-02 [x] 连字符变体探测**：`module_path_candidates`（字面量优先，
  下划线段按段序枚举 kebab 变体，上限 4）+ probe/`back` 根映射/probe_pkg
  三探测点接入。验证：`cargo check -p auto-lang` 零新警告（181 警告均
  预存 vm/ffi 面）；`plan649_bp_tests` t01-t05 probe 单测绿。
- **T-03 [x] icon 修向落地 + filetree 恢复**：候选①——
  `register_display_widgets` media 段 Icon spec（ark/iced/vue
  `BackendMapping::new("Icon", None)`，ImageSurface 同 precedent）；
  filetree `spec.md` palette 恢复 + 注记改写。验证：t06 正/负 +
  `cargo t palette` 11/11（含 plan643 既有 `palette_has_no_drift` 不回归）。
- **T-04 [x] data-table 债行复核改写**：不补映射（映射已在，见 T-01）；
  KNOWN-DEBT 640 行按事实改写（"未入 registry/缺 vue 映射"两说不成立，
  回避维持理由更正为零消费）；同批核销 KNOWN-DEBT:90（L1 行）与
  P643-D1（icon 词面）。验证：DEBT diff（commit 20789bec1）。
- **T-05 [x] 端到端与门禁兜底**：t08-t10 双轨端到端——empty-state
  （640 AC-08"全黑"对照转绿，VM view 文案断言 + vue SFC import 断言）、
  sidebar-shell、data-table-crud（≥3 连字符包）+ signup 不回归；裸
  `cargo tv` = 3785/3786 绿，**唯一红 `test_display_family_codegen_arm_fixture`
  为预存**（stash 全部 649 改动在干净基线 b13af592 同红，A/B 归因；
  `b13af592..master` rust.rs/fixture 零提交 → master 同带；未在案，
  已登记 DEBT 649 行）。门禁档位=Category B 全过。

依赖链：T-01 → T-03/T-04；T-02 无依赖；T-05 最后。

## 9. 复审记录

- 2026-09-18 draft handoff（/auto-plan:new）：plan_revision 1，stage: new，
  outcome: pass（授权范围内可交付 work），next: work。
  待用户确认项见 §10（两项，均有默认裁定，不阻塞开工）。
- 2026-09-18 work handoff（/auto-plan:work）：plan_revision 1（无契约变更，
  §10 两项均按预授权默认落地——变体探测作用域=全解析链、icon 落候选①），
  stage: work，outcome: **pass**，code_commit: **20789bec1**
  （plan-649-dev @ worktree `D:/autostack/.wt/lang-649/auto-lang`，base
  b13af5927），task_ids: T-01..T-05 全 [x]，evidence: `cargo t plan649`
  10/10、`cargo t palette` 11/11、裸 `cargo tv` 3785/3786（唯一红=
  预存 display_family golden，基线 A/B 归因非本计划引入，已登记 DEBT
  649 行）、DEBT 三债核销/改写 diff、SD-01/SD-02 落 contract.md（随分支
  合入），blockers: 无，next: **review**。
  执行注记：前会话已在工作树留下 T-02/T-03/T-04 未提交实现，本会话按技能
  对账核实（逐 diff 核对 + T-01 结论代码实证抽查）后续跑门禁、补登记、
  提交。AC 映射：AC-01=t08/t09/t10、AC-02=t06+SD-02、AC-03=DEBT 改写+t07、
  AC-04=DEBT diff、AC-05=SD 落地（specs.json upsert 归 merge）。
  协同注记：工作树基线 b13af592 落后 master（4817b51e），merge 前需
  rebase/合并 master 刷新（070 filetree 交叠面在基线..master 区间无
  提交，冲突风险低）。

## 10. 待澄清事项

1. **icon 修向①/②**（§5.2）：默认①（registry 补注册，词面最干净）；
   T-01 若实证 vue 发射面空白，自动落②（tier 白名单，双端支持面如实
   VM-only 标注）——两向都在授权内，无需回来确认。
2. **变体探测作用域**：默认作用于 `resolve_module_path` 全解析链（含普通
   模块，字面量优先保零扰动）；若你希望仅限 `bps.*` 前缀的 bp 导入路径，
   T-02 加一个前缀门槛即可（一行差异，执行期按默认走）。
3. （记录性）filetree palette 终值默认最小集 `["icon","text"]`；其 spec 本义
   演进归 070，本计划不越界扩写。
