"""Python oracle for py_re parity (Plan 369 P6 Task 24).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/re.at) because the comparator joins backends by name.

Scope
-----
Plan 369 P6 Task 24 targets the Python `re` standard-library module. The suite
covers the four primary regex operations and the types they return across the
PyFFI boundary:

- `re.sub(pat, repl, str)`         -> str                 (clean scalar)
- `re.sub(pat, repl, str, count)`  -> str                 (4-arg call)
- `re.findall(pat, str)`           -> list[str]           (handle; __len__)
- `re.search(pat, str)`            -> Match object        (handle; group/start/end)
- `re.split(pat, str)`             -> list[str]           (handle; __getitem__)
- `re.escape(str)`                 -> str                 (pure)

This oracle IS the source of truth, so by construction it emits `ok` for every
case. The Auto test (re.at) performs the same operations and checks them
against the same expected values; if AutoVM's PyFFI reproduces the result it
emits `ok` too (consistent), otherwise `not ok` (an AutoVM bug).

Regex literal note
------------------
Auto has no raw-string (`r"..."`) syntax, so regex patterns are written with
escaped backslashes (`"\\d+"`) in BOTH this oracle and the Auto test. Using the
same escaped form on both sides keeps the literal byte-for-byte identical and
avoids any cross-backend interpretation drift.
"""
import re


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    # 1. sub replaces all matches by default.
    r = re.sub("\\d+", "N", "a1b22c333")
    if r == "aNbNcN":
        tap_ok(1, "test_sub_all")
    else:
        tap_not_ok(1, "test_sub_all", "got {}".format(r))

    # 2. sub with a count limits the number of replacements (4-arg call).
    rc = re.sub("\\d+", "N", "a1b22c333", 1)
    if rc == "aNb22c333":
        tap_ok(2, "test_sub_count")
    else:
        tap_not_ok(2, "test_sub_count", "got {}".format(rc))

    # 3. findall returns a list of matches. Read its length and elements.
    fa = re.findall("\\d+", "a1b22c333")
    if fa.__len__() == 3 and fa.__getitem__(0) == "1" and fa.__getitem__(2) == "333":
        tap_ok(3, "test_findall")
    else:
        tap_not_ok(3, "test_findall", "got {} {} {}".format(
            fa.__len__(), fa.__getitem__(0), fa.__getitem__(2)))

    # 4. search returns a Match object; group(n) reads capture groups.
    m = re.search("(\\w+)@(\\w+)", "user@example.com")
    g0 = m.group(0)
    g1 = m.group(1)
    g2 = m.group(2)
    if g0 == "user@example" and g1 == "user" and g2 == "example":
        tap_ok(4, "test_search_groups")
    else:
        tap_not_ok(4, "test_search_groups", "got {} {} {}".format(g0, g1, g2))

    # 5. Match.start()/end() give the match span offsets (ints).
    m2 = re.search("\\d+", "abc123def")
    if m2.start() == 3 and m2.end() == 6:
        tap_ok(5, "test_match_span")
    else:
        tap_not_ok(5, "test_match_span", "got {} {}".format(m2.start(), m2.end()))

    # 6. A pattern that does not match yields an EMPTY findall list. This is a
    #    clean, portable "no match" signal: the empty list is a real Python list
    #    on every backend, so __len__() == 0 agrees across the oracle, a2py, and
    #    AutoVM. (re.search on no match returns Python None; the AutoVM marshals
    #    that to int 0, but pure Python's `None == 0` is False, so a search-based
    #    no-match assertion diverges between AutoVM and a2py. findall-empty
    #    avoids that — see the README "Known limitations".)
    fa_empty = re.findall("zzz", "abc")
    if fa_empty.__len__() == 0:
        tap_ok(6, "test_findall_no_match")
    else:
        tap_not_ok(6, "test_findall_no_match", "got {}".format(fa_empty.__len__()))

    # 7. split returns a list of the pieces between matches.
    sp = re.split("\\s+", "a  b   c")
    if sp.__len__() == 3 and sp.__getitem__(0) == "a" and sp.__getitem__(2) == "c":
        tap_ok(7, "test_split")
    else:
        tap_not_ok(7, "test_split", "got {} {} {}".format(
            sp.__len__(), sp.__getitem__(0), sp.__getitem__(2)))

    # 8. escape escapes regex metacharacters; pure str -> str.
    esc = re.escape("a.b*c")
    if esc == "a\\.b\\*c":
        tap_ok(8, "test_escape")
    else:
        tap_not_ok(8, "test_escape", "got {}".format(esc))
