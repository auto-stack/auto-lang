use std::thread;

fn main() {
    let cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    println!("CPU cores: {}", cores);
}
