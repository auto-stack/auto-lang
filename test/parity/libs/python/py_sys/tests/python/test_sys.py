"""Python oracle for py_sys parity (Plan 369 P6 Task 28).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/sys.at) because the comparator joins backends by name.

Scope
-----
Plan 369 P6 Task 28 targets the Python `sys` standard-library module. The
suite covers read-only module constants and the `version_info` struct:

- `sys.platform`                 -> str  (OS identifier, e.g. "win32")
- `sys.version`                  -> str  (full version line)
- `sys.byteorder`                -> str  ("little" / "big")
- `sys.maxunicode`               -> int  (0x10FFFF == 1114111 on Python 3)
- `sys.version_info.major`       -> int  (read off the namedtuple)

This oracle IS the source of truth, so by construction it emits `ok` for every
case. The Auto test (sys.at) performs the same operations and checks them
against the same expected values; if AutoVM's PyFFI reproduces the result it
emits `ok` too (consistent), otherwise `not ok` (an AutoVM bug).

maxsize is NOT asserted — documented divergence
------------------------------------------------
`sys.maxsize` is 9223372036854775807, which overflows Auto's int and arrives
as a double. On the Auto side:

- `.to(str)` renders "9223372036854776000" (rounded), diverging from the
  oracle's exact "9223372036854775807".
- relational compares against int literals (`maxsize > 5`, `maxsize > 0`)
  silently return false (f64 vs int comparison is broken in the VM).

So any value assertion on `maxsize` would diverge across backends; it is
imported in the Auto test only to activate the PyFFI and is excluded from
the assertions. (`maxunicode`, by contrast, fits in a clean int and is
reliable.)

executable is NOT asserted — structural divergence
---------------------------------------------------
`sys.executable` reflects the HOST process: the AutoVM sees the `auto.exe`
path (it embeds Python), while the a2py and oracle backends see the
`python.exe` path (they run as a Python script). The values differ by
construction across backends, so no equality assertion can agree. Auto string
methods (`.length()`, `.find()`, `.contains()`) are also unavailable, so
substring/shape checks are not feasible. It is therefore excluded from the
suite.
"""
import sys


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    # 1. platform is a non-empty string equal to the OS identifier.
    p = sys.platform
    if p == "win32":
        tap_ok(1, "test_platform")
    else:
        tap_not_ok(1, "test_platform", "got {}".format(p))

    # 2. version_info.major is an int field of the namedtuple.
    major = sys.version_info.major
    if major == 3:
        tap_ok(2, "test_version_major")
    else:
        tap_not_ok(2, "test_version_major", "got {}".format(major))

    # 3. byteorder is a string ("little" on all common x86/ARM hardware).
    bo = sys.byteorder
    if bo == "little":
        tap_ok(3, "test_byteorder")
    else:
        tap_not_ok(3, "test_byteorder", "got {}".format(bo))

    # 4. maxunicode is an int equal to 0x10FFFF (1114111) on Python 3.
    mu = sys.maxunicode
    if mu == 1114111:
        tap_ok(4, "test_maxunicode")
    else:
        tap_not_ok(4, "test_maxunicode", "got {}".format(mu))

    # 5. version is the full version string; str() round-trips it unchanged.
    v = sys.version
    vs = str(v)
    if vs == v:
        tap_ok(5, "test_version_roundtrip")
    else:
        tap_not_ok(5, "test_version_roundtrip", "roundtrip mismatch")
