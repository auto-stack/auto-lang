use.py builtins: sorted

fn main() {
    var a = "a"
    var b = "b"
    var c = "c"
    var d = "d"
    var j1 = a + b + c + d
    print("j1=" + j1)

    var j2 = a.to(str) + b.to(str) + c.to(str) + d.to(str)
    print("j2=" + j2)

    var lst = sorted("dcba")
    var e0 = py_call(lst, "__getitem__", 0)
    var e1 = py_call(lst, "__getitem__", 1)
    var e2 = py_call(lst, "__getitem__", 2)
    var e3 = py_call(lst, "__getitem__", 3)
    print("e0=" + e0.to(str))
    print("e1=" + e1.to(str))
    var j3 = e0.to(str) + e1.to(str)
    print("j3=" + j3)
    var j4 = e0.to(str) + e1.to(str) + e2.to(str)
    print("j4=" + j4)
    var j5 = e0.to(str) + e1.to(str) + e2.to(str) + e3.to(str)
    print("j5=" + j5)
}
