use serde_json::{Value, json};

use super::super::decision::{
    rows_with_promote_bias_candidate_ids, rows_with_promote_bias_symbols,
};
use super::super::fields::{
    first_symbol, i64_field, string_array_pointer, string_field, unique_sorted_strings,
};

pub(in crate::retest_status) fn research_factory_progression(
    latest_universe: &Value,
    rows: &[Value],
    promotion_passed: bool,
    shadow_created: bool,
    paper_created: bool,
) -> Value {
    let candidate_symbols = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    );
    let candidate_ids = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "candidate_id")),
    );
    let research_replayed_symbols = unique_sorted_strings(rows.iter().filter_map(|row| {
        (i64_field(row, "replay_run_count").unwrap_or(0) > 0)
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }));
    let research_replayed_candidate_ids = unique_sorted_strings(rows.iter().filter_map(|row| {
        (i64_field(row, "replay_run_count").unwrap_or(0) > 0)
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }));
    let promotion_ready_symbols = unique_sorted_strings(rows.iter().filter_map(|row| {
        (string_field(row, "next_action") == Some("promotion_gate_ready_for_review"))
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }));
    let promotion_ready_candidate_ids = unique_sorted_strings(rows.iter().filter_map(|row| {
        (string_field(row, "next_action") == Some("promotion_gate_ready_for_review"))
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }));
    let promoted_symbols = if promotion_passed {
        promotion_ready_symbols.clone()
    } else {
        rows_with_promote_bias_symbols(rows)
    };
    let promoted_candidate_ids = if promotion_passed {
        promotion_ready_candidate_ids.clone()
    } else {
        rows_with_promote_bias_candidate_ids(rows)
    };
    json!({
        "major50_observed_symbol_count": latest_universe.get("observed_symbol_count").and_then(Value::as_u64).unwrap_or_else(|| string_array_pointer(latest_universe, "/observed_symbols").len() as u64),
        "major50_approved_symbol_count": latest_universe.get("approved_symbol_count").and_then(Value::as_u64).unwrap_or_else(|| string_array_pointer(latest_universe, "/approved_symbols").len() as u64),
        "candidate_generated_symbol_count": candidate_symbols.len(),
        "candidate_generated_candidate_count": candidate_ids.len(),
        "research_replayed_symbol_count": research_replayed_symbols.len(),
        "research_replayed_candidate_count": research_replayed_candidate_ids.len(),
        "promotion_ready_symbol_count": promotion_ready_symbols.len(),
        "promotion_ready_candidate_count": promotion_ready_candidate_ids.len(),
        "promoted_symbol_count": promoted_symbols.len(),
        "promoted_candidate_count": promoted_candidate_ids.len(),
        "shadow_created": shadow_created,
        "paper_created": paper_created,
        "live_enabled": false,
        "symbols": {
            "candidate_generated": candidate_symbols,
            "research_replayed": research_replayed_symbols,
            "promotion_ready": promotion_ready_symbols,
            "promoted": promoted_symbols
        },
        "candidates": {
            "candidate_generated": candidate_ids,
            "research_replayed": research_replayed_candidate_ids,
            "promotion_ready": promotion_ready_candidate_ids,
            "promoted": promoted_candidate_ids
        }
    })
}
