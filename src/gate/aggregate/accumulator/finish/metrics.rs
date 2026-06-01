use crate::gate::aggregate::accumulator::state::AggregateAccumulator;
use crate::gate::metrics::{mean, profit_factor_ppm, ratio_ppm, weighted_mean, weighted_ratio_ppm};
use crate::gate::sample::{
    cost_stressed_mean_net_after_cost_bps, regime_summaries, train_validation_split_summary,
};
use crate::model::{RegimeReplaySummary, ResearchGatePolicy, TrainValidationSplitSummary};

pub(super) struct FinishMetrics {
    pub(super) win_rate_ppm: Option<u64>,
    pub(super) mean_raw_return_bps: Option<f64>,
    pub(super) mean_btc_adjusted_return_bps: Option<f64>,
    pub(super) mean_net_after_cost_bps: Option<f64>,
    pub(super) unweighted_profit_factor_ppm: Option<u64>,
    pub(super) weighted_win_rate_ppm: Option<u64>,
    pub(super) weighted_mean_net_after_cost_bps: Option<f64>,
    pub(super) weighted_profit_factor_ppm: Option<u64>,
    pub(super) estimated_cost_bps: Option<f64>,
    pub(super) cost_stressed_mean_net_after_cost_bps: Option<f64>,
    pub(super) distinct_replay_window_count: usize,
    pub(super) inferred_unseen_window_count: usize,
    pub(super) unavailable_ratio_ppm: u64,
    pub(super) regime_summaries: Vec<RegimeReplaySummary>,
    pub(super) train_validation_split_summary: TrainValidationSplitSummary,
}

pub(super) fn compute_finish_metrics(
    accumulator: &AggregateAccumulator,
    policy: &ResearchGatePolicy,
) -> FinishMetrics {
    let distinct_replay_window_count = accumulator.active_replay_windows.len();
    let unavailable_count = accumulator.invalid_input_count
        + accumulator.missing_market_replay_data_count
        + accumulator.insufficient_evidence_count;

    FinishMetrics {
        win_rate_ppm: ratio_ppm(accumulator.positive_net_count, accumulator.completed_count),
        mean_raw_return_bps: mean(&accumulator.raw_returns),
        mean_btc_adjusted_return_bps: mean(&accumulator.btc_adjusted_returns),
        mean_net_after_cost_bps: mean(&accumulator.net_after_costs),
        unweighted_profit_factor_ppm: profit_factor_ppm(
            accumulator.gross_positive_net_bps,
            accumulator.gross_negative_net_bps_abs,
        ),
        weighted_win_rate_ppm: weighted_ratio_ppm(
            accumulator.weighted_positive_sample_weight,
            accumulator.effective_completed_sample_weight,
        ),
        weighted_mean_net_after_cost_bps: weighted_mean(
            accumulator.weighted_net_after_cost_sum,
            accumulator.effective_completed_sample_weight,
        ),
        weighted_profit_factor_ppm: profit_factor_ppm(
            accumulator.weighted_positive_net_bps,
            accumulator.weighted_negative_net_bps_abs,
        ),
        estimated_cost_bps: mean(&accumulator.cost_estimates),
        cost_stressed_mean_net_after_cost_bps: cost_stressed_mean_net_after_cost_bps(
            &accumulator.completed_samples,
            policy.cost_stress_multiplier_for_shadow,
        ),
        distinct_replay_window_count,
        inferred_unseen_window_count: distinct_replay_window_count.saturating_sub(1),
        unavailable_ratio_ppm: ratio_ppm(unavailable_count, accumulator.replay_run_count)
            .unwrap_or_default(),
        regime_summaries: regime_summaries(&accumulator.completed_samples),
        train_validation_split_summary: train_validation_split_summary(
            accumulator.train_validation_split_required,
            &accumulator.completed_samples,
        ),
    }
}
