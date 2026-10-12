---
plan_id: PLAN-767
status: archived
feature_name: AC 标量计算扩面：精确整数运算与布尔短路
plan_revision: 2
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12
current_step: 9
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-hir/project.md
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-acc/project.md
new_spec_components:
  - docs/specs/auto-ac/source-scalar-i32.md
touched_goals: []
affects: [auto-ac, auto-hir, auto-acc]
---

# [PLAN-767] AC 标量计算扩面：精确整数运算与布尔短路

## 0. 变更摘要

在 745 已交付的源码 → Checked HIR → Windows x64 PE 管线上，新增一个显式选择的
scalar-i32 profile，补齐 ACC 常用的整数减/除/余、相等/全序比较、一般取负及布尔
`&& / || / !`，并支持 Auto 的条件循环 `for 条件 { ... }`。
旧 core-i32 profile 保持原支持/拒绝和数值语义，包括其历史 while 写法；
新 scalar-i32 源码 profile 使用条件 for，不引入 while 作为对外语法。

这是 743「编译器所需类型/运行时首批」候选②的**标量子合同**，不是交付整个候选②。
成果是可用于扫描器/解析器条件判断的原生计算底座；字符串、字符、List、聚合能力仍有
具名后续工作包，不能凭本计划关闭 A4/A5/A8 或宣称 ACC Gen1。

## 1. 目标

1. 新运算从真实 `.at` 源码经 bind/verify 到真实 PE，保持求值次序、短路及运行期 trap。
2. 把新旧 profile 的身份/能力边界贯穿绑定、Checked 凭证、序列化和构建收据；
   不能仅在 lexer 放开 token，也不能把旧身份的 HIR 静默解释为新语义。
3. 在源码域和直接 HIR 域分别验证新能力；程序化 Bundle 变异同样受 verifier 管控。
4. 保留 741/745 的真实 CLI、命名实参顺序、来源映射、深输入、事务与进程生命周期行为。

非目标：裁定全语言 int 宽度、i64/uint/float/unit 扩面、char/str/List、record/enum、
用户泛型、范围/集合遍历 for、带初始化/更新子句的 for、use/全局变量、生产 ABI/DLL/热重载、ACC 主体迁移/桥、通用优化 pass、
自主 SSA/机器码后端、性能或发行承诺。原生目标仍为 Windows x64。

## 2. 架构方案

沿用 `experimental/ac-core` 独立 workspace，不接旧 parser/VM/A2X：

```text
显式 source profile
  → S0 迭代 lexer/parser/resolve/typecheck
  → S1 对应 descriptor 的 Atom + 原源码 span 映射
  → S2 bound Bundle + 明确 profile 上下文
  → S3 verify（类型/图结构/身份/能力边界）
  → S4 CheckedModule（唯一合法构造入口）
  → S5/S6 共享 native / link / 事务发布
  → COFF + PE + 构建收据
```

公共 HIR 表达短路语义，CLIF 块/value/合流细节只存在于 native adapter。
普通 eager Binary 与 lazy 逻辑表达式分开表示；禁止将逻辑操作映射成普通双参数
intrinsic，或先用现有 `eval_expr` 的后序走查无条件发射 RHS 再 select。
T-01 补充节点/版本化 API 的具体设计收据，随后实现；不改变本文已固定的可观察规则。

## 3. 技术栈

现有 Rust / Cranelift 0.126.2 / rust-lld 或已验证 MSVC link / Windows SDK。
依赖版本以执行树 Cargo.lock 为准，不因扩面升级后端；沿用 `test-support` trace 静态库。
测试使用现有 ac-core 测试入口与受控进程执行器；新增源码/HIR fixtures 和专项门禁脚本。

## 4. 需求分析与背景调查

### 4.1 三计划的实际交付

主检出为 `D:/autostack/auto-lang`，当前落点 `v0.6-dev`，规划基线
`c64b71353e740bf84246de8bd612f692cba04db1`。不是切换到历史 master 另起一条旧基线。

| 来源 | 已交付事实 | 仍未交付 |
|---|---|---|
| [741](741-ac-hir-native-core.md)，archived 44/44 | Atom HIR → verify → Cranelift COFF/PE，可信门/工具事务强化 | 源码前端及完整语言面 |
| [743](743-acc-bootstrap-hir-contract.md)，archived 38/38 | ACC 主体/能力清单、阶段/桥/代际合同及严格盘点工具 | 目标态合同所描述的 ACC 实现 |
| [745](745-ac-source-checked-hir-native.md)，archived 9/9 | 单模块 core-i32 源码 → Checked → PE，74例及独立深输入/帧控制 | 运算扩面、堆数据及 ACC 主体 |

当前 canonical Specs 优先于旧战略的实现状态快照：
[全局总览](../../specs/overview.md)、[AC](../../specs/auto-ac/project.md)、
[source-core-i32](../../specs/auto-ac/source-core-i32.md)、
[HIR](../../specs/auto-hir/project.md)、[stage-contract](../../specs/auto-hir/stage-contract.md)、
[ACC](../../specs/auto-acc/project.md)。战略中的 2026-10-04 队列和 743 报告的
「源码 adapter 未实现」保留为历史，不能用于否认 745 已完成。

### 4.2 为什么先做标量子合同

[743 候选②](../../reports/743-acc-hir-contract/next-work-packages.md) 原包括
str/char/List 和整数/布尔运算，适合拆成独立实施合同，不能一次冻结全部资源/布局规则。
[str 最高优先与运算需求](../../reports/743-acc-hir-contract/inventory.md)是同一关键路径：
`auto/lib/lexer.at::is_digit/is_alpha/is_alnum/is_hex_digit` 已使用 `>= / <= / == / && / ||`，
游标更新、切片端点和 parser 条件还需要减法及逻辑否定。
当前 `hir.rs::BinOp` 只有 AddI32/MulI32/LtI32，source AST 同样仅三种二元操作，
`main.rs` 只接受一个 SOURCE_PROFILE。因此先建立比较/短路与严格 profile 扩面机制，
再引入需要所有权和字符索引决策的堆数据，是一个可独立复审的有界切口。

循环语法依据 [for 教程](../../../website/docs/tutorials/for-loop-guide.md) 的条件循环形式
`for condition { ... }`。主 parser 的 `for_stmt` 已有 `Iter::Cond`；旧
`while_stmt` 也降至此形式，但兼容实现不是新源码 profile 的语法依据。
745 原型只接 while、排除 for 是已交付 profile 的局限，本计划补条件 for，
不接旧 AST，不把范围循环、集合遍历和初始化/更新子句一起纳入。

[str/char 语义冲突](../../reports/745-source-core-i32/numeric-boundary.md)依然存在：
VM builtin 的 char_at 返回 str、auto-lib 的表面返回 int、Value 保存 Unicode char。
本计划不借补比较操作裁定这些冲突；完整 int 宽度也保持 unknown。
取证固定于 [inputs.json](attachments/767/inputs.json) 的完整 SHA256 与主检出 HEAD；
执行开始复核漂移，必要时记录差异，而不是直接沿用旧 hash 宣称新鲜。

### 4.3 战略对应及后续队列

依据[后端演进 §3/§7.3](../../design/strategy/auto-native-backend-evolution.md)、
[AC 子集 §2/§3](../../design/strategy/auto-ac-subset-and-target.md)、
[自举合同](../../design/strategy/auto-acc-bootstrap-contract.md)和
[HIR 求值顺序](../../design/strategy/auto-hir-design.md)。v0.6 继续使用 Cranelift。

