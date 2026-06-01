use serde_json::Value;

use super::super::fields::{first_symbol, string_field, unique_sorted_strings};

pub(in crate::retest_status) fn rows_with_promote_bias_symbols(rows: &[Value]) -> Vec<String> {
    unique_sorted_strings(rows.iter().filter_map(|row| {
        has_promote_bias(row)
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }))
}

pub(in crate::retest_status) fn rows_with_promote_bias_candidate_ids(
    rows: &[Value],
) -> Vec<String> {
    unique_sorted_strings(rows.iter().filter_map(|row| {
        has_promote_bias(row)
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }))
}

fn has_promote_bias(row: &Value) -> bool {
    row.get("gate_biases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .any(|bias| bias.starts_with("PROMOTE"))
}
