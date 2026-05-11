use log::{info, LevelFilter, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

fn main() {
    log::set_boxed_logger(Box::new(SimpleLogger)).unwrap();
    log::set_max_level(LevelFilter::Debug);
    info!("custom logger active");
    println!("done");
}
