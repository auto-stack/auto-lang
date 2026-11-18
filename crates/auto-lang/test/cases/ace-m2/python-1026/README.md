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

## 表面偏差（语义保持，共五处；记录于计划 §5.3）

对照 §5.1 冻结正文，语料有五处**不含语义变化**的偏差：

1. `lr: 0.1d` —— 961 冻结的浮点字面量后缀（等价 `0.1`）；
2. `for step < 60` —— aggregates 面的条件循环拼写（等价 `while step < 60`）；
3. `fn main() int` 包裹 + `return 0` —— 本前门的入口合同（§5.1 正文写 `fn main()`）；
4. `var mparams = model.parameters()` —— 语义保持 hoist（"嵌套调用结果直接作位置实参"这一形态在 v1 **未验证**：既不主张支持，也不得宣称已拒绝）；
5. `var lr_v = 0.1d` —— 语义保持 hoist（"动态浮点关键字实参"在 v1 **已证拒绝**，诊断码 `source.py-domain`）。

第 4、5 条都是**语义保持**的显式提升（不是语义放宽）：`SGD(mparams, lr: lr_v)` 传入同一对象引用与同一 0.1 值，torch 侧行为与逐字正文一致。但两条的**边界性质不同**：第 5 条是已证拒绝（`source.py-domain`）；第 4 条只是**未验证**——`py_is_object` 显式接受内联 py-op 结果、`py_frame_val` 可封送这类值，所以它既不是"明确拒绝"，也不能当成已支持。要主张拒绝必须先补反例；将来若放宽第 4 条，须同时补 V-02/V-03 反例。

## 边界（v1 冻结）

串行会话、单解释器、CPU；动态 float 实参不跨界（浮点字面量经编译期
`F:` 编码折叠）；显示通道 ASCII ≤256B；非正规/非有限浮点值在 provider
侧具名拒绝。console 准入在本切片放宽为：**非 profile-8** 的模块若携带
**至少一条 foreign 调用**、且 host-import 表全部为 `auto.console.*`，即可
准入 console import（纯标量模块仍需 host-console profile；compile 期的
对应条件是"模块内存在任意 foreign 声明"——只声明不调用的角落编译期准入、
运行期以 `host.profile` 拒绝）。规则见 SD-06。
详见 `docs/plans/1026-python-interop-standard-auto-syntax.md` §5.3。