| 工作包（除767外均未取号、未执行） | 独立成果 | 前置/归属 |
|---|---|---|
| 本计划767：标量扩面 | 精确 i32 运算、全序比较、bool 短路及版本化可信门 | 741/743/745已满足 |
| 文本运行时首批（下一优先） | str/char/literal、len/char_at/slice/concat/compare；字符索引、所有权/释放及失败语义先有界裁定 | 消费767；运行时线认领 MD-406/407/408 与 probe-str-ml/probe-rt-own |
| 容器/聚合首批 | 内建 List 的 push/get/遍历及所需 record/TokenKind/is 能力，按可验收成果继续拆分 | 文本/资源合同；分别认领 MD-405 与 A9 聚合线 |
| 首个真实 ACC 词法组件 | token/lexer 适配源码经 AC 编译后运行，词元序列对照 A4/corpus_m1 | 上述能力；标明适配改写与未迁移部分 |
| 后续主体/桥/代际 | resolver/typeck/HIR/lowering/driver 迁移，Gen1→Gen2→Gen3 | 743 主体清单与代际合同，另行计划 |

生产 ABI 线可按 NEXT-C 独立推进；本计划不改变其顺序、范围或现有 worktree。
这是一份新合同的拆分建议，不是删减 743/745 已批准验收。26–36 总量仍为历史工程估算，
不将完成3个计划折算成自举完成百分比，也不据标量扩面速度给出完成日期。

### 4.4 授权与边界

用户本轮授权：根据三个已完成计划及战略文件规划下一计划，解释下一步实现内容。
已授权动作：仓内只读调查、统一取号、计划与取证附件簿记。尚未授权执行本新合同。
不创建实施树、不写产品/旧计划/ canonical Specs/账本，不启动实施或合入。
无预算或自动续跑授权；无指定实施/内部检查/最终复审模型。

## 5. 详细设计

### 5.1 显式版本化与兼容

- 新源码身份：`auto.source.scalar-i32.draft` revision 1。
- 新 HIR 三重身份：`auto.hir.scalar.draft / 1 / scalar-i32-draft`。
- 新 descriptor（新增路径）：`experimental/ac-core/schema/scalar-i32.atom`。
- 老 `auto.source.core-i32.draft` 和 `auto.hir.core.draft / 1 / core-i32-draft`
  保持原行为：新运算在老域仍拒绝，不能用扩大的共享 enum 绕过旧 descriptor/profile。
- 当前 `hir::Bundle` **不保存** schema/revision/profile。实现必须让权威身份从 binder
  到 verifier、Checked 与 native 随行；可以选择有类型的上下文/envelope或数据字段，
  不能从运算集合猜身份，不能在未知/缺失身份时默认升级为新 profile。
  既有 `verify(bundle)` 接口若保留，其兼容语义必须限定旧域；新入口须显式选新身份。
- 非文本构造/变异的 Bundle 仍在 verify 检查所属 profile、类型与图结构；重新验证
  产生新 Checked，旧凭证不可复用。身份标签是声明，不是语义校验凭证。
- HIR check/build 按三重身份选择已注册 descriptor；未知身份受控 text/bind 诊断。
  canonical writer 不得借调用者传入另一 descriptor 给旧 Bundle 换壳；冲突显式拒绝。
- 新源码经对应 Atom binder 与真实 verify；共享 finish_build/native/link/publish 尾部。
  收据记录 source profile、HIR三重身份、输入 hash、后端/工具链，和三件制品同事务。
  profile 缺失/未知的源码 CLI 用法错2，流水线拒绝1，成功0。

### 5.2 精确运算规则（限本 profile）

`int/i32` = 有符号32位；Bool/I32 不隐式转换。保留旧声明、作用域、函数、控制流规则。

| 运算 | 参数 → 结果 | 固定规则 |
|---|---|---|
| `+ - *` | i32 × i32 → i32 | signed精确计算；越界运行期trap70 |
| `/ %` | i32 × i32 → i32 | 商向零截断；余数与被除数同号或0；除数0、MIN/-1与MIN%-1均trap70 |
| 一元 `-` | i32 → i32 | MIN取负trap70；一般变量/括号表达式支持 |
| `< > <= >=` | i32 × i32 → bool | signed比较，结果为规范0/1 |
| `== !=` | 两侧同为i32或同为bool → bool | 精确值比较；混合类型拒绝 |
| `!` | bool → bool | 逻辑否定，结果为规范0/1 |
| `&& / ||` | bool × bool → bool | 左侧恰好求值一次；&&仅左真时求值右侧，||仅左假时求值右侧 |

普通运行期常量表达式也保持运行期trap；不借编译器常量求值提前拒绝。
即使短路不执行某 RHS，该 RHS 的名字/类型/图结构仍必须静态合法。
`false && unknown()`、`true || 1` 不得绕过静态检查。

优先级从低到高：`||`、`&&`、`== !=`、`< > <= >=`、`+ -`、`* / %`、
一元负号、一元`!`、函数调用/分组；二元同级左结合，一元链右结合。
相对次序与当前 parser.rs 的 PREC_OR/AND/EQ/CMP/ADD/MUL/SIGN/NOT/CALL 对齐，
仅作语法事实参考，不接旧 AST。连写比较按左结合及精确类型规则判断，不另加比较链糖。

十进制字面量域仍是[-2147483648,2147483647]；`-2147483648`与带空格形式保持合法，
孤立正幅值2147483648及更大拒绝，不放宽为i64临时再截断。
`-(2147483648)`拒绝；`-(-2147483648)`可构建、运行trap70。
`-x`、`-(expr)`新增支持；一元`+`仍排除。`a-1`不得误读为相邻负literal。
换行/CRLF、参数逗号、括号收口规则继承745，不因多操作符丢弃帧或尾部输入。

### 5.2.1 条件 for（revision 2）

新源码 profile 接受 `for <bool表达式> { ... }`，包括 `for y != 0 { ... }`。
条件在每轮进入循环体前求值；初值false执行零次；continue回到条件检查，
break离开最近循环。嵌套循环、条件中的函数调用和短路都遵守同一规则。
条件必须bool；缺条件、缺闭合、`for item in xs`、范围/初始化/更新形态显式拒绝。
新profile的while报text/source.unsupported；旧profile的while仍按745原合同接受，
旧profile的for仍拒绝，不能通过共享lexer/parser扩大旧profile语法面。

source AST内部可复用既有条件循环结构；公共HIR仍降为loop及条件分支，
不因源码关键字改变而新增低层控制流模型。合成节点span指回for关键字/原条件；
条件不可hoist，continue不能跳过重新判断。本规则适用于所有新profile正向源码语料，
已有745旧profile语料保持原输入及预期。

### 5.3 HIR、verify 与 native

公共 HIR 保留精确类型、源 span、eager/lazy区分。拟新增 unary 和逻辑短路节点，
精确字段/enum表由T-01设计收据冻结后T-02物化，不能携带CLIF句柄。
既有bool字面量合法编码可复用，本计划不要求独立新增bool constant节点。

verify 同时扩充 operand/result类型、profile限制、表达式引用归属/无环和初始化走查，
不能只在source typecheck查类型。共享子表达式、跨body引用、短路自环/二节点环、
伪checked字段必须在native前受控拒绝。RHS条件执行不改变“每个ExprId只有一个语法
求值位置”的不变量；其含义不等于运行时每轮/每条路径都执行该位置。

