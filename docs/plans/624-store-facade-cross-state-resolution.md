---
plan_id: PLAN-624
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: store-facade-cross-state-resolution
author: [zhaopuming]
created_at: 2026-09-14
updated_at: 2026-09-14

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [ui/store-facade-cross-state]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-lang/vm]
current_step: 0
total_steps: 6
---

# [PLAN-624] store-facade-cross-state-resolution

## 变更摘要

修复 jade-garden facade 切换实机（2026-09-14）暴露的**合并单态跨状态字段解析**
残余接缝——PLAN-622 清偿缺口簇⑥后，jade 侧按 README §8 预案实装
`use store: Tabs` facade 切换，Save/Edit 臂实测崩溃，暴露四层面（一项核心 +
三项伴生）：

| # | 面 | 病症（实机） |
| --- | --- | --- |
| P1 | **合并单态跨状态字段读（核心）** | widget handler 读 store 字段，GET_FIELD 解析到 widget 自身状态类型 → `RuntimeError("Field 'active_path' not found on type instance App_State")`（crash in handler_App_OpenFile ip=0x10b4）——tab 条/编辑区全链不可用 |
| P2 | `?str` 跨状态读 | `RuntimeError("Invalid object ID: 18446744071562067969")`（= 0x80000001 符号扩展，?str 值编码跨状态对象边界损坏） |
| P3 | `&&`/`||` 非布尔操作数 | VM 上按布尔逻辑求值——web JS 惯用法链 `title = fm && fm.title || fallback` 把 title 写成 `true`（jade 实机读回 bool） |
| P4 | `findIndex` 缺席 | 列表原生面无 find_index（有 find 无 find_index），调用静默失效 |

P1/P2 为 facade 形态启用的**硬前置**（P022/064 系整改的延续）；P3 需语义裁定
（本计划默认口径：**非布尔操作数的 `&&`/`||` 编译期报错**——迫使显式空值守卫，
避免静默值腐坏；value 语义作为备选记录）；P4 为机械补原生（镜像 PLAN-622
splice 2071 先例）。

**关键实证约束**：PLAN-622 最小语料（plan622_store_facade_gap_tests a 臂）的
widget handler 跨状态读 `.count` **通过**，而 jade 真实 app（root 并用
`use back.api:` 十契约 fn + `use auto.http`、root 模型 18 字段、store 经
`use store: Tabs` + snake 扫描回退定位）同形态崩溃——语料未覆盖触发形态，
差异定位是本计划 T-01 的有界调查。

## 目标

- **G1 P1 修复**：合并单态下 widget handler 读 store 字段（含嵌套对象字段、
  map 字面量参数表达式内读取）解析到 store 状态值——红测先红后绿。
- **G2 P2 修复**：`?str`（Option）store 字段跨状态读不崩、值正确（None/Some
  双态断言）。
- **G3 P3 语义裁定落地**：非布尔操作数 `&&`/`||` 的行为定死并成文（默认：
  编译期报错 + 语料钉死；备选 value 语义需在裁决工件中记录否决理由）。
- **G4 P4 修复**：`auto.list.find_index` 原生落地（返回首命中索引 / -1），
  静默失效转可用。
- **G5 既有语料零回归**：tv 全量 + tf（预存 F-01 docs_gen 漂移除外——
  该漂移已由 master 0a3c9b26b 的 F-01 归一修复收敛，复验确认）。
- **G6 跨仓验收通路**：jade facade 切换以本计划产物复跑 vm-smoke 全臂绿
  （fold 后由 jade 侧执行，AUTO_EXE 可覆盖已具备）。

**非目标**：jade 仓内任何改动（facade 切换的正式落地属 jade 侧，repro 材料仅
作证据）；confirm 模态/rfd 选择器/tick 原语（宿主能力包另案）；store 消费位的
其余已证健康面（PLAN-622 SD-01 契约覆盖，不动）。

## 架构方案

四面的修复落点分两层：

