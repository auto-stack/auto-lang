# ac-core — PLAN-741 AC 首个闭环原型(745 源码 profile / 767 标量扩面 / 770 文本与字符)

独立 Cargo workspace:Schema-bound Atom HIR 文本 → 类型化绑定 → 语义校验 →
Cranelift native lowering → Windows x64 COFF 对象 → rust-lld 链接 → 运行 PE。

PLAN-745 在同一管线上新增**显式源码 profile** `auto.source.core-i32.draft`:
Auto 源码计算子集(.at)→ span 化前端 → 词法/类型解析 → 生成 Atom 文本(带
源 span 映射)→ 复用既有 binder/verifier → 同一 Checked→native→事务发布路径。
这是计算核心 profile 的第二条纵向实现:**"Auto 源码子集 → 原生执行"**,
仍不是完整 Auto 源码编译器,也不是 AAC 自举完成。

PLAN-767 再增**标量扩面 profile** `auto.source.scalar-i32.draft`(HIR 身份
`auto.hir.scalar.draft / 1 / scalar-i32-draft`,descriptor
`schema/scalar-i32.atom`):整数减/除/余(向零截断,除数 0、MIN/-1、MIN%-1
运行期 trap70)、全序比较、同类 eq/ne、一元 `-`/`!`、布尔 `&&/||` 短路
(HIR 以 lazy `Logical` 节点与 eager `Binary` 分离表示)与**条件 for**
`for <bool> { ... }`(每轮重判;本 profile 拒绝 while)。身份从 binder 经
verify(分域入口 `verify`/`verify_scalar`)、Checked、canonical writer 到
构建收据全程随行;旧 core 域的接受/拒绝集合零改动。范围与验收见
[docs/plans/767-ac-scalar-ops-short-circuit.md](../../docs/plans/767-ac-scalar-ops-short-circuit.md)。

PLAN-770 再增**文本与字符 profile** `auto.source.text.draft`(HIR 身份
`auto.hir.text.draft / 1 / text-i32-draft`,descriptor `schema/text.atom`):
标量面之上加入 `str`(借用 UTF-8 切片)/`Str`(拥有缓冲)/`char`(独立
Unicode 标量)。字符串/字符字面量(跨行、白名单转义、`\u{...}`)、
`len`/`len_bytes`/`char_at`/`slice`(按 Unicode 标量索引)、`+` 拼接、
`clone`/`.to(Str)`/`.to(str)`、`==`/`!=`(字节精确,无 normalization)、
显式 `c.to_codepoint()`/`char.from_codepoint(n)`(代理/越界为运行期
RT-FAIL+trap70)。资源沿 774 数据-资源合同:拥有/借用分语境,`.move`
调用点转移所有权,LIFO 出口清理由 verifier 的 TextResourcePlan 冻结、
后端只消费;native 侧经独立 `runtime/text` 静态库(`ac_rt_*` ABI)真实
分配/复制/释放。文本模块须声明并显式提供能力 `ac.runtime.text.v1`,
且 `--support-lib` 携带伴随 `<lib>.contract.json` 合同(api/revision/
target/hash 对真实库校验,缺/错在发布前拒绝)。范围与验收见
[docs/plans/770-ac-text-char-native.md](../../docs/plans/770-ac-text-char-native.md)。

