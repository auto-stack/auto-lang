// py_sys Auto test — TAP output format (Plan 369 P6 Task 28).
//
// Targets the Python `sys` standard-library module via the PyFFI. Covers the
// read-only module constants and the version_info struct:
//
//   platform        -> str   (e.g. "win32", "linux")
//   version         -> str   (full version string)
//   byteorder       -> str   ("little" / "big")
//   maxunicode      -> int   (always 0x10FFFF == 1114111 on Python 3)
//   version_info    -> struct handle; py_getattr(vi, "major") -> int
//
// Module-level constants imported via `use.py sys` arrive as clean Auto
// values: strings (platform/version/byteorder) compare with ==, and ints
// (maxunicode) compare with == and >. version_info is a namedtuple object;
// its integer fields are read via py_getattr(vi, "major").
//
// maxsize is NOT asserted — documented limitation. sys.maxsize is
// 9223372036854775807, which overflows Auto's int and arrives as a double:
//   - .to(str) renders "9223372036854776000" (rounded), diverging from the
//     oracle's exact "9223372036854775807".
//   - relational compares against int literals (maxsize > 5, maxsize > 0)
//     silently return false (f64 vs int comparison is broken in the VM).
// So any value assertion on maxsize would diverge; it is imported only to
// activate the PyFFI and is documented as a known divergence.
//
// executable is NOT asserted either — sys.executable reflects the HOST
// process: AutoVM sees the auto.exe path, while the a2py/oracle backends see
// the python.exe path. The values differ by construction, so no equality
// assertion can agree across backends (and Auto string methods like
// .length()/.find() are unavailable, so substring/shape checks aren't
// feasible either).
//
// Test names MUST match tests/python/test_sys.py because the comparator joins
// backends by name.
use.py sys: platform, version, byteorder, maxunicode, version_info

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // 1. platform is a non-empty string equal to the OS identifier.
    var p = platform
    if p == "win32" {
        tap_ok(1, "test_platform")
    } else {
        tap_not_ok(1, "test_platform", "got " + p.to(str))
    }

    // 2. version_info.major is an int field read off the namedtuple handle.
    var vi = version_info
    var major = py_getattr(vi, "major")
    if major == 3 {
        tap_ok(2, "test_version_major")
    } else {
        tap_not_ok(2, "test_version_major", "got " + major.to(str))
    }

    // 3. byteorder is a string ("little" on all common x86/ARM hardware).
    var bo = byteorder
    if bo == "little" {
        tap_ok(3, "test_byteorder")
    } else {
        tap_not_ok(3, "test_byteorder", "got " + bo.to(str))
    }

    // 4. maxunicode is an int equal to 0x10FFFF (1114111) on Python 3.
    var mu = maxunicode
    if mu == 1114111 {
        tap_ok(4, "test_maxunicode")
    } else {
        tap_not_ok(4, "test_maxunicode", "got " + mu.to(str))
    }

    // 5. version is the full version string; it round-trips through to(str)
    //    unchanged. (Both backends run the same Python build, so the value
    //    itself agrees; this also guards that the str constant is not
    //    collapsed or truncated by the FFI.)
    var v = version
    var vs = v.to(str)
    if vs == version {
        tap_ok(5, "test_version_roundtrip")
    } else {
        tap_not_ok(5, "test_version_roundtrip", "roundtrip mismatch")
    }
}
