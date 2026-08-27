/// struct module parity tests — calcsize + pack/unpack round-trips.
///
/// Plan 369 P3 fixed multi-arg FFI calls, so pack(fmt, *values) and
/// unpack(fmt, buffer) now work. pack returns bytes and unpack returns a
/// tuple; both marshal to opaque heap values in the AutoVM, so decoded values
/// are reached via py_call(tup, "__getitem__", i) (Plan 369 Task 12).
///
/// Test names MUST match tests/python/test_struct.py.
use.py struct: calcsize, pack, unpack

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn check(n int, name str, actual int, expected int) {
    if actual == expected {
        tap_ok(n, name)
    } else {
        tap_not_ok(n, name, "got " + actual.to(str) + " expected " + expected.to(str))
    }
}

fn main() {
    // calcsize — single string arg, int return.
    check(1, "test_calcsize_uint_be", calcsize(">I"), 4)
    check(2, "test_calcsize_ushort_be", calcsize(">H"), 2)
    check(3, "test_calcsize_double", calcsize(">d"), 8)
    check(4, "test_calcsize_char", calcsize(">c"), 1)
    check(5, "test_calcsize_3char", calcsize("3s"), 3)
    check(6, "test_calcsize_float", calcsize(">f"), 4)
    check(7, "test_calcsize_longlong", calcsize(">q"), 8)
    check(8, "test_calcsize_2int", calcsize(">II"), 8)

    // pack/unpack round-trips (multi-arg FFI). unpack returns a tuple; reach
    // the decoded value(s) via py_call(tup, "__getitem__", i).

    // Big-endian uint32 round-trip.
    var p1 = pack(">I", 258)
    var u1 = unpack(">I", p1)
    var v1 = py_call(u1, "__getitem__", 0)
    if v1 == 258 {
        tap_ok(9, "test_pack_unpack_uint_be")
    } else {
        tap_not_ok(9, "test_pack_unpack_uint_be", "got " + v1.to(str))
    }

    // Multi-value pack: two big-endian uint16 values.
    var p2 = pack(">HH", 1, 2)
    var u2 = unpack(">HH", p2)
    var a2 = py_call(u2, "__getitem__", 0)
    var b2 = py_call(u2, "__getitem__", 1)
    if a2 == 1 {
        if b2 == 2 {
            tap_ok(10, "test_pack_unpack_two_ushort_be")
        } else {
            tap_not_ok(10, "test_pack_unpack_two_ushort_be", "second got " + b2.to(str))
        }
    } else {
        tap_not_ok(10, "test_pack_unpack_two_ushort_be", "first got " + a2.to(str))
    }

    // Signed int32 round-trip (negative value).
    var p3 = pack(">i", -1)
    var u3 = unpack(">i", p3)
    var v3 = py_call(u3, "__getitem__", 0)
    if v3 == -1 {
        tap_ok(11, "test_pack_unpack_signed_int")
    } else {
        tap_not_ok(11, "test_pack_unpack_signed_int", "got " + v3.to(str))
    }

    // Unsigned char (single byte) round-trip.
    var p4 = pack(">B", 255)
    var u4 = unpack(">B", p4)
    var v4 = py_call(u4, "__getitem__", 0)
    if v4 == 255 {
        tap_ok(12, "test_pack_unpack_byte")
    } else {
        tap_not_ok(12, "test_pack_unpack_byte", "got " + v4.to(str))
    }
}
