// Book listing a2r transpilation tests
// Verifies that main.at transpiles to match main.expected.rs
// Run: cargo test -p auto-lang --lib -- book_listing

use crate::{
    error::AutoResult,
    trans::rust::transpile_rust,
};
use std::fs;

fn test_book_listing(chapter: &str, listing: &str) -> AutoResult<()> {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Navigate from auto-lang/crates/auto-lang to book/rust/listings
    let listing_dir = d
        .join("../../../book/rust/listings")
        .join(chapter)
        .join(listing);

    let at_path = listing_dir.join("main.at");
    let exp_path = listing_dir.join("main.expected.rs");

    let src = fs::read_to_string(&at_path)
        .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;
    let expected = fs::read_to_string(&exp_path)
        .map_err(|e| format!("Failed to read {}: {}", exp_path.display(), e))?;

    let mut rcode = transpile_rust("main", &src)?;
    let actual = String::from_utf8_lossy(&rcode.done()?).to_string();

    if actual != expected {
        // Write .wrong.rs for comparison
        let wrong_path = listing_dir.join("main.wrong.rs");
        fs::write(&wrong_path, &actual)?;
        assert_eq!(
            actual, expected,
            "\nTranspilation mismatch for {}/{}. \
             See main.wrong.rs vs main.expected.rs",
            chapter, listing
        );
    }

    Ok(())
}

/// Generate main.expected.rs for all book listings.
/// Run: cargo test -p auto-lang --lib -- generate_book_expected --nocapture
#[test]
fn generate_book_expected() -> AutoResult<()> {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let listings_base = d.join("../../../book/rust/listings");

    let chapters = ["ch01", "ch02", "ch03", "ch04", "ch05", "ch06", "ch07", "ch08", "ch09"];
    let mut generated = 0;
    let mut failed = 0;

    for ch in &chapters {
        let ch_dir = listings_base.join(ch);
        if !ch_dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&ch_dir)? {
            let entry = entry?;
            let listing_dir = entry.path();
            if !listing_dir.is_dir() {
                continue;
            }

            let at_path = listing_dir.join("main.at");
            if !at_path.exists() {
                continue;
            }

            let listing_name = listing_dir
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let src = fs::read_to_string(&at_path)?;

            match transpile_rust("main", &src) {
                Ok(mut rcode) => {
                    let rs_code = String::from_utf8_lossy(&rcode.done()?).to_string();
                    let exp_path = listing_dir.join("main.expected.rs");
                    fs::write(&exp_path, &rs_code)?;
                    println!("OK: {}/{}", ch, listing_name);
                    generated += 1;
                }
                Err(e) => {
                    eprintln!("FAIL: {}/{}: {}", ch, listing_name, e);
                    failed += 1;
                }
            }
        }
    }

    println!(
        "\nGenerated {} expected.rs files, {} failed",
        generated, failed
    );
    Ok(())
}

// Ch01
#[test]
fn book_ch01_01() -> AutoResult<()> {
    test_book_listing("ch01", "listing-01-01")
}

// Ch02
#[test]
fn book_ch02_01() -> AutoResult<()> {
    test_book_listing("ch02", "listing-02-01")
}
#[test]
fn book_ch02_03() -> AutoResult<()> {
    test_book_listing("ch02", "listing-02-03")
}
#[test]
fn book_ch02_04() -> AutoResult<()> {
    test_book_listing("ch02", "listing-02-04")
}
#[test]
fn book_ch02_05() -> AutoResult<()> {
    test_book_listing("ch02", "listing-02-05")
}
#[test]
fn book_ch02_06() -> AutoResult<()> {
    test_book_listing("ch02", "listing-02-06")
}

// Ch03
#[test]
fn book_ch03_01() -> AutoResult<()> {
    test_book_listing("ch03", "listing-03-01")
}
#[test]
fn book_ch03_02() -> AutoResult<()> {
    test_book_listing("ch03", "listing-03-02")
}
#[test]
fn book_ch03_03() -> AutoResult<()> {
    test_book_listing("ch03", "listing-03-03")
}
#[test]
fn book_ch03_04() -> AutoResult<()> {
    test_book_listing("ch03", "listing-03-04")
}
#[test]
fn book_ch03_05() -> AutoResult<()> {
    test_book_listing("ch03", "listing-03-05")
}
#[test]
fn book_ch03_06() -> AutoResult<()> {
    test_book_listing("ch03", "listing-03-06")
}
#[test]
fn book_ch03_07() -> AutoResult<()> {
    test_book_listing("ch03", "listing-03-07")
}
#[test]
fn book_ch03_08() -> AutoResult<()> {
    test_book_listing("ch03", "listing-03-08")
}

