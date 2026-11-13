//! PLAN-892 probe crate library surface (bins + shared vendored inputs).

#[path = "load256_vendored.rs"]
pub mod load256;

/// PLAN-921 CA-A: the controlled MSVC C11 compile+run leg (corpus,
/// certification and worker ops share it).
pub mod coll_formal_load;
pub mod crun;
pub mod pyrun;

#[cfg(test)]
mod tests {
    #[test]
    fn frozen_load256_facts_hold() {
        let (bytes, fns) = crate::load256::frozen_facts_ok();
        assert_eq!((bytes, fns), (273_575, 115));
    }
}
