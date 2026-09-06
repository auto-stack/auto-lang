---
plan_id: PLAN-567
status: archived                 # drafting → executing → execution_done → reviewed → archived
feature_name: script-mode-w25-tail-w3-oracle
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05   # executing 起算（worktree D:/autostack/.wt/lang-567/auto-lang, branch plan-567-dev）

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "auto-lang/trans/design/s2s-lowering.md: 规则表新增 E1 隐式传播（桥调用 may 化+.?）与 with-as 块形态回译契约"
  - "auto-lang/frontend/overview.md: with 条目扩展（as 绑定形态/pratt 截断）"
  - "auto-lang/vm/overview.md: Err 值通道拦截 catch/主边界带错退出/ADD 先读后放池纪律/PyException 前缀族"
new_spec_components:
  - "auto-lang/trans: W3 注解预言机（inspect_return_annotation/PY_RETURN_ANNOTATIONS 表/D4 授权强制/W0010 lint）"
  - "auto-lang/vm: py 桥 may 变体族 453/476/477/478 + py_raise 479 + py_int 480 + py_enter 返 __enter__ 值"
  - "parity/python: py_anno 套件（本地注解模块 + runner PYTHONPATH 绝对路径注入）"
touched_goals:
  - "GOAL-005: Python parity 第三维度——脚本模式 W2.5 收尾+W3 收官，P539-D1/D5、P560-D1/D2/D3/D6 六债清偿，20 套件 173 用例三方全绿"
  - "GOAL-001: 语言核心成熟——字符串池 rc 先读后放纪律修复（P053-8 幻影根因）+ CALL_NAT/COUNTED 非 FFI 错误传播加固"

affects: [auto-lang/vm, auto-lang/trans, auto-lang/frontend]   # 受影响的 specs 路径
current_step: 20
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

- [✅ 已完成] **T01**（2026-09-05）py_list ADD 修复：根因比计划预判深一层——双重释放之外，
  ADD 臂 release 在读之前，左结合链中间结果槽被 FREE+tombstone 后 read 归空（池日志实证
  retain"ab"0→1→release1→0→FREE→intern"cd"）。修复=先读后放（对齐 STR_CAT 纪律）+删第二对
  release。证据：`parity run py_list` 8/8（commit 83ae4e9a6）；探针 scratch/p567/t01_probe*.as
  j2 "cd"→"abcd"。
- [✅ 已完成] **T02**（2026-09-05）回归单测：engine.rs 新 tests_add_concat_rc 模块 ×2
  （rc1_operands / chain_intermediate_survives，断言 underflow==0 && phantom==0）。
  证据：`cargo t tests_add_concat_rc` 2/2 PASS。
- [✅ 已完成] **T03**（2026-09-05）namedtuple opaque 分流：判别式修正为"带属性面 tuple"
  ——`_fields`（namedtuple）+ `n_sequence_fields`（PyStructSequence；sys.version_info 实测
  无 `_fields`）双探测，普通 tuple 维持拍平。证据：py_sys 5/5、py_list 8/8；三分支封送单测
  `test_marshal_structseq_namedtuple_opaque_plain_tuple_list` PASS（`cargo test -p auto-lang
  --lib --features python`，py_ffi 测试需 python feature）。commit 056eabd7a。
- [✅ 已完成] **T04**（2026-09-05）p7 全相位归零：8 套件 55/55 三方全绿（configparser 5/
  hashlib 5/json 5/list 8/os 8/re 8/string 8/sys 5）；KNOWN-DEBT P539-D1、P560-D6 核销
  （worktree e69f865b8）；known-divergences.md DIV-PY-TUPLE-1 补注带属性面 tuple 不拍平裁定。
- [✅ 已完成] **T05**（2026-09-05）D3 前缀统一：py_exc helper + ~20 站点迁移（桥内部组参类
  保留描述形态）+ CALL_NAT 臂 FFI→RuntimeError 补齐。证据：探针 catch 载荷
  `PyException ValueError:`/`PyException IndexError:` 前缀；`cargo test --features python
  py_ffi::` 32/32。commit（T05 批）。
