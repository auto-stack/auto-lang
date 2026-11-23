"""Independent native Python reference for the shared Auto slicing cases."""

import json
import sys


def mark(tag, n):
    print(tag)
    return n


def trace(numpy):
    slice = 99
    row = numpy.array(range(mark("recv", 6)))[mark(tag="start", n=0):mark("stop", 3):mark("step", 2)]
    print(row.tolist())
    print(slice)


def values(row):
    for result in (
        row[0:3:2], row[:3], row[2:], row[:], row[1:4:], row[::2], row[-3:]
    ):
        print(result.tolist())


def positive():
    import numpy
    import torch

    n = numpy.array(range(6))
    values(n)
    print(n[::-1].tolist())
    print(n[4:0:-2].tolist())
    print(int(n[::-1][0]))

    values(torch.arange(6))


def compatibility():
    import numpy

    n = numpy.array(range(6))
    print(n[slice(0, 3, 2)].tolist())
    print(n[n[1]:n[4]].tolist())
    print(n.reshape(2, 3)[0][::2].tolist())
    print([4, 5][0])
    print("abcd"[1:3])
    print(json.loads('{"key":42}')["key"])
    trace(numpy)


def render_error(error):
    return f"PyException {type(error).__name__}: {error}"


def rejected_step(case):
    caught = 0
    if case == "numpy-zero":
        import numpy
        row = numpy.arange(6)
    else:
        import torch
        row = torch.arange(6)
    zero = 0
    try:
        if case == "numpy-zero":
            print(row[::zero].tolist())
        elif case == "torch-negative":
            print(row[::-1].tolist())
        else:
            print(row[::0].tolist())
    except ValueError as error:
        if case == "torch-zero":
            print(render_error(error), file=sys.stderr)
            return 1
        caught += 1
        print(render_error(error))
    print(caught)
    print(row[1:4:2].tolist())
    return 0


def inner_failure():
    import numpy

    caught = 0
    try:
        result = numpy.arange(mark("recv", 6))[mark("start", 0):int(json.loads("{")):mark("step", 0)]
        print(result.tolist())
    except json.JSONDecodeError as error:
        caught += 1
        print(render_error(error))
    print(caught)
    print(numpy.arange(3)[::2].tolist())


mode = sys.argv[1]
if mode == "numpy-torch":
    positive()
elif mode == "order-compatibility":
    compatibility()
elif mode in ("bound-error", "bound-error-shifted"):
    inner_failure()
elif mode in ("numpy-zero", "torch-zero", "torch-negative"):
    sys.exit(rejected_step(mode))
else:
    raise SystemExit(f"unknown reference case: {mode}")
