"""Plan 567 T20 (W3 annotation oracle): annotated pure functions.

The return annotations are the contract Auto's registration-time oracle
reads (typing.get_type_hints). `fakey` deliberately returns an int under a
float annotation — Python tolerates it, and Auto's D4-authorized coercion
(GIL float()) must too.
"""


def scale(x: float) -> float:
    return x * 2


def half(x: float) -> float:
    return x / 2


def count_up(n: int) -> int:
    return n + 10


def find_lo(idx: int) -> 'int | None':
    data = [10, 20, 30]
    if 0 <= idx < len(data):
        return data[idx]
    return None


def mix(a: float, b: float) -> float:
    return a + b


def fakey(x: float) -> float:
    return 7
