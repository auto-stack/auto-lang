// py_os Auto test — TAP output format (Plan 369 P6 Task 23).
//
// Targets the Python `os` / `os.path` standard-library modules via the PyFFI.
// The suite focuses on PURE, deterministic functions (no side-effecting
// filesystem mutation) so the three backends agree without coordinating state:
//
//   os.path.basename(p)   -> str                 (forward-slash input, portable)
//   os.path.dirname(p)    -> str                 (forward-slash input, portable)
//   os.path.splitext(p)   -> tuple(root, ext)    (handle; read via __getitem__)
//   os.path.join(a,b,c)   -> str                 (uses os.sep; asserted vs sep)
//   os.path.normpath(p)   -> str                 (collapses ..; follows os.sep)
//   os.path.exists(p)     -> bool -> int         (read-only predicate)
//   os.listdir(p)         -> list[str]           (handle; len via __len__)
//
// Platform note: os.sep / join / normpath produce platform-specific separators
// (\\ on Windows, / on POSIX). Rather than hardcoding a literal that only
// passes on one platform, the Auto test asserts the join result against `sep`
// (imported as a constant via use.py) and builds the expected string
// dynamically. Because the oracle, a2py, and AutoVM all run on the same host,
// they observe the same os.sep and agree.
//
// A tuple returned by splitext is held as an opaque PyObjectHandle; it is read
// via py_call(sp, "__getitem__", i). A list returned by listdir is likewise a
// handle; its length is read via py_call(lst, "__len__"). Neither handle can be
// printed directly (to(str) yields a raw handle marker), so every assertion
// goes through element access or a method that returns a primitive.
//
// The parity runner sets the working directory to the library root
// (parity/libs/py_os), so exists(".") and listdir(".") refer to this directory.
// Test names MUST match tests/python/test_os.py because the comparator joins
// backends by name.
use.py os: listdir, sep
use.py os.path: join, exists, basename, dirname, splitext, normpath

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // sep is imported as a module constant; it marshals to its string form
    // ( "\\" on Windows, "/" on POSIX ).
    var sepstr = sep.to(str)

    // 1. basename extracts the final path component.
    var b = basename("/tmp/test.txt")
    if b == "test.txt" {
        tap_ok(1, "test_basename")
    } else {
        tap_not_ok(1, "test_basename", "got " + b.to(str))
    }

    // 2. dirname extracts the directory portion.
    var d = dirname("/tmp/test.txt")
    if d == "/tmp" {
        tap_ok(2, "test_dirname")
    } else {
        tap_not_ok(2, "test_dirname", "got " + d.to(str))
    }

    // 3. splitext returns a (root, ext) tuple handle. Read via __getitem__.
    var sp = splitext("/tmp/test.txt")
    var sp0 = py_call(sp, "__getitem__", 0)
    var sp1 = py_call(sp, "__getitem__", 1)
    if sp0 == "/tmp/test" {
        if sp1 == ".txt" {
            tap_ok(3, "test_splitext")
        } else {
            tap_not_ok(3, "test_splitext", "ext got " + sp1.to(str))
        }
    } else {
        tap_not_ok(3, "test_splitext", "root got " + sp0.to(str))
    }

    // 4. join concatenates with os.sep. Build the expected string from sep so
    //    the assertion is portable.
    var j = join("a", "b", "c")
    var expected4 = "a" + sepstr + "b" + sepstr + "c"
    if j == expected4 {
        tap_ok(4, "test_join_uses_sep")
    } else {
        tap_not_ok(4, "test_join_uses_sep", "got " + j.to(str))
    }

    // 5. normpath collapses `..`; the separator follows os.sep.
    var np = normpath("a/b/../c")
    var expected5 = "a" + sepstr + "c"
    if np == expected5 {
        tap_ok(5, "test_normpath")
    } else {
        tap_not_ok(5, "test_normpath", "got " + np.to(str))
    }

    // 6. exists is a read-only predicate; the working directory always exists.
    var e = exists(".")
    if e == 1 {
        tap_ok(6, "test_exists_dot")
    } else {
        tap_not_ok(6, "test_exists_dot", "got " + e.to(str))
    }

    // 7. exists on a path that does not exist returns 0.
    var ne = exists("definitely_no_such_dir_xyz_123")
    if ne == 0 {
        tap_ok(7, "test_exists_missing")
    } else {
        tap_not_ok(7, "test_exists_missing", "got " + ne.to(str))
    }

    // 8. listdir returns the entry names as a list handle. The lib dir holds
    //    the test files, so its length is >= 1.
    var lst = listdir(".")
    var n = py_call(lst, "__len__")
    if n >= 1 {
        tap_ok(8, "test_listdir_len")
    } else {
        tap_not_ok(8, "test_listdir_len", "got " + n.to(str))
    }
}
