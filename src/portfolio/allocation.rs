mod artifacts;
mod candidate;
mod counts;
mod index;
mod state;

use super::snapshot::build_snapshot;
use crate::model::{
    IntelCandidateEvidenceBundle, PortfolioAllocationSnapshot, PortfolioReduceOnlySignal,
    PortfolioRiskRejectEvent, ResearchBias, ResearchRunReport,
};
use candidate::PortfolioCandidate;
use index::PortfolioInputIndex;
use state::AllocationState;

pub fn build_portfolio_artifacts(
    report: &ResearchRunReport,
    bundles: &[IntelCandidateEvidenceBundle],
    computed_at_ms: i64,
) -> (
    PortfolioAllocationSnapshot,
    Vec<PortfolioRiskRejectEvent>,
    Vec<PortfolioReduceOnlySignal>,
) {
    let index = PortfolioInputIndex::new(report, bundles);
    let mut state = AllocationState::default();

    for finding in report
        .summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::PromoteToShadowBias)
    {
        let Some(candidate) = PortfolioCandidate::from_finding(finding, &index) else {
            continue;
        };
        state.apply_candidate(candidate, computed_at_ms);
    }

    let artifacts = state.finish();

    let snapshot = build_snapshot(
        report,
        artifacts.allocations,
        &artifacts.rejects,
        &artifacts.reduce_only_signals,
        computed_at_ms,
    );
    (snapshot, artifacts.rejects, artifacts.reduce_only_signals)
}
