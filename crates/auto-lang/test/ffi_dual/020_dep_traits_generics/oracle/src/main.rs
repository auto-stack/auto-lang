//! PLAN-596 T-03/T-08: 020 oracle 腿——与 input.at 逐行镜像。

use autolang_traits::{pick, Pair, Tagged, Temp};

fn main() {
    let t: Temp = Temp::of("abc".to_string());
    let s = t.to_string();
    println!("{s}");
    println!("{}", t.degree);

    let src = Tagged::new("orig".to_string(), 1);
    let mut cp = src.clone();
    cp.hits = 9;
    println!("{}", src.hits);
    println!("{}", cp.hits);
    println!("{}", src.tag);

    let a = pick(3, 9);
    println!("{a}");
    let b = pick("a".to_string(), "z".to_string());
    println!("{b}");

    let p = Pair::new(2, 7);
    println!("{}", p.pick_max(20, 70));
}
