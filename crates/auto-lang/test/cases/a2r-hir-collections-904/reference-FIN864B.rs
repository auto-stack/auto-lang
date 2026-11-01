// reference-FIN864B.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
// Item { name Str; code int } modeled as (String, i32)
fn main() {
    let mut xs: Vec<(String, i32)> = Vec::new();
    xs.push(("ab".to_string(), 1));
    xs.push(("cde".to_string(), 2));
    let mut total = 0;
    for it in &xs {
        total += it.0.chars().count() as i32 * 10 + it.1;
    }
    println!("value:{}", total);
}
