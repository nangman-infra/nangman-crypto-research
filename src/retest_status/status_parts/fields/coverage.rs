use super::accessors::{first_symbol, string_array_pointer, string_field};
use super::sets::{intersection_sorted, unique_sorted_strings};
use serde_json::Value;
use std::collections::BTreeSet;

pub(in crate::retest_status::status_parts) fn candidate_symbols_in_approved_universe_len(
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

pub(in crate::retest_status::status_parts) fn eligible_candidate_symbols_in_approved_universe_len(
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

pub(in crate::retest_status::status_parts) fn coverage(
    numerator: usize,
    denominator: usize,
) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}
