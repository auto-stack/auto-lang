# PLAN-1126 — ACE Python 库同源验收（NumPy ＋ 标准库 `json`，AV ＋ A2PY）

唯一的 Auto 作者源是
[`language/script/python-libraries.case.ad`](../../language/script/python-libraries.case.ad)，
由 `language.suite.ad` 引用。每个 Case 只有一份 `### Auto`，两条真实执行检查共用它和同一份冻结 oracle：

- **AV**：普通源码入口 `ac-av/language-source-run-v1`（进程内 CPython，967 py provider）；
- **A2PY**：1110 交付的 `ac-a2py/hir-stdlib-run-v1`（worker `run-py`，生成的原生 Python 在隔离模式运行）。

**任何一端都只读这个作者文件**，不得另存手写副本；三个源码字节的 sha256 由
`py_libraries_1126.rs` 钉住。

| Case | 验证点 |
|---|---|
| `py.libraries.1126.numpy-json` | 正例。NumPy：`arange`/`reshape`/广播、属性派生 `shape` 上的 int 索引、`slice` 对象带步长切片、`sum`/`mean`；数组留在 Python 端，Auto 只显式提取 6 个小值。`json`：文本键、嵌套叶子、关键字参数、`ensure_ascii: false` 的 Unicode 序列化与结构往返。 |
| `py.libraries.1126.numpy-shape-error-caught` | 真实 NumPy `ValueError`（`arange(6).reshape(4, 2)`）进入 Auto handler 恰一次后继续执行；类别 `python.5`，位置是失败调用表达式的字节范围。 |
| `py.libraries.1126.json-decode-error-uncaught` | 真实 `json.JSONDecodeError`（`loads("{")`）未捕获：AV 为语言错误终态且无部分输出，A2PY 进程退出 1；两端类别均为 `python.10`，位置为 `loads("{")` 的字节范围。 |

## 独立参考（不经 ACE）

本目录只放**独立参考**——手写原生 Python，直接调用 NumPy 与 `json`，从不读取 ACE 产物——不放任何
Auto 源副本。它们与 Auto 侧共用同样的固定输入和参数，结果由独立 OS Python 运行得出：

| 参考 | 对应 Case | 输出 |
|---|---|---|
| `numpy_json_ref.py` | `numpy-json` | 11 行 UTF-8 JSON（每行一个值，类型显式）；切片用 NumPy 原生 `row[0:3:2]` 而非 `slice()` 对象；另断言解码树与原输入结构、类型相等。 |
| `numpy_shape_error_ref.py` | `numpy-shape-error-caught` | NumPy 抛出的原始类名与消息、捕获计数、合法 `reshape` 的首轴。 |
| `json_decode_error_ref.py` | `json-decode-error-uncaught` | 不捕获，进程退出 1，traceback 末行为 `json.decoder.JSONDecodeError: …`。 |

`experimental/ace/crates/a2x-python/tests/py_libraries_1126.rs` 运行这些参考并与作者文件里的冻结 oracle
按**值**比对：整数与文本精确相等，均值按 `rtol=atol=1e-6`；同时核对两端 oracle 互相一致、源码指纹、
失败位置等于原源字节范围，以及错值（切片和、JSON 字段、类别、位置、类型）均判不一致。

## 运行方式

```text
两端用例：cargo t case-av-language-parity case-a2py-language-parity   （先 --prepare-only）
独立参考：cargo nextest --manifest-path experimental/ace/Cargo.toml -p ace-a2x-python --test py_libraries_1126
单 Case  ：auto-case module-case --suite crates/auto-lang/test/cases/language.suite.ad --case <id> --out <dir>
```

## 环境与 NumPy 来源

Windows x64、CPython 3.14.2、NumPy 2.5.1（`numpy-2.5.1-cp314-cp314-win_amd64.whl`）。

- **AV** 用装配的 CPython，NumPy 取自该解释器可见的 site（本机为用户 site）。
- **A2PY** 的 `run-py` 按 1042 冻结合同以 `python -I -B a2x_main.py` **隔离模式**运行：隔离模式不加载
  用户 site，因此其**声明解释器**（`A2PY_PYTHON`，缺省依次探测 `python3`/`python`/`py -3`）必须在自身的
  site-packages 里带 NumPy。解释器看不到 NumPy 时用例以真实 `ModuleNotFoundError`（`python.1`）失败，
  不会跳过，也不会通过。
- A2PY 的程序输出走解释器文本流，Windows 上行尾为 CRLF（`value:` 观察行为原始字节 LF），故 A2PY 的流
  oracle 用 `compare: "lf"`；AV 的流 oracle 保持字节精确。
- 隔离模式下 `PYTHONUTF8` 不生效；本机 ANSI 代码页为 cp65001，stdout 仍为 UTF-8。参考脚本显式按
  UTF-8 字节输出，不依赖宿主区域设置。

解释器或 NumPy 缺失时两端均具名失败，不静默跳过。
