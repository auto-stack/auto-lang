//! PLAN-592 T5: 017 oracle 腿——与 input.at 逐行镜像,输出格式必须一致。
//! ChainInPlace 语义 = Rust 侧原地修改(&mut 链不产生新对象)。

use autolang_counter::{Config, Counter, drop_count};

fn main() {
    println!("{}", drop_count());

    let mut cfg = Config::new();
    cfg.verbose(true);
    println!("{}", cfg.is_verbose());
    cfg.level(11);
    println!("{}", cfg.level_value());

    let mut c = Counter::new("orig".to_string());
    let mut cl = c.clone_reset();
    c.increment();
    c.increment();
    println!("{}", c.value());
    println!("{}", cl.value());
    cl.set_label("clone".to_string());
    println!("{}", c.label());
    println!("{}", cl.label());
}
