// Reference: Rust Cookbook - Handle errors correctly in main (Box<dyn Error>)
// Source: errors/handle/main.md
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let result: Result<(), Box<dyn Error>> = Ok(());
    result
}
