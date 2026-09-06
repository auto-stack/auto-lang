---
plan_id: PLAN-567
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: script-mode-w25-tail-w3-oracle
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-lang/trans, auto-lang/frontend]   # 受影响的 specs 路径
current_step: 0
total_steps: 20
---

# [PLAN-567] 脚本模式收官波：W2.5 收尾批（p7 双红清偿 + Err 通道汇合 + with-as 绑定）+ W3 注解预言机

## 变更摘要

按 [script-mode-interop.md](../design/strategy/script-mode-interop.md) §9 波次骨架，本计划收束脚本模式
设计线的剩余两件事：**W2.5 收尾小批**与**W3 注解预言机（设计上最后一波）**。四块工作：

1. **p7 相位两例 master 既有红清偿**（P560-D6 / P539-D1）：
   - `py_list test_sorted_getitem`——engine ADD string-concat 臂**双重 rc_release 实锤**
     （engine.rs:5796-5797 与 :5814-5815 两对），rc 下溢回绕 u32::MAX → 字符串池 freelist
     幻影条目（P053-8 签名）→ 拼接中间结果即产即死、"got d"；
   - `py_sys version_info`——顶层 tuple 拍平臂（py_ffi.rs:2273-2292）把 namedtuple
     `(3,12,1,'final',0)` 拍成 List，随后 `py_getattr(vi,"major")` 把 ListData 转回
     Python list → AttributeError。namedtuple 分流 opaque 句柄通道，普通 tuple
     维持拍平（DIV-PY-TUPLE-1 裁定不动）。
2. **P560-D3 + P560-D2：Err 通道汇合**——py 桥错误前缀统一 `PyException <Type>: <msg>`；
   `.as` 隐式 `!T` 传播自动化（s2s 补 `.?` + 桥调用转 may 变体 + 引擎 ERROR_PROPAGATE
   值通道拦截 catch），实现设计 §4「隐式 !T + Err 单通道 + 边界 catch」终态。
3. **P560-D1：with-as 绑定语法落地**——with 头部 pratt 截断（`as` Cast 臂上下文旗标），
   as 形态降为 `{ var __w = e; var x = py_enter(__w); try { body } finally { py_exit(__w) } }`
   块形态（parser.rs:1251-1252 已注释的目标形态；py_enter 改返 `__enter__` 值）。
4. **W3 注解预言机**——`typing.get_type_hints` 内省（全仓尚无任何调用，空白落点），
   注解灌 `PySignature`/`py_return_types` 知识表：`-> float` 承诺 → D4 授权标量
   强制（shim 出口 coercer）；`-> T | None` → nullability lint（W 诊断警告）。
   定位是**优化与糖的授权，不是正确性地基**（注解撒谎最坏退回 Python 行为，设计 §7）。

## 目标

- p7 相位（py_configparser/py_hashlib/py_json/py_list/py_os/py_re/py_string/py_sys）三方全绿，
  master 既有双红归零；P053-8 phantom freelist 签名清零。
- `.as` 脚本的 py 错误成为**可传播可捕获的 Err 值**：隐式传播零样板（不写 `.?` 也不写
  py_call_may），catch 拦值绑定 `PyException <Type>: <msg>` 载荷，a2py 侧零代码（ErrorPropagate
  → 直落，python.rs:293-294 已备）。
- `with expr as x { body }` 三方一致：AutoVM（含体内 Err 传播后 py_exit 必执行）≡ a2py
  `with expr as x:` ≡ Python 原生 with。
- use.py 导入函数的返回注解被读取并消费：注解承诺的 float/int 自动强制；nullable 返回
  未判空直用时编译警告。无注解路径行为零变化（backward 兼容）。

### 非目标

- py_subclass 类派生（P539-D4，独立延期案）；中缀 `@`/`**` 运算符（roadmap §7.1/7.2 独立
  立项）；P539-D2 方法分派动态化根治（本波 D4 强制只在 use.py 注册函数面，方法调用糖不涉）。
- 异常抑制型 with（`__exit__` 返回真值吞异常）——§10 裁决默认不做。
- 注解覆盖 typeshed/内置函数补全（builtins 无 `__annotations__`，运行期回退 Auto 是设计内行为）。

