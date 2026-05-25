use crate::admission::horizon_ms;
use crate::error::{AppError, AppResult};
use crate::model::{
    IntelCandidateEvidenceBundle, RETEST_HORIZON_PLAN_SCHEMA_VERSION, ResearchPartitionAggregate,
    ResearchRunReport,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetestHorizonPlanBuildOptions {
    pub generated_at_ms: i64,
    pub manifest_label: String,
    pub report_label: String,
    pub latest_l1_as_of_ms: Option<i64>,
}

#[derive(Debug, Clone)]
struct HorizonPlanRow {
    candidate_id: String,
    candidate_lifecycle_key: String,
    symbols: Vec<String>,
    primary_symbol: Option<String>,
    hypothesis_type: String,
    research_priority: String,
    horizon: String,
    horizon_ms: Option<i64>,
    decision_available_at_ms: i64,
    forbidden_lookahead_boundary_ms: i64,
    horizon_due_ms: Option<i64>,
    latest_l1_as_of_ms: Option<i64>,
    horizon_market_data_materialized: Option<bool>,
    replay_run_count: usize,
    completed_count: usize,
    effective_completed_sample_weight: f64,
    completed_sample_deficit: usize,
    inferred_unseen_window_count: usize,
    required_unseen_window_count: usize,
    unseen_window_deficit: usize,
    train_validation_split_required: bool,
    train_validation_split_materialized: bool,
    liquidity_filter_required: bool,
    liquidity_filter_materialized_count: usize,
    missing_market_replay_data_count: usize,
    aggregate_count: usize,
    gate_biases: Vec<String>,
    reason_codes: Vec<String>,
    candidate_reason_codes: Vec<String>,
    next_action: String,
}

pub fn build_retest_horizon_plan(
    bundles: &[IntelCandidateEvidenceBundle],
    report: &ResearchRunReport,
    options: &RetestHorizonPlanBuildOptions,
) -> AppResult<Value> {
    if bundles.is_empty() {
        return Err(AppError::validation(
            "retest horizon plan requires at least one candidate bundle",
        ));
    }
    if report.research_gate_policy.min_completed_samples_for_shadow == 0 {
        return Err(AppError::validation(
            "research gate policy min_completed_samples_for_shadow must be greater than zero",
        ));
    }

    let rows = bundles
        .iter()
        .flat_map(|bundle| {
            bundle
                .allowed_horizons
                .iter()
                .map(|horizon| build_row(bundle, horizon, report, options.latest_l1_as_of_ms))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return Err(AppError::validation(
            "retest horizon plan requires at least one candidate horizon",
        ));
    }

    Ok(json!({
        "schema_version": RETEST_HORIZON_PLAN_SCHEMA_VERSION,
        "generated_at_ms": options.generated_at_ms,
        "manifest_file": options.manifest_label,
        "report_file": options.report_label,
        "latest_l1_as_of_ms": options.latest_l1_as_of_ms,
        "research_gate_policy": report.research_gate_policy,
        "summary": summary(&rows, bundles.len()),
        "by_candidate": by_candidate(&rows),
        "horizon_rows": rows.iter().map(row_json).collect::<Vec<_>>()
    }))
}

fn build_row(
    bundle: &IntelCandidateEvidenceBundle,
    horizon: &str,
    report: &ResearchRunReport,
    latest_l1_as_of_ms: Option<i64>,
) -> HorizonPlanRow {
    let horizon_ms = horizon_ms(horizon);
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

struct NextActionInput<'a> {
    horizon_ms: Option<i64>,
    horizon_due_ms: Option<i64>,
    latest_l1_as_of_ms: Option<i64>,
    matched_count: usize,
    reason_codes: &'a [String],
    completed_count: usize,
    min_completed: usize,
    inferred_unseen_window_count: usize,
    required_unseen_window_count: usize,
    train_validation_split_required: bool,
    train_validation_split_materialized: bool,
    liquidity_filter_required: bool,
    liquidity_filter_materialized_count: usize,
}

fn next_action(input: NextActionInput<'_>) -> String {
    if input.horizon_ms.is_none() {
        return "define_horizon_duration".to_owned();
    }
    let Some(horizon_due_ms) = input.horizon_due_ms else {
        return "define_replay_boundary".to_owned();
    };
    let Some(latest_l1_as_of_ms) = input.latest_l1_as_of_ms else {
        return "discover_latest_market_l1_as_of".to_owned();
    };
    if latest_l1_as_of_ms < horizon_due_ms {
        return "wait_for_market_l1_horizon".to_owned();
    }
    if input.matched_count == 0 {
        return "run_research_replay_for_horizon".to_owned();
    }
    if contains_reason(input.reason_codes, "missing_native_replay_market_data")
        || contains_reason(input.reason_codes, "native_replay_horizon_not_materialized")
    {
        return "extend_market_l1_horizon_coverage".to_owned();
    }
    if input.completed_count == 0 {
        return "materialize_completed_native_replay_sample".to_owned();
    }
    if input.completed_count < input.min_completed {
        return "accumulate_completed_native_replay_samples".to_owned();
    }
    if input.inferred_unseen_window_count < input.required_unseen_window_count {
        return "materialize_unseen_replay_windows".to_owned();
    }
    if input.train_validation_split_required && !input.train_validation_split_materialized {
        return "materialize_train_validation_split".to_owned();
    }
    if input.liquidity_filter_required
        && input.liquidity_filter_materialized_count < input.completed_count
    {
        return "materialize_liquidity_filter_inputs".to_owned();
    }
    if !input.reason_codes.is_empty() {
        return "inspect_remaining_gate_reasons".to_owned();
    }
    "promotion_gate_ready_for_review".to_owned()
}

fn row_json(row: &HorizonPlanRow) -> Value {
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

fn summary(rows: &[HorizonPlanRow], candidate_count: usize) -> Value {
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

fn by_candidate(rows: &[HorizonPlanRow]) -> Vec<Value> {
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

fn unique_sorted<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    values
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn contains_reason(reason_codes: &[String], expected: &str) -> bool {
    reason_codes.iter().any(|reason| reason == expected)
}
