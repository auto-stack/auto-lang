use semver::Version;

fn main() {
    let versions = ["1.0.0", "2.1.0", "1.5.0", "2.0.1"];
    let max = versions.iter().map(|v| Version::parse(v).unwrap()).max();
    println!("Latest: {:?}", max);
}