PLAN-771 再增**record profile** `auto.source.record.draft`(HIR 身份
`auto.hir.record.draft / 1 / record-i32-draft`,774 interface-contract
§7.2 冻结;descriptor `schema/record.atom`):名义 record 类型
(`type Name { 字段 [;] ... static fn/fn 方法 }`,字段 i32/bool/嵌套
record,前向引用合法、含图必须无环)、位置构造 `Pair(1, true)` 与命名
构造 `Pair { left: 1, right: true }`(求值按**源码序**,绑定按字段名)、
字段读写路径(`p.inner.left`、`p.left = v`)、`static fn`(如 `Pair.new`)
与只读实例方法(`self`/前导 `.field`,receiver 视图借用且只求值一次)、
显式 `mut` 自由函数参数(`fn f(mut r Pair)`,调用点 `f(p.mut)`)。
HIR 增 `record_ctor`/`field_get`/`borrow_view`/`borrow_mut` 表达式、
`field_path` place、`method` 声明与参数 `mode`;verify 前门执行名义
类型、迭代含图环检测、构造完整性、逐段字段归属、借位模式与 per-call
mut 排他(`verify.mut-conflict`/`verify.ownership-required`,774 D-04)。
native 走 774 D-05 内部 ABI:字段声明序 4 对齐 size=Σ字段尺寸,ABI-D
`*const` view / ABI-E `*mut` + out-slot 返回(无聚合 RAX/sret);record
局部即存储指针,赋值=值拷贝;生产型表达式(构造/调用/字段读)带
copy-elision 直写目标存储,嵌套构造链零临时槽——R1 修复起,直写仅在
目标与 RHS 触集不相交(目标根局部不在子树触集)时转发,否则 RHS 先落
新槽再提交,`p = P { a: p.b, b: p.a }`/`p = swap(p)` 类别名自赋值保持
读-后-写正确;R2 修复起,record 值操作数(构造字段绑定的 ReadLocal)在
其自身源码序求值位即时快照,更晚实参的 `.mut` 副作用不再回溯改写已捕获
字段,借位实参(view/mut)仍传活指针。深输入预算:类型含图
1024 层、字段路径 4096 段(V-09 PE 实证)。范围与验收见
[docs/plans/771-ac-record-method-native.md](../../docs/plans/771-ac-record-method-native.md)。

## 流水线

```text
[HIR 模式:check/build]
.atom 文本
  → atom_text.rs        分词/解析,带字节 span 的语法树
  → descriptor.rs       按文档头三重身份选注册 descriptor(core/scalar)绑定
  → hir.rs              类型化 ID + Unchecked HIR + HirDomain 身份
  → verify.rs           语义校验(verify=core 域/verify_scalar=scalar 域;
                        profile-op/引用边界门先于索引走查)→ CheckedModule
  → native.rs           Cranelift lowering(算术/取负溢出、除/余护栏走
                        trap70=ExitProcess(70);短路用分支+变量合流)
  → link.rs             rust-lld COFF 链接(暂存+备份+发布,失败回滚 + 收据;
                        全子进程硬截止覆盖等待与输出收集,Job Object 管控进程树)
  → Windows PE

[源码模式:check-source/build-source,745/767/770/771]
.at 源码 + 显式 --source-profile core|scalar|text|record
  → source/lexer.rs     UTF-8/ASCII 标识符/行敏感 token(排除面 token 化;
                        text profile 下 "/'/str/Str/char/to/move/mut 进
                        上下文敏感 token,转义与 \u{} 在词法层解码校验)
  → source/parser.rs    语句型语法(core:+ <* <;scalar:||<&&<==!=<关系<+-<* / %
                        + 一元链右结合 + 条件 for;迭代优先级攀爬机;
                        text 另有方法后缀链 .len()/.char_at/.slice/.clone/
                        .to(str|Str)/.to_codepoint/.move 与 char 类型级
                        接收者 char.from_codepoint(n),mode 词 mut|move)
  → source/resolve.rs   签名预登记/块作用域/参数映射(bind 级 source.* 诊断;
                        text:接收者类型绑定内建方法,用户同名遮蔽优先,
                        字面量语境定型 str/Str,moved 绑定数据流)
  → source/typecheck.rs 精确类型检查(verify 级 source.type-mismatch;
                        text:双文本 +→concat、eq/ne 同型、无隐式 char/int)
  → source/adapter.rs   生成 Atom 文本 + span 再锚(循环→loop/if 降低、
                        bool=0<1/0<0、&&/||→Logical、-x/!x→Unary、return 后
                        死代码独立块、stray break phantom loop、条目序 span
                        投影 + UTF8 边界后验;text:文本节点/mode/moved 发射
                        与 ac.runtime.text.v1 requires)
  → 同上 descriptor/verify/native/link(无旁路、无回退;text 域 native
                        lowering 走 runtime/text 静态库的 ac_rt_* 调用与
                        TextResourcePlan 出口清理)
```

## 使用

