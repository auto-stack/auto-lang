---
plan_id: PLAN-622
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: store-facade-vm-semantics
author: [zhaopuming]
created_at: 2026-09-14
updated_at: 2026-09-14

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [ui/store-facade-semantics]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-lang/interpreter]
current_step: 0
total_steps: 7
---

# [PLAN-622] store-facade-vm-semantics

## 变更摘要

修复 jade-garden PLAN-064 T-05 执行期实证的 **store facade VM 缺口簇（五条）**。
`store` 定义 + `use store:` 绑定是 Plan 442 落地的既有语言形态（web 轨发射为
Pinia，VM 轨原生解释），本计划**零语法新增**，只修该形态在 VM 侧五个消费位上的
实现语义缺口：

| # | 缺口 | 病症 |
| --- | --- | --- |
| a | handler 内读 store 字段 | widget handler 上下文读 store 字段得 0/"" 哨兵（442 corpus 只证了 model-var 读路径） |
| b | 视图对 store 字段不响应回读 | 视图 text 节点绑定 store 字段，Init 后 handler 改字段视图冻结在初值 |
| c | store 内 `#[api]` 调用不走 340 改写 | store 模块的 on-handler 内调契约 fn 静默执行 stub `return None`（不落盘） |
| d | widget 模型数组 splice 静默失效 | onclick 内 `.items.splice(...)` 无效果 |
| e | lambda 捕获语义缺口 | lambda 捕获 handler 本地变量 = "undefined variable" 编译错（msg 参数捕获可用）；lambda 比较表达式内读 self 字段运行时静默失效 |

**为什么是关键路径**：这五条迫使 jade 侧每个 widget 迁移都用「widget 状态机
等价实现」绕行（064 T-05 实证），不修则 29-widget 批量迁移债务 ×28；修复后
jade tabs_store 回归 `use store: Tabs` facade 形态（跨仓回归路径已在彼侧
README §8 备案，属 jade 侧后续小步，非本计划范围）。

## 目标

- **G1 五条各补最小复现语料，先红后绿**（TDD 纪律，修复前红/修复后绿证据留痕）。
- **G2 既有语料零回归**：442 facade/webcompat、340、370、aura、musk_vm_track
  等在册语料全量绿。
- **G3 web 双发射面零扰动**：不动 a2ts 发射器，金样逐字节不变（本计划纯
  VM/解释器/改写器侧）。
- **G4 jade 侧不回归承诺**：jade vm-smoke tabs 臂（等价实现形态）照常绿；
  facade 正式切换的跨仓复验路径写清移交项。

**非目标**：不新增任何 DSL 语法/原语面；①rfd 目录选择器、②confirm 模态、
③CALL_SPEC 返回列表 RC、④tick/timer 原语（另案「宿主能力包」）；⑤ P063
渲染树两项（scroll_to 绝对偏移 / AnchorSlot downcast，已另案在案）；jade 仓内
任何代码改动（含 tabs_store facade 切换）。

## 架构方案

五条缺口分两个修复面，均在 `crates/auto-lang/src/`：

```
                    ┌─ 求值/绑定面（a, b, d, e）──────────────────────┐
 .at 源             │  interpreter/（字段解析上下文）                  │
 store{...}         │  ui/handler_codegen.rs（handler 代码生成）       │
 use store: ───────▶│  ui/aura_view_builder.rs（视图绑定/响应边/splice）│
                    └─────────────────────────────────────────────────┘
                    ┌─ 改写面（c）────────────────────────────────────┐
 #[api] 契约 fn ───▶│  lib.rs（emit_api_http_call）                    │
                    │  ui/dynamic.rs；语料 plan340_tests.rs            │
                    └─────────────────────────────────────────────────┘
```

**共享病因假设**（执行期证实或证伪）：a/b/e 同根——store 绑定的字段解析只在
「model-var 读 + 视图初读」路径贯通，handler 上下文、视图增量回读、lambda
作用域三个消费位没接到同一张解析表；c 是独立的改写器遍历范围缺口；d 是变异
原语缺写回路径。修复按消费位逐条 TDD，不引入新语法面，不做架构重构。

## 需求分析与背景调查

**授权记录（2026-09-14，auto-down 仓会话）**：用户批准就 jade PLAN-064 提案⑥
在彼仓立项，范围仅⑥五条；①-④宿主能力包与⑤渲染树两项明确不在本计划。

**跨仓证据链（均在 jade 仓在案，2026-09-14 引用）**：

| 证据 | 位置 |
| --- | --- |
| 六件提案清单（⑥五条细节 + 回归方案注记） | jade `jade-garden/front/desktop/README.md` §8 |
| DEBTS 行（⑥ + 五件旁证） | jade `DEBTS.md` 064 行 |
| 五条实证的 work 复审记录 | jade `docs/plans/archived/064-jade-vm-ext-audit-tabs-flow.md` §9 |
| 缺口触发表（tabs_store_ext 行） | jade `jade-garden/front/desktop/ext-registry.json` |
| 等价实现绕行现状（五断言全绿×2） | jade `jade-garden/front/desktop/vm-smoke.mjs` tabs 臂 |

