// Reference: Rust Cookbook - Calculate triangle side length
// Source: science/mathematics/trigonometry/side-length.md
fn main() {
    let angle: f64 = 1.0;
    let side_length = 80.0;

    let hypotenuse = side_length / angle.sin();

    println!("Hypotenuse: {}", hypotenuse);
}
