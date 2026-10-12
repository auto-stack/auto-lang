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

