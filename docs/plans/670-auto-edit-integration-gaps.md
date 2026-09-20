---
plan_id: PLAN-670
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: auto-edit-integration-gaps
author: [zcode]
created_at: 2026-09-21
updated_at: 2026-09-21
plan_revision: 1
current_step: 4
total_steps: 4

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 auto-man/project.md api_gen 行-server 生成器状态种子契约, SD-02 auto-lang/ui/overview.md code_editor 段-事件参数绑定语义]
touched_goals: [GOAL-003]      # F-R1 a2r 生成物可编译=三方一致面；F-W3 事件契约 VM/前端一致

affects: [docs/specs/auto-man/project.md, docs/specs/auto-lang/ui/overview.md]
---

# [PLAN-670] auto-edit 集成缺口批（F-R1 a2r server 生成器 + F-W3 code_editor 事件载荷）

## 0. 变更摘要

承接 PLAN-669 §10-1 在册未立项的 auto-edit 集成缺口两件（auto-edit 勘定
报告转交，2026-09-20/21）：

- **F-R1**（a2r 轨 server 生成器，`crates/auto-man/src/api_gen.rs`）：
  ①`generate_main_rs` legacy seed-state 路径无条件发 `use api::Db`（:2612）
  ——契约无共享 `Db` 类型时生成工程 E0432 编译失败；②不在 db.at 覆盖内/
  无类型契约的端点被生成为 `// TODO: Implement` 空体桩（:1620/:1692 等），
  auto-edit 的 `rust-workspace/auto-edit-back/` 生成物在案对照。
- **F-W3**（code_editor 事件载荷，`crates/auto-lang/src/ui/aura_view_builder.rs`）：
  `convert_code_editor` 的 on_change/on_cursor 用**光杆 `event_to_message`**
  （:10301-10308 一带）——不解析事件参数绑定，`oninput: .SrcChanged(i)` 的
  循环变量 `i` 落空串派发给 int 形参 → handler IndexError（auto-edit 矩阵
  日志 18 次）。input 部件 Plan 062 T9 已修同款（换 `event_to_message_with`）。

立项裁定（2026-09-21 用户）：**两件合并本批**；F-R1-B（契约 fn 空体→真体）
尺寸未知，批内首任务先勘定三态分类，重活则缩面拆出（见 §10-1）。

## 1. 目标

- **G1（F-R1-A）**：无 `Db` 共享类型的契约经 a2r server 生成器产出的
  工程**可编译**（E0432 消失）——模板按契约实际形态条件化。
- **G2（F-R1-B 勘定）**：钉死"契约 fn 空体"支路的真实尺寸并落三态决策
  件：小修（纯模板级，就地修）/ 重活（真 .at→Rust 降体，本批缩面+
  债登记或拆独立计划）/ 之间。
- **G3（F-W3）**：code_editor 的 oninput/oncursor 事件**参数绑定到达**
  ——`.SrcChanged(i)`/`.CursorMoved(i)` 的循环变量与字面量实参按
  Plan 062 T9 同款语义解析派发，int 形参收到 int 而非 `Str("")`。
- **G4**：auto-edit 复跑交接（merge 后通知，PLAN-003 侧收口；同 669
  §10-4 模式）。

**非目标（Non-goals）**：

- **不动** auto-edit 仓任何代码（本计划 100% auto-lang；auto-edit 是
  发现方与复跑验收方）。
- F-R1-B 若勘定为重活：**不在本批做真降体**——缩面（升 rev）+ KNOWN-DEBT
  登记或另立计划，本批只交付 A 半。
- 不做 Request 对象注入（stdlib spec §4.2 P1，669 已注记）。
- 不重构 api_gen.rs 生成器架构（只做条件化与桩支路的最小正确化）。
- 不动 input 部件已修行为（062 T9 回归由 AC-05 守护）。

**成功样貌**：041-auto-edit 语料在仓内可验证 F-W3（事件参数到达）；
最小无 Db 契约样本经生成器产出后 `cargo build` 过（E0432 消失）；
`cargo t`/`tf` 基线口径零回归；auto-edit 侧 rust-workspace 轨与
SrcChanged/CursorMoved 复跑复活（跨仓尾项）。

