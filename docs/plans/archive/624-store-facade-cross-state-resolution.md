---
plan_id: PLAN-624
status: archived                # drafting → executing → execution_done → reviewed → archived（终态；merge 收据 PLAN-624:r2 见 §9）
feature_name: store-facade-cross-state-resolution
author: [zhaopuming]
created_at: 2026-09-14
updated_at: 2026-09-15

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [ui/store-facade-cross-state]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-lang/vm]
current_step: 6
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
| P3 | `&&`/`||` 非布尔操作数 | VM 上按布尔逻辑求值——web JS 惯用法链 `title = fm && fm.title || fallback` 把 title 写成 `true`（jade 实机读回 bool）。**裁定（用户 2026-09-14，rev 2）：实装 JS value 语义**——短路返回操作数值，VM/web 语义对齐 |
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
- **G3 P3 value 语义落地**：`&&`/`||` 实装 JS 语义（短路透传操作数值：
  `a && b` = a 真值 ? b : a；`a || b` = a 真值 ? a : b；真值定义沿用运行时
  既有 truthy）——web 惯用法在 VM 轨原样可用，纯布尔用法行为不变。
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

### P3 语义证据（rev 2 附，2026-09-14 work 会话）

- 腐坏复现：plan624_cross_state/p3_chain_app.at——`t.frontmatter &&
  t.frontmatter.title || "fallback"` 求值为 Bool（静默腐坏）。
- error-out 朴素设计已证伪：`infer_object_type` 粒度不足（`ops[i]` 实为
  int 推断 NestedObject），编译期守卫误伤 20 个存量 tv 语料，已回退
  （守卫 diff 未保留；tv 3702/3702 恢复绿）。
- 结论：报错路线依赖真类型追踪（infer/ 接线，工作量与风险最大），否决；
  value 语义（②）用户批准实装。

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

### P3 `&&`/`||` value 语义（rev 2 裁定：用户批准选项②）

- 语义定义：`a && b` ≡ a 真值 ? b : a；`a || b` ≡ a 真值 ? a : b——与
  web 轨 TS/JS 对齐。真值判定沿用运行时既有 truthy（JMP_IF_Z/NZ 同一
  判定面），None/""/0/空列表 falsy。
- 实装（compile_expr 的 &&/|| 短路臂与 eager AND/OR 臂两处统一）：
  ```
  a && b:  [a] DUP JMP_IF_Z Lshort  [b] POP  JMP Lend   Lshort: (a 留栈)
  a || b:  [a] DUP JMP_IF_NZ Lshort  [b] POP  JMP Lend   Lshort: (a 留栈)
  ```
  即短路路径留 LHS 本体、求值路径 POP 掉 LHS 留 RHS——双路径栈平衡，
  末端 AND/OR 归一化发射移除（不再 PUSH_BOOL 占位）。last_expr_type 取
  RHS 推断类型。
- 存量影响面：依赖布尔归一结果的 `x = a && b` 用法（非布尔 a/b）行为
  改变——以 tv 全量语料扫描清点，逐个确认属"应被修复的腐坏"而非语义
  依赖；发现语义依赖则该处登记并评估。

### P4 `auto.list.find_index` 原生

- 镜像 PLAN-622 splice 先例：native_catalog 三表（shim 表 / 签名表 /
  id 表，新 id 邻接 2071 段位）+ `shim_list_find_index`（返回首命中下标，
  未命中 -1；ListData<i32>/ListData<Value> 双路；闭包谓词形态镜像
  find 2063 的谓词消费方式）。

### 规范增量