CLI 的二进制名是 `auto-ac-prototype`("ac-probe" 为历史文档名),子命令
check/build/check-source/build-source 直接跟在 `--` 之后(与
`scripts/verify-ac-741.ps1`、`scripts/verify-ac-source-745.ps1` 一致):

```powershell
cd experimental/ace/crates/core

# 结构 + 语义检查(退出码 0 / 1 / 2)
cargo run --locked -- check ../../docs/design/strategy/hir-examples/01-add.atom

# 原生构建(零参数、返回 i32 的已声明 DefRef 作为入口)
cargo run --locked -- build `
  fixtures/native/add-2-3.atom `
  --entry d_entry --output target/ac-artifacts/add.exe

# 需要 trace 能力时必须显式声明 + 链接支持库
cargo run --locked -- build `
  ../../docs/design/strategy/hir-examples/03-call-order.atom `
  --entry d_caller --output target/ac-artifacts/trace.exe `
  --capability hir.test.trace.v1 `
  --support-lib test-support/target/debug/ac_trace_support.lib
```

构建产物(.obj/.exe/.ac-link.txt 收据)放在 `--output` 所指目录;
失败的构建不会覆盖既有成功制品(发布阶段失败自动回滚)。

源码模式(745/767;profile 必须显式给出,不按扩展名猜模式;入口是
**源码函数名**,零参数、返回 int/i32):

```powershell
cd experimental/ace/crates/core

# 源码检查(退出码 0 / 1 / 2)
cargo run --locked -- check-source fixtures/source/valid/p01-precedence.at `
  --source-profile auto.source.core-i32.draft

# 源码原生构建 + 运行(源码 profile 与输入 hash 写入 .ac-link.txt 同事务)
cargo run --locked -- build-source fixtures/source/valid/p15-named-eval-order-x2.at `
  --source-profile auto.source.core-i32.draft `
  --entry main --output target/ac-artifacts/p15.exe `
  --capability hir.test.trace.v1 `
  --support-lib test-support/target/debug/ac_trace_support.lib

# 标量扩面 profile(767):减/除/余、比较、eq/ne、一元、短路与条件 for
cargo run --locked -- check-source fixtures/scalar-source/valid/p-gcd-euclid.at `
  --source-profile auto.source.scalar-i32.draft
cargo run --locked -- build-source fixtures/scalar-source/valid/p-gcd-euclid.at `
  --source-profile auto.source.scalar-i32.draft `
  --entry main --output target/ac-artifacts/gcd.exe

# 文本与字符 profile(770):真实分配/复制/释放的 str/Str/char
# 先构建 runtime 静态库(独立 workspace,产 .lib + 合同 sidecar 由其测试维护)
cargo build --locked --release --manifest-path runtime/text/Cargo.toml -p ac-text-runtime-shim
# 直接 HIR 模式(build);源码模式用 build-source --source-profile auto.source.text.draft
cargo run --locked -- build fixtures/text-hir/valid/n01-native-ops.atom `
  --entry d_main --output target/ac-artifacts/text-hir.exe `
  --capability ac.runtime.text.v1 `
  --support-lib runtime/text/target/release/ac_text_runtime.lib
