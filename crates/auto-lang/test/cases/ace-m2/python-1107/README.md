# PLAN-1107 — ACE Python 同源工程（AV ＋ A2PY）

`pytorch_train.as` 是 PLAN-1107 §5.1 冻结的同源验收程序：一份普通 Auto 源经**同一
共享前门**（`use.py` 前门 → 绑定/受检 HIR/共享 A2X 视图）分别由两条链真实运行：

- **AV**：`ace_abi::py_session::PySession`（967 provider/PyO3/CPython）；
- **A2PY**：`ace_a2x_python::PythonTargetProvider` 生成的 Python 文件集，在声明的 OS
  解释器里直接运行（原生 import/call/getattr/getitem/binop/转换，无 ctypes、无 PyO3）；
- **978 工程**：`M2Build::build(TargetKind::A2py, …)` / `build_sdk` + `run_sdk`。

**任何一端都必须读这个文件**，不得各自维护手写副本（`p1107_v01_corpus_identity_is_pinned`
钉住 sha256 `0d5dc8504252fbb1caff55a0e2adf8b0cb8b2b4ab83d0583409d97015a0e9d38`）。

## 与 1026 语料的差异（r2 记录，共两处，均为语言拼写）

1026 已交语料（`python-1026/pytorch_train.as`）有五处表面调整；在当前 HEAD 实测后，
本 Plan 只保留两处**非语义**拼写修正，其余三处不再需要：

1. `for step < 60` —— AutoLang 的条件循环拼写（`while` 不是本语言关键字；见
   `docs/conformance/04-control-flow.md` 与 ACE parser profile 规则）；
2. `fn main() int` + `return 0` —— ACE 各 source profile 要求入口标注返回类型。

不再需要的三处：浮点后缀（`0.1` 直接可用）、`var mparams = model.parameters()` 与
`var lr_v = 0.1` 两处 hoist（嵌套调用位置实参与动态浮点 kwarg 均已被接受）。

## 运行方式

```text
AV  ：cargo nextest -p ace-abi --test conformance -E 'test(p1107_v01)'
A2PY：cargo nextest -p ace-a2x-python -E 'test(p1107_t03)'
工程：cargo nextest -p ace-m2-build -E 'test(p1107_t05)'
```

环境：Windows x64、`C:\Python314`（可用 `AUTO_PYTHON_HOME` 覆盖）、CPython 3.14.2、
torch 2.14.0+cpu、`PYO3_PYTHON` 已设。解释器或 torch 缺失时具名失败（不静默跳过）。

## 独立 oracle（不经 ACE）

seed=0、torch 2.14.0+cpu、CPU 线性回归 60 步；AV/A2PY 两端实测打印：

- 60 个有限 loss：首个 `0.2370201`（f32 显示），最终 `0.101308905`；
- `prediction.shape` 显示 `torch.Size([4, 1])`；
- `prediction[0].to(float)` ≈ `0.47284803`。

断言用 `rtol=1e-5, atol=1e-6` 量级（Auto 侧 f32 显示），并检查 loss 下降且 < 1。

## 已知观测

- 两端对 `float(requires_grad tensor)` 都触发 torch 自身的 `UserWarning`（stderr），
  属同一 Python 协议行为，不影响 stdout oracle；若 V-03/V-04 要求静默，须在共享语义层统一处置。
- A2PY 运行时入口额外打印 `value:<i32>` 运行观测行（目标运行通道，不属于程序输出）。