**本仓代码锚点（实勘 2026-09-14）**：

| 锚点 | 位置 | 关联缺口 |
| --- | --- | --- |
| store facade 语料基底 | `crates/auto-lang/src/plan442_store_facade_tests.rs` + `crates/auto-lang/test/ui/plan442_*` | 全部（扩展基底） |
| `use store` 解析面 | `crates/auto-lang/src/ast.rs`、`parser.rs` | a/b/d/e |
| handler 代码生成 | `crates/auto-lang/src/ui/handler_codegen.rs` | a/d/e |
| 视图绑定/响应边 | `crates/auto-lang/src/ui/aura_view_builder.rs` | b/d |
| 340 api_over_http 改写 | `crates/auto-lang/src/lib.rs`（emit_api_http_call）、`ui/dynamic.rs`、`plan340_tests.rs` | c |
| 求值器 | `crates/auto-lang/src/interpreter/` | a/e |

**覆盖缺口根因**：442 语料验证面是「store Init、msg 派发、视图读（初值）」，
五条缺口的消费位（handler 内读、视图增量回读、store 内 api 调用、模型数组
变异、lambda 捕获）全部在 442 验收面之外。

## 详细设计

每条按「最小复现语料 → 修复方向假设 → 转绿断言」记录。病因假设在执行期证实
或证伪；证伪时按等价实现内裁定记录实际病因与修法，不扩授权。

### a) handler 内读 store 字段（哨兵值）

- 复现：store 计数器 + button onclick handler 内读 store 字段并回写 widget
  状态行；当前断言得 0/""。
- 方向假设：handler 求值上下文缺 store 绑定解析臂——把 store 绑定接入
  handler 上下文的字段解析表（对齐 model-var 读的既有解析路径，
  interpreter/ + handler_codegen.rs）。

### b) 视图对 store 字段不响应回读（初值冻结）

- 复现：视图 text 节点绑 store 字段，button handler 改字段，snapshot 断言
  文本节点跟随；当前冻结在初值。
- 方向假设：aura_view_builder 的绑定注册只建了初值读，缺「store 字段 → 视图
  节点」依赖边；补增量失效/重渲染路径（442 视图读已有初值机制可依托）。

### c) store 模块内 `#[api]` 调用未走 340 改写

- 复现：store on-handler 内调契约 fn（形态对齐 jade tabs_store 的
  read_wiki/write_wiki），AUTO_BACKEND 侧断言收到 HTTP；当前静默 stub None。
- 方向假设：emit_api_http_call 的遍历入口只覆盖 widget/handler 上下文，未进
  store 模块体；扩遍历范围（执行期 T-04 先核实遍历入口准确位置，再动刀）。
- 注意：POST 体字段级标量约定、通配 splice、query 发射、percent 编码等 340
  既有语义在 store 上下文下必须原样保持（plan340_tests 全量回归卡）。

### d) widget 模型数组 splice 静默失效

- 复现：widget model 持数组，onclick 内 `.items.splice(...)`，断言状态与
  视图列表变化；当前静默无效。
- 方向假设：splice 在 widget model 载体上缺写回路径（aura_view_builder /
  handler_codegen 的发射面）。注意与 DEBTS 022 登记的「Vec 参数按值传递」
  语义边界的关系——若同根，按该边界的稳定改写形态处置并更新登记，不引入
  引用语义新面。

### e) lambda 捕获（编译错 + self 字段静默失效）

- 复现两形态：`.list.find(t => t.path == local_var)`（捕获 handler 本地变量，
  当前 "undefined variable" 编译错）与 `.list.find(t => t.path == .active_path)`
  （lambda 内读 self 字段，当前运行时静默失效）。msg 参数捕获已可用，为对齐
  基线。
- 方向假设：lambda 捕获分析缺 handler 本地作用域臂；self 字段在 lambda 作用域
  的解析缺失。两条同在捕获/解析语义，一并修。

### 规范增量

| delta | add/modify | 目标 | before/after | rationale |
| --- | --- | --- | --- | --- |
| SD-01 | add | docs/specs/auto-lang/ui/store-facade-semantics | before：442 只证 Init/msg 派发/视图初读；after：五消费位语义契约钉死（handler 读、视图响应回读、store 内 api 改写、模型数组变异、lambda 捕获） | facade 形态成为 29-widget 迁移既定目标形态前的语义冻结 |

## 测试设计

