trait Calculator {
    fn add(&self, a: i32, b: i32) -> i32;
    fn multiply(&self, a: i32, b: i32) -> i32;
}


struct MathEngine {}

impl MathEngine {
    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
    fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
}

struct Computer {
    engine: MathEngine,
}

impl Calculator for Computer {
    fn add(&self, a: i32, b: i32) -> i32 {
        self.engine.add(a, b)
    }
    fn multiply(&self, a: i32, b: i32) -> i32 {
        self.engine.multiply(a, b)
    }
}

fn main() {
    let comp: Computer = Computer {};
    let result1 = comp.add(5, 3);
    let result2 = comp.multiply(4, 7);
    println!("{}", result1);
    println!("{}", result2);
}
