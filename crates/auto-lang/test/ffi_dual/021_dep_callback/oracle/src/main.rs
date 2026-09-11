//! PLAN-596 复审 F-1: 021 oracle 腿——回调面,与 input.at 逐行镜像。

use autolang_traits::Invoker;

fn main() {
    let inv = Invoker::new();
    let r = inv.apply(Box::new(|x: i64| x * 2 + 1), 5);
    println!("{r}");
}
