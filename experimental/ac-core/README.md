# ac-core — PLAN-741 AC 首个闭环原型（PLAN-745 扩展源码 profile）

独立 Cargo workspace:Schema-bound Atom HIR 文本 → 类型化绑定 → 语义校验 →
Cranelift native lowering → Windows x64 COFF 对象 → rust-lld 链接 → 运行 PE。

PLAN-745 在同一管线上新增**显式源码 profile** `auto.source.core-i32.draft`:
Auto 源码计算子集(.at)→ span 化前端 → 词法/类型解析 → 生成 Atom 文本(带
源 span 映射)→ 复用既有 binder/verifier → 同一 Checked→native→事务发布路径。
这是计算核心 profile 的第二条纵向实现:**"Auto 源码子集 → 原生执行"**,
仍不是完整 Auto 源码编译器,也不是 AAC 自举完成。范围、非目标与验收见
[docs/plans/745-ac-source-checked-hir-native.md](../../docs/plans/745-ac-source-checked-hir-native.md)。

## 流水线

```text
[HIR 模式:check/build]
.atom 文本
  → atom_text.rs        分词/解析,带字节 span 的语法树
  → descriptor.rs       core-i32-draft descriptor 绑定(schema/core-i32.atom)
  → hir.rs              类型化 ID + Unchecked HIR
  → verify.rs           语义校验 → CheckedModule(唯一构造路径;块包含图
                        可达/无环结构门先于数据流走查)
  → native.rs           Cranelift lowering(溢出 trap = ExitProcess(70))
  → link.rs             rust-lld COFF 链接(暂存+备份+发布,失败回滚 + 收据;
                        全子进程硬截止覆盖等待与输出收集,Job Object 管控进程树)
  → Windows PE

[源码模式:check-source/build-source,PLAN-745]
.at 源码 + 显式 --source-profile auto.source.core-i32.draft
  → source/lexer.rs     UTF-8/ASCII 标识符/行敏感 token(排除面 token 化)
  → source/parser.rs    语句型语法(优先级 * > + > <、有符号字面量域检查)
  → source/resolve.rs   签名预登记/块作用域/参数映射(bind 级 source.* 诊断)
  → source/typecheck.rs 精确类型检查(verify 级 source.type-mismatch)
  → source/adapter.rs   生成 Atom 文本 + span 再锚(while→loop/if 降低、
                        bool=0<1/0<0、return 后死代码独立块、stray break
                        phantom loop、条目序 span 投影 + UTF8 边界后验)
  → 同上 descriptor/verify/native/link(无旁路、无回退)
```

## 使用

CLI 的二进制名是 `auto-ac-prototype`("ac-probe" 为历史文档名),子命令
check/build/check-source/build-source 直接跟在 `--` 之后(与
`scripts/verify-ac-741.ps1`、`scripts/verify-ac-source-745.ps1` 一致):

```powershell
cd experimental/ac-core

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

源码模式(PLAN-745;profile 必须显式给出,不按扩展名猜模式;入口是
**源码函数名**,零参数、返回 int/i32):

```powershell
cd experimental/ac-core

# 源码检查(退出码 0 / 1 / 2)
cargo run --locked -- check-source fixtures/source/valid/p01-precedence.at `
  --source-profile auto.source.core-i32.draft

# 源码原生构建 + 运行(源码 profile 与输入 hash 写入 .ac-link.txt 同事务)
cargo run --locked -- build-source fixtures/source/valid/p15-named-eval-order-x2.at `
  --source-profile auto.source.core-i32.draft `
  --entry main --output target/ac-artifacts/p15.exe `
  --capability hir.test.trace.v1 `
  --support-lib test-support/target/debug/ac_trace_support.lib
```

## 一键验证

```powershell
# HIR 模式回归(741)
powershell -NoProfile -ExecutionPolicy Bypass -File ../../scripts/verify-ac-741.ps1
# 源码 profile 全语料(745:74 例串行,正例真实构建+运行,60s/例硬截止)
powershell -NoProfile -ExecutionPolicy Bypass -File ../../scripts/verify-ac-source-745.ps1
```

