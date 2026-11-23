# Python 生态集成（ace.python/2，AV ＋ A2PY 同源）

> 来源 PLAN-1107。本文是**已交并实测**的 Python 集成面：同一份普通 Auto 源经同一个
> 共享前门（`use.py` 前门 → 受检 HIR → 共享 A2X 视图）分别由两条链真实运行——
> **AV 切片**（`ace_abi::py_session::PySession`，进程内 CPython，967 provider）与
> **A2PY 目标**（生成纯 Python 3 文件集，在声明的 OS 解释器里原生运行）。
> 二者共享**同一份 py 操作事实**，不存在第二解析器/类型检查器，也不存在「AV 兜底 A2PY」
> 的运行路径。982/1042/1026/977/978 的既有冻结事实仍然有效，本文只追加本次实际消费面。

## 1. 单源、单前门、双后端

```text
main.at (use.py …)
   └─ ACE 共享前门（source::compile_aggregates_source_with_foreign）
        ├─ py_ops 事实：操作种类/成员/操作数角色/span（操作发生处记录）
        ├─ AV 后端：帧文本 p967.frame/1 → FOREIGN_CALL → 967 provider → CPython
        └─ A2PY 后端：按事实 lowering → generated.py + a2x_py_runtime.py
```

- **操作集合（17，冻结）**：`py_import`、`py_call`、`py_getattr`、`py_getitem`、
  `py_binop`、`py_to_f64`、`py_f64_lo`、`py_to_i32`、`py_to_bool`、`py_stage_str`、
  `py_str_word`、`py_admission`、`py_alive`、`py_release`、`py_session_close`、
  `py_error_class`、`py_error_stage`。
- **声明单一来源**：`ace-foreign-contract::py_ops`（中立契约，review id
  `p1107/py-language-contract/1`，content `p1107-py-ops-v1`，错误码 −100..−111），
  A2PY 只消费该表，不依赖 PyO3/provider 源码；AV provider 声明与中立契约由 abi 一致性
  测试钉住（`py_ops_1107`）。
- **求值恰一次**：帧的操作数由 `__py_chunk_i/_o/_b(n)` 一次求值生成
  `"tag <len>\n<value>\n"`；浮点操作数由 `__py_chunk_d/_f(v)` 一次求值（PLAN-1129）；内联 py 调用
  不会被求值两次（PLAN-1107 修复；此前两位 token 会出现声明长度与 payload 不符、并重复副作用）。

## 2. 两端语义（实测）

| 面 | AV 切片 | A2PY 目标 |
|---|---|---|
| 运行体 | 进程内 CPython（967 provider，pinned PyO3） | 生成的纯 Python 3，声明解释器直接执行 |
| 外部运行依赖 | 无（随 AV 宿主） | OS 解释器 + 项目声明的第三方包 |
| Python 侧入口 | `p967.frame/1` 帧 → provider 成员表 | `import` + 直接调用（无 ctypes、无 PyO3） |
| 失败 | `Trap{ForeignStatus, span}`（span 指向失败调用） | 捕获：进入 Auto `catch`（kind=4 + 消息）；未捕获：结构化语言错误（原类名 + `python.N` 类别 + 原始 span）+ 非零退出 + stderr `PyException <类>: <消息>`（PLAN-1110） |
| 资源 | AV 侧资源账/生命周期（alive/release/close） | 解释器自身生命周期；运行观测为退出码/`value:` |

- 共享用例 `crates/auto-lang/test/cases/ace-m2/python-1107/`：`pytorch_train.as`（60 步
  线性回归 + shape + 首个预测）、`protocol_1107.at`（绑定方法/嵌套调用/关键字实参/
  循环分支值流/显式转换）、`error_uncaught_1107.at`（未捕获异常）。
- 独立 oracle：手写 Python 参考（不经 ACE），逐值比对；1026 的 seed=0 数值冻结在计划 §4。

## 3. 支持矩阵（实测，2026-11-20）