## 2. 架构方案

批结构 = **勘定先行 + 双修并行**：

```
T-01 F-R1-B 勘定（三态决策件）──┬→ 重活：缩面呈报（§10-1）
                                └→ 小修：并入 T-02
T-02 F-R1-A E0432 条件化（auto-man/api_gen.rs generate_main_rs）
T-03 F-W3 code_editor 事件绑定换 _with 版（aura_view_builder.rs convert_code_editor）
T-04 门禁 + spec 增量 + 交接
```

- F-R1-A 与 F-W3 互不依赖，可并行推进；T-01 决定 F-R1-B 是否进批。
- 两件各有独立验证面（api_gen 生成物断言 + 编译验证 / code_editor 事件
  单测 + 041 语料 + autoui 双端），复审按缺陷分组核对，不糊面。

## 3. 技术栈

- Rust 既有栈零新依赖：api_gen.rs 字符串模板生成（沿现状）、
  `event_to_message_with`/`Bindings` 既有机制（Plan 062 T9 建成）。
- 测试档位：`cargo t` 日常 + 涉 ui/code_editor 用 `cargo t`（模块滤串）
  + 合入前 `cargo tf`；生成器验证 = 单测断言生成文本 + 生成工程现场
  `cargo build`（api_gen 既有测试基建沿 Plan 399/musk-022 模式）。
- 不触 aavm/trans 语料/transpiler 主体 → 零 `taa/tt` 触发（F-R1 是
  auto-man 生成器模板，非 trans/ 降体本体——若 T-01 勘定误判此边界，
  以勘定件为准回填本节）。

## 4. 需求分析与背景调查

### 4.0 授权记录

- 2026-09-21 用户裁定：①F-R1/F-W3 **合并一个计划**（此前 669 §10-1
  建议"各另立"，用户追问合并可行性后裁定合并，先勘定条件成立）；
  ②归属 **auto-lang 仓**（病灶代码全在本仓：auto-man crate +
  crates/auto-lang/src/ui）；③本会话授权=**起草**（drafting → executing
  flip 待用户确认）。执行期标准流程（lang-670 worktree）。
- 无预算/自动续作限制指令。

### 4.1 F-R1 源码级事实（本会话独立勘定，2026-09-21）

| # | 事实 | 锚点 |
|---|---|---|
| R1-F1 | `generate_main_rs` 的 `db_full_cover=false` legacy seed 路径无条件 `use api::Db` + `State<Db>` 注入；`db_full_cover=true` 分支已示范"不发"先例（:2572-2576 注释在案） | crates/auto-man/src/api_gen.rs:2577-2612 |
| R1-F2 | E0432 触发条件：契约 api.at 无 `Db` 类型定义且 db.at 缺席/部分覆盖 → `db_full_cover=false` → main.rs 引用不存在的 `api::Db` | auto-edit 报告 + R1-F1 链 |
| R1-F3 | 空体桩三支路：①端点 fn 不在 db.at 名单（:1620 `pub async fn name() { // TODO: Implement }`）；②**无类型契约 fallback**——"No types defined, generating skeleton handlers" 全量桩（:1692 一带）；③:2331 一带（db.rs 生成的 toggle 推断支路邻域，勘定时归类） | api_gen.rs:1620/:1692/:2331 |
| R1-F4 | auto-edit 侧对照物：`rust-workspace/auto-edit-back/` 现成生成物（彼仓），勘定 T-01 的输入证据 | auto-edit 报告 §六 |
| R1-F5 | 生成器已有测试基建（Plan 399 db_full_cover 两路 + musk-022 events/db 模块条件）——F-R1-A 断言沿此扩展 | api_gen.rs 同域测试 |

### 4.2 F-W3 源码级事实（本会话独立勘定，根因已钉）

