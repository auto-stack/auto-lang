"""uuid module parity tests — uuid5 reference suite.

NOTE: uuid5(namespace, name) requires:
1. A UUID object as first arg (NAMESPACE_DNS is a module constant — not importable via PyFFI)
2. Two-arg FFI call

Both are currently broken. This oracle runs the full suite for reference.
"""
import uuid


def tap_ok(n, name):
    print(f"ok {n} - {name}")


def tap_not_ok(n, name, diag):
    print(f"not ok {n} - {name} # {diag}")


if __name__ == "__main__":
    u = uuid.uuid5(uuid.NAMESPACE_DNS, "example.com")
    if str(u) == "cfbff0d1-9375-5685-968c-48ce8b15ae0b":
        tap_ok(1, "test_uuid5_dns")
    else:
        tap_not_ok(1, "test_uuid5_dns", f"got {u}")

    u2 = uuid.uuid5(uuid.NAMESPACE_URL, "https://example.com")
    if str(u2) == "a8a2b2c8-5b33-5b87-b15c-cb1d4875f8b5":
        tap_ok(2, "test_uuid5_url")
    else:
        tap_not_ok(2, "test_uuid5_url", f"got {u2}")

    u3 = uuid.uuid5(uuid.NAMESPACE_DNS, "example.com")
    if str(u) == str(u3):
        tap_ok(3, "test_uuid5_deterministic")
    else:
        tap_not_ok(3, "test_uuid5_deterministic", "not deterministic")
