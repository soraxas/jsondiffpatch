use chrono::{DateTime, Utc};
use serde_json::Value;

pub fn date_reviver(key: &str, value: &Value) -> Value {
    if key == "$date" {
        if let Some(date_str) = value.as_str() {
            if let Ok(datetime) = DateTime::parse_from_rfc3339(date_str) {
                return serde_json::to_value(datetime.with_timezone(&Utc)).unwrap_or(value.clone());
            }
        }
    }
    value.clone()
}

pub fn serialize_date(date: &DateTime<Utc>) -> Value {
    serde_json::json!({
        "$date": date.to_rfc3339()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_date_reviver() {
        let date_str = "2023-01-01T12:00:00Z";
        let date_value = serde_json::Value::String(date_str.to_string());
        let result = date_reviver("$date", &date_value);

        // The result should be a parsed DateTime
        assert!(result.is_object());
    }

    #[test]
    fn test_serialize_date() {
        let date = Utc.ymd(2023, 1, 1).and_hms(12, 0, 0);
        let serialized = serialize_date(&date);

        assert!(serialized.is_object());
        if let Some(date_obj) = serialized.as_object() {
            assert!(date_obj.contains_key("$date"));
        }
    }
}
