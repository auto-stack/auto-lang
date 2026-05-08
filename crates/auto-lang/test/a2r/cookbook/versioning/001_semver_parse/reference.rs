use semver::Version;

fn main() {
    let v = Version::parse("1.2.3").unwrap();
    println!("Major: {}, Minor: {}, Patch: {}", v.major, v.minor, v.patch);
}
