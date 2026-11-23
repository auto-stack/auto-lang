---
plan_id: PLAN-1130
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: AAC CL 原语面与 std 闭包实现
author: [DeepSeek]
created_at: 2026-11-24
updated_at: 2026-11-24

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-ac/aac-std-closure.md]
touched_goals: [GOAL-002]

affects: [auto-ac]             # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 9
plan_revision: 1
---

# [PLAN-1130] AAC CL 原语面与 std 闭包实现

## 0. 变更摘要

2026-11-24 起草（r1，仅规划，未实施、未建树、未运行产品）。由 PLAN-1054 的 blocked 收据与
逐模块门面清单派生：自举 AAC 前端要吃到"全树 S + Windows std 闭包"，当前缺三类面
（parser 列表迭代、checker 原语/泛型构造、stdlib 闭包/待办面），且这些面的生产 owner 在
1046–1052 归档后空缺。本计划是**生产方**实现计划（不是认证计划）：只负责把上述 CL 面实现到
"全树 project 编译可越过 parse/bind/typecheck 进入 HIR/MIR 并可被 AV 消费"的边界；
Gen2/代际签章仍归 PLAN-1054 T-02，G2（mn 16MiB）归 AGE elem-view owner。

依据证据（PLAN-1054 材料区，尚未入 git）：
`plan-artifacts/plan-1054/cl-frontdoor-inventory.md`（SHA256 前 16 `7567F4153D81358D`）
与 `module-frontdoor-scan.txt`：85 模块中 66 个解析干净、7 个 parser 缺口、6 个 checker 缺口、
4 个 `aac.pending-work/1`、1 个 `hir.unsupported`。

## 1. 目标

让自举 AAC 前端（`experimental/ace/aac/**`）接受并降到 base MIR 的 CL 面，覆盖

1. **parser**：`for <binding> in <collection> { ... }` 列表迭代（7 个 std 模块的主缺口形态）；
2. **checker**：`Str.char_at(i)`、`char.to_codepoint()`、`char.from_codepoint(p)` 以及
   `List<T>.new()`（含嵌套 `List<List<int>>.new()`）的事实/类型；
3. **HIR/MIR/ABC**：上述形态的 checked HIR 表示、base MIR/ABC lowering 与 AV 消费端独立执行；
4. **std 闭包**：S 实际 import 的 `std.{bytes,conv,env,file,path,process,syscommon,sysplat_win,text,time}`
   在 project 车道可绑定/定型；`aac.pending-work/1` 面要么实现，要么**具名拒绝并分期**，
   不得静默通过。

出口（§7）：全树 85 模块 project 编译不再被 parser/checker/std 面挡住——
要么产出 MIR/ABC，要么给出**完整具名拒绝清单**（阶段+code+span），且 4 个固定组无新增红。
本计划**不**承诺 Gen2 native PE（1054 T-02）、不承诺 G2（AGE elem-view）、不承诺完整 53 个 CL 族
的机器执行（只覆盖 S/std 闭包实际到达的行）。

非目标：A2C/A2PY/A2R 的 emit parity；运行时性能；Linux host；把 CL 行从 1053 的 53 族里删除。

## 2. 架构方案

沿用既有 AAC 主体链，不新建第二套编译器：
`原始源码 → syntax_parse/syntax_program（规则程序+CST/AST）→ semantic_tree.parse_semantic_source
→ binding → declarations/typecheck → core_hir（Checked HIR）→ mir/{base_ir,lower_base,verify_base,base_abc}
→ ABC → AV/native 消费端`。

改动只落在**生产点**：
- 语法/投影：`experimental/ace/aac/syntax/{syntax_parse,syntax_program}.at`，必要时
  `experimental/ace/grammars/auto-source.at`（共享 grammar 源）+ 重新生成
  `experimental/ace/aac/syntax/syntax_grammar_data.at`（生成器在 T-01 具名，不手改数据文件）。
