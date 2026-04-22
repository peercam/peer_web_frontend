use serde_json::Value;

/// Extract a GraphQL Decimal value (serialized as JSON string) as f64.
pub fn decimal_val(v: &Value) -> f64 {
    v.as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| v.as_f64())
        .expect("Expected a numeric value (string or number)")
}
