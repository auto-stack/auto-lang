use log::info;
use simplelog::*;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create("app.log")?;
    WriteLogger::init(LevelFilter::Info, Config::default(), file)?;
    info!("logging to custom file");
    println!("log written to app.log");
    Ok(())
}