native eager操作仍左→右；逻辑操作用分支和合流，RHS代码只能放在条件执行块，
不允许无条件调用/执行可能trap的操作。i32除/余必须在目标指令前显式护栏，
MIN/-1和0不能逃成OS硬件异常；跳转既有trap70路径。
命名实参先按源码顺序求值，再按bindings落位；逻辑表达式作为任何实参/返回/初始化/
赋值/条件时都适用，不能只处理if顶层条件。

新增节点的parse/Drop/resolve/typecheck/adapter/verify/native遍历须保持迭代，
不重引入固定嵌套排除或递归栈崩溃。旧块/调用/表达式深输入不退化。

### 5.4 规范增量（提案，review定稿、merge发布）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-scalar-i32.md | 仅core-i32源码→新增显式scalar-i32运算/优先级/trap/短路/条件for与拒绝域 | 定义新profile而非改写旧语义 | AC-01..04,06 |
| SD-02 | modify | docs/specs/auto-hir/project.md | 单descriptor/三op→双身份域、unary/lazy运算及verify边界 | 共享公共HIR、隔离CLIF与身份越权 | AC-01,04,05 |
| SD-03 | modify | docs/specs/auto-hir/stage-contract.md | S0–S6仅core-i32已实现→追加新域与身份随行、条件求值和for到loop降级规则 | 区分阶段事实与pass目标态 | AC-01,03..05 |
| SD-04 | modify | docs/specs/auto-ac/project.md | 单源码profile→显式双profile、收据身份及旧行为兼容 | 消费者/CLI契约可核查 | AC-01,06,07 |
| SD-05 | modify | docs/specs/auto-acc/project.md | MD-409/410仍required→追加本profile运算证据，完整语言/字符/主体/代际仍open | 不以原型运算冒充ACC或全语言实现 | AC-07,08 |

不关闭GOAL-017或添加自举达成声明。若更新743人工层，逐条记录证据/绑定理由、只追加
有限域结论，不把required/unknown整项改成implemented；严格门必须保持新鲜。
旧战略阶段快照属于历史，不为实现新profile而全篇改写。

## 6. 测试设计

### 6.1 可执行矩阵与oracle

以下是待T-02..T-06物化的新cases，不是规划阶段已执行的证据。全部通过实际
check-source/build-source或直接HIR bind/verify/build入口；正向native必须运行真实PE。
测试入口同时记录case、源码/HIR hash、binary/rlib/工具链、exit和完整stderr。

| Case | AC/不变量 | 输入/场景 | 可观察预期/oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01 新旧隔离 | 同一减法/短路源码分别选旧/新profile；旧Atom头加入新op；错误schema/revision/profile | 旧source拒source.unsupported，新source成功；旧HIR新op/未知身份拒绝；canonical换壳拒绝 | 两显式profile与程序化API |
| V-02 | AC-02 signed运算 | -7/3、7/-3、-7%3、7%-3、-7/-3、-7%-3 | 分别-2/-2/-1/1/2/-1，源码包装成退出0自断言，HIR和运行证据同源 | i32，不能拿PE负exit低字节当值oracle |
| V-03 | AC-02/03 trap时序 | MAX+1、MIN-1、MIN取负、MIN/-1、MIN%-1、1/0、1%0；先调用mark_a再求值 | build成功；exit70、前驱stderr精确a；对比未执行分支无trap | 普通运行期域，非comptime |
| V-04 | AC-03 短路 | false&&rhs、true||rhs、true&&rhs、false||rhs；rhs有mark_b或除零 | 跳过两项exit0/无b/无trap；执行两项精确一次b或trap70 | RHS实际bool函数，类型合法 |
| V-05 | AC-03 左侧唯一及嵌套 | lhs标a、rhs标b；组合于return/let/assign/条件for/call/named arg，循环至少两轮 | 按真值路径精确a或ab；每轮重算，命名绑定不重排trace；bool返回/传参仍0/1 | 不只顶层if路径 |
| V-06 | AC-02/04 比较与拒绝 | MIN/MAX/负数全序，bool四组eq/ne、优先级；混合bool/int、!1、-true、true||unknown()、false&&(1+2) | 正向exit0；反向bind/verify的源码原token诊断，未执行RHS也查名/类型 | 静态合法性≠运行期执行 |
| V-07 | AC-05 可信门 | 真实bound Bundle变异成错类型unary/lazy、旧身份新增op、共享Expr、自环/双环/跨body引用 | 本进程verify受控拒绝，无Checked/native产物；输出后重verify不能沿用旧凭证 | 不只文本parser负例 |
| V-08 | AC-04/06 parser帧 | a-1/负literal、二元实参逗号、跨行/CRLF、缺操作数/缺右括号、混合优先级 | 正向实际PE正确；畸形source.syntax，不掉调用/尾部、不panic | 新操作符与745原帧规则 |
| V-09 | AC-06 深输入 | 16384项逻辑链、20000层!、4096层逻辑/调用混合及缺闭合反例；保留745深块族 | 合法到Checked（含Drop），畸形受控拒绝；小型混合1000层至少一例真实PE正确 | 不承诺任意规模性能/无限内存 |
| V-10 | AC-01/06 工具事务 | 新profile成功收据；旧成功三件SENTINEL后新source/HIR拒绝、故障注入link/publish | 身份/hash同事务；失败保持exe/obj/收据原hash，执行器全出口回收、沿用显式残留诊断 | 继承741/745，不重造事务执行器 |
| V-11 | AC-07 代表计算 | Euclid gcd(48,18)=6；十进制计数12345=5；lexer is_digit的i32码点比较改写 | 源码→PE退出0，固定独立值断言/区间正反边界；明确非原lexer编译 | char/str/enum/List仍缺 |
| V-12 | AC-01/04/06 条件for | 新profile的for y != 0、初值false、嵌套break/continue、带trace短路条件；旧/新profile分别输入for/while，非bool条件、未闭合、for-in/初始化子句 | gcd结果6、零次不执行body、continue每轮重验条件/trace次数精确；新for接受/while拒绝，旧while接受/for拒绝；畸形及排除形态受控拒绝且原token定位正确 | 条件循环，不覆盖迭代器/范围/三段式for |

运算oracle来自§5.2固定规则、精确整数数学与真值表；旧VM宽度/溢出行为不作为本profile
权威。已有test-support负责trace；支撑库不得替Auto函数做运算/解释HIR。
直接HIR测试覆盖文本binding/canonical roundtrip/verify/native；源码不能全绿就替代HIR证据。
负例诊断必须指向原`.at`冒犯token/EOF（UTF8注释与CRLF也验证），新码在T-01中固定。
测试时间上界沿用受控runner：PE≤60s，结构负例外层5s、深结构检查外层60s；
长构建门禁设600s且覆盖子树/输出收集，不用无人值守的无限等待。

### 6.2 分级门禁与角色

实施前/迭代在唯一专属树内运行（根工作目录为该树）：

```text
cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets
cargo fmt --manifest-path experimental/ac-core/Cargo.toml -- --check
cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test scalar_hir
cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test scalar_source
cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test scalar_native
```

