use super::*;

#[test]
fn classifies_candidate_keys_by_bias() {
    let findings = vec![
        finding("candidate-a", ResearchBias::PruneBias),
        finding("candidate-b", ResearchBias::RetestBias),
        finding("candidate-c", ResearchBias::PromoteToShadowBias),
    ];

    assert_eq!(pruned_candidate_keys(&findings), vec!["candidate-a"]);
    assert_eq!(retest_candidate_keys(&findings), vec!["candidate-b"]);
}

#[test]
fn surviving_candidate_keys_include_shadow_and_paper_biases() {
    let findings = vec![
        finding("candidate-a", ResearchBias::PruneBias),
        finding("candidate-b", ResearchBias::PromoteToShadowBias),
        finding("candidate-c", ResearchBias::PromoteToPaperBias),
    ];

    assert_eq!(
        surviving_candidate_keys(&findings),
        vec!["candidate-b", "candidate-c"]
    );
}

#[test]
fn research_status_is_invalid_when_all_candidates_have_invalid_inputs() {
    assert_eq!(research_run_status(2, 2), ResearchRunStatus::InvalidInput);
}

#[test]
fn research_status_is_partial_when_only_some_candidates_have_invalid_inputs() {
    assert_eq!(research_run_status(1, 2), ResearchRunStatus::Partial);
}

#[test]
fn research_status_is_completed_without_invalid_inputs() {
    assert_eq!(research_run_status(0, 2), ResearchRunStatus::Completed);
}

fn finding(candidate_lifecycle_key: &str, bias: ResearchBias) -> SummaryFinding {
    SummaryFinding {
        candidate_id: candidate_lifecycle_key.to_owned(),
        candidate_lifecycle_key: candidate_lifecycle_key.to_owned(),
        bias,
        reason_codes: Vec::new(),
    }
}
