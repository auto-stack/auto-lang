---
plan: 432
title: aavm-core-port（AAVM v2 核心移植：垂直切片到 VM 内跑通示例）
affects: [docs/specs/aavm/project.md]
status: draft
---

# Plan 432: AAVM v2 核心移植——垂直切片，直到在 AutoVM 里跑通示例

> **For Claude:** 执行上下文：worktree 名 `plan-432/core-port-<slice>`（每切片独立 worktree/分支）。
> 构建/测试：`cargo test -p auto-lang --lib -- test_aavm2 --include-ignored`（431-E2 基建）。
> 前置：Plan 431 全部（规范是唯一依据）；Plan 430 Phase E（shim 子集就绪）。
> 基线：v0.5 tag。**每层切片有独立一致性判据闸门，过闸才进下一层。**

## Goal / 目标

按 431 的映射表，用纯 Rust 模式的 Auto 逐层移植核心编译器+VM：

```
token/lexer → ast/parser核心 → typeinfo核心 → opcode声明 → codegen核心 → engine核心
```

**里程碑（对齐并超越 v0.3-v0.4 的成就）**：

- M1：AAVM lexer 与 Rust lexer 在 corpus 上 token 流一致；
- M2：AAVM parser 与 Rust parser 在核心语料上 AST dump 一致；
- M3：**AAVM 在 AutoVM 里编译并运行 helloworld + fib(10)**（等价旧版 run_bytecode 能力）；
- M4：corpus 渐扩（行为用例集），收敛到核心语言覆盖。

## 背景 / 已确认的决策

- 旧 AAVM 的全部 hack 不带入 v2（Map<str,str> 状态机、11 字段上帝节点、`__s__` 前缀、
  逗号串拆参数、`%256` 拆字节、callee 后缀匹配方法调用、静态绝对地址单遍编译——逐一替换为
  直译 Rust 的真实结构：struct AST 节点、Vec/HashMap 状态、位运算、MethodCall 节点、两遍编译回填）。
- 保留的旧资产：opcode 编号与 Rust 对齐、Pratt 优先级表、token/keyword 全集、
  99_bootstrap 可回收语料。
- 遇到 Auto/a2r/VM 的能力缺口（语法不支持、shim 缺方法、类型推不过）：
  **优先绕过改写并记 divergence；确属语言/工具链缺陷的进 242 tracker 或 430 例外表**，
  不为移植顺手大改 Rust 主实现（除非 blocker 级，需在本文件记录决策）。
- 性能预算按 429 B2 / 431 D2 执行，超限语料只跑 AAVM-Rust 路径。

## 任务（按切片）

### S1：token + lexer（P0，预估 3-5 天）

- [ ] 移植 `token.at`（~140 TokenKind 全量）、`error.at`/`pos.at`（真实行列追踪）、`lexer.at`
  （含 f-string、raw/multi 字符串、进制数字、`.view/.mut/.move/.take`、LexerState 快照回滚）。
- [ ] 一致性闸门 M1：corpus（431 D1 lexer 层）token 流 diff Rust 侧 = 0。
- [ ] Snapshot 头 + Coverage/Missing 回填。

### S2：AST + parser 核心（P1，预估 1-2 周，可按语句族拆子切片）

- [ ] `ast.at`：按 431 A2 裁剪后的 Stmt/Expr 子集，per-kind 结构（拒绝上帝节点）；
    `parser.at`：递归下降 + Pratt（优先级表直译）、闭包/f-string/is-match/use（含 use.rust）/
    类型解析/泛型参数。UI/task/store 区不移植（解析到即报"v2 不支持"，报错含位置）。
- [ ] 闸门 M2：核心语料 AST dump diff = 0（Rust 侧 dump 工具用 431 E2 的）。
- [ ] 子切片建议：S2a 声明+表达式 / S2b 控制流+模式 / S2c 类型+use+泛型。

### S3：typeinfo 核心（P2，预估 1 周）

- [ ] `typeinfo.at`：单一 TypeStore（类型/函数/spec 声明表）+ infer 核心传播 + 简化 unification。
  不复刻历史 5 套 registry（TypeStore 一套到底）。
- [ ] 闸门：类型相关现有测试语料（类型错误样例 + 推断样例）结果与 Rust 一致。

### S4：opcode 声明 + codegen 核心（P2，预估 1-2 周）

- [ ] `opcode.at` 全量声明（编号对齐 Rust）；`codegen.at`：两遍编译（先收集符号/地址再发射）、
  脚本 wrapper（FN_PROLOG + 顶层语句）语义对齐 Rust codegen、字符串池、跳转 patch。
- [ ] 闸门：对 corpus 生成的字节码与 Rust codegen 产物**结构级**一致（序列化对比允许元数据差异，
  指令流一致；差异逐条解释或修）。

### S5：engine 核心（P2，预估 1-2 周）

- [ ] `engine.at`：栈机 + 任务最小核（单任务先行，调度只留接口）、堆对象（type struct + kind）、
  native 子集（print/断言/String/Vec/HashMap 所需，走 430 生成的 shim 包）、函数调用/RET 帧、
  JMP 家族。debugger/trace/并发/async 不移植（遇指令报 v2 不支持）。
- [ ] **闸门 M3：AAVM 全管线在 AutoVM 内编译并运行 helloworld + fib(10)，
  输出与 Rust 参考实现一致**。这是本计划的主里程碑，达成即恢复旧版成就水平。
- [ ] M4 扩 corpus：99_bootstrap 038-053 回收语料 + test/vm 行为子集，逐例迁移进 `test/vm/aavm2/`，
  失败例逐个归因（移植 bug / 已知 divergence / 语料超 v2 范围）。

### 收尾

- [ ] 各 .at 文件 Snapshot/Coverage/Missing 回填；divergences.md 汇总定稿；
  旧 `auto/lib-legacy` 正式封存（README 指向 v2）。
- [ ] Rust 侧重构建议（431 A4 清单 + 执行期新增）整理进债务簿。

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| 移植中发现 Rust 侧结构耦合无法直译（parser 内嵌 infer 等） | 允许在 .at 侧做"解析与推断分离"的更好结构（这本来就是重构建议的验证），记 divergence 即可 |
| Auto 语言表达力缺口（闭包捕获/泛型/trait）成为 blocker | 逐项决策：绕过 / 242 tracker 排期 / 极少数 blocker 允许动 Rust 主实现（须记录） |
| AAVM-in-VM 性能不足 | corpus 预算（431 D2）；--release；超限语料留给 433 的 AAVM-Rust 路径 |
| 工期失控 | 每切片独立 worktree + 独立闸门，M3 达成即可宣布阶段性成功对外交付 |
| 语句级 1:1 被无意滑向"顺手重新设计" | Code review 检查点：任何偏离映射表的结构需在本计划文件登记理由 |

## Out of Scope

- UI/task/store/widget/routes/scene/msg 方言（遇到即明确报错）
- async/并发/actor 状态/debugger/trace
- a2r 转译 AAVM（→ 433）；Auto 版 a2r（→ 434）
- 多文件/模块链接（单文件优先，多文件若 corpus 需要再评估）

## Verification

1. M1/M2/S3/S4 各闸门 diff=0（或有逐条解释的差异清单）；
2. M3 演示：一条命令在 AutoVM 内经 AAVM（auto/lib v2）编译运行 helloworld 与 fib，输出正确；
3. `test/vm/aavm2/` 用例数与通过率报表；divergences.md 与各 Snapshot 头回填完整；
4. 全程未对 Rust 主实现做未登记的顺手修改。

## 执行结果

（待执行后回填）