## 架构方案

沿用 Plan 560 的分层不变：**糖 →（s2s/parser lowering）→ py_xxx 桥 → VM**，语义本体在桥上。

```
.as 源 ──parser(with-as 块形态/py_with 闭包形态)──┐
      ──s2s 规则链(隐式 .? 传播/may 变体改写)────┤→ 正常模式 .at IR ──VM codegen──→ 指令
                                                  │
use.py 导入 ──inspect.signature(param_count 已有)──┤
           ──typing.get_type_hints(本波新增)──────┴→ PySignature/PyType 知识表
                                                      ├→ shim 出口 D4 coercer
                                                      └→ codegen nullability lint
```

- **值通道汇合点选在引擎 ERROR_PROPAGATE**：Err 是值（Result.Err 堆实例），传播是
  ERROR_PROPAGATE 帧展开（engine.rs:3399-3510）；catch 既有 intercept_error 只拦 VMError
  异常通道（engine.rs:9477-9509）。新机制唯一一件：ERROR_PROPAGATE 展开前查当前帧
  handler_stack，处于 try 体内则跳 catch_pc 绑 Err 载荷（设计 §4.3「值通道拦截是唯一新机制」）。
- **may 变体补桥不改编译期**：py_call_may(453) 已是 Err 值出口范式（py_ffi.rs:761-781）；
  getattr/getitem 照抄新增 476/477。s2s 规则在 `.as` 体内把严格桥调用改写为 may+.?——
  `.at` 正常模式零变化（显式通道仍可表达）。
- **with-as 走块形态而非闭包参**：出口保证靠 VM 已备的 try-finally 编译（codegen.rs:1187-1243，
  Plan 012 P2）；绑定值靠 py_enter 返回 `__enter__` 结果。a2py 侧对规范块序列模式匹配回译
  `with e as x:`（既有 py_with 1 参闭包映射 python.rs:417-426 保留）。
- **预言机数据流**：`inspect_return_annotation()`（照 inspect_param_count py_ffi.rs:1588-1642
  模式）→ `PySignature.returns` 从 all_auto 升级（py_ffi_types.rs:48-53）→ codegen
  `py_return_types`（codegen.rs:417/:5013）+ shim 出口 coercer 双消费。

## 技术栈

