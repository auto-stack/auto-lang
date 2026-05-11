use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let choices = ["a", "b", "c", "d", "e"];
    let choice = choices[rng.gen_range(0..choices.len())];
    println!("Chose: {}", choice);
}
