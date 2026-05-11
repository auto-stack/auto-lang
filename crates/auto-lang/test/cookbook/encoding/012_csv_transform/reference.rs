use csv::{Reader, Writer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = "name,age\nAlice,30\nBob,25";
    let mut reader = Reader::from_reader(data.as_bytes());
    let mut writer = Writer::from_writer(Vec::new());
    writer.write_record(&["name", "age", "decade"])?;
    for result in reader.records() {
        let record = result?;
        let name = record.get(0).unwrap();
        let age: i32 = record.get(1).unwrap().parse().unwrap();
        let decade = age / 10 * 10;
        writer.write_record(&[name, &age.to_string(), &decade.to_string()])?;
    }
    let output = String::from_utf8(writer.into_inner()?)?;
    println!("{}", output);
    Ok(())
}
