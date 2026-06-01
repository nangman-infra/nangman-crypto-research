use std::collections::BTreeMap;

use super::super::summary::CandidateShadowState;
use super::super::time::min_optional_ms;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CandidateShadowCounts {
    pub(super) candidate_lifecycle_count: usize,
    pub(super) target_waiting_count: usize,
    pub(super) partially_materialized_count: usize,
    pub(super) pending_target_window_candidate_count: usize,
    pub(super) sample_ready_count: usize,
    pub(super) deficient_count: usize,
    pub(super) pending_count: usize,
    pub(super) total_sample_deficit: i64,
    pub(super) next_observation_not_before_ms: Option<i64>,
}

pub(super) fn count_candidate_shadow_states(
    candidates: &BTreeMap<String, CandidateShadowState>,
) -> CandidateShadowCounts {
    let mut counts = CandidateShadowCounts {
        candidate_lifecycle_count: candidates.len(),
        target_waiting_count: 0,
        partially_materialized_count: 0,
        pending_target_window_candidate_count: 0,
        sample_ready_count: 0,
        deficient_count: 0,
        pending_count: 0,
        total_sample_deficit: 0,
        next_observation_not_before_ms: None,
    };

    for state in candidates.values() {
        count_candidate_shadow_state(&mut counts, state);
    }

    counts
}

fn count_candidate_shadow_state(counts: &mut CandidateShadowCounts, state: &CandidateShadowState) {
    if state.target_materialized_count == 0 && state.observed_count > 0 {
        counts.target_waiting_count += 1;
    }
    if state.target_materialized_count > 0 && state.target_materialized_count < state.observed_count
    {
        counts.partially_materialized_count += 1;
    }
    if state.pending_target_count > 0 {
        counts.pending_target_window_candidate_count += 1;
    }
    if state.sample_requirement_met() {
        counts.sample_ready_count += 1;
    }
    if state.sample_deficit() > 0 {
        counts.deficient_count += 1;
    }
    if state.pending_count > 0 {
        counts.pending_count += 1;
    }
    counts.total_sample_deficit += state.sample_deficit();
    counts.next_observation_not_before_ms = min_optional_ms(
        counts.next_observation_not_before_ms,
        state.next_pending_target_deadline_ms,
    );
}
