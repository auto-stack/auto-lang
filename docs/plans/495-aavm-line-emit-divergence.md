---
plan_id: PLAN-495
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-line-emit-divergence
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/aavm/design/m4-bytecode-format.md: §发射模式考古 `.line` 条文修订——原「按语句首 token 行号,由语句步行者发射」扩为「+同线去重(cur_line 状态机)+is 单表达式/单跳转 arm 体按臂体首 token 行发射(块体 arm 归语句步行)」"
new_spec_components: []
touched_goals:                # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "GOAL-017: 自举六道闸门——M4 闸门 cargo tv aavm2 全系绿(P485-2 清偿)+b14 回归钉"

affects: [auto-lang/vm, aavm]
current_step: 5
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
   [✅ 已完成] 红复现（worktree）+ 证据表回写「执行证据」节；实测方向与分诊相反：rust 发 `.line 10/11`、aavm 缺（left=stdout=aavm 断言方向澄清）；根因两级=arm 体行号发射缺失 + 同线去重状态机缺失。
2. **规范定案**：按证据表 + corpus 事实基准定案（预期=同线去重），一行
   裁定记入本文件。
   验证：裁定行成文。
   [✅ 已完成] 裁定回写「T2 规范定案」节：以 rust 为规范=语句边界发射+arm 体行发射+相邻同线去重；待澄清①不触发。
3. **aavm 侧修复**：AUTO_LIB_FILES_V2 对应 .at 源码的 emit 段补同线去重
   状态（文件定位以 `grep -rn "line" crates/aavm`/语料清单为准）。
   验证：`cargo tv -p auto-lang aavm2_m4`（b13 转绿）。
   [✅ 已完成] 修复落 `auto/lib/codegen.at` 四处：CG 增 `cur_line` 字段+构造初始化、新增 `cg_line` helper（镜像宿主 emit_source_line：`ln>0 && ln!=cur_line` 才发）、`cg_stmts` 改走 cg_line（同线去重）、`cg_is_arm_body` 非块体路径补 arm 体首 token 行发射（镜像 parse_expr_or_body 的 stmt_line）；`cargo tv -p auto-lang aavm2_m4` 2 测全绿（b13 corpus 转绿）。
4. **回归钉**：新增最小 fixture（同线双语句 → `.line` 单发断言，挂 aavm2
   对拍套件同族）。
   验证：`cargo tv -p auto-lang aavm2_m4`（新钉绿）。
   [✅ 已完成] 新增 `crates/auto-lang/test/vm/aavm2/corpus_m4/b14_line_dedup.at`（同线双 let + is 单表达式 arm，对拍逐行相等即断言）；实测形态：`.line 10` 同线双语句单发（去重锁）、`.line 13/14` arm 体行发射锁；corpus 2 测全绿。
5. **全档+清偿回写**：`cargo tv` 全量绿；KNOWN-DEBT P485-2 标已清偿；
   状态翻 execution_done。
   验证：`cargo tv && cargo check -p auto-lang`。
   [✅ 已完成] `cargo tv` 全档（no-fail-fast）3443 测 3441 绿——仅余 2 红
   均为 master 既有 cookbook 红（cb_asynchronous_channel/cb_devtools_log_error，
   master 单跑双证同红、与 .line 改动无关，已登记 KNOWN-DEBT P495-1）；
   aavm2 全系（m1-m5）绿；`cargo check -p auto-lang` 过（160 警告=既有
   基线，本计划零 Rust 改动零新增）；`cargo t` 默认档 3302/3302 绿；
   KNOWN-DEBT P485-2 标已清偿（含分诊左右颠倒修正注记）。

## 复审记录

（2026-08-31，/auto-plan:review，worktree `plan-495-dev` @`606639ee3` 现场重验）

**验收标准逐条裁定：**

1. `cargo tv` 全绿（含全量 aavm2 corpus）——**pass（计划范围）**。scoped
   重验 `cargo tv -p auto-lang aavm2_m4` 2/2 绿；全档 no-fail-fast
   3443 测 3441 绿，aavm2 全系（m1-m5 corpus）绿；仅余 2 红
   （cb_asynchronous_channel/cb_devtools_log_error）为 **master 既有红**
   （master 默认 checkout 单跑双证同红、失败形态一致、本计划零 Rust 源码
   改动），已登记 KNOWN-DEBT P495-1——非本计划引入 regression，不 block。
2. 证据表+定案成文；回归钉入仓——**pass**。§执行证据（双端并排表+断言
   方向澄清+两级根因）在册；b14_line_dedup.at 在 worktree 分支，实测形态
   双锁（`.line 10` 同线单发 / `.line 13/14` arm 行发射）。
3. KNOWN-DEBT P485-2 标已清偿——**pass**（master `279bc5bc1`，含分诊
   左右颠倒修正注记）。