- 检查：`experimental/ace/aac/semantics/{binding,typecheck,types}.at`。
- HIR：`experimental/ace/aac/hir/core_hir.at`（含 HIR 文本 schema 与 verify）。
- MIR/ABC：`experimental/ace/aac/mir/{base_ir,lower_base,verify_base,base_abc}.at`。
- std 闭包：`experimental/ace/tests/aac-syntax/semantic_driver.at`（project 装载面）与
  `experimental/ace/tests/aac-gen1/src/bin/gen-next.rs`（闭包物化；1054 已起步）。

边界：Rust 侧（`experimental/ace/crates/{core,av-core,av,native,a2x-rust}`）只作为**独立 oracle
与消费端**使用；本计划不修改其语义，若必须新增 AV runtime op，走 `av-core` 合同 + 独立测试，
并由实际 owner 评审（见 §10）。

## 3. 技术栈

Auto 自举前端（Gen0 编译阶段 driver）+ ACE AV/AC 消费端；Windows x64 hosted 首版；
构建/验证全部在 auto-lang 仓内固定组进行；不引入新的外部依赖。

## 4. 需求分析与背景调查

**授权**：用户 2026-11-24 裁定"新开实现计划"（在"新开计划 vs 给 1054 加 phase"中选定新计划）；
未给出预算/自动续行上限；本仓修改范围限 auto-lang；仓库外的 AGE elem-view 不在本计划。
本计划起草阶段不改产品代码、不建 worktree、不运行产品（本节仅静态取材与既证引用）。

**规范输入**（实读，含内容 SHA256 前 16）：

| 来源 | 指纹/事实 |
|---|---|
| `docs/specs/auto-ac/aac-source-frontend.md` | `42E8A4A562DC9E1D`；8 个 profile；`aac` 组合普通/AGE/包内编译期语法 |
| `docs/specs/auto-ac/aac-complete-semantics.md` | `08D475D1EB2BB4C5`；明确"分支/循环/早退、?T/!T、闭包…为具名后继" |
| `docs/specs/auto-ac/aac-mir-lowering.md` | `B30A9DEF9C41D048`；base MIR 已含标量/record/owned-text/短路/break/continue |
| `docs/specs/auto-ac/aac-baseline-and-subject.md` | `C5EEA8EB376AD839`；主体与阶段范围 |
| `experimental/ace/grammars/auto-source.at` | `9E175910BAB18FC5`；行 107 记"集合族最小语句面（for-in 迭代…）"，行 112 记 range-for 不在集合面 |
| `experimental/ace/aac/syntax/syntax_grammar_data.at` | `71D5E8B94A345F1D`（341,082B，由共享 grammar 生成，**不得手改**） |
| `experimental/ace/aac/syntax/syntax_parse.at` | `F383A49924B034E5` |
| `experimental/ace/aac/semantics/typecheck.at` | `46793688DFC3F213` |
| `experimental/ace/tests/aac-syntax/semantic_driver.at` | `1D7726A45F5F5B`（project/inventory 面） |

**实测缺口（PLAN-1054 goal 第 6 轮，单文件门面扫描；85 模块）**：

- parser（7）：`std/file` 6097、`std/list` 783、`std/process` 7605、`std/syscommon` 8165、
  `std/sysplat_win` 12327、`std/zip` 5015（形态均为 `for x in <collection>`）、`std/text` 7209（宽 span，需 T-01 定位）。
- checker（6）：`paged.at` 1869 `List<List<int>>.new()`；`std/char` 756 `.to_codepoint()`；
  `std/conv` 661 / `std/hash` 752 `.char_at()`；`std/map` 628 泛型构造；
  `syntax_grammar_data` 359 `List<Str>.new()`；另 `hir_schema_data` `type.return`（合成模块，T-01 判定是否真实缺口）。
- 前端显式待办：`aac.pending-work/1` = `std/env`、`std/path`、`std/rand`、`std/time`；
  `hir.unsupported` = `std/math` 4796。