| 轴 | 已验证 | 证据 | 未宣称 |
|---|---|---|---|
| OS/arch | Windows x64 | 端到端真实运行 | Linux/macOS（按 2026-11-14 裁定归专用环境） |
| AV 解释器 | 进程内 CPython 3.14.2（967 provider） | V-01/V-02 AV 运行 | 其他 3.x 版本 |
| A2PY 解释器 | CPython 3.14.2。`run_sdk`：PATH 上的 `python`，`-B`；`run-py`：`A2PY_PYTHON`（缺省依次探测 `python3`/`python`/`py -3`），`-I -B` 隔离运行（§7） | A2PY 原生运行、`run_sdk`、`case-a2py-language-parity` | 其他版本；`run_sdk` 缺解释器 → `m2build.run.interpreter-failed`，`run-py` 候选耗尽 → 具名失败 |
| 第三方包 | torch 2.14.0+cpu（CPU 线性回归）、numpy 2.5.1、标准库（list/len/repr/round、`json`） | 共享用例；PLAN-1126 `python-libraries` 语料（§7） | CUDA/其他 torch 构建；未声明包 → `m2build.run.runtime-dependency` |
| 语言面 | 17 操作、int/bool/text 字面量与 py 对象、`.to(int/bool/float/str)`、嵌套调用、kwarg、绑定方法；int/文本/Python 对象（`slice`）键索引与属性派生对象的继续索引（PLAN-1126，未新增操作）；运行时 `f32/f64` 实参与 Auto 文本值（PLAN-1129） | 端到端 | 容器/记录跨边界 |
| 目标 | AV（in-process）、A2PY（原生 Python 文件集）、978 SDK A2py | T-01/T-03/T-05 | 其他 978 目标不变 |
| 失败 | 缺解释器/包/工件/授权 → 具名 `m2build.*`；未捕获异常 → 两端非 PASS；A2PY Python 异常 → Auto 错误类型 + 原始位置（`ValueError`→`python.5`、`IndexError`→`python.4`，PLAN-1110） | V-04/V-07；1110 语料 V-01..04 | 其余异常类按 `A2xPythonError` 类别表归 `python.10`（other），映射矩阵未逐类实测验交（`json.JSONDecodeError`→`python.10` 已实测，PLAN-1126） |

## 4. 工程集成（978）

- 入口：`M2Build::build(TargetKind::A2py, …)` / `build_sdk` / `run_sdk`；`use.py` 源自动
  选择 Aggregates + 中立 py 声明契约（`cold_assemble_py`），非 Python 源走冻结 Scalar 路径。
- 身份与缓存：冷构建产片段；同输入为 `cache-hit`（`rebuilt=false`，零工具 spawn）；
  改 body 使身份变更并重产；项目名、caps、participating provider 工件都是身份轴。
- 分发：SDK 可重定位（异根运行）、缺工件/缺依赖具名拒绝，不把 traceback 计为运行。

## 5. 未交与边界

- 不支持：容器/记录跨边界、属性派生值、CPython 版本矩阵、Linux 腿。
  PLAN-1126 实测的是属性读取的**结果仍为 Python 对象**，可继续索引、调用并显式提取叶子（§7）；
  属性结果不进入 Auto 值域，容器/记录仍不跨边界，本条边界不据此改写。
- 异常映射仅覆盖受检 py 操作运行中实际抛出的普通 `Exception`；`A2xTrap`、进程级中断
  （`KeyboardInterrupt`/`SystemExit` 等 `BaseException`）与解释器/包准入失败（具名
  `m2build.*`）维持各自路径，不进入 Auto 错误映射。
- AV 的通用源面/协议最小切片仍属 PLAN-1094 范围；本文不扩张 AV 语言面。

## 6. A2PY 异常映射与原始位置（PLAN-1110，已交并实测）

- **包装与位置**：每个受检 py 操作发射为 `a2x_py_op(start, end, helper, args)`——helper
  真实执行，普通 `Exception` 被包成 `A2xPythonError`，携带原类名、消息、类别
  （`ImportError/ModuleNotFoundError→1`、`AttributeError→2`、`TypeError→3`、
  `IndexError→4`、`ValueError→5`、`OverflowError→6`、`KeyError→7`、
  `ZeroDivisionError→8`、`RuntimeError→9`、其余→10，观测 kind 为 `python.<N>`）与
  **原 Auto 源半开字节范围**（来自 `PyOpFact.span`，随事实 lowering 内联，不依赖生成
  行号映射）。
- **捕获**：Auto `try/catch` 经 HIR `LoopUnwind` 生成 `except A2xPythonError` 拦截，
  错误消息进 handler（kind=4 语言错误通道常量，与 a2x-rust 1115 约定一致），
  `a2x_py_intercept` 记录观察并计 handled 次数；求值次数与源序由既有单次求值规则保证
  （嵌套 `pop()` 副作用用例钉住）。
- **未捕获**：`a2x_run_main` 捕获逃逸的 `A2xPythonError`，stderr 输出
  `PyException <类名>: <消息>`，进程退出 1；观察 JSON（name/kind/start/end/escaped/
  handled）经 `ACE_A2PY_ERROR_OBSERVATION` 指定路径**独占创建**写出，上限 4096 字节。
