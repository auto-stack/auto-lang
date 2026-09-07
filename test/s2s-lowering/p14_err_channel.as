// P-560-14 (T11): Err 通道六条观测——可捕/载荷/退出码/None 永不传播
use.py torch: arange
fn risky(k int) -> int {
    return py_float(py_getitem(arange(3), k))
}
fn main() {
    // 条款 2：None 是值（null 判等通道——550 面）
    var n = null
    print(n == null)
    // 条款 3：catch 拦值绑定（载荷含 Python 异常类型字样）
    try {
        print(risky(99))
    } catch e {
        print("caught")
        print(e)
    }
    // 条款 4 隐式传播作用域（显式 May 通道表达）
    var bad = py_call_may(arange(3), "no_such")
    var msg = bad.?("fallback")
    print(msg)
}
