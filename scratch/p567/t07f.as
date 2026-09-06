use.py builtins: list

fn main() {
    var l = list("cab")
    var j = py_call_may(l, "index", "a")
    print("E: " + (j.?(-1)).to(str))
    print("F: " + j.?(-1).to(str))
}
