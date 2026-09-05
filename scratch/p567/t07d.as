use.py builtins: list

fn main() {
    var l = list("cab")
    var j = py_call_may(l, "index", "a")
    print("A: " + j?.(-1).to(str))
    var k = j.?
    print("B: " + k.to(str))
    var m = [1, 2]
    var mv = m[0]
    print("C: " + (mv.?(-1)).to(str))
}
