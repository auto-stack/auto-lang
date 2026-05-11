use syslog::{BasicLogger, Facility, Formatter3164};
use log::{info, LevelFilter};

fn main() {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "myprogram".into(),
        pid: std::process::id() as i32,
    };
    let logger = syslog::unix(formatter).unwrap();
    log::set_boxed_logger(Box::new(BasicLogger::new(logger))).unwrap();
    log::set_max_level(LevelFilter::Info);
    info!("sending to syslog");
    info!("daemon started");
    println!("syslog configured");
}
