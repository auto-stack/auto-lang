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
  `"tag <len>\n<value>\n"`；内联 py 调用不会被求值两次（PLAN-1107 修复；此前两位 token
  会出现声明长度与 payload 不符、并重复副作用）。

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
| A2PY 解释器 | PATH 上的声明 `python3`（实测 3.14.2），`-B` 运行 | A2PY 原生运行、`run_sdk` | 其他版本；缺失 → `m2build.run.interpreter-failed` |
| 第三方包 | torch 2.14.0+cpu（CPU 线性回归）、标准库（list/len/repr/round） | 共享用例 | CUDA/其他 torch 构建；未声明包 → `m2build.run.runtime-dependency` |
| 语言面 | 17 操作、int/bool/text 字面量与 py 对象、`.to(int/bool/float/str)`、嵌套调用、kwarg、绑定方法 | 端到端 | 容器/记录跨边界、动态浮点实参（仅字面量编码） |
| 目标 | AV（in-process）、A2PY（原生 Python 文件集）、978 SDK A2py | T-01/T-03/T-05 | 其他 978 目标不变 |
| 失败 | 缺解释器/包/工件/授权 → 具名 `m2build.*`；未捕获异常 → 两端非 PASS；A2PY Python 异常 → Auto 错误类型 + 原始位置（`ValueError`→`python.5`、`IndexError`→`python.4`，PLAN-1110） | V-04/V-07；1110 语料 V-01..04 | 其余异常类按 `A2xPythonError` 类别表归 `python.10`（other），映射矩阵未逐类实测验交 |

## 4. 工程集成（978）

- 入口：`M2Build::build(TargetKind::A2py, …)` / `build_sdk` / `run_sdk`；`use.py` 源自动
  选择 Aggregates + 中立 py 声明契约（`cold_assemble_py`），非 Python 源走冻结 Scalar 路径。
- 身份与缓存：冷构建产片段；同输入为 `cache-hit`（`rebuilt=false`，零工具 spawn）；
  改 body 使身份变更并重产；项目名、caps、participating provider 工件都是身份轴。
- 分发：SDK 可重定位（异根运行）、缺工件/缺依赖具名拒绝，不把 traceback 计为运行。

## 5. 未交与边界

- 不支持：容器/记录跨边界、动态浮点实参、属性派生值、CPython 版本矩阵、Linux 腿。
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