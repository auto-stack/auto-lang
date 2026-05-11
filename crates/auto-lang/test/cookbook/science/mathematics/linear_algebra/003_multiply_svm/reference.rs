use ndarray::arr1;

fn main() {
    let scalar = 2;
    let v = arr1(&[1, 2, 3]);
    let result = scalar * v;
    println!("Scaled vector: {}, {}, {}", result[0], result[1], result[2]);
}