| # | 事实 | 锚点 |
|---|---|---|
| W3-F1 | `convert_code_editor` 的 `on_change`（oninput/input/onchange/change/onupdate/update 六别名链）与 `on_cursor`（oncursor/cursor）均用**光杆 `event_to_message(&event.handler)`**——不传 bindings | aura_view_builder.rs:10301-10308（dispatch 入口 :2082/:4027，实现 :10256） |
| W3-F2 | 光杆版 `event_to_message` 内部 `Bindings::new()` 空表——事件参数（循环变量/字面量）全部落 `parse_event_param_literal` 空串 | aura_view_builder.rs:11692-11696 |
| W3-F3 | **同类已修先例**：input 部件 Plan 062 T9 把 onchange 从光杆版换 `event_to_message_with(&event, bindings)`（`.Filter(.block.id)` 此前不烘焙=同病，059 §4.3 过滤失效真身） | aura_view_builder.rs:9735-9751 |
| W3-F4 | `convert_code_editor` 签名**已收 bindings**（dispatch 传 `self.convert_code_editor(props, events, bindings)`）——修复=纯换调用，零签名变更 | aura_view_builder.rs:10256/:2083 |
| W3-F5 | 消费语料在仓：`examples/ui/041-auto-edit/src/front/app.at` `oninput: .SrcChanged(i)`（:175）+ `editor_store.at` `SrcChanged(int), CursorMoved(int)`（:90）——int 形参×2 双事件复现面 | examples/ui/041-auto-edit |
| W3-F6 | 症状面：merged/split 两侧 args=[Str("")] 派发 → handler IndexError（auto-edit 矩阵 18 次）；vue 臂 `.at` 同源——builder 烘焙修好后两模式同愈（vm_bridge 侧零 SrcChanged 硬编码，勘定 lite 在 T-03 验证内确认） | auto-edit 报告 §六 |

### 4.3 多会话与基线

- PLAN-668 已终态 archived+cleaned（328145aca，2026-09-21）；PLAN-669
  archived（8442a77ea）。docs/plans 现无在途计划，.next-id=670 取号无冲突。
- 预存红基线：668 收口后日档基线红集已清零（其 R-25 musk p053 家族按
  "只呈报"销号、ui::layout 环境红豁免在册）——本批门禁按"绝对全绿
  减在册豁免"口径（债册 564-Q6 总条注记）。
- F-W3 的 code_editor 面在 668 红清单中零出现；F-R1 的 auto-man api_gen
  测试面零在册红。

## 5. 详细设计

### 5.1 F-R1-A：`use api::Db` 条件化

`generate_main_rs` 增加"契约实际可提供 `Db`"判定（api_module 类型表中
存在 `Db` 定义），legacy seed 路径仅在判定为真时发 `use api::Db` +
`State<Db>` 种子；判定为假且 `db_full_cover=false` 时走**无状态退化路径**
（沿 `db_full_cover=true` 分支的"no use api::Db, no with_state"形态——
:3277 已有注释先例）。生成物断言三态：有 Db+部分覆盖 / 无 Db+无覆盖 /
db_full_cover 既有两路不回归。

### 5.2 F-R1-B：勘定三态（T-01 决策件）

以 auto-edit 契约为输入跑生成器，钉死命中 R1-F3 三支路中的哪条，并按
下表分类（决策件落 `docs/reports/p670-fr1b-survey.md` 或计划内附录，
含生成物摘录与行数估计）：

| 态 | 判据 | 处置 |
|---|---|---|
| S-小修 | 空体源于模板条件错位（如 fns 名单匹配缺陷/无类型 fallback 误入），修正后既有 db.rs 委托路径即可覆盖 | 并入 T-02 就地修，AC 增补 |
| S-重活 | 真需 .at fn 体→Rust 降体（trans/rust.rs 接线或新降体器），>~300 行或动 trans/ 本体 | 本批缩面（升 rev）：只交付 A 半；B 半 KNOWN-DEBT 登记 P670-D1 + 呈报用户裁定拆计划 |
| S-之间 | 有限模板级真体（如 CRUD 惯用法模板，:2331 邻域已有 toggle 推断先例） | 批内做，AC 以 auto-edit 契约生成物"非空体+可编译"为准 |

### 5.3 F-W3：code_editor 事件参数绑定

