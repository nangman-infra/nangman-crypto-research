use serde_json::{Value, json};
use std::collections::BTreeSet;

use super::super::decision::{
    rows_with_promote_bias_candidate_ids, rows_with_promote_bias_symbols,
};
use super::super::fields::{
    difference_sorted, first_symbol, i64_field, string_array_pointer, string_field,
    unique_sorted_strings,
};

pub(in crate::retest_status) fn coverage_gaps(
    latest_universe: &Value,
    driver: &Value,
    rows: &[Value],
    shadow_created: bool,
) -> Value {
    let approved_symbols = string_array_pointer(latest_universe, "/approved_symbols")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let candidate_symbols = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    let eligible_symbols = {
        let symbols = string_array_pointer(driver, "/manifest/eligible_candidate_symbols");
        if symbols.is_empty() {
            candidate_symbols.clone()
        } else {
            symbols.into_iter().collect()
        }
    };
    let candidate_ids = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "candidate_id")),
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    let research_replayed_candidate_ids = unique_sorted_strings(rows.iter().filter_map(|row| {
        (i64_field(row, "replay_run_count").unwrap_or(0) > 0)
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }))
    .into_iter()
    .collect::<BTreeSet<_>>();
    let research_replayed_symbols = unique_sorted_strings(rows.iter().filter_map(|row| {
        (i64_field(row, "replay_run_count").unwrap_or(0) > 0)
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }))
    .into_iter()
    .collect::<BTreeSet<_>>();
    let promotion_ready_candidate_ids = unique_sorted_strings(rows.iter().filter_map(|row| {
        (string_field(row, "next_action") == Some("promotion_gate_ready_for_review"))
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }))
    .into_iter()
    .collect::<BTreeSet<_>>();
    let promotion_ready_symbols = unique_sorted_strings(rows.iter().filter_map(|row| {
        (string_field(row, "next_action") == Some("promotion_gate_ready_for_review"))
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }))
    .into_iter()
    .collect::<BTreeSet<_>>();
    let promoted_candidate_ids = rows_with_promote_bias_candidate_ids(rows)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let promoted_symbols = rows_with_promote_bias_symbols(rows)
        .into_iter()
        .collect::<BTreeSet<_>>();
    json!({
        "approved_symbols_without_candidate": difference_sorted(&approved_symbols, &eligible_symbols),
        "approved_symbols_without_selected_candidate": difference_sorted(&approved_symbols, &candidate_symbols),
        "approved_symbols_without_eligible_candidate": difference_sorted(&approved_symbols, &eligible_symbols),
        "unselected_eligible_candidate_symbols": string_array_pointer(driver, "/manifest/unselected_eligible_candidate_symbols"),
        "candidate_symbols_without_replay": difference_sorted(&candidate_symbols, &research_replayed_symbols),
        "candidate_ids_without_replay": difference_sorted(&candidate_ids, &research_replayed_candidate_ids),
        "replayed_symbols_without_promotion_ready": difference_sorted(&research_replayed_symbols, &promotion_ready_symbols),
        "replayed_symbols_without_promotion": difference_sorted(&research_replayed_symbols, &promoted_symbols),
        "replayed_candidate_ids_without_promotion_ready": difference_sorted(&research_replayed_candidate_ids, &promotion_ready_candidate_ids),
        "replayed_candidate_ids_without_promotion": difference_sorted(&research_replayed_candidate_ids, &promoted_candidate_ids),
        "promotion_ready_symbols_without_shadow": if shadow_created { Vec::<String>::new() } else { promotion_ready_symbols.iter().cloned().collect::<Vec<_>>() },
        "promotion_ready_candidate_ids_without_shadow": if shadow_created { Vec::<String>::new() } else { promotion_ready_candidate_ids.iter().cloned().collect::<Vec<_>>() },
        "promoted_symbols_without_shadow": if shadow_created { Vec::<String>::new() } else { promoted_symbols.iter().cloned().collect::<Vec<_>>() },
        "promoted_candidate_ids_without_shadow": if shadow_created { Vec::<String>::new() } else { promoted_candidate_ids.iter().cloned().collect::<Vec<_>>() }
    })
}