- S 的 std import 集合：`std.{bytes,conv,env,file,path,process,syscommon,sysplat_win,text,time}`。
- 111 个 S/std 模块在**单文件**模式下 66 个停在 `bind.module`（=解析干净）。

**依赖与边界**：PLAN-1054 消费本计划交付（T-02/T-03）；PLAN-1068 是最终三端集成方（仍 drafting）；
G2 归 AGE elem-view owner。1053/T-07 的 53 个 blocked CL 族是**分母**，本计划只实现 S/std 到达的行，
不删除其它行。

## 5. 详细设计

### 5.1 parser：列表迭代

形态：`for <binding> in <iterable> { <body> }`，`<binding>` 为局部不可变绑定，作用域=body；
`<iterable>` 支持 `List<T>` 与 `Str`（逐标量）两种；range 形 `for i in a..b` 保原语义不变。

决策点（T-01）：共享 grammar 已有 for-in 生产（auto-source.at 行 107），需先判定三选一：
(a) `aac` profile 未包含该生产 → 扩 profile + 重新生成 grammar data；
(b) 生产在但 `syntax_parse` 的 AST 投影/执行未接 → 补投影；
(c) 两者皆有但形状不匹配 std 用法 → 有界修正。
T-01 产出决策记录（含实测命令与结论），再进 T-02；不得为了绕过而手改 `syntax_grammar_data.at`。

### 5.2 checker：原语值与泛型构造

- 新增/补齐事实：`Str.char_at(int) -> char`；`char.to_codepoint() -> int`；
  `char.from_codepoint(int) -> char`；`List<T>.new() -> List<T>`（含嵌套泛型实参）。
- 现有 `type.unresolved`（未知方法/未知静态）**保持**为拒绝（负控）。
- 不引入隐式转换；`char` 仍与 `int` 区分（`char` 的表示沿用现有 char_type）。

### 5.3 HIR：builtin callee 形

primitive 调用没有可指向的声明行，HIR 增加**闭集 builtin callee** 形（如 `callee: "b:char.from_codepoint"`）：
- 文本 HIR 对既有程序**零变化**（仅新形态出现新键值）；
- `verify_core_document` 只接受闭集内的 builtin 名，未知名具名拒绝；
- builtin 调用的实参按既有 `eval_args`/`bindings` 映射校验。

### 5.4 MIR/ABC 与 AV 消费

- 迭代：lower 成"索引循环"（ListLen/ListGet + 合成索引槽），绑定槽按 body 作用域；`Str` 迭代
  走标量序列（若 runtime 无现成 op，按 §10 的 op 增量流程）。
- `char_at`/`to_codepoint`/`from_codepoint`：优先复用 AV 合同已有 op（`av-core` 里文本/char 族），
  缺 op 时按 `av-core` 合同新增 op + 独立 conformance，不改变既有 op 语义。
- 泛型构造：LIST_NEW（元素类型进类型表）。
- 拒绝分类：不支持面必须落 `hir.unsupported`/`mir.*`/`av.*` 的具名阶段码，不允许 trap 或空成功。

### 5.5 std 闭包与待办面

- project 车道：`semantic_driver.at` 按 `std/<stem>` 装载；`gen-next` 物化
  `stdlib_assembled_files()+stdlib_sys_files()`（Windows 过滤 `*_lnx`）。
