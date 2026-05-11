use csv::ReaderBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = "name;age\nAlice;30\nBob;25";
    let mut reader = ReaderBuilder::new().delimiter(b';').from_reader(data.as_bytes());
    for result in reader.records() {
        let record = result?;
        println!("Name: {}, Age: {}", record.get(0).unwrap(), record.get(1).unwrap());
    }
    Ok(())
}
