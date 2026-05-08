use semver::Version;

fn main() {
    let v = Version::parse("1.2.3").unwrap();
    if v > Version::parse("1.0.0").unwrap() {
        println!("version is greater than 1.0.0");
    }
}
