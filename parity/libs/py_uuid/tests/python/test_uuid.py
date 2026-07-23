"""uuid module parity tests — uuid5 reference suite.

uuid5(namespace, name) is deterministic and SHA-1 based: the same inputs always
produce the same UUID string. The suite exercises two namespaces (NAMESPACE_DNS,
NAMESPACE_URL) plus a determinism check.

Expected values are produced by Python's own uuid module — the Auto test and
this oracle are kept byte-for-byte identical so the three-way comparator
(AutoVM, a2py, oracle) can join results by test name.
"""
import uuid


def tap_ok(n, name):
    print(f"ok {n} - {name}")


def tap_not_ok(n, name, diag):
    print(f"not ok {n} - {name} # {diag}")


if __name__ == "__main__":
    u = uuid.uuid5(uuid.NAMESPACE_DNS, "example.com")
    if str(u) == "cfbff0d1-9375-5685-968c-48ce8b15ae17":
        tap_ok(1, "test_uuid5_dns")
    else:
        tap_not_ok(1, "test_uuid5_dns", f"got {u}")

    u2 = uuid.uuid5(uuid.NAMESPACE_URL, "https://example.com")
    if str(u2) == "4fd35a71-71ef-5a55-a9d9-aa75c889a6d0":
        tap_ok(2, "test_uuid5_url")
    else:
        tap_not_ok(2, "test_uuid5_url", f"got {u2}")

    u3 = uuid.uuid5(uuid.NAMESPACE_DNS, "example.com")
    if str(u) == str(u3):
        tap_ok(3, "test_uuid5_deterministic")
    else:
        tap_not_ok(3, "test_uuid5_deterministic", "not deterministic")
