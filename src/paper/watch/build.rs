use super::aggregate::best_paper_watch_aggregate;
use super::candidate::paper_watch_candidate;
use crate::model::{
    IntelCandidateEvidenceBundle, PaperWatchCandidate, ResearchBias, ResearchRunReport,
};
use crate::paper::artifacts::default_paper_account_profile;
use crate::paper::shared::{aggregates_by_candidate_key, has_major_failure_event};
use std::collections::BTreeMap;

pub fn build_paper_watch_candidates(
    report: &ResearchRunReport,
    bundles: &[IntelCandidateEvidenceBundle],
    created_at_ms: i64,
) -> Vec<PaperWatchCandidate> {
    let profile = default_paper_account_profile();
    let bundle_by_key = bundles
        .iter()
        .map(|bundle| (bundle.candidate_lifecycle_key.as_str(), bundle))
        .collect::<BTreeMap<_, _>>();
    let aggregates_by_candidate_key = aggregates_by_candidate_key(&report.partition_aggregates);
    let mut candidates = Vec::new();

    for finding in report
        .summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::RetestBias)
    {
        let Some(bundle) = bundle_by_key.get(finding.candidate_lifecycle_key.as_str()) else {
            continue;
        };
        if !bundle.approved_universe_symbol || has_major_failure_event(bundle) {
            continue;
        }
        let Some(aggregate) = best_paper_watch_aggregate(
            aggregates_by_candidate_key
                .get(finding.candidate_lifecycle_key.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]),
        ) else {
            continue;
        };

        candidates.push(paper_watch_candidate(
            report,
            finding,
            bundle,
            aggregate,
            &profile,
            created_at_ms,
        ));
    }

    candidates
}
