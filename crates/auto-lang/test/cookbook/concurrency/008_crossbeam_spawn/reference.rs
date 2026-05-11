use crossbeam;

fn main() {
    crossbeam::scope(|s| {
        s.spawn(|_| {
            println!("Hello from scoped thread 1");
        });
        s.spawn(|_| {
            println!("Hello from scoped thread 2");
        });
    }).unwrap();
    println!("All threads completed");
}
