fn main() {
    let x: i32 = 42;
    let ptr: *const i32 = &x;
    let val = unsafe { *ptr };
    println!("{}", val);
}
