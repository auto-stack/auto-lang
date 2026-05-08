use std::process::{Command, Stdio};

fn main() -> std::io::Result<()> {
    let output = Command::new("echo")
        .arg("hello")
        .stdout(Stdio::piped())
        .output()?;
    let stdout = String::from_utf8(output.stdout).unwrap();
    let trimmed = stdout.trim();
    println!("Output: {}", trimmed);
    Ok(())
}
