use rayon::prelude::*;

fn main() {
    let v: Vec<i32> = (1..=10).collect();
    let target = 7;
    let found = v.par_iter().find_any(|&&x| x == target);
    match found {
        Some(val) => println!("Found: {}", val),
        None => println!("Not found"),
    }
}
