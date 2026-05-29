use crate::model::{PaperWatchCandidate, ResearchBias, ResearchRunReport};
use std::collections::BTreeMap;

pub(super) struct ResearchAlertMetrics {
    pub(super) promote_paper_count: usize,
    pub(super) promote_shadow_count: usize,
    pub(super) retest_count: usize,
    pub(super) prune_count: usize,
    pub(super) paper_count: usize,
    pub(super) paper_watch_count: usize,
    pub(super) shadow_count: usize,
    pub(super) max_total_notional_pct: f64,
}

impl ResearchAlertMetrics {
    pub(super) fn from_report(
        report: &ResearchRunReport,
        paper_watch_candidates: &[PaperWatchCandidate],
    ) -> Self {
        let counts = bias_counts(report);
        Self {
            promote_paper_count: *counts
                .get(ResearchBias::PromoteToPaperBias.report_key())
                .unwrap_or(&0),
            promote_shadow_count: *counts
                .get(ResearchBias::PromoteToShadowBias.report_key())
                .unwrap_or(&0),
            retest_count: *counts
                .get(ResearchBias::RetestBias.report_key())
                .unwrap_or(&0),
            prune_count: *counts
                .get(ResearchBias::PruneBias.report_key())
                .unwrap_or(&0),
            paper_count: report.paper_trade_candidates.len(),
            paper_watch_count: report
                .paper_watch_candidates
                .len()
                .max(paper_watch_candidates.len()),
            shadow_count: report.shadow_validation_runs.len(),
            max_total_notional_pct: report
                .portfolio_allocation_snapshot
                .as_ref()
                .map(|snapshot| snapshot.max_total_notional_pct)
                .unwrap_or_default(),
        }
    }
}

fn bias_counts(report: &ResearchRunReport) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for finding in &report.summary_findings {
        *counts.entry(finding.bias.report_key()).or_insert(0) += 1;
    }
    counts
}
