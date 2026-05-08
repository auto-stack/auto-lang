use csv::Writer;
use serde::Serialize;

#[derive(Serialize)]
struct Record {
    name: String,
    age: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let records = vec![
        Record { name: "Alice".into(), age: 30 },
        Record { name: "Bob".into(), age: 25 },
    ];
    let mut writer = Writer::from_writer(Vec::new());
    for record in records {
        writer.serialize(record)?;
    }
    let output = String::from_utf8(writer.into_inner()?)?;
    println!("{}", output);
    Ok(())
}
