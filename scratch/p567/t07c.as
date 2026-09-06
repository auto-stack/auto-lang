use.py builtins: list

fn main() {
    var l = list("cab")
    var j = py_call_may(l, "index", "a")
    print("may453: " + j.?(-1).to(str))
    var i = py_call_kw_may(l, "index", ["a"], [], [])
    print("kw478: " + i.?(-1).to(str))
    var raw = py_call_kw_may(l, "index", ["a"], [], [])
    print("raw: " + raw.to(str))
}
