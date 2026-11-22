# Python 互操作（av.python/1，AV 切片）

> 实验性 typed 执行能力；来源 PLAN-1026。实现分三层：ACE 共享前门/受检语义
> （`experimental/ace/crates/{core,source-query,lowering}` 的 py 面）、AV 执行与
> 宿主准入（`ace-av-core`）、可选 CPython 适配器（`ace-abi::rustside::py_bridge`，
> 967 B-01 构建闭包 + pinned PyO3，公开产品入口 `ace_abi::py_session::PySession`）。
> 普通（无 `use.py`）程序不构建/不加载 CPython。不改变旧 AutoVM、默认 CLI/UI/CTE，
> 也不代替 967 的四端义务或 M2 退役分母。

## 1. 身份与分层

本切片**不新增 ABC profile、wire 世代或 MIR/ABC 版本轴**：py 程序走既有 `Closures`
族（aggregates 单元 + foreign 调用），`FOREIGN_CALL`/`FOREIGN_CALL_CB` 的编解码族门
由 `ForeignScalar|Owned` 扩为 `ForeignScalar|Owned|Closures`（opcode、payload 布局与
wire 版本不变）。唯一变化的身份轴是 lowering 契约版本：`MIR_LOWERING_VERSION = "5"`
（见 [hir-lowering-boundary](../auto-mir/hir-lowering-boundary.md)）。

分层（单向依赖，禁止反向）：

```
Auto 源/use.py → 统一 Schema/CST → 共享受检 HIR（Python 外部对象与操作/效果）
  → 唯一共享 HIR→MIR lowering → AV foreign stub → 967 自动生成的 C 兼容 shim
  → Rust 适配器（非泛型公开入口）→ PyO3 → CPython C API
```

`av-core`、共享 HIR/MIR 与 `foreign-contract` **不依赖 PyO3/CPython**；适配器不依赖
AV 解释器或 964 的动态执行实现。适配器只持真实 `Py<PyAny>` 与解释器身份，不建第二套
资源/generation 账本（唯一账本是 967 的统一资源 owner）。

## 2. 源前门与标准语法

- `use.py <点路径>: <成员清单>`（多导入、成员按源序）在 **parse 与 resolve 之间**做一次
  纯文本/结构变换，仅作用于 aggregates py 通道（程序含 `use.py` 时启用）：注入冻结的
  运行时 helper（`__py_dlen`）并为每个成员前置一条 `var <member> = py_import(<静态帧>)`，
  **入口处按源序各执行一次**（R-01/V-03）。没有 `use.py` 的程序特征零改动。
- `py_locals` 域纪律：resolver 只对"初始化器是 `py_import` 外部调用的 i32 局部"应用
  py 操作糖；非 sanctioned 用途具名拒绝 `source.py-domain`（不把任意 i32 当句柄）。
- 标准 Auto 操作映射（应用作者**不写** `py_call/py_getattr/py_float` 等桥）：
  调用 `f(x, k: v)`／绑定方法 `m.f()`／属性 `m.f`／整数索引 `x[i]`／四则
  `+ - * /`／`.to(float|double|int|bool|str)`／`print(py)` 显示。
- 静态侧不枚举 PyTorch 成员：成员存在性与可调用性在运行期按 Python 对象协议判断；
  不按 torch 名/变量名特判，也不要求 `inspect.signature`。

## 3. 值模型与封送

| 类别 | 表示 | 规则 |
|---|---|---|
| Python 对象（module/类/实例/Tensor/参数/优化器/迭代器/shape 等） | AV 侧 i32 token（`Value::External` 形态，provider 私有 registry 持真实对象） | 不 clone/不 tolist/不读写 storage；同对象不同引用共享身份，原地修改可见；token 不等于裸地址，也不等于 Python `is` 的 oracle |
| 精确标量 → Auto | i32 / bool / char / owned Str / 浮点 | 显式 `.to(...)`；超宽/越界按已支持类型规则拒绝，不截断 |
| 浮点过界 | bits 对（两个 i32：`py_to_f64` 取高位、`py_f64_lo` 取低位），Auto 侧由 `__py_f64_bits` 逐字 u32 零扩展 + `±1.0` 乘法取负精确重建 | 非正规/非有限值在 provider 侧具名拒绝（类 23）；不引入新 ABI tag |
| 文本显示 | `py_stage_str`（ASCII ≤256B 冻结）→ `py_str_word` 逐字读回 | 超界明确拒绝，不静默截断 |
| console 输出 | 公共 `auto.console.print/eprint.{f64,f32,i32,bool,str,char}.v1` | 由 947/972 的公共输出面消费（见 [host-import-and-console](host-import-and-console.md)）；非 Python 专用通道 |

