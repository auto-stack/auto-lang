// reference-L04.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
// Item { name Str; code int } modeled as (String, i32)
fn main() {
    let name: String = "hello".to_string();
    let code = 37;
    let mut xs: Vec<(String, i32)> = Vec::new();
    xs.push((name, code));
    let r = xs[0].0.chars().count() as i32 + xs[0].1;
    println!("value:{}", r);
}
