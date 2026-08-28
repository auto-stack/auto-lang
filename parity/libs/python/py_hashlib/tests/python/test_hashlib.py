"""Python oracle for py_hashlib parity (Plan 369 P6 Task 27).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/hashlib.at) because the comparator joins backends by name.

Scope
-----
Plan 369 P6 Task 27 targets the Python `hashlib` standard-library module.
The suite covers hash-object construction and its read-only metadata
attributes:

- `hashlib.new(algo_name)`         -> hash object
- `h.name`                         -> str (algorithm name)
- `h.digest_size`                  -> int (digest length in bytes)
- `h.block_size`                   -> int (internal block size in bytes)

This oracle IS the source of truth, so by construction it emits `ok` for every
case. The Auto test (hashlib.at) performs the same operations and checks them
against the same expected values; if AutoVM's PyFFI reproduces the result it
emits `ok` too (consistent), otherwise `not ok` (an AutoVM bug).

DATA HASHING IS NOT COVERED — documented limitation
---------------------------------------------------
hashlib's data-taking constructors and `update()` require Python `bytes`, but
the AutoVM PyFFI marshals an Auto str to a Python str (not bytes), so:

    sha256("hello")   -> TypeError: Strings must be encoded before hashing
    md5("hello")      -> same
    h.update("hello") -> TypeError: object supporting the buffer API required

Additionally the NO-ARG constructors (`sha256()`, `md5()`, ...) hit the PyFFI
extra-arg bug (a stray non-bytes arg is passed), so even an empty hash object
cannot be constructed that way. The ONLY reliable construction path is
`new(algo_name)` with a single string arg, which yields a fresh empty hash
object whose metadata attributes are readable. So this suite validates
hash-object METADATA (algorithm identity + sizes), not digests. There is no
`hexdigest()` / `update()` coverage because those require a `bytes` payload
that Auto cannot currently produce.
"""
import hashlib


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    # 1. new("md5") -> name "md5".
    m = hashlib.new("md5")
    mnm = m.name
    if mnm == "md5":
        tap_ok(1, "test_md5_name")
    else:
        tap_not_ok(1, "test_md5_name", "got {}".format(mnm))

    # 2. md5 digest_size == 16.
    mds = m.digest_size
    if mds == 16:
        tap_ok(2, "test_md5_digest_size")
    else:
        tap_not_ok(2, "test_md5_digest_size", "got {}".format(mds))

    # 3. new("sha256") -> name "sha256".
    s = hashlib.new("sha256")
    snm = s.name
    if snm == "sha256":
        tap_ok(3, "test_sha256_name")
    else:
        tap_not_ok(3, "test_sha256_name", "got {}".format(snm))

    # 4. sha256 digest_size == 32 AND block_size == 64.
    sds = s.digest_size
    sbs = s.block_size
    if sds == 32 and sbs == 64:
        tap_ok(4, "test_sha256_sizes")
    else:
        tap_not_ok(4, "test_sha256_sizes", "got {} {}".format(sds, sbs))

    # 5. new("sha512") -> digest_size 64, block_size 128.
    s512 = hashlib.new("sha512")
    s512ds = s512.digest_size
    s512bs = s512.block_size
    if s512ds == 64 and s512bs == 128:
        tap_ok(5, "test_sha512_sizes")
    else:
        tap_not_ok(5, "test_sha512_sizes", "got {} {}".format(s512ds, s512bs))
