use crate::model::ResearchBias;

pub(super) fn no_completed_samples() -> (ResearchBias, Vec<String>) {
    (
        ResearchBias::RetestBias,
        vec!["no_completed_native_replay_samples".to_owned()],
    )
}

pub(super) fn non_positive_edge() -> (ResearchBias, Vec<String>) {
    (
        ResearchBias::PruneBias,
        vec!["aggregate_net_edge_non_positive".to_owned()],
    )
}

pub(super) fn finish_evaluation(blockers: Vec<String>) -> (ResearchBias, Vec<String>) {
    if blockers.len() == 1 {
        return (
            ResearchBias::PromoteToShadowBias,
            vec!["deterministic_shadow_gate_passed".to_owned()],
        );
    }

    (ResearchBias::RetestBias, blockers)
}
