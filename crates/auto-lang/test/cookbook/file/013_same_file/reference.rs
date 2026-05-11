use std::path::Path;
use same_file::is_same_file;

fn main() {
    let path1 = Path::new("a.txt");
    let path2 = Path::new("a.txt");
    if is_same_file(path1, path2).unwrap() {
        println!("Same file, skipping copy");
    }
}
