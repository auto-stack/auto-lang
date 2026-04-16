enum Coin {
    Penny = 0,
    Nickel = 1,
    Dime = 2,
    Quarter = 3,
}

fn main() {
    let c = Coin::Penny;
    match c {
        Coin::Penny => println!("Penny!"),
        Coin::Nickel => println!("Nickel!"),
        _ => println!("Other"),
    }
}
