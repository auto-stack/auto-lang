// P-560-07 (T04): py_contains 470 + py_module 471 端到端
use.py math: pi
fn main() {
    var torch = py_module("torch")
    print(obj_type_name(torch))
    var nn = py_getattr(torch, "nn")
    print(obj_type_name(nn))
    var d = py_call0(py_getattr(torch, "device"), "cpu")
    print(obj_type_name(d))
    if py_contains([1, 2, 3], 2) { print("contains-hit") }
    if !py_contains([1, 2, 3], 9) { print("contains-miss") }
}
