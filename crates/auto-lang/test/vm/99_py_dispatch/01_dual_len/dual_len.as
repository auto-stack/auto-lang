// Plan 569 D2: py 返回值方法分派动态化——双通道两臂各一例。
//
// `var t = arange(6)` 的 py 桥返回值（PyObjectHandle，历史上被谎记为
// String 后 str.len 静态路由读到垃圾数字）经 py-类型侧表改发 obj_len
// 组合子：运行期 tag 双通道——句柄 → GIL len（=6）；Auto str 照常
// 原生 str.len 字符数（=3）。.as 与 .at 双模式同值（s2s B6/A1 与
// codegen 侧表两条路殊途同归）。
use.py numpy: arange

fn main() {
    var t = arange(6)
    print(t.len())
    var s = "abc"
    print(s.len())
}
