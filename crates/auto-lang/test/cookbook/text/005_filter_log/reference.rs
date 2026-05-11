use regex::Regex;

fn main() {
    let re = Regex::new(r"ERROR: (.+)").unwrap();
    let log = "INFO: started\nERROR: file not found\nWARN: retrying\nERROR: disk full";
    for line in log.lines() {
        if let Some(caps) = re.captures(line) {
            let msg = caps.get(1).unwrap().as_str();
            println!("Error: {}", msg);
        }
    }
}
