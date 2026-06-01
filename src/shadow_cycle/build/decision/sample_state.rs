use std::collections::BTreeSet;

use crate::model::ShadowCycleSampleState;

use super::summary_counts::CandidateShadowCounts;

pub(super) fn build_shadow_sample_state(
    shadow_validation_count: usize,
    target_window_materialized_count: usize,
    symbols: BTreeSet<String>,
    counts: &CandidateShadowCounts,
) -> ShadowCycleSampleState {
    ShadowCycleSampleState {
        shadow_validation_count,
        target_window_materialized_count,
        candidate_lifecycle_count: counts.candidate_lifecycle_count,
        partially_materialized_candidate_count: counts.partially_materialized_count,
        pending_target_window_candidate_count: counts.pending_target_window_candidate_count,
        total_sample_deficit: counts.total_sample_deficit,
        symbols: symbols.into_iter().collect(),
    }
}
