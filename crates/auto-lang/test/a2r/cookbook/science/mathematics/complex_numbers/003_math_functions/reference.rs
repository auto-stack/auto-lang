use num::Complex;

fn main() {
    let z = Complex::new(3.0, 4.0);
    let r = z.norm();
    let theta = z.arg();
    println!("Complex: {}", z);
    println!("Magnitude: {}", r);
    println!("Phase: {}", theta);
}