| delta | add/modify | 目标 | before/after | rationale | AC |
| --- | --- | --- | --- | --- | --- |
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md「store facade 消费位语义」节 | before：五消费位契约（PLAN-622 SD-01）；after：增补「跨状态字段读」消费位（handler 直读 store 字段/嵌套字段/?str 双态）+ `&&`/`||` 短路值语义（rev 2 裁定②——落地改写，原文「编译期报错」作废） | facade 形态启用的语义冻结补完 | AC-01..04 |
| SD-02 | add | docs/specs/auto-lang/ui/overview.md 同节内「列表原生面增补」注记 | before：无 find_index；after：auto.list.find_index（谓词/语义与 find 对齐，返回索引） | P4 消费面 | AC-04 |
| SD-03 | add（执行期新增） | 同节「闭包激活帧协议」 | before：无闭包帧契约；after：闭包激活入 call_stack 恰一帧、RET 弹恰一帧（帧协议对闭包闭合） | jade P2 面真因契约化（T-03 执行期发现） | AC-02 |

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
| AC-3 | P3 value 语义落地：`&&`/`\|\|` 短路透传操作数值 | p3 语料构建 + Probe 断言 | 链式回退求值得 fallback 字符串（Str，非 Bool）；纯布尔用法行为不变；存量影响面清点在案 |
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
| T-04 | P3 value 语义实装 | `vm/codegen.rs` 短路臂/eager 臂统一透传发射 + 存量扫描清点 | p3 臂绿（Str 回退值）；tv 全量清点在案 |
| T-05 | P4 find_index 原生 | native_catalog 三表 + `native.rs` shim | find_index 臂绿 |
| T-06 | 全量回归 + spec 落账 + 跨仓移交 | tv/tf/定向套件；overview.md SD-01/SD-02；vm-smoke 移交材料 | AC-1..7 全过 |

每步完成后在本节追加 `[✅ 已完成]` 一行证据（对齐彼仓执行规约）。

### 执行进度（2026-09-14 work 会话一）

- [✅ 已完成] T-01 有界调查 phase 1（commit 3537683f9，worktree
  plan-624-dev @ base 8aeb8150e）：语料 `plan624_cross_state` 七件 +
  测试 5 臂。实证矩阵：**P2 复现**（`?str` store 字段的 SetBody 派发崩
  Invalid object ID 0x80000001 符号扩展；与 app 形态无关，店 handler 体
  即崩）；**P3 复现**（&&/|| 非布尔链静默布尔化，synthesis 无诊断）；
  **P4 复现**（find_index 静默 Nil）；**P1 未复现**（open+find+mirror
  形态 split/merged 双绿——多 handler 并存/mock 后端/`.split` 前置逐项
  排除）。
- [✅ 已完成] T-05（P4）commit 3537683f9：`auto.list.find_index`（2072）
  原生落地（谓词闭包消费镜像 find 2063；命中下标/未命中 -1），红转绿。
  **【review 2026-09-15 部分重开 → F-02 已续补（commit 4f4069b5e），
  重开闭合：miss=Int(-1) 断言绿】**
- **T-02/T-03（P1/P2）**：未完成，带精确诊断挂起——P2 病灶收窄至
  「店 handler 读自家 `?str` 字段（含 lambda 形态）」；P1 在最小语料
  不复现，且崩溃轮 exe 为 623/625 会话脏树构建（`551-ge0c404f57-dirty`/
  `623-g061b86622-dirty`）——**脏产物伪影待排除**（干净树重放 facade
  切换为准）。
- **T-04（P3）needs_replan**：error-out 设计已实装并**证伪回退**——
  `infer_object_type` 粒度不足（`ops[i]` 实为 int 推断 NestedObject，
  20 个存量 tv 语料误伤）。语义三选一需裁决：①真类型追踪后报错（infer/
  子系统接线，工作量最大）；②JS value 语义实装（AND/OR 短路透传操作数
  值，中等）；③文档偏差登记（现状 + 禁用指引，零成本）。P3 红测保留为
  pending 标记（plan624_p3，当前 FAILED 属预期）。

### 执行进度（2026-09-15 work 会话二，自 58a8c6595 续）

续做清单四项全部收敛，T-01..T-06 全数完成：

- [✅ 已完成] T-04 ①（commit dbb3c3f9b）：`||` falsy 路径缺陷收敛——
  WIP 发射的 POP 在 RHS 编译**之后**，弹掉的是 RHS 本体，真值路径恒返回
  LHS（`falsy || fb` 得 Nil 即此）；修正为 POP 前置。同 commit 撤销
  WIP 的 truthy 原生 2073：JMP_IF_Z/NZ 本就是 Plan 406 nv_truthy 标签
  优先判定面（null/哨兵假），raw-i32 归一反而丢 NV 栈签且 sp 中性
  CALL_NAT 不结算 DUP 副本 stake。p3 臂绿（hop_c=Str("fallback")）。
