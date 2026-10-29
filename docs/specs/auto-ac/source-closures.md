# 源码闭包域（auto.source.closures.draft）

> 状态：实验性、显式选择的源码 profile。由 PLAN-865 交付；不改变其他源码 profile、默认入口或 `av.closures/1` 的 wire/运行时合同。

## Profile 与语法

`auto.source.closures.draft` 在 `auto.source.collections.draft` 的集合源码面上增加闭包函数类型、lambda、捕获分析和间接调用，并沿用 862 的入口/Unit 面。它使用独立的 HIR `Closures` 域与 descriptor；调用方必须显式选择该 profile，不能由文件后缀或内容推断。

lambda 支持零参、多参、表达式体和语句块体：

- `(x int, y bool) => expr` 与 `(x int, y bool) => { statements }`
- `move` 可放在任一 lambda 形式前，例如 `move (value Str) => ...`
- 函数类型写作 `fn(T0, ...) -> R`；现有语料的相邻结果类型形式 `fn(T) R` 在函数签名位置仍可用。

闭包签名由参数和函数体推导，也可由声明的函数类型约束；参数顺序、个数和返回类型必须一致。函数类型局部值可像普通函数一样调用，调用降为闭包间接调用。lambda 可在有期望类型的位置赋值、传参或返回；直接调用 lambda 时先用括号分组。命名函数项尚不是一等函数值。

## 捕获与所有权（SEM-04）

捕获分析在 HIR/lowering 前完成。自由局部变量按其在 lambda 中首次出现的源码顺序去重，lambda 参数和块内局部不算捕获；该次序同时决定 `CREATE_CLOSURE` 的捕获顺序及闭包 `env` 字段顺序。lowering 消费已判定的捕获列表与模式，不得自行猜测或修补。

| 捕获情形 | 规则 |
|---|---|
| `int`、`bool`、`char` 等 Copy 值 | 隐式 lambda 与 `move` lambda 均按值复制；源局部仍可用，不插入 clone。 |
| 可支持的非 Copy 拥有值，使用 `move` | 所有权转入闭包环境；源局部之后不可再读或调用其已移走的闭包。闭包沿 823 生命周期路径恰好释放环境一次。 |
| 已拥有闭包的显式 `.clone()` | 深复制当前 owned 环境并返回独立闭包句柄，沿用 823 clone 路径；闭包构造不隐式插入 clone。 |
| 非 `move` 的 `Str` 借用 | 仅限可静态证明同步、非逃逸的本函数内直接调用；借用环境用 823 的 `StrView` 表示，宿主 `Str` 必须活到最后一次调用。 |
| 写入捕获局部、借用逃逸或无法证明寿命的捕获 | 编译期具名拒绝；不依赖运行期兜底。 |

借用闭包不能返回、存入字段或容器、作为参数传给调用方，或被另一闭包捕获。写入捕获局部属于共享可变捕获，本 profile 拒绝。集合支持不会自动扩大闭包环境类型域：捕获类型必须同时被源码 profile 与 823 `av.closures/1` 环境准入支持；嵌套 List、函数值 List 元素、借用 token 等不因本计划获得支持。

## MIR 与执行绑定

本 profile 将函数签名和值降到已有函数/闭包类型，lambda 降到 823 的 `CREATE_CLOSURE`、`CALL_CLOSURE` 及 env 布局；没有新增闭包 wire 身份或执行指令。闭包 body 的 `env` 声明和创建点捕获字段必须逐位一致，`fn:` 绑定的环境声明在链接阶段精确比较。env 漂移在发布候选前拒绝，已安装的有效版本保持可调用。

源码编译进入现有 `ClosuresUpdateSession` 候选路径。函数体编辑、捕获集合变化和签名变化按函数单元失效；未变化函数不应重新生产。源码诊断、MIR/链接拒绝和运行时结果保持分态。

## 诊断与边界

常见具名拒绝包括：`source.closure-use-after-move`、`source.closure-capture-unsupported`、`source.closure-borrow-escape`、`source.closure-shared-mutable`、`source.closure-unsupported` 和 `source.closure-env-mismatch`。间接调用的元数与签名错误由普通参数/类型诊断拒绝。

下列能力仍不在本切片：默认/变长参数、命名函数项作为值、集合 `map/filter/reduce` 或方法 HOF、闭包作为字段/List 元素、共享可变捕获及循环环境、async/生成器、跨线程、持久闭包和 native 闭包 ABI。共享可变捕获 C-19 仍由 R5-O 承接；HOF 库/方法分派另立能力包。

## ACE M1 覆盖

PLAN-865 将原 K01–K06 `.at` 语料绑定到真实源码编译和 823 执行会话，recipe 为 `ac-av/closures-source-run-v1`。K01、K02、K03、K05 经源码到 AV 执行；K04 在源码编译期以 `source.closure-borrow-escape` 拒绝；K06 以 `source.closure-shared-mutable` 拒绝，仍保留 `gap:R5-O`。K05 的独立性反例返回 11；若 clone 与原环境别名则为 12。旧 AVM/a2r 对 K02、K05 的 `diff-legacy-grammar(parse)` 保持原样，不因 AV 源码路径交付而改写。

详见[闭包环境与执行合同](../auto-av/closure-environments.md)、[闭包捕获 MIR](../auto-mir/closure-capture-profile.md)和[ACE M1 一致性语料](../auto-test-case/ace-m1-conformance.md)中的 PLAN-865 绑定。
