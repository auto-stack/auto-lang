# ACE 完整控制流错误传播与生成器（M2-A 切片）

> PLAN-965；实现位于 `core/src/source/{resolve.rs,agg_lower.rs,adapter.rs}`（try/catch/finally、
> `rethrow`、`fail`、`.?`/`.?(d)`、`??`、`is` 表达式/语句、跨帧控制态、生成器状态记录与 close 清理）、
> `av-core/src/{abc,compact_abc,contract,instr_spec,mir_verify,ownership,av}.rs`（`LANG_FAIL` wire opcode、
> `ExecOutcome::Error`、终止符/所有权/验证）、`lowering/src/hir_adapter.rs`、`native/*`（LangFail 车道）、
> `a2x-{rust,c,worker}`、`av/src/bin/av-worker.rs`（`ac-av/aggregates-source-run-v1` recipe）、
> `crates/auto-lang/test/cases/ace-m2/control_effects/`（语言 TestCase 语料）。
> 本文件只记录**已验证**的支持、接口、拒绝与边界；未列入的能力不视为已支持。分母行归属见
> [replacement-scope-and-evidence.md](replacement-scope-and-evidence.md) SD-09：L-18(is)/L-21(`??`)/L-26..30。

## 支持入口与身份

控制效果面经 `ace_core::source::compile_aggregates_source`（`SourceProfile::Aggregates`）显式进入；
四端消费同一共享 MIR（AV interpreter、native Cranelift PE、A2X-Rust、A2X-C），无 legacy fallback。
语言级 TestCase 组为 `case-ace-m2-control-effects-965`（`scripts/test-groups.toml`），15 case（其中 9 个带 negative 标签），逐 case 一条 check。控制态实现为每函数一组 `__agg_ck_*`（kind）/`__agg_ce_*`（message）/`__agg_ctl_*`
（target loop）/`__agg_crv_*`（return payload）本地槽，跨 try/catch/finally 帧保存/恢复。

## 能力

| 能力 | 支持 | 语义 |
|---|---|---|
| try/catch（L-27） | ✓ | `catch e` 只绑定**本帧**语言错误（`fail`/`rethrow`/未抑制的 `.?`）；错误以控制态 `kind=4`+message 记录并在帧出口分派；`rethrow(e)` 仅可在 catch 内使用（`source.rethrow-outside-catch`），重抛替换待处理错误 |
| finally（L-27/L-26） | ✓ | 每帧恰执行一次；try/catch 内的 `return`/`break`/`continue`/错误先存控制态，finally 结束后再分派（break/continue 定向到最近的**用户**循环）；finally 内新错误覆盖先前待处理错误 |
| `fail(msg)` | ✓ | 语言错误终止：AV `ExecOutcome::Error{message,span}`；native `Trap(9)`（无 payload）；A2X `AutoTrap::LangError`（**class only**，不跨目标携带 message/span）。跨调用边界的 `fail` 是终止性语言错误，**不被调用者 catch 绑定**（无跨帧展开） |
| `.?` / `.?(d)`（L-28） | ✓ | `.?` 把错误 payload 传播到最近的本帧 catch，帧根处升级为语言错误；`.?(d)` 抑制并求值为 `d`；传播目标返回类型不兼容时 `source.propagate-incompatible` 具名拒 |
| `??`（L-21） | ✓ | `Option` 空值合并：左支 `Some` 直接取 payload，`None` 才**惰性**求值右支（右支副作用不得发生，V-02） |
| `is` 表达式/语句（L-18） | ✓ | `is v { 1 -> 10, 2 => { 20 }, else -> 30 }`（表达式）与语句形态；重复臂/非穷尽分别 `source.pattern-duplicate`/`source.pattern-nonexhaustive`；块臂的尾表达式即臂值 |
| 嵌套 break/continue/return（L-26） | ✓ | 穿过任意层 try/finally 时先执行 finally，再落到目标循环/函数出口；`break`/`continue` 无用户循环时 `source.break-outside-loop`/`source.continue-outside-loop`（try 帧**不**算循环） |
| 生成器 `~Iter<T>`（L-30） | ✓ | 惰性启动（首次 `next()` 前不执行体）；`yield` 挂起状态记录并返回值；耗尽后 `next()` 返回 `None`；`close()` 幂等（`state=-1`，释放恰一次）；`for x in g()` 提前 `break`/正常 drain 都会关闭，并执行未完成的 `finally` |
| 生成器 close 期错误（L-30/V-08） | ✓ | close 重放 `finally` 时若置 `kind=4`（`fail`/`rethrow`），按语言错误分派：AV `ExecOutcome::Error`、native `Trap(9)`、A2X class-only；drain、提前 break、显式 `g.close()` 三路一致 |
| 生成器间接返回 | ✓ | 无 `yield` 的 `~Iter<T>` 函数（`return g()` / 尾调用 `g()`）按普通函数返回可迭代值；状态机只在 `block_has_yield` 时生成 |
| 表达式语句（aggregates） | ✓ | 非尾部、非 unit、非控制效果的表达式语句**求值后丢弃**（resolver 落 `RStmt::Let` → HIR `Let { __agg_ev_* }`，不是 `Eval`：collections 域校验器拒绝非 unit 的 `Eval`）——调用必须真实执行；入口/宿主轮廓保留 `source.entry-tail` 拒绝（表达式语句只允许作脚本尾） |
| 借用生成器参数 | ✗ 具名拒 | `source.generator-borrow-param`：状态记录按值保存参数，`mut` 参数与记录视图参数即使从不跨 `yield` 使用也被拒绝（借用文本 `str` 在 aggregates 轮廓更早由类型面拒绝，本拒绝对 `str` 源码不可观测）；owned 句柄（`List`/`Object`/`Variant`/owned `Str`）与纯标量支持 |
| 宿主权限失败 | ✗ 不可捕获 | `host.unauthorized` 等宿主拒绝不进入语言 catch（dispatch=0），与 D3 四分类一致 |

