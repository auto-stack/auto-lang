use crossbeam::channel::unbounded;
use std::thread;

fn main() {
    let (tx, rx) = unbounded();

    thread::spawn(move || {
        tx.send("hello").unwrap();
        tx.send("world").unwrap();
    });

    for msg in rx {
        println!("Received: {}", msg);
        if msg == "world" {
            break;
        }
    }
}
