use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> std::io::Result<()> {
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"hello from stdin")?;
    }
    let output = child.wait_with_output()?;
    println!("Output: {}", String::from_utf8(output.stdout).unwrap());
    Ok(())
}
