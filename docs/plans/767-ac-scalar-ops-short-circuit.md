---
plan_id: PLAN-767
status: drafting
feature_name: AC 标量计算扩面：精确整数运算与布尔短路
plan_revision: 1
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12
current_step: 0
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
`&& / || / !`。旧 core-i32 profile 保持原支持/拒绝和数值语义。

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
用户泛型、for/use/全局变量、生产 ABI/DLL/热重载、ACC 主体迁移/桥、通用优化 pass、
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
| [741](archive/741-ac-hir-native-core.md)，archived 44/44 | Atom HIR → verify → Cranelift COFF/PE，可信门/工具事务强化 | 源码前端及完整语言面 |
| [743](archive/743-acc-bootstrap-hir-contract.md)，archived 38/38 | ACC 主体/能力清单、阶段/桥/代际合同及严格盘点工具 | 目标态合同所描述的 ACC 实现 |
| [745](archive/745-ac-source-checked-hir-native.md)，archived 9/9 | 单模块 core-i32 源码 → Checked → PE，74例及独立深输入/帧控制 | 运算扩面、堆数据及 ACC 主体 |

当前 canonical Specs 优先于旧战略的实现状态快照：
[全局总览](../specs/overview.md)、[AC](../specs/auto-ac/project.md)、
[source-core-i32](../specs/auto-ac/source-core-i32.md)、
[HIR](../specs/auto-hir/project.md)、[stage-contract](../specs/auto-hir/stage-contract.md)、
[ACC](../specs/auto-acc/project.md)。战略中的 2026-10-04 队列和 743 报告的
「源码 adapter 未实现」保留为历史，不能用于否认 745 已完成。

### 4.2 为什么先做标量子合同

[743 候选②](../reports/743-acc-hir-contract/next-work-packages.md) 原包括
str/char/List 和整数/布尔运算，适合拆成独立实施合同，不能一次冻结全部资源/布局规则。
[str 最高优先与运算需求](../reports/743-acc-hir-contract/inventory.md)是同一关键路径：
`auto/lib/lexer.at::is_digit/is_alpha/is_alnum/is_hex_digit` 已使用 `>= / <= / == / && / ||`，
游标更新、切片端点和 parser 条件还需要减法及逻辑否定。
当前 `hir.rs::BinOp` 只有 AddI32/MulI32/LtI32，source AST 同样仅三种二元操作，
`main.rs` 只接受一个 SOURCE_PROFILE。因此先建立比较/短路与严格 profile 扩面机制，
再引入需要所有权和字符索引决策的堆数据，是一个可独立复审的有界切口。

[str/char 语义冲突](../reports/745-source-core-i32/numeric-boundary.md)依然存在：
VM builtin 的 char_at 返回 str、auto-lib 的表面返回 int、Value 保存 Unicode char。
本计划不借补比较操作裁定这些冲突；完整 int 宽度也保持 unknown。
取证固定于 [inputs.json](attachments/767/inputs.json) 的完整 SHA256 与主检出 HEAD；
执行开始复核漂移，必要时记录差异，而不是直接沿用旧 hash 宣称新鲜。

### 4.3 战略对应及后续队列

