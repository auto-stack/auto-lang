use rayon::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

fn main() {
    let mut v: Vec<i32> = (0..100).collect();
    v.shuffle(&mut thread_rng());
    v.par_sort();
    println!("Sorted first: {}, last: {}", v[0], v[99]);
}
