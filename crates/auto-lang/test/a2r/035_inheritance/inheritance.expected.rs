struct Animal {
    name: String,
}

impl Animal {
    fn speak(&self) {
        println!("Animal sound");
    }
}

struct Dog {
    name: String,
    breed: String,
}

impl Dog {
    fn bark(&self) {
        println!("Woof!");
    }
    fn speak(&self) {
        println!("Animal sound");
    }
}

fn main() {
    let dog: Dog = Dog {};
    dog.name = "Buddy";
    dog.breed = "Labrador";

    dog.speak();
    dog.bark();
}