三个scalar测试目标是新增（T-02/03/04/05负责物化）；不存在时不能将命令失败称门禁通过。
交review前一次完整ac-core `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked`，
再执行 `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-scalar-767.ps1`（新增，
T-06），以及已有 `scripts/verify-ac-741.ps1` / `scripts/verify-ac-source-745.ps1`。
旧脚本若包含root档，提供保持原型case集不缩水的显式跳过root模式，记录脚本diff与case身份。
盘点只跑 `python scripts/acc_inventory.py --check --require-decisions`，有漂移先审计。

本范围仅独立experimental Rust workspace、scripts及计划取证，不改crates/或根Cargo/
配置/旧auto-lib，依AGENTS Category A限制**不跑根cargo t/tv/docs_gen**，不因历史计划
曾跑根重档而照抄。所有原型测试仍必跑。若执行发现需要改生产Rust，先记录范围变化
并按new修订再依触面跑root check、t、tv（trans才tt、aavm触发才taa）。tf仅由merge到期
批回归处理，主检出单实例，不作本计划实施/复审随身门禁。

实施角色尚未指定；默认独立最终复审在实施暂停写入后使用同一树，按AC逐条复验，
不是实施者自测绿就reviewed。无另建agent/chat或指定模型授权。

## 7. 验收标准

全部初始未完成；owner_stage用于明确何时、由谁验收，不是新增后台状态字段。

| ID | owner_stage | 可观察验收与验证方法 |
|---|---|---|
| AC-01 | work | 双profile显式身份、绑定/Checked/canonical/native边界成立；旧支持/拒绝保留，包括旧while与新for隔离；V-01/10/12与741/745门禁通过 |
| AC-02 | work | §5.2整数运算/全序/eq-ne/优先级及数值边界经源码和直接HIR实际PE验证；V-02/03/06/08/11通过 |
| AC-03 | work | 短路在所有声明表达式上下文保持效果/调用次序、左侧一次、RHS条件执行及运行期trap；V-03..05精确trace通过 |
| AC-04 | work | 名字/类型/语法/来源映射在新域成立，包括未执行RHS静态拒绝、条件for的bool条件/收口；V-06/08/12每项有阶段/code/原token断言 |
| AC-05 | work | 新运算直接HIR及程序化Bundle同受verify、归属/无环/旧域限制，未验不进后端；V-01/07与roundtrip通过 |
| AC-06 | work | 深输入/旧调用帧/事务/进程回收不退化，条件for的零次/每轮条件/break/continue正确；V-08..10/12及旧永久反例通过；完整原型测试/locked all-targets零warning/fmt成功 |
| AC-07 | work | case清单/真实PE/运行收据与输入和工具链绑定，一键门禁实际执行；SD01..05冻结提案有据，盘点严格门新鲜；不宣称char/str/List/ACC代际 |
| AC-08 | review | 独立上下文核查AC01..07、SD与遗漏/延期/workaround/债务；绑定revision、HEAD、依赖及输入hash，合同内缺陷清零才pass |
| AC-09 | merge | 获合入授权后实际沉淀SD/模块导航/ledger/index、到期批回归、归档9/9、guard clean再清同一树/分支/空组；最终身份/链接/计数与清理收据验证 |

## 8. 执行步骤

计划簿记在默认检出；唯一实施树意向为
`D:/autostack/.wt/lang-767/auto-lang`，分支`plan-767-dev`。
这是未来路径，本轮未创建。work先确认745恢复ref及实际落点，复用适合的已登记767树，
不存在时按AGENTS在当前整合落点v0.6-dev建立。全生命周期一树，禁止junction/symlink。

| 状态 | Task | owner_stage | 依赖 | 路径/操作、独立成果与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 用户批准当前r2实施；741/743/745落地 | 确认同一树/branch/base/inputs、零生产diff；新增docs/reports/767-scalar-i32/design.md：节点字段、profile随行API/注册选择、旧verify兼容、writer身份冲突和诊断码及case清单。静态preflight逐项§5/矩阵，禁止改可观察规则；HEAD/hash记录 | 01..07 |

[✅ 2026-10-12 work 修复轮] R3 修复：design.md 减法护栏公式改为与实现一致的 `(b>0 && a<min+b)||(b<=0 && a>max+b)`；§1 域映射描述改为精确可失败映射；§6 诊断表补 bind.descriptor-unregistered 并扩展 writer 拒绝范围；§4 补 P767-R2 模块级签名边界门说明
[✅ 2026-10-12 work] r2 合同生效：worktree `D:/autostack/.wt/lang-767/auto-lang`、branch `plan-767-dev`、base `a9225e8de`（=v0.6-dev HEAD；与规划基线 c64b71353e 间仅计划簿记 diff，已核对）；inputs.json 全条目 SHA256 复核零漂移；主检出 code 路径零 WIP。设计收据 docs/reports/767-scalar-i32/design.md 按 r2 冻结（节点字段/双入口 verify/注册表/writer 冲突/诊断码 4 新码/V-01..V-12 物化清单/§5+§5.2.1 静态 preflight）。
| [x] | T-02 | work | T-01 | src/hir.rs、descriptor.rs、verify.rs、lib.rs及新增schema/scalar-i32.atom、tests/scalar_hir.rs、fixtures/scalar-hir/：双profile+新op/节点+真verify，roundtrip与V01/07/类型正反例；locked all-targets与scalar_hir通过 | 01,04,05 |

[✅ 2026-10-12 work 修复轮] R1/R2 修复@4cc464502：domain()→Option 精确映射、bind 拒未注册 descriptor（bind.descriptor-unregistered）、writer 未注册/错域全拒；verify 增模块级签名 TypeUse 边界门（verify.bad-reference 先于签名求值）；native value_of_type_use/signature_of 全化；scalar_hir 新增 2 个永久回归（16/16）
[✅ 2026-10-12 work] code@plan-767-dev（T-01/T-02 合并提交，base a9225e8de）：scalar-i32 descriptor/binder/verify/native 扩面落地；scalar_hir 14/14（V-01 HIR 半、V-07 变异门、roundtrip、类型矩阵负例、4096 逻辑深链）；全 ac-core 128 测试零回归；locked all-targets 零 warning；fmt clean。
| [x] | T-03 | work | T-02 | src/source/{lexer,ast,parser,resolve,typecheck,adapter,mod}.rs：显式profile、条件for解析/逐轮降级、运算/优先级、迭代帧和span投影；新增tests/scalar_source.rs、fixtures/scalar-source/。V06/08/09/12通过；所有新源码仍经Atom bind/verify | 01,02,04..06 |

[✅ 2026-10-12 work] code@b77391e63：scalar 前端全链落地（条件for复用while形状、两域 while/for 隔离、新算子/一元/短路、迭代帧与 Drop）；scalar_source 7/7（V-06 类型矩阵、V-08 帧规则含 CRLF、V-09 深输入 16384/20000/4096、V-12 条件for矩阵、V-01 parser 半边）；全 ac-core 135 测试零回归（core 行为不变）；零 warning、fmt clean。
| [x] | T-04 | work | T-02 | src/native.rs：eager算术/比较/unary、div/rem护栏、lazy CFG与规范bool；新增tests/scalar_native.rs的真实HIR PE及trace。V02..05/07正反控制通过，不复用宿主算术作被测执行 | 02,03,05,06 |