`convert_code_editor` 的 `on_change`/`on_cursor` 两处 `event_to_message
(&event.handler)` → `event_to_message_with(&event, bindings)`（062 T9
逐字镜像）。语义：循环变量引用按 bindings 解析、字面量直取、前导点
路径走 `parse_event_param_expr`（051 C1/059 §B8 既有链）——与 input
部件完全一致。VM/merged 派发侧（vm_bridge/dynamic）以 041 语料 e2e
验证同愈，若 vm 侧另有独立烘焙点则在 T-03 内一并换（勘定 lite）。

### 5.4 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-man/project.md 模块表 api_gen 行 | before：api_gen 行仅"API/后端/扩展代码生成器"泛述；after：补 server 生成器状态种子契约——`use api::Db`/`State<Db>` 仅当契约实际定义 Db；无 Db 契约走无状态退化路径（生成物必须可编译） | E0432 是生成器对外契约破坏（消费方无法编译）；落档防回归 | AC-01 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md code_editor 段 | before：code_editor 事件接线未记参数绑定语义；after：补 oninput/oncursor 事件参数按视图绑定解析（循环变量/字面量/前导点路径，与 input 062 T9 同一语义）派发 | 事件参数绑定是跨部件一致性契约（input 已修，code_editor 漏修=F-W3 真身），落档钉死 | AC-03/AC-04 |

（若 T-01 勘定改判 S-小修/S-之间，SD-01 随之扩一行桩支路规则；S-重活
则 SD-01 保持 A 半口径，B 半债行在 KNOWN-DEBT。）

## 6. 测试设计

- **F-R1**（auto-man 域）：
  - 单测：最小无 Db 契约（仅带参 GET/POST 端点、无类型块）→ 生成
    main.rs **不含** `use api::Db`、不含 `State<Db>` 注入；有 Db 契约
    legacy 两路既有断言不回归（沿 Plan 399 测试形态）。
  - 编译验证：生成的最小工程落 temp 目录现场 `cargo build`（或沿
    api_gen 既有编译类测试基建）——E0432 消失的硬证据。
  - S-小修/S-之间态：auto-edit 契约形状样本的生成物断言非空体+可编译。
- **F-W3**（ui 域）：
  - 单测：构造 code_editor 节点 `oninput: .SrcChanged(i)` 于循环内，
    断言 DynamicMessage args=[int(i)] 非 Str("")；oncursor 同款；
    字面量/前导点路径两参数形态各一例（对齐 062 T9 断言面）。
  - 语料：041-auto-edit e2e（merged/split 两模式事件派发断言，
    autoui-verifier 沿用）；vm 侧同愈确认。
- **回归**：input 部件 062 行为（onchange 绑定解析）守护断言；既有
  `cargo t` 全绿（668 后基线=绝对绿减在册豁免）；合入前 `cargo tf`。

## 7. 验收标准

- **AC-01**（F-R1-A E0432 消失）：无 Db 类型契约的生成工程可编译——
  生成物断言（无 `use api::Db`/无 State 注入）+ 现场 `cargo build`
  成功；有 Db 既有两路生成物字节不回归。
- **AC-02**（F-R1-B 勘定决策件）：三态选态落档且证据链完整（auto-edit
  契约命中支路+生成物摘录+尺寸估计）；若 S-重活则缩面呈报完整（本条
  在 S-重活态下=呈报件本身）。
- **AC-03**（F-W3 oninput）：循环变量/字面量/点路径三类实参经
  code_editor oninput 正确解析派发（单测 int 直达，无 Str("")）；
  041 语料 merged 侧事件驱动不再 IndexError。
- **AC-04**（F-W3 oncursor 同类修）：`.CursorMoved(i)` 同语义派发
  （单测+语料）。
- **AC-05**（回归不退）：input 部件 062 T9 行为、code_editor 既有
  413-428 全链测试、api_gen Plan 399/musk-022 既有断言全绿；`cargo t`
  基线口径零新增红；合入前 `cargo tf` 同口径。

## 8. 执行步骤

