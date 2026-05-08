use ndarray::arr1;

fn main() {
    let a = arr1(&[1.0, 2.0, 3.0]);
    let b = arr1(&[1.0, 2.0, 3.0]);
    let equal = a == b;
    println!("Vectors equal: {}", equal);
}
