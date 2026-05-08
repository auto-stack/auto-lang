// Reference: Rust Cookbook - Sort a Vector of Structs
// Source: algorithms/sorting/sort_struct.md
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Person {
    name: String,
    age: u32
}

impl Person {
    pub fn new(name: &str, age: u32) -> Self {
        Person {
            name: name.to_string(),
            age
        }
    }
}

fn main() {
    let mut people = vec![
        Person::new("Zoe", 25),
        Person::new("Al", 60),
        Person::new("John", 1),
    ];

    people.sort();

    assert_eq!(
        people,
        vec![
            Person::new("Al", 60),
            Person::new("John", 1),
            Person::new("Zoe", 25),
        ]);

    people.sort_by(|a, b| b.age.cmp(&a.age));

    assert_eq!(
        people,
        vec![
            Person::new("Al", 60),
            Person::new("Zoe", 25),
            Person::new("John", 1),
        ]);
}