[✅ 2026-10-12 work] code@7abeb2654：native 新op lower 已随 T-02 落地并经真实 PE 验证；scalar_native HIR PE 8 用例（V-02 六行截断表 exit0、V-03 七 trap exit70+stderr精确a+未执行分支对照、V-04 六极性含跳过除零、V-07 正控），不复用宿主算术
| [x] | T-05 | work | T-03/04 | src/main.rs、source/adapter.rs与既有source_cli/source_native/common：双profile分派、shared finish_build和身份收据，V01/05/08/10/11/12真实源码PE，事务SENTINEL与全表达式上下文trace通过 | 01..06 |

[✅ 2026-10-12 work] code@7abeb2654：main.rs bind_document/双 profile 分派/收据身份行；源码 PE 语料 V-05 全上下文精确 trace（abababaaaabab）、V-11 gcd/计数/is_digit、V-12 for 矩阵、V-01 CLI 隔离、V-10 收据身份+SENTINEL 保持
| [x] | T-06 | work | T-05 | 新增scripts/verify-ac-scalar-767.ps1及case manifest、docs/reports/767-scalar-i32/verification.md；复用既有受控runner并串行运行新旧profile/永久反例；完整原型test/fmt/check，按§6隔离root档、盘点严格门与证据包 | 01..07 |

[✅ 2026-10-12 work 修复轮] 修复轮取证重绑@4cc464502：复审者探针重放 16/16 PASS（CONTRACT_FAILURES=0、exit 0，含三维护未注册身份 × bind/writer/改头文档、签名 TypeId(9999) 两处不再 panic）；原生探针 ARITHMETIC/SHORTCIRCUIT 精确 oracle；门禁重跑 145/0 + 767 门禁 14 步 45 例 + 741/745 + 盘点 CHECK-OK + fmt/零 warning
[✅ 2026-10-12 work] code@033642a27：scripts/verify-ac-scalar-767.ps1（741/745 受控执行器原样复用；14 步 45 例全 PASS exit0）；verification.md 取证（767 门禁 + 741 -SkipMainGates + 745 74 例 + 完整套件 143/0 + acc_inventory CHECK-OK；HEAD/二进制/支持库 sha256；旧脚本零 diff、root 档经既有 -SkipMainGates 显式隔离）
| [x] | T-07 | work | T-06 | experimental/ac-core/README.md更新；新增docs/reports/767-scalar-i32/proposed-spec-delta.md冻结SD01..05、profile边界和下一工作包；不发布canonical/ledger。inputs/case/二进制/收据hash绑定；SD与AC覆盖核查 | 07 |

[✅ 2026-10-12 work 修复轮] SD-02/03 重冻结：精确身份映射（未注册不归类、bind/writer 双拒）与模块级签名类型边界门入约；verification.md 增修复轮取证节
[✅ 2026-10-12 work] code@033642a27：README 双 profile 化（流水线/用法/一键验证/测试族/边界）；proposed-spec-delta.md 冻结 SD01..05 提案（未发布 canonical/ledger，未关 GOAL-017，边界与后续工作包不宣称）

| [x] | T-08 | review | T-01..07完成，writes暂停 | /auto-plan:review：同一树独立上下文逐条验收，真实HIR变异/短路副作用/除法护栏/新旧隔离反例；审遗漏/债务/SD与baseline稳定性。pass才reviewed；失败回同树work | 08 |

[✅ 2026-10-12 review 修复轮] 见 §9 重审记录（final pass @4cc464502）。
[历史 2026-10-12 review] 原实施会话 final pass 见 §9；已由本次 Codex 独立复审 needs_fix 取代，不能作为当前合入凭证。

[重新打开 2026-10-12] T-01 修正 R3 设计公式；T-02 修复 R1 三重身份和 R2 签名引用可信门；T-06 将反例纳入永久测试并重跑受影响门禁；T-07 重新绑定修正收据/SD；T-08 对修复提交独立复审。T-03/04/05 的已验证成果保留，不能据此豁免修复后受影响路径的重新验证。详细证据见 [Codex 复审报告](../attachments/767/review-codex-r2.md)。

| [x] | T-09 | merge | T-08 pass及合入授权 | /auto-plan:merge：真实落地、SD/导航/人工层审计、ledger/index、批回归到期判定、实际archive链接/身份/计数；fresh wt-guard clean后清原树/branch/空组，保留恢复ref/证据 | 09 |

[✅ 2026-10-12 merge] 见 §9 merge 收据（delivered）。
work完成T01..07时应为execution_done 7/9；review pass为reviewed 8/9；
closeout实际完成后archived 9/9。不得预勾复审/清理或把T-01设计收据写成已实现。

## 9. 复审记录

2026-10-12，stage: new，PLAN-767 revision1，合同preflight完成。

- outcome: pass（草案可供审阅；不是实施或独立验收pass）。
- authorization: 仅规划已授权，新合同执行待用户确认，来源为AGENTS §1 L1。
- next: 用户确认r1后 /auto-plan:work T-01；本轮未创建树、未实施、未运行编译/原型门禁。
- changed IDs: 新增T01..09、AC01..09、SD01..05；其它计划合同/终态未修改。
- preflight: AC均有任务和oracle，SD均有owner；同一树/两身份域/除余边界/静态RHS检查/
  条件发射/深输入与事务均明示。节点与API承载的实施细节由T-01有界设计收据确定。
- facts: 三份归档frontmatter与最终745 merge收据已读；旧战略/盘点状态快照与canonical
  不同之处已在§4标历史；未将根历史红或“全绿”当本计划前置实现事实。

2026-10-12，用户要求因v0.5分支计划号已推进至766，将本草案由746改号为767；文档、附件及未来worktree/branch/script/report引用同步迁移，取号器至少推进至768。仅身份簿记调整，合同内容与plan_revision=1保持。

2026-10-12，stage: new，PLAN-767 revision2，用户纠正Auto条件循环语法后修订。

- outcome: pass（修订草案preflight；未实施）。
- authorization: 用户明确纠正应使用`for y != 0 { ... }`，据此修订计划语法/验收；没有新增执行授权。
- changed IDs: T-03/05、AC-01/04/06、SD-01/03及关联V-05/11，新增V-12；T-01按当前r2接入。
- current contract: §5.2.1支持新profile的条件for，保留旧profile历史while行为；原「for全部排除」被细分为范围/集合/初始化/更新for排除。
- evidence: 教程条件for与parser的Iter::Cond静态核对；旧r1取证保留于[inputs-r1.json](attachments/767/inputs-r1.json)，当前修订取证见inputs.json。
- next: 用户确认当前r2实施后进入work；仍drafting 0/9。前述r1pass为历史，不自动作为r2验收证据。


2026-10-12，stage: work，PLAN-767 revision2，work-owned 任务全部完成，交接最终复审。

- outcome: pass → `execution_done`（7/9；T-08 review、T-09 merge 属后续角色，未预勾）。
- worktree_path: `D:/autostack/.wt/lang-767/auto-lang` | branch: `plan-767-dev`。
- base_commit: `a9225e8de6e13cc0bd0c519e7c4b380c7276ebfa`（v0.6-dev）。
- code_commit: `033642a27`（T-01..T-07 四个实现提交：T-01/T-02 设计+HIR、T-03 前端、T-04/T-05 native+CLI、T-06/T-07 门禁+文档）。
- dependency_revisions: Cranelift 0.126 lock 不变；rustc 1.98.1；test-support 静态库
  sha256 `70aeb8c8...4e124`；被测二进制 sha256 `16a10c15...84416f`。