- **T-01** F-R1-B 勘定三态决策件
  涉及：`crates/auto-man/src/api_gen.rs`（只读跑生成）、auto-edit
  契约样本（从报告/R1-F4 提取最小形状）、决策件（报告或计划附录）。
  验证：决策件含三态选态+证据；S-重活则出缩面呈报（§10-1）。
  → AC-02。**新路径**：决策件。
  [✅ 2026-09-21 已完成] 决策件 `docs/reports/p670-fr1b-survey.md`（worktree
  14522af28）：**S-重活**——auto-edit 契约命中支路②（无类型骨架 fallback
  :1693，无 pub type+无 db.at）；真体四缺口=伴生模块通用转译（生成器只认
  db.at）+route A 接线（ApiEndpoint.body 生产不读）+trans/rust.rs 内建面
  （`Env.get` 大写零覆盖/`fs.tree` 无映射）+a2r-std 新宿主 `fs.tree`（JSON
  形状须与 VM 逐字节对齐）；~210-350 行跨三 crate 动 trans/ 本体。处置按
  §5.2 预授权：缩面交付 A 半，P670-D1 登记 KNOWN-DEBT，呈报 §10-1。
- **T-02** F-R1-A `use api::Db` 条件化（+S-小修态并入时扩面）
  涉及：`generate_main_rs`（api_gen.rs:2577-2620 一带）+ 单测/编译验证。
  验证：`cargo t -p auto-man api_gen`（滤串）+ 生成工程现场 build。
  → AC-01（S-小修态另挂 AC-02 扩面）。依赖：无（可与 T-03 并行；
  S-之间态则依赖 T-01 选态）。
  [✅ 2026-09-21 已完成] worktree 14522af28：`has_db_type =
  primary_type_name_pub(api_module).is_some()` 门 legacy 种子路径；无 Db
  契约退化无状态形态（镜像 db_full_cover 分支）。`cargo t -p auto-man
  api_gen` 30/30 绿（新测 test_main_rs_stateless_when_no_db_type 双形态+
  legacy 回归）；编译验证=scratch 工程（组目录）经 generate_api 生成后
  `cargo build` **Finished 1m26s 零 error**（E0432 消失硬证据；对照=修复前
  auto-edit 003 review 源级核对在案 E0432 必然）。
- **T-03** F-W3 code_editor 事件绑定换 `_with`
  涉及：`convert_code_editor`（aura_view_builder.rs:10301-10308）两处
  调用替换 + 单测三类实参 + 041 语料 e2e + vm 侧同愈确认（勘定 lite）。
  验证：`cargo t`（code_editor/事件绑定滤串）+ 041 双模式事件断言。
  → AC-03/AC-04。依赖：无（与 T-02 并行）。
  [✅ 2026-09-21 已完成] worktree 51421cbe8：两处换 `event_to_message_with
  (&event, bindings)`（062 T9 逐字镜像；oncontextmenu 按 §5.3 范围不动）。
  新测 `plan670_code_editor_event_tests` 3/3 绿：循环变量/字面量/点路径
  三形态+无参等价回归+**041 语料级**（build_example_component 真实树：
  烘焙 Int(0) 断言+comp.on() 生产派发入口 edits+1/tabs[0].dirty 置真，
  VmRef 经桥物化读字段）。**红相位双向**：旧码上循环变量+语料双测试红、
  无参回归绿（两版等价面吻合）。vm_bridge 零 SrcChanged/CursorMoved
  硬编码 grep 在案=单点烘焙两模式同愈（勘定 lite 收口）。e2e 形态按
  §10-3 降级条款执行：单测+vm 语料级事件断言双证（语料 e2e=MCP 重装备
  超界），041 交互级 e2e 归 auto-edit 复跑侧验收（彼仓矩阵）。
- **T-04** 门禁 + 规范增量 + 交接
  涉及：SD-01/SD-02 落档、`cargo t` → `cargo tf`、跨仓交接注记。
  验证：SD 表对照 diff；门禁基线口径全绿；auto-edit 复跑通知事项
  呈报（含 F-R1-B 态走向）。
  → AC-05。依赖：T-01..T-03。
  [✅ 2026-09-21 已完成] worktree 49b02517c：SD-01/SD-02 落档+
  P670-D1 登记；docs_gen 4/4 绿；`cargo t --no-fail-fast` 5324/5344 绿，
  20 红全数在册豁免（musk p053×4+p054×2=R-25 呈报面；ui::layout×14 环境
  豁免）**零新增红**；`cargo tf` **3684/3684 绝对全绿**。交接呈报见 §10。

