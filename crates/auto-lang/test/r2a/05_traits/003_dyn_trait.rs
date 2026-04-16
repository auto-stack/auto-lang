fn main() {
    let v: Vec<Box<dyn Flyer>> = vec![];
    println!("empty");
}

trait Flyer {
    fn fly(&self);
}
