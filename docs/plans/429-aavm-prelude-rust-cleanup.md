---
plan: 429
title: aavm-prelude-rust-cleanup（AAVM 前奏：Rust 参考实现清理与风险收敛）
affects: []   # 本计划不改 specs 语义，仅代码整理与调研产出
status: draft
---

# Plan 429: AAVM 前奏——Rust 参考实现清理与风险收敛

> **For Claude:** 执行上下文：worktree 名 `plan-429/rust-cleanup`（在 master 取号后创建）。
> 构建/测试：`cargo test -p auto-lang --lib`（全量）+ `cargo test -p auto-lang --lib -- a2r_tests`。
> 回归要求：改动前后全量测试结果一致（当前基线 3097+6 全绿）。
> **本计划是 AAVM 自举系列（429→434）的第一步，只做安全清理与风险盘点，不做大重构。**

## Goal / 目标

为 AAVM v2 自举（见附录 A 系列总纲）做前置准备：

1. 修复 Rust 参考实现中三处"安全小病灶"（有明确证据、改动面极小）；
2. 把三类未知风险变成**已知清单**（shim 缺口、性能倍数、a2r 语法面），并消化高频项；
3. 冻结基线：以 v0.5 tag 作为后续所有移植计划的对照锚点。

**目标不是"消灭所有风险"**——shim/a2r 的长尾只能在移植过程中暴露（计划 432），本计划只负责把它们从"未知"变成"已知清单 + 高频项已消化"。

## 背景 / 已确认的决策

（来自 2026-08-23 AAVM 自举调研，四个并行调研 agent 的结论 + 与用户的三轮方案讨论）

- 旧 AAVM（`auto/lib/*.at`，13 文件 ~6,200 行，基线 fe666a6f/ddbb161a/7cc484b1，2026-05）已部分失效：
  `test/vm/99_bootstrap/` 残留 23 个 `.wrong.out`，060-080 字节码/BVM 组全挂。**不做修复**，由计划 432 起整体重写替代。
- 自基线以来 Rust 侧 2,449 commit：a2r 近整体重写、opcode ~100→194、字符串池 u32 化（8-22）、
  native ID 撞号修复（8 月中）。**增量同步旧 AAVM 无意义，决策为基于"纯 Rust 模式"全新重写。**
- 大重构（parser.rs 17.5k 拆分、多套 registry 统一）**刻意不在本计划**：移植本身是这些重构
  最好的反馈源（"AAVM 复刻不干净的地方 = Rust 侧需要重构的地方"），先拆后写容易优化错方向。
  记录为系列债务，由计划 431/432 的产出反向驱动。

## 任务（按阶段）

### Phase A：安全小修（每项独立 commit，可单独回滚）

- [ ] A1 **删除孤儿文件 `crates/auto-lang/src/parser.rs_helper.rs`**（28 行）
  来自 2026-01-28 标题为 "temp" 的 commit 194c41ab；先全仓 grep（`mod parser.rs_helper` /
  `include!` / `mod parser_rs_helper`）确认真无引用再删。
- [ ] A2 **合并 `symbol.rs`（34 行）与 `symbols.rs`（28 行）的重复定义**
  两份几乎相同的 `SymbolLocation` + `CodePak`（都注明 "extracted from universe.rs, Plan 091"）。
  保留一份（建议 symbols.rs），另一份 `pub use` 转发一个周期后删除；grep 全部引用点改路径。
- [ ] A3 **统一 `AUTO_LIB_FILES` 两份清单**
  `src/tests/vm_file_tests.rs`（12 个文件，含 generics.at）vs `src/lib.rs` 的 `run_vm_file_test` 路径
  （11 个，漏 generics.at）。统一为一份（抽到公共 const 或函数），lib.rs 侧引用之。
  注意：99_bootstrap 的 060-080 组当前本来就挂，此修复**不要求**把它们跑绿，只需保证清单一致、
  其余用例不回归。
- [ ] A4 全量测试确认无回归（`cargo test -p auto-lang --lib` 与改动前基线一致）。

### Phase B：风险盘点（产出三份报告，放 `docs/plans/reports/429-*.md`）

- [ ] B1 **shim 需求盘点工具**（rust 脚本或一次性 bin，后续被计划 430 复用为元信息工具输入）
  - 扫描对象（核心自举范围）：`lexer.rs`、`token.rs`、`parser.rs` 剔 UI 区（以计划 431 边界为准，
    本阶段先用函数名粗粒度过滤 widget/store/task/scene/routes/msg/on-events）、`types.rs`、`infer/`、
    `vm/opcode.rs`、`vm/codegen.rs`、`vm/engine.rs`、`vm/native_catalog.rs`。
  - 提取这些文件用到的 `std::` 类型与方法调用（`Vec<HashMap<..>>`、`.chars()`、`.entry()` 等），
    输出去重后的 (Type, method) 清单。
  - 对照 `vm/ffi/stdlib.rs` dispatch 3000 的 ~111 个 match 臂 + `ffi.rs known_signature` +
    `BUILTIN_OPAQUE_CRATES` 白名单，输出缺口报告：`已覆盖 / 缺失 / 需沙箱`。
  - **高频缺口当场补 shim 臂**（预计集中在 String/Vec/HashMap 的十几个方法），
    长尾留给计划 430 的元信息工具。
