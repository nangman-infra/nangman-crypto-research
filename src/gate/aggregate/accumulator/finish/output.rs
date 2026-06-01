use crate::gate::aggregate::accumulator::state::AggregateAccumulator;
use crate::model::{ResearchBias, ResearchPartitionAggregate, SurvivalBand};

use super::metrics::FinishMetrics;

pub(super) fn build_partition_aggregate(
    accumulator: AggregateAccumulator,
    metrics: FinishMetrics,
    survival_band: SurvivalBand,
    gate_bias: ResearchBias,
    gate_reason_codes: Vec<String>,
) -> ResearchPartitionAggregate {
    ResearchPartitionAggregate {
        research_aggregate_key: accumulator.research_aggregate_key,
        research_partition_keys: accumulator.research_partition_keys.into_iter().collect(),
        source_candidate_ids: accumulator.source_candidate_ids.into_iter().collect(),
        source_candidate_lifecycle_keys: accumulator
            .source_candidate_lifecycle_keys
            .into_iter()
            .collect(),
        symbol_canonical: accumulator.symbol_canonical,
        hypothesis_type: accumulator.hypothesis_type,
        validation_adapter: accumulator.validation_adapter,
        strategy_id_or_family: accumulator.strategy_id_or_family,
        parameter_variant_id: accumulator.parameter_variant_id,
        replay_run_count: accumulator.replay_run_count,
        active_replay_run_count: accumulator.active_replay_run_count,
        expired_replay_run_count: accumulator.expired_replay_run_count,
        completed_count: accumulator.completed_count,
        decayed_completed_count: accumulator.decayed_completed_count,
        expired_completed_count: accumulator.expired_completed_count,
        effective_completed_sample_weight: accumulator.effective_completed_sample_weight,
        invalid_input_count: accumulator.invalid_input_count,
        missing_market_replay_data_count: accumulator.missing_market_replay_data_count,
        insufficient_evidence_count: accumulator.insufficient_evidence_count,
        liquidity_filter_materialized_count: accumulator.liquidity_filter_materialized_count,
        liquidity_filter_passed_count: accumulator.liquidity_filter_passed_count,
        liquidity_filter_failed_count: accumulator.liquidity_filter_failed_count,
        positive_net_count: accumulator.positive_net_count,
        non_positive_net_count: accumulator.non_positive_net_count,
        win_rate_ppm: metrics.win_rate_ppm,
        mean_raw_return_bps: metrics.mean_raw_return_bps,
        mean_btc_adjusted_return_bps: metrics.mean_btc_adjusted_return_bps,
        mean_net_after_cost_bps: metrics.mean_net_after_cost_bps,
        gross_positive_net_bps: accumulator.gross_positive_net_bps,
        gross_negative_net_bps_abs: accumulator.gross_negative_net_bps_abs,
        profit_factor_ppm: metrics.unweighted_profit_factor_ppm,
        weighted_win_rate_ppm: metrics.weighted_win_rate_ppm,
        weighted_mean_net_after_cost_bps: metrics.weighted_mean_net_after_cost_bps,
        weighted_profit_factor_ppm: metrics.weighted_profit_factor_ppm,
        estimated_cost_bps: metrics.estimated_cost_bps,
        cost_stressed_mean_net_after_cost_bps: metrics.cost_stressed_mean_net_after_cost_bps,
        distinct_replay_window_count: metrics.distinct_replay_window_count,
        inferred_unseen_window_count: metrics.inferred_unseen_window_count,
        market_regime_labels: accumulator.market_regime_labels.into_iter().collect(),
        regime_summaries: metrics.regime_summaries,
        train_validation_split_summary: metrics.train_validation_split_summary,
        survival_band,
        gate_bias,
        gate_reason_codes,
    }
}
