use log::info;
use chrono::Local;
use env_logger::Builder;
use std::io::Write;

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(buf, "[{}] {} - {}", Local::now().format("%Y-%m-%d %H:%M:%S"), record.level(), record.args())
        })
        .init();
    info!("application started");
    info!("with timestamp logging");
}
