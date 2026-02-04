trait Engine {
    fn start(&self);
}


trait Weapon {
    fn fire(&self);
}


struct WarpDrive {}

impl WarpDrive {
    fn start(&self) {
        println!("WarpDrive engaging");
    }
}

struct LaserCannon {}

impl LaserCannon {
    fn fire(&self) {
        println!("Pew! Pew!");
    }
}

struct Starship {
    core: WarpDrive,
    weapon: LaserCannon,
}

impl Engine for Starship {
    fn start(&self) {
        self.core.start()
    }
}

impl Weapon for Starship {
    fn fire(&self) {
        self.weapon.fire()
    }
}

fn main() {
    let ship: Starship = Starship {};
    ship.start();
    ship.fire();
}
