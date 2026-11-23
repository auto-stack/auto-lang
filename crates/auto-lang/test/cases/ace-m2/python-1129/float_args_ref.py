"""Independent native-Python reference for python-float-args.case.ad (PLAN-1129).

Run as ``python -I -B float_args_ref.py <case-name>``. The reference calls the
same library APIs the Auto source calls (``struct.pack``, ``math.*`` and the
builtins) and reads no compiler output. A Python float is a double, so every
value the Auto source holds as ``float`` (f32) is rounded here through
``struct`` at each step — the width the language assigns to that value. An
``f64`` value needs no rounding.
"""
import math
import struct
import sys


def f32(x):
    """Round to f32 precision, ties to even, and widen back: an Auto ``float``."""
    return struct.unpack("<f", struct.pack("<f", x))[0]


def f32_hex(x):
    return struct.pack(">f", x).hex()


def f64_hex(x):
    return struct.pack(">d", x).hex()


def runtime_values():
    step = 3
    rate = f32(f32(0.05) + f32(f32(step) * f32(0.01)))
    print(f32_hex(rate))
    print(f64_hex(0.1 * 3.0))
    print(f32_hex(f32(1.0 * f32(0.1))))
    again = f32(0.25)
    again = f32(0.5)
    print(f32_hex(again))
    print(f32_hex(f32(0.1)))
    print(f64_hex(0.1))
    print(f64_hex(-0.0))
    print(f32_hex(-0.0))
    print(f64_hex(0.0))


def types_and_text():
    obj = repr
    step = 1
    # The class name of the very object each argument arrives as.
    for value in [
        "F:1.25",
        f32(0.1),
        0.1,
        f32(f32(0.5) + f32(0.25)),
        7,
        True,
        obj,
        f32(step),
        step,
    ]:
        print(getattr(value, "__class__").__name__)
    for value in ["F:1.25", f32(0.1), 0.1, True]:
        print(repr(value))
    joined_text = "F:1.25" + "F:2.5"
    print(repr(joined_text))


def containers_and_binop():
    d = {}
    missing = d.get(0.5, "missing")
    print(missing)
    seven = 7
    placed = d.setdefault(2.5, seven)
    print(placed)
    print(d[2.5])
    print(d.get(2.5, 0))
    s = repr("a")
    joined = s + "F:1.25"
    print(joined)
    doubled = s + s
    four = doubled * 2
    print(four)


def order_and_once():
    def mark(label_text, v):
        print(label_text)
        return v

    a = math.isclose(mark("a", f32(1.5)), mark("b", f32(1.5)), rel_tol=mark("rel", f32(0.0)))
    print(1 if a else 0)
    caught = 0
    try:
        r = math.sqrt(mark("neg", f32(-4.0)))
        print(r)
    except ValueError as error:
        caught += 1
        print(f"PyException ValueError: {error}")
    print(caught)
    b = math.isclose(mark("c", f32(2.5)), mark("d", f32(3.5)), rel_tol=mark("rel2", f32(0.0)))
    print(1 if b else 0)


def rounds_first():
    third = f32(1.0 / f32(3.0))
    triple = f32(third * f32(3.0))
    print(f32_hex(f32(triple * f32(3.0))))
    dthird = 0.1 / 3.0
    print(f64_hex(dthird * 3.0))
    total = f32(0.1)
    for _ in range(9):
        total = f32(total + f32(0.1))
    print(f32_hex(total))


def boundaries():
    big = 1.0
    for _ in range(11):
        big *= 100000000000000000000000000000000.0
    print(1 if math.isinf(big) else 0)
    print(f64_hex(big))
    print(f64_hex(math.copysign(1.0, big)))
    nan = big - big
    print(1 if math.isnan(nan) else 0)
    tiny = 1.0
    for _ in range(320):
        tiny /= 10.0
    print(f64_hex(tiny))
    small = f32(1.0)
    for _ in range(40):
        small = f32(small / f32(10.0))
    print(f32_hex(small))
    print(f32_hex(big))
    print(f64_hex(0.0 * -1.0))