```
                ┌─ 解析层（P1/P2 核心）──────────────────────────────┐
 .at handler    │ GET_FIELD '.store_field' 的状态对象解析链：         │
 读 store 字段 ─▶│  现状：解析到 __state（widget 自身 App_State）→ miss │
                │  目标：child_state_map / 合并态回退解析到 store 状态 │
                │  锚点：vm/engine.rs GET_FIELD 臂；ui/handler_codegen │
                │  （synthesize_handler_fn 的 __state 单参形态，       │
                │  STATE_PARAM=".."；Plan 398/056 sibling 重写仅覆盖   │
                │  msg 兄弟调用，未覆盖字段读取）                      │
                └────────────────────────────────────────────────────┘
                ┌─ 语义/原生层（P3/P4）──────────────────────────────┐
 &&/|| 非布尔 ─▶│ codegen Bina 臂：操作数类型非 Bool → 编译期报错      │
 find_index ───▶│ native_catalog 三表 + shim（镜像 splice 2071 形态） │
                └────────────────────────────────────────────────────┘
```

**T-01 有界调查先行**（差异定位是修复设计的前置）：PLAN-622 语料 a 臂与 jade
真实 app 的解析分叉点定位——候选差异维度：①root 是否并用 `use back.api:`；
②store 定位通道（显式 vs snake 扫描回退）；③字段读取是否发生在 store msg
派发之后的 map 字面量；④root 模型规模/字段名重叠度。产出：一个彼仓红测语料
（复现 Field not found）+ 差异结论入档。若四维度穷尽后仍无法在彼仓复现，
以 jade 切换 diff 为 fixture 直测（跨仓语料引用，plan-022 先例），转
needs_replan 评估。

## 需求分析与背景调查

**授权记录（2026-09-14，auto-down 会话）**：用户批准在彼仓为残余接缝立项；
范围 = P1/P2/P3/P4 四面；jade 仓改动不在本计划（facade 正式切换属 jade 侧
后续，依赖本计划产物）。

**跨仓证据链（auto-down 侧在案，2026-09-14）**：

| 证据 | 位置 |
| --- | --- |
| 残余接缝四面的精化登记 + 实机结果注记 | auto-down commit 4c8c8ca（DEBTS 064 行⑥精化）+ 140775f（desktop README §8 注记「facade 切换实机结果」） |
| facade 切换 repro（app.at facade 形态 + tabs_store.at 副本含三处 VM delta） | 切换 diff 未提交（按门纪律回退），形态记录于 README §8 注记；可在彼仓语料重建 |
| 实机错误原文 | `Field 'active_path' not found on type instance App_State`（handler_App_OpenFile ip=0x10b4）；`Invalid object ID: 18446744071562067969`（Edit 臂 map 字面量） |
| PLAN-622 已修面 + 守卫语料 | auto-lang 622（e3d17db71 / 7e2fd920a / 050f54ee7）；plan622_store_facade_gap_tests 8 臂 |

**本仓代码锚点（实勘 2026-09-14，同会话 PLAN-622 work 所勘）**：

| 锚点 | 位置 | 关联 |
| --- | --- | --- |
| handler 合成（__state 单参） | `crates/auto-lang/src/ui/handler_codegen.rs`（synthesize_handler_fn；STATE_PARAM 常量） | P1 |
| sibling 重写（仅 msg 调用） | 同上 L2070-2104（Plan 398/056：`.Sibling()` → handler_<W>_<Sibling>；字段读取未覆盖） | P1 |
| GET_FIELD 运行时 | `crates/auto-lang/src/vm/engine.rs`（GET_FIELD 臂；child_state_map 归属 vm_bridge） | P1/P2 |
| &&/|| 求值 | `crates/auto-lang/src/vm/codegen.rs`（Expr::Bina 臂） | P3 |
| 原生三表 + shim 先例 | `crates/auto-lang/src/vm/native_catalog.rs`（splice=2071 三处）；`native.rs` shim_list_splice | P4 |
| 622 语料基底 | `crates/auto-lang/src/plan622_store_facade_gap_tests.rs` + `test/ui/plan622_store_facade/` | 全部（扩展基底） |

## 详细设计

每面按「最小复现语料 → 修复方向假设 → 转绿断言」推进；病因假设执行期证实或
证伪，证伪按等价实现内裁定记录。

