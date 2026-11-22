# Independent native-Python reference for the `numpy-shape-error-caught` case
# (PLAN-1126). No ACE is involved: NumPy raises the exception itself and the
# handler reports the ORIGINAL class name and message.
import json
import sys

import numpy as np


def emit(value):
    sys.stdout.buffer.write((json.dumps(value, ensure_ascii=False) + "\n").encode("utf-8"))


caught = 0
try:
    bad = np.arange(6).reshape(4, 2)
    emit(len(bad))  # never reached: the reshape fails first
except ValueError as error:
    emit(f"{type(error).__qualname__}: {error}")
    caught += 1
emit(caught)

ok = np.arange(6).reshape(2, 3)
emit(ok.shape[0])