- [✅ 已完成] T-04 ②：tv 12 失败逐个清点——**全数系坏发射伪影**：修正
  后 tv 全量 3706/3706 绿，纯布尔用法值语义与布尔归一值相等，无金样更新、
  无语义依赖登记。存量 `.at` 语料 `&&`/`||` 用法 335 处扫描清点均为
  纯布尔面（AC-3 存量影响面清点在案）。
- [✅ 已完成] T-02/T-03 ④（commit 37585e4be）：P2 崩溃根因修复——
  **【review 2026-09-15 部分重开 → F-01/F-03 已续补（commit 4f4069b5e），
  重开闭合】**——
  **真因与 ?str 编码无关**：`call_closure` 方法与 `CALL_CLOSURE` 码激活
  闭包不入 call_stack 帧，闭包体 RET 无条件弹一帧=弹走外层函数的帧；
  store handler 内 `find(λ)` 后 store RET 弹空栈不恢复，最深被调帧的
  current_fn_n_args（=2）泄漏进 0 参 widget handler（需 1），`__state`
  参数寻址越界走 NULL 守卫 → SET_FIELD 收 NULL 哨兵 0x80000001 →
  "Invalid object ID: 0xFFFFFFFF80000001"（= jade ?str 面实机错误原文）。
  带参 handler（OpenPage n_args=2）泄漏值恰等自身故不可见——语料
  OpenPage 臂绿而 Edit 臂红的分叉由此解释。修复：两路闭包激活推
  CallFrame（RET 协议闭合），方法 Err/Terminated/AwaitFuture 出口手动
  退帧。P2 臂全流程绿（SetBody 落账 + ?str 跨状态读 + find + dirty/
  edited 断言）；P1 双模臂绿（P1 最小语料本就未复现，T-01 结论维持；
  SD-03 帧协议顺带覆盖 jade 实机 P1/P2 面的候选机制，干净树重放归
  AC-6 merge 门）。
- [✅ 已完成] T-06（commit 2d47174dc）：回归 + spec 落账 + 移交——
  tv 3706/3706 + tf 3560/3560 + plan622 8/8 + plan442 17/17 + plan340
  11/11 + plan624 5/5 全绿；SD-01/02/03 delta 落 worktree
  docs/specs/auto-lang/ui/overview.md（canonical 落账归 merge）。
  **跨仓移交材料（AC-6）**：jade 侧 facade 切换复跑用 vm-smoke
  AUTO_EXE 直指本计划构建产物（plan-624-dev @ 2d47174dc）全臂绿为
  merge 前置门；facade 正式切换属 jade 侧（非本计划）。

- [✅ 已完成] T-02/T-03/T-04/T-06 全部收口（会话一已完成 T-01/T-05），
  AC-1..5、AC-7 在案；AC-6 为 merge 前置跨仓门（jade 侧执行）。
  【review 2026-09-15：T-03/T-05 部分重开（F-01/F-02/F-03）→
  work-repair 已闭合（4f4069b5e），current_step 6/6，状态
  execution_done——见 §9 两行】

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
- 2026-09-14 bounded revision（/auto-plan:new，rev 1→2）：用户裁定 P3 选
  项②（JS value 语义）——P3 设计/任务/验收按实装改写（T-04、AC-3、SD-03
  新增），本阶段证据（腐坏复现、朴素守卫 20 语料误伤、infer 粒度结论）随附
  §4。目标/其余任务/验收不变，属限范围修订。
- 2026-09-14 work 续（rev 2 裁定②执行）：调和 master（plan066/627/625 并行
  移动）；**T-04 WIP**（commit 58a8c6595，分支态）——value 语义发射已实装
  （短路透传 + auto.vm.truthy 2073 归一跳转，NULL 不再误判真值），p3 臂仍红：
  `||` falsy 路径得 Nil（疑栈/NV 交互，未收敛）；tv 12 失败 = short-circuit
  契约语料族（语义变更预期，金样待更新）+ 布尔归一依赖面（逐个清点未做）。
  plan624 套件态：P1×2/P4 绿，P2/P3 红（预期）。**T-04 未完，计划保持
  `executing`**；续做清单：①`||` falsy 路径缺陷收敛；②tv 12 失败逐个清点
  （金样更新 or 语义依赖登记）；③plan622/442/340 回归；④P1/P2 沿 622 记录
  继续。worktree 保留。
