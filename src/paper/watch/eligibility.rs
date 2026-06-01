use crate::model::{ResearchBias, ResearchPartitionAggregate};

pub(super) fn paper_watch_eligible_aggregate(aggregate: &ResearchPartitionAggregate) -> bool {
    let missing_market_data_ratio_ppm = ratio_ppm(
        aggregate.missing_market_replay_data_count,
        aggregate.replay_run_count,
    );
    aggregate.gate_bias == ResearchBias::RetestBias
        && aggregate.positive_net_count > 0
        && aggregate.completed_count > 0
        && aggregate.non_positive_net_count == 0
        && missing_market_data_ratio_ppm <= 500_000
        && aggregate
            .weighted_mean_net_after_cost_bps
            .or(aggregate.mean_net_after_cost_bps)
            .is_some_and(|value| value > 0.0)
        && !aggregate
            .gate_reason_codes
            .iter()
            .any(|reason| reason == "aggregate_net_edge_non_positive")
        && !aggregate
            .gate_reason_codes
            .iter()
            .any(|reason| reason == "native_replay_net_edge_non_positive")
}

fn ratio_ppm(numerator: usize, denominator: usize) -> u64 {
    if denominator == 0 {
        return 1_000_000;
    }
    ((numerator as u128 * 1_000_000) / denominator as u128) as u64
}
