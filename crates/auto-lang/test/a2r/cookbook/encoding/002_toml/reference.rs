use serde::Deserialize;
use toml;

#[derive(Deserialize, Debug)]
struct Config {
    title: Option<String>,
    owner: Owner,
}

#[derive(Deserialize, Debug)]
struct Owner {
    name: String,
}

fn main() {
    let toml_str = r#"[owner]
name = "Alice""#;
    let config: Config = toml::from_str(toml_str).unwrap();
    println!("Owner: {}", config.owner.name);
}
