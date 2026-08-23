---
plan: 433
title: aavm-a2r-closure（a2r 闭环：AAVM 转译为纯 Rust，四向对比）
affects: [docs/plans/242-a2r-feature-gap-tracker.md]
status: draft
---

# Plan 433: a2r 闭环——AAVM 转译为纯 Rust 并与参考实现四向对比

> **For Claude:** 执行上下文：worktree 名 `plan-433/a2r-closure`。
> 构建/测试：复用 parity 体系（`parity/` workspace，runner + TAP 对齐）与 a2r 编译冒烟
> （`a2r_compile_smoke_*` 模式，rustc --emit=metadata）。
> 前置：Plan 432 M3 已达成（AAVM 全管线可跑）。

## Goal / 目标

用**Rust 版 a2r** 把 AAVM v2 全部 `.at` 转译成纯 Rust（零 `a2r_std` 依赖，Plan 270 机制），
`cargo build` 编译运行，并建立**四向对比**矩阵验证等价性：

| 方 | 说明 |
|---|---|
| ① 参考 Rust | auto-lang 原生实现（oracle） |
| ② AAVM-Rust | AAVM .at 经 a2r 转译编译后的编译器+VM |
| ③ AAVM-VM | AutoVM 解释执行 AAVM .at（432 已建） |
| ④ 预期输出 | corpus 的 golden |

同一 corpus 上 ②③ 行为一致且等价 ①：即证明"同一份 Auto 源码，解释执行与转译执行语义一致，
且都与 Rust 参考实现一致"。**本计划完成态 = Plan 242 #16（自举）/ 415-D 的正式收口。**

## 背景 / 已确认的决策

- AAVM v2 采用纯 Rust 模式（431 定稿），故 a2r 产物天然零 a2r_std 依赖、可独立 cargo 编译
  ——这是"直接对比"的前提，由前序计划保证。
- 预期困难：a2r 对 AAVM 所用 Auto 语法结构的覆盖长尾（429 B3 能力表的 ⚠️/❌ 项）会在本计划
  集中爆发；处理原则与 432 相同：绕过改写记 divergence / 进 242 tracker / blocker 才动主实现。
- 编译冒烟（rustc metadata）必须先行：golden 文本全绿也可能藏着 E0308 级错误（DIV-A2R-STRPARAM-1 教训）。

## 任务（按阶段）

### Phase A：转译冒烟（2-3 天）

- [ ] A1 `auto trans --merge` 对 `auto/lib` v2 全量转译；产物 `rustc --crate-type=lib --emit=metadata`
  编译零错（对齐 427 冒烟模式）。编译错误逐个归因：a2r 缺陷 → 修（进 242 tracker）；
  AAVM 写法问题 → 改写并记 divergence。
- [ ] A2 转译产物代码质量抽查：与 Rust 参考实现并排 diff 抽样（结构对齐度、divergence 是否
  如 431 C2 所记录可解释）。

### Phase B：AAVM-Rust 运行（3-5 天）

- [ ] B1 组装 AAVM-Rust 二进制（转译产物 + 最小 main harness：读 .at 源 → AAVM 管线 → 输出）。
- [ ] B2 在 corpus（431 D1 执行层）上跑 ②，与 ④ golden 对齐；失败例逐个归因
  （转译 bug / 移植 bug / divergence / 语料超范围）。
- [ ] B3 性能记录：② 的编译+运行耗时 vs ①（期望同数量级；显著劣化则定位生成代码模式）。

### Phase C：四向对比矩阵（3-5 天）

- [ ] C1 把 ①②③④ 接入 parity runner（扩展 TAP 对齐：新增 backend `aavm_rust` / `aavm_vm`），
  产出矩阵报告（HTML 仪表盘追加一节或独立页）。
- [ ] C2 长尾收口：矩阵全绿的 corpus 集合冻结为"自举稳定集"；剩余差异逐条归档
  （divergence 可解释 / 已知缺陷进 242 / 超范围剔除）。
- [ ] C3 回填 Plan 242 #16 为 Done；在 415 文档勾选 415-D；KNOWN-DEBT 复核相关条目。

### Phase D：自举回路演示（余力，1-2 天）

- [ ] D1 演示链：`auto/lib` v2 --a2r--> 纯 Rust 编译器+VM（②）--> 该 VM 解释执行 helloworld.at。
  即"Auto 写的 AutoVM 跑 Auto 程序"的完整回路（虽然转译那步用的还是 Rust 版 a2r——
  纯闭环是 434 的任务）。

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| a2r 长尾缺口大面积爆发 | 429 B3 已有预期能力表；严重缺口在 432 期间就应陆续暴露，本计划只处理残余 |
| 转译产物与手写 Rust 的"等价"标准扯皮 | 等价 = 四向矩阵行为一致（黑盒）；代码级只做抽样 diff，不追求逐行 |
| AAVM-VM 路径性能导致 ③ 跑不完 corpus | 431 D2 预算：超限语料 ③ 标 skip、② ① 照跑，矩阵注明 |
| parity 体系扩展成本 | 复用 runner 骨架只加 backend 定义；不为矩阵新造框架 |

## Out of Scope

- Auto 版 a2r（→ 434）
- a2r 生成代码的风格优化（只修正确性缺陷）
- multi-file 项目级转译（`--merge` 单产物够用）

## Verification

1. AAVM 转译产物可独立 `cargo build` 零错、零 a2r_std 依赖；
2. 四向矩阵在"自举稳定集"上全绿，报告可复现（parity CLI 一条命令）；
3. Plan 242 #16 / 415-D 状态正式回填；
4. D1 回路演示命令与输出记录于本文件执行结果节。

## 执行结果

（待执行后回填）
