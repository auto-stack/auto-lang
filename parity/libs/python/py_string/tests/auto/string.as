// py_string Auto test — TAP output format (Plan 369 P4 Task 18).
//
// Auto strings live on the stack as string-tagged values, not as
// PyObjectHandles. But the py_call(obj, "method", ...args) built-in accepts a
// stack string as its first argument: PyFFI marshals it to a Python str (via
// pop_auto_py_arg) and calls the method on it. The result is then marshalled
// back:
//   - str  -> Auto string (compares equal to literals)
//   - bool -> Auto int (Python True/False -> 1/0)
//   - int  -> Auto int
//   - list -> opaque PyObjectHandle (cannot be stringified; probe via
//     __len__ / __getitem__, as exercised in py_list)
//
// py_call is only linked in when a `use.py` import is present, so this file
// imports a single builtin (sorted) purely to activate the Python FFI surface.
// The strings under test are plain Auto stack strings.
//
// The parity runner sets the working directory to the library root
// (parity/libs/py_string). Test names MUST match tests/python/test_string.py
// because the comparator joins backends by name.
use.py builtins: sorted

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // 1. upper / lower (str -> str).
    var up = py_call("hello", "upper")
    var lo = py_call("WORLD", "lower")
    if up == "HELLO" {
        if lo == "world" {
            tap_ok(1, "test_upper_lower")
        } else {
            tap_not_ok(1, "test_upper_lower", "lower got " + lo.to(str))
        }
    } else {
        tap_not_ok(1, "test_upper_lower", "upper got " + up.to(str))
    }

    // 2. title / capitalize (str -> str).
    var t = py_call("hello world", "title")
    var cap = py_call("hello world", "capitalize")
    if t == "Hello World" {
        if cap == "Hello world" {
            tap_ok(2, "test_title_capitalize")
        } else {
            tap_not_ok(2, "test_title_capitalize", "capitalize got " + cap.to(str))
        }
    } else {
        tap_not_ok(2, "test_title_capitalize", "title got " + t.to(str))
    }

    // 3. replace (multi-arg: old, new; and 3-arg with count).
    var rep = py_call("hello", "replace", "l", "L")
    var repn = py_call("aaaa", "replace", "a", "b", 2)
    if rep == "heLLo" {
        if repn == "bbaa" {
            tap_ok(3, "test_replace")
        } else {
            tap_not_ok(3, "test_replace", "replace-n got " + repn.to(str))
        }
    } else {
        tap_not_ok(3, "test_replace", "replace got " + rep.to(str))
    }

    // 4. strip / lstrip / rstrip.
    var st = py_call("  hi  ", "strip")
    var ls = py_call("  hi  ", "lstrip")
    var rs = py_call("  hi  ", "rstrip")
    if st == "hi" {
        if ls == "hi  " {
            if rs == "  hi" {
                tap_ok(4, "test_strip")
            } else {
                tap_not_ok(4, "test_strip", "rstrip got |" + rs.to(str) + "|")
            }
        } else {
            tap_not_ok(4, "test_strip", "lstrip got |" + ls.to(str) + "|")
        }
    } else {
        tap_not_ok(4, "test_strip", "strip got |" + st.to(str) + "|")
    }

    // 5. startswith / endswith (bool -> int 1).
    var sw = py_call("hello world", "startswith", "hello")
    var ew = py_call("hello world", "endswith", "world")
    if sw == 1 {
        if ew == 1 {
            tap_ok(5, "test_startswith_endswith")
        } else {
            tap_not_ok(5, "test_startswith_endswith", "endswith got " + ew.to(str))
        }
    } else {
        tap_not_ok(5, "test_startswith_endswith", "startswith got " + sw.to(str))
    }

    // 6. find (returns int, -1 when missing) and count (returns int).
    var f = py_call("hello world", "find", "world")
    var fnf = py_call("hello world", "find", "xyz")
    var c = py_call("hello", "count", "l")
    if f == 6 {
        if fnf == -1 {
            if c == 2 {
                tap_ok(6, "test_find_count")
            } else {
                tap_not_ok(6, "test_find_count", "count got " + c.to(str))
            }
        } else {
            tap_not_ok(6, "test_find_count", "find-missing got " + fnf.to(str))
        }
    } else {
        tap_not_ok(6, "test_find_count", "find got " + f.to(str))
    }

    // 7. split (returns a list) + join (str.join(list) -> str). The split
    //    result is an opaque PyObjectHandle, so probe it via __len__ /
    //    __getitem__ before joining.
    var parts = py_call("a,b,c", "split", ",")
    var plen = py_call(parts, "__len__")
    var p2 = py_call(parts, "__getitem__", 2)
    var joined = py_call("-", "join", parts)
    if plen == 3 {
        if p2 == "c" {
            if joined == "a-b-c" {
                tap_ok(7, "test_split_join")
            } else {
                tap_not_ok(7, "test_split_join", "join got " + joined.to(str))
            }
        } else {
            tap_not_ok(7, "test_split_join", "[2] got " + p2.to(str))
        }
    } else {
        tap_not_ok(7, "test_split_join", "len got " + plen.to(str))
    }

    // 8. __len__ (int) and __getitem__ (str -> 1-char str).
    var len = py_call("hello", "__len__")
    var ch = py_call("hello", "__getitem__", 1)
    if len == 5 {
        if ch == "e" {
            tap_ok(8, "test_len_getitem")
        } else {
            tap_not_ok(8, "test_len_getitem", "[1] got " + ch.to(str))
        }
    } else {
        tap_not_ok(8, "test_len_getitem", "len got " + len.to(str))
    }
}
