use rayon::prelude::*;

fn main() {
    let v = [1, 2, 3, 4, 5];
    let has_even = v.par_iter().any(|&x| x % 2 == 0);
    let all_positive = v.par_iter().all(|&x| x > 0);
    println!("Has even: {}, All positive: {}", has_even, all_positive);
}
