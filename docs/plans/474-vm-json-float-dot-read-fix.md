---
plan_id: PLAN-474
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-json-float-dot-read-fix
author: [zhaopuming, zcode]
created_at: 2026-08-29
updated_at: 2026-08-29

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-val]   # 受影响的 specs 路径
current_step: 0
total_steps: 8
---

# [PLAN-474] VM 轨 `__json_object` 浮点字段 Dot 读误码修复（plan011④）

## 变更摘要

VM 解释器路径下，经宿主桥/JSON 进入前端的浮点数据不可信：JSON 对象浮点字段单跳 Dot 读出错乱值（`54.16` → `-1073741824`/0xC0000000，`.floor()` 得 0），整数字段（`as_i64`→Int）与字符串字段正常。本计划在本仓建立最小复现 → 沿读取链二分定位注入点 → 根因修复 → 回归钉死，并在修复落地后为 auto-os-config 撤除 T12 整数化绕法（os-config 侧动作，本计划只登记移交项）。

证据链与位型指纹分析已预置于 `docs/plans/KNOWN-DEBT-AND-RISKS.md` p1(plan011④) 条目（commit 949389cb5），本计划是其执行载体。

## 目标

1. **最小复现**：在 auto-lang 仓内构造 VM 级复现（json → `__json_object` → 编译后 Dot 读 → 断言位型），脱离 os-config worktree 依赖。
2. **定位注入点**：二分读取链（GET_FIELD 入栈 → 局部槽存储 → 消费方弹栈/插值/回读桥），找到 0xC0000000（-2.0f32 完整位型或 int -2^30）的注入位置。
3. **根因修复**：在注入点修复（tag 守卫/tag-first decode，对齐 fb06cd8b2①/bb6608f75③ 先例风格），禁止在 os-config 侧绕。
4. **回归钉死**：浮点/整数/字符串/bool/null 字段全类型回归测试；全量门禁绿。
5. **债务销账**：KNOWN-DEBT-AND-RISKS.md ④ 条目翻 ✅ 并附修复提交引用。

## 架构方案

不引入新模块。修复点预期落在既有读取链的三层之一（按嫌疑排序）：

- **L1 消费方层**（最可能）：GET_FIELD 结果 nv（裸 f64 位，不装箱）被消费方按 i32/f32 弹栈解码——expr_type_hint 对 `__json_object` 未知字段的类型提示、整型算术/插值的 pop 路径、float 方法分派（`.floor()` 得 0 提示 operand 未进 float 分派）。
- **L2 物化层**：vm_bridge 模型字面量物化为 GenericInstanceData 的中间路径（若读取的实例并非 json_to_vm_value 直建的那只）。
- **L3 回读桥层**：UI 状态回读（autoui_state 系）对 Double 值的序列化。

写入侧已三级排除（见需求分析），不在猎场内。

## 需求分析与背景调查

**来源**：docs/specs/overview.md（auto-lang：lexer/…/VM 九模块；auto-val：Value/Node 体系含 nano_value）＋ 本会话两轮核实 ＋ os-config plan011 待澄清#5④（commit 117826b 回写线索）＋ auto-os-config 探针实测。

**症状**（os-config 现场，worktree plan-011-dev）：Rust 侧 `system_info_json()` 返回 `{"storage_free_gb": 54.16}` → `auto.host.call` → `auto.json.to_value` → 前端 `.storage_free_gb` 单跳 Dot 读 → `-1073741824`；对该值 `.floor()` 返回 0。整数/字符串字段正常。T12 后正式代码已全整数化绕开（f85ab91）。

**写入侧三级排除**（代码级核实，file:line 为 master 当前）：

1. `crates/auto-lang/src/vm/ffi/stdlib.rs:2404-2411`（`json_to_vm_value_inner` Number 分支）：`as_i64`→`Int(i as i32)`，否则 `Double(f)`——浮点产 `Value::Double`，正确。
2. `stdlib.rs:2363-2380`（Object 臂）：字段值经 inner **直存** `GenericInstanceData::new_with_names`，无栈往返——os-config 探针实测亦证 `Double(f)` 正常入字段。
3. `crates/auto-lang/src/vm/engine.rs:5144`（GET_FIELD GenericInstanceData 臂 Double 分支）：`task.ram.push_f64(*d)`——正确。

**位型指纹约束**（本轮算实，排查方向的定海神针）：