// Ch04
#[test]
fn book_ch04_01() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-01")
}
#[test]
fn book_ch04_02() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-02")
}
#[test]
fn book_ch04_03() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-03")
}
#[test]
fn book_ch04_04() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-04")
}
#[test]
fn book_ch04_05() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-05")
}
#[test]
fn book_ch04_06() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-06")
}
#[test]
fn book_ch04_07() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-07")
}
#[test]
fn book_ch04_08() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-08")
}
#[test]
fn book_ch04_09() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-09")
}
#[test]
fn book_ch04_10() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-10")
}
#[test]
fn book_ch04_11() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-11")
}
#[test]
fn book_ch04_12() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-12")
}
#[test]
fn book_ch04_13() -> AutoResult<()> {
    test_book_listing("ch04", "listing-04-13")
}

// Ch05
#[test]
fn book_ch05_01() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-01")
}
#[test]
fn book_ch05_02() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-02")
}
#[test]
fn book_ch05_03() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-03")
}
#[test]
fn book_ch05_04() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-04")
}
#[test]
fn book_ch05_05() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-05")
}
#[test]
fn book_ch05_06() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-06")
}
#[test]
fn book_ch05_07() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-07")
}
#[test]
fn book_ch05_08() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-08")
}
#[test]
fn book_ch05_09() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-09")
}
#[test]
fn book_ch05_10() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-10")
}
#[test]
fn book_ch05_11() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-11")
}
#[test]
fn book_ch05_12() -> AutoResult<()> {
    test_book_listing("ch05", "listing-05-12")
}

// Ch06
#[test]
fn book_ch06_01() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-01")
}
#[test]
fn book_ch06_02() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-02")
}
#[test]
fn book_ch06_03() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-03")
}
#[test]
fn book_ch06_04() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-04")
}
#[test]
fn book_ch06_05() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-05")
}
#[test]
fn book_ch06_06() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-06")
}
#[test]
fn book_ch06_07() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-07")
}
#[test]
fn book_ch06_08() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-08")
}
#[test]
fn book_ch06_09() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-09")
}
#[test]
fn book_ch06_10() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-10")
}
#[test]
fn book_ch06_11() -> AutoResult<()> {
    test_book_listing("ch06", "listing-06-11")
}

// Ch07
#[test]
fn book_ch07_01() -> AutoResult<()> {
    test_book_listing("ch07", "listing-07-01")
}
#[test]
fn book_ch07_02() -> AutoResult<()> {
    test_book_listing("ch07", "listing-07-02")
}
#[test]
fn book_ch07_03() -> AutoResult<()> {
    test_book_listing("ch07", "listing-07-03")
}
#[test]
fn book_ch07_04() -> AutoResult<()> {
    test_book_listing("ch07", "listing-07-04")
}
#[test]
fn book_ch07_05() -> AutoResult<()> {
    test_book_listing("ch07", "listing-07-05")
}

// Ch08
#[test]
fn book_ch08_01() -> AutoResult<()> {
    test_book_listing("ch08", "listing-08-01")
}
#[test]
fn book_ch08_02() -> AutoResult<()> {
    test_book_listing("ch08", "listing-08-02")
}
#[test]
fn book_ch08_03() -> AutoResult<()> {
    test_book_listing("ch08", "listing-08-03")
}
#[test]
fn book_ch08_04() -> AutoResult<()> {
    test_book_listing("ch08", "listing-08-04")
}
#[test]
fn book_ch08_05() -> AutoResult<()> {
    test_book_listing("ch08", "listing-08-05")
}
#[test]
fn book_ch08_06() -> AutoResult<()> {
    test_book_listing("ch08", "listing-08-06")
}
#[test]
fn book_ch08_07() -> AutoResult<()> {
    test_book_listing("ch08", "listing-08-07")
}

// Ch09
#[test]
fn book_ch09_01() -> AutoResult<()> {
    test_book_listing("ch09", "listing-09-01")
}
#[test]
fn book_ch09_02() -> AutoResult<()> {
    test_book_listing("ch09", "listing-09-02")
}
#[test]
fn book_ch09_03() -> AutoResult<()> {
    test_book_listing("ch09", "listing-09-03")
}
#[test]
fn book_ch09_04() -> AutoResult<()> {
    test_book_listing("ch09", "listing-09-04")
}
#[test]
fn book_ch09_05() -> AutoResult<()> {
    test_book_listing("ch09", "listing-09-05")
}
#[test]
fn book_ch09_06() -> AutoResult<()> {
    test_book_listing("ch09", "listing-09-06")
}
