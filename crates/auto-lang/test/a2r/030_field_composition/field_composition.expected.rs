struct Engine {
    hp: i32,
}

trait Engine {
}

struct Car {
    hp: i32,
}

impl Engine for Car {
}
