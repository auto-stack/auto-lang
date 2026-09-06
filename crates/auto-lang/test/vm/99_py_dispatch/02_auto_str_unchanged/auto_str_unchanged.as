// Plan 569 D2: Auto 臂零变化红线——纯 Auto 值（str/List）的 .len() 行为
// 逐字节不变：str.len 字符数语义 + Array.len ARRAY_LEN 直发路径照常，
// 侧表不置位（py_typed_vars 空），无任何组合子发射。
fn main() {
    var s = "abcdef"
    print(s.len())
    var xs = [1, 2, 3, 4]
    print(xs.len())
    var t = "x" + s
    print(t.len())
}
