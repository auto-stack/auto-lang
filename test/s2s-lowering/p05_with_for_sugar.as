// P-560-05: E 族控制流糖现状（with / for-in handle）
use.py torch: arange
fn main() {
    var t = arange(3)
    for x in t {
        print(py_float(x))
    }
}
