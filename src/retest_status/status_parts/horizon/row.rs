use serde_json::{Value, json};

use crate::retest_status::status_parts::fields::{
    bool_field, first_symbol, i64_field, string_array_field, string_field,
};

pub(super) fn candidate_horizon_state(rows: &[Value], horizon: &str) -> Value {
    let row = rows
        .iter()
        .find(|row| string_field(row, "horizon") == Some(horizon));
    if let Some(row) = row {
        let next_action = string_field(row, "next_action").unwrap_or("unknown");
        return json!({
            "horizon": horizon,
            "requested": true,
            "next_action": next_action,
            "horizon_market_data_materialized": bool_field(row, "horizon_market_data_materialized").unwrap_or(false),
            "replay_run_count": i64_field(row, "replay_run_count").unwrap_or(0),
            "completed_count": i64_field(row, "completed_count").unwrap_or(0),
            "completed_sample_deficit": row.get("completed_sample_deficit").cloned().unwrap_or(Value::Null),
            "inferred_unseen_window_count": i64_field(row, "inferred_unseen_window_count").unwrap_or(0),
            "unseen_window_deficit": row.get("unseen_window_deficit").cloned().unwrap_or(Value::Null),
            "train_validation_split_materialized": bool_field(row, "train_validation_split_materialized").unwrap_or(false),
            "liquidity_filter_materialized_count": i64_field(row, "liquidity_filter_materialized_count").unwrap_or(0),
            "missing_market_replay_data_count": i64_field(row, "missing_market_replay_data_count").unwrap_or(0),
            "gate_biases": string_array_field(row, "gate_biases"),
            "reason_codes": string_array_field(row, "reason_codes"),
            "promotion_gate_ready_for_review": next_action == "promotion_gate_ready_for_review"
        });
    }
    json!({
        "horizon": horizon,
        "requested": false,
        "next_action": "not_requested",
        "horizon_market_data_materialized": false,
        "replay_run_count": 0,
        "completed_count": 0,
        "completed_sample_deficit": Value::Null,
        "inferred_unseen_window_count": 0,
        "unseen_window_deficit": Value::Null,
        "train_validation_split_materialized": false,
        "liquidity_filter_materialized_count": 0,
        "missing_market_replay_data_count": 0,
        "gate_biases": [],
        "reason_codes": ["horizon_not_requested_by_candidate_bundle"],
        "promotion_gate_ready_for_review": false
    })
}

pub(super) fn compact_horizon_row(row: &Value) -> Value {
    json!({
        "candidate_id": string_field(row, "candidate_id"),
        "candidate_lifecycle_key": string_field(row, "candidate_lifecycle_key"),
        "primary_symbol": string_field(row, "primary_symbol").or_else(|| first_symbol(row)),
        "symbols": string_array_field(row, "symbols"),
        "hypothesis_type": string_field(row, "hypothesis_type"),
        "research_priority": string_field(row, "research_priority"),
        "horizon": string_field(row, "horizon"),
        "horizon_market_data_materialized": bool_field(row, "horizon_market_data_materialized").unwrap_or(false),
        "replay_run_count": i64_field(row, "replay_run_count").unwrap_or(0),
        "completed_count": i64_field(row, "completed_count").unwrap_or(0),
        "completed_sample_deficit": row.get("completed_sample_deficit").cloned().unwrap_or(Value::Null),
        "inferred_unseen_window_count": i64_field(row, "inferred_unseen_window_count").unwrap_or(0),
        "unseen_window_deficit": row.get("unseen_window_deficit").cloned().unwrap_or(Value::Null),
        "train_validation_split_required": bool_field(row, "train_validation_split_required").unwrap_or(false),
        "train_validation_split_materialized": bool_field(row, "train_validation_split_materialized").unwrap_or(false),
        "liquidity_filter_required": bool_field(row, "liquidity_filter_required").unwrap_or(false),
        "liquidity_filter_materialized_count": i64_field(row, "liquidity_filter_materialized_count").unwrap_or(0),
        "missing_market_replay_data_count": i64_field(row, "missing_market_replay_data_count").unwrap_or(0),
        "gate_biases": string_array_field(row, "gate_biases"),
        "reason_codes": string_array_field(row, "reason_codes"),
        "next_action": string_field(row, "next_action")
    })
}
