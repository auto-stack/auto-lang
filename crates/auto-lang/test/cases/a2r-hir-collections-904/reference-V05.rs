// reference-V05.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
// enum Token { Data(Item), End } — Data arm; Item { name Str; code int }
fn main() {
    let name = "hello".to_string();
    let code = 37i32;
    match (name, code) {
        (n, c) => println!("value:{}", n.chars().count() as i32 + c),
    }
}
