use serde_json::Value;

pub(in crate::retest_status) fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

pub(in crate::retest_status) fn i64_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64)
}

pub(in crate::retest_status::status_parts) fn bool_field(
    value: &Value,
    field: &str,
) -> Option<bool> {
    value.get(field).and_then(Value::as_bool)
}

pub(in crate::retest_status) fn bool_pointer(value: &Value, pointer: &str) -> Option<bool> {
    value.pointer(pointer).and_then(Value::as_bool)
}

pub(in crate::retest_status) fn first_symbol(value: &Value) -> Option<&str> {
    value
        .get("symbols")
        .and_then(Value::as_array)
        .and_then(|symbols| symbols.first())
        .and_then(Value::as_str)
}

pub(in crate::retest_status::status_parts) fn string_array_field(
    value: &Value,
    field: &str,
) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

pub(in crate::retest_status::status_parts) fn string_array_pointer(
    value: &Value,
    pointer: &str,
) -> Vec<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}
