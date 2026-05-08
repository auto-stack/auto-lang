use std::fs;
use tempfile::NamedTempFile;

fn main() -> std::io::Result<()> {
    let mut temp = NamedTempFile::new()?;
    fs::write(temp.path(), "line1\nline2\nline3")?;
    let content = fs::read_to_string(temp.path())?;
    for line in content.lines() {
        println!("Line: {}", line);
    }
    temp.close()?;
    Ok(())
}
