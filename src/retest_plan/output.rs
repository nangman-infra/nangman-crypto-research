use super::types::HorizonPlanRow;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn row_json(row: &HorizonPlanRow) -> Value {
    json!({
        "candidate_id": row.candidate_id,
        "candidate_lifecycle_key": row.candidate_lifecycle_key,
        "symbols": row.symbols,
        "primary_symbol": row.primary_symbol,
        "hypothesis_type": row.hypothesis_type,
        "research_priority": row.research_priority,
        "horizon": row.horizon,
        "horizon_ms": row.horizon_ms,
        "decision_available_at_ms": row.decision_available_at_ms,
        "forbidden_lookahead_boundary_ms": row.forbidden_lookahead_boundary_ms,
        "horizon_due_ms": row.horizon_due_ms,
        "latest_l1_as_of_ms": row.latest_l1_as_of_ms,
        "horizon_market_data_materialized": row.horizon_market_data_materialized,
        "replay_run_count": row.replay_run_count,
        "completed_count": row.completed_count,
        "effective_completed_sample_weight": row.effective_completed_sample_weight,
        "completed_sample_deficit": row.completed_sample_deficit,
        "inferred_unseen_window_count": row.inferred_unseen_window_count,
        "required_unseen_window_count": row.required_unseen_window_count,
        "unseen_window_deficit": row.unseen_window_deficit,
        "train_validation_split_required": row.train_validation_split_required,
        "train_validation_split_materialized": row.train_validation_split_materialized,
        "liquidity_filter_required": row.liquidity_filter_required,
        "liquidity_filter_materialized_count": row.liquidity_filter_materialized_count,
        "missing_market_replay_data_count": row.missing_market_replay_data_count,
        "aggregate_count": row.aggregate_count,
        "gate_biases": row.gate_biases,
        "reason_codes": row.reason_codes,
        "candidate_reason_codes": row.candidate_reason_codes,
        "next_action": row.next_action
    })
}

pub(super) fn summary(rows: &[HorizonPlanRow], candidate_count: usize) -> Value {
    json!({
        "candidate_count": candidate_count,
        "horizon_count": rows.len(),
        "symbols": unique_sorted(rows.iter().filter_map(|row| row.primary_symbol.as_deref())),
        "next_action_counts": next_action_counts(rows),
        "ready_for_replay_count": rows.iter().filter(|row| {
            row.next_action == "run_research_replay_for_horizon"
                || row.next_action == "materialize_completed_native_replay_sample"
        }).count(),
        "waiting_for_market_l1_count": rows.iter().filter(|row| row.next_action == "wait_for_market_l1_horizon").count(),
        "market_l1_coverage_extension_count": rows.iter().filter(|row| row.next_action == "extend_market_l1_horizon_coverage").count(),
        "sample_accumulation_count": rows.iter().filter(|row| row.next_action == "accumulate_completed_native_replay_samples").count(),
        "promotion_ready_for_review_count": rows.iter().filter(|row| row.next_action == "promotion_gate_ready_for_review").count()
    })
}

pub(super) fn by_candidate(rows: &[HorizonPlanRow]) -> Vec<Value> {
    let mut grouped = BTreeMap::<String, Vec<&HorizonPlanRow>>::new();
    for row in rows {
        grouped
            .entry(row.candidate_id.clone())
            .or_default()
            .push(row);
    }
    grouped
        .into_values()
        .map(|candidate_rows| {
            let first = candidate_rows[0];
            json!({
                "candidate_id": first.candidate_id,
                "symbols": first.symbols,
                "horizons": candidate_rows
                    .iter()
                    .map(|row| {
                        json!({
                            "horizon": row.horizon,
                            "horizon_due_ms": row.horizon_due_ms,
                            "horizon_market_data_materialized": row.horizon_market_data_materialized,
                            "replay_run_count": row.replay_run_count,
                            "completed_count": row.completed_count,
                            "completed_sample_deficit": row.completed_sample_deficit,
                            "inferred_unseen_window_count": row.inferred_unseen_window_count,
                            "unseen_window_deficit": row.unseen_window_deficit,
                            "next_action": row.next_action,
                            "reason_codes": row.reason_codes
                        })
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect()
}

fn next_action_counts(rows: &[HorizonPlanRow]) -> Vec<Value> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        *counts.entry(row.next_action.clone()).or_default() += 1;
    }
    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| right.0.cmp(&left.0)));
    counts
        .into_iter()
        .map(|(next_action, count)| json!({ "next_action": next_action, "count": count }))
        .collect()
}

fn unique_sorted<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    values
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
