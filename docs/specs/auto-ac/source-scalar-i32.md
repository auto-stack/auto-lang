# Auto 源码 scalar-i32 profile

> **Status**: experimental；`auto.source.scalar-i32.draft` revision 1（PLAN-772，开发线原PLAN-767；旧编号脚本/报告路径保留）。
> 实现：`experimental/ace/crates/core/src/source/`（与 core-i32 共用前端骨架，profile
> 显式选择）；消费端见 [project.md](project.md)；姊妹 profile 见
> [source-core-i32.md](source-core-i32.md)。

本 profile 在 core-i32 闭环之上补齐标量计算扩面：精确整数减/除/余、全序比较、
同类相等、一般取负与布尔 `&& / || / !` 短路，以及条件循环
`for <bool> { ... }`。它是 743 候选②的**标量子合同**，不是完整语言面：
char/str/List/聚合/泛型/完整 int 宽度仍由具名后续工作包承接。

## 数值与支持面

- `int`/`i32` 同义，精确有符号 32 位；Bool 与 I32 不隐式互转（沿 core-i32）。
- 字面量域与 core-i32 完全相同（[-2147483648,2147483647]；`- 2147483648` 合法；
  孤立正幅值 2147483648 及更大拒绝）。新增一般取负：`-x`、`-(expr)`、
  `- -x` 右结合链；`a-1` 为二元减，不误读相邻负字面量；一元 `+` 仍排除。
- 二元运算与优先级（低→高）：`||`、`&&`、`== !=`、`< > <= >=`、`+ -`、
  `* / %`、一元 `- !`、函数调用/分组；二元同级左结合，一元链右结合；
  连写比较按左结合 + 精确类型规则判断，无比较链糖。
- 精确语义（本 profile 固定规则）：
  - `+ - *` signed 精确计算，越界运行期 ExitProcess(70)（含普通常量表达式，
    不编译期求值、不提升未执行分支 trap）；
  - `/ %` 商向零截断，余数与被除数同号或 0；除数 0、MIN/-1 与 **MIN%-1 均
    trap70**（MIN%-1 是本 profile 的固定测试语义，不推导全语言/生产异常 ABI）；
  - 一元 `-` 对 MIN trap70；
  - `< > <= >=` signed 比较，结果规范 0/1；
  - `== !=` 两侧同为 i32 或同为 bool，混合拒绝；
  - `!` bool→bool；`&& / ||` 左侧恰好求值一次，`&&` 仅左真求值右侧、`||`
    仅左假求值右侧。
- 静态合法性独立于运行路径：短路未执行的 RHS 仍查名字/类型/图结构
  （`true || unknown()`、`false && (1+2)` 不得绕过静态检查）。
- 条件 for（区别于 core-i32 的 while）：`for <bool表达式> { ... }` 每轮进入
  循环体前求值条件；初值 false 零次执行；continue 回条件检查，break 离最近
  循环；条件可含函数调用与短路。HIR 侧降级为 `loop { if cond { body }
  else { break } }`（每轮重判、条件不 hoist），不新增低层控制流模型。
- 声明/作用域/函数/if/loop/break/continue、全 positional 或全 named 实参、
  源码序求值 + bindings 落位、迭代深输入与行纪律均沿 core-i32 规范不变。

## 排除与拒绝

- 本 profile 拒绝 `while`（`text/source.unsupported`，指向 while 关键字）；
  core-i32 profile 反向拒绝 `for`——两域语法面互不扩大。
- `for item in xs`、范围 for、带初始化/更新子句 for、非 bool 条件、缺闭合
  均受控拒绝并定位原 token；一元 `+`、char/str/float/其它宽度/后缀/进制、
  聚合/泛型/方法/多模块、标签维持排除（诊断分类沿 core-i32：unsupported/
  lex/syntax/bind/type-mismatch）。

## 身份与可信边界

- HIR 身份 `auto.hir.scalar.draft / 1 / scalar-i32-draft`（descriptor
  `experimental/ace/crates/core/schema/scalar-i32.atom`，core 的超集）；身份映射为
  **精确可失败映射**——未注册 schema/revision/profile 由 bind 整体拒绝
  （`bind.descriptor-unregistered`），canonical writer 对未注册/错域一律拒绝。
- verify 分域入口：`verify_scalar` 限本域；`verify` 限 core 域；域错位
  `verify.domain`。程序化 Bundle 变异受完整可信门：core 域混入 scalar-only
  运算/节点（`verify.profile-op`）、body 引用与模块级函数签名 params/result
  的 TypeUse 越界（`verify.bad-reference`，先于签名求值，不进后端不 panic）、
  共享表达式/自环/二节点环/跨 body 引用均受控拒绝；重新验证产生新 Checked。
- native eager 左→右；逻辑短路用分支 + 变量合流，RHS 只发射进条件执行块；
  i32 除/余在目标指令前显式护栏（0、MIN/-1、MIN%-1 走 trap70，不逃硬件
  异常）；短路表达式可用于任何实参/返回/初始化/赋值/条件位置。

## CLI、制品与测试能力

`check-source/build-source` 与 core-i32 共用命令面，`--source-profile
auto.source.scalar-i32.draft` 显式选择（漏/未知用法错 2；0/1/2 语义不变；
不按扩展名猜测）。build-source 收据在同一发布事务内记录 source profile、
HIR 三重身份（schema/revision/profile）与输入 hash；事务语义（失败不覆盖
既有成功制品）与硬截止进程约束两域一致。trace 能力沿 core-i32。

## 验收与范围来源

`scripts/verify-ac-scalar-767.ps1`：13 步 45 例（11 源码正例 + 15 HIR 正例
经 `ac-native` 真实 PE、19 阶段/code 反例、CLI 双向隔离、收据身份）；测试族
scalar_hir/scalar_source（旧 scalar_native PE 套件随 PLAN-871 旧 lowering 退役，
其 oracle 由脚本 ac-native 腿与 `ac-mir-native/tests/shadow_tier_871.rs` 承接）；
独立复审（含 needs_fix 三发现修复轮）反例与控制见
[attachments/767](../../plans/attachments/767/review-codex-r2.md)。
实现收据：[verification.md](../../reports/767-scalar-i32/verification.md)；
设计收据：[design.md](../../reports/767-scalar-i32/design.md)；
过程：[PLAN-772](../../plans/archive/772-ac-scalar-ops-short-circuit.md)。
char/str/List、完整 int 宽度、ACC 主体/代际仍 open，未重开 GOAL-017。
