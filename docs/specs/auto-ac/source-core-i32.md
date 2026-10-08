# Auto 源码 core-i32 profile

> **Status**: experimental；`auto.source.core-i32.draft` revision 1（PLAN-745）。
> 实现：`experimental/ac-core/src/source/`；消费端见 [project.md](project.md)。

本 profile 提供单模块 Auto 计算源码 → Checked HIR → Windows x64 PE 的有限域闭环。
它不替代生产 parser/VM/A2X，不证明完整 Auto 语言支持或 ACC native 自举。

## 数值与支持面

- `int`/`i32` 同义，精确有符号32位；Bool 与 I32 不隐式互转。
- 无后缀十进制字面量域 [-2147483648,2147483647]；越界报
  `text/source.literal-range`。`-2147483648`/`- 2147483648` 合法；负号只用于
  直接后随数字的字面量，`-x`/`-(expr)` 排除，不提供通用取负。
- 运算仅 `*` > `+` > `<`，左结合，lt 为 signed；加乘 signed 溢出运行期
  ExitProcess(70)，包括普通常量表达式；不编译期求值，不提升未执行分支 trap。
  70 是测试 profile 约定，不是全语言异常 ABI；完整 int 宽度继续 open。
- 函数参数/返回必须注解 int/i32/bool；let/var 始终初始化，可省类型注解；
  let 不可写，var 可写；同块重名拒绝，内层初始化 RHS 先读原环境，再绑定新名字。
- 有值 return、if/else、while、loop、无标签 break/continue，目标为最近外层循环。
  全文件函数均检查，含未调用函数/未执行分支；允许前向/直接/相互递归。
- 调用只允许全 positional 或全 named；实参严格按源码顺序求值，参数绑定单独
  建立双射。用户函数与局部绑定优先，不能被测试 intrinsic 劫持。
- ASCII 名字；完整 Auto 关键词不可作名字。语句用换行/分号，支持 `//`；
  未括起表达式的相邻 token 不跨语句换行；括号/实参表内换行是空白，闭括号
  后可在其所在行续算。callee 与 `(` 不跨换行。
- 没有固定嵌套计数排除：表达式/调用/块解析、Drop、resolve/typecheck、adapter
  （终止分类 memo）、verify 数据流、native 发射均显式迭代。合法深输入与未闭合
  深输入的正反验收保持；这不承诺无限内存或任意规模性能。

## 排除与拒绝

char/str/float、其它整数宽度/后缀/进制、聚合/泛型/方法/多模块、for、标签、
全序整数/逻辑运算、无值表达式语句不支持。识别但排除的 token 为
`text/source.unsupported`；坏 token 为 source.lex，坏语法/未闭合/剩余输入为
source.syntax。名字错误在 bind（undefined/duplicate/argument/not-callable），精确
类型错误在 verify/source.type-mismatch；后段保留既有 bind.*/verify.* 诊断。
verify 保守初始化、块唯一角色/可达/无环、全路径返回、终结后死语句规则仍适用。

## 来源与可信边界

源码 profile 显式选择；source AST/签名/词法与类型解析 → 生成 Atom 文本和来源表
→ 真正 bind_source → 全 Bundle span 投影 → verify 返回 CheckedModule → 共享
finish_build → native/暂存链接/事务发布。后端只接受本进程 verify 凭证，无旧 AST、
VM/A2R 回退，不补名字/类型检查；HIR 三重身份
`auto.hir.core.draft / 1 / core-i32-draft`、descriptor/运算面不变。
生成条目与 token 映射覆盖所有来源；bool 合成比较和 while 降低指回原 literal/条件。
范围/UTF8边界/无生成位置残留检查失败受控拒绝，缺映射不伪造位置、不 panic。
所有错误指向原 .at 冒犯 token/EOF，保留 UTF8/CRLF 行列与阶段/code。
未闭合输入不能产生 Checked 或发布制品；不能静默丢弃未闭合的调用帧。

## CLI、制品与测试能力

`check-source <file> --source-profile auto.source.core-i32.draft`；
`build-source <file> --source-profile auto.source.core-i32.draft --entry <源码函数名>
--output <exe> [--capability N]... [--support-lib L]...`。
profile 漏/未知是用法错2；成功/流水线拒绝/用法错为0/1/2，输入模式不按扩展名猜测。
build 入口要求零参数 int/i32；源码函数 native 符号加 sf_，允许用户使用 ac_start/
ExitProcess 名字，绑定仍走 DefId。源码 profile/输入 hash 与 exe/obj/link 收据同事务
发布；所有失败保留原三件，事务与硬截止进程约束见 [project.md](project.md)。
mark_a/mark_b 仅显式 `hir.test.trace.v1` 激活，nullary→int；用户声明优先。
原生 trace 支持库须显式链接，不是生产 I/O/运行时契约。

## 验收与范围来源

`scripts/verify-ac-source-745.ps1`：13步/74例（21真实PE，53阶段/code反例）；
source_frontend/hir/native/cli、deep_regression 覆盖 token、语义、求值顺序、trap、
真实Bundle变异、事务SENTINEL及块/表达式/调用深输入。
最终独立复审另含逗号/未闭合帧与24项块流控制：
[报告](../../plans/attachments/745/review-r7/REVIEW.md)。
数值取证：[numeric-boundary.md](../../reports/745-source-core-i32/numeric-boundary.md)；
实现收据：[verification.md](../../reports/745-source-core-i32/verification.md)；
过程：[PLAN-745](../../plans/archive/745-ac-source-checked-hir-native.md)。
完整宽度/char语义及ACC主体/Gen1–3仍 open，未重开 GOAL-017。
