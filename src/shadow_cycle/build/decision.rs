mod identity;
mod safety;
mod sample_state;
mod summary_counts;

use crate::model::{
    SHADOW_CYCLE_DECISION_SCHEMA_VERSION, ShadowCycleDecision, ShadowValidationRun,
};

use super::actions::{blocked_actions, safe_next_actions, select_scheduler_action};
use super::summary::summarize_shadow_runs;
use super::time::iso8601_ms;
use identity::shadow_cycle_decision_id;
use safety::local_shadow_cycle_decision_safety;
use sample_state::build_shadow_sample_state;
use summary_counts::count_candidate_shadow_states;

pub fn build_shadow_cycle_decision(
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
    generated_at_ms: i64,
) -> ShadowCycleDecision {
    let mut summary = summarize_shadow_runs(shadow_runs, latest_l1_as_of_ms);
    summary.run_identity_parts.sort_unstable();
    let counts = count_candidate_shadow_states(&summary.candidates);

    let (source_verdict, scheduler_action) = select_scheduler_action(
        shadow_runs.is_empty(),
        latest_l1_as_of_ms.is_some(),
        counts.target_waiting_count,
        counts.partially_materialized_count,
        counts.deficient_count,
        counts.pending_count,
        counts.sample_ready_count,
    );

    let run_not_before_ms = scheduler_action
        .is_wait_action()
        .then_some(counts.next_observation_not_before_ms)
        .flatten();

    ShadowCycleDecision {
        schema_version: SHADOW_CYCLE_DECISION_SCHEMA_VERSION.to_owned(),
        generated_at: iso8601_ms(generated_at_ms),
        decision_id: shadow_cycle_decision_id(
            source_verdict,
            latest_l1_as_of_ms,
            generated_at_ms,
            &summary.run_identity_parts,
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
        shadow_sample_state: build_shadow_sample_state(
            shadow_runs.len(),
            summary.target_materialized_count,
            summary.symbols,
            &counts,
        ),
        safe_next_actions: safe_next_actions(source_verdict),
        blocked_actions: blocked_actions(source_verdict),
        safety: local_shadow_cycle_decision_safety(),
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
