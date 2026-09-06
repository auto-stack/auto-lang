use.py builtins: list

fn main() {
    var l = list("cab")
    var j = py_call_may(l, "index", "a")
    var k = j.?
    print("B: " + k.to(str))
    var d = j.?(-1)
    print("D: " + d.to(str))
}
