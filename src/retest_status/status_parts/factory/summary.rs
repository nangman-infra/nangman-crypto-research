use serde_json::{Value, json};

use super::super::decision::{
    rows_with_promote_bias_candidate_ids, rows_with_promote_bias_symbols,
};
use super::super::fields::{
    first_symbol, i64_field, string_array_pointer, string_field, unique_sorted_strings,
};
use super::gaps::coverage_gaps;

pub(in crate::retest_status) fn research_factory_gap_summary(
    latest_universe: &Value,
    driver: &Value,
    rows: &[Value],
    promotion_passed: bool,
    shadow_created: bool,
    paper_created: bool,
    safe_next_actions: &[String],
) -> Value {
    let gaps = coverage_gaps(latest_universe, driver, rows, shadow_created);
    let blocking_stage = if gaps
        .get("approved_symbols_without_eligible_candidate")
        .and_then(Value::as_array)
        .map(|values| !values.is_empty())
        .unwrap_or(false)
    {
        "candidate_generation_coverage"
    } else if gaps
        .get("candidate_ids_without_replay")
        .and_then(Value::as_array)
        .map(|values| !values.is_empty())
        .unwrap_or(false)
    {
        "research_replay_coverage"
    } else if gaps
        .get("promotion_ready_symbols_without_shadow")
        .and_then(Value::as_array)
        .map(|values| !values.is_empty())
        .unwrap_or(false)
    {
        "shadow_review_gate"
    } else if !promotion_passed {
        "promotion_evidence"
    } else if !paper_created {
        "paper_validation_gate"
    } else {
        "human_live_approval_boundary"
    };
    json!({
        "blocking_stage": blocking_stage,
        "stage_counts": {
            "major50_observed": latest_universe.get("observed_symbol_count").and_then(Value::as_u64).unwrap_or_else(|| string_array_pointer(latest_universe, "/observed_symbols").len() as u64),
            "major50_approved": latest_universe.get("approved_symbol_count").and_then(Value::as_u64).unwrap_or_else(|| string_array_pointer(latest_universe, "/approved_symbols").len() as u64),
            "candidate_generated": unique_sorted_strings(rows.iter().filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))).len(),
            "candidate_generated_candidates": unique_sorted_strings(rows.iter().filter_map(|row| string_field(row, "candidate_id"))).len(),
            "research_replayed": unique_sorted_strings(rows.iter().filter_map(|row| (i64_field(row, "replay_run_count").unwrap_or(0) > 0).then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row))).flatten())).len(),
            "research_replayed_candidates": unique_sorted_strings(rows.iter().filter_map(|row| (i64_field(row, "replay_run_count").unwrap_or(0) > 0).then(|| string_field(row, "candidate_id")).flatten())).len(),
            "promotion_ready": unique_sorted_strings(rows.iter().filter_map(|row| (string_field(row, "next_action") == Some("promotion_gate_ready_for_review")).then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row))).flatten())).len(),
            "promotion_ready_candidates": unique_sorted_strings(rows.iter().filter_map(|row| (string_field(row, "next_action") == Some("promotion_gate_ready_for_review")).then(|| string_field(row, "candidate_id")).flatten())).len(),
            "promoted": rows_with_promote_bias_symbols(rows).len(),
            "promoted_candidates": rows_with_promote_bias_candidate_ids(rows).len()
        },
        "gap_counts": {
            "approved_symbols_without_candidate": gaps.get("approved_symbols_without_candidate").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "approved_symbols_without_selected_candidate": gaps.get("approved_symbols_without_selected_candidate").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "approved_symbols_without_eligible_candidate": gaps.get("approved_symbols_without_eligible_candidate").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "unselected_eligible_candidate_symbols": gaps.get("unselected_eligible_candidate_symbols").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "candidate_symbols_without_replay": gaps.get("candidate_symbols_without_replay").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "candidate_ids_without_replay": gaps.get("candidate_ids_without_replay").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "replayed_symbols_without_promotion_ready": gaps.get("replayed_symbols_without_promotion_ready").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "replayed_symbols_without_promotion": gaps.get("replayed_symbols_without_promotion").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "replayed_candidate_ids_without_promotion_ready": gaps.get("replayed_candidate_ids_without_promotion_ready").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "replayed_candidate_ids_without_promotion": gaps.get("replayed_candidate_ids_without_promotion").and_then(Value::as_array).map(Vec::len).unwrap_or(0)
        },
        "safe_next_actions": safe_next_actions,
        "shadow_created": shadow_created,
        "paper_created": paper_created,
        "live_enabled": false
    })
}