Tensor 保留计算图：`.to(float)` 只是显式标量提取，不提前把 Tensor 降为标量；多元素
转换明确失败且原对象仍可用。

## 4. 操作集（v1 冻结，17 个帧操作）

`p967.frame/1` 帧（8 位置实参 / 8 关键字 / 4096B 文本 / 64B 名字上限，`MAX_CALL_BYTES=65536`、
`MAX_RUN_BYTES=1048576` 沿 967 交付）：

| symbol | 形态 | 错误码/类 |
|---|---|---|
| `py.admission.v1` | fallible | `py.init:-100` |
| `py.import.v1` | fallible | `py.import:-101` |
| `py.getattr.v1` | fallible | `py.attr:-102` |
| `py.call.v1` | fallible | `py.call:-103` |
| `py.binop.v1`（v1：`add/sub/mul/truediv`） | fallible | `py.binop:-104` |
| `py.getitem.v1` | fallible | `py.index:-105` |
| `py.to_f64.v1` / `py.f64_lo.v1` | fallible / pure | `py.to-f64:-106` |
| `py.to_i32.v1` | fallible | `py.to-i32:-107` |
| `py.to_bool.v1` | fallible | `py.to-bool:-108` |
| `py.stage_str.v1` / `py.str_word.v1` | fallible / pure | `py.stage-str:-109` |
| `py.error_stage.v1` / `py.error_class.v1` | pure | — |
| `py.release.v1` | fallible | `py.release:-110` |
| `py.alive.v1` | pure | — |
| `py.session_close.v1` | fallible | `py.close:-111` |

异常→类映射（`py_error_class`）：0 none、1 import、2 attribute、3 type、4 index、
5 value、6 overflow、7 key、8 zero-div、9 runtime、10 other；桥自身类 20 domain
（坏帧/坏 op）、21 dead（未知/已释放 token）、22 text、23 float boundary。错误文本经
`py_stage_str` 暂存后读回，映射到 965 的语言错误/trap 体系（含源码位置），不产
0/-1/裸 ID。

## 5. 生命周期与准入

- 产品入口 `PySession`：每次 run 编译（aggregates foreign 入口）→ 受检 lowering →
  foreign ABC 准入 → console+foreign 合并执行；会话串行（单解释器、单 owner 线程，
  v1 支持边界）。`close()` **只释放本桥自有引用**，绝不 `Py_Finalize`（解释器是宿主
  基础设施，R-06）；重新 open 即新 generation。
- 适配器 cdylib 的构建产物缓存在 OS 临时目录，跨进程共享、加锁且幂等（热快路不重编译
  pinned PyO3）。
- 准入：provider/解释器身份在 Python 副作用前检查（缺 CPython/缺 provider 具名失败，
  不静默跳过）；console import 在非 profile-8 但**携带 foreign 调用**的模块内准入
  （见 `host-import-and-console.md`）；未声明 capability 一律拒绝。
- 缓存/可移植：code cache 只含声明与依赖身份，不存 PyObject/地址/活句柄/旧授权；
  活 Python 对象的 portable freeze 明确 Unsupported（不塞 pickle）。

## 6. 支持边界（v1 冻结）

单进程受信任 CPython（3.14.x，GIL 标准构建；free-threaded 与其他版本具名拒绝）、
Windows x64 冻结环境（CPython 3.14.2 / PyO3 0.29.3 / torch 2.14.0+cpu）、CPU、
串行训练会话。**非目标**：跨线程/回调中的 Python 抢占、async/await 中的 Python、
GPU/CUDA、任意 Python 包零配置、模型序列化/热迁移、全部 Python 协议、静态推断
任意动态反射、性能优化。

