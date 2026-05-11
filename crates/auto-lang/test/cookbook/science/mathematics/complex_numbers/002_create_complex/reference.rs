use num::Complex;

fn main() {
    let z = Complex::new(3.0, 4.0);
    let re = z.re;
    let im = z.im;
    let norm = z.norm();
    println!("z = {} + {}i, |z| = {}", re, im, norm);
}
