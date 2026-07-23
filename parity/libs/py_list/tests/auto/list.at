// py_list Auto test — TAP output format (Plan 369 P4 Task 17).
//
// Plan 369 P4 Task 16 found that Auto list literals (e.g. `[1, 2, 3]`) do NOT
// marshal to a Python `list`, so passing one to a Python function failed. This
// suite approaches list parity from the other direction: Python functions that
// RETURN lists (`sorted`, `list`, `reversed`), and whether Auto can manipulate
// those returned list objects via the py_call / py_getattr built-ins.
//
// A Python list returned through PyFFI is held as an opaque PyObjectHandle in
// the AutoVM heap (the live object is kept, not stringified). Auto interacts
// with it via method/dunder calls:
//   py_call(lst, "__len__")           -> len(lst)
//   py_call(lst, "__getitem__", i)    -> lst[i]
//   py_call(lst, "__contains__", x)   -> x in lst   (bool -> int)
//   py_call(lst, "__add__", other)    -> lst + other
//   py_call(lst, "__mul__", n)        -> lst * n
//   py_call(lst, "count", x)          -> lst.count(x)
//   py_call(lst, "index", x)          -> lst.index(x)
//   py_call(lst, "__reversed__")      -> reversed(lst) (iterator; feed to list())
//
// The list itself cannot be printed directly (Auto's to(str) on a list handle
// yields a raw handle marker, e.g. "4000000"), so every assertion goes through
// element access, length, or a method that returns a primitive.
//
// The parity runner sets the working directory to the library root
// (parity/libs/py_list), so `use.py builtins: sorted, list` resolves the
// Python builtins. Test names MUST match tests/python/test_list.py because the
// comparator joins backends by name.
use.py builtins: sorted, list

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    // 1. sorted() returns a list; __len__ gives its size.
    var lst = sorted("dcba")
    var n1 = py_call(lst, "__len__")
    if n1 == 4 {
        tap_ok(1, "test_sorted_returns_list_len")
    } else {
        tap_not_ok(1, "test_sorted_returns_list_len", "got " + n1.to(str))
    }

    // 2. __getitem__ reads sorted elements in order. Concatenate the four
    //    single-char strings and compare to "abcd".
    var e0 = py_call(lst, "__getitem__", 0)
    var e1 = py_call(lst, "__getitem__", 1)
    var e2 = py_call(lst, "__getitem__", 2)
    var e3 = py_call(lst, "__getitem__", 3)
    var joined = e0.to(str) + e1.to(str) + e2.to(str) + e3.to(str)
    if joined == "abcd" {
        tap_ok(2, "test_sorted_getitem")
    } else {
        tap_not_ok(2, "test_sorted_getitem", "got " + joined)
    }

    // 3. list() builds a list from a string.
    var chars = list("hello")
    var cn = py_call(chars, "__len__")
    var c0 = py_call(chars, "__getitem__", 0)
    if cn == 5 {
        if c0 == "h" {
            tap_ok(3, "test_list_from_string")
        } else {
            tap_not_ok(3, "test_list_from_string", "first got " + c0.to(str))
        }
    } else {
        tap_not_ok(3, "test_list_from_string", "len got " + cn.to(str))
    }

    // 4. __contains__ (the `in` operator) returns a bool -> int (1).
    var has = py_call(chars, "__contains__", "l")
    if has == 1 {
        tap_ok(4, "test_list_contains")
    } else {
        tap_not_ok(4, "test_list_contains", "got " + has.to(str))
    }

    // 5. __add__ concatenates two lists.
    var ab = py_call(list("ab"), "__add__", list("cd"))
    var abn = py_call(ab, "__len__")
    var ab0 = py_call(ab, "__getitem__", 0)
    var ab3 = py_call(ab, "__getitem__", 3)
    if abn == 4 {
        if ab0 == "a" {
            if ab3 == "d" {
                tap_ok(5, "test_list_add")
            } else {
                tap_not_ok(5, "test_list_add", "[3] got " + ab3.to(str))
            }
        } else {
            tap_not_ok(5, "test_list_add", "[0] got " + ab0.to(str))
        }
    } else {
        tap_not_ok(5, "test_list_add", "len got " + abn.to(str))
    }

    // 6. __mul__ repeats a list.
    var aaa = py_call(list("ab"), "__mul__", 3)
    var aan = py_call(aaa, "__len__")
    var aa0 = py_call(aaa, "__getitem__", 0)
    var aa5 = py_call(aaa, "__getitem__", 5)
    if aan == 6 {
        if aa0 == "a" {
            if aa5 == "b" {
                tap_ok(6, "test_list_mul")
            } else {
                tap_not_ok(6, "test_list_mul", "[5] got " + aa5.to(str))
            }
        } else {
            tap_not_ok(6, "test_list_mul", "[0] got " + aa0.to(str))
        }
    } else {
        tap_not_ok(6, "test_list_mul", "len got " + aan.to(str))
    }

    // 7. count() and index() methods on a list.
    var ms = list("mississippi")
    var scount = py_call(ms, "count", "s")
    var pindex = py_call(ms, "index", "p")
    if scount == 4 {
        if pindex == 8 {
            tap_ok(7, "test_list_count_index")
        } else {
            tap_not_ok(7, "test_list_count_index", "index got " + pindex.to(str))
        }
    } else {
        tap_not_ok(7, "test_list_count_index", "count got " + scount.to(str))
    }

    // 8. __reversed__ returns an iterator; list() materialises it back to a list.
    var reviter = py_call(list("abcd"), "__reversed__")
    var rev = list(reviter)
    var r0 = py_call(rev, "__getitem__", 0)
    var r3 = py_call(rev, "__getitem__", 3)
    if r0 == "d" {
        if r3 == "a" {
            tap_ok(8, "test_reversed_via_list")
        } else {
            tap_not_ok(8, "test_reversed_via_list", "[3] got " + r3.to(str))
        }
    } else {
        tap_not_ok(8, "test_reversed_via_list", "[0] got " + r0.to(str))
    }
}
