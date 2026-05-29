use super::*;

pub(in crate::cli) fn append_unique_bundles(
    target: &mut Vec<IntelCandidateEvidenceBundle>,
    bundles: Vec<IntelCandidateEvidenceBundle>,
) {
    let mut existing_ids = target
        .iter()
        .map(|bundle| bundle.candidate_id.clone())
        .collect::<BTreeSet<_>>();
    for bundle in bundles {
        if existing_ids.insert(bundle.candidate_id.clone()) {
            target.push(bundle);
        }
    }
}

pub(in crate::cli) fn append_unique_replay_runs(target: &mut Vec<ReplayRun>, runs: Vec<ReplayRun>) {
    let mut existing_ids = target
        .iter()
        .map(|run| run.replay_run_id.clone())
        .collect::<BTreeSet<_>>();
    for run in runs {
        if existing_ids.insert(run.replay_run_id.clone()) {
            target.push(run);
        }
    }
}

pub(in crate::cli) fn filter_historical_replay_runs_for_current_research(
    runs: Vec<ReplayRun>,
    current_replay_runs: &[ReplayRun],
) -> Vec<ReplayRun> {
    let current_aggregate_keys = current_replay_runs
        .iter()
        .map(|run| run.research_aggregate_key.as_str())
        .collect::<BTreeSet<_>>();
    runs.into_iter()
        .filter(|run| current_aggregate_keys.contains(run.research_aggregate_key.as_str()))
        .collect()
}

pub(in crate::cli) fn append_unique_oss_adapter_runs(
    target: &mut Vec<OssAdapterRun>,
    runs: Vec<OssAdapterRun>,
) {
    let mut existing_ids = target
        .iter()
        .map(|run| run.oss_adapter_run_id.clone())
        .collect::<BTreeSet<_>>();
    for run in runs {
        if existing_ids.insert(run.oss_adapter_run_id.clone()) {
            target.push(run);
        }
    }
}

pub(in crate::cli) fn append_unique_shadow_validation_runs(
    target: &mut Vec<ShadowValidationRun>,
    runs: Vec<ShadowValidationRun>,
) {
    let mut existing_ids = target
        .iter()
        .map(|run| run.shadow_validation_run_id.clone())
        .collect::<BTreeSet<_>>();
    for run in runs {
        if existing_ids.insert(run.shadow_validation_run_id.clone()) {
            target.push(run);
        }
    }
}
