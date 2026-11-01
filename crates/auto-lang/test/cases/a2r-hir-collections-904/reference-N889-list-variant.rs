// reference-N889-list-variant.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
// enum Token { Data(Item), End }; Item { name Str; code int }
fn main() {
    let mut xs: Vec<Result<(String, i32), ()>> = Vec::new();
    xs.push(Ok(("中".to_string(), 41)));
    let t = xs.remove(0);
    match t {
        Ok((n, c)) => println!("value:{}", n.chars().count() as i32 + c),
        Err(_) => println!("value:0"),
    }
}