### T-01 有界调查：语料-vs-真实 app 解析分叉点

四维度矩阵逐项二分（back.api 并用 / store 定位通道 / 派发后读取 / 模型规模），
产出彼仓红测语料 + 差异结论。若穷尽不复现：以 jade 切换 diff 为 fixture 直测
（跨仓语料先例），计划转 needs_replan 评估。

### P1 跨状态字段读（核心）

- 复现（T-01 产出）：widget handler 内读 store 字段（含 map 字面量参数
  表达式），当前 Field not found 崩。
- 方向假设（三选一，T-01 裁决）：①GET_FIELD miss 后按 child_state_map 回退
  到 store 状态实例（运行时兜底，engine 臂）；②合成期把已知 store 字段的
  `.field` 重写为 child 状态寻址（handler_codegen，镜像 Plan 398/056 的
  sibling 重写形态）；③__state 参数升格为合并态视图对象。
  侧效约束：**widget 自身同名字段优先**（局部遮蔽语义不变——PLAN-622
  F-02 同族教训：解析链改动不得扰动既有面）。

### P2 `?str` 跨状态读

- 复现：store `?str` 字段跨状态读 → Invalid object ID（0x80000001 符号扩展
  佐证 Option 编码跨界损坏）。
- 方向假设：?str 的 Some/None 编码在跨状态 GET_FIELD 路径未按 tagged value
  搬运。若 P1 修复走 child-state 寻址，本面可能随之消解——随 T-01 结论
  同步裁定，独立断言钉死。

### P3 `&&`/`||` 非布尔操作数语义

- 复现：`title = fm && fm.title || fallback`（fm 为 map）→ title = true。
- 默认裁定：codegen Bina 臂对非 Bool 操作数的 `&&`/`||` **编译期报错**
  （报错信息指引显式空值守卫形态）；web 轨 TS 语义不受影响（发射器不动）。
  备选（value 语义/short-circuit 透传）在裁决工件中记录否决理由后方可翻案。
- 注意：既有 .at 语料/corpus 中若有 `&&` 于 bool 之外的用法，属存量暴露面——
  逐个确认语义等价改写或登记。

### P4 `auto.list.find_index` 原生

- 镜像 PLAN-622 splice 先例：native_catalog 三表（shim 表 / 签名表 /
  id 表，新 id 邻接 2071 段位）+ `shim_list_find_index`（返回首命中下标，
  未命中 -1；ListData<i32>/ListData<Value> 双路；闭包谓词形态镜像
  find 2063 的谓词消费方式）。

### 规范增量

| delta | add/modify | 目标 | before/after | rationale | AC |
| --- | --- | --- | --- | --- | --- |
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md「store facade 消费位语义」节 | before：五消费位契约（PLAN-622 SD-01）；after：增补「跨状态字段读」消费位（handler 直读 store 字段/嵌套字段/?str 双态）+ `&&`/`||` 非布尔编译期报错裁定 | facade 形态启用的语义冻结补完 | AC-01..04 |
| SD-02 | add | docs/specs/auto-lang/ui/overview.md 同节内「列表原生面增补」注记 | before：无 find_index；after：auto.list.find_index（谓词/语义与 find 对齐，返回索引） | P4 消费面 | AC-04 |

## 测试设计

- **红测语料（先红后绿）**：扩展 `plan622_store_facade_gap_tests.rs` 或新增
  `plan624_cross_state_tests.rs`；语料源 `test/ui/plan624_cross_state/`（jade
  facade 形态重建：root 并用 back.api + store + handler 跨状态读/map 字面量/
  ?str 双态/&&-非布尔/find_index）。T-01 矩阵结论决定最小触发形态。
- **回归面**：`cargo tv` 全量（VM 语料金样）；`cargo tf`（VM 核心改动全档，
  预存 F-01 已由 master 0a3c9b26b 收敛，复验确认）；plan442/plan340/plan622
  定向套件。
- **P3 特检**：存量 .at 全域扫描 `&&`/`||` 非布尔用法（报错面的存量暴露清点）。
- **跨仓复验（merge 前置检查）**：jade vm-smoke AUTO_EXE 全臂绿（fold 后
  jade 侧执行，或 AUTO_EXE 直指本计划构建产物——vm-smoke 已支持覆盖）。