依据[后端演进 §3/§7.3](../design/strategy/auto-native-backend-evolution.md)、
[AC 子集 §2/§3](../design/strategy/auto-ac-subset-and-target.md)、
[自举合同](../design/strategy/auto-acc-bootstrap-contract.md)和
[HIR 求值顺序](../design/strategy/auto-hir-design.md)。v0.6 继续使用 Cranelift。

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
| SD-01 | add | docs/specs/auto-ac/source-scalar-i32.md | 仅core-i32源码→新增显式scalar-i32运算/优先级/trap/短路/拒绝域 | 定义新profile而非改写旧语义 | AC-01..04,06 |
| SD-02 | modify | docs/specs/auto-hir/project.md | 单descriptor/三op→双身份域、unary/lazy运算及verify边界 | 共享公共HIR、隔离CLIF与身份越权 | AC-01,04,05 |
| SD-03 | modify | docs/specs/auto-hir/stage-contract.md | S0–S6仅core-i32已实现→追加新域与身份随行、条件求值规则 | 区分阶段事实与pass目标态 | AC-01,03..05 |
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
| V-05 | AC-03 左侧唯一及嵌套 | lhs标a、rhs标b；组合于return/let/assign/while/call/named arg，循环至少两轮 | 按真值路径精确a或ab；每轮重算，命名绑定不重排trace；bool返回/传参仍0/1 | 不只顶层if路径 |
| V-06 | AC-02/04 比较与拒绝 | MIN/MAX/负数全序，bool四组eq/ne、优先级；混合bool/int、!1、-true、true||unknown()、false&&(1+2) | 正向exit0；反向bind/verify的源码原token诊断，未执行RHS也查名/类型 | 静态合法性≠运行期执行 |
| V-07 | AC-05 可信门 | 真实bound Bundle变异成错类型unary/lazy、旧身份新增op、共享Expr、自环/双环/跨body引用 | 本进程verify受控拒绝，无Checked/native产物；输出后重verify不能沿用旧凭证 | 不只文本parser负例 |
| V-08 | AC-04/06 parser帧 | a-1/负literal、二元实参逗号、跨行/CRLF、缺操作数/缺右括号、混合优先级 | 正向实际PE正确；畸形source.syntax，不掉调用/尾部、不panic | 新操作符与745原帧规则 |
| V-09 | AC-06 深输入 | 16384项逻辑链、20000层!、4096层逻辑/调用混合及缺闭合反例；保留745深块族 | 合法到Checked（含Drop），畸形受控拒绝；小型混合1000层至少一例真实PE正确 | 不承诺任意规模性能/无限内存 |
| V-10 | AC-01/06 工具事务 | 新profile成功收据；旧成功三件SENTINEL后新source/HIR拒绝、故障注入link/publish | 身份/hash同事务；失败保持exe/obj/收据原hash，执行器全出口回收、沿用显式残留诊断 | 继承741/745，不重造事务执行器 |
| V-11 | AC-07 代表计算 | Euclid gcd(48,18)=6；十进制计数12345=5；lexer is_digit的i32码点比较改写 | 源码→PE退出0，固定独立值断言/区间正反边界；明确非原lexer编译 | char/str/enum/List仍缺 |

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
| AC-01 | work | 双profile显式身份、绑定/Checked/canonical/native边界成立；旧支持/拒绝保留；V-01/10与741/745门禁通过 |
| AC-02 | work | §5.2整数运算/全序/eq-ne/优先级及数值边界经源码和直接HIR实际PE验证；V-02/03/06/08/11通过 |
| AC-03 | work | 短路在所有声明表达式上下文保持效果/调用次序、左侧一次、RHS条件执行及运行期trap；V-03..05精确trace通过 |
| AC-04 | work | 名字/类型/语法/来源映射在新域成立，包括未执行RHS静态拒绝；V-06/08每项有阶段/code/原token断言 |
| AC-05 | work | 新运算直接HIR及程序化Bundle同受verify、归属/无环/旧域限制，未验不进后端；V-01/07与roundtrip通过 |
| AC-06 | work | 深输入/旧调用帧/事务/进程回收不退化，V-08..10及旧永久反例通过；完整原型测试/locked all-targets零warning/fmt成功 |
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
| [ ] | T-01 | work | 用户批准r1；741/743/745落地 | 确认同一树/branch/base/inputs、零生产diff；新增docs/reports/767-scalar-i32/design.md：节点字段、profile随行API/注册选择、旧verify兼容、writer身份冲突和诊断码及case清单。静态preflight逐项§5/矩阵，禁止改可观察规则；HEAD/hash记录 | 01..07 |
| [ ] | T-02 | work | T-01 | src/hir.rs、descriptor.rs、verify.rs、lib.rs及新增schema/scalar-i32.atom、tests/scalar_hir.rs、fixtures/scalar-hir/：双profile+新op/节点+真verify，roundtrip与V01/07/类型正反例；locked all-targets与scalar_hir通过 | 01,04,05 |
| [ ] | T-03 | work | T-02 | src/source/{lexer,ast,parser,resolve,typecheck,adapter,mod}.rs：显式profile、运算/优先级、迭代帧和span投影；新增tests/scalar_source.rs、fixtures/scalar-source/。V06/08/09通过；所有新源码仍经Atom bind/verify | 01,02,04..06 |
| [ ] | T-04 | work | T-02 | src/native.rs：eager算术/比较/unary、div/rem护栏、lazy CFG与规范bool；新增tests/scalar_native.rs的真实HIR PE及trace。V02..05/07正反控制通过，不复用宿主算术作被测执行 | 02,03,05,06 |
| [ ] | T-05 | work | T-03/04 | src/main.rs、source/adapter.rs与既有source_cli/source_native/common：双profile分派、shared finish_build和身份收据，V01/05/08/10/11真实源码PE，事务SENTINEL与全表达式上下文trace通过 | 01..06 |
| [ ] | T-06 | work | T-05 | 新增scripts/verify-ac-scalar-767.ps1及case manifest、docs/reports/767-scalar-i32/verification.md；复用既有受控runner并串行运行新旧profile/永久反例；完整原型test/fmt/check，按§6隔离root档、盘点严格门与证据包 | 01..07 |
| [ ] | T-07 | work | T-06 | experimental/ac-core/README.md更新；新增docs/reports/767-scalar-i32/proposed-spec-delta.md冻结SD01..05、profile边界和下一工作包；不发布canonical/ledger。inputs/case/二进制/收据hash绑定；SD与AC覆盖核查 | 07 |
| [ ] | T-08 | review | T-01..07完成，writes暂停 | /auto-plan:review：同一树独立上下文逐条验收，真实HIR变异/短路副作用/除法护栏/新旧隔离反例；审遗漏/债务/SD与baseline稳定性。pass才reviewed；失败回同树work | 08 |
| [ ] | T-09 | merge | T-08 pass及合入授权 | /auto-plan:merge：真实落地、SD/导航/人工层审计、ledger/index、批回归到期判定、实际archive链接/身份/计数；fresh wt-guard clean后清原树/branch/空组，保留恢复ref/证据 | 09 |

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

## 10. 待澄清事项

1. 执行前需确认本新r1合同：先补标量与短路，再推进文本运行时。这是本轮建议的
   有界切分；并非用户已批准实现。审批依据为仓库AGENTS §1 L1「present for confirmation before executing」。
2. 本profile明确MIN%-1也trap70；这是新域的测试语义，不推导全语言/生产异常ABI。
   若用户要求不同规则，应在执行前修订§5.2/V03，而非由后端硬件偶然决定。
3. char索引/返回类型、str资源协议、List ownership、完整int宽度保持open，由§4.3具名
   工作包认领；不是本计划未定义的成功oracle。T-01只定节点/API等效机制，不能借此
   加堆数据/换旧前端/升级后端或改变本文边界。
