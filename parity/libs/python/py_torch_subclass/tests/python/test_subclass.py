"""py_torch_subclass native Python oracle (Plan 602).

Mirrors tests/auto/subclass.as case-for-case. Prints TAP lines; the parity
comparator joins the three backends by test name.

Test 1's forward scalar is pinned to the same constant the Auto corpus pins
(manual_seed(0), torch CPU); tests 2/4 are exact integers; test 3 asserts the
same convergence envelope as the Auto side.
"""
import torch
import torch.nn as nn


def tap_ok(n, name):
    print(f"ok {n} - {name}")


def tap_not_ok(n, name, diag):
    print(f"not ok {n} - {name} # {diag}")


def main():
    # ---- 1. custom nn.Module: Python __init__ builds the layer, forward
    #         callable (the Auto side binds its closure the same way).
    torch.manual_seed(0)

    class MyNet(nn.Module):
        def __init__(self):
            import torch.nn as nn
            super().__init__()
            self.lin = nn.Linear(4, 1)

        def forward(self, x):
            return self.lin(x)

    net = MyNet()
    xb = torch.ones(4)
    y1 = net(xb)
    v1 = float(y1[0])
    if v1 == -0.7075908780097961:
        tap_ok(1, "test_custom_module_forward")
    else:
        tap_not_ok(1, "test_custom_module_forward", f"got {v1}")

    # ---- 2. Dataset subclass: __len__ / __getitem__ protocol.
    class MyData(torch.utils.data.Dataset):
        def __init__(self):
            self.data = [3, 1, 4, 1, 5]

        def __len__(self):
            return len(self.data)

        def __getitem__(self, i):
            return self.data[i]

    ds = MyData()
    dlen = len(ds)
    d0 = ds[0]
    d4 = ds[4]
    if dlen == 5 and d0 == 3 and d4 == 5:
        tap_ok(2, "test_dataset_len_getitem")
    else:
        tap_not_ok(2, "test_dataset_len_getitem", f"len={dlen} d0={d0} d4={d4}")

    # ---- 3. training loop (num_workers=0 discipline mirrored: the plain
    #         loop steps the same optimizer the same number of times).
    torch.manual_seed(0)

    class Reg(nn.Module):
        def __init__(self):
            import torch.nn as nn
            super().__init__()
            self.lin = nn.Linear(2, 1)

        def forward(self, x):
            return self.lin(x)

    reg = Reg()
    mse = nn.MSELoss()
    sgd = torch.optim.SGD(reg.parameters(), lr=0.05)
    xin = torch.ones(2)
    target = torch.zeros(1)
    first = 0.0
    last = 0.0
    i = 0
    while i < 10:
        pred = reg(xin)
        loss = mse(pred, target)
        sgd.zero_grad()
        loss.backward()
        sgd.step()
        lv = float(loss)
        if i == 0:
            first = lv
        if i == 9:
            last = lv
        i = i + 1
    if first > last and last < 0.001:
        tap_ok(3, "test_training_loop")
    else:
        tap_not_ok(3, "test_training_loop", f"first={first} last={last}")

    # ---- 4. reentrancy chain (D5): the Auto side drives
    #         outer → mid(Auto closure) → leaf at ≥3 nested shim levels; the
    #         oracle just pins the same arithmetic result.
    class Chain:
        def __init__(self):
            self.k = 7

        def outer(self, x):
            return self.mid(x + 1)

        def leaf(self, v):
            return v * 2

        def mid(self, v):
            return self.leaf(self.k + v)

    chain = Chain()
    r4 = chain.outer(1)
    if r4 == 18:
        tap_ok(4, "test_reentry_depth3")
    else:
        tap_not_ok(4, "test_reentry_depth3", f"got {r4}")


if __name__ == "__main__":
    main()
