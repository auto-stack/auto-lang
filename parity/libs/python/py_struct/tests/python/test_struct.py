"""struct module parity tests — calcsize + pack/unpack round-trips.

Plan 369 P3 fixed multi-arg FFI calls, so pack(fmt, *values) and
unpack(fmt, buffer) now work. The Auto test reaches decoded tuple values via
py_call(tup, "__getitem__", i); this oracle mirrors the same cases.

Test names match tests/auto/struct.at because the comparator joins backends
by name.
"""
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
    # calcsize — single string arg, int return.
    check(1, "test_calcsize_uint_be", struct.calcsize(">I"), 4)
    check(2, "test_calcsize_ushort_be", struct.calcsize(">H"), 2)
    check(3, "test_calcsize_double", struct.calcsize(">d"), 8)
    check(4, "test_calcsize_char", struct.calcsize(">c"), 1)
    check(5, "test_calcsize_3char", struct.calcsize("3s"), 3)
    check(6, "test_calcsize_float", struct.calcsize(">f"), 4)
    check(7, "test_calcsize_longlong", struct.calcsize(">q"), 8)
    check(8, "test_calcsize_2int", struct.calcsize(">II"), 8)

    # pack/unpack round-trips.
    p1 = struct.pack(">I", 258)
    u1 = struct.unpack(">I", p1)
    if u1[0] == 258:
        tap_ok(9, "test_pack_unpack_uint_be")
    else:
        tap_not_ok(9, "test_pack_unpack_uint_be", f"got {u1[0]}")

    p2 = struct.pack(">HH", 1, 2)
    u2 = struct.unpack(">HH", p2)
    if u2[0] == 1 and u2[1] == 2:
        tap_ok(10, "test_pack_unpack_two_ushort_be")
    else:
        tap_not_ok(10, "test_pack_unpack_two_ushort_be", f"got {u2}")

    p3 = struct.pack(">i", -1)
    u3 = struct.unpack(">i", p3)
    if u3[0] == -1:
        tap_ok(11, "test_pack_unpack_signed_int")
    else:
        tap_not_ok(11, "test_pack_unpack_signed_int", f"got {u3[0]}")

    p4 = struct.pack(">B", 255)
    u4 = struct.unpack(">B", p4)
    if u4[0] == 255:
        tap_ok(12, "test_pack_unpack_byte")
    else:
        tap_not_ok(12, "test_pack_unpack_byte", f"got {u4[0]}")
