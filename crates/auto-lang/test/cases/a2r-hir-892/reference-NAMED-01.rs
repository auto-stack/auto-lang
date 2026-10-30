// hand-written independent reference: named args bind b-slot=mark_b()+1=3
// then a-slot=mark_a()*2=2 (SOURCE evaluation order), result a*10+b=23,
// trace marks in execution order (stderr): mark_b then mark_a.
fn main() {
    eprintln!("mark_b");
    eprintln!("mark_a");
    println!("value:{}", (1 * 2) * 10 + (2 + 1));
}
