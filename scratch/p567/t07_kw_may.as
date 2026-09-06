use.py builtins: list

fn main() {
    var l = list("ba")
    var r = py_call_kw_may(l, "sort", [], ["reverse"], [false])
    var ok = r.?("ERR")
    print("ok-form: " + ok.to(str))
    var bad = py_call_kw_may(l, "no_such", [], ["reverse"], [false])
    var fb = bad.?("ERR")
    print("err-form: " + fb.to(str))
}