- 54.16 裸 f64 位 = `0x404B147AE147AE14`，低 32 位 = `0xE147AE14`；54.16 f32 位 = `0x4258A3D7`。**三者均非 0xC0000000** ⇒ 误读值**不是** 54.16 自身位型的宽度错读/截断。
- 0xC0000000 = **-2.0f32 完整位型**（`encode_f32` payload，`nano_value.rs:82`）；亦 = int -2^30 = bool/null 哨兵 `i32::MIN`（0x80000000，`nano_value.rs:66-73`）**算术右移 1 位**。
- nanbox 关键事实：裸 f64 不装箱（`encode_f64 = 原始位`，`nano_value.rs:46`；`is_f64 = !is_nanboxed`）；`decode_i32` 无 tag 守卫恒取低 32 位（`nano_value.rs:130`）。
- ⇒ 猎场 = 「读取链在哪注入了 -2.0f32 / -2^30 哨兵」，**不是**「哪里把 54.16 读错宽度」。

**同族先例**（修复风格对齐对象）：

- `fb06cd8b2`①：`decode_i64_full/u64_full` 兜底一律 `decode_i32`，f32 位型被当整读（280.0f32 → 1133248512）——修法：兜底改 tag-first。
- `bb6608f75`③：extract Float 分支无 tag 守卫，字符串哨兵被 `pop_i32` 解码成 -2——修法：加 tag 守卫。
- `bb6608f75`⑥：f32 算术盲 `pop_f32` 把 int 80 位重解释——修法：tag 驱动弹出。

**旁证**：`.floor()` 返回 0 与「operand 未进 float 方法分派、落默认 0 臂」一致（引擎各 match 的 `_ => push_i32(0)` 兜底族）。

## 详细设计

### 阶段 A：最小复现（脱离 os-config）

新建测试模块 `crates/auto-lang/src/tests/vm_json_float_read_tests.rs`，在 `crates/auto-lang/src/lib.rs` 测试区（≈:5725-5772，`mod vm_types_tests;` 同款扁平声明）注册。测试走**真实编译+执行链**（非直接 API 调用）：

1. `json_to_vm_value` 物化 `{"storage_free_gb": 54.16, "n_cpu": 8, "host": "abc"}` 为堆上 `__json_object`；
2. 编译执行等价 `.at` 片段（复用仓内既有「编译+run+取栈顶」测试基建，参照 plan340_tests 的 harness 用法）对 `info.storage_free_gb` 求值；
3. 断言栈顶 nv == `encode_f64(54.16)`（位级相等，不是数值相等——位级才能钉死注入）。

**判定分支**：RED ⇒ 注入点在仓内基础链，进入阶段 B 仓内二分；GREEN ⇒ 注入点在 os-config 特有链（vm_bridge 物化/UI 回读），复现载体扩展为「vm_bridge 模型物化 + 前端绑定」形态后再二分（同样在本仓，auto-man/ui 面证据：d068d1ab2 ① 曾证 vm_bridge 把模型字面量物化为 GenericInstanceData）。

### 阶段 B：读取链二分（探针法，临时 eprintln 探针随修摘除）

按序打点，相邻两点间首次出现 0xC0000000 即收敛区间：

- P1：`engine.rs:5144` push_f64 之后立即读回 `task.ram` 栈顶（预期正确）；
- P2：Dot 表达式求值完成、store 局部槽之后；
- P3：消费方弹栈点（插值/算术/方法分派的 pop 侧）；
- P4（若 A 判定走 os-config 链）：UI 状态回读桥序列化前后。

辅助静态扫（低成本先行）：`grep -rn '\-2\.0' crates/auto-lang/src/vm crates/auto-val/src`（直接哨兵字面量）、全量 `decode_f32/pop_f32` 调用点逐一核 tag 处理、expr_type_hint Dot 读对未知/`__json_object` 字段的提示分支。

### 阶段 C：根因修复 + 回归

- 在注入点加 tag 守卫或改 tag-first 解码（对齐先例风格；禁止新增哨兵）；
- 回归矩阵：Double/Int/Str/Bool/Null 字段 × {Dot 读, `.floor()` 派发, f-string 插值}；
- 探针摘除，stderr 零残留（对齐 ac2e08856 摘针纪律）。

### 阶段 D：销账与移交

