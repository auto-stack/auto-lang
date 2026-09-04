"""py_torch_train native Python oracle (Plan 539 W2).

Mirrors tests/auto/train.at case-for-case. Prints TAP lines; the parity
comparator joins the three backends by test name.
"""
import torch
import torch.nn as nn
from importlib import import_module


def tap_ok(n, name):
    print(f"ok {n} - {name}")


def tap_not_ok(n, name, diag):
    print(f"not ok {n} - {name} # {diag}")


def main():
    torch.manual_seed(0)

    # 1. kwargs constructor shape
    layer = nn.Linear(2, 3, bias=False)
    w = layer.weight
    shape = w.shape
    dims = 0
    for _d in shape:
        dims = dims + 1
    d0 = shape[0]
    if dims == 2 and d0 == 3:
        tap_ok(1, "test_linear_kwargs_shape")
    else:
        tap_not_ok(1, "test_linear_kwargs_shape", f"dims {dims} d0 {d0}")

    # 2. bias is None via kwargs
    b = layer.bias
    if b is None:
        tap_ok(2, "test_linear_bias_null")
    else:
        tap_not_ok(2, "test_linear_bias_null", "not null")

    # 3. forward shape
    x = torch.randn(4, 2)
    y = layer(x)
    yshape = y.shape
    if yshape[0] == 4 and yshape[1] == 3:
        tap_ok(3, "test_forward_shape")
    else:
        tap_not_ok(3, "test_forward_shape", f"{yshape}")

    # 4. seeded training convergence
    torch.manual_seed(0)
    model = nn.Linear(2, 1, bias=False)
    xin = (torch.arange(8).reshape(4, 2)) / 2
    target = torch.ones(4, 1)
    loss_fn = nn.MSELoss()
    optim_cls = getattr(import_module("torch.optim"), "SGD")
    params = model.parameters()
    opt = optim_cls(params, lr=0.1)
    first_loss = 0.0
    for i in range(60):
        out = model(xin)
        loss = loss_fn(out, target)
        opt.zero_grad()
        loss.backward()
        opt.step()
        if i == 0:
            first_loss = float(loss)
    out2 = model(xin)
    lf = float(loss_fn(out2, target))
    if lf < first_loss and lf < 1:
        tap_ok(4, "test_train_converges")
    else:
        tap_not_ok(4, "test_train_converges", f"l0 {first_loss} lf {lf}")

    # 5. scalar extraction
    s = torch.tensor([2.0, 3.0])
    total = float(s.sum())
    if total == 5:
        tap_ok(5, "test_item_scalar")
    else:
        tap_not_ok(5, "test_item_scalar", f"got {total}")

    # 6. exception fallback
    try:
        model.no_such_method()
        msg = "unexpected"
    except AttributeError:
        msg = "E"
    if msg == "E":
        tap_ok(6, "test_may_exception")
    else:
        tap_not_ok(6, "test_may_exception", msg)

    # 7. list argument round-trip
    t7 = torch.tensor([10, 20, 30])
    s7f = float(t7.sum())
    if s7f == 60:
        tap_ok(7, "test_list_arg_roundtrip")
    else:
        tap_not_ok(7, "test_list_arg_roundtrip", f"got {s7f}")

    # 8. tuple flatten
    m8 = torch.zeros(2, 5)
    sz = m8.size()
    nd = 0
    for _d in sz:
        nd = nd + 1
    if nd == 2 and sz[0] == 2 and sz[1] == 5:
        tap_ok(8, "test_tuple_flatten")
    else:
        tap_not_ok(8, "test_tuple_flatten", f"{sz}")

    # 9. dict round-trip
    d9 = dict(mode=7, rank=9)
    mode = d9["mode"]
    sd = model.state_dict()
    sdl = len(sd)
    if mode == 7 and sdl == 1:
        tap_ok(9, "test_state_dict_getitem")
    else:
        tap_not_ok(9, "test_state_dict_getitem", f"mode {mode} len {sdl}")

    # 10. no_grad inference
    ng = torch.no_grad()
    gflag = 1
    with ng:
        xg2 = torch.arange(3).float()
        xg2.requires_grad_(True)
        yg = xg2 * 2
        gflag = int(yg.requires_grad)
    if gflag == 0:
        tap_ok(10, "test_no_grad_inference")
    else:
        tap_not_ok(10, "test_no_grad_inference", f"g {gflag}")


if __name__ == "__main__":
    main()
