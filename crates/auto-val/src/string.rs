pub use ecow::EcoString as AutoStr;

pub static ASTR_EMPTY: AutoStr = AutoStr::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astr_as_hash_key() {
        use std::collections::HashMap;

        let mut map: HashMap<AutoStr, AutoStr> = HashMap::new();
        let key = AutoStr::from("key");
        let val = AutoStr::from("value");
        map.insert(key.clone(), val.clone());

        let key1 = AutoStr::from("key");
        assert_eq!(map.get(&key1), Some(&val));
        assert_eq!(map.get(&key), Some(&val));
    }
}