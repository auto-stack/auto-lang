// py_configparser Auto test — TAP output format (Plan 369 P6 Task 26).
//
// Targets the Python `configparser` standard-library module via the PyFFI.
// Covers INI parsing on FLAT config (sections whose values are scalars):
//
//   ConfigParser()                          -> parser handle (class construct)
//   py_call(cfg, "read_string", ini_text)   -> parse INI text into the parser
//   py_call(cfg, "get", section, key)       -> str value
//   py_call(cfg, "getint", section, key)    -> int value (comparable)
//   py_call(cfg, "getboolean", section, key)-> bool -> int (True case reliable)
//   py_call(cfg, "sections") + __len__      -> section count
//
// A ConfigParser instance is held as an opaque PyObjectHandle; Auto reads it
// via py_call(h, "method", ...args). Returned scalars (str/int) come back
// clean; returned containers (the sections list) are read via __len__.
//
// Boolean-False limitation: a Python False returned from a method call
// (e.g. has_option on a missing key) does NOT marshal reliably to int 0 in
// Auto — `.to(str)` yields an empty/unstable marker and equality-with-0 is
// unreliable. The True case (getboolean("debug") on a true value) marshals
// cleanly to 1. So the suite only asserts the True branch of booleans and
// never the False branch; section existence is checked via sections().__len__
// instead of has_section.
//
// get-with-default limitation: get(section, key, fallback) passes 4 args and
// hits the "takes 3 positional arguments but 4 were given" extra-arg path, so
// the suite uses plain 2-key get / getint / getboolean only.
//
// INI literal note: Auto treats '...' as a CHARACTER literal, not a string,
// so INI text is written with double quotes and escaped newlines
// ("[db]\nhost = localhost\n"). The same escaped form is used in the Python
// oracle so the literal is byte-for-byte identical across backends.
//
// Test names MUST match tests/python/test_configparser.py because the
// comparator joins backends by name.
use.py configparser: ConfigParser

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    var INI = "[db]\nhost = localhost\nport = 5432\n[app]\nname = myapp\ndebug = true\n"

    var cfg = ConfigParser()
    py_call(cfg, "read_string", INI)

    // 1. get(section, key) returns the string value.
    var host = py_call(cfg, "get", "db", "host")
    if host == "localhost" {
        tap_ok(1, "test_get_str")
    } else {
        tap_not_ok(1, "test_get_str", "got " + host.to(str))
    }

    // 2. getint returns an int value, comparable to a literal.
    var port = py_call(cfg, "getint", "db", "port")
    if port == 5432 {
        tap_ok(2, "test_getint")
    } else {
        tap_not_ok(2, "test_getint", "got " + port.to(str))
    }

    // 3. getboolean on a "true" value marshals to int 1 (True case is reliable).
    var dbg = py_call(cfg, "getboolean", "app", "debug")
    if dbg == 1 {
        tap_ok(3, "test_getboolean_true")
    } else {
        tap_not_ok(3, "test_getboolean_true", "got " + dbg.to(str))
    }

    // 4. sections() returns a list; __len__ gives the section count.
    var secs = py_call(cfg, "sections")
    var nsec = py_call(secs, "__len__")
    if nsec == 2 {
        tap_ok(4, "test_sections_len")
    } else {
        tap_not_ok(4, "test_sections_len", "got " + nsec.to(str))
    }

    // 5. get on a different section/key returns its string value.
    var aname = py_call(cfg, "get", "app", "name")
    if aname == "myapp" {
        tap_ok(5, "test_get_second_section")
    } else {
        tap_not_ok(5, "test_get_second_section", "got " + aname.to(str))
    }
}