- **新语料（先红后绿）**：`crates/auto-lang/src/plan622_store_facade_gap_tests.rs`
  五组（a-e 各一），`.at` 语料源放 `crates/auto-lang/test/ui/plan622_store_facade/`
  （命名与目录对齐 442 惯例）。每组含：修复前红证据（commit 留痕）+ 修复后绿。
- **回归面**：`cargo test` 全量（442 store_facade/webcompat、plan340、370、
  aura、musk_vm_track 等在册语料）。
- **发射面零扰动**：a2ts 金样断言（本计划不动发射器的构造性证明 + 金样跑一遍）。
- **跨仓不回归（merge 前置检查项）**：jade `vm-smoke.mjs` tabs 臂照常绿。
  执行方式：若 vm-smoke 支持指定 auto.exe 路径则指到本计划构建产物跑；否则
  在 fold 主检出后由 jade 侧复验，merge 收据注明。不作为彼仓语料门的替代。

## 验收标准

| ID | 标准 | 验证方法 | 期望 |
| --- | --- | --- | --- |
| AC-1 | 五组新语料先红后绿 | `cargo test plan622` | 修复前 5 红（证据在 commit/复审记录）、修复后 5 绿 |
| AC-2 | 既有语料零回归 | `cargo test`（全量） | 与 master 基线同口径全绿 |
| AC-3 | 发射面零扰动 | a2ts 金样 | 逐字节不变 |
| AC-4 | jade 不回归 | jade vm-smoke tabs 臂（merge 前置检查） | 全绿或收据注明 fold 后复验安排 |
| AC-5 | 语义契约入账 | docs/specs/auto-lang/ui/store-facade-semantics | 五消费位契约 + 语料索引 |

## 执行步骤

| 步骤 | 任务 | 文件/操作 | 验证 |
| --- | --- | --- | --- |
| T-01 | 五组红测语料落地 | 新建 `plan622_store_facade_gap_tests.rs` + `test/ui/plan622_store_facade/`（a-e 各一，形态取自详细设计复现节） | `cargo test plan622` → 5 红 |
| T-02 | 修 a：handler 内读 store 字段 | `interpreter/` + `ui/handler_codegen.rs`（字段解析表接 store 绑定） | a 组转绿；442 全绿 |
| T-03 | 修 b：视图响应回读 | `ui/aura_view_builder.rs`（store 字段依赖边 + 增量重渲染） | b 组转绿；a 组保持绿 |
| T-04 | 修 c：340 改写器进 store 模块体 | 先核实 `lib.rs` 遍历入口，再扩 `emit_api_http_call` 作用域 | c 组转绿；`cargo test plan340` 全绿 |
| T-05 | 修 d：模型数组 splice 写回 | `ui/aura_view_builder.rs` / `ui/handler_codegen.rs`（写回路径；DEBTS 022 边界核对） | d 组转绿 |
| T-06 | 修 e：lambda 捕获两形态 | 捕获分析 + lambda 作用域 self 解析（对齐 msg 参数捕获路径） | e 组两形态转绿 |
| T-07 | 全量回归 + 金样 + specs 落账 + 跨仓移交 | `cargo test` 全量、a2ts 金样、`ui/store-facade-semantics` 落账、jade vm-smoke 复验安排写入 merge 材料 | AC-1..5 全过 |

每步完成后在本节追加 `[✅ 已完成]` 一行证据（对齐彼仓执行规约）。

## 分支与提交归属

- worktree：`D:/autostack/.wt/lang-622/auto-lang`（分组平铺，Plan 529 布局），
  分支 `plan-622-dev`；实现全部在 worktree 内进行，plan 簿记在主检出。
- **红线（Plan 529）**：worktree 内禁止创建 junction/symlink；移除前必跑
  `bash D:/autostack/wt-guard.sh <worktree 路径>`。跨仓引用 jade 证据用
  绝对路径（`D:/autostack/auto-down` 主检出），不建链接。
- 提交归属：`plan-622-dev`；跨仓验证的运行记录（vm-smoke 输出）归档到本计划
  附件或 jade 侧 064 后续行，不在彼仓混提交。

## 复审记录

- 2026-09-14 stage:new handoff（/auto-plan:new，rev 1）：任务 T-01..07 覆盖
  AC-1..5 与 SD-01；五缺口锚点与跨仓证据链实勘在案；无阻断性待澄清。
  `outcome: pass`，`next: work`（worktree 建好后 T-01 语料先行——红测是全部
  后续修复的裁判）。

## 待澄清事项

无阻断项。三个执行期调查点已内嵌任务：①340 改写器遍历入口的准确位置（T-04
先核实再动刀）；②d 与 DEBTS 022「Vec 按值」边界的同根性核对（T-05）；③
jade vm-smoke 的 auto.exe 路径可覆盖性（T-07 merge 前核实，不可覆盖则按
AC-4 的 fold 后复验安排）。
