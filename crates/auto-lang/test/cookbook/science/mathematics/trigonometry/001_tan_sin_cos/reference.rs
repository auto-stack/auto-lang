// Reference: Rust Cookbook - Verify tan = sin/cos
// Source: science/mathematics/trigonometry/tan-sin-cos.md
fn main() {
    let x: f64 = 6.0;

    let a = x.tan();
    let b = x.sin() / x.cos();

    assert_eq!(a, b);
}
