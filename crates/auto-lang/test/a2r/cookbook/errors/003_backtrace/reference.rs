use std::backtrace::Backtrace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bt = Backtrace::capture();
    println!("Backtrace: {:?}", bt);
    Ok(())
}
