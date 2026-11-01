// reference-LSTR864.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let mut xs: Vec<String> = Vec::new();
    xs.push("hello".to_string());
    xs.push("中".to_string());
    let removed = xs.remove(0);
    // Str.len counts Unicode scalars: 5*10 + 1
    println!("value:{}", removed.chars().count() as i32 * 10 + xs[0].chars().count() as i32);
}
