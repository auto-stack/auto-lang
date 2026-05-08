use std::fs::File;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create("archive.tar.gz")?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);
    tar.append_dir_all("backup", "src")?;
    tar.finish()?;
    println!("Created archive.tar.gz");
    Ok(())
}