已知 v1 前端边界（不得当成已支持，也不得当成已证拒绝）：嵌套调用结果直接作位置实参
**未验证**（`py_is_object` 接受内联 py-op 结果、`py_frame_val` 可封送此类值，故要主张
拒绝须先补反例）；动态浮点关键字实参**已证拒绝**（`source.py-domain`）。文本显示
ASCII ≤256B；非正规/非有限浮点不可过界。

## 7. 证据与消费方

- 固定组 `cargo t ace-python`（`-p ace-abi --test conformance -E test(p1026)`，10 例：
  训练验收、语料双向身份、close/reopen、math/torch ABI 片段、F-1 位对重建、F-2 步进
  预算、V-09/10 身份与动态状态、通用 float console lane）。
- 语料与独立参考：`crates/auto-lang/test/cases/ace-m2/python-1026/`（`pytorch_train.as`
  + README + 五处表面偏差说明）；独立原生 Python 参考数值（first=`0.2370201051235199`、
  final=`0.1013089045882225`、pred0=`0.4728480279445648`、shape=`torch.Size([4, 1])`）。
- 消费的上游合同：964 IF-VAL（外部值/别名/清理钩子）、967 IF-CALL/IF-RES（带 tag 帧、
  generation/owner/pin/drop、失败清理）、965 错误与控制流、947/972 公共输出。
- 未交边界：新 A2PY 与其他后端的 Python 运行适配未实现（缺 provider 具名拒绝，不静默
  调用旧引擎）；AV 切片不等于 967 四端或 M2 退役全量通过。

来源：[PLAN-1026](../../plans/archive/1026-python-interop-standard-auto-syntax.md)。

## 附：PLAN-1107 追加（同源双后端与操作数单次求值）

> 本节追加到 `docs/specs/auto-av/python-interop.md`，不改写 1026 的 v1 冻结事实。

1. **同源**：`use.py` 前门同时服务 AV 切片与 A2PY 目标；py 操作在**操作发生处**记录为
   结构化事实（种类/成员/操作数角色/span），AV 用帧文本、A2PY 按事实 lowering，二者
   不各自重解析。声明单一来源是 `ace-foreign-contract::py_ops`（`p1107/py-language-contract/1`），
   abi 一致性测试把 AV provider 声明与中立契约钉住。
2. **操作数单次求值（实现约束）**：帧里的动态操作数经 `__py_chunk_i/_o/_b(n)` 一次求值
   生成 `"tag <len>\n<value>\n"`。旧实现把同一操作数写两次（`dlen(expr)` + `expr`），
   内联 py 调用因此被求值两次：token ≥ 10 时声明长度与 payload 不符（`payload not
   LF-terminated` / `expected the positional count line`），token < 10 时静默重复副作用。
   `p1107_v02_av_frame_operand_is_evaluated_once` 与 `..._inline_side_effect_is_evaluated_once`
   是该约束的回归。
3. **失败面**：未捕获 Python 异常 = `Trap{ForeignStatus}`，span 覆盖失败调用；运行不返回
   Success，stdout 不发布部分输出（`p1107_v04_…`）。
4. **本轮实际消费的源面**：`use.py` 多成员、`l.append/count/pop` 绑定方法、嵌套调用、
   关键字实参（如 `round(2.5, ndigits: 0)`）、循环/分支内的 py 值流与 `.to(int/bool/float/str)`。
   仍未交：容器/记录跨边界、动态浮点实参、属性派生值。
5. **AV 帧 helper 不再被源生成目标发射**：`__py_dlen`/`__py_chunk_*` 只服务 AV 帧文本，
   A2PY 与 978 组装按 `is_py_lane_helper` 过滤，避免为生成目标引入不可 lower 的帧构造。

## 附：PLAN-1094 追加（普通 AV 脚本面，不改写 v1/1107 冻结事实）

> 本节追加到 `docs/specs/auto-av/python-interop.md`，1026 的 v1 冻结事实与 1107 附录
> 保持原样；1094 交付的是**普通 AV 语言入口**（`ac-av/language-source-run-v1` 带
> `foreign: {kind, provider: "py"}`）的脚本/Python 对象协议，供 1110 A2PY 按同源
> 事实接续。