4. `cargo check -p auto-lang` 零警告；`cargo t` 不回归——**pass**。
   review 全量门禁 **`cargo tf` 3303/3303 全绿**（含 1M churn 档）；
   check 160 警告=master 既有基线（worktree crates/ 与 master 逐字节同，
   零新增）。

**遗漏/延后/workaround 扫描：** 无遗漏（5 步产物齐备）、无未批准延后、
无 workaround（修复为宿主语义逐句镜像非绕道）。复审新发现登记：

- **P495-2（债务候选，已登记 KNOWN-DEBT）**：is **块体** arm 作用域语义
  潜在分叉——rust arm 体统一走 `Stmt::Block`（codegen.rs:3790，
  push/pop_scope，depth>2 时 arm 块尾发槽释放组），aavm `cg_is_arm_body`
  块体路径走 `cg_body_inline`（codegen.at，不推作用域，arm 内 var 归
  外层域）——arm 块体内声明 var 时释放组位置/时机分叉；corpus 现无块体
  arm 语料故未暴露。
- 附注：表达式级 `.line` 发射位（`Expr::Block`/表达式 is arm，
  codegen.rs:9905/9934）aavm cg_expr 尚无对应实现——属 M4 fn-only 语料
  能力边界，未来实现时需同款 `cg_line`（随 P495-2 一并留意）。

**spec-impact 元数据已填**（supersedes：m4-bytecode-format.md `.line`
条文修订；touched：GOAL-017）。

**裁定：通过，`status: reviewed`。**

## 执行证据

### T1 证据表：双端 `.line` 发射序列并排（b13_is_enum.at，worktree 现场提取 2026-08-31）

> **断言方向澄清**：对拍 `assert_eq!(stdout, expected)`（aavm2_m4.rs:179）
> 中 **left=stdout=aavm**、**right=expected=rust**——分诊原文的"rust 去重不
> 重发 / aavm 逐语句发射"左右标注颠倒，实测方向相反：**rust 发射
> `.line 10/11`，aavm 缺失**。

b13 源（arm 体在行 10/11）：

```
 8    let v = Val.VI(42)
 9    is v {
10        Val.VI(x) -> print(x)
11        else -> print(0)
12    }
```

双端字节码并排（is arm 区段）：

| 偏移 | rust 侧（expected，实测有 .line） | aavm 侧（stdout，实测缺） | 差异 |
|---|---|---|---|
| 0040 | `store.local 2` | `store.local 2` | 同 |
| — | **`.line 10`** | （无） | **rust 发 / aavm 缺 ← 分叉①** |
| 0042/0045 | `load.loc.2 2` … | `load.loc.2 2` … | 同（偏移差 3B=.line 尺寸） |
| — | **`.line 11`** | （无） | **rust 发 / aavm 缺 ← 分叉②** |
| … | `const.i32 0; call.nat` | `const.i32 0; call.nat` | 同 |

根因两级（均 aavm 侧滞后于 rust 事实基准）：

1. **is 单表达式 arm 体行号**：rust `parser.rs:7650 parse_expr_or_body`
   给单表达式 arm 记 `stmt_line`（arm 体首 token 行）入
   `body.source_lines`，codegen 经 `Stmt::Block`（codegen.rs:3790）→
   Block 语句循环 `emit_source_line`（codegen.rs:1119-1122）发射；
   aavm `cg_is_arm_body`（codegen.at:1039）对非块体直接 `cg_expr`，
   **不发 `.line`**——b13 红的直接原因。
2. **同线去重状态机**：rust `emit_source_line`（codegen.rs:10055）带
   `current_source_line` 状态（`line > 0 && line != current` 才发）；
   aavm `cg_stmts`（codegen.at:1383）逐语句**无条件**发射——当前
   corpus 无同线双语句语料故未红，属潜在分叉（与分诊"去重"线索对
   应，但方向是 aavm 需**补**去重，而非 rust 去掉）。

### T2 规范定案（裁定）

**裁定：以 rust 侧为规范。** `.line` 发射语义 = ①语句边界发射（fn 体/
内联体每语句，首 token 行）；②is 单表达式/单跳转 arm 体按 arm 体首
token 行发射（块体 arm 归①）；③相邻同线去重状态机
（`line > 0 && line != current_source_line`）。依据：corpus 其余文件在
rust 语义下全绿 = 事实基准；分诊预期方向（rust 为准）确认，待澄清①
（反向定案）不触发。

## 待澄清事项

- **① 反向定案的影响**：若裁定为"逐语句发射"（改 rust 侧去重），需全
  corpus 复跑评估波及面（其余文件的既有绿会翻红则成本高——预期不采纳，
  仅登记）。
- aavm 侧修复文件的确切路径以 T1 时 `AUTO_LIB_FILES_V2` 语料清单定位为准
  （自举 .at 源在 aavm crate 实验区）。
