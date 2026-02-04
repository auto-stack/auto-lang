trait Flyer {
    fn fly(&self);
}


struct Pigeon {}

impl Pigeon {
    fn fly(&self) {
        println!("Flap");
    }
}

impl Flyer for Pigeon {
    fn fly(&self) {
{
            println!("Flap");
        }    }
}

fn main() {
    let p: Pigeon = Pigeon {};
    p.fly();
}
