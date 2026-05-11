use ndarray::arr2;
use serde_json;

fn main() {
    let json = "[[1,2,3],[4,5,6]]";
    let matrix: Vec<Vec<i32>> = serde_json::from_str(json).unwrap();
    println!("Matrix: {:?}", matrix);
}
