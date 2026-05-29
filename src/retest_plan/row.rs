use super::action::{NextActionInput, next_action};
use super::types::HorizonPlanRow;
use crate::admission::horizon_ms as resolve_horizon_ms;
use crate::model::{IntelCandidateEvidenceBundle, ResearchPartitionAggregate, ResearchRunReport};
use std::collections::BTreeSet;

pub(super) fn build_row(
    bundle: &IntelCandidateEvidenceBundle,
    horizon: &str,
    report: &ResearchRunReport,
    latest_l1_as_of_ms: Option<i64>,
) -> HorizonPlanRow {
    let horizon_ms = resolve_horizon_ms(horizon);
    let boundary_ms = bundle.forbidden_lookahead_boundary_ms;
    let horizon_due_ms = horizon_ms.map(|duration_ms| boundary_ms + duration_ms);
    let matched = report
        .partition_aggregates
        .iter()
        .filter(|aggregate| {
            aggregate
                .source_candidate_ids
                .iter()
                .any(|candidate_id| candidate_id == &bundle.candidate_id)
                && horizon_from_aggregate_key(&aggregate.research_aggregate_key) == horizon
        })
        .collect::<Vec<_>>();
    let min_completed = report.research_gate_policy.min_completed_samples_for_shadow;
    let completed_count = matched
        .iter()
        .map(|aggregate| aggregate.completed_count)
        .max()
        .unwrap_or(0);
    let effective_completed_sample_weight = matched
        .iter()
        .map(|aggregate| aggregate.effective_completed_sample_weight)
        .fold(0.0_f64, f64::max);
    let replay_run_count = matched
        .iter()
        .map(|aggregate| aggregate.replay_run_count)
        .sum();
    let inferred_unseen_window_count = matched
        .iter()
        .map(|aggregate| aggregate.inferred_unseen_window_count)
        .max()
        .unwrap_or(0);
    let required_unseen_window_count = bundle.validation_requirements.min_unseen_windows;
    let train_validation_split_materialized = matched
        .iter()
        .any(|aggregate| aggregate.train_validation_split_summary.materialized);
    let liquidity_filter_materialized_count = matched
        .iter()
        .map(|aggregate| aggregate.liquidity_filter_materialized_count)
        .max()
        .unwrap_or(0);
    let missing_market_replay_data_count = matched
        .iter()
        .map(|aggregate| aggregate.missing_market_replay_data_count)
        .sum();
    let reason_codes = aggregate_reason_codes(&matched);
    let candidate_reason_codes = candidate_reason_codes(report, &bundle.candidate_id);
    let horizon_market_data_materialized = match (latest_l1_as_of_ms, horizon_due_ms) {
        (Some(latest_l1), Some(due_ms)) => Some(latest_l1 >= due_ms),
        _ => None,
    };
    let next_action = next_action(NextActionInput {
        horizon_ms,
        horizon_due_ms,
        latest_l1_as_of_ms,
        matched_count: matched.len(),
        reason_codes: &reason_codes,
        completed_count,
        min_completed,
        inferred_unseen_window_count,
        required_unseen_window_count,
        train_validation_split_required: bundle
            .validation_requirements
            .required_train_validation_split,
        train_validation_split_materialized,
        liquidity_filter_required: bundle.validation_requirements.include_liquidity_filter,
        liquidity_filter_materialized_count,
    });

    HorizonPlanRow {
        candidate_id: bundle.candidate_id.clone(),
        candidate_lifecycle_key: bundle.candidate_lifecycle_key.clone(),
        symbols: bundle.normalized_symbols.clone(),
        primary_symbol: bundle.normalized_symbols.first().cloned(),
        hypothesis_type: bundle.hypothesis_type.clone(),
        research_priority: bundle.research_priority.clone(),
        horizon: horizon.to_owned(),
        horizon_ms,
        decision_available_at_ms: bundle.decision_available_at_ms,
        forbidden_lookahead_boundary_ms: boundary_ms,
        horizon_due_ms,
        latest_l1_as_of_ms,
        horizon_market_data_materialized,
        replay_run_count,
        completed_count,
        effective_completed_sample_weight,
        completed_sample_deficit: min_completed.saturating_sub(completed_count),
        inferred_unseen_window_count,
        required_unseen_window_count,
        unseen_window_deficit: required_unseen_window_count
            .saturating_sub(inferred_unseen_window_count),
        train_validation_split_required: bundle
            .validation_requirements
            .required_train_validation_split,
        train_validation_split_materialized,
        liquidity_filter_required: bundle.validation_requirements.include_liquidity_filter,
        liquidity_filter_materialized_count,
        missing_market_replay_data_count,
        aggregate_count: matched.len(),
        gate_biases: gate_biases(&matched),
        reason_codes,
        candidate_reason_codes,
        next_action,
    }
}

fn horizon_from_aggregate_key(key: &str) -> &str {
    key.split(':').nth(3).unwrap_or("unknown")
}

fn aggregate_reason_codes(aggregates: &[&ResearchPartitionAggregate]) -> Vec<String> {
    let mut reason_codes = BTreeSet::<String>::new();
    for aggregate in aggregates {
        reason_codes.extend(aggregate.gate_reason_codes.iter().cloned());
        if aggregate.missing_market_replay_data_count > 0 {
            reason_codes.insert("missing_native_replay_market_data".to_owned());
        }
    }
    reason_codes.into_iter().collect()
}

fn candidate_reason_codes(report: &ResearchRunReport, candidate_id: &str) -> Vec<String> {
    let mut reason_codes = BTreeSet::<String>::new();
    for finding in &report.summary_findings {
        if finding.candidate_id == candidate_id {
            reason_codes.extend(finding.reason_codes.iter().cloned());
        }
    }
    reason_codes.into_iter().collect()
}

fn gate_biases(aggregates: &[&ResearchPartitionAggregate]) -> Vec<String> {
    let mut values = BTreeSet::<String>::new();
    for aggregate in aggregates {
        values.insert(
            serde_json::to_value(&aggregate.gate_bias)
                .ok()
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| "unknown".to_owned()),
        );
    }
    values.into_iter().collect()
}
