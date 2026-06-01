use super::aggregate::{aggregate_metrics, matching_aggregates};
use super::reasons::{aggregate_reason_codes, candidate_reason_codes, gate_biases};
use crate::admission::horizon_ms as resolve_horizon_ms;
use crate::model::{IntelCandidateEvidenceBundle, ResearchRunReport};
use crate::retest_plan::action::{NextActionInput, next_action};
use crate::retest_plan::types::HorizonPlanRow;

pub(in crate::retest_plan) fn build_row(
    bundle: &IntelCandidateEvidenceBundle,
    horizon: &str,
    report: &ResearchRunReport,
    latest_l1_as_of_ms: Option<i64>,
) -> HorizonPlanRow {
    let horizon_ms = resolve_horizon_ms(horizon);
    let boundary_ms = bundle.forbidden_lookahead_boundary_ms;
    let horizon_due_ms = horizon_ms.map(|duration_ms| boundary_ms + duration_ms);
    let matched = matching_aggregates(bundle, horizon, report);
    let metrics = aggregate_metrics(&matched);
    let min_completed = report.research_gate_policy.min_completed_samples_for_shadow;
    let required_unseen_window_count = bundle.validation_requirements.min_unseen_windows;
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
        completed_count: metrics.completed_count,
        min_completed,
        inferred_unseen_window_count: metrics.inferred_unseen_window_count,
        required_unseen_window_count,
        train_validation_split_required: bundle
            .validation_requirements
            .required_train_validation_split,
        train_validation_split_materialized: metrics.train_validation_split_materialized,
        liquidity_filter_required: bundle.validation_requirements.include_liquidity_filter,
        liquidity_filter_materialized_count: metrics.liquidity_filter_materialized_count,
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
        replay_run_count: metrics.replay_run_count,
        completed_count: metrics.completed_count,
        effective_completed_sample_weight: metrics.effective_completed_sample_weight,
        completed_sample_deficit: min_completed.saturating_sub(metrics.completed_count),
        inferred_unseen_window_count: metrics.inferred_unseen_window_count,
        required_unseen_window_count,
        unseen_window_deficit: required_unseen_window_count
            .saturating_sub(metrics.inferred_unseen_window_count),
        train_validation_split_required: bundle
            .validation_requirements
            .required_train_validation_split,
        train_validation_split_materialized: metrics.train_validation_split_materialized,
        liquidity_filter_required: bundle.validation_requirements.include_liquidity_filter,
        liquidity_filter_materialized_count: metrics.liquidity_filter_materialized_count,
        missing_market_replay_data_count: metrics.missing_market_replay_data_count,
        aggregate_count: matched.len(),
        gate_biases: gate_biases(&matched),
        reason_codes,
        candidate_reason_codes,
        next_action,
    }
}