- KNOWN-DEBT-AND-RISKS.md ④ 条目翻 ✅（附修复 commit）；
- 移交项登记（非本计划执行体）：auto-os-config 撤 `memory_display`/磁盘 `display` 展示串绕法、恢复原始数值字段、e2e-vm 全绿后走 T11/backend 退役与 `/auto-plan:review` 收口（其 plan011 待澄清#5④）。

## 测试设计

| 层 | 用例 | 期望 |
|---|---|---|
| 单元（新增 `vm_json_float_read_tests.rs`） | `__json_object` Double 字段 Dot 读 | 栈顶 nv 位级 == `encode_f64(54.16)` |
| 单元 | 同实例 `.floor()` 方法调用 | 54（不落默认 0 臂） |
| 单元 | Int 字段 / Str 字段 / Bool 字段 / Null（缺键，engine.rs:5173 路径）Dot 读 | 8 / "abc" / encode_bool / encode_null 原样 |
| 单元 | f-string 插值 `"{info.storage_free_gb}"` | "54.16" |
| 既有回归 | 全仓浮点族（437 几何/Plan 340 转换器相关） | 不退化 |

门禁分级（AGENTS.md Category B）：开发期 `cargo check -p auto-lang` + `cargo t vm_json_float`；折叠前一次 `cargo tf` 全量。

## 验收标准

1. 最小复现测试在 master 基线为 RED（或 GREEN 时已按判定分支扩展复现载体至 RED），并记录实测误读值；
2. 修复后：浮点字段 Dot 读位级精确、`.floor()` 正确、Int/Str/Bool/Null 字段与缺键路径行为不变（回归矩阵全绿）；
3. `cargo check -p auto-lang` 零警告、无探针/调试打印残留；
4. `cargo tf` 全量绿（折叠前一次）；
5. KNOWN-DEBT-AND-RISKS.md ④ 条目 ✅ 销账，os-config 移交项已登记；
6. 修复为根因级（tag 守卫/tag-first），无新哨兵、无消费方白名单式绕行。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **S1**：master 上 commit 本 plan + `.next-id`（475）；创建 worktree `git worktree add .worktrees/plan-474-dev -b plan-474-dev`。验证：`git -C .worktrees/plan-474-dev log --oneline -1` 指向 master 头。
- **S2**：新建 `crates/auto-lang/src/tests/vm_json_float_read_tests.rs`（Double Dot 读位级断言 + floor + Int/Str/Bool/Null 用例骨架），在 `crates/auto-lang/src/lib.rs` 测试区加 `mod vm_json_float_read_tests;`。验证：`cargo t vm_json_float` —— 记录 RED 实测值（预期见 0xC0000000 或判定分支走 GREEN）。
- **S3**：按阶段 B 打点二分（P1→P4）＋静态扫（-2.0 字面量 / decode_f32 调用面 / expr_type_hint Dot 分支），把注入点收敛到具体 file:line，证据（探针输出）记入本节。验证：注入点唯一且可解释 0xC0000000 来源。
- **S4**：在注入点实施根因修复（tag 守卫/tag-first；改 `crates/auto-lang/src/vm/` 或 `crates/auto-val/src/nano_value.rs` 消费侧，不动写入侧三级已排除路径）。验证：`cargo t vm_json_float` 全绿。
- **S5**：补齐回归矩阵剩余用例（f-string 插值、缺键 null、bool 比较），探针全摘除。验证：`grep -rn 'P474' crates/ | wc -l` == 0 且 `cargo t vm_json_float` 绿。
- **S6**：`cargo check -p auto-lang` 零警告；`cargo t 340` `cargo t 437`（浮点族邻近模块）绿。
- **S7**：全量门禁 `cargo tf` 一次通过（Plan 466 档位）。
- **S8**：回 master 更新 `docs/plans/KNOWN-DEBT-AND-RISKS.md` ④ 条目 ✅（附修复 commit）；`/auto-plan:review` 独立复审；折叠 master、归档本 plan（`docs/plans/archive/`）、清理 worktree。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

1. （移交，非本计划体）os-config 撤 T12 整数化绕法的时序：本计划折叠 master 后由 os-config 侧会话执行，其 e2e-vm 全绿后走 T11 收口。
2. 若阶段 A 判定 GREEN（仓内基础链无辜），注入点在 vm_bridge/UI 链时是否拆独立 plan：届时按改动面大小定，≤3 文件则留本计划，超出则拆 M2。
