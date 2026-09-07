// P-560-06: Err 通道六条形态现状（try-catch 已有；隐式传播/拦值载荷未知）
use.py torch: arange
fn risky(k int) -> int {
    return py_float(py_getitem(arange(3), k))
}
fn main() {
    try {
        print(risky(99))
    } catch e {
        print(e)
    }
}