后者覆盖 ac-core 门禁(check/fmt/all-targets 零 warning)、四个 source 测试族
与完整 case 表;前者覆盖 §6 全部命令:定向测试族(text_binding / hir_verify /
native_execution / cli / trace_execution)、CLI 实机构建运行、fmt 与 all-targets
零 warning 检查,以及主仓最终门禁(`cargo check -p auto-lang`、`cargo t`、
`cargo tv`)。所有子进程有截止时间。

## 测试族

| 测试 | 内容 |
|---|---|
| text_binding | 四份设计样例绑定、作者/显式/canonical 往返语义等价、enum 简写、15 类拒绝 |
| hir_verify | §9 拒绝矩阵 + 组合反例(共享表达式/块、环、跨 body、初始化交集、返回路径、块图自环/断开环) |
| native_execution | 真实 PE 运行:add(2,3)=5、count 循环、i32 溢出 trap=70、COFF 符号 |
| cli | check/build CLI 退出码、入口/能力/链接错误路径、失败不覆盖、块图反例 5s 内定位拒绝 |
| trace_execution | 显式 capability + 支持库:原生调用顺序 b→a、pair 结果 12 |
| lib(link 单测) | 执行器:1s 截止杀挂起 helper、收集阶段同截止(后代持管道)、大输出完整、发布事务失败矩阵(准备/备份/三次发布) |
| source_frontend | 745:21 正例解析、23 后段负例必须通过前端、30 text 级 code+token 精确拒绝、结构/span/行列抽查 |
| source_hir | 745:全管线到 CheckedModule、23 bind/verify 级 stage/code/token 拒绝、while/bool 编码结构、4 类 Atom 变异拒收 |
| source_native | 745:21 正例真实 PE 构建+运行(退出码/stderr oracle、60s 硬截止、三制品断言) |
| source_cli | 745:0/1/2 分级、profile 必填、entry 按名、失败零制品、trace 无库链接失败、旧 check 仍仅 HIR |

## 边界(勿过度宣称)

- HIR 与源码两模式都只接受 `auto.hir.core.draft / revision 1 / core-i32-draft`
  身份;未知 tag/case/字段/能力一律拒绝。
- 类型只有精确 i32/bool;运算 add_i32/mul_i32/lt_i32;加/乘必须声明
  `overflow: trap`。
- 源码 profile `auto.source.core-i32.draft` 内 int/i32 = 精确有符号 32 位
  (越界 source.literal-range 拒绝);这是**有限域规则,不是全语言 int 宽度
  冻结**——旧 VM/AST/trans 表面宽度并存(见
  [docs/reports/745-source-core-i32/numeric-boundary.md](../../docs/reports/745-source-core-i32/numeric-boundary.md))。
- 源码子集不含 char/str/聚合/泛型/方法/模块引用;char 一律
  source.unsupported(char↔int 映射在宿主表面间不一致,保持 open)。
- 关键字不可作名字:profile 自身的 15 个关键字与生产词法器的完整 Auto
  关键字集合(含 type/enum/use/mut/nil 等)都按 `source.unsupported` 拒绝于
  名字位;识别但排除(关键字/排除 literal/排除运算)与坏 token(source.lex)、
  坏语法(source.syntax)三类诊断分开。
- 源码函数的 native 符号带 `sf_` 前缀(声明 `name` 字段),与后端固定的
  `ac_start`/`ExitProcess` 内部符号隔离——源码可以合法声明并调用/入口这两个
  名字,不新增源码保留名;HIR 身份与调用绑定走 def id,不受前缀影响。
- 未括起的表达式不跨语句换行(运算符右值、callee 到 `(` 都以表达式首行为
  界);括号与实参表内换行是空白。
- 单模块内解析引用;跨模块引用、字符串/聚合/所有权/泛型、生产 ABI、
  DLL/热重载均不在本原型范围。
- trace intrinsic 是测试能力(`hir.test.trace.v1`),不是生产 I/O;用户
  函数/local 同名优先,不能被名字劫持。
- 溢出 trap 固定退出码 70,正常测试入口退出 0;这不是全语言异常 ABI。
