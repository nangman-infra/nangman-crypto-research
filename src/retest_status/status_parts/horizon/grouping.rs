use serde_json::{Value, json};
use std::collections::BTreeMap;

use super::row::compact_horizon_row;
use crate::retest_status::status_parts::decision::{action_counts, count_action, count_actions};
use crate::retest_status::status_parts::fields::{
    first_symbol, horizon_rank, i64_field, string_array_field, string_field, unique_sorted_strings,
};

pub(in crate::retest_status) fn by_symbol(rows: &[Value]) -> Vec<Value> {
    let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in rows {
        let symbol = string_field(row, "primary_symbol")
            .or_else(|| first_symbol(row))
            .unwrap_or("UNKNOWN")
            .to_owned();
        groups.entry(symbol).or_default().push(row.clone());
    }
    groups
        .into_iter()
        .map(|(symbol, mut rows)| {
            rows.sort_by_key(|row| {
                (
                    string_field(row, "candidate_id").unwrap_or("").to_owned(),
                    horizon_rank(string_field(row, "horizon").unwrap_or("")),
                )
            });
            let candidate_ids =
                unique_sorted_strings(rows.iter().filter_map(|row| string_field(row, "candidate_id")));
            let candidates = candidate_ids
                .iter()
                .map(|candidate_id| {
                    let candidate_rows = rows
                        .iter()
                        .filter(|row| string_field(row, "candidate_id") == Some(candidate_id))
                        .cloned()
                        .collect::<Vec<_>>();
                    let first = candidate_rows.first().unwrap_or(&Value::Null);
                    json!({
                        "candidate_id": candidate_id,
                        "candidate_lifecycle_key": string_field(first, "candidate_lifecycle_key"),
                        "symbols": string_array_field(first, "symbols"),
                        "hypothesis_type": string_field(first, "hypothesis_type"),
                        "research_priority": string_field(first, "research_priority"),
                        "horizons": candidate_rows.iter().map(compact_horizon_row).collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "symbol": symbol,
                "candidate_count": candidate_ids.len(),
                "horizon_count": rows.len(),
                "horizons": horizon_counts(&rows),
                "next_action_counts": action_counts(&rows),
                "ready_for_replay_count": count_actions(&rows, &["run_research_replay_for_horizon", "materialize_completed_native_replay_sample"]),
                "waiting_for_market_l1_count": count_action(&rows, "wait_for_market_l1_horizon"),
                "market_l1_coverage_extension_count": count_action(&rows, "extend_market_l1_horizon_coverage"),
                "sample_accumulation_count": count_action(&rows, "accumulate_completed_native_replay_samples"),
                "promotion_ready_for_review_count": count_action(&rows, "promotion_gate_ready_for_review"),
                "candidates": candidates
            })
        })
        .collect()
}

pub(in crate::retest_status) fn by_horizon(rows: &[Value]) -> Vec<Value> {
    horizon_counts(rows)
}

fn horizon_counts(rows: &[Value]) -> Vec<Value> {
    let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in rows {
        if let Some(horizon) = string_field(row, "horizon") {
            groups
                .entry(horizon.to_owned())
                .or_default()
                .push(row.clone());
        }
    }
    let mut counts = groups
        .into_iter()
        .map(|(horizon, rows)| {
            json!({
                "horizon": horizon,
                "horizon_count": rows.len(),
                "candidate_count": unique_sorted_strings(rows.iter().filter_map(|row| string_field(row, "candidate_id"))).len(),
                "next_action_counts": action_counts(&rows),
                "waiting_for_market_l1_count": count_action(&rows, "wait_for_market_l1_horizon"),
                "market_l1_coverage_extension_count": count_action(&rows, "extend_market_l1_horizon_coverage"),
                "ready_for_replay_count": count_actions(&rows, &["run_research_replay_for_horizon", "materialize_completed_native_replay_sample"]),
                "sample_accumulation_count": count_action(&rows, "accumulate_completed_native_replay_samples"),
                "promotion_ready_for_review_count": count_action(&rows, "promotion_gate_ready_for_review"),
                "max_completed_sample_deficit": rows.iter().filter_map(|row| i64_field(row, "completed_sample_deficit")).max().unwrap_or(0),
                "max_unseen_window_deficit": rows.iter().filter_map(|row| i64_field(row, "unseen_window_deficit")).max().unwrap_or(0)
            })
        })
        .collect::<Vec<_>>();
    counts.sort_by_key(|value| {
        horizon_rank(value.get("horizon").and_then(Value::as_str).unwrap_or(""))
    });
    counts
}
