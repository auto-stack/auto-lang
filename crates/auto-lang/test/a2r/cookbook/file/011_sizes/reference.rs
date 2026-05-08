use std::fs;

fn main() -> std::io::Result<()> {
    let mut total: u64 = 0;
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let meta = entry.metadata()?;
        total += meta.len();
    }
    println!("Total size: {} bytes", total);
    Ok(())
}
