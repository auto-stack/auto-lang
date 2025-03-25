pub use ecow::EcoString as AutoStr;

pub static ASTR_EMPTY: AutoStr = AutoStr::new();

pub trait StrExt {
    fn to_camel(&self) -> AutoStr;
}

impl StrExt for AutoStr {
    fn to_camel(&self) -> AutoStr {
        let mut camel = AutoStr::new();
        let mut capitalize_next = true;

        for c in self.chars() {
            if c.is_whitespace() {
                capitalize_next = true;
            } else if c == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                camel.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                camel.push(c);
            }
        }

        camel
    }
}

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

    #[test]
    fn test_to_camel() {
        let input = AutoStr::from("hello world");
        let expected = AutoStr::from("HelloWorld");
        assert_eq!(input.to_camel(), expected);

        let input = AutoStr::from("hello_world");
        let expected = AutoStr::from("HelloWorld");
        assert_eq!(input.to_camel(), expected);
    }
}
