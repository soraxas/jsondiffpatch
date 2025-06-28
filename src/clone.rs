use serde_json::Value;

pub fn clone(value: &Value) -> Value {
    // Use serde_json's built-in clone functionality
    value.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_clone_null() {
        let value = json!(null);
        let cloned = clone(&value);
        assert_eq!(cloned, value);
    }

    #[test]
    fn test_clone_bool() {
        let value = json!(true);
        let cloned = clone(&value);
        assert_eq!(cloned, value);
    }

    #[test]
    fn test_clone_number() {
        let value = json!(42);
        let cloned = clone(&value);
        assert_eq!(cloned, value);
    }

    #[test]
    fn test_clone_string() {
        let value = json!("hello");
        let cloned = clone(&value);
        assert_eq!(cloned, value);
    }

    #[test]
    fn test_clone_array() {
        let value = json!([1, 2, 3]);
        let cloned = clone(&value);
        assert_eq!(cloned, value);
    }

    #[test]
    fn test_clone_object() {
        let value = json!({"a": 1, "b": "hello"});
        let cloned = clone(&value);
        assert_eq!(cloned, value);
    }

    #[test]
    fn test_clone_nested() {
        let value = json!({
            "nested": {
                "array": [1, 2, {"object": true}],
                "string": "hello"
            }
        });
        let cloned = clone(&value);
        assert_eq!(cloned, value);
    }
}
