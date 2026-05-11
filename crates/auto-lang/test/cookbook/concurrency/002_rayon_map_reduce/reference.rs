use rayon::prelude::*;

fn main() {
    let v = [1, 2, 3, 4, 5];
    let sum: i32 = v.par_iter().map(|&x| x * 2).sum();
    println!("Sum of doubled: {}", sum);
}
