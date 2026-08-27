"""Python oracle for py_torch parity (Plan 461).

Emits TAP output so the auto-parity runner can parse it with the same TAP
parser used for the AutoVM and a2py backends. Test names MUST match the Auto
test file (tests/auto/torch.at) because the comparator joins backends by name.
"""
import torch


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    if int(torch.arange(5).sum()) == 10:
        tap_ok(1, "test_arange_sum")
    else:
        tap_not_ok(1, "test_arange_sum", "got {}".format(torch.arange(5).sum()))
    if int(torch.ones(4).sum()) == 4:
        tap_ok(2, "test_ones_sum")
    else:
        tap_not_ok(2, "test_ones_sum", "got {}".format(torch.ones(4).sum()))
    if int(torch.zeros(3).sum()) == 0:
        tap_ok(3, "test_zeros_sum")
    else:
        tap_not_ok(3, "test_zeros_sum", "got {}".format(torch.zeros(3).sum()))

    # linspace(0,-4,5) = [0,-1,-2,-3,-4].
    neg = torch.linspace(0.0, -4.0, 5)
    if int(torch.relu(neg).sum()) == 0:
        tap_ok(4, "test_relu_sum")
    else:
        tap_not_ok(4, "test_relu_sum", "got {}".format(torch.relu(neg).sum()))
    if int(torch.abs(neg).sum()) == 10:
        tap_ok(5, "test_abs_sum")
    else:
        tap_not_ok(5, "test_abs_sum", "got {}".format(torch.abs(neg).sum()))

    if str(torch.linspace(0.0, 1.0, 5).type()) == "torch.FloatTensor":
        tap_ok(6, "test_type_float_str")
    else:
        tap_not_ok(6, "test_type_float_str", "got {}".format(str(torch.linspace(0.0, 1.0, 5).type())))

    if str(torch.arange(5).type()) == "torch.LongTensor":
        tap_ok(7, "test_type_str")
    else:
        tap_not_ok(7, "test_type_str", "got {}".format(str(torch.arange(5).type())))
