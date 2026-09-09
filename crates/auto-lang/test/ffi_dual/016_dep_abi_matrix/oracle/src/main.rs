//! PLAN-592 T3: 016 oracle 腿——与 input.at 逐行镜像,输出格式必须一致。

use autolang_abi_matrix::{free_ll, free_noop, free_s, Num, Pt, Words};

fn main() {
    let n = Num::new();
    println!("{}", n.echo_i8(-128));
    println!("{}", n.echo_i8(127));
    println!("{}", n.echo_i16(32767));
    println!("{}", n.echo_i32(-2_000_000_000));
    println!("{}", n.echo_i64(9_223_372_036_854_775_807));
    println!("{}", n.echo_u8(200));
    println!("{}", n.echo_u16(60000));
    println!("{}", n.echo_u32(2_000_000_000));
    println!("{}", n.echo_u64(9_223_372_036_854_775_807));
    println!("{}", n.echo_usize(123456789));
    println!("{}", n.echo_f32(0.5));
    println!("{}", n.echo_f64(100.25));
    println!("{}", n.echo_bool(true));
    println!("{}", n.echo_bool(false));
    println!("{}", n.add2(20, 22));
    println!("{}", n.i64_min());
    println!("{}", n.i64_max());

    let w = Words::new();
    println!("{}", w.greet("auto"));
    println!("{}", w.shout("rust".to_string()));
    println!("{}", w.unicode());
    println!("{}", w.empty());

    println!("{}", free_noop());
    println!("{}", free_s(w.unicode()));
    println!("{}", free_ll(4, 2));

    let p = Pt::new(9, "mid".to_string(), -7);
    println!("{}", p.a);
    println!("{}", p.b);
    println!("{}", p.c);
}
