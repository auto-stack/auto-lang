trait Flyer {
    fn fly(&self);
}


struct Pigeon {}

impl Pigeon {
    fn fly(&self) {
        println!("Flap Flap");
    }
}

impl Flyer for Pigeon {
    fn fly(&self) {
{
            println!("Flap Flap");
        }    }
}

struct Hawk {}

impl Hawk {
    fn fly(&self) {
        println!("Gawk! Gawk!");
    }
}

impl Flyer for Hawk {
    fn fly(&self) {
{
            println!("Gawk! Gawk!");
        }    }
}

fn main() {

    let arr: &[dyn Flyer] = [b1, b2];
    for b in arr {
        b.fly();
    }
}
