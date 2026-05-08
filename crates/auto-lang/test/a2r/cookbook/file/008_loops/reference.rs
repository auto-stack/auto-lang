use std::path::PathBuf;
use std::collections::HashSet;

fn visit(path: &PathBuf, visited: &mut HashSet<PathBuf>) {
    if visited.contains(path) {
        println!("Loop detected: {}", path.display());
        return;
    }
    visited.insert(path.clone());
}

fn main() {
    let mut visited = HashSet::new();
    visit(&PathBuf::from("."), &mut visited);
}
