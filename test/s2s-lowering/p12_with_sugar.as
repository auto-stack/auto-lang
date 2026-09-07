// P-560-12 (T09): with 上下文管理器糖（infer 套件 14 例形态）
use.py torch: arange
fn main() {
    var torch = py_module("torch")
    var ng = py_call0(py_getattr(torch, "no_grad"))
    var gflag = 1
    with ng {
        var xg = arange(3)
        py_call(xg, "requires_grad_", true)
        var yg = xg * 2
        gflag = py_getattr(yg, "requires_grad").to(int)
    }
    print(gflag)
    with py_call0(py_getattr(torch, "no_grad")) as ctx {
        print(obj_type_name(ctx))
    }
}