- **同源验收**：语料 `python-context.case.ad` 四 Case（1094 捕获/未捕获 + 1110 嵌套
  索引/嵌套成功对照），每 Case 一份 `### Auto` 作者源、av/a2py 两个真实执行 check，
  父侧断言同一次执行的源 SHA256、类型、结果与位置；错类型/错位置/缺观察负控均判 FAIL。
- **缓存身份**：`ace-a2x-store` 对含 py 操作的片段附加位置事实上下文哈希（语义指纹
  忽略位置，位移原源后缓存片段不再复用旧 span）；`ace-m2-build` 的 A2PY 构建身份在源
  含 `use.py` 时加入实际 Python helper 内容哈希轴（`python-object-runtime`）。
- **文本边界**：py 字面量在边界按 Python `str` 表示编码（非 bytes），含 BMP 外字符的
  `\U` 八位转义；AV 数字表示与 A2PY 本地表示按语义归一比较。
- **证据**：`py_native_run_1107` 新增原生捕获/未捕获观察（含缓存前缀位移重跑）、
  `py_project_1107` 工程腿、语料固定组 `case-a2py-language-parity`（Windows x64 /
  CPython 3.14.2）。

## 7. NumPy 与 json 同源验收（PLAN-1126，已交并实测）

- **语料**：唯一作者 `crates/auto-lang/test/cases/language/script/python-libraries.case.ad`
  （`language.suite.ad` 引用，3 Case）：`numpy-json` 正例、`numpy-shape-error-caught`、
  `json-decode-error-uncaught`。每 Case 一份 `### Auto`，AV 普通源码入口
  （`ac-av/language-source-run-v1`）与 A2PY `run-py`（`ac-a2py/hir-stdlib-run-v1`）各一条
  真实执行检查，共用同一份冻结 oracle。A2PY 的流 oracle 用 `compare: "lf"`：程序输出走
  解释器文本流，Windows 上行尾为 CRLF，而 `value:` 观察行是原始字节 LF；AV 流 oracle 字节精确。
- **实测对象操作（未新增操作或索引值类别）**：同一 `GetItem`（AV `py_getitem` →
  `obj.get_item`；A2PY `a2x_py_getitem` → `recv[idx]`）覆盖 int 键、文本键与 Python 对象键
  （`row[slice(0, 3, 2)]`）。属性读取的结果仍是 Python 对象，可继续索引/调用（`a.shape` →
  `shape[0]`）。`__add__` 广播、绑定方法（`reshape`/`sum`/`mean`）、关键字实参
  （`sort_keys: true, ensure_ascii: false`）与 `.to(int/float/bool/str)` 对 NumPy 标量和
  `json` 叶子成立。数组与解析树留在 Python 端，Auto 只显式提取小值。
- **异常**：类别按类型 qualname 精确匹配，两端一致——`ValueError`→`python.5`；
  `json.JSONDecodeError`→`python.10`（不在类表内，不按 `ValueError` 子类归 5）；
  `ModuleNotFoundError`→`python.1`。失败位置是失败调用表达式整体的原 Auto 源半开字节范围
  （如 `arange(6).reshape(4, 2)`、`loads("{")`），导入失败是 `use.py` 行的成员名。
- **独立参考**：`crates/auto-lang/test/cases/ace-m2/python-1126/` 的手写原生 Python 直接调用
  NumPy 与 `json`，不读 ACE 产物，由 OS Python 运行。`ace-a2x-python` 的 `py_libraries_1126`
  把参考输出与作者文件的冻结 oracle 按**值**比对（整数/文本精确，均值 `rtol=atol=1e-6`），
  并钉住三个源码字节的 sha256、两端 oracle 互相一致与失败位置等于原源字节范围。
- **运行环境（Windows x64）**：CPython 3.14.2；NumPy 2.5.1（`numpy-2.5.1-cp314-cp314-win_amd64.whl`）。
  AV 取装配 CPython 可见的 NumPy。A2PY 的 `run-py` 按 1042 合同以 `python -I -B` **隔离模式**
  运行，不加载用户 site，所以其声明解释器（`A2PY_PYTHON`；缺省依次探测 `python3`/`python`/
  `py -3`）必须在自身的 site-packages 里带 NumPy；否则 NumPy 用例以真实
  `ModuleNotFoundError`（`python.1`）失败，不跳过、不通过。`run_sdk` 腿不带 `-I`，不受此限。
  本次 A2PY 的 Unicode stdout 实测使用 Windows ANSI 代码页 cp65001。
