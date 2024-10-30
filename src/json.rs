use crate::value::Value;
use serde_json;

pub fn to_json(value: &Value) -> String {
    serde_json::to_string(&value).unwrap()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_json() {
        let value = Value::Int(1);
        let json = to_json(&value);
        assert_eq!(json, "{\"Int\":1}");
    }
}
