use flate2::read::GzDecoder;
use std::fs::File;
use tar::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("archive.tar.gz")?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);
    archive.set_prefix_strip(3);
    for entry in archive.entries()? {
        let entry = entry?;
        println!("Extracted: {}", entry.path()?.display());
    }
    Ok(())
}
