"""Python oracle for py_os parity (Plan 369 P6 Task 23).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/os.at) because the comparator joins backends by name.

Scope
-----
Plan 369 P6 Task 23 targets the Python `os` / `os.path` standard-library
modules. The suite focuses on PURE, deterministic functions (no side-effecting
filesystem mutation) so the three backends agree without coordinating state:

- `os.path.basename(p)`  -> str (platform-agnostic for forward-slash input)
- `os.path.dirname(p)`   -> str (platform-agnostic for forward-slash input)
- `os.path.splitext(p)`  -> tuple(str, str) (the root and the extension)
- `os.path.join(a, b, c)`-> str (uses os.sep; asserted against os.sep, not a
  hardcoded separator, so it passes on both POSIX and Windows)
- `os.path.normpath(p)`  -> str (collapses `..`; separator follows os.sep)
- `os.path.exists(p)`    -> bool (read-only existence check) -> int
- `os.listdir(p)`        -> list[str] handle (the dir's entry names)

This oracle IS the source of truth, so by construction it emits `ok` for every
case. The Auto test (os.at) performs the same operations and checks them
against the same expected values; if AutoVM's PyFFI reproduces the result it
emits `ok` too (consistent), otherwise `not ok` (an AutoVM bug).

Platform note
-------------
`os.sep`, `os.path.join`, and `os.path.normpath` produce platform-specific
separators (`\\` on Windows, `/` on POSIX). Rather than hardcoding a literal
that would only pass on one platform, the oracle and the Auto test both assert
the join result against `os.sep` (imported as a constant) and construct the
expected string dynamically. Because the oracle, the a2py backend, and the
AutoVM all run on the same host, they observe the same platform and agree.
"""
import os


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    sep = os.sep  # imported-and-recomputed constant; asserted as str below

    # 1. basename extracts the final path component. For forward-slash input
    #    the result is platform-agnostic.
    if os.path.basename("/tmp/test.txt") == "test.txt":
        tap_ok(1, "test_basename")
    else:
        tap_not_ok(1, "test_basename", "got {}".format(os.path.basename("/tmp/test.txt")))

    # 2. dirname extracts the directory portion. Forward-slash input keeps it
    #    platform-agnostic.
    if os.path.dirname("/tmp/test.txt") == "/tmp":
        tap_ok(2, "test_dirname")
    else:
        tap_not_ok(2, "test_dirname", "got {}".format(os.path.dirname("/tmp/test.txt")))

    # 3. splitext returns a (root, ext) tuple. Indexing via __getitem__ mirrors
    #    how the Auto side reads the tuple handle.
    sp = os.path.splitext("/tmp/test.txt")
    if sp.__getitem__(0) == "/tmp/test" and sp.__getitem__(1) == ".txt":
        tap_ok(3, "test_splitext")
    else:
        tap_not_ok(3, "test_splitext", "got {} {}".format(sp.__getitem__(0), sp.__getitem__(1)))

    # 4. join concatenates with os.sep. Assert against sep (not a hardcoded
    #    literal) so the test is portable.
    j = os.path.join("a", "b", "c")
    expected = "a" + sep + "b" + sep + "c"
    if j == expected:
        tap_ok(4, "test_join_uses_sep")
    else:
        tap_not_ok(4, "test_join_uses_sep", "got {}".format(j))

    # 5. normpath collapses `..` segments; the separator follows os.sep.
    np = os.path.normpath("a/b/../c")
    if np == "a" + sep + "c":
        tap_ok(5, "test_normpath")
    else:
        tap_not_ok(5, "test_normpath", "got {}".format(np))

    # 6. exists is a read-only boolean predicate. The script's own working
    #    directory always exists, so exists(".") == True.
    e = os.path.exists(".")
    if e == 1:
        tap_ok(6, "test_exists_dot")
    else:
        tap_not_ok(6, "test_exists_dot", "got {}".format(e))

    # 7. exists on a path that does not exist returns False (0).
    ne = os.path.exists("definitely_no_such_dir_xyz_123")
    if ne == 0:
        tap_ok(7, "test_exists_missing")
    else:
        tap_not_ok(7, "test_exists_missing", "got {}".format(ne))

    # 8. listdir returns the entry names of a directory as a list. The working
    #    directory is non-empty (it contains the test files), so its length is
    #    >= 1. Assert only the lower bound to stay deterministic.
    lst = os.listdir(".")
    n = lst.__len__()
    if n >= 1:
        tap_ok(8, "test_listdir_len")
    else:
        tap_not_ok(8, "test_listdir_len", "got {}".format(n))