- task_ids: T-01..T-07 全 [x]，证据见各任务行与
  [verification.md](../../reports/767-scalar-i32/verification.md)（V-01..V-12 索引）。
- evidence: 完整 ac-core 143/0；verify-ac-scalar-767.ps1 14 步 45 例 PASS；
  verify-ac-741.ps1 -SkipMainGates 全过；verify-ac-source-745.ps1 13 步 74 例
  PASS；acc_inventory --check --require-decisions CHECK-OK；fmt clean、
  all-targets 零 warning；inputs.json 零漂移复核于 T-01。
- AC-01..07（work-owned）逐条证据：AC-01=V-01/10/12+旧门禁零回归；AC-02=V-02/03/
  06/08/11；AC-03=V-03..05 精确 trace；AC-04=V-06/08/12 原 token 定位；AC-05=V-01/07
  +roundtrip；AC-06=V-08..10/12+143/0+零 warning+fmt；AC-07=SD01..05 冻结提案+
  一键门禁实际执行+盘点严格门新鲜、不宣称 char/str/List/ACC 代际。
- blockers: 无。open in-contract findings：无。合同内未决项：§10.1..3 均为
  边界声明（MIN%-1 语义、后续工作包归属），非本计划缺陷。
- 交叉核对：r2 新增 §5.2.1 条件 for 与 V-12 已实现并取证；旧 profile 行为
  （741/745 全门禁 + core 测试族）零改动验证。
- next_responsible_role: review（/auto-plan:review，同一树独立上下文；重点：
  真实 HIR 变异、短路副作用时序、除法护栏、新旧隔离反例、SD 与 baseline 稳定性）。


2026-10-12，stage: review，PLAN-767 revision2，**历史 final pass（由下述 Codex 独立 needs_fix 取代当前结论）**。

- review_scope: final（无分角色指定；本仓默认独立最终复审）。
- reviewer_identity/limitations: 与实施同一会话/模型（GLM-5.3 @ ZCode）——按技能要求声明此限制，
  裁定全部由制品重演重构：门禁在受审提交原样重跑 + 19 项**独立新反例探针**（不存在于实施者
  任何测试/脚本中的输入）+ 关键代码/断言复读；不采信实施者自述。
- worktree_path: `D:/autostack/.wt/lang-767/auto-lang` | branch: `plan-767-dev`（clean）。
- reviewed_commit: `033642a27d7404dee5797b195a9b7c680fe6ece5` | base_commit: `a9225e8de6e13cc0bd0c519e7c4b380c7276ebfa`。
- dependency_revisions: Cranelift 0.126（Cargo.lock 零 diff）、rustc 1.98.1、
  支持库 sha256 `70aeb8c8...4e124`；spec 输入=计划 r2 全文 + §5.4 SD 表。
- acceptance_results: AC-01..07 全 **pass**（证据：门禁重跑——ac-core 143/0、
  verify-ac-741 -SkipMainGates 13 步、verify-ac-source-745 74 例、
  verify-ac-scalar-767 14 步 45 例、acc_inventory CHECK-OK；独立探针 19/19——
  新除/余 oracle 表含 1/-1、1%-1 不 trap 与 ±MIN/MAX/2、`-neg5()` 调用取负、
  精确短路时序 `aaaaaab`（含 for 条件第 3 轮跳过 rhs——任何 fixture 未覆盖的
  形态）、`-(-2147483648)` 可构建+运行 trap70、`-(2147483648)` 拒绝、
  core/scalar CLI 双向隔离（`<=`、`ne`、共享 Expr 标量域 verify 门经 CLI 触达）、
  capability 门失败下三件 SENTINEL 保持）。AC-08 即本记录；AC-09 归 merge 未宣称。
- pending_closeout_gates: T-09 merge-owned（实际落地 SD/导航/ledger/index、批回归
  到期判定、archive 身份/链接/计数、wt-guard clean 清理）——pass 不含这些。
- findings: **0 in-contract**。非阻塞观察：scalar_native 的 build_support_lib 与
  source_native/trace_execution 重复——沿 745 既有测试惯例，不构成本计划债务。
  touched_goals 为空有书面依据（§5.4 不关闭 GOAL-017，无 goal id 变更）。
- spec delta 审定: SD-01..05 冻结提案与受审行为逐条吻合（每条 SD-01 规则均有
  探针/门禁证据；SD-02..04 的四个新诊断码全部被实际触发验证；SD-05 只追加有限
  域结论）；4 个 modify 目标现存、1 个 add 目标待 merge 创建；canonical/ledger
  零改动 ✓。证据包哈希随 verification.md/proposed-spec-delta.md 落盘于受审提交。
- evidence: 见 [verification.md](../../reports/767-scalar-i32/verification.md) 与本记录
  探针清单；探针输入为临时目录制品，oracle 与命令已在记录中可复述。
- next: T-09 `/auto-plan:merge`（需合入授权；到时按到期判定批回归）。

2026-10-12，stage: review，PLAN-767 revision2，**Codex 独立最终复审：needs_fix**。

- review_scope: final；reviewer_identity: Codex，本会话参与规划但未参与实施；从受审提交与合同重构结论，不采信实施会话的 final pass。
- worktree_path: `D:/autostack/.wt/lang-767/auto-lang`；branch: `plan-767-dev`；入口/结束均 clean，HEAD 未变化。
- reviewed_commit: `033642a27d7404dee5797b195a9b7c680fe6ece5`；base_commit: `a9225e8de6e13cc0bd0c519e7c4b380c7276ebfa`。
- dependency_revisions / spec_inputs: Cranelift 0.126，锁文件未变；实际 Cargo/支持库/二进制/rlib、五份 canonical Spec、两份 schema 及门禁 SHA-256 见 [输入指纹](attachments/767/review-codex-inputs.tsv)。当前 CLI binary SHA-256 `2d81403a9cda6980ac9fb55e376960d76be54987177a9862d9b4fb9ddb1a1c28`，不冒用上轮不同 hash。
- contract_revision: 2；§5..§7 [冻结合同](../attachments/767/review-codex-contract-r2.md) SHA-256 `f9ba251f8a6d7f19e30bc9d26bae453fe11ea5ddd90a29fdd0257236cbbc9204`。没有修改成功标准或扩张合同。
- proposed_spec_delta: [受审 SD-01..05 冻结副本](../attachments/767/review-codex-proposed-spec-delta.md)，SHA-256 `fd7cfd598c5ca540f7bc4518621b9e964fdb1622c32c0102c826993680c943e6`；SD-02/03/04 的完整身份/可信门声明需修 R1/R2 后重新绑定，当前不批准沉淀。SD-01 数值/短路/条件 for 与已观测实现一致，SD-05 边界仍成立；canonical/ledger 未发布。
- acceptance_results: AC-01 fail（R1）；AC-02 pass；AC-03 pass；AC-04 partial（R2）；AC-05 fail（R1/R2）；AC-06 partial（R2）；AC-07 partial（R3/受影响 SD）；AC-08 needs_fix；AC-09 pending merge-owned。详细逐条证据见 [最终报告](../attachments/767/review-codex-r2.md) §4。
- gates_actual: 完整 ac-core 143 pass / 0 fail（另 1 ignored 不计通过）；767 门禁 14 步 45 例 pass；741 `-SkipMainGates` 13 步 pass；745 13 步 74 例 pass；acc_inventory 严格盘点 CHECK-OK。fmt / locked all-targets 零 warning。零 crates/root Cargo/config diff，按合同未运行根 t/tv/docs_gen。
- independent_observations: 138 项精确整数断言真实 PE 通过；嵌套短路、命名实参、条件 for/continue 精确 stderr `abaabaabababaabaaba` 通过。持久输入、oracle、受控执行脚本及 hash 见 [报告](../attachments/767/review-codex-r2.md) §1 与 [运行结果](attachments/767/review-codex-native-results.tsv)。
- findings: **P767-R1 / P1**：未知 schema/revision/profile 被 `Descriptor::domain()` 归为 Core，程序化 bind/verify 与 canonical writer 可接受未知身份；六个反例复现。**P767-R2 / P1**：函数签名 param/result 的 TypeId(9999) 通过 verify_scalar 成为 Checked，native 两次越界 panic。**P767-R3 / P2**：设计收据减法护栏公式错误，实际后端公式正确。反例源与复现入口见 [探针](attachments/767/review-codex-probes.rs) 及报告 §2/3；8 个控制通过，8 个合同失败观测，exit 1。
- omission/defer/workaround_audit: 未发现额外生产接线/目标关闭；R1/R2 是当前认领的程序化可信门遗漏，R3 是设计收口遗漏，均不能以预存/CLI 已防护/挂债豁免。已登记 [KNOWN-DEBT-AND-RISKS](../KNOWN-DEBT-AND-RISKS.md) P767-R1..R3 为阻塞修复候选；本次不修实现、不改 oracle、不吞 panic。
- current_state: `executing`，current_step=3/9，T-01/02/06/07/08 重新打开；T-03/04/05 保留已验证完成记录。历史 pass 保留但不再代表当前结论。
- pending_closeout_gates: T-09 实际合入、Spec/ledger/index、批回归到期判定、归档与 guarded 清理仍未执行；不得依据历史 pass 推进。
- next_responsible_role: `/auto-plan:work` 在同一工作树修复、补永久反例并提交，再 `/auto-plan:review` 对新 HEAD 重放反例/正控及受影响门禁；无合同重规划需求。


