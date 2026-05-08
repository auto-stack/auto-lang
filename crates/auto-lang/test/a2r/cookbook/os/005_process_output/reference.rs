use std::process::Command;

fn main() -> std::io::Result<()> {
    let output = Command::new("ls")
        .args(["-la"])
        .output()?;
    println!("Exit code: {}", output.status);
    println!("Stdout: {}", String::from_utf8(output.stdout).unwrap());
    Ok(())
}
