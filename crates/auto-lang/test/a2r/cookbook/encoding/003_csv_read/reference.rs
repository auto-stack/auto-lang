use csv;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let csv_str = "name,age
Alice,30
Bob,25";
    let mut reader = csv::Reader::from_reader(csv_str.as_bytes());
    for result in reader.records() {
        let record = result?;
        println!("Name: {}, Age: {}", record.get(0).unwrap(), record.get(1).unwrap());
    }
    Ok(())
}