- 2026-09-14 stage:work 阶段收口（/auto-plan:work）：**outcome:
  needs_replan**（T-04 P3 语义三选一待裁决；P1 脏树伪影待排除），计划保持
  `executing`。code commits（plan-624-dev）：3537683f9（T-01 语料矩阵 +
  T-05 find_index 2072）。已交付：P4 修复绿、P2/P3 红测与病灶收窄、P1 五维
  排除记录。tv 3702/3702 绿（守卫回退后）。worktree 保留：
  `D:/autostack/.wt/lang-624/{auto-lang,auto-down}`。next: new（P3 语义
  裁定的 bounded revision，随附 P1 排除计划）；unblock 后 work 续
  T-02/T-03。
- 2026-09-15 stage:work 收口（/auto-plan:work，rev 2，自 58a8c6595 续）：
  **outcome: pass**，状态 `execution_done`，next: review。code commits
  （plan-624-dev）：58a8c6595（T-04 WIP）→ dbb3c3f9b（T-04 收口：POP
  前置 + truthy 2073 撤销；tv 3706/3706 绿，12 失败系坏发射伪影无需金样
  更新）→ 37585e4be（T-02/T-03：闭包激活入 call_stack 帧——P2
  "Invalid object ID 0x80000001" 真因修复，与 ?str 编码无关；plan624
  5/5 绿）→ 2d47174dc（T-06：SD-01/02/03 delta 落 worktree overview）。
  回归：tv 3706/3706 + tf 3560/3560 + plan622 8/8 + plan442 17/17 +
  plan340 11/11 全绿。task_ids：T-01..T-06 全数完成（AC-1..5、AC-7
  在案；AC-6 = merge 前置跨仓门，jade 侧 vm-smoke AUTO_EXE 直指
  plan-624-dev @ 2d47174dc）。blockers：无。依赖 worktree
  `lang-624/auto-down`（auto-lang-dev @ 140775f）clean 未动。worktree
  保留待 review/merge。
- 2026-09-15 stage:review | PLAN-624 | rev 2 | **outcome: needs_fix** |
  reviewed_commit: 2d47174dcc953a8fa281d77f7e570204939865f0（plan-624-dev）|
  base_commit: a06efd9f7（master 含 rev2 簿记；代码全量 diff =
  a06efd9f7..2d47174dc，15 文件 +785/−10，全在计划范围）|
  dependency_revisions: lang-624/auto-down @ auto-lang-dev 140775f0c
  （clean 未动）| spec_inputs: docs/specs/auto-lang/ui/overview.md
  （worktree @ 2d47174dc，SD-01/02/03 增补节）。
  **独立性声明**：评审在实现会话内进行，结论以工件与当轮复现重建，
  不采信执行摘要。
  **acceptance_results**：AC-1 pass（P1 双模臂绿 + T-01 差异结论入档；
  偏差注记：「修复前红」相经 T-01 四维穷尽不可复现，属实证结论非跳过，
  SD-03 帧协议覆盖同症状族已证机制）；AC-3 pass（p3 臂 Str("fallback")
  非 Bool + 纯布尔 tv 金样不变 + 存量 335 处清点）；AC-5 pass（当轮
  复现：tv 3706/3706 + tf 3560/3560 + plan622 8/8 + plan442 17/17 +
  plan340 11/11 + plan624 5/5，命令 `cargo tv` / `cargo tf` /
  `cargo test -p auto-lang --features ui-iced --lib plan{624,622,442,340}`）；
  AC-6 pass-by-design（merge 前置跨仓门，移交材料在案：jade vm-smoke
  AUTO_EXE → 2d47174dc）；AC-7 pass（SD 文本质审：描述现行为、无执行
  日记体；new_spec_components [ui/store-facade-cross-state] 合 622 先例
  约定，canonical 落账归 merge）。
  **findings**：
  - **F-01**（AC-2，medium）：?str **None 态**跨状态读无执行断言——
    p2 臂读 `.active_path` 均在 Open 之后（恒 Some）；G2/AC-2 明文
    「None/Some 双态断言」只覆盖 Some 半边。更正：p2 臂续补 None 态
    读（Open 前派发读 `active_path` 的探针，断言不崩且值正确）。
  - **F-02**（AC-4，low）：find_index **未命中 -1** 分支无执行断言——
    p4 臂仅断言命中=1。更正：p4 臂续补 miss 断言。
  - **F-03**（G1/AC-1 括注面，low）：**map 字面量实参内**跨状态读仅被
    编译、从未执行断言——app.at `.Edit`（`SetBody({ path: .active_path,
    ... })`，jade 原始崩形）为 compile-only，p2 臂 map 实参为字面串。
    更正：续补一臂执行该形态并断言值落地。
  **evidence**（可复现命令 + 工件）：见上 acceptance_results；工件 =
  plan-624-dev diff a06efd9f7..2d47174dc（vm/engine.rs 帧协议修复、
  vm/codegen.rs 短路值语义发射、native{,_catalog}.rs find_index 2072、
  plan624_cross_state_tests.rs + test/ui/plan624_cross_state/ 八语料、
  overview.md SD 节）；plan624 臂绿输出（P1 split/merged、P2 dirty=
  true+edited、P3 hop_c=Str(fallback)、P4 idx=Int(1)）当轮复现在案。
  非阻塞观察：call_closure 预算耗尽路径落入成功出口、不退新推帧
  （1M 步闭包=病态域，bp/ram 帧本已失衡，属既有破碎类，不构成本计划
  阻塞）。
  **next: work**（bounded：F-01/F-02/F-03 全为语料/断言续补，T-03/T-05
  重开，current_step 4/6，状态 executing；修复后回 review 复验）。
