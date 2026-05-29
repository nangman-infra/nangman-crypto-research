use std::collections::{BTreeMap, BTreeSet};

use crate::model::{ShadowValidationRun, ShadowValidationStatus};

use super::time::{min_optional_ms, target_exit_deadline_ms};

#[derive(Debug, Clone)]
pub(super) struct CandidateShadowState {
    pub(super) symbol_set: BTreeSet<String>,
    pub(super) observed_count: usize,
    pub(super) target_materialized_count: usize,
    pub(super) pending_target_count: usize,
    pub(super) pending_count: usize,
    pub(super) required_count: usize,
    pub(super) next_pending_target_deadline_ms: Option<i64>,
}

impl CandidateShadowState {
    fn new() -> Self {
        Self {
            symbol_set: BTreeSet::new(),
            observed_count: 0,
            target_materialized_count: 0,
            pending_target_count: 0,
            pending_count: 0,
            required_count: 0,
            next_pending_target_deadline_ms: None,
        }
    }

    pub(super) fn sample_deficit(&self) -> i64 {
        self.required_count
            .saturating_sub(self.target_materialized_count) as i64
    }

    pub(super) fn sample_requirement_met(&self) -> bool {
        self.required_count > 0 && self.target_materialized_count >= self.required_count
    }
}

#[derive(Debug)]
pub(super) struct ShadowCycleBuildSummary {
    pub(super) candidates: BTreeMap<String, CandidateShadowState>,
    pub(super) symbols: BTreeSet<String>,
    pub(super) target_materialized_count: usize,
    pub(super) run_identity_parts: Vec<String>,
}

pub(super) fn summarize_shadow_runs(
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
) -> ShadowCycleBuildSummary {
    let mut candidates = BTreeMap::<String, CandidateShadowState>::new();
    let mut symbols = BTreeSet::new();
    let mut target_materialized_count = 0usize;
    let mut run_identity_parts = Vec::new();

    for run in shadow_runs {
        run_identity_parts.push(run.shadow_validation_run_id.clone());
        symbols.insert(run.symbol_canonical.clone());
        let target_deadline_ms = target_exit_deadline_ms(run);
        let target_materialized = latest_l1_as_of_ms
            .zip(target_deadline_ms)
            .is_some_and(|(latest, target)| latest >= target);
        if target_materialized {
            target_materialized_count += 1;
        }

        let state = candidates
            .entry(run.candidate_lifecycle_key.clone())
            .or_insert_with(CandidateShadowState::new);
        state.symbol_set.insert(run.symbol_canonical.clone());
        state.observed_count += 1;
        state.required_count = state
            .required_count
            .max(run.watch_window_policy.min_shadow_samples);
        if run.status == ShadowValidationStatus::Pending {
            state.pending_count += 1;
        }
        if target_materialized {
            state.target_materialized_count += 1;
        } else if let Some(deadline) = target_deadline_ms {
            state.pending_target_count += 1;
            state.next_pending_target_deadline_ms =
                min_optional_ms(state.next_pending_target_deadline_ms, Some(deadline));
        }
    }

    ShadowCycleBuildSummary {
        candidates,
        symbols,
        target_materialized_count,
        run_identity_parts,
    }
}