- **持久安排（Windows x64 冻结环境）**：A2PY 隔离腿的声明解释器是一个只装 `numpy==2.5.1` 的专用
  venv（基座 CPython 3.14.2），经**用户级**环境变量 `A2PY_PYTHON` 声明；AV 腿不需要它（进程内
  CPython 可见用户 site）。该变量只被 `a2x-worker` 的 `resolve_python`（及沿用它的测试）读取，
  `run_sdk` 与原生参考脚本不受影响。换机器时按同样方式准备，否则 NumPy 用例在 A2PY 端具名失败。

## Python 冒号切片（PLAN-1127）

A2PY 从共享受检 `GetItem` 事实读取接收者和 start/stop/step 三槽，按源序
求值、以 Python 内建 `slice` 构造键，并用原生订阅协议运行。省略槽位是
Python `None`。错误观测记录原异常名、`python.N` 分类、原逻辑路径与半开
UTF-8 字节范围、捕获次数及未捕获时的进程退出码。切片失败使用包含接收者的
整个下标范围，槽内失败使用内层操作的范围。

Windows x64 的 CPython 3.14.2、NumPy 2.5.1、PyTorch 2.14.0+cpu 已通过
同源双端及独立原生对照。两库支持正步长和省略槽，NumPy 支持负步长；
两库的零步长及 PyTorch 的负步长实测为 `ValueError`（`python.5`）。
切片语料的 `A2PY_PYTHON` 声明解释器须能在 `-I -B` 下导入两库，
AV 从其装配的 CPython 解析同组包。环境配法见
[原生参考目录](../../../crates/auto-lang/test/cases/ace-m2/python-1127/README.md)。

来源见 [PLAN-1127](../../plans/archive/1127-ace-python-object-slice-syntax.md)。

## 运行时浮点实参（PLAN-1129，已交并实测）

- **共享语义**：Python 调用的浮点实参不再限于字面量。A2PY 与 AV 消费同一份共享事实——
  每个浮点操作数记为 `Float`，只有字面量才带字面量值，对局部变量的读取一律不带。事实
  随 `ace-foreign-contract::py_ops` 的中立契约 `p1107-py-ops-v2` 一起换代：三个带值的
  操作（`py_call`、`py_getitem`、`py_binop`）逐条记录其线路版本，AV provider 源的注解与
  之逐项一致，由 `py_float_1129` 钉住。
- **A2PY 运行时值**：字面量按自身宽度发射为 Python 字面量，`float` 字面量是其加宽后的
  精确值（`0.1` 发射为 `0.10000000149011612`），有符号零保留符号。其余浮点按 span 取受检
  表达式原生 lowering，变量在调用处按当前值读取。此前事实给初始化为字面量的局部变量带上
  旧字面量，重赋值后两端都传旧值；该缺陷随本次一并修掉。
- **f32 精度**：Python 浮点是双精度，A2PY 此前对 `float` 算术不舍入，f32 类型的值实际携带
  双精度。现在 `float` 的 `+ - * /` 每一步经 `a2x_f32_round` 舍入（ties-to-even，溢出为
  `+/-inf`），与 TypeScript 目标既有的“每步 round”规则一致；传入 Python 调用的 f32 操作数在
  边界再显式经 `a2x_f32_round`。于是十个 `0.1f` 累加在 A2PY 上得到 `3f800001`，与 AV 相同，
  而非双精度累加后一次转换的 `3f800000`。
- **文本值**：程序计算的 Auto 文本作为实参时，经 `a2x_py_text` 以它所保存的 Python `str`
  传入；此前传的是运行时自己的 Str 包装对象。文本字面量仍按字面量发射。
- **非有限值**：`+/-inf` 与 NaN 原生传入，与 AV 的保留合同一致（AV 的 NaN 为规范静默 NaN）。
- **对照**：语料 `python-float-args.case.ad` 的十三个 Case 在 `case-a2py-language-parity`
  逐行匹配冻结预期；`ace-a2x-python` 的 `py_float_args_1129` 用独立原生 Python 参考产生
  同一组预期，并断言生成的 Python 以变量而非折算数字传递运行时学习率。
- **工程入口**：`M2Build::build(TargetKind::A2py, …)` 编译同一份 PyTorch 源，`ace-m2-build`
  的 `py_float_args_1129` 运行生成程序并核对优化器自报的学习率。
- **缓存身份**：`a2x_f32_round` 进入基础运行时，运行时内容哈希随之变化，既有片段缓存按身份
  自然换代。
- **已知边界**：A2PY 的操作数查找把文本拼接表达式与帧构造节点一并跳过，拼接表达式须先绑定
  局部变量再作为实参（早于本计划）。

来源见 [PLAN-1129](../../plans/archive/1129-ace-python-runtime-float-arguments.md)。
