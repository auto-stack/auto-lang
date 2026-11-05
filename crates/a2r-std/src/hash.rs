/// File hashing has the VM's catchable failure behavior; it never substitutes
/// an empty digest for an unreadable file.
pub fn file_sha256(path: &str) -> String {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("file_sha256: {path}: {error}"));
    format!("{:x}", Sha256::digest(&bytes))
}

#[cfg(test)]
mod tests {
    #[test]
    fn sha256_uses_file_bytes_and_missing_file_is_catchable() {
        let path = std::env::temp_dir().join(format!("reader-sha-{}", crate::uuid()));
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(
            super::file_sha256(path.to_str().unwrap()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        std::fs::remove_file(&path).unwrap();
        assert!(std::panic::catch_unwind(|| super::file_sha256(path.to_str().unwrap())).is_err());
    }
}
