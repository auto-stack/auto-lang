# py_torch_train (Plan 539 W2 training-loop parity)

torch training loop through the Python FFI bridge. Three-way: AutoVM vs
a2py vs native Python. Everything seeded (`torch.manual_seed(0)`) and
CPU-deterministic.

## Scope (10 cases)

- **kwargs constructors** — `Linear(2, 1, bias: false)` and
  `SGD(params, lr: 0.1)` item-import direct calls with named args.
- **training loop** — forward (`py_call0(model, x)`) → `MSELoss` →
  `zero_grad`/`backward`/`step`, 60 ticks, final loss < first and < 1
  (seeded convergence).
- **scalar extraction** — `py_float(x)` (`float(x)`); 0-dim tensors stay
  opaque handles so `backward` works on the loss.
- **May channel** — exception → `.?("E")` fallback.
- **round-trips** — Auto list arg (`tensor([10, 20, 30])`), tuple flatten
  (`zeros(2, 5).size()` → Auto List), kwargs-built dict `py_getitem`,
  `state_dict()` length.
- **no_grad inference** — the inline `py_with` bracket.

## Notes

- **Loss tensors must stay tensors**: only exact Python floats marshal to
  f64 eagerly (W2 rule); `.item()`/`float()` results are real floats.
  Extract scalars with `py_float(x)` — never assert on a tensor handle.
- `.len()` on py values was unreliable pre-PLAN-569 (type-lie dispatch,
  P539-D2 已清偿——py-类型侧表+组合子双通道路由)；本套件沿用
  `for x in v` 计数 + `v[0]` 索引惯例，去规避留待自然触碰。
- Float literals int-ify in a2py (`1.0` → `1`) — build float tensors via
  `arange(n).float()` or true division (`t / 2`), never
  `tensor([1.0, ...])` when dtype matters.
- `state_dict()` VALUES stringify under the Plan-300 dict marshal — only
  scalar dict values round-trip; tensor values need `py_getattr` paths.
