use rayon::prelude::*;

fn main() {
    let mut v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    v.par_iter_mut().for_each(|x| *x *= 2);
    println!("Doubled: {:?}", v);
}
