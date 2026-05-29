use std::collections::BTreeSet;

use crate::model::{ResearchBias, ResearchRunReport, ShadowValidationRun};

pub(super) fn paper_candidate_keys(report: &ResearchRunReport) -> BTreeSet<String> {
    report
        .summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::PromoteToPaperBias)
        .map(|finding| finding.candidate_lifecycle_key.clone())
        .collect()
}

pub(super) fn shadow_holding_policy_supported(shadow_run: &ShadowValidationRun) -> bool {
    shadow_run.holding_policy.target_max_holding_hours <= 24
        && shadow_run.holding_policy.absolute_max_holding_hours <= 72
}
