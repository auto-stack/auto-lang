# PLAN-1026 — ACE Python 互操作（AV PyTorch 训练闭环）

`pytorch_train.as` 是 PLAN-1026 §5.1 冻结的验收程序：一个普通 Auto 源文件，
`use.py` 导入 torch 成员，60 步 CPU 线性回归训练，`print(loss.to(float))`
逐轮打印损失，最后打印 `prediction.shape` 与 `prediction[0].to(float)`。

## 运行方式

```text
固定测试组：cargo t ace-python          （experimental/ace，显式 opt-in）
公共 API：  ace_abi::py_session::PySession::open()?.run(&src)
```

环境（T-01 冻结）：CPython 3.14（`C:\Python314`，可用 `AUTO_PYTHON_HOME`
覆盖）、PyO3 0.29.3（构建闭包内 pinned，离线缓存）、torch 2.14.0+cpu。
解释器/torch 缺失时该组**具名失败**，不会静默跳过。

## 期望输出（对拍基准）

独立原生 Python 参考（不经 ACE、不用 adapter；seed=0，torch 2.14.0+cpu，
2026-11-17 记录于 plan-artifacts/plan-1026/ref/train_ref_out.json）：

- 首个 loss ≈ `0.2370201051235199`，最终 loss ≈ `0.1013089045882225`
  （f32 打印；`allclose(rtol=1e-5, atol=1e-6)` 内一致，最终 < 首个且 < 1）；
- `prediction.shape` 显示 `torch.Size([4, 1])`；
- `prediction[0].to(float)` ≈ `0.4728480279445648`。

## 表面拼写适配（语义不变，记录于计划）

- `lr: 0.1d` —— 961 冻结的浮点字面量后缀（等价 `0.1`）；
- `for step < 60` —— aggregates 面的条件循环拼写（等价 `while step < 60`）。

## 边界（v1 冻结）

串行会话、单解释器、CPU；动态 float 实参不跨界（浮点字面量经编译期
`F:` 编码折叠）；显示通道 ASCII ≤256B；非正规/非有限浮点值在 provider
侧具名拒绝。详见 `docs/plans/1026-python-interop-standard-auto-syntax.md` §5.3。
