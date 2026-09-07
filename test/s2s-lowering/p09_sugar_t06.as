// P-560-09 (T06): B5/B6/A5/D7 端到端
use.py torch: arange
fn main() {
    var x = arange(10)
    print(len(x))
    var a = x[2..5]
    print(obj_len(a))
    var d = x[1..9..2]
    print(obj_len(d))
    print(x)
}
