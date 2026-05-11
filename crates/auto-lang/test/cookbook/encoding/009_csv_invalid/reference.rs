use csv::Reader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = "name,age\nAlice,30\nBob,twenty-five\nCharlie,35";
    let mut reader = Reader::from_reader(data.as_bytes());
    for result in reader.records() {
        match result {
            Ok(record) => {
                println!("Name: {}, Age: {}", record.get(0).unwrap(), record.get(1).unwrap());
            }
            Err(e) => {
                println!("Error reading record: {}", e);
            }
        }
    }
    Ok(())
}
