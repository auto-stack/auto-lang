// py_hashlib Auto test — TAP output format (Plan 369 P6 Task 27).
//
// Targets the Python `hashlib` standard-library module via the PyFFI. Covers
// hash-object construction and its read-only metadata attributes:
//
//   new("sha256")               -> hash object handle (single str arg avoids
//                                  the no-arg extra-arg bug)
//   py_getattr(h, "name")       -> str (algorithm name)
//   py_getattr(h, "digest_size") -> int (comparable)
//   py_getattr(h, "block_size")  -> int (comparable)
//
// A hash object returned by `new()` is held as an opaque PyObjectHandle; Auto
// reads its attributes via py_getattr(h, "attr"). Returned str/int attributes
// come back as clean Auto values and compare to literals normally.
//
// DATA HASHING IS NOT COVERED — documented limitation. hashlib's data-taking
// constructors and update() require Python `bytes`, but the AutoVM PyFFI
// marshals an Auto str to a Python str (not bytes), so:
//   sha256("hello")   -> TypeError: Strings must be encoded before hashing
//   md5("hello")      -> same
//   h.update("hello") -> TypeError: object supporting the buffer API required
// Additionally the NO-ARG constructors (sha256(), md5(), ...) hit the PyFFI
// extra-arg bug (a stray non-bytes arg is passed), so even an empty hash
// object cannot be constructed that way. The ONLY reliable construction path
// is `new(algo_name)` with a single string arg, which yields a fresh empty
// hash object whose metadata attributes are then readable. So this suite
// validates hash-object METADATA (algorithm identity + sizes), not digests.
//
// Test names MUST match tests/python/test_hashlib.py because the comparator
// joins backends by name.
use.py hashlib: new

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // 1. new("md5") -> name "md5", digest_size 16.
    var m = new("md5")
    var mnm = py_getattr(m, "name")
    if mnm == "md5" {
        tap_ok(1, "test_md5_name")
    } else {
        tap_not_ok(1, "test_md5_name", "got " + mnm.to(str))
    }

    // 2. md5 digest_size == 16.
    var mds = py_getattr(m, "digest_size")
    if mds == 16 {
        tap_ok(2, "test_md5_digest_size")
    } else {
        tap_not_ok(2, "test_md5_digest_size", "got " + mds.to(str))
    }

    // 3. new("sha256") -> name "sha256", digest_size 32, block_size 64.
    var s = new("sha256")
    var snm = py_getattr(s, "name")
    if snm == "sha256" {
        tap_ok(3, "test_sha256_name")
    } else {
        tap_not_ok(3, "test_sha256_name", "got " + snm.to(str))
    }

    // 4. sha256 digest_size == 32 AND block_size == 64.
    var sds = py_getattr(s, "digest_size")
    var sbs = py_getattr(s, "block_size")
    if sds == 32 {
        if sbs == 64 {
            tap_ok(4, "test_sha256_sizes")
        } else {
            tap_not_ok(4, "test_sha256_sizes", "block_size got " + sbs.to(str))
        }
    } else {
        tap_not_ok(4, "test_sha256_sizes", "digest_size got " + sds.to(str))
    }

    // 5. new("sha512") -> digest_size 64, block_size 128.
    var s512 = new("sha512")
    var s512ds = py_getattr(s512, "digest_size")
    var s512bs = py_getattr(s512, "block_size")
    if s512ds == 64 {
        if s512bs == 128 {
            tap_ok(5, "test_sha512_sizes")
        } else {
            tap_not_ok(5, "test_sha512_sizes", "block_size got " + s512bs.to(str))
        }
    } else {
        tap_not_ok(5, "test_sha512_sizes", "digest_size got " + s512ds.to(str))
    }
}
