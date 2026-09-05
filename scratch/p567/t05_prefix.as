use.py builtins: int, list

fn main() {
    try {
        int("abc")
        print("no-error")
    } catch e {
        print("caught: " + e.to(str))
    }
    var l = list("ab")
    try {
        py_getitem(l, 5)
        print("no-error2")
    } catch e2 {
        print("caught2: " + e2.to(str))
    }
}
