# PLAN-1129 — runtime float arguments reference

`float_args_ref.py` is the independent native-Python reference for
[python-float-args.case.ad](../../language/script/python-float-args.case.ad).
It reads no compiler output.

Run one case with the declared interpreter:

```text
python -I -B float_args_ref.py <case-name>
```

Python floats are doubles. Every value the Auto source holds as `float`
(f32) is rounded through `struct` at the step the language computes it, so
the reference follows the same width the program assigns. `show_f32` spells
a value the way the Auto console does, because the training case prints
`float` values rather than text.

The acceptance environment is Windows x64 with CPython 3.14.2 and CPU
PyTorch 2.14.0+cpu. AV uses its configured CPython installation. Set
`A2PY_PYTHON` to an interpreter that can import `torch` under `-I -B` for
the A2PY and reference checks. The `torch-lr-computed` case computes its
learning rate from dyadic constants, so the rate is exact in every width
and both runs end differently.

Prepare the AV provider and workers before running the cases with the
fixed-group preparation stage.
