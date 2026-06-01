#[cfg(test)]
mod tests;

use crate::model::{ResearchBias, ResearchRunStatus, SummaryFinding};

pub(super) fn pruned_candidate_keys(summary_findings: &[SummaryFinding]) -> Vec<String> {
    candidate_keys_with_bias(summary_findings, ResearchBias::PruneBias)
}

pub(super) fn retest_candidate_keys(summary_findings: &[SummaryFinding]) -> Vec<String> {
    candidate_keys_with_bias(summary_findings, ResearchBias::RetestBias)
}

pub(super) fn surviving_candidate_keys(summary_findings: &[SummaryFinding]) -> Vec<String> {
    summary_findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.bias,
                ResearchBias::PromoteToShadowBias | ResearchBias::PromoteToPaperBias
            )
        })
        .map(|finding| finding.candidate_lifecycle_key.clone())
        .collect()
}

pub(super) fn research_run_status(
    invalid_input_candidate_count: usize,
    candidate_count: usize,
) -> ResearchRunStatus {
    if invalid_input_candidate_count == 0 {
        return ResearchRunStatus::Completed;
    }
    if invalid_input_candidate_count == candidate_count {
        ResearchRunStatus::InvalidInput
    } else {
        ResearchRunStatus::Partial
    }
}

fn candidate_keys_with_bias(
    summary_findings: &[SummaryFinding],
    bias: ResearchBias,
) -> Vec<String> {
    summary_findings
        .iter()
        .filter(|finding| finding.bias == bias)
        .map(|finding| finding.candidate_lifecycle_key.clone())
        .collect()
}
