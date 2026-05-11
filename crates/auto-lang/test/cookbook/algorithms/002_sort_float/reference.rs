// Reference: Rust Cookbook - Sort a Vector of Floats
// Source: algorithms/sorting/sort_float.md
fn main() {
    let mut vec = vec![1.1_f64, 1.15, 5.5, 1.123, 2.0];

    vec.sort_by(|a, b| a.total_cmp(b));

    assert_eq!(vec, vec![1.1, 1.123, 1.15, 2.0, 5.5]);
}
