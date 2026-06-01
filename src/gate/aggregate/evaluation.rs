mod blockers;
mod result;

use crate::model::{RegimeReplaySummary, ResearchGatePolicy, TrainValidationSplitSummary};

use super::accumulator::AggregateAccumulator;

pub(in crate::gate::aggregate) struct GateEvaluationInputs<'a> {
    pub(in crate::gate::aggregate) accumulator: &'a AggregateAccumulator,
    pub(in crate::gate::aggregate) policy: &'a ResearchGatePolicy,
    pub(in crate::gate::aggregate) win_rate_ppm: Option<u64>,
    pub(in crate::gate::aggregate) mean_net_after_cost_bps: Option<f64>,
    pub(in crate::gate::aggregate) profit_factor_ppm: Option<u64>,
    pub(in crate::gate::aggregate) unavailable_ratio_ppm: u64,
    pub(in crate::gate::aggregate) inferred_unseen_window_count: usize,
    pub(in crate::gate::aggregate) cost_stressed_mean_net_after_cost_bps: Option<f64>,
    pub(in crate::gate::aggregate) regime_summaries: &'a [RegimeReplaySummary],
    pub(in crate::gate::aggregate) train_validation_split_summary: &'a TrainValidationSplitSummary,
}

pub(in crate::gate::aggregate) fn evaluate_gate(
    inputs: &GateEvaluationInputs<'_>,
) -> (crate::model::ResearchBias, Vec<String>) {
    let Some(mean_net_after_cost_bps) = inputs.mean_net_after_cost_bps else {
        return result::no_completed_samples();
    };

    if mean_net_after_cost_bps <= 0.0 {
        return result::non_positive_edge();
    }

    let blockers = blockers::promotion_blockers(inputs, mean_net_after_cost_bps);
    result::finish_evaluation(blockers)
}
