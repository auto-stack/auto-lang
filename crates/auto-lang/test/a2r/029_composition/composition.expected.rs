struct Wing {}

impl Wing {
    fn fly(&self) {
        println!("flying");
    }
}

trait Wing {
    fn fly(&self);
}

struct Duck {}

impl Wing for Duck {
    fn fly(&self) {
        // TODO: Implement fly method body from Wing
    }
}

impl Duck {
    fn fly(&self) {
        println!("flying");
    }
}

fn main() {
    let d: Duck = Duck {};
    d.fly();
}
