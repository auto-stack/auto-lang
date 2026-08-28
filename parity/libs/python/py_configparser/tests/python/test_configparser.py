"""Python oracle for py_configparser parity (Plan 369 P6 Task 26).

Emits TAP output (`ok N - <name>` / `not ok N - <name> # <diag>`) so the
auto-parity runner can parse it with the same TAP parser used for the AutoVM
and a2py backends. Test names MUST match the Auto test file
(tests/auto/configparser.at) because the comparator joins backends by name.

Scope
-----
Plan 369 P6 Task 26 targets the Python `configparser` standard-library module.
The suite covers INI parsing on FLAT config (sections whose values are
scalars: str / int / bool):

- `ConfigParser()`                  -> parser object
- `cfg.read_string(ini_text)`       -> parse INI text
- `cfg.get(section, key)`           -> str value
- `cfg.getint(section, key)`        -> int value (compared to a literal)
- `cfg.getboolean(section, key)`    -> bool (True case; see limitation note)
- `cfg.sections().__len__()`        -> section count

This oracle IS the source of truth, so by construction it emits `ok` for every
case. The Auto test (configparser.at) performs the same operations and checks
them against the same expected values; if AutoVM's PyFFI reproduces the result
it emits `ok` too (consistent), otherwise `not ok` (an AutoVM bug).

Boolean-False limitation
------------------------
A Python `False` returned from a method call on a handle (e.g.
`has_option`/`has_section` on a missing key) does not marshal reliably to
int 0 in the AutoVM PyFFI — `.to(str)` yields an empty/unstable marker and
equality-with-0 is unreliable. The True case (`getboolean("debug")` on a true
value) marshals cleanly to 1. So the suite only asserts the True branch of
booleans; section existence is checked via `sections().__len__()` rather than
`has_section`.

get-with-default limitation
---------------------------
`get(section, key, fallback)` passes 4 positional args and hits the AutoVM
PyFFI extra-arg path ("takes 3 positional arguments but 4 were given"), so the
suite uses plain 2-key `get` / `getint` / `getboolean` only.

INI literal note
----------------
Auto treats `'...'` as a character literal, not a string, so INI text is
written with double quotes and escaped newlines (`"[db]\\nhost = ...\\n"`) in
BOTH this oracle and the Auto test, keeping the literal byte-for-byte identical
across backends.
"""
import configparser


def tap_ok(n, name):
    print("ok {} - {}".format(n, name))


def tap_not_ok(n, name, diag):
    print("not ok {} - {} # {}".format(n, name, diag))


if __name__ == "__main__":
    INI = "[db]\nhost = localhost\nport = 5432\n[app]\nname = myapp\ndebug = true\n"

    cfg = configparser.ConfigParser()
    cfg.read_string(INI)

    # 1. get(section, key) returns the string value.
    host = cfg.get("db", "host")
    if host == "localhost":
        tap_ok(1, "test_get_str")
    else:
        tap_not_ok(1, "test_get_str", "got {}".format(host))

    # 2. getint returns an int value, comparable to a literal.
    port = cfg.getint("db", "port")
    if port == 5432:
        tap_ok(2, "test_getint")
    else:
        tap_not_ok(2, "test_getint", "got {}".format(port))

    # 3. getboolean on a "true" value marshals to int 1 (True case is reliable).
    dbg = cfg.getboolean("app", "debug")
    if dbg == 1:
        tap_ok(3, "test_getboolean_true")
    else:
        tap_not_ok(3, "test_getboolean_true", "got {}".format(dbg))

    # 4. sections() returns a list; __len__ gives the section count.
    secs = cfg.sections()
    nsec = secs.__len__()
    if nsec == 2:
        tap_ok(4, "test_sections_len")
    else:
        tap_not_ok(4, "test_sections_len", "got {}".format(nsec))

    # 5. get on a different section/key returns its string value.
    aname = cfg.get("app", "name")
    if aname == "myapp":
        tap_ok(5, "test_get_second_section")
    else:
        tap_not_ok(5, "test_get_second_section", "got {}".format(aname))
