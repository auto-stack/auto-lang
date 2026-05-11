use semver::Version;

fn main() {
    let mut v = Version::parse("1.2.3").unwrap();
    v.patch += 1;
    println!("Next patch: {}", v);
    v.minor += 1;
    v.patch = 0;
    println!("Next minor: {}", v);
}