Rust（crates/auto-lang：parser.rs / vm/engine.rs / vm/codegen.rs / py_ffi.rs /
py_ffi_types.rs / trans/auto_s2s.rs / trans/s2s_rules.rs / trans/py_known.rs /
trans/python.rs / lib.rs / autovm_persistent.rs / error.rs）；Auto `.as` 语料与
parity 三方套件（parity/libs/python/*，auto-parity 二进制运行）。

## 需求分析与背景调查

（取材：script-mode-interop.md §4/§7/§9/§10、python-parity-roadmap.md §7.3/7.4、
KNOWN-DEBT-AND-RISKS.md P539-D1/D5、P560-D1/D2/D3/D6、四路代码调研 2026-09-05）

- **波次语境**：设计 §9 四波——W0 null 家族、W1 动态分派地基、W2 语法糖批（Plan 560
  已折，19 个 py 套件全量 .as）、W3 注解预言机。W2 折后遗留 W2.5 收尾候选四件即本波
  前三块 + p7 双红。W3 是设计上最后一波，本计划收束整条设计线。
- **p7 双红根因已定位**（master 二进制同形复现，非 560 回归）：
  - py_list：ADD string-concat 分支 engine.rs:5793-5818 第一对 release :5796-5797
    （"操作数 stake 随消费死亡"）之后 :5814-5815 在 add_string 前又 release 同一
    a_bits/b_bits——对照 STR_CAT 臂 :3950-3951 只 release 一次。rc=1 操作数被扣两次 →
    第一次归零入 freelist、第二次 fetch_sub 从 0 回绕 0xFFFFFFFF（rc.rs:588-599 记
    underflow）→ 槽在 freelist 但 rc>0，add_string pop 到时按 P053-8 不变量丢弃
    （engine.rs:1029-1043 计 phantom_drops）。症状即 "got d"（拼接中间结果即产即死）。
  - py_sys：`sys.version_info` 是**扁平** namedtuple；tuple 拍平臂 :2273-2292 命中
    `is_instance_of::<PyTuple>`（namedtuple 是 tuple 子类）→ List `[3,12,1,"final",0]`
    TAG_OBJECT；`py_getattr(vi,"major")` 的 pop_auto_py_arg TAG_OBJECT 臂（:2028-2048）
    把 ListData 转回 Python list → `getattr(list,"major")` 必然 AttributeError。
    py_sys README 的"namedtuple 当 opaque handle"设计意图与拍平裁定冲突点即此。
- **Err 通道现状**：py_call_may(453) 已产 `Result.Err` 值、载荷 `PyException <Type>: <msg>`
  （:761-781，单测 :2983-3022 在位）；严格变体的 py 异常走 `VMError::FFI("Python method
  {}.{}() failed: {}")` 前缀族（py_ffi.rs:81/117/136/177/450/599/650/706/926/963/1087 等
  十余处），560 T11 只在 CALL_NAT_COUNTED 臂做了 FFI→RuntimeError 转换
  （engine.rs:7719-7741），CALL_NAT 路径 :7696-7698 原样透传。ERROR_PROPAGATE
  （0x79，engine.rs:3399-3510）是 May 值通道：Result.Err → 帧展开提前返回，Option/null
  同通道；但值不进 handler_stack——catch 拦不到（两通道未汇合 = P560-D2 本体）。
  p14 探针（scratch/p560/p14_err_channel.as）已实证显式 py_call_may+`.?` 通道可表达。
- **with-as 现状**：with_stmt（parser.rs:1253-1295）用 parse_expr 吞上下文表达式，而
  `as` Cast 臂 :2605-2611 **不比 min_power、无条件吞** `as Type`——`with e as x` 的
  `e as x` 整体变 Cast，:1258-1267 响亮拒绝（消息含 P560 债指引）。无 as 形态直产
  `py_with(ctx, ()=>{body})`（539 通道：with_shim py_ffi.rs:1060-1125 自持 enter/exit
  + 闭包抛错 best-effort 清理）；VM 内联形态（codegen.rs:5163-5200）为直线
  py_enter;body;py_exit（仅 0 参闭包）。a2py 已支持 py_with 1 参闭包 → `with ctx as p:`
  （python.rs:417-426）——即 a2py 侧 as 通道可达，缺的是 Auto 层语法。VM try-finally
  编译已备（Plan 012 P2，codegen.rs:1187-1243），parser finally_body 已存在
  （parser.rs:1329/1383-1395）。py_enter shim 现推 encode_null 丢弃 `__enter__` 值
  （py_ffi.rs:1146）。
- **注解预言机现状**：内省只有 param_count——inspect_param_count（py_ffi.rs:1588-1642，
  Plan 300）+ discover_module_callables（:1646-1703），注册一律
  `PySignature::all_auto(n)`（lib.rs:677/:710 两条路径 + autovm_persistent.rs:180/:204
  REPL 路径）；`typing.get_type_hints`/`__annotations__` 全仓零调用，返回类型注解完全
  丢弃。codegen `py_return_types` 表（:417，:5013 灌 Auto，:9027-9040 last_expr_type
  消费）是现成灌点。py_float(465) 桥在（shim :1241-1261，a2py 反映射 python.rs:1084-1093）；
  py_int 桥不存在（需 478）。lint 挂点：error.rs W0001-W0009 警告诊断体系（:1030-1146）
  是现成发射通道（无独立 `auto lint` 命令，Plan 550/560 的 E5501 门禁是编译错误通道）。
- **门禁语境**：py parity 走 auto-parity 二进制（`cd parity && cargo run -- run py_list`，
  相位 `-- phase p7`），非 cargo tv；各计划门禁只跑 p5/p8/p9 导致 p7 红长期不可见
  （KNOWN-DEBT P539-D1 注）——本波修绿后 p7 纳入本计划门禁。

## 详细设计

### Phase A：p7 双红清偿（先清基线，纯存量修复）

**A1 ADD 双重释放**：删除 engine.rs:5814-5815 的第二对 release（保留 :5796-5797 的
stake 消费语义，与 STR_CAT :3950-3951 的单次纪律对齐）。不改池结构、不加宽限——
病灶是确定的重复扣减。修复后 phantom_drops 归零、`test_sorted_getitem` 输出 "abcd"。

**A2 namedtuple opaque 分流**：py_ffi.rs tuple 拍平臂（:2273-2292）入口先判
`getattr(obj, "_fields").is_ok()`（namedtuple 判别式）——命中则走 PyObjectHandle
encode（opaque，保 `py_getattr(vi,"major")` 语义），未命中维持 List 拍平。DIV-PY-TUPLE-1
（普通 tuple 拍平 + W2 套件单变量规避）不动。known-divergences.md 相应补记。

### Phase B：Err 通道汇合（D3 前缀 → D2 隐式传播）

**B1 前缀统一（D3）**：py_ffi.rs 新 helper
`fn py_exc_msg(type_name: &str, e: &pyo3::PyErr) -> String { format!("PyException {}: {}", type_name, e) }`
（或从 PyErr 直接取 `e.get_type().name()`，与 py_call_may :774 的构造对齐）。全部
`VMError::FFI(format!("Python ... failed: {}", e))` 构造点迁移为 `PyException` 前缀
+原文内嵌。engine.rs CALL_NAT 臂 :7696-7698 补 560 T11 同款 FFI→RuntimeError 转换
（两条 native 分发路径 catch 载荷一致）。a2py 无涉（catch→except，无前缀面）。

**B2 may 变体补桥**：新增 `NATIVE_PY_GETATTR_MAY=476`、`NATIVE_PY_GETITEM_MAY=477`
（号段 450-475 已满、500 起是动态模块 id，476-499 空闲）。shim 照 py_call_may 范式
（:761-781）：异常 → Result.Err 堆实例载荷 `"PyException <Type>: <msg>"`；成功 →
`wrap_tos_as_result_ok`。三处注册镜像：py_ffi.rs shim/BIGVM_NATIVES（lib.rs:760-769
区）、codegen.rs:5279-5293 常量表、a2py 反映射（python.rs:1028-1050 `_auto_may`
家族扩展）。**py_call_may 补 kwargs 组合（P539-D5）顺带核销**：以探针定 ABI 后让
py_call_kw 形态可走 may 出口（最小实现：kwargs 5 槽约定下复用同一 Err 出口）。

**B3 引擎值通道拦截**：ERROR_PROPAGATE（engine.rs:3399-3510）在 Result.Err 分支
（:3454-3458）帧展开**之前**查当前 task 的 handler_stack：非空且属于当前帧 → 弹
handler、绑 Err 载荷串到 catch 变量、跳 catch_pc（语义对齐 intercept_error 的
RuntimeError 取原文分支 :9486-9489）；栈空 → 维持现有帧展开。null/-1/Option.None
分支不进 catch（null 是值不传播，设计 §4.2）。**边界**：跨帧 Err（被调函数 return
的 Err 值在本帧被 `.?`）同样经此路径——即 catch 拦的是"传播动作"而非"异常对象"。

**B4 s2s 隐式传播规则**：新规则 `rule_err_propagate`（挂 builtin_rules 链尾，
s2s_rules.rs）：对 `.as` 函数体 AST 单遍——
- py 桥调用（py_call/py_getattr/py_getitem/py_call_kw…）→ 对应 may 变体，调用点
  追加 `Expr::ErrorPropagate`；
- Auto 用户函数调用点（同文件 fn / use 导入 fn）→ 追加 `.?`（运行期裸值直通，
  ERROR_PROPAGATE 对非 May 值零开销透传，engine :3467 裸值分支已在）；
- **第一批作用域**：调用 + 索引 + 属性（桥面糖产物天然覆盖）；null 运算数与算术
  TypeError 族归 W0 null 家族既有运行期面，不在本批 s2s 改写。
保守性对齐 py_known（分支不敏感单遍）；快照单测钉改写产物。`.at` 模式零改动——
E5501 门禁继续拒绝 .at 里的 use.py，显式 py_call_may+`.?` 通道依旧合法。

**B5 main 边界**：未捕获 Err 传播到 bp==0 → push+Terminate（engine :3488-3509 既有
分支）——核对进程带非零退出码与载荷打印（= Python traceback 等价物）。

### Phase C：with-as 绑定（D1）

**C1 parser**：`Parser` 加 `with_header: bool` 上下文旗标；with_stmt（parser.rs:1253）
在 parse_expr 前置位、解析后复位；`as` Cast 臂 :2605-2611 检查旗标——置位时**不吃**
`as`（返回左表达式），让 with_stmt 自己消费 `as Ident`。`.at` 正常模式 Cast 语义
零变化（旗标只在 with 头部窗口内生效）。歧义回归用例：`with x as T {}`（T 是类型
名）不再被误吞为 Cast；`var y = a as int` 等 Cast 惯用法不受影响。

**C2 块形态 lowering**：as 形态直产块语句序列（跟随 parser.rs:1251-1252 注释目标）：

```auto
{ var __w = <expr>; var x = py_enter(__w); try { <body> } finally { py_exit(__w) } }
```

- py_enter shim（py_ffi.rs:1132-1153）返回值 `encode_null()` → `__enter__` 结果经
  `py_auto_marshal_return` 封送（with_shim 自持通道不受影响；无 as 的 py_with 闭包
  形态维持现状）。
- `__w` 命名避开用户变量（前缀 `__`）；块作用域天然隔离。
- VM 侧走通用 try/finally 编译（Plan 012 P2 已备），不进 compile_py_with_inline
  内联臂（该臂仅匹配 0 参闭包，维持现状即自动分流）。

**C3 a2py 回译**：python.rs 识别规范块序列（`var __w = e; py_enter(__w); try …
finally py_exit(__w)`，`__w` 单用途）→ `with e:` 块（as 绑定 → `with e as x:`）。
py_with 1 参闭包既有映射（:417-426）保留。快照单测钉两形态。

### Phase D：注解预言机（W3）

**D1 内省与类型表示**：py_ffi.rs 新 `inspect_return_annotation(func) -> Option<PyType>`
——`typing.get_type_hints(func)` 取 `return` 项（照 inspect_param_count :1588-1642 的
import/attach/回退模式；get_type_hints 失败或 builtins 空注解 → None）。py_ffi_types.rs
`PyType` 扩展：`Float`（`float`）、`Int`（`int`）、`Nullable(Box<PyType>)`（`T | None` /
`Optional[T]`，解 `types.UnionType`/`typing.Optional` 两形态）。param 注解本波只读不消费。

**D2 注册接线**：lib.rs init_py_ffi 两条路径（裸模块 :677 / 命名导入 :710）+
autovm_persistent.rs:180/:204——`PySignature::all_auto(n)` 升级为
`PySignature { params: all_auto(n), returns: <注解或 Auto> }`。codegen `py_return_types`
（:5013）随之携带知识。常量通道（register_constant :511-545）同法读常量类型注解
（模块级 `x: float = …`，get_type_hints(module) 可读）。

**D3 D4 授权强制**：use.py 注册函数的 shim 出口在 `py_auto_marshal_return` 前查
`returns`——`Float` → GIL `float()` 强制后封送（py_float :1241-1261 同语义内联）；
`Int` → 新 `NATIVE_PY_INT=478`（GIL `int()`，三处注册镜像 + a2py `int(x)` 反映射）。
**无注解 = Auto = 现状零变化**（保守规则不引入：比较/print 强制本波不做，留位）。
撒谎注解最坏行为 = 提前强制成注解类型（与 Python 运行时行为可能分叉）——设计 §7
已裁定接受（不做正确性地基）。**方法调用糖（t.sum()）不涉**——oracle 只覆盖注册期
已知的模块函数。

**D4 nullability lint**：注册期收集 nullable 返回知识 → codegen 消费：已知 nullable
的 py 调用结果**未判空**（无 `.?`/`== null` 守卫）直接流入二元运算或方法调用时发
**警告级**诊断（error.rs 新 W 码，如 W0010 `py-nullable-unguarded`）。不阻断编译、
不自动改写（lint 只报告；强制归 D4 coercer 的 float/int 面）。

**D5 parity 套件**：新 `py_anno` 套件（parity/libs/python/py_anno/）——tests/python/
放本地注解模块 `anno_mod.py`（纯 Python、确定性）：`def scale(x: float) -> float`、
`def find(x: int) -> int | None`、`def ratio(a: float, b: float) -> float` 等 6-8 用例；
tests/auto/anno.as 用 use.py 导入对拍。覆盖面：float 强制（返回值直入算术/比较）、
int 强制、nullable 返回（判空路径 + lint 触发样例）、无注解函数零变化。注册进 p7
相位（auto-parity main.rs :455-467 表）。

## 测试设计

- **单测（cargo t 覆盖）**：
  - engine：ADD string-concat rc 配平回归（构造 rc=1 操作数拼接，断言无 underflow/
    phantom 计数）；ERROR_PROPAGATE 值通道拦截（try 体内 Err 传播 → catch 绑定
    `PyException …` 载荷；null 传播不进 catch）。
  - py_ffi：namedtuple opaque 分流（namedtuple→句柄可 getattr；普通 tuple→List）；
    476/477/478 注册与 Err 载荷形态；inspect_return_annotation（注解模块命中 /
    builtins 回退 None）；PyType 解析（`float`/`int`/`T | None`/`Optional[T]`）。
  - parser：with_header 旗标（`with e as x {}` 接受、`with x as T {}` 不误吞、
    正常模式 Cast 零回归）；块形态产物快照。
  - s2s：rule_err_propagate 改写快照（桥调用→may+.?、用户调用→+.?、.at 不改写）。
  - trans/python：with 块形态回译快照；may 桥 `_auto_may` 反映射。
  - codegen：nullability lint 触发/不触发样例。
- **vm file 语料（cargo tv）**：新增 .as 语料——隐式传播穿透函数边界、catch 拦值、
  with-as 出口保证（体内传播后 py_exit 仍执行——行为可观测化：exit 记录标志）。
- **parity（auto-parity 二进制）**：`cd parity && cargo run -- phase p7` 全绿（含
  新 py_anno）；既有 p5/p8/p9 复跑零回归；py_torch_infer 补 with-as 用例（no_grad
  as 形态，三方一致）。
- **合入前**：`cargo tf` 全量一次（VM 核心协议改动，Plan 466 门禁）。

## 验收标准

1. p7 相位三方全绿：8 个既有套件 + py_anno；py_list 8/8（"got d" 消失）、py_sys
   5/5（version_info.major == 3）；P053-8 phantom freelist 签名零出现。
2. p5/p8/p9 相位、`cargo t`、`cargo tv`、`cargo tt` 零回归；合入前 `cargo tf` 绿。
3. `.as` 内：不写任何 `.?`/py_call_may，py 错误（AttributeError/KeyError/IndexError）
   自动传播到调用者或被 catch 拦截，载荷严格 `PyException <Type>: <msg>` 前缀；
   未捕获时进程带错退出。`.at` 行为零变化（显式通道照旧、E5501 门禁不放松）。
4. `with expr as x { body }` 三方一致：AutoVM（含体内 Err 传播后 py_exit 必执行）
   ≡ a2py `with expr as x:` ≡ Python；`as` Cast 语义在正常模式与 with 头部之外零变化。
5. use.py 注解函数：`-> float`/`-> int` 返回值自动强制（py_anno 用例三方一致）；
   `-> T | None` 未判空直用时出 W 级警告；无注解函数行为与 master 二进制逐字节同形。
6. KNOWN-DEBT 核销：P539-D1、P539-D5（kwargs×may）、P560-D1、P560-D2、P560-D3、
   P560-D6 全部标记 resolved（附本计划链接）；known-divergences.md 补记 namedtuple
   裁定；script-mode-interop.md §3/§7/§9 状态回写（W2.5/W3 收官）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T01** py_list ADD 双重释放修复：`crates/auto-lang/src/vm/engine.rs` 删除
  :5814-5815 第二对 release（保留 :5796-5797），旁注对照 STR_CAT :3950-3951 单次纪律。
  验证：`cargo check -p auto-lang` && `cargo t vm` && `cd parity && cargo run -- run py_list`（8/8、无 P053-8）。
- [ ] **T02** ADD rc 配平回归单测：engine 测试模块加 string-concat rc=1 操作数用例
  （断言 phantom_drops==0、拼接值完整）。验证：`cargo t engine`（或最小模块名）。
- [ ] **T03** py_sys namedtuple opaque 分流：`crates/auto-lang/src/py_ffi.rs` tuple 拍平臂
  :2273-2292 入口判 `_fields` → PyObjectHandle encode。验证：`cd parity && cargo run --
  run py_sys`（5/5）&& `cargo run -- run py_list`（无回归）。
- [ ] **T04** p7 全相位归零 + 登记核销：`cd parity && cargo run -- phase p7` 全绿；
  `docs/plans/KNOWN-DEBT-AND-RISKS.md` P539-D1、P560-D6 标记 resolved→PLAN-567。
- [ ] **T05** D3 前缀 helper 与迁移：`py_ffi.rs` 新 `py_exc_msg`（对齐 :774 构造），
  全部 `VMError::FFI("Python … failed: …")` 构造点迁移（:81/:117/:136/:177/:450/:599/
  :650/:706/:926/:963/:1087 等逐点）；`vm/engine.rs` CALL_NAT 臂 :7696-7698 补
  FFI→RuntimeError 转换（对齐 :7732-7734）。验证：`cargo t py_ffi` + 既有 catch
  语料载荷断言更新后绿。
- [ ] **T06** may 变体补桥：`py_ffi.rs` 新 476 py_getattr_may / 477 py_getitem_may
  （照 :761-781 范式）+ `lib.rs` BIGVM_NATIVES 注册 + `vm/codegen.rs`:5279-5293 常量
  镜像 + `trans/python.rs` `_auto_may` 反映射（对齐 :1028-1050）。验证：`cargo t
  py_ffi` + 新 shim 单测（Err 载荷 `PyException AttributeError:` 前缀断言）。
- [ ] **T07** P539-D5 kwargs×may：探针定 ABI 后 py_call_kw 形态可走 may 出口（最小：
  kwargs 5 槽约定下复用 Err 出口）；scratch 探针 + py_ffi 单测。验证：`cargo t py_ffi`。
- [ ] **T08** 引擎值通道拦截：`vm/engine.rs` ERROR_PROPAGATE Result.Err 分支
  （:3454-3458）帧展开前查 handler_stack → 跳 catch_pc 绑载荷（对齐 intercept_error
  :9486-9489 取原文）；null 分支不进 catch。验证：`cargo t engine`（新增 try 体内
  Err 传播 → catch 命中单测 ×2：命中/null 不命中）。
- [ ] **T09** s2s 隐式传播规则：`trans/s2s_rules.rs` 新 `rule_err_propagate`（桥调用→
  may+.?、同文件/use 导入 fn 调用→+.?；分支不敏感单遍）+ `trans/auto_s2s.rs`
  builtin_rules 注册 + 改写快照单测（含 `.at` 不改写反例）。验证：`cargo t trans`。
- [ ] **T10** .as 语料与 main 边界：vm file 语料新增（隐式传播穿透 / catch 拦值 /
  未捕获带错退出）；核对退出码非零。验证：`cargo tv`。
- [ ] **T11** D2/D3 债务核销 + 设计回写：KNOWN-DEBT P560-D2、P560-D3 resolved→
  PLAN-567；`docs/design/strategy/script-mode-interop.md` §3-E/§4 现状注记。
  验证：文档 diff 自查。
- [ ] **T12** parser with-as：`parser.rs` `with_header` 旗标 + Cast 臂 :2605-2611
  截断 + with_stmt 消费 `as Ident`（:1258-1267 拒绝臂删除）+ 单测（接受/as-类型名
  歧义/正常模式 Cast 回归）。验证：`cargo t parser`。
- [ ] **T13** py_enter 返回值 + 块形态：`py_ffi.rs` enter_shim :1146 encode_null →
  `py_auto_marshal_return`；`parser.rs` with_stmt as 形态直产 `var __w/py_enter/try/
  finally/py_exit` 块。验证：`cargo t py_ffi` + `cargo t parser`（产物快照）。
- [ ] **T14** a2py 回译：`trans/python.rs` 规范块序列模式匹配 → `with e as x:`
  （py_with 1 参映射保留）+ 快照单测。验证：`cargo tt`。
- [ ] **T15** with-as 三方用例：`parity/libs/python/py_torch_infer/tests/auto/infer.as`
  增 as 形态用例（no_grad as g 或 open 句柄模拟）+ oracle 对拍件同步；vm file 语料
  加"体内 Err 传播后 py_exit 必执行"可观测用例。验证：`cd parity && cargo run --
  run py_torch_infer` && `cargo tv`。KNOWN-DEBT P560-D1 核销。
- [ ] **T16** PyType 扩展 + 内省：`py_ffi_types.rs` PyType 增 Float/Int/Nullable；
  `py_ffi.rs` 新 `inspect_return_annotation()`（get_type_hints + Union/Optional 解析，
  失败回退 None）+ 单测（注解模块命中/builtins 空/`T | None`/`Optional[T]`）。
  验证：`cargo t py_ffi`。
- [ ] **T17** 注册接线：`lib.rs` init_py_ffi :677/:710 两路径 + `autovm_persistent.rs`
  :180/:204 升级 PySignature.returns；常量通道 :511-545 同法。验证：`cargo t`（注册
  相关单测 + 全量快测无回归）。
- [ ] **T18** D4 授权强制：shim 出口 coercer（Float→GIL float()；新 478 py_int 三处
  注册镜像 + a2py `int(x)` 反映射）；无注解路径零变化断言。验证：`cargo t py_ffi`
  + `cargo tt`。
- [ ] **T19** nullability lint：`error.rs` 新 W 码（W0010 py-nullable-unguarded）+
  codegen 消费（nullable 返回未判空直入二元运算/方法调用告警）+ 触发/不触发单测。
  验证：`cargo t vm_codegen`（或最小模块）。
- [ ] **T20** py_anno parity 套件 + 收官回写：`parity/libs/python/py_anno/`（本地
  anno_mod.py 6-8 用例 + anno.as + README）+ `parity/crates/auto-parity/src/main.rs`
  p7 相位表注册；`cd parity && cargo run -- phase p7` 全绿；script-mode-interop.md
  §9 波次状态收官注记 + specs 沉淀准备（review 阶段填 frontmatter）。验证：
  parity 三方报告 + `cargo tf`（合入前一次）。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

1. **with-as 绑定表面**（P560-D1 裁定）：本计划推荐 **pratt 截断保留 `as`**（与
   Python 表面逐字对齐：`with open(f) as fh:`；设计 §10 已裁 `with expr as x` 形态，
   换关键字 `with e -> x` 属设计修订）。若用户偏好换关键字，T12/T13/T14 相应改。
2. **D2 第一批传播作用域**：推荐 调用+索引+属性（桥面 s2s 改写）；null 运算数与
   算术 TypeError 的隐式传播归 W0 null 家族运行期面（`null+1`→TypeError→Err），
   本波不做 s2s 化。全量扩面（含运算）由用户裁定后另批。
3. **lint 级别**：推荐警告级（W 码、不阻断）——oracle 定位是授权不是地基，阻断级
   会在注解撒谎时误伤。若要阻断级需改 E 码门禁族并加逃生开关。
4. **478 py_int 桥**：推荐本波新建（D4 int 承诺的落点，三处镜像成本小）；若砍掉
   则 D4 强制限 float 面注解。
5. **py_call_may kwargs（T07/P539-D5）**：最小实现（kwargs 5 槽复用 Err 出口）若
   ABI 探针发现不兼容，允许降级为"py_call_kw 严格 + 文档规避"并保留债务（需用户
   确认降级案）。
