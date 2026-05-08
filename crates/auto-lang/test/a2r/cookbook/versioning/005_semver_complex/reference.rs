use semver::{Version, VersionReq};

fn main() {
    let req = VersionReq::parse(">=1.2.0, <2.0.0").unwrap();
    let v1 = Version::parse("1.5.0").unwrap();
    let v2 = Version::parse("2.0.0").unwrap();
    println!("1.5.0 matches: {}", req.matches(&v1));
    println!("2.0.0 matches: {}", req.matches(&v2));
}