```

## 一键验证

```powershell
# HIR 模式回归(741;-SkipMainGates 跳过主仓 root 档)
powershell -NoProfile -ExecutionPolicy Bypass -File ../../scripts/verify-ac-741.ps1 -SkipMainGates
# 源码 profile 全语料(745:74 例串行,正例真实构建+运行,60s/例硬截止)
powershell -NoProfile -ExecutionPolicy Bypass -File ../../scripts/verify-ac-source-745.ps1
# 标量扩面 profile 全语料(767:11 源码 + 15 HIR 正例真实运行 + 19 负例 +
# CLI 隔离与收据身份;受控执行器与 741/745 同源)
powershell -NoProfile -ExecutionPolicy Bypass -File ../../scripts/verify-ac-scalar-767.ps1
# 文本与字符 profile(770:ac-core 全套含 text_* 五族 + runtime/text 三档 +
# 旧 gates 741/745/767 串行 + scope 隔离 + 库/合同指纹收据)
powershell -NoProfile -ExecutionPolicy Bypass -File ../../scripts/verify-ac-text-770.ps1
```

三脚本均覆盖 ac-core 门禁(check/fmt/all-targets 零 warning)与各自 case 表;
741 另含主仓最终门禁(`cargo check -p auto-lang`、`cargo t`、`cargo tv`,
本仓 Category A/B 分级下按需以 -SkipMainGates 跳过)。所有子进程有截止时间。

## 测试族

| 测试 | 内容 |
|---|---|
| text_binding | 四份设计样例绑定、作者/显式/canonical 往返语义等价、enum 简写、15 类拒绝 |
| hir_verify | §9 拒绝矩阵 + 组合反例(共享表达式/块、环、跨 body、初始化交集、返回路径、块图自环/断开环) |
| native_execution | 真实 PE 运行:add(2,3)=5、count 循环、i32 溢出 trap=70、COFF 符号 |
| cli | check/build CLI 退出码、入口/能力/链接错误路径、失败不覆盖、块图反例 5s 内定位拒绝 |
| trace_execution | 显式 capability + 支持库:原生调用顺序 b→a、pair 结果 12 |
| lib(link 单测) | 执行器:1s 截止杀挂起 helper、收集阶段同截止(后代持管道)、大输出完整、发布事务失败矩阵(准备/备份/三次发布) |
| source_frontend | 745:21+9 正例解析、23 后段负例必须通过前端、30+11 text 级 code+token 精确拒绝、结构/span/行列抽查 |
| source_hir | 745:全管线到 CheckedModule、23 bind/verify 级 stage/code/token 拒绝、while/bool 编码结构、4 类 Atom 变异拒收 |
| source_native | 745:21+7 正例真实 PE 构建+运行(退出码/stderr oracle、60s 硬截止、三制品断言)+1001 层深块 native |
| source_cli | 745:0/1/2 分级、profile 必填、entry 按名、失败零制品、未闭合调用输入不入制品(SENTINEL)、trace 无库链接失败、旧 check 仍仅 HIR |
| deep_regression | 745:任意深度合法链/括号/调用/块(2 万层)全阶段无递归通过;未闭合任意深度受控拒绝 |
| scalar_hir | 767:双身份隔离(bind_document 分派/域门 verify/writer 换壳拒)、V-07 程序化变异门(错类型/共享/自环/二环/跨 body)、canonical 往返、4096 逻辑深链 |
| scalar_source | 767:11+19 源码正反例、core 面在 token 级拒新语法、深输入(16384 项逻辑链/2 万层 !/4096 嵌套调用)到 Checked |
| scalar_native | 767:HIR PE(除余六行表、七 trap 行 exit70+精确 stderr、短路六极性)+ 源码 PE(gcd/计数/is_digit/条件 for/CRLF/全上下文 trace)+ CLI 隔离 + 收据身份与 SENTINEL 事务保持 |
| text_hir | 770:HirDomain::Text 隔离、verify_text 资源前门(owned-scope-escape/borrow-escape/use-after-move/double-move/mut-conflict/resource-assign)与 TextResourcePlan 精确断言 |
| text_source | 770:V-02/03/08/09 source 腿(Unicode/码点/遮蔽/静态拒绝/4096 深链)到 Checked |
| text_native | 770:真实 PE——V-02 Unicode(source+HIR)、V-03 码点往返与 invalid-scalar trap70、V-04 生命周期 trace(LIT/CONCAT/CLONE/FREE 配对、LIFO、.move 转移、跨帧拥有返回)、V-07 顺序/短路、V-09 4096 链/循环/调用链、V-10 合同门变异与 SENTINEL、标量面隔离 |
| text_resources | 770:V-05 失败类矩阵(slice 越界/反向/负界、char_at 越界、invalid scalar 全部 trap70+RT-FAIL code)、零长拥有、合流 LIFO、move 单次释放;预算/注入边界归 runtime 测试 |

## 边界(勿过度宣称)

- HIR 与源码两模式接受**已注册身份**:core 域 `auto.hir.core.draft / 1 /
  core-i32-draft` 与 scalar 域 `auto.hir.scalar.draft / 1 /
  scalar-i32-draft`(各自 descriptor);未知 tag/case/字段/能力/三重身份一律
  受控拒绝,不猜测、不回退、不换壳。
- 类型只有精确 i32/bool。core 域运算 add_i32/mul_i32/lt_i32(加/乘声明
  `overflow: trap`);scalar 域另有 sub/div/rem(除/余护栏:0、MIN/-1、
  MIN%-1 → trap70,不逃硬件异常)、le/gt/ge、同类 eq/ne、一元 neg_i32/
  not_bool 与 lazy Logical and/or(短路,与 eager Binary 分离表示)。
- 源码 profile `auto.source.core-i32.draft` 内 int/i32 = 精确有符号 32 位
  (越界 source.literal-range 拒绝);这是**有限域规则,不是全语言 int 宽度
  冻结**——旧 VM/AST/trans 表面宽度并存(见
  [docs/reports/745-source-core-i32/numeric-boundary.md](../../docs/reports/745-source-core-i32/numeric-boundary.md))。
- scalar 源码 profile 的循环写法是条件 `for <bool> { ... }`(每轮重判、
  continue 回条件);`while` 在该 profile 报 source.unsupported,core
  profile 的 while 原样、其 for 拒绝。MIN%-1 的 trap70 是本 profile 的
  固定测试语义,不推导全语言/生产异常 ABI。
- 源码子集不含 char/str/聚合/泛型/方法/模块引用;char 一律
  source.unsupported(char↔int 映射在宿主表面间不一致,保持 open)。
  **text profile 例外(770)**:`str`/`Str`/`char` 与上表内建方法是该
  profile 的显式面;core/scalar profile 下这些 token 仍按原样拒绝
  (零行为变化)。文本面自身不含 List/Map/record/enum 文本字段、
  字素索引/normalization/f-string/raw/正则、生产 print/File/process
  或任何公开 ABI。
- 文本运行时是**内部测试 ABI**(`ac.runtime.text.v1`,runtime/text
  静态库,774 合同同构):能力须显式声明提供,support-lib 伴随
  `<lib>.contract.json`(api/revision/target/hash 对真实库文件校验);
  RT-FAIL stderr 行 + 退出码 70 是失败类取证;预算/故障注入无语言
  入口(归 runtime 测试)。两个 Rust no_std staticlib(trace 支持库与
  文本运行时)因 `__rustc::rust_begin_unwind` 重复不能共链一个 PE;
  mark×text 共链面用 `test-support/marks.rs` 的**纯 COFF 对象**
  (`rustc --emit=obj`,无 panic 入口符号)承载(F-01 裁定路径 A)。
- 关键字不可作名字:profile 自身的 15 个关键字与生产词法器的完整 Auto
  关键字集合(含 type/enum/use/mut/nil 等)都按 `source.unsupported` 拒绝于
  名字位;识别但排除(关键字/排除 literal/排除运算)与坏 token(source.lex)、
  坏语法(source.syntax)三类诊断分开。
- 源码函数的 native 符号带 `sf_` 前缀(声明 `name` 字段),与后端固定的
  `ac_start`/`ExitProcess` 内部符号隔离——源码可以合法声明并调用/入口这两个
  名字,不新增源码保留名;HIR 身份与调用绑定走 def id,不受前缀影响。
- 未括起的表达式不跨语句换行(运算符右值、callee 到 `(` 都以表达式首行为
  界);括号与实参表内换行是空白。
- 表达式、调用与**语句块**嵌套均无上限:解析/解析树 Drop/resolve/typecheck/
  adapter(含终止分类 memo)/verify 数据流/native 发射全部显式栈迭代,
  无每层递归,无嵌套计数护栏;任意深度的未闭合输入受控拒绝于 text 阶段
  (缺 `)`/缺 `}` 精确指向当前 token 或 EOF),合法深输入全阶段可运行。
- 单模块内解析引用;跨模块引用、字符串/聚合/所有权/泛型、生产 ABI、
  DLL/热重载均不在本原型范围。
- trace intrinsic 是测试能力(`hir.test.trace.v1`),不是生产 I/O;用户
  函数/local 同名优先,不能被名字劫持。
- 溢出 trap 固定退出码 70,正常测试入口退出 0;这不是全语言异常 ABI。
