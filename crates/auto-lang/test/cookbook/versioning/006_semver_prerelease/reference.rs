use semver::Version;

fn main() {
    let v = Version::parse("1.0.0-alpha.1").unwrap();
    println!("Major: {}", v.major);
    println!("Pre: {}", v.pre);
    let is_pre = !v.pre.is_empty();
    println!("Is prerelease: {}", is_pre);
}
