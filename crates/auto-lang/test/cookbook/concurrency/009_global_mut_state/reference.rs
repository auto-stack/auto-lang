use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c1 = Arc::clone(&counter);
    let c2 = Arc::clone(&counter);

    let t1 = thread::spawn(move || {
        for _ in 0..100 {
            c1.fetch_add(1, Ordering::SeqCst);
        }
    });

    let t2 = thread::spawn(move || {
        for _ in 0..100 {
            c2.fetch_add(1, Ordering::SeqCst);
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();
    println!("Final count: {}", counter.load(Ordering::SeqCst));
}
