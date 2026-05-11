use csv::Writer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = Writer::from_writer(Vec::new());
    writer.write_record(&["name", "age"])?;
    writer.write_record(&["Alice", "30"])?;
    writer.write_record(&["Bob", "25"])?;
    let output = String::from_utf8(writer.into_inner()?)?;
    println!("{}", output);
    Ok(())
}
