use ndarray::arr2;
use ndarray_linalg::Inverse;

fn main() {
    let a = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
    let inv = a.inv().unwrap();
    println!("Inverse: {} {}", inv[[0, 0]], inv[[0, 1]]);
    println!("         {} {}", inv[[1, 0]], inv[[1, 1]]);
}
