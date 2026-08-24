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

- [x] 移植 `token.at`（~140 TokenKind 全量）、`error.at`/`pos.at`（真实行列追踪）、`lexer.at`
  （含 f-string、raw/multi 字符串、进制数字、`.view/.mut/.move/.take`、LexerState 快照回滚）。
  （2026-08-24 核心子集落地:token.at 全量 139 变体+58 keyword+kind_name(脚本自基线生成);
  lexer.at 主循环核心子集(见 Missing);error.at/pos.at 以 Pos 记账语义内嵌 lex_dump,
  独立文件待 S2 需要错误类型时拆出)
- [x] 一致性闸门 M1：corpus（431 D1 lexer 层）token 流 diff Rust 侧 = 0。
  （corpus_m1 四文件(c01 算术/c02 字符串注释/c03 数字/c04 控制流)全绿——
  kind|text|line|at|len 五字段逐 token 一致;test_aavm2_m1_lexer_corpus）
- [x] Snapshot 头 + Coverage/Missing 回填。（两文件头五字段模板齐;divergences.md 10 处登记）
  **S1 残余(进 M1 扩闸)**:f-string、raw/multi/c 字符串、#comptime、byte 字面量、
  .view/.mut/.take/.move/.? 属性词、}+换行 else 抑制、LexerState 回滚——corpus 扩到
  含这些构造成时逐个收编(现为 Missing,遇之报 Unknown,闸门即时暴露)。

### S2：AST + parser 核心（P1，预估 1-2 周，可按语句族拆子切片）

- [~] `parser.at`(D20:parse_dump 直出,无独立 ast.at)：递归下降 + Pratt(优先级表
    直译)、类型解析、fn 参数、let 内联推断、for/if/while 全形态。
    闭包/f-string/is-match/use/泛型 = Missing(遇之报 v2 unsupported,corpus 无)。
    UI/task/store 区解析到即报 "v2 不支持"(含行号)。
- [~] 闸门 M2:语料+Rust 侧 dump 工具均就绪(18 文件黄金输出全绿),
    **但 AAVM 侧运行被 VM 字符串池 RC 回归阻断(D26)——闸门测试挂 ignore 待 VM 修复**。
- [x] 事实上的子切分:本切片一次落地声明+表达式+控制流+类型(语料所需全量)。

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

## 执行结果（进行中）

- **S1**(2026-08-24,单切片会话):
  - token.at:139 TokenKind + keyword_kind(58 词)+ kind_name 全量,自 b3bd64f5
    基线机械生成(docs/specs/aavm/data/ 脚本复用);
  - lexer.at(~500 行):主循环核心子集直译——空白/换行(基线 at 记账 quirk 照抄)/
    行+块+Doc 注释(含基线 '*' 吞字符 quirk)/标识符(连字符规则)/数字(进制+后缀+
    下划线剥离语义)/字符串转义/字符字面量/全部定界符与复合运算符;
  - M1 闸门:test_aavm2_m1(Rust 侧 Debug 名 dumper vs AAVM lex_dump,五字段逐
    token 对比),corpus_m1 四文件 diff=0;
  - **发现 VM 真 bug(挂 242)**:循环体含调用语句时 continue 穿透执行后续语句
    (D17,20 行最小复现 p11/p12;break 不受影响)——v2 侧全部改 else-if 链规避;
  - **发现基线 quirk(挂 432 债务簿)**:块注释内容 '*' 后一字符被吞(D18)、
    Newline token 行号取递增后值、数字 len 用剥离后长度——均照抄保闸门。
  - 已登记 AUTO_LIB_FILES_V2(token→lexer);全量 3129 绿 + aavm2 三用例绿。
- **S2 前置考古**(2026-08-24 续):M2 目标格式清单全量落盘
  (docs/specs/aavm/m2-ast-dump-format.md——顶层/语句/表达式/Op 的 S-expr
  逐字格式 + 来源行号,免下会话重做);实施决策 D20(parse_dump 直出)/
  D21(List.* sanctioned,C3 修正)预定;Pratt 表数值与 Branch 格式为开工首日项。

- **S2**(2026-08-24 续,worktree 432-core-port-s2):
  - **lexer.at 重构(D13/D14 落地)**:`tokenize(source) -> List<Token>` 直出
    真实 Token 结构;`lex_dump` 降为 List 上的格式化包装;**M1 闸门保持 diff=0**。
  - **parser.at(~1200 行)**:parse_dump 直出 S-expr。语句(let/var/const/shared、
    fn、if/else-if、for 全七形态、while/loop 脱糖 `(for (cond)...)`、break/
    continue/return、块语句);表达式(Pratt 表数值直译:asn 1/2 … dot 35/36、
    一元 26/28、postfix pair 8/call 30/index 32;方法调用重写 `(call (dot o.m))`、
    下标、Range、数组/元组/对象字面量、全字面量);类型解析(builtin 表+?T/!T);
    fn 参数(mode view 缺省/类型缺省 int/默认值);**let 无注解时的 parser 内联
    推断**(parse_store_stmt:7864 的 infer_type_expr 核心子集,经 E.ity 构造期
    计算——Bina unify/Array/Tuple/作用域查找);EmptyLine(n)与 expect_eos 语义。
  - **Rust 侧 dump 工具落地**(aavm2_m2.rs:rust_parse_dump = Parser::from +
    format!("{}", code));**corpus_m2 14 文件**(99_bootstrap 009-021 构造同源,
    未定义变量片段按 check_symbol 语义补绑定)+ corpus_m1 4 文件(全程序)
    = 18 文件黄金输出全绿(含 `(type int)` 推断、`(ret void)`、`(mode view))`、
    `(double 1)` 剥零、`(char '')` 原样、`(object (pair "k" ...))` 等 quirk)。
  - **语料修正 2 处**(S1 自建语料的解析性问题,非闸门放宽):
    c02 `print('')`(D19 转义收尾吞字符 quirk 使其不可解析)→ `print('\')`;
    c04 `match_or_like:` label 前缀(Rust parser 报 "Expected term, got While")
    → 去前缀。M1 为活对比,lexer 层覆盖不受影响。
  - **M2 闸门被 VM 字符串池 RC 回归阻断(D26,挂 242:S2-432)**:循环内以
    运行期字符串 `List.push` 后读回即 UAF canary(12 行最小复现,提升变量/
    改 for 均不可绕);**master 的 conformance_bootstrap 同类 canary 已红**
    (heap 4000001),99_bootstrap parser 系列测试 ignored——疑似 Plan 419/423
    RC 改造存量回归,非本切片引入。M2 闸门测试挂 ignore 注明;**VM 修复后
    移除 ignore 即可验闸**(parse_dump 逻辑侧已按黄金输出逐构造对齐)。
  - 发现并规避的 VM 缺陷(D25):构造参数内联原生调用返回值丢失、List.pop
    栈污染;全部以 .at 侧写法规避(提升变量 + depth 计数)。
  - AUTO_LIB_FILES_V2 已登记 parser.at;M1 闸门 + aavm2 smoke(001/002)全绿。