- [✅ 已完成] **T06**（2026-09-05）may 变体补桥：476/477 + 共享 Err 出口 helper（py_call_may
  原地版同源化）+ 三处注册镜像 + a2py `_auto_may` 反映射 ×2 + 前缀单测。证据：
  test_may_variant_shims_err_payload_prefix PASS。
- [✅ 已完成] **T07**（2026-09-05）kwargs×may：478 py_call_kw_may（同 py_call_kw 5 槽 ABI，
  a2py 无臂对齐 py_call_kw 现状——a2py 走糖源）+ Ok/Err 双路单测 + 端到端探针（Err→fallback
  通）。P539-D5 核销。注：`.?(d)` 表达式位链式消费为存量缺陷（453 同形复现）→ 待澄清⑥。
- [✅ 已完成] **T06**（2026-09-05）may 变体补桥：476 py_getattr_may / 477 py_getitem_may
  + 共享 Err 出口 helper（push_py_exception_err_value，py_call_may 原地版同源化）+ 三处
  注册镜像 + a2py `_auto_may` 反映射 ×2 + Err 载荷前缀单测（test_may_variant_shims_
  err_payload_prefix PASS，含 477 的 Ok 包裹形态）。
- [✅ 已完成] **T07**（2026-09-05）P539-D5 kwargs×may：478 py_call_kw_may（与 py_call_kw
  同 5 槽 ABI；a2py 无臂对齐 py_call_kw 现状——a2py 走糖源）+ Ok/Err 双路单测
  （test_py_call_kw_may_kwargs_combo PASS）+ scratch 端到端探针（Err→`.?` fallback 通；
  var 中转形态 Ok 正确）。`.?(d)` 表达式位链式消费为存量缺陷（453 同形复现）→ 待澄清⑥。
- [✅ 已完成] **T08**（2026-09-05）引擎值通道拦截：ERROR_PROPAGATE Err 分支查
  handler_stack（bp 匹配）→ 弹 handler、栈回卷 handler.sp、载荷串入栈、跳 catch_pc；
  null/None 不进 catch。证据：tests_err_value_channel 3/3（命中/null 不命中/Ok 解包）。
- [✅ 已完成] **T09**（2026-09-05）s2s 隐式传播规则：rule_err_propagate（MAY_RENAMES
  py_call/getattr/getitem→may+.?；同文件用户函数调用+.?；无 use.py 零改写保 legacy -1
  哨兵面）+ codegen is_py_call_kw_form 收 py_call_may→478 + wrap_tos_as_result_ok 补
  TAG_LIST/null 臂（`.?` 解包 list 归 Int 垃圾的 p7 四套件中止根因）。证据：tests_s2s
  8/8（含语料幂等）；**全部 19 py 套件 127/127 三方绿（隐式传播激活态）**。
- [✅ 已完成] **T10**（2026-09-05）主边界 + 语料：未捕获 Err → RuntimeError(PyException
  载荷) 退出码 1（master 对照原为静默 Terminated）；tv 框架 .as 语料支持（.at 优先回落）
  + 99_script_err 三例（传播穿透/catch 拦值/未捕获带错，python feature 门控）3/3；
  顺修 CALL_NAT/COUNTED 臂 `if let Err(FFI)` 静默吞非 FFI 错误回归（alloc_array/
  Config.parse 二例实证，master 对照定位）。证据：`cargo tv --no-fail-fast` 3606/3607
  （唯一红=charts master 既有）。
- [✅ 已完成] **T11**（2026-09-05）D2/D3 债务核销 + 设计回写：KNOWN-DEBT P560-D2、
  P560-D3 标 ~~已清偿~~（Err 通道汇合落地实证）；script-mode-interop.md §4 落地现状
  注记 + §3-E2 进度标注（worktree commit 546c9d275）。证据：grep 六债全 ✅。
