use crate::hash::stable_id;
use crate::model::{
    IntelCandidateEvidenceBundle, OssAdapterRun, ReplayRun, ReplayRunStatus, ShadowValidationRun,
};

pub(super) fn report_id(
    research_packet_id: &str,
    run_scope: &str,
    bundles: &[IntelCandidateEvidenceBundle],
    replay_runs: &[ReplayRun],
    oss_adapter_runs: &[OssAdapterRun],
    completed_shadow_validation_runs: &[ShadowValidationRun],
) -> String {
    let candidate_identity = candidate_identity_parts(bundles).join("|");
    let replay_identity = replay_identity_parts(replay_runs).join("|");
    let oss_identity = oss_identity_parts(oss_adapter_runs).join("|");
    let shadow_identity = shadow_identity_parts(completed_shadow_validation_runs).join("|");
    stable_id(
        "research_report",
        &[
            research_packet_id,
            run_scope,
            &bundles.len().to_string(),
            &candidate_identity,
            &replay_identity,
            &oss_identity,
            &shadow_identity,
        ],
    )
}

pub(super) fn candidate_identity_parts(bundles: &[IntelCandidateEvidenceBundle]) -> Vec<String> {
    let mut parts = bundles
        .iter()
        .map(|bundle| {
            format!(
                "{}:{}:{}:{}",
                bundle.candidate_id,
                bundle.candidate_lifecycle_key,
                bundle.bundle_key,
                bundle.idempotency_key
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts
}

pub(super) fn replay_identity_parts(replay_runs: &[ReplayRun]) -> Vec<String> {
    let mut parts = replay_runs
        .iter()
        .map(|run| run.replay_run_id.clone())
        .collect::<Vec<_>>();
    parts.sort();
    parts
}

pub(super) fn oss_identity_parts(oss_adapter_runs: &[OssAdapterRun]) -> Vec<String> {
    let mut parts = oss_adapter_runs
        .iter()
        .map(|run| {
            format!(
                "{}:{}:{:?}",
                run.oss_adapter_run_id, run.candidate_lifecycle_key, run.normalized_verdict_bias
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts
}

pub(super) fn shadow_identity_parts(shadow_validation_runs: &[ShadowValidationRun]) -> Vec<String> {
    let mut parts = shadow_validation_runs
        .iter()
        .map(|run| {
            format!(
                "{}:{}:{:?}:{}",
                run.shadow_validation_run_id, run.candidate_lifecycle_key, run.status, run.passed
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts
}

pub(super) fn invalid_input_candidate_keys(
    bundles: &[IntelCandidateEvidenceBundle],
    replay_runs: &[ReplayRun],
) -> Vec<String> {
    bundles
        .iter()
        .filter(|bundle| {
            let candidate_runs = replay_runs
                .iter()
                .filter(|run| run.source_candidate_id == bundle.candidate_id)
                .collect::<Vec<_>>();
            !candidate_runs.is_empty()
                && candidate_runs
                    .iter()
                    .all(|run| run.result_summary.status == ReplayRunStatus::InvalidInput)
        })
        .map(|bundle| bundle.candidate_lifecycle_key.clone())
        .collect()
}
