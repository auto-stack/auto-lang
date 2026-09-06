---
plan_id: PLAN-574
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-dual-interp-shutdown
author: [zhaopuming]
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/aavm/project.md: 修改——验证矩阵(2×2)节新增路径地位裁定注记(VM 内解释两格=双重解释器路径非真实需求,重型验证只走②/⑤腿转译+编译+运行;新计划避免该路径重型化)"
new_spec_components: []
touched_goals:                # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "GOAL-017: 自举——验证矩阵路径地位修订落地(双重解释器列 Windows 关闭/Linux-CI 保留),VM 执行线程栈护栏修复(RUST_MIN_STACK 恢复生效)" 

affects: [aavm]               # 测试基建:crates/auto-lang/src/tests/aavm2_*
current_step: 6
total_steps: 6
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
   〔修订 2026-09-06 T1 实证:最小锚不可行(路径级阈值,与用例规模无关)
   → 裁定原案二选一取"关闭"支;复核口径=12 测试保留编译体+Linux/CI
   全量运行(每路径锚=CI 侧原测试),Windows 按 cfg_attr 关闭;T6 栈
   修复后 Windows 已可跑(001_smoke 2.29s 实证)但按裁定维持关闭〕
3. 覆盖对账表留档(关闭面 × 正统替代路径)。
4. tf 零变化;⑤腿/at_mode/gen2 零改动。
5. KNOWN-DEBT 572 条目结算;AGENTS 测试档注记更新。

## 执行步骤
（原子任务;代码改动 in worktree `D:/autostack/.wt/lang-574/auto-lang`）

1. [✅ 已完成] T1 路径对账:逐一确认 12 测试(及同路径未爆同胞)的执行形态
   与最小锚选件;覆盖对账表落 scratch/p574/coverage-map.md。
   ——关键实证:12/12 均为 `run_with_capture(470KB lib 拼合)` 进程内形态;
   **001_smoke 用例本体单行 print 仍爆栈 → 路径级爆栈,最小锚裁剪
   不可行**(原预案"最小锚化"否决,收敛为 cfg_attr 关闭,覆盖零损失)。
2. [✅ 已完成] T2 最小锚化+关闭:按 T1 对账改造 6 个测试文件
   (aavm2_m1/m2/m3/m4/m5/a2r + aavm_runner_tests),`#[ignore]` 注记
   统一引用"572 待澄清②裁定(2026-09-06)"。
   ——落地:12 测试 `#[cfg_attr(windows, ignore = "...裁定...")]`
   (commit 2e0bc22ba);Windows 关闭/Linux+CI 保留全量。
3. [✅ 已完成] T3 门禁验证:裸 taa 失败集 13→1;tf 3460/3461;保留锚全
   PASS;⑤腿 compile_corpus 58/58 复跑确认零波及。
   ——实测:taa 3612 通过/1 失败(charts_gallery 预存)/607 skipped;
   tf 3460/3461 恒;compile_corpus PASS(17.41s)。
4. [✅ 已完成] T4 文档与规约结算:
   - aavm/project.md 验证矩阵 2×2 节裁定注记(VM 内解释两格降格,
     重型只走②/⑤腿,新计划避免该路径重型化)✓;
   - AGENTS.md AAVM 档裁定注记 ✓;
   - KNOWN-DEBT 572 条目结算(✅已结算)✓;
   - at_mode 文档头 feature 标注修正(test-aavm)✓;
   - 复审留档(本节)。
6. [✅ 已完成] T6 执行线程栈护栏修复(2026-09-06 用户裁定:修):lib.rs 五处
   VM 执行线程硬编码 4MB → `vm_thread_stack_size()`(RUST_MIN_STACK
   可覆盖/缺省 16MB/下限 4MB),恢复 Plan 423 护栏意图。
   ——证据:commit 37fae7959;红→绿:001_smoke 摘 ignore 后 **2.29s
   通过**(修复前 4MB 爆栈);12 测试 ignore 维持(裁定不重开);
   taa 3612/3613(仅 charts 预存)+tf 3460/3461 恒。
