---
plan_id: PLAN-574
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-dual-interp-shutdown
author: [zhaopuming]
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [aavm]               # 测试基建:crates/auto-lang/src/tests/aavm2_*
current_step: 0
total_steps: 4
---

# [PLAN-574] avm+aavm 双重解释器测试路径关闭/瘦身(572 待澄清②裁定承接)

## 变更摘要

**裁定(2026-09-06,用户,572 待澄清②)**:裸 `cargo taa` 的 12 个
STATUS_STACK_OVERFLOW 测试,根因定性 = **avm+aavm / avm+aa2r 进程内
双重解释器路径本身非真实需求**——2×2 四路径(avm+aavm; a2r+aavm;
a2r+aa2r; avm+aa2r)测试矩阵是对称性设计产物;绝无真实场景"用 Rust
版 AutoVM 解释 aavm.at 再当解释器跑真实程序"。真实自举路径 = **用
a2r 把 aavm.at 转译成可执行文件再运行**(同 Rust 自举"编译器跑编译
器")。代码天然多递归 → 双层解释递归累积爆栈是结构性必然,修栈是
治标。

**处置**:关闭重型双重解释器测试,**每路径保留最小正确性锚**(简单
表达式/单层函数调用级,浅递归栈安全);重型验证的正统路径已在位
(⑤腿 compile_corpus 58/58 / compile_use_corpus / at_mode / P532 gen2
管道,全部 a2r 转译+编译+运行形态)。

## 目标

1. 12 个栈溢出测试(及其同路径未爆同胞)按裁定落地:重型语料逐件跑
   关闭,最小锚保留。
2. 覆盖对账:被关闭面在正统路径(⑤腿/at_mode/gen2)有对等覆盖或有
   明示接受的降级(逐里程碑对账表留档)。
3. 门禁健康:Windows 裸 `cargo taa` 失败数 **13 → 1**(仅余
   charts_gallery 预存,564-Q6 域);tf 不动(3460/3461)。
4. 文档同步:AGENTS AAVM 测试档说明补"双重解释器路径已裁定关闭"注
   记;KNOWN-DEBT 572 条目结算。

### 非目标(Out of Scope)

- charts_gallery 预存红(564-Q6 域,另案);
- at_mode b34_struct 宿主侧红(572 待澄清④,归因另案);
- ⑤腿/at_mode/gen2 正统路径的任何改动;
- VM 递归深度/栈治理(裁定已定性为路径问题,非栈问题)。

## 架构方案

现状(2026-09-06 侦察):12 爆栈测试全部为进程内
`run_with_capture(完整 aavm lib + 语料)` 形态(宿主 VM 解释
aavm.at+lib,再由其执行语料):

```text
路径            测试(爆栈)                                   正统替代(保留)
─────────────  ───────────────────────────────────────────  ─────────────────────
avm+aavm       aavm2_m1_lexer_corpus(5 件)                  ⑤腿 compile_corpus
               aavm2_m2_parser_corpus(30 件)                (a2r+aavm,子进程)
               aavm2_m3_typeinfo_corpus(8 件)               compile_use_corpus
               aavm2_m4_codegen_corpus(58 件)               at_mode(宿主 a2r)
               aavm2_m4_use_corpus                          P532 gen2(572 落地)
               aavm2_m5_engine_corpus / use_corpus /
                 use_errors / m3_milestone_fib
               aavm_runner_tests::test_aavm2_001_smoke
               test_aavm2_p532_lib_static_diff(aavm 腿)
avm+aa2r       aavm2_a2r_is_corpus(18 件)                   a2r+aa2r:⑤腿 harness
                                                          --trans + P532 固定点
```

改造形态(每测试二选一):
- **最小锚化**:语料清单裁至 1-2 件最小代表(如 m1 留 c01_arith;
  m2 留最小表达式件;runner 留 001_smoke 本体即最小)——保留路径
  正确性的快速诊断锚。
- **关闭**:`#[ignore = "..."]`(带裁定引用注记)优于删除——历史
  可溯,CI 显式带 `--ignored` 仍可选择性恢复跑(Linux 栈大,可能
  本就不爆)。

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

- 上游:Plan 572(T3-T5b)已修复 AA2R 挂死并闭合自举代际对拍
  (转译固定点 PASS);正统路径全绿。
- 572 复审登记 KNOWN-DEBT 572(🟢 环境限制)→ 本计划结算该条。
- 12 爆栈测试基点即红(f2ae1cb29 四路径同签名,572 t4_gate.md
  对拍在案)——关闭不构成任何回归遮挡。
- RUST_MIN_STACK=16MB([.cargo/config.toml] Plan 423)只护 libtest
  线程,nextest 主线程不受控——裁定后不再需要扩大此值。

## 详细设计

(T1 对账后回填;预案)

- **m1-m3 语料族**:测试体不动,`for` 语料清单改为常量最小集
  (m1: c01_arith; m2: 最小表达式 1 件; m3: 最小 1 件);其余件以
  裁定注释留档于清单上方。
- **m4/m5 语料族**:codegen_corpus(58)最小锚 = b01_hello 级 1 件;
  use_corpus/use_errors/engine/milestone_fib 同款最小锚化或
  `#[ignore]`(T1 按各测试语义定)。
- **aavm_runner_tests::test_aavm2 族**(001_smoke 等 `#[ignore]`
  映射):001_smoke 本身即最小——改"锚保留+栈深注记";若仍爆则
  `#[ignore]`。
- **p532_lib_static_diff**:rust 腿保留,aavm 腿(双重解释)按裁定
  `#[ignore]` 或断言降级注记——P532 判定资产归 P532 复核口径。
- **a2r_is_corpus(avm+aa2r)**:最小锚 = is 语料最小 1 件;其余
  关闭。⑤腿 aavm2_a2r 其余测试(main_dump/goldens_check)非双重
  解释形态,不动。

## 测试设计(TDD)

- 红基线:改造前裸 `cargo taa --no-fail-fast` 失败集 = 13(在案,
  scratch/p572/taa_failures_*.txt)。
- 绿判据:改造后 = **1**(charts_gallery);tf 恒 3460/3461 不动。
- 保留锚全部 PASS(浅递归不触栈上限)。

## 验收标准

1. 12 个双重解释器爆栈测试全部消失于默认门禁(最小锚化 PASS 或
   `#[ignore]` 带裁定注记),失败集 13 → 1。
2. 每路径 ≥1 最小正确性锚保留且 PASS。
3. 覆盖对账表留档(关闭面 × 正统替代路径)。
4. tf 零变化;⑤腿/at_mode/gen2 零改动。
5. KNOWN-DEBT 572 条目结算;AGENTS 测试档注记更新。

## 执行步骤
（原子任务;代码改动 in worktree `D:/autostack/.wt/lang-574/auto-lang`）

1. [ ] T1 路径对账:逐一确认 12 测试(及同路径未爆同胞)的执行形态
   与最小锚选件;覆盖对账表落 scratch/p574/coverage-map.md。
2. [ ] T2 最小锚化+关闭:按 T1 对账改造 6 个测试文件
   (aavm2_m1/m2/m3/m4/m5/a2r + aavm_runner_tests),`#[ignore]` 注记
   统一引用"572 待澄清②裁定(2026-09-06)"。
3. [ ] T3 门禁验证:裸 taa 失败集 13→1;tf 3460/3461;保留锚全
   PASS;⑤腿 compile_corpus 58/58 复跑确认零波及。
4. [ ] T4 文档结算:AGENTS AAVM 档注记 + KNOWN-DEBT 572 条目结算
   + 本计划复审留档。

## 复审记录

## 待澄清事项

1. **m4_use/m5_use_errors 等的语料语义**:这些语料(错误路径/多文件
   use)在正统路径的对等物是 compile_use_corpus(⑤腿)——T1 对账
   确认粒度差异是否可接受(最小锚保留诊断粒度即可,重型逐件对等
   不再保留)。
