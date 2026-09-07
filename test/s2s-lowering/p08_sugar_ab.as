// P-560-08 (T05): A/B 族糖端到端——经 s2s 管线
use.py torch: arange
fn main() {
    var t = arange(6)
    print(obj_len(t))
    print(py_float(py_call(t, "sum")))
    var w = t.shape
    print(obj_len(w))
    print(py_float(t[0]))
}
