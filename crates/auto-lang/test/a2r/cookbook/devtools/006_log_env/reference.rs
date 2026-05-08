use log::info;
use env_logger;

fn main() {
    env_logger::init();
    info!("this is an info message");
    println!("check RUST_LOG env var");
}
