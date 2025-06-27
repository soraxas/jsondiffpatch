use serde_json::Value;

pub fn clone(value: &Value) -> Value {
    match value {
        Value::Null => Value::Null,
        Value::Bool(b) => Value::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(serde_json::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                // Note: This is a simplified approach. In practice, you'd want to handle
                // floating point precision more carefully
                if let Some(n) = serde_json::Number::from_f64(f) {
                    Value::Number(n)
                } else {
                    Value::Null
                }
            } else {
                Value::Null
            }
        }
        Value::String(s) => Value::String(s.clone()),
        Value::Array(arr) => {
            let cloned_arr: Vec<Value> = arr.iter().map(clone).collect();
            Value::Array(cloned_arr)
        }
        Value::Object(obj) => {
            let cloned_obj: serde_json::Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), clone(v)))
                .collect();
            Value::Object(cloned_obj)
        }
    }
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
            "array": [1, 2, {"nested": true}],
            "object": {"key": "value"}
        });
        let cloned = clone(&value);
        assert_eq!(cloned, value);
    }
}