- [ ] B2 **性能摸底**
  - 写一个 `use.rust` 密集的基准程序（String 拼接/查找 + Vec push/迭代 + HashMap 插入/查找循环，
  规模 ~10^4-10^5 操作），分别：① AutoVM 解释执行（`--release`）② a2r 转译后 cargo 编译执行。
  - 量化倍数写入报告；据此给出 AAVM corpus 规模预算建议（如"单用例 VM 解释执行 ≤ N 秒"）。
  - 不做优化，只摸底。
- [ ] B3 **a2r 语法面盘点**
  - 盘点核心自举范围 Rust 代码用到的语言构造（enum/模式匹配/闭包捕获/泛型/trait/迭代器链/
  Rc/Arc/宏使用），对照现有 a2r golden（186 例）与 parity 覆盖矩阵，产出"预期能力表"：
  `✅ 已覆盖（golden 编号）/ ⚠️ 部分 / ❌ 缺失`。
  - 缺失项逐条给出对策预案（绕过写法 或 进 242 tracker 排期），**不在本计划修 a2r**。

### Phase C：基线冻结

- [ ] C1 确认/等待 v0.5 tag 落地；记录 tag 下核心文件清单及各自 commit hash
  （作为计划 432 起新 AAVM 各 `.at` 文件头 Snapshot 的统一基线）。
- [ ] C2 报告汇总：三份 B 阶段报告 + 基线 hash 表，收进本文件"执行结果"节，
  并在 `docs/guides/aavm-sync-guide.md` 头部加"已被 429-434 系列取代"的指向注记（内容暂不重写）。

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| A2/A3 触碰引用面广 | 均为机械替换，逐项独立 commit，任何一项可单独回滚 |
| B1 的"UI 区粗过滤"不准导致盘点遗漏 | 过滤口径记录在报告中，计划 431 定稿边界后可增量重跑（工具化，成本低） |
| v0.5 tag 尚未打 | C1 可等待；A/B 两阶段不依赖 tag，先行 |
| 旧 99_bootstrap 测试现状混乱干扰判断 | 明确口径：本计划不修旧 AAVM，060-080 挂着是已知事实 |

## Out of Scope

- parser.rs 大拆分、类型 registry 统一、engine 巨型 match 拆分（→ 系列债务，由 431/432 反向驱动）
- 修复旧 AAVM（auto/lib/*.at）的任何代码
- shim 元信息工具本体（→ Plan 430）
- a2r 任何代码改动（B3 只盘点不修）

## Verification

1. `cargo test -p auto-lang --lib` 与计划前基线一致（3097+6 级别全绿，无新增 fail）；
2. 三份报告存在于 `docs/plans/reports/`，其中 B1 报告的"高频缺口"项已在 dispatch 3000 落地并有对应
   ffi_dual/手动用例验证；
3. B2 给出量化倍数与 corpus 预算建议；
4. B3 能力表覆盖核心范围全部构造，缺失项均有对策标注；
5. C1 基线 hash 表就位（或明确记录 tag 待打的阻塞状态）。

---

## 附录 A：AAVM 自举系列总纲（429→434）

| # | 计划 | 一句话 | 出口判据 |
|---|---|---|---|
| 429 | Rust 清理与风险收敛（本计划） | 小修 + 盘点 + 冻结基线 | 三报告 + 基线 hash 表 |
| 430 | shim 元信息工具 | rustdoc→元信息→生成 shim，手动白名单渐进归零 | String/Vec/HashMap 子集全通 + 40 crate 迁移开始 |
| 431 | 移植规范与核心边界 | 定 core/UI 边界、文件映射表、divergence 规则、corpus | 规范文档定稿，AAVM v2 目录骨架就绪 |
| 432 | AAVM v2 核心移植 | 垂直切片逐层移植（纯 Rust 模式） | AAVM 在 AutoVM 里编译运行 helloworld/fib，corpus 渐扩 |
| 433 | a2r 闭环 | Rust 版 a2r 转译 AAVM 为纯 Rust，四向对比 | AAVM-Rust 与参考实现 corpus 行为一致（收口 242 #16/415-D） |
| 434 | AA2R（余力） | Auto 版 a2r，共享 AAVM 前端 | AAVM 自身 a2r 转译 AAVM → 纯 Rust AutoVM → 跑 .at |

排序依据：风险先行（429/430 消化可先行收敛的风险），但 shim 长尾与 a2r 语法长尾**只在 432 移植中暴露**，
故 432 的垂直切片闸门（每层一致性判据）是质量主控。434 为余力项，独立排期。