- [✅ 已完成] **T12**（2026-09-06）parser with-as：with_header 旗标 + As 终止符 + Cast 臂截断
  + `as Ident` 消费 + 单测 ×3（块形态/as 类型名不误吞/正常模式 Cast 零回归）。实现注记：块语句
  以 Stmt::Expr(Expr::Block) 承载（躲 convert_last_block 尾块转换）；convert_last_block 收窄为
  纯 pair 块才转对象（语句块保留）；emit 以 `if true` 恒真包装多语句块（E0007 `)`+换行+`{`
  歧义与 UI 尾块歧义全免疫，幂等实证）。479 py_raise 再抛通道配套。
- [✅ 已完成] **T13**（2026-09-06）py_enter 返 `__enter__` 值 + 块形态降低落地：try-catch-finally
  出口保证（正常路径 finally、Err 路径 T08 拦截→catch py_exit+py_raise 再抛、`__exit__` 恰一次）。
  端到端实证：open 句柄正常写 flush（f1）+ 错误路径部分写 flush（f2）+ exit 1 + PyException
  ValueError 外传（scratch/p567/t13_with_as.as）。
- [✅ 已完成] **T14**（2026-09-06）a2py 回译：match_with_as_block 规范序列识别 → `with e as x:`
  （transpile 产物逐字对齐 Python 原生）+ a2p golden 567_with_as/001_with_as（test_16_002 绿）。
- [✅ 已完成] **T15**（2026-09-06）三方用例：py_torch_infer test18（with-as 糖形态 no_grad）18/18
  三方绿；tv 99_script_err/04_with_as_propagate 语料（catch 还原链路）绿。执行注记：master 同步
  折入 Plan 568 测试拆档（merge 58fd18229，vm_file_tests 冲突双并解），合并后 cargo t 零新增红
  （19=基线）、tv/tt 唯一红=charts 基线、p7/p9 全绿。P560-D1 核销（worktree 侧 KNOWN-DEBT）。
- [✅ 已完成] **T16**（2026-09-06）PyType 增 Nullable(Box) + 全局 PY_RETURN_ANNOTATIONS
  表/lint 收集器（py_ffi_types，无 pyo3 依赖，无 python feature 恒空）+ inspect_return_
  annotation（classify_return_annotation：标量恒等 + T|None/Optional[T] 双形态剥 None）+
  常量通道 inspect_constant_annotation + 单测（分类器 4 断言组全过）。
- [✅ 已完成] **T17**（2026-09-06）注册接线：init_py_ffi 三路径（命名导入/裸模块发现/常量）
  灌注定签名 + record_return_annotation；codegen handle_py_import 按 oracle 表灌注
  py_return_types（无知识=Auto 零变化）。
- [✅ 已完成] **T18**（2026-09-06）D4 授权强制：shim 出口 Float/Int 臂改 GIL float()/int()
  强制（coerce_scalar_* helper + 单测：int→float/str→float/float→int/不可转 PyException）
  + Nullable 臂（None=null 值；内型走强制/动态）+ 480 py_int 桥（三镜像 + a2py int(x)）。
  注：478/479 已被 kw_may/py_raise 占用，py_int 顺延 480。
- [✅ 已完成] **T19**（2026-09-06）nullability lint：W0010 py-nullable-unguarded——语句位
  裸消费（Store 根裸 Call）形态匹配 → log::warn + 收集器去重（单测过）。实现注记：error.rs
  W 码体系挂 parser 面，567 以 log+收集器落地（等价可见性，review 时可议挂 W 码）。
- [✅ 已完成] **T20**（2026-09-06）py_anno 套件：本地注解模块 anno_mod.py（含撒谎注解
  fakey）+ anno.as/test_anno.py 九用例 + parity runner PYTHONPATH 注入（**绝对路径**——
  相对路径被子进程 cwd 二次解析的坑，双后端）+ p7 相位注册。证据：py_anno 9/9 三方绿；
  p7 九套件 64/64；fakey 用例实证撒谎注解退回 Python 行为；设计 §7/§9 收官注记
  （commit 11b8946ad）。

## 复审记录

