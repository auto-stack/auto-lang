// Reference: Rust Cookbook - Read Environment Variable
// Source: os/external/read-env-variable.md
use std::env;
use std::fs;
use std::io::Error;

fn main() -> Result<(), Error> {
    let config_path = env::var("CONFIG")
        .unwrap_or("/etc/myapp/config".to_string());

    let config: String = fs::read_to_string(config_path)?;
    println!("Config: {}", config);

    Ok(())
}
