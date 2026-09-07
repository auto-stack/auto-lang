// P-560-11 (T08): C3/C4 句柄真值 + C7 is
use.py torch: arange, tensor
fn main() {
    var z = tensor([0.0])
    if z { print("truthy-0") } else { print("falsy-0") }
    var nz = tensor([2.0])
    if nz { print("truthy-2") } else { print("falsy-2") }
    var a = tensor([1.0])
    var b = a
    var c = tensor([1.0])
    print(a is b)
    print(a is c)
}
