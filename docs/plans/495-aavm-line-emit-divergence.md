---
plan_id: PLAN-495
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-line-emit-divergence
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, aavm]
current_step: 0
total_steps: 5
---

# [PLAN-495] P485-2 专项——aavm/rust 双后端 `.line` 发射对齐

## 变更摘要

清偿 P485-2（master `cargo tv` 唯一先在红，488 T11 骑手分诊已精确归属）：
`tests::aavm2_m4::test_aavm2_m4_codegen_corpus`（b13_is_enum.at）是 **rust
编译器 vs aavm .at 自举编译器的实时对拍**（无静态期望文件），分诊定案为
双后端 `.line` 发射**真分叉**——rust `Codegen::emit_source_line` 的同线去重
（`current_source_line`，`crates/auto-lang/src/vm/codegen.rs:10041`）使
`.line 10/11` 同线不重发；aavm 侧（`AUTO_LIB_FILES_V2` 的 .at 编译器实现）
逐语句发射、无同线去重。本计划定案规范语义并对齐滞后侧，`cargo tv` 全绿。

## 目标

- **G1 规范定案**：`.line` 同线去重的规范语义成文（预期：以 rust 侧去重为
  准——字节码更精简，且 corpus 其余文件在 rust 语义下绿=事实基准；反向则
  走待澄清①影响评估）。
- **G2 对齐修复**：aavm .at 编译器的 `emit_source_line` 等价物补同线去重
  状态机（.at 源码层修复，非 rust 侧）。
- **G3 门禁收口**：`cargo tv` 全绿 + 回归钉；KNOWN-DEBT P485-2 标已清偿。
- **非目标**：`.line` 语义本身的设计变更（仅对齐既有事实基准）；其余
  aavm2 对拍红（当前仅此一例）。

## 架构方案

```
对拍测试（tests/aavm2_m4.rs::test_aavm2_m4_codegen_corpus, b13_is_enum.at）
   rust Codegen::emit_source_line（codegen.rs:10041, current_source_line 去重）
   vs aavm .at 编译器 emit（AUTO_LIB_FILES_V2 内 .at 源码, 逐语句无去重）
   → 对齐 aavm 侧补去重（G2），corpus 自然绿
```

## 技术栈

纯 .at 源码修复（aavm 自举库）+ 既有测试设施。零新依赖。

## 需求分析与背景调查

（现场核验 2026-08-31，依据 KNOWN-DEBT P485-2 条目——488 T11 分诊原文）

- 红：master @`3a4aacf19` 即红，嫌疑 051-C7/484 合入线；488 分支 codegen
  触碰仅两行 intrinsic 表注册（dnd_start，与 .line 无关，实证先在）。
- 测试形态：实时对拍（rust 编译器与 aavm 各自编译同一 .at 语料比对字节码），
  **无静态期望文件**——"更新期望"路径不存在，必须真对齐。
- 消费者风险评估：`.line` 为调试信息行号指令，aavm 为自举实验线
  （overview：experimental、无生产消费者）——对齐属低风险语义归一。
- 排程：队列空、无冲突；VM 线小专项，任何空闲会话可领。

## 详细设计

1. **T1 证据表**：提取双后端对 b13_is_enum.at 的 `.line` 发射序列并排成文
   （差异行逐条标注），回写本计划——规范定案的依据物。
2. **T2 定案**：依据证据表 + corpus 其余文件事实基准，定"同线去重"为规范
   （若反向，按待澄清①评估后改 rust 侧并全 corpus 复跑）。
3. **T3 修复**：aavm lib 的 emit 段（AUTO_LIB_FILES_V2 内 .at 源码）补
   `current_source_line` 同款去重状态（编译器状态变量 + 发射前判定）。
4. **T4 钉与门禁**：corpus 绿；新增回归钉（同线双语句语料的最小 .at fixture，
   锁"同线只发一次"语义，防再次分叉）；`cargo tv` 全量绿。
5. **T5 清偿回写**：KNOWN-DEBT P485-2 标已清偿 + 本计划证据链接。

## 测试设计

- 既有 corpus 对拍即主测试（b13_is_enum.at）；新增最小回归钉 fixture
  （同线双语句 → `.line` 单发断言）。
- `cargo tv` 全档复跑（唯一受影响门禁档）。

## 验收标准

1. `cargo tv` 全绿（含全量 aavm2 corpus）。
2. 证据表 + 规范定案成文本计划内；回归钉入仓。
3. KNOWN-DEBT P485-2 标已清偿。
4. `cargo check -p auto-lang` 零警告；`cargo t`（默认档）不回归。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **复现+证据表**：跑 `cargo tv -p auto-lang aavm2_m4`，提取双端 `.line`
   序列并排差异表回写本文件。
   验证：差异表成文（差异行=去重缺失处）。
2. **规范定案**：按证据表 + corpus 事实基准定案（预期=同线去重），一行
   裁定记入本文件。
   验证：裁定行成文。
3. **aavm 侧修复**：AUTO_LIB_FILES_V2 对应 .at 源码的 emit 段补同线去重
   状态（文件定位以 `grep -rn "line" crates/aavm`/语料清单为准）。
   验证：`cargo tv -p auto-lang aavm2_m4`（b13 转绿）。
4. **回归钉**：新增最小 fixture（同线双语句 → `.line` 单发断言，挂 aavm2
   对拍套件同族）。
   验证：`cargo tv -p auto-lang aavm2_m4`（新钉绿）。
5. **全档+清偿回写**：`cargo tv` 全量绿；KNOWN-DEBT P485-2 标已清偿；
   状态翻 execution_done。
   验证：`cargo tv && cargo check -p auto-lang`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **① 反向定案的影响**：若裁定为"逐语句发射"（改 rust 侧去重），需全
  corpus 复跑评估波及面（其余文件的既有绿会翻红则成本高——预期不采纳，
  仅登记）。
- aavm 侧修复文件的确切路径以 T1 时 `AUTO_LIB_FILES_V2` 语料清单定位为准
  （自举 .at 源在 aavm crate 实验区）。
