use std::error::Error;

fn inner() -> Result<String, Box<dyn Error>> {
    Err("inner error".into())
}

fn outer() -> Result<String, Box<dyn Error>> {
    let result = inner()?;
    Ok(result)
}

fn main() {
    match outer() {
        Ok(val) => println!("Success: {}", val),
        Err(e) => println!("Error: {}", e),
    }
}
