use log::{info, debug};

mod network {
    use log::debug;

    pub fn connect() {
        debug!("connecting to server");
    }
}

fn process() {
    debug!("processing step 1");
    debug!("processing step 2");
    info!("processing complete");
}

fn main() {
    env_logger::init();
    info!("app started");
    network::connect();
    process();
    println!("done");
}
