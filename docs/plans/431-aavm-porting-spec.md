---
plan: 431
title: aavm-porting-spec（AAVM v2 移植规范与核心模块边界）
affects: [docs/specs/aavm/project.md]
status: draft
---

# Plan 431: AAVM v2 移植规范与核心模块边界

> **For Claude:** 执行上下文：worktree 名 `plan-431/porting-spec`。本计划以**调研与文档产出为主**，
> 代码改动仅限测试基建骨架。前置：Plan 429 全部（尤其 B1/B3 报告与 C1 基线 hash 表）。
> 基线锚点：**v0.5 tag**（429-C1 记录），所有引用行号以 tag 快照为准。

## Goal / 目标

产出 AAVM v2 移植的**全套规范**，使计划 432 的移植工作可以按文档机械执行、可多人/多轮并行：

1. **核心/UI 边界清单**：parser.rs（17,576 行）等大文件中哪些行区间/函数属于自举核心；
2. **文件级映射表**：每个 Rust 源文件 ↔ 对应的 `.at` 文件 ↔ 移植优先级；
3. **divergence 规则**：哪些 Rust 惯用法允许（及如何）局部改写；
4. **corpus 选定**：每层移植的一致性判据用什么测试语料；
5. **AAVM v2 目录骨架与测试基建**：新 `auto/`（v2）布局 + VM 文件测试 + a2r 编译对比 runner 骨架；
6. **旧 AAVM 归档策略**。

## 背景 / 已确认的决策

- AAVM v2 = **语句级移植**（非重新设计）：每个 Auto 文件对应一个 Rust 核心文件，数据结构直译，
  函数一一对应；采用**纯 Rust 模式**（`use.rust` 直调 std/三方 crate，**禁用 Auto 自身 stdlib**），
  使 Auto 代码与 Rust 参考实现语句级一对一，a2r 产物为零 `a2r_std` 依赖的纯 Rust。
- 旧 AAVM（`auto/lib/*.at`，13 文件）**归档不删除**：108 个 bootstrap 用例的 `.expected.out` 断言语料、
  a2r 类型映射规则、opcode 编号兼容性是可回收资产；代码本身保留作参考。
- 核心范围已由 429 调研确定（约 5.5-7.5 万行 Rust 当量），本计划把它细化到函数/行区间粒度。
- 不可 100% 语句级：`Peekable<Chars>`、`Rc/Arc`、trait 对象、迭代器链等无 Auto 直接对应，
  允许结构性改写但必须记录。

## 任务（按阶段）

### Phase A：核心/UI 边界清单（2-3 天）

- [ ] A1 parser.rs 功能分区表（基于 429 调研的行号区间，校准到 v0.5 tag）：
  标注每个 `parse_*` 函数为 `core`（语句/表达式/类型/闭包/use/模式）或
  `ui`（widget/store/scene/msg/routes/on-events/tag/grid/cover）或 `task`。
  产出：函数级清单（可 CSV），含行区间、依赖的辅助函数归属传播。
- [ ] A2 同法处理 `ast/`（32 文件：ui/task/route/on/tag/grid/cover 剔除）、
  `vm/codegen.rs`（UI/actor/config-accum 段剔除）、`vm/engine.rs`（debugger/trace/UI console/
  异步 HTTP native 剔除）、`infer/`（task_types 剔除）、`native_catalog.rs`（310 条 →
  核心所需子集，依据 429 B1 的使用清单）。
- [ ] A3 opcode 处置表：194 条逐条标注 `移植`（核心执行路径）/`仅声明`（枚举占位，engine 不实现）/
  `剔除`（UI/并发/actor 专属）。**编号保持与 Rust 一致**（兼容性资产）。
- [ ] A4 边界清单反向价值：标注"Rust 侧若按此边界拆分会更干净"的重构建议清单
  （喂给 432 执行期的债务记录，不提前重构）。

### Phase B：文件级映射表（1-2 天）

- [ ] B1 映射表（Markdown，附于本文件）：

  | Rust 文件（tag 快照行数） | .at 文件 | 优先级 | 备注 |
  |---|---|---|---|
  | token.rs (516) | aavm/token.at | P0 | 全量直译（~140 TokenKind） |
  | lexer.rs (2,029) | aavm/lexer.at | P0 | 极稳定（3 个月 5 commit），近乎照抄结构 |
  | ast.rs + ast/ 核心 | aavm/ast.at | P1 | 按 A2 裁剪后的 Stmt/Expr 子集 |
  | parser.rs 核心区 (~10-11k) | aavm/parser.at | P1 | Pratt 优先级表直译 |
  | types.rs + infer/ 核心 | aavm/typeinfo.at | P2 | 单一 TypeStore，不复刻历史 5 套 registry |
  | vm/opcode.rs (925) | aavm/opcode.at | P2 | 全量声明 |
  | vm/codegen.rs 核心 | aavm/codegen.at | P2 | |
  | vm/engine.rs 核心 | aavm/engine.at | P2 | 栈机 + 调度最小核 |
  | vm/native_catalog.rs 子集 | aavm/natives.at | P2 | X-macro 模式保留 |
  | error.rs / pos | aavm/error.at | P0 | 真实行列追踪（旧版 Pos.line 恒 1 的教训） |

