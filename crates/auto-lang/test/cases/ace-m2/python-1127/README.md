# Python slicing native reference

The Auto sources, expected streams and error observations are authored in
[python-slicing.case.ad](../../language/script/python-slicing.case.ad). The
same source runs through the ordinary AV and A2PY checks. This handwritten
reference uses Python subscription directly and reads no compiler output.

Run the reference as `python -I -B slicing_ref.py <case-name>`. The
acceptance environment is Windows x64 with CPython 3.14.2, NumPy 2.5.1 and
PyTorch 2.14.0+cpu. AV uses its configured CPython installation.
Set `A2PY_PYTHON` to an interpreter that can import both libraries under
`-I -B` for the A2PY and independent-reference checks. A user-site-only
installation is invisible in that mode. Prepare the AV provider and workers
before running the cases using the fixed-group preparation stage.
