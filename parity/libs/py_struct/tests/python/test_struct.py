"""struct module parity tests — calcsize (the subset that works via PyFFI)."""
import struct


def tap_ok(n, name):
    print(f"ok {n} - {name}")


def tap_not_ok(n, name, diag):
    print(f"not ok {n} - {name} # {diag}")


def check(n, name, actual, expected):
    if actual == expected:
        tap_ok(n, name)
    else:
        tap_not_ok(n, name, f"got {actual} expected {expected}")


if __name__ == "__main__":
    # calcsize — single string arg, int return. Works via PyFFI.
    check(1, "test_calcsize_uint_be", struct.calcsize(">I"), 4)
    check(2, "test_calcsize_ushort_be", struct.calcsize(">H"), 2)
    check(3, "test_calcsize_double", struct.calcsize(">d"), 8)
    check(4, "test_calcsize_char", struct.calcsize(">c"), 1)
    check(5, "test_calcsize_3char", struct.calcsize("3s"), 3)
    check(6, "test_calcsize_float", struct.calcsize(">f"), 4)
    check(7, "test_calcsize_longlong", struct.calcsize(">q"), 8)
    check(8, "test_calcsize_2int", struct.calcsize(">II"), 8)
