trait Flyer {
    fn fly(&self);
}

struct Pigeon {}

impl Flyer for Pigeon {
    fn fly(&self) {
        println!("Flap Flap");
    }
}

fn main() {
    let p = Pigeon {};
    p.fly();
}
