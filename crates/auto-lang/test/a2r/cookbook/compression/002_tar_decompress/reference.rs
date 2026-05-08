use std::fs::File;
use flate2::read::GzDecoder;
use tar::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("archive.tar.gz")?;
    let dec = GzDecoder::new(file);
    let mut archive = Archive::new(dec);
    archive.unpack("target")?;
    println!("Unpacked archive.tar.gz");
    Ok(())
}
