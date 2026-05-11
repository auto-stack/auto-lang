use walkdir::WalkDir;
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let walker = WalkDir::new(".").into_iter();

    thread::spawn(move || {
        for entry in walker {
            let _ = tx.send(entry);
        }
    });

    let mut count = 0;
    for entry in rx {
        let entry = entry.unwrap();
        count += 1;
    }
    println!("Total entries: {}", count);
}
