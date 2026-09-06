use.py builtins: list

fn main() {
    var l = list("cab")
    var i = py_call_kw_may(l, "index", ["a"], [], [])
    print("index: " + i.?(-1).to(str))
    var n = py_call_kw_may(l, "sort", [], ["reverse"], [false])
    print("null-ret: " + n.?("ERR").to(str))
}
