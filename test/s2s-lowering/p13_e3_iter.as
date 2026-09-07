// P-560-13 (T10): E3 双通道——sized（array/GIL len）+ 迭代器（py_iter/py_next）
use.py torch: arange
fn main() {
    // 通道一：sized handle（array 通道=GIL len+getitem，539 已通）
    var t = arange(3)
    var s1 = 0
    for x in t { s1 = s1 + py_float(x) }
    print(s1)
    // 通道二：显式迭代器（py_iter 物化 + py_next 拉取——unsized 源语义）
    var it = py_iter(t)
    var s2 = 0
    var v = py_next(it)
    s2 = s2 + py_float(v)
    v = py_next(it)
    s2 = s2 + py_float(v)
    v = py_next(it)
    s2 = s2 + py_float(v)
    print(s2)
    v = py_next(it)
    print(v == null)
}
