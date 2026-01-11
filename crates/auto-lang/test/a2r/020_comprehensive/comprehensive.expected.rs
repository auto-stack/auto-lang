struct Math {
    value: i32,
}

impl Math {
    fn square(&self, x: i32) -> i32 {
        x * x
    }
}

fn main() {
    let m: Math = Math { value: 0 };
    let sq = m.square(5);

    if sq > 20 {
        println!("Square is large: {}", sq);
    } else {
        println!("Square is small: {}", sq);
    }


    match sq {
        25 => println!("Square is 25"),
        _ => println!("Square is not 25"),
    }

    let arr: [i32; 3] = [1, 2, 3];
    println!("Array: {}", arr);
}
