use std::collections::BTreeMap;

use super::assembly::{CandidatePaperBuildInput, build_candidate_paper_artifacts};
use super::eligibility::{paper_candidate_keys, shadow_holding_policy_supported};
use super::profile::default_paper_account_profile;
use super::types::PaperArtifacts;
use crate::model::{IntelCandidateEvidenceBundle, ResearchRunReport, ShadowValidationRun};
use crate::paper::shared::{
    aggregate_by_candidate_key, has_major_failure_event, passed_shadow_by_candidate_key,
};

pub fn build_paper_artifacts(
    report: &ResearchRunReport,
    bundles: &[IntelCandidateEvidenceBundle],
    completed_shadow_validation_runs: &[ShadowValidationRun],
    created_at_ms: i64,
) -> PaperArtifacts {
    let profile = default_paper_account_profile();
    let bundle_by_key = bundles
        .iter()
        .map(|bundle| (bundle.candidate_lifecycle_key.as_str(), bundle))
        .collect::<BTreeMap<_, _>>();
    let aggregate_by_candidate_key = aggregate_by_candidate_key(&report.partition_aggregates);
    let passed_shadow_by_candidate_key =
        passed_shadow_by_candidate_key(completed_shadow_validation_runs);

    let mut artifacts = PaperArtifacts::default();
    for candidate_lifecycle_key in paper_candidate_keys(report) {
        let Some(bundle) = bundle_by_key.get(candidate_lifecycle_key.as_str()) else {
            continue;
        };
        if has_major_failure_event(bundle) {
            continue;
        }
        let Some(aggregate) = aggregate_by_candidate_key.get(candidate_lifecycle_key.as_str())
        else {
            continue;
        };
        let Some(shadow_run) = passed_shadow_by_candidate_key.get(candidate_lifecycle_key.as_str())
        else {
            continue;
        };
        if !shadow_holding_policy_supported(shadow_run) {
            continue;
        }

        let rows = build_candidate_paper_artifacts(CandidatePaperBuildInput {
            report,
            candidate_lifecycle_key: &candidate_lifecycle_key,
            aggregate,
            shadow_run,
            profile: &profile,
            created_at_ms,
        });
        artifacts.candidates.push(rows.candidate);
        artifacts.runs.push(rows.run);
        artifacts.summaries.push(rows.summary);
        artifacts.marks.push(rows.mark);
    }

    artifacts
}
