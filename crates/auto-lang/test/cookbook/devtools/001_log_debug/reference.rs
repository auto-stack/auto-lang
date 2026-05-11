use log::{debug, info};

fn main() {
    env_logger::init();
    debug!("starting operation");
    let value = 42;
    debug!("value = {}", value);
    println!("done");
}