def signed_zero_f32():
    # The sign of a zero is a bit, not a comparison: every zero here is one
    # the program computes (a product, a widening), except the last literal.
    zero = f32(0.0)
    minus = f32(-1.0)
    print(f32_hex(f32(zero * minus)))
    print(f64_hex(float(zero) * -1.0))
    print(f32_hex(zero))
    print(f32_hex(-0.0))


def binop_float():
    # A float on the right of a Python object's operator (`py_binop`): the
    # standard library's exact rational takes the f32 the program computed.
    from fractions import Fraction

    half = Fraction(3, 2)
    quarter = f32(f32(0.5) * f32(0.5))
    print(half + quarter)
    print(half - 0.25)
    print(half * 2)


def show_f32(v):
    """The Auto console's display of an `f32` (the language's own rule, as
    the 1107 corpus is printed): NaN / Infinity spellings, a whole number
    without a decimal part, otherwise the shortest decimal that reads back
    as the same f32."""
    if v != v:
        return "NaN"
    if v == math.inf:
        return "Infinity"
    if v == -math.inf:
        return "-Infinity"
    r = round(v)
    if abs(v - r) < 1e-9 and abs(r) < 1e15:
        return str(int(r))
    for precision in range(1, 10):
        text = format(v, f".{precision}g")
        if f32(float(text)) == v:
            return text
    return repr(float(v))


def torch_lr_computed():
    # CPU PyTorch, the 1107 Linear/MSELoss/SGD skeleton. The learning rate
    # is COMPUTED from a per-run input with dyadic constants (1/32 and 3/32),
    # so it is exact in every width and both runs end differently; the
    # optimizer is asked what rate it actually holds.
    import torch

    for run in range(2):
        training_step = run
        rate = f32(training_step) * 0.0625 + 0.03125
        torch.manual_seed(0)
        x = torch.arange(8).reshape(4, 2) / 2
        target = torch.ones(4, 1)
        model = torch.nn.Linear(2, 1, bias=False)
        loss_fn = torch.nn.MSELoss()
        optimizer = torch.optim.SGD(model.parameters(), lr=rate)
        print(show_f32(rate))
        print(show_f32(optimizer.param_groups[0]["lr"]))
        for step in range(3):
            prediction = model(x)
            loss = loss_fn(prediction, target)
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            if step in (0, 2):
                print(show_f32(loss.item()))
        prediction = model(x)
        print(prediction.shape)
        print(show_f32(prediction[0].item()))


def sqrt_runtime_negative():
    # a runtime negative: the sign comes from a function of a parameter
    value = -f32(4.0)
    try:
        root = math.sqrt(value)
        print(root)
    except ValueError as error:
        print(f"PyException ValueError: {error}")
        print(1)
    # the catch happened exactly once and the program went on
    print(int(math.sqrt(f32(9.0))))


def sqrt_uncaught():
    value = -f32(4.0)
    try:
        print(math.sqrt(value))
    except ValueError as error:
        # the Auto program fails: the original class and message are the
        # whole stderr, and the process exits 1
        sys.stderr.write(f"PyException ValueError: {error}\n")
        sys.exit(1)


def sqrt_nested_first():
    # the inner argument fails before the outer call can run
    try:
        root = math.sqrt(math.sqrt(-f32(4.0)) + 100.0)
        print(root)
    except ValueError as error:
        print(f"PyException ValueError: {error}")
        print(1)


def sqrt_nested_shifted():
    sqrt_nested_first()


CASES = {
    "runtime-values": runtime_values,
    "types-and-text": types_and_text,
    "order-and-once": order_and_once,
    "rounds-first": rounds_first,
    "boundaries": boundaries,
    "signed-zero-f32": signed_zero_f32,
    "binop-float": binop_float,
    "containers-and-binop": containers_and_binop,
    "torch-lr-computed": torch_lr_computed,
    "sqrt-runtime-negative": sqrt_runtime_negative,
    "sqrt-uncaught": sqrt_uncaught,
    "sqrt-nested-first": sqrt_nested_first,
    "sqrt-nested-shifted": sqrt_nested_shifted,
}

if __name__ == "__main__":
    name = sys.argv[1] if len(sys.argv) > 1 else ""
    if name not in CASES:
        sys.stderr.write("usage: float_args_ref.py <case-name>\n")
        sys.exit(2)
    CASES[name]()
