use ndarray::arr2;

fn main() {
    let a = arr2(&[[1, 2], [3, 4]]);
    let b = arr2(&[[5, 6], [7, 8]]);
    let c = a + b;
    println!("{} {}", c[[0, 0]], c[[0, 1]]);
    println!("{} {}", c[[1, 0]], c[[1, 1]]);
}
