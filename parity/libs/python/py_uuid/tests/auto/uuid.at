// py_uuid Auto test — TAP output format (Plan 369 Python parity).
// The parity runner sets the working directory to the library root
// (parity/libs/py_uuid), so `use.py uuid` resolves the Python stdlib module.
//
// Test names MUST match tests/python/test_uuid.py because the comparator joins
// backends by name.
//
// uuid5(namespace, name) is deterministic and SHA-1 based, so the same inputs
// always produce the same UUID string. It requires:
//   - NAMESPACE_DNS / NAMESPACE_URL — Python module constants, now importable
//     via `use.py uuid: NAMESPACE_DNS` (Plan 369 P3 fixed constant imports;
//     P4 Task 13 fixed the parser so constants resolve in argument position).
//   - a UUID object as the first argument to uuid5 (multi-arg FFI, fixed in P3).
// The UUID object marshals to an opaque heap value, so we reach its string
// form via py_call(u, "__str__") (Plan 369 Task 12).
use.py uuid: uuid5, NAMESPACE_DNS, NAMESPACE_URL

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // uuid5(NAMESPACE_DNS, "example.com") is deterministic.
    var u = uuid5(NAMESPACE_DNS, "example.com")
    var s = py_call(u, "__str__")
    if s == "cfbff0d1-9375-5685-968c-48ce8b15ae17" {
        tap_ok(1, "test_uuid5_dns")
    } else {
        tap_not_ok(1, "test_uuid5_dns", "got " + s)
    }

    // Different namespace → different UUID.
    var u2 = uuid5(NAMESPACE_URL, "https://example.com")
    var s2 = py_call(u2, "__str__")
    if s2 == "4fd35a71-71ef-5a55-a9d9-aa75c889a6d0" {
        tap_ok(2, "test_uuid5_url")
    } else {
        tap_not_ok(2, "test_uuid5_url", "got " + s2)
    }

    // Determinism: same input → same output.
    var u3 = uuid5(NAMESPACE_DNS, "example.com")
    var s3 = py_call(u3, "__str__")
    if s3 == s {
        tap_ok(3, "test_uuid5_deterministic")
    } else {
        tap_not_ok(3, "test_uuid5_deterministic", "not deterministic")
    }
}
