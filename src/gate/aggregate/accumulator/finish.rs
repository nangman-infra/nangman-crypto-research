mod metrics;
mod output;

use super::state::AggregateAccumulator;
use crate::model::{ResearchGatePolicy, ResearchPartitionAggregate};

use super::super::evaluation::{GateEvaluationInputs, evaluate_gate};
use super::super::survival::survival_band;

impl AggregateAccumulator {
    pub(in crate::gate::aggregate) fn finish(
        self,
        policy: &ResearchGatePolicy,
    ) -> ResearchPartitionAggregate {
        let metrics = metrics::compute_finish_metrics(&self, policy);
        let gate_inputs = GateEvaluationInputs {
            accumulator: &self,
            policy,
            win_rate_ppm: metrics.weighted_win_rate_ppm,
            mean_net_after_cost_bps: metrics.weighted_mean_net_after_cost_bps,
            profit_factor_ppm: metrics.weighted_profit_factor_ppm,
            unavailable_ratio_ppm: metrics.unavailable_ratio_ppm,
            inferred_unseen_window_count: metrics.inferred_unseen_window_count,
            cost_stressed_mean_net_after_cost_bps: metrics.cost_stressed_mean_net_after_cost_bps,
            regime_summaries: &metrics.regime_summaries,
            train_validation_split_summary: &metrics.train_validation_split_summary,
        };
        let (gate_bias, gate_reason_codes) = evaluate_gate(&gate_inputs);
        let survival_band = survival_band(
            &gate_bias,
            self.completed_count,
            metrics.mean_net_after_cost_bps,
            metrics.unweighted_profit_factor_ppm,
        );

        output::build_partition_aggregate(
            self,
            metrics,
            survival_band,
            gate_bias,
            gate_reason_codes,
        )
    }
}