1. **普通入口接线**：带 `use.py` 的 auto-script/.as 源经 `compile_language_source_with_foreign`
   的 front_door（helper 注入＋hoisted `var <member> = py_import(frame)`）编译，随后
   `verify_foreign_closures` → ABC → `execute_console_foreign`（`bind_python_error_observers`
   附加）。多成员 `use.py builtins: int, str` 按源序形准入。无 `use.py` 的纯 Auto 源
   走既有普通链，**不加载 CPython**。
2. **py 对象分派**：`.len()` 在 py 对象上路由 `__len__`（numpy 首轴 6 等），纯 Auto
   str/List `.len()` 保持原生计数（6/4/7）——旧 obj_call 对 Auto 值的假阳性**不复制**。
3. **异常分类**：失败 py 操作按 provider class 映射 `LanguageErrorKind::Python(class)`
   （1..=10 语义类别），worker 渲染 `python.{class}`；未捕获 = 语言错误运行终态
   （`lang.error`＋原 `source_span`），**取代** 1107 附录 §3 的 `Trap{ForeignStatus}`
   仅适用于该冻结的 aggregates/A2PY 面——普通 AV 入口按本节。捕获时 handler 恰一次。
4. **文本通道升级**：`py_stage_str`/`py_str_word`/`py_error_stage` 从 v1 ASCII 字节
   升级为 codepoint-wise UTF-32-LE word（一 word 一 Unicode 标量；ASCII 输入数值
   等价），中文与内嵌 NUL 逐字节往返（`\u{0}` 规范拼写）。1026 测试 word oracle
   随通道升级同步（同 p981 grammar id 重冻结惯例）。
5. **缺解释器**：`prepare-av-providers` 写 `py-unavailable` 标记时父侧显式
   `Status::NotRun`（"never PASS"）；不因缺 CPython 把语言判红或充绿。
6. **with 语句**：`with <expr> as <name> { body }`（Python 上下文管理器）——
   `__enter__` 绑定、`__exit__` 每条出口（normal/return/break/continue/语言错误）
   恰一次、truthy `__exit__` 抑制错误、enter 失败不 exit。

来源：[PLAN-1094](../../plans/1094-av-parity-script-and-python-protocols.md)。

## 附：PLAN-1126 追加（NumPy 与 json 库实测，不改写 v1/1107/1094 冻结事实）

> 本节追加到 `docs/specs/auto-av/python-interop.md`，1026 的 v1 冻结事实与 1107/1094 附录
> 保持原样。1126 没有新增受检操作、provider op 或索引值类别；以下是把 NumPy 2.5.1 与
> 标准库 `json` 经普通 AV 入口真实运行后实测到的**支持边界**。

1. **索引值类别（既有 `py.getitem.v1`）**：`obj[k]` 的 `k` 可以是 int、文本字面量，或产生
   Python 对象的表达式（如 `row[slice(0, 3, 2)]`，`slice` 经 `use.py builtins: slice` 导入）。
   provider 侧一律 `obj.get_item(key)`，不按键类型分派。
2. **属性派生对象**：`a.shape` 的结果是普通 Python 对象（AV 侧 token），可以继续索引或调用
   （`shape[0].to(int)`、`a.sum()`）。属性结果不进入 Auto 值域，容器/记录仍不跨边界。
3. **标量与文本提取**：`.to(int)` 对 NumPy 整数标量、`.to(float)` 对 NumPy 浮点标量、
   `.to(bool)` 对 Python `bool`、`.to(str)` 对含非 ASCII 的 `str` 成立；
   `json.dumps(..., ensure_ascii: false)` 的 UTF-8 文本经 codepoint 文本通道逐字往返
   （上限仍是 256 codepoint）。
4. **异常类别**：provider 的 `record_py_err` 按类型 qualname 精确匹配——
   `json.JSONDecodeError` 不在类表内，归 10（other），不按 `ValueError` 子类归 5。未捕获时是
   `lang.error` 终态，不发布部分输出，`source_span` 是失败调用表达式的原源字节范围。缺
   NumPy 时 `py_import` 抛 `ModuleNotFoundError`（类 1），同样不跳过。
5. **证据**：`crates/auto-lang/test/cases/language/script/python-libraries.case.ad` 的 3 条 AV
   检查，由 `case-av-language-parity` 运行（Windows x64 / CPython 3.14.2 / NumPy 2.5.1）。

来源：PLAN-1126。
