---
plan: 434
title: aavm-auto-a2r（AA2R：Auto 版 a2r 转译器，终极自举闭环）
affects: [docs/specs/aavm/project.md]
status: draft
---

# Plan 434: AA2R——Auto 版 a2r 转译器

> **For Claude:** 执行上下文：worktree 名 `plan-434/auto-a2r`（按切片可拆 `plan-434/auto-a2r-<slice>`）。
> 构建/测试：`test/vm/aavm2/` 新增 aa2r 组用例 + 433 的四向矩阵扩展为五方。
> 前置：Plan 432 M3；建议 433 至少 Phase A/B 完成（先证明 Rust 版 a2r 能吃下 AAVM 全量源，
  再谈用 Auto 重写它）。**本计划为余力项，独立排期，不阻塞系列收官。**

## Goal / 目标

把 a2r 的核心子集移植为 Auto 版（`auto/lib/a2r.at` v2），与 AAVM 共享前端
（token/lexer/ast/parser/typeinfo 均为 432 成果），实现：

- **G1**：AA2R 能转译 AAVM 自身以外的普通 Auto 程序（corpus 级）；
- **G2（终极闭环）**：AA2R 转译 AAVM 自身 → 纯 Rust 的 AutoVM → 该 VM 能运行 .at 程序。
  至此自举回路中不再有任何 Rust 手写的编译组件：**Auto 写的 a2r 转译 Auto 写的 AutoVM，
  产物是可独立编译的 Rust**。

## 背景 / 已确认的决策

- 旧 AAVM 已有 778 行的 a2r.at v1（覆盖表达式/声明级 60-70%，Phase E1-E6 的映射规则可回收：
  类型映射表、`format!` 形状、构造器展开、borrow 语义等）。v2 是**对 Rust 版 trans/rust.rs 核心子集
  的移植**（与 432 同一方法论），不是从 v1 增量演化。
- 移植范围（裁剪自 trans/rust.rs 20,831 行）：
  核心表达式/语句/类型发射 + use.rust 直通 + Cargo.toml 依赖推导（`dep` + 内建豁免清单）；
  **不含**：多目标（c/python/gdscript/js）、r2a、escape/ 逃逸分析的完整移植
  （保底用 Rc/clone 粗粒度策略，对齐主 a2r 的现状）、post_process 正则家族的完整移植
  （只移植 AA2R 自身产物需要的子集）。
- 五方对比 = 433 四方 + ⑤ AA2R 转译产物（行为上应与 ② 不可区分）。

## 任务（按切片）

### S1：发射核心（预估 1 周）

- [ ] `a2r.at` v2 骨架 + Sink（输出缓冲）+ 类型映射（复用 v1 规则表并按主 a2r 校准）。
- [ ] 表达式/语句/声明级发射（let/var/fn/if/for/while/match-is/闭包/f-string/struct/enum/
  impl/spec/use 全家），对照主 a2r 的 golden `01_basics`…`16_interop` 语料逐组移植。
- [ ] 闸门：AA2R 对非 UI golden 语料的转译输出与主 a2r 输出**文本级一致或差异可解释**。

### S2：use.rust 直通与 Cargo.toml（预估 3-5 天）

- [ ] `use.rust` 发射（`::` 连接、`::{}` 展开、companion trait 导入表子集）+
  `dep` 结构化 spec → 依赖表渲染 + 内建 crate 豁免（Plan 190 清单）。
- [ ] `a2r_std_used` 追踪（Plan 270 机制）：纯 Rust 模式零依赖输出。
- [ ] 闸门：golden `17_rust_std`/`18_pure_rust` 语料通过。

### S3：AA2R 自举（预估 1 周，本计划核心）

- [ ] 用 AA2R 转译 AAVM 全量（`auto/lib` v2）→ 产物独立 cargo build → 运行 corpus。
- [ ] 失败归因三分类：AA2R 移植 bug / 432 已记录 divergence 未覆盖转译侧 / 主 a2r 本身缺陷（进 242）。
- [ ] **闸门 G2 演示**：AA2R --(转译)--> AAVM-Rust'（≠ 433 的 ②，这次转译器是 Auto 的）-->
  编译 --> 该 VM 运行 helloworld.at 与 fib.at 成功。
- [ ] 五方矩阵接入（⑤=AA2R 产物 backend），稳定集上全绿。

### S4：收尾

- [ ] a2r.at v2 Snapshot/Coverage 回填；divergences.md 增补转译侧条目。
- [ ] 总纲收官：429-434 系列复盘文档（成就/遗留/下一定位），旧 a2r.at v1 随 lib-legacy 封存。

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| AA2R 需要的 Auto 特性比 AAVM 更多（发射器是字符串密集+递归深） | 与 432 同原则：绕过/tracker/blocker 决策记录；golden 语料先行暴露 |
| 双重偏差（移植的编译器 × 移植的转译器）难以归因 | 五方矩阵天然分轴：⑤ 对比 ② 隔离转译器差异，② 对比 ① 隔离编译器差异 |
| post_process 缺失导致产物不可编译 | S1 闸门就要求 rustc metadata 冒烟，不许欠账进 S3 |
| 范围蔓延成"全功能 a2r" | Out of Scope 纪律：多目标/逃逸分析完整版明确不做 |

## Out of Scope

- a2c/a2python/a2js 等其它目标
- r2a 反向转译器
- escape/ 逃逸分析完整移植、post_process 完整家族
- IDE/source map 级质量（只要能编译、行为一致）

## Verification

1. S1/S2 闸门：指定 golden 语料组通过（文本一致或差异清单可解释）+ rustc 冒烟零错；
2. G2 终极闭环演示可复现（命令序列 + 输出记录）；
3. 五方矩阵稳定集全绿报告；
4. 系列复盘文档定稿。

## 执行结果

（待执行后回填）
