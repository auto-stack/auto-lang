use log::error;

fn main() {
    env_logger::init();
    error!("something went wrong");
    println!("operation completed with errors");
}