5. [✅ 已完成] T5 机制定量调查(用户问询触发,2026-09-06):爆栈精确
   机制探针测定。
   ——证据:commit fccca72bf;探针(临时,用后即删)直接调
   execute_autovm 于可控栈线程:**4MB 爆 / 5MB 过 / 8MB 2.7s 跑通**;
   基点 f2ae1cb29 同阈值(515KB lib);对照组(无 lib)22ms。
   真因 = `run_autovm_capture`(lib.rs:451,`run_with_capture` 底层;
   `run_with_capture_and_path` L357 同款)**硬编码 4MB 执行线程**,
   显式 stack_size 绕过 Plan 423 的 RUST_MIN_STACK=16MB 护栏;外层
   测试线程栈(libtest/nextest)与执行栈无关——解释了双路径同爆与
   "16MB 也不够"假象。递归**有限且浅**,需求随 lib 规模线性增长;
   "路径级必然爆栈"表述已证伪并全面修正(裁定战略结论不变)。

## 复审记录

**复审人**:zhaopuming(auto-plan:review,2026-09-06 18:0x;worktree
`.wt/lang-574/auto-lang`@fa030f3f2,分支基点 2fc544511,4 commits)

**逐条验收复验(现跑)**:

| # | 验收标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | 12 爆栈测试消失于默认门禁,失败集 13→1 | **PASS** | 复审现跑裸 taa:3612/3613,**唯一失败=charts_gallery 预存**,607 skipped;cfg_attr 计数 12/12(m1/m2/m3 各1+m4 三+m5 四+a2r/runner 各1) |
| 2 | 每路径 ≥1 最小正确性锚(修订口径见上注记) | **PASS(修订)** | 12 测试全部保留编译体(diff 仅 +属性行零删除)→ Linux/CI 全量运行=每路径锚;Windows 关闭系裁定原文二选一的"关闭"支,T1 实证驱动,非静默缩面 |
| 3 | 覆盖对账表留档 | **PASS** | scratch/p574/coverage-map.md(52 行,12 测试×正统替代路径逐行+机制定量修正段,branch 内) |
| 4 | tf 零变化;⑤腿/at_mode/gen2 零改动 | **PASS** | tf 现跑 3460/3461 与基点恒;compile_corpus 含于 taa 绿中零波及;diff 不触 ⑤腿/at_mode/gen2 任何文件(仅 lib.rs 栈helper+tests 属性+文档) |
| 5 | KNOWN-DEBT 572 结算+AGENTS 注记 | **PASS** | KNOWN-DEBT ✅已结算(master 66f156013 链);AGENTS AAVM 档裁定注记+资源表 taa 行新态(fa030f3f2);at_mode 文档头修正(in branch) |
| T6 | 栈护栏修复红→绿 | **PASS** | lib.rs 五处硬编码 4MB→vm_thread_stack_size()(RUST_MIN_STACK 可覆盖/缺省 16MB/下限 4MB),残留硬编码 0;红→绿:001_smoke 摘 ignore 2.29s 通过(修复前爆栈);复审门禁双绿 |

**遗漏/延后/workaround 猎查**:无未批准延后;无 workaround(cfg_attr=裁定
落地形态)。发现并已处理:①验收#2 原文按"最小锚"预设书写,T1 实证
转向"关闭"支——已注记(非静默);②AGENTS 资源表 taa 行陈旧——已补
P574 后新态(fa030f3f2)。pre-existing 余项:charts_gallery(564-Q6 域,
非本计划范围)。健康:diff 12 文件 +101/-6,零 debug 残留,注释全带
依据;Rust 改动仅 lib.rs 栈 helper(cargo check 零错)。

**结论:6/6 验收 PASS(其中#2 按裁定修订口径),零未批准延后 →
status: reviewed,可入 /auto-plan:merge。**

## 待澄清事项

1. **m4_use/m5_use_errors 等的语料语义**:这些语料(错误路径/多文件
   use)在正统路径的对等物是 compile_use_corpus(⑤腿)——T1 对账
   确认粒度差异是否可接受(最小锚保留诊断粒度即可,重型逐件对等
   不再保留)。〔T1 已按对账表了结:全量关闭,Linux/CI 保留〕
2. **〔已裁定并执行:修→T6(2026-09-06 用户 OK)〕原 `run_autovm_capture` 族硬编码 4MB 执行线程是否顺手修**(T5 发现,真因即此(lib.rs:451 + L357;RUST_MIN_STACK
   护栏被绕过=Plan 423 意图失效点)。一行改 16/32MB 可恢复护栏意图,
   使**其余**走 run_autovm_capture 的常规 VM 语料测试(vm_file_tests
   等)获得 lib/语料增长余量(它们现未爆但同理可越阈);**不重开**
   已关闭的 12 个双重解释器测试(裁定不变)。裁定:修/不修/修多少。