## 四分类退出（AV `ExecOutcome`）

`Success{value,steps}` / `Error{message,span}`（语言级，本包新增）/ `Trap{kind,span}` /
`Resource{kind,message}` / `Cancelled` / `Infra(_)`（执行器内部）。语言 catch **不得**吞掉
`Trap`/`Resource`/`Cancelled`/宿主拒绝；`fail` 与未抑制的 `.?` 产生 `Error` 而非 `Trap`。

## Wire/IR 合同

- `Terminator::LangFail { message }`，wire opcode **`0x40`**（指令空间 `0x01–0x3F` 已满，
  `m2-dynamic` 冻结 `0x3D/0x3E/0x3F`；`av-core/src/{abc,compact_abc}.rs` 与
  `instr_spec.rs` 的 `LANG_FAIL` 行同值，V-11 钉表 57 行）。
- ABC/compact-ABC 双向 codec、`mir_verify` 终止符/可达性/所有权门、`owner` 守恒（owned 值经
  return/throw/drain-close 各释放恰一次；取消路径同样 `live==0`）。
- native：`LangFail` → `Trap(9)`，不带 payload；`a2x-{rust,c}`：`a2x_coll_fail::<T>() -> Err(AutoTrap::LangError)`。

## 拒绝与边界（具名诊断）

`source.propagate-incompatible`、`source.yield-outside-generator`、`source.rethrow-outside-catch`、
`source.pattern-nonexhaustive`、`source.pattern-duplicate`、`source.generator-borrow-param`、
`source.break-outside-loop`、`source.continue-outside-loop`；入口轮廓另有
`source.entry-tail`/`source.entry-tail-type`（表达式语句只允许作脚本尾）。

## 已知边界（本轮复核记录）

- **`int` 函数末尾 try 可正常完成**：`fn f() int { try { g() } catch e { return 42 } }`（try 体可正常完成、
  函数无返回）保持 `verify.no-return-path` 拒绝；round-3 曾判其为"合法形态"，round-4/5 复核确认该拒绝
  正确（`int` 函数无隐式尾返回）。
- **完全发散的 try/catch 之后的语句**：`try { fail(..) } catch e { return 0 } return 1` 现在编译通过，
  尾部 `return 1` 为不可达死代码（不再触发 `verify.block-structure`）；语言不对死代码另设诊断。
- **close 期错误与状态归一化顺序**：close 重放的 `if kind==4 { fail(err) }` 位于字段复位与
  `__state=-1` 之前，即"先抛错后归一化"；因语言错误终止整个执行、close 不会复发，当前不可观测；
  将来若允许 close 出错后继续执行，需重排该顺序。
- **非 unit 丢弃的生成器开销**：每次被丢弃的调用在生成器状态记录中多一个 `__agg_ev_*` 字段
  （空间膨胀，非泄漏；所有权探针 `live==0 && allocs==frees`）。

## 四端支持域矩阵

| 能力 | AV | native(mn) | a2x-rust | a2x-c |
|---|---|---|---|---|
| try/catch/finally、rethrow、fail、`.?`、`??`、`is` | ✓ | ✓ | ✓ | ✓ |
| 生成器 `~Iter<T>`（惰性/挂起/close/finally） | ✓ | ✓ | ✓ | ✓ |
| close 期语言错误 | `ExecOutcome::Error` | `Trap(9)` | `LangError` 类 | `LangError` 类 |
| 引用/owned 句柄生成器参数 | ✓ | `source.generator-borrow-param`（ABI 标量/owned 记录） | 同左 | 同左 |

## 四端证据

- 语言 TestCase：`cargo t case-ace-m2-control-effects-965` → PASS 15/15，`gate_passed=true`
  （`run-v_q04h3j`，wall 6.9 s）。
