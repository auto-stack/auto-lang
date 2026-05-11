use ndarray::arr1;

fn main() {
    let v = arr1(&[3.0, 4.0]);
    let norm = v.dot(&v).sqrt();
    println!("L2 norm: {}", norm);
}