## 验收标准

| ID | 标准 | 验证方法 | 期望 |
| --- | --- | --- | --- |
| AC-1 | P1 红转绿：跨状态字段读红测先红后绿 | `cargo test -p auto-lang --features ui-iced plan624` | 修复前 Field not found 红、修复后绿；差异结论入档 |
| AC-2 | P2 红转绿：?str 双态断言 | 同上 | None/Some 双态读值正确，不崩 |
| AC-3 | P3 裁定落地：非布尔 &&/\|\| 编译期报错 | 报错语料 + 存量扫描清点 | 非布尔操作数报错（含指引文案）；bool 用法不受扰 |
| AC-4 | P4 find_index 可用 | find_index 语料 | 命中返回下标 / 未命中 -1 |
| AC-5 | 零回归 | cargo tv 全量 + cargo tf + plan442/340/622 | 全绿（F-01 收敛后无预存例外） |
| AC-6 | 跨仓验收 | jade vm-smoke（AUTO_EXE） | 全臂绿；facade 切换由 jade 侧落地（不在本计划） |
| AC-7 | 语义契约入账 | docs/specs/auto-lang/ui/overview.md | 跨状态读 + P3 裁定 + find_index 注记 |

## 执行步骤

| 步骤 | 任务 | 文件/操作 | 验证 |
| --- | --- | --- | --- |
| T-01 | 有界调查：复现形态定位 + 红测语料 | `plan624_cross_state_tests.rs`（新）+ `test/ui/plan624_cross_state/`（新）；四维度矩阵二分 | 红测稳定复现（或按详细设计转 needs_replan） |
| T-02 | P1 修复 | `vm/engine.rs` GET_FIELD 回退 / `ui/handler_codegen.rs` 合成期重写（按 T-01 裁决） | P1 臂转绿；622 守卫臂全绿 |
| T-03 | P2 修复 | ?str 编码搬运（随 T-02 结论定位） | P2 臂转绿 |
| T-04 | P3 语义落地 | `vm/codegen.rs` Bina 臂报错 + 存量扫描清点 | 报错语料绿；存量清点在案 |
| T-05 | P4 find_index 原生 | native_catalog 三表 + `native.rs` shim | find_index 臂绿 |
| T-06 | 全量回归 + spec 落账 + 跨仓移交 | tv/tf/定向套件；overview.md SD-01/SD-02；vm-smoke 移交材料 | AC-1..7 全过 |

每步完成后在本节追加 `[✅ 已完成]` 一行证据（对齐彼仓执行规约）。

## 分支与提交归属

- worktree：`D:/autostack/.wt/lang-624/auto-lang`（分组平铺，Plan 529），分支
  `plan-624-dev`；实现全在 worktree，plan 簿记在主检出。
- **红线（Plan 529）**：worktree 内禁 junction/symlink；移除前必跑
  `bash D:/autostack/wt-guard.sh <worktree>`。跨仓证据引用 auto-down 主检出
  绝对路径，不建链接。
- 注意：621/623 会话并行推进中，master 持续移动——fold 前 reconcile 并刷新
  受影响验证（本计划 merge 阶段职责）。

## 复审记录

- 2026-09-14 stage:new handoff（/auto-plan:new，rev 1）：任务 T-01..06 覆盖
  AC-1..7 与 SD-01/02；四面锚点与跨仓证据链实勘在案。`outcome: pass`，
  `next: work`（worktree 建好后 T-01 有界调查先行——红测形态是全部修复的
  裁判；若穷尽不复现即转 needs_replan）。

## 待澄清事项

无阻断项。三个执行期裁定点已内嵌任务：①P1 修复三选一（运行时回退 / 合成期
重写 / 合并态视图对象）由 T-01 差异结论裁决；②P3 报错口径的存量暴露清点
（T-04 内完成，存量用法逐个确认等价改写或登记）；③跨仓复验的执行位置
（AUTO_EXE 直跑 vs fold 后 jade 侧复跑，T-06 按 merge 材料记录）。
