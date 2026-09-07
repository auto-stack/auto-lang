// P-560-01: A 族调用糖现状（passthrough 下预期编译错/桥直呼形态）
use.py torch: arange
fn main() {
    var t = arange(6)
    print(t.sum())
    print(t.sum(dim: 0))
}
