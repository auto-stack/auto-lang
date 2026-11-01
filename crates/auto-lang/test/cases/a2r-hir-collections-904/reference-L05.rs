// reference-L05.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
// Item { name Str; code int } modeled as (String, i32)
fn main() {
    let mut xs: Vec<(String, i32)> = Vec::new();
    xs.push(("hello".to_string(), 37));
    let x = xs.remove(0);
    println!("value:{}", x.0.chars().count() as i32 + x.1 + xs.len() as i32);
}
