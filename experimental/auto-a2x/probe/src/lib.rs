//! PLAN-892 probe crate library surface (bins + shared vendored inputs).

#[path = "load256_vendored.rs"]
pub mod load256;

#[cfg(test)]
mod tests {
    #[test]
    fn frozen_load256_facts_hold() {
        let (bytes, fns) = crate::load256::frozen_facts_ok();
        assert_eq!((bytes, fns), (273_575, 115));
    }
}