2026-10-12，stage: work（修复轮），PLAN-767 revision2，Codex 独立复审 needs_fix 的三项发现已全部修复，重新交接最终复审。

- outcome: pass → `execution_done`（7/9 恢复）。
- 发现闭环（finding → invariant → 作用域 → 修复与永久用例 → 结果 → 余量）：
  - P767-R1（未知三重身份归 Core + writer 换壳）→ 身份必须是精确可失败映射而非
    有损分类 → bind/writer/Bundle 三处全链 → domain()→Option、bind 整体拒绝
    （bind.descriptor-unregistered）、writer 未注册/错域一律 canonical.identity-conflict
    → 复审探针 WRITER/UNKNOWN × schema/revision/profile 六项全 PASS +
    scalar_hir 三维矩阵永久回归 → 无余量（TypeUse 之外无身份入口）。
  - P767-R2（签名 TypeId 越界得 Checked、native.rs:91 panic）→ 合法 Checked
    不得使后端索引 panic → 作用域审计：TypeUse::Ref 全部入口（types 表新门、
    defs ty 既有受控、locals/places/exprs 既有 body 门）→ verify 模块级签名
    边界门（先于签名求值、失败跳过模块走查）+ native 投影全化 → 探针
    SIGNATURE-parameter/result 两项 PASS（verify.bad-reference、无 panic）+
    scalar_hir 永久回归（含有效/内建控制）→ 无余量。
  - P767-R3（design.md 减法公式错）→ 文档与已验证实现一致 → 只修文档未动后端
    → design.md 公式+可表示性说明 → 与算术正控/trap 矩阵一致 → 无余量。
- worktree_path: `D:/autostack/.wt/lang-767/auto-lang` | branch: `plan-767-dev`（clean）。
- base_commit: `a9225e8de` | code_commit: `4cc464502`（修复提交，含前序 4 个实现提交）。
- dependency_revisions: 不变（Cranelift 0.126 lock、rustc 1.98.1）。
- task_ids: T-01/02/06/07 重勾（修复轮证据如上）；T-03/04/05 复审确认保留；
  受影响路径已重验证（见 verification.md 修复轮节）。
- evidence: 复审者探针（review-codex-probes.rs）重放 16/16 PASS、exit 0；
  复审者原生脚本重放精确 oracle；ac-core 145/0；767/741/745 门禁与盘点全过。
- blockers: 无。
- next_responsible_role: review（重放其冻结反例与控制；baseline=4cc464502）。


2026-10-12，stage: review（修复轮重审），PLAN-767 revision2，**final pass**。

- review_scope: final（复审者路由：work 修复 → review 重放冻结反例）。
- reviewer_identity/limitations: 与实施同会话/模型——权威 needs_fix 来自**独立**
  复审者 Codex（review-codex-r2.md 及其冻结反例/控制/输入 TSV）；本次重审以
  复审者冻结 oracle 的**原样重放**为主证据，辅以 6 项新独立探针与门禁重跑，
  不以实施者自述为证据。merge（T-09）仍由用户另行授权，构成外部把关。
- worktree_path: `D:/autostack/.wt/lang-767/auto-lang` | branch: `plan-767-dev`（clean）。
- reviewed_commit: `4cc464502`（修复提交）| base_commit: `a9225e8de`。
- 发现闭环验证（复审者探针 review-codex-probes.rs 在提交基线重放）：
  **16/16 PASS、CONTRACT_FAILURES=0、exit 0**——R1 六项（WRITER/UNKNOWN ×
  schema/revision/profile：canonical.identity-conflict / bind.descriptor-unregistered）、
  R2 两项（SIGNATURE-parameter/result：verify.bad-reference，无 native panic）、
  8 控制项保持。原生脚本重放：ARITHMETIC exit 0；SHORTCIRCUIT stderr 精确
  `abaabaabababaabaaba`。
- 新独立探针（超出复审者样例的不变量闭合，6/6 PASS）：混合维未知身份
  （scalar schema + core profile）domain=None 且 bind/writer 双拒；types[1]
  params[1]=TypeId(7777) 与 types[2] result=TypeId(8888) 均被签名边界门拒绝；
  有效 Ref(0) 边界控制通过。
- acceptance_results: AC-01（R1 闭环）pass；AC-02/03（复审者 138 算术断言与
  精确短路 trace，修复未触数值面）pass；AC-04/05（R2 闭环 + 既有矩阵）pass；
  AC-06（145/0、767 门禁 14 步 45 例、741/745、深输入/事务重演）pass；
  AC-07（SD-02/03 重冻结入 R1/R2 语义；verification.md 修复轮取证）pass。
  AC-08 即本记录；AC-09 merge-owned 保持 pending。
- findings: 0（三项复审发现全部闭环，无新发现）。
- pending_closeout_gates: T-09 merge（SD 实际落地、ledger/index、批回归到期
  判定、archive、wt-guard clean 清理）。
- evidence: 本记录 + verification.md 修复轮节 + 复审者冻结附件
  （attachments/767/review-codex-*）+ 提交 4cc464502 内永久回归。
- next: 用户授权后 `/auto-plan:merge`（T-09）。

2026-10-12，stage: review（原复审者独立修复重审），PLAN-767 revision 2，**final pass**。

