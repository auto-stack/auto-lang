// py_re Auto test — TAP output format (Plan 369 P6 Task 24).
//
// Targets the Python `re` standard-library module via the PyFFI. Covers the four
// primary regex operations and the types they return across the boundary:
//
//   re.sub(pat, repl, str)        -> str           (clean scalar)
//   re.sub(pat, repl, str, count) -> str           (4-arg call)
//   re.findall(pat, str)          -> list[str]     (handle; __len__/__getitem__)
//   re.search(pat, str)           -> Match object  (handle; group/start/end)
//   re.split(pat, str)            -> list[str]     (handle; __getitem__)
//   re.escape(str)                -> str           (pure)
//
// Regex literal note: Auto has no raw-string (r"...") syntax, so patterns are
// written with escaped backslashes ("\\d+"). The same escaped form is used in
// the Python oracle so the literals are byte-for-byte identical.
//
// A Match object / list returned through PyFFI is held as an opaque
// PyObjectHandle; Auto reads it via py_call(h, "method", ...args). A Match
// handle cannot be printed directly (to(str) yields a raw handle marker), so
// every assertion goes through group()/start()/end() or list element access.
// Python None returned from a no-match search marshals to the integer 0, which
// compares cleanly against the literal 0.
//
// Test names MUST match tests/python/test_re.py because the comparator joins
// backends by name.
use.py re: sub, findall, search, split, escape

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // 1. sub replaces all matches by default.
    var r = sub("\\d+", "N", "a1b22c333")
    if r == "aNbNcN" {
        tap_ok(1, "test_sub_all")
    } else {
        tap_not_ok(1, "test_sub_all", "got " + r.to(str))
    }

    // 2. sub with a count limits replacements (4-arg call).
    var rc = sub("\\d+", "N", "a1b22c333", 1)
    if rc == "aNb22c333" {
        tap_ok(2, "test_sub_count")
    } else {
        tap_not_ok(2, "test_sub_count", "got " + rc.to(str))
    }

    // 3. findall returns a list handle; read length and elements.
    var fa = findall("\\d+", "a1b22c333")
    var fa_n = py_call(fa, "__len__")
    var fa_0 = py_call(fa, "__getitem__", 0)
    var fa_2 = py_call(fa, "__getitem__", 2)
    if fa_n == 3 {
        if fa_0 == "1" {
            if fa_2 == "333" {
                tap_ok(3, "test_findall")
            } else {
                tap_not_ok(3, "test_findall", "[2] got " + fa_2.to(str))
            }
        } else {
            tap_not_ok(3, "test_findall", "[0] got " + fa_0.to(str))
        }
    } else {
        tap_not_ok(3, "test_findall", "len got " + fa_n.to(str))
    }

    // 4. search returns a Match handle; group(n) reads capture groups.
    var m = search("(\\w+)@(\\w+)", "user@example.com")
    var g0 = py_call(m, "group", 0)
    var g1 = py_call(m, "group", 1)
    var g2 = py_call(m, "group", 2)
    if g0 == "user@example" {
        if g1 == "user" {
            if g2 == "example" {
                tap_ok(4, "test_search_groups")
            } else {
                tap_not_ok(4, "test_search_groups", "g2 got " + g2.to(str))
            }
        } else {
            tap_not_ok(4, "test_search_groups", "g1 got " + g1.to(str))
        }
    } else {
        tap_not_ok(4, "test_search_groups", "g0 got " + g0.to(str))
    }

    // 5. Match.start()/end() give the span offsets (ints).
    var m2 = search("\\d+", "abc123def")
    var ms = py_call(m2, "start")
    var me = py_call(m2, "end")
    if ms == 3 {
        if me == 6 {
            tap_ok(5, "test_match_span")
        } else {
            tap_not_ok(5, "test_match_span", "end got " + me.to(str))
        }
    } else {
        tap_not_ok(5, "test_match_span", "start got " + ms.to(str))
    }

    // 6. A pattern that does not match yields an EMPTY findall list. This is a
    //    clean, portable "no match" signal: the empty list is a real Python
    //    list on every backend, so __len__() == 0 agrees across the oracle,
    //    a2py, and AutoVM. (re.search on no match returns Python None; the
    //    AutoVM marshals None to int 0, but pure Python's `None == 0` is False,
    //    so a search-based no-match assertion would diverge between AutoVM and
    //    a2py. findall-empty avoids that — see the README "Known limitations".)
    var fae = findall("zzz", "abc")
    var fae_n = py_call(fae, "__len__")
    if fae_n == 0 {
        tap_ok(6, "test_findall_no_match")
    } else {
        tap_not_ok(6, "test_findall_no_match", "got " + fae_n.to(str))
    }

    // 7. split returns a list handle of the pieces between matches.
    var sp = split("\\s+", "a  b   c")
    var sp_n = py_call(sp, "__len__")
    var sp_0 = py_call(sp, "__getitem__", 0)
    var sp_2 = py_call(sp, "__getitem__", 2)
    if sp_n == 3 {
        if sp_0 == "a" {
            if sp_2 == "c" {
                tap_ok(7, "test_split")
            } else {
                tap_not_ok(7, "test_split", "[2] got " + sp_2.to(str))
            }
        } else {
            tap_not_ok(7, "test_split", "[0] got " + sp_0.to(str))
        }
    } else {
        tap_not_ok(7, "test_split", "len got " + sp_n.to(str))
    }

    // 8. escape escapes regex metacharacters; pure str -> str.
    var esc = escape("a.b*c")
    if esc == "a\\.b\\*c" {
        tap_ok(8, "test_escape")
    } else {
        tap_not_ok(8, "test_escape", "got " + esc.to(str))
    }
}
