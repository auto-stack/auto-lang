struct Counter {
    count: i32,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
    fn increment(&mut self) {
        self.count = self.count + 1;
    }
    fn get_count(&self) -> i32 {
        self.count
    }
}

fn main() {
    let c = Counter::new();
    c.increment();
    c.increment();
    println!("{}", c.get_count());
}
