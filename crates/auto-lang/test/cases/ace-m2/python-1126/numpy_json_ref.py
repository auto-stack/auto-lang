# Independent native-Python reference for the `numpy-json` case (PLAN-1126).
# No ACE is involved: every value comes straight from NumPy and the standard
# `json` module, with the same fixed inputs as the Auto author source in
# language/script/python-libraries.case.ad. Each observed value is written as
# one UTF-8 JSON line so its type is explicit and the output does not depend
# on the host locale or on Python's text-mode newline translation.
import json
import sys

import numpy as np


def emit(value):
    sys.stdout.buffer.write((json.dumps(value, ensure_ascii=False) + "\n").encode("utf-8"))


# ---- NumPy (V-01) ---------------------------------------------------------
base = np.arange(6).reshape(2, 3)
a = base + np.arange(3)
assert a.tolist() == [[0, 2, 4], [3, 5, 7]], a.tolist()
shape = a.shape
row = a[1]
selected = row[0:3:2]  # NumPy's own strided slice, not the slice() object
assert selected.tolist() == [3, 7], selected.tolist()

emit(int(shape[0]))
emit(int(shape[1]))
emit(int(row[2]))
emit(int(a.sum()))
emit(int(selected.sum()))
emit(float(a.mean()))

# ---- json (V-02) ----------------------------------------------------------
text = '{"values":[2,3,5],"label":"雪","meta":{"enabled":true}}'
data = json.loads(text)
emit(data["values"][2])
emit(data["meta"]["enabled"])
encoded = json.dumps(data, sort_keys=True, ensure_ascii=False)
decoded = json.loads(encoded)
emit(decoded["label"])
emit(encoded)
emit(json.dumps(decoded, sort_keys=True, ensure_ascii=False))

# The decoded tree equals the original input structure, types included.
assert decoded == {"values": [2, 3, 5], "label": "雪", "meta": {"enabled": True}}
assert type(decoded["values"][2]) is int
assert type(decoded["meta"]["enabled"]) is bool
assert type(decoded["label"]) is str
