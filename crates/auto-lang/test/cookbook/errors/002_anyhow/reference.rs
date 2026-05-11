use anyhow::{Context, Result};

fn read_config() -> Result<String> {
    let content = std::fs::read_to_string("config.toml")
        .context("Failed to read config")?;
    Ok(content)
}

fn main() {
    let result = read_config();
    println!("Config result: {:?}", result);
}