## 9. 复审记录

- 2026-09-21（draft handoff）：`stage: new`，PLAN-670 rev 1。
  `outcome: pass`（起草完成；F-R1/F-W3 均已源码级独立勘定——R1-F1..F5/
  W3-F1..F6 在案，F-W3 根因钉死于光杆 event_to_message；任务/AC/SD 齐；
  F-R1-B 尺寸不确定性已由 T-01 先勘定结构兜住）。`next: work`——待
  用户确认 flip `executing` 后建 `D:/autostack/.wt/lang-670/auto-lang`
  worktree 开工。
- 2026-09-21（work handoff）：`stage: work` | PLAN-670 | rev 1 |
  `outcome: pass` | code_commit: plan-670-dev@14522af28→51421cbe8→
  49b02517c（base master b6ed53d61；worktree D:/autostack/.wt/lang-670/
  auto-lang 零 WIP；组兄弟 auto-down@fba6563 detached 依赖位） |
  task_ids: T-01..T-04 | evidence: AC-01 ✓（api_gen 30/30+scratch
  `cargo build` 零 error=E0432 消失+legacy 双路回归）；AC-02 ✓（勘定件
  docs/reports/p670-fr1b-survey.md：S-重活选态+四缺口证据链+缩面呈报=
  本条在 S-重活态下的呈报件本身）；AC-03/AC-04 ✓（三形态单测+041 语料
  烘焙/派发双证+红相位双向）；AC-05 ✓（input 062 守护/059/063/api_gen
  回归批+cargo t 减在册豁免零新增红+tf 3684/3684 绝对全绿） |
  blockers: 无阻塞项；**呈报待裁**（§10-1 F-R1-B 拆计划） | next: review
  （execution_done 已置；F-R1-B 走向不阻塞本批复审——B 半已按预授权缩面）。

## 10. 待澄清事项

1. **F-R1-B 三态走向**（T-01 产出后呈报）：S-小修/S-之间→批内收口
   （默认授权就地修，AC 扩面）；**S-重活→须用户裁定**（缩面拆计划 or
   债登记节奏）——此项不默认授权，勘定后单独呈报。
   **〔2026-09-21 勘定呈报〕T-01 选态=S-重活**（判据双条全中：动 trans/
   本体+~300 行量级，另含 a2r-std 新宿主运行面 fs.tree 与 VM JSON 形状
   对齐风险；证据链=docs/reports/p670-fr1b-survey.md）。已按 §5.2 预授权
   执行缩面：本批只交付 A 半（E0432 已消），B 半登记 **P670-D1**
   （KNOWN-DEBT）待用户裁定拆计划节奏——**非阻塞项**，本批复审/合并
   不受影响。
2. **auto-edit 复跑闭环**：同 669 §10-4——merge 后通知 auto-edit 复跑
   （彼仓 PLAN-003 收口面：rust-workspace 轨编译/SrcChanged-CursorMoved
   事件复活）；F-W3 的 18 次 IndexError 症状是否全数消失以彼侧矩阵为准。
   **〔merge 后动作〕**：通知要点=①rust 轨重跑生成（E0432 消失后编译
   可过，六端点为空体桩=P670-D1 已知面）；②merged/split 矩阵重跑
   （SrcChanged/CursorMoved 应收 Int 实参）；③F-W1/F-W2 上游并案面
   （669 已修实参按名绑定；F-W2 env 注入差异另案）。
3. **041-auto-edit 语料的 e2e 形态**：现无 041 专属 e2e 基建——T-03 验证
   若需新建测试件，沿 examples e2e 既有模式（如 020/031 fixture 形态），
   建设成本预计中小；若超界（需 MCP/交互重装备）则降级为单测+vm 事件
   断言双证，语料 e2e 记债。
   **〔2026-09-21 执行裁定〕**按降级条款执行：单测三形态+041 语料级
   烘焙/派发双证（plan670_code_editor_event_tests::corpus_041_*）；
   交互级 e2e 归 auto-edit 复跑侧（彼仓矩阵即验收面），**不另记债**
   （语料级断言已覆盖 builder→派发全链，交互重装备属彼侧常态验收）。
