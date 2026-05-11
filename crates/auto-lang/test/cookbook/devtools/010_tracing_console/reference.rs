use tracing::{info, debug};
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();
    info!("application started");
    let count = 10;
    debug!("processing {} items", count);
    info!("application finished");
}