- 2026-09-15 stage:work（repair）| PLAN-624 | rev 2 | **outcome: pass** |
  code_commit: 4f4069b5e（plan-624-dev）| task_ids: T-03/T-05 重开部分
  （F-01/F-02/F-03）| evidence: plan624 5/5 绿——F-01 `probe_none=Nil
  status=Str("probed-none")`（Open 前 None 态 ?str 跨状态读落地不崩）、
  F-02 `idx=Int(1) idx_miss=Int(-1)`（miss 路径落地，哨兵 -99 可判
  no-op）、F-03 p2 Edit 的 SetBody map 字面量实参改携 `.active_path`
  （jade 原始崩形）端到端 `view_dirty=Bool(true) status=Str("edited")`；
  plan622 8/8 绿（范围门：测试/语料面改动，AGENTS Category A/B）。
  blockers：无。**next: review**（复验基线 4f4069b5e；AC-6 跨仓门仍归
  merge）。
- 2026-09-15 stage:review（re-review）| PLAN-624 | rev 2 | **outcome:
  pass** | reviewed_commit: 4f4069b5e6ba6e8ca9d85c6a89511434023922d5
  （plan-624-dev）| base_commit: a06efd9f7 | dependency_revisions:
  lang-624/auto-down @ auto-lang-dev 140775f0c（clean 未动）|
  spec_inputs: docs/specs/auto-lang/ui/overview.md @ 2d47174dc（与上轮
  pass 评审**逐字节相同**——`git diff 2d47174dc..4f4069b5e -- docs/specs/`
  为空，证据按明示理由复用）。独立性声明同上轮（实现会话内评审，结论
  以当轮复现重建）。
  **增量范围**：2d47174dc..4f4069b5e 仅 3 个测试/语料文件（+52/−3），
  生产代码零改动。
  **acceptance_results（全部 pass）**：
  AC-1 pass（F-03 闭合：p2 Edit 的 SetBody map 字面量实参携
  `.active_path`——jade 原始崩形首次被**执行**断言，端到端
  `view_dirty=Bool(true) status=Str("edited")`）；AC-2 pass（F-01 闭合：
  Open 前 None 态 ?str 跨状态读 `probe_none=Nil status=Str("probed-none")`
  不崩不 READ_ERR；Some 态经 dirty/edited 值证）；AC-3 pass（当轮 p3 臂
  绿）；AC-4 pass（F-02 闭合：`idx=Int(1) idx_miss=Int(-1)`，哨兵 -99
  判别 no-op）；AC-5 pass（当轮复现：tv 3706/3706 + tf 3560/3560 +
  plan622 8/8 + plan442 17/17 + plan340 11/11 + plan624 5/5）；AC-6
  pass-by-design（merge 前置跨仓门，AUTO_EXE → 4f4069b5e）；AC-7 pass
  （SD 文本未变，上轮质审证据复用）。
  **findings**: 无新增。**环境波动记录（非回归）**：首轮 tf 单测
  `ffi_dual_019_dep_layout_invariants` 失败（1980/3560，并行负载
  6.1s）——与本计划增量无共享路径（仅 plan624 断言/语料改动），隔离
  单跑通过、本会话早前两轮 tf 亦通过、复跑全量即绿：判定为 dep-FFI
  负载敏感波动（本机多会话并行构建在案），非计划回归。
  **evidence**：可复现命令 `cargo test -p auto-lang --features ui-iced
  --lib plan{624,622,442,340}`、`cargo tv`、`cargo tf` @ 4f4069b5e；
  工件 = diff 2d47174dc..4f4069b5e（plan624_cross_state_tests.rs +
  p2_app.at + p4_app.at）。
  **next: merge**（AC-6 jade vm-smoke AUTO_EXE 全臂绿为 merge 前置门；
  canonical spec 落账与 ledger 刷新随 merge）。
