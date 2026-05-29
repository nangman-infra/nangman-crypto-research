use serde_json::{Value, json};
use std::collections::BTreeSet;

use super::super::fields::{
    coverage, difference_sorted, first_symbol, intersection_sorted, string_array_pointer,
    string_field, unique_sorted_strings,
};

pub(in crate::retest_status) fn major50_state(
    latest_universe: &Value,
    driver: &Value,
    rows: &[Value],
) -> Value {
    let candidate_symbols = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    );
    let approved_symbols = string_array_pointer(latest_universe, "/approved_symbols");
    let observed_symbols = string_array_pointer(latest_universe, "/observed_symbols");
    let eligible_candidate_symbols =
        string_array_pointer(driver, "/manifest/eligible_candidate_symbols")
            .into_iter()
            .collect::<BTreeSet<_>>();
    let eligible_candidate_symbols = if eligible_candidate_symbols.is_empty() {
        candidate_symbols.iter().cloned().collect::<BTreeSet<_>>()
    } else {
        eligible_candidate_symbols
    };
    let approved_symbol_set = approved_symbols.iter().cloned().collect::<BTreeSet<_>>();
    let candidate_symbol_set = candidate_symbols.iter().cloned().collect::<BTreeSet<_>>();
    let candidate_symbols_in_approved_universe =
        intersection_sorted(&candidate_symbol_set, &approved_symbol_set);
    let eligible_candidate_symbols_in_approved_universe =
        intersection_sorted(&eligible_candidate_symbols, &approved_symbol_set);
    let approved_symbols_without_selected_candidate =
        difference_sorted(&approved_symbol_set, &candidate_symbol_set);
    let approved_symbols_without_eligible_candidate =
        difference_sorted(&approved_symbol_set, &eligible_candidate_symbols);
    json!({
        "universe_mode": driver.pointer("/manifest/universe_mode").cloned().unwrap_or(Value::Null),
        "latest_universe_present": latest_universe.get("present").cloned().unwrap_or(Value::Null),
        "observed_symbol_count": latest_universe.get("observed_symbol_count").and_then(Value::as_u64).unwrap_or(observed_symbols.len() as u64),
        "approved_symbol_count": latest_universe.get("approved_symbol_count").and_then(Value::as_u64).unwrap_or(approved_symbols.len() as u64),
        "excluded_symbol_count": latest_universe.get("excluded_symbol_count").cloned().unwrap_or(Value::Null),
        "candidate_symbol_count": candidate_symbols.len(),
        "candidate_symbols": candidate_symbols,
        "eligible_candidate_pool_count": driver.pointer("/manifest/eligible_candidate_pool_count").cloned().unwrap_or(Value::Null),
        "selected_candidate_limit_reached": driver.pointer("/manifest/selected_candidate_limit_reached").cloned().unwrap_or(Value::Null),
        "unselected_eligible_candidate_count": driver.pointer("/manifest/unselected_eligible_candidate_count").cloned().unwrap_or(Value::Null),
        "selected_current_approved_candidate_count": driver.pointer("/manifest/selected_current_approved_candidate_count").cloned().unwrap_or(Value::Null),
        "eligible_candidate_symbols": eligible_candidate_symbols.iter().cloned().collect::<Vec<_>>(),
        "unselected_eligible_candidate_symbols": string_array_pointer(driver, "/manifest/unselected_eligible_candidate_symbols"),
        "candidate_symbols_in_approved_universe": candidate_symbols_in_approved_universe,
        "eligible_candidate_symbols_in_approved_universe": eligible_candidate_symbols_in_approved_universe,
        "approved_symbols_without_selected_candidate": approved_symbols_without_selected_candidate,
        "approved_symbols_without_eligible_candidate": approved_symbols_without_eligible_candidate,
        "selected_symbols_not_in_approved_universe": if approved_symbols.is_empty() { Vec::<String>::new() } else { difference_sorted(&candidate_symbol_set, &approved_symbol_set) },
        "candidate_symbol_coverage_of_approved_universe": coverage(candidate_symbols_in_approved_universe_len(latest_universe, rows), approved_symbols.len()),
        "eligible_candidate_symbol_coverage_of_approved_universe": coverage(eligible_candidate_symbols_in_approved_universe_len(latest_universe, driver, rows), approved_symbols.len())
    })
}

fn candidate_symbols_in_approved_universe_len(latest_universe: &Value, rows: &[Value]) -> usize {
    super::super::fields::candidate_symbols_in_approved_universe_len(latest_universe, rows)
}

fn eligible_candidate_symbols_in_approved_universe_len(
    latest_universe: &Value,
    driver: &Value,
    rows: &[Value],
) -> usize {
    super::super::fields::eligible_candidate_symbols_in_approved_universe_len(
        latest_universe,
        driver,
        rows,
    )
}
