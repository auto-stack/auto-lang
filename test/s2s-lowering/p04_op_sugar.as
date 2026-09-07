// P-560-04: C 族运算符糖现状（@ ** is 句柄真值）
use.py torch: arange, tensor
fn main() {
    var a = tensor([[1, 2], [3, 4]])
    var b = tensor([[5, 6], [7, 8]])
    print(py_float(py_call(py_matmul(a, b), "sum")))
}