- 2026-09-15 merge 收据 **PLAN-624:r2**：
  - `prepared` ✅ fold master 入 plan-624-dev（merge 141ed2c4d，无冲突；
    master 侧 22 commits 含 PLAN-018/015 线）+ fold 后刷新验证 plan624
    5/5、plan622 8/8、tv 3707/3707、tf 3561/3561；账本投影预备
    （P624-1..6 六节镜像 622 schema，staged 校验读回过）。
  - **AC-6 前置门 ✅**：jade vm-smoke（AUTO_EXE=worktree 构建
    auto 0.1.0+v0.4.2-715-g141ed2c4d）**split PASS + merged PASS**
    （各 15✓ 断言行，fixture 前后哈希一致）。排障实录：facade WIP 形态
    首跑红 =彼仓 tabs_store.at 模型**缺 active_path 声明**（622 转写
    遗失——注释在、声明行失），SET_FIELD 严格臂报
    "Field 'active_path' not found on type instance App_State"；
    彼仓 WIP 内一行声明修复（`var active_path str = ""`，未提交，归
    jade 侧）后 smoke 全绿。已提交（workaround）形态双模 PASS 证
    本计划零跨仓回归；facade 形态排除 VM 缺陷后仅余彼仓字段契约
    未对齐（active_* → view_* 改名未同步 smoke），归 jade 侧落地。
  - `landed` ✅ master merge `0b5a23d08`（Merge branch 'plan-624-dev'；
    落地瞬间 master 已再前进至 c240fb216——PLAN-015 D4 engine_menu_take
    shim，自动合并成功）+ 落地后主检出复验 plan624 5/5、plan622 8/8、
    plan442 17/17、plan340 11/11、tv 3707/3707、tf 3561/3561 全绿；
    overview.md「PLAN-624」节在 master 在案。
  - `ledger_refreshed` ✅ .autoos/specs.json（运行时账本，未跟踪）原子
    发布 P624-1..6 六节（staged→os.replace；发布前基线一致性校验过——
    剥离 P624 项后与当前账本逐字节等价；读回 6/6，总 518 items）。
  - `archived` ✅ plan → docs/plans/archive/624-store-facade-cross-state-
    resolution.md + status: archived（本次提交）。
  - `cleaned` ✅ 双 worktree guard clean（wt-guard: auto-lang/auto-down
    均无 reparse point）→ worktree 移除（auto-lang 由本仓、auto-down 由
    彼仓注销）+ 分支删除（plan-624-dev @ 141ed2c4d ∈ master 0b5a23d08
    祖先链已验证 / auto-lang-dev @ 140775f 未改动）→ 空组目录
    D:/autostack/.wt/lang-624/ 移除。全检查点闭合。

## 待澄清事项

无阻断项。三个执行期裁定点已内嵌任务：①P1 修复三选一（运行时回退 / 合成期
重写 / 合并态视图对象）由 T-01 差异结论裁决；②P3 报错口径的存量暴露清点
（T-04 内完成，存量用法逐个确认等价改写或登记）；③跨仓复验的执行位置
（AUTO_EXE 直跑 vs fold 后 jade 侧复跑，T-06 按 merge 材料记录）。
