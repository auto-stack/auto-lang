// Reference: Rust Cookbook - Sort a Vector of Integers
// Source: algorithms/sorting/sort.md
fn main() {
    let mut vec = vec![1, 5, 10, 2, 15];

    vec.sort();

    assert_eq!(vec, vec![1, 2, 5, 10, 15]);
}