- S 需要的 10 个 std 模块绑定/定型可达；`aac.pending-work/1` 四模块（env/path/rand/time）：
  逐条决定"实现"或"具名分期"（含阶段码与受影响 CL 行），不得静默通过；`std/math` 的
  `hir.unsupported` 同法处置。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-ac/aac-source-frontend.md` | 增补 `for <binding> in <collection>` 语句面（aac profile）、AST 角色与无损投影；range-for 语义不变 | parser 缺口是 S/std 的第一墙 | AC-02 |
| SD-02 | modify | `docs/specs/auto-ac/aac-complete-semantics.md` | 增补 Str/char 原语方法与 `char.from_codepoint` 静态、泛型构造事实；未知方法仍拒绝 | checker 缺口 | AC-03 |
| SD-03 | modify | `docs/specs/auto-ac/aac-mir-lowering.md` | 增补 builtin callee 的 HIR 形、迭代/原语 MIR/ABC lowering 与拒绝分类 | 生产面落地 | AC-04/05 |
| SD-04 | add | `docs/specs/auto-ac/aac-std-closure.md` | 新增编译车道 std 闭包装载、Windows 平台选择、`aac.pending-work/1` 分期规则 | 目前无规范描述该车道 | AC-06/07 |

## 6. 测试设计

验证入口：固定组 `ace-aac-syntax`（1046/1047 driver；须 `AAC1046_NATIVE_DIR`/`AAC1047_NATIVE_DIR`
经 `prepare-native` 预备）、`ace-aac-gen1`（`AAC1053_GEN1_DIR`）、`ace-aac`（support 包）、
语义车道命令
`cargo nextest run --offline -p ace-aac-syntax-conformance --features aac-semantics --lib --tests --config-file ../../.config/nextest-full.toml --test-threads 1`；
全树 project 扫描工具复用 PLAN-1054 的 `module-frontdoor-scan`（按 §8 T-07 落为本计划材料区脚本）。
预期命令/结果在 §8 各任务中逐条写；新增测试一律先标"未执行"。

| Case ID | AC / owner | 输入/状态 | 入口 | 独立预期与边界 |
|---|---|---|---|---|
| V-01 | AC-01 / work | 现 master 与 S/std 闭包（85 模块） | T-01 扫描 + 决策记录 | 逐模块阶段/span 与 1054 清单一致或差异有解释；不得改变既有固定组 oracle |
| V-02 | AC-02 / work | `for x in <List>`、`for c in <Str>`、空集合、嵌套迭代；非法近邻（缺 in、range 误用、绑定重名） | `ace-aac-syntax` + 新 golden（parse/AST 报告） | 合法形 CST 无损、AST 绑定角色正确；非法形在 source 阶段具名拒绝；range-for 既有 goldens 不变 |
| V-03 | AC-03 / work | 三类原语调用 + `List<T>.new()`（含嵌套）；错参（char_at(Str)、from_codepoint(bool)）、未知名 | 语义车道 type 事实/负控 | 正确事实与类型；非法在 type 阶段具名拒绝；未知方法仍 `type.unresolved` |
| V-04 | AC-04 / work | 上述合法形 | HIR 文本 + verify | HIR 文本对既有程序逐字节不变；新形 builtin callee 进闭集；未知 builtin 拒绝 |
| V-05 | AC-05 / work | 代表性程序（迭代求和、char_at/索引、to_codepoint/from_codepoint 往返） | `run_base_mir_case` + 独立 AV 消费端 | MIR/ABC 产出；AV 执行值与**独立字面 oracle**一致；资源/失败分类正确 |
| V-06 | AC-06 / work | S 的 10 个 std 模块 + 4 个 pending 面 | project 车道（`gen-next` 闭包） | 绑定/定型可达；pending 面逐条具名（实现或分期带阶段码），不得空成功 |
| V-07 | AC-07 / work | 全树 85 模块 project | `gen-next`（或等价 project 编译命令） | 越过 parse/bind/typecheck 进入 HIR/MIR；剩余拒绝为**完整具名清单**（阶段+code+span），零 trap |
| V-08 | AC-08 / review | 冻结 revision 的全部证据 | `/auto-plan:review` | 每 AC 逐条核对；漏项/workaround/警告/格式/规范 delta |
| V-09 | AC-09 / merge | review pass + 落主身份 | `/auto-plan:merge` | 规范沉淀、归档、清理；与 1054 的交接 |

负控与边界：不得用"Gen0 重编"冒充 AAC 结果（driver 处理被测源码时禁用 Rust 源码前端）；
不得把 pending 面当 PASS；不得删 53 族行；固定组原 oracle/条件保持。

## 7. 验收标准

- [ ] **AC-01 / owner: work**：T-01 决策记录落盘（生成器/形态/三选一路径），刷新 85 模块清单，
  输入身份（S/std 指纹）与 1054 证据可追溯；固定组基线记录。
- [ ] **AC-02 / owner: work**：`for <binding> in <collection>` 在 `aac` profile 实现并投影，
  V-02 全绿；range-for 与既有 parse goldens 零回归。
- [ ] **AC-03 / owner: work**：原语方法与泛型构造事实落地，V-03 全绿；未知方法仍拒绝。
- [ ] **AC-04 / owner: work**：HIR builtin callee 闭集 + verify，V-04 全绿；既有 HIR 文本零变化。
- [ ] **AC-05 / owner: work**：迭代/原语 MIR/ABC lowering + AV 独立执行，V-05 全绿；拒绝分类正确。
- [ ] **AC-06 / owner: work**：S 的 std 闭包在 project 车道可绑定/定型；pending 面逐条具名，V-06 全绿。
- [ ] **AC-07 / owner: work**：全树 project 编译越 parser/checker/std 面，产出 MIR/ABC 或完整具名清单，
  固定组（`ace-aac-syntax`/`ace-aac-gen1`/`ace-aac`）无新增红；证据折入本计划。
- [ ] **AC-08 / owner: review**：独立复审逐 AC 核证据与规范 delta；不足返回 needs_fix/needs_replan。
- [ ] **AC-09 / owner: merge**：落主 + 规范沉淀 + 归档清理；向 PLAN-1054 交可消费的生产方收据。

## 8. 执行步骤

> 实施前在当前 harness 注册这些任务 ID 并逐条跟踪；每条完成追加 `[✅ 已完成]` 证据行。
> 专树：`D:/autostack/.wt/lang-1130/auto-lang`，分支 `plan-1130-dev`；材料区唯一
> `D:/autostack/plan-artifacts/plan-1130/`（禁止 junction/symlink，禁止写仓目录）。

- [ ] T-01 基线、清单刷新与 parser 缺口决策（work）
  - 依赖：本计划获用户确认；master 当前 HEAD（起草时 `66e866017` 系）。
  - 文件/符号：`experimental/ace/grammars/auto-source.at`、`experimental/ace/aac/syntax/{syntax_parse,syntax_program}.at`、
    grammar data 生成器（在 `experimental/ace/crates/syntax-grammar` 内定位）、`semantic_driver.at`。
  - 产出：`plan-artifacts/plan-1130/gap-decision.md`（生成器命令、profile 选择、为何当前 7 个 std 模块被拒的实测证据）。
  - 验证：85 模块扫描表与 1054 清单逐条对齐（差异有解释）；`prepare-native` 双 driver 构建 rc=0。
    AC-01。
- [ ] T-02 parser：列表迭代语法（work）
  - 依赖：T-01。
  - 文件/符号：grammar 源/profile（按 T-01 结论）、`syntax_grammar_data` 重新生成、`syntax_parse.at`
    语句分支、`semantic_tree.at` 绑定点（如需）。
  - 产出：7 个 std 模块不再 `source.syntax`；新 golden（合法/非法/空集/嵌套）。
  - 验证：`cargo t ace-aac-syntax`（预期 51/51 或仅既有并发账）；语义车道 type/parse 用例全绿；V-02。AC-02。
- [ ] T-03 checker：原语与泛型构造事实（work）
  - 依赖：T-02（可并行，若 parser 不阻塞）。
  - 文件/符号：`experimental/ace/aac/semantics/{types,typecheck,binding}.at`。
  - 产出：`type.unresolved` 清单从 6 项收敛（`paged.at`/两处内核模块单列）；负控保持拒绝。
  - 验证：语义车道 + 新负控；V-03。AC-03。
- [ ] T-04 HIR：builtin callee 与 verify（work）
  - 依赖：T-03。
  - 文件/符号：`experimental/ace/aac/hir/core_hir.at`（闭集、verify、文本形）。
  - 产出：既有 HIR 文本逐字节不变（对 1050/1046 goldens 校验）；新形可见。
  - 验证：语义车道 HIR 用例；V-04。AC-04。
- [ ] T-05 MIR/ABC：迭代与原语 lowering（work）
  - 依赖：T-04；若需新 AV op，先按 `av-core` 合同走（§10）。
  - 文件/符号：`experimental/ace/aac/mir/{base_ir,lower_base,verify_base,base_abc}.at`。
  - 产出：V-05 三例真实执行；阶段码分类。
  - 验证：`run_base_mir_case` + AV 独立 oracle；语义车道 mir 用例；V-05。AC-05。
- [ ] T-06 std 闭包与待办面（work）
  - 依赖：T-02..T-05（按模块推进）。
  - 文件/符号：`experimental/ace/tests/aac-syntax/semantic_driver.at`、`experimental/ace/tests/aac-gen1/src/bin/gen-next.rs`、
    `stdlib/ace/**`（只读，除确需修复的模块另具名）。
  - 产出：10 个 std 模块绑定/定型；4 个 `aac.pending-work/1` 逐条决定（实现或具名分期+阶段码）。
  - 验证：V-06；`cargo t ace-aac ace-aac-gen1` 无新增红。AC-06。
- [ ] T-07 全树 project 门与固定组（work）
  - 依赖：T-06。
  - 文件/符号：扫描脚本落 `plan-artifacts/plan-1130/`；`cargo t` 固定组并集。
  - 产出：85 模块 project 编译结论（MIR/ABC 或完整具名清单）；台账记录执行/排除/耗时。
  - 验证：V-07；`cargo t ace-aac-syntax`、`ace-aac-gen1`、`ace-aac`；证据折入计划。AC-07。
- [ ] T-08 独立复审（review）
  - 依赖：T-07；冻结 revision。
  - 产出：逐 AC 结论 + 漏项/延期/workaround/警告/格式/规范 delta 核对。
  - 验证：V-08。AC-08。
- [ ] T-09 合入、沉淀与清理（merge）
  - 依赖：T-08 pass。
  - 产出：落主、Spec 沉淀（SD-01..04）、归档、材料区清空、wt-guard clean。
  - 验证：V-09。AC-09。

## 9. 复审记录

- 2026-11-24 stage: new | PLAN-1130 | r1 | outcome: pass（草稿就绪；生产方实现计划，输入=1054 门面清单/53 族/Design59 归属；未实施、未建树、未运行产品） | next: work（待用户确认本计划稿后开专树执行） | changed: 新建 AC-01..09、T-01..09、SD-01..04、V-01..09；无既有任务/规范被删除

## 10. 待澄清事项

1. **本计划稿需用户确认**（起草依据是用户"新开实现计划"的裁定；确认后即开
   `D:/autostack/.wt/lang-1130/auto-lang` @ `plan-1130-dev` 并执行 T-01）。
2. **AV runtime op 边界**：T-05 若发现现有 `av-core` 合同缺 char 迭代/转换 op，可按合同增补并配
   conformance，但涉及 `av-core`/ACE AV owner 的语义变更须具名交接，不在本计划内私改其它计划的
   冻结面。
3. **G2（mn 16MiB）不在本计划**；全树 Gen2 native PE 仍归 PLAN-1054 T-02，触发点为 1054 续跑的
   `gen-next`。本计划交付后 1054 需重新绑定 S 身份并重跑 T-02..T-09。
4. **1054 已在途的两处工具改动**（`gen-next` std 闭包、512KiB 准入）在 master 尚未合入；本计划
   T-06/T-07 以 master 为准自行落地同等能力，或等 1054 合入后复用（T-01 决定，避免重复实现）。
