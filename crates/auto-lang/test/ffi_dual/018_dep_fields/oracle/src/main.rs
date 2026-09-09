//! PLAN-591 T3: 018 oracle 腿——与 input.at 逐行镜像,输出格式必须一致。
//! V1-4 Option:VM 侧 null 判定 == a2r `== None` == oracle `is_none()`。
//! V1-5 Result:三轨同走 .unwrap()(Err 面在 VM 腿测试体内 err-assert,不进 golden)。

use autolang_shapes::{Messy, Outer, Point, Shape};

fn main() {
    let m = Messy::new(42, "alpha", true);
    let hit = m.lookup("alpha");
    println!("{}", hit.is_none());
    let miss = m.lookup("nope");
    println!("{}", miss.is_none());

    let p = Point::parse("3,4").unwrap();
    println!("{}", p.x);
    println!("{}", p.y);

    let o = Outer::new(5, "inner-label");
    println!("{}", o.inner.n);
    println!("{}", o.inner.label);

    let s1 = Shape::new(0, 9);
    println!("{}", s1.is_circle());
    let s2 = Shape::new(1, 9);
    println!("{}", s2.is_circle());
    println!("{}", s2.is_square());
    let s3 = Shape::new(2, 9);
    println!("{}", s3.is_triangle());
}
