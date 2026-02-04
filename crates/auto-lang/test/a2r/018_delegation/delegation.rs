trait Engine {
    fn start(&self);
}


struct WarpDrive {}

impl WarpDrive {
    fn start(&self) {
        println!("WarpDrive engaging");
    }
}

struct Starship {
    core: WarpDrive,
}

impl Engine for Starship {
    fn start(&self) {
        self.core.start()
    }
}

fn main() {
    let ship: Starship = Starship {};
    ship.start();
}
