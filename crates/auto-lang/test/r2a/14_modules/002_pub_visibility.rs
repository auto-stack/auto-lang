pub struct Config {
    name: String,
    value: i32,
}

impl Config {
    pub fn new(name: String) -> Config {
        Config { name, value: 0 }
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }
}

pub fn create_config(name: String) -> Config {
    Config::new(name)
}