- 固定组（`551f7795e` 冻结树，组序执行；**共享 `experimental/ace/target` 的 ad-hoc 复验会链接陈旧 rlib 给出修复前语义的假红，必须用干净 `--target-dir`**）：`ace-source` 523/524、`ace-av` 1129/1131、`ace-native` 274/278、
  `ace-a2x` PASS 238/238（干净 `--target-dir`，共享 target 会假红；见 round-7 收据）；红集逐名等于 `master` 既有 `text.char-multi` 族（6 例）与
  `ace-runtime` text shim 链接红（19 例），本包 diff 引入的红为 0。
- V 行映射与逐 F-ID 修复证据见 PLAN-965 §9 修复收据（round 4）。
## PLAN-1083 增量（2026-11-20，AVP-05 模式/控制流语言事实）

965 is/try/yield 基线保持；本包仅增 auto 面模式组合事实：

- **guard 臂**（auto 面独占）：`pat if cond -> body`——guard 在模式命中后、载荷 Let 之后求值（绑定可见），false 经 else 链继续下一臂（同一剩余链双引用：guard-else clone + pattern-else 原件）；未选臂 guard 不求值（B36 证据）；同 disc 重复仅当每一前序同 disc 臂皆 guard（纯重复保持 `source.pattern-duplicate` 冻结）。
- **or 臂**：`A | B -> body` 投影展开为共享 rhs 的多臂（绑定元数不一致=展开后 undefined 具名拒）。
- **语句 fall-through**（auto 面独占）：非穷尽语句 `is` 无匹配继续后续语句（穷尽检查与 return filler 按 `has_following` 门控——尾 filler 会使后继代码不可达）；值位（is 表达式）与冻结面语句穷尽合同不变。
- **尾值规范化**（infer::complete，parse→resolve 间纯 AST 改写）：fn 尾 `is` → `Return(IsExpr)`（return 臂尾保持语句形）、script 尾 `is` → 尾值 ExprStmt；`!T` 全枚举 Err 载荷 → 显式 `Result<T, E>` + 尾 `Ok` 归一 `return Ok(v)`。
- 证据：`plan1083_canonical_verify`（patterns 12 例）/`plan1083_records_variants_patterns`（B34/B36/B37/B38/O53）；重放 `cargo t case-av-language-parity`。

## PLAN-1087 增量（普通 Auto AV）

普通 Auto 的受检 HIR 标出异常可退出的循环作用域及其控制状态与拥有型消息槽。MIR 与固定 ABC 的 `INVOKE` 区分调用成功和错误两条控制边，验证器分别核对调用结果所在的成功边和消息所在的错误边。`RAISE` 向当前激活的处理器交付错误并标记实际源码位置。`LANG_FAIL_AT` 将未处理错误移交给调用者，保留最早的来源信息。运行时在错误边结束该作用域的借用并释放穿过的调用帧。窄 profile 继续使用其既有控制流和 artifact 契约。

catch 与 finally 使用统一的退出状态。跨调用的程序错误可触发 finally 并到达外层 catch。预算、授权、取消和宿主基础设施失败使用各自的终态。编译和运行观测均用虚拟源码路径和 UTF-8 字节区间定位。复现见 [PLAN-1087](../../plans/archive/1087-av-parity-errors-and-unwinding.md)。

## PLAN-1089 增量（普通 Auto AV 流）

在普通 Auto 面，直接 for-in 的生成器可以在首次挂起前使用调用者的只读或 mut 借用。可验证的挂起前段按原形参 ABI 运行，mut 结果写回原 owner，传入普通迭代状态的数据为标量或拥有值。跨挂起引用在绑定时拒绝，产生带借用流的调用须由同一表达式的 for-in 消费。

生成器成功恢复继续原控制流。耗尽与关闭释放保存在状态记录上的拥有数据。显式 close 先安排 terminal 状态，再从内到外执行悬挂的 finally；关闭失败仍清理留存数据，然后将程序错误移交调用者。调用者在可能捕获错误的作用域内移交记录时保留 terminal 值，符合普通记录的成功写回 ABI。预算、授权与取消仍由 AV 自身终态报告。语料与真实所有权观测见 [PLAN-1089](../../plans/archive/1089-av-parity-generators-and-stream-lifetimes.md)。

## PLAN-1090 增量（普通 Auto AV 异步执行）

等待链通过现有 `INVOKE` 错误边接收异步体的程序错误，保留原始源码字节区间。生成器与任务可在同一程序中运行。取消、预算和宿主 I/O 失败保持各自的 AV 终态。普通单文件执行在调度前校验控制台授权，未授权调用报告 `host.unauthorized` 和源位置，宿主派发和程序 catch 均为零。

已有 AV foreign 执行入口把显式授权的当前代调用表与运行字节账带入调度链。实际 Rust F0 提供方完成一次调用和一次等待唤醒。撤权与缺少运行时绑定均在派发前拒绝。[PLAN-1090](../../plans/archive/1090-av-parity-tasks-async-and-actors.md)