**复审人/时间**：ZCode（auto-plan:review）· 2026-09-06 · worktree `D:/autostack/.wt/lang-567/auto-lang`（branch `plan-567-dev`，复审期新增 3 提交）

**逐条验收裁定**（verify, don't trust——全部复审期重跑实证）：

1. **p7 三方全绿**：✅ PASS——复审重跑 p7 相位 9 套件（含 py_anno）全 100%；py_list 8/8、
   py_sys 5/5；`P053-8/phantom` 在 py_list 直跑中零出现。
2. **零回归门禁**：✅ PASS——`cargo tf` 3449/3450、`cargo tv` 3589/3590、`cargo tt` 3796/3797
   （三者唯一红 = charts，master 既有基线红，master 同形复现）；`cargo t` 失败集与 master
   环境基线族零差（d2/d8 为 plan370 环境闪断，双向漂移）；p5/p6/p8/p9 四相位 20 套件
   三方全绿（173 用例）。
3. **Err 通道**：✅ PASS——catch 载荷 `PyException ValueError:`/`PyException IndexError:`
   双前缀探针实证；未捕获 Err exit=1 + PyException 消息；`.at` 含 use.py 仍被 E5501
   硬门拒绝（门禁不放松实证）。
4. **with-as 三方一致**：✅ PASS——open 句柄探针：正常路径 finally flush（f1=
   hello-with-as）、错误路径 catch flush（f2=partial）、exit=1；infer test18 18/18
   三方；`a as int` 正常模式 Cast 零回归（parser 单测）。
5. **注解预言机**：✅ PASS——py_anno 9/9 三方（含 fakey 撒谎注解退回 Python 行为）；
   分类器/强制 helper/lint 去重单测全过。
6. **债务核销与回写**：✅ PASS（复审修复后）——P539-D1/D5、P560-D1/D2/D3/D6 六条
   全部 ✅ 已清偿；known-divergences DIV-PY-TUPLE-1 补注；script-mode-interop §4/§7/§9
   收官注记。

**遗漏/延后/workaround 猎查**（Step 3 显式猎查结果）：

- **遗漏 ×2（复审当场修复，commit "复审遗漏双修" + a0a9dda8e）**：
  ① P539-D5 债务条目未核销（T07 已交付 478 py_call_kw_may 但登记漏做）——已补核销；
  ② `autovm_persistent` REPL 两注册路径未接预言机（T17 计划文本点名 :180/:204，
  执行只接了 lib.rs 三路径）——已补齐（注解签名 + 知识表记录，与 init_py_ffi 对齐），
  修复后 cargo t 零新增红 + p7/p9 复跑全绿。
- **偏差（已登记 P567-R1/R2 债务候选）**：T19 lint 走 log::warn+收集器而非 error.rs
  W 码体系（计划文本写 error.rs——该体系挂 parser 面，codegen 接线超收口预算，
  可见性等价 + 单测在案）；CLI 错误路径丢已缓冲 stdout（存量，master 同形实证）。
- **号位偏差**：py_int 落 480（计划写 478——478/479 被 kw_may/py_raise 顺延占用），
  执行注记在案。
- **存量缺陷登记**：`.?(d)` 表达式位链式消费缺陷（453 同形复现）——计划待澄清⑥。

**裁定：PASS → status: reviewed**。无阻断债；P567-R1/R2 为低风险偿还路径明确的
登记债。

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
6. **（执行期新登记，2026-09-05）`.?(default)` 表达式位链式消费缺陷（存量）**：
   `j.?(-1).to(str)`（及加括号形态）在表达式位置返回 Result 容器原 bits 的 i32 解码
   （如 4000001），var 中转（`var d = j.?(-1)`）则正确——py_call_may(453) 同形复现，
   **非 567 引入**。规避：var 中转（539 套件既有惯例）。病面：NULL_COALESCE 的
   codegen 发射顺序或 `.?(` 解析歧义，待专项排查；本波 s2s 隐式传播用纯 `.?`
   （ERROR_PROPAGATE）形态不受影响（B 探针实证 `j.?` 正确）。
