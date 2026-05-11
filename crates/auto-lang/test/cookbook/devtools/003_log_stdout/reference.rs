use log::info;
use env_logger;

fn main() {
    env_logger::init();
    info!("application started");
    info!("processing data");
    println!("done");
}