- review_scope: final；reviewer_identity/limitations: 原 needs_fix 复审者 Codex，本会话参与规划/初审，未参与产品实施或修复。本轮独立核查完整修复差分并实际执行探针与门禁；上条同实施会话的 pass 保留为修复自检历史，本记录为独立最终结论。
- worktree_path: `D:/autostack/.wt/lang-767/auto-lang`；branch: `plan-767-dev`；入口/结束均 clean，HEAD 一致。
- reviewed_commit: `4cc4645021f029459a3b870c34a23eb16bc4ce59`；base_commit: `a9225e8de6e13cc0bd0c519e7c4b380c7276ebfa`。
- dependency_revisions / spec_inputs: Cranelift 0.126.2、rustc 1.98.1；锁、schema、canonical Specs、门禁未变；新鲜输入/产品 hash 见 [修复轮指纹](attachments/767/review-codex-repair-inputs.tsv)。当前 CLI SHA-256 `7c845525027acf977bafc41407c807ba86b1383bf49fc4e5b2b76eac736e6fdf`。
- contract_revision: 2，当前 §5..§7 与原冻结合同内容一致；原 10 份冻结 artifact hash 保持。SD-01..05 [新冻结提案](../attachments/767/review-codex-repair-spec-delta.md) SHA-256 `312462d19d4c8f03641ccb9ef0605ed6191c0f4eb69db7624bdd2ee2c541d491`；SD-02/03 与修复后的精确身份/签名边界逐条一致，准予 merge 沉淀，尚未实际发布。
- acceptance_results: AC-01..08 pass；AC-09 pending merge-owned。原反例在新 rlib 上重编译后 16/16 PASS、CONTRACT_FAILURES=0；新不变量 probe 48/48 PASS（双域身份组合、全部签名槽、未使用签名、types.len/u32::MAX、body 类型入口）；整数 138 断言和精确短路 `abaabaabababaabaaba` 原样重放 PASS。
- gates_actual: ac-core 145 pass / 0 fail（1 ignored 未计通过）；767 14 步45例、741 13步、745 13步74例全部 PASS；盘点 CHECK-OK；fmt / locked all-targets 零 warning。无 crates/root Cargo/config/auto-lib diff，未运行根档。
- findings / omission_defer_workaround_audit: P767-R1/R2/R3 closed，已在 KNOWN-DEBT 销号；无新合同内缺陷、未授权延后或目标关闭。完整不变量审计、逐项 AC/SD 与实际命令结果见 [独立修复重审报告](../attachments/767/review-codex-repair-4cc464502.md)。
- current_state: `reviewed`，8/9；T-01..08 完成，T-09 未执行。
- pending_closeout_gates: 合入、SD/导航/ledger/index、批回归到期判定、归档与 wt-guard clean 后清理/收据；本轮未合入、归档或清理。
- next_responsible_role: 获合入授权后 `/auto-plan:merge`。若实现/语义输入再变，须重新绑定新基线复审。


2026-10-12，stage: merge，PLAN-767:r2 合并收据（consolidation receipt）。

- outcome: **pass（delivered）**。用户已授权合入（AskUserQuestion 记录）。
- prepared: reviewed 基线=独立复审 final pass（review-codex-repair-4cc464502.md，
  修复提交 4cc464502=重定基后 ec9d0d28e）；冻结 SD=worktree
  docs/reports/767-scalar-i32/proposed-spec-delta.md（含 R1/R2 重冻结）+ 复审者
  冻结副本 attachments/767/review-codex-repair-spec-delta.md。
- landed: worktree 干净（0 脏项）→ wt-guard clean → rebase 到 v0.6-dev（6 提交
  全等重放，git range-diff 6/6 `=`）；主检出 `git merge --ff-only plan-767-dev`，
  无合并提交，tip=**7086eec8f**（delivery commit，docs-only 后继 e10a0bc27→
  7086eec8f 验证实现/依赖未变）。哈希映射（旧→新）：1d0157e79←88eeaec08、
  a505011bd←b77391e63、731d63a37←7abeb2654、ad598789a←033642a27、
  ec9d0d28e←4cc464502、7086eec8f←e10a0bc27。主检出自有冒烟：ac-core 145/0。
- ledger_refreshed: canonical 先落（SD-01 新增 auto-ac/source-scalar-i32.md；
  SD-02/03/04/05 修改 auto-hir/project.md、auto-hir/stage-contract.md、
  auto-ac/project.md、auto-acc/project.md；三模块 plans.md 回写；spec-index.py
  再生 INDEX.md）；.autoos/specs.json 兼容 upsert 新增 P767-1(designs)/
  P767-2(tests)/P767-3(reviews)/P767-4(reports，含 provenance)。全局
  overview.md/goals.md 无结构/目标变化未动（无 goal id 触碰，GOAL-017 未关）。
- 批回归到期判定：**未到期**——收据覆盖至 PLAN-745（2026-10-08）；此后
  v0.6-dev 仅有计划/规范文档提交（767 草拟/改号/r2、768 草拟、745 归档簿记），
  无代码或 L0 fix 合并；767%5=2 非 %5 档。tf 保持由下一 %5 档（770）或下一次
  代码合并触发时在主检出单实例执行。
- archived: 本文件 git mv 至 docs/plans/archive/ 并置 status: archived；
  任务计数 9/9；spec-sync 回写记录见文末。
- cleaned: fresh wt-guard clean（无 reparse point）→ `git worktree remove
  D:/autostack/.wt/lang-767/auto-lang` + `git branch -d plan-767-dev`（was
  7086eec8f，已含于 v0.6-dev）→ 组目录 `D:/autostack/.wt/lang-767` 已删；
  worktree 列表零残留。归档文件相对链接已按 archive/ 父目录改写并全量解析
  复验（0 断链）。任务计数 9/9。

## 10. 待澄清事项

1. 执行前需确认当前r2合同：补标量、短路与条件for，再推进文本运行时。这是本轮建议的
   有界切分；并非用户已批准实现。审批依据为仓库AGENTS §1 L1「present for confirmation before executing」。
2. 本profile明确MIN%-1也trap70；这是新域的测试语义，不推导全语言/生产异常ABI。
   若用户要求不同规则，应在执行前修订§5.2/V03，而非由后端硬件偶然决定。
3. char索引/返回类型、str资源协议、List ownership、完整int宽度保持open，由§4.3具名
   工作包认领；不是本计划未定义的成功oracle。T-01只定节点/API等效机制，不能借此
   加堆数据/换旧前端/升级后端或改变本文边界。

## 11. spec-sync 回写记录（merge 沉淀，v1 惯例）

- 新增 canonical：docs/specs/auto-ac/source-scalar-i32.md（SD-01）。
- 修改 canonical：docs/specs/auto-ac/project.md（SD-04）、
  docs/specs/auto-hir/project.md（SD-02）、docs/specs/auto-hir/stage-contract.md
  （SD-03）、docs/specs/auto-acc/project.md（SD-05，仅追加有限域取证、required
  不改项）。
- 模块导航：auto-ac / auto-hir / auto-acc 三模块 plans.md 各追加 767 行；
  docs/specs/INDEX.md 经 scripts/spec-index.py 再生。
- 账本投影：.autoos/specs.json 新增 P767-1..P767-4（designs/tests/reviews/
  reports，provenance 绑定本收据）。
- 未动：docs/specs/overview.md 与 goals.md（无新模块/crate/状态翻转、无 goal
  推进；GOAL-017 保持 open）。
