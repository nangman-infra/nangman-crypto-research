use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::BTreeSet;

pub(in crate::retest_status) fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

pub(in crate::retest_status) fn i64_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64)
}

pub(super) fn bool_field(value: &Value, field: &str) -> Option<bool> {
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

pub(super) fn string_array_field(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn string_array_pointer(value: &Value, pointer: &str) -> Vec<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

pub(in crate::retest_status) fn unique_sorted_strings<'a>(
    values: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    values
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn intersection_sorted(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
) -> Vec<String> {
    left.intersection(right).cloned().collect()
}

pub(super) fn difference_sorted(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.difference(right).cloned().collect()
}

pub(super) fn candidate_symbols_in_approved_universe_len(
    latest_universe: &Value,
    rows: &[Value],
) -> usize {
    let approved_symbols = string_array_pointer(latest_universe, "/approved_symbols")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let candidate_symbols = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    intersection_sorted(&candidate_symbols, &approved_symbols).len()
}

pub(super) fn eligible_candidate_symbols_in_approved_universe_len(
    latest_universe: &Value,
    driver: &Value,
    rows: &[Value],
) -> usize {
    let approved_symbols = string_array_pointer(latest_universe, "/approved_symbols")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let eligible_symbols = {
        let symbols = string_array_pointer(driver, "/manifest/eligible_candidate_symbols");
        if symbols.is_empty() {
            unique_sorted_strings(rows.iter().filter_map(|row| {
                string_field(row, "primary_symbol").or_else(|| first_symbol(row))
            }))
        } else {
            symbols
        }
    }
    .into_iter()
    .collect::<BTreeSet<_>>();
    intersection_sorted(&eligible_symbols, &approved_symbols).len()
}

pub(super) fn coverage(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

pub(super) fn horizon_rank(horizon: &str) -> usize {
    match horizon {
        "1h" => 1,
        "4h" => 2,
        "24h" | "1d" => 3,
        "72h" => 4,
        "7d" => 5,
        _ => 99,
    }
}

pub(in crate::retest_status) fn iso8601_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    DateTime::<Utc>::from_timestamp(secs, 0)
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}