- [ ] B2 每个 P0/P1 文件给出"移植顺序内 topo 依赖"（token→lexer→ast→parser→typeinfo→opcode→codegen→engine）。

### Phase C：divergence 规则（1-2 天）

- [ ] C1 允许的改写模式清单（首版至少覆盖）：
  - `Peekable<Chars>` → `Vec<char>` + 下标游标（lexer）；
  - `Rc/Arc` → 直接值/`Box`（AAVM 单线程域内）；
  - trait 对象 `Box<dyn HeapObject>` → Auto 的 type struct + kind 鉴别器（若 Auto 无对应）；
  - 迭代器链 → 显式 for 循环；
  - Rust 宏（matches!/vec! 等）→ 展开后的等价代码；
  - `?` 传播 → 显式 match（或 Auto `.?`，以 B3 盘点为准）。
- [ ] C2 强制记录格式：每处 divergence 在 `.at` 文件内以 `// DIVERGE(n): 原因` 注释标记，
  并汇总到 `docs/specs/aavm/divergences.md`（含计数与理由），供 433 四向对比时解释差异来源。
- [ ] C3 Auto 侧"纯 Rust 模式"编码规范：`use.rust` 风格、命名（保留 Rust 函数名以便对照）、
  禁用 Auto stdlib 的 lint 方式（如何在测试基建里检测违规引用）。

### Phase D：corpus 选定（1 天）

- [ ] D1 分层 corpus（每层移植的一致性判据语料）：
  - lexer 层：现有 `test/vm/` + `test/a2r/` 全部 `.at` 源文件做 tokenize 输入，对比 Rust lexer 的
    token 流（序列化格式在基建里定义）；
  - parser 层：同上语料 parse 后 AST dump 对比（复用 `.expected.rust_ast` 思路）；
  - 执行层：99_bootstrap 的 038-053 求值类语料（回收断言）+ `test/vm/` 行为用例子集 +
    a2r golden 17/18 组；
  - 终局：`parity/libs/` 中非 UI 库（如 string_utils）。
- [ ] D2 corpus 预算：依据 429 B2 性能报告限定单用例 VM 解释执行的时限，超限语料标记"仅 AAVM-Rust 跑"。

### Phase E：目录骨架与测试基建（2-3 天，代码）

- [ ] E1 AAVM v2 目录：`auto/lib/` 重写为 v2（旧 13 文件移入 `auto/lib-legacy/` 归档 + README），
  `auto/pac.at` 更新；或新顶层目录——**在本计划定稿，二选一**（建议前者，保留 pac 构建回路）。
- [ ] E2 VM 文件测试基建 v2：新用例目录 `test/vm/aavm2/`，runner 复用 99_bootstrap 的
  "lib 前置拼接"法（修掉双清单问题，429-A3 已做）；AUTO_LIB_FILES v2 清单单一事实源。
- [ ] E3 a2r 编译对比 runner 骨架：`auto trans --merge` AAVM → cargo build → 运行 corpus →
  与预期输出 diff（复用 parity runner 的 TAP 对齐机制，仅搭骨架 + 一个冒烟用例）。
- [ ] E4 `.at` 文件头 Snapshot 模板定稿（Rust ref / Baseline=v0.5 tag / Coverage / Missing，
  沿用旧机制、统一基线）。

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| 边界清单做完 Rust 侧又漂移 | 一切锚定 v0.5 tag 快照；worktree 基于 tag 开出 |
| 语句级 1:1 在个别模块不可行引发范围争论 | C1/C2 的 divergence 机制就是减压阀：允许改写但强制记录 |
| corpus 对比格式（token 流/AST dump）在 Rust 侧无现成序列化 | E2/E3 里先做 Rust 侧 dump 工具（debug 打印即可，无需稳定性承诺） |
| 测试基建工作量挤占规范产出 | E3 只做骨架冒烟，完整四向对比是 433 的任务 |

## Out of Scope

- 任何 Rust 侧大重构（只产出 A4 建议清单）
- 实际移植任何编译器模块（→ 432）
- 运行四向对比（→ 433）

## Verification

1. 边界清单（A1-A3）覆盖核心范围全部文件/函数/opcode，可被 432 直接引用执行；
2. 映射表 + divergence 规则 + corpus 清单 + 编码规范四份文档定稿（附录或独立文件）；
3. `auto/lib-legacy/` 归档完成，`test/vm/aavm2/` runner 跑通一个冒烟用例（内容为占位 main）；
4. a2r 对比 runner 骨架能把"Hello World 级最小 .at"转译、编译、运行并 diff 通过。

## 执行结果

（待执行后回填）
