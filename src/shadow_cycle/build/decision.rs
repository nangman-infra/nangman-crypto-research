use crate::hash::stable_id;
use crate::model::{
    SHADOW_CYCLE_DECISION_SCHEMA_VERSION, ShadowCycleDecision, ShadowCycleDecisionSafety,
    ShadowCycleSampleState, ShadowValidationRun,
};

use super::actions::{blocked_actions, safe_next_actions, select_scheduler_action};
use super::summary::{CandidateShadowState, summarize_shadow_runs};
use super::time::{iso8601_ms, min_optional_ms};

pub fn build_shadow_cycle_decision(
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
    generated_at_ms: i64,
) -> ShadowCycleDecision {
    let mut summary = summarize_shadow_runs(shadow_runs, latest_l1_as_of_ms);
    summary.run_identity_parts.sort_unstable();
    let candidate_lifecycle_count = summary.candidates.len();
    let target_waiting_count = summary
        .candidates
        .values()
        .filter(|state| state.target_materialized_count == 0 && state.observed_count > 0)
        .count();
    let partially_materialized_count = summary
        .candidates
        .values()
        .filter(|state| {
            state.target_materialized_count > 0
                && state.target_materialized_count < state.observed_count
        })
        .count();
    let pending_target_window_candidate_count = summary
        .candidates
        .values()
        .filter(|state| state.pending_target_count > 0)
        .count();
    let sample_ready_count = summary
        .candidates
        .values()
        .filter(|state| state.sample_requirement_met())
        .count();
    let deficient_count = summary
        .candidates
        .values()
        .filter(|state| state.sample_deficit() > 0)
        .count();
    let pending_count = summary
        .candidates
        .values()
        .filter(|state| state.pending_count > 0)
        .count();
    let total_sample_deficit = summary
        .candidates
        .values()
        .map(CandidateShadowState::sample_deficit)
        .sum();
    let next_observation_not_before_ms =
        summary.candidates.values().fold(None, |current, state| {
            min_optional_ms(current, state.next_pending_target_deadline_ms)
        });

    let (source_verdict, scheduler_action) = select_scheduler_action(
        shadow_runs.is_empty(),
        latest_l1_as_of_ms.is_some(),
        target_waiting_count,
        partially_materialized_count,
        deficient_count,
        pending_count,
        sample_ready_count,
    );

    let run_not_before_ms = scheduler_action
        .is_wait_action()
        .then_some(next_observation_not_before_ms)
        .flatten();

    ShadowCycleDecision {
        schema_version: SHADOW_CYCLE_DECISION_SCHEMA_VERSION.to_owned(),
        generated_at: iso8601_ms(generated_at_ms),
        decision_id: stable_id(
            "shadow_cycle_decision",
            &[
                source_verdict,
                &latest_l1_as_of_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                &generated_at_ms.to_string(),
                &summary.run_identity_parts.join("|"),
            ],
        ),
        source_cycle_summary_file: None,
        run_dir: None,
        scheduler_action,
        source_verdict: source_verdict.to_owned(),
        run_not_before_ms,
        run_not_before_at: run_not_before_ms.map(iso8601_ms),
        run_not_before_source: run_not_before_ms
            .map(|_| "pending_shadow_target_exit_deadline_ms".to_owned()),
        focused_research_manifest_file: None,
        focused_research_summary_file: None,
        latest_l1_as_of_ms,
        shadow_sample_state: ShadowCycleSampleState {
            shadow_validation_count: shadow_runs.len(),
            target_window_materialized_count: summary.target_materialized_count,
            candidate_lifecycle_count,
            partially_materialized_candidate_count: partially_materialized_count,
            pending_target_window_candidate_count,
            total_sample_deficit,
            symbols: summary.symbols.into_iter().collect(),
        },
        safe_next_actions: safe_next_actions(source_verdict),
        blocked_actions: blocked_actions(source_verdict),
        safety: ShadowCycleDecisionSafety {
            s3_write: false,
            ecs_task_started: false,
            dispatcher_mode_changed: false,
            local_decision_only: true,
            shadow_status_mutated: false,
            paper_live_enabled: false,
            live_enabled: false,
            order_execution_enabled: false,
        },
    }
}

pub fn shadow_sample_deficit_lifecycle_keys(
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
) -> Vec<String> {
    summarize_shadow_runs(shadow_runs, latest_l1_as_of_ms)
        .candidates
        .into_iter()
        .filter_map(|(candidate_lifecycle_key, state)| {
            (state.observed_count > 0
                && state.pending_target_count == 0
                && state.sample_deficit() > 0)
                .then_some(candidate_lifecycle_key)
        })
        .collect()
}
