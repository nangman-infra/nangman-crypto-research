use super::state::AggregateAccumulator;
use crate::gate::metrics::{mean, profit_factor_ppm, ratio_ppm, weighted_mean, weighted_ratio_ppm};
use crate::gate::sample::{
    cost_stressed_mean_net_after_cost_bps, regime_summaries, train_validation_split_summary,
};
use crate::model::{ResearchGatePolicy, ResearchPartitionAggregate};

use super::super::evaluation::{GateEvaluationInputs, evaluate_gate};
use super::super::survival::survival_band;

impl AggregateAccumulator {
    pub(in crate::gate::aggregate) fn finish(
        self,
        policy: &ResearchGatePolicy,
    ) -> ResearchPartitionAggregate {
        let win_rate_ppm = ratio_ppm(self.positive_net_count, self.completed_count);
        let mean_raw_return_bps = mean(&self.raw_returns);
        let mean_btc_adjusted_return_bps = mean(&self.btc_adjusted_returns);
        let mean_net_after_cost_bps = mean(&self.net_after_costs);
        let unweighted_profit_factor_ppm =
            profit_factor_ppm(self.gross_positive_net_bps, self.gross_negative_net_bps_abs);
        let weighted_win_rate_ppm = weighted_ratio_ppm(
            self.weighted_positive_sample_weight,
            self.effective_completed_sample_weight,
        );
        let weighted_mean_net_after_cost_bps = weighted_mean(
            self.weighted_net_after_cost_sum,
            self.effective_completed_sample_weight,
        );
        let weighted_profit_factor_ppm = profit_factor_ppm(
            self.weighted_positive_net_bps,
            self.weighted_negative_net_bps_abs,
        );
        let estimated_cost_bps = mean(&self.cost_estimates);
        let distinct_replay_window_count = self.active_replay_windows.len();
        let inferred_unseen_window_count = distinct_replay_window_count.saturating_sub(1);
        let cost_stressed_mean_net_after_cost_bps = cost_stressed_mean_net_after_cost_bps(
            &self.completed_samples,
            policy.cost_stress_multiplier_for_shadow,
        );
        let regime_summaries = regime_summaries(&self.completed_samples);
        let train_validation_split_summary = train_validation_split_summary(
            self.train_validation_split_required,
            &self.completed_samples,
        );
        let unavailable_count = self.invalid_input_count
            + self.missing_market_replay_data_count
            + self.insufficient_evidence_count;
        let unavailable_ratio_ppm =
            ratio_ppm(unavailable_count, self.replay_run_count).unwrap_or_default();
        let gate_inputs = GateEvaluationInputs {
            accumulator: &self,
            policy,
            win_rate_ppm: weighted_win_rate_ppm,
            mean_net_after_cost_bps: weighted_mean_net_after_cost_bps,
            profit_factor_ppm: weighted_profit_factor_ppm,
            unavailable_ratio_ppm,
            inferred_unseen_window_count,
            cost_stressed_mean_net_after_cost_bps,
            regime_summaries: &regime_summaries,
            train_validation_split_summary: &train_validation_split_summary,
        };
        let (gate_bias, gate_reason_codes) = evaluate_gate(&gate_inputs);
        let survival_band = survival_band(
            &gate_bias,
            self.completed_count,
            mean_net_after_cost_bps,
            unweighted_profit_factor_ppm,
        );

        ResearchPartitionAggregate {
            research_aggregate_key: self.research_aggregate_key,
            research_partition_keys: self.research_partition_keys.into_iter().collect(),
            source_candidate_ids: self.source_candidate_ids.into_iter().collect(),
            source_candidate_lifecycle_keys: self
                .source_candidate_lifecycle_keys
                .into_iter()
                .collect(),
            symbol_canonical: self.symbol_canonical,
            hypothesis_type: self.hypothesis_type,
            validation_adapter: self.validation_adapter,
            strategy_id_or_family: self.strategy_id_or_family,
            parameter_variant_id: self.parameter_variant_id,
            replay_run_count: self.replay_run_count,
            active_replay_run_count: self.active_replay_run_count,
            expired_replay_run_count: self.expired_replay_run_count,
            completed_count: self.completed_count,
            decayed_completed_count: self.decayed_completed_count,
            expired_completed_count: self.expired_completed_count,
            effective_completed_sample_weight: self.effective_completed_sample_weight,
            invalid_input_count: self.invalid_input_count,
            missing_market_replay_data_count: self.missing_market_replay_data_count,
            insufficient_evidence_count: self.insufficient_evidence_count,
            liquidity_filter_materialized_count: self.liquidity_filter_materialized_count,
            liquidity_filter_passed_count: self.liquidity_filter_passed_count,
            liquidity_filter_failed_count: self.liquidity_filter_failed_count,
            positive_net_count: self.positive_net_count,
            non_positive_net_count: self.non_positive_net_count,
            win_rate_ppm,
            mean_raw_return_bps,
            mean_btc_adjusted_return_bps,
            mean_net_after_cost_bps,
            gross_positive_net_bps: self.gross_positive_net_bps,
            gross_negative_net_bps_abs: self.gross_negative_net_bps_abs,
            profit_factor_ppm: unweighted_profit_factor_ppm,
            weighted_win_rate_ppm,
            weighted_mean_net_after_cost_bps,
            weighted_profit_factor_ppm,
            estimated_cost_bps,
            cost_stressed_mean_net_after_cost_bps,
            distinct_replay_window_count,
            inferred_unseen_window_count,
            market_regime_labels: self.market_regime_labels.into_iter().collect(),
            regime_summaries,
            train_validation_split_summary,
            survival_band,
            gate_bias,
            gate_reason_codes,
        }
    }
}
