use csv::Reader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = "name,age\nAlice,30\nBob,25\nCharlie,35";
    let mut reader = Reader::from_reader(data.as_bytes());
    for result in reader.records() {
        let record = result?;
        let age: i32 = record.get(1).unwrap().parse().unwrap();
        if age > 28 {
            println!("Name: {}, Age: {}", record.get(0).unwrap(), record.get(1).unwrap());
        }
    }
    Ok(())
